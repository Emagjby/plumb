use std::path::{Path, PathBuf};

use atomicwrites::{AtomicFile, OverwriteBehavior};
use strata::{decode::decode, encode::encode, map, string, value::Value};
use thiserror::Error;

use crate::workspace::resolve_workspace_root;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Failed to read the session file\n\n{0}")]
    ReadError(String),
    #[error("Failed to write the session file\n\n{0}")]
    WriteError(String),
    #[error("No active session found")]
    NoActiveSession,
    #[error("Failed to resolve workspace root\n\n{0}")]
    ResolveWorkspaceRootError(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Todo,
    InProgress,
    Done,
}

pub struct Item {
    pub id: String,
    pub rel_path: String,
    pub state: State,
}

pub fn active_session_id(root: &Path) -> Result<String, StoreError> {
    let workspace_root = resolve_workspace_root(root).map_err(|e| {
        StoreError::ResolveWorkspaceRootError(format!("Could not resolve workspace root: {e}"))
    })?;
    let session_file = workspace_root.join(".plumb").join("session");

    let content = std::fs::read_to_string(&session_file)
        .map_err(|e| StoreError::ReadError(format!("Could not read session file: {e}")))?;

    let session_id = content.trim().to_string();
    if session_id.is_empty() {
        Err(StoreError::NoActiveSession)
    } else {
        Ok(session_id)
    }
}

pub fn session_dir(root: &Path, session_id: &str) -> Result<PathBuf, StoreError> {
    let workspace_root = resolve_workspace_root(root).map_err(|e| {
        StoreError::ResolveWorkspaceRootError(format!("Could not resolve workspace root: {e}"))
    })?;

    Ok(workspace_root
        .join(".plumb")
        .join("sessions")
        .join(session_id))
}

pub fn items_path(root: &Path, session_id: &str) -> Result<PathBuf, StoreError> {
    let workspace_root = resolve_workspace_root(root).map_err(|e| {
        StoreError::ResolveWorkspaceRootError(format!("Could not resolve workspace root: {e}"))
    })?;

    Ok(workspace_root
        .join(".plumb")
        .join("sessions")
        .join(session_id)
        .join("items.scb"))
}

pub fn load_items(root: &Path) -> Result<Vec<Item>, StoreError> {
    let session_id = active_session_id(root)?;
    let items_file = items_path(root, &session_id)?;

    if !items_file.exists() {
        return Ok(vec![]);
    }

    let scb = std::fs::read_to_string(&items_file)
        .map_err(|e| StoreError::ReadError(format!("Could not read items file: {e}")))?;

    let scb = scb.trim().as_bytes();
    let decoded =
        decode(scb).map_err(|e| StoreError::ReadError(format!("Failed to decode SCB: {:?}", e)))?;

    let items_list = match decoded {
        Value::List(list) => list,
        _ => {
            return Err(StoreError::ReadError(
                "Expected a list of items".to_string(),
            ));
        }
    };

    let items: Result<Vec<Item>, _> = items_list
        .into_iter()
        .map(|value| {
            let map = match value {
                Value::Map(map) => map,
                _ => {
                    return Err(StoreError::ReadError(
                        "Expected each item to be a map".to_string(),
                    ));
                }
            };

            let id = match map.get("id") {
                Some(Value::String(s)) => s.clone(),
                _ => {
                    return Err(StoreError::ReadError(
                        "Missing or invalid 'id' field".to_string(),
                    ));
                }
            };

            let rel_path = match map.get("rel_path") {
                Some(Value::String(s)) => s.clone(),
                _ => {
                    return Err(StoreError::ReadError(
                        "Missing or invalid 'rel_path' field".to_string(),
                    ));
                }
            };

            let state = match map.get("state") {
                Some(Value::String(s)) => match s.as_str() {
                    "todo" => State::Todo,
                    "in_progress" => State::InProgress,
                    "done" => State::Done,
                    _ => return Err(StoreError::ReadError(format!("Invalid state: {}", s))),
                },
                _ => {
                    return Err(StoreError::ReadError(
                        "Missing or invalid 'state' field".to_string(),
                    ));
                }
            };

            Ok(Item {
                id,
                rel_path,
                state,
            })
        })
        .collect();

    items
}

pub fn save_items(root: &Path, session_id: &str, items: &[Item]) -> Result<(), StoreError> {
    let items_file = items_path(root, session_id)?;

    if let Some(parent) = items_file.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| StoreError::WriteError(format!("Failed to create directory: {}", e)))?;
    }

    let items_value: Value = Value::List(
        items
            .iter()
            .map(|item| {
                let state_str = match item.state {
                    State::Todo => "todo",
                    State::InProgress => "in_progress",
                    State::Done => "done",
                };
                map! {
                    "id" => string!(&item.id),
                    "rel_path" => string!(&item.rel_path),
                    "state" => string!(state_str)
                }
            })
            .collect(),
    );

    let scb = encode(&items_value)
        .map_err(|e| StoreError::WriteError(format!("Failed to encode items: {:?}", e)))?;

    let af = AtomicFile::new(&items_file, OverwriteBehavior::AllowOverwrite);
    af.write(|f| std::io::Write::write_all(&mut std::io::BufWriter::new(f), &scb))
        .map_err(|e| StoreError::WriteError(format!("Failed to atomically write items: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_workspace_with_session(tmp: &TempDir, session_id: &str) -> TempDir {
        let workspace = tempfile::tempdir_in(tmp.path()).unwrap();
        let plumb_dir = workspace.path().join(".plumb");
        fs::create_dir_all(&plumb_dir).unwrap();
        fs::write(plumb_dir.join("session"), session_id).unwrap();
        workspace
    }

    #[test]
    fn test_active_session_id_success() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_workspace_with_session(&tmp, "test-session-123");

        let result = active_session_id(workspace.path()).unwrap();
        assert_eq!(result, "test-session-123");
    }

    #[test]
    fn test_active_session_id_no_session_file() {
        let tmp = TempDir::new().unwrap();
        let workspace = tempfile::tempdir_in(tmp.path()).unwrap();

        let result = active_session_id(workspace.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StoreError::ReadError(_)));
    }

    #[test]
    fn test_active_session_id_empty_session() {
        let tmp = TempDir::new().unwrap();
        let workspace = tempfile::tempdir_in(tmp.path()).unwrap();
        let plumb_dir = workspace.path().join(".plumb");
        fs::create_dir_all(&plumb_dir).unwrap();
        fs::write(plumb_dir.join("session"), "").unwrap();

        let result = active_session_id(workspace.path());
        assert!(matches!(result.unwrap_err(), StoreError::NoActiveSession));
    }

    #[test]
    fn test_load_items_with_items() {
        use strata::{map, string, value::Value};

        let tmp = TempDir::new().unwrap();
        let workspace = create_workspace_with_session(&tmp, "test-session");

        let sessions_dir = workspace
            .path()
            .join(".plumb")
            .join("sessions")
            .join("test-session");
        fs::create_dir_all(&sessions_dir).unwrap();

        let items_value = Value::List(vec![
            map! {
                "id" => string!("1"),
                "rel_path" => string!("src/main.rs"),
                "state" => string!("todo")
            },
            map! {
                "id" => string!("2"),
                "rel_path" => string!("src/lib.rs"),
                "state" => string!("in_progress")
            },
            map! {
                "id" => string!("3"),
                "rel_path" => string!("README.md"),
                "state" => string!("done")
            },
        ]);
        let items_scb = strata::encode::encode(&items_value).unwrap();
        fs::write(sessions_dir.join("items.scb"), items_scb).unwrap();

        let result = load_items(workspace.path()).unwrap();

        assert_eq!(result.len(), 3);

        assert_eq!(result[0].id, "1");
        assert_eq!(result[0].rel_path, "src/main.rs");
        assert_eq!(result[0].state, State::Todo);

        assert_eq!(result[1].id, "2");
        assert_eq!(result[1].rel_path, "src/lib.rs");
        assert_eq!(result[1].state, State::InProgress);

        assert_eq!(result[2].id, "3");
        assert_eq!(result[2].rel_path, "README.md");
        assert_eq!(result[2].state, State::Done);
    }

    #[test]
    fn test_load_items_invalid_scb() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_workspace_with_session(&tmp, "test-session");

        let sessions_dir = workspace
            .path()
            .join(".plumb")
            .join("sessions")
            .join("test-session");
        fs::create_dir_all(&sessions_dir).unwrap();
        fs::write(sessions_dir.join("items.scb"), "invalid scb data").unwrap();

        let result = load_items(workspace.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_save_items_creates_file() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_workspace_with_session(&tmp, "test-session");

        let items = vec![Item {
            id: "1".to_string(),
            rel_path: "src/main.rs".to_string(),
            state: State::Todo,
        }];

        save_items(workspace.path(), "test-session", &items).unwrap();

        let sessions_dir = workspace
            .path()
            .join(".plumb")
            .join("sessions")
            .join("test-session");
        assert!(sessions_dir.join("items.scb").exists());

        let loaded = load_items(workspace.path()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "1");
        assert_eq!(loaded[0].rel_path, "src/main.rs");
        assert_eq!(loaded[0].state, State::Todo);
    }

    #[test]
    fn test_save_items_multiple_items() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_workspace_with_session(&tmp, "test-session");

        let items = vec![
            Item {
                id: "1".to_string(),
                rel_path: "src/main.rs".to_string(),
                state: State::Todo,
            },
            Item {
                id: "2".to_string(),
                rel_path: "src/lib.rs".to_string(),
                state: State::InProgress,
            },
            Item {
                id: "3".to_string(),
                rel_path: "README.md".to_string(),
                state: State::Done,
            },
        ];

        save_items(workspace.path(), "test-session", &items).unwrap();

        let loaded = load_items(workspace.path()).unwrap();
        assert_eq!(loaded.len(), 3);

        assert_eq!(loaded[0].id, "1");
        assert_eq!(loaded[0].state, State::Todo);
        assert_eq!(loaded[1].id, "2");
        assert_eq!(loaded[1].state, State::InProgress);
        assert_eq!(loaded[2].id, "3");
        assert_eq!(loaded[2].state, State::Done);
    }

    #[test]
    fn test_save_items_empty_list() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_workspace_with_session(&tmp, "test-session");

        save_items(workspace.path(), "test-session", &[]).unwrap();

        let sessions_dir = workspace
            .path()
            .join(".plumb")
            .join("sessions")
            .join("test-session");
        assert!(sessions_dir.join("items.scb").exists());

        let loaded = load_items(workspace.path()).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_save_items_overwrites_existing() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_workspace_with_session(&tmp, "test-session");

        let items1 = vec![Item {
            id: "old".to_string(),
            rel_path: "old.rs".to_string(),
            state: State::Todo,
        }];
        save_items(workspace.path(), "test-session", &items1).unwrap();

        let items2 = vec![Item {
            id: "new".to_string(),
            rel_path: "new.rs".to_string(),
            state: State::Done,
        }];
        save_items(workspace.path(), "test-session", &items2).unwrap();

        let loaded = load_items(workspace.path()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "new");
        assert_eq!(loaded[0].state, State::Done);
    }
}
