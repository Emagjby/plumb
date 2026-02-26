use std::{
    io::{self, Write},
    path::Path,
};

use thiserror::Error;

use crate::{
    fs::atomic_write,
    helpers::{HelperError, load_baseline, resolve_item},
    store::items::{Item, State, StoreError, active_session_id, load_items},
    workspace::{WorkspaceError, resolve_workspace_root},
};

#[derive(Error, Debug)]
pub enum RestoreError {
    #[error("{0}")]
    FileReadError(String),
    #[error("{0}")]
    FileWriteError(String),
    #[error("{0}")]
    NoActiveSession(String),
    #[error("{0}")]
    FileNotInQueue(String),
    #[error("{0}")]
    HelperError(#[from] HelperError),
    #[error("{0}")]
    StoreError(#[from] StoreError),
    #[error("{0}")]
    WorkspaceError(#[from] WorkspaceError),
    #[error("{0}")]
    UnknownError(String),
}

pub fn plumb_restore(target: String) -> Result<(), RestoreError> {
    let cwd = std::env::current_dir().map_err(|e| RestoreError::UnknownError(e.to_string()))?;
    plumb_restore_from_cwd_with_ops(&cwd, target, prompt_confirmation, write_baseline_bytes)
}

fn plumb_restore_from_cwd_with_ops<FConfirm, FWrite>(
    cwd: &Path,
    target: String,
    mut confirm_fn: FConfirm,
    mut write_fn: FWrite,
) -> Result<(), RestoreError>
where
    FConfirm: FnMut(&str) -> Result<bool, RestoreError>,
    FWrite: FnMut(&Path, &[u8]) -> Result<(), RestoreError>,
{
    let root = resolve_workspace_root(cwd)?;

    let session_id = active_session_id(&root).map_err(|_| {
        RestoreError::NoActiveSession(
            "no active session found, use `plumb start` to start a session".to_string(),
        )
    })?;

    let items = load_items(&root)?;
    let (item_id, normalized_path, state) = resolve_restore_target(&root, &items, &target)?;

    if state == State::Todo {
        return Err(RestoreError::FileReadError(
            "no baseline snapshot. Run `plumb go` first".to_string(),
        ));
    }

    let full_path = root.join(&normalized_path);
    ensure_restore_destination_ready(&full_path, &normalized_path)?;

    let baseline = load_baseline(&root, &session_id, item_id).map_err(|e| match e {
        HelperError::BaselineReadError(msg) => RestoreError::FileReadError(msg),
        other => RestoreError::HelperError(other),
    })?;

    if !confirm_fn(&normalized_path)? {
        println!("Restore cancelled.");
        return Ok(());
    }

    write_fn(&full_path, &baseline)?;
    println!("Restored: [{}] {}", item_id, normalized_path);

    Ok(())
}

fn resolve_restore_target(
    root: &Path,
    items: &[Item],
    target: &str,
) -> Result<(usize, String, State), RestoreError> {
    resolve_item(root, items, target).map_err(|e| match e {
        HelperError::FileNotInQueue(msg) => RestoreError::FileNotInQueue(msg),
        other => RestoreError::HelperError(other),
    })
}

fn ensure_restore_destination_ready(path: &Path, rel_path: &str) -> Result<(), RestoreError> {
    if !path.exists() {
        return Err(RestoreError::FileReadError(format!(
            "file does not exist: {}",
            rel_path
        )));
    }

    if path.is_dir() {
        return Err(RestoreError::FileReadError(format!(
            "cannot restore a folder: {}",
            rel_path
        )));
    }

    std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| RestoreError::FileWriteError(format!("cannot write to file: {e}")))?;

    Ok(())
}

fn write_baseline_bytes(path: &Path, baseline: &[u8]) -> Result<(), RestoreError> {
    atomic_write(path, baseline).map_err(|e| RestoreError::FileWriteError(e.to_string()))
}

fn prompt_confirmation(path: &str) -> Result<bool, RestoreError> {
    println!("Restore {} to baseline snapshot?", path);
    println!("All changes since go-time will be lost.");
    print!("Are you sure? [y/N] ");
    io::stdout()
        .flush()
        .map_err(|e| RestoreError::UnknownError(format!("failed to flush output: {e}")))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| RestoreError::UnknownError(format!("failed to read confirmation: {e}")))?;

    Ok(parse_confirmation(&input))
}

fn parse_confirmation(input: &str) -> bool {
    let answer = input.trim();
    answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::items::{load_items, save_items};
    use std::{cell::Cell, fs, path::PathBuf};
    use tempfile::TempDir;

    fn make_item(id: usize, rel_path: &str, state: State) -> Item {
        Item {
            id,
            rel_path: rel_path.to_string(),
            state,
        }
    }

    fn create_workspace_with_active_session(items: &[Item]) -> (TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let session_id = "deadbeef".to_string();

        fs::create_dir_all(
            root.join(".plumb")
                .join("sessions")
                .join(&session_id)
                .join("snapshots"),
        )
        .unwrap();
        fs::write(root.join(".plumb").join("active"), &session_id).unwrap();
        save_items(root, &session_id, items).unwrap();

        (temp_dir, session_id)
    }

    fn baseline_path(root: &Path, session_id: &str, item_id: usize) -> PathBuf {
        root.join(".plumb")
            .join("sessions")
            .join(session_id)
            .join("snapshots")
            .join(format!("{}.baseline", item_id))
    }

    #[test]
    fn restore_errors_without_active_session() {
        let temp_dir = TempDir::new().unwrap();
        let err = plumb_restore_from_cwd_with_ops(
            temp_dir.path(),
            "1".to_string(),
            |_| Ok(true),
            |_, _| Ok(()),
        )
        .unwrap_err();
        assert!(matches!(err, RestoreError::NoActiveSession(msg) if msg.contains("plumb start")));
    }

    #[test]
    fn restore_errors_when_target_not_in_queue() {
        let items = vec![make_item(1, "src/a.rs", State::InProgress)];
        let (workspace, _) = create_workspace_with_active_session(&items);

        let err = plumb_restore_from_cwd_with_ops(
            workspace.path(),
            "99".to_string(),
            |_| Ok(true),
            |_, _| Ok(()),
        )
        .unwrap_err();
        assert!(matches!(err, RestoreError::FileNotInQueue(msg) if msg.contains("99")));
    }

    #[test]
    fn restore_errors_for_todo_item_without_baseline() {
        let items = vec![make_item(1, "src/a.rs", State::Todo)];
        let (workspace, _) = create_workspace_with_active_session(&items);
        fs::create_dir_all(workspace.path().join("src")).unwrap();
        fs::write(workspace.path().join("src/a.rs"), "current").unwrap();

        let err = plumb_restore_from_cwd_with_ops(
            workspace.path(),
            "1".to_string(),
            |_| Ok(true),
            |_, _| Ok(()),
        )
        .unwrap_err();
        assert!(
            matches!(err, RestoreError::FileReadError(msg) if msg.contains("Run `plumb go` first"))
        );
    }

    #[test]
    fn restore_errors_when_baseline_missing_on_disk() {
        let items = vec![make_item(1, "src/a.rs", State::InProgress)];
        let (workspace, _) = create_workspace_with_active_session(&items);
        fs::create_dir_all(workspace.path().join("src")).unwrap();
        fs::write(workspace.path().join("src/a.rs"), "changed").unwrap();

        let err = plumb_restore_from_cwd_with_ops(
            workspace.path(),
            "1".to_string(),
            |_| Ok(true),
            |_, _| Ok(()),
        )
        .unwrap_err();
        assert!(
            matches!(err, RestoreError::FileReadError(msg) if msg.contains("baseline snapshot not found"))
        );
    }

    #[test]
    fn restore_errors_when_file_missing_even_with_baseline() {
        let items = vec![make_item(1, "src/a.rs", State::InProgress)];
        let (workspace, session_id) = create_workspace_with_active_session(&items);
        fs::write(baseline_path(workspace.path(), &session_id, 1), "before").unwrap();

        let err = plumb_restore_from_cwd_with_ops(
            workspace.path(),
            "1".to_string(),
            |_| Ok(true),
            |_, _| Ok(()),
        )
        .unwrap_err();
        assert!(matches!(err, RestoreError::FileReadError(msg) if msg.contains("does not exist")));
    }

    #[test]
    fn resolve_target_by_id_returns_matching_item() {
        let root = TempDir::new().unwrap();
        let items = vec![make_item(5, "src/a.rs", State::InProgress)];

        let (id, rel_path, state) = resolve_restore_target(root.path(), &items, "5").unwrap();

        assert_eq!(id, 5);
        assert_eq!(rel_path, "src/a.rs");
        assert_eq!(state, State::InProgress);
    }

    #[test]
    fn resolve_target_by_path_supports_normalized_dot_segments() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join("src")).unwrap();
        fs::write(root.path().join("src/a.rs"), "").unwrap();
        let items = vec![make_item(3, "src/a.rs", State::InProgress)];
        let target = root.path().join("src/./a.rs").to_string_lossy().to_string();

        let (id, rel_path, state) = resolve_restore_target(root.path(), &items, &target).unwrap();

        assert_eq!(id, 3);
        assert_eq!(rel_path, "src/a.rs");
        assert_eq!(state, State::InProgress);
    }

    #[test]
    fn restore_cancelled_does_not_write_file() {
        let items = vec![make_item(1, "src/a.rs", State::InProgress)];
        let (workspace, session_id) = create_workspace_with_active_session(&items);
        fs::create_dir_all(workspace.path().join("src")).unwrap();
        let file_path = workspace.path().join("src/a.rs");
        fs::write(&file_path, "changed").unwrap();
        fs::write(baseline_path(workspace.path(), &session_id, 1), "before").unwrap();

        let write_called = Cell::new(false);
        plumb_restore_from_cwd_with_ops(
            workspace.path(),
            "1".to_string(),
            |_| Ok(false),
            |_, _| {
                write_called.set(true);
                Ok(())
            },
        )
        .unwrap();

        assert!(!write_called.get());
        assert_eq!(fs::read_to_string(file_path).unwrap(), "changed");
    }

    #[test]
    fn restore_writes_baseline_and_preserves_item_state() {
        let items = vec![make_item(1, "src/a.rs", State::Done)];
        let (workspace, session_id) = create_workspace_with_active_session(&items);
        fs::create_dir_all(workspace.path().join("src")).unwrap();
        let file_path = workspace.path().join("src/a.rs");
        fs::write(&file_path, "changed").unwrap();
        fs::write(
            baseline_path(workspace.path(), &session_id, 1),
            b"before\x00bytes",
        )
        .unwrap();

        plumb_restore_from_cwd_with_ops(
            workspace.path(),
            "1".to_string(),
            |_| Ok(true),
            write_baseline_bytes,
        )
        .unwrap();

        assert_eq!(fs::read(file_path).unwrap(), b"before\x00bytes");

        let persisted = load_items(workspace.path()).unwrap();
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].state, State::Done);
    }

    #[test]
    fn parse_confirmation_accepts_yes_variants_only() {
        assert!(!parse_confirmation(""));
        assert!(parse_confirmation("y"));
        assert!(parse_confirmation("Y"));
        assert!(parse_confirmation("yes"));
        assert!(parse_confirmation("YES"));
        assert!(!parse_confirmation("n"));
        assert!(!parse_confirmation("anything else"));
    }
}
