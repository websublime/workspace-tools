# Sublime Standard Tools

[![Crates.io](https://img.shields.io/crates/v/sublime_standard_tools.svg)](https://crates.io/crates/sublime_standard_tools)
[![Documentation](https://docs.rs/sublime_standard_tools/badge.svg)](https://docs.rs/sublime_standard_tools)
[![License](https://img.shields.io/crates/l/sublime_standard_tools.svg)](https://github.com/websublime/workspace-node-tools/blob/main/LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/websublime/workspace-node-tools/ci.yml?branch=main)](https://github.com/websublime/workspace-node-tools/actions)

A comprehensive Rust toolkit for working with Node.js projects, package managers, and development workflows. This crate provides a unified, type-safe interface for interacting with Node.js ecosystems from Rust applications.

## 🚀 Features

- **🎯 Unified Project Detection**: Automatically detect and work with both simple Node.js projects and monorepos
- **📦 Package Manager Support**: Full support for npm, yarn, pnpm, bun, and jsr package managers
- **🔧 Monorepo Management**: Advanced monorepo detection and workspace analysis across different formats
- **⚡ Command Execution**: Robust command execution with queuing, streaming, and timeout management
- **📁 Filesystem Operations**: Safe, async filesystem operations with retry logic and validation
- **🔧 Flexible Configuration**: Comprehensive configuration system with environment variable overrides
- **🛡️ Error Handling**: Structured error handling with recovery strategies and detailed context
- **🏗️ Async-First**: Built with async/await from the ground up for optimal performance
- **🌍 Cross-Platform**: Full support for Windows, macOS, and Linux environments

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sublime_standard_tools = "0.1"
```

For async support, make sure you have tokio in your dependencies:

```toml
[dependencies]
sublime_standard_tools = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

## 📚 Table of Contents

- [Quick Start](#-quick-start)
- [Core Modules](#-core-modules)
  - [Project Detection](#project-detection)
  - [Monorepo Management](#monorepo-management)
  - [Command Execution](#command-execution)
  - [Filesystem Operations](#filesystem-operations)
  - [Configuration Management](#configuration-management)
  - [Error Handling](#error-handling-examples)
- [Configuration System](#️-configuration-system)
- [Architecture](#️-architecture)
- [Real-World Examples](#-real-world-examples)
- [API Reference](#-api-reference)
- [Complete API Specification](#-complete-api-specification)
- [Contributing](#-contributing)

## 🏃 Quick Start

### Version Information

```rust
use sublime_standard_tools;

fn main() {
    println!("Using sublime_standard_tools version: {}", sublime_standard_tools::version());
}
```

### Basic Project Detection

```rust
use sublime_standard_tools::project::ProjectDetector;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let detector = ProjectDetector::new();
    
    match detector.detect(Path::new("."), None).await {
        Ok(project) => {
            let info = project.as_project_info();
            println!("Found {} project", info.kind().name());
            
            if let Some(pm) = info.package_manager() {
                println!("Using {} package manager", pm.kind().command());
            }
        }
        Err(e) => eprintln!("Detection failed: {}", e),
    }
    
    Ok(())
}
```

## 🧩 Core Modules

### Project Detection

#### Basic Project Detection and Validation

```rust
use sublime_standard_tools::project::{ProjectDetector, ProjectValidator};
use sublime_standard_tools::config::StandardConfig;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let detector = ProjectDetector::new();
    let project = detector.detect(Path::new("."), None).await?;
    let info = project.as_project_info();
    
    println!("Project Details:");
    println!("  Type: {}", info.kind().name());
    println!("  Root: {}", info.root().display());
    
    if let Some(pm) = info.package_manager() {
        println!("  Package Manager: {}", pm.kind().command());
        println!("  Supports Workspaces: {}", pm.supports_workspaces());
    }
    
    // Validate project configuration
    let validator = ProjectValidator::new(StandardConfig::default());
    let validation_result = validator.validate(&project).await?;
    
    println!("Validation Status: {:?}", validation_result.status());
    if !validation_result.errors().is_empty() {
        println!("Validation Errors:");
        for error in validation_result.errors() {
            println!("  - {}", error);
        }
    }
    
    Ok(())
}
```

#### Working with Package Manager Detection

```rust
use sublime_standard_tools::node::{PackageManager, PackageManagerKind};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Detect package manager from current directory
    let manager = PackageManager::detect(Path::new("."))?;
    println!("Detected package manager: {}", manager.command());
    
    // Check specific capabilities
    match manager.kind() {
        PackageManagerKind::Npm => {
            println!("Using npm with lock file support");
        }
        PackageManagerKind::Yarn => {
            println!("Using Yarn with workspace support: {}", manager.supports_workspaces());
        }
        PackageManagerKind::Pnpm => {
            println!("Using pnpm with efficient workspace handling");
        }
        PackageManagerKind::Bun => {
            println!("Using Bun with fast package installation");
        }
        PackageManagerKind::Jsr => {
            println!("Using JSR package registry");
        }
    }
    
    // Get lock file information
    if let Some(lock_file) = manager.lock_file_name() {
        println!("Lock file: {}", lock_file);
    }
    
    Ok(())
}
```

### Monorepo Management

#### Comprehensive Monorepo Analysis

```rust
use sublime_standard_tools::monorepo::MonorepoDetector;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let detector = MonorepoDetector::new();
    
    // Check if current directory is a monorepo
    if let Some(kind) = detector.is_monorepo_root(".")? {
        println!("This directory is a {} monorepo", kind.name());
        
        // Analyze the monorepo structure
        let monorepo = detector.detect_monorepo(".").await?;
        
        println!("\nMonorepo Analysis:");
        println!("  Type: {}", monorepo.kind().name());
        println!("  Root: {}", monorepo.root().display());
        println!("  Packages: {}", monorepo.packages().len());
        
        // List all packages
        println!("\nWorkspace Packages:");
        for package in monorepo.packages() {
            println!("  📦 {} v{}", package.name, package.version);
            println!("     Location: {}", package.location.display());
            println!("     Absolute: {}", package.absolute_path.display());
            
            if !package.dependencies.is_empty() {
                println!("     Dependencies: {}", package.dependencies.len());
            }
            if !package.dev_dependencies.is_empty() {
                println!("     Dev Dependencies: {}", package.dev_dependencies.len());
            }
        }
        
        // Generate dependency graph
        let graph = monorepo.get_dependency_graph();
        println!("\nDependency Graph Analysis:");
        for (package, deps) in graph {
            if !deps.is_empty() {
                println!("  {} depends on:", package);
                for dep in deps {
                    println!("    ├─ {} ({})", dep.name, dep.version_requirement);
                }
            }
        }
        
        // Check for workspace configuration
        if let Some(config) = monorepo.workspace_config() {
            println!("\nWorkspace Configuration:");
            println!("  Patterns: {:?}", config.patterns);
            if let Some(exclude) = &config.exclude {
                println!("  Excludes: {:?}", exclude);
            }
        }
    } else {
        println!("This directory is not a monorepo root");
    }
    
    Ok(())
}
```

### Command Execution

#### Basic Command Execution (Async and Sync)

```rust
use sublime_standard_tools::command::{
    CommandBuilder, DefaultCommandExecutor, SyncCommandExecutor, Executor
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Async command execution
    let executor = DefaultCommandExecutor::new();
    
    let cmd = CommandBuilder::new("npm")
        .args(["--version"])
        .timeout(Duration::from_secs(10))
        .build();
    
    let output = executor.execute(cmd).await?;
    
    if output.success() {
        println!("npm version: {}", output.stdout().trim());
    } else {
        eprintln!("Command failed: {}", output.stderr());
    }
    
    // Sync command execution (for simple cases)
    let sync_executor = SyncCommandExecutor::new();
    let sync_cmd = CommandBuilder::new("node")
        .args(["--version"])
        .build();
    
    let sync_output = sync_executor.execute(sync_cmd)?;
    if sync_output.success() {
        println!("Node.js version: {}", sync_output.stdout().trim());
    }
    
    Ok(())
}
```

#### Command Execution with Queuing and Priorities

```rust
use sublime_standard_tools::command::{
    CommandBuilder, CommandQueue, CommandPriority, CommandQueueConfig
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a command queue with custom configuration
    let queue_config = CommandQueueConfig {
        max_concurrent_commands: 3,
        collection_window: Duration::from_millis(100),
        collection_sleep: Duration::from_micros(500),
        idle_sleep: Duration::from_millis(50),
    };
    
    let mut queue = CommandQueue::new_with_config(queue_config).start()?;
    
    // Build commands with different priorities
    let install_cmd = CommandBuilder::new("npm")
        .args(["install"])
        .timeout(Duration::from_secs(300))
        .build();
        
    let build_cmd = CommandBuilder::new("npm")
        .args(["run", "build"])
        .timeout(Duration::from_secs(60))
        .build();
        
    let test_cmd = CommandBuilder::new("npm")
        .args(["test"])
        .timeout(Duration::from_secs(30))
        .build();
    
    // Enqueue commands with priorities
    let install_id = queue.enqueue(install_cmd, CommandPriority::High).await?;
    let build_id = queue.enqueue(build_cmd, CommandPriority::Normal).await?;
    let test_id = queue.enqueue(test_cmd, CommandPriority::Low).await?;
    
    // Wait for all commands to complete
    let install_result = queue.wait_for_command(&install_id, Duration::from_secs(360)).await?;
    let build_result = queue.wait_for_command(&build_id, Duration::from_secs(120)).await?;
    let test_result = queue.wait_for_command(&test_id, Duration::from_secs(90)).await?;
    
    println!("Install result: {:?}", install_result.status);
    println!("Build result: {:?}", build_result.status);
    println!("Test result: {:?}", test_result.status);
    
    // Get queue statistics
    let stats = queue.stats().await?;
    println!("Queue processed {} commands", stats.total_processed);
    
    queue.shutdown().await?;
    Ok(())
}
```

#### Streaming Command Output

```rust
use sublime_standard_tools::command::{
    CommandBuilder, DefaultCommandExecutor, Executor, StreamConfig, StreamOutput
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executor = DefaultCommandExecutor::new();
    
    let stream_config = StreamConfig {
        buffer_size: 1024,
        read_timeout: Duration::from_secs(1),
    };
    
    let cmd = CommandBuilder::new("npm")
        .args(["install", "--verbose"])
        .build();
    
    let (mut stream, _child) = executor.execute_stream(cmd, stream_config).await?;
    
    println!("Streaming npm install output:");
    while let Ok(Some(output)) = stream.next_timeout(Duration::from_secs(1)).await {
        match output {
            StreamOutput::Stdout(line) => {
                println!("📦 {}", line.trim());
            }
            StreamOutput::Stderr(line) => {
                eprintln!("⚠️  {}", line.trim());
            }
            StreamOutput::End => {
                println!("🎉 Installation completed!");
                break;
            }
        }
    }
    
    Ok(())
}
```

### Filesystem Operations

#### Basic Filesystem Operations

```rust
use sublime_standard_tools::filesystem::{
    FileSystemManager, AsyncFileSystem, NodePathKind, PathExt, PathUtils
};
use sublime_standard_tools::config::StandardConfig;
use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create filesystem manager with default configuration
    let fs = FileSystemManager::new();
    
    // Or load with project configuration
    let fs_with_config = FileSystemManager::new_with_project_config(Path::new(".")).await?;
    
    // Basic file operations
    let package_json_path = Path::new("package.json");
    
    if fs.exists(package_json_path).await? {
        println!("📄 package.json exists");
        
        // Read file contents
        let contents = fs.read_to_string(package_json_path).await?;
        let parsed: serde_json::Value = serde_json::from_str(&contents)?;
        
        if let Some(name) = parsed.get("name").and_then(|n| n.as_str()) {
            println!("📦 Package name: {}", name);
        }
        
        // Get file metadata
        let metadata = fs.metadata(package_json_path).await?;
        println!("📊 File size: {} bytes", metadata.len());
    }
    
    // Directory operations
    let node_modules = Path::new("node_modules");
    if fs.is_dir(node_modules).await? {
        println!("📁 node_modules directory exists");
        
        // List directory contents (first level)
        let entries = fs.read_dir(node_modules).await?;
        println!("📋 Found {} entries in node_modules", entries.len());
        
        for entry in entries.into_iter().take(5) {
            println!("   - {}", entry.display());
        }
    }
    
    // Path utilities
    let current_dir = PathBuf::from(".");
    
    // Check Node.js specific paths
    if current_dir.is_package_json_dir() {
        println!("✅ Current directory contains package.json");
    }
    
    if current_dir.is_node_project() {
        println!("✅ Current directory is a Node.js project");
    }
    
    // Find package.json
    if let Some(package_json) = current_dir.find_package_json() {
        println!("📍 Found package.json at: {}", package_json.display());
    }
    
    // Get Node.js path kind
    match current_dir.node_path_kind() {
        NodePathKind::ProjectRoot => println!("📂 This is a project root"),
        NodePathKind::PackageDirectory => println!("📦 This is a package directory"),
        NodePathKind::NodeModules => println!("🗂️  This is node_modules"),
        NodePathKind::SourceDirectory => println!("📝 This is a source directory"),
        NodePathKind::Other => println!("❓ Other path type"),
    }
    
    Ok(())
}
```

#### Advanced Filesystem Operations with Retries

```rust
use sublime_standard_tools::filesystem::{FileSystemManager, AsyncFileSystemConfig};
use sublime_standard_tools::error::{FileSystemError, FileSystemResult};
use std::time::Duration;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure filesystem with retry logic
    let fs_config = AsyncFileSystemConfig {
        buffer_size: 8192,
        max_concurrent_operations: 10,
        operation_timeout: Duration::from_secs(30),
        retry_config: Some(sublime_standard_tools::filesystem::RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        }),
        ignore_patterns: vec![
            ".git".to_string(),
            "node_modules".to_string(),
            "target".to_string(),
        ],
    };
    
    let fs = FileSystemManager::new_with_config(fs_config);
    
    // Safe file operations with automatic retries
    async fn safe_read_file(
        fs: &FileSystemManager,
        path: &Path,
    ) -> FileSystemResult<String> {
        match fs.read_to_string(path).await {
            Ok(content) => Ok(content),
            Err(FileSystemError::NotFound { .. }) => {
                eprintln!("⚠️  File not found: {}", path.display());
                Ok(String::new())
            }
            Err(FileSystemError::PermissionDenied { .. }) => {
                eprintln!("🔒 Permission denied: {}", path.display());
                Ok(String::new())
            }
            Err(e) => Err(e),
        }
    }
    
    // Read multiple files concurrently
    let files = vec!["package.json", "tsconfig.json", "README.md"];
    let mut handles = Vec::new();
    
    for file in files {
        let path = Path::new(file);
        let fs_clone = fs.clone(); // FileSystemManager is cloneable for concurrent use
        
        let handle = tokio::spawn(async move {
            (file, safe_read_file(&fs_clone, path).await)
        });
        
        handles.push(handle);
    }
    
    // Wait for all files to be read
    for handle in handles {
        let (file, result) = handle.await?;
        match result {
            Ok(content) if !content.is_empty() => {
                println!("✅ Read {}: {} bytes", file, content.len());
            }
            Ok(_) => {
                println!("📄 File {} is empty or not found", file);
            }
            Err(e) => {
                eprintln!("❌ Failed to read {}: {}", file, e);
            }
        }
    }
    
    Ok(())
}
```

### Configuration Management

#### Advanced Configuration Usage

```rust
use sublime_standard_tools::config::{
    ConfigManager, StandardConfig, ConfigBuilder, ConfigSource, ConfigSourcePriority
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a configuration manager with multiple sources
    let config_manager = ConfigManager::<StandardConfig>::builder()
        .with_defaults()
        .with_file_optional("~/.config/sublime/config.toml")
        .with_file_optional("repo.config.toml")
        .with_file_optional("repo.config.yml")
        .with_file_optional("repo.config.json")
        .with_env_prefix("SUBLIME")
        .build()?;
    
    // Load configuration with source tracking
    let config = config_manager.load().await?;
    
    println!("🔧 Configuration loaded successfully");
    println!("📊 Package manager detection order: {:?}", config.package_managers.detection_order);
    println!("⏱️  Default command timeout: {:?}", config.commands.default_timeout);
    println!("🔍 Max search depth: {}", config.monorepo.max_search_depth);
    println!("📁 Workspace patterns: {:?}", config.monorepo.workspace_patterns);
    
    // Access specific configuration sections
    let pm_config = &config.package_managers;
    println!("\n📦 Package Manager Configuration:");
    println!("  Detection order: {:?}", pm_config.detection_order);
    println!("  Detect from env: {}", pm_config.detect_from_env);
    println!("  Environment variable: {}", pm_config.env_var_name);
    
    if let Some(fallback) = &pm_config.fallback {
        println!("  Fallback manager: {}", fallback);
    }
    
    // Command configuration
    let cmd_config = &config.commands;
    println!("\n⚡ Command Configuration:");
    println!("  Default timeout: {:?}", cmd_config.default_timeout);
    println!("  Max concurrent: {}", cmd_config.max_concurrent_commands);
    println!("  Stream buffer size: {}", cmd_config.stream_buffer_size);
    println!("  Inherit environment: {}", cmd_config.inherit_env);
    
    // Filesystem configuration
    let fs_config = &config.filesystem;
    println!("\n📁 Filesystem Configuration:");
    println!("  Ignore patterns: {:?}", fs_config.ignore_patterns);
    println!("  Async buffer size: {}", fs_config.async_io.buffer_size);
    println!("  Max concurrent ops: {}", fs_config.async_io.max_concurrent_operations);
    
    // Save modified configuration (if needed)
    // let mut modified_config = config.clone();
    // modified_config.commands.default_timeout = Duration::from_secs(45);
    // config_manager.save(&modified_config).await?;
    
    Ok(())
}
```

#### Creating Custom Configuration

```rust
use sublime_standard_tools::config::{
    StandardConfig, PackageManagerConfig, MonorepoConfig, CommandConfig,
    FilesystemConfig, ValidationConfig, ConfigManager
};
use std::time::Duration;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a custom configuration programmatically
    let mut custom_config = StandardConfig {
        version: "1.0".to_string(),
        package_managers: PackageManagerConfig {
            detection_order: vec![
                "bun".to_string(),
                "pnpm".to_string(),
                "yarn".to_string(),
                "npm".to_string(),
            ],
            detect_from_env: true,
            env_var_name: "MY_PACKAGE_MANAGER".to_string(),
            fallback: Some("npm".to_string()),
            custom_lock_files: HashMap::new(),
            binary_paths: HashMap::new(),
        },
        monorepo: MonorepoConfig {
            workspace_patterns: vec![
                "packages/*".to_string(),
                "apps/*".to_string(),
                "libs/*".to_string(),
            ],
            package_directories: vec![
                "packages".to_string(),
                "apps".to_string(),
                "libs".to_string(),
            ],
            exclude_patterns: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "dist".to_string(),
                "build".to_string(),
            ],
            max_search_depth: 3,
            follow_symlinks: false,
            custom_workspace_fields: vec!["@myorg/".to_string()],
        },
        commands: CommandConfig {
            default_timeout: Duration::from_secs(60),
            stream_buffer_size: 2048,
            stream_read_timeout: Duration::from_millis(500),
            max_concurrent_commands: 6,
            inherit_env: true,
            queue_collection_window: Duration::from_millis(10),
            queue_collection_sleep: Duration::from_micros(200),
            queue_idle_sleep: Duration::from_millis(20),
            timeout_overrides: HashMap::from([
                ("npm install".to_string(), Duration::from_secs(600)),
                ("npm run build".to_string(), Duration::from_secs(300)),
            ]),
            env_vars: HashMap::from([
                ("NODE_ENV".to_string(), "production".to_string()),
                ("CI".to_string(), "true".to_string()),
            ]),
        },
        filesystem: FilesystemConfig::default(),
        validation: ValidationConfig::default(),
    };
    
    // Use the custom configuration with components
    println!("🎛️  Using custom configuration:");
    println!("  Package manager order: {:?}", custom_config.package_managers.detection_order);
    println!("  Command timeout: {:?}", custom_config.commands.default_timeout);
    println!("  Workspace patterns: {:?}", custom_config.monorepo.workspace_patterns);
    
    Ok(())
}
```

### Error Handling Examples

#### Comprehensive Error Handling and Recovery

```rust
use sublime_standard_tools::error::{
    Error, ErrorRecoveryManager, RecoveryStrategy, RecoveryResult, LogLevel,
    FileSystemError, CommandError, MonorepoError
};
use sublime_standard_tools::project::ProjectDetector;
use sublime_standard_tools::command::{CommandBuilder, DefaultCommandExecutor, Executor};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an error recovery manager
    let mut recovery_manager = ErrorRecoveryManager::new();
    
    // Configure recovery strategies
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
            alternative: "Use shorter timeout".to_string(),
        },
    );
    
    // Example: Robust project detection with error handling
    async fn robust_project_detection(
        recovery_manager: &mut ErrorRecoveryManager,
        path: &Path,
    ) -> Result<(), Error> {
        let detector = ProjectDetector::new();
        
        match detector.detect(path, None).await {
            Ok(project) => {
                let info = project.as_project_info();
                println!("✅ Successfully detected {} project", info.kind().name());
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Project detection failed: {}", e);
                
                // Attempt recovery
                let recovery_result = recovery_manager
                    .recover("project_detection", &e, LogLevel::Warn)
                    .await;
                
                match recovery_result {
                    RecoveryResult::Recovered => {
                        println!("🔄 Recovered from project detection error");
                        Ok(())
                    }
                    RecoveryResult::Failed(recovery_error) => {
                        eprintln!("💥 Recovery failed: {}", recovery_error);
                        Err(e)
                    }
                    RecoveryResult::NoStrategy => {
                        eprintln!("🤷 No recovery strategy available");
                        Err(e)
                    }
                }
            }
        }
    }
    
    // Example: Error classification and handling
    async fn handle_command_execution() -> Result<(), Error> {
        let executor = DefaultCommandExecutor::new();
        
        let cmd = CommandBuilder::new("npm")
            .args(["run", "nonexistent-script"])
            .timeout(Duration::from_secs(10))
            .build();
        
        match executor.execute(cmd).await {
            Ok(output) if output.success() => {
                println!("✅ Command executed successfully");
                Ok(())
            }
            Ok(output) => {
                let error_msg = format!(
                    "Command failed with exit code {}: {}",
                    output.status().code().unwrap_or(-1),
                    output.stderr()
                );
                eprintln!("❌ {}", error_msg);
                Err(Error::Operation(error_msg))
            }
            Err(Error::Command(CommandError::Timeout { duration })) => {
                eprintln!("⏰ Command timed out after {:?}", duration);
                Err(Error::Operation("Command timeout".to_string()))
            }
            Err(Error::Command(CommandError::ExecutionFailed { command, source })) => {
                eprintln!("💥 Failed to execute command '{}': {}", command, source);
                Err(Error::Operation("Execution failed".to_string()))
            }
            Err(e) => {
                eprintln!("🚫 Unexpected error: {}", e);
                Err(e)
            }
        }
    }
    
    // Example: Filesystem error handling
    async fn handle_filesystem_errors() -> Result<(), Error> {
        use sublime_standard_tools::filesystem::{FileSystemManager, AsyncFileSystem};
        
        let fs = FileSystemManager::new();
        let file_path = Path::new("nonexistent-file.txt");
        
        match fs.read_to_string(file_path).await {
            Ok(content) => {
                println!("📄 File content: {}", content);
                Ok(())
            }
            Err(FileSystemError::NotFound { path }) => {
                eprintln!("📄❌ File not found: {}", path.display());
                
                // Create the file as a recovery strategy
                println!("🔄 Creating file as recovery...");
                if let Err(e) = fs.write(file_path, "Default content").await {
                    eprintln!("💥 Failed to create file: {}", e);
                    return Err(Error::FileSystem(e));
                }
                
                println!("✅ File created successfully");
                Ok(())
            }
            Err(FileSystemError::PermissionDenied { path, .. }) => {
                eprintln!("🔒 Permission denied: {}", path.display());
                Err(Error::Operation("Permission denied".to_string()))
            }
            Err(e) => {
                eprintln!("💥 Filesystem error: {}", e);
                Err(Error::FileSystem(e))
            }
        }
    }
    
    // Run examples with error handling
    let current_dir = PathBuf::from(".");
    
    println!("🔍 Testing project detection...");
    if let Err(e) = robust_project_detection(&mut recovery_manager, &current_dir).await {
        eprintln!("Project detection ultimately failed: {}", e);
    }
    
    println!("\n⚡ Testing command execution...");
    if let Err(e) = handle_command_execution().await {
        eprintln!("Command execution failed: {}", e);
    }
    
    println!("\n📁 Testing filesystem operations...");
    if let Err(e) = handle_filesystem_errors().await {
        eprintln!("Filesystem operations failed: {}", e);
    }
    
    // Display recovery manager statistics
    let stats = recovery_manager.stats();
    println!("\n📊 Error Recovery Statistics:");
    println!("  Total recovery attempts: {}", stats.total_attempts);
    println!("  Successful recoveries: {}", stats.successful_recoveries);
    println!("  Failed recoveries: {}", stats.failed_recoveries);
    
    Ok(())
}
```

## ⚙️ Configuration System

Sublime Standard Tools provides a comprehensive configuration system that supports multiple sources and formats. Configuration is loaded automatically from project files and can be customized through environment variables.

### Configuration Files

The crate automatically loads configuration from these files (in order of precedence):

1. `repo.config.toml` (project root)
2. `repo.config.yml` (project root)
3. `repo.config.yaml` (project root)
4. `repo.config.json` (project root)
5. `~/.config/sublime/config.toml` (user config)
6. Environment variables with `SUBLIME_` prefix

### Configuration Structure

```toml
# Configuration version for migration support
version = "1.0"

[package_managers]
# Detection order for package managers
detection_order = ["bun", "pnpm", "yarn", "npm", "jsr"]

# Whether to detect from environment variables
detect_from_env = true

# Environment variable name for preferred package manager
env_var_name = "SUBLIME_PACKAGE_MANAGER"

# Custom lock file names for each package manager
[package_managers.custom_lock_files]
npm = "package-lock.json"
yarn = "yarn.lock"

# Custom binary paths for package managers
[package_managers.binary_paths]
npm = "/usr/local/bin/npm"

# Fallback package manager if none detected
fallback = "npm"

[monorepo]
# Custom workspace directory patterns
workspace_patterns = [
    "packages/*",
    "apps/*", 
    "libs/*",
    "modules/*",
    "components/*",
    "services/*"
]

# Additional directories to check for packages
package_directories = [
    "packages",
    "apps",
    "libs", 
    "components",
    "modules",
    "services",
    "tools",
    "shared",
    "core"
]

# Patterns to exclude from package detection
exclude_patterns = [
    "node_modules",
    ".git",
    "dist",
    "build", 
    "coverage",
    ".next",
    ".nuxt",
    "out"
]

# Maximum depth for recursive package search
max_search_depth = 5

# Whether to follow symlinks during search
follow_symlinks = false

# Custom patterns for workspace detection in package.json
custom_workspace_fields = ["@myorg/"]

[commands]
# Default timeout for command execution
default_timeout = "30s"

# Buffer size for command output streaming
stream_buffer_size = 1024

# Read timeout for streaming output
stream_read_timeout = "1s"

# Maximum concurrent commands in queue
max_concurrent_commands = 4

# Whether to inherit parent process environment
inherit_env = true

# Queue collection window duration
queue_collection_window_ms = 5

# Queue collection sleep duration
queue_collection_sleep_us = 100

# Queue idle sleep duration
queue_idle_sleep_ms = 10

# Timeout overrides for specific commands
[commands.timeout_overrides]
"npm install" = "300s"
"npm run build" = "600s"

# Environment variables to set for all commands
[commands.env_vars]
NODE_ENV = "development"

[filesystem]
# Patterns to ignore during directory traversal
ignore_patterns = [
    ".git",
    "node_modules",
    "target", 
    ".DS_Store",
    "Thumbs.db"
]

# Async I/O configuration
[filesystem.async_io]
buffer_size = 8192
max_concurrent_operations = 10
operation_timeout = "5s"

# File operation retry configuration
[filesystem.retry]
max_attempts = 3
initial_delay = "100ms"
max_delay = "5s"
backoff_multiplier = 2.0

# Path conventions overrides
[filesystem.path_conventions]
node_modules = "node_modules"
package_json = "package.json"

[validation]
# Whether to require package.json at project root
require_package_json = true

# Required fields in package.json
required_package_fields = []

# Whether to validate dependency versions
validate_dependencies = true

# Whether to fail on validation warnings
strict_mode = false

# Custom validation rules
[validation.custom_rules]
min_node_version = "16.0.0"
```

### Environment Variable Overrides

All configuration options can be overridden using environment variables. The crate supports these environment variables:

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

### Auto-Loading Configuration

Most components in the crate support automatic configuration loading:

```rust
use sublime_standard_tools::{
    project::ProjectDetector,
    monorepo::MonorepoDetector,
    filesystem::FileSystemManager,
    command::DefaultCommandExecutor,
};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Each component can auto-load from project config files
    let project_detector = ProjectDetector::new(); // Uses default config
    let monorepo_detector = MonorepoDetector::new_with_project_config(Path::new(".")).await?;
    let filesystem = FileSystemManager::new_with_project_config(Path::new(".")).await?;
    let executor = DefaultCommandExecutor::new_with_project_config(Path::new(".")).await?;
    
    // Configuration is applied automatically
    Ok(())
}
```

## 🏗️ Architecture

The crate follows a clean architectural approach with clear separation of concerns:

```text
┌─────────────────────────────────────────────────────────────┐
│                    sublime_standard_tools                   │
├─────────────────────────────────────────────────────────────┤
│  config/      │  Flexible configuration management          │
│  ├─manager    │  ├─ ConfigManager (multi-source loading)   │
│  ├─standard   │  ├─ StandardConfig (crate configuration)   │
│  └─sources    │  └─ ConfigSource (files, env, defaults)    │
├─────────────────────────────────────────────────────────────┤
│  project/     │  Unified project detection and management   │
│  ├─detector   │  ├─ ProjectDetector (any project type)     │
│  ├─manager    │  ├─ ProjectManager (lifecycle management)  │
│  └─types      │  └─ ProjectInfo trait (common interface)   │
├─────────────────────────────────────────────────────────────┤
│  node/        │  Generic Node.js concepts                  │
│  ├─types      │  ├─ RepoKind (Simple vs Monorepo)         │
│  ├─package_*  │  ├─ PackageManager & PackageManagerKind   │
│  └─repository │  └─ RepositoryInfo trait                  │
├─────────────────────────────────────────────────────────────┤
│  monorepo/    │  Monorepo-specific functionality           │
│  ├─detector   │  ├─ MonorepoDetector (workspace detection) │
│  ├─descriptor │  ├─ MonorepoDescriptor (full structure)    │
│  └─kinds      │  └─ MonorepoKind (npm, yarn, pnpm, etc.)  │
├─────────────────────────────────────────────────────────────┤
│  command/     │  Robust command execution                  │
│  ├─executor   │  ├─ CommandExecutor (sync & async)        │
│  ├─queue      │  ├─ CommandQueue (prioritized execution)  │
│  └─stream     │  └─ CommandStream (real-time output)      │
├─────────────────────────────────────────────────────────────┤
│  filesystem/  │  Safe async filesystem operations          │
│  ├─manager    │  ├─ FileSystemManager (main interface)    │
│  ├─paths      │  ├─ PathUtils (Node.js path extensions)   │
│  └─types      │  └─ AsyncFileSystem trait                 │
├─────────────────────────────────────────────────────────────┤
│  error/       │  Comprehensive error handling              │
│  ├─types      │  ├─ Domain-specific error types           │
│  ├─recovery   │  ├─ ErrorRecoveryManager                  │
│  └─traits     │  └─ Error context and recovery traits     │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Principles

- **🔄 Async-First**: All I/O operations use async/await for optimal performance
- **🛡️ Error Safety**: Comprehensive error handling with recovery strategies
- **🔧 Configuration-Driven**: Flexible configuration system with multiple sources
- **🌍 Cross-Platform**: Full support for Windows, macOS, and Linux
- **📦 Modular Design**: Clean separation of concerns between modules
- **🚀 Performance**: Optimized for large monorepos and complex workflows

## 🛡️ Error Handling

The crate provides comprehensive error handling with structured error types and recovery strategies:

```rust
use sublime_standard_tools::error::{Error, FileSystemError, CommandError, MonorepoError};

// All errors implement Display and Error traits
match some_operation() {
    Ok(result) => println!("Success: {:?}", result),
    Err(Error::FileSystem(FileSystemError::NotFound { path })) => {
        eprintln!("File not found: {}", path.display());
    }
    Err(Error::Command(CommandError::Timeout { duration })) => {
        eprintln!("Command timed out after {:?}", duration);
    }
    Err(Error::Monorepo(MonorepoError::ManagerNotFound)) => {
        eprintln!("No package manager detected");
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## 🚀 Real-World Examples

### Complete Monorepo Analysis Tool

```rust
use sublime_standard_tools::{
    project::ProjectDetector,
    monorepo::MonorepoDetector,
    command::{CommandBuilder, DefaultCommandExecutor, Executor, CommandQueue, CommandPriority},
    config::{ConfigManager, StandardConfig},
    filesystem::{FileSystemManager, AsyncFileSystem},
    error::Result,
};
use std::path::Path;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Monorepo Analysis Tool");
    
    // Load custom configuration
    let config_manager = ConfigManager::<StandardConfig>::builder()
        .with_defaults()
        .with_file("repo.config.toml")
        .with_env_prefix("SUBLIME")
        .build()?;
    
    let config = config_manager.load().await?;
    println!("🔧 Loaded configuration with {} workspace patterns", 
             config.monorepo.workspace_patterns.len());

    // Initialize filesystem and command executor
    let fs = FileSystemManager::new_with_project_config(Path::new(".")).await?;
    let executor = DefaultCommandExecutor::new();
    let mut command_queue = CommandQueue::new().start()?;

    // Detect project type
    let project_detector = ProjectDetector::new();
    let project = project_detector.detect(Path::new("."), None).await?;
    
    println!("📦 Detected {} project", project.as_project_info().kind().name());
    
    // Analyze monorepo if applicable
    if project.as_project_info().kind().is_monorepo() {
        let monorepo_detector = MonorepoDetector::new();
        let monorepo = monorepo_detector.detect_monorepo(".").await?;
        
        println!("🏗️  Monorepo Analysis Results:");
        println!("   Type: {}", monorepo.kind().name());
        println!("   Root: {}", monorepo.root().display());
        println!("   Packages: {}", monorepo.packages().len());
        
        // Analyze dependencies
        let graph = monorepo.get_dependency_graph();
        println!("📊 Dependency Graph:");
        for (pkg_name, deps) in graph {
            println!("   {} → {} workspace dependencies", pkg_name, deps.len());
            for dep in deps {
                println!("     ├─ {} ({})", dep.name, dep.version_requirement);
            }
        }
        
        // Queue analysis commands for each package
        let mut command_handles = Vec::new();
        
        for package in monorepo.packages() {
            println!("🔍 Queuing analysis for package: {}", package.name);
            
            // Package size analysis
            let size_cmd = CommandBuilder::new("du")
                .args(["-sh", &package.absolute_path.to_string_lossy()])
                .timeout(Duration::from_secs(10))
                .build();
            
            let size_id = command_queue.enqueue(size_cmd, CommandPriority::Normal).await?;
            
            // Check for tests
            let test_check_cmd = CommandBuilder::new("find")
                .args([
                    &package.absolute_path.to_string_lossy(),
                    "-name", "*.test.*", "-o", "-name", "*.spec.*"
                ])
                .timeout(Duration::from_secs(5))
                .build();
            
            let test_id = command_queue.enqueue(test_check_cmd, CommandPriority::Low).await?;
            
            command_handles.push((package.name.clone(), size_id, test_id));
        }
        
        // Wait for all analysis commands to complete
        println!("⏳ Waiting for analysis to complete...");
        
        for (pkg_name, size_id, test_id) in command_handles {
            // Get package size
            match command_queue.wait_for_command(&size_id, Duration::from_secs(15)).await {
                Ok(result) if result.status.success() => {
                    let size = result.output.stdout().trim();
                    println!("📏 {}: {}", pkg_name, size);
                }
                Ok(result) => {
                    eprintln!("❌ Size analysis failed for {}: {}", 
                             pkg_name, result.output.stderr());
                }
                Err(e) => {
                    eprintln!("💥 Error analyzing size for {}: {}", pkg_name, e);
                }
            }
            
            // Check test coverage
            match command_queue.wait_for_command(&test_id, Duration::from_secs(10)).await {
                Ok(result) if result.status.success() => {
                    let test_files = result.output.stdout().lines().count();
                    if test_files > 0 {
                        println!("🧪 {}: {} test files found", pkg_name, test_files);
                    } else {
                        println!("⚠️  {}: No test files found", pkg_name);
                    }
                }
                Ok(_) | Err(_) => {
                    println!("❓ {}: Test analysis inconclusive", pkg_name);
                }
            }
        }
        
        // Generate summary report
        println!("\n📋 Analysis Summary:");
        println!("   Total packages: {}", monorepo.packages().len());
        
        // Check for common files across packages
        let mut package_json_count = 0;
        let mut typescript_count = 0;
        
        for package in monorepo.packages() {
            if fs.exists(&package.absolute_path.join("package.json")).await? {
                package_json_count += 1;
            }
            if fs.exists(&package.absolute_path.join("tsconfig.json")).await? {
                typescript_count += 1;
            }
        }
        
        println!("   Packages with package.json: {}", package_json_count);
        println!("   TypeScript packages: {}", typescript_count);
        
        // Workspace dependency analysis
        let total_workspace_deps: usize = monorepo.packages()
            .iter()
            .map(|p| p.dependencies.len() + p.dev_dependencies.len())
            .sum();
        
        println!("   Total workspace dependencies: {}", total_workspace_deps);
        
    } else {
        println!("📄 This is a simple Node.js project");
        
        // Analyze simple project
        let info = project.as_project_info();
        if let Some(pm) = info.package_manager() {
            println!("   Package manager: {}", pm.kind().command());
            
            // Check for common files
            if fs.exists(Path::new("package.json")).await? {
                let package_json = fs.read_to_string(Path::new("package.json")).await?;
                let parsed: serde_json::Value = serde_json::from_str(&package_json)?;
                
                if let Some(name) = parsed.get("name").and_then(|n| n.as_str()) {
                    println!("   Package name: {}", name);
                }
                
                if let Some(version) = parsed.get("version").and_then(|v| v.as_str()) {
                    println!("   Version: {}", version);
                }
            }
        }
    }
    
    // Cleanup
    command_queue.shutdown().await?;
    println!("✅ Analysis complete!");
    
    Ok(())
}
```

### Development Workflow Automation

```rust
use sublime_standard_tools::{
    project::ProjectDetector,
    command::{CommandBuilder, CommandQueue, CommandPriority, DefaultCommandExecutor, Executor},
    error::Result,
};
use std::path::Path;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🛠️  Starting Development Workflow");
    
    // Detect project and set up command queue
    let detector = ProjectDetector::new();
    let project = detector.detect(Path::new("."), None).await?;
    let mut queue = CommandQueue::new().start()?;
    
    let info = project.as_project_info();
    println!("🏗️  Working with {} project", info.kind().name());
    
    // Define workflow commands
    let commands = if let Some(pm) = info.package_manager() {
        let pm_cmd = pm.kind().command();
        vec![
            ("install", vec![pm_cmd, "install"], CommandPriority::High),
            ("lint", vec![pm_cmd, "run", "lint"], CommandPriority::Normal),
            ("test", vec![pm_cmd, "test"], CommandPriority::Normal),
            ("build", vec![pm_cmd, "run", "build"], CommandPriority::Low),
        ]
    } else {
        vec![
            ("install", vec!["npm", "install"], CommandPriority::High),
            ("test", vec!["npm", "test"], CommandPriority::Normal),
        ]
    };
    
    // Queue all commands
    let mut command_ids = Vec::new();
    
    for (name, args, priority) in commands {
        println!("📝 Queuing: {}", name);
        
        let cmd = CommandBuilder::new(args[0])
            .args(&args[1..])
            .timeout(Duration::from_secs(300))
            .build();
        
        let id = queue.enqueue(cmd, priority).await?;
        command_ids.push((name, id));
    }
    
    // Monitor execution
    println!("🚀 Executing workflow...");
    
    for (name, id) in command_ids {
        println!("⏳ Waiting for: {}", name);
        
        match queue.wait_for_command(&id, Duration::from_secs(360)).await {
            Ok(result) if result.status.success() => {
                println!("✅ {} completed successfully", name);
            }
            Ok(result) => {
                println!("❌ {} failed with exit code: {:?}", 
                        name, result.status.code());
                eprintln!("Error output: {}", result.output.stderr());
            }
            Err(e) => {
                println!("💥 {} failed with error: {}", name, e);
            }
        }
    }
    
    queue.shutdown().await?;
    println!("🎉 Development workflow completed!");
    
    Ok(())
}
```

## 📚 API Reference

### Quick Reference

| Module | Main Types | Purpose |
|--------|------------|---------|
| `config` | `ConfigManager`, `StandardConfig` | Configuration management |
| `project` | `ProjectDetector`, `ProjectInfo` | Project detection and management |
| `node` | `PackageManager`, `RepoKind` | Node.js abstractions |
| `monorepo` | `MonorepoDetector`, `WorkspacePackage` | Monorepo analysis |
| `command` | `CommandExecutor`, `CommandQueue` | Command execution |
| `filesystem` | `FileSystemManager`, `PathExt` | Filesystem operations |
| `error` | `Error`, `ErrorRecoveryManager` | Error handling |

## 📖 Complete API Specification

For comprehensive technical documentation including detailed API signatures, trait definitions, configuration options, and implementation examples, see the [**API Specification**](SPEC.md).

The SPEC.md file provides:
- **Complete API Documentation** - Every public method and type
- **Detailed Configuration Reference** - All configuration options and environment variables
- **Comprehensive Examples** - Working code examples for every module
- **Architecture Overview** - Design patterns and best practices
- **Error Handling Guide** - Complete error types and recovery strategies

### Common Patterns

#### Initialize with Configuration
```rust
let detector = MonorepoDetector::new_with_project_config(Path::new(".")).await?;
let fs = FileSystemManager::new_with_project_config(Path::new(".")).await?;
let executor = DefaultCommandExecutor::new_with_project_config(Path::new(".")).await?;
```

#### Error Handling Pattern
```rust
match operation() {
    Ok(result) => { /* handle success */ }
    Err(Error::FileSystem(fs_err)) => { /* handle filesystem errors */ }
    Err(Error::Command(cmd_err)) => { /* handle command errors */ }
    Err(e) => { /* handle other errors */ }
}
```

#### Async Operations Pattern  
```rust
let handles: Vec<_> = items.into_iter().map(|item| {
    tokio::spawn(async move { process_item(item).await })
}).collect();

for handle in handles {
    let result = handle.await??;
    // Process result
}
```

## 🔧 Troubleshooting

### Common Issues

#### Permission Denied Errors
```bash
# Linux/macOS: Check file permissions
ls -la package.json

# Windows: Run as administrator or check file attributes
```

#### Command Timeout Issues
Set longer timeouts in configuration:
```toml
[commands]
default_timeout = "300s"  # 5 minutes

[commands.timeout_overrides]
"npm install" = "600s"    # 10 minutes for installs
```

#### Memory Issues with Large Monorepos
Adjust concurrency settings:
```toml
[commands]
max_concurrent_commands = 2  # Reduce concurrent commands

[filesystem.async_io]  
max_concurrent_operations = 5  # Reduce concurrent I/O
```

### Environment Variables for Debugging
```bash
export RUST_LOG=sublime_standard_tools=debug
export SUBLIME_COMMAND_TIMEOUT=600
export SUBLIME_MAX_CONCURRENT=2
```

## 🤝 Contributing

Contributions are welcome! Please read our Contributing Guidelines and Code of Conduct in the repository.

### Development Setup

```bash
git clone https://github.com/websublime/workspace-node-tools.git
cd workspace-node-tools/crates/standard
cargo test --all-features
```

### Running Examples
```bash
# Run real-world usage tests
cargo test real_world_usage --features full -- --nocapture

# Run specific module tests  
cargo test filesystem::tests --features full
cargo test monorepo::tests --features full
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/websublime/workspace-node-tools/blob/main/LICENSE) file for details.

## 🔗 Related Projects

- [sublime-package-tools](../pkg/) - Package management and dependency analysis
- [sublime-git-tools](../git/) - Git repository management  
- [sublime-monorepo-tools](../monorepo/) - Advanced monorepo workflow automation

---

<div align="center">

**Built with ❤️ by the [Websublime](https://github.com/websublime) team**

[Documentation](https://docs.rs/sublime_standard_tools) •
[Crates.io](https://crates.io/crates/sublime_standard_tools) •
[Repository](https://github.com/websublime/workspace-node-tools)

</div>