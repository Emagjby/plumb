use std::path::Path;

use thiserror::Error;

use crate::{
    fs::{FsError, atomic_write},
    helpers::{HelperError, resolve_item},
    store::items::{Item, State, StoreError, active_session_id, load_items, save_items},
    workspace::resolve_workspace_root,
};

#[derive(Error, Debug)]
pub enum GoError {
    #[error("{0}")]
    AlreadyDone(String),
    #[error("{0}")]
    AlreadyInProgress(String),
    #[error("{0}")]
    NoActiveSession(String),
    #[error("{0}")]
    FileNotInQueue(String),
    #[error("{0}")]
    EditorError(String),
    #[error("{0}")]
    BaselineCaptureError(String),
    #[error("{0}")]
    HelperError(#[from] HelperError),
    #[error("{0}")]
    StoreError(#[from] StoreError),
    #[error("{0}")]
    FsError(#[from] FsError),
    #[error("{0}")]
    UnknownError(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GoAction {
    ReopenInProgress,
    StartTodo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GoPlan {
    action: GoAction,
    item_id: usize,
    normalized_path: String,
}

pub fn plumb_go(target: String) -> Result<(), GoError> {
    let cwd = std::env::current_dir().map_err(|e| GoError::UnknownError(e.to_string()))?;
    plumb_go_from_cwd_with_ops(
        &cwd,
        target,
        open_in_editor,
        capture_baseline,
        mark_item_in_progress,
    )
}

fn plumb_go_from_cwd_with_ops<FOpen, FCapture, FMark>(
    cwd: &Path,
    target: String,
    mut open_in_editor_fn: FOpen,
    mut capture_baseline_fn: FCapture,
    mut mark_item_in_progress_fn: FMark,
) -> Result<(), GoError>
where
    FOpen: FnMut(&Path) -> Result<(), GoError>,
    FCapture: FnMut(&Path, &str, usize, &str) -> Result<(), GoError>,
    FMark: FnMut(&Path, &str, usize) -> Result<(), GoError>,
{
    let root = resolve_workspace_root(cwd).map_err(|e| GoError::UnknownError(e.to_string()))?;

    let session_id = active_session_id(&root).map_err(|_| {
        GoError::NoActiveSession(
            "no active session found, use `plumb start` to start a session".to_string(),
        )
    })?;

    let items = load_items(&root)?;
    let (item_id, normalized_path, state) =
        resolve_item(&root, &items, &target).map_err(|e| match e {
            HelperError::FileNotInQueue(msg) => GoError::FileNotInQueue(msg),
            other => GoError::HelperError(other),
        })?;

    let plan = go_plan(&items, item_id, &normalized_path, state, || {
        ensure_baseline_source_ready(&root, &normalized_path)
    })?;

    let full_path = root.join(&plan.normalized_path);

    match plan.action {
        GoAction::ReopenInProgress => {
            open_in_editor_fn(&full_path)?;
        }
        GoAction::StartTodo => {
            capture_baseline_fn(&root, &session_id, plan.item_id, &plan.normalized_path)?;
            mark_item_in_progress_fn(&root, &session_id, plan.item_id)?;
            open_in_editor_fn(&full_path)?;
        }
    }

    Ok(())
}

fn go_plan<F>(
    _items: &[Item],
    item_id: usize,
    normalized_path: &str,
    state: State,
    mut pre_baseline_check: F,
) -> Result<GoPlan, GoError>
where
    F: FnMut() -> Result<(), GoError>,
{
    if state == State::Done {
        return Err(GoError::AlreadyDone(format!(
            "file is already marked as done: {}",
            normalized_path
        )));
    }

    if state == State::InProgress {
        return Ok(GoPlan {
            action: GoAction::ReopenInProgress,
            item_id,
            normalized_path: normalized_path.to_string(),
        });
    }

    pre_baseline_check()?;

    Ok(GoPlan {
        action: GoAction::StartTodo,
        item_id,
        normalized_path: normalized_path.to_string(),
    })
}

fn open_in_editor(path: &Path) -> Result<(), GoError> {
    open_in_editor_with(
        path,
        || std::env::var("EDITOR").ok(),
        |editor, file_path| {
            let status = std::process::Command::new(editor)
                .arg(file_path)
                .status()
                .map_err(|e| GoError::EditorError(format!("failed to open editor: {}", e)))?;

            if !status.success() {
                return Err(GoError::EditorError(format!(
                    "editor exited with status: {}",
                    status
                )));
            }

            Ok(())
        },
    )
}

fn open_in_editor_with<FEditor, FRun>(
    path: &Path,
    mut editor_from_env: FEditor,
    mut run_editor: FRun,
) -> Result<(), GoError>
where
    FEditor: FnMut() -> Option<String>,
    FRun: FnMut(&str, &Path) -> Result<(), GoError>,
{
    let editor = editor_from_env()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "vim".to_string());
    run_editor(&editor, path)
}

fn ensure_baseline_source_ready(root: &Path, item_path: &str) -> Result<(), GoError> {
    let full_item_path = root.join(item_path);
    if !full_item_path.exists() {
        return Err(GoError::BaselineCaptureError(format!(
            "file does not exist: {}",
            item_path
        )));
    }
    if full_item_path.is_dir() {
        return Err(GoError::BaselineCaptureError(format!(
            "cannot capture baseline for a folder: {}",
            item_path
        )));
    }

    std::fs::File::open(&full_item_path)
        .map_err(|e| GoError::BaselineCaptureError(format!("failed to read file: {}", e)))?;

    Ok(())
}

fn mark_item_in_progress(root: &Path, session_id: &str, item_id: usize) -> Result<(), GoError> {
    let mut items = load_items(root)?;
    let item = items
        .iter_mut()
        .find(|item| item.id == item_id)
        .ok_or_else(|| GoError::FileNotInQueue(format!("no file with ID {} in queue", item_id)))?;

    let rel_path = item.rel_path.clone();

    item.state = State::InProgress;
    save_items(root, session_id, &items)?;

    println!("Started: [{}] {} (baseline captured)", item_id, rel_path);

    Ok(())
}

fn capture_baseline(
    root: &Path,
    session_id: &str,
    item_id: usize,
    item_path: &str,
) -> Result<(), GoError> {
    ensure_baseline_source_ready(root, item_path)?;

    let snapshot_path = root
        .join(".plumb")
        .join("sessions")
        .join(session_id)
        .join("snapshots")
        .join(format!("{}.baseline", item_id));

    let full_item_path = root.join(item_path);
    let content = std::fs::read(&full_item_path)
        .map_err(|e| GoError::BaselineCaptureError(format!("failed to read file: {}", e)))?;

    atomic_write(&snapshot_path, &content)?;

    Ok(())
}

// ---- Unit Tests ----
#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::{cell::Cell, fs};
    use tempfile::TempDir;

    fn make_item(id: usize, rel_path: &str, state: State) -> Item {
        Item {
            id,
            rel_path: rel_path.to_string(),
            state,
        }
    }

    fn create_workspace_with_items(items: &[Item]) -> (TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let session_id = "deadbeef".to_string();

        fs::create_dir_all(
            root.join(".plumb/sessions")
                .join(&session_id)
                .join("snapshots"),
        )
        .unwrap();
        fs::write(root.join(".plumb/active"), &session_id).unwrap();
        save_items(root, &session_id, items).unwrap();

        (temp_dir, session_id)
    }

    fn state_snapshot(root: &Path) -> Vec<(usize, String, State)> {
        load_items(root)
            .unwrap()
            .into_iter()
            .map(|item| (item.id, item.rel_path, item.state))
            .collect()
    }

    #[test]
    fn go_rejects_done_item() {
        let items = vec![make_item(1, "done.rs", State::Done)];
        let (workspace, _) = create_workspace_with_items(&items);
        let root = workspace.path();

        let open_calls = Cell::new(0usize);
        let err = plumb_go_from_cwd_with_ops(
            root,
            "1".to_string(),
            |_| {
                open_calls.set(open_calls.get() + 1);
                Ok(())
            },
            |_, _, _, _| Ok(()),
            |_, _, _| Ok(()),
        )
        .unwrap_err();

        assert!(matches!(err, GoError::AlreadyDone(msg) if msg.contains("done.rs")));
        assert_eq!(open_calls.get(), 0);
    }

    #[test]
    fn go_allows_reopen_when_item_already_in_progress() {
        let items = vec![
            make_item(1, "inprogress.rs", State::InProgress),
            make_item(2, "todo.rs", State::Todo),
        ];
        let (workspace, _) = create_workspace_with_items(&items);
        let root = workspace.path();
        fs::write(root.join("inprogress.rs"), "current").unwrap();

        let before = state_snapshot(root);
        let open_calls = Cell::new(0usize);
        let capture_calls = Cell::new(0usize);
        let mark_calls = Cell::new(0usize);

        plumb_go_from_cwd_with_ops(
            root,
            "1".to_string(),
            |_| {
                open_calls.set(open_calls.get() + 1);
                Ok(())
            },
            |_, _, _, _| {
                capture_calls.set(capture_calls.get() + 1);
                Ok(())
            },
            |_, _, _| {
                mark_calls.set(mark_calls.get() + 1);
                Ok(())
            },
        )
        .unwrap();

        let after = state_snapshot(root);
        assert_eq!(open_calls.get(), 1);
        assert_eq!(capture_calls.get(), 0);
        assert_eq!(mark_calls.get(), 0);
        assert_eq!(after, before);
    }

    #[test]
    fn go_rejects_directory_target_when_not_in_queue() {
        let items = vec![make_item(1, "tracked.rs", State::Todo)];
        let (workspace, _) = create_workspace_with_items(&items);
        let root = workspace.path();
        fs::create_dir_all(root.join("src")).unwrap();
        let target = root.join("src").to_string_lossy().to_string();

        let err = plumb_go_from_cwd_with_ops(
            root,
            target,
            |_| Ok(()),
            |_, _, _, _| Ok(()),
            |_, _, _| Ok(()),
        )
        .unwrap_err();

        assert!(matches!(err, GoError::FileNotInQueue(msg) if msg.contains("src")));
    }

    #[test]
    #[allow(non_snake_case)]
    fn open_in_editor_uses_EDITOR_env_when_set() {
        let mut selected_editor = String::new();
        open_in_editor_with(
            Path::new("file.rs"),
            || Some("nvim".to_string()),
            |editor, _| {
                selected_editor = editor.to_string();
                Ok(())
            },
        )
        .unwrap();

        assert_eq!(selected_editor, "nvim");
    }

    #[test]
    #[allow(non_snake_case)]
    fn open_in_editor_defaults_to_vim_when_EDITOR_missing() {
        let mut selected_editor = String::new();
        open_in_editor_with(
            Path::new("file.rs"),
            || None,
            |editor, _| {
                selected_editor = editor.to_string();
                Ok(())
            },
        )
        .unwrap();

        assert_eq!(selected_editor, "vim");
    }

    #[test]
    fn baseline_capture_errors_when_file_missing() {
        let (workspace, session_id) = create_workspace_with_items(&[]);
        let root = workspace.path();
        let baseline_path = root
            .join(".plumb/sessions")
            .join(&session_id)
            .join("snapshots/1.baseline");

        let err = capture_baseline(root, &session_id, 1, "missing.rs").unwrap_err();
        assert!(
            matches!(err, GoError::BaselineCaptureError(msg) if msg.contains("file does not exist"))
        );
        assert!(!baseline_path.exists());
    }

    #[test]
    fn baseline_capture_errors_when_path_is_directory() {
        let (workspace, session_id) = create_workspace_with_items(&[]);
        let root = workspace.path();
        fs::create_dir_all(root.join("src")).unwrap();
        let baseline_path = root
            .join(".plumb/sessions")
            .join(&session_id)
            .join("snapshots/2.baseline");

        let err = capture_baseline(root, &session_id, 2, "src").unwrap_err();
        assert!(matches!(err, GoError::BaselineCaptureError(msg) if msg.contains("folder")));
        assert!(!baseline_path.exists());
    }

    #[test]
    fn mark_item_in_progress_updates_only_target_item_and_persists() {
        let items = vec![
            make_item(1, "a.rs", State::Todo),
            make_item(2, "b.rs", State::Todo),
            make_item(3, "c.rs", State::Done),
        ];
        let (workspace, session_id) = create_workspace_with_items(&items);
        let root = workspace.path();

        mark_item_in_progress(root, &session_id, 2).unwrap();

        let items = load_items(root).unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].state, State::Todo);
        assert_eq!(items[1].state, State::InProgress);
        assert_eq!(items[2].state, State::Done);
    }

    #[test]
    fn go_only_persists_state_after_successful_baseline_capture() {
        let items = vec![make_item(1, "a.rs", State::Todo)];
        let (workspace, _) = create_workspace_with_items(&items);
        let root = workspace.path();
        fs::write(root.join("a.rs"), "content").unwrap();

        let mark_calls = Cell::new(0usize);
        let open_calls = Cell::new(0usize);
        let err = plumb_go_from_cwd_with_ops(
            root,
            "1".to_string(),
            |_| {
                open_calls.set(open_calls.get() + 1);
                Ok(())
            },
            |_, _, _, _| {
                Err(GoError::BaselineCaptureError(
                    "simulated baseline failure".to_string(),
                ))
            },
            |_, _, _| {
                mark_calls.set(mark_calls.get() + 1);
                Ok(())
            },
        )
        .unwrap_err();

        assert!(
            matches!(err, GoError::BaselineCaptureError(msg) if msg.contains("simulated baseline failure"))
        );
        assert_eq!(mark_calls.get(), 0);
        assert_eq!(open_calls.get(), 0);
        assert_eq!(load_items(root).unwrap()[0].state, State::Todo);
    }

    #[test]
    fn go_plan_returns_no_mutation_on_pre_baseline_failures() {
        let items = vec![make_item(1, "a.rs", State::Todo)];
        let before: Vec<(usize, String, State)> = items
            .iter()
            .map(|item| (item.id, item.rel_path.clone(), item.state.clone()))
            .collect();

        for message in [
            "file does not exist: a.rs",
            "failed to read file: permission denied",
        ] {
            let err = go_plan(&items, 1, "a.rs", State::Todo, || {
                Err(GoError::BaselineCaptureError(message.to_string()))
            })
            .unwrap_err();
            assert!(matches!(err, GoError::BaselineCaptureError(err_msg) if err_msg == message));

            let after: Vec<(usize, String, State)> = items
                .iter()
                .map(|item| (item.id, item.rel_path.clone(), item.state.clone()))
                .collect();
            assert_eq!(after, before);
        }
    }

    #[test]
    #[cfg(unix)]
    fn go_plan_returns_no_mutation_on_unreadable_file_check() {
        let items = vec![make_item(1, "private.rs", State::Todo)];
        let (workspace, _) = create_workspace_with_items(&items);
        let root = workspace.path();
        let private_path = root.join("private.rs");
        fs::write(&private_path, "top secret").unwrap();

        let mut perms = fs::metadata(&private_path).unwrap().permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&private_path, perms).unwrap();

        let before = state_snapshot(root);
        let err = go_plan(&items, 1, "private.rs", State::Todo, || {
            ensure_baseline_source_ready(root, "private.rs")
        })
        .unwrap_err();
        assert!(matches!(err, GoError::BaselineCaptureError(_)));

        let mut restore = fs::metadata(&private_path).unwrap().permissions();
        restore.set_mode(0o644);
        fs::set_permissions(&private_path, restore).unwrap();

        let after = state_snapshot(root);
        assert_eq!(after, before);
    }
}
