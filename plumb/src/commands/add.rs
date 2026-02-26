use std::path::Path;

use thiserror::Error;

use crate::{
    fs::{
        InputError, collect_folder_files, lexical_normalize, normalize_rel_path_from_cwd,
        to_slash_path,
    },
    output::OutputMessage,
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

pub fn plumb_add(file: String, folder: bool) -> Result<(), AddError> {
    let cwd = std::env::current_dir().map_err(|e| AddError::UnknownError(e.to_string()))?;
    plumb_add_from_cwd(&cwd, file, folder, |root, session_id, items| {
        save_items(root, session_id, items)
    })
}

fn plumb_add_from_cwd<F>(
    cwd: &Path,
    file: String,
    folder: bool,
    mut save_items_fn: F,
) -> Result<(), AddError>
where
    F: FnMut(&Path, &str, &[Item]) -> Result<(), StoreError>,
{
    let root = resolve_workspace_root(cwd).map_err(|e| AddError::UnknownError(e.to_string()))?;

    let session_id = active_session_id(&root).map_err(|e| match e {
        StoreError::NoActiveSession => AddError::NoActiveSession(
            "no active session found, use `plumb start` to start a session".to_string(),
        ),
        other => AddError::StoreError(other),
    })?;
    let normalized_path = normalize_rel_path_from_cwd(&root, Path::new(&file), cwd)?;

    let mut items = load_items(&root)?;

    if folder {
        let added_count = plumb_add_folder(&mut items, &normalized_path, &session_id, &root)?;
        if added_count > 0 {
            save_items_fn(&root, &session_id, &items).map_err(AddError::StoreError)?;
        }

        return Ok(());
    }

    if root.join(&normalized_path).is_dir() {
        return Err(AddError::InputError(InputError::InvalidPath(format!(
            "path is a directory: {}",
            normalized_path
        ))));
    }

    plumb_add_file(&mut items, &normalized_path)?;
    save_items_fn(&root, &session_id, &items).map_err(AddError::StoreError)?;
    print!(
        "{}",
        OutputMessage::ok("PLB-OUT-ITM-001", "item added to queue")
            .with_command("plumb add")
            .with_context("session_id", session_id)
            .with_context("path", normalized_path)
    );

    Ok(())
}

fn plumb_add_folder(
    items: &mut Vec<Item>,
    normalized_path: &String,
    session_id: &str,
    root: &Path,
) -> Result<usize, AddError> {
    let folder_path = root.join(normalized_path);

    let files =
        collect_folder_files(&folder_path).map_err(|e| AddError::UnknownError(e.to_string()))?;

    let mut added_count = 0usize;
    let files_found = files.len();
    for file in files {
        let file_rel_path = normalize_rel_path_from_cwd(root, &file, root)?;
        match plumb_add_file(items, &file_rel_path) {
            Ok(()) => added_count += 1,
            Err(AddError::FileAlreadyInQueue(_)) => {
                print!(
                    "{}",
                    OutputMessage::warn("PLB-OUT-ITM-003", "item already in queue")
                        .with_command("plumb add -f")
                        .with_context("path", file_rel_path)
                        .with_action("skipped")
                )
            }
            Err(e) => print!(
                "{}",
                OutputMessage::warn("PLB-OUT-ITM-004", "failed to add file during folder scan")
                    .with_command("plumb add -f")
                    .with_context("path", file_rel_path)
                    .with_note(e.to_string())
                    .with_action("skipped")
            ),
        }
    }
    print!(
        "{}",
        OutputMessage::ok("PLB-OUT-ITM-002", "folder scan completed")
            .with_command("plumb add -f")
            .with_context("session_id", session_id)
            .with_context("path", normalized_path)
            .with_context("files_found", files_found.to_string())
            .with_context("items_added", added_count.to_string())
    );

    Ok(added_count)
}

fn plumb_add_file(items: &mut Vec<Item>, normalized_path: &str) -> Result<(), AddError> {
    if check_duplicates(items, normalized_path) {
        return Err(AddError::FileAlreadyInQueue(format!(
            "file already in queue: {}",
            normalized_path
        )));
    }

    let new_item = Item {
        id: next_id(items),
        rel_path: normalized_path.to_string(),
        state: State::Todo,
    };

    items.push(new_item);

    Ok(())
}

fn check_duplicates(items: &[Item], normalized_path: &str) -> bool {
    let candidate = normalize_item_path(normalized_path);
    items
        .iter()
        .any(|item| normalize_item_path(&item.rel_path) == candidate)
}

fn next_id(items: &[Item]) -> usize {
    items.iter().map(|item| item.id).max().unwrap_or(0) + 1
}

fn normalize_item_path(path: &str) -> String {
    to_slash_path(&lexical_normalize(Path::new(path)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::items::State;
    use std::cell::Cell;
    use std::fs;
    use tempfile::TempDir;

    fn create_workspace_with_active_session() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let session_id = "deadbeef";

        fs::create_dir_all(root.join(".plumb/sessions").join(session_id)).unwrap();
        fs::write(root.join(".plumb/active"), session_id).unwrap();

        temp_dir
    }

    fn ids_and_paths(root: &Path) -> Vec<(usize, String)> {
        let items = load_items(root).unwrap();
        items
            .into_iter()
            .map(|item| (item.id, item.rel_path))
            .collect()
    }

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

    #[test]
    fn duplicates_detected_after_normalization() {
        let items = vec![Item {
            id: 1,
            rel_path: "src/./a.rs".to_string(),
            state: State::Todo,
        }];
        assert!(check_duplicates(&items, "src/a.rs"));
    }

    #[test]
    fn next_id_continues_from_max_even_if_items_unsorted() {
        let items = vec![
            Item {
                id: 10,
                rel_path: "a.rs".to_string(),
                state: State::Todo,
            },
            Item {
                id: 3,
                rel_path: "b.rs".to_string(),
                state: State::Todo,
            },
            Item {
                id: 7,
                rel_path: "c.rs".to_string(),
                state: State::Todo,
            },
        ];

        assert_eq!(next_id(&items), 11);
    }

    #[test]
    fn add_without_folder_rejects_directory_path() {
        let workspace = create_workspace_with_active_session();
        let root = workspace.path();

        fs::create_dir_all(root.join("src")).unwrap();

        let result = plumb_add_from_cwd(root, "src".to_string(), false, save_items);
        let err = result.unwrap_err();

        assert!(matches!(
            err,
            AddError::InputError(InputError::InvalidPath(msg)) if msg.contains("directory")
        ));
    }

    #[test]
    fn add_with_folder_accepts_relative_and_absolute_paths_same_result() {
        let relative_workspace = create_workspace_with_active_session();
        let relative_root = relative_workspace.path();
        fs::create_dir_all(relative_root.join("src/deep")).unwrap();
        fs::write(relative_root.join("src/deep/a.rs"), "").unwrap();
        fs::write(relative_root.join("src/b.rs"), "").unwrap();

        plumb_add_from_cwd(relative_root, "src".to_string(), true, save_items).unwrap();
        let relative_items = ids_and_paths(relative_root);

        let absolute_workspace = create_workspace_with_active_session();
        let absolute_root = absolute_workspace.path();
        fs::create_dir_all(absolute_root.join("src/deep")).unwrap();
        fs::write(absolute_root.join("src/deep/a.rs"), "").unwrap();
        fs::write(absolute_root.join("src/b.rs"), "").unwrap();

        let abs_folder = absolute_root.join("src").to_string_lossy().to_string();
        plumb_add_from_cwd(absolute_root, abs_folder, true, save_items).unwrap();
        let absolute_items = ids_and_paths(absolute_root);

        assert_eq!(relative_items, absolute_items);
    }

    #[test]
    fn add_folder_persists_items_once_per_invocation() {
        let workspace = create_workspace_with_active_session();
        let root = workspace.path();

        fs::create_dir_all(root.join("src/deep")).unwrap();
        fs::write(root.join("src/a.rs"), "").unwrap();
        fs::write(root.join("src/deep/b.rs"), "").unwrap();

        let save_count = Cell::new(0usize);
        plumb_add_from_cwd(
            root,
            "src".to_string(),
            true,
            |save_root, session_id, items| {
                save_count.set(save_count.get() + 1);
                save_items(save_root, session_id, items)
            },
        )
        .unwrap();

        assert_eq!(save_count.get(), 1, "folder add should save once");
    }

    #[test]
    fn add_folder_with_zero_eligible_files_is_noop() {
        let workspace = create_workspace_with_active_session();
        let root = workspace.path();

        fs::create_dir_all(root.join("batch/.git")).unwrap();
        fs::create_dir_all(root.join("batch/node_modules/pkg")).unwrap();
        fs::create_dir_all(root.join("batch/target/build")).unwrap();
        fs::create_dir_all(root.join("batch/.plumb/cache")).unwrap();
        fs::write(root.join("batch/.git/skip.txt"), "").unwrap();
        fs::write(root.join("batch/node_modules/pkg/skip.js"), "").unwrap();
        fs::write(root.join("batch/target/build/skip.rs"), "").unwrap();
        fs::write(root.join("batch/.plumb/cache/skip.txt"), "").unwrap();

        let save_count = Cell::new(0usize);
        plumb_add_from_cwd(
            root,
            "batch".to_string(),
            true,
            |save_root, session_id, items| {
                save_count.set(save_count.get() + 1);
                save_items(save_root, session_id, items)
            },
        )
        .unwrap();

        assert_eq!(save_count.get(), 0, "no-op folder add should not persist");
        assert!(load_items(root).unwrap().is_empty());
    }
}
