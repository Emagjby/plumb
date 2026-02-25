use std::path::{Component, Path, PathBuf};

use atomicwrites::{AtomicFile, OverwriteBehavior};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InputError {
    #[error("invalid input path: {0}")]
    InvalidPath(String),
    #[error("path escapes workspace root: {0}")]
    EscapesRoot(String),
}

#[derive(Error, Debug)]
pub enum FsError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("atomic write error: {0}")]
    AtomicWriteError(String),
}

pub fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), FsError> {
    let af = AtomicFile::new(path, OverwriteBehavior::AllowOverwrite);
    af.write(|f| std::io::Write::write_all(f, contents))
        .map_err(|e| FsError::AtomicWriteError(e.to_string()))?;

    Ok(())
}

pub fn normalize_rel_path(root: &Path, input: &Path) -> Result<String, InputError> {
    let cwd = std::env::current_dir()
        .map_err(|e| InputError::InvalidPath(format!("failed to get current directory: {e}")))?;

    normalize_rel_path_from_cwd(root, input, &cwd)
}

pub(crate) fn normalize_rel_path_from_cwd(
    root: &Path,
    input: &Path,
    cwd: &Path,
) -> Result<String, InputError> {
    let root_abs = if root.is_absolute() {
        root.to_path_buf()
    } else {
        cwd.join(root)
    };

    let input_abs = if input.is_absolute() {
        input.to_path_buf()
    } else {
        cwd.join(input)
    };

    let root_norm = lexical_normalize(&root_abs);
    let input_norm = lexical_normalize(&input_abs);

    if !input_norm.starts_with(&root_norm) {
        return Err(InputError::EscapesRoot(input.to_string_lossy().to_string()));
    }

    let rel = input_norm
        .strip_prefix(&root_norm)
        .map_err(|_| InputError::EscapesRoot(input.to_string_lossy().to_string()))?;

    Ok(to_slash_path(rel))
}

pub fn collect_folder_files(path: &Path) -> Result<Vec<PathBuf>, FsError> {
    if !path.is_dir() {
        return Err(FsError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("path is not a directory: {}", path.to_string_lossy()),
        )));
    }

    let mut files = Vec::new();

    let should_skip_dir = |p: &Path| {
        matches!(
            p.file_name().and_then(|n| n.to_str()),
            Some(".plumb" | ".git" | "target" | "node_modules")
        )
    };

    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !should_skip_dir(e.path()))
    {
        let entry = entry.map_err(|e| FsError::IoError(e.into()))?;
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort_by_cached_key(|p| to_slash_path(&lexical_normalize(p)));

    Ok(files)
}

pub(crate) fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();

    for comp in path.components() {
        match comp {
            Component::Prefix(p) => out.push(p.as_os_str()),
            Component::RootDir => out.push(comp.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            Component::Normal(p) => out.push(p),
        }
    }

    out
}

pub(crate) fn to_slash_path(path: &Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    let s = s.trim_start_matches('/').to_string();

    if s.is_empty() {
        ".".to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::{PermissionsExt, symlink};
    use tempfile::TempDir;

    #[test]
    fn test_normalize_rel_path_simple() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "").unwrap();

        let result = normalize_rel_path(root, root.join("src/main.rs").as_path()).unwrap();
        assert_eq!(result, "src/main.rs");
    }

    #[test]
    fn test_normalize_rel_path_with_dot_prefix() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "").unwrap();

        let result = normalize_rel_path(root, root.join("./src/main.rs").as_path()).unwrap();
        assert_eq!(result, "src/main.rs");
    }

    #[test]
    fn test_normalize_rel_path_escapes_root() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let outside = root.parent().unwrap().join("outside");
        let result = normalize_rel_path(root, &outside);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), InputError::EscapesRoot(_)));
    }

    #[test]
    fn test_normalize_rel_path_nonexistent_file() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let input = root.join("nonexistent/file.rs");
        let result = normalize_rel_path(root, &input).unwrap();
        assert_eq!(result, "nonexistent/file.rs");
    }

    #[test]
    fn test_normalize_rel_path_dot_prefix_absolute() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "").unwrap();

        let input = root.join("./src/main.rs");

        let result = normalize_rel_path(root, &input).unwrap();
        assert_eq!(result, "src/main.rs");
    }

    #[test]
    fn test_normalize_rel_path_windows_separators() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "").unwrap();

        let input_with_backslash = root.join("src\\main.rs");
        let result = normalize_rel_path(root, &input_with_backslash).unwrap();
        assert_eq!(result, "src/main.rs");
    }

    #[test]
    fn test_normalize_rel_path_from_subdirectory() {
        // Simulates: root=/project, user is in /project/test, types "t3"
        // The absolute resolved path would be /project/test/t3
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("test")).unwrap();

        let input = root.join("test/t3");
        let result = normalize_rel_path(root, &input).unwrap();
        assert_eq!(result, "test/t3");
    }

    #[test]
    fn test_normalize_rel_path_both_forms_match() {
        // "plumb add test/t3" from root and "plumb add t3" from test/ should
        // produce the same workspace-relative path
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("test")).unwrap();

        let from_root = root.join("test/t3");
        let from_subdir = root.join("test/t3"); // same absolute after cwd resolution

        let result_a = normalize_rel_path(root, &from_root).unwrap();
        let result_b = normalize_rel_path(root, &from_subdir).unwrap();
        assert_eq!(result_a, result_b);
        assert_eq!(result_a, "test/t3");
    }

    #[test]
    fn test_normalize_rel_path_with_parent_traversal() {
        // Simulates: root=/project, user is in /project/test, types "../src/main.rs"
        // Resolved absolute: /project/test/../src/main.rs -> /project/src/main.rs
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();

        let input = root.join("test/../src/main.rs");
        let result = normalize_rel_path(root, &input).unwrap();
        assert_eq!(result, "src/main.rs");
    }

    #[test]
    fn test_normalize_rel_path_from_nested_cwd() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        fs::create_dir_all(root.join("nested/deep")).unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(root.join("nested/deep")).unwrap();

        let result = normalize_rel_path(&root, Path::new("t3"));

        std::env::set_current_dir(&original_cwd).unwrap();
        drop(tmp);

        let result = result.unwrap();
        assert_eq!(result, "nested/deep/t3");
    }

    #[test]
    fn collect_folder_files_rejects_non_dir_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.rs");
        fs::write(&file_path, "").unwrap();

        let result = collect_folder_files(&file_path);
        let err = result.unwrap_err();
        assert!(matches!(err, FsError::IoError(_)));
    }

    #[test]
    fn collect_folder_files_excludes_dirs_anywhere_in_tree() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::create_dir_all(root.join("src/deep")).unwrap();
        fs::create_dir_all(root.join("src/.git")).unwrap();
        fs::create_dir_all(root.join("src/node_modules/pkg")).unwrap();
        fs::create_dir_all(root.join("src/target/build")).unwrap();
        fs::create_dir_all(root.join("src/.plumb/cache")).unwrap();

        fs::write(root.join("src/deep/keep.rs"), "").unwrap();
        fs::write(root.join("src/.git/ignore.rs"), "").unwrap();
        fs::write(root.join("src/node_modules/pkg/index.js"), "").unwrap();
        fs::write(root.join("src/target/build/tmp.rs"), "").unwrap();
        fs::write(root.join("src/.plumb/cache/meta.rs"), "").unwrap();

        let files = collect_folder_files(root).unwrap();
        let rels: Vec<_> = files
            .iter()
            .map(|p| to_slash_path(p.strip_prefix(root).unwrap()))
            .collect();

        assert_eq!(rels, vec!["src/deep/keep.rs"]);
    }

    #[test]
    fn collect_folder_files_excludes_when_root_is_excluded_dir() {
        let temp_dir = TempDir::new().unwrap();
        let excluded_root = temp_dir.path().join(".git");
        fs::create_dir_all(&excluded_root).unwrap();
        fs::write(excluded_root.join("config"), "").unwrap();

        let files = collect_folder_files(&excluded_root).unwrap();
        assert!(
            files.is_empty(),
            "excluded root directory should not yield files"
        );
    }

    #[test]
    fn collect_folder_files_sorts_by_normalized_workspace_relative_slash_path() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::create_dir_all(root.join("src/nested")).unwrap();
        fs::create_dir_all(root.join("a")).unwrap();
        fs::write(root.join("src/nested/z.rs"), "").unwrap();
        fs::write(root.join("src/a.rs"), "").unwrap();
        fs::write(root.join("a/main.rs"), "").unwrap();

        let files = collect_folder_files(root).unwrap();
        let rels: Vec<_> = files
            .iter()
            .map(|p| to_slash_path(p.strip_prefix(root).unwrap()))
            .collect();

        assert_eq!(rels, vec!["a/main.rs", "src/a.rs", "src/nested/z.rs"]);
    }

    #[cfg(unix)]
    #[test]
    fn walkdir_does_not_follow_symlink_dirs_no_cycles() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::create_dir_all(root.join("real")).unwrap();
        fs::write(root.join("real/file.rs"), "").unwrap();
        symlink(root, root.join("real/loop")).unwrap();

        let files = collect_folder_files(root).unwrap();
        let rels: Vec<_> = files
            .iter()
            .map(|p| to_slash_path(p.strip_prefix(root).unwrap()))
            .collect();

        assert_eq!(rels, vec!["real/file.rs"]);
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_file_policy_is_consistent() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("real.rs"), "").unwrap();
        symlink(root.join("real.rs"), root.join("linked.rs")).unwrap();

        let files = collect_folder_files(root).unwrap();
        let rels: Vec<_> = files
            .iter()
            .map(|p| to_slash_path(p.strip_prefix(root).unwrap()))
            .collect();

        assert!(rels.contains(&"real.rs".to_string()));
        assert!(
            !rels.contains(&"linked.rs".to_string()),
            "symlinked file entries should be skipped"
        );
    }

    #[cfg(unix)]
    #[test]
    fn collect_folder_files_unreadable_entry_behavior() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let blocked = root.join("blocked");
        fs::create_dir_all(blocked.join("inner")).unwrap();
        fs::write(blocked.join("inner/secret.rs"), "").unwrap();
        fs::set_permissions(&blocked, fs::Permissions::from_mode(0o000)).unwrap();

        let result = collect_folder_files(root);

        fs::set_permissions(&blocked, fs::Permissions::from_mode(0o755)).unwrap();

        match result {
            Ok(files) => {
                let rels: Vec<_> = files
                    .iter()
                    .map(|p| to_slash_path(p.strip_prefix(root).unwrap()))
                    .collect();
                assert!(
                    !rels.iter().any(|p| p.starts_with("blocked/")),
                    "unreadable entries should be skipped if traversal continues"
                );
            }
            Err(FsError::IoError(_)) => {}
            Err(other) => panic!("unexpected error variant: {other}"),
        }
    }

    #[test]
    fn test_atomic_write_overwrites_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        fs::write(&file_path, "old").unwrap();

        atomic_write(&file_path, b"new").unwrap();

        let contents = fs::read_to_string(file_path).unwrap();
        assert_eq!(contents, "new");
    }

    #[test]
    fn test_atomic_write_creates_file_if_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("newfile.txt");

        atomic_write(&file_path, b"content").unwrap();

        let contents = fs::read_to_string(file_path).unwrap();
        assert_eq!(contents, "content");
    }

    #[test]
    fn test_atomic_write_does_not_leave_tmp_files() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.txt");

        atomic_write(&file_path, b"data").unwrap();

        let tmp_files: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("file.txt"))
            .collect();

        assert_eq!(tmp_files.len(), 1);
    }
}
