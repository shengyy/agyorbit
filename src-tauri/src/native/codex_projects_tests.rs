use super::super::transaction;
use super::*;

const A: &str = "codex:user-a:workspace-a";
const B: &str = "codex:user-b:workspace-b";
const C: &str = "codex:user-c:workspace-c";
const HOST: &str = "local:/fixtures/codex";

fn project(id: &str, roots: &[&str]) -> Value {
    json!({"id":id,"name":id,"rootPaths":roots,"createdAt":1,"updatedAt":2})
}

fn globals(id: &str, path: &str, thread: &str) -> Value {
    json!({
        PROJECTS:{id:project(id, &[path])},
        "project-order":[id],
        ASSIGNMENTS:{thread:{"projectKind":"local","projectId":id}},
        HOSTS:{thread:"local"},
        SERVER_IDS:{HOST:{id:format!("server-{id}")}},
        "sidebar-project-thread-orders":{id:{"threadIds":[thread],"sortKey":"manual"}}
    })
}

fn state() -> Value {
    let mut state = globals("project-a", "/fixtures/client", "thread-a");
    state["electron-persisted-atom-state"] = json!({"cloud-session":"a-only"});
    state["unrelated"] = json!({"keep":true});
    state[ACCOUNTS] = json!({
        "owner":owner(A).unwrap(),
        "inactive":{
            owner(B).unwrap():{"atoms":{"cloud-session":"b-only"},"globals":globals("project-b","/fixtures/other","thread-b")},
            owner(C).unwrap():{"atoms":{"cloud-session":"c-only"},"globals":globals("project-c","/fixtures/private","thread-c")}
        }
    });
    state
}

fn run(state: Value, target: &str) -> Result<Value> {
    merged(state, target, &[A.into(), B.into()], HOST, None)
}

fn target(state: &Value, id: &str) -> Value {
    let key = owner(id).unwrap();
    if state[ACCOUNTS]["owner"] == key {
        state.clone()
    } else {
        state[ACCOUNTS]["inactive"][key]["globals"].clone()
    }
}

#[test]
fn merges_registered_local_projects_without_copying_account_atoms() {
    let before = state();
    let after = run(before.clone(), B).unwrap();
    let destination = target(&after, B);
    assert_eq!(object(&destination[PROJECTS]).unwrap().len(), 2);
    assert_eq!(
        destination["project-order"],
        json!(["project-a", "project-b"])
    );
    assert_eq!(
        destination[ASSIGNMENTS]["thread-a"]["projectId"],
        "project-a"
    );
    assert_eq!(
        destination[SERVER_IDS][HOST]["project-a"],
        "server-project-a"
    );
    assert_eq!(
        after[ACCOUNTS]["inactive"][owner(B).unwrap()]["atoms"]["cloud-session"],
        before[ACCOUNTS]["inactive"][owner(B).unwrap()]["atoms"]["cloud-session"]
    );
    assert_eq!(after[ACCOUNTS]["owner"], before[ACCOUNTS]["owner"]);
    assert_eq!(
        after[ACCOUNTS]["inactive"][owner(C).unwrap()],
        before[ACCOUNTS]["inactive"][owner(C).unwrap()]
    );
    assert_eq!(after[PROJECTS], before[PROJECTS]);
    assert_eq!(after["unrelated"], before["unrelated"]);
    assert_eq!(
        destination[MIGRATIONS][HOST]["pendingThreadAssignmentIds"],
        json!(["thread-a"])
    );
}

#[test]
fn repairs_active_profile_from_registered_inactive_projects() {
    let before = state();
    let after = run(before.clone(), A).unwrap();
    assert_eq!(
        after[PROJECTS]["project-b"],
        before[ACCOUNTS]["inactive"][owner(B).unwrap()]["globals"][PROJECTS]["project-b"]
    );
    assert_eq!(after[ASSIGNMENTS]["thread-b"]["projectId"], "project-b");
    assert_eq!(after[ACCOUNTS], before[ACCOUNTS]);
    assert_eq!(
        after[ATOMS]["cloud-session"],
        before[ATOMS]["cloud-session"]
    );
    assert!(after[PROJECTS].get("project-c").is_none());
}

#[test]
fn new_account_gets_local_projects_and_empty_private_atoms() {
    let after = run(state(), "codex:new:workspace").unwrap();
    let key = owner("codex:new:workspace").unwrap();
    assert!(
        after[ACCOUNTS]["inactive"][&key]["atoms"]
            .get("cloud-session")
            .is_none()
    );
    assert_eq!(
        after[ACCOUNTS]["inactive"][&key]["atoms"]["sidebar-project-list-expanded-v1"],
        true
    );
    assert_eq!(
        object(&after[ACCOUNTS]["inactive"][key]["globals"][PROJECTS])
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn outgoing_sidebar_order_and_project_expansion_follow_directory_aliases() {
    let mut before = state();
    before[PROJECTS]["project-extra"] = project("project-extra", &["/fixtures/extra"]);
    before[PROJECTS]["g-p-private"] = project("g-p-private", &["/fixtures/cloud"]);
    before[ATOMS] = json!({
        "cloud-session":"a-only",
        "unified-sidebar-project-order-v1":["codex:project:project-extra","codex:project:g-p-private","codex:project:project-a"],
        "sidebar-project-expanded-v1-codex:project-a":false,
        "sidebar-project-expanded-v1-codex:project-extra":true,
        "sidebar-project-expanded-v1-codex:g-p-private":true,
        "flat-project-sidebar-preferences-v1":{"mode":"project","projectSortMode":"manual","initialized":true,"chatSortMode":"source-only"}
    });
    let b = owner(B).unwrap();
    before[ACCOUNTS]["inactive"][&b]["globals"][PROJECTS]["project-b"]["rootPaths"] =
        json!(["/fixtures/client"]);
    before[ACCOUNTS]["inactive"][&b]["atoms"] = json!({
        "cloud-session":"b-only",
        "unified-sidebar-project-order-v1":["cloud:private","codex:project:project-b","section:private"],
        "sidebar-project-expanded-v1-codex:project-b":true,
        "sidebar-collapsed-sections-v1":{"threads":true,"cloud":true,"pinned":false},
        "sidebar-project-list-expanded-v1":false,
        "flat-project-sidebar-preferences-v1":{"mode":"list","projectSortMode":"updated_at","chatSortMode":"destination-only"}
    });
    let after = run(before, B).unwrap();
    let atoms = &after[ACCOUNTS]["inactive"][&b]["atoms"];
    assert_eq!(
        atoms["unified-sidebar-project-order-v1"],
        json!([
            "cloud:private",
            "codex:project:project-extra",
            "section:private",
            "codex:project:project-b"
        ])
    );
    assert_eq!(
        target(&after, B)["project-order"],
        json!(["project-extra", "project-b"])
    );
    assert_eq!(atoms["sidebar-project-expanded-v1-codex:project-b"], false);
    assert_eq!(
        atoms["sidebar-project-expanded-v1-codex:project-extra"],
        true
    );
    assert!(
        atoms
            .get("sidebar-project-expanded-v1-codex:g-p-private")
            .is_none()
    );
    assert_eq!(
        atoms["sidebar-collapsed-sections-v1"],
        json!({"threads":false,"cloud":true,"pinned":false})
    );
    assert_eq!(atoms["sidebar-project-list-expanded-v1"], true);
    assert_eq!(
        atoms["flat-project-sidebar-preferences-v1"]["mode"],
        "project"
    );
    assert_eq!(
        atoms["flat-project-sidebar-preferences-v1"]["chatSortMode"],
        "destination-only"
    );
    assert_eq!(atoms["cloud-session"], "b-only");
}

#[test]
fn every_switch_and_restart_reopens_the_local_project_section_and_full_list() {
    let mut once = activate(run(state(), B).unwrap(), B);
    once[ATOMS]["sidebar-collapsed-sections-v1"] = json!({"threads":true,"cloud":true});
    once[ATOMS]["sidebar-project-list-expanded-v1"] = false.into();
    once[ATOMS]["sidebar-project-expanded-v1-codex:project-a"] = false.into();
    let repaired = run(once, B).unwrap();
    assert_eq!(
        repaired[ATOMS]["sidebar-collapsed-sections-v1"]["threads"],
        false
    );
    assert_eq!(repaired[ATOMS]["sidebar-project-list-expanded-v1"], true);
    assert_eq!(
        repaired[ATOMS]["sidebar-project-expanded-v1-codex:project-a"],
        false
    );
    let mut next = repaired;
    next[ACCOUNTS]["inactive"][owner(A).unwrap()]["atoms"]["sidebar-collapsed-sections-v1"] =
        json!({"threads":true});
    next[ACCOUNTS]["inactive"][owner(A).unwrap()]["atoms"]["sidebar-project-list-expanded-v1"] =
        false.into();
    let switched = activate(run(next, A).unwrap(), A);
    assert_eq!(
        switched[ATOMS]["sidebar-collapsed-sections-v1"]["threads"],
        false
    );
    assert_eq!(switched[ATOMS]["sidebar-project-list-expanded-v1"], true);
}

#[test]
fn repairs_already_migrated_but_hidden_sidebar_without_restarting_migrations() {
    let mut once = activate(run(state(), B).unwrap(), B);
    once[MIGRATIONS][HOST] = json!({"version":1,"projectsMigrated":true,"threadAssignmentsMigrated":true,"threadAssignmentsReadMigrated":true,"pendingThreadAssignmentIds":[]});
    once[ATOMS]["unified-sidebar-project-order-v1"] = json!(["codex:project:project-b"]);
    once[ACCOUNTS]["inactive"][owner(A).unwrap()]["globals"] = target(&once, B);
    once[ACCOUNTS]["inactive"][owner(A).unwrap()]["atoms"]["unified-sidebar-project-order-v1"] =
        json!(["codex:project:project-a", "codex:project:project-b"]);
    let migration = once[MIGRATIONS].clone();
    let repaired = run(once, B).unwrap();
    assert_eq!(
        repaired[ATOMS]["unified-sidebar-project-order-v1"],
        json!(["codex:project:project-a", "codex:project:project-b"])
    );
    assert_eq!(repaired[MIGRATIONS], migration);
}

#[test]
fn explicit_outgoing_profile_wins_over_older_profiles_even_after_signed_out_startup() {
    let mut before = state();
    before[ATOMS]["unified-sidebar-project-order-v1"] = json!(["codex:project:project-a"]);
    before[ATOMS]["sidebar-organize-mode-v1"] = "project".into();
    before[ACCOUNTS]["inactive"][owner(A).unwrap()] =
        json!({"atoms":before[ATOMS],"globals":before});
    before[ACCOUNTS]["owner"] = Value::Null;
    before[ACCOUNTS]["inactive"][owner(C).unwrap()]["atoms"] = json!({
        "unified-sidebar-project-order-v1":["codex:project:project-c","codex:project:project-a"],
        "sidebar-organize-mode-v1":"list"
    });
    before[ACCOUNTS]["inactive"][owner(C).unwrap()]["globals"][PROJECTS]["project-a"] =
        project("project-a", &["/fixtures/client"]);
    let after = merged(before, B, &[A.into(), B.into(), C.into()], HOST, Some(A)).unwrap();
    let atoms = &after[ACCOUNTS]["inactive"][owner(B).unwrap()]["atoms"];
    let order = strings(atoms.get("unified-sidebar-project-order-v1")).unwrap();
    assert_eq!(order.first().unwrap(), "codex:project:project-a");
    assert_eq!(order.len(), 3);
    assert_eq!(atoms["sidebar-organize-mode-v1"], "project");
}

#[test]
fn no_local_projects_leave_sidebar_atoms_unchanged() {
    let before = json!({ATOMS:{"sidebar-project-list-expanded-v1":false},ACCOUNTS:{"owner":owner(A).unwrap(),"inactive":{}}});
    assert_eq!(run(before.clone(), A).unwrap(), before);
}

#[test]
fn unsupported_sidebar_shapes_are_rejected_before_native_writes() {
    for (key, value) in [
        ("unified-sidebar-project-order-v1", json!([42])),
        ("sidebar-project-expanded-v1-codex:project-a", json!("yes")),
        ("flat-project-sidebar-preferences-v1", json!({"mode":42})),
        ("sidebar-organize-mode-v1", json!(true)),
    ] {
        let mut before = state();
        before[ATOMS][key] = value;
        assert!(run(before, B).is_err(), "{key}");
    }
    let mut before = state();
    before[ACCOUNTS]["inactive"][owner(B).unwrap()]["atoms"]["sidebar-collapsed-sections-v1"] =
        json!([]);
    assert!(run(before, B).is_err());
}

#[test]
fn same_primary_directory_keeps_target_project_identity_and_settings() {
    let mut before = state();
    let b = owner(B).unwrap();
    let destination = &mut before[ACCOUNTS]["inactive"][&b]["globals"];
    destination[PROJECTS]["project-b"]["rootPaths"] = json!(["/fixtures/client/"]);
    destination[PROJECTS]["project-b"]["name"] = "My chosen name".into();
    destination["project-appearances"] = json!({"project-b":{"color":"blue"}});
    let after = run(before, B).unwrap();
    let destination = target(&after, B);
    assert_eq!(object(&destination[PROJECTS]).unwrap().len(), 1);
    assert_eq!(destination[PROJECTS]["project-b"]["name"], "My chosen name");
    assert_eq!(
        destination[ASSIGNMENTS]["thread-a"]["projectId"],
        "project-b"
    );
    assert_eq!(
        destination[SERVER_IDS][HOST]["project-b"],
        "server-project-b"
    );
    assert_eq!(
        destination["sidebar-project-thread-orders"]["project-b"]["threadIds"],
        json!(["thread-b", "thread-a"])
    );
    assert_eq!(
        destination["project-appearances"]["project-b"]["color"],
        "blue"
    );
}

#[test]
fn shared_secondary_root_does_not_collapse_distinct_projects() {
    let mut before = state();
    before[PROJECTS]["project-a"]["rootPaths"] = json!(["/fixtures/client", "/fixtures/other"]);
    before[ACCOUNTS]["inactive"][owner(B).unwrap()]["globals"][PROJECTS]["project-b"]["rootPaths"] =
        json!(["/fixtures/other", "/fixtures/client"]);
    assert_eq!(
        object(&target(&run(before, B).unwrap(), B)[PROJECTS])
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn target_membership_and_explicit_projectless_choice_win() {
    let mut before = state();
    before[ASSIGNMENTS]["thread-b"] = json!({"projectKind":"local","projectId":"project-a"});
    before[ASSIGNMENTS]["unassigned"] = json!({"projectKind":"local","projectId":"project-a"});
    before[ACCOUNTS]["inactive"][owner(B).unwrap()]["globals"]["projectless-thread-ids"] =
        json!(["unassigned"]);
    let destination = target(&run(before, B).unwrap(), B);
    assert_eq!(
        destination[ASSIGNMENTS]["thread-b"]["projectId"],
        "project-b"
    );
    assert!(destination[ASSIGNMENTS].get("unassigned").is_none());
    assert_eq!(destination["projectless-thread-ids"], json!(["unassigned"]));
}

#[test]
fn cloud_projects_remote_hosts_and_unknown_accounts_are_not_shared() {
    let mut before = state();
    before[PROJECTS]["g-p-cloud"] = project("g-p-cloud", &["/fixtures/cloud"]);
    before[PROJECTS]["backed"] = project("backed", &["/fixtures/backed"]);
    before[PROJECTS]["backed"]["chatGptBacking"] =
        json!({"projectId":"cloud-project","userId":"user-a"});
    for (thread, id, host) in [
        ("remote-thread", "project-a", "remote-host"),
        ("cloud-thread", "g-p-cloud", "local"),
        ("backed-thread", "backed", "local"),
    ] {
        before[ASSIGNMENTS][thread] = json!({"projectKind":"local","projectId":id});
        before[HOSTS][thread] = host.into();
    }
    before["remote-projects"] = json!({"secret":"do-not-copy"});
    before[SERVER_IDS]["remote:/fixtures"] = json!({"project-a":"remote-server-project"});
    let destination = target(&run(before, B).unwrap(), B);
    for id in ["g-p-cloud", "backed", "project-c"] {
        assert!(destination[PROJECTS].get(id).is_none())
    }
    for id in ["remote-thread", "cloud-thread", "backed-thread", "thread-c"] {
        assert!(destination[ASSIGNMENTS].get(id).is_none())
    }
    assert!(destination.get("remote-projects").is_none());
    assert!(destination[SERVER_IDS].get("remote:/fixtures").is_none());
}

#[test]
fn project_id_collision_with_another_directory_fails_closed() {
    let mut before = state();
    before[ACCOUNTS]["inactive"][owner(B).unwrap()]["globals"][PROJECTS]["project-a"] =
        project("project-a", &["/fixtures/different"]);
    assert!(run(before, B).is_err());
}

#[test]
fn completed_migrations_are_not_reset_without_new_data() {
    let mut once = run(state(), B).unwrap();
    let g = &mut once[ACCOUNTS]["inactive"][owner(B).unwrap()]["globals"];
    g[MIGRATIONS][HOST] = json!({"version":1,"projectsMigrated":true,"threadAssignmentsMigrated":true,"threadAssignmentsReadMigrated":true,"pendingThreadAssignmentIds":[]});
    assert_eq!(run(once.clone(), B).unwrap(), once);
}

#[test]
fn repeated_repairs_are_idempotent_and_keep_source_profiles() {
    let before = state();
    let once = run(before.clone(), A).unwrap();
    assert_eq!(run(once.clone(), A).unwrap(), once);
    assert_eq!(once[ACCOUNTS], before[ACCOUNTS]);
}

fn activate(mut state: Value, id: &str) -> Value {
    // Reproduce the inspected desktop's owner selection, including private atoms.
    let key = owner(id).unwrap();
    let previous = state[ACCOUNTS]["owner"].as_str().unwrap().to_owned();
    let next = state[ACCOUNTS]["inactive"][&key].clone();
    let keys = [
        PROJECTS,
        ASSIGNMENTS,
        HOSTS,
        SERVER_IDS,
        MIGRATIONS,
        "project-order",
        "sidebar-project-thread-orders",
        "projectless-thread-ids",
    ];
    let mut previous_globals = Object::new();
    for name in keys {
        if let Some(value) = state.as_object_mut().unwrap().remove(name) {
            previous_globals.insert(name.into(), value);
        }
        if let Some(value) = next["globals"].get(name) {
            state[name] = value.clone()
        }
    }
    state[ACCOUNTS]["inactive"][previous] =
        json!({"atoms":state["electron-persisted-atom-state"], "globals":previous_globals});
    state[ACCOUNTS]["inactive"]
        .as_object_mut()
        .unwrap()
        .remove(&key);
    state[ACCOUNTS]["owner"] = key.into();
    state["electron-persisted-atom-state"] = next["atoms"].clone();
    state
}

#[test]
fn a_to_b_to_a_keeps_projects_old_chats_and_new_chats() {
    let after_b = activate(run(state(), B).unwrap(), B);
    assert_eq!(object(&after_b[PROJECTS]).unwrap().len(), 2);
    assert_eq!(
        after_b["electron-persisted-atom-state"]["cloud-session"],
        "b-only"
    );
    let mut after_b = after_b;
    after_b[ASSIGNMENTS]["new-chat-in-b"] = json!({"projectKind":"local","projectId":"project-b"});
    after_b[HOSTS]["new-chat-in-b"] = "local".into();
    let after_a = activate(run(after_b, A).unwrap(), A);
    assert_eq!(object(&after_a[PROJECTS]).unwrap().len(), 2);
    assert_eq!(object(&after_a[ASSIGNMENTS]).unwrap().len(), 3);
    assert_eq!(
        after_a[ASSIGNMENTS]["new-chat-in-b"]["projectId"],
        "project-b"
    );
    assert_eq!(
        after_a["electron-persisted-atom-state"]["cloud-session"],
        "a-only"
    );
    assert_eq!(after_a[ASSIGNMENTS]["thread-a"]["projectId"], "project-a");
}

#[test]
fn unknown_migration_version_is_not_downgraded() {
    let mut before = state();
    before[ACCOUNTS]["inactive"][owner(B).unwrap()]["globals"][MIGRATIONS] =
        json!({HOST:{"version":2}});
    assert!(run(before, B).is_err());
}

#[test]
fn signed_out_startup_can_restore_the_target_inactive_profile() {
    let mut before = state();
    let source = before.clone();
    before[ACCOUNTS]["inactive"][owner(A).unwrap()] =
        json!({"atoms":{"cloud-session":"a-only"},"globals":source});
    before[ACCOUNTS]["owner"] = Value::Null;
    for key in [
        PROJECTS,
        ASSIGNMENTS,
        HOSTS,
        "project-order",
        SERVER_IDS,
        "sidebar-project-thread-orders",
    ] {
        before.as_object_mut().unwrap().remove(key);
    }
    let after = run(before, B).unwrap();
    assert_eq!(object(&target(&after, B)[PROJECTS]).unwrap().len(), 2);
    assert!(after[PROJECTS].is_null());
}

#[test]
fn malformed_selected_profile_or_project_fails_without_writes() {
    for broken in [
        json!({"owner":[],"inactive":{}}),
        json!({"owner":owner(A).unwrap(),"inactive":[]}),
    ] {
        let mut before = state();
        before[ACCOUNTS] = broken;
        assert!(run(before, B).is_err());
    }
    let mut before = state();
    before[PROJECTS]["project-a"]["rootPaths"] = json!([42]);
    assert!(run(before, B).is_err());
    let mut before = state();
    before[ACCOUNTS]["inactive"][owner(B).unwrap()]["atoms"] = json!([]);
    assert!(run(before, B).is_err());
}

#[test]
fn older_clients_without_account_scoping_are_unchanged() {
    let before = globals("project-a", "/fixtures/client", "thread-a");
    assert_eq!(run(before.clone(), B).unwrap(), before);
}

fn prepared(backup: bool) -> (tempfile::TempDir, Update) {
    let dir = tempfile::tempdir().unwrap();
    let mut bytes = serde_json::to_vec_pretty(&state()).unwrap();
    bytes.push(b'\n');
    fs::write(dir.path().join(STATE), &bytes).unwrap();
    if backup {
        fs::write(
            dir.path().join(format!("{STATE}.bak")),
            b"original backup bytes\n",
        )
        .unwrap()
    }
    let update = prepare(dir.path(), B, &[A.into(), B.into()], Some(A))
        .unwrap()
        .unwrap();
    (dir, update)
}

#[test]
fn writes_both_state_and_backup_and_reads_back_before_success() {
    let (dir, update) = prepared(true);
    update.apply().unwrap();
    assert_eq!(fs::read(dir.path().join(STATE)).unwrap(), update.updated);
    assert_eq!(
        fs::read(dir.path().join(format!("{STATE}.bak"))).unwrap(),
        update.updated
    );
    let after: Value = serde_json::from_slice(&update.updated).unwrap();
    assert!(target(&after, B)[PROJECTS].get("project-a").is_some());
}

#[test]
fn failed_second_file_write_restores_original_bytes_and_credentials() {
    let (dir, update) = prepared(true);
    let calls = Cell::new(0);
    let credential = Cell::new("original");
    let result = transaction::apply(
        || {
            credential.set("target");
            update.apply_with(|path, bytes| {
                calls.set(calls.get() + 1);
                if calls.get() == 2 {
                    return Err(Error::VerifyFailed);
                }
                storage::write_private(path, bytes)
            })
        },
        || {
            credential.set("original");
            update.restore()
        },
    );
    assert!(matches!(result, Err(Error::VerifyFailed)));
    assert_eq!(credential.get(), "original");
    assert_eq!(fs::read(dir.path().join(STATE)).unwrap(), update.original);
    assert_eq!(
        fs::read(dir.path().join(format!("{STATE}.bak"))).unwrap(),
        update.backup.unwrap()
    );
}

#[test]
fn corrupt_readback_restores_state_and_removes_new_backup() {
    let (dir, update) = prepared(false);
    let result = transaction::apply(
        || update.apply_with(|path, _| storage::write_private(path, b"corrupt")),
        || update.restore(),
    );
    assert!(matches!(result, Err(Error::VerifyFailed)));
    assert_eq!(fs::read(dir.path().join(STATE)).unwrap(), update.original);
    assert!(!dir.path().join(format!("{STATE}.bak")).exists());
}

#[test]
fn late_account_failure_restores_successful_project_writes() {
    let (dir, update) = prepared(false);
    let result = transaction::apply(
        || {
            update.apply()?;
            Err(Error::AccountNotFound)
        },
        || update.restore(),
    );
    assert!(matches!(result, Err(Error::AccountNotFound)));
    assert_eq!(fs::read(dir.path().join(STATE)).unwrap(), update.original);
    assert!(!dir.path().join(format!("{STATE}.bak")).exists());
}

#[test]
fn failed_project_restore_reports_rollback_failure_and_attempts_both_files() {
    let (dir, update) = prepared(true);
    let result = transaction::apply(
        || {
            update.apply()?;
            fs::remove_file(&update.path)?;
            fs::create_dir(&update.path)?;
            Err(Error::VerifyFailed)
        },
        || update.restore(),
    );
    assert!(matches!(result, Err(Error::RollbackFailed)));
    assert_eq!(
        fs::read(dir.path().join(format!("{STATE}.bak"))).unwrap(),
        update.backup.unwrap()
    );
}

#[test]
fn concurrent_writer_is_not_overwritten_or_rolled_back() {
    let (dir, update) = prepared(true);
    fs::write(dir.path().join(STATE), b"external writer").unwrap();
    assert!(transaction::apply(|| update.apply(), || update.restore()).is_err());
    assert_eq!(
        fs::read(dir.path().join(STATE)).unwrap(),
        b"external writer"
    );
}

#[test]
fn absent_state_is_noop_and_malformed_state_is_retained() {
    let dir = tempfile::tempdir().unwrap();
    assert!(
        prepare(dir.path(), B, &[A.into()], Some(A))
            .unwrap()
            .is_none()
    );
    fs::write(dir.path().join(STATE), b"broken").unwrap();
    assert!(prepare(dir.path(), B, &[A.into()], Some(A)).is_err());
    assert_eq!(fs::read(dir.path().join(STATE)).unwrap(), b"broken");
}

#[cfg(unix)]
#[test]
fn linked_state_or_backup_is_rejected_without_touching_the_destination() {
    let (dir, update) = prepared(false);
    let outside = dir.path().join("outside");
    fs::write(&outside, b"keep").unwrap();
    std::os::unix::fs::symlink(&outside, dir.path().join(format!("{STATE}.bak"))).unwrap();
    assert!(prepare(dir.path(), B, &[A.into()], Some(A)).is_err());
    assert!(update.apply().is_err());
    assert_eq!(fs::read(&outside).unwrap(), b"keep");
}
