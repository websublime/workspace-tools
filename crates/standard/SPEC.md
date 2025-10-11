# Sublime Standard Tools API Specification

This document provides a comprehensive specification of the public API for the `sublime_standard_tools` crate, a robust toolkit for working with Node.js projects from Rust applications.

## Table of Contents

- [Overview](#overview)
- [Version Information](#version-information)
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
  - [Project Validation](#project-validation)
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

## Version Information

```rust
/// Version of the crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the version of the crate
#[must_use]
pub fn version() -> &'static str;
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
    
    /// Adds a configuration file as an optional source (won't fail if missing).
    pub fn with_file_optional(self, path: impl Into<PathBuf>) -> Self;
    
    /// Adds environment variables with a prefix as a source.
    pub fn with_env_prefix(self, prefix: impl Into<String>) -> Self;
    
    /// Adds a custom configuration source.
    pub fn with_source(self, source: ConfigSource) -> Self;
    
    /// Builds the ConfigManager.
    pub fn build(self) -> ConfigResult<ConfigManager<T>>;
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

#### ConfigProvider Trait

```rust
/// Trait for configuration providers.
#[async_trait]
pub trait ConfigProvider<T: Configurable>: Send + Sync {
    /// Loads configuration from this provider.
    async fn load(&self) -> ConfigResult<Option<T>>;
    
    /// Saves configuration to this provider (if supported).
    async fn save(&self, config: &T) -> ConfigResult<()>;
    
    /// Returns the priority of this provider.
    fn priority(&self) -> ConfigSourcePriority;
    
    /// Returns whether this provider supports saving.
    fn supports_save(&self) -> bool;
}
```

#### Configuration Sources

```rust
/// Different sources of configuration data.
#[derive(Debug, Clone)]
pub enum ConfigSource {
    /// Default values
    Defaults,
    /// Environment variables with prefix
    Environment { prefix: String },
    /// Configuration file
    File { path: PathBuf, optional: bool },
    /// Custom provider
    Custom(Box<dyn ConfigProvider<T>>),
}

/// Priority levels for configuration sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigSourcePriority {
    Lowest = 0,
    Low = 25,
    Medium = 50,
    High = 75,
    Highest = 100,
}
```

#### Configuration Formats

```rust
/// Supported configuration file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// TOML format
    Toml,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
}

impl ConfigFormat {
    /// Detects format from file extension.
    pub fn from_extension(ext: &str) -> Option<Self>;
    
    /// Returns the default file extension for this format.
    pub fn extension(&self) -> &'static str;
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

impl Default for StandardConfig {
    fn default() -> Self;
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
    pub detection_order: Vec<String>,
    
    /// Custom lock file names for each package manager
    pub custom_lock_files: HashMap<String, String>,
    
    /// Whether to detect from environment variables
    pub detect_from_env: bool,
    
    /// Environment variable name for preferred package manager
    pub env_var_name: String,
    
    /// Custom binary paths for package managers
    pub binary_paths: HashMap<String, String>,
    
    /// Fallback package manager if none detected
    pub fallback: Option<String>,
}

impl Default for PackageManagerConfig {
    fn default() -> Self;
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

impl Default for MonorepoConfig {
    fn default() -> Self;
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

impl Default for CommandConfig {
    fn default() -> Self;
}
```

#### FilesystemConfig

```rust
/// Filesystem operation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemConfig {
    /// Patterns to ignore during directory traversal
    pub ignore_patterns: Vec<String>,
    
    /// Async I/O configuration
    pub async_io: AsyncIoConfig,
    
    /// File operation retry configuration
    pub retry: Option<RetryConfig>,
    
    /// Path conventions overrides
    pub path_conventions: PathConventions,
}

/// Async I/O configuration for filesystem operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncIoConfig {
    /// Buffer size for async I/O operations
    pub buffer_size: usize,
    
    /// Maximum concurrent filesystem operations
    pub max_concurrent_operations: usize,
    
    /// Timeout for individual operations
    #[serde(with = "humantime_serde")]
    pub operation_timeout: Duration,
}

/// Retry configuration for filesystem operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    
    /// Initial delay between retries
    #[serde(with = "humantime_serde")]
    pub initial_delay: Duration,
    
    /// Maximum delay between retries
    #[serde(with = "humantime_serde")]
    pub max_delay: Duration,
    
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

/// Path naming conventions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConventions {
    /// Node modules directory name
    pub node_modules: String,
    
    /// Package.json file name
    pub package_json: String,
}

impl Default for FilesystemConfig {
    fn default() -> Self;
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

impl Default for ValidationConfig {
    fn default() -> Self;
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
- `SUBLIME_COLLECTION_WINDOW_MS`: Queue collection window in milliseconds (1-1000)
- `SUBLIME_COLLECTION_SLEEP_US`: Queue collection sleep in microseconds (10-10000)
- `SUBLIME_IDLE_SLEEP_MS`: Queue idle sleep in milliseconds (1-1000)

#### Filesystem Configuration
- `SUBLIME_IGNORE_PATTERNS`: Comma-separated filesystem ignore patterns
- `SUBLIME_ASYNC_BUFFER_SIZE`: Async I/O buffer size in bytes (1024-1048576)
- `SUBLIME_MAX_CONCURRENT_IO`: Maximum concurrent I/O operations (1-1000)
- `SUBLIME_IO_TIMEOUT`: I/O operation timeout in seconds (1-300)

#### Examples

```rust
use sublime_standard_tools::config::{ConfigManager, StandardConfig, ConfigBuilder};
use std::path::Path;

// Load configuration with auto-detection
async fn load_config() -> Result<StandardConfig, Box<dyn std::error::Error>> {
    let manager = ConfigManager::<StandardConfig>::builder()
        .with_defaults()
        .with_file_optional("~/.config/sublime/config.toml")
        .with_file_optional(".sublime.toml")
        .with_env_prefix("SUBLIME")
        .build()?;

    let config = manager.load().await?;
    Ok(config)
}

// Use configuration in components
let config = load_config().await?;
println!("Detection order: {:?}", config.package_managers.detection_order);
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    
    /// Parses a package manager kind from string.
    pub fn from_str(s: &str) -> Option<Self>;
    
    /// Returns all available package manager kinds.
    pub fn all() -> &'static [PackageManagerKind];
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
    pub fn detect(path: impl AsRef<Path>) -> Result<Self, Error>;
    
    /// Detects which package manager is being used with custom configuration.
    pub fn detect_with_config(
        path: impl AsRef<Path>, 
        config: &PackageManagerConfig
    ) -> Result<Self, Error>;
    
    /// Returns the kind of package manager.
    pub fn kind(&self) -> PackageManagerKind;
    
    /// Returns the root directory path of the package manager.
    pub fn root(&self) -> &Path;
    
    /// Returns the command name for this package manager.
    pub fn command(&self) -> &'static str;
    
    /// Returns the lock file name for this package manager.
    pub fn lock_file_name(&self) -> Option<&'static str>;
    
    /// Returns the full path to the lock file for this package manager.
    pub fn lock_file_path(&self) -> Option<PathBuf>;
    
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
    
    /// Returns the root directory of the repository.
    fn root(&self) -> &Path;
}
```

#### Examples

```rust
use sublime_standard_tools::node::{PackageManager, PackageManagerKind, RepoKind};
use sublime_standard_tools::config::PackageManagerConfig;
use std::path::Path;

// Package manager detection with default configuration
let manager = PackageManager::detect(Path::new("."))?;
println!("Using package manager: {}", manager.command());
println!("Supports workspaces: {}", manager.supports_workspaces());

// Package manager detection with custom configuration
let config = PackageManagerConfig {
    detection_order: vec![
        "pnpm".to_string(),
        "yarn".to_string(),
        "npm".to_string(),
    ],
    detect_from_env: true,
    ..Default::default()
};

let manager = PackageManager::detect_with_config(Path::new("."), &config)?;

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
    fn package_json(&self) -> Option<&serde_json::Value>;
    
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
    pub fn new(
        root: PathBuf, 
        kind: ProjectKind, 
        package_manager: Option<PackageManager>,
        package_json: Option<serde_json::Value>,
    ) -> Self;
    
    /// Checks if this project is a monorepo.
    pub fn is_monorepo(&self) -> bool;
    
    /// Gets the project dependencies.
    pub fn dependencies(&self) -> &Dependencies;
    
    /// Returns internal workspace dependencies for monorepos.
    pub fn internal_dependencies(&self) -> &[WorkspacePackage];
    
    /// Sets the validation status for this project.
    pub fn set_validation_status(&mut self, status: ProjectValidationStatus);
}

impl ProjectInfo for Project {
    fn root(&self) -> &Path;
    fn package_manager(&self) -> Option<&PackageManager>;
    fn package_json(&self) -> Option<&serde_json::Value>;
    fn validation_status(&self) -> &ProjectValidationStatus;
    fn kind(&self) -> ProjectKind;
}
```

#### Dependencies

```rust
/// Represents project dependencies information.
#[derive(Debug, Clone, Default)]
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

impl Dependencies {
    /// Creates a new empty Dependencies instance.
    pub fn new() -> Self;
    
    /// Returns all dependency names across all types.
    pub fn all_names(&self) -> Vec<&str>;
    
    /// Checks if a dependency exists in any category.
    pub fn contains(&self, name: &str) -> bool;
    
    /// Gets the version requirement for a dependency.
    pub fn get_version(&self, name: &str) -> Option<&str>;
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
    
    /// Consumes the descriptor and returns the inner project.
    pub fn into_project(self) -> Project;
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
    
    /// Returns the status as a string.
    pub fn status(&self) -> &str;
}
```

### Project Detection

#### ProjectDetectorTrait

```rust
/// Async trait for project detection.
#[async_trait]
pub trait ProjectDetectorTrait: Send + Sync {
    /// Asynchronously detects and analyzes a project at the given path.
    async fn detect(
        &self, 
        path: &Path, 
        config: Option<&StandardConfig>
    ) -> Result<ProjectDescriptor, Error>;
    
    /// Asynchronously detects only the project kind without full analysis.
    async fn detect_kind(&self, path: &Path) -> Result<ProjectKind, Error>;
    
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
    ) -> Vec<Result<ProjectDescriptor, Error>>;
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
    
    /// Creates a new ProjectDetector with project-specific configuration.
    pub async fn new_with_project_config(project_root: &Path) -> Result<Self, Error>;
}

impl<F: AsyncFileSystem + Clone + 'static> ProjectDetector<F> {
    /// Creates a new ProjectDetector with a custom async filesystem.
    pub fn with_filesystem(fs: F) -> Self;
    
    /// Creates a new ProjectDetector with filesystem and configuration.
    pub fn with_filesystem_and_config(fs: F, config: StandardConfig) -> Self;
}

impl<F: AsyncFileSystem + Clone + Send + Sync + 'static> ProjectDetector<F> {
    /// Detects and analyzes a project using configuration-controlled detection.
    pub async fn detect(
        &self,
        path: impl AsRef<Path>,
        config: Option<&StandardConfig>,
    ) -> Result<ProjectDescriptor, Error>;
    
    /// Detects only the project kind using default configuration.
    pub async fn detect_kind(&self, path: impl AsRef<Path>) -> Result<ProjectKind, Error>;
    
    /// Detects the project kind using custom configuration.
    pub async fn detect_kind_with_config(
        &self,
        path: impl AsRef<Path>,
        config: &StandardConfig,
    ) -> Result<ProjectKind, Error>;
    
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
    
    /// Creates a new ProjectManager with project configuration.
    pub async fn new_with_project_config(project_root: &Path) -> Result<Self, Error>;
}

impl<F: AsyncFileSystem + Clone> ProjectManager<F> {
    /// Creates a new ProjectManager with a custom filesystem.
    pub fn with_filesystem(fs: F) -> Self;
    
    /// Creates a new ProjectManager with filesystem and configuration.
    pub fn with_filesystem_and_config(fs: F, config: StandardConfig) -> Self;
    
    /// Creates a project descriptor from a path with configuration.
    pub async fn create_project(
        &self, 
        path: impl AsRef<Path>, 
        config: Option<&StandardConfig>
    ) -> Result<ProjectDescriptor, Error>;
    
    /// Updates an existing project with new information.
    pub async fn update_project(
        &self,
        project: &mut Project,
        config: Option<&StandardConfig>,
    ) -> Result<(), Error>;
}
```

### Project Validation

#### ProjectValidator

```rust
/// Validates Node.js project structures and configurations.
pub struct ProjectValidator {
    // Private fields
}

impl ProjectValidator {
    /// Creates a new ProjectValidator with the given configuration.
    pub fn new(config: StandardConfig) -> Self;
    
    /// Validates a project and returns the validation status.
    pub async fn validate(&self, project: &Project) -> Result<ProjectValidationStatus, Error>;
    
    /// Validates a project descriptor.
    pub async fn validate_descriptor(
        &self, 
        project: &ProjectDescriptor
    ) -> Result<ProjectValidationStatus, Error>;
    
    /// Validates only package.json requirements.
    pub async fn validate_package_json(
        &self,
        package_json: &serde_json::Value,
    ) -> Result<Vec<String>, Error>;
    
    /// Validates dependency structures.
    pub async fn validate_dependencies(
        &self,
        dependencies: &Dependencies,
    ) -> Result<Vec<String>, Error>;
}
```

#### Examples

```rust
use sublime_standard_tools::project::{
    ProjectDetector, ProjectDetectorTrait, ProjectDescriptor, ProjectInfo, ProjectValidator
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
        
        // Validate the project
        let validator = ProjectValidator::new(StandardConfig::default());
        let validation_status = validator.validate(&nodejs_project).await?;
        println!("Validation: {:?}", validation_status);
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonorepoKind {
    /// Npm workspaces monorepo
    NpmWorkspaces,
    /// Yarn Workspaces monorepo
    YarnWorkspaces,
    /// pnpm Workspaces monorepo
    PnpmWorkspaces,
    /// Bun workspaces monorepo
    BunWorkspaces,
    /// Deno workspaces monorepo
    DenoWorkspaces,
    /// Lerna monorepo
    Lerna,
    /// Rush monorepo
    Rush,
    /// Nx monorepo
    Nx,
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
    pub fn config_file(&self) -> String;
    
    /// Creates a custom monorepo kind with the specified name and config file.
    pub fn custom(name: String, config_file: String) -> Self;
    
    /// Returns all known monorepo kinds (excluding custom).
    pub fn all_known() -> &'static [MonorepoKind];
    
    /// Parses a monorepo kind from its name.
    pub fn from_name(name: &str) -> Option<Self>;
}
```

#### WorkspacePackage

```rust
/// Represents a single package within a monorepo workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePackage {
    /// Name of the package
    pub name: String,
    /// Version of the package
    pub version: String,
    /// Location of the package relative to the monorepo root
    pub location: PathBuf,
    /// Absolute path to the package
    pub absolute_path: PathBuf,
    /// Production dependencies within the workspace
    pub dependencies: HashMap<String, String>,
    /// Development dependencies within the workspace
    pub dev_dependencies: HashMap<String, String>,
    /// Workspace dependencies (internal dependencies)
    pub workspace_dependencies: Vec<String>,
    /// Workspace dev dependencies (internal dev dependencies)
    pub workspace_dev_dependencies: Vec<String>,
}

impl WorkspacePackage {
    /// Creates a new WorkspacePackage.
    pub fn new(
        name: String,
        version: String,
        location: PathBuf,
        absolute_path: PathBuf,
    ) -> Self;
    
    /// Adds a workspace dependency.
    pub fn add_workspace_dependency(&mut self, name: String);
    
    /// Adds a workspace dev dependency.
    pub fn add_workspace_dev_dependency(&mut self, name: String);
    
    /// Checks if this package depends on another workspace package.
    pub fn depends_on_workspace(&self, name: &str) -> bool;
    
    /// Returns all workspace dependencies (both prod and dev).
    pub fn all_workspace_dependencies(&self) -> Vec<&str>;
}
```

#### WorkspaceConfig

```rust
/// Configuration for workspace detection and management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Glob patterns for workspace packages
    pub patterns: Vec<String>,
    
    /// Packages to exclude from workspace
    pub exclude: Option<Vec<String>>,
    
    /// Additional workspace-specific configuration
    pub additional_config: HashMap<String, serde_json::Value>,
}

impl WorkspaceConfig {
    /// Creates a new WorkspaceConfig with the given patterns.
    pub fn new(patterns: Vec<String>) -> Self;
    
    /// Adds an exclusion pattern.
    pub fn exclude(&mut self, pattern: String);
    
    /// Checks if a path matches any workspace pattern.
    pub fn matches_pattern(&self, path: &Path) -> bool;
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
        package_json: Option<serde_json::Value>,
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
    
    /// Returns the workspace configuration if available.
    pub fn workspace_config(&self) -> Option<&WorkspaceConfig>;
    
    /// Adds a package to the monorepo.
    pub fn add_package(&mut self, package: WorkspacePackage);
    
    /// Removes a package from the monorepo.
    pub fn remove_package(&mut self, name: &str) -> Option<WorkspacePackage>;
}

impl ProjectInfo for MonorepoDescriptor {
    fn root(&self) -> &Path;
    fn package_manager(&self) -> Option<&PackageManager>;
    fn package_json(&self) -> Option<&serde_json::Value>;
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
    async fn is_monorepo_root(&self, path: &Path) -> Result<Option<MonorepoKind>, Error>;
    
    /// Finds the nearest monorepo root by walking up from the given path.
    async fn find_monorepo_root(
        &self,
        start_path: &Path,
    ) -> Result<Option<(PathBuf, MonorepoKind)>, Error>;
    
    /// Detects and analyzes a monorepo at the given path.
    async fn detect_monorepo(&self, path: &Path) -> Result<MonorepoDescriptor, Error>;
    
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
    async fn detect_multiple(&self, paths: &[&Path]) -> Vec<Result<MonorepoDescriptor, Error>>;
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
    
    /// Creates a new MonorepoDetector with project-specific configuration.
    pub async fn new_with_project_config(project_root: &Path) -> Result<Self, Error>;
}

impl<F: AsyncFileSystem + Clone> MonorepoDetector<F> {
    /// Creates a new MonorepoDetector with a custom filesystem.
    pub fn with_filesystem(fs: F) -> Self;
    
    /// Creates a new MonorepoDetector with filesystem and configuration.
    pub fn with_filesystem_and_config(fs: F, config: MonorepoConfig) -> Self;
}

impl<F: AsyncFileSystem + Clone + Send + Sync + 'static> MonorepoDetector<F> {
    /// Checks if a path is the root of a monorepo.
    pub async fn is_monorepo_root(
        &self, 
        path: impl AsRef<Path>
    ) -> Result<Option<MonorepoKind>, Error>;
    
    /// Finds the nearest monorepo root by walking up from the given path.
    pub async fn find_monorepo_root(
        &self,
        start_path: impl AsRef<Path>,
    ) -> Result<Option<(PathBuf, MonorepoKind)>, Error>;
    
    /// Detects and analyzes a monorepo at the given path.
    pub async fn detect_monorepo(
        &self, 
        path: impl AsRef<Path>
    ) -> Result<MonorepoDescriptor, Error>;
    
    /// Checks if a directory contains multiple packages.
    pub async fn has_multiple_packages(&self, path: &Path) -> bool;
    
    /// Gets all workspace packages in a directory.
    pub async fn get_workspace_packages(
        &self,
        root: &Path,
        config: &MonorepoConfig,
    ) -> Result<Vec<WorkspacePackage>, Error>;
}
```

### Workspace Management

#### PnpmWorkspaceConfig

```rust
/// Configuration structure for PNPM workspaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnpmWorkspaceConfig {
    /// Package locations (glob patterns)
    pub packages: Vec<String>,
    
    /// Packages to exclude
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
}

impl PnpmWorkspaceConfig {
    /// Creates a new PnpmWorkspaceConfig.
    pub fn new(packages: Vec<String>) -> Self;
    
    /// Adds a package pattern.
    pub fn add_package(&mut self, pattern: String);
    
    /// Adds an exclusion pattern.
    pub fn add_exclusion(&mut self, pattern: String);
}
```

#### Examples

```rust
use sublime_standard_tools::monorepo::{
    MonorepoDetector, MonorepoDetectorTrait, MonorepoKind, WorkspacePackage
};
use sublime_standard_tools::config::MonorepoConfig;
use std::path::Path;

// Configuration-aware monorepo detection
let config = MonorepoConfig {
    workspace_patterns: vec!["packages/*".to_string(), "apps/*".to_string()],
    max_search_depth: 3,
    exclude_patterns: vec!["node_modules".to_string()],
    ..Default::default()
};

let detector = MonorepoDetector::new_with_project_config(Path::new(".")).await?;

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
        
        // Print workspace dependencies
        let workspace_deps = package.all_workspace_dependencies();
        if !workspace_deps.is_empty() {
            println!("  Workspace Dependencies:");
            for dep in workspace_deps {
                println!("    - {}", dep);
            }
        }
    }
    
    // Generate dependency graph
    let graph = monorepo.get_dependency_graph();
    for (pkg_name, deps) in graph {
        println!("{} depends on {} workspace packages", pkg_name, deps.len());
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
#[derive(Debug, Clone)]
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
    
    /// Returns a string representation of the command.
    pub fn to_string(&self) -> String;
}
```

#### Executor

```rust
/// Trait for executing commands.
#[async_trait]
pub trait Executor: Send + Sync {
    /// Executes a command and returns the output.
    async fn execute(&self, command: Command) -> Result<CommandOutput, CommandError>;
    
    /// Executes a command with streaming output.
    async fn execute_stream(
        &self,
        command: Command,
        stream_config: StreamConfig,
    ) -> Result<(CommandStream, tokio::process::Child), CommandError>;
}
```

#### DefaultCommandExecutor

```rust
/// Default async command executor implementation.
#[derive(Debug, Clone)]
pub struct DefaultCommandExecutor {
    // Private fields
}

impl DefaultCommandExecutor {
    /// Creates a new DefaultCommandExecutor.
    pub fn new() -> Self;
    
    /// Creates a new DefaultCommandExecutor with project configuration.
    pub async fn new_with_project_config(project_root: &Path) -> Result<Self, Error>;
    
    /// Creates a new DefaultCommandExecutor with custom configuration.
    pub fn with_config(config: CommandConfig) -> Self;
}

impl Executor for DefaultCommandExecutor {
    async fn execute(&self, command: Command) -> Result<CommandOutput, CommandError>;
    
    async fn execute_stream(
        &self,
        command: Command,
        stream_config: StreamConfig,
    ) -> Result<(CommandStream, tokio::process::Child), CommandError>;
}
```

#### SyncCommandExecutor

```rust
/// Synchronous command executor for blocking operations.
#[derive(Debug)]
pub struct SyncCommandExecutor {
    // Private fields
}

impl SyncCommandExecutor {
    /// Creates a new SyncCommandExecutor.
    pub fn new() -> Self;
    
    /// Creates a new SyncCommandExecutor with custom configuration.
    pub fn with_config(config: CommandConfig) -> Self;
    
    /// Executes a command synchronously.
    pub fn execute(&self, command: Command) -> Result<CommandOutput, CommandError>;
}
```

#### SharedSyncExecutor

```rust
/// Thread-safe shared synchronous executor.
pub type SharedSyncExecutor = std::sync::Arc<std::sync::Mutex<SyncCommandExecutor>>;
```

### Command Builder

#### CommandBuilder

```rust
/// Builder for creating Command instances.
#[derive(Debug, Clone)]
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
    
    /// Sets multiple environment variables.
    pub fn envs<I, K, V>(mut self, vars: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>;
    
    /// Sets the current directory.
    pub fn current_dir(mut self, path: impl AsRef<Path>) -> Self;
    
    /// Sets the timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self;
    
    /// Clears all arguments.
    pub fn clear_args(mut self) -> Self;
    
    /// Clears all environment variables.
    pub fn clear_env(mut self) -> Self;
}
```

### Command Output

#### CommandOutput

```rust
/// Represents the output of a command execution.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    // Private fields
}

impl CommandOutput {
    /// Creates a new CommandOutput instance.
    pub fn new(
        status: std::process::ExitStatus, 
        stdout: String, 
        stderr: String, 
        duration: Duration
    ) -> Self;
    
    /// Returns the exit status.
    pub fn status(&self) -> std::process::ExitStatus;
    
    /// Returns the standard output content.
    pub fn stdout(&self) -> &str;
    
    /// Returns the standard error content.
    pub fn stderr(&self) -> &str;
    
    /// Returns the command execution duration.
    pub fn duration(&self) -> Duration;
    
    /// Returns true if the command was successful (exit code 0).
    pub fn success(&self) -> bool;
    
    /// Returns the exit code if available.
    pub fn exit_code(&self) -> Option<i32>;
}
```

### Command Queue

#### CommandQueue

```rust
/// Manages queued command execution with priority.
#[derive(Debug)]
pub struct CommandQueue {
    // Private fields
}

impl CommandQueue {
    /// Creates a new CommandQueue with default configuration.
    pub fn new() -> Self;
    
    /// Creates a new CommandQueue with custom configuration.
    pub fn new_with_config(config: CommandQueueConfig) -> Self;
    
    /// Creates a new CommandQueue with a custom executor.
    pub fn with_executor<E: Executor + 'static>(executor: E) -> Self;
    
    /// Starts the command queue processing.
    pub fn start(mut self) -> Result<Self, CommandError>;
    
    /// Enqueues a command with priority.
    pub async fn enqueue(
        &mut self, 
        command: Command, 
        priority: CommandPriority
    ) -> Result<String, CommandError>;
    
    /// Enqueues multiple commands.
    pub async fn enqueue_batch(
        &mut self, 
        commands: Vec<(Command, CommandPriority)>
    ) -> Result<Vec<String>, CommandError>;
    
    /// Gets the status of a command.
    pub async fn get_status(&self, id: &str) -> Option<CommandStatus>;
    
    /// Gets the result of a command.
    pub async fn get_result(&self, id: &str) -> Option<CommandQueueResult>;
    
    /// Waits for a command to complete.
    pub async fn wait_for_command(
        &mut self, 
        id: &str, 
        timeout: Duration
    ) -> Result<CommandQueueResult, CommandError>;
    
    /// Waits for all commands to complete.
    pub async fn wait_for_completion(&mut self) -> Result<(), CommandError>;
    
    /// Returns queue statistics.
    pub async fn stats(&self) -> Result<CommandQueueStats, CommandError>;
    
    /// Shuts down the queue.
    pub async fn shutdown(&mut self) -> Result<(), CommandError>;
}
```

#### CommandQueueConfig

```rust
/// Configuration for command queue behavior.
#[derive(Debug, Clone)]
pub struct CommandQueueConfig {
    /// Maximum concurrent commands
    pub max_concurrent_commands: usize,
    /// Collection window for batching
    pub collection_window: Duration,
    /// Sleep duration during collection
    pub collection_sleep: Duration,
    /// Sleep duration when idle
    pub idle_sleep: Duration,
}

impl Default for CommandQueueConfig {
    fn default() -> Self;
}
```

#### CommandPriority

```rust
/// Priority levels for commands in the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandPriority {
    /// Low priority command
    Low = 0,
    /// Normal priority command
    Normal = 1,
    /// High priority command
    High = 2,
    /// Critical priority command
    Critical = 3,
}
```

#### CommandStatus

```rust
/// Status of a command in the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandStatus {
    /// Command is queued for execution
    Queued,
    /// Command is currently running
    Running,
    /// Command completed successfully
    Completed,
    /// Command failed during execution
    Failed,
    /// Command was cancelled
    Cancelled,
}
```

#### CommandQueueResult

```rust
/// Result of a command execution from the queue.
#[derive(Debug, Clone)]
pub struct CommandQueueResult {
    /// The command output
    pub output: CommandOutput,
    /// The final status
    pub status: CommandStatus,
    /// Time when the command was queued
    pub queued_at: std::time::SystemTime,
    /// Time when the command started execution
    pub started_at: Option<std::time::SystemTime>,
    /// Time when the command completed
    pub completed_at: Option<std::time::SystemTime>,
}
```

#### CommandQueueStats

```rust
/// Statistics about command queue performance.
#[derive(Debug, Clone)]
pub struct CommandQueueStats {
    /// Total commands processed
    pub total_processed: usize,
    /// Commands currently running
    pub currently_running: usize,
    /// Commands in queue
    pub queued: usize,
    /// Average execution time
    pub average_execution_time: Duration,
}
```

### Command Stream

#### CommandStream

```rust
/// Represents a stream of command output.
#[derive(Debug)]
pub struct CommandStream {
    // Private fields
}

impl CommandStream {
    /// Receives the next output line with timeout.
    pub async fn next_timeout(
        &mut self,
        timeout_duration: Duration,
    ) -> Result<Option<StreamOutput>, CommandError>;
    
    /// Receives the next output line without timeout.
    pub async fn next(&mut self) -> Result<Option<StreamOutput>, CommandError>;
    
    /// Cancels the stream.
    pub fn cancel(&self);
    
    /// Checks if the stream is finished.
    pub fn is_finished(&self) -> bool;
}
```

#### StreamOutput

```rust
/// Represents output from a command stream.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Buffer size for reading output
    pub buffer_size: usize,
    /// Timeout for read operations
    pub read_timeout: Duration,
}

impl StreamConfig {
    /// Creates a new StreamConfig with specified parameters.
    pub fn new(buffer_size: usize, read_timeout: Duration) -> Self;
}

impl Default for StreamConfig {
    fn default() -> Self;
}
```

#### Examples

```rust
use sublime_standard_tools::command::{
    CommandBuilder, DefaultCommandExecutor, Executor, 
    CommandQueue, CommandPriority, StreamConfig, StreamOutput
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
let stream_config = StreamConfig::new(1024, Duration::from_secs(1));
let cmd = CommandBuilder::new("npm").args(["run", "build"]).build();

let (mut stream, _child) = executor.execute_stream(cmd, stream_config).await?;
while let Ok(Some(output)) = stream.next_timeout(Duration::from_secs(1)).await {
    match output {
        StreamOutput::Stdout(line) => println!("STDOUT: {}", line),
        StreamOutput::Stderr(line) => eprintln!("STDERR: {}", line),
        StreamOutput::End => break,
    }
}

// Command queue with priority
let config = CommandQueueConfig {
    max_concurrent_commands: 3,
    collection_window: Duration::from_millis(100),
    collection_sleep: Duration::from_micros(500),
    idle_sleep: Duration::from_millis(50),
};

let mut queue = CommandQueue::new_with_config(config).start()?;

let high_priority_cmd = CommandBuilder::new("npm").arg("test").build();
let normal_priority_cmd = CommandBuilder::new("npm").arg("lint").build();

let id1 = queue.enqueue(high_priority_cmd, CommandPriority::High).await?;
let id2 = queue.enqueue(normal_priority_cmd, CommandPriority::Normal).await?;

let result = queue.wait_for_command(&id1, Duration::from_secs(30)).await?;
println!("High priority command result: {:?}", result.status);

// Get queue statistics
let stats = queue.stats().await?;
println!("Processed {} commands", stats.total_processed);

queue.shutdown().await?;
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
    async fn read(&self, path: &Path) -> Result<Vec<u8>, FileSystemError>;
    
    /// Writes bytes to a file.
    async fn write(&self, path: &Path, contents: &[u8]) -> Result<(), FileSystemError>;
    
    /// Reads a file and returns its contents as a string.
    async fn read_to_string(&self, path: &Path) -> Result<String, FileSystemError>;
    
    /// Writes a string to a file.
    async fn write_string(&self, path: &Path, contents: &str) -> Result<(), FileSystemError>;
    
    /// Creates a directory and all parent directories.
    async fn create_dir_all(&self, path: &Path) -> Result<(), FileSystemError>;
    
    /// Removes a file or directory.
    async fn remove(&self, path: &Path) -> Result<(), FileSystemError>;
    
    /// Removes a file.
    async fn remove_file(&self, path: &Path) -> Result<(), FileSystemError>;
    
    /// Removes a directory and all its contents.
    async fn remove_dir_all(&self, path: &Path) -> Result<(), FileSystemError>;
    
    /// Checks if a path exists.
    async fn exists(&self, path: &Path) -> Result<bool, FileSystemError>;
    
    /// Checks if a path is a file.
    async fn is_file(&self, path: &Path) -> Result<bool, FileSystemError>;
    
    /// Checks if a path is a directory.
    async fn is_dir(&self, path: &Path) -> Result<bool, FileSystemError>;
    
    /// Gets metadata for a file or directory.
    async fn metadata(&self, path: &Path) -> Result<std::fs::Metadata, FileSystemError>;
    
    /// Reads a directory and returns its entries.
    async fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FileSystemError>;
    
    /// Walks a directory tree and returns all paths.
    async fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FileSystemError>;
    
    /// Copies a file from source to destination.
    async fn copy(&self, from: &Path, to: &Path) -> Result<(), FileSystemError>;
    
    /// Moves a file from source to destination.
    async fn rename(&self, from: &Path, to: &Path) -> Result<(), FileSystemError>;
}
```

#### AsyncFileSystemConfig

```rust
/// Configuration for async filesystem operations.
#[derive(Debug, Clone)]
pub struct AsyncFileSystemConfig {
    /// Buffer size for I/O operations
    pub buffer_size: usize,
    
    /// Maximum concurrent operations
    pub max_concurrent_operations: usize,
    
    /// Timeout for individual operations
    pub operation_timeout: Duration,
    
    /// Retry configuration for failed operations
    pub retry_config: Option<RetryConfig>,
    
    /// Patterns to ignore during operations
    pub ignore_patterns: Vec<String>,
}

impl Default for AsyncFileSystemConfig {
    fn default() -> Self;
}
```

#### FileSystemManager

```rust
/// Default async implementation of the AsyncFileSystem trait.
#[derive(Debug, Clone)]
pub struct FileSystemManager {
    // Private fields
}

impl FileSystemManager {
    /// Creates a new FileSystemManager with default configuration.
    pub fn new() -> Self;
    
    /// Creates a new FileSystemManager with custom configuration.
    pub fn new_with_config(config: AsyncFileSystemConfig) -> Self;
    
    /// Creates a new FileSystemManager with project configuration.
    pub async fn new_with_project_config(project_root: &Path) -> Result<Self, Error>;
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
    /// Finds the root directory of a Node.js project by walking up the directory tree.
    pub fn find_project_root(start: &Path) -> Option<PathBuf>;
    
    /// Gets the current working directory.
    pub fn current_dir() -> Result<PathBuf, FileSystemError>;
    
    /// Makes a path relative to a base path.
    pub fn make_relative(path: &Path, base: &Path) -> Result<PathBuf, FileSystemError>;
    
    /// Normalizes a path by resolving `.` and `..` components.
    pub fn normalize(path: &Path) -> PathBuf;
    
    /// Joins multiple path components safely.
    pub fn join<P: AsRef<Path>>(base: &Path, paths: &[P]) -> PathBuf;
    
    /// Checks if a path is absolute.
    pub fn is_absolute(path: &Path) -> bool;
    
    /// Gets the parent directory of a path.
    pub fn parent(path: &Path) -> Option<&Path>;
    
    /// Gets the file name from a path.
    pub fn file_name(path: &Path) -> Option<&str>;
    
    /// Gets the file extension from a path.
    pub fn extension(path: &Path) -> Option<&str>;
}
```

### Node.js Path Extensions

#### NodePathKind

```rust
/// Represents common directory and file types in Node.js projects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodePathKind {
    /// Project root directory (contains package.json)
    ProjectRoot,
    /// Package directory (workspace package)
    PackageDirectory,
    /// Node modules directory
    NodeModules,
    /// Source directory (src, lib, etc.)
    SourceDirectory,
    /// Distribution directory (dist, build, out, etc.)
    DistDirectory,
    /// Test directory (test, tests, __tests__, etc.)
    TestDirectory,
    /// Configuration directory (.config, config, etc.)
    ConfigDirectory,
    /// Documentation directory (docs, documentation, etc.)
    DocsDirectory,
    /// Examples directory
    ExamplesDirectory,
    /// Tools directory (scripts, tools, etc.)
    ToolsDirectory,
    /// Other path type
    Other,
}

impl NodePathKind {
    /// Returns the default path string for the given Node.js path kind.
    pub fn default_path(self) -> &'static str;
    
    /// Detects the path kind from a directory name.
    pub fn from_dir_name(name: &str) -> Self;
    
    /// Returns all known path kinds.
    pub fn all() -> &'static [NodePathKind];
}
```

#### PathExt

```rust
/// Extension trait for Path with Node.js-specific functionality.
pub trait PathExt {
    /// Normalizes a path by resolving `.` and `..` components.
    fn normalize(&self) -> PathBuf;
    
    /// Checks if this path contains a package.json file.
    fn is_package_json_dir(&self) -> bool;
    
    /// Checks if this path is inside a Node.js project.
    fn is_node_project(&self) -> bool;
    
    /// Gets the path relative to the nearest Node.js project root.
    fn relative_to_project(&self) -> Option<PathBuf>;
    
    /// Finds the package.json file by walking up the directory tree.
    fn find_package_json(&self) -> Option<PathBuf>;
    
    /// Determines the Node.js path kind for this path.
    fn node_path_kind(&self) -> NodePathKind;
    
    /// Joins a Node.js path kind to this path.
    fn node_path(&self, kind: NodePathKind) -> PathBuf;
    
    /// Checks if this path is in node_modules.
    fn is_in_node_modules(&self) -> bool;
    
    /// Checks if this path is a workspace package directory.
    fn is_workspace_package(&self) -> bool;
    
    /// Gets the workspace package name if this is a package directory.
    fn workspace_package_name(&self) -> Option<String>;
}

impl PathExt for Path {
    // All PathExt methods implemented
}

impl PathExt for PathBuf {
    // All PathExt methods implemented (delegated to Path)
}
```

#### Examples

```rust
use sublime_standard_tools::filesystem::{
    AsyncFileSystem, FileSystemManager, AsyncFileSystemConfig, NodePathKind, PathExt, PathUtils
};
use std::path::Path;
use std::time::Duration;

// Configuration-aware filesystem operations
let config = AsyncFileSystemConfig {
    buffer_size: 8192,
    max_concurrent_operations: 10,
    operation_timeout: Duration::from_secs(30),
    retry_config: Some(RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(5),
        backoff_multiplier: 2.0,
    }),
    ignore_patterns: vec![".git".to_string(), "node_modules".to_string()],
};

let fs = FileSystemManager::new_with_config(config);

// Or load with project configuration
let fs = FileSystemManager::new_with_project_config(Path::new(".")).await?;

// Basic file operations
if fs.exists(Path::new("package.json")).await? {
    let content = fs.read_to_string(Path::new("package.json")).await?;
    println!("Package.json content: {}", content);
}

// Write file
fs.write_string(Path::new("output.txt"), "Hello, world!").await?;

// Directory operations
let entries = fs.read_dir(Path::new(".")).await?;
for entry in entries {
    if fs.is_dir(&entry).await? {
        println!("Directory: {}", entry.display());
    } else {
        println!("File: {}", entry.display());
    }
}

// Path extensions
let path = Path::new("src/components/Button.js");
let normalized = path.normalize();

if path.is_node_project() {
    println!("Path is within a Node.js project");
    
    if let Some(package_json) = path.find_package_json() {
        println!("Package.json found at: {}", package_json.display());
    }
}

// Node.js specific paths and detection
let current_path = Path::new(".");
match current_path.node_path_kind() {
    NodePathKind::ProjectRoot => println!("This is a project root"),
    NodePathKind::PackageDirectory => println!("This is a package directory"),
    NodePathKind::SourceDirectory => println!("This is a source directory"),
    other => println!("Path kind: {:?}", other),
}

// Path utilities
if let Some(project_root) = PathUtils::find_project_root(Path::new(".")) {
    println!("Project root: {}", project_root.display());
}

let relative = PathUtils::make_relative(
    Path::new("/home/user/project/src/main.js"), 
    Path::new("/home/user/project")
)?;
println!("Relative path: {}", relative.display());
```

## Error Module

The error module provides comprehensive error handling for all operations within the crate.

### Error Types

#### Main Error Types

```rust
/// General error type for the standard tools library.
#[derive(Debug, Clone)]
pub enum Error {
    /// Monorepo-related error.
    Monorepo(MonorepoError),
    
    /// Filesystem-related error.
    FileSystem(FileSystemError),
    
    /// Workspace-related error.
    Workspace(WorkspaceError),
    
    /// Command-related error.
    Command(CommandError),
    
    /// Configuration-related error.
    Config(ConfigError),
    
    /// General purpose errors with a custom message.
    Operation(String),
}

impl Error {
    /// Creates a new operational error.
    pub fn operation(message: impl Into<String>) -> Self;
    
    /// Returns a string representation of the error category.
    pub fn kind(&self) -> &'static str;
    
    /// Checks if this error is of a specific type.
    pub fn is_filesystem_error(&self) -> bool;
    pub fn is_command_error(&self) -> bool;
    pub fn is_monorepo_error(&self) -> bool;
    pub fn is_config_error(&self) -> bool;
    pub fn is_workspace_error(&self) -> bool;
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>;
}
```

#### ConfigError

```rust
/// Errors that can occur during configuration operations.
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// Configuration file not found.
    NotFound { 
        path: PathBuf 
    },
    
    /// Failed to parse configuration file.
    ParseError { 
        path: PathBuf, 
        source: String 
    },
    
    /// Invalid configuration values.
    InvalidConfig { 
        message: String 
    },
    
    /// Configuration validation failed.
    ValidationFailed { 
        errors: Vec<String> 
    },
    
    /// Unsupported configuration format.
    UnsupportedFormat { 
        format: String 
    },
    
    /// I/O error during configuration operations.
    Io { 
        source: String 
    },
    
    /// Environment variable parsing error.
    EnvVarError { 
        var_name: String, 
        reason: String 
    },
    
    /// Configuration merge conflict.
    MergeConflict { 
        field: String, 
        reason: String 
    },
}
```

#### FileSystemError

```rust
/// Errors that can occur during filesystem operations.
#[derive(Debug, Clone)]
pub enum FileSystemError {
    /// Path not found.
    NotFound { 
        path: PathBuf 
    },

    /// Permission denied for accessing the path.
    PermissionDenied { 
        path: PathBuf, 
        operation: String 
    },

    /// Generic I/O error during filesystem operation.
    Io { 
        path: PathBuf, 
        operation: String,
        source: String 
    },

    /// Attempted an operation requiring a directory on a file.
    NotADirectory { 
        path: PathBuf 
    },

    /// Attempted an operation requiring a file on a directory.
    NotAFile { 
        path: PathBuf 
    },

    /// Failed to decode UTF-8 content from a file.
    Utf8Decode { 
        path: PathBuf, 
        source: String 
    },

    /// Path validation failed.
    Validation { 
        path: PathBuf, 
        reason: String 
    },
    
    /// Operation timed out.
    Timeout { 
        path: PathBuf, 
        operation: String,
        duration: Duration 
    },
    
    /// Maximum retry attempts exceeded.
    MaxRetriesExceeded { 
        path: PathBuf, 
        operation: String,
        attempts: usize 
    },
}
```

#### CommandError

```rust
/// Errors that can occur during command execution.
#[derive(Debug, Clone)]
pub enum CommandError {
    /// The command failed to start.
    SpawnFailed { 
        command: String, 
        source: String 
    },

    /// The command execution process itself failed.
    ExecutionFailed { 
        command: String, 
        source: String 
    },

    /// The command executed but returned a non-zero exit code.
    NonZeroExitCode { 
        command: String, 
        exit_code: Option<i32>, 
        stderr: String 
    },

    /// The command timed out.
    Timeout { 
        command: String,
        duration: Duration 
    },

    /// The command was killed or interrupted.
    Killed { 
        command: String,
        reason: String 
    },

    /// Invalid configuration provided for the command.
    InvalidConfiguration { 
        field: String,
        reason: String 
    },

    /// Failed to capture stdout or stderr.
    CaptureFailed { 
        command: String,
        stream: String 
    },

    /// Error occurred while reading stdout or stderr stream.
    StreamReadError { 
        command: String,
        stream: String, 
        source: String 
    },

    /// Command queue operation failed.
    QueueError { 
        operation: String,
        reason: String 
    },

    /// Generic error during command processing.
    Generic(String),
}
```

#### MonorepoError

```rust
/// Errors that can occur during monorepo operations.
#[derive(Debug, Clone)]
pub enum MonorepoError {
    /// Failed to detect the monorepo type.
    DetectionFailed { 
        path: PathBuf,
        reason: String 
    },
    
    /// Failed to parse the monorepo configuration file.
    ConfigParsingFailed { 
        file_path: PathBuf,
        format: String,
        source: String 
    },
    
    /// Failed to read the monorepo configuration file.
    ConfigReadFailed { 
        file_path: PathBuf,
        source: FileSystemError 
    },
    
    /// Failed to write the monorepo configuration file.
    ConfigWriteFailed { 
        file_path: PathBuf,
        source: FileSystemError 
    },
    
    /// No package manager found for the monorepo.
    NoPackageManager { 
        monorepo_root: PathBuf 
    },
    
    /// Invalid workspace configuration.
    InvalidWorkspaceConfig { 
        reason: String 
    },
    
    /// Package not found in workspace.
    PackageNotFound { 
        package_name: String,
        workspace_root: PathBuf 
    },
    
    /// Circular dependency detected.
    CircularDependency { 
        packages: Vec<String> 
    },
    
    /// Workspace analysis failed.
    AnalysisFailed { 
        reason: String 
    },
}
```

#### WorkspaceError

```rust
/// Errors that can occur during workspace operations.
#[derive(Debug, Clone)]
pub enum WorkspaceError {
    /// Error parsing package.json format.
    InvalidPackageJson { 
        path: PathBuf,
        reason: String 
    },
    
    /// Error parsing workspaces pattern.
    InvalidWorkspacesPattern { 
        pattern: String,
        reason: String 
    },
    
    /// Error parsing pnpm workspace configuration.
    InvalidPnpmWorkspace { 
        config_path: PathBuf,
        reason: String 
    },
    
    /// Package not found in workspace.
    PackageNotFound { 
        package_name: String 
    },
    
    /// Workspace not found.
    WorkspaceNotFound { 
        workspace_path: PathBuf 
    },
    
    /// Workspace configuration is missing.
    WorkspaceConfigMissing { 
        expected_path: PathBuf 
    },
    
    /// Dependency resolution failed.
    DependencyResolution { 
        package_name: String,
        reason: String 
    },
    
    /// Workspace validation failed.
    ValidationFailed { 
        errors: Vec<String> 
    },
}
```

### Result Types

```rust
/// Convenient type aliases for Results with domain-specific errors
pub type FileSystemResult<T> = std::result::Result<T, FileSystemError>;
pub type MonorepoResult<T> = std::result::Result<T, MonorepoError>;
pub type WorkspaceResult<T> = std::result::Result<T, WorkspaceError>;
pub type CommandResult<T> = std::result::Result<T, CommandError>;
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

/// General result type for operations that may return various error types
pub type Result<T> = std::result::Result<T, Error>;
```

### Error Recovery

#### ErrorRecoveryManager

```rust
/// Manages error recovery strategies and provides context-aware error handling.
#[derive(Debug)]
pub struct ErrorRecoveryManager {
    // Private fields
}

impl ErrorRecoveryManager {
    /// Creates a new ErrorRecoveryManager with default strategies.
    pub fn new() -> Self;
    
    /// Adds a recovery strategy for a specific error pattern.
    pub fn add_strategy(&mut self, name: &str, strategy: RecoveryStrategy);
    
    /// Removes a recovery strategy by name.
    pub fn remove_strategy(&mut self, name: &str) -> Option<RecoveryStrategy>;
    
    /// Attempts to recover from an error using registered strategies.
    pub async fn recover(
        &mut self, 
        context: &str, 
        error: &Error, 
        log_level: LogLevel
    ) -> RecoveryResult;
    
    /// Logs an error with appropriate level and context.
    pub fn log_error(&self, error: &Error, context: &str, level: LogLevel);
    
    /// Returns statistics about recovery attempts.
    pub fn stats(&self) -> RecoveryStats;
    
    /// Resets all statistics.
    pub fn reset_stats(&mut self);
}
```

#### RecoveryStrategy

```rust
/// Defines different recovery strategies for handling errors.
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation with exponential backoff.
    Retry {
        max_attempts: usize,
        delay: Duration,
    },
    
    /// Use a fallback value or operation.
    Fallback {
        alternative: String,
    },
    
    /// Ignore the error and continue.
    Ignore,
    
    /// Log the error and continue.
    LogAndContinue {
        log_level: LogLevel,
    },
    
    /// Custom recovery function.
    Custom {
        name: String,
        handler: String, // Placeholder for function pointer
    },
}
```

#### RecoveryResult

```rust
/// Result of an error recovery attempt.
#[derive(Debug, Clone)]
pub enum RecoveryResult {
    /// Recovery was successful.
    Recovered,
    
    /// Recovery failed with an error.
    Failed(String),
    
    /// No recovery strategy was available.
    NoStrategy,
}
```

#### LogLevel

```rust
/// Logging levels for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Error level - critical issues that need immediate attention.
    Error,
    /// Warning level - potential issues that should be noted.
    Warn,
    /// Info level - general information messages.
    Info,
    /// Debug level - detailed debugging information.
    Debug,
    /// Trace level - very detailed tracing information.
    Trace,
}
```

#### RecoveryStats

```rust
/// Statistics about error recovery operations.
#[derive(Debug, Clone)]
pub struct RecoveryStats {
    /// Total number of recovery attempts.
    pub total_attempts: usize,
    
    /// Number of successful recoveries.
    pub successful_recoveries: usize,
    
    /// Number of failed recoveries.
    pub failed_recoveries: usize,
    
    /// Number of errors with no available strategy.
    pub no_strategy_available: usize,
    
    /// Recovery attempts by error type.
    pub attempts_by_error_type: std::collections::HashMap<String, usize>,
}
```

#### ErrorContext Trait

```rust
/// Provides additional context for errors.
pub trait ErrorContext {
    /// Adds context information to an error.
    fn with_context(self, context: String) -> Error;
    
    /// Adds formatted context information to an error.
    fn with_context_f(self, f: impl FnOnce() -> String) -> Error;
}

impl<T, E> ErrorContext for std::result::Result<T, E> 
where 
    E: Into<Error>
{
    fn with_context(self, context: String) -> Error {
        match self {
            Ok(_) => unreachable!("ErrorContext called on Ok result"),
            Err(e) => {
                let mut error = e.into();
                // Add context to error (implementation detail)
                error
            }
        }
    }
    
    fn with_context_f(self, f: impl FnOnce() -> String) -> Error {
        self.with_context(f())
    }
}
```

#### Examples

```rust
use sublime_standard_tools::error::{
    Error, ErrorRecoveryManager, RecoveryStrategy, RecoveryResult, LogLevel,
    FileSystemError, CommandError, MonorepoError, ErrorContext
};
use std::time::Duration;

// Error handling with pattern matching
fn handle_operation_error(error: Error) {
    match error {
        Error::FileSystem(FileSystemError::NotFound { path }) => {
            eprintln!("File not found: {}", path.display());
        }
        Error::Command(CommandError::Timeout { command, duration }) => {
            eprintln!("Command '{}' timed out after {:?}", command, duration);
        }
        Error::Monorepo(MonorepoError::NoPackageManager { monorepo_root }) => {
            eprintln!("No package manager found in monorepo at {}", monorepo_root.display());
        }
        Error::Config(config_err) => {
            eprintln!("Configuration error: {}", config_err);
        }
        Error::Operation(msg) => {
            eprintln!("Operation error: {}", msg);
        }
        _ => {
            eprintln!("Unknown error: {}", error);
        }
    }
}

// Error recovery manager setup
let mut recovery_manager = ErrorRecoveryManager::new();

// Add recovery strategies
recovery_manager.add_strategy(
    "file_not_found",
    RecoveryStrategy::Retry {
        max_attempts: 3,
        delay: Duration::from_millis(100),
    },
);

recovery_manager.add_strategy(
    "command_timeout",
    RecoveryStrategy::Fallback {
        alternative: "Use default timeout".to_string(),
    },
);

recovery_manager.add_strategy(
    "config_error",
    RecoveryStrategy::LogAndContinue {
        log_level: LogLevel::Warn,
    },
);

// Use error context for better error messages
async fn operation_with_context() -> Result<(), Error> {
    some_filesystem_operation()
        .await
        .map_err(Error::FileSystem)
        .with_context("Failed to read project configuration".to_string())?;
    
    Ok(())
}

// Recovery attempt
async fn handle_with_recovery(error: Error) {
    match recovery_manager.recover("project_analysis", &error, LogLevel::Error).await {
        RecoveryResult::Recovered => {
            println!("Successfully recovered from error");
        }
        RecoveryResult::Failed(reason) => {
            eprintln!("Recovery failed: {}", reason);
        }
        RecoveryResult::NoStrategy => {
            eprintln!("No recovery strategy available for error: {}", error);
        }
    }
}

// Get recovery statistics
let stats = recovery_manager.stats();
println!("Recovery Statistics:");
println!("  Total attempts: {}", stats.total_attempts);
println!("  Successful: {}", stats.successful_recoveries);
println!("  Failed: {}", stats.failed_recoveries);
println!("  No strategy: {}", stats.no_strategy_available);
```

---

This comprehensive API specification reflects the current architectural approach of the `sublime_standard_tools` crate with:

- **Clean separation of concerns**: Each module has a clearly defined responsibility
- **Unified project handling**: Consistent APIs for both simple and monorepo projects  
- **Async-first design**: All I/O operations use async/await for optimal performance
- **Robust configuration management**: Flexible, multi-source configuration system
- **Comprehensive error handling**: Structured errors with recovery strategies
- **Cross-platform support**: Full compatibility with Windows, macOS, and Linux
- **Type safety**: Strong typing prevents common errors and improves API usability
- **Performance optimization**: Designed for large repositories and complex workflows

The crate provides a consistent, well-documented interface for working with Node.js projects from Rust applications, making it easy to build robust tooling and automation systems.