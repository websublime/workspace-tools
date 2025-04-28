//! Project structure management for Node.js projects.
//!
//! What:
//! This module provides functionality for detecting, validating, and managing
//! Node.js project structures. It can identify project roots, validate project
//! configurations, and handle package.json parsing.
//!
//! Who:
//! Used by developers who need to:
//! - Detect and validate Node.js project structures
//! - Parse package.json files
//! - Check project integrity
//! - Manage project configuration
//!
//! Why:
//! Proper project structure management is essential for:
//! - Reliable tool operation across different projects
//! - Consistent project validation
//! - Safe project manipulation
//! - Dependency management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{
    fs as std_fs,
    path::{Path, PathBuf},
};

use super::{FileSystem, FileSystemManager, PackageManager, ProjectConfig};
use crate::error::{FileSystemError, StandardError, StandardResult};

/// Validation status of a Node.js project.
///
/// Represents the result of validating a project's structure and configuration.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::ValidationStatus;
///
/// let status = ValidationStatus::Valid;
/// assert!(matches!(status, ValidationStatus::Valid));
///
/// let warnings = ValidationStatus::Warning(vec!["Missing test directory".to_string()]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationStatus {
    Valid,
    Warning(Vec<String>),
    Error(Vec<String>),
    NotValidated,
}

/// Package.json content structure for Node.js projects.
///
/// Represents the parsed contents of a package.json file with essential fields.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::PackageJson;
/// use std::collections::HashMap;
///
/// // Create a minimal package.json structure
/// let package_json = PackageJson {
///     name: "test-project".to_string(),
///     version: "1.0.0".to_string(),
///     dependencies: HashMap::new(),
///     dev_dependencies: HashMap::new(),
///     scripts: HashMap::new(),
/// };
/// ```
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

/// Represents a Node.js project with its configuration and validation status.
///
/// Encapsulates the structure and metadata of a Node.js project.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::{Project, ProjectConfig};
/// use std::path::Path;
///
/// let config = ProjectConfig::new();
/// let project = Project::new(".", config);
/// ```
#[derive(Debug)]
pub struct Project {
    /// Root directory of the project
    root: PathBuf,
    /// Detected package manager (if any)
    package_manager: Option<PackageManager>,
    /// Project configuration
    config: ProjectConfig,
    /// Validation status of the project
    validation: ValidationStatus,
    /// Parsed package.json (if available)
    package_json: Option<PackageJson>,
}

impl Project {
    /// Creates a new Project instance.
    ///
    /// # Arguments
    ///
    /// * `root` - Root directory of the project
    /// * `config` - Configuration for project detection and validation
    ///
    /// # Returns
    ///
    /// A new Project instance
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{Project, ProjectConfig};
    ///
    /// let config = ProjectConfig::new();
    /// let project = Project::new(".", config);
    /// ```
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

    /// Gets the root directory of the project.
    ///
    /// # Returns
    ///
    /// The root directory path
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{Project, ProjectConfig};
    /// use std::path::Path;
    ///
    /// let project = Project::new(".", ProjectConfig::new());
    /// assert_eq!(project.root(), Path::new("."));
    /// ```
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Gets the package manager for the project.
    ///
    /// # Returns
    ///
    /// The package manager, if detected
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{Project, ProjectConfig};
    ///
    /// let project = Project::new(".", ProjectConfig::new());
    /// if let Some(pm) = project.package_manager() {
    ///     println!("Using package manager: {}", pm.kind().command());
    /// }
    /// ```
    #[must_use]
    pub fn package_manager(&self) -> Option<&PackageManager> {
        self.package_manager.as_ref()
    }

    /// Gets the validation status of the project.
    ///
    /// # Returns
    ///
    /// The validation status
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{Project, ProjectConfig, ValidationStatus};
    ///
    /// let project = Project::new(".", ProjectConfig::new());
    /// match project.validation_status() {
    ///     ValidationStatus::Valid => println!("Project is valid"),
    ///     ValidationStatus::Warning(warnings) => println!("Project has warnings: {:?}", warnings),
    ///     ValidationStatus::Error(errors) => println!("Project has errors: {:?}", errors),
    ///     ValidationStatus::NotValidated => println!("Project has not been validated"),
    /// }
    /// ```
    #[must_use]
    pub fn validation_status(&self) -> &ValidationStatus {
        &self.validation
    }

    /// Gets the parsed package.json.
    ///
    /// # Returns
    ///
    /// The parsed package.json, if available
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{Project, ProjectConfig};
    ///
    /// let project = Project::new(".", ProjectConfig::new());
    /// if let Some(pkg) = project.package_json() {
    ///     println!("Project name: {}", pkg.name);
    ///     println!("Project version: {}", pkg.version);
    /// }
    /// ```
    #[must_use]
    pub fn package_json(&self) -> Option<&PackageJson> {
        self.package_json.as_ref()
    }
}

/// Manager for Node.js project operations.
///
/// Provides functionality for detecting, validating, and managing Node.js projects.
///
/// # Examples
///
/// ```no_run
/// use sublime_standard_tools::project::{ProjectManager, ProjectConfig};
///
/// let manager = ProjectManager::new();
/// let config = ProjectConfig::new();
///
/// // Detect a project in the current directory
/// match manager.detect_project(".", &config) {
///     Ok(project) => println!("Detected project: {}", project.root().display()),
///     Err(e) => println!("Failed to detect project: {}", e),
/// }
/// ```
#[derive(Debug)]
pub struct ProjectManager<F: FileSystem = FileSystemManager> {
    fs: F,
}

impl ProjectManager<FileSystemManager> {
    /// Creates a new ProjectManager with the default filesystem manager.
    ///
    /// # Returns
    ///
    /// A new ProjectManager instance
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectManager;
    ///
    /// let manager = ProjectManager::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { fs: FileSystemManager::new() }
    }
}

impl<F: FileSystem> ProjectManager<F> {
    /// Creates a new ProjectManager with a custom filesystem implementation.
    ///
    /// # Arguments
    ///
    /// * `fs` - The filesystem implementation to use
    ///
    /// # Returns
    ///
    /// A new ProjectManager instance
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{ProjectManager, FileSystemManager};
    ///
    /// let fs = FileSystemManager::new();
    /// let manager = ProjectManager::with_filesystem(fs);
    /// ```
    pub fn with_filesystem(fs: F) -> Self {
        Self { fs }
    }

    /// Detects a Node.js project in the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to detect the project in
    /// * `config` - Configuration for project detection
    ///
    /// # Returns
    ///
    /// A Project instance or an error if detection fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_standard_tools::project::{ProjectManager, ProjectConfig};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = ProjectManager::new();
    /// let config = ProjectConfig::new();
    ///
    /// let project = manager.detect_project(".", &config)?;
    /// println!("Project name: {}", project.package_json().unwrap().name);
    /// # Ok(())
    /// # }
    /// ```
    pub fn detect_project(
        &self,
        path: impl AsRef<Path>,
        config: &ProjectConfig,
    ) -> StandardResult<Project> {
        let path = path.as_ref();
        let mut project = Project::new(path, config.clone());

        let package_json_path = path.join("package.json");
        let package_json_content =
            self.fs.read_file_string(&package_json_path).map_err(StandardError::FileSystem)?; // Map FS error

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

    /// Validates a Node.js project structure.
    ///
    /// Checks for required files, directory structure, and package manager consistency.
    ///
    /// # Arguments
    ///
    /// * `project` - The project to validate
    ///
    /// # Returns
    ///
    /// Success or an error if validation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_standard_tools::project::{ProjectManager, ProjectConfig};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = ProjectManager::new();
    /// let config = ProjectConfig::new();
    ///
    /// let mut project = manager.detect_project(".", &config)?;
    /// manager.validate_project(&mut project)?;
    ///
    /// println!("Validation status: {:?}", project.validation_status());
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate_project(&self, project: &mut Project) -> StandardResult<()> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Ensure package.json was loaded or load it now
        if project.package_json.is_none() {
            let package_json_path = project.root.join("package.json");
            match self.fs.read_file_string(&package_json_path) {
                Ok(content) => match serde_json::from_str::<PackageJson>(&content) {
                    Ok(parsed_json) => project.package_json = Some(parsed_json),
                    Err(e) => errors.push(format!("Invalid package.json format: {e}")),
                },
                Err(FileSystemError::NotFound { .. }) => {
                    errors.push("Missing package.json file.".to_string());
                }
                Err(e) => errors.push(format!("Failed to read package.json for validation: {e}")),
            }
        }

        // Check package manager consistency
        if project.config.detect_package_manager {
            if let Some(pm) = &project.package_manager {
                if !self.fs.exists(&pm.lock_file_path()) {
                    warnings.push(format!(
                        "Detected {} but its lockfile ({}) is missing.",
                        pm.kind().command(),
                        pm.lock_file_path().display()
                    ));
                }
            } else if project.package_json.is_some() {
                // Only warn if package.json exists
                warnings
                    .push("Package manager could not be detected (missing lock file).".to_string());
            }
        }

        // Check node_modules existence and type
        if let Some(package_json) = &project.package_json {
            let has_deps =
                !package_json.dependencies.is_empty() || !package_json.dev_dependencies.is_empty();
            if has_deps {
                let node_modules_path = project.root.join("node_modules");
                if self.fs.exists(&node_modules_path) {
                    // Use std::fs::metadata for synchronous check
                    match std_fs::metadata(&node_modules_path) {
                        Ok(metadata) => {
                            if !metadata.is_dir() {
                                errors.push(
                                    "node_modules exists but is not a directory.".to_string(),
                                );
                            }
                        }
                        Err(e) => {
                            // Map IO error during metadata check to a warning
                            warnings.push(format!(
                                "Could not check node_modules type: {}",
                                FileSystemError::from_io(e, &node_modules_path)
                            ));
                        }
                    }
                } else {
                    warnings.push(
                        "Missing node_modules directory. Dependencies may not be installed."
                            .to_string(),
                    );
                }
            }
        } else if errors.is_empty() {
            // Only report this if package.json itself wasn't the main issue
            errors.push(
                "Could not verify dependencies status due to missing/invalid package.json."
                    .to_string(),
            );
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
    use crate::project::PackageManagerKind;

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

    #[allow(clippy::expect_used)]
    #[allow(clippy::panic)]
    #[test]
    fn test_project_detection_real_fs() -> StandardResult<()> {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        create_test_project_real(temp_dir.path())?;

        let manager = ProjectManager::new();
        let project = manager.detect_project(temp_dir.path(), &ProjectConfig::new())?;

        assert!(
            matches!(project.validation_status(), ValidationStatus::Warning(_)),
            "Validation status should be Warning due to missing node_modules"
        );
        if let ValidationStatus::Warning(warnings) = project.validation_status() {
            assert!(warnings.iter().any(|w| w.contains("Missing node_modules")));
        } else {
            panic!("Expected Warning status");
        }

        let package_manager = project.package_manager().expect("Package manager should be present");
        let package_json = project.package_json().expect("Package JSON should be present");

        assert!(project.package_manager().is_some());
        assert_eq!(package_manager.kind(), PackageManagerKind::Npm); // using npm lock file now
        assert!(project.package_json().is_some());
        assert_eq!(package_json.name, "test-project");

        Ok(())
    }

    #[allow(clippy::expect_used)]
    #[allow(clippy::panic)]
    #[test]
    fn test_project_validation_real_fs() -> StandardResult<()> {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        create_test_project_real(temp_dir.path())?;

        let manager = ProjectManager::new();
        let mut project = manager.detect_project(temp_dir.path(), &ProjectConfig::new())?;

        // Initially should be Warning (missing node_modules)
        assert!(matches!(project.validation_status(), ValidationStatus::Warning(_)));

        // Create node_modules as a directory
        let node_modules_path = temp_dir.path().join("node_modules");
        std_fs::create_dir(&node_modules_path).expect("Failed to create node_modules directory");

        // Re-validate
        manager.validate_project(&mut project)?;
        assert!(
            matches!(project.validation_status(), ValidationStatus::Valid),
            "Should be valid after creating node_modules"
        );

        // Test error: node_modules is a file
        std_fs::remove_dir(&node_modules_path).expect("Failed to remove node_modules directory");
        std_fs::write(&node_modules_path, "not a dir").expect("Failed to write to node_modules");
        manager.validate_project(&mut project)?;
        assert!(matches!(project.validation_status(), ValidationStatus::Error(_)));
        if let ValidationStatus::Error(errors) = project.validation_status() {
            assert!(errors
                .iter()
                .any(|e| e.contains("node_modules exists but is not a directory")));
        } else {
            panic!("Expected validation error for file node_modules");
        }

        Ok(())
    }

    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_detect_project_no_package_json() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ProjectManager::new();
        let result = manager.detect_project(temp_dir.path(), &ProjectConfig::new());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StandardError::FileSystem(FileSystemError::NotFound { .. })
        ));
    }

    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_detect_project_invalid_package_json() -> StandardResult<()> {
        let temp_dir = TempDir::new().unwrap();
        let fs = FileSystemManager::new();
        // Need to create the directory first for write_file_string to work
        std_fs::create_dir_all(temp_dir.path())?;
        fs.write_file_string(&temp_dir.path().join("package.json"), "{ invalid json }")?;

        let manager = ProjectManager::new();
        let result = manager.detect_project(temp_dir.path(), &ProjectConfig::new());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StandardError::Json(_)));
        Ok(())
    }
}
