use std::path::Path;

use strata::{decode::decode, encode::encode, value::Value};

use crate::fs::atomic_write;

use super::items::{StoreError, session_dir};

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub name: String,
    pub created_at_nanos: i128,
}

pub fn close_session(root: &Path, session_id: &str) -> Result<(), StoreError> {
    mark_session_finished(root, session_id)?;
    clear_active_session(root, session_id)?;
    Ok(())
}

fn mark_session_finished(root: &Path, session_id: &str) -> Result<(), StoreError> {
    let dir = session_dir(root, session_id)?;
    let session_file = dir.join("session.scb");

    let raw = std::fs::read(&session_file)
        .map_err(|e| StoreError::ReadError(format!("Could not read session.scb: {e}")))?;

    let decoded = decode(&raw)
        .map_err(|e| StoreError::ReadError(format!("Failed to decode session.scb: {:?}", e)))?;

    let mut map = match decoded {
        Value::Map(m) => m,
        _ => {
            return Err(StoreError::ReadError(
                "session.scb: expected a map".to_string(),
            ));
        }
    };

    map.insert("status".to_string(), Value::String("finished".to_string()));

    let updated = encode(&Value::Map(map))
        .map_err(|e| StoreError::WriteError(format!("Failed to encode session.scb: {:?}", e)))?;

    atomic_write(&session_file, &updated)
        .map_err(|e| StoreError::WriteError(format!("Failed to write session.scb: {e}")))?;

    Ok(())
}

fn clear_active_session(root: &Path, session_id: &str) -> Result<(), StoreError> {
    let active_path = root.join(".plumb").join("active");
    if !active_path.is_file() {
        return Ok(());
    }

    let active_id = std::fs::read_to_string(&active_path)
        .map_err(|e| StoreError::ReadError(format!("Could not read active file: {e}")))?;

    if active_id.trim() == session_id {
        atomic_write(&active_path, b"")
            .map_err(|e| StoreError::WriteError(format!("Failed to clear active file: {e}")))?;
    }

    Ok(())
}

pub fn load_session(root: &Path, session_id: &str) -> Result<Session, StoreError> {
    let dir = session_dir(root, session_id)?;
    let session_file = dir.join("session.scb");

    let raw = std::fs::read(&session_file)
        .map_err(|e| StoreError::ReadError(format!("Could not read session.scb: {e}")))?;

    let decoded = decode(&raw)
        .map_err(|e| StoreError::ReadError(format!("Failed to decode session.scb: {:?}", e)))?;

    let map = match decoded {
        Value::Map(m) => m,
        _ => {
            return Err(StoreError::ReadError(
                "session.scb: expected a map".to_string(),
            ));
        }
    };

    let session_id = match map.get("session_id") {
        Some(Value::Bytes(b)) => String::from_utf8(b.clone()).map_err(|e| {
            StoreError::ReadError(format!("session.scb: invalid session_id bytes: {e}"))
        })?,
        _ => {
            return Err(StoreError::ReadError(
                "session.scb: missing or invalid 'session_id' field".to_string(),
            ));
        }
    };

    let name = match map.get("name") {
        Some(Value::String(s)) => s.clone(),
        None => String::new(),
        _ => {
            return Err(StoreError::ReadError(
                "session.scb: missing or invalid 'name' field".to_string(),
            ));
        }
    };

    let created_at_nanos = match map.get("created_at") {
        Some(Value::Bytes(b)) => {
            let arr: [u8; 16] = b.as_slice().try_into().map_err(|_| {
                StoreError::ReadError(
                    "session.scb: 'created_at' must be exactly 16 bytes".to_string(),
                )
            })?;
            i128::from_le_bytes(arr)
        }
        _ => {
            return Err(StoreError::ReadError(
                "session.scb: missing or invalid 'created_at' field".to_string(),
            ));
        }
    };

    Ok(Session {
        session_id,
        name,
        created_at_nanos,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use strata::{bytes, encode::encode, map, string};
    use tempfile::TempDir;

    fn create_workspace_with_session_scb(
        tmp: &TempDir,
        session_id: &str,
        name: &str,
        created_at_nanos: i128,
    ) -> TempDir {
        let workspace = tempfile::tempdir_in(tmp.path()).unwrap();
        let plumb_dir = workspace.path().join(".plumb");
        fs::create_dir_all(&plumb_dir).unwrap();
        fs::write(plumb_dir.join("active"), session_id).unwrap();

        let session_dir = plumb_dir.join("sessions").join(session_id);
        fs::create_dir_all(&session_dir).unwrap();

        let payload = map! {
            "session_id" => bytes!(session_id.as_bytes()),
            "name" => string!(name),
            "created_at" => bytes!(created_at_nanos.to_le_bytes())
        };
        let scb = encode(&payload).unwrap();
        fs::write(session_dir.join("session.scb"), scb).unwrap();

        workspace
    }

    fn session_map(root: &Path, session_id: &str) -> std::collections::BTreeMap<String, Value> {
        let raw = fs::read(
            root.join(".plumb")
                .join("sessions")
                .join(session_id)
                .join("session.scb"),
        )
        .unwrap();
        match decode(&raw).unwrap() {
            Value::Map(m) => m,
            _ => panic!("session.scb should decode to map"),
        }
    }

    #[test]
    fn close_session_marks_session_finished_and_clears_active_pointer() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_workspace_with_session_scb(&tmp, "deadbeef", "my-session", 42);

        close_session(workspace.path(), "deadbeef").unwrap();

        let active = fs::read_to_string(workspace.path().join(".plumb").join("active")).unwrap();
        assert!(active.trim().is_empty(), "active file should be cleared");

        let map = session_map(workspace.path(), "deadbeef");
        assert!(matches!(map.get("status"), Some(Value::String(s)) if s == "finished"));
    }

    #[test]
    fn close_session_does_not_clear_active_if_it_points_to_other_session() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_workspace_with_session_scb(&tmp, "deadbeef", "my-session", 42);
        fs::write(workspace.path().join(".plumb").join("active"), "cafebabe").unwrap();

        close_session(workspace.path(), "deadbeef").unwrap();

        let active = fs::read_to_string(workspace.path().join(".plumb").join("active")).unwrap();
        assert_eq!(active, "cafebabe");
    }

    #[test]
    fn test_load_session_success() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_workspace_with_session_scb(&tmp, "deadbeef", "my-session", 42);

        let session = load_session(workspace.path(), "deadbeef").unwrap();
        assert_eq!(session.session_id, "deadbeef");
        assert_eq!(session.name, "my-session");
        assert_eq!(session.created_at_nanos, 42);
    }

    #[test]
    fn test_load_session_empty_name() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_workspace_with_session_scb(&tmp, "aabbccdd", "", 100);

        let session = load_session(workspace.path(), "aabbccdd").unwrap();
        assert_eq!(session.session_id, "aabbccdd");
        assert_eq!(session.name, "");
        assert_eq!(session.created_at_nanos, 100);
    }

    #[test]
    fn test_load_session_missing_file() {
        let tmp = TempDir::new().unwrap();
        let workspace = tempfile::tempdir_in(tmp.path()).unwrap();
        let plumb_dir = workspace.path().join(".plumb");
        fs::create_dir_all(plumb_dir.join("sessions").join("deadbeef")).unwrap();

        let result = load_session(workspace.path(), "deadbeef");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StoreError::ReadError(_)));
    }

    #[test]
    fn test_load_session_invalid_scb() {
        let tmp = TempDir::new().unwrap();
        let workspace = tempfile::tempdir_in(tmp.path()).unwrap();
        let plumb_dir = workspace.path().join(".plumb");
        let session_dir = plumb_dir.join("sessions").join("deadbeef");
        fs::create_dir_all(&session_dir).unwrap();
        fs::write(session_dir.join("session.scb"), "not valid scb").unwrap();

        let result = load_session(workspace.path(), "deadbeef");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StoreError::ReadError(_)));
    }
}
