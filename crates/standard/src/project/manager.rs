//! # Project Management Implementation
//!
//! ## What
//! This file implements the `ProjectManager` struct, providing methods to
//! manage Node.js projects regardless of their type (simple or monorepo).
//! It offers a unified interface for common project operations.
//!
//! ## How
//! The manager uses the project detector to identify project types and
//! delegates to appropriate handlers for project-specific operations.
//! It provides methods for project creation, validation, and management.
//!
//! ## Why
//! Project management should be consistent across all project types.
//! This manager provides a single interface for common operations while
//! leveraging the specialized functionality of different project types.

use super::detector::ProjectDetector;
use super::types::{ProjectConfig, ProjectDescriptor};
use super::validator::ProjectValidator;
use crate::error::{Error, Result};
use crate::filesystem::{AsyncFileSystem, FileSystemManager};
use std::path::{Path, PathBuf};

/// Manages Node.js projects with a unified interface.
///
/// This struct provides methods for creating, validating, and managing
/// Node.js projects regardless of whether they are simple repositories
/// or monorepos. It acts as a facade over project-specific functionality.
///
/// # Type Parameters
///
/// * `F` - An async filesystem implementation that satisfies the `AsyncFileSystem` trait.
///   Defaults to `FileSystemManager` for standard operations.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::{ProjectManager, ProjectConfig};
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let manager = ProjectManager::new();
/// let config = ProjectConfig::new();
///
/// match manager.create_project(Path::new("."), &config).await {
///     Ok(project) => {
///         println!("Created {} project", project.as_project_info().kind().name());
///     }
///     Err(e) => eprintln!("Failed to create project: {}", e),
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ProjectManager<F: AsyncFileSystem = FileSystemManager> {
    /// Project detector for identifying project types
    detector: ProjectDetector<F>,
    /// Project validator for validation operations
    validator: ProjectValidator<F>,
}

impl ProjectManager<FileSystemManager> {
    /// Creates a new `ProjectManager` with the default filesystem implementation.
    ///
    /// # Returns
    ///
    /// A new `ProjectManager` instance using the `FileSystemManager`.
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
        let detector = ProjectDetector::new();
        let validator = ProjectValidator::new();
        Self { detector, validator }
    }
}

impl<F: AsyncFileSystem + Clone> ProjectManager<F> {
    /// Creates a new `ProjectManager` with a custom filesystem implementation.
    ///
    /// # Arguments
    ///
    /// * `fs` - The filesystem implementation to use
    ///
    /// # Returns
    ///
    /// A new `ProjectManager` instance using the provided filesystem.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use sublime_standard_tools::project::ProjectManager;
    ///
    /// let fs = FileSystemManager::new();
    /// let manager = ProjectManager::with_filesystem(fs);
    /// ```
    #[must_use]
    pub fn with_filesystem(fs: F) -> Self {
        let detector = ProjectDetector::with_filesystem(fs.clone());
        let validator = ProjectValidator::with_filesystem(fs);
        Self { detector, validator }
    }

    /// Creates and initializes a project descriptor for the given path.
    ///
    /// This method detects the project type, loads project information,
    /// and optionally validates the project structure based on the
    /// provided configuration.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the project directory
    /// * `config` - Configuration options for project creation
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - The path does not exist or cannot be accessed
    /// - The path does not contain a valid Node.js project
    /// - Project files cannot be read or parsed
    /// - Validation fails when enabled
    ///
    /// # Returns
    ///
    /// * `Ok(ProjectDescriptor)` - A fully initialized project descriptor
    /// * `Err(Error)` - If project creation failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{ProjectManager, ProjectConfig};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = ProjectManager::new();
    /// let config = ProjectConfig::new()
    ///     .with_validate_structure(true);
    ///
    /// let project = manager.create_project(Path::new("."), &config).await?;
    /// println!("Created project: {}", project.as_project_info().kind().name());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_project(
        &self,
        path: impl AsRef<Path>,
        config: &ProjectConfig,
    ) -> Result<ProjectDescriptor> {
        let path = path.as_ref();

        // Detect the project type and create descriptor
        let mut project = self.detector.detect(path, config).await?;

        // Validate if requested
        if config.validate_structure {
            self.validator.validate_project(&mut project).await?;
        }

        Ok(project)
    }

    /// Validates an existing project descriptor.
    ///
    /// This method performs validation on a project descriptor,
    /// updating its validation status based on the project's
    /// current state and structure.
    ///
    /// # Arguments
    ///
    /// * `project` - A mutable reference to the project descriptor
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
    /// * `Err(Error)` - If an error occurred during validation
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{ProjectManager, ProjectConfig};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = ProjectManager::new();
    /// let config = ProjectConfig::new();
    ///
    /// let mut project = manager.create_project(Path::new("."), &config).await?;
    /// manager.validate_project(&mut project).await?;
    ///
    /// let info = project.as_project_info();
    /// println!("Validation status: {:?}", info.validation_status());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate_project(&self, project: &mut ProjectDescriptor) -> Result<()> {
        self.validator.validate_project(project).await
    }

    /// Checks if a path contains a valid Node.js project.
    ///
    /// This method performs a quick validation to determine if the
    /// specified path contains a valid Node.js project structure.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to validate
    ///
    /// # Returns
    ///
    /// `true` if the path contains a valid Node.js project, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectManager;
    /// use std::path::Path;
    ///
    /// # async fn example() {
    /// let manager = ProjectManager::new();
    /// if manager.is_valid_project(Path::new(".")).await {
    ///     println!("Current directory is a valid Node.js project");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub async fn is_valid_project(&self, path: impl AsRef<Path>) -> bool {
        self.detector.is_valid_project(path).await
    }

    /// Finds the root directory of a Node.js project.
    ///
    /// This method traverses upward from the given path to find the
    /// nearest directory that contains a package.json file, indicating
    /// the root of a Node.js project.
    ///
    /// # Arguments
    ///
    /// * `start_path` - The path to start searching from
    ///
    /// # Returns
    ///
    /// * `Some(PathBuf)` - The path to the project root if found
    /// * `None` - If no project root was found
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectManager;
    /// use std::path::Path;
    ///
    /// # async fn example() {
    /// let manager = ProjectManager::new();
    /// if let Some(root) = manager.find_project_root(Path::new("src/components")).await {
    ///     println!("Project root found at: {}", root.display());
    /// }
    /// # }
    /// ```
    #[must_use]
    pub async fn find_project_root(&self, start_path: impl AsRef<Path>) -> Option<PathBuf> {
        let start_path = start_path.as_ref();

        // Start from the given path and traverse upward
        let mut current = Some(start_path);
        while let Some(path) = current {
            if self.detector.is_valid_project(path).await {
                return Some(path.to_path_buf());
            }
            current = path.parent();
        }

        None
    }

    /// Creates a project descriptor from a specific root path.
    ///
    /// This method creates a project descriptor for a known project root,
    /// bypassing the root detection process. It's useful when you already
    /// know the project root location.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the project
    /// * `config` - Configuration options for project creation
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - The root path does not contain a valid Node.js project
    /// - Project files cannot be read or parsed
    /// - Validation fails when enabled
    ///
    /// # Returns
    ///
    /// * `Ok(ProjectDescriptor)` - A project descriptor for the root
    /// * `Err(Error)` - If project creation failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{ProjectManager, ProjectConfig};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = ProjectManager::new();
    /// let config = ProjectConfig::new();
    ///
    /// let project = manager.create_project_from_root(Path::new("/my/project"), &config).await?;
    /// println!("Created project from root: {}", project.as_project_info().root().display());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_project_from_root(
        &self,
        root: impl AsRef<Path>,
        config: &ProjectConfig,
    ) -> Result<ProjectDescriptor> {
        let root = root.as_ref();

        // Validate that the root contains a project
        if !self.detector.is_valid_project(root).await {
            return Err(Error::operation(format!(
                "Path is not a valid Node.js project: {}",
                root.display()
            )));
        }

        self.create_project(root, config).await
    }

    /// Gets access to the underlying project detector.
    ///
    /// This method provides access to the detector for advanced
    /// project detection operations.
    ///
    /// # Returns
    ///
    /// A reference to the `ProjectDetector` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{ProjectManager, ProjectConfig};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = ProjectManager::new();
    /// let config = ProjectConfig::new();
    ///
    /// let detector = manager.detector();
    /// if let Ok(kind) = detector.detect_kind(Path::new("."), &config).await {
    ///     println!("Project kind: {}", kind.name());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn detector(&self) -> &ProjectDetector<F> {
        &self.detector
    }

    /// Gets access to the underlying project validator.
    ///
    /// This method provides access to the validator for advanced
    /// project validation operations.
    ///
    /// # Returns
    ///
    /// A reference to the `ProjectValidator` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectManager;
    ///
    /// let manager = ProjectManager::new();
    /// let validator = manager.validator();
    /// // Use validator for custom validation operations
    /// ```
    #[must_use]
    pub fn validator(&self) -> &ProjectValidator<F> {
        &self.validator
    }
}

impl<F: AsyncFileSystem + Clone> Default for ProjectManager<F>
where
    F: Default,
{
    fn default() -> Self {
        let detector = ProjectDetector::default();
        let validator = ProjectValidator::default();
        Self { detector, validator }
    }
}