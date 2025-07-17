# Sublime Standard Tools API Specification

This document provides a comprehensive specification of the public API for the `sublime_standard_tools` crate, a robust toolkit for working with Node.js projects from Rust applications.

## Table of Contents

- [Overview](#overview)
- [Node Module](#node-module)
  - [Repository Types](#repository-types)
  - [Package Manager Abstractions](#package-manager-abstractions)
  - [Repository Information](#repository-information)
- [Project Module](#project-module)
  - [Project Types](#project-types)
  - [Project Detection](#project-detection)
  - [Project Management](#project-management)
  - [Configuration Management](#configuration-management)
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
  - [Filesystem Abstraction](#filesystem-abstraction)
  - [Path Utilities](#path-utilities)
  - [Node.js Path Extensions](#nodejs-path-extensions)
- [Error Module](#error-module)
  - [Error Types](#error-types)
  - [Result Types](#result-types)

## Overview

The `sublime_standard_tools` crate provides a comprehensive set of utilities for working with Node.js projects from Rust applications. It follows a clean architectural approach with clear separation of concerns:

- **Node Module**: Generic Node.js concepts (repositories, package managers)
- **Project Module**: Unified project detection and management
- **Monorepo Module**: Monorepo-specific functionality and workspace management
- **Command Module**: Robust command execution framework
- **Filesystem Module**: Safe filesystem operations and path utilities
- **Error Module**: Comprehensive error handling

```rust
// Version information
const VERSION: &str = "..."; // Returns the current crate version

// Get version
fn version() -> &'static str;
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

#### Examples

```rust
use sublime_standard_tools::node::RepoKind;
use sublime_standard_tools::monorepo::MonorepoKind;

// Simple repository
let simple_repo = RepoKind::Simple;
assert_eq!(simple_repo.name(), "simple");
assert!(!simple_repo.is_monorepo());

// Monorepo repository
let yarn_mono = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);
assert_eq!(yarn_mono.name(), "yarn monorepo");
assert!(yarn_mono.is_monorepo());
assert_eq!(yarn_mono.monorepo_kind(), Some(&MonorepoKind::YarnWorkspaces));
```

### Package Manager Abstractions

#### PackageManagerKind

```rust
/// Represents the type of package manager used in a Node.js project.
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

impl PackageManagerKind {
    /// Returns the command name for this package manager.
    pub fn command(self) -> &'static str;
    
    /// Returns the lock file name for this package manager.
    pub fn lock_file(self) -> &'static str;
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
    
    /// Detects which package manager is being used in the specified path.
    pub fn detect(path: impl AsRef<Path>) -> Result<Self>;
    
    /// Returns the kind of package manager.
    pub fn kind(&self) -> PackageManagerKind;
    
    /// Returns the root directory path of the package manager.
    pub fn root(&self) -> &Path;
    
    /// Returns the full path to the lock file for this package manager.
    pub fn lock_file_path(&self) -> PathBuf;
}
```

#### Examples

```rust
use sublime_standard_tools::node::{PackageManager, PackageManagerKind};
use std::path::Path;

// Package manager detection
let manager = PackageManager::detect(Path::new("."))?;
println!("Using package manager: {}", manager.kind().command());
println!("Lock file: {}", manager.lock_file_path().display());

// Package manager characteristics
assert_eq!(PackageManagerKind::Npm.command(), "npm");
assert_eq!(PackageManagerKind::Npm.lock_file(), "package-lock.json");
assert_eq!(PackageManagerKind::Yarn.command(), "yarn");
assert_eq!(PackageManagerKind::Yarn.lock_file(), "yarn.lock");
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

#### ProjectDescriptor

```rust
/// Represents different types of Node.js projects with their specific data.
#[derive(Debug)]
pub enum ProjectDescriptor {
    /// A simple Node.js project
    Simple(Box<SimpleProject>),
    /// A monorepo project
    Monorepo(Box<MonorepoDescriptor>),
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

#### ProjectDetector

```rust
/// Provides unified detection and analysis of Node.js projects.
pub struct ProjectDetector<F: FileSystem = FileSystemManager> {
    // Private fields
}

impl ProjectDetector<FileSystemManager> {
    /// Creates a new ProjectDetector with the default filesystem.
    pub fn new() -> Self;
}

impl<F: FileSystem + Clone> ProjectDetector<F> {
    /// Creates a new ProjectDetector with a custom filesystem.
    pub fn with_filesystem(fs: F) -> Self;
    
    /// Detects and analyzes a project at the given path.
    pub fn detect(&self, path: impl AsRef<Path>, config: &ProjectConfig) -> Result<ProjectDescriptor>;
    
    /// Detects only the project kind without full analysis.
    pub fn detect_kind(&self, path: impl AsRef<Path>, config: &ProjectConfig) -> Result<ProjectKind>;
    
    /// Checks if the path contains a valid Node.js project.
    pub fn is_valid_project(&self, path: impl AsRef<Path>) -> bool;
}
```

#### ProjectConfig

```rust
/// Configuration options for project detection and validation.
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    // Private fields
}

impl ProjectConfig {
    /// Creates a new ProjectConfig with default values.
    pub fn new() -> Self;
    
    /// Sets the root directory for project detection.
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self;
    
    /// Sets whether to detect package managers.
    pub fn with_detect_package_manager(mut self, detect: bool) -> Self;
    
    /// Sets whether to validate project structure.
    pub fn with_validate_structure(mut self, validate: bool) -> Self;
    
    /// Sets whether to detect monorepo structures.
    pub fn with_detect_monorepo(mut self, detect: bool) -> Self;
}
```

### Project Management

#### ProjectManager

```rust
/// Manages Node.js project detection and validation.
pub struct ProjectManager<F: FileSystem = FileSystemManager> {
    // Private fields
}

impl ProjectManager<FileSystemManager> {
    /// Creates a new ProjectManager with the default filesystem.
    pub fn new() -> Self;
}

impl<F: FileSystem + Clone> ProjectManager<F> {
    /// Creates a new ProjectManager with a custom filesystem.
    pub fn with_filesystem(fs: F) -> Self;
    
    /// Creates a project descriptor from a path.
    pub fn create_project(&self, path: impl AsRef<Path>, config: &ProjectConfig) -> Result<ProjectDescriptor>;
    
    /// Validates a project and updates its validation status.
    pub fn validate_project(&self, project: &mut ProjectDescriptor) -> Result<()>;
}
```

#### SimpleProject

```rust
/// Represents a simple Node.js project (single package.json).
#[derive(Debug)]
pub struct SimpleProject {
    // Private fields
}

impl SimpleProject {
    /// Creates a new SimpleProject instance.
    pub fn new(root: impl Into<PathBuf>, config: ProjectConfig) -> Self;
    
    /// Returns the root directory of the project.
    pub fn root(&self) -> &Path;
    
    /// Returns the package manager for the project, if detected.
    pub fn package_manager(&self) -> Option<&PackageManager>;
    
    /// Returns the validation status of the project.
    pub fn validation_status(&self) -> &ProjectValidationStatus;
    
    /// Returns the parsed package.json for the project, if available.
    pub fn package_json(&self) -> Option<&PackageJson>;
}

impl ProjectInfo for SimpleProject {
    fn root(&self) -> &Path;
    fn package_manager(&self) -> Option<&PackageManager>;
    fn package_json(&self) -> Option<&PackageJson>;
    fn validation_status(&self) -> &ProjectValidationStatus;
    fn kind(&self) -> ProjectKind;
}
```

### Configuration Management

#### ConfigManager

```rust
/// Manages configuration across different scopes and file formats.
#[derive(Debug, Clone)]
pub struct ConfigManager {
    // Private fields
}

impl ConfigManager {
    /// Creates a new ConfigManager.
    pub fn new() -> Self;
    
    /// Sets the path for a configuration scope.
    pub fn set_path(&mut self, scope: ConfigScope, path: impl Into<PathBuf>);
    
    /// Gets the path for a configuration scope.
    pub fn get_path(&self, scope: ConfigScope) -> Option<&PathBuf>;
    
    /// Loads all configuration files.
    pub fn load_all(&self) -> Result<()>;
    
    /// Loads configuration from a specific file.
    pub fn load_from_file(&self, path: &Path) -> Result<()>;
    
    /// Saves all configuration changes.
    pub fn save_all(&self) -> Result<()>;
    
    /// Saves configuration to a specific file.
    pub fn save_to_file(&self, path: &Path) -> Result<()>;
    
    /// Gets a configuration value.
    pub fn get(&self, key: &str) -> Option<ConfigValue>;
    
    /// Sets a configuration value.
    pub fn set(&self, key: &str, value: ConfigValue);
    
    /// Removes a configuration value.
    pub fn remove(&self, key: &str) -> Option<ConfigValue>;
}
```

#### ConfigScope

```rust
/// Configuration scope levels.
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
```

#### ConfigValue

```rust
/// A configuration value that can represent different data types.
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

impl ConfigValue {
    /// Type checking methods
    pub fn is_string(&self) -> bool;
    pub fn is_integer(&self) -> bool;
    pub fn is_float(&self) -> bool;
    pub fn is_boolean(&self) -> bool;
    pub fn is_array(&self) -> bool;
    pub fn is_map(&self) -> bool;
    pub fn is_null(&self) -> bool;
    
    /// Value extraction methods
    pub fn as_string(&self) -> Option<&str>;
    pub fn as_integer(&self) -> Option<i64>;
    pub fn as_float(&self) -> Option<f64>;
    pub fn as_boolean(&self) -> Option<bool>;
    pub fn as_array(&self) -> Option<&[ConfigValue]>;
    pub fn as_map(&self) -> Option<&HashMap<String, ConfigValue>>;
}
```

#### Examples

```rust
use sublime_standard_tools::project::{
    ProjectDetector, ProjectConfig, ProjectDescriptor, ProjectInfo,
    ConfigManager, ConfigScope, ConfigValue
};
use std::path::Path;

// Project detection
let detector = ProjectDetector::new();
let config = ProjectConfig::new()
    .with_detect_package_manager(true)
    .with_validate_structure(true);

let project = detector.detect(Path::new("."), &config)?;
match project {
    ProjectDescriptor::Simple(simple) => {
        println!("Found simple project at {}", simple.root().display());
        if let Some(pm) = simple.package_manager() {
            println!("Using package manager: {}", pm.kind().command());
        }
    }
    ProjectDescriptor::Monorepo(monorepo) => {
        println!("Found {} with {} packages", 
                 monorepo.kind().name(),
                 monorepo.packages().len());
    }
}

// Configuration management
let mut config_manager = ConfigManager::new();
config_manager.set("theme", ConfigValue::String("dark".to_string()));
config_manager.set("debug", ConfigValue::Boolean(true));

if let Some(theme) = config_manager.get("theme") {
    if let Some(theme_str) = theme.as_string() {
        println!("Current theme: {}", theme_str);
    }
}
```

## Monorepo Module

The monorepo module provides specialized functionality for detecting and managing monorepo structures across different package managers.

### Monorepo Types

#### MonorepoKind

```rust
/// Represents the type of monorepo system being used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonorepoKind {
    /// Npm workspaces monorepo
    NpmWorkSpace,
    /// Yarn Workspaces monorepo
    YarnWorkspaces,
    /// pnpm Workspaces monorepo
    PnpmWorkspaces,
    /// Bun workspaces monorepo
    BunWorkspaces,
    /// Deno workspaces monorepo
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
#[derive(Debug, Clone)]
pub struct MonorepoDescriptor {
    // Private fields
}

impl MonorepoDescriptor {
    /// Creates a new MonorepoDescriptor instance.
    pub fn new(kind: MonorepoKind, root: PathBuf, packages: Vec<WorkspacePackage>) -> Self;
    
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

#### MonorepoDetector

```rust
/// Detects and analyzes monorepo structures.
pub struct MonorepoDetector<F: FileSystem = FileSystemManager> {
    // Private fields
}

impl MonorepoDetector<FileSystemManager> {
    /// Creates a new MonorepoDetector with the default filesystem.
    pub fn new() -> Self;
}

impl<F: FileSystem> MonorepoDetector<F> {
    /// Creates a new MonorepoDetector with a custom filesystem.
    pub fn with_filesystem(fs: F) -> Self;
    
    /// Checks if a path is the root of a monorepo.
    pub fn is_monorepo_root(&self, path: impl AsRef<Path>) -> Result<Option<MonorepoKind>>;
    
    /// Finds the nearest monorepo root by walking up from the given path.
    pub fn find_monorepo_root(
        &self,
        start_path: impl AsRef<Path>,
    ) -> Result<Option<(PathBuf, MonorepoKind)>>;
    
    /// Detects and analyzes a monorepo at the given path.
    pub fn detect_monorepo(&self, path: impl AsRef<Path>) -> Result<MonorepoDescriptor>;
    
    /// Checks if a directory contains multiple packages.
    pub fn has_multiple_packages(&self, path: &Path) -> bool;
}
```

### Workspace Management

#### PnpmWorkspaceConfig

```rust
/// Configuration structure for PNPM workspaces.
#[derive(Debug, Clone, Deserialize)]
pub struct PnpmWorkspaceConfig {
    pub packages: Vec<String>,
}
```

#### Examples

```rust
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoKind};
use std::path::Path;

// Monorepo detection
let detector = MonorepoDetector::new();

if let Some(kind) = detector.is_monorepo_root(".")? {
    println!("This directory is a {} monorepo", kind.name());
    
    // Analyze the monorepo
    let monorepo = detector.detect_monorepo(".")?;
    
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
    
    // Generate dependency graph
    let graph = monorepo.get_dependency_graph();
    for (package, deps) in graph {
        println!("{} depends on {} packages", package, deps.len());
    }
}

// Find monorepo root
if let Some((root, kind)) = detector.find_monorepo_root(".")? {
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

#### Executor

```rust
/// Trait for executing commands.
#[async_trait::async_trait]
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

/// Default command executor implementation.
pub struct DefaultCommandExecutor;

impl DefaultCommandExecutor {
    /// Creates a new DefaultCommandExecutor.
    pub fn new() -> Self;
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

/// Represents output from a command stream.
pub enum StreamOutput {
    /// Standard output line
    Stdout(String),
    /// Standard error line
    Stderr(String),
    /// End of stream
    End,
}

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

/// Priority levels for commands.
pub enum CommandPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Status of a command in the queue.
pub enum CommandStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Result of a queued command.
pub struct CommandQueueResult {
    pub id: String,
    pub status: CommandStatus,
    pub output: Option<CommandOutput>,
    pub error: Option<String>,
}
```

#### Examples

```rust
use sublime_standard_tools::command::{
    CommandBuilder, DefaultCommandExecutor, Executor, 
    CommandQueue, CommandPriority, StreamConfig
};
use std::time::Duration;

// Basic command execution
let executor = DefaultCommandExecutor::new();
let cmd = CommandBuilder::new("echo")
    .arg("Hello world")
    .timeout(Duration::from_secs(5))
    .build();

let output = executor.execute(cmd).await?;
if output.success() {
    println!("Command output: {}", output.stdout());
} else {
    println!("Command failed with exit code {}: {}", output.status(), output.stderr());
}

// Stream command output
let stream_config = StreamConfig::default();
let cmd = CommandBuilder::new("ls")
    .arg("-la")
    .build();

let (mut stream, mut child) = executor.execute_stream(cmd, stream_config).await?;
while let Ok(Some(output)) = stream.next_timeout(Duration::from_secs(1)).await {
    match output {
        StreamOutput::Stdout(line) => println!("STDOUT: {}", line),
        StreamOutput::Stderr(line) => eprintln!("STDERR: {}", line),
        StreamOutput::End => break,
    }
}

// Command queue
let mut queue = CommandQueue::new().start()?;

let cmd1 = CommandBuilder::new("echo").arg("First").build();
let cmd2 = CommandBuilder::new("echo").arg("Second").build();

let id1 = queue.enqueue(cmd1, CommandPriority::High).await?;
let id2 = queue.enqueue(cmd2, CommandPriority::Normal).await?;

let result1 = queue.wait_for_command(&id1, Duration::from_secs(10)).await?;
println!("Command 1 result: {:?}", result1);

queue.shutdown().await?;
```

## Filesystem Module

The filesystem module provides safe abstractions for interacting with the filesystem and Node.js-specific path utilities.

### Filesystem Abstraction

#### FileSystem

```rust
/// Trait for filesystem operations.
pub trait FileSystem: Send + Sync {
    /// Reads a file and returns its contents as bytes.
    fn read_file(&self, path: &Path) -> Result<Vec<u8>>;
    
    /// Writes bytes to a file.
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()>;
    
    /// Reads a file and returns its contents as a string.
    fn read_file_string(&self, path: &Path) -> Result<String>;
    
    /// Writes a string to a file.
    fn write_file_string(&self, path: &Path, contents: &str) -> Result<()>;
    
    /// Creates a directory and all parent directories.
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    
    /// Removes a file or directory.
    fn remove(&self, path: &Path) -> Result<()>;
    
    /// Checks if a path exists.
    fn exists(&self, path: &Path) -> bool;
    
    /// Reads a directory and returns its entries.
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
    
    /// Walks a directory tree and returns all paths.
    fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
}
```

#### FileSystemManager

```rust
/// Default implementation of the FileSystem trait.
pub struct FileSystemManager;

impl FileSystemManager {
    /// Creates a new FileSystemManager.
    pub fn new() -> Self;
}

impl FileSystem for FileSystemManager {
    // All FileSystem trait methods implemented
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
    FileSystem, FileSystemManager, NodePathKind, PathExt, PathUtils
};
use std::path::Path;

// Filesystem operations
let fs = FileSystemManager::new();
let content = fs.read_file_string(Path::new("package.json"))?;
fs.write_file_string(Path::new("output.txt"), "Hello, world!")?;

// Directory operations
let entries = fs.read_dir(Path::new("."))?;
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

// Find project root
if let Some(root) = PathUtils::find_project_root(Path::new(".")) {
    println!("Project root: {}", root.display());
}
```

## Error Module

The error module provides comprehensive error handling for all operations within the crate.

### Error Types

#### Main Error Types

```rust
/// General error type for the standard tools library.
#[derive(ThisError, Debug)]
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
    
    /// General purpose errors with a custom message.
    #[error("Operation error: {0}")]
    Operation(String),
}

impl Error {
    /// Creates a new operational error.
    pub fn operation(message: impl Into<String>) -> Self;
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
pub type Result<T> = std::result::Result<T, Error>;
```

This comprehensive API specification reflects the new architectural approach with clean separation of concerns, unified project handling, and robust error management. The crate now provides a consistent, type-safe interface for working with Node.js projects from Rust applications.