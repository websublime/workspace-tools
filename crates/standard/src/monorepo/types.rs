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
};

use crate::filesystem::{FileSystem, FileSystemManager};
use crate::node::PackageManager;

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
/// use sublime_standard_tools::project::ProjectValidationStatus;
///
/// // Example of creating a monorepo descriptor
/// let root = PathBuf::from("/projects/my-monorepo");
/// let packages = vec![
///     // Package definitions would go here
/// ];
/// let descriptor = MonorepoDescriptor::new(
///     MonorepoKind::YarnWorkspaces,
///     root,
///     packages,
///     None, // package_manager
///     None, // package_json
///     ProjectValidationStatus::NotValidated
/// );
/// ```
#[derive(Debug)]
pub struct MonorepoDescriptor {
    /// Type of monorepo detected
    pub(crate) kind: MonorepoKind,
    /// Root directory of the monorepo
    pub(crate) root: PathBuf,
    /// Package locations (paths relative to root)
    pub(crate) packages: Vec<WorkspacePackage>,
    /// Map of package names to their locations
    pub(crate) name_to_package: HashMap<String, usize>,
    /// Package manager detected for this monorepo
    pub(crate) package_manager: Option<PackageManager>,
    /// Root package.json content (if available)
    pub(crate) package_json: Option<package_json::PackageJson>,
    /// Validation status of the monorepo
    pub(crate) validation_status: crate::project::ProjectValidationStatus,
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
