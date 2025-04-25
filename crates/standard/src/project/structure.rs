        fs.add_dir(root.join("node_modules")); // Add node_modules dir
        fs
    }

    #[test]
    fn test_project_detection_mock_fs() -> StandardResult<()> {
        let root = PathBuf::from("/mock_project");
        let fs = setup_mock_fs(&root);
        let manager = ProjectManager::with_filesystem(fs);

        let project = manager.detect_project(&root, ProjectConfig::new())?;

        assert_eq!(project.root(), root.as_path());
        assert!(project.package_manager().is_some());
        assert_eq!(project.package_manager().unwrap().kind(), super::PackageManagerKind::Yarn);
        assert!(project.package_json().is_some());
        assert_eq!(project.package_json().unwrap().name, "test-project");
        assert!(matches!(project.validation_status(), ValidationStatus::Valid)); // Should be valid now

        Ok(())
    }

     #[test]
     fn test_project_validation_mock_fs() -> StandardResult<()> {
         let root = PathBuf::from("/mock_project");
         let mut fs = setup_mock_fs(&root);
         // Remove node_modules to trigger warning
         fs.dirs.remove(&root.join("node_modules"));

         let manager = ProjectManager::with_filesystem(fs);
         let mut project = manager.detect_project(&root, ProjectConfig::new())?; // detect also validates if configured

         assert!(matches!(project.validation_status(), ValidationStatus::Warning(_)));
         if let ValidationStatus::Warning(warnings) = project.validation_status() {
             assert!(warnings.iter().any(|w| w.contains("Missing node_modules")));
         }

         // Test error case: Invalid package.json
         let mut fs_invalid = MockFileSystem::default();
         fs_invalid.add_file(root.join("package.json"), "{ invalid json }");
         fs_invalid.add_file(root.join("yarn.lock"), "");
         let manager_invalid = ProjectManager::with_filesystem(fs_invalid);
         let result = manager_invalid.detect_project(&root, ProjectConfig::new());
         assert!(result.is_err());
         assert!(matches!(result.unwrap_err(), StandardError::Json(_)));


         // Test error case: node_modules is a file
         let mut fs_file_node_modules = setup_mock_fs(&root);
         fs_file_node_modules.dirs.remove(&root.join("node_modules"));
         fs_file_node_modules.add_file(root.join("node_modules"), "");
         let manager_file_node_modules = ProjectManager::with_filesystem(fs_file_node_modules);
         let mut project_file_node_modules = manager_file_node_modules.detect_project(&root, ProjectConfig::new())?;
         assert!(matches!(project_file_node_modules.validation_status(), ValidationStatus::Error(_)));
          if let ValidationStatus::Error(errors) = project_file_node_modules.validation_status() {
             assert!(errors.iter().any(|e| e.contains("node_modules exists but is not a directory")));
         }

         Ok(())
     }

    // Keep TempDir tests for integration with real filesystem
     #[test]
     fn test_project_detection_real_fs() -> StandardResult<()> {
         let temp_dir = TempDir::new().unwrap();
         create_test_project_real(temp_dir.path())?; // Use real FS helper

         let manager = ProjectManager::new(); // Uses real FS
         let project = manager.detect_project(temp_dir.path(), ProjectConfig::new())?;

         // Adjust validation assertion based on real FS state (node_modules won't exist)
         assert!(matches!(project.validation_status(), ValidationStatus::Warning(_)));
         assert!(project.package_manager().is_some());
         assert_eq!(project.package_json().unwrap().name, "test-project");

         Ok(())
     }

    // Helper for real FS tests
    fn create_test_project_real(dir: &Path) -> StandardResult<()> {
        let fs = FileSystemManager::new();
        let package_json = r#"{
             "name": "test-project",
             "version": "1.0.0",
             "dependencies": { "test-dep": "^1.0.0" }
         }"#;
         fs.write_file_string(&dir.join("package.json"), package_json)?;
         fs.write_file_string(&dir.join("package-lock.json"), "{}")?; // npm lock file
         Ok(())
     }
}
//! Project structure management for Node.js projects.

use std::{
    fs as std_fs, // Alias std::fs to avoid conflict with fs field
    io, // Import io for metadata error check
    path::{Path, PathBuf},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    FileSystem, FileSystemManager, PackageManager, ProjectConfig,
};
use crate::error::{FileSystemError, StandardError, StandardResult};

/// Validation status of a project
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationStatus {
    Valid,
    Warning(Vec<String>),
    Error(Vec<String>),
    NotValidated,
}

/// Package.json content structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageJson {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    #[serde(default)]
    pub dev_dependencies: HashMap<String, String>,
    #[serde(default)]
    pub scripts: HashMap<String, String>,
}

/// Represents a Node.js project
#[derive(Debug)]
pub struct Project {
    root: PathBuf,
    package_manager: Option<PackageManager>,
    config: ProjectConfig,
    validation: ValidationStatus,
    package_json: Option<PackageJson>,
}

impl Project {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, config: ProjectConfig) -> Self {
        Self {
            root: root.into(),
            package_manager: None,
            config,
            validation: ValidationStatus::NotValidated,
            package_json: None,
        }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn package_manager(&self) -> Option<&PackageManager> {
        self.package_manager.as_ref()
    }

    #[must_use]
    pub fn validation_status(&self) -> &ValidationStatus {
        &self.validation
    }

    #[must_use]
    pub fn package_json(&self) -> Option<&PackageJson> {
        self.package_json.as_ref()
    }
}

/// Manager for project operations
#[derive(Debug)]
pub struct ProjectManager<F: FileSystem = FileSystemManager> {
    fs: F,
}

impl ProjectManager<FileSystemManager> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            fs: FileSystemManager::new(),
        }
    }
}

impl<F: FileSystem> ProjectManager<F> {
    pub fn with_filesystem(fs: F) -> Self {
        Self { fs }
    }

    pub fn detect_project(
        &self,
        path: impl AsRef<Path>,
        config: ProjectConfig,
    ) -> StandardResult<Project> {
        let path = path.as_ref();
        let mut project = Project::new(path, config.clone());

        let package_json_path = path.join("package.json");
        let package_json_content = self
            .fs
            .read_file_string(&package_json_path)
            .map_err(StandardError::FileSystem)?; // Map FS error

        project.package_json = Some(serde_json::from_str(&package_json_content)?); // Propagate serde_json::Error

        if project.config.detect_package_manager {
            match PackageManager::detect(path) {
                Ok(pm) => project.package_manager = Some(pm),
                Err(e) => {
                    if project.config.validate_structure {
                        log::warn!("Could not detect package manager at {}: {}", path.display(), e);
                    }
                    // Do not error out here, let validation handle missing manager if needed
                }
            }
        }

        if project.config.validate_structure {
            self.validate_project(&mut project)?;
        }

        Ok(project)
    }

    pub fn validate_project(&self, project: &mut Project) -> StandardResult<()> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Ensure package.json was loaded or load it now
        if project.package_json.is_none() {
            let package_json_path = project.root.join("package.json");
            match self.fs.read_file_string(&package_json_path) {
                Ok(content) => {
                    match serde_json::from_str::<PackageJson>(&content) {
                        Ok(parsed_json) => project.package_json = Some(parsed_json),
                        Err(e) => errors.push(format!("Invalid package.json format: {e}")),
                    }
                },
                Err(FileSystemError::NotFound { .. }) => errors.push("Missing package.json file.".to_string()),
                Err(e) => errors.push(format!("Failed to read package.json for validation: {e}")),
            }
        }

        // Check package manager consistency
        if project.config.detect_package_manager {
            if let Some(pm) = &project.package_manager {
                if !self.fs.exists(&pm.lock_file_path()) {
                    warnings.push(format!(
                        "Detected {} but its lockfile ({}) is missing.",
                        pm.kind().command(), pm.lock_file_path().display()
                    ));
                }
            } else if project.package_json.is_some() { // Only warn if package.json exists
                 warnings.push("Package manager could not be detected (missing lock file).".to_string());
            }
        }

        // Check node_modules existence and type
        if let Some(package_json) = &project.package_json {
            let has_deps = !package_json.dependencies.is_empty() || !package_json.dev_dependencies.is_empty();
            if has_deps {
                let node_modules_path = project.root.join("node_modules");
                if !self.fs.exists(&node_modules_path) {
                    warnings.push("Missing node_modules directory. Dependencies may not be installed.".to_string());
                } else {
                    // Use std::fs::metadata for synchronous check
                    match std_fs::metadata(&node_modules_path) {
                        Ok(metadata) => {
                            if !metadata.is_dir() {
                                errors.push("node_modules exists but is not a directory.".to_string());
                            }
                        }
                        Err(e) => {
                            // Map IO error during metadata check to a warning
                             warnings.push(format!("Could not check node_modules type: {}", FileSystemError::from_io(e, &node_modules_path)));
                        }
                    }
                }
            }
        } else if errors.is_empty() { // Only report this if package.json itself wasn't the main issue
            errors.push("Could not verify dependencies status due to missing/invalid package.json.".to_string());
        }

        // Update validation status
        project.validation = if !errors.is_empty() {
            ValidationStatus::Error(errors)
        } else if !warnings.is_empty() {
            ValidationStatus::Warning(warnings)
        } else {
            ValidationStatus::Valid
        };

        Ok(())
    }
}

impl Default for ProjectManager<FileSystemManager> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    // Commented out MockFileSystem tests
    /*
    use std::collections::{HashMap, HashSet};
    #[derive(Debug, Default, Clone)]
    struct MockFileSystem { files: HashMap<PathBuf, String>, dirs: HashSet<PathBuf> }
    impl MockFileSystem { ... }
    impl FileSystem for MockFileSystem { ... }
    fn setup_mock_fs(root: &Path) -> MockFileSystem { ... }
    #[test]
    fn test_project_detection_mock_fs() -> StandardResult<()> { ... }
    #[test]
    fn test_project_validation_mock_fs() -> StandardResult<()> { ... }
    */

    // Helper for real FS tests
    fn create_test_project_real(dir: &Path) -> StandardResult<()> {
        let fs = FileSystemManager::new();
        let package_json = r#"{
            "name": "test-project",
            "version": "1.0.0",
            "dependencies": { "test-dep": "^1.0.0" }
        }"#;
        fs.write_file_string(&dir.join("package.json"), package_json)?;
        fs.write_file_string(&dir.join("package-lock.json"), "{}")?; // npm lock file
        Ok(())
    }

     #[test]
     fn test_project_detection_real_fs() -> StandardResult<()> {
         let temp_dir = TempDir::new().unwrap();
         create_test_project_real(temp_dir.path())?;

         let manager = ProjectManager::new();
         let project = manager.detect_project(temp_dir.path(), ProjectConfig::new())?;

         assert!(matches!(project.validation_status(), ValidationStatus::Warning(_)), "Validation status should be Warning due to missing node_modules");
         if let ValidationStatus::Warning(warnings) = project.validation_status() {
              assert!(warnings.iter().any(|w| w.contains("Missing node_modules")));
         } else {
             panic!("Expected Warning status");
         }

         assert!(project.package_manager().is_some());
         assert_eq!(project.package_manager().unwrap().kind(), super::PackageManagerKind::Npm); // using npm lock file now
         assert!(project.package_json().is_some());
         assert_eq!(project.package_json().unwrap().name, "test-project");

         Ok(())
     }

     #[test]
     fn test_project_validation_real_fs() -> StandardResult<()> {
         let temp_dir = TempDir::new().unwrap();
         create_test_project_real(temp_dir.path())?;

         let manager = ProjectManager::new();
         let mut project = manager.detect_project(temp_dir.path(), ProjectConfig::new())?;

         // Initially should be Warning (missing node_modules)
         assert!(matches!(project.validation_status(), ValidationStatus::Warning(_)));

         // Create node_modules as a directory
         let node_modules_path = temp_dir.path().join("node_modules");
         std_fs::create_dir(&node_modules_path).unwrap();

         // Re-validate
         manager.validate_project(&mut project)?;
         assert!(matches!(project.validation_status(), ValidationStatus::Valid), "Should be valid after creating node_modules");

         // Test error: node_modules is a file
         std_fs::remove_dir(&node_modules_path).unwrap();
         std_fs::write(&node_modules_path, "not a dir").unwrap();
         manager.validate_project(&mut project)?;
         assert!(matches!(project.validation_status(), ValidationStatus::Error(_)));
          if let ValidationStatus::Error(errors) = project.validation_status() {
             assert!(errors.iter().any(|e| e.contains("node_modules exists but is not a directory")));
         } else {
             panic!("Expected validation error for file node_modules");
         }

         Ok(())
     }

      #[test]
      fn test_detect_project_no_package_json() {
         let temp_dir = TempDir::new().unwrap();
         let manager = ProjectManager::new();
         let result = manager.detect_project(temp_dir.path(), ProjectConfig::new());
         assert!(result.is_err());
         assert!(matches!(result.unwrap_err(), StandardError::FileSystem(FileSystemError::NotFound { .. })));
      }

       #[test]
       fn test_detect_project_invalid_package_json() -> StandardResult<()> {
         let temp_dir = TempDir::new().unwrap();
         let fs = FileSystemManager::new();
         // Need to create the directory first for write_file_string to work
         std_fs::create_dir_all(temp_dir.path())?;
         fs.write_file_string(&temp_dir.path().join("package.json"), "{ invalid json }")?;

         let manager = ProjectManager::new();
         let result = manager.detect_project(temp_dir.path(), ProjectConfig::new());
         assert!(result.is_err());
         assert!(matches!(result.unwrap_err(), StandardError::Json(_)));
         Ok(())
       }
}
