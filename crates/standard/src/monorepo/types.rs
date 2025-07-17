//! # Monorepo Type Definitions
//!
//! ## What
//! This file defines the core types needed to represent monorepo structures,
//! including the different kinds of monorepos, package information, and
//! the overall monorepo descriptor.
//!
//! ## How
//! Types are defined as enums and structs that model the structure of
//! monorepos and their components. The `MonorepoKind` enum represents different
//! monorepo implementations, `WorkspacePackage` stores information about individual
//! packages, and `MonorepoDescriptor` ties everything together.
//!
//! ## Why
//! A well-defined type system ensures that monorepo structures are represented
//! consistently and safely throughout the codebase, enabling accurate dependency
//! analysis and project structure navigation.

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::filesystem::{FileSystem, FileSystemManager};
use crate::project::GenericProject;

/// Type alias for GenericProject to maintain compatibility
pub type Project = GenericProject;

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
    /// Direct `dev_dependencies` within the workspace
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




/// Configuration scope levels.
///
/// This enum defines the different scopes that configuration can apply to,
/// from the most global (system-wide) to the most specific (runtime only).
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::monorepo::types::ConfigScope;
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
/// use sublime_standard_tools::monorepo::types::ConfigFormat;
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
/// use sublime_standard_tools::monorepo::types::ConfigValue;
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
/// ``
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
/// use sublime_standard_tools::monorepo::types::{ConfigManager, ConfigScope, ConfigValue};
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
