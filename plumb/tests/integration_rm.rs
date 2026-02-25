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
fn rm_by_path_keeps_existing_ids_and_autoincrements() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    fs::write(root.join("a.rs"), "").unwrap();
    fs::write(root.join("b.rs"), "").unwrap();
    fs::write(root.join("c.rs"), "").unwrap();

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
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("c.rs")
        .assert()
        .success();

    plumb_binary()
        .current_dir(root)
        .arg("rm")
        .arg("b.rs")
        .assert()
        .success();

    let (ids, paths) = item_ids_and_paths(read_items(root));
    assert_eq!(ids, vec![1, 3]);
    assert_eq!(paths, vec!["a.rs".to_string(), "c.rs".to_string()]);

    fs::write(root.join("d.rs"), "").unwrap();
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("d.rs")
        .assert()
        .success();

    let (ids, _) = item_ids_and_paths(read_items(root));
    assert_eq!(ids, vec![1, 3, 4]);
}

#[test]
fn rm_by_id_keeps_existing_ids() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    fs::write(root.join("a.rs"), "").unwrap();
    fs::write(root.join("b.rs"), "").unwrap();
    fs::write(root.join("c.rs"), "").unwrap();

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
    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("c.rs")
        .assert()
        .success();

    plumb_binary()
        .current_dir(root)
        .arg("rm")
        .arg("2")
        .assert()
        .success();

    let (ids, paths) = item_ids_and_paths(read_items(root));
    assert_eq!(ids, vec![1, 3]);
    assert_eq!(paths, vec!["a.rs".to_string(), "c.rs".to_string()]);
}

#[test]
fn rm_removes_done_item_and_deletes_baseline_snapshot() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    let session_id = get_session_id(root);

    fs::write(root.join("done.rs"), "").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("done.rs")
        .assert()
        .success();

    let session_dir = root.join(".plumb").join("sessions").join(&session_id);
    let items_path = session_dir.join("items.scb");

    use strata::{encode::encode, map, string, value::Value};
    let modified_items = Value::List(vec![map! {
        "id" => strata::int!(1_i64),
        "rel_path" => string!("done.rs"),
        "state" => string!("done")
    }]);
    let encoded = encode(&modified_items).unwrap();
    fs::write(&items_path, encoded).unwrap();

    let snapshots_dir = session_dir.join("snapshots");
    fs::create_dir_all(&snapshots_dir).unwrap();

    let baseline_path = snapshots_dir.join("1.baseline");
    fs::write(&baseline_path, "original content").unwrap();
    assert!(
        baseline_path.exists(),
        "baseline snapshot should exist before rm"
    );

    plumb_binary()
        .current_dir(root)
        .arg("rm")
        .arg("done.rs")
        .assert()
        .success();

    assert!(
        !baseline_path.exists(),
        "baseline snapshot should be deleted after rm"
    );

    let items_value = read_items(root);
    let Value::List(items) = items_value else {
        panic!("items should be a list");
    };
    assert!(items.is_empty(), "item should be removed");
}

#[test]
fn rm_refuses_to_remove_in_progress_item() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    let session_id = get_session_id(root);

    fs::write(root.join("inprogress.rs"), "").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("add")
        .arg("inprogress.rs")
        .assert()
        .success();

    let session_dir = root.join(".plumb").join("sessions").join(&session_id);
    let items_path = session_dir.join("items.scb");

    use strata::{encode::encode, map, string, value::Value};
    let modified_items = Value::List(vec![map! {
        "id" => strata::int!(1_i64),
        "rel_path" => string!("inprogress.rs"),
        "state" => string!("in_progress")
    }]);
    let encoded = encode(&modified_items).unwrap();
    fs::write(&items_path, encoded).unwrap();

    let output = plumb_binary()
        .current_dir(root)
        .arg("rm")
        .arg("inprogress.rs")
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "rm should fail for in-progress items"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("in progress"),
        "error message should mention in progress: {}",
        stderr
    );

    let items_value = read_items(root);
    let Value::List(items) = items_value else {
        panic!("items should be a list");
    };
    assert_eq!(items.len(), 1, "item should still exist after failed rm");
}

#[test]
fn rm_accepts_id_and_path_via_shared_resolve_item() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.rs"), "").unwrap();
    fs::write(root.join("src/b.rs"), "").unwrap();

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
        .arg("rm")
        .arg("1")
        .assert()
        .success();

    plumb_binary()
        .current_dir(root)
        .arg("rm")
        .arg("src/b.rs")
        .assert()
        .success();

    let Value::List(items) = read_items(root) else {
        panic!("items should be a list");
    };
    assert!(
        items.is_empty(),
        "both id and path forms should resolve and remove items"
    );
}
