use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{
    diff::render_baseline_diff,
    helpers::{HelperError, resolve_item},
    store::items::{Item, State, StoreError, active_session_id, load_items},
    workspace::resolve_workspace_root,
};

#[derive(Error, Debug)]
pub enum DiffError {
    #[error("{0}")]
    FileReadError(String),
    #[error("{0}")]
    FileWriteError(String),
    #[error("{0}")]
    DiffComputationError(String),
    #[error("{0}")]
    NoActiveSession(String),
    #[error("{0}")]
    FileNotInQueue(String),
    #[error("{0}")]
    HelperError(#[from] HelperError),
    #[error("{0}")]
    StoreError(#[from] StoreError),
    #[error("{0}")]
    UnknownError(String),
}

pub fn plumb_diff(target: Option<String>) -> Result<(), DiffError> {
    let cwd = std::env::current_dir().map_err(|e| DiffError::UnknownError(e.to_string()))?;
    let root = resolve_workspace_root(&cwd).map_err(|e| DiffError::UnknownError(e.to_string()))?;

    let session_id = active_session_id(&root).map_err(|_| {
        DiffError::NoActiveSession(
            "no active session found, use `plumb start` to start a session".to_string(),
        )
    })?;

    let items = load_items(&root)?;

    match target {
        Some(t) => plumb_diff_target(t, &root, &session_id, &items),
        None => plumb_diff_all(&root, &session_id, &items),
    }
}

fn plumb_diff_target(
    target: String,
    root: &Path,
    session_id: &str,
    items: &[Item],
) -> Result<(), DiffError> {
    let (item_id, normalized_path, state) =
        resolve_item(root, items, &target).map_err(|e| match e {
            HelperError::FileNotInQueue(msg) => DiffError::FileNotInQueue(msg),
            other => DiffError::HelperError(other),
        })?;

    if state == State::Todo {
        return Err(DiffError::FileReadError(
            "no baseline snapshot. Run `plumb go` first".to_string(),
        ));
    }

    let baseline_path = baseline_path(root, session_id, item_id);
    let baseline_bytes = load_baseline_bytes(&baseline_path)?;

    let current_path = root.join(&normalized_path);
    let current_bytes = load_current_bytes(&current_path)?;

    let diff = compute_display_diff(&normalized_path, &baseline_bytes, &current_bytes)?;
    if !diff.is_empty() {
        print!("{diff}");
    }

    Ok(())
}

fn plumb_diff_all(root: &Path, session_id: &str, items: &[Item]) -> Result<(), DiffError> {
    plumb_diff_all_with(root, session_id, items, plumb_diff_target)
}

fn plumb_diff_all_with<F>(
    root: &Path,
    session_id: &str,
    items: &[Item],
    mut diff_target_fn: F,
) -> Result<(), DiffError>
where
    F: FnMut(String, &Path, &str, &[Item]) -> Result<(), DiffError>,
{
    for (_, rel_path) in collect_in_progress_items(items) {
        diff_target_fn(rel_path, root, session_id, items)?;
    }

    Ok(())
}

fn collect_in_progress_items(items: &[Item]) -> Vec<(usize, String)> {
    items
        .iter()
        .filter(|item| item.state == State::InProgress)
        .map(|item| (item.id, item.rel_path.clone()))
        .collect()
}

fn baseline_path(root: &Path, session_id: &str, item_id: usize) -> PathBuf {
    root.join(".plumb")
        .join("sessions")
        .join(session_id)
        .join("snapshots")
        .join(format!("{}.baseline", item_id))
}

fn load_baseline_bytes(path: &Path) -> Result<Vec<u8>, DiffError> {
    std::fs::read(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            DiffError::FileReadError(
                "baseline snapshot not found on disk (this should not happen in normal use)"
                    .to_string(),
            )
        } else {
            DiffError::FileReadError(format!("failed to read baseline snapshot: {e}"))
        }
    })
}

fn load_current_bytes(path: &Path) -> Result<Vec<u8>, DiffError> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(DiffError::FileReadError(format!(
            "failed to read current file: {e}"
        ))),
    }
}

fn compute_display_diff(
    path_label: &str,
    baseline: &[u8],
    current: &[u8],
) -> Result<String, DiffError> {
    Ok(render_baseline_diff(path_label, baseline, current))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::items::save_items;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
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

        std::fs::create_dir_all(
            root.join(".plumb")
                .join("sessions")
                .join(&session_id)
                .join("snapshots"),
        )
        .unwrap();
        std::fs::write(root.join(".plumb").join("active"), &session_id).unwrap();
        save_items(root, &session_id, items).unwrap();

        (temp_dir, session_id)
    }

    #[test]
    fn load_current_bytes_missing_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let bytes = load_current_bytes(&tmp.path().join("missing.txt")).unwrap();
        assert!(bytes.is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn load_current_bytes_other_error_is_error() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("unreadable.txt");
        std::fs::write(&path, "secret").unwrap();

        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o000);
        std::fs::set_permissions(&path, perms).unwrap();

        let result = load_current_bytes(&path);

        let mut restore = std::fs::metadata(&path).unwrap().permissions();
        restore.set_mode(0o644);
        std::fs::set_permissions(&path, restore).unwrap();

        assert!(matches!(result, Err(DiffError::FileReadError(_))));
    }

    #[test]
    fn diff_target_errors_when_item_state_todo() {
        let items = vec![make_item(1, "src/a.txt", State::Todo)];
        let tmp = TempDir::new().unwrap();

        let err = plumb_diff_target("1".to_string(), tmp.path(), "deadbeef", &items).unwrap_err();

        assert!(
            matches!(err, DiffError::FileReadError(msg) if msg.contains("Run `plumb go` first"))
        );
    }

    #[test]
    fn diff_target_errors_when_baseline_missing_on_disk_even_if_state_in_progress() {
        let items = vec![make_item(1, "src/a.txt", State::InProgress)];
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        std::fs::write(tmp.path().join("src/a.txt"), "current").unwrap();

        let err = plumb_diff_target("1".to_string(), tmp.path(), "deadbeef", &items).unwrap_err();
        assert!(
            matches!(err, DiffError::FileReadError(msg) if msg.contains("baseline snapshot not found"))
        );
    }

    #[test]
    fn baseline_path_points_to_sessions_sid_snapshots_id_baseline() {
        let root = Path::new("/tmp/workspace");
        let path = baseline_path(root, "a1b2c3d4", 42);
        let expected = root
            .join(".plumb")
            .join("sessions")
            .join("a1b2c3d4")
            .join("snapshots")
            .join("42.baseline");
        assert_eq!(path, expected);
    }

    #[test]
    fn diff_all_only_runs_for_in_progress_items() {
        let items = vec![
            make_item(1, "todo.txt", State::Todo),
            make_item(2, "in_a.txt", State::InProgress),
            make_item(3, "done.txt", State::Done),
            make_item(4, "in_b.txt", State::InProgress),
        ];
        let mut seen_targets = Vec::new();

        plumb_diff_all_with(Path::new("/tmp"), "deadbeef", &items, |target, _, _, _| {
            seen_targets.push(target);
            Ok(())
        })
        .unwrap();

        assert_eq!(
            seen_targets,
            vec!["in_a.txt".to_string(), "in_b.txt".to_string()]
        );
    }

    #[test]
    fn diff_all_no_in_progress_is_noop_and_success() {
        let items = vec![
            make_item(1, "todo.txt", State::Todo),
            make_item(2, "done.txt", State::Done),
        ];
        let mut calls = 0usize;

        plumb_diff_all_with(Path::new("/tmp"), "deadbeef", &items, |_, _, _, _| {
            calls += 1;
            Ok(())
        })
        .unwrap();

        assert_eq!(calls, 0);
    }

    #[test]
    fn collect_in_progress_items_returns_ids_and_paths() {
        let items = vec![
            make_item(10, "a.txt", State::Todo),
            make_item(11, "b.txt", State::InProgress),
            make_item(12, "c.txt", State::InProgress),
        ];

        let in_progress = collect_in_progress_items(&items);
        assert_eq!(
            in_progress,
            vec![(11, "b.txt".to_string()), (12, "c.txt".to_string())]
        );
    }

    #[test]
    fn diff_target_succeeds_for_in_progress_item_with_existing_baseline() {
        let items = vec![make_item(1, "src/a.txt", State::InProgress)];
        let (workspace, session_id) = create_workspace_with_active_session(&items);
        let root = workspace.path();

        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/a.txt"), "now").unwrap();
        std::fs::write(baseline_path(root, &session_id, 1), "before").unwrap();

        let result = plumb_diff_target("1".to_string(), root, &session_id, &items);
        assert!(result.is_ok());
    }
}
