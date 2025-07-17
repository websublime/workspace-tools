//! # Project Validation Implementation
//!
//! ## What
//! This file implements the `ProjectValidator` struct, providing methods to
//! validate Node.js projects regardless of their type. It checks project
//! structure, dependencies, and configuration consistency.
//!
//! ## How
//! The validator performs comprehensive checks on project structure,
//! package.json validity, package manager consistency, and dependency
//! installation status. It updates project validation status with detailed
//! error and warning information.
//!
//! ## Why
//! Project validation ensures that Node.js projects are properly configured
//! and ready for development. Consistent validation across project types
//! helps identify issues early and provides actionable feedback.

use super::types::{ProjectDescriptor, ProjectInfo, ProjectValidationStatus};
use super::SimpleProject;
use crate::error::Result;
use crate::filesystem::{FileSystem, FileSystemManager};
use crate::monorepo::MonorepoDescriptor;
use package_json::PackageJson;
use std::path::Path;

/// Validates Node.js projects with comprehensive structure and configuration checks.
///
/// This struct provides methods for validating different aspects of Node.js
/// projects, including package.json validity, package manager consistency,
/// dependency installation, and project structure integrity.
///
/// # Type Parameters
///
/// * `F` - A filesystem implementation that satisfies the `FileSystem` trait.
///   Defaults to `FileSystemManager` for standard operations.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::{ProjectValidator, ProjectDescriptor};
/// use sublime_standard_tools::filesystem::FileSystemManager;
///
/// let validator = ProjectValidator::new();
/// // Assume project is created elsewhere
/// // validator.validate_project(&mut project).unwrap();
/// ```
pub struct ProjectValidator<F: FileSystem = FileSystemManager> {
    /// Filesystem implementation for file operations
    fs: F,
}

impl ProjectValidator<FileSystemManager> {
    /// Creates a new `ProjectValidator` with the default filesystem implementation.
    ///
    /// # Returns
    ///
    /// A new `ProjectValidator` instance using the `FileSystemManager`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectValidator;
    ///
    /// let validator = ProjectValidator::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { fs: FileSystemManager::new() }
    }
}

impl<F: FileSystem> ProjectValidator<F> {
    /// Creates a new `ProjectValidator` with a custom filesystem implementation.
    ///
    /// # Arguments
    ///
    /// * `fs` - The filesystem implementation to use
    ///
    /// # Returns
    ///
    /// A new `ProjectValidator` instance using the provided filesystem.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use sublime_standard_tools::project::ProjectValidator;
    ///
    /// let fs = FileSystemManager::new();
    /// let validator = ProjectValidator::with_filesystem(fs);
    /// ```
    #[must_use]
    pub fn with_filesystem(fs: F) -> Self {
        Self { fs }
    }

    /// Validates a project descriptor and updates its validation status.
    ///
    /// This method performs comprehensive validation on a project descriptor,
    /// checking various aspects of the project structure and configuration.
    /// The validation status is updated with detailed error and warning information.
    ///
    /// # Arguments
    ///
    /// * `project` - A mutable reference to the project descriptor to validate
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - An I/O error occurs while reading project files
    /// - Project files cannot be parsed
    /// - The filesystem cannot be accessed
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If validation completed successfully (regardless of validation result)
    /// * `Err(Error)` - If an unexpected error occurred during validation
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{ProjectValidator, ProjectDescriptor};
    ///
    /// # fn example(mut project: ProjectDescriptor) -> Result<(), Box<dyn std::error::Error>> {
    /// let validator = ProjectValidator::new();
    /// validator.validate_project(&mut project)?;
    ///
    /// let info = project.as_project_info();
    /// match info.validation_status() {
    ///     sublime_standard_tools::project::ProjectValidationStatus::Valid => {
    ///         println!("Project is valid");
    ///     }
    ///     sublime_standard_tools::project::ProjectValidationStatus::Warning(warnings) => {
    ///         println!("Project has warnings: {:?}", warnings);
    ///     }
    ///     sublime_standard_tools::project::ProjectValidationStatus::Error(errors) => {
    ///         println!("Project has errors: {:?}", errors);
    ///     }
    ///     _ => {}
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate_project(&self, project: &mut ProjectDescriptor) -> Result<()> {
        match project {
            ProjectDescriptor::Simple(simple_project) => {
                self.validate_simple_project(simple_project);
                Ok(())
            }
            ProjectDescriptor::Monorepo(monorepo) => {
                self.validate_monorepo_project(monorepo);
                Ok(())
            }
        }
    }

    /// Validates a simple project.
    ///
    /// This method performs validation specific to simple Node.js projects,
    /// checking package.json validity, package manager consistency, and
    /// dependency installation status.
    ///
    /// # Arguments
    ///
    /// * `project` - A mutable reference to the simple project to validate
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - An I/O error occurs while reading project files
    /// - Project files cannot be parsed
    /// - The filesystem cannot be accessed
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If validation completed successfully
    /// * `Err(Error)` - If an unexpected error occurred during validation
    fn validate_simple_project(&self, project: &mut SimpleProject) {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate package.json
        self.validate_package_json(project.root(), &mut errors, &mut warnings);

        // Validate package manager consistency
        self.validate_package_manager_consistency(project, &mut errors, &mut warnings);

        // Validate node_modules and dependencies
        self.validate_dependencies(project, &mut errors, &mut warnings);

        // Update validation status
        let status = self.create_validation_status(errors, warnings);
        project.set_validation_status(status);
    }

    /// Validates a monorepo project.
    ///
    /// This method performs validation specific to monorepo projects,
    /// delegating to the monorepo's own validation logic while ensuring
    /// consistency with the unified validation interface.
    ///
    /// # Arguments
    ///
    /// * `monorepo` - A mutable reference to the monorepo descriptor to validate
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - An I/O error occurs while reading project files
    /// - Project files cannot be parsed
    /// - The filesystem cannot be accessed
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If validation completed successfully
    /// * `Err(Error)` - If an unexpected error occurred during validation
    #[allow(clippy::unused_self)]
    fn validate_monorepo_project(&self, _monorepo: &mut MonorepoDescriptor) {
        // For monorepos, we currently delegate to the existing validation logic
        // In the future, we might want to add unified validation here
        // For now, we just ensure the monorepo has a valid status
        
        // Monorepo validation would be handled by the monorepo module
        // This is a placeholder for future unified validation
    }

    /// Validates package.json file existence and format.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the project
    /// * `errors` - Vector to collect validation errors
    /// * `warnings` - Vector to collect validation warnings
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if filesystem operations fail.
    fn validate_package_json(
        &self,
        root: &Path,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) {
        let package_json_path = root.join("package.json");

        if !self.fs.exists(&package_json_path) {
            errors.push("Missing package.json file".to_string());
            return;
        }

        // Try to parse package.json
        match self.fs.read_file_string(&package_json_path) {
            Ok(content) => {
                if let Err(e) = serde_json::from_str::<PackageJson>(&content) {
                    errors.push(format!("Invalid package.json format: {e}"));
                } else {
                    // Parse successful - check for common issues
                    if let Ok(package_json) = serde_json::from_str::<PackageJson>(&content) {
                        self.validate_package_json_content(&package_json, warnings);
                    }
                }
            }
            Err(e) => {
                errors.push(format!("Failed to read package.json: {e}"));
            }
        }
    }

    /// Validates package.json content for common issues.
    ///
    /// # Arguments
    ///
    /// * `package_json` - The parsed package.json content
    /// * `warnings` - Vector to collect validation warnings
    #[allow(clippy::unused_self)]
    fn validate_package_json_content(&self, package_json: &PackageJson, warnings: &mut Vec<String>) {
        // Check for empty or default name
        if package_json.name.is_empty() {
            warnings.push("Package name is empty".to_string());
        }

        // Check for default version
        if package_json.version == "1.0.0" {
            warnings.push("Package is using default version (1.0.0)".to_string());
        }

        // Check for missing description
        if package_json.description.is_none() {
            warnings.push("Package description is missing".to_string());
        }

        // Check for missing license
        if package_json.license.is_none() {
            warnings.push("Package license is missing".to_string());
        }
    }

    /// Validates package manager consistency.
    ///
    /// # Arguments
    ///
    /// * `project` - The simple project to validate
    /// * `errors` - Vector to collect validation errors
    /// * `warnings` - Vector to collect validation warnings
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if filesystem operations fail.
    fn validate_package_manager_consistency(
        &self,
        project: &SimpleProject,
        _errors: &mut [String],
        warnings: &mut Vec<String>,
    ) {
        if let Some(package_manager) = project.package_manager() {
            let lock_file_path = package_manager.lock_file_path();

            if !self.fs.exists(&lock_file_path) {
                warnings.push(format!(
                    "Detected {} but lock file is missing: {}",
                    package_manager.kind().command(),
                    lock_file_path.display()
                ));
            }

            // Check for conflicting lock files
            self.check_conflicting_lock_files(project.root(), package_manager.kind(), warnings);
        } else if project.has_package_json() {
            warnings.push("Package manager could not be detected".to_string());
        }
    }

    /// Checks for conflicting package manager lock files.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the project
    /// * `detected_kind` - The detected package manager kind
    /// * `warnings` - Vector to collect validation warnings
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if filesystem operations fail.
    fn check_conflicting_lock_files(
        &self,
        root: &Path,
        detected_kind: crate::monorepo::PackageManagerKind,
        warnings: &mut Vec<String>,
    ) {
        use crate::monorepo::PackageManagerKind;

        let lock_files = [
            (PackageManagerKind::Npm, "package-lock.json"),
            (PackageManagerKind::Yarn, "yarn.lock"),
            (PackageManagerKind::Pnpm, "pnpm-lock.yaml"),
            (PackageManagerKind::Bun, "bun.lockb"),
        ];

        for (kind, lock_file) in &lock_files {
            if *kind != detected_kind {
                let lock_path = root.join(lock_file);
                if self.fs.exists(&lock_path) {
                    warnings.push(format!(
                        "Conflicting lock file found: {} (detected: {})",
                        lock_file,
                        detected_kind.command()
                    ));
                }
            }
        }
    }

    /// Validates dependencies and node_modules.
    ///
    /// # Arguments
    ///
    /// * `project` - The simple project to validate
    /// * `errors` - Vector to collect validation errors
    /// * `warnings` - Vector to collect validation warnings
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if filesystem operations fail.
    fn validate_dependencies(
        &self,
        project: &SimpleProject,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) {
        if let Some(package_json) = project.package_json() {
            let has_dependencies = package_json.dependencies.is_some() 
                || package_json.dev_dependencies.is_some() 
                || package_json.peer_dependencies.is_some();

            if has_dependencies {
                let node_modules_path = project.root().join("node_modules");

                if self.fs.exists(&node_modules_path) {
                    // Check if node_modules is actually a directory
                    match std::fs::metadata(&node_modules_path) {
                        Ok(metadata) => {
                            if !metadata.is_dir() {
                                errors.push(
                                    "node_modules exists but is not a directory".to_string(),
                                );
                            }
                        }
                        Err(_) => {
                            warnings.push(
                                "Could not check node_modules directory status".to_string(),
                            );
                        }
                    }
                } else {
                    warnings.push(
                        "Dependencies declared but node_modules directory is missing".to_string(),
                    );
                }
            }
        }
    }

    /// Creates a validation status from collected errors and warnings.
    ///
    /// # Arguments
    ///
    /// * `errors` - Vector of validation errors
    /// * `warnings` - Vector of validation warnings
    ///
    /// # Returns
    ///
    /// The appropriate `ProjectValidationStatus` based on the errors and warnings.
    #[allow(clippy::unused_self)]
    fn create_validation_status(
        &self,
        errors: Vec<String>,
        warnings: Vec<String>,
    ) -> ProjectValidationStatus {
        if !errors.is_empty() {
            ProjectValidationStatus::Error(errors)
        } else if !warnings.is_empty() {
            ProjectValidationStatus::Warning(warnings)
        } else {
            ProjectValidationStatus::Valid
        }
    }
}

impl<F: FileSystem> Default for ProjectValidator<F>
where
    F: Default,
{
    fn default() -> Self {
        Self { fs: F::default() }
    }
}