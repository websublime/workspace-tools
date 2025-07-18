//! # Project Type Definitions
//!
//! ## What
//! This file defines the core types and traits for representing Node.js projects
//! in a unified way, regardless of their structure (simple or monorepo).
//!
//! ## How
//! Types are defined as enums and traits that model project structures and their
//! characteristics. The `ProjectInfo` trait provides a common interface, while
//! `ProjectKind` and `ProjectDescriptor` handle type differentiation.
//!
//! ## Why
//! A well-defined type system ensures that all project types are represented
//! consistently and safely throughout the codebase, enabling uniform handling
//! of different project structures while maintaining type safety.

use crate::monorepo::MonorepoKind;
use crate::node::{PackageManager, RepoKind};
use package_json::PackageJson;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

/// Represents the type of Node.js project.
///
/// This enum uses the repository-first approach where all projects
/// are fundamentally repositories with different characteristics.
/// This creates a cleaner hierarchy and better separation of concerns.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::ProjectKind;
/// use sublime_standard_tools::node::RepoKind;
/// use sublime_standard_tools::monorepo::MonorepoKind;
///
/// let simple = ProjectKind::Repository(RepoKind::Simple);
/// let yarn_mono = ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::YarnWorkspaces));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectKind {
    /// A repository-based project (simple or monorepo)
    Repository(RepoKind),
}

impl ProjectKind {
    /// Returns a human-readable name for the project kind.
    ///
    /// This method delegates to the underlying repository kind for
    /// consistent naming across the type hierarchy.
    ///
    /// # Returns
    ///
    /// A string describing the project type.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::node::RepoKind;
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// assert_eq!(ProjectKind::Repository(RepoKind::Simple).name(), "simple");
    /// assert_eq!(ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::YarnWorkspaces)).name(), "yarn monorepo");
    /// ```
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Repository(repo_kind) => repo_kind.name(),
        }
    }

    /// Checks if this is a monorepo project.
    ///
    /// This method delegates to the underlying repository kind for
    /// consistent behavior across the type hierarchy.
    ///
    /// # Returns
    ///
    /// `true` if the project is any type of monorepo, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::node::RepoKind;
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// assert!(!ProjectKind::Repository(RepoKind::Simple).is_monorepo());
    /// assert!(ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::NpmWorkSpace)).is_monorepo());
    /// ```
    #[must_use]
    pub fn is_monorepo(&self) -> bool {
        match self {
            Self::Repository(repo_kind) => repo_kind.is_monorepo(),
        }
    }

    /// Returns the repository kind for this project.
    ///
    /// This provides direct access to the underlying repository type
    /// for repository-specific operations.
    ///
    /// # Returns
    ///
    /// A reference to the `RepoKind` representing the repository type.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::node::RepoKind;
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// let simple = ProjectKind::Repository(RepoKind::Simple);
    /// assert_eq!(simple.repo_kind(), &RepoKind::Simple);
    ///
    /// let yarn_mono = ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::YarnWorkspaces));
    /// assert_eq!(yarn_mono.repo_kind(), &RepoKind::Monorepo(MonorepoKind::YarnWorkspaces));
    /// ```
    #[must_use]
    pub fn repo_kind(&self) -> &RepoKind {
        match self {
            Self::Repository(repo_kind) => repo_kind,
        }
    }

    /// Gets the monorepo kind if this is a monorepo project.
    ///
    /// This method delegates to the underlying repository kind for
    /// consistent behavior across the type hierarchy.
    ///
    /// # Returns
    ///
    /// * `Some(&MonorepoKind)` - If this is a monorepo project
    /// * `None` - If this is a simple project
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectKind;
    /// use sublime_standard_tools::node::RepoKind;
    /// use sublime_standard_tools::monorepo::MonorepoKind;
    ///
    /// let simple = ProjectKind::Repository(RepoKind::Simple);
    /// assert_eq!(simple.monorepo_kind(), None);
    ///
    /// let yarn_mono = ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::YarnWorkspaces));
    /// assert_eq!(yarn_mono.monorepo_kind(), Some(&MonorepoKind::YarnWorkspaces));
    /// ```
    #[must_use]
    pub fn monorepo_kind(&self) -> Option<&MonorepoKind> {
        match self {
            Self::Repository(repo_kind) => repo_kind.monorepo_kind(),
        }
    }
}

/// Common interface for all Node.js project types.
///
/// This trait provides a unified API for accessing project information
/// regardless of whether it's a simple repository or a monorepo.
/// All project implementations must provide these basic capabilities.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::ProjectInfo;
/// use std::path::Path;
///
/// fn print_project_info(project: &impl ProjectInfo) {
///     println!("Project root: {}", project.root().display());
///     println!("Project type: {}", project.kind().name());
///     
///     if let Some(pm) = project.package_manager() {
///         println!("Package manager: {:?}", pm.kind());
///     }
/// }
/// ```
pub trait ProjectInfo: Send + Sync {
    /// Returns the root directory of the project.
    ///
    /// # Returns
    ///
    /// A reference to the Path representing the project's root directory.
    fn root(&self) -> &Path;

    /// Returns the package manager for the project, if detected.
    ///
    /// # Returns
    ///
    /// * `Some(&PackageManager)` - If a package manager was detected
    /// * `None` - If no package manager was detected or detection was disabled
    fn package_manager(&self) -> Option<&PackageManager>;

    /// Returns the parsed package.json for the project, if available.
    ///
    /// # Returns
    ///
    /// * `Some(&PackageJson)` - If package.json was successfully parsed
    /// * `None` - If package.json was not found or could not be parsed
    fn package_json(&self) -> Option<&PackageJson>;

    /// Returns the validation status of the project.
    ///
    /// # Returns
    ///
    /// A reference to the `ProjectValidationStatus` indicating the validation state.
    fn validation_status(&self) -> &ProjectValidationStatus;

    /// Returns the kind of project.
    ///
    /// # Returns
    ///
    /// The `ProjectKind` enum value representing the type of project.
    fn kind(&self) -> ProjectKind;
}

/// Represents different types of Node.js projects with their specific data.
///
/// This enum serves as a container that can hold either a simple project
/// or a monorepo descriptor, providing type-safe access to project-specific
/// functionality while maintaining a common interface through the `ProjectInfo` trait.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::{ProjectDescriptor, ProjectInfo};
///
/// fn process_project(descriptor: ProjectDescriptor) {
///     match descriptor {
///         ProjectDescriptor::NodeJs(project) => {
///             println!("Processing project at {}", project.root().display());
///             if project.is_monorepo() {
///                 println!("Type: Monorepo with {} packages", project.internal_dependencies.len());
///             } else {
///                 println!("Type: Simple project");
///             }
///         }
///     }
/// }
/// ```
#[derive(Debug)]
pub enum ProjectDescriptor {
    /// A Node.js project (simple or monorepo)
    NodeJs(super::Project),
}

impl ProjectDescriptor {
    /// Returns a reference to the project as a trait object.
    ///
    /// This method provides unified access to project information
    /// regardless of the underlying type.
    ///
    /// # Returns
    ///
    /// A reference to the project implementing the `ProjectInfo` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::{ProjectDescriptor, ProjectInfo};
    /// # fn example(descriptor: ProjectDescriptor) {
    /// let info = descriptor.as_project_info();
    /// println!("Project type: {}", info.kind().name());
    /// # }
    /// ```
    #[must_use]
    pub fn as_project_info(&self) -> &dyn ProjectInfo {
        match self {
            Self::NodeJs(project) => project,
        }
    }
}

/// Status of a project validation operation.
///
/// This enum represents the different states a project can be in
/// after validation, providing detailed information about any
/// issues found during the validation process.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::ProjectValidationStatus;
///
/// let status = ProjectValidationStatus::Valid;
/// assert!(status.is_valid());
///
/// let warnings = ProjectValidationStatus::Warning(vec!["Missing LICENSE".to_string()]);
/// assert!(!warnings.is_valid());
/// assert!(warnings.has_warnings());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectValidationStatus {
    /// Project structure is valid
    Valid,
    /// Project has warnings but is usable
    Warning(Vec<String>),
    /// Project has errors that need to be fixed
    Error(Vec<String>),
    /// Project has not been validated
    NotValidated,
}

impl ProjectValidationStatus {
    /// Checks if the project validation passed without errors.
    ///
    /// # Returns
    ///
    /// `true` if the status is Valid, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectValidationStatus;
    ///
    /// assert!(ProjectValidationStatus::Valid.is_valid());
    /// assert!(!ProjectValidationStatus::Error(vec!["Missing package.json".to_string()]).is_valid());
    /// ```
    #[must_use]
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    /// Checks if the project has warnings.
    ///
    /// # Returns
    ///
    /// `true` if the status contains warnings, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectValidationStatus;
    ///
    /// let warnings = ProjectValidationStatus::Warning(vec!["Old dependencies".to_string()]);
    /// assert!(warnings.has_warnings());
    /// assert!(!ProjectValidationStatus::Valid.has_warnings());
    /// ```
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        matches!(self, Self::Warning(_))
    }

    /// Checks if the project has errors.
    ///
    /// # Returns
    ///
    /// `true` if the status contains errors, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectValidationStatus;
    ///
    /// let errors = ProjectValidationStatus::Error(vec!["Invalid package.json".to_string()]);
    /// assert!(errors.has_errors());
    /// assert!(!ProjectValidationStatus::Valid.has_errors());
    /// ```
    #[must_use]
    pub fn has_errors(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Gets the list of warnings if any.
    ///
    /// # Returns
    ///
    /// * `Some(&[String])` - If the status contains warnings
    /// * `None` - Otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectValidationStatus;
    ///
    /// let warnings = ProjectValidationStatus::Warning(vec!["Missing README".to_string()]);
    /// assert_eq!(warnings.warnings(), Some(&["Missing README".to_string()][..]));
    /// ```
    #[must_use]
    pub fn warnings(&self) -> Option<&[String]> {
        match self {
            Self::Warning(warnings) => Some(warnings),
            _ => None,
        }
    }

    /// Gets the list of errors if any.
    ///
    /// # Returns
    ///
    /// * `Some(&[String])` - If the status contains errors
    /// * `None` - Otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectValidationStatus;
    ///
    /// let errors = ProjectValidationStatus::Error(vec!["Missing package.json".to_string()]);
    /// assert_eq!(errors.errors(), Some(&["Missing package.json".to_string()][..]));
    /// ```
    #[must_use]
    pub fn errors(&self) -> Option<&[String]> {
        match self {
            Self::Error(errors) => Some(errors),
            _ => None,
        }
    }
}

/// Configuration options for project detection and validation.
///
/// This struct controls how projects are detected and validated,
/// allowing fine-grained control over the detection process.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::ProjectConfig;
///
/// let config = ProjectConfig::new()
///     .with_detect_package_manager(true)
///     .with_validate_structure(true)
///     .with_detect_monorepo(true);
/// ```
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    /// Root directory for project detection (None uses current directory)
    pub(crate) root: Option<PathBuf>,
    /// Whether to detect the package manager
    pub(crate) detect_package_manager: bool,
    /// Whether to validate project structure
    pub(crate) validate_structure: bool,
    /// Whether to detect monorepo structures
    pub(crate) detect_monorepo: bool,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectConfig {
    /// Creates a new `ProjectConfig` with default values.
    ///
    /// Default settings:
    /// - No specified root (uses current directory)
    /// - Package manager detection enabled
    /// - Structure validation enabled
    /// - Monorepo detection enabled
    ///
    /// # Returns
    ///
    /// A new `ProjectConfig` with default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ProjectConfig;
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
    /// use sublime_standard_tools::project::ProjectConfig;
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
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .with_detect_package_manager(false);
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
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .with_validate_structure(false);
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
    /// use sublime_standard_tools::project::ProjectConfig;
    ///
    /// let config = ProjectConfig::new()
    ///     .with_detect_monorepo(false);
    /// ```
    #[must_use]
    pub fn with_detect_monorepo(mut self, detect: bool) -> Self {
        self.detect_monorepo = detect;
        self
    }
}

/// Represents a detected Node.js project.
///
/// This struct contains information about a Node.js project,
/// including its root directory, package manager, validation status,
/// and parsed package.json. This is a generic project structure that
/// can represent both simple and monorepo projects.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::{GenericProject, ProjectConfig};
///
/// // Create a new project
/// let config = ProjectConfig::default();
/// let project = GenericProject::new("/path/to/project", config);
///
/// // Access project properties
/// println!("Project root: {}", project.root().display());
/// ```
#[derive(Debug)]
pub struct GenericProject {
    /// Root directory of the project
    pub(crate) root: PathBuf,
    /// Detected package manager (if any)
    pub(crate) package_manager: Option<PackageManager>,
    /// Project configuration
    pub(crate) config: ProjectConfig,
    /// Validation status of the project
    pub(crate) validation: ProjectValidationStatus,
    /// Parsed package.json (if available)
    pub(crate) package_json: Option<PackageJson>,
}

impl GenericProject {
    /// Creates a new GenericProject instance with the specified root and configuration.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the project
    /// * `config` - Configuration options for the project
    ///
    /// # Returns
    ///
    /// A new GenericProject instance configured with the provided options.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::project::{GenericProject, ProjectConfig};
    ///
    /// let config = ProjectConfig::default();
    /// let project = GenericProject::new("/path/to/project", config);
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
    /// # use sublime_standard_tools::project::{GenericProject, ProjectConfig};
    /// #
    /// # let config = ProjectConfig::default();
    /// # let project = GenericProject::new("/path/to/project", config);
    /// #
    /// let root = project.root();
    /// assert_eq!(root, Path::new("/path/to/project"));
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
    /// # use sublime_standard_tools::project::{GenericProject, ProjectConfig};
    /// #
    /// # let config = ProjectConfig::default();
    /// # let project = GenericProject::new("/path/to/project", config);
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
    /// # use sublime_standard_tools::project::{GenericProject, ProjectConfig};
    /// #
    /// # let config = ProjectConfig::default();
    /// # let project = GenericProject::new("/path/to/project", config);
    /// #
    /// match project.validation_status() {
    ///     sublime_standard_tools::project::ProjectValidationStatus::Valid => println!("Project is valid"),
    ///     sublime_standard_tools::project::ProjectValidationStatus::Warning(warnings) => {
    ///         println!("Project has warnings:");
    ///         for warning in warnings {
    ///             println!("  - {}", warning);
    ///         }
    ///     },
    ///     sublime_standard_tools::project::ProjectValidationStatus::Error(errors) => {
    ///         println!("Project has errors:");
    ///         for error in errors {
    ///             println!("  - {}", error);
    ///         }
    ///     },
    ///     sublime_standard_tools::project::ProjectValidationStatus::NotValidated => {
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
    /// # use sublime_standard_tools::project::{GenericProject, ProjectConfig};
    /// #
    /// # let config = ProjectConfig::default();
    /// # let project = GenericProject::new("/path/to/project", config);
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
    /// # use sublime_standard_tools::project::{GenericProject, ProjectConfig};
    /// #
    /// # let config = ProjectConfig::default();
    /// # let project = GenericProject::new("/path/to/project", config);
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

    /// Sets the package manager for this project.
    ///
    /// # Arguments
    ///
    /// * `package_manager` - The package manager to set
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::{GenericProject, ProjectConfig};
    /// # use sublime_standard_tools::node::{PackageManager, PackageManagerKind};
    /// #
    /// # let config = ProjectConfig::default();
    /// # let mut project = GenericProject::new("/path/to/project", config);
    /// #
    /// let manager = PackageManager::new(PackageManagerKind::Npm, "/path/to/project");
    /// project.set_package_manager(Some(manager));
    /// ```
    pub fn set_package_manager(&mut self, package_manager: Option<PackageManager>) {
        self.package_manager = package_manager;
    }

    /// Sets the package.json content for this project.
    ///
    /// # Arguments
    ///
    /// * `package_json` - The package.json content to set
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::{GenericProject, ProjectConfig};
    /// # use package_json::PackageJson;
    /// #
    /// # let config = ProjectConfig::default();
    /// # let mut project = GenericProject::new("/path/to/project", config);
    /// #
    /// // Assuming package_json is loaded from file
    /// // project.set_package_json(Some(package_json));
    /// ```
    pub fn set_package_json(&mut self, package_json: Option<PackageJson>) {
        self.package_json = package_json;
    }

    /// Sets the validation status for this project.
    ///
    /// # Arguments
    ///
    /// * `validation` - The validation status to set
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::{GenericProject, ProjectConfig, ProjectValidationStatus};
    /// #
    /// # let config = ProjectConfig::default();
    /// # let mut project = GenericProject::new("/path/to/project", config);
    /// #
    /// project.set_validation_status(ProjectValidationStatus::Valid);
    /// ```
    pub fn set_validation_status(&mut self, validation: ProjectValidationStatus) {
        self.validation = validation;
    }
}

/// Configuration scope levels.
///
/// This enum defines the different scopes that configuration can apply to,
/// from the most global (system-wide) to the most specific (runtime only).
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::ConfigScope;
///
/// // Accessing different configuration scopes
/// let global = ConfigScope::Global;
/// let user = ConfigScope::User;
/// let project = ConfigScope::Project;
/// let runtime = ConfigScope::Runtime;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigScope {
    /// Global configuration (system-wide)
    Global,
    /// User configuration (user-specific)
    User,
    /// Project configuration (project-specific)
    Project,
    /// Runtime configuration (in-memory only)
    Runtime,
}

/// Configuration file formats.
///
/// This enum defines the supported file formats for configuration files,
/// allowing the system to properly parse and serialize configuration data.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::ConfigFormat;
///
/// // Supporting different configuration formats
/// let json_format = ConfigFormat::Json;
/// let toml_format = ConfigFormat::Toml;
/// let yaml_format = ConfigFormat::Yaml;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// JSON format
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
}

/// A configuration value that can represent different data types.
///
/// This enum provides a flexible way to store and manipulate configuration values
/// of various types, including strings, numbers, booleans, arrays, and nested maps.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use sublime_standard_tools::project::ConfigValue;
///
/// // String value
/// let name = ConfigValue::String("Project Name".to_string());
///
/// // Boolean value
/// let debug_mode = ConfigValue::Boolean(true);
///
/// // Number value
/// let version = ConfigValue::Float(1.5);
///
/// // Array value
/// let tags = ConfigValue::Array(vec![
///     ConfigValue::String("tag1".to_string()),
///     ConfigValue::String("tag2".to_string()),
/// ]);
///
/// // Map (object) value
/// let mut settings = HashMap::new();
/// settings.insert("name".to_string(), ConfigValue::String("My Project".to_string()));
/// settings.insert("version".to_string(), ConfigValue::Float(2.0));
/// let config = ConfigValue::Map(settings);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Array of values
    Array(Vec<ConfigValue>),
    /// Map of values
    Map(HashMap<String, ConfigValue>),
    /// Null value
    Null,
}

/// Manages configuration across different scopes and file formats.
///
/// This struct provides functionality to load, save, and manipulate configuration
/// settings. It supports multiple scopes (global, user, project, runtime) and
/// different file formats (JSON, TOML, YAML).
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::{ConfigManager, ConfigScope, ConfigValue};
/// use std::path::PathBuf;
///
/// // Create a new configuration manager
/// let mut config_manager = ConfigManager::new();
///
/// // Set paths for different configuration scopes
/// config_manager.set_path(ConfigScope::User, PathBuf::from("~/.config/myapp.json"));
/// config_manager.set_path(ConfigScope::Project, PathBuf::from("./project-config.json"));
///
/// // Set a configuration value
/// config_manager.set("theme", ConfigValue::String("dark".to_string()));
///
/// // Get a configuration value
/// if let Some(theme) = config_manager.get("theme") {
///     if let Some(theme_str) = theme.as_string() {
///         println!("Current theme: {}", theme_str);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ConfigManager {
    /// Configuration settings
    pub(crate) settings: Arc<RwLock<HashMap<String, ConfigValue>>>,
    /// Paths for different configuration scopes
    pub(crate) files: HashMap<ConfigScope, PathBuf>,
}

