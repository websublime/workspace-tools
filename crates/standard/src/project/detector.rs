//! # Project Detection Implementation
//!
//! ## What
//! This file implements the `ProjectDetector` struct, which provides a truly
//! unified interface for detecting and analyzing Node.js projects, whether they
//! are simple repositories or monorepos.
//!
//! ## How
//! The detector uses a unified detection strategy that eliminates code duplication
//! by centralizing common operations like package.json parsing and package manager
//! detection. It determines project type through a single analysis pass and
//! constructs appropriate descriptors based on the results.
//!
//! ## Why
//! A truly unified detector eliminates maintenance overhead, ensures consistency
//! across project types, and provides a single source of truth for project
//! detection logic. This approach follows DRY principles and makes the codebase
//! more maintainable and robust.

use super::types::{ProjectConfig, ProjectDescriptor, ProjectKind, ProjectValidationStatus};
use super::SimpleProject;
use crate::error::{Error, Result};
use crate::filesystem::{FileSystem, FileSystemManager};
use crate::monorepo::MonorepoDetector;
use crate::node::{PackageManager, RepoKind};
use package_json::PackageJson;
use std::path::{Path, PathBuf};

/// Internal structure to hold unified project metadata.
///
/// This structure centralizes common project information that is used
/// across both simple and monorepo project types.
#[derive(Debug)]
struct ProjectMetadata {
    /// The root path of the project
    root: PathBuf,
    /// Parsed package.json content, if available
    package_json: Option<PackageJson>,
    /// Detected package manager, if any
    package_manager: Option<PackageManager>,
    /// Validation status for the project structure
    validation_status: ProjectValidationStatus,
}

/// Provides unified detection and analysis of Node.js projects.
///
/// This detector implements a truly unified approach to project detection,
/// eliminating code duplication by centralizing common operations and using
/// a single analysis strategy for all project types.
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
#[derive(Debug)]
pub struct ProjectDetector<F: FileSystem = FileSystemManager> {
    /// Filesystem implementation for file operations
    fs: F,
    /// Monorepo detector for identifying monorepo structures
    monorepo_detector: MonorepoDetector<F>,
}

impl ProjectMetadata {
    /// Creates new project metadata with the given root path.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the project
    ///
    /// # Returns
    ///
    /// A new `ProjectMetadata` instance with unvalidated status.
    fn new(root: PathBuf) -> Self {
        Self {
            root,
            package_json: None,
            package_manager: None,
            validation_status: ProjectValidationStatus::NotValidated,
        }
    }
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

    /// Loads metadata specifically for simple projects.
    ///
    /// This method centralizes metadata loading for simple projects,
    /// eliminating code duplication from the original detect_simple_project method.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to analyze
    /// * `config` - Configuration options for detection
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - The package.json file cannot be read or parsed
    /// - An I/O error occurs while reading files
    ///
    /// # Returns
    ///
    /// * `Ok(ProjectMetadata)` - Loaded project metadata
    /// * `Err(Error)` - If metadata loading failed
    fn load_simple_project_metadata(&self, path: &Path, config: &ProjectConfig) -> Result<ProjectMetadata> {
        let mut metadata = ProjectMetadata::new(path.to_path_buf());

        // Load package.json if it exists
        let package_json_path = path.join("package.json");
        if self.fs.exists(&package_json_path) {
            let content = self.fs.read_file_string(&package_json_path)?;
            metadata.package_json = Some(
                serde_json::from_str::<PackageJson>(&content)
                    .map_err(|e| Error::operation(format!("Invalid package.json: {e}")))?,
            );
        }

        // Detect package manager if enabled
        if config.detect_package_manager {
            if let Ok(pm) = PackageManager::detect(path) {
                metadata.package_manager = Some(pm);
            }
            // Package manager detection failure is not fatal for simple projects
        }

        // Set validation status - simple projects start as not validated
        metadata.validation_status = ProjectValidationStatus::NotValidated;

        Ok(metadata)
    }

    /// Detects and analyzes a Node.js project using unified detection logic.
    ///
    /// This method uses a truly unified approach that eliminates code duplication
    /// by centralizing common operations and determining project type through
    /// a single analysis pass.
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
        
        // First validate basic path requirements
        self.validate_project_path(path)?;
        
        // Try to detect as monorepo first if enabled (preserving original behavior)
        if config.detect_monorepo {
            if let Ok(monorepo) = self.monorepo_detector.detect_monorepo(path) {
                return Ok(ProjectDescriptor::Monorepo(Box::new(monorepo)));
            }
        }
        
        // If not a monorepo, create simple project with unified metadata loading
        let metadata = self.load_simple_project_metadata(path, config)?;
        let simple_project = SimpleProject::with_validation(
            metadata.root,
            metadata.package_manager,
            metadata.package_json,
            metadata.validation_status,
        );
        
        Ok(ProjectDescriptor::Simple(Box::new(simple_project)))
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
    ///     ProjectKind::Repository(repo_kind) => {
    ///         if let Some(monorepo_kind) = repo_kind.monorepo_kind() {
    ///             println!("Monorepo: {}", monorepo_kind.name());
    ///         } else {
    ///             println!("Simple project");
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn detect_kind(&self, path: impl AsRef<Path>, config: &ProjectConfig) -> Result<ProjectKind> {
        let path = path.as_ref();
        
        // Validate path and package.json existence
        self.validate_project_path(path)?;
        
        // Check for monorepo if enabled (same logic as detect())
        if config.detect_monorepo {
            if let Some(monorepo_kind) = self.monorepo_detector.is_monorepo_root(path)? {
                return Ok(ProjectKind::Repository(RepoKind::Monorepo(monorepo_kind)));
            }
        }
        
        Ok(ProjectKind::Repository(RepoKind::Simple))
    }

    /// Validates that a path contains the basic requirements for a Node.js project.
    ///
    /// This method centralizes path validation logic used across detection methods.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to validate
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - The path does not exist
    /// - The path does not contain a package.json file
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the path is valid
    /// * `Err(Error)` - If validation failed
    fn validate_project_path(&self, path: &Path) -> Result<()> {
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
        
        Ok(())
    }
    
    /// Determines if a path contains a valid Node.js project.
    ///
    /// This method performs a comprehensive validation to check if the path
    /// contains all requirements for a valid Node.js project.
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
        
        // Use unified validation logic
        if self.validate_project_path(path).is_err() {
            return false;
        }

        // Additional validation: ensure package.json can be parsed
        let package_json_path = path.join("package.json");
        if let Ok(content) = self.fs.read_file_string(&package_json_path) {
            serde_json::from_str::<PackageJson>(&content).is_ok()
        } else {
            false
        }
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