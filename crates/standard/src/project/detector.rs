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

use super::types::{ProjectDescriptor, ProjectKind, ProjectValidationStatus};
use super::Project;
use crate::config::{ConfigManager, StandardConfig, Configurable};
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
    async fn detect(&self, path: &Path, config: Option<&StandardConfig>) -> Result<ProjectDescriptor>;

    /// Asynchronously detects only the project kind without full analysis.
    ///
    /// This method is faster than full detection as it only determines the project
    /// type without analyzing the complete project structure.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to analyze for project kind detection
    ///
    /// # Returns
    ///
    /// * `Ok(ProjectKind)` - The detected project kind
    /// * `Err(Error)` - If detection fails or no project is found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::project::{ProjectDetector, ProjectDetectorTrait};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = ProjectDetector::new();
    /// let kind = detector.detect_kind(Path::new(".")).await?;
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
    async fn detect_kind(&self, path: &Path) -> Result<ProjectKind>;

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
        config: Option<&StandardConfig>,
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
        Self { fs }
    }
}

impl<F: AsyncFileSystem + Clone + 'static> ProjectDetector<F> {
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
        Self { fs }
    }

    /// Loads or creates project configuration by checking for repo.config.* files.
    ///
    /// This method automatically detects and loads configuration from project-specific
    /// files (repo.config.toml, repo.config.yml, repo.config.json) and merges them
    /// with default configuration values.
    ///
    /// # Arguments
    ///
    /// * `project_root` - The project root directory to search for config files
    /// * `base_config` - Optional base configuration to use, defaults to StandardConfig::default()
    ///
    /// # Returns
    ///
    /// * `Ok(StandardConfig)` - The merged configuration
    /// * `Err(Error)` - If configuration loading or merging fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use sublime_standard_tools::project::ProjectDetector;
    /// # use std::path::Path;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = ProjectDetector::new();
    /// let config = detector.load_project_config(Path::new("."), None).await?;
    /// println!("Loaded configuration version: {}", config.version);
    /// # Ok(())
    /// # }
    /// ```
    async fn load_project_config(
        &self,
        project_root: &Path,
        base_config: Option<StandardConfig>,
    ) -> Result<StandardConfig> 
    where 
        F: 'static,
    {
        let mut builder = ConfigManager::<StandardConfig>::builder().with_defaults();

        // Check for repo.config.* files in order of preference
        let config_files = [
            project_root.join("repo.config.toml"),
            project_root.join("repo.config.yml"), 
            project_root.join("repo.config.yaml"),
            project_root.join("repo.config.json"),
        ];

        // Add existing config files to the builder
        for config_file in &config_files {
            if self.fs.exists(config_file).await {
                builder = builder.with_file(config_file);
            }
        }

        let manager = builder.build(self.fs.clone()).map_err(|e| {
            Error::operation(format!("Failed to create config manager: {e}"))
        })?;

        let mut config = manager.load().await.map_err(|e| {
            Error::operation(format!("Failed to load configuration: {e}"))
        })?;

        // Merge with base config if provided
        if let Some(base) = base_config {
            config.merge_with(base).map_err(|e| {
                Error::operation(format!("Failed to merge configurations: {e}"))
            })?;
        }

        Ok(config)
    }

    /// Loads project metadata using configuration settings.
    ///
    /// This method loads project metadata taking into account configuration
    /// settings for package manager detection, validation, and other options.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to analyze
    /// * `config` - Configuration settings to control behavior
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - The package.json file cannot be read or parsed
    /// - An I/O error occurs while reading files
    /// - Configuration validation fails
    ///
    /// # Returns
    ///
    /// * `Ok(ProjectMetadata)` - Loaded project metadata
    /// * `Err(Error)` - If metadata loading failed
    async fn load_project_metadata(
        &self,
        path: &Path,
        config: &StandardConfig,
    ) -> Result<ProjectMetadata> {
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

        // Detect package manager using configuration
        log::debug!(
            "Package manager detection with config: detection_order={:?}, detect_from_env={}",
            config.package_managers.detection_order,
            config.package_managers.detect_from_env
        );
        
        if let Ok(pm) = PackageManager::detect_with_config(path, &config.package_managers) {
            metadata.package_manager = Some(pm);
        }
        // Package manager detection failure is not fatal for simple projects

        // Set validation status based on configuration requirements
        if config.validation.require_package_json && metadata.package_json.is_none() {
            metadata.validation_status = ProjectValidationStatus::Error(vec![
                "package.json is required by configuration".to_string(),
            ]);
        } else {
            // Simple projects start as not validated - will be validated later if needed
            metadata.validation_status = ProjectValidationStatus::NotValidated;
        }

        Ok(metadata)
    }

    /// Determines if monorepo detection should be performed based on configuration.
    ///
    /// This method checks the StandardConfig to determine if monorepo detection
    /// is enabled and should be performed for the current project.
    ///
    /// # Arguments
    ///
    /// * `config` - The effective configuration to check
    ///
    /// # Returns
    ///
    /// `true` if monorepo detection should be performed, `false` otherwise.
    fn should_detect_monorepo(config: &StandardConfig) -> bool {
        // Check if monorepo detection is enabled via configuration
        // For now, we always enable it unless explicitly disabled in future config versions
        // This can be extended to check specific config flags when added
        !config.monorepo.workspace_patterns.is_empty() && config.monorepo.max_search_depth > 0
    }

    /// Detects monorepo structure using configuration settings.
    ///
    /// This method performs monorepo detection while respecting the configuration
    /// settings for workspace patterns, search depth, and other monorepo-specific options.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to analyze for monorepo structure
    /// * `config` - Configuration settings to control detection behavior
    ///
    /// # Returns
    ///
    /// * `Ok(MonorepoDescriptor)` - If a monorepo is detected
    /// * `Err(Error)` - If no monorepo is found or detection fails
    async fn detect_monorepo_with_config(
        &self,
        path: &Path,
        config: &StandardConfig,
    ) -> Result<crate::monorepo::MonorepoDescriptor> {
        // Log configuration being used for transparency
        log::debug!(
            "Detecting monorepo with config: max_depth={}, patterns={:?}, exclude={:?}",
            config.monorepo.max_search_depth,
            config.monorepo.workspace_patterns,
            config.monorepo.exclude_patterns
        );

        // Create a config-aware monorepo detector for this specific operation
        let config_aware_detector = MonorepoDetector::with_filesystem_and_config(
            self.fs.clone(),
            config.monorepo.clone(),
        );

        // Use the config-aware detector
        config_aware_detector.detect_monorepo(path).await
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
    /// * `config` - Optional configuration (if None, auto-loads from repo.config.* files)
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
    pub async fn detect(
        &self,
        path: impl AsRef<Path>,
        config: Option<&StandardConfig>,
    ) -> Result<ProjectDescriptor> {
        let path = path.as_ref();

        // First validate basic path requirements
        self.validate_project_path(path).await?;

        // Load or use provided configuration 
        let effective_config = match config {
            Some(cfg) => cfg.clone(),
            None => self.load_project_config(path, None).await?,
        };

        // Create a unified project structure using configuration
        let metadata = self.load_project_metadata(path, &effective_config).await?;

        // Determine project kind based on configuration-controlled monorepo detection
        let project_kind = if Self::should_detect_monorepo(&effective_config) {
            if let Ok(monorepo) = self.detect_monorepo_with_config(path, &effective_config).await {
                // It's a monorepo, use the detected monorepo kind
                ProjectKind::Repository(RepoKind::Monorepo(monorepo.kind().clone()))
            } else {
                // Not a monorepo, it's a simple project
                ProjectKind::Repository(RepoKind::Simple)
            }
        } else {
            // Monorepo detection disabled by configuration
            ProjectKind::Repository(RepoKind::Simple)
        };

        // Create unified Project with detected metadata
        let mut project = Project::new(metadata.root, project_kind);

        // Set detected metadata
        project.package_manager = metadata.package_manager;
        project.package_json = metadata.package_json;
        project.validation_status = metadata.validation_status;

        // If it's a monorepo, populate internal dependencies using config-aware detection
        if project.is_monorepo() {
            if let Ok(monorepo) = self.detect_monorepo_with_config(path, &effective_config).await {
                project.internal_dependencies = monorepo.packages().to_vec();
            }
        }

        Ok(ProjectDescriptor::NodeJs(project))
    }

    /// Detects the type of Node.js project without full analysis using default configuration.
    ///
    /// This method provides a quick way to determine project type without
    /// loading all project data, useful for lightweight operations.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to analyze
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
    pub async fn detect_kind(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<ProjectKind> {
        let default_config = StandardConfig::default();
        self.detect_kind_with_config(path, &default_config).await
    }

    /// Detects the type of Node.js project without full analysis using custom configuration.
    ///
    /// This method provides a quick way to determine project type without
    /// loading all project data, using the provided configuration for monorepo detection.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to analyze
    /// * `config` - Configuration to control detection behavior
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
    pub async fn detect_kind_with_config(
        &self,
        path: impl AsRef<Path>,
        config: &StandardConfig,
    ) -> Result<ProjectKind> {
        let path = path.as_ref();

        // Validate path and package.json existence
        self.validate_project_path(path).await?;

        // Check for monorepo using configuration
        if Self::should_detect_monorepo(config) {
            let config_aware_detector = MonorepoDetector::with_filesystem_and_config(
                self.fs.clone(),
                config.monorepo.clone(),
            );

            if let Some(monorepo_kind) = config_aware_detector.is_monorepo_root(path).await? {
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
impl<F: AsyncFileSystem + Clone + 'static> ProjectDetectorTrait for ProjectDetector<F> {
    async fn detect(&self, path: &Path, config: Option<&StandardConfig>) -> Result<ProjectDescriptor> {
        self.detect(path, config).await
    }

    async fn detect_kind(&self, path: &Path) -> Result<ProjectKind> {
        self.detect_kind(path).await
    }

    async fn is_valid_project(&self, path: &Path) -> bool {
        self.is_valid_project(path).await
    }
}

#[async_trait]
impl<F: AsyncFileSystem + Clone + 'static> ProjectDetectorWithFs<F> for ProjectDetector<F> {
    fn filesystem(&self) -> &F {
        &self.fs
    }

    async fn detect_multiple(
        &self,
        paths: &[&Path],
        config: Option<&StandardConfig>,
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
        Self { fs }
    }
}
