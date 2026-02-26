use std::path::Path;

use thiserror::Error;

use crate::{
    fs::normalize_rel_path,
    store::items::{Item, State},
};

#[derive(Debug, Error)]
pub enum HelperError {
    #[error("{0}")]
    PathNormalizationError(String),
    #[error("{0}")]
    FileNotInQueue(String),
    #[error("{0}")]
    BaselineReadError(String),
    #[error("{0}")]
    UnknownError(String),
}

pub fn load_baseline(root: &Path, session_id: &str, target: usize) -> Result<Vec<u8>, HelperError> {
    let snapshots_path = root
        .join(".plumb")
        .join("sessions")
        .join(session_id)
        .join("snapshots");

    let baseline_path = snapshots_path.join(format!("{}.baseline", target));
    std::fs::read(&baseline_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            HelperError::BaselineReadError(
                "baseline snapshot not found on disk (this should not happen in normal use)"
                    .to_string(),
            )
        } else {
            HelperError::BaselineReadError(format!("failed to read baseline snapshot: {e}"))
        }
    })
}

pub fn resolve_item(
    root: &Path,
    items: &[Item],
    target: &str,
) -> Result<(usize, String, State), HelperError> {
    if target.chars().all(|c| c.is_ascii_digit())
        && let Ok(id) = target.parse::<usize>()
    {
        if let Some(item) = items.iter().find(|item| item.id == id) {
            return Ok((item.id, item.rel_path.clone(), item.state.clone()));
        }

        return Err(HelperError::FileNotInQueue(format!(
            "no file with ID {} in queue",
            id
        )));
    }

    let normalized_path = normalize_rel_path(root, Path::new(target))
        .map_err(|e| HelperError::PathNormalizationError(e.to_string()))?;

    let item = items
        .iter()
        .find(|item| item.rel_path == normalized_path)
        .ok_or_else(|| {
            HelperError::FileNotInQueue(format!("file not in queue: {}", normalized_path))
        })?;

    Ok((item.id, normalized_path, item.state.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_item(id: usize, rel_path: &str, state: State) -> Item {
        Item {
            id,
            rel_path: rel_path.to_string(),
            state,
        }
    }

    #[test]
    fn resolve_item_by_id_returns_id_path_state() {
        let root = TempDir::new().unwrap();
        let items = vec![make_item(1, "src/a.rs", State::Todo)];

        let result = resolve_item(root.path(), &items, "1").unwrap();
        assert_eq!(result.0, 1);
        assert_eq!(result.1, "src/a.rs");
        assert_eq!(result.2, State::Todo);
    }

    #[test]
    fn resolve_item_by_path_normalizes_and_finds_item() {
        let root = TempDir::new().unwrap();
        let items = vec![make_item(3, "src/a.rs", State::InProgress)];
        let absolute_target = root.path().join("src/./a.rs");

        let result = resolve_item(root.path(), &items, &absolute_target.to_string_lossy()).unwrap();
        assert_eq!(result.0, 3);
        assert_eq!(result.1, "src/a.rs");
        assert_eq!(result.2, State::InProgress);
    }

    #[test]
    fn resolve_item_returns_file_not_in_queue_for_unknown_target() {
        let root = TempDir::new().unwrap();
        let items = vec![make_item(1, "src/a.rs", State::Todo)];
        let unknown_target = root.path().join("src/b.rs");

        let err = resolve_item(root.path(), &items, &unknown_target.to_string_lossy()).unwrap_err();
        assert!(matches!(err, HelperError::FileNotInQueue(msg) if msg.contains("src/b.rs")));
    }

    #[test]
    fn resolve_item_returns_path_normalization_error_on_bad_path() {
        let root = TempDir::new().unwrap();
        let items = vec![make_item(1, "src/a.rs", State::Todo)];
        let bad_target = root.path().parent().unwrap().join("outside.rs");

        let err = resolve_item(root.path(), &items, &bad_target.to_string_lossy()).unwrap_err();
        assert!(
            matches!(err, HelperError::PathNormalizationError(msg) if msg.contains("outside.rs"))
        );
    }

    #[test]
    fn resolve_item_numeric_unknown_target_returns_file_not_in_queue() {
        let root = TempDir::new().unwrap();
        let items = vec![make_item(1, "src/a.rs", State::Todo)];

        let err = resolve_item(root.path(), &items, "99").unwrap_err();
        assert!(matches!(err, HelperError::FileNotInQueue(msg) if msg.contains("99")));
    }

    #[test]
    fn load_baseline_reads_exact_bytes() {
        let root = TempDir::new().unwrap();
        let snapshots = root
            .path()
            .join(".plumb")
            .join("sessions")
            .join("deadbeef")
            .join("snapshots");
        fs::create_dir_all(&snapshots).unwrap();

        let expected = vec![0_u8, 159, 255, 10, b'a', b'\n'];
        fs::write(snapshots.join("7.baseline"), &expected).unwrap();

        let baseline = load_baseline(root.path(), "deadbeef", 7).unwrap();
        assert_eq!(baseline, expected);
    }

    #[test]
    fn load_baseline_missing_returns_not_found_error() {
        let root = TempDir::new().unwrap();

        let err = load_baseline(root.path(), "deadbeef", 7).unwrap_err();
        assert!(
            matches!(err, HelperError::BaselineReadError(msg) if msg.contains("baseline snapshot not found"))
        );
    }
}
