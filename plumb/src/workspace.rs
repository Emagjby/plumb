use std::path::{Path, PathBuf};

use atomicwrites::{AtomicFile, OverwriteBehavior};
use rand::Rng;
use strata::{bytes, encode::encode, map, string, value::Value};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("active session {session_id} detected in workspace at {root}")]
    SessionAlreadyActive { root: PathBuf, session_id: String },
    #[error("{0}")]
    UnknownError(String),
}

pub fn resolve_workspace_root(cwd: &Path) -> Result<PathBuf, WorkspaceError> {
    let mut dir = cwd;

    loop {
        if dir.join(".plumb").is_dir() {
            return Ok(dir.to_path_buf());
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    Ok(cwd.to_path_buf())
}

pub fn ensure_plumb_dir(root: &Path) -> Result<(), WorkspaceError> {
    let plumb_path = plumb_dir(root);
    ensure_dir(&plumb_path)?;

    Ok(())
}

pub fn ensure_no_active_session(root: &Path) -> Result<(), WorkspaceError> {
    let plumb_path = plumb_dir(root);

    let active_path = plumb_path.join("active");
    if active_path.is_file() {
        let contents = std::fs::read_to_string(&active_path)
            .map_err(|e| WorkspaceError::UnknownError(e.to_string()))?;

        let session_id = contents.trim();

        if session_id.len() == 8 && session_id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(WorkspaceError::SessionAlreadyActive {
                root: root.to_path_buf(),
                session_id: session_id.to_string(),
            });
        }

        return Err(WorkspaceError::UnknownError(format!(
            "corrupted session id in active file at {:?}",
            active_path
        )));
    } else if active_path.exists() {
        return Err(WorkspaceError::UnknownError(format!(
            "expected 'active' to be a file at {:?}",
            active_path
        )));
    }

    Ok(())
}

pub fn initialize_session(root: &Path, name: &str) -> Result<(), WorkspaceError> {
    let session_id = random_session_id();

    ensure_sessions_dir(root)?;

    let session_dir = ensure_new_session_dir(root, &session_id)?;
    ensure_snapshot_dir(&session_dir)?;

    initialize_session_metadata(&session_dir, &session_id, name)?;
    write_active_session(root, &session_id)?;

    if name.is_empty() {
        println!("Started new session with id {}", session_id);
    } else {
        println!("Started new session '{}' with id {}", name, session_id);
    }

    Ok(())
}

fn initialize_session_metadata(
    session_dir: &Path,
    session_id: &str,
    name: &str,
) -> Result<(), WorkspaceError> {
    let payload = map! {
        "session_id" => bytes!(session_id.as_bytes()),
        "name" => string!(name),
        "created_at" => bytes!(now_timestamp_bytes()),
    };

    let scb = encode(&payload).map_err(|e| {
        WorkspaceError::UnknownError(format!("failed to encode session metadata: {:?}", e))
    })?;

    let metadata_path = session_dir.join("session.scb");
    atomic_write(&metadata_path, &scb)?;

    let empty_items = Value::List(vec![]);
    let items_scb = encode(&empty_items).map_err(|e| {
        WorkspaceError::UnknownError(format!("failed to encode empty items: {:?}", e))
    })?;
    let items_path = session_dir.join("items.scb");
    atomic_write(&items_path, &items_scb)?;

    Ok(())
}

fn now_timestamp_bytes() -> [u8; 16] {
    let ns: i128 = OffsetDateTime::now_utc().unix_timestamp_nanos();
    ns.to_le_bytes()
}

fn write_active_session(root: &Path, session_id: &str) -> Result<(), WorkspaceError> {
    let active_path = plumb_dir(root).join("active");

    atomic_write(&active_path, session_id.as_bytes())
}

fn ensure_new_session_dir(root: &Path, session_id: &str) -> Result<PathBuf, WorkspaceError> {
    let session_path = plumb_dir(root).join("sessions").join(session_id);
    ensure_dir(&session_path)?;

    Ok(session_path)
}

fn ensure_snapshot_dir(session_dir: &Path) -> Result<(), WorkspaceError> {
    let snapshot_path = session_dir.join("snapshots");
    ensure_dir(&snapshot_path)?;

    Ok(())
}

fn ensure_sessions_dir(root: &Path) -> Result<(), WorkspaceError> {
    let sessions_path = plumb_dir(root).join("sessions");
    ensure_dir(&sessions_path)?;

    Ok(())
}

fn random_session_id() -> String {
    let mut bytes = [0u8; 4];
    rand::rng().fill_bytes(&mut bytes);
    format!(
        "{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3]
    )
}

fn plumb_dir(root: &Path) -> PathBuf {
    root.join(".plumb")
}

fn ensure_dir(path: &Path) -> Result<(), WorkspaceError> {
    if !path.exists() {
        std::fs::create_dir(path).map_err(|e| WorkspaceError::UnknownError(e.to_string()))?;
    } else if !path.is_dir() {
        return Err(WorkspaceError::UnknownError(format!(
            "expected directory at {:?}",
            path
        )));
    }

    Ok(())
}

fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), WorkspaceError> {
    let af = AtomicFile::new(path, OverwriteBehavior::AllowOverwrite);
    af.write(|f| {
        use std::io::Write;
        f.write_all(contents)
    })
    .map_err(|e| WorkspaceError::UnknownError(e.to_string()))?;

    Ok(())
}

// ---- Unit Tests ----
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn mk_dir(path: &Path) {
        fs::create_dir_all(path).unwrap();
    }

    fn mk_plumb(path: &Path) {
        mk_dir(&path.join(".plumb"));
    }

    #[test]
    fn nested_dir_finds_nearest_parent_with_plumb() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("root");
        let nested = root.join("nested");

        mk_plumb(&root);
        mk_dir(&nested);

        let resolved = resolve_workspace_root(&nested).unwrap();
        assert_eq!(resolved, root);
    }

    #[test]
    fn no_plumb_falls_back_to_cwd() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path().join("dir");

        mk_dir(&dir);

        let resolved = resolve_workspace_root(&dir).unwrap();
        assert_eq!(resolved, dir);
    }

    #[test]
    fn multiple_plumb_dirs_finds_nearest() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("root");
        let nested = root.join("nested");

        mk_plumb(&root);
        mk_plumb(&nested);

        let resolved = resolve_workspace_root(&nested).unwrap();
        assert_eq!(resolved, nested);
    }

    #[test]
    fn ensure_plumb_dir_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("root");

        mk_dir(&root);

        ensure_plumb_dir(&root).unwrap();

        assert!(root.join(".plumb").is_dir());
    }

    #[test]
    fn ensure_plumb_dir_noop_if_exists() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("root");

        mk_plumb(&root);

        ensure_plumb_dir(&root).unwrap();

        assert!(root.join(".plumb").is_dir());
    }

    #[test]
    fn ensure_no_active_session_returns_ok_if_none() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("root");

        mk_plumb(&root);

        let result = ensure_no_active_session(&root);
        assert!(result.is_ok());
    }

    #[test]
    fn ensure_no_active_session_returns_error_if_active() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("root");

        mk_plumb(&root);
        write_active_session(&root, "deadbeef").unwrap();

        let result = ensure_no_active_session(&root);
        assert!(matches!(
            result,
            Err(WorkspaceError::SessionAlreadyActive { root, session_id } ) if session_id == "deadbeef"
        ));
    }

    #[test]
    fn write_active_session_creates_active_file() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("root");

        mk_plumb(&root);

        write_active_session(&root, "deadbeef").unwrap();

        let active_path = root.join(".plumb").join("active");
        assert!(active_path.is_file());

        let contents = fs::read_to_string(active_path).unwrap();
        assert_eq!(contents.trim(), "deadbeef");
    }

    #[test]
    fn random_session_id_returns_8_char_hex() {
        let session_id = random_session_id();
        assert_eq!(session_id.len(), 8);
        assert!(session_id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn initialize_session_creates_session_dir_and_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("root");

        mk_plumb(&root);

        initialize_session(&root, "test session").unwrap();

        assert!(root.join(".plumb").join("sessions").is_dir());
        let entries = fs::read_dir(root.join(".plumb").join("sessions"))
            .unwrap()
            .filter_map(|e| e.ok())
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].file_type().unwrap().is_dir());
        assert!(entries[0].path().join("session.scb").is_file());
        assert!(entries[0].path().join("items.scb").is_file());
        assert!(entries[0].path().join("snapshots").is_dir());
        assert!(root.join(".plumb").join("active").is_file());
    }

    #[test]
    fn initialize_session_fails_if_plumb_dir_missing() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("root");

        mk_dir(&root);

        let result = initialize_session(&root, "test session");
        assert!(matches!(result, Err(WorkspaceError::UnknownError(_))));
    }

    #[test]
    fn ensure_dir_returns_error_if_path_is_file() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("root");

        mk_plumb(&root);
        let file_path = root.join(".plumb").join("file.txt");
        fs::write(&file_path, "test").unwrap();

        let result = ensure_dir(&file_path);
        assert!(matches!(result, Err(WorkspaceError::UnknownError(_))));
    }

    #[test]
    fn ensure_dir_noop_if_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("root");

        mk_plumb(&root);
        let dir_path = root.join(".plumb").join("dir");
        mk_dir(&dir_path);

        let result = ensure_dir(&dir_path);
        assert!(result.is_ok());
    }

    #[test]
    fn ensure_dir_creates_directory_if_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("root");

        mk_plumb(&root);
        let dir_path = root.join(".plumb").join("newdir");

        let result = ensure_dir(&dir_path);
        assert!(result.is_ok());
        assert!(dir_path.is_dir());
    }

    #[test]
    fn atomic_write_overwrites_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("root");

        mk_plumb(&root);
        let file_path = root.join(".plumb").join("file.txt");
        fs::write(&file_path, "old").unwrap();

        atomic_write(&file_path, b"new").unwrap();

        let contents = fs::read_to_string(file_path).unwrap();
        assert_eq!(contents, "new");
    }

    #[test]
    fn atomic_write_creates_file_if_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("root");

        mk_plumb(&root);
        let file_path = root.join(".plumb").join("newfile.txt");

        atomic_write(&file_path, b"content").unwrap();

        let contents = fs::read_to_string(file_path).unwrap();
        assert_eq!(contents, "content");
    }

    #[test]
    fn atomic_write_does_not_leave_tmp_files() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("root");

        mk_plumb(&root);
        let file_path = root.join(".plumb").join("file.txt");

        atomic_write(&file_path, b"data").unwrap();

        let tmp_files: Vec<_> = fs::read_dir(root.join(".plumb"))
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("file.txt"))
            .collect();

        assert_eq!(tmp_files.len(), 1);
    }
}
