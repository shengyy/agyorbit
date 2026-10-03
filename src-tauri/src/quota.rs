//! Keeping each account's quota and plan current.

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::task::JoinSet;

use crate::error::Error;
use crate::google::cloudcode;
use crate::model::QuotaState;
use crate::state::Orbit;

/// Refreshes every account concurrently. A refresh already in flight wins.
pub async fn refresh_all(orbit: &Arc<Orbit>) {
    let ids = {
        let mut state = orbit.state();
        if state.refreshing {
            return;
        }
        state.refreshing = true;
        state
            .registry
            .accounts()
            .iter()
            .map(|a| a.id.clone())
            .collect::<Vec<_>>()
    };
    orbit.emit();
    let mut tasks = JoinSet::new();
    for id in ids {
        let orbit = orbit.clone();
        tasks.spawn(async move { refresh_one(&orbit, &id).await });
    }
    while tasks.join_next().await.is_some() {}
    orbit.update(|state| {
        state.refreshing = false;
        state.refreshed_at = Some(Utc::now());
    });
}

/// Refreshes when the last full refresh is older than `max_age`.
pub async fn refresh_if_stale(orbit: &Arc<Orbit>, max_age: Duration) {
    let stale = orbit
        .state()
        .refreshed_at
        .is_none_or(|at| (Utc::now() - at).to_std().unwrap_or_default() > max_age);
    if stale {
        refresh_all(orbit).await;
    }
}

pub async fn refresh_one(orbit: &Arc<Orbit>, account_id: &str) {
    let result = async {
        let token = orbit.access_token(account_id).await?;
        let (groups, plan) = tokio::join!(
            cloudcode::quota(&orbit.http, &token),
            cloudcode::plan(&orbit.http, &token)
        );
        Ok::<_, Error>((groups?, plan))
    }
    .await;
    orbit.update(|state| {
        let quota = match result {
            Ok((groups, plan)) => {
                match plan {
                    Ok(plan) => {
                        if let Err(err) = state.registry.set_plan(account_id, plan) {
                            log::warn!("could not save plan: {err}");
                        }
                    }
                    Err(err) => log::warn!("plan lookup failed: {err}"),
                }
                QuotaState::Ready {
                    groups,
                    fetched_at: Utc::now(),
                }
            }
            Err(Error::TokenRevoked) => QuotaState::Reauth,
            Err(err) => {
                log::warn!("quota refresh failed: {err}");
                QuotaState::Failed {
                    code: err.code().into(),
                    message: err.to_string(),
                }
            }
        };
        // Keep the account's last good numbers visible through a transient error.
        let keep_previous = matches!(quota, QuotaState::Failed { .. })
            && matches!(state.quotas.get(account_id), Some(QuotaState::Ready { .. }));
        if !keep_previous && state.registry.get(account_id).is_some() {
            state.quotas.insert(account_id.to_owned(), quota);
        }
    });
}
