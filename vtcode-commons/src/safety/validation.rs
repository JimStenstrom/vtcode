//! Path validation and sanitization functions
//!
//! This module provides utilities for validating paths to ensure they are safe
//! for file operations and respect workspace boundaries.
//!
//! # Key Functions
//!
//! - [`validate_workspace_path`]: Validate a path is within workspace boundaries
//! - [`sanitize_path`]: Sanitize a path to prevent directory traversal attacks
//! - [`is_safe_filename`]: Check if a filename is safe (no special characters)
//! - [`normalize_path`]: Normalize a path by resolving `.` and `..` components

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf, Component};

/// Validate that a path is within workspace boundaries
///
/// This function ensures that the given file path, when resolved, stays within
/// the workspace directory. It prevents directory traversal attacks using `..`
/// and symlink attacks.
///
/// # Arguments
///
/// * `workspace` - The workspace root directory
/// * `file_path` - The file path to validate (can be relative or absolute)
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated, absolute path within the workspace
/// * `Err(_)` - If the path is invalid or outside the workspace
///
/// # Examples
///
/// ```rust,ignore
/// use vtcode_commons::safety::validation::validate_workspace_path;
/// use std::path::PathBuf;
///
/// let workspace = PathBuf::from("/home/user/project");
/// let file_path = PathBuf::from("src/main.rs");
///
/// match validate_workspace_path(&workspace, &file_path) {
///     Ok(valid_path) => println!("Safe path: {}", valid_path.display()),
///     Err(e) => eprintln!("Unsafe path: {}", e),
/// }
/// ```
///
/// # Security
///
/// This function protects against:
/// - Directory traversal: `../../etc/passwd`
/// - Symlink attacks: symlinks pointing outside workspace
/// - Absolute path injection: `/etc/passwd`
pub fn validate_workspace_path(workspace: &Path, file_path: &Path) -> Result<PathBuf> {
    // Ensure workspace is absolute
    let workspace_abs = if workspace.is_absolute() {
        workspace.to_path_buf()
    } else {
        workspace
            .canonicalize()
            .map_err(|e| anyhow!("Failed to canonicalize workspace '{}': {}", workspace.display(), e))?
    };

    // Resolve the file path
    let resolved = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        workspace_abs.join(file_path)
    };

    // Normalize the path to resolve . and .. components
    let normalized = normalize_path(&resolved);

    // For existing paths, canonicalize to resolve symlinks
    let final_path = if normalized.exists() {
        match normalized.canonicalize() {
            Ok(canonical) => canonical,
            Err(e) => {
                // If canonicalization fails, use normalized path but warn
                eprintln!(
                    "Warning: Could not canonicalize path '{}': {}",
                    normalized.display(),
                    e
                );
                normalized
            }
        }
    } else {
        // For non-existent paths, use the normalized path
        normalized
    };

    // Check if the final path is within the workspace
    if !final_path.starts_with(&workspace_abs) {
        return Err(anyhow!(
            "Path '{}' is outside workspace '{}'. Resolved to: '{}'",
            file_path.display(),
            workspace.display(),
            final_path.display()
        ));
    }

    Ok(final_path)
}

/// Sanitize a path by removing dangerous components
///
/// This function removes or replaces potentially dangerous path components
/// to prevent directory traversal and other path-based attacks.
///
/// # Arguments
///
/// * `path` - The path to sanitize
///
/// # Returns
///
/// A sanitized path with dangerous components removed
///
/// # Examples
///
/// ```rust
/// use vtcode_commons::safety::validation::sanitize_path;
/// use std::path::Path;
///
/// let dangerous = Path::new("../../etc/passwd");
/// let safe = sanitize_path(dangerous);
/// // Returns "etc/passwd" with traversal components removed
/// ```
pub fn sanitize_path(path: &Path) -> PathBuf {
    let mut sanitized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Normal(part) => {
                // Keep normal path components
                sanitized.push(part);
            }
            Component::RootDir => {
                // Keep root directory if present
                if sanitized.as_os_str().is_empty() {
                    sanitized.push(component);
                }
            }
            Component::CurDir => {
                // Skip current directory references (.)
                continue;
            }
            Component::ParentDir => {
                // Skip parent directory references (..)
                // This prevents directory traversal attacks
                continue;
            }
            Component::Prefix(_) => {
                // On Windows, keep the drive prefix
                if sanitized.as_os_str().is_empty() {
                    sanitized.push(component);
                }
            }
        }
    }

    sanitized
}

/// Normalize a path by resolving `.` and `..` components
///
/// Similar to `std::path::Path::canonicalize()`, but doesn't require the path
/// to exist and doesn't resolve symlinks.
///
/// # Arguments
///
/// * `path` - The path to normalize
///
/// # Returns
///
/// A normalized path with `.` and `..` resolved
///
/// # Examples
///
/// ```rust
/// use vtcode_commons::safety::validation::normalize_path;
/// use std::path::Path;
///
/// let path = Path::new("/home/user/project/../other/./file.txt");
/// let normalized = normalize_path(path);
/// // Returns "/home/user/other/file.txt"
/// ```
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::ParentDir => {
                // Pop the last component if possible
                if !normalized.pop() {
                    // If we can't pop, we're at the root, so just skip
                    continue;
                }
            }
            Component::CurDir => {
                // Skip current directory references
                continue;
            }
            _ => {
                normalized.push(component);
            }
        }
    }

    normalized
}

/// Check if a filename contains only safe characters
///
/// This function validates that a filename doesn't contain special characters
/// that could be dangerous or cause issues across different file systems.
///
/// # Arguments
///
/// * `filename` - The filename to check
///
/// # Returns
///
/// * `true` if the filename is safe
/// * `false` if it contains dangerous characters
///
/// # Examples
///
/// ```rust
/// use vtcode_commons::safety::validation::is_safe_filename;
///
/// assert!(is_safe_filename("file.txt"));
/// assert!(is_safe_filename("my_file-123.rs"));
/// assert!(!is_safe_filename("../etc/passwd"));
/// assert!(!is_safe_filename("file\0.txt")); // null byte
/// ```
pub fn is_safe_filename(filename: &str) -> bool {
    // Check for empty filename
    if filename.is_empty() {
        return false;
    }

    // Check for special directory names
    if filename == "." || filename == ".." {
        return false;
    }

    // Check for dangerous characters
    // Disallow: null bytes, path separators, and other special chars
    for ch in filename.chars() {
        if ch == '\0' || ch == '/' || ch == '\\' || ch < ' ' || ch == '\x7F' {
            return false;
        }
    }

    // Additional checks for Windows reserved names
    let name_upper = filename.to_uppercase();
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
        "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    // Check if filename matches a reserved name (with or without extension)
    let name_without_ext = if let Some(pos) = name_upper.find('.') {
        &name_upper[..pos]
    } else {
        &name_upper
    };

    if reserved.contains(&name_without_ext) {
        return false;
    }

    true
}

/// Check if a path contains any parent directory (`..`) components
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Returns
///
/// * `true` if the path contains `..` components
/// * `false` otherwise
///
/// # Examples
///
/// ```rust
/// use vtcode_commons::safety::validation::has_parent_components;
/// use std::path::Path;
///
/// assert!(has_parent_components(Path::new("../file.txt")));
/// assert!(has_parent_components(Path::new("dir/../../file.txt")));
/// assert!(!has_parent_components(Path::new("dir/file.txt")));
/// ```
pub fn has_parent_components(path: &Path) -> bool {
    path.components()
        .any(|c| matches!(c, Component::ParentDir))
}

/// Ensure a path has a specific file extension
///
/// # Arguments
///
/// * `path` - The path to check
/// * `extension` - The required extension (without the dot)
///
/// # Returns
///
/// * `Ok(())` if the path has the required extension
/// * `Err(_)` if it doesn't
///
/// # Examples
///
/// ```rust,ignore
/// use vtcode_commons::safety::validation::ensure_extension;
/// use std::path::Path;
///
/// let path = Path::new("file.txt");
/// assert!(ensure_extension(path, "txt").is_ok());
/// assert!(ensure_extension(path, "rs").is_err());
/// ```
pub fn ensure_extension(path: &Path, extension: &str) -> Result<()> {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if ext == extension => Ok(()),
        Some(ext) => Err(anyhow!(
            "Path '{}' has extension '{}', expected '{}'",
            path.display(),
            ext,
            extension
        )),
        None => Err(anyhow!(
            "Path '{}' has no extension, expected '{}'",
            path.display(),
            extension
        )),
    }
}

/// Check if a path is absolute
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Returns
///
/// * `true` if the path is absolute
/// * `false` if it's relative
///
/// # Examples
///
/// ```rust
/// use vtcode_commons::safety::validation::is_absolute;
/// use std::path::Path;
///
/// assert!(is_absolute(Path::new("/home/user/file.txt")));
/// assert!(!is_absolute(Path::new("relative/file.txt")));
/// ```
pub fn is_absolute(path: &Path) -> bool {
    path.is_absolute()
}

/// Convert a path to an absolute path relative to a base directory
///
/// # Arguments
///
/// * `base` - The base directory
/// * `path` - The path to make absolute
///
/// # Returns
///
/// An absolute path
///
/// # Examples
///
/// ```rust
/// use vtcode_commons::safety::validation::make_absolute;
/// use std::path::Path;
///
/// let base = Path::new("/home/user/project");
/// let relative = Path::new("src/main.rs");
/// let absolute = make_absolute(base, relative);
/// // Returns "/home/user/project/src/main.rs"
/// ```
pub fn make_absolute(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_workspace_path_success() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = temp_dir.path();

        // Create a test file
        let file = workspace.join("test.txt");
        fs::write(&file, "test").unwrap();

        // Should succeed for file within workspace
        let result = validate_workspace_path(workspace, &file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_workspace_path_traversal() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = temp_dir.path();

        // Try to escape workspace with ..
        let malicious = Path::new("../../etc/passwd");
        let result = validate_workspace_path(workspace, malicious);

        // Should fail - path escapes workspace
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_path() {
        let dangerous = Path::new("../../etc/passwd");
        let sanitized = sanitize_path(dangerous);

        // Should remove .. components
        assert!(!sanitized.to_string_lossy().contains(".."));
        assert_eq!(sanitized, PathBuf::from("etc/passwd"));
    }

    #[test]
    fn test_normalize_path() {
        let path = Path::new("/home/user/project/../other/./file.txt");
        let normalized = normalize_path(path);

        assert_eq!(normalized, PathBuf::from("/home/user/other/file.txt"));
    }

    #[test]
    fn test_is_safe_filename() {
        // Safe filenames
        assert!(is_safe_filename("file.txt"));
        assert!(is_safe_filename("my-file_123.rs"));
        assert!(is_safe_filename("README.md"));

        // Unsafe filenames
        assert!(!is_safe_filename(""));
        assert!(!is_safe_filename("."));
        assert!(!is_safe_filename(".."));
        assert!(!is_safe_filename("file/path.txt"));
        assert!(!is_safe_filename("file\0.txt"));
        assert!(!is_safe_filename("CON"));
        assert!(!is_safe_filename("PRN.txt"));
    }

    #[test]
    fn test_has_parent_components() {
        assert!(has_parent_components(Path::new("../file.txt")));
        assert!(has_parent_components(Path::new("dir/../../file.txt")));
        assert!(!has_parent_components(Path::new("dir/file.txt")));
        assert!(!has_parent_components(Path::new("/absolute/path")));
    }

    #[test]
    fn test_ensure_extension() {
        let path = Path::new("file.txt");
        assert!(ensure_extension(path, "txt").is_ok());
        assert!(ensure_extension(path, "rs").is_err());

        let no_ext = Path::new("file");
        assert!(ensure_extension(no_ext, "txt").is_err());
    }

    #[test]
    fn test_is_absolute() {
        assert!(is_absolute(Path::new("/home/user/file.txt")));
        assert!(!is_absolute(Path::new("relative/file.txt")));
    }

    #[test]
    fn test_make_absolute() {
        let base = Path::new("/home/user/project");
        let relative = Path::new("src/main.rs");
        let absolute = make_absolute(base, relative);

        assert_eq!(absolute, PathBuf::from("/home/user/project/src/main.rs"));

        // Already absolute path should remain unchanged
        let abs_path = Path::new("/tmp/file.txt");
        let result = make_absolute(base, abs_path);
        assert_eq!(result, abs_path);
    }
}
