---
type: Definition
title: API Specification for `sublime_standard_tools` Crate
tags: [workspace-tools, rust]
---

# API Specification for `sublime_standard_tools` Crate

[Workspace Tools](https://app.capacities.io/f8d3f900-cfe2-4c6d-b44c-906946be3e3c/52c8c019-b006-462b-8c5d-6b518afa0d64)

## Overview

The `sublime_standard_tools` crate provides foundational utilities for Node.js tooling in Rust. It includes command execution, package manager detection, path resolution, and error handling functionalities to support operations on Node.js projects.

## Architecture

This crate is designed with a modular approach, providing independent but interconnected utilities for common Node.js tooling tasks:

1. **Command Execution** - Safe execution of external commands with proper error handling

2. **Package Manager Support** - Detection and representation of JavaScript package managers

3. **Project Path Resolution** - Utilities to locate and work with project root directories

4. **Error Handling** - Comprehensive error types with proper context information

## Crate Details

- **Name**: `sublime_standard_tools`

- **Version**: 0.1.0

- **Dependencies**:

    - `thiserror` - For structured error handling

    - `serde` with `derive` feature - For serialization/deserialization support

## Public API

### Command Execution

#### Types

```rust
pub type ComandResult<T> = Result<T, CommandError>;
```

A type alias for command execution results with appropriate error handling.

#### Functions

```rust
pub fn execute<P, I, F, S, R>(
    cmd: S, 
    path: P, 
    args: I, 
    process: F
) -> Result<R, CommandError>
```

Executes an external command with the provided arguments in a specified directory.

- **Parameters**:

    - `cmd`: The command to execute (e.g., "git", "npm")

    - `path`: Directory in which to execute the command

    - `args`: Iterator of arguments to pass to the command

    - `process`: Function to process the command's output

- **Returns**: Result containing either the processed output or a `CommandError`

- **Example**:

    ```rust
    let git_version = execute("git", ".", ["--version"], |message, _output| {
        Ok(message.to_string())
    })?;
    ```

### Error Handling

#### Types

```rust
pub enum CommandError { ... }
```

Represents errors that may occur when executing commands.

- **Variants**:

    - `Run(std::io::Error)` - IO errors preventing command execution

    - `Execution` - Command execution process failed

    - `Failure { stdout: String, stderr: String }` - Command returned non-zero exit code

- **Traits Implemented**:

    - `Error`, `Debug`, `Clone`, `AsRef<str>`

### Package Manager Handling

#### Types

```rust
pub enum CorePackageManager { Npm, Yarn, Pnpm, Bun }
```

Represents supported package managers for JavaScript/TypeScript projects.

- **Traits Implemented**:

    - `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`, `Deserialize`, `Display`

    - `TryFrom<&str>`, `TryFrom<String>`

```rust
pub enum CorePackageManagerError { ... }
```

Error for package manager parsing operations.

- **Variants**:

    - `ParsePackageManagerError(String)` - Occurs when trying to parse an unsupported manager name

#### Functions

```rust
pub fn detect_package_manager(path: &Path) -> Option<CorePackageManager>
```

Detects the package manager used in a project by examining lock files.

- **Parameters**:

    - `path`: Directory to check for package manager lock files

- **Returns**: The detected package manager or `None` if no manager is detected

- **Example**:

    ```rust
    if let Some(manager) = detect_package_manager(Path::new("/path/to/project")) {
        println!("Using package manager: {}", manager);
    }
    ```

### Path Utilities

```rust
pub fn get_project_root_path(root: Option<PathBuf>) -> Option<PathBuf>
```

Attempts to determine the root path of the current project by looking for package manager lock files.

- **Parameters**:

    - `root`: Optional starting directory (if `None`, uses current working directory)

- **Returns**: Path to the detected project root or the current directory if no root is found

- **Example**:

    ```rust
    let project_root = get_project_root_path(None).unwrap();
    ```

### Utility Functions

```rust
pub fn strip_trailing_newline(input: &String) -> String
```

Removes trailing newline characters from a string.

- **Parameters**:

    - `input`: String from which to remove trailing newlines

- **Returns**: A new string with trailing newlines removed

- **Example**:

    ```rust
    let clean_output = strip_trailing_newline(&command_output);
    ```

## Implementation Details

### Command Execution

The command execution module provides a safe and consistent way to execute external commands:

1. It canonicalizes the provided path to ensure absolute paths are used

2. Captures both stdout and stderr for comprehensive error handling

3. Provides a callback mechanism to process command output

4. Handles UTF-8 decoding of command output

### Package Manager Detection

The package manager detection works by:

1. Looking for specific lock files in the provided directory:

    - `package-lock.json` or `npm-shrinkwrap.json` for npm

    - `yarn.lock` for Yarn

    - `pnpm-lock.yaml` for pnpm

    - `bun.lockb` for Bun

2. If no lock file is found, recursively checking parent directories

3. Returning the corresponding `CorePackageManager` enum variant when found

### Project Root Detection

The project root detection works by:

1. Starting from either the provided directory or current working directory

2. Walking up the directory tree looking for package manager lock files

3. Returning the first directory that contains a lock file, or the current directory if none is found

## Usage Examples

### Executing Git Commands

```rust
use sublime_standard_tools::{execute, CommandError};
fn get_git_version() -> Result<String, CommandError> {
    execute("git", ".", ["--version"], |message, _output| {
        Ok(message.to_string())
    })
}
```

### Detecting Package Manager

```rust
use sublime_standard_tools::{detect_package_manager, CorePackageManager};
use std::path::Path;
fn use_appropriate_install_command(project_dir: &Path) -> String {
    match detect_package_manager(project_dir) {
        Some(CorePackageManager::Npm) => "npm install",
        Some(CorePackageManager::Yarn) => "yarn",
        Some(CorePackageManager::Pnpm) => "pnpm install",
        Some(CorePackageManager::Bun) => "bun install",
        None => "npm install", // Default to npm
    }.to_string()
}
```

### Finding Project Root

```rust
use sublime_standard_tools::get_project_root_path;
use std::path::PathBuf;
fn open_package_json() -> std::io::Result<()> {
    let project_root = get_project_root_path(None).unwrap_or(PathBuf::from("."));
    let package_json_path = project_root.join("package.json");
    
    // Open and parse package.json
    // ...
    
    Ok(())
}
```

## Limitations and Considerations

1. **Package Manager Detection**: Only detects package managers with lock files present in the directory tree. Projects without lock files will not have a detectable package manager.

2. **Command Execution**:

    - Requires commands to be in the system PATH

    - Assumes command output is valid UTF-8

    - May not handle very large command outputs efficiently

3. **Project Root Detection**:

    - Relies on the presence of package manager lock files

    - Will fall back to the current directory if no project root is detected

## Future Enhancements

Potential areas for expansion:

1. Support for more package managers as they gain adoption

2. Enhanced command execution options (timeouts, environment variables, etc.)

3. Additional project structure detection mechanisms beyond lock files

## Testing

The crate includes comprehensive tests for:

- Command execution

- Package manager detection and parsing

- Project path resolution

Tests use a combination of real system commands (e.g., git) and temporary directory structures to validate functionality.

### Integration with sublime_standard_tools

