//! # Project Detection Implementation - Async Only
//!
//! ## What
//! This file implements async project detection, providing a unified interface
//! for detecting and analyzing Node.js projects using async I/O operations.
//! All sync operations have been removed for architectural clarity.
//!
//! ## How
//! The detector uses async filesystem operations to analyze project structures,
//! eliminating code duplication by centralizing common operations like package.json
//! parsing and package manager detection.
//!
//! ## Why
//! Async project detection is essential for performance when dealing with large
//! monorepos where multiple projects need to be detected concurrently. This unified
//! async-only approach eliminates confusion and provides consistent API.

use super::types::{ProjectConfig, ProjectDescriptor, ProjectKind, ProjectValidationStatus};
use super::Project;
use crate::error::{Error, Result};
use crate::filesystem::{AsyncFileSystem, FileSystemManager};
use crate::monorepo::{MonorepoDetector, MonorepoDetectorTrait};
use crate::node::{PackageManager, RepoKind};
use async_trait::async_trait;
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

/// Async trait for project detection.
///
/// This trait provides async methods for detecting and analyzing Node.js projects
/// in a non-blocking manner, allowing for concurrent detection operations.
///
/// # Examples
///
/// ```rust
/// use sublime_standard_tools::project::{ProjectDetector, ProjectDetectorTrait, ProjectConfig};
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let detector = ProjectDetector::new();
/// let config = ProjectConfig::new();
/// let project = detector.detect(Path::new("."), &config).await?;
/// println!("Found project: {:?}", project.as_project_info().kind());
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait ProjectDetectorTrait: Send + Sync {
    /// Asynchronously detects and analyzes a project at the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to analyze for project detection
    /// * `config` - Configuration options for project detection
    ///
    /// # Returns
    ///
    /// * `Ok(ProjectDescriptor)` - The detected project descriptor
    /// * `Err(Error)` - If detection fails or no project is found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::project::{ProjectDetector, ProjectDetectorTrait, ProjectConfig};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = ProjectDetector::new();
    /// let config = ProjectConfig::new();
    /// let project = detector.detect(Path::new("."), &config).await?;
    /// println!("Detected project kind: {:?}", project.as_project_info().kind());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist
    /// - No valid project is found at the path
    /// - Filesystem operations fail
    /// - Project structure is invalid
    async fn detect(&self, path: &Path, config: &ProjectConfig) -> Result<ProjectDescriptor>;

    /// Asynchronously detects only the project kind without full analysis.
    ///
    /// This method is faster than full detection as it only determines the project
    /// type without analyzing the complete project structure.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to analyze for project kind detection
    /// * `config` - Configuration options for project detection
    ///
    /// # Returns
    ///
    /// * `Ok(ProjectKind)` - The detected project kind
    /// * `Err(Error)` - If detection fails or no project is found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::project::{ProjectDetector, ProjectDetectorTrait, ProjectConfig};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = ProjectDetector::new();
    /// let config = ProjectConfig::new();
    /// let kind = detector.detect_kind(Path::new("."), &config).await?;
    /// println!("Project kind: {:?}", kind);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist
    /// - No valid project is found at the path
    /// - Filesystem operations fail
    async fn detect_kind(&self, path: &Path, config: &ProjectConfig) -> Result<ProjectKind>;

    /// Asynchronously checks if the path contains a valid Node.js project.
    ///
    /// This method performs a quick check to determine if a path contains a
    /// valid Node.js project without full analysis.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check for project validity
    ///
    /// # Returns
    ///
    /// * `true` - If the path contains a valid Node.js project
    /// * `false` - If the path does not contain a valid Node.js project
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::project::{ProjectDetector, ProjectDetectorTrait};
    /// use std::path::Path;
    ///
    /// # async fn example() {
    /// let detector = ProjectDetector::new();
    /// if detector.is_valid_project(Path::new(".")).await {
    ///     println!("This is a valid Node.js project");
    /// } else {
    ///     println!("This is not a valid Node.js project");
    /// }
    /// # }
    /// ```
    async fn is_valid_project(&self, path: &Path) -> bool;
}

/// Async trait for project detection with custom filesystem.
///
/// This trait extends `ProjectDetectorTrait` to allow custom filesystem implementations
/// for testing or specialized use cases.
///
/// # Type Parameters
///
/// * `F` - The filesystem implementation type
///
/// # Examples
///
/// ```rust
/// use sublime_standard_tools::project::{ProjectDetector, ProjectDetectorWithFs, ProjectConfig};
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = FileSystemManager::new();
/// let detector = ProjectDetector::with_filesystem(fs);
/// let config = ProjectConfig::new();
/// let project = detector.detect(Path::new("."), &config).await?;
/// println!("Found project: {:?}", project.as_project_info().kind());
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait ProjectDetectorWithFs<F: AsyncFileSystem>: ProjectDetectorTrait {
    /// Gets a reference to the filesystem implementation.
    ///
    /// # Returns
    ///
    /// A reference to the filesystem implementation.
    fn filesystem(&self) -> &F;

    /// Asynchronously detects projects in multiple paths concurrently.
    ///
    /// This method processes multiple paths in parallel for improved performance
    /// when analyzing multiple project directories.
    ///
    /// # Arguments
    ///
    /// * `paths` - A slice of paths to analyze for projects
    /// * `config` - Configuration options for project detection
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Result<ProjectDescriptor>>)` - Results for each path
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::project::{ProjectDetector, ProjectDetectorWithFs, ProjectConfig};
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let detector = ProjectDetector::with_filesystem(fs);
    /// let config = ProjectConfig::new();
    /// let paths = vec![Path::new("."), Path::new("../other-project")];
    /// let results = detector.detect_multiple(&paths, &config).await;
    /// for (i, result) in results.iter().enumerate() {
    ///     match result {
    ///         Ok(project) => println!("Path {}: {:?}", i, project.as_project_info().kind()),
    ///         Err(e) => println!("Path {}: Error - {}", i, e),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Each result in the vector may contain an error if detection fails for that path.
    async fn detect_multiple(
        &self,
        paths: &[&Path],
        config: &ProjectConfig,
    ) -> Vec<Result<ProjectDescriptor>>;
}

/// Provides unified detection and analysis of Node.js projects.
///
/// This detector implements a truly unified approach to project detection,
/// eliminating code duplication by centralizing common operations and using
/// async I/O operations for maximum performance.
///
/// # Type Parameters
///
/// * `F` - An async filesystem implementation that satisfies the `AsyncFileSystem` trait.
///   Defaults to `FileSystemManager` for standard operations.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::{ProjectDetector, ProjectConfig};
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let detector = ProjectDetector::new();
/// let config = ProjectConfig::new();
///
/// match detector.detect(Path::new("."), &config).await {
///     Ok(project) => {
///         println!("Detected {} project", project.as_project_info().kind().name());
///     }
///     Err(e) => eprintln!("Detection failed: {}", e),
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ProjectDetector<F: AsyncFileSystem = FileSystemManager> {
    /// Async filesystem implementation for file operations
    fs: F,
    /// Monorepo detector for identifying monorepo structures
    monorepo_detector: MonorepoDetector<F>,
}

impl ProjectDetector<FileSystemManager> {
    /// Creates a new `ProjectDetector` with the default async filesystem implementation.
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

impl<F: AsyncFileSystem + Clone> ProjectDetector<F> {
    /// Creates a new `ProjectDetector` with a custom async filesystem implementation.
    ///
    /// # Arguments
    ///
    /// * `fs` - The async filesystem implementation to use
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
    async fn load_simple_project_metadata(&self, path: &Path, config: &ProjectConfig) -> Result<ProjectMetadata> {
        let mut metadata = ProjectMetadata::new(path.to_path_buf());

        // Load package.json if it exists
        let package_json_path = path.join("package.json");
        if self.fs.exists(&package_json_path).await {
            let content = self.fs.read_file_string(&package_json_path).await?;
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
    /// async analysis.
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
    pub async fn detect(&self, path: impl AsRef<Path>, config: &ProjectConfig) -> Result<ProjectDescriptor> {
        let path = path.as_ref();
        
        // First validate basic path requirements
        self.validate_project_path(path).await?;
        
        // Create a unified project structure
        let metadata = self.load_simple_project_metadata(path, config).await?;
        
        // Determine project kind based on monorepo detection
        let project_kind = if config.detect_monorepo {
            if let Ok(monorepo) = self.monorepo_detector.detect_monorepo(path).await {
                // It's a monorepo, use the detected monorepo kind
                ProjectKind::Repository(RepoKind::Monorepo(monorepo.kind))
            } else {
                // Not a monorepo, it's a simple project
                ProjectKind::Repository(RepoKind::Simple)
            }
        } else {
            // Monorepo detection disabled, treat as simple project
            ProjectKind::Repository(RepoKind::Simple)
        };
        
        // Create unified Project with detected metadata
        let mut project = Project::with_config(
            metadata.root,
            project_kind,
            config.clone(),
        );
        
        // Set detected metadata
        project.package_manager = metadata.package_manager;
        project.package_json = metadata.package_json;
        project.validation_status = metadata.validation_status;
        
        // If it's a monorepo, populate internal dependencies
        if project.is_monorepo() {
            if let Ok(monorepo) = self.monorepo_detector.detect_monorepo(path).await {
                project.internal_dependencies = monorepo.packages;
            }
        }
        
        Ok(ProjectDescriptor::NodeJs(project))
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
    pub async fn detect_kind(&self, path: impl AsRef<Path>, config: &ProjectConfig) -> Result<ProjectKind> {
        let path = path.as_ref();
        
        // Validate path and package.json existence
        self.validate_project_path(path).await?;
        
        // Check for monorepo if enabled (same logic as detect())
        if config.detect_monorepo {
            if let Some(monorepo_kind) = self.monorepo_detector.is_monorepo_root(path).await? {
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
    async fn validate_project_path(&self, path: &Path) -> Result<()> {
        if !self.fs.exists(path).await {
            return Err(Error::operation(format!("Path does not exist: {}", path.display())));
        }

        let package_json_path = path.join("package.json");
        if !self.fs.exists(&package_json_path).await {
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
    /// # async fn example() {
    /// let detector = ProjectDetector::new();
    /// if detector.is_valid_project(Path::new(".")).await {
    ///     println!("Current directory is a valid Node.js project");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub async fn is_valid_project(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();
        
        // Use unified validation logic
        if self.validate_project_path(path).await.is_err() {
            return false;
        }

        // Additional validation: ensure package.json can be parsed
        let package_json_path = path.join("package.json");
        if let Ok(content) = self.fs.read_file_string(&package_json_path).await {
            serde_json::from_str::<PackageJson>(&content).is_ok()
        } else {
            false
        }
    }
}

#[async_trait]
impl<F: AsyncFileSystem + Clone> ProjectDetectorTrait for ProjectDetector<F> {
    async fn detect(&self, path: &Path, config: &ProjectConfig) -> Result<ProjectDescriptor> {
        self.detect(path, config).await
    }

    async fn detect_kind(&self, path: &Path, config: &ProjectConfig) -> Result<ProjectKind> {
        self.detect_kind(path, config).await
    }

    async fn is_valid_project(&self, path: &Path) -> bool {
        self.is_valid_project(path).await
    }
}

#[async_trait]
impl<F: AsyncFileSystem + Clone> ProjectDetectorWithFs<F> for ProjectDetector<F> {
    fn filesystem(&self) -> &F {
        &self.fs
    }

    async fn detect_multiple(
        &self,
        paths: &[&Path],
        config: &ProjectConfig,
    ) -> Vec<Result<ProjectDescriptor>> {
        let mut results = Vec::with_capacity(paths.len());
        
        // Process all paths concurrently
        let futures = paths.iter().map(|path| self.detect(path, config));
        
        // Collect all results
        for future in futures {
            results.push(future.await);
        }
        
        results
    }
}

impl<F: AsyncFileSystem + Clone> Default for ProjectDetector<F>
where
    F: Default,
{
    fn default() -> Self {
        let fs = F::default();
        Self { monorepo_detector: MonorepoDetector::with_filesystem(fs.clone()), fs }
    }
}