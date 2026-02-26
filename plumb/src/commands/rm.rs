use std::path::Path;

use thiserror::Error;

use crate::{
    helpers::{HelperError, resolve_item},
    output::OutputMessage,
    store::items::{Item, State, active_session_id, load_items, save_items, session_dir},
    workspace::resolve_workspace_root,
};

#[derive(Debug, Error)]
pub enum RmError {
    #[error("{0}")]
    StoreError(#[from] crate::store::items::StoreError),
    #[error("{0}")]
    ItemInProgress(String),
    #[error("{0}")]
    NoActiveSession(String),
    #[error("{0}")]
    FileNotInQueue(String),
    #[error("{0}")]
    HelperError(#[from] HelperError),
    #[error("{0}")]
    UnknownError(String),
}

pub fn plumb_rm(target: String) -> Result<(), RmError> {
    let cwd = std::env::current_dir().map_err(|e| RmError::UnknownError(e.to_string()))?;
    let root = resolve_workspace_root(&cwd).map_err(|e| RmError::UnknownError(e.to_string()))?;

    let session_id = active_session_id(&root).map_err(|_| {
        RmError::NoActiveSession(
            "no active session found, use `plumb start` to start a session".to_string(),
        )
    })?;

    let items = load_items(&root)?;

    let (item_id, normalized_path, state) = resolve_item(&root, &items, &target)?;
    if state == State::InProgress {
        return Err(RmError::ItemInProgress(format!(
            "cannot remove file in progress: {}",
            normalized_path
        )));
    }

    cleanup_snapshot(&root, &session_id, item_id)?;

    let new_items = remove_item(&items, item_id)?;
    save_items(&root, &session_id, &new_items)?;

    print!(
        "{}",
        OutputMessage::ok("PLB-OUT-ITM-005", "item removed from queue")
            .with_command("plumb rm")
            .with_context("item_id", item_id.to_string())
            .with_context("path", normalized_path)
    );

    Ok(())
}

fn remove_item(items: &[Item], item_id: usize) -> Result<Vec<Item>, RmError> {
    if !items.iter().any(|item| item.id == item_id) {
        return Err(RmError::FileNotInQueue(format!(
            "no file with ID {} in queue",
            item_id
        )));
    }

    Ok(items
        .iter()
        .filter(|&item| item.id != item_id)
        .cloned()
        .collect())
}

fn cleanup_snapshot(root: &Path, session_id: &str, fid: usize) -> Result<(), RmError> {
    let session_dir = session_dir(root, session_id)?;
    let snapshot_path = session_dir
        .join("snapshots")
        .join(format!("{}.baseline", fid));
    if snapshot_path.exists() {
        std::fs::remove_file(&snapshot_path)
            .map_err(|e| RmError::UnknownError(format!("failed to remove snapshot: {e}")))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::items::State;

    fn make_item(id: usize, rel_path: &str, state: State) -> Item {
        Item {
            id,
            rel_path: rel_path.to_string(),
            state,
        }
    }

    #[test]
    fn test_remove_item_first() {
        let items = vec![
            make_item(1, "a.rs", State::Todo),
            make_item(2, "b.rs", State::Todo),
            make_item(3, "c.rs", State::Todo),
        ];
        let result = remove_item(&items, 1).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 2);
        assert_eq!(result[1].id, 3);
    }

    #[test]
    fn test_remove_item_middle() {
        let items = vec![
            make_item(1, "a.rs", State::Todo),
            make_item(2, "b.rs", State::Todo),
            make_item(3, "c.rs", State::Todo),
        ];
        let result = remove_item(&items, 2).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[1].id, 3);
    }

    #[test]
    fn test_remove_item_last() {
        let items = vec![
            make_item(1, "a.rs", State::Todo),
            make_item(2, "b.rs", State::Todo),
            make_item(3, "c.rs", State::Todo),
        ];
        let result = remove_item(&items, 3).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[1].id, 2);
    }

    #[test]
    fn test_remove_item_last_keeps_gap() {
        let items = vec![
            make_item(1, "a.rs", State::Todo),
            make_item(3, "c.rs", State::Todo),
        ];
        let result = remove_item(&items, 1).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, 3);
    }

    #[test]
    fn test_remove_item_not_found() {
        let items = vec![make_item(1, "a.rs", State::Todo)];
        let result = remove_item(&items, 999);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RmError::FileNotInQueue(msg) if msg.contains("999")
        ));
    }

    #[test]
    fn test_remove_item_empty_list() {
        let items: Vec<Item> = vec![];
        let result = remove_item(&items, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_item_with_different_states() {
        let items = vec![
            make_item(1, "a.rs", State::Todo),
            make_item(2, "b.rs", State::InProgress),
            make_item(3, "c.rs", State::Done),
        ];
        let result = remove_item(&items, 2).unwrap();
        assert_eq!(result.len(), 2);
        assert!(!result.iter().any(|i| i.id == 2));
    }
}
