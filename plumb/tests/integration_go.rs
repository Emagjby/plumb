use assert_cmd::cargo;
use assert_cmd::prelude::*;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
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

fn read_items(root: &Path) -> Value {
    let session_id = get_session_id(root);
    let items_path = root
        .join(".plumb")
        .join("sessions")
        .join(&session_id)
        .join("items.scb");
    let data = fs::read(&items_path).unwrap();
    decode(&data).unwrap()
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

fn item_state(root: &Path, rel_path: &str) -> String {
    let Value::List(items) = read_items(root) else {
        panic!("items should be a list");
    };

    for item in items {
        let Value::Map(map) = item else {
            panic!("item should be a map");
        };

        let Value::String(path) = map.get("rel_path").unwrap() else {
            panic!("rel_path should be string");
        };
        if path == rel_path {
            let Value::String(state) = map.get("state").unwrap() else {
                panic!("state should be string");
            };
            return state.clone();
        }
    }

    panic!("item not found: {}", rel_path);
}

fn baseline_path(root: &Path, session_id: &str, item_id: usize) -> PathBuf {
    root.join(".plumb")
        .join("sessions")
        .join(session_id)
        .join("snapshots")
        .join(format!("{}.baseline", item_id))
}

#[test]
fn go_baseline_capture_matches_bytes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    let source_path = root.join("a.rs");
    fs::write(&source_path, b"fn main() {\n    println!(\"hello\");\n}\n").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("a.rs")
        .assert()
        .success();

    let output = go_with_true_editor(root, "1");
    assert!(
        output.status.success(),
        "go should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let session_id = get_session_id(root);
    let baseline = fs::read(baseline_path(root, &session_id, 1)).unwrap();
    let source = fs::read(source_path).unwrap();
    assert_eq!(baseline, source);
}

#[test]
fn go_marks_item_in_progress_and_persists() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    fs::write(root.join("tracked.rs"), "let a = 1;\n").unwrap();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("tracked.rs")
        .assert()
        .success();

    let output = go_with_true_editor(root, "1");
    assert!(
        output.status.success(),
        "go should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(item_state(root, "tracked.rs"), "in_progress");
}

#[test]
fn go_missing_file_fails_leaves_todo_and_no_baseline_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    fs::write(root.join("a.rs"), "a").unwrap();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("a.rs")
        .assert()
        .success();

    fs::remove_file(root.join("a.rs")).unwrap();
    let output = go_with_true_editor(root, "1");
    assert!(
        !output.status.success(),
        "go should fail when file is missing"
    );

    assert_eq!(item_state(root, "a.rs"), "todo");
    let session_id = get_session_id(root);
    assert!(!baseline_path(root, &session_id, 1).exists());
}

#[test]
#[cfg(unix)]
fn go_unreadable_file_fails_leaves_todo_and_no_baseline_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    let source_path = root.join("secret.rs");
    fs::write(&source_path, "secret").unwrap();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("secret.rs")
        .assert()
        .success();

    let mut perms = fs::metadata(&source_path).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&source_path, perms).unwrap();

    let output = go_with_true_editor(root, "1");

    let mut restore = fs::metadata(&source_path).unwrap().permissions();
    restore.set_mode(0o644);
    fs::set_permissions(&source_path, restore).unwrap();

    assert!(
        !output.status.success(),
        "go should fail for unreadable files"
    );
    assert_eq!(item_state(root, "secret.rs"), "todo");

    let session_id = get_session_id(root);
    assert!(!baseline_path(root, &session_id, 1).exists());
}

#[test]
fn go_with_editor_env_runs_noninteractive_editor() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    fs::write(root.join("a.rs"), "hello").unwrap();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("a.rs")
        .assert()
        .success();

    let output = go_with_true_editor(root, "1");
    assert!(
        output.status.success(),
        "go should succeed with EDITOR=true: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn go_item_already_in_progress_reopens_editor_and_does_not_create_new_baseline() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    let source_path = root.join("a.rs");
    fs::write(&source_path, "first contents").unwrap();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("a.rs")
        .assert()
        .success();

    let first_go = go_with_true_editor(root, "1");
    assert!(
        first_go.status.success(),
        "first go should succeed: {}",
        String::from_utf8_lossy(&first_go.stderr)
    );

    let session_id = get_session_id(root);
    let baseline = baseline_path(root, &session_id, 1);
    let before = fs::read(&baseline).unwrap();

    fs::write(&source_path, "second contents").unwrap();
    let second_go = go_with_true_editor(root, "1");
    assert!(
        second_go.status.success(),
        "second go should reopen and succeed: {}",
        String::from_utf8_lossy(&second_go.stderr)
    );

    let after = fs::read(&baseline).unwrap();
    assert_eq!(before, after, "baseline should not be recaptured");
}

#[test]
fn go_without_active_session_fails_with_clear_message() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::write(root.join("a.rs"), "hello").unwrap();
    let output = go_with_true_editor(root, "1");

    assert!(!output.status.success(), "go should fail without a session");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no active session found"),
        "expected no-active-session message, got: {}",
        stderr
    );
    assert!(
        stderr.contains("plumb start"),
        "expected guidance to run plumb start, got: {}",
        stderr
    );
}
