//! Allowlisted local workspace presentation, never account or cloud state.
use super::{Object, PROJECTS, Result, Value, append_unique, invalid, local_root, map_at, strings};
use serde_json::json;
use std::collections::BTreeSet;

const ORDER: &str = "unified-sidebar-project-order-v1";
const EXPANDED: &str = "sidebar-project-expanded-v1-codex:";
const COLLAPSED: &str = "sidebar-collapsed-sections-v1";
const PREFS: &str = "flat-project-sidebar-preferences-v1";

fn local_ids(globals: &Object) -> Result<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    if let Some(projects) = map_at(globals, PROJECTS)? {
        for (id, project) in projects {
            if local_root(id, project)?.is_some() {
                ids.insert(id.clone());
            }
        }
    }
    Ok(ids)
}

fn order_key(id: &str) -> String {
    format!("codex:project:{id}")
}

pub(super) fn merge(
    globals: &mut Object,
    atoms: &mut Object,
    source_globals: &Object,
    source: &Object,
    aliases: &Object,
    prefer_source: bool,
) -> Result<()> {
    if aliases.is_empty() {
        return Ok(());
    }
    let ids = local_ids(globals)?;
    let keys: BTreeSet<_> = ids.iter().map(|id| order_key(id)).collect();
    let existing = strings(atoms.get(ORDER))?;
    let mut source_order = Vec::new();
    let source_stored_order = strings(source.get(ORDER))?;
    let source_global_order = strings(source_globals.get("project-order"))?;
    for id in source_stored_order
        .iter()
        .filter_map(|key| key.strip_prefix("codex:project:"))
        .chain(source_global_order.iter().map(String::as_str))
        .chain(aliases.keys().map(String::as_str))
    {
        if let Some(id) = aliases.get(id).and_then(Value::as_str) {
            append_unique(&mut source_order, [order_key(id)]);
        }
    }
    // A previously hidden workspace can already have copied globals but only one sidebar item.
    let prefer_order = prefer_source
        || source_order.len() > existing.iter().filter(|key| keys.contains(*key)).count();
    let mut local_order: Vec<_> = if prefer_order {
        source_order.clone()
    } else {
        existing
            .iter()
            .filter(|key| keys.contains(*key))
            .cloned()
            .collect()
    };
    append_unique(
        &mut local_order,
        existing.iter().filter(|key| keys.contains(*key)).cloned(),
    );
    append_unique(&mut local_order, source_order);
    let mut remaining = local_order.into_iter();
    // Leave destination cloud/custom entries in place while replacing only native local slots.
    let mut order: Vec<_> = existing
        .into_iter()
        .filter_map(|key| {
            if keys.contains(&key) {
                remaining.next()
            } else {
                Some(key)
            }
        })
        .collect();
    append_unique(&mut order, remaining);
    atoms.insert(ORDER.into(), json!(order));
    if prefer_order {
        let mut project_order: Vec<_> = strings(atoms.get(ORDER))?
            .iter()
            .filter_map(|key| {
                key.strip_prefix("codex:project:")
                    .filter(|id| ids.contains(*id))
                    .map(str::to_owned)
            })
            .collect();
        append_unique(&mut project_order, strings(globals.get("project-order"))?);
        globals.insert("project-order".into(), json!(project_order));
    }
    for (source_id, destination) in aliases {
        let destination = destination.as_str().ok_or_else(invalid)?;
        if let Some(value) = source.get(&format!("{EXPANDED}{source_id}")) {
            if !value.is_null() && !value.is_boolean() {
                return Err(invalid());
            }
            let key = format!("{EXPANDED}{destination}");
            if prefer_source || !atoms.contains_key(&key) {
                atoms.insert(key, value.clone());
            }
        }
    }
    if let Some(source_prefs) = map_at(source, PREFS)? {
        let mut prefs = map_at(atoms, PREFS)?.cloned().unwrap_or_default();
        for key in ["mode", "projectSortMode", "initialized"] {
            if let Some(value) = source_prefs.get(key) {
                if (key == "initialized" && !value.is_boolean())
                    || (key != "initialized" && !value.is_string())
                {
                    return Err(invalid());
                }
                if prefer_source || !prefs.contains_key(key) {
                    prefs.insert(key.into(), value.clone());
                }
            }
        }
        atoms.insert(PREFS.into(), prefs.into());
    }
    for (key, fallback) in [
        ("sidebar-organize-mode-v1", "mode"),
        ("codex-sidebar-sort-mode-v1", "projectSortMode"),
    ] {
        let value = source
            .get(key)
            .or_else(|| source.get(PREFS).and_then(|prefs| prefs.get(fallback)));
        if let Some(value) = value {
            if !value.is_string() {
                return Err(invalid());
            }
            if prefer_source || !atoms.contains_key(key) {
                atoms.insert(key.into(), value.clone());
            }
        }
    }
    Ok(())
}

pub(super) fn reveal(globals: &Object, atoms: &mut Object) -> Result<()> {
    let ids = local_ids(globals)?;
    if ids.is_empty() {
        return Ok(());
    }
    let mut collapsed = map_at(atoms, COLLAPSED)?.cloned().unwrap_or_default();
    collapsed.insert("threads".into(), json!(false));
    atoms.insert(COLLAPSED.into(), collapsed.into());
    atoms.insert("sidebar-project-list-expanded-v1".into(), json!(true));
    let mut order = strings(atoms.get(ORDER))?;
    let mut project_order = strings(globals.get("project-order"))?;
    append_unique(&mut project_order, ids.iter().cloned());
    append_unique(
        &mut order,
        project_order
            .into_iter()
            .filter(|id| ids.contains(id))
            .map(|id| order_key(&id)),
    );
    atoms.insert(ORDER.into(), json!(order));
    Ok(())
}
