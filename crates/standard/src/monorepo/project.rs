//! # Project Management Implementation
//!
//! ## What
//! This file implements functionality for the Project and `ProjectManager` structs,
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

use super::{PackageManager, Project};
use crate::project::{ProjectConfig, ProjectValidationStatus};
use package_json::PackageJson;
use std::path::{Path, PathBuf};


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
    /// A reference to the `ProjectValidationStatus` indicating the validation state.
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

    /// Returns the configuration options used for this project.
    ///
    /// # Returns
    ///
    /// A reference to the `ProjectConfig` used to create this project.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::monorepo::{Project, ProjectConfig};
    /// #
    /// # let config = ProjectConfig::default();
    /// # let project = Project::new("/path/to/project", config);
    /// #
    /// let config = project.config();
    /// println!("Validate structure: {}", config.validate_structure);
    /// ```
    #[must_use]
    pub fn config(&self) -> &ProjectConfig {
        &self.config
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

