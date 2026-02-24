use std::path::Path;

use thiserror::Error;

use crate::{
    fs::{InputError, normalize_rel_path},
    store::items::{Item, State, StoreError, active_session_id, load_items, save_items},
    workspace::resolve_workspace_root,
};

#[derive(Debug, Error)]
pub enum AddError {
    #[error("{0}")]
    StoreError(#[from] StoreError),
    #[error("{0}")]
    FileAlreadyInQueue(String),
    #[error("{0}")]
    InputError(#[from] InputError),
    #[error("{0}")]
    NoActiveSession(String),
    #[error("{0}")]
    UnknownError(String),
}

pub fn plumb_add(file: String) -> Result<(), AddError> {
    let cwd = &std::env::current_dir().map_err(|e| AddError::UnknownError(e.to_string()))?;
    let root = resolve_workspace_root(cwd).map_err(|e| AddError::UnknownError(e.to_string()))?;

    let session_id = active_session_id(&root).map_err(|e| match e {
        StoreError::NoActiveSession => AddError::NoActiveSession(
            "no active session found, please start a session first".to_string(),
        ),
        _ => AddError::StoreError(e),
    })?;
    let normalized_path = normalize_rel_path(&root, Path::new(&file))?;

    let items = load_items(&root)?;

    if check_duplicates(&items, &normalized_path) {
        return Err(AddError::FileAlreadyInQueue(format!(
            "file already in queue: {}",
            normalized_path
        )));
    }

    let new_item = Item {
        id: next_id(&items),
        rel_path: normalized_path.clone(),
        state: State::Todo,
    };

    let mut new_items = items;
    new_items.push(new_item);

    save_items(&root, &session_id, &new_items)?;

    println!("Added: [{}] {}", session_id, normalized_path);

    Ok(())
}

fn check_duplicates(items: &[Item], normalized_path: &str) -> bool {
    items.iter().any(|item| item.rel_path == normalized_path)
}

fn next_id(items: &[Item]) -> usize {
    items.iter().map(|item| item.id).max().unwrap_or(0) + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::items::State;

    #[test]
    fn test_next_id_from_existing_items() {
        let items = vec![
            Item {
                id: 1,
                rel_path: "a.rs".to_string(),
                state: State::Todo,
            },
            Item {
                id: 2,
                rel_path: "b.rs".to_string(),
                state: State::Todo,
            },
            Item {
                id: 3,
                rel_path: "c.rs".to_string(),
                state: State::Todo,
            },
        ];
        assert_eq!(next_id(&items), 4);
    }

    #[test]
    fn test_next_id_single_item() {
        let items = vec![Item {
            id: 5,
            rel_path: "a.rs".to_string(),
            state: State::Todo,
        }];
        assert_eq!(next_id(&items), 6);
    }

    #[test]
    fn test_next_id_empty_list() {
        let items: Vec<Item> = vec![];
        assert_eq!(next_id(&items), 1);
    }

    #[test]
    fn test_check_duplicates_same_path() {
        let items = vec![Item {
            id: 1,
            rel_path: "src/main.rs".to_string(),
            state: State::Todo,
        }];
        assert!(check_duplicates(&items, "src/main.rs"));
    }

    #[test]
    fn test_check_duplicates_different_path() {
        let items = vec![Item {
            id: 1,
            rel_path: "src/main.rs".to_string(),
            state: State::Todo,
        }];
        assert!(!check_duplicates(&items, "src/lib.rs"));
    }

    #[test]
    fn test_check_duplicates_normalized_path_matching() {
        let items = vec![Item {
            id: 1,
            rel_path: "x.rs".to_string(),
            state: State::Todo,
        }];
        assert!(check_duplicates(&items, "x.rs"));
    }
}
