//! # Simple Project Implementation
//!
//! ## What
//! This file implements the `SimpleProject` struct, representing a standard
//! Node.js project with a single package.json file (not a monorepo).
//!
//! ## How
//! The implementation provides methods for creating, managing, and accessing
//! information about simple Node.js projects, implementing the `ProjectInfo`
//! trait for unified access.
//!
//! ## Why
//! Many Node.js projects are simple repositories with a single package.json.
//! This implementation provides the same rich functionality available for
//! monorepos to simple projects, ensuring feature parity across project types.

use super::types::{ProjectInfo, ProjectKind, ProjectValidationStatus};
use crate::monorepo::PackageManager;
use package_json::PackageJson;
use std::path::{Path, PathBuf};

/// Represents a simple Node.js project (non-monorepo).
///
/// A simple project is characterized by having a single package.json file
/// at its root and no workspace configuration. This struct provides access
/// to all project information and implements the `ProjectInfo` trait for
/// unified handling.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use sublime_standard_tools::project::{SimpleProject, ProjectInfo};
///
/// let project = SimpleProject::new(
///     PathBuf::from("/path/to/project"),
///     None, // package_manager
///     None, // package_json
/// );
///
/// assert_eq!(project.root(), Path::new("/path/to/project"));
/// assert!(!project.kind().is_monorepo());
/// ```
#[derive(Debug)]
pub struct SimpleProject {
    /// Root directory of the project
    root: PathBuf,
    /// Detected package manager, if any
    package_manager: Option<PackageManager>,
    /// Parsed package.json content, if available
    package_json: Option<PackageJson>,
    /// Current validation status
    validation_status: ProjectValidationStatus,
}

impl SimpleProject {
    /// Creates a new `SimpleProject` instance.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the project
    /// * `package_manager` - Optional detected package manager
    /// * `package_json` - Optional parsed package.json content
    ///
    /// # Returns
    ///
    /// A new `SimpleProject` instance with validation status set to `NotValidated`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_standard_tools::project::SimpleProject;
    ///
    /// let project = SimpleProject::new(
    ///     PathBuf::from("/my/project"),
    ///     None,
    ///     None,
    /// );
    /// ```
    #[must_use]
    pub fn new(
        root: PathBuf,
        package_manager: Option<PackageManager>,
        package_json: Option<PackageJson>,
    ) -> Self {
        Self {
            root,
            package_manager,
            package_json,
            validation_status: ProjectValidationStatus::NotValidated,
        }
    }

    /// Creates a new `SimpleProject` with a specific validation status.
    ///
    /// This method is useful when creating a project that has already
    /// been validated or when setting a specific validation state.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the project
    /// * `package_manager` - Optional detected package manager
    /// * `package_json` - Optional parsed package.json content
    /// * `validation_status` - The validation status to set
    ///
    /// # Returns
    ///
    /// A new `SimpleProject` instance with the specified validation status.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_standard_tools::project::{SimpleProject, ProjectValidationStatus};
    ///
    /// let project = SimpleProject::with_validation(
    ///     PathBuf::from("/my/project"),
    ///     None,
    ///     None,
    ///     ProjectValidationStatus::Valid,
    /// );
    /// ```
    #[must_use]
    pub fn with_validation(
        root: PathBuf,
        package_manager: Option<PackageManager>,
        package_json: Option<PackageJson>,
        validation_status: ProjectValidationStatus,
    ) -> Self {
        Self { root, package_manager, package_json, validation_status }
    }

    /// Updates the package manager for this project.
    ///
    /// # Arguments
    ///
    /// * `package_manager` - The new package manager to set
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_standard_tools::project::SimpleProject;
    /// use sublime_standard_tools::monorepo::{PackageManager, PackageManagerKind};
    ///
    /// let mut project = SimpleProject::new(PathBuf::from("/project"), None, None);
    /// let pm = PackageManager::new(PackageManagerKind::Npm, "/project");
    /// project.set_package_manager(Some(pm));
    /// ```
    pub fn set_package_manager(&mut self, package_manager: Option<PackageManager>) {
        self.package_manager = package_manager;
    }

    /// Updates the package.json for this project.
    ///
    /// # Arguments
    ///
    /// * `package_json` - The new package.json to set
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_standard_tools::project::SimpleProject;
    /// use package_json::PackageJson;
    ///
    /// let mut project = SimpleProject::new(PathBuf::from("/project"), None, None);
    /// // Assuming package_json is loaded from file
    /// // project.set_package_json(Some(package_json));
    /// ```
    pub fn set_package_json(&mut self, package_json: Option<PackageJson>) {
        self.package_json = package_json;
    }

    /// Updates the validation status for this project.
    ///
    /// # Arguments
    ///
    /// * `status` - The new validation status
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_standard_tools::project::{SimpleProject, ProjectValidationStatus};
    ///
    /// let mut project = SimpleProject::new(PathBuf::from("/project"), None, None);
    /// project.set_validation_status(ProjectValidationStatus::Valid);
    /// assert!(project.validation_status().is_valid());
    /// ```
    pub fn set_validation_status(&mut self, status: ProjectValidationStatus) {
        self.validation_status = status;
    }

    /// Gets a mutable reference to the validation status.
    ///
    /// This is useful for validators that need to update the status in place.
    ///
    /// # Returns
    ///
    /// A mutable reference to the project's validation status.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_standard_tools::project::{SimpleProject, ProjectValidationStatus};
    ///
    /// let mut project = SimpleProject::new(PathBuf::from("/project"), None, None);
    /// *project.validation_status_mut() = ProjectValidationStatus::Valid;
    /// ```
    pub fn validation_status_mut(&mut self) -> &mut ProjectValidationStatus {
        &mut self.validation_status
    }

    /// Checks if this project has a package.json file.
    ///
    /// # Returns
    ///
    /// `true` if package.json has been loaded, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_standard_tools::project::SimpleProject;
    ///
    /// let project = SimpleProject::new(PathBuf::from("/project"), None, None);
    /// assert!(!project.has_package_json());
    /// ```
    #[must_use]
    pub fn has_package_json(&self) -> bool {
        self.package_json.is_some()
    }

    /// Checks if a package manager has been detected for this project.
    ///
    /// # Returns
    ///
    /// `true` if a package manager has been detected, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_standard_tools::project::SimpleProject;
    ///
    /// let project = SimpleProject::new(PathBuf::from("/project"), None, None);
    /// assert!(!project.has_package_manager());
    /// ```
    #[must_use]
    pub fn has_package_manager(&self) -> bool {
        self.package_manager.is_some()
    }
}

impl ProjectInfo for SimpleProject {
    fn root(&self) -> &Path {
        &self.root
    }

    fn package_manager(&self) -> Option<&PackageManager> {
        self.package_manager.as_ref()
    }

    fn package_json(&self) -> Option<&PackageJson> {
        self.package_json.as_ref()
    }

    fn validation_status(&self) -> &ProjectValidationStatus {
        &self.validation_status
    }

    fn kind(&self) -> ProjectKind {
        ProjectKind::Simple
    }
}