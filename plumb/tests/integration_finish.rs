use assert_cmd::cargo;
use assert_cmd::prelude::*;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use strata::decode::decode;
use strata::value::Value;

fn plumb_binary() -> Command {
    Command::new(cargo::cargo_bin!("plumb"))
}

fn get_session_id(root: &Path) -> String {
    fs::read_to_string(root.join(".plumb/active"))
        .unwrap()
        .trim()
        .to_string()
}

fn go_with_true_editor(root: &Path, target: &str) -> Output {
    plumb_binary()
        .current_dir(root)
        .env("EDITOR", "true")
        .arg("go")
        .arg(target)
        .output()
        .unwrap()
}

fn session_scb_path(root: &Path, session_id: &str) -> std::path::PathBuf {
    root.join(".plumb")
        .join("sessions")
        .join(session_id)
        .join("session.scb")
}

fn read_session_status(root: &Path, session_id: &str) -> String {
    let raw = fs::read(session_scb_path(root, session_id)).unwrap();
    let decoded = decode(&raw).unwrap();
    let Value::Map(map) = decoded else {
        panic!("session.scb should decode to map");
    };
    let Some(Value::String(status)) = map.get("status") else {
        panic!("session.scb should include string status");
    };
    status.clone()
}

#[test]
fn finish_happy_path_clears_active_and_sets_session_finished() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    fs::write(root.join("a.rs"), "").unwrap();
    fs::write(root.join("b.rs"), "").unwrap();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("a.rs")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("b.rs")
        .assert()
        .success();

    let session_id = get_session_id(root);
    let finish = plumb_binary()
        .current_dir(root)
        .arg("finish")
        .output()
        .unwrap();
    assert!(
        finish.status.success(),
        "finish should succeed: {}",
        String::from_utf8_lossy(&finish.stderr)
    );

    let active_path = root.join(".plumb").join("active");
    assert!(
        !active_path.exists(),
        "active session pointer file should be removed"
    );

    let status = read_session_status(root, &session_id);
    assert_eq!(status, "finished");
}

#[test]
fn finish_refuses_when_item_in_progress_and_keeps_active_session() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    fs::write(root.join("a.rs"), "").unwrap();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("a.rs")
        .assert()
        .success();

    let session_id = get_session_id(root);
    let go = go_with_true_editor(root, "1");
    assert!(
        go.status.success(),
        "go should succeed: {}",
        String::from_utf8_lossy(&go.stderr)
    );

    let finish = plumb_binary()
        .current_dir(root)
        .arg("finish")
        .output()
        .unwrap();
    assert!(
        !finish.status.success(),
        "finish should fail while item is in progress"
    );

    let stderr = String::from_utf8_lossy(&finish.stderr).to_lowercase();
    assert!(stderr.contains("in progress"));
    assert_eq!(get_session_id(root), session_id);
}

#[test]
fn structural_atomic_write_usage_for_core_state_files() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let workspace_src = fs::read_to_string(root.join("src/workspace.rs")).unwrap();
    assert!(workspace_src.contains("atomic_write(&metadata_path"));
    assert!(workspace_src.contains("atomic_write(&items_path"));
    assert!(workspace_src.contains("atomic_write(&active_path"));

    let items_src = fs::read_to_string(root.join("src/store/items.rs")).unwrap();
    assert!(items_src.contains("atomic_write(&items_file"));

    let session_src = fs::read_to_string(root.join("src/store/session.rs")).unwrap();
    assert!(session_src.contains("atomic_write(&session_file"));
    assert!(session_src.contains("remove_file(&active_path"));
}
