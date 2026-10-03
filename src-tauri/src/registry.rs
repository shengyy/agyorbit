//! Account metadata, persisted as `accounts.json` in the app data directory.
//! Holds no secrets: refresh tokens live in the vault.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::Result;
use crate::google::cloudcode::Plan;

const FILE: &str = "accounts.json";
const VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    /// Google's stable account id (`sub`).
    pub id: String,
    pub email: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub picture: Option<String>,
    #[serde(default)]
    pub plan: Option<Plan>,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct File {
    version: u32,
    accounts: Vec<Account>,
    /// Fingerprint of the IDE refresh token last seen, so a new sign-in made
    /// inside Antigravity is adopted once and a removed account stays removed.
    #[serde(default)]
    live_fingerprint: Option<String>,
}

pub struct Registry {
    path: PathBuf,
    file: File,
}

/// Short, non-reversible fingerprint of a refresh token.
pub fn fingerprint(refresh_token: &str) -> String {
    Sha256::digest(refresh_token.as_bytes())[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

impl Registry {
    pub fn load(dir: &Path) -> Result<Registry> {
        let path = dir.join(FILE);
        let file = match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)?,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => File {
                version: VERSION,
                ..File::default()
            },
            Err(err) => return Err(err.into()),
        };
        Ok(Registry { path, file })
    }

    pub fn accounts(&self) -> &[Account] {
        &self.file.accounts
    }

    pub fn get(&self, id: &str) -> Option<&Account> {
        self.file.accounts.iter().find(|a| a.id == id)
    }

    /// Inserts a new account or refreshes an existing one in place, keeping
    /// its position, `added_at` and known plan.
    pub fn upsert(&mut self, account: Account) -> Result<()> {
        match self.file.accounts.iter_mut().find(|a| a.id == account.id) {
            Some(existing) => {
                existing.email = account.email;
                existing.name = account.name.or(existing.name.take());
                existing.picture = account.picture.or(existing.picture.take());
                existing.plan = account.plan.or(existing.plan.take());
            }
            None => self.file.accounts.push(account),
        }
        self.save()
    }

    pub fn set_plan(&mut self, id: &str, plan: Plan) -> Result<()> {
        match self.file.accounts.iter_mut().find(|a| a.id == id) {
            Some(account) if account.plan.as_ref() != Some(&plan) => {
                account.plan = Some(plan);
                self.save()
            }
            _ => Ok(()),
        }
    }

    pub fn remove(&mut self, id: &str) -> Result<()> {
        self.file.accounts.retain(|a| a.id != id);
        self.save()
    }

    pub fn live_fingerprint(&self) -> Option<&str> {
        self.file.live_fingerprint.as_deref()
    }

    pub fn set_live_fingerprint(&mut self, fingerprint: String) -> Result<()> {
        if self.file.live_fingerprint.as_deref() == Some(fingerprint.as_str()) {
            return Ok(());
        }
        self.file.live_fingerprint = Some(fingerprint);
        self.save()
    }

    fn save(&self) -> Result<()> {
        let dir = self.path.parent().expect("registry path has a parent");
        fs::create_dir_all(dir)?;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(&self.file)?)?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(id: &str, name: Option<&str>) -> Account {
        Account {
            id: id.into(),
            email: format!("{id}@example.com"),
            name: name.map(Into::into),
            picture: None,
            plan: None,
            added_at: Utc::now(),
        }
    }

    #[test]
    fn upsert_keeps_order_and_known_fields() {
        let dir = std::env::temp_dir().join(format!("agyorbit-registry-{}", std::process::id()));
        let mut registry = Registry::load(&dir).unwrap();
        registry.upsert(account("a", Some("Ada"))).unwrap();
        registry.upsert(account("b", None)).unwrap();
        registry.upsert(account("a", None)).unwrap();
        let reloaded = Registry::load(&dir).unwrap();
        let ids: Vec<_> = reloaded.accounts().iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids, ["a", "b"]);
        assert_eq!(reloaded.get("a").unwrap().name.as_deref(), Some("Ada"));
        fs::remove_dir_all(dir).unwrap();
    }
}
