use std::path::Path;

use thiserror::Error;

use crate::{
    output::OutputMessage,
    store::{
        items::{State, StoreError, active_session_id, load_items},
        session::close_session,
    },
    workspace::{WorkspaceError, resolve_workspace_root},
};

#[derive(Error, Debug)]
pub enum FinishError {
    #[error("{0}")]
    NoActiveSession(String),
    #[error("{0}")]
    StoreError(#[from] StoreError),
    #[error("{0}")]
    WorkspaceError(#[from] WorkspaceError),
    #[error("{0}")]
    UnknownError(String),
}

pub fn plumb_finish() -> Result<(), FinishError> {
    let cwd = std::env::current_dir().map_err(|e| FinishError::UnknownError(e.to_string()))?;
    let root = resolve_workspace_root(&cwd)?;

    let session_id = active_session_id(&root).map_err(|_| {
        FinishError::NoActiveSession(
            "no active session found, use `plumb start` to start a session".to_string(),
        )
    })?;

    check_in_progress_items(&root)?;
    close_session(&root, &session_id)?;

    print!(
        "{}",
        OutputMessage::ok("PLB-OUT-SES-003", "session finished")
            .with_command("plumb finish")
            .with_context("session_id", session_id)
    );

    Ok(())
}

fn check_in_progress_items(root: &Path) -> Result<(), FinishError> {
    if load_items(root)?
        .into_iter()
        .any(|item| item.state == State::InProgress)
    {
        return Err(FinishError::UnknownError(
            "cannot finish session, at least one item is still in progress".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::items::{Item, save_items};
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

    #[test]
    fn finish_refuses_when_any_item_in_progress() {
        let items = vec![
            make_item(1, "todo.rs", State::Todo),
            make_item(2, "active.rs", State::InProgress),
            make_item(3, "done.rs", State::Done),
        ];
        let (workspace, _) = create_workspace_with_active_session(&items);

        let err = check_in_progress_items(workspace.path()).unwrap_err();
        assert!(matches!(err, FinishError::UnknownError(msg) if msg.contains("in progress")));
    }

    #[test]
    fn finish_allowed_when_none_in_progress() {
        let items = vec![
            make_item(1, "todo.rs", State::Todo),
            make_item(2, "done.rs", State::Done),
        ];
        let (workspace, _) = create_workspace_with_active_session(&items);

        let result = check_in_progress_items(workspace.path());
        assert!(result.is_ok());
    }
}
