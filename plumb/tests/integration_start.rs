use assert_cmd::cargo;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use strata::decode::decode;
use strata::value::Value;

fn plumb_binary() -> Command {
    Command::new(cargo::cargo_bin!("plumb"))
}

#[test]
fn start_creates_active_and_session_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join(".plumb")).unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    assert!(
        root.join(".plumb/active").is_file(),
        ".plumb/active should exist"
    );

    let sessions_dir = root.join(".plumb/sessions");
    assert!(sessions_dir.is_dir(), ".plumb/sessions should exist");

    let session_dirs: Vec<_> = fs::read_dir(sessions_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    assert_eq!(
        session_dirs.len(),
        1,
        "should have exactly one session directory"
    );

    let session_dir = &session_dirs[0].path();
    assert!(
        session_dir.join("session.scb").is_file(),
        "session.scb should exist"
    );
    assert!(
        session_dir.join("snapshots").is_dir(),
        "snapshot directory should exist"
    );
}

#[test]
fn start_creates_session_from_cold_start() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .success();

    assert!(
        root.join(".plumb/active").is_file(),
        ".plumb/active should exist"
    );

    let sessions_dir = root.join(".plumb/sessions");
    assert!(sessions_dir.is_dir(), ".plumb/sessions should exist");

    let session_dirs: Vec<_> = fs::read_dir(sessions_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    assert_eq!(
        session_dirs.len(),
        1,
        "should have exactly one session directory"
    );

    let session_dir = &session_dirs[0].path();
    assert!(
        session_dir.join("session.scb").is_file(),
        "session.scb should exist"
    );
    assert!(
        session_dir.join("snapshots").is_dir(),
        "snapshot directory should exist"
    );
}

#[test]
fn start_creates_named_session() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .arg("my-session")
        .assert()
        .success();

    let sessions_dir = root.join(".plumb/sessions");
    assert!(sessions_dir.is_dir(), ".plumb/sessions should exist");

    let session_dirs: Vec<_> = fs::read_dir(sessions_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    assert_eq!(
        session_dirs.len(),
        1,
        "should have exactly one session directory"
    );

    let session_dir = &session_dirs[0].path();
    let session_scb_path = session_dir.join("session.scb");
    assert!(session_scb_path.is_file(), "session.scb should exist");

    let session_data = fs::read(&session_scb_path).unwrap();
    let decoded: Value = decode(&session_data).unwrap();
    if let Value::Map(session_map) = &decoded {
        if let Some(name_value) = session_map.get("name") {
            if let Value::String(name) = name_value {
                assert!(
                    name.contains("my-session"),
                    "session.scb should contain the session name 'my-session'"
                );
            } else {
                panic!("name field should be a string");
            }
        } else {
            panic!("session.scb should contain a 'name' field");
        }
    } else {
        panic!("session.scb should decode to a map");
    }
}

#[test]
fn start_refuses_when_active_session_exists() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    fs::create_dir_all(root.join(".plumb/sessions")).unwrap();
    fs::write(root.join(".plumb/active"), "deadbeef").unwrap();

    plumb_binary()
        .current_dir(root)
        .arg("start")
        .assert()
        .failure()
        .stderr(predicate::str::contains("active session"));
}

#[test]
fn running_from_nested_folder_uses_nearest_parent_root() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    let nested = root.join("project").join("src");
    let nested_path = nested.clone();

    fs::create_dir_all(&nested).unwrap();
    fs::create_dir_all(root.join(".plumb")).unwrap();

    plumb_binary()
        .current_dir(&nested)
        .arg("start")
        .assert()
        .success();

    assert!(
        root.join(".plumb/active").is_file(),
        ".plumb/active should be in parent root"
    );
    assert!(
        !nested_path.join(".plumb").exists(),
        ".plumb should NOT be created in nested folder"
    );
}

#[test]
fn running_from_nested_folder_with_own_plumb_uses_nested_root() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    let nested = root.join("project").join("src");
    let nested_path = nested.clone();

    fs::create_dir_all(&nested).unwrap();
    fs::create_dir_all(root.join(".plumb")).unwrap();
    fs::create_dir_all(nested.join(".plumb")).unwrap();

    plumb_binary()
        .current_dir(&nested)
        .arg("start")
        .assert()
        .success();

    assert!(
        nested_path.join(".plumb/active").is_file(),
        ".plumb/active should be in nested root"
    );
    assert!(
        !root.join(".plumb/active").is_file(),
        ".plumb/active should NOT be in parent root"
    );
}
