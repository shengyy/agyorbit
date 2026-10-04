//! Shared application state and the snapshot pipeline to the frontend.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use chrono::{DateTime, Utc};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{OnceCell, OwnedMutexGuard, oneshot};

use crate::antigravity::install::Install;
use crate::antigravity::oauth_client::{self, OAuthClient};
use crate::antigravity::process;
use crate::error::{Error, Result};
use crate::google::{USER_AGENT, oauth};
use crate::model::{
    AccountView, AntigravityStatus, Operation, QuotaState, SNAPSHOT_EVENT, Snapshot,
};
use crate::registry::Registry;
use crate::tokens::TokenCache;
use crate::updater::UpdateState;
use crate::vault;

pub struct Orbit {
    pub app: AppHandle,
    pub http: reqwest::Client,
    pub tokens: TokenCache,
    inner: Mutex<Inner>,
    client: OnceCell<OAuthClient>,
    op_lock: Arc<tokio::sync::Mutex<()>>,
    pending_auth: Mutex<Option<oneshot::Sender<()>>>,
}

pub struct Inner {
    pub registry: Registry,
    pub quotas: HashMap<String, QuotaState>,
    pub active_id: Option<String>,
    pub operation: Operation,
    pub refreshing: bool,
    pub refreshed_at: Option<DateTime<Utc>>,
    pub install: Option<Install>,
    pub running: bool,
    pub update: UpdateState,
}

/// Holds the single operation slot; dropping it returns AgyOrbit to idle.
pub struct OpGuard {
    orbit: Arc<Orbit>,
    _lock: OwnedMutexGuard<()>,
}

impl Drop for OpGuard {
    fn drop(&mut self) {
        self.orbit.update(|state| state.operation = Operation::Idle);
    }
}

impl Orbit {
    pub fn new(app: AppHandle) -> Result<Orbit> {
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|err| Error::Invalid(err.to_string()))?;
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()?;
        let install = Install::locate();
        let running = process::is_app_running(install.as_ref());
        Ok(Orbit {
            app,
            http,
            tokens: TokenCache::default(),
            inner: Mutex::new(Inner {
                registry: Registry::load(&data_dir)?,
                quotas: HashMap::new(),
                active_id: None,
                operation: Operation::Idle,
                refreshing: false,
                refreshed_at: None,
                install,
                running,
                update: UpdateState::default(),
            }),
            client: OnceCell::new(),
            op_lock: Arc::new(tokio::sync::Mutex::new(())),
            pending_auth: Mutex::new(None),
        })
    }

    pub fn state(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().expect("state poisoned")
    }

    /// Mutates state and pushes a fresh snapshot to the frontend.
    pub fn update<R>(&self, f: impl FnOnce(&mut Inner) -> R) -> R {
        let result = f(&mut self.state());
        self.emit();
        result
    }

    pub fn snapshot(&self) -> Snapshot {
        let state = self.state();
        Snapshot {
            platform: std::env::consts::OS,
            version: env!("CARGO_PKG_VERSION"),
            accounts: state
                .registry
                .accounts()
                .iter()
                .map(|account| AccountView {
                    id: account.id.clone(),
                    email: account.email.clone(),
                    name: account.name.clone(),
                    picture: account.picture.clone(),
                    plan: account.plan.clone(),
                    quota: state
                        .quotas
                        .get(&account.id)
                        .cloned()
                        .unwrap_or(QuotaState::Loading),
                })
                .collect(),
            active_id: state.active_id.clone(),
            operation: state.operation.clone(),
            antigravity: AntigravityStatus {
                installed: state.install.is_some(),
                running: state.running,
            },
            refreshing: state.refreshing,
            refreshed_at: state.refreshed_at,
            update: state.update.snapshot(),
        }
    }

    pub fn emit(&self) {
        if let Err(err) = self.app.emit(SNAPSHOT_EVENT, self.snapshot()) {
            log::warn!("failed to emit snapshot: {err}");
        }
    }

    /// Re-checks whether Antigravity is installed and running.
    pub fn refresh_status(&self) {
        let install = self
            .state()
            .install
            .clone()
            .filter(|i| i.main_exe.is_file())
            .or_else(Install::locate);
        let running = process::is_app_running(install.as_ref());
        self.update(|state| {
            state.install = install;
            state.running = running;
        });
    }

    pub fn install(&self) -> Result<Install> {
        if let Some(install) = self
            .state()
            .install
            .clone()
            .filter(|i| i.main_exe.is_file())
        {
            return Ok(install);
        }
        let found = Install::locate();
        self.state().install = found.clone();
        found.ok_or(Error::AntigravityNotFound)
    }

    pub async fn oauth_client(&self) -> Result<OAuthClient> {
        self.client
            .get_or_try_init(|| async {
                oauth_client::discover(&self.http, &self.install()?).await
            })
            .await
            .cloned()
    }

    /// A valid access token for the account, refreshed from the vault when
    /// the cached one is about to expire.
    pub async fn access_token(&self, account_id: &str) -> Result<String> {
        if let Some(token) = self.tokens.get(account_id) {
            return Ok(token);
        }
        let refresh_token = vault::get(account_id)?.ok_or(Error::AccountNotFound)?;
        let client = self.oauth_client().await?;
        let grant = oauth::refresh(&self.http, &client, &refresh_token).await?;
        self.tokens
            .put(account_id, grant.access_token.clone(), grant.expires_in);
        Ok(grant.access_token)
    }

    /// Claims the single operation slot, or fails with `Busy`.
    pub fn begin(self: &Arc<Self>, operation: Operation) -> Result<OpGuard> {
        let lock = self
            .op_lock
            .clone()
            .try_lock_owned()
            .map_err(|_| Error::Busy)?;
        self.update(|state| state.operation = operation);
        Ok(OpGuard {
            orbit: self.clone(),
            _lock: lock,
        })
    }

    pub fn set_operation(&self, operation: Operation) {
        self.update(|state| state.operation = operation);
    }

    pub fn is_idle(&self) -> bool {
        self.state().operation == Operation::Idle
    }

    pub fn set_pending_auth(&self, cancel: oneshot::Sender<()>) {
        *self.pending_auth.lock().expect("pending auth poisoned") = Some(cancel);
    }

    pub fn cancel_pending_auth(&self) {
        if let Some(cancel) = self
            .pending_auth
            .lock()
            .expect("pending auth poisoned")
            .take()
        {
            let _ = cancel.send(());
        }
    }
}
