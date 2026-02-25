use assert_cmd::cargo;
use assert_cmd::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use strata::{encode::encode, map, string, value::Value};

fn plumb_binary() -> Command {
    Command::new(cargo::cargo_bin!("plumb"))
}

fn get_session_id(root: &Path) -> String {
    fs::read_to_string(root.join(".plumb/active"))
        .unwrap()
        .trim()
        .to_string()
}

fn snapshots_dir(root: &Path) -> PathBuf {
    root.join(".plumb")
        .join("sessions")
        .join(get_session_id(root))
        .join("snapshots")
}

fn items_path(root: &Path) -> PathBuf {
    root.join(".plumb")
        .join("sessions")
        .join(get_session_id(root))
        .join("items.scb")
}

fn go_with_true_editor(root: &Path, target: &str) {
    let output = plumb_binary()
        .current_dir(root)
        .env("EDITOR", "true")
        .arg("go")
        .arg(target)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "go should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_diff(root: &Path, target: Option<&str>) -> std::process::Output {
    let mut cmd = plumb_binary();
    cmd.current_dir(root).arg("diff");
    if let Some(target) = target {
        cmd.arg(target);
    }
    cmd.output().unwrap()
}

fn write_items_state(root: &Path, items: Vec<(i64, &str, &str)>) {
    let items_value = Value::List(
        items
            .into_iter()
            .map(|(id, rel_path, state)| {
                map! {
                    "id" => strata::int!(id),
                    "rel_path" => string!(rel_path),
                    "state" => string!(state),
                }
            })
            .collect(),
    );
    let encoded = encode(&items_value).unwrap();
    fs::write(items_path(root), encoded).unwrap();
}

fn fixture(name: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("diff")
            .join(name),
    )
    .unwrap()
}

#[test]
fn diff_one_line_mod_hunk_contains_change() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.txt"), "hello\nworld\n").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/a.txt")
        .assert()
        .success();
    go_with_true_editor(root, "1");

    fs::write(root.join("src/a.txt"), "hello\nWORLD\n").unwrap();
    let output = run_diff(root, Some("1"));
    assert!(output.status.success(), "diff should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("@@"), "stdout should contain hunk header");
    assert!(
        stdout.contains("-world"),
        "stdout should contain removed line"
    );
    assert!(
        stdout.contains("+WORLD"),
        "stdout should contain added line"
    );
}

#[test]
fn diff_deletion_missing_current_treated_as_empty() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.txt"), "alpha\nbeta\n").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/a.txt")
        .assert()
        .success();
    go_with_true_editor(root, "1");

    fs::remove_file(root.join("src/a.txt")).unwrap();
    let output = run_diff(root, Some("1"));
    assert!(
        output.status.success(),
        "diff should succeed when current file is missing"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-alpha"));
    assert!(stdout.contains("-beta"));
}

#[test]
fn diff_errors_helpfully_when_baseline_missing_without_go() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.txt"), "hello\n").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/a.txt")
        .assert()
        .success();

    let output = run_diff(root, Some("1"));
    assert!(!output.status.success(), "diff should fail for todo item");

    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    assert!(stderr.contains("baseline"));
    assert!(stderr.contains("go"));
}

#[test]
fn diff_no_arg_with_no_in_progress_is_success_and_empty_output() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.txt"), "hello\n").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/a.txt")
        .assert()
        .success();

    let output = run_diff(root, None);
    assert!(
        output.status.success(),
        "diff without target should succeed"
    );
    assert!(
        output.stdout.is_empty(),
        "stdout should be empty when no item is in_progress"
    );
}

#[test]
fn diff_no_arg_with_two_in_progress_prints_two_headers() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.txt"), "a-new\n").unwrap();
    fs::write(root.join("src/b.txt"), "b-new\n").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/a.txt")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/b.txt")
        .assert()
        .success();

    write_items_state(
        root,
        vec![
            (1, "src/a.txt", "in_progress"),
            (2, "src/b.txt", "in_progress"),
        ],
    );
    let snapshots = snapshots_dir(root);
    fs::write(snapshots.join("1.baseline"), "a-old\n").unwrap();
    fs::write(snapshots.join("2.baseline"), "b-old\n").unwrap();

    let output = run_diff(root, None);
    assert!(
        output.status.success(),
        "diff without target should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let headers = stdout.matches("--- baseline:").count();
    assert_eq!(headers, 2, "expected two baseline headers, got: {}", stdout);
}

#[test]
fn diff_by_id_and_by_path_match() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.txt"), "hello\nworld\n").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/a.txt")
        .assert()
        .success();
    go_with_true_editor(root, "1");

    fs::write(root.join("src/a.txt"), "hello\nWORLD\n").unwrap();
    let by_id = run_diff(root, Some("1"));
    let by_path = run_diff(root, Some("src/a.txt"));

    assert!(by_id.status.success());
    assert!(by_path.status.success());
    assert_eq!(by_id.stdout, by_path.stdout);
}

#[test]
fn diff_binary_file_prints_binary_message() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.bin"), [0xff, 0x00, 0x01]).unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/a.bin")
        .assert()
        .success();
    go_with_true_editor(root, "1");

    fs::write(root.join("src/a.bin"), [0xff, 0x00, 0x02]).unwrap();
    let output = run_diff(root, Some("1"));
    assert!(output.status.success(), "binary diff should still succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Binary files differ: src/a.bin"));
}

#[test]
fn golden_one_line_mod_diff_matches_fixture() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.txt"), "hello\nworld\n").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/a.txt")
        .assert()
        .success();
    go_with_true_editor(root, "1");

    fs::write(root.join("src/a.txt"), "hello\nWORLD\n").unwrap();
    let output = run_diff(root, Some("1"));
    assert!(output.status.success());

    let actual = String::from_utf8(output.stdout).unwrap();
    let expected = fixture("one_line_mod.diff");
    assert_eq!(actual, expected);
}

#[test]
fn golden_deletion_diff_matches_fixture() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.txt"), "alpha\nbeta\n").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/a.txt")
        .assert()
        .success();
    go_with_true_editor(root, "1");

    fs::remove_file(root.join("src/a.txt")).unwrap();
    let output = run_diff(root, Some("1"));
    assert!(output.status.success());

    let actual = String::from_utf8(output.stdout).unwrap();
    let expected = fixture("deletion.diff");
    assert_eq!(actual, expected);
}
