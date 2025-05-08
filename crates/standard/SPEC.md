Sublime Standard Tools API Specification

This document provides a comprehensive specification of the public API for the `sublime_standard_tools` crate, a robust toolkit for working with Node.js projects from Rust applications.

## Table of Contents

- [Overview](#overview)
- [Command Module](#command-module)
  - [Command Execution](#command-execution)
  - [Command Queue](#command-queue)
  - [Command Stream](#command-stream)
- [Filesystem Module](#filesystem-module)
  - [FileSystem Trait](#filesystem-trait)
  - [FileSystemManager](#filesystemmanager)
  - [Path Extensions](#path-extensions)
- [Monorepo Module](#monorepo-module)
  - [Monorepo Detection](#monorepo-detection)
  - [Package Management](#package-management)
  - [Project Management](#project-management)
  - [Configuration Management](#configuration-management)
- [Error Module](#error-module)
  - [Error Types](#error-types)
  - [Result Types](#result-types)

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
    // Not directly accessible, use CommandBuilder
}

pub struct CommandBuilder {
    // Not directly accessible, use methods
}

pub struct CommandOutput {
    // Not directly accessible, use methods
}

pub enum StreamOutput {
    Stdout(String),
    Stderr(String),
    End,
}

pub struct StreamConfig {
    // Not directly accessible, use new() or default()
}

pub struct CommandStream {
    // Not directly accessible, use methods
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

#### Command Builder Methods

```rust
impl CommandBuilder {
    pub fn new(program: impl Into<String>) -> Self;
    pub fn build(self) -> Command;
    pub fn arg(mut self, arg: impl Into<String>) -> Self;
    pub fn timeout(mut self, timeout: Duration) -> Self;
    pub fn current_dir(mut self, path: impl AsRef<Path>) -> Self;
}
```

#### Command Output Methods

```rust
impl CommandOutput {
    pub fn new(status: i32, stdout: String, stderr: String, duration: Duration) -> Self;
    pub fn status(&self) -> i32;
    pub fn stdout(&self) -> &str;
    pub fn stderr(&self) -> &str;
    pub fn duration(&self) -> Duration;
    pub fn success(&self) -> bool;
}
```

#### Command Stream Methods

```rust
impl CommandStream {
    pub async fn next_timeout(
        &mut self,
        timeout_duration: Duration,
    ) -> Result<Option<StreamOutput>>;
    pub fn cancel(&self);
}
```

#### StreamConfig Methods

```rust
impl StreamConfig {
    pub fn new(buffer_size: usize, read_timeout: Duration) -> Self;
}

impl Default for StreamConfig {
    fn default() -> Self;  // buffer_size: 1024, read_timeout: Duration::from_secs(1)
}
```

#### Examples

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
    pub fn get_status(&self, id: &str) -> Option<CommandStatus>;
    pub fn get_result(&self, id: &str) -> Option<CommandQueueResult>;
    pub async fn wait_for_command(&self, id: &str, timeout: Duration) -> Result<CommandQueueResult>;
    pub async fn wait_for_completion(&self) -> Result<()>;
    pub async fn shutdown(&mut self) -> Result<()>;
}

impl CommandQueueConfig {
    // Default implementation provides reasonable defaults
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

#### Examples

```rust
use sublime_standard_tools::command::{CommandQueue, CommandBuilder, CommandPriority, CommandQueueConfig};
use std::time::Duration;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a command queue with custom configuration
    let config = CommandQueueConfig {
        max_concurrent_commands: 4,
        rate_limit: Some(Duration::from_millis(100)),
        default_timeout: Duration::from_secs(30),
        shutdown_timeout: Duration::from_secs(10),
    };
    
    let mut queue = CommandQueue::with_config(config).start()?;
    
    // Enqueue commands with different priorities
    let cmd1 = CommandBuilder::new("echo").arg("Low priority task").build();
    let id1 = queue.enqueue(cmd1, CommandPriority::Low).await?;
    
    let cmd2 = CommandBuilder::new("echo").arg("High priority task").build();
    let id2 = queue.enqueue(cmd2, CommandPriority::High).await?;
    
    // Wait for a specific command to complete
    let result1 = queue.wait_for_command(&id1, Duration::from_secs(5)).await?;
    if result1.is_successful() {
        println!("Command 1 output: {}", result1.output.unwrap().stdout());
    }
    
    // Wait for all commands to complete
    queue.wait_for_completion().await?;
    
    // Shutdown the queue when done
    queue.shutdown().await?;
    
    Ok(())
}
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
pub struct FileSystemManager;

impl FileSystemManager {
    pub fn new() -> Self;
}

impl FileSystem for FileSystemManager {
    // Implements all FileSystem trait methods
}
```

### Path Extensions

```rust
pub enum NodePathKind {
    NodeModules,
    PackageJson,
    Src,
    Dist,
    Test,
}

impl NodePathKind {
    pub fn default_path(self) -> &'static str;
}

pub struct PathUtils;

impl PathUtils {
    pub fn find_project_root(start: &Path) -> Option<PathBuf>;
    pub fn current_dir() -> FileSystemResult<PathBuf>;
    pub fn make_relative(path: &Path, base: &Path) -> FileSystemResult<PathBuf>;
}

pub trait PathExt {
    fn normalize(&self) -> PathBuf;
    fn is_in_project(&self) -> bool;
    fn relative_to_project(&self) -> Option<PathBuf>;
    fn node_path(&self, kind: NodePathKind) -> PathBuf;
    fn canonicalize(&self) -> Result<PathBuf>;
}

impl PathExt for Path {
    // Implements all PathExt trait methods
}
```

#### Examples

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
pub enum MonorepoKind {
    NpmWorkSpace,
    YarnWorkspaces,
    PnpmWorkspaces,
    BunWorkspaces,
    DenoWorkspaces,
    Custom {
        name: String,
        config_file: String,
    },
}

impl MonorepoKind {
    pub fn name(&self) -> String;
    pub fn config_file(self) -> String;
    pub fn set_custom(&self, name: String, config_file: String) -> Self;
}

pub struct WorkspacePackage {
    pub name: String,
    pub version: String,
    pub location: PathBuf,
    pub absolute_path: PathBuf,
    pub workspace_dependencies: Vec<String>,
    pub workspace_dev_dependencies: Vec<String>,
}

pub struct MonorepoDescriptor {
    // Not directly accessible, use methods
}

impl MonorepoDescriptor {
    pub fn new(kind: MonorepoKind, root: PathBuf, packages: Vec<WorkspacePackage>) -> Self;
    pub fn kind(&self) -> &MonorepoKind;
    pub fn root(&self) -> &Path;
    pub fn packages(&self) -> &[WorkspacePackage];
    pub fn get_package(&self, name: &str) -> Option<&WorkspacePackage>;
    pub fn get_dependency_graph(&self) -> HashMap<&str, Vec<&WorkspacePackage>>;
    pub fn find_dependencies_by_name(&self, package_name: &str) -> Vec<&WorkspacePackage>;
    pub fn find_package_for_path(&self, path: &Path) -> Option<&WorkspacePackage>;
}

pub struct MonorepoDetector<F: FileSystem = FileSystemManager> {
    // Not directly accessible, use methods
}

impl MonorepoDetector<FileSystemManager> {
    pub fn new() -> Self;
}

impl<F: FileSystem> MonorepoDetector<F> {
    pub fn with_filesystem(fs: F) -> Self;
    pub fn is_monorepo_root(&self, path: impl AsRef<Path>) -> Result<Option<MonorepoKind>>;
    pub fn find_monorepo_root(
        &self,
        start_path: impl AsRef<Path>,
    ) -> Result<Option<(PathBuf, MonorepoKind)>>;
    pub fn detect_monorepo(&self, path: impl AsRef<Path>) -> Result<MonorepoDescriptor>;
    pub fn has_multiple_packages(&self, path: &Path) -> bool;
}

pub struct PnpmWorkspaceConfig {
    // Not directly accessible
}
```

#### Examples

```rust
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoKind};
use std::path::Path;

fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a monorepo detector
    let detector = MonorepoDetector::new();
    
    // Check if a path is a monorepo root
    if let Some(kind) = detector.is_monorepo_root(".")? {
        println!("This is a {} monorepo", kind.name());
    }
    
    // Find the nearest monorepo root
    if let Some((root, kind)) = detector.find_monorepo_root("src/components")? {
        println!("Found {} monorepo at {}", kind.name(), root.display());
    }
    
    // Detect and analyze a monorepo
    let monorepo = detector.detect_monorepo(".")?;
    
    println!("Detected {} monorepo", monorepo.kind().name());
    println!("Root directory: {}", monorepo.root().display());
    println!("Contains {} packages:", monorepo.packages().len());
    
    for package in monorepo.packages() {
        println!("- {} v{} at {}", package.name, package.version, package.location.display());
    }
    
    // Get the dependency graph
    let graph = monorepo.get_dependency_graph();
    
    // Find all packages that depend on a specific package
    if let Some(dependents) = graph.get("shared-lib") {
        println!("Packages depending on shared-lib:");
        for dep in dependents {
            println!("- {}", dep.name);
        }
    }
    
    Ok(())
}
```

### Package Management

```rust
pub enum PackageManagerKind {
    Npm,
    Yarn,
    Pnpm,
    Bun,
    Jsr,
}

impl PackageManagerKind {
    pub fn lock_file(self) -> &'static str;
    pub fn command(self) -> &'static str;
}

pub struct PackageManager {
    // Not directly accessible, use methods
}

impl PackageManager {
    pub fn new(kind: PackageManagerKind, root: impl Into<PathBuf>) -> Self;
    pub fn detect(path: impl AsRef<Path>) -> Result<Self>;
    pub fn kind(&self) -> PackageManagerKind;
    pub fn root(&self) -> &Path;
    pub fn lock_file_path(&self) -> PathBuf;
}
```

#### Examples

```rust
use sublime_standard_tools::monorepo::{PackageManager, PackageManagerKind};
use std::path::Path;

fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a package manager instance
    let npm = PackageManager::new(PackageManagerKind::Npm, "/project/root");
    println!("Lock file: {}", npm.lock_file_path().display());
    
    // Detect package manager from a directory
    let detected = PackageManager::detect(".")?;
    println!("Detected package manager: {}", detected.kind().command());
    
    Ok(())
}
```

### Project Management

```rust
pub struct ProjectConfig {
    // Not directly accessible, use methods
}

impl ProjectConfig {
    pub fn new() -> Self;
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self;
    pub fn with_detect_package_manager(mut self, detect: bool) -> Self;
    pub fn with_validate_structure(mut self, validate: bool) -> Self;
    pub fn with_detect_monorepo(mut self, detect: bool) -> Self;
}

pub enum ProjectValidationStatus {
    Valid,
    Warning(Vec<String>),
    Error(Vec<String>),
    NotValidated,
}

pub struct Project {
    // Not directly accessible, use methods
}

impl Project {
    pub fn new(root: impl Into<PathBuf>, config: ProjectConfig) -> Self;
    pub fn root(&self) -> &Path;
    pub fn package_manager(&self) -> Option<&PackageManager>;
    pub fn validation_status(&self) -> &ProjectValidationStatus;
    pub fn package_json(&self) -> Option<&PackageJson>;
}

pub struct ProjectManager<F: FileSystem = FileSystemManager> {
    // Not directly accessible, use methods
}

impl ProjectManager<FileSystemManager> {
    pub fn new() -> Self;
}

impl<F: FileSystem> ProjectManager<F> {
    pub fn with_filesystem(fs: F) -> Self;
    pub fn detect_project(
        &self,
        path: impl AsRef<Path>,
        config: &ProjectConfig,
    ) -> Result<Project>;
    pub fn validate_project(&self, project: &mut Project) -> Result<()>;
}
```

#### Examples

```rust
use sublime_standard_tools::monorepo::{ProjectManager, ProjectConfig, ProjectValidationStatus};
use std::path::Path;

fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a project manager
    let manager = ProjectManager::new();
    
    // Configure project detection
    let config = ProjectConfig::new()
        .with_detect_package_manager(true)
        .with_validate_structure(true)
        .with_detect_monorepo(true);
    
    // Detect a project
    let project = manager.detect_project(".", &config)?;
    
    println!("Project root: {}", project.root().display());
    
    // Check validation status
    match project.validation_status() {
        ProjectValidationStatus::Valid => {
            println!("Project structure is valid");
        },
        ProjectValidationStatus::Warning(warnings) => {
            println!("Project has warnings:");
            for warning in warnings {
                println!("  - {}", warning);
            }
        },
        ProjectValidationStatus::Error(errors) => {
            println!("Project has errors:");
            for error in errors {
                println!("  - {}", error);
            }
        },
        ProjectValidationStatus::NotValidated => {
            println!("Project has not been validated");
        }
    }
    
    // Access package.json if available
    if let Some(package_json) = project.package_json() {
        println!("Project name: {}", package_json.name);
        println!("Project version: {}", package_json.version);
    }
    
    Ok(())
}
```

### Configuration Management

```rust
pub enum ConfigScope {
    Global,
    User,
    Project,
    Runtime,
}

pub enum ConfigFormat {
    Json,
    Toml,
    Yaml,
}

pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Map(HashMap<String, ConfigValue>),
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

pub struct ConfigManager {
    // Not directly accessible, use methods
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

#### Examples

```rust
use sublime_standard_tools::monorepo::{ConfigManager, ConfigScope, ConfigValue, ConfigFormat};
use std::path::Path;

fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a configuration manager
    let mut config_manager = ConfigManager::new();
    
    // Set paths for different configuration scopes
    config_manager.set_path(ConfigScope::User, "~/.config/myapp.json");
    config_manager.set_path(ConfigScope::Project, "./project-config.json");
    
    // Set configuration values
    config_manager.set("theme", ConfigValue::String("dark".to_string()));
    config_manager.set("debug", ConfigValue::Boolean(true));
    config_manager.set("version", ConfigValue::Float(1.5));
    
    // Get configuration values
    if let Some(theme) = config_manager.get("theme") {
        if let Some(theme_str) = theme.as_string() {
            println!("Theme: {}", theme_str);
        }
    }
    
    // Save configuration to file
    config_manager.save_to_file(Path::new("./config.json"))?;
    
    // Load configuration from file
    config_manager.load_from_file(Path::new("./config.json"))?;
    
    // Remove a configuration value
    if let Some(_) = config_manager.remove("temp") {
        println!("Removed temporary value");
    }
    
    Ok(())
}
```

## Error Module

The error module provides comprehensive error types for various operations within the crate.

### Error Types

```rust
pub enum FileSystemError {
    NotFound { path: PathBuf },
    PermissionDenied { path: PathBuf },
    Io { path: PathBuf, source: io::Error },
    NotADirectory { path: PathBuf },
    NotAFile { path: PathBuf },
    Utf8Decode { path: PathBuf, source: std::string::FromUtf8Error },
    Validation { path: PathBuf, reason: String },
}

pub enum MonorepoError {
    Detection { source: FileSystemError },
    Parsing { source: FileSystemError },
    Reading { source: FileSystemError },
    Writing { source: FileSystemError },
    ManagerNotFound,
}

pub enum WorkspaceError {
    InvalidPackageJson(String),
    InvalidWorkspacesPattern(String),
    InvalidPnpmWorkspace(String),
    PackageNotFound(String),
    WorkspaceNotFound(String),
    WorkspaceConfigMissing(String),
}

pub enum CommandError {
    SpawnFailed { cmd: String, source: io::Error },
    ExecutionFailed { cmd: String, source: Option<io::Error> },
    NonZeroExitCode { cmd: String, code: Option<i32>, stderr: String },
    Timeout { duration: Duration },
    Killed { reason: String },
    Configuration { description: String },
    CaptureFailed { stream: String },
    StreamReadError { stream: String, source: io::Error },
    Generic(String),
}

pub enum Error {
    Monorepo(MonorepoError),
    FileSystem(FileSystemError),
    Workspace(WorkspaceError),
    Command(CommandError),
    Operation(String),
}

impl Error {
    pub fn operation(message: impl Into<String>) -> Self;
}
```

### Result Types

```rust
pub type FileSystemResult<T> = std::result::Result<T, FileSystemError>;
pub type MonorepoResult<T> = std::result::Result<T, MonorepoError>;
pub type WorkspaceResult<T> = std::result::Result<T, WorkspaceError>;
pub type CommandResult<T> = std::result::Result<T, CommandError>;
pub type Result<T> = std::result::Result<T, Error>;
```

#### Examples

```rust
use sublime_standard_tools::error::{Error, FileSystemError, Result};
use std::path::PathBuf;

fn example() -> Result<()> {
    // Handle filesystem errors
    let path = PathBuf::from("/nonexistent/path");
    let result = std::fs::read_to_string(&path);
    
    if let Err(io_error) = result {
        return Err(Error::FileSystem(FileSystemError::NotFound { path }));
    }
    
    // Create an operation error
    if true {
        return Err(Error::operation("Operation failed for some reason"));
    }
    
    Ok(())
}

// Using the Result type alias
fn another_example() -> Result<String> {
    // Implementation...
    Ok("Success".to_string())
}
```

This API specification covers the major functionality exposed by the `sublime_standard_tools` crate. Each section includes the relevant types, methods, and examples showing how to use them effectively.