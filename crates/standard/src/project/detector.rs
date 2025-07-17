//! # Project Detection Implementation
//!
//! ## What
//! This file implements the `ProjectDetector` struct, which provides methods
//! to identify and analyze Node.js projects, determining whether they are
//! simple repositories or monorepos.
//!
//! ## How
//! The detector uses filesystem analysis to examine project structure,
//! package.json files, and lock files to determine project type. It leverages
//! the existing monorepo detection capabilities while adding support for
//! simple project identification.
//!
//! ## Why
//! Unified project detection is essential for tools that need to work with
//! any Node.js project type. This detector provides a single entry point
//! for identifying project characteristics and returning appropriate
//! representations.

use super::types::{ProjectConfig, ProjectDescriptor, ProjectKind};
use super::SimpleProject;
use crate::error::{Error, Result};
use crate::filesystem::{FileSystem, FileSystemManager};
use crate::monorepo::{MonorepoDetector, PackageManager};
use package_json::PackageJson;
use std::path::Path;

/// Detects and analyzes Node.js projects to determine their type and characteristics.
///
/// This struct provides a unified interface for identifying whether a directory
/// contains a simple Node.js project or a monorepo, and returns appropriate
/// descriptors with project information.
///
/// # Type Parameters
///
/// * `F` - A filesystem implementation that satisfies the `FileSystem` trait.
///   Defaults to `FileSystemManager` for standard operations.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::{ProjectDetector, ProjectConfig};
/// use std::path::Path;
///
/// let detector = ProjectDetector::new();
/// let config = ProjectConfig::new();
///
/// match detector.detect(Path::new("."), &config) {
///     Ok(project) => {
///         println!("Detected {} project", project.as_project_info().kind().name());
///     }
///     Err(e) => eprintln!("Detection failed: {}", e),
/// }
/// ```
pub struct ProjectDetector<F: FileSystem = FileSystemManager> {
    /// Filesystem implementation for file operations
    fs: F,
    /// Monorepo detector for identifying monorepo structures
    monorepo_detector: MonorepoDetector<F>,
}

impl ProjectDetector<FileSystemManager> {
    /// Creates a new `ProjectDetector` with the default filesystem implementation.
    ///
    /// # Returns
    ///
    /// A new `ProjectDetector` instance using the `FileSystemManager`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectDetector;
    ///
    /// let detector = ProjectDetector::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        let fs = FileSystemManager::new();
        Self { monorepo_detector: MonorepoDetector::with_filesystem(fs.clone()), fs }
    }
}

impl<F: FileSystem + Clone> ProjectDetector<F> {
    /// Creates a new `ProjectDetector` with a custom filesystem implementation.
    ///
    /// # Arguments
    ///
    /// * `fs` - The filesystem implementation to use
    ///
    /// # Returns
    ///
    /// A new `ProjectDetector` instance using the provided filesystem.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use sublime_standard_tools::project::ProjectDetector;
    ///
    /// let fs = FileSystemManager::new();
    /// let detector = ProjectDetector::with_filesystem(fs);
    /// ```
    #[must_use]
    pub fn with_filesystem(fs: F) -> Self {
        Self { monorepo_detector: MonorepoDetector::with_filesystem(fs.clone()), fs }
    }

    /// Detects and analyzes a Node.js project at the given path.
    ///
    /// This method examines the directory structure to determine if it contains
    /// a simple Node.js project or a monorepo, then returns an appropriate
    /// descriptor with project information.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to analyze for a Node.js project
    /// * `config` - Configuration options for project detection
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - The path does not exist or cannot be accessed
    /// - The path does not contain a valid Node.js project
    /// - An I/O error occurs while reading project files
    /// - Configuration files cannot be parsed
    ///
    /// # Returns
    ///
    /// * `Ok(ProjectDescriptor)` - A descriptor containing all project information
    /// * `Err(Error)` - If the path is not a valid project or an error occurred
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{ProjectDetector, ProjectConfig};
    /// use std::path::Path;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = ProjectDetector::new();
    /// let config = ProjectConfig::new();
    ///
    /// let project = detector.detect(Path::new("."), &config)?;
    /// match project {
    ///     sublime_standard_tools::project::ProjectDescriptor::Simple(simple) => {
    ///         println!("Found simple project at {}", simple.root().display());
    ///     }
    ///     sublime_standard_tools::project::ProjectDescriptor::Monorepo(monorepo) => {
    ///         println!("Found {} with {} packages", 
    ///                  monorepo.kind().name(),
    ///                  monorepo.packages().len());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn detect(&self, path: impl AsRef<Path>, config: &ProjectConfig) -> Result<ProjectDescriptor> {
        let path = path.as_ref();

        // Ensure the path exists and is a directory
        if !self.fs.exists(path) {
            return Err(Error::operation(format!("Path does not exist: {}", path.display())));
        }

        // Check if it's a valid Node.js project (must have package.json)
        let package_json_path = path.join("package.json");
        if !self.fs.exists(&package_json_path) {
            return Err(Error::operation(format!(
                "No package.json found at: {}",
                package_json_path.display()
            )));
        }

        // Try to detect as monorepo first if enabled
        if config.detect_monorepo {
            if let Ok(monorepo) = self.monorepo_detector.detect_monorepo(path) {
                return Ok(ProjectDescriptor::Monorepo(Box::new(monorepo)));
            }
        }

        // If not a monorepo, treat as simple project
        self.detect_simple_project(path, config)
    }

    /// Detects the type of Node.js project without full analysis.
    ///
    /// This method provides a quick way to determine project type without
    /// loading all project data, useful for lightweight operations.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to analyze
    /// * `config` - Configuration options for detection
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - The path does not exist or cannot be accessed
    /// - The path does not contain a package.json file
    /// - An I/O error occurs while reading files
    ///
    /// # Returns
    ///
    /// * `Ok(ProjectKind)` - The type of project detected
    /// * `Err(Error)` - If detection failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{ProjectDetector, ProjectConfig, ProjectKind};
    /// use std::path::Path;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = ProjectDetector::new();
    /// let config = ProjectConfig::new();
    ///
    /// let kind = detector.detect_kind(Path::new("."), &config)?;
    /// match kind {
    ///     ProjectKind::Simple => println!("Simple project"),
    ///     ProjectKind::Monorepo(monorepo_kind) => {
    ///         println!("Monorepo: {}", monorepo_kind.name());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn detect_kind(&self, path: impl AsRef<Path>, config: &ProjectConfig) -> Result<ProjectKind> {
        let path = path.as_ref();

        // Ensure the path exists and has package.json
        if !self.fs.exists(path) {
            return Err(Error::operation(format!("Path does not exist: {}", path.display())));
        }

        let package_json_path = path.join("package.json");
        if !self.fs.exists(&package_json_path) {
            return Err(Error::operation(format!(
                "No package.json found at: {}",
                package_json_path.display()
            )));
        }

        // Check for monorepo if enabled
        if config.detect_monorepo {
            if let Some(monorepo_kind) = self.monorepo_detector.is_monorepo_root(path)? {
                return Ok(ProjectKind::Monorepo(monorepo_kind));
            }
        }

        Ok(ProjectKind::Simple)
    }

    /// Determines if a path contains a valid Node.js project.
    ///
    /// This method performs a basic validation to check if the path
    /// contains the minimum requirements for a Node.js project.
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
    /// use sublime_standard_tools::project::ProjectDetector;
    /// use std::path::Path;
    ///
    /// let detector = ProjectDetector::new();
    /// if detector.is_valid_project(Path::new(".")) {
    ///     println!("Current directory is a valid Node.js project");
    /// }
    /// ```
    #[must_use]
    pub fn is_valid_project(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();

        // Must be a directory
        if !self.fs.exists(path) {
            return false;
        }

        // Must have package.json
        let package_json_path = path.join("package.json");
        if !self.fs.exists(&package_json_path) {
            return false;
        }

        // Try to parse package.json to ensure it's valid
        if let Ok(content) = self.fs.read_file_string(&package_json_path) {
            serde_json::from_str::<PackageJson>(&content).is_ok()
        } else {
            false
        }
    }

    /// Detects and creates a simple project descriptor.
    ///
    /// This method creates a `SimpleProject` with the specified configuration,
    /// optionally loading package manager and package.json information.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the simple project
    /// * `config` - Configuration options for project detection
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - The package.json file cannot be read or parsed
    /// - Package manager detection fails when enabled
    /// - An I/O error occurs while reading files
    ///
    /// # Returns
    ///
    /// * `Ok(ProjectDescriptor::Simple)` - A descriptor for the simple project
    /// * `Err(Error)` - If project creation failed
    fn detect_simple_project(
        &self,
        path: &Path,
        config: &ProjectConfig,
    ) -> Result<ProjectDescriptor> {
        let mut package_manager = None;
        let mut package_json = None;

        // Load package.json if it exists
        let package_json_path = path.join("package.json");
        if self.fs.exists(&package_json_path) {
            let content = self.fs.read_file_string(&package_json_path)?;
            package_json = Some(
                serde_json::from_str::<PackageJson>(&content)
                    .map_err(|e| Error::operation(format!("Invalid package.json: {e}")))?,
            );
        }

        // Detect package manager if enabled
        if config.detect_package_manager {
            match PackageManager::detect(path) {
                Ok(pm) => package_manager = Some(pm),
                Err(_) => {
                    // Package manager detection failure is not fatal for simple projects
                    // We'll create the project without a package manager
                }
            }
        }

        let simple_project = SimpleProject::new(path.to_path_buf(), package_manager, package_json);

        Ok(ProjectDescriptor::Simple(Box::new(simple_project)))
    }
}

impl<F: FileSystem + Clone> Default for ProjectDetector<F>
where
    F: Default,
{
    fn default() -> Self {
        let fs = F::default();
        Self { monorepo_detector: MonorepoDetector::with_filesystem(fs.clone()), fs }
    }
}