use std::path::Path;

use thiserror::Error;

use crate::{
    store::items::{Item, State, active_session_id, load_items},
    workspace::{WorkspaceError, resolve_workspace_root},
};

#[derive(Error, Debug)]
pub enum NextError {
    #[error("{0}")]
    NoTodoInQueue(String),
    #[error("{0}")]
    NoActiveSession(String),
    #[error("{0}")]
    WorkspaceError(#[from] WorkspaceError),
    #[error("{0}")]
    UnknownError(String),
}

pub fn plumb_next() -> Result<(), NextError> {
    let cwd = std::env::current_dir().map_err(|e| NextError::UnknownError(e.to_string()))?;
    plumb_next_from_cwd(&cwd)
}

fn plumb_next_from_cwd(cwd: &Path) -> Result<(), NextError> {
    let root = resolve_workspace_root(&cwd)?;

    let _session_id = active_session_id(&root).map_err(|_| {
        NextError::NoActiveSession(
            "no active session found, use `plumb start` to start a session".to_string(),
        )
    })?;

    print_next_item(&root)?;

    Ok(())
}

fn print_next_item(root: &Path) -> Result<(), NextError> {
    let items = load_items(root).map_err(|e| NextError::UnknownError(e.to_string()))?;

    let next_item = next_todo_item(&items).ok_or_else(|| {
        NextError::NoTodoInQueue("no 'To Do' items found in the queue".to_string())
    })?;

    println!("Next item: {} (ID: {})", next_item.rel_path, next_item.id);

    Ok(())
}

fn next_todo_item(items: &[Item]) -> Option<&Item> {
    items.iter().find(|item| item.state == State::Todo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::items::save_items;
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
    fn next_errors_without_active_session() {
        let temp_dir = TempDir::new().unwrap();
        let err = plumb_next_from_cwd(temp_dir.path()).unwrap_err();
        assert!(matches!(err, NextError::NoActiveSession(msg) if msg.contains("plumb start")));
    }

    #[test]
    fn next_errors_when_no_todo_items_exist() {
        let items = vec![
            make_item(1, "src/a.rs", State::Done),
            make_item(2, "src/b.rs", State::InProgress),
        ];
        let (workspace, _) = create_workspace_with_active_session(&items);
        let root = workspace.path();

        let err = print_next_item(root).unwrap_err();
        assert!(matches!(err, NextError::NoTodoInQueue(msg) if msg.contains("To Do")));
    }

    #[test]
    fn next_selects_first_todo_in_stored_order() {
        let items = vec![
            make_item(10, "done.txt", State::Done),
            make_item(42, "first_todo.txt", State::Todo),
            make_item(1, "second_todo.txt", State::Todo),
        ];

        let next = next_todo_item(&items).unwrap();
        assert_eq!(next.id, 42);
        assert_eq!(next.rel_path, "first_todo.txt");
    }

    #[test]
    fn next_skips_done_and_in_progress_and_returns_next_todo_in_order() {
        let items = vec![
            make_item(1, "done.txt", State::Done),
            make_item(2, "current.txt", State::InProgress),
            make_item(9, "next.txt", State::Todo),
            make_item(3, "later.txt", State::Todo),
        ];

        let next = next_todo_item(&items).unwrap();
        assert_eq!(next.id, 9);
        assert_eq!(next.rel_path, "next.txt");
    }

    #[test]
    fn next_does_not_mutate_items() {
        let items = vec![
            make_item(1, "a.txt", State::Todo),
            make_item(2, "b.txt", State::Done),
        ];
        let (workspace, _) = create_workspace_with_active_session(&items);
        let root = workspace.path();
        let before = item_states(root);

        print_next_item(root).unwrap();

        let after = item_states(root);
        assert_eq!(after, before);
    }
}
