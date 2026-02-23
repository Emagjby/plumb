use std::path::{Component, Path, PathBuf};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum InputError {
    #[error("invalid input path: {0}")]
    InvalidPath(String),
    #[error("path escapes workspace root: {0}")]
    EscapesRoot(String),
}

pub fn normalize_rel_path(root: &Path, input: &Path) -> Result<String, InputError> {
    let cwd = std::env::current_dir()
        .map_err(|e| InputError::InvalidPath(format!("failed to get current directory: {}", e)))?;

    let full_path = if input.is_absolute() {
        input.to_path_buf()
    } else {
        cwd.join(input)
    };

    let canonical_root = root
        .canonicalize()
        .map_err(|e| InputError::InvalidPath(format!("failed to canonicalize root: {}", e)))?;

    let canonical_input = match full_path.canonicalize() {
        Ok(p) => p,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let resolved = normalize_without_canonicalize(&full_path)?;
            check_path_within_root(&canonical_root, &resolved)?;
            let relative = resolved
                .strip_prefix(&canonical_root)
                .map_err(|_| InputError::EscapesRoot(resolved.display().to_string()))?;
            return Ok(normalize_separators(relative));
        }
        Err(e) => {
            return Err(InputError::InvalidPath(format!(
                "failed to canonicalize path: {}",
                e
            )))
        }
    };

    check_path_within_root(&canonical_root, &canonical_input)?;
    let relative = canonical_input
        .strip_prefix(&canonical_root)
        .map_err(|_| InputError::EscapesRoot(canonical_input.display().to_string()))?;

    Ok(normalize_separators(relative))
}

fn normalize_without_canonicalize(path: &Path) -> Result<PathBuf, InputError> {
    let mut components = path.components().peekable();
    let mut normalized = PathBuf::new();

    while let Some(comp) = components.next() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(InputError::EscapesRoot(path.display().to_string()));
                }
            }
            _ => normalized.push(comp.as_os_str()),
        }
    }

    if normalized.as_os_str().is_empty() {
        normalized.push(".");
    }

    Ok(normalized)
}

fn check_path_within_root(root: &Path, input: &Path) -> Result<(), InputError> {
    let root_parts: Vec<_> = root.components().collect();
    let input_parts: Vec<_> = input.components().collect();

    if input_parts.len() < root_parts.len() {
        return Err(InputError::EscapesRoot(input.display().to_string()));
    }

    for (root_part, input_part) in root_parts.iter().zip(input_parts.iter()) {
        if root_part != input_part {
            return Err(InputError::EscapesRoot(input.display().to_string()));
        }
    }

    Ok(())
}

fn normalize_separators(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string()
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
}
