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

## 🏃 Quick Start

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

### Working with Monorepos

```rust
use sublime_standard_tools::monorepo::MonorepoDetector;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let detector = MonorepoDetector::new();
    
    if let Some(kind) = detector.is_monorepo_root(".")? {
        println!("This directory is a {} monorepo", kind.name());
        
        // Analyze the monorepo structure
        let monorepo = detector.detect_monorepo(".").await?;
        
        println!("Monorepo contains {} packages:", monorepo.packages().len());
        for package in monorepo.packages() {
            println!("- {} v{} at {}", 
                     package.name, 
                     package.version, 
                     package.location.display());
        }
        
        // Generate dependency graph
        let graph = monorepo.get_dependency_graph();
        println!("Dependency analysis:");
        for (package, deps) in graph {
            println!("  {} has {} workspace dependencies", package, deps.len());
        }
    }
    
    Ok(())
}
```

### Command Execution with Queuing

```rust
use sublime_standard_tools::command::{
    CommandBuilder, CommandQueue, CommandPriority, DefaultCommandExecutor
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a command queue with concurrent execution
    let mut queue = CommandQueue::new().start()?;
    
    // Build commands
    let build_cmd = CommandBuilder::new("npm")
        .arg("run")
        .arg("build")
        .timeout(Duration::from_secs(60))
        .build();
        
    let test_cmd = CommandBuilder::new("npm")
        .arg("test")
        .timeout(Duration::from_secs(30))
        .build();
    
    // Enqueue commands with different priorities
    let build_id = queue.enqueue(build_cmd, CommandPriority::High).await?;
    let test_id = queue.enqueue(test_cmd, CommandPriority::Normal).await?;
    
    // Wait for completion
    let build_result = queue.wait_for_command(&build_id, Duration::from_secs(120)).await?;
    let test_result = queue.wait_for_command(&test_id, Duration::from_secs(60)).await?;
    
    println!("Build result: {:?}", build_result.status);
    println!("Test result: {:?}", test_result.status);
    
    queue.shutdown().await?;
    Ok(())
}
```

### Streaming Command Output

```rust
use sublime_standard_tools::command::{
    CommandBuilder, DefaultCommandExecutor, Executor, StreamConfig, StreamOutput
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executor = DefaultCommandExecutor::new();
    let stream_config = StreamConfig::default();
    
    let cmd = CommandBuilder::new("npm")
        .args(["install", "--verbose"])
        .build();
    
    let (mut stream, _child) = executor.execute_stream(cmd, stream_config).await?;
    
    println!("Streaming npm install output:");
    while let Ok(Some(output)) = stream.next_timeout(Duration::from_secs(1)).await {
        match output {
            StreamOutput::Stdout(line) => println!("📦 {}", line),
            StreamOutput::Stderr(line) => eprintln!("⚠️  {}", line),
            StreamOutput::End => break,
        }
    }
    
    println!("Installation completed!");
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

### Using Configuration in Code

```rust
use sublime_standard_tools::config::{ConfigManager, StandardConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from all sources
    let config_manager = ConfigManager::<StandardConfig>::builder()
        .with_defaults()
        .with_file("repo.config.toml")
        .with_env_prefix("SUBLIME")
        .build()?;

    let config = config_manager.load().await?;
    
    // Use configuration values
    println!("Package manager detection order: {:?}", 
             config.package_managers.detection_order);
    println!("Command timeout: {:?}", config.commands.default_timeout);
    println!("Max search depth: {}", config.monorepo.max_search_depth);
    
    Ok(())
}
```

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
│  filesystem/  │  Safe filesystem operations               │
│  config/      │  Flexible configuration management        │
│  error/       │  Comprehensive error handling             │
└─────────────────────────────────────────────────────────────┘
```

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

## 🚀 Real-World Example: Monorepo Analysis Tool

Here's a complete example of building a monorepo analysis tool:

```rust
use sublime_standard_tools::{
    project::ProjectDetector,
    monorepo::MonorepoDetector,
    command::{CommandBuilder, DefaultCommandExecutor, Executor},
    config::{ConfigManager, StandardConfig},
    error::Result,
};
use std::path::Path;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Load custom configuration
    let config_manager = ConfigManager::<StandardConfig>::builder()
        .with_defaults()
        .with_file("repo.config.toml")
        .with_env_prefix("SUBLIME")
        .build()?;
    
    let config = config_manager.load().await?;
    println!("🔧 Loaded configuration with {} workspace patterns", 
             config.monorepo.workspace_patterns.len());

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
        println!("   Packages: {}", monorepo.packages().len());
        
        // Analyze dependencies
        let graph = monorepo.get_dependency_graph();
        println!("📊 Dependency Graph:");
        for (pkg_name, deps) in graph {
            println!("   {} → {} dependencies", pkg_name, deps.len());
            for dep in deps {
                println!("     ├─ {}", dep.name);
            }
        }
        
        // Run analysis commands
        let executor = DefaultCommandExecutor::new();
        
        // Get package info for each workspace package
        for package in monorepo.packages() {
            println!("🔍 Analyzing package: {}", package.name);
            
            let size_cmd = CommandBuilder::new("du")
                .args(["-sh", &package.absolute_path.to_string_lossy()])
                .timeout(Duration::from_secs(10))
                .build();
            
            match executor.execute(size_cmd).await {
                Ok(output) if output.success() => {
                    println!("   Size: {}", output.stdout().trim());
                }
                Ok(output) => {
                    eprintln!("   Size analysis failed: {}", output.stderr());
                }
                Err(e) => {
                    eprintln!("   Error analyzing size: {}", e);
                }
            }
        }
    }
    
    println!("✅ Analysis complete!");
    Ok(())
}
```

## 📚 More Examples

Check out the examples directory for more comprehensive examples:

- **Project Detection**: Advanced project detection and validation
- **Monorepo Management**: Complete monorepo workspace management
- **Command Execution**: Advanced command execution patterns
- **Configuration**: Custom configuration setups
- **Error Handling**: Comprehensive error handling strategies

## 🤝 Contributing

Contributions are welcome! Please read our Contributing Guidelines and Code of Conduct in the repository.

### Development Setup

```bash
git clone https://github.com/websublime/workspace-node-tools.git
cd workspace-node-tools/crates/standard
cargo test
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