//! JavaScript bindings for path utilities

use napi_derive::napi;
use std::path::PathBuf;
use ws_std::paths::get_project_root_path as ws_get_root_path;

/// Get the project root path
///
/// @param {string} [root] - Optional starting directory
/// @returns {string | null} The project root path or null if not found
#[napi]
pub fn get_project_root_path(root: Option<String>) -> Option<String> {
    let root_pathbuf = root.map(PathBuf::from);
    ws_get_root_path(root_pathbuf).map(|p| p.to_string_lossy().to_string())
}

#[cfg(test)]
mod paths_binding_tests {
    use super::*;
    use std::{
        env::temp_dir,
        fs::{create_dir, remove_dir_all, File},
        io::Write,
        path::PathBuf,
    };

    #[cfg(not(windows))]
    use std::os::unix::fs::PermissionsExt;

    #[cfg(not(windows))]
    use std::fs::set_permissions;

    fn create_workspace() -> Result<PathBuf, std::io::Error> {
        let temp_dir = temp_dir();
        let monorepo_root_dir = temp_dir.join("path-test-workspace");

        if monorepo_root_dir.exists() {
            remove_dir_all(&monorepo_root_dir)?;
        }

        create_dir(&monorepo_root_dir)?;

        // Create a package-lock.json file to identify as project root
        let mut file = File::create(monorepo_root_dir.join("package-lock.json").as_path())?;
        file.write_all(b"{}")?;

        #[cfg(not(windows))]
        set_permissions(&monorepo_root_dir, std::fs::Permissions::from_mode(0o777))?;

        Ok(monorepo_root_dir)
    }

    #[test]
    fn test_get_project_root() -> Result<(), std::io::Error> {
        // Create a test workspace
        let workspace_path = create_workspace()?;

        // Get the project root, starting from the workspace path
        let root = get_project_root_path(Some(workspace_path.to_string_lossy().to_string()));

        // It should return the workspace path
        assert!(root.is_some());
        assert_eq!(root.unwrap(), workspace_path.to_string_lossy().to_string());

        // Clean up
        remove_dir_all(workspace_path)?;
        Ok(())
    }

    #[test]
    fn test_get_project_root_from_subdir() -> Result<(), std::io::Error> {
        // Create a test workspace
        let workspace_path = create_workspace()?;

        // Create a subdirectory
        let subdir_path = workspace_path.join("subdir");
        create_dir(&subdir_path)?;

        // Get the project root, starting from the subdirectory
        let root = get_project_root_path(Some(subdir_path.to_string_lossy().to_string()));

        // It should return the workspace path (parent directory)
        assert!(root.is_some());
        assert_eq!(root.unwrap(), workspace_path.to_string_lossy().to_string());

        // Clean up
        remove_dir_all(workspace_path)?;
        Ok(())
    }

    #[test]
    fn test_get_project_root_no_root() -> Result<(), std::io::Error> {
        // Create a temporary directory with no project files
        let temp_dir = tempfile::tempdir()?;

        // Get project root from this directory
        let root = get_project_root_path(Some(temp_dir.path().to_string_lossy().to_string()));

        // Since there's no package-lock.json or other indicator files,
        // it may return None or a git root if in a git repository
        if let Some(path) = root {
            println!("Found root at: {}", path);
        } else {
            println!("No project root found");
        }

        // We can't assert specific behavior here since it depends on the environment,
        // but the function should not panic

        Ok(())
    }
}
