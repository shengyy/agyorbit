//! View models sent to the frontend. The frontend renders `Snapshot` and
//! never sees tokens.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::google::cloudcode::{Plan, QuotaGroup};

pub const SNAPSHOT_EVENT: &str = "agyorbit://snapshot";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub platform: &'static str,
    pub version: &'static str,
    pub accounts: Vec<AccountView>,
    /// The account Antigravity is signed in with, when AgyOrbit knows it.
    pub active_id: Option<String>,
    pub operation: Operation,
    pub antigravity: AntigravityStatus,
    pub refreshing: bool,
    pub refreshed_at: Option<DateTime<Utc>>,
    pub update: UpdateStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub checking: bool,
    pub available: Option<UpdateInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountView {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub plan: Option<Plan>,
    pub quota: QuotaState,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum QuotaState {
    Loading,
    #[serde(rename_all = "camelCase")]
    Ready {
        groups: Vec<QuotaGroup>,
        fetched_at: DateTime<Utc>,
    },
    Failed {
        code: String,
        message: String,
    },
    /// Google revoked the refresh token; the account must sign in again.
    Reauth,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AntigravityStatus {
    pub installed: bool,
    pub running: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Operation {
    Idle,
    Adding,
    #[serde(rename_all = "camelCase")]
    Switching {
        target_id: String,
        step: SwitchStep,
    },
    Stopping,
    Restarting,
    Updating {
        step: UpdateStep,
        downloaded: u64,
        total: Option<u64>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateStep {
    Downloading,
    Verifying,
    Installing,
    Restarting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SwitchStep {
    Preparing,
    Closing,
    Writing,
    Launching,
}
