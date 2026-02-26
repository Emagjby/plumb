use std::path::Path;

use thiserror::Error;

use crate::{
    helpers::{HelperError, resolve_item},
    output::OutputMessage,
    store::items::{State, StoreError, active_session_id, load_items, save_items},
    workspace::{WorkspaceError, resolve_workspace_root},
};

#[derive(Error, Debug)]
pub enum DoneError {
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

pub fn plumb_done(target: String) -> Result<(), DoneError> {
    let cwd = std::env::current_dir().map_err(|e| DoneError::UnknownError(e.to_string()))?;
    plumb_done_from_cwd(&cwd, target)
}

fn plumb_done_from_cwd(cwd: &Path, target: String) -> Result<(), DoneError> {
    let root = resolve_workspace_root(cwd)?;

    let session_id = active_session_id(&root).map_err(|_| {
        DoneError::NoActiveSession(
            "no active session found, use `plumb start` to start a session".to_string(),
        )
    })?;

    mark_as_done(&root, &session_id, target)?;
    Ok(())
}

fn mark_as_done(root: &Path, session_id: &str, target: String) -> Result<(), DoneError> {
    let mut items = load_items(root)?;
    let (item_id, normalized_path, state) =
        resolve_item(root, &items, &target).map_err(|e| match e {
            HelperError::FileNotInQueue(msg) => DoneError::FileNotInQueue(msg),
            other => DoneError::HelperError(other),
        })?;

    if state != State::InProgress {
        return Err(DoneError::UnknownError(format!(
            "item is not 'In Progress': {}",
            normalized_path
        )));
    }

    for item in &mut items {
        if item.id == item_id {
            item.state = State::Done;
            break;
        }
    }

    save_items(root, session_id, &items).map_err(DoneError::StoreError)?;

    print!(
        "{}",
        OutputMessage::ok("PLB-OUT-ITM-006", "item marked as done")
            .with_command("plumb done")
            .with_context("item_id", item_id.to_string())
            .with_context("path", normalized_path)
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::items::{Item, load_items, save_items};
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

        std::fs::create_dir_all(root.join(".plumb").join("sessions").join(&session_id)).unwrap();
        std::fs::write(root.join(".plumb").join("active"), &session_id).unwrap();
        save_items(root, &session_id, items).unwrap();

        (temp_dir, session_id)
    }

    fn item_states(root: &Path) -> Vec<(usize, String, State)> {
        load_items(root)
            .unwrap()
            .into_iter()
            .map(|item| (item.id, item.rel_path, item.state))
            .collect()
    }

    #[test]
    fn done_errors_without_active_session() {
        let temp_dir = TempDir::new().unwrap();
        let err = plumb_done_from_cwd(temp_dir.path(), "1".to_string()).unwrap_err();
        assert!(matches!(err, DoneError::NoActiveSession(msg) if msg.contains("plumb start")));
    }

    #[test]
    fn done_errors_when_target_not_in_queue_id_variant() {
        let items = vec![make_item(1, "src/a.rs", State::InProgress)];
        let (workspace, session_id) = create_workspace_with_active_session(&items);
        let root = workspace.path();

        let err = mark_as_done(root, &session_id, "99".to_string()).unwrap_err();
        assert!(matches!(err, DoneError::FileNotInQueue(msg) if msg.contains("99")));
    }

    #[test]
    fn done_errors_when_target_not_in_queue_path_variant() {
        let items = vec![make_item(1, "src/a.rs", State::InProgress)];
        let (workspace, session_id) = create_workspace_with_active_session(&items);
        let root = workspace.path();
        let target = root.join("src/missing.rs").to_string_lossy().to_string();

        let err = mark_as_done(root, &session_id, target).unwrap_err();
        assert!(matches!(err, DoneError::FileNotInQueue(msg) if msg.contains("src/missing.rs")));
    }

    #[test]
    fn done_rejects_todo_item() {
        let items = vec![make_item(1, "src/a.rs", State::Todo)];
        let (workspace, session_id) = create_workspace_with_active_session(&items);
        let root = workspace.path();
        let before = item_states(root);

        let err = mark_as_done(root, &session_id, "1".to_string()).unwrap_err();
        assert!(matches!(err, DoneError::UnknownError(msg) if msg.contains("not 'In Progress'")));

        let after = item_states(root);
        assert_eq!(after, before);
    }

    #[test]
    fn done_rejects_done_item() {
        let items = vec![make_item(1, "src/a.rs", State::Done)];
        let (workspace, session_id) = create_workspace_with_active_session(&items);
        let root = workspace.path();
        let before = item_states(root);

        let err = mark_as_done(root, &session_id, "1".to_string()).unwrap_err();
        assert!(matches!(err, DoneError::UnknownError(msg) if msg.contains("not 'In Progress'")));

        let after = item_states(root);
        assert_eq!(after, before);
    }

    #[test]
    fn done_succeeds_for_in_progress_item() {
        let items = vec![make_item(1, "src/a.rs", State::InProgress)];
        let (workspace, session_id) = create_workspace_with_active_session(&items);
        let root = workspace.path();

        mark_as_done(root, &session_id, "1".to_string()).unwrap();

        let after = item_states(root);
        assert_eq!(after, vec![(1, "src/a.rs".to_string(), State::Done)]);
    }

    #[test]
    fn done_by_id_marks_only_that_item_done() {
        let items = vec![
            make_item(1, "src/a.rs", State::Todo),
            make_item(2, "src/b.rs", State::InProgress),
            make_item(3, "src/c.rs", State::Done),
        ];
        let (workspace, session_id) = create_workspace_with_active_session(&items);
        let root = workspace.path();

        mark_as_done(root, &session_id, "2".to_string()).unwrap();

        let after = item_states(root);
        assert_eq!(
            after,
            vec![
                (1, "src/a.rs".to_string(), State::Todo),
                (2, "src/b.rs".to_string(), State::Done),
                (3, "src/c.rs".to_string(), State::Done),
            ]
        );
    }

    #[test]
    fn done_by_path_marks_only_that_item_done() {
        let items = vec![
            make_item(1, "src/a.rs", State::Todo),
            make_item(2, "src/b.rs", State::InProgress),
            make_item(3, "src/c.rs", State::Done),
        ];
        let (workspace, session_id) = create_workspace_with_active_session(&items);
        let root = workspace.path();
        let target = root.join("src/b.rs").to_string_lossy().to_string();

        mark_as_done(root, &session_id, target).unwrap();

        let after = item_states(root);
        assert_eq!(
            after,
            vec![
                (1, "src/a.rs".to_string(), State::Todo),
                (2, "src/b.rs".to_string(), State::Done),
                (3, "src/c.rs".to_string(), State::Done),
            ]
        );
    }
}
