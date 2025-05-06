//! # Monorepo Type Definitions
//!
//! ## What
//! This file defines the core types needed to represent monorepo structures,
//! including the different kinds of monorepos, package information, and
//! the overall monorepo descriptor.
//!
//! ## How
//! Types are defined as enums and structs that model the structure of
//! monorepos and their components. The MonorepoKind enum represents different
//! monorepo implementations, WorkspacePackage stores information about individual
//! packages, and MonorepoDescriptor ties everything together.
//!
//! ## Why
//! A well-defined type system ensures that monorepo structures are represented
//! consistently and safely throughout the codebase, enabling accurate dependency
//! analysis and project structure navigation.

use package_json::PackageJson;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

use crate::filesystem::{FileSystem, FileSystemManager};

/// Represents the type of monorepo system being used.
///
/// Different package managers implement workspace concepts differently,
/// and this enum captures those variations to enable format-specific processing.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::monorepo::MonorepoKind;
///
/// let yarn_monorepo = MonorepoKind::YarnWorkspaces;
/// assert_eq!(yarn_monorepo.name(), "yarn");
/// assert_eq!(yarn_monorepo.config_file(), "package.json");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonorepoKind {
    /// Npm monorepo
    NpmWorkSpace,
    /// Yarn Workspaces monorepo
    YarnWorkspaces,
    /// pnpm Workspaces monorepo
    PnpmWorkspaces,
    /// Bun monorepo
    BunWorkspaces,
    /// Deno Workspaces monorepo
    DenoWorkspaces,
    /// Custom monorepo (generic structure detection)
    Custom {
        /// The name of the custom monorepo kind
        name: String,
        /// The path to the configuration file
        config_file: String,
    },
}

/// Represents a single package within a monorepo workspace.
///
/// Contains information about the package including its name, version,
/// location within the monorepo, and relationships to other workspace packages.
///
/// # Examples
///
/// ```
/// use std::path::{Path, PathBuf};
/// use sublime_standard_tools::monorepo::WorkspacePackage;
///
/// // Create a package representation
/// let package = WorkspacePackage {
///     name: "ui-components".to_string(),
///     version: "1.0.0".to_string(),
///     location: PathBuf::from("packages/ui-components"),
///     absolute_path: PathBuf::from("/projects/my-monorepo/packages/ui-components"),
///     workspace_dependencies: vec!["shared".to_string()],
///     workspace_dev_dependencies: vec!["test-utils".to_string()],
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct WorkspacePackage {
    /// Name of the package
    pub name: String,
    /// Version of the package
    pub version: String,
    /// Location of the package relative to the monorepo root
    pub location: PathBuf,
    /// Absolute path to the package
    pub absolute_path: PathBuf,
    /// Direct dependencies within the workspace
    pub workspace_dependencies: Vec<String>,
    /// Direct dev_dependencies within the workspace
    pub workspace_dev_dependencies: Vec<String>,
}

/// Describes a complete monorepo structure.
///
/// This struct provides the container for all information about a monorepo,
/// including its type, root location, packages, and provides methods for
/// querying relationships between packages.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use sublime_standard_tools::monorepo::{MonorepoDescriptor, MonorepoKind, WorkspacePackage};
///
/// // Example of creating a monorepo descriptor
/// let root = PathBuf::from("/projects/my-monorepo");
/// let packages = vec![
///     // Package definitions would go here
/// ];
/// let descriptor = MonorepoDescriptor::new(
///     MonorepoKind::YarnWorkspaces,
///     root,
///     packages
/// );
/// ```
#[derive(Debug, Clone)]
pub struct MonorepoDescriptor {
    /// Type of monorepo detected
    pub(crate) kind: MonorepoKind,
    /// Root directory of the monorepo
    pub(crate) root: PathBuf,
    /// Package locations (paths relative to root)
    pub(crate) packages: Vec<WorkspacePackage>,
    /// Map of package names to their locations
    pub(crate) name_to_package: HashMap<String, usize>,
}

/// Represents the type of package manager used in a Node.js project.
///
/// Different package managers use different lock files and commands,
/// and this enum captures those variations to enable manager-specific processing.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::monorepo::types::PackageManagerKind;
///
/// let npm = PackageManagerKind::Npm;
/// assert_eq!(npm.lock_file(), "package-lock.json");
/// assert_eq!(npm.command(), "npm");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManagerKind {
    /// npm package manager (default for Node.js)
    Npm,
    /// Yarn package manager
    Yarn,
    /// pnpm package manager (performance-oriented)
    Pnpm,
    /// Bun package manager and runtime
    Bun,
    /// Jsr package manager and runtime
    Jsr,
}

/// Represents a package manager detected in a Node.js project.
///
/// Contains information about the type of package manager and its
/// location within the filesystem.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use sublime_standard_tools::monorepo::types::{PackageManager, PackageManagerKind};
///
/// // Create a package manager representation
/// let manager = PackageManager::new(PackageManagerKind::Npm, "/project/root");
/// assert_eq!(manager.kind(), PackageManagerKind::Npm);
/// assert_eq!(manager.root(), Path::new("/project/root"));
/// ```
#[derive(Debug, Clone)]
pub struct PackageManager {
    /// The type of package manager
    pub(crate) kind: PackageManagerKind,
    /// The root directory of the project
    pub(crate) root: PathBuf,
}

/// Detects and analyzes monorepo structures within a filesystem.
///
/// This struct provides functionality to scan a directory structure,
/// identify the type of monorepo being used, and gather information about
/// its workspace packages and relationships.
///
/// # Type Parameters
///
/// * `F` - A filesystem implementation that satisfies the `FileSystem` trait.
///   Defaults to `FileSystemManager` for standard operations.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use sublime_standard_tools::monorepo::types::{MonorepoDetector, MonorepoKind};
/// use sublime_standard_tools::filesystem::FileSystemManager;
///
/// // Create a detector with the default filesystem implementation
/// let fs = FileSystemManager::new();
/// let detector = MonorepoDetector::new(fs);
///
/// // Detect a monorepo at a specific path
/// let path = Path::new("/path/to/project");
/// match detector.detect(path) {
///     Ok(descriptor) => {
///         println!("Detected monorepo: {:?}", descriptor.kind());
///         println!("Found {} packages", descriptor.packages().len());
///     },
///     Err(e) => println!("No monorepo detected: {}", e)
/// }
/// ```
///
/// The detector uses heuristics based on configuration files and directory
/// structures to identify different types of monorepos (Yarn, pnpm, npm, etc.)
/// and builds a comprehensive description of the workspace structure.
#[derive(Debug, Clone)]
pub struct MonorepoDetector<F: FileSystem = FileSystemManager> {
    /// File system interface for filesystem operations
    pub(crate) fs: F,
}

/// Configuration structure for PNPM workspaces.
///
/// This struct represents the parsed content of a pnpm-workspace.yaml file,
/// which defines the package locations in a PNPM workspace monorepo.
///
/// # Examples
///
/// ```
/// use serde_yaml;
/// use sublime_standard_tools::monorepo::PnpmWorkspaceConfig;
///
/// let yaml_content = r#"
/// packages:
///   - 'packages/*'
///   - 'apps/*'
///   - '!**/test/**'
/// "#;
///
/// let config: PnpmWorkspaceConfig = serde_yaml::from_str(yaml_content).unwrap();
/// assert_eq!(config.packages.len(), 3);
/// assert_eq!(config.packages[0], "packages/*");
/// assert_eq!(config.packages[1], "apps/*");
/// assert_eq!(config.packages[2], "!**/test/**");
/// ```
///
/// The `packages` field contains glob patterns that define which directories
/// contain packages in the monorepo, including negative patterns (prefixed with `!`)
/// which exclude matching directories.
#[derive(Debug, Clone, Deserialize)]
pub struct PnpmWorkspaceConfig {
    /// Package locations (glob patterns)
    pub(crate) packages: Vec<String>,
}

/// Configuration options for project detection and validation.
///
/// This struct defines options that control how projects are detected,
/// including whether to detect package managers, validate structure,
/// and identify monorepo patterns.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use sublime_standard_tools::monorepo::ProjectConfig;
///
/// // Create a configuration with default settings
/// let config = ProjectConfig::default();
///
/// // Create a custom configuration
/// let custom_config = ProjectConfig::new()
///     .with_root("/path/to/project")
///     .with_detect_package_manager(true)
///     .with_validate_structure(true)
///     .with_detect_monorepo(true);
/// ```
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    /// Root directory to start searching from
    pub(crate) root: Option<PathBuf>,
    /// Whether to automatically detect package manager
    pub(crate) detect_package_manager: bool,
    /// Whether to validate project structure
    pub(crate) validate_structure: bool,
    /// Whether to detect monorepo structure
    pub(crate) detect_monorepo: bool,
}

/// Status of a project validation operation.
///
/// This enum represents the different states a project can be in
/// after validation, including valid, warnings, errors, or not validated.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::monorepo::ProjectValidationStatus;
///
/// // A valid project
/// let valid = ProjectValidationStatus::Valid;
///
/// // A project with warnings
/// let warnings = ProjectValidationStatus::Warning(vec![
///     "Missing lock file".to_string(),
///     "Dependencies not installed".to_string(),
/// ]);
///
/// // A project with errors
/// let errors = ProjectValidationStatus::Error(vec![
///     "Invalid package.json".to_string(),
/// ]);
///
/// // A project that hasn't been validated
/// let not_validated = ProjectValidationStatus::NotValidated;
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

/// Represents a detected Node.js project.
///
/// This struct contains information about a Node.js project,
/// including its root directory, package manager, validation status,
/// and parsed package.json.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::monorepo::{Project, ProjectConfig};
///
/// // Create a new project
/// let config = ProjectConfig::default();
/// let project = Project::new("/path/to/project", config);
///
/// // Access project properties
/// println!("Project root: {}", project.root().display());
/// ```
#[derive(Debug)]
pub struct Project {
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

/// Manages Node.js project detection and validation.
///
/// This struct provides functionality to detect, analyze, and validate
/// Node.js projects within a filesystem.
///
/// # Type Parameters
///
/// * `F` - A filesystem implementation that satisfies the `FileSystem` trait.
///   Defaults to `FileSystemManager` for standard operations.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::monorepo::{ProjectManager, ProjectConfig};
///
/// // Create a manager with the default filesystem implementation
/// let manager = ProjectManager::new();
///
/// // Create a manager with a custom filesystem
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// let fs = FileSystemManager::new();
/// let manager = ProjectManager::with_filesystem(fs);
///
/// // Detect a project
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = ProjectConfig::default();
/// let project = manager.detect_project(".", &config)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ProjectManager<F: FileSystem = FileSystemManager> {
    pub(crate) fs: F,
}
