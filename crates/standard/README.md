# `sublime_standard_tools` - Node.js Project Utilities

This crate provides a collection of utilities for working with Node.js projects, including command execution, package manager detection, project root path discovery, and more.

## Overview

`sublime_standard_tools` offers a robust set of tools for interacting with Node.js projects from Rust applications. It enables seamless integration with Node.js ecosystems, allowing Rust applications to:

- Execute shell commands with proper error handling and output parsing
- Detect which package manager (npm, yarn, pnpm, or bun) is being used in a project
- Find the root of a project by locating package manager lock files
- Handle common string manipulations needed for command outputs

## Main Features

### Command Execution

Execute external commands with robust error handling and customizable output processing:

```rust
use sublime_standard_tools::{execute, ComandResult};

// Execute a git command and process its output
let version = execute("git", ".", ["--version"], |output, _| {
    Ok(output.to_string())
})?;

println!("Git version: {}", version);
```

### Package Manager Detection

Automatically detect which Node.js package manager is being used in a project:

```rust
use sublime_standard_tools::{detect_package_manager, CorePackageManager};
use std::path::Path;

let project_dir = Path::new("./my-node-project");
match detect_package_manager(project_dir) {
    Some(CorePackageManager::Npm) => println!("Using npm"),
    Some(CorePackageManager::Yarn) => println!("Using yarn"),
    Some(CorePackageManager::Pnpm) => println!("Using pnpm"),
    Some(CorePackageManager::Bun) => println!("Using bun"),
    None => println!("No package manager detected"),
}
```

### Project Root Discovery

Find the root directory of a Node.js project by locating package manager lock files:

```rust
use sublime_standard_tools::get_project_root_path;

// Find the project root from the current directory
if let Some(root_path) = get_project_root_path(None) {
    println!("Project root: {}", root_path.display());
}
```

## Modules

### `command` Module

This module provides functionality for executing external commands with proper error handling.

#### Key Functions

- `execute<P, I, F, S, R>(cmd: S, path: P, args: I, process: F) -> Result<R, CommandError>` - Executes a command with the specified arguments in the given directory.

#### Examples

```rust
use sublime_standard_tools::{execute, ComandResult};

fn get_git_commit_count() -> ComandResult<usize> {
    execute("git", ".", ["rev-list", "--count", "HEAD"], |output, _| {
        output.trim().parse::<usize>().map_err(|_| CommandError::Execution)
    })
}
```

### `error` Module

Provides error types used throughout the crate, primarily for command execution errors.

#### Key Types

- `CommandError` - Represents errors that can occur when executing commands:
  - `Run` - The command failed to start (e.g., command not found)
  - `Execution` - The command execution process failed
  - `Failure` - The command executed but returned a non-zero exit code

### `manager` Module

Provides functionality for detecting and working with Node.js package managers.

#### Key Types

- `CorePackageManager` - Enum representing supported package managers (Npm, Yarn, Pnpm, Bun)
- `CorePackageManagerError` - Error type for package manager operations

#### Key Functions

- `detect_package_manager(path: &Path) -> Option<CorePackageManager>` - Detects which package manager is being used in a project by examining lock files

#### Examples

```rust
use sublime_standard_tools::{detect_package_manager, CorePackageManager};
use std::path::Path;

let path = Path::new("./my-project");
if let Some(manager) = detect_package_manager(path) {
    match manager {
        CorePackageManager::Npm => println!("Running npm install..."),
        CorePackageManager::Yarn => println!("Running yarn install..."),
        CorePackageManager::Pnpm => println!("Running pnpm install..."),
        CorePackageManager::Bun => println!("Running bun install..."),
    }
}
```

### `path` Module

Provides utilities for locating project root directories and navigating project structures.

#### Key Functions

- `get_project_root_path(root: Option<PathBuf>) -> Option<PathBuf>` - Finds the root directory of a project by looking for package manager lock files

### `utils` Module

Provides general utility functions.

#### Key Functions

- `strip_trailing_newline(input: &String) -> String` - Removes trailing newline characters from a string

## Error Handling

The crate uses a custom error type (`CommandError`) that provides detailed information about command execution failures:

```rust
match execute("git", ".", ["status"], |out, _| Ok(out.to_string())) {
    Ok(status) => println!("Git status: {}", status),
    Err(CommandError::Run(e)) => eprintln!("Failed to run git: {}", e),
    Err(CommandError::Execution) => eprintln!("Git command execution failed"),
    Err(CommandError::Failure { stdout, stderr }) => {
        eprintln!("Git command failed with output: {}", stdout);
        eprintln!("Error: {}", stderr);
    }
}
```

## Type Aliases

- `ComandResult<T>` - A type alias for `Result<T, CommandError>` to simplify function signatures for command execution

## Implementation Details

### Command Execution

Commands are executed using Rust's `std::process::Command` with:
- Working directory set to the specified path
- Captured stdout and stderr
- Proper error handling for all failure scenarios

### Package Manager Detection

Package managers are detected by checking for the presence of specific lock files:
- `package-lock.json` or `npm-shrinkwrap.json` for npm
- `yarn.lock` for Yarn
- `pnpm-lock.yaml` for pnpm
- `bun.lockb` for Bun

### Project Root Detection

The project root is detected by walking up the directory tree from the current directory (or a specified directory) until a package manager lock file is found.

## Cross-Platform Support

The crate is designed to work on all major platforms:
- Windows
- macOS
- Linux

Platform-specific differences (like file path separators and newline characters) are properly handled.

## License

This crate is licensed under the terms specified in the workspace configuration.