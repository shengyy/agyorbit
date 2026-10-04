//! Signed app updates. Account operations share the same installation guard.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tauri_plugin_updater::{Update, UpdaterExt};

use crate::error::{Error, Result};
use crate::model::{Operation, UpdateInfo, UpdateStatus, UpdateStep};
use crate::state::Orbit;

const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Default)]
pub struct UpdateState {
    checking: bool,
    pending: Option<Update>,
    last_attempt: Option<Instant>,
}

impl UpdateState {
    pub fn snapshot(&self) -> UpdateStatus {
        UpdateStatus {
            checking: self.checking,
            available: self.pending.as_ref().map(info),
        }
    }

    fn due(&self) -> bool {
        self.last_attempt
            .is_none_or(|last| last.elapsed() >= CHECK_INTERVAL)
    }
}

fn info(update: &Update) -> UpdateInfo {
    UpdateInfo {
        version: update.version.clone(),
        notes: update.body.clone(),
    }
}

pub async fn check_background(orbit: &Arc<Orbit>) {
    if !orbit.is_idle() {
        return;
    }
    if let Err(err) = check(orbit, false).await {
        log::warn!("could not check for app updates: {err}");
    }
}

pub async fn check(orbit: &Arc<Orbit>, force: bool) -> Result<Option<UpdateInfo>> {
    let started = orbit.update(|state| {
        if state.update.checking || matches!(state.operation, Operation::Updating { .. }) {
            return Err(Error::Busy);
        }
        if !force && !state.update.due() {
            return Ok(false);
        }
        state.update.checking = true;
        state.update.last_attempt = Some(Instant::now());
        Ok(true)
    })?;
    if !started {
        return Ok(orbit.state().update.snapshot().available);
    }

    let result = async {
        orbit
            .app
            .updater_builder()
            .timeout(Duration::from_secs(30))
            .build()?
            .check()
            .await
    }
    .await;
    orbit.update(|state| {
        state.update.checking = false;
        // A failed check keeps an update already offered to the user.
        if let Ok(update) = &result {
            state.update.pending = update.clone();
        }
    });
    Ok(result?.as_ref().map(info))
}

pub async fn install(orbit: &Arc<Orbit>, version: &str) -> Result<()> {
    ensure_installed()?;
    let _guard = orbit.begin(progress(UpdateStep::Downloading, 0, None))?;
    let mut update = {
        let state = orbit.state();
        if state.update.checking {
            return Err(Error::Busy);
        }
        state
            .update
            .pending
            .as_ref()
            .filter(|update| update.version == version)
            .cloned()
            .ok_or(Error::UpdateUnavailable)?
    };
    update.timeout = Some(Duration::from_secs(5 * 60));
    let mut downloaded = 0;
    let mut last_emit = Instant::now();
    let bytes = update
        .download(
            |chunk, total| {
                downloaded += chunk as u64;
                if last_emit.elapsed() >= Duration::from_millis(100) || Some(downloaded) == total {
                    orbit.set_operation(progress(UpdateStep::Downloading, downloaded, total));
                    last_emit = Instant::now();
                }
            },
            || orbit.set_operation(progress(UpdateStep::Verifying, 0, None)),
        )
        .await?;
    // download() verifies the signature before returning any installable bytes.
    orbit.set_operation(progress(UpdateStep::Installing, 0, None));
    tokio::task::spawn_blocking(move || update.install(bytes))
        .await
        .map_err(|err| Error::Invalid(err.to_string()))??;
    // Windows' NSIS updater exits and relaunches us; macOS needs an explicit restart.
    #[cfg(not(target_os = "windows"))]
    {
        orbit.set_operation(progress(UpdateStep::Restarting, 0, None));
        orbit.app.restart();
    }
    #[allow(unreachable_code)]
    Ok(())
}

fn progress(step: UpdateStep, downloaded: u64, total: Option<u64>) -> Operation {
    Operation::Updating {
        step,
        downloaded,
        total,
    }
}

fn ensure_installed() -> Result<()> {
    #[cfg(target_os = "macos")]
    if is_macos_bundle(&tauri::utils::platform::current_exe()?) {
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    if tauri::utils::platform::bundle_type() == Some(tauri::utils::config::BundleType::Nsis) {
        return Ok(());
    }
    Err(Error::UpdateNotInstalled)
}

#[cfg(target_os = "macos")]
fn is_macos_bundle(executable: &std::path::Path) -> bool {
    let Some(macos) = executable.parent() else {
        return false;
    };
    let Some(contents) = macos.parent() else {
        return false;
    };
    let Some(bundle) = contents.parent() else {
        return false;
    };
    macos.file_name().is_some_and(|name| name == "MacOS")
        && contents.file_name().is_some_and(|name| name == "Contents")
        && bundle
            .extension()
            .is_some_and(|extension| extension == "app")
}

#[cfg(test)]
#[path = "updater_tests.rs"]
mod tests;
