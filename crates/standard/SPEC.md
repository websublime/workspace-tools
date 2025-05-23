# Sublime Standard Tools API Specification

This document provides a comprehensive specification of the public API for the `sublime_standard_tools` crate, a robust toolkit for working with Node.js projects from Rust applications.

## Table of Contents

- [Overview](#overview)
- [Command Module](#command-module)
  - [Command Execution](#command-execution)
  - [Command Builder](#command-builder)
  - [Command Output](#command-output)
  - [Command Queue](#command-queue)
  - [Command Stream](#command-stream)
- [Error Module](#error-module)
  - [Error Types](#error-types)
  - [Result Types](#result-types)
- [Filesystem Module](#filesystem-module)
  - [FileSystem Trait](#filesystem-trait)
  - [FileSystemManager](#filesystemmanager)
  - [Path Extensions](#path-extensions)
  - [Node Path Utilities](#node-path-utilities)
- [Monorepo Module](#monorepo-module)
  - [Monorepo Detection](#monorepo-detection)
  - [Monorepo Descriptor](#monorepo-descriptor)
  - [Package Manager](#package-manager)
  - [Project Management](#project-management)
  - [Configuration Management](#configuration-management)

## Overview

The `sublime_standard_tools` crate provides a comprehensive set of utilities for working with Node.js projects from Rust applications. It handles project structure detection, command execution, environment management, and various other tasks required when interacting with Node.js ecosystems.

```rust
// Version information
const VERSION: &str = "..."; // Returns the current crate version

// Get version
fn version() -> &'static str;
```

## Command Module

The command module provides utilities for executing shell commands with various options, including streaming output, queueing commands with priorities, and handling timeouts.

### Command Execution

#### Types

```rust
// Core command types
pub struct Command {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) env: HashMap<String, String>,
    pub(crate) current_dir: Option<PathBuf>,
    pub(crate) timeout: Option<Duration>,
}

pub struct CommandBuilder {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) env: HashMap<String, String>,
    pub(crate) current_dir: Option<PathBuf>,
    pub(crate) timeout: Option<Duration>,
}

pub struct CommandOutput {
    pub(crate) status: i32,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) duration: Duration,
}

pub enum StreamOutput {
    Stdout(String),
    Stderr(String),
    End,
}

pub struct StreamConfig {
    pub(crate) buffer_size: usize,
    pub(crate) read_timeout: Duration,
}

pub struct CommandStream {
    pub(crate) rx: mpsc::Receiver<StreamOutput>,
    pub(crate) cancel: Arc<AtomicBool>,
}
```

#### Traits

```rust
#[async_trait::async_trait]
pub trait CommandExecutor: Send + Sync {
    async fn execute(&self, command: Command) -> Result<CommandOutput>;
    async fn execute_stream(
        &self,
        command: Command,
        stream_config: StreamConfig,
    ) -> Result<(CommandStream, tokio::process::Child)>;
}
```

#### Implementations

```rust
// Default implementation
pub struct DefaultCommandExecutor;

impl DefaultCommandExecutor {
    pub fn new() -> Self;
}
```

### Command Builder

```rust
impl CommandBuilder {
    /// Creates a new CommandBuilder instance.
    pub fn new(program: impl Into<String>) -> Self;
    
    /// Builds the final Command instance.
    pub fn build(self) -> Command;
    
    /// Adds an argument to the command.
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self;
    
    /// Sets the command timeout.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self;
    
    /// Sets the working directory for the command.
    #[must_use]
    pub fn current_dir(mut self, path: impl AsRef<Path>) -> Self;
}
```

### Command Output

```rust
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

### Command Queue

#### Types

```rust
pub enum CommandPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

pub enum CommandStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

pub struct CommandQueueResult {
    pub id: String,
    pub status: CommandStatus,
    pub output: Option<CommandOutput>,
    pub error: Option<String>,
}

pub struct CommandQueueConfig {
    pub max_concurrent_commands: usize,
    pub rate_limit: Option<Duration>,
    pub default_timeout: Duration,
    pub shutdown_timeout: Duration,
}

pub struct CommandQueue {
    // Not directly accessible, use methods
}
```

#### Methods

```rust
impl CommandQueue {
    pub fn new() -> Self;
    pub fn with_config(config: CommandQueueConfig) -> Self;
    pub fn with_executor<E: CommandExecutor + 'static>(executor: E) -> Self;
    pub fn start(mut self) -> Result<Self>;
    pub async fn enqueue(&self, command: Command, priority: CommandPriority) -> Result<String>;
    pub async fn enqueue_batch(&self, commands: Vec<(Command, CommandPriority)>) -> Result<Vec<String>>;
    pub fn get_status(&self, id: &str) -> Option<CommandStatus>;
    pub fn get_result(&self, id: &str) -> Option<CommandQueueResult>;
    pub async fn wait_for_command(&self, id: &str, timeout: Duration) -> Result<CommandQueueResult>;
    pub async fn wait_for_completion(&self) -> Result<()>;
    pub async fn shutdown(&mut self) -> Result<()>;
}

impl CommandStatus {
    pub fn is_completed(self) -> bool;
    pub fn is_successful(self) -> bool;
}

impl CommandQueueResult {
    pub fn success(id: String, output: CommandOutput) -> Self;
    pub fn failure(id: String, error: String) -> Self;
    pub fn cancelled(id: String) -> Self;
    pub fn is_successful(&self) -> bool;
}
```

### Command Stream

```rust
impl Default for StreamConfig {
    fn default() -> Self {
        Self { buffer_size: 1024, read_timeout: Duration::from_secs(1) }
    }
}

impl StreamConfig {
    /// Creates a new StreamConfig with custom settings
    pub fn new(buffer_size: usize, read_timeout: Duration) -> Self;
}

impl CommandStream {
    /// Receives the next output line with timeout
    pub async fn next_timeout(
        &mut self,
        timeout_duration: Duration,
    ) -> Result<Option<StreamOutput>>;
    
    /// Cancels the stream
    pub fn cancel(&self);
}
```

#### Example Usage

```rust
use sublime_standard_tools::command::{CommandBuilder, DefaultCommandExecutor, CommandExecutor, StreamConfig};
use std::time::Duration;

// Basic command execution
async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let executor = DefaultCommandExecutor::new();
    
    // Create and execute a command
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
    
    let status = child.wait().await?;
    println!("Process exited with: {}", status);
    
    Ok(())
}
```

## Error Module

The error module provides comprehensive error types for various operations within the crate.

### Error Types

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

    /// Path validation failed (e.g., contains '..', absolute path, symlink).
    #[error("Path validation failed for '{path}': {reason}")]
    Validation { path: PathBuf, reason: String },
}

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
    #[error("Invalid workspaces pattern: {0}")]
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

/// Errors that can occur during command execution.
#[derive(ThisError, Debug)]
pub enum CommandError {
    /// The command failed to start (e.g., not found).
    #[error("Failed to spawn command '{cmd}': {source}")]
    SpawnFailed { cmd: String, source: io::Error },

    /// The command execution process itself failed.
    #[error("Command execution failed for '{cmd}': {source:?}")]
    ExecutionFailed { cmd: String, source: Option<io::Error> },

    /// The command executed but returned a non-zero exit code.
    #[error("Command '{cmd}' failed with exit code {code:?}. Stderr: {stderr}")]
    NonZeroExitCode { cmd: String, code: Option<i32>, stderr: String },

    /// The command timed out after the specified duration.
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

### Result Types

```rust
pub type FileSystemResult<T> = CoreResult<T, FileSystemError>;
pub type MonorepoResult<T> = CoreResult<T, MonorepoError>;
pub type WorkspaceResult<T> = CoreResult<T, WorkspaceError>;
pub type CommandResult<T> = CoreResult<T, CommandError>;
pub type Result<T> = CoreResult<T, Error>;
```

## Filesystem Module

The filesystem module provides abstractions for interacting with the filesystem in a safe and consistent manner.

### FileSystem Trait

```rust
pub trait FileSystem: Send + Sync {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>>;
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()>;
    fn read_file_string(&self, path: &Path) -> Result<String>;
    fn write_file_string(&self, path: &Path, contents: &str) -> Result<()>;
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    fn remove(&self, path: &Path) -> Result<()>;
    fn exists(&self, path: &Path) -> bool;
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
    fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
}
```

### FileSystemManager

```rust
pub struct FileSystemManager {}

impl FileSystemManager {
    /// Creates a new FileSystemManager instance.
    pub fn new() -> Self;
    
    /// Validates that a path exists, returning an error if it doesn't.
    fn validate_path(&self, path: &Path) -> Result<&Self>;
}
```

### Path Extensions

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
    // Implemented methods
}
```

### Node Path Utilities

```rust
pub struct PathUtils;

impl PathUtils {
    /// Finds the root directory of a Node.js project by traversing upward
    /// from the given starting directory until it finds a package.json file.
    pub fn find_project_root(start: &Path) -> Option<PathBuf>;
    
    /// Gets the current working directory as a PathBuf.
    pub fn current_dir() -> FileSystemResult<PathBuf>;
    
    /// Makes a path relative to a base path.
    pub fn make_relative(path: &Path, base: &Path) -> FileSystemResult<PathBuf>;
}
```

#### Example Usage

```rust
use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager, NodePathKind, PathExt, PathUtils};
use std::path::Path;

fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a filesystem manager
    let fs = FileSystemManager::new();
    
    // Read and write files
    let content = fs.read_file_string(Path::new("package.json"))?;
    fs.write_file_string(Path::new("output.txt"), "Hello, world!")?;
    
    // Directory operations
    let entries = fs.read_dir(Path::new("."))?;
    for entry in entries {
        println!("Found: {}", entry.display());
    }
    
    // Use path extensions
    let path = Path::new("src/components/Button.js");
    let normalized = path.normalize();
    
    // Check if path is in a Node.js project
    if path.is_in_project() {
        println!("Path is within a Node.js project");
        
        if let Some(relative) = path.relative_to_project() {
            println!("Path relative to project: {}", relative.display());
        }
    }
    
    // Work with Node.js specific paths
    let node_modules = Path::new(".").node_path(NodePathKind::NodeModules);
    println!("Node modules path: {}", node_modules.display());
    
    // Find project root
    if let Some(root) = PathUtils::find_project_root(Path::new(".")) {
        println!("Project root: {}", root.display());
    }
    
    Ok(())
}
```

## Monorepo Module

The monorepo module provides tools for detecting and working with monorepo structures, including support for different package managers and workspace configurations.

### Monorepo Detection

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

/// Configuration structure for PNPM workspaces.
#[derive(Debug, Clone, Deserialize)]
pub struct PnpmWorkspaceConfig {
    pub packages: Vec<String>,
}
```

```rust
pub struct MonorepoDetector<F: FileSystem = FileSystemManager> {
    pub(crate) fs: F,
}

impl MonorepoDetector<FileSystemManager> {
    /// Creates a new MonorepoDetector with the default filesystem implementation.
    pub fn new() -> Self;
}

impl<F: FileSystem> MonorepoDetector<F> {
    /// Creates a new MonorepoDetector with a custom filesystem implementation.
    pub fn with_filesystem(fs: F) -> Self;
    
    /// Checks if a path is the root of a monorepo by examining lock files.
    pub fn is_monorepo_root(&self, path: impl AsRef<Path>) -> Result<Option<MonorepoKind>>;
    
    /// Finds the nearest monorepo root by walking up from the given path.
    pub fn find_monorepo_root(
        &self,
        start_path: impl AsRef<Path>,
    ) -> Result<Option<(PathBuf, MonorepoKind)>>;
    
    /// Detects and analyzes a monorepo at the given path.
    pub fn detect_monorepo(&self, path: impl AsRef<Path>) -> Result<MonorepoDescriptor>;
    
    /// Checks if a directory contains multiple packages based on common patterns.
    pub fn has_multiple_packages(&self, path: &Path) -> bool;
}
```

### Monorepo Descriptor

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

/// Describes a complete monorepo structure.
#[derive(Debug, Clone)]
pub struct MonorepoDescriptor {
    // Internal fields
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
```

### Package Manager

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
    /// Returns the name of the lock file used by this package manager.
    pub fn lock_file(self) -> &'static str;
    
    /// Returns the command used to invoke this package manager.
    pub fn command(self) -> &'static str;
}

/// Represents a package manager detected in a Node.js project.
#[derive(Debug, Clone)]
pub struct PackageManager {
    // Internal fields
}

impl PackageManager {
    /// Creates a new PackageManager instance with the specified kind and root directory.
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

### Project Management

```rust
/// Configuration options for project detection and validation.
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    // Internal fields
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

/// Represents a detected Node.js project.
#[derive(Debug)]
pub struct Project {
    // Internal fields
}

impl Project {
    /// Creates a new Project instance with the specified root and configuration.
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

/// Manages Node.js project detection and validation.
#[derive(Debug)]
pub struct ProjectManager<F: FileSystem = FileSystemManager> {
    // Internal fields
}

impl ProjectManager<FileSystemManager> {
    /// Creates a new ProjectManager instance with the default filesystem implementation.
    pub fn new() -> Self;
}

impl<F: FileSystem> ProjectManager<F> {
    /// Creates a new ProjectManager with a custom filesystem implementation.
    pub fn with_filesystem(fs: F) -> Self;
    
    /// Detects and analyzes a Node.js project at the given path.
    pub fn detect_project(
        &self,
        path: impl AsRef<Path>,
        config: &ProjectConfig,
    ) -> Result<Project>;
    
    /// Validates a Node.js project structure and updates its validation status.
    pub fn validate_project(&self, project: &mut Project) -> Result<()>;
}
```

### Configuration Management

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

/// Configuration file formats.
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
    pub fn is_string(&self) -> bool;
    pub fn is_integer(&self) -> bool;
    pub fn is_float(&self) -> bool;
    pub fn is_boolean(&self) -> bool;
    pub fn is_array(&self) -> bool;
    pub fn is_map(&self) -> bool;
    pub fn is_null(&self) -> bool;
    pub fn as_string(&self) -> Option<&str>;
    pub fn as_integer(&self) -> Option<i64>;
    pub fn as_float(&self) -> Option<f64>;
    pub fn as_boolean(&self) -> Option<bool>;
    pub fn as_array(&self) -> Option<&[ConfigValue]>;
    pub fn as_map(&self) -> Option<&HashMap<String, ConfigValue>>;
}

/// Manages configuration across different scopes and file formats.
#[derive(Debug, Clone)]
pub struct ConfigManager {
    // Internal fields
}

impl ConfigManager {
    pub fn new() -> Self;
    pub fn set_path(&mut self, scope: ConfigScope, path: impl Into<PathBuf>);
    pub fn get_path(&self, scope: ConfigScope) -> Option<&PathBuf>;
    pub fn load_all(&self) -> Result<()>;
    pub fn load_from_file(&self, path: &Path) -> Result<()>;
    pub fn save_all(&self) -> Result<()>;
    pub fn save_to_file(&self, path: &Path) -> Result<()>;
    pub fn get(&self, key: &str) -> Option<ConfigValue>;
    pub fn set(&self, key: &str, value: ConfigValue);
    pub fn remove(&self, key: &str) -> Option<ConfigValue>;
}
```

#### Example Usage

```rust
use sublime_standard_tools::monorepo::{MonorepoDetector, PackageManagerKind, ProjectConfig, ProjectManager};
use std::path::Path;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Detect a monorepo
    let detector = MonorepoDetector::new();
    
    if let Some(kind) = detector.is_monorepo_root(".")? {
        println!("This directory is a {} monorepo", kind.name());
    }
    
    // Find the nearest monorepo root
    if let Some((root, kind)) = detector.find_monorepo_root(".")? {
        println!("Found {} monorepo at {}", kind.name(), root.display());
        
        // Analyze the monorepo
        let monorepo = detector.detect_monorepo(&root)?;
        
        println!("Monorepo contains {} packages:", monorepo.packages().len());
        for package in monorepo.packages() {
            println!("- {} v{} at {}", package.name, package.version, package.location.display());
            
            // Print dependencies
            let deps = monorepo.find_dependencies_by_name(&package.name);
            if !deps.is_empty() {
                println!("  Dependencies:");
                for dep in deps {
                    println!("  - {}", dep.name);
                }
            }
        }
    }
    
    // Project management
    let project_manager = ProjectManager::new();
    let config = ProjectConfig::new()
        .with_detect_package_manager(true)
        .with_validate_structure(true);
    
    let project = project_manager.detect_project(".", &config)?;
    println!("Project root: {}", project.root().display());
    
    if let Some(package_manager) = project.package_manager() {
        println!("Using package manager: {}", package_manager.kind().command());
    }
    
    match project.validation_status() {
        sublime_standard_tools::monorepo::ProjectValidationStatus::Valid => {
            println!("Project structure is valid");
        }
        _ => println!("Project structure has issues"),
    }
    
    Ok(())
}
```

This API specification provides a comprehensive overview of the `sublime_standard_tools` crate functionality, organized by modules and including all public types, methods, and examples.

