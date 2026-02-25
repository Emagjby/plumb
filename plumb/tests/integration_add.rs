use assert_cmd::cargo;
use assert_cmd::prelude::*;
use std::fs;
use std::process::Command;
use strata::decode::decode;
use strata::value::Value;

fn plumb_binary() -> Command {
    Command::new(cargo::cargo_bin!("plumb"))
}

fn get_session_id(root: &std::path::Path) -> String {
    fs::read_to_string(root.join(".plumb/active"))
        .unwrap()
        .trim()
        .to_string()
}

fn read_items(root: &std::path::Path) -> Value {
    let session_id = get_session_id(root);
    let items_path = root
        .join(".plumb")
        .join("sessions")
        .join(&session_id)
        .join("items.scb");
    let data = fs::read(&items_path).unwrap();
    decode(&data).unwrap()
}

fn item_ids_and_paths(items_value: Value) -> (Vec<i64>, Vec<String>) {
    let Value::List(items) = items_value else {
        panic!("items should be a list");
    };

    let mut ids = Vec::new();
    let mut paths = Vec::new();

    for item in items {
        let Value::Map(m) = item else {
            panic!("item should be a map");
        };

        let Value::Int(id) = m.get("id").unwrap() else {
            panic!("id should be an int");
        };
        let Value::String(path) = m.get("rel_path").unwrap() else {
            panic!("rel_path should be a string");
        };

        ids.push(*id);
        paths.push(path.clone());
    }

    (ids, paths)
}

#[test]
fn add_creates_items_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    let file1 = root.join("file1.rs");
    fs::write(&file1, "").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("file1.rs")
        .assert()
        .success();

    let session_id = get_session_id(root);
    let items_path = root
        .join(".plumb")
        .join("sessions")
        .join(&session_id)
        .join("items.scb");
    assert!(
        items_path.is_file(),
        "items.scb should exist at .plumb/sessions/{}/items.scb",
        session_id
    );
}

#[test]
fn add_persists_items_with_correct_ids_and_paths() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    let file1 = root.join("src").join("main.rs");
    let file2 = root.join("src").join("lib.rs");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(&file1, "").unwrap();
    fs::write(&file2, "").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/main.rs")
        .assert()
        .success();

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("src/lib.rs")
        .assert()
        .success();

    let items_value = read_items(root);
    let Value::List(items) = items_value else {
        panic!("items should be a list");
    };

    assert_eq!(items.len(), 2, "should have 2 items");

    let first = &items[0];
    let Value::Map(first_map) = first else {
        panic!("first item should be a map");
    };

    let id1 = first_map.get("id").unwrap();
    let Value::Int(id1_val) = id1 else {
        panic!("id should be an int");
    };
    assert_eq!(*id1_val, 1, "first item id should be 1");

    let rel_path1 = first_map.get("rel_path").unwrap();
    let Value::String(rel_path1_str) = rel_path1 else {
        panic!("rel_path should be a string");
    };
    assert_eq!(
        rel_path1_str, "src/main.rs",
        "rel_path should be relative, not absolute"
    );

    let second = &items[1];
    let Value::Map(second_map) = second else {
        panic!("second item should be a map");
    };

    let id2 = second_map.get("id").unwrap();
    let Value::Int(id2_val) = id2 else {
        panic!("id should be an int");
    };
    assert_eq!(*id2_val, 2, "second item id should be 2");

    let rel_path2 = second_map.get("rel_path").unwrap();
    let Value::String(rel_path2_str) = rel_path2 else {
        panic!("rel_path should be a string");
    };
    assert_eq!(
        rel_path2_str, "src/lib.rs",
        "rel_path should be relative, not absolute"
    );
}

#[test]
fn add_detects_duplicate_with_notice() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    let file1 = root.join("file1.rs");
    fs::write(&file1, "").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("file1.rs")
        .assert()
        .success();

    let Value::List(items_before) = read_items(root) else {
        panic!("items should be a list");
    };
    let count_before = items_before.len();

    let output = plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("./file1.rs")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("already"),
        "stderr should contain notice about duplicate"
    );

    let Value::List(items_after) = read_items(root) else {
        panic!("items should be a list");
    };
    assert_eq!(
        items_after.len(),
        count_before,
        "item count should remain unchanged after duplicate add"
    );
}

#[test]
fn add_normalizes_dot_prefix() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    let file1 = root.join("file1.rs");
    fs::write(&file1, "").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("./file1.rs")
        .assert()
        .success();

    let items_value = read_items(root);
    let Value::List(items) = items_value else {
        panic!("items should be a list");
    };

    assert_eq!(items.len(), 1);

    let first = &items[0];
    let Value::Map(first_map) = first else {
        panic!("item should be a map");
    };

    let rel_path = first_map.get("rel_path").unwrap();
    let Value::String(rel_path_str) = rel_path else {
        panic!("rel_path should be a string");
    };
    assert_eq!(rel_path_str, "file1.rs", "dot prefix should be normalized");
}

#[test]
fn status_shows_correct_counts() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    let file1 = root.join("file1.rs");
    let file2 = root.join("file2.rs");
    fs::write(&file1, "").unwrap();
    fs::write(&file2, "").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("file1.rs")
        .assert()
        .success();

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("file2.rs")
        .assert()
        .success();

    let output = plumb_binary()
        .current_dir(root)
        .arg("status")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("2 item(s) [TODO]"),
        "status should show 2 todo items, got: {}",
        stdout
    );
    assert!(
        stdout.contains("0 item(s) [IN_PROGRESS]"),
        "status should show 0 in progress items, got: {}",
        stdout
    );
    assert!(
        stdout.contains("0 item(s) [DONE]"),
        "status should show 0 done items, got: {}",
        stdout
    );
}

#[test]
fn add_folder_happy_path_excludes_dirs_and_orders_lexicographically() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    fs::create_dir_all(root.join("src/nested")).unwrap();
    fs::create_dir_all(root.join("src/.git")).unwrap();
    fs::create_dir_all(root.join("src/node_modules/pkg")).unwrap();
    fs::create_dir_all(root.join("src/target/build")).unwrap();
    fs::create_dir_all(root.join("src/.plumb/cache")).unwrap();

    fs::write(root.join("src/z.rs"), "").unwrap();
    fs::write(root.join("src/a.rs"), "").unwrap();
    fs::write(root.join("src/nested/m.rs"), "").unwrap();
    fs::write(root.join("src/.git/ignored.rs"), "").unwrap();
    fs::write(root.join("src/node_modules/pkg/ignored.js"), "").unwrap();
    fs::write(root.join("src/target/build/ignored.rs"), "").unwrap();
    fs::write(root.join("src/.plumb/cache/ignored.txt"), "").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("-f")
        .arg("src")
        .assert()
        .success();

    let (ids, paths) = item_ids_and_paths(read_items(root));
    assert_eq!(ids, vec![1, 2, 3]);
    assert_eq!(
        paths,
        vec![
            "src/a.rs".to_string(),
            "src/nested/m.rs".to_string(),
            "src/z.rs".to_string()
        ]
    );
}

#[test]
fn add_folder_twice_is_deterministic_and_noop_on_second_run() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    fs::create_dir_all(root.join("src/deep")).unwrap();
    fs::write(root.join("src/b.rs"), "").unwrap();
    fs::write(root.join("src/deep/a.rs"), "").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("-f")
        .arg("src")
        .assert()
        .success();

    let first = item_ids_and_paths(read_items(root));

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("-f")
        .arg("src")
        .assert()
        .success();

    let second = item_ids_and_paths(read_items(root));

    assert_eq!(first, second, "second folder add should be a no-op");
}

#[test]
fn add_folder_id_continuation_after_single_file_add() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    fs::write(root.join("first.rs"), "").unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/b.rs"), "").unwrap();
    fs::write(root.join("src/a.rs"), "").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("first.rs")
        .assert()
        .success();

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("-f")
        .arg("src")
        .assert()
        .success();

    let (ids, paths) = item_ids_and_paths(read_items(root));
    assert_eq!(ids, vec![1, 2, 3]);
    assert_eq!(
        paths,
        vec![
            "first.rs".to_string(),
            "src/a.rs".to_string(),
            "src/b.rs".to_string()
        ]
    );
}

#[test]
fn add_folder_requires_active_session() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.rs"), "").unwrap();

    let output = plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("-f")
        .arg("src")
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "should fail without active session"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr
            .to_lowercase()
            .contains("use `plumb start` to start a session"),
        "expected rm/status-style no-active-session message, got: {}",
        stderr
    );
    assert!(
        stderr.to_lowercase().contains("no active session found"),
        "expected no active session message, got: {}",
        stderr
    );
    assert!(
        !stderr
            .to_lowercase()
            .contains("failed to read the active file"),
        "expected active-session related error, got: {}",
        stderr
    );
}

#[test]
fn add_folder_with_only_excluded_files_adds_nothing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    fs::create_dir_all(root.join("batch/.git")).unwrap();
    fs::create_dir_all(root.join("batch/node_modules/pkg")).unwrap();
    fs::create_dir_all(root.join("batch/target/build")).unwrap();
    fs::create_dir_all(root.join("batch/.plumb/cache")).unwrap();
    fs::write(root.join("batch/.git/skip.txt"), "").unwrap();
    fs::write(root.join("batch/node_modules/pkg/skip.js"), "").unwrap();
    fs::write(root.join("batch/target/build/skip.rs"), "").unwrap();
    fs::write(root.join("batch/.plumb/cache/skip.txt"), "").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("-f")
        .arg("batch")
        .assert()
        .success();

    let (ids, paths) = item_ids_and_paths(read_items(root));
    assert!(ids.is_empty());
    assert!(paths.is_empty());
}

#[test]
fn add_folder_dot_path_behavior_is_defined() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
    fs::create_dir_all(root.join("target/build")).unwrap();

    fs::write(root.join("root.rs"), "").unwrap();
    fs::write(root.join("src/lib.rs"), "").unwrap();
    fs::write(root.join(".git/ignored.txt"), "").unwrap();
    fs::write(root.join("node_modules/pkg/ignored.js"), "").unwrap();
    fs::write(root.join("target/build/ignored.rs"), "").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("-f")
        .arg(".")
        .assert()
        .success();

    let (ids, paths) = item_ids_and_paths(read_items(root));
    assert_eq!(ids, vec![1, 2]);
    assert_eq!(paths, vec!["root.rs".to_string(), "src/lib.rs".to_string()]);
}
