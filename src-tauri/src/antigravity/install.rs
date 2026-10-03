//! Locating the local Antigravity installation.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Install {
    /// macOS: the `.app` bundle. Windows: the install directory.
    pub root: PathBuf,
    /// The main GUI executable, used to tell the app process from its helpers.
    pub main_exe: PathBuf,
    /// Directory holding the bundled `language_server` binaries.
    pub bin_dir: PathBuf,
}

impl Install {
    /// Finds the installation, preferring standard locations, then the OS
    /// index, then whatever copy is currently running.
    pub fn locate() -> Option<Install> {
        candidates().into_iter().find_map(Install::at)
    }

    /// Whether `path` belongs to this installation.
    pub fn contains(&self, path: &Path) -> bool {
        path.starts_with(&self.root)
    }

    fn at(root: PathBuf) -> Option<Install> {
        let (main_exe, bin_dir) = layout(&root);
        main_exe.is_file().then_some(Install {
            root,
            main_exe,
            bin_dir,
        })
    }
}

#[cfg(target_os = "macos")]
pub const BUNDLE_ID: &str = "com.google.antigravity";

#[cfg(target_os = "macos")]
fn layout(root: &Path) -> (PathBuf, PathBuf) {
    let contents = root.join("Contents");
    (
        contents.join("MacOS/Antigravity"),
        contents.join("Resources/bin"),
    )
}

#[cfg(target_os = "macos")]
fn candidates() -> Vec<PathBuf> {
    let mut out = vec![PathBuf::from("/Applications/Antigravity.app")];
    if let Some(home) = std::env::home_dir() {
        out.push(home.join("Applications/Antigravity.app"));
    }
    // Spotlight knows bundles installed anywhere else.
    if let Ok(output) = std::process::Command::new("/usr/bin/mdfind")
        .arg(format!("kMDItemCFBundleIdentifier == '{BUNDLE_ID}'"))
        .output()
    {
        out.extend(
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(PathBuf::from),
        );
    }
    out.extend(running_roots(|exe| {
        exe.ancestors()
            .nth(3)
            .map(Path::to_path_buf)
            .filter(|root| {
                root.extension().is_some_and(|ext| ext == "app")
                    && exe.ends_with("Contents/MacOS/Antigravity")
            })
    }));
    out
}

#[cfg(windows)]
fn layout(root: &Path) -> (PathBuf, PathBuf) {
    (
        root.join("Antigravity.exe"),
        root.join("resources").join("bin"),
    )
}

#[cfg(windows)]
fn candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        out.push(PathBuf::from(local).join("Programs").join("Antigravity"));
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        out.push(PathBuf::from(program_files).join("Antigravity"));
    }
    out.extend(running_roots(|exe| {
        exe.file_name()
            .is_some_and(|name| name.eq_ignore_ascii_case("Antigravity.exe"))
            .then(|| exe.parent().map(Path::to_path_buf))
            .flatten()
    }));
    out
}

#[cfg(not(any(target_os = "macos", windows)))]
fn layout(root: &Path) -> (PathBuf, PathBuf) {
    (root.join("antigravity"), root.join("resources").join("bin"))
}

#[cfg(not(any(target_os = "macos", windows)))]
fn candidates() -> Vec<PathBuf> {
    Vec::new()
}

/// Install roots derived from the executables of running processes.
#[cfg(any(target_os = "macos", windows))]
fn running_roots(root_of: impl Fn(&Path) -> Option<PathBuf>) -> Vec<PathBuf> {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
    );
    let mut roots: Vec<PathBuf> = system
        .processes()
        .values()
        .filter_map(|process| process.exe().and_then(&root_of))
        .collect();
    roots.dedup();
    roots
}
