use assert_cmd::cargo;
use assert_cmd::prelude::*;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn plumb_binary() -> Command {
    Command::new(cargo::cargo_bin!("plumb"))
}

fn get_session_id(root: &Path) -> String {
    fs::read_to_string(root.join(".plumb/active"))
        .unwrap()
        .trim()
        .to_string()
}

fn baseline_path(root: &Path, session_id: &str, item_id: usize) -> PathBuf {
    root.join(".plumb")
        .join("sessions")
        .join(session_id)
        .join("snapshots")
        .join(format!("{}.baseline", item_id))
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

fn run_restore(root: &Path, target: &str, stdin_input: Option<&str>) -> Output {
    let mut cmd = plumb_binary();
    cmd.current_dir(root)
        .arg("restore")
        .arg(target)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if stdin_input.is_some() {
        cmd.stdin(Stdio::piped());
    }

    let mut child = cmd.spawn().unwrap();
    if let Some(input) = stdin_input {
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(input.as_bytes()).unwrap();
    }

    child.wait_with_output().unwrap()
}

#[test]
fn restore_happy_path_rewrites_file_byte_for_byte() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    let before = b"\x00pre\nline\xff".to_vec();
    let after = b"\x01post\nline\xfe".to_vec();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    fs::write(root.join("a.rs"), &before).unwrap();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("a.rs")
        .assert()
        .success();

    let go_output = go_with_true_editor(root, "1");
    assert!(
        go_output.status.success(),
        "go should succeed: {}",
        String::from_utf8_lossy(&go_output.stderr)
    );

    fs::write(root.join("a.rs"), &after).unwrap();

    let output = run_restore(root, "1", Some("y\n"));
    assert!(
        output.status.success(),
        "restore should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let session_id = get_session_id(root);
    let expected = fs::read(baseline_path(root, &session_id, 1)).unwrap();
    let current = fs::read(root.join("a.rs")).unwrap();
    assert_eq!(current, expected);
}

#[test]
fn restore_declined_confirmation_keeps_file_unchanged() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    fs::write(root.join("a.rs"), "before\n").unwrap();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("a.rs")
        .assert()
        .success();

    let go_output = go_with_true_editor(root, "1");
    assert!(
        go_output.status.success(),
        "go should succeed: {}",
        String::from_utf8_lossy(&go_output.stderr)
    );

    fs::write(root.join("a.rs"), "changed\n").unwrap();

    let output = run_restore(root, "1", Some("\n"));
    assert!(
        output.status.success(),
        "restore cancellation should still exit success"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Restore cancelled."));
    assert_eq!(fs::read_to_string(root.join("a.rs")).unwrap(), "changed\n");
}

#[test]
fn restore_missing_file_errors_and_does_not_recreate_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    fs::write(root.join("a.rs"), "before\n").unwrap();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("a.rs")
        .assert()
        .success();

    let go_output = go_with_true_editor(root, "1");
    assert!(
        go_output.status.success(),
        "go should succeed: {}",
        String::from_utf8_lossy(&go_output.stderr)
    );

    fs::remove_file(root.join("a.rs")).unwrap();

    let output = run_restore(root, "1", None);
    assert!(
        !output.status.success(),
        "restore should fail when destination file is missing"
    );

    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    assert!(stderr.contains("does not exist"));
    assert!(!root.join("a.rs").exists(), "file should remain missing");
}
