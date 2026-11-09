//! Carry native local projects between saved Codex accounts after all writers stop.
//! Account-specific atoms, cloud projects and remote memberships stay with their owner.
use super::storage;
use crate::error::{Error, Result};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{cell::Cell, collections::BTreeSet, fs, path::Path};

const STATE: &str = ".codex-global-state.json";
const ACCOUNTS: &str = "electron-account-renderer-state-v1";
const PROJECTS: &str = "local-projects";
const ASSIGNMENTS: &str = "thread-project-assignments";
const HOSTS: &str = "thread-project-membership-host-ids";
const SERVER_IDS: &str = "app-server-project-id-by-legacy-project-id-by-host";
const MIGRATIONS: &str = "app-server-projects-migration-by-host";

type Object = Map<String, Value>;

fn invalid() -> Error {
    Error::Invalid("unsupported Codex local-project state; native sign-in was not changed".into())
}

fn object(value: &Value) -> Result<&Object> {
    value.as_object().ok_or_else(invalid)
}

fn map_at<'a>(value: &'a Object, key: &str) -> Result<Option<&'a Object>> {
    value.get(key).map(object).transpose()
}

fn strings(value: Option<&Value>) -> Result<Vec<String>> {
    value
        .map(|value| {
            value
                .as_array()
                .ok_or_else(invalid)?
                .iter()
                .map(|item| item.as_str().map(str::to_owned).ok_or_else(invalid))
                .collect()
        })
        .unwrap_or_else(|| Ok(Vec::new()))
}

fn append_unique(target: &mut Vec<String>, items: impl IntoIterator<Item = String>) {
    for item in items {
        if !target.contains(&item) {
            target.push(item);
        }
    }
}

fn owner(id: &str) -> Option<String> {
    let (user, account) = id.strip_prefix("codex:")?.split_once(':')?;
    if user.is_empty() || account.is_empty() {
        return None;
    }
    let bytes = serde_json::to_vec(&[user, account]).ok()?;
    Some(
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    )
}

fn local_root(id: &str, value: &Value) -> Result<Option<String>> {
    let project = object(value)?;
    if id.starts_with("g-p-")
        || [
            "chatGptBacking",
            "chatGptBackingCreation",
            "retiredChatGptBacking",
        ]
        .iter()
        .any(|key| project.get(*key).is_some_and(|value| !value.is_null()))
    {
        return Ok(None);
    }
    if project.get("id").and_then(Value::as_str) != Some(id)
        || project.get("name").and_then(Value::as_str).is_none()
    {
        return Err(invalid());
    }
    let roots = strings(project.get("rootPaths"))?;
    let root = roots
        .first()
        .filter(|root| !root.is_empty())
        .ok_or_else(invalid)?;
    // The first root identifies the project; two projects can share additional roots.
    #[cfg(windows)]
    let root = root.replace('\\', "/").to_lowercase();
    Ok(Some(root.trim_end_matches(['/', '\\']).to_owned()))
}

fn merge_local(target: &mut Object, source: &Object, host: &str) -> Result<()> {
    let before = target.clone();
    let mut projects = map_at(target, PROJECTS)?.cloned().unwrap_or_default();
    let source_projects = map_at(source, PROJECTS)?.cloned().unwrap_or_default();
    let mut aliases = Object::new();
    for (id, project) in &source_projects {
        let Some(root) = local_root(id, project)? else {
            continue;
        };
        let same_root = if let Some(existing) = projects.get(id) {
            if local_root(id, existing)?.as_ref() != Some(&root) {
                return Err(invalid());
            }
            Some(id.clone())
        } else {
            let mut found = None;
            for (existing_id, existing) in &projects {
                if local_root(existing_id, existing)?.as_ref() == Some(&root) {
                    found = Some(existing_id.clone());
                    break;
                }
            }
            found
        };
        let destination = same_root.unwrap_or_else(|| id.clone());
        projects
            .entry(destination.clone())
            .or_insert_with(|| project.clone());
        aliases.insert(id.clone(), destination.into());
    }
    if aliases.is_empty() {
        return Ok(());
    }
    let mut order = strings(target.get("project-order"))?;
    let source_order = strings(source.get("project-order"))?;
    append_unique(
        &mut order,
        source_order
            .iter()
            .chain(aliases.keys())
            .filter_map(|id| aliases.get(id).and_then(Value::as_str).map(str::to_owned)),
    );
    target.insert(PROJECTS.into(), projects.into());
    target.insert("project-order".into(), json!(order));

    let mut assignments = map_at(target, ASSIGNMENTS)?.cloned().unwrap_or_default();
    let mut hosts = map_at(target, HOSTS)?.cloned().unwrap_or_default();
    let source_hosts = map_at(source, HOSTS)?;
    let projectless = strings(target.get("projectless-thread-ids"))?;
    let mut imported = Vec::new();
    if let Some(source_assignments) = map_at(source, ASSIGNMENTS)? {
        for (thread, assignment) in source_assignments {
            let assignment = object(assignment)?;
            let destination = assignment
                .get("projectId")
                .and_then(Value::as_str)
                .and_then(|id| aliases.get(id))
                .and_then(Value::as_str);
            if assignment.get("projectKind").and_then(Value::as_str) != Some("local")
                || assignment.get("projectOrigin").and_then(Value::as_str) == Some("chatgpt")
                || source_hosts
                    .and_then(|hosts| hosts.get(thread))
                    .is_some_and(|host| host != "local")
                || hosts.get(thread).is_some_and(|host| host != "local")
                || assignments.contains_key(thread)
                || projectless.contains(thread)
            {
                continue;
            }
            if let Some(destination) = destination {
                assignments.insert(
                    thread.clone(),
                    json!({"projectKind":"local", "projectId":destination}),
                );
                hosts.insert(thread.clone(), "local".into());
                imported.push(thread.clone());
            }
        }
    }
    target.insert(ASSIGNMENTS.into(), assignments.clone().into());
    target.insert(HOSTS.into(), hosts.into());

    let mut orders = map_at(target, "sidebar-project-thread-orders")?
        .cloned()
        .unwrap_or_default();
    if let Some(source_orders) = map_at(source, "sidebar-project-thread-orders")? {
        for (id, value) in source_orders {
            let Some(id) = aliases.get(id).and_then(Value::as_str) else {
                continue;
            };
            let source_order = object(value)?;
            let mut order = orders
                .get(id)
                .map(object)
                .transpose()?
                .cloned()
                .unwrap_or_default();
            let mut threads = strings(order.get("threadIds"))?;
            append_unique(
                &mut threads,
                strings(source_order.get("threadIds"))?
                    .into_iter()
                    .filter(|thread| {
                        assignments
                            .get(thread)
                            .and_then(|a| a.get("projectId"))
                            .and_then(Value::as_str)
                            == Some(id)
                    }),
            );
            for (key, value) in source_order {
                order.entry(key.clone()).or_insert_with(|| value.clone());
            }
            order.insert("threadIds".into(), json!(threads));
            orders.insert(id.into(), order.into());
        }
    }
    target.insert("sidebar-project-thread-orders".into(), orders.into());

    let mut mappings = map_at(target, SERVER_IDS)?.cloned().unwrap_or_default();
    let mut local = mappings
        .get(host)
        .map(object)
        .transpose()?
        .cloned()
        .unwrap_or_default();
    if let Some(source) = map_at(source, SERVER_IDS)?.and_then(|mapping| mapping.get(host)) {
        for (id, server_id) in object(source)? {
            if let Some(id) = aliases.get(id).and_then(Value::as_str) {
                if !server_id.is_string() {
                    return Err(invalid());
                }
                local
                    .entry(id.to_owned())
                    .or_insert_with(|| server_id.clone());
            }
        }
    }
    mappings.insert(host.into(), local.into());
    target.insert(SERVER_IDS.into(), mappings.into());
    if *target == before {
        return Ok(());
    }
    // Let Codex migrate the copied associations through its own app-server API.
    // Never rewrite its history databases or creator-account fields.
    let mut migrations = map_at(target, MIGRATIONS)?.cloned().unwrap_or_default();
    let mut migration = migrations
        .get(host)
        .map(object)
        .transpose()?
        .cloned()
        .unwrap_or_default();
    if migration.get("version").is_some_and(|version| version != 1) {
        return Err(invalid());
    }
    let mut pending = strings(migration.get("pendingThreadAssignmentIds"))?;
    append_unique(&mut pending, imported);
    migration.extend([
        ("version".into(), json!(1)),
        ("projectsMigrated".into(), json!(false)),
        ("threadAssignmentsMigrated".into(), json!(false)),
        ("threadAssignmentsReadMigrated".into(), json!(false)),
        ("pendingThreadAssignmentIds".into(), json!(pending)),
    ]);
    migrations.insert(host.into(), migration.into());
    target.insert(MIGRATIONS.into(), migrations.into());
    Ok(())
}

fn merged(mut state: Value, target_id: &str, ids: &[String], host: &str) -> Result<Value> {
    let target_owner = owner(target_id).ok_or_else(invalid)?;
    let allowed: BTreeSet<_> = ids
        .iter()
        .filter_map(|id| owner(id))
        .chain([target_owner.clone()])
        .collect();
    let root = object(&state)?;
    let Some(accounts) = map_at(root, ACCOUNTS)? else {
        return Ok(state);
    };
    let current = accounts.get("owner").ok_or_else(invalid)?;
    if !current.is_null() && !current.is_string() {
        return Err(invalid());
    }
    let inactive = map_at(accounts, "inactive")?.ok_or_else(invalid)?;
    let active_target = current.as_str() == Some(&target_owner);
    let mut target = if active_target {
        root.clone()
    } else {
        inactive
            .get(&target_owner)
            .map(object)
            .transpose()?
            .map(|profile| map_at(profile, "globals")?.cloned().ok_or_else(invalid))
            .transpose()?
            .unwrap_or_default()
    };
    let before = target.clone();
    // Prefer the currently visible workspace over older inactive copies.
    if current.as_str().is_some_and(|key| allowed.contains(key)) && !active_target {
        merge_local(&mut target, root, host)?;
    }
    for (key, profile) in inactive {
        if key != &target_owner && allowed.contains(key) {
            let profile = object(profile)?;
            object(profile.get("atoms").ok_or_else(invalid)?)?;
            merge_local(
                &mut target,
                map_at(profile, "globals")?.ok_or_else(invalid)?,
                host,
            )?;
        }
    }
    if target == before {
        return Ok(state);
    }
    let root = state.as_object_mut().ok_or_else(invalid)?;
    if active_target {
        // Only allowlisted fields changed; account ownership and every atom remain untouched.
        *root = target;
    } else {
        let inactive = root
            .get_mut(ACCOUNTS)
            .and_then(Value::as_object_mut)
            .and_then(|accounts| accounts.get_mut("inactive"))
            .and_then(Value::as_object_mut)
            .ok_or_else(invalid)?;
        let profile = inactive
            .entry(target_owner)
            .or_insert_with(|| json!({"atoms":{}, "globals":{}}));
        let profile = profile.as_object_mut().ok_or_else(invalid)?;
        object(profile.get("atoms").ok_or_else(invalid)?)?;
        profile.insert("globals".into(), target.into());
    }
    Ok(state)
}

fn read(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => Err(invalid()),
        Ok(_) => fs::read(path).map(Some).map_err(Error::from),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub(super) struct Update {
    path: std::path::PathBuf,
    original: Vec<u8>,
    backup: Option<Vec<u8>>,
    updated: Vec<u8>,
    dirty: [Cell<bool>; 2],
}

pub(super) fn prepare(root: &Path, target_id: &str, ids: &[String]) -> Result<Option<Update>> {
    let path = root.join(STATE);
    let Some(original) = read(&path)? else {
        return Ok(None);
    };
    let state: Value = serde_json::from_slice(&original)?;
    // Codex's host identity uses its configured home, not Rust's Windows extended path prefix.
    let host = format!("local:{}", root.to_string_lossy());
    let updated = merged(state.clone(), target_id, ids, &host)?;
    if updated == state {
        return Ok(None);
    }
    let backup = read(&path.with_extension("json.bak"))?;
    Ok(Some(Update {
        path,
        original,
        backup,
        updated: serde_json::to_vec(&updated)?,
        dirty: [Cell::new(false), Cell::new(false)],
    }))
}

impl Update {
    #[cfg(test)]
    pub(super) fn planned_counts(&self, id: &str) -> Result<[(usize, usize); 2]> {
        let key = owner(id).ok_or_else(invalid)?;
        let count = |bytes: &[u8]| -> Result<(usize, usize)> {
            let state: Value = serde_json::from_slice(bytes)?;
            let globals = if state[ACCOUNTS]["owner"].as_str() == Some(&key) {
                &state
            } else {
                &state[ACCOUNTS]["inactive"][&key]["globals"]
            };
            Ok((
                globals[PROJECTS].as_object().map_or(0, Map::len),
                globals[ASSIGNMENTS].as_object().map_or(0, Map::len),
            ))
        };
        Ok([count(&self.original)?, count(&self.updated)?])
    }

    pub(super) fn apply(&self) -> Result<()> {
        self.apply_with(storage::write_private)
    }

    fn apply_with(&self, mut write: impl FnMut(&Path, &[u8]) -> Result<()>) -> Result<()> {
        let backup_path = self.path.with_extension("json.bak");
        if read(&self.path)?.as_deref() != Some(&self.original)
            || read(&backup_path)? != self.backup
        {
            return Err(Error::Invalid(
                "Codex project state changed while preparing the switch".into(),
            ));
        }
        for (index, path) in [&self.path, &backup_path].into_iter().enumerate() {
            self.dirty[index].set(true);
            write(path, &self.updated)?;
            if read(path)?.as_deref() != Some(&self.updated) {
                return Err(Error::VerifyFailed);
            }
        }
        Ok(())
    }

    pub(super) fn restore(&self) -> Result<()> {
        let backup_path = self.path.with_extension("json.bak");
        let mut result = Ok(());
        for (index, (path, bytes)) in [
            (&self.path, Some(&self.original)),
            (&backup_path, self.backup.as_ref()),
        ]
        .into_iter()
        .enumerate()
        {
            if !self.dirty[index].get() {
                continue;
            }
            let restored = match bytes {
                Some(bytes) => storage::write_private(path, bytes),
                None => match fs::remove_file(path) {
                    Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error.into()),
                    _ => Ok(()),
                },
            }
            .and_then(|()| {
                if read(path)?.as_ref() == bytes {
                    Ok(())
                } else {
                    Err(Error::VerifyFailed)
                }
            });
            result = result.and(restored);
        }
        result
    }
}

#[cfg(test)]
#[path = "codex_projects_tests.rs"]
mod tests;
