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

fn run_plumb(root: &Path, args: &[&str]) -> Output {
    let mut cmd = plumb_binary();
    cmd.current_dir(root);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.output().unwrap()
}

fn run_go_with_true_editor(root: &Path, target: &str) -> Output {
    plumb_binary()
        .current_dir(root)
        .env("EDITOR", "true")
        .arg("go")
        .arg(target)
        .output()
        .unwrap()
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

fn item_state(root: &Path, rel_path: &str) -> String {
    let Value::List(items) = read_items(root) else {
        panic!("items should be list");
    };

    for item in items {
        let Value::Map(map) = item else {
            panic!("item should be map");
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

fn setup_three_file_workspace(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.rs"), "a\n").unwrap();
    fs::write(root.join("src/b.rs"), "b\n").unwrap();
    fs::write(root.join("src/c.rs"), "c\n").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/a.rs")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/b.rs")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/c.rs")
        .assert()
        .success();
}

#[test]
fn workflow_end_to_end_status_go_done_next() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    setup_three_file_workspace(root);

    let status_before = run_plumb(root, &["status"]);
    assert!(status_before.status.success());
    let status_before_out = String::from_utf8_lossy(&status_before.stdout);
    assert!(status_before_out.contains("3 item(s) [TODO]"));
    assert!(status_before_out.contains("0 item(s) [IN_PROGRESS]"));
    assert!(status_before_out.contains("0 item(s) [DONE]"));

    let go1 = run_go_with_true_editor(root, "1");
    assert!(go1.status.success());
    let status_after_go1 = run_plumb(root, &["status"]);
    assert!(status_after_go1.status.success());
    let status_after_go1_out = String::from_utf8_lossy(&status_after_go1.stdout);
    assert!(status_after_go1_out.contains("2 item(s) [TODO]"));
    assert!(status_after_go1_out.contains("1 item(s) [IN_PROGRESS]"));
    assert!(status_after_go1_out.contains("0 item(s) [DONE]"));

    let done1 = run_plumb(root, &["done", "1"]);
    assert!(done1.status.success());
    let status_after_done1 = run_plumb(root, &["status"]);
    assert!(status_after_done1.status.success());
    let status_after_done1_out = String::from_utf8_lossy(&status_after_done1.stdout);
    assert!(status_after_done1_out.contains("2 item(s) [TODO]"));
    assert!(status_after_done1_out.contains("0 item(s) [IN_PROGRESS]"));
    assert!(status_after_done1_out.contains("1 item(s) [DONE]"));

    let next_after_done1 = run_plumb(root, &["next"]);
    assert!(next_after_done1.status.success());
    let next_after_done1_out = String::from_utf8_lossy(&next_after_done1.stdout);
    assert!(next_after_done1_out.contains("src/b.rs"));

    assert!(run_go_with_true_editor(root, "2").status.success());
    assert!(run_plumb(root, &["done", "2"]).status.success());
    let next_after_done2 = run_plumb(root, &["next"]);
    assert!(next_after_done2.status.success());
    let next_after_done2_out = String::from_utf8_lossy(&next_after_done2.stdout);
    assert!(next_after_done2_out.contains("src/c.rs"));

    assert!(run_go_with_true_editor(root, "3").status.success());
    assert!(run_plumb(root, &["done", "3"]).status.success());

    let next_after_done3 = run_plumb(root, &["next"]);
    assert!(
        !next_after_done3.status.success(),
        "next should fail when queue has no todo"
    );
    let stderr = String::from_utf8_lossy(&next_after_done3.stderr);
    assert!(stderr.contains("To Do"));
}

#[test]
fn done_requires_target() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    let output = run_plumb(root, &["done"]);
    assert!(!output.status.success(), "done should require target");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Usage") || stderr.contains("required"),
        "expected clap usage error, got: {}",
        stderr
    );
}

#[test]
fn done_on_todo_fails_and_state_unchanged() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.rs"), "a\n").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/a.rs")
        .assert()
        .success();

    let output = run_plumb(root, &["done", "1"]);
    assert!(!output.status.success(), "done should fail for todo item");
    assert_eq!(item_state(root, "src/a.rs"), "todo");
}

#[test]
fn done_on_in_progress_succeeds_and_state_persisted() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.rs"), "a\n").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/a.rs")
        .assert()
        .success();
    let go = run_go_with_true_editor(root, "1");
    assert!(go.status.success(), "go should set item in_progress");

    let done = run_plumb(root, &["done", "1"]);
    assert!(done.status.success(), "done should succeed for in_progress");
    assert_eq!(item_state(root, "src/a.rs"), "done");
}

#[test]
fn next_outputs_expected_format() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    setup_three_file_workspace(root);

    let output = run_plumb(root, &["next"]);
    assert!(output.status.success(), "next should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Next item:"));
    assert!(stdout.contains("(ID:"));
}

#[test]
fn next_no_todo_is_nonzero_and_clear_message() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.rs"), "a\n").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/a.rs")
        .assert()
        .success();
    assert!(run_go_with_true_editor(root, "1").status.success());
    assert!(run_plumb(root, &["done", "1"]).status.success());

    let output = run_plumb(root, &["next"]);
    assert!(
        !output.status.success(),
        "next should fail when no todo exists"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("To Do"),
        "stderr should mention no To Do: {}",
        stderr
    );
}

#[test]
fn next_deterministic_across_runs_when_items_unchanged() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    setup_three_file_workspace(root);

    let first = run_plumb(root, &["next"]);
    let second = run_plumb(root, &["next"]);

    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn done_by_path_and_done_by_id_both_work() {
    let id_workspace = tempfile::tempdir().unwrap();
    let id_root = id_workspace.path();
    fs::create_dir_all(id_root.join("src")).unwrap();
    fs::write(id_root.join("src/a.rs"), "a\n").unwrap();

    plumb_binary()
        .current_dir(id_root)
        .arg("start")
        .assert()
        .success();
    plumb_binary()
        .current_dir(id_root)
        .arg("add")
        .arg("src/a.rs")
        .assert()
        .success();
    assert!(run_go_with_true_editor(id_root, "1").status.success());
    assert!(run_plumb(id_root, &["done", "1"]).status.success());
    assert_eq!(item_state(id_root, "src/a.rs"), "done");

    let path_workspace = tempfile::tempdir().unwrap();
    let path_root = path_workspace.path();
    fs::create_dir_all(path_root.join("src")).unwrap();
    fs::write(path_root.join("src/a.rs"), "a\n").unwrap();

    plumb_binary()
        .current_dir(path_root)
        .arg("start")
        .assert()
        .success();
    plumb_binary()
        .current_dir(path_root)
        .arg("add")
        .arg("src/a.rs")
        .assert()
        .success();
    assert!(run_go_with_true_editor(path_root, "1").status.success());
    assert!(run_plumb(path_root, &["done", "src/a.rs"]).status.success());
    assert_eq!(item_state(path_root, "src/a.rs"), "done");
}
