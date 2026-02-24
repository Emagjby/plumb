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

fn lexical_normalize(path: &Path) -> PathBuf {
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

fn to_slash_path(path: &Path) -> String {
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
