//! # Project Management Implementation
//!
//! ## What
//! This file implements functionality for the Project and ProjectManager structs,
//! providing methods to detect, validate, and work with Node.js projects.
//!
//! ## How
//! The implementation provides methods for creating projects, accessing project
//! properties, detecting project structures, and validating project configurations.
//!
//! ## Why
//! Projects need consistent analysis and validation to ensure they're correctly
//! configured. This module provides tools to examine project structures, validate
//! their integrity, and access their properties in a standardized way.

use super::{PackageManager, Project, ProjectConfig, ProjectManager, ProjectValidationStatus};
use crate::{
    error::{Error, FileSystemError, Result, WorkspaceError},
    filesystem::{FileSystem, FileSystemManager},
};
use package_json::PackageJson;
use std::path::{Path, PathBuf};

impl Default for ProjectConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectConfig {
    /// Creates a new ProjectConfig with default values.
    ///
    /// # Returns
    ///
    /// A new ProjectConfig with default settings:
    /// - No specified root (uses current directory)
    /// - Package manager detection enabled
    /// - Structure validation enabled
    /// - Monorepo detection enabled
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::monorepo::ProjectConfig;
    ///
    /// let config = ProjectConfig::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: None,
            detect_package_manager: true,
            validate_structure: true,
            detect_monorepo: true,
        }
    }

    /// Sets the root directory for project detection.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory to use
    ///
    /// # Returns
    ///
    /// Self with the root directory updated.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::monorepo::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .with_root("/path/to/project");
    /// ```
    #[must_use]
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
        self
    }

    /// Sets whether to detect package managers.
    ///
    /// # Arguments
    ///
    /// * `detect` - Whether to detect package managers
    ///
    /// # Returns
    ///
    /// Self with the package manager detection setting updated.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::monorepo::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .with_detect_package_manager(true);
    /// ```
    #[must_use]
    pub fn with_detect_package_manager(mut self, detect: bool) -> Self {
        self.detect_package_manager = detect;
        self
    }

    /// Sets whether to validate project structure.
    ///
    /// # Arguments
    ///
    /// * `validate` - Whether to validate project structure
    ///
    /// # Returns
    ///
    /// Self with the validation setting updated.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::monorepo::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .with_validate_structure(true);
    /// ```
    #[must_use]
    pub fn with_validate_structure(mut self, validate: bool) -> Self {
        self.validate_structure = validate;
        self
    }

    /// Sets whether to detect monorepo structures.
    ///
    /// # Arguments
    ///
    /// * `detect` - Whether to detect monorepo structures
    ///
    /// # Returns
    ///
    /// Self with the monorepo detection setting updated.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::monorepo::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .with_detect_monorepo(true);
    /// ```
    #[must_use]
    pub fn with_detect_monorepo(mut self, detect: bool) -> Self {
        self.detect_monorepo = detect;
        self
    }
}

impl Project {
    /// Creates a new Project instance with the specified root and configuration.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the project
    /// * `config` - Configuration options for the project
    ///
    /// # Returns
    ///
    /// A new Project instance configured with the provided options.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::monorepo::{Project, ProjectConfig};
    ///
    /// let config = ProjectConfig::default();
    /// let project = Project::new("/path/to/project", config);
    /// ```
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, config: ProjectConfig) -> Self {
        Self {
            root: root.into(),
            package_manager: None,
            config,
            validation: ProjectValidationStatus::NotValidated,
            package_json: None,
        }
    }

    /// Returns the root directory of the project.
    ///
    /// # Returns
    ///
    /// A reference to the Path of the project's root directory.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use sublime_standard_tools::monorepo::{Project, ProjectConfig};
    /// #
    /// # let config = ProjectConfig::default();
    /// # let project = Project::new("/path/to/project", config);
    /// #
    /// let root = project.root();
    /// assert_eq!(root, PathBuf::from("/path/to/project"));
    /// ```
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the package manager for the project, if detected.
    ///
    /// # Returns
    ///
    /// * `Some(&PackageManager)` - If a package manager was detected
    /// * `None` - If no package manager was detected
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::monorepo::{Project, ProjectConfig};
    /// #
    /// # let config = ProjectConfig::default();
    /// # let project = Project::new("/path/to/project", config);
    /// #
    /// if let Some(manager) = project.package_manager() {
    ///     println!("Using package manager: {:?}", manager.kind());
    /// }
    /// ```
    #[must_use]
    pub fn package_manager(&self) -> Option<&PackageManager> {
        self.package_manager.as_ref()
    }

    /// Returns the validation status of the project.
    ///
    /// # Returns
    ///
    /// A reference to the ProjectValidationStatus indicating the validation state.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::monorepo::{Project, ProjectConfig};
    /// #
    /// # let config = ProjectConfig::default();
    /// # let project = Project::new("/path/to/project", config);
    /// #
    /// match project.validation_status() {
    ///     sublime_standard_tools::monorepo::ProjectValidationStatus::Valid => println!("Project is valid"),
    ///     sublime_standard_tools::monorepo::ProjectValidationStatus::Warning(warnings) => {
    ///         println!("Project has warnings:");
    ///         for warning in warnings {
    ///             println!("  - {}", warning);
    ///         }
    ///     },
    ///     sublime_standard_tools::monorepo::ProjectValidationStatus::Error(errors) => {
    ///         println!("Project has errors:");
    ///         for error in errors {
    ///             println!("  - {}", error);
    ///         }
    ///     },
    ///     sublime_standard_tools::monorepo::ProjectValidationStatus::NotValidated => {
    ///         println!("Project has not been validated");
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn validation_status(&self) -> &ProjectValidationStatus {
        &self.validation
    }

    /// Returns the parsed package.json for the project, if available.
    ///
    /// # Returns
    ///
    /// * `Some(&PackageJson)` - If package.json was successfully parsed
    /// * `None` - If package.json was not found or could not be parsed
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::monorepo::{Project, ProjectConfig};
    /// #
    /// # let config = ProjectConfig::default();
    /// # let project = Project::new("/path/to/project", config);
    /// #
    /// if let Some(package_json) = project.package_json() {
    ///     println!("Project name: {}", package_json.name);
    ///     println!("Project version: {}", package_json.version);
    /// }
    /// ```
    #[must_use]
    pub fn package_json(&self) -> Option<&PackageJson> {
        self.package_json.as_ref()
    }
}

impl Default for ProjectManager<FileSystemManager> {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectManager<FileSystemManager> {
    /// Creates a new ProjectManager instance with the default filesystem implementation.
    ///
    /// # Returns
    ///
    /// A new ProjectManager instance using the FileSystemManager.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::monorepo::ProjectManager;
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
    /// A new ProjectManager instance using the provided filesystem.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use sublime_standard_tools::monorepo::ProjectManager;
    ///
    /// let fs = FileSystemManager::new();
    /// let manager = ProjectManager::with_filesystem(fs);
    /// ```
    pub fn with_filesystem(fs: F) -> Self {
        Self { fs }
    }

    /// Detects and analyzes a Node.js project at the given path.
    ///
    /// This method examines the directory structure, loads package.json,
    /// identifies the package manager (if configured), and validates
    /// the project structure (if configured).
    ///
    /// # Arguments
    ///
    /// * `path` - The path to analyze for a Node.js project
    /// * `config` - Configuration options for project detection
    ///
    /// # Returns
    ///
    /// * `Ok(Project)` - A project descriptor containing all project information
    /// * `Err(Error)` - If the path is not a valid project or an error occurred
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::monorepo::{ProjectManager, ProjectConfig};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = ProjectManager::new();
    /// let config = ProjectConfig::default();
    ///
    /// let project = manager.detect_project(".", &config)?;
    /// println!("Detected project at: {}", project.root().display());
    /// # Ok(())
    /// # }
    /// ```
    pub fn detect_project(
        &self,
        path: impl AsRef<Path>,
        config: &ProjectConfig,
    ) -> Result<Project> {
        let path = path.as_ref();
        let mut project = Project::new(path, config.clone());

        let package_json_path = path.join("package.json");
        let package_json_content = self.fs.read_file_string(&package_json_path)?;
        let package_json = serde_json::from_str(&package_json_content)
            .map_err(|e| Error::Workspace(WorkspaceError::InvalidPackageJson(e.to_string())))?;

        project.package_json = Some(package_json);

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

    /// Validates a Node.js project structure and updates its validation status.
    ///
    /// This method checks various aspects of the project structure such as:
    /// - Presence and validity of package.json
    /// - Consistency of the detected package manager with lock files
    /// - Existence of node_modules (if the project has dependencies)
    ///
    /// # Arguments
    ///
    /// * `project` - A mutable reference to the Project to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If validation was completed (regardless of validation result)
    /// * `Err(Error)` - If an unexpected error occurred during validation
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::monorepo::{ProjectManager, Project, ProjectConfig};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = ProjectManager::new();
    /// let config = ProjectConfig::default();
    /// let mut project = Project::new(".", config);
    ///
    /// manager.validate_project(&mut project)?;
    ///
    /// match project.validation_status() {
    ///     sublime_standard_tools::monorepo::ProjectValidationStatus::Valid => {
    ///         println!("Project structure is valid");
    ///     },
    ///     sublime_standard_tools::monorepo::ProjectValidationStatus::Warning(warnings) => {
    ///         println!("Project has warnings: {:?}", warnings);
    ///     },
    ///     sublime_standard_tools::monorepo::ProjectValidationStatus::Error(errors) => {
    ///         println!("Project has errors: {:?}", errors);
    ///     },
    ///     _ => {}
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate_project(&self, project: &mut Project) -> Result<()> {
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
                package_json.dependencies.is_none() || package_json.dev_dependencies.is_none();
            if has_deps {
                let node_modules_path = project.root.join("node_modules");
                if self.fs.exists(&node_modules_path) {
                    // Use std::fs::metadata for synchronous check
                    match std::fs::metadata(&node_modules_path) {
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
            ProjectValidationStatus::Error(errors)
        } else if !warnings.is_empty() {
            ProjectValidationStatus::Warning(warnings)
        } else {
            ProjectValidationStatus::Valid
        };

        Ok(())
    }
}
