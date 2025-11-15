//! Path resolution and project identification utilities

use std::fs;
use std::path::{Path, PathBuf};

/// Determine project-specific config path if it exists
///
/// Looks for configuration in `.vtcode/projects/<project-name>/config/<config-file>`
pub fn project_config_path(
    config_dir: &Path,
    workspace_root: &Path,
    config_file_name: &str,
) -> Option<PathBuf> {
    let project_name = identify_current_project(workspace_root)?;
    let project_config_path = config_dir
        .join("projects")
        .join(project_name)
        .join("config")
        .join(config_file_name);

    if project_config_path.exists() {
        Some(project_config_path)
    } else {
        None
    }
}

/// Identify the current project name
///
/// First checks for `.vtcode-project` file, falls back to directory name
pub fn identify_current_project(workspace_root: &Path) -> Option<String> {
    let project_file = workspace_root.join(".vtcode-project");
    if let Ok(contents) = fs::read_to_string(&project_file) {
        let name = contents.trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }

    workspace_root
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
}

/// Ensure parent directory exists for a given path
///
/// This is a re-export of the bootstrap module's function for convenience
#[cfg(feature = "bootstrap")]
pub use super::bootstrap::ensure_parent_dir;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn identify_project_from_directory_name() {
        let temp_dir = assert_fs::TempDir::new().expect("failed to create temp dir");
        let workspace = temp_dir.path().join("my-project");
        fs::create_dir(&workspace).expect("failed to create workspace");

        let project_name = identify_current_project(&workspace);
        assert_eq!(project_name, Some("my-project".to_string()));
    }

    #[test]
    fn identify_project_from_vtcode_project_file() {
        let temp_dir = assert_fs::TempDir::new().expect("failed to create temp dir");
        let workspace = temp_dir.path();

        fs::write(workspace.join(".vtcode-project"), "custom-name")
            .expect("failed to write project file");

        let project_name = identify_current_project(workspace);
        assert_eq!(project_name, Some("custom-name".to_string()));
    }

    #[test]
    fn project_config_path_when_exists() {
        let temp_dir = assert_fs::TempDir::new().expect("failed to create temp dir");
        let workspace = temp_dir.path();
        let config_dir = workspace.join(".vtcode");

        // Create project config structure
        let project_config_dir = config_dir
            .join("projects")
            .join("my-project")
            .join("config");
        fs::create_dir_all(&project_config_dir).expect("failed to create config dir");

        let config_file = project_config_dir.join("vtcode.toml");
        fs::write(&config_file, "# test config").expect("failed to write config");

        fs::write(workspace.join(".vtcode-project"), "my-project")
            .expect("failed to write project file");

        let result = project_config_path(&config_dir, workspace, "vtcode.toml");
        assert_eq!(result, Some(config_file));
    }

    #[test]
    fn project_config_path_when_not_exists() {
        let temp_dir = assert_fs::TempDir::new().expect("failed to create temp dir");
        let workspace = temp_dir.path();
        let config_dir = workspace.join(".vtcode");
        fs::create_dir_all(&config_dir).expect("failed to create config dir");

        let result = project_config_path(&config_dir, workspace, "vtcode.toml");
        assert_eq!(result, None);
    }
}
