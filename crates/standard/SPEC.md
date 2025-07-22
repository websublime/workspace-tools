# Sublime Standard Tools API Specification

This document provides a comprehensive specification of the public API for the `sublime_standard_tools` crate, a robust toolkit for working with Node.js projects from Rust applications.

## Table of Contents

- [Overview](#overview)
- [Config Module](#config-module)
  - [Configuration Management](#configuration-management)
  - [Standard Configuration](#standard-configuration)
  - [Configuration Sources and Formats](#configuration-sources-and-formats)
  - [Environment Variable Overrides](#environment-variable-overrides)
- [Node Module](#node-module)
  - [Repository Types](#repository-types)
  - [Package Manager Abstractions](#package-manager-abstractions)
  - [Repository Information](#repository-information)
- [Project Module](#project-module)
  - [Project Types](#project-types)
  - [Project Detection](#project-detection)
  - [Project Management](#project-management)
- [Monorepo Module](#monorepo-module)
  - [Monorepo Types](#monorepo-types)
  - [Monorepo Detection](#monorepo-detection)
  - [Workspace Management](#workspace-management)
- [Command Module](#command-module)
  - [Command Execution](#command-execution)
  - [Command Builder](#command-builder)
  - [Command Output](#command-output)
  - [Command Queue](#command-queue)
  - [Command Stream](#command-stream)
- [Filesystem Module](#filesystem-module)
  - [Async Filesystem Abstraction](#async-filesystem-abstraction)
  - [Path Utilities](#path-utilities)
  - [Node.js Path Extensions](#nodejs-path-extensions)
- [Error Module](#error-module)
  - [Error Types](#error-types)
  - [Result Types](#result-types)
  - [Error Recovery](#error-recovery)

## Overview

The `sublime_standard_tools` crate provides a comprehensive set of utilities for working with Node.js projects from Rust applications. It follows a clean architectural approach with clear separation of concerns:

- **Config Module**: Flexible configuration management with multiple sources and formats
- **Node Module**: Generic Node.js concepts (repositories, package managers)
- **Project Module**: Unified project detection and management
- **Monorepo Module**: Monorepo-specific functionality and workspace management
- **Command Module**: Robust command execution framework
- **Filesystem Module**: Async filesystem operations and path utilities
- **Error Module**: Comprehensive error handling with recovery strategies

```rust
// Version information
const VERSION: &str = "..."; // Returns the current crate version

// Get version
fn version() -> &'static str;
```

## Config Module

The config module provides a comprehensive configuration framework that supports multiple sources (files, environment variables, defaults), various formats (TOML, JSON, YAML), and hierarchical configuration with proper merging semantics.

### Configuration Management

#### ConfigManager

```rust
/// Generic configuration manager for any Configurable type.
#[derive(Debug)]
pub struct ConfigManager<T: Configurable> {
    // Private fields
}

impl<T: Configurable> ConfigManager<T> {
    /// Creates a new ConfigBuilder for building a ConfigManager.
    pub fn builder() -> ConfigBuilder<T>;
    
    /// Loads configuration from all registered sources.
    pub async fn load(&self) -> ConfigResult<T>;
    
    /// Saves current configuration to writable sources.
    pub async fn save(&self, config: &T) -> ConfigResult<()>;
    
    /// Reloads configuration from all sources.
    pub async fn reload(&mut self) -> ConfigResult<T>;
}
```

#### ConfigBuilder

```rust
/// Builder for creating ConfigManager instances with various sources.
#[derive(Debug)]
pub struct ConfigBuilder<T: Configurable> {
    // Private fields
}

impl<T: Configurable> ConfigBuilder<T> {
    /// Creates a new ConfigBuilder.
    pub fn new() -> Self;
    
    /// Adds default values as a configuration source.
    pub fn with_defaults(self) -> Self;
    
    /// Adds a configuration file as a source.
    pub fn with_file(self, path: impl Into<PathBuf>) -> Self;
    
    /// Adds environment variables with a prefix as a source.
    pub fn with_env_prefix(self, prefix: impl Into<String>) -> Self;
    
    /// Adds a custom configuration source.
    pub fn with_source(self, source: ConfigSource) -> Self;
    
    /// Builds the ConfigManager with the specified filesystem.
    pub fn build<F: AsyncFileSystem + Clone + 'static>(
        self, 
        filesystem: F
    ) -> ConfigResult<ConfigManager<T>>;
}
```

#### Configurable Trait

```rust
/// Trait for types that can be used with the configuration system.
pub trait Configurable: Clone + Default + Serialize + DeserializeOwned + Send + Sync {
    /// Validates the configuration.
    fn validate(&self) -> ConfigResult<()>;
    
    /// Merges this configuration with another.
    fn merge_with(&mut self, other: Self) -> ConfigResult<()>;
}
```

### Standard Configuration

#### StandardConfig

```rust
/// The standard configuration for sublime-standard-tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardConfig {
    /// Configuration version for migration support
    pub version: String,
    
    /// Package manager configuration
    pub package_managers: PackageManagerConfig,
    
    /// Monorepo detection configuration
    pub monorepo: MonorepoConfig,
    
    /// Command execution configuration
    pub commands: CommandConfig,
    
    /// Filesystem configuration
    pub filesystem: FilesystemConfig,
    
    /// Validation configuration
    pub validation: ValidationConfig,
}

impl Configurable for StandardConfig {
    fn validate(&self) -> ConfigResult<()>;
    fn merge_with(&mut self, other: Self) -> ConfigResult<()>;
}
```

#### PackageManagerConfig

```rust
/// Package manager detection and behavior configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManagerConfig {
    /// Detection order for package managers
    pub detection_order: Vec<PackageManagerKind>,
    
    /// Custom lock file names for each package manager
    pub custom_lock_files: HashMap<PackageManagerKind, String>,
    
    /// Whether to detect from environment variables
    pub detect_from_env: bool,
    
    /// Environment variable name for preferred package manager
    pub env_var_name: String,
    
    /// Custom binary paths for package managers
    pub binary_paths: HashMap<PackageManagerKind, PathBuf>,
    
    /// Fallback package manager if none detected
    pub fallback: Option<PackageManagerKind>,
}
```

#### MonorepoConfig

```rust
/// Monorepo detection and workspace configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonorepoConfig {
    /// Custom workspace directory patterns
    pub workspace_patterns: Vec<String>,
    
    /// Additional directories to check for packages
    pub package_directories: Vec<String>,
    
    /// Patterns to exclude from package detection
    pub exclude_patterns: Vec<String>,
    
    /// Maximum depth for recursive package search
    pub max_search_depth: usize,
    
    /// Whether to follow symlinks during search
    pub follow_symlinks: bool,
    
    /// Custom patterns for workspace detection in package.json
    pub custom_workspace_fields: Vec<String>,
}
```

#### CommandConfig

```rust
/// Command execution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    /// Default timeout for command execution
    #[serde(with = "humantime_serde")]
    pub default_timeout: Duration,
    
    /// Timeout overrides for specific commands
    pub timeout_overrides: HashMap<String, Duration>,
    
    /// Buffer size for command output streaming
    pub stream_buffer_size: usize,
    
    /// Read timeout for streaming output
    #[serde(with = "humantime_serde")]
    pub stream_read_timeout: Duration,
    
    /// Maximum concurrent commands in queue
    pub max_concurrent_commands: usize,
    
    /// Environment variables to set for all commands
    pub env_vars: HashMap<String, String>,
    
    /// Whether to inherit parent process environment
    pub inherit_env: bool,
    
    /// Queue collection window duration in milliseconds
    pub queue_collection_window_ms: u64,
    
    /// Queue collection sleep duration in microseconds
    pub queue_collection_sleep_us: u64,
    
    /// Queue idle sleep duration in milliseconds
    pub queue_idle_sleep_ms: u64,
}
```

#### FilesystemConfig

```rust
/// Filesystem operation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemConfig {
    /// Path conventions overrides
    pub path_conventions: HashMap<String, PathBuf>,
    
    /// Async I/O configuration
    pub async_io: AsyncIoConfig,
    
    /// File operation retry configuration
    pub retry: RetryConfig,
    
    /// Patterns to ignore during directory traversal
    pub ignore_patterns: Vec<String>,
}
```

#### ValidationConfig

```rust
/// Project validation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Whether to require package.json at project root
    pub require_package_json: bool,
    
    /// Required fields in package.json
    pub required_package_fields: Vec<String>,
    
    /// Whether to validate dependency versions
    pub validate_dependencies: bool,
    
    /// Custom validation rules
    pub custom_rules: HashMap<String, serde_json::Value>,
    
    /// Whether to fail on validation warnings
    pub strict_mode: bool,
}
```

### Environment Variable Overrides

The configuration system supports extensive environment variable overrides with validation and reasonable bounds:

#### Package Manager Configuration
- `SUBLIME_PACKAGE_MANAGER_ORDER`: Comma-separated list of package managers (npm,yarn,pnpm,bun,jsr)
- `SUBLIME_PACKAGE_MANAGER`: Preferred package manager name

#### Monorepo Configuration  
- `SUBLIME_WORKSPACE_PATTERNS`: Comma-separated workspace patterns (e.g., "packages/*,apps/*")
- `SUBLIME_PACKAGE_DIRECTORIES`: Comma-separated package directory names
- `SUBLIME_EXCLUDE_PATTERNS`: Comma-separated exclude patterns for monorepo detection
- `SUBLIME_MAX_SEARCH_DEPTH`: Maximum search depth (1-20)

#### Command Configuration
- `SUBLIME_COMMAND_TIMEOUT`: Command execution timeout in seconds (1-3600)
- `SUBLIME_MAX_CONCURRENT`: Maximum concurrent commands (1-100)
- `SUBLIME_BUFFER_SIZE`: Command output buffer size in bytes (256-65536)

#### Examples

```rust
use sublime_standard_tools::config::{ConfigManager, StandardConfig, ConfigBuilder};
use std::path::Path;

// Load configuration with auto-detection
async fn load_config() -> Result<StandardConfig, Box<dyn std::error::Error>> {
    let manager = ConfigManager::<StandardConfig>::builder()
        .with_defaults()
        .with_file("~/.config/sublime/config.toml")
        .with_file(".sublime.toml")
        .with_env_prefix("SUBLIME")
        .build(FileSystemManager::new())?;

    let config = manager.load().await?;
    Ok(config)
}

// Use configuration in components
let config = load_config().await?;
let package_manager = PackageManager::detect_with_config(Path::new("."), &config.package_managers)?;
```

## Node Module

The node module provides generic Node.js repository concepts that are used across all project types. It establishes fundamental abstractions for repository identification and package management.

### Repository Types

#### RepoKind

```rust
/// Represents the type of Node.js repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoKind {
    /// Simple Node.js repository with a single package.json
    Simple,
    /// Monorepo repository with specific monorepo type
    Monorepo(MonorepoKind),
}

impl RepoKind {
    /// Returns a human-readable name for the repository kind.
    pub fn name(&self) -> String;
    
    /// Checks if this repository is a monorepo.
    pub fn is_monorepo(&self) -> bool;
    
    /// Gets the monorepo kind if this is a monorepo repository.
    pub fn monorepo_kind(&self) -> Option<&MonorepoKind>;
    
    /// Checks if this repository matches a specific monorepo kind.
    pub fn is_monorepo_kind(&self, kind: &MonorepoKind) -> bool;
}
```

### Package Manager Abstractions

#### PackageManagerKind

```rust
/// Represents the type of package manager used in a Node.js project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
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

impl PackageManagerKind {
    /// Returns the command name for this package manager.
    pub fn command(self) -> &'static str;
    
    /// Returns the lock file name for this package manager.
    pub fn lock_file(self) -> &'static str;
    
    /// Returns a human-readable name for this package manager.
    pub fn name(self) -> &'static str;
    
    /// Checks if this package manager supports workspaces natively.
    pub fn supports_workspaces(self) -> bool;
    
    /// Returns the workspace configuration file for this package manager.
    pub fn workspace_config_file(self) -> Option<&'static str>;
}
```

#### PackageManager

```rust
/// Represents a package manager detected in a Node.js project.
#[derive(Debug, Clone)]
pub struct PackageManager {
    // Private fields
}

impl PackageManager {
    /// Creates a new PackageManager instance.
    pub fn new(kind: PackageManagerKind, root: impl Into<PathBuf>) -> Self;
    
    /// Detects which package manager is being used using default configuration.
    pub fn detect(path: impl AsRef<Path>) -> Result<Self>;
    
    /// Detects which package manager is being used with custom configuration.
    pub fn detect_with_config(path: impl AsRef<Path>, config: &PackageManagerConfig) -> Result<Self>;
    
    /// Returns the kind of package manager.
    pub fn kind(&self) -> PackageManagerKind;
    
    /// Returns the root directory path of the package manager.
    pub fn root(&self) -> &Path;
    
    /// Returns the command name for this package manager.
    pub fn command(&self) -> &'static str;
    
    /// Returns the lock file name for this package manager.
    pub fn lock_file(&self) -> &'static str;
    
    /// Returns the full path to the lock file for this package manager.
    pub fn lock_file_path(&self) -> PathBuf;
    
    /// Checks if this package manager supports workspaces.
    pub fn supports_workspaces(&self) -> bool;
    
    /// Returns the workspace configuration file path if applicable.
    pub fn workspace_config_path(&self) -> Option<PathBuf>;
}
```

### Repository Information

#### RepositoryInfo

```rust
/// Provides information about repository characteristics.
pub trait RepositoryInfo {
    /// Returns the repository kind.
    fn repo_kind(&self) -> &RepoKind;
    
    /// Returns the repository name.
    fn name(&self) -> String;
    
    /// Checks if this is a monorepo.
    fn is_monorepo(&self) -> bool;
}
```

#### Examples

```rust
use sublime_standard_tools::node::{PackageManager, PackageManagerKind, RepoKind};
use sublime_standard_tools::config::PackageManagerConfig;
use std::path::Path;

// Package manager detection with configuration
let config = PackageManagerConfig {
    detection_order: vec![PackageManagerKind::Pnpm, PackageManagerKind::Yarn],
    detect_from_env: true,
    ..Default::default()
};

let manager = PackageManager::detect_with_config(Path::new("."), &config)?;
println!("Using package manager: {}", manager.command());
println!("Supports workspaces: {}", manager.supports_workspaces());

// Repository types
let simple_repo = RepoKind::Simple;
assert!(!simple_repo.is_monorepo());

let yarn_mono = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);
assert!(yarn_mono.is_monorepo());
```

## Project Module

The project module provides a unified API for detecting, managing, and working with Node.js projects, regardless of whether they are simple repositories or monorepos.

### Project Types

#### ProjectKind

```rust
/// Represents the type of Node.js project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectKind {
    /// A repository-based project (simple or monorepo)
    Repository(RepoKind),
}

impl ProjectKind {
    /// Returns a human-readable name for the project kind.
    pub fn name(&self) -> String;
    
    /// Checks if this is a monorepo project.
    pub fn is_monorepo(&self) -> bool;
    
    /// Returns the repository kind for this project.
    pub fn repo_kind(&self) -> &RepoKind;
    
    /// Gets the monorepo kind if this is a monorepo project.
    pub fn monorepo_kind(&self) -> Option<&MonorepoKind>;
}
```

#### ProjectInfo

```rust
/// Common interface for all Node.js project types.
pub trait ProjectInfo: Send + Sync {
    /// Returns the root directory of the project.
    fn root(&self) -> &Path;
    
    /// Returns the package manager for the project, if detected.
    fn package_manager(&self) -> Option<&PackageManager>;
    
    /// Returns the parsed package.json for the project, if available.
    fn package_json(&self) -> Option<&PackageJson>;
    
    /// Returns the validation status of the project.
    fn validation_status(&self) -> &ProjectValidationStatus;
    
    /// Returns the kind of project.
    fn kind(&self) -> ProjectKind;
}
```

#### Project

```rust
/// Represents a unified Node.js project structure.
#[derive(Debug)]
pub struct Project {
    // Private fields
}

impl Project {
    /// Creates a new Project instance.
    pub fn new(root: PathBuf, kind: ProjectKind) -> Self;
    
    /// Checks if this project is a monorepo.
    pub fn is_monorepo(&self) -> bool;
    
    /// Gets the project dependencies.
    pub fn dependencies(&self) -> &Dependencies;
    
    /// Returns internal workspace dependencies for monorepos.
    pub fn internal_dependencies(&self) -> &[WorkspacePackage];
}

impl ProjectInfo for Project {
    fn root(&self) -> &Path;
    fn package_manager(&self) -> Option<&PackageManager>;
    fn package_json(&self) -> Option<&PackageJson>;
    fn validation_status(&self) -> &ProjectValidationStatus;
    fn kind(&self) -> ProjectKind;
}
```

#### Dependencies

```rust
/// Represents project dependencies information.
#[derive(Debug, Clone)]
pub struct Dependencies {
    /// Production dependencies
    pub dependencies: HashMap<String, String>,
    /// Development dependencies
    pub dev_dependencies: HashMap<String, String>,
    /// Optional dependencies
    pub optional_dependencies: HashMap<String, String>,
    /// Peer dependencies
    pub peer_dependencies: HashMap<String, String>,
}
```

#### ProjectDescriptor

```rust
/// Represents different types of Node.js projects with their specific data.
#[derive(Debug)]
pub enum ProjectDescriptor {
    /// A Node.js project (simple or monorepo)
    NodeJs(Project),
}

impl ProjectDescriptor {
    /// Returns a reference to the project as a trait object.
    pub fn as_project_info(&self) -> &dyn ProjectInfo;
}
```

#### ProjectValidationStatus

```rust
/// Status of a project validation operation.
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
    pub fn is_valid(&self) -> bool;
    
    /// Checks if the project has warnings.
    pub fn has_warnings(&self) -> bool;
    
    /// Checks if the project has errors.
    pub fn has_errors(&self) -> bool;
    
    /// Gets the list of warnings if any.
    pub fn warnings(&self) -> Option<&[String]>;
    
    /// Gets the list of errors if any.
    pub fn errors(&self) -> Option<&[String]>;
}
```

### Project Detection

#### ProjectDetectorTrait

```rust
/// Async trait for project detection.
#[async_trait]
pub trait ProjectDetectorTrait: Send + Sync {
    /// Asynchronously detects and analyzes a project at the given path.
    async fn detect(&self, path: &Path, config: Option<&StandardConfig>) -> Result<ProjectDescriptor>;
    
    /// Asynchronously detects only the project kind without full analysis.
    async fn detect_kind(&self, path: &Path) -> Result<ProjectKind>;
    
    /// Asynchronously checks if the path contains a valid Node.js project.
    async fn is_valid_project(&self, path: &Path) -> bool;
}
```

#### ProjectDetectorWithFs

```rust
/// Async trait for project detection with custom filesystem.
#[async_trait]
pub trait ProjectDetectorWithFs<F: AsyncFileSystem>: ProjectDetectorTrait {
    /// Gets a reference to the filesystem implementation.
    fn filesystem(&self) -> &F;
    
    /// Asynchronously detects projects in multiple paths concurrently.
    async fn detect_multiple(
        &self,
        paths: &[&Path],
        config: Option<&StandardConfig>,
    ) -> Vec<Result<ProjectDescriptor>>;
}
```

#### ProjectDetector

```rust
/// Provides unified detection and analysis of Node.js projects.
#[derive(Debug)]
pub struct ProjectDetector<F: AsyncFileSystem = FileSystemManager> {
    // Private fields
}

impl ProjectDetector<FileSystemManager> {
    /// Creates a new ProjectDetector with the default async filesystem.
    pub fn new() -> Self;
}

impl<F: AsyncFileSystem + Clone + 'static> ProjectDetector<F> {
    /// Creates a new ProjectDetector with a custom async filesystem.
    pub fn with_filesystem(fs: F) -> Self;
    
    /// Detects and analyzes a project using configuration-controlled detection.
    pub async fn detect(
        &self,
        path: impl AsRef<Path>,
        config: Option<&StandardConfig>,
    ) -> Result<ProjectDescriptor>;
    
    /// Detects only the project kind using default configuration.
    pub async fn detect_kind(&self, path: impl AsRef<Path>) -> Result<ProjectKind>;
    
    /// Detects the project kind using custom configuration.
    pub async fn detect_kind_with_config(
        &self,
        path: impl AsRef<Path>,
        config: &StandardConfig,
    ) -> Result<ProjectKind>;
    
    /// Checks if the path contains a valid Node.js project.
    pub async fn is_valid_project(&self, path: impl AsRef<Path>) -> bool;
}
```

### Project Management

#### ProjectManager

```rust
/// Manages Node.js project lifecycle and operations.
pub struct ProjectManager<F: AsyncFileSystem = FileSystemManager> {
    // Private fields
}

impl ProjectManager<FileSystemManager> {
    /// Creates a new ProjectManager with the default filesystem.
    pub fn new() -> Self;
}

impl<F: AsyncFileSystem + Clone> ProjectManager<F> {
    /// Creates a new ProjectManager with a custom filesystem.
    pub fn with_filesystem(fs: F) -> Self;
    
    /// Creates a project descriptor from a path with configuration.
    pub async fn create_project(
        &self, 
        path: impl AsRef<Path>, 
        config: Option<&StandardConfig>
    ) -> Result<ProjectDescriptor>;
}
```

#### Examples

```rust
use sublime_standard_tools::project::{
    ProjectDetector, ProjectDetectorTrait, ProjectDescriptor, ProjectInfo
};
use sublime_standard_tools::config::StandardConfig;
use std::path::Path;

// Auto-loading project detection with configuration
let detector = ProjectDetector::new();
let project = detector.detect(Path::new("."), None).await?; // Auto-loads config

match project {
    ProjectDescriptor::NodeJs(nodejs_project) => {
        println!("Found {} project", nodejs_project.kind().name());
        if let Some(pm) = nodejs_project.package_manager() {
            println!("Using package manager: {}", pm.command());
        }
        
        if nodejs_project.is_monorepo() {
            println!("Packages: {}", nodejs_project.internal_dependencies().len());
        }
    }
}

// Multiple project detection
let fs = FileSystemManager::new();
let detector = ProjectDetector::with_filesystem(fs);
let paths = vec![Path::new("."), Path::new("../other-project")];
let results = detector.detect_multiple(&paths, None).await;
```

## Monorepo Module

The monorepo module provides specialized functionality for detecting and managing monorepo structures across different package managers.

### Monorepo Types

#### MonorepoKind

```rust
/// Represents the type of monorepo system being used.
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

impl MonorepoKind {
    /// Returns the name of the monorepo kind as a string.
    pub fn name(&self) -> String;
    
    /// Returns the primary configuration file for this monorepo kind.
    pub fn config_file(self) -> String;
    
    /// Creates a custom monorepo kind with the specified name and config file.
    pub fn set_custom(&self, name: String, config_file: String) -> Self;
}
```

#### WorkspacePackage

```rust
/// Represents a single package within a monorepo workspace.
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
```

#### MonorepoDescriptor

```rust
/// Describes a complete monorepo structure.
#[derive(Debug)]
pub struct MonorepoDescriptor {
    // Private fields
}

impl MonorepoDescriptor {
    /// Creates a new MonorepoDescriptor instance.
    pub fn new(
        kind: MonorepoKind,
        root: PathBuf,
        packages: Vec<WorkspacePackage>,
        package_manager: Option<PackageManager>,
        package_json: Option<PackageJson>,
        validation_status: ProjectValidationStatus,
    ) -> Self;
    
    /// Returns the kind of monorepo.
    pub fn kind(&self) -> &MonorepoKind;
    
    /// Returns the root directory of the monorepo.
    pub fn root(&self) -> &Path;
    
    /// Returns all packages in the monorepo.
    pub fn packages(&self) -> &[WorkspacePackage];
    
    /// Gets a package by name.
    pub fn get_package(&self, name: &str) -> Option<&WorkspacePackage>;
    
    /// Generates a dependency graph for the monorepo.
    pub fn get_dependency_graph(&self) -> HashMap<&str, Vec<&WorkspacePackage>>;
    
    /// Finds all workspace dependencies of a given package.
    pub fn find_dependencies_by_name(&self, package_name: &str) -> Vec<&WorkspacePackage>;
    
    /// Finds the package that contains a specific path.
    pub fn find_package_for_path(&self, path: &Path) -> Option<&WorkspacePackage>;
}

impl ProjectInfo for MonorepoDescriptor {
    fn root(&self) -> &Path;
    fn package_manager(&self) -> Option<&PackageManager>;
    fn package_json(&self) -> Option<&PackageJson>;
    fn validation_status(&self) -> &ProjectValidationStatus;
    fn kind(&self) -> ProjectKind;
}
```

### Monorepo Detection

#### MonorepoDetectorTrait

```rust
/// Async trait for monorepo detection.
#[async_trait]
pub trait MonorepoDetectorTrait: Send + Sync {
    /// Checks if a path is the root of a monorepo.
    async fn is_monorepo_root(&self, path: &Path) -> Result<Option<MonorepoKind>>;
    
    /// Finds the nearest monorepo root by walking up from the given path.
    async fn find_monorepo_root(
        &self,
        start_path: &Path,
    ) -> Result<Option<(PathBuf, MonorepoKind)>>;
    
    /// Detects and analyzes a monorepo at the given path.
    async fn detect_monorepo(&self, path: &Path) -> Result<MonorepoDescriptor>;
    
    /// Checks if a directory contains multiple packages.
    async fn has_multiple_packages(&self, path: &Path) -> bool;
}
```

#### MonorepoDetectorWithFs

```rust
/// Async trait for monorepo detection with custom filesystem.
#[async_trait]
pub trait MonorepoDetectorWithFs<F: AsyncFileSystem>: MonorepoDetectorTrait {
    /// Gets a reference to the filesystem implementation.
    fn filesystem(&self) -> &F;
    
    /// Detects monorepos in multiple paths concurrently.
    async fn detect_multiple(&self, paths: &[&Path]) -> Vec<Result<MonorepoDescriptor>>;
}
```

#### MonorepoDetector

```rust
/// Detects and analyzes monorepo structures.
pub struct MonorepoDetector<F: AsyncFileSystem = FileSystemManager> {
    // Private fields
}

impl MonorepoDetector<FileSystemManager> {
    /// Creates a new MonorepoDetector with the default filesystem.
    pub fn new() -> Self;
}

impl<F: AsyncFileSystem + Clone> MonorepoDetector<F> {
    /// Creates a new MonorepoDetector with a custom filesystem.
    pub fn with_filesystem(fs: F) -> Self;
    
    /// Creates a new MonorepoDetector with filesystem and configuration.
    pub fn with_filesystem_and_config(fs: F, config: MonorepoConfig) -> Self;
    
    /// Checks if a path is the root of a monorepo.
    pub async fn is_monorepo_root(&self, path: impl AsRef<Path>) -> Result<Option<MonorepoKind>>;
    
    /// Finds the nearest monorepo root by walking up from the given path.
    pub async fn find_monorepo_root(
        &self,
        start_path: impl AsRef<Path>,
    ) -> Result<Option<(PathBuf, MonorepoKind)>>;
    
    /// Detects and analyzes a monorepo at the given path.
    pub async fn detect_monorepo(&self, path: impl AsRef<Path>) -> Result<MonorepoDescriptor>;
    
    /// Checks if a directory contains multiple packages.
    pub async fn has_multiple_packages(&self, path: &Path) -> bool;
}
```

### Workspace Management

#### PnpmWorkspaceConfig

```rust
/// Configuration structure for PNPM workspaces.
#[derive(Debug, Clone, Deserialize)]
pub struct PnpmWorkspaceConfig {
    /// Package locations (glob patterns)
    pub packages: Vec<String>,
}
```

#### Examples

```rust
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait, MonorepoKind};
use sublime_standard_tools::config::MonorepoConfig;
use std::path::Path;

// Configuration-aware monorepo detection
let config = MonorepoConfig {
    workspace_patterns: vec!["packages/*".to_string(), "apps/*".to_string()],
    max_search_depth: 3,
    exclude_patterns: vec!["node_modules".to_string()],
    ..Default::default()
};

let detector = MonorepoDetector::with_filesystem_and_config(
    FileSystemManager::new(),
    config
);

if let Some(kind) = detector.is_monorepo_root(".").await? {
    println!("This directory is a {} monorepo", kind.name());
    
    // Analyze the monorepo
    let monorepo = detector.detect_monorepo(".").await?;
    
    println!("Monorepo contains {} packages:", monorepo.packages().len());
    for package in monorepo.packages() {
        println!("- {} v{} at {}", 
                 package.name, 
                 package.version, 
                 package.location.display());
        
        // Print dependencies
        let deps = monorepo.find_dependencies_by_name(&package.name);
        if !deps.is_empty() {
            println!("  Dependencies:");
            for dep in deps {
                println!("    - {}", dep.name);
            }
        }
    }
}

// Find monorepo root
if let Some((root, kind)) = detector.find_monorepo_root(".").await? {
    println!("Found {} monorepo at {}", kind.name(), root.display());
}
```

## Command Module

The command module provides a comprehensive framework for executing external commands with proper error handling, streaming, and queue management.

### Command Execution

#### Command

```rust
/// Represents a command to be executed.
pub struct Command {
    // Private fields
}

impl Command {
    /// Creates a new Command with the specified program.
    pub fn new(program: impl Into<String>) -> Self;
    
    /// Returns the program name.
    pub fn program(&self) -> &str;
    
    /// Returns the command arguments.
    pub fn args(&self) -> &[String];
    
    /// Returns the environment variables.
    pub fn env(&self) -> &HashMap<String, String>;
    
    /// Returns the current directory.
    pub fn current_dir(&self) -> Option<&Path>;
    
    /// Returns the timeout.
    pub fn timeout(&self) -> Option<Duration>;
}
```

#### Executor

```rust
/// Trait for executing commands.
#[async_trait]
pub trait Executor: Send + Sync {
    /// Executes a command and returns the output.
    async fn execute(&self, command: Command) -> Result<CommandOutput>;
    
    /// Executes a command with streaming output.
    async fn execute_stream(
        &self,
        command: Command,
        stream_config: StreamConfig,
    ) -> Result<(CommandStream, tokio::process::Child)>;
}
```

#### DefaultCommandExecutor

```rust
/// Default async command executor implementation.
pub struct DefaultCommandExecutor {
    // Private fields
}

impl DefaultCommandExecutor {
    /// Creates a new DefaultCommandExecutor.
    pub fn new() -> Self;
    
    /// Creates a new DefaultCommandExecutor with project configuration.
    pub async fn new_with_project_config(project_root: &Path) -> Result<Self>;
}

impl Executor for DefaultCommandExecutor {
    // Implementation of async execute methods
}
```

#### SyncCommandExecutor

```rust
/// Synchronous command executor for blocking operations.
pub struct SyncCommandExecutor {
    // Private fields
}

impl SyncCommandExecutor {
    /// Creates a new SyncCommandExecutor.
    pub fn new() -> Self;
    
    /// Executes a command synchronously.
    pub fn execute_sync(&self, command: Command) -> Result<CommandOutput>;
}
```

#### SharedSyncExecutor

```rust
/// Thread-safe shared synchronous executor.
pub type SharedSyncExecutor = Arc<Mutex<SyncCommandExecutor>>;
```

### Command Builder

#### CommandBuilder

```rust
/// Builder for creating Command instances.
pub struct CommandBuilder {
    // Private fields
}

impl CommandBuilder {
    /// Creates a new CommandBuilder instance.
    pub fn new(program: impl Into<String>) -> Self;
    
    /// Builds the final Command instance.
    pub fn build(self) -> Command;
    
    /// Adds an argument to the command.
    pub fn arg(mut self, arg: impl Into<String>) -> Self;
    
    /// Adds multiple arguments to the command.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>;
    
    /// Sets an environment variable.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self;
    
    /// Sets the current directory.
    pub fn current_dir(mut self, path: impl AsRef<Path>) -> Self;
    
    /// Sets the timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self;
}
```

### Command Output

#### CommandOutput

```rust
/// Represents the output of a command execution.
pub struct CommandOutput {
    // Private fields
}

impl CommandOutput {
    /// Creates a new CommandOutput instance.
    pub fn new(status: i32, stdout: String, stderr: String, duration: Duration) -> Self;
    
    /// Returns the exit status code.
    pub fn status(&self) -> i32;
    
    /// Returns the standard output content.
    pub fn stdout(&self) -> &str;
    
    /// Returns the standard error content.
    pub fn stderr(&self) -> &str;
    
    /// Returns the command execution duration.
    pub fn duration(&self) -> Duration;
    
    /// Returns true if the command was successful (exit code 0).
    pub fn success(&self) -> bool;
}
```

### Command Stream

#### CommandStream

```rust
/// Represents a stream of command output.
pub struct CommandStream {
    // Private fields
}

impl CommandStream {
    /// Receives the next output line with timeout.
    pub async fn next_timeout(
        &mut self,
        timeout_duration: Duration,
    ) -> Result<Option<StreamOutput>>;
    
    /// Cancels the stream.
    pub fn cancel(&self);
}
```

#### StreamOutput

```rust
/// Represents output from a command stream.
pub enum StreamOutput {
    /// Standard output line
    Stdout(String),
    /// Standard error line
    Stderr(String),
    /// End of stream
    End,
}
```

#### StreamConfig

```rust
/// Configuration for command streaming.
pub struct StreamConfig {
    // Private fields
}

impl StreamConfig {
    /// Creates a new StreamConfig.
    pub fn new(buffer_size: usize, read_timeout: Duration) -> Self;
}

impl Default for StreamConfig {
    fn default() -> Self;
}
```

### Command Queue

#### CommandQueue

```rust
/// Manages queued command execution with priority.
pub struct CommandQueue {
    // Private fields
}

impl CommandQueue {
    /// Creates a new CommandQueue.
    pub fn new() -> Self;
    
    /// Creates a new CommandQueue with custom configuration.
    pub fn with_config(config: CommandQueueConfig) -> Self;
    
    /// Creates a new CommandQueue with a custom executor.
    pub fn with_executor<E: Executor + 'static>(executor: E) -> Self;
    
    /// Starts the command queue.
    pub fn start(mut self) -> Result<Self>;
    
    /// Enqueues a command with priority.
    pub async fn enqueue(&self, command: Command, priority: CommandPriority) -> Result<String>;
    
    /// Enqueues multiple commands.
    pub async fn enqueue_batch(&self, commands: Vec<(Command, CommandPriority)>) -> Result<Vec<String>>;
    
    /// Gets the status of a command.
    pub fn get_status(&self, id: &str) -> Option<CommandStatus>;
    
    /// Gets the result of a command.
    pub fn get_result(&self, id: &str) -> Option<CommandQueueResult>;
    
    /// Waits for a command to complete.
    pub async fn wait_for_command(&self, id: &str, timeout: Duration) -> Result<CommandQueueResult>;
    
    /// Waits for all commands to complete.
    pub async fn wait_for_completion(&self) -> Result<()>;
    
    /// Shuts down the queue.
    pub async fn shutdown(&mut self) -> Result<()>;
}
```

#### CommandPriority

```rust
/// Priority levels for commands.
pub enum CommandPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}
```

#### CommandStatus

```rust
/// Status of a command in the queue.
pub enum CommandStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}
```

#### Examples

```rust
use sublime_standard_tools::command::{
    CommandBuilder, DefaultCommandExecutor, Executor, 
    CommandQueue, CommandPriority, StreamConfig
};
use std::time::Duration;

// Configuration-aware command execution
let executor = DefaultCommandExecutor::new_with_project_config(Path::new(".")).await?;

let cmd = CommandBuilder::new("npm")
    .arg("install")
    .env("NODE_ENV", "production")
    .timeout(Duration::from_secs(60))
    .build();

let output = executor.execute(cmd).await?;
if output.success() {
    println!("Command output: {}", output.stdout());
} else {
    println!("Command failed: {}", output.stderr());
}

// Stream command output
let stream_config = StreamConfig::default();
let cmd = CommandBuilder::new("npm").arg("run").arg("build").build();

let (mut stream, mut child) = executor.execute_stream(cmd, stream_config).await?;
while let Ok(Some(output)) = stream.next_timeout(Duration::from_secs(1)).await {
    match output {
        StreamOutput::Stdout(line) => println!("STDOUT: {}", line),
        StreamOutput::Stderr(line) => eprintln!("STDERR: {}", line),
        StreamOutput::End => break,
    }
}

// Command queue with priority
let mut queue = CommandQueue::new().start()?;

let high_priority_cmd = CommandBuilder::new("npm").arg("test").build();
let normal_priority_cmd = CommandBuilder::new("npm").arg("lint").build();

let id1 = queue.enqueue(high_priority_cmd, CommandPriority::High).await?;
let id2 = queue.enqueue(normal_priority_cmd, CommandPriority::Normal).await?;

let result = queue.wait_for_command(&id1, Duration::from_secs(30)).await?;
println!("High priority command result: {:?}", result);
```

## Filesystem Module

The filesystem module provides async abstractions for interacting with the filesystem and Node.js-specific path utilities.

### Async Filesystem Abstraction

#### AsyncFileSystem

```rust
/// Trait for async filesystem operations.
#[async_trait]
pub trait AsyncFileSystem: Send + Sync {
    /// Reads a file and returns its contents as bytes.
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>>;
    
    /// Writes bytes to a file.
    async fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()>;
    
    /// Reads a file and returns its contents as a string.
    async fn read_file_string(&self, path: &Path) -> Result<String>;
    
    /// Writes a string to a file.
    async fn write_file_string(&self, path: &Path, contents: &str) -> Result<()>;
    
    /// Creates a directory and all parent directories.
    async fn create_dir_all(&self, path: &Path) -> Result<()>;
    
    /// Removes a file or directory.
    async fn remove(&self, path: &Path) -> Result<()>;
    
    /// Checks if a path exists.
    async fn exists(&self, path: &Path) -> bool;
    
    /// Reads a directory and returns its entries.
    async fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
    
    /// Walks a directory tree and returns all paths.
    async fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
}
```

#### FileSystemManager

```rust
/// Default async implementation of the AsyncFileSystem trait.
pub struct FileSystemManager {
    // Private fields
}

impl FileSystemManager {
    /// Creates a new FileSystemManager.
    pub fn new() -> Self;
    
    /// Creates a new FileSystemManager with project configuration.
    pub async fn new_with_project_config(project_root: &Path) -> Result<Self>;
}

impl AsyncFileSystem for FileSystemManager {
    // All AsyncFileSystem trait methods implemented
}
```

### Path Utilities

#### PathUtils

```rust
/// Utility functions for working with paths.
pub struct PathUtils;

impl PathUtils {
    /// Finds the root directory of a Node.js project.
    pub fn find_project_root(start: &Path) -> Option<PathBuf>;
    
    /// Gets the current working directory.
    pub fn current_dir() -> FileSystemResult<PathBuf>;
    
    /// Makes a path relative to a base path.
    pub fn make_relative(path: &Path, base: &Path) -> FileSystemResult<PathBuf>;
}
```

### Node.js Path Extensions

#### NodePathKind

```rust
/// Represents common directory and file types in Node.js projects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodePathKind {
    /// Node modules directory
    NodeModules,
    /// Package configuration
    PackageJson,
    /// Source directory
    Src,
    /// Distribution directory
    Dist,
    /// Test directory
    Test,
}

impl NodePathKind {
    /// Returns the default path string for the given Node.js path kind.
    pub fn default_path(self) -> &'static str;
}
```

#### PathExt

```rust
/// Extension trait for Path with Node.js-specific functionality.
pub trait PathExt {
    /// Normalizes a path by resolving `.` and `..` components.
    fn normalize(&self) -> PathBuf;
    
    /// Checks if this path is inside a Node.js project.
    fn is_in_project(&self) -> bool;
    
    /// Gets the path relative to the nearest Node.js project root.
    fn relative_to_project(&self) -> Option<PathBuf>;
    
    /// Joins a Node.js path kind to this path.
    fn node_path(&self, kind: NodePathKind) -> PathBuf;
    
    /// Canonicalizes a path, resolving symlinks if needed.
    fn canonicalize(&self) -> Result<PathBuf>;
}

impl PathExt for Path {
    // All PathExt methods implemented
}
```

#### Examples

```rust
use sublime_standard_tools::filesystem::{
    AsyncFileSystem, FileSystemManager, NodePathKind, PathExt, PathUtils
};
use std::path::Path;

// Configuration-aware filesystem operations
let fs = FileSystemManager::new_with_project_config(Path::new(".")).await?;
let content = fs.read_file_string(Path::new("package.json")).await?;
fs.write_file_string(Path::new("output.txt"), "Hello, world!").await?;

// Directory operations
let entries = fs.read_dir(Path::new(".")).await?;
for entry in entries {
    println!("Found: {}", entry.display());
}

// Path extensions
let path = Path::new("src/components/Button.js");
let normalized = path.normalize();

if path.is_in_project() {
    println!("Path is within a Node.js project");
    
    if let Some(relative) = path.relative_to_project() {
        println!("Path relative to project: {}", relative.display());
    }
}

// Node.js specific paths
let node_modules = Path::new(".").node_path(NodePathKind::NodeModules);
println!("Node modules path: {}", node_modules.display());
```

## Error Module

The error module provides comprehensive error handling for all operations within the crate.

### Error Types

#### Main Error Types

```rust
/// General error type for the standard tools library.
#[derive(ThisError, Debug, Clone)]
pub enum Error {
    /// Monorepo-related error.
    #[error("Monorepo execution error")]
    Monorepo(#[from] MonorepoError),
    
    /// Filesystem-related error.
    #[error("FileSystem execution error")]
    FileSystem(#[from] FileSystemError),
    
    /// Workspace-related error.
    #[error("Workspace execution error")]
    Workspace(#[from] WorkspaceError),
    
    /// Command-related error.
    #[error("Command execution error")]
    Command(#[from] CommandError),
    
    /// Configuration-related error.
    #[error("Configuration error")]
    Config(#[from] ConfigError),
    
    /// General purpose errors with a custom message.
    #[error("Operation error: {0}")]
    Operation(String),
}

impl Error {
    /// Creates a new operational error.
    pub fn operation(message: impl Into<String>) -> Self;
}
```

#### ConfigError

```rust
/// Errors that can occur during configuration operations.
#[derive(ThisError, Debug, Clone)]
pub enum ConfigError {
    /// Configuration file not found.
    #[error("Configuration file not found: {path}")]
    NotFound { path: PathBuf },
    
    /// Failed to parse configuration file.
    #[error("Failed to parse configuration file '{path}': {source}")]
    ParseError { path: PathBuf, source: String },
    
    /// Invalid configuration values.
    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },
    
    /// Configuration validation failed.
    #[error("Configuration validation failed: {errors:?}")]
    ValidationFailed { errors: Vec<String> },
    
    /// Unsupported configuration format.
    #[error("Unsupported configuration format: {format}")]
    UnsupportedFormat { format: String },
    
    /// I/O error during configuration operations.
    #[error("I/O error during configuration operation: {source}")]
    Io { source: std::io::Error },
}
```

#### FileSystemError

```rust
/// Errors that can occur during filesystem operations.
#[derive(ThisError, Debug)]
pub enum FileSystemError {
    /// Path not found.
    #[error("Path not found: {path}")]
    NotFound { path: PathBuf },

    /// Permission denied for accessing the path.
    #[error("Permission denied for path: {path}")]
    PermissionDenied { path: PathBuf },

    /// Generic I/O error during filesystem operation.
    #[error("I/O error accessing path '{path}': {source}")]
    Io { path: PathBuf, source: io::Error },

    /// Attempted an operation requiring a directory on a file.
    #[error("Expected a directory but found a file: {path}")]
    NotADirectory { path: PathBuf },

    /// Attempted an operation requiring a file on a directory.
    #[error("Expected a file but found a directory: {path}")]
    NotAFile { path: PathBuf },

    /// Failed to decode UTF-8 content from a file.
    #[error("Failed to decode UTF-8 content in file: {path}")]
    Utf8Decode { path: PathBuf, source: std::string::FromUtf8Error },

    /// Path validation failed.
    #[error("Path validation failed for '{path}': {reason}")]
    Validation { path: PathBuf, reason: String },
}
```

#### CommandError

```rust
/// Errors that can occur during command execution.
#[derive(ThisError, Debug)]
pub enum CommandError {
    /// The command failed to start.
    #[error("Failed to spawn command '{cmd}': {source}")]
    SpawnFailed { cmd: String, source: io::Error },

    /// The command execution process itself failed.
    #[error("Command execution failed for '{cmd}': {source:?}")]
    ExecutionFailed { cmd: String, source: Option<io::Error> },

    /// The command executed but returned a non-zero exit code.
    #[error("Command '{cmd}' failed with exit code {code:?}. Stderr: {stderr}")]
    NonZeroExitCode { cmd: String, code: Option<i32>, stderr: String },

    /// The command timed out.
    #[error("Command timed out after {duration:?}")]
    Timeout { duration: Duration },

    /// The command was killed.
    #[error("Command was killed: {reason}")]
    Killed { reason: String },

    /// Invalid configuration provided for the command.
    #[error("Invalid command configuration: {description}")]
    Configuration { description: String },

    /// Failed to capture stdout or stderr.
    #[error("Failed to capture {stream} stream")]
    CaptureFailed { stream: String },

    /// Error occurred while reading stdout or stderr stream.
    #[error("Error reading {stream} stream: {source}")]
    StreamReadError { stream: String, source: io::Error },

    /// Generic error during command processing.
    #[error("Command processing error: {0}")]
    Generic(String),
}
```

#### MonorepoError

```rust
/// Errors that can occur during monorepo operations.
#[derive(ThisError, Debug)]
pub enum MonorepoError {
    /// Failed to detect the monorepo type.
    #[error("Failed to detect monorepo type: {source}")]
    Detection { source: FileSystemError },
    
    /// Failed to parse the monorepo descriptor file.
    #[error("Failed to parse monorepo descriptor: {source}")]
    Parsing { source: FileSystemError },
    
    /// Failed to read the monorepo descriptor file.
    #[error("Failed to read monorepo descriptor: {source}")]
    Reading { source: FileSystemError },
    
    /// Failed to write the monorepo descriptor file.
    #[error("Failed to write monorepo descriptor: {source}")]
    Writing { source: FileSystemError },
    
    /// Failed to find a package manager for the monorepo.
    #[error("Failed to find package manager")]
    ManagerNotFound,
}
```

#### WorkspaceError

```rust
/// Errors that can occur during workspace operations.
#[derive(ThisError, Debug)]
pub enum WorkspaceError {
    /// Error parsing package.json format.
    #[error("Invalid package json format: {0}")]
    InvalidPackageJson(String),
    
    /// Error parsing workspaces pattern.
    #[error("Invalid workspaces pattern: {0}")]
    InvalidWorkspacesPattern(String),
    
    /// Error parsing pnpm workspace configuration.
    #[error("Invalid pnpm workspace configuration: {0}")]
    InvalidPnpmWorkspace(String),
    
    /// Package not found in workspace.
    #[error("Package not found: {0}")]
    PackageNotFound(String),
    
    /// Workspace not found.
    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),
    
    /// Workspace configuration is missing.
    #[error("Workspace config is missing: {0}")]
    WorkspaceConfigMissing(String),
}
```

### Result Types

```rust
/// Convenient type aliases for Results
pub type FileSystemResult<T> = std::result::Result<T, FileSystemError>;
pub type MonorepoResult<T> = std::result::Result<T, MonorepoError>;
pub type WorkspaceResult<T> = std::result::Result<T, WorkspaceError>;
pub type CommandResult<T> = std::result::Result<T, CommandError>;
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;
pub type Result<T> = std::result::Result<T, Error>;
```

### Error Recovery

#### ErrorRecoveryManager

```rust
/// Manages error recovery strategies and provides context-aware error handling.
pub struct ErrorRecoveryManager {
    // Private fields
}

impl ErrorRecoveryManager {
    /// Creates a new ErrorRecoveryManager.
    pub fn new() -> Self;
    
    /// Attempts to recover from an error using registered strategies.
    pub async fn recover<T>(&self, error: &Error, context: &str) -> RecoveryResult<T>;
    
    /// Registers a recovery strategy for a specific error type.
    pub fn register_strategy(&mut self, strategy: Box<dyn RecoveryStrategy>);
    
    /// Logs an error with appropriate level and context.
    pub fn log_error(&self, error: &Error, context: &str, level: LogLevel);
}
```

#### RecoveryStrategy

```rust
/// Trait for implementing error recovery strategies.
#[async_trait]
pub trait RecoveryStrategy: Send + Sync {
    /// Attempts to recover from the given error.
    async fn recover<T>(&self, error: &Error, context: &str) -> RecoveryResult<T>;
    
    /// Checks if this strategy can handle the given error.
    fn can_handle(&self, error: &Error) -> bool;
    
    /// Returns the priority of this strategy (higher is better).
    fn priority(&self) -> u8;
}
```

#### RecoveryResult

```rust
/// Result of an error recovery attempt.
pub enum RecoveryResult<T> {
    /// Recovery was successful.
    Recovered(T),
    /// Recovery failed, but error was handled gracefully.
    HandledGracefully,
    /// Recovery failed, original error should be propagated.
    Failed,
}
```

#### LogLevel

```rust
/// Logging levels for error reporting.
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}
```

This comprehensive API specification reflects the current architectural approach with clean separation of concerns, unified project handling, robust configuration management, async-first design, and comprehensive error handling. The crate provides a consistent, type-safe interface for working with Node.js projects from Rust applications with extensive configuration capabilities and performance optimizations.