---
type: Page
title: '`sublime_standard_tools` Crate Guide'
description: '`sublime_standard_tools` is a utility crate for working with Node.js projects from Rust applications, providing robust command execution, package manager detection, project structure navigation, and common helper functions.'
icon: 🔧
createdAt: '2025-04-09T23:26:44.668Z'
creationDate: 2025-04-10 00:26
modificationDate: 2025-04-10 00:33
tags: [rust, workspace-tools]
coverImage: null
---

# `sublime_standard_tools` Crate Guide

[Workspace Tools](https://app.capacities.io/f8d3f900-cfe2-4c6d-b44c-906946be3e3c/52c8c019-b006-462b-8c5d-6b518afa0d64)

## Core Features

- **Command Execution**: Run external commands with standardized error handling

- **Package Manager Support**: Detect and work with npm, yarn, pnpm, and bun

- **Project Navigation**: Find project roots by examining lock files

- **Output Processing**: Process command output with custom handlers

## Installation

Add the crate to your `Cargo.toml`:

```text
[dependencies]
sublime_standard_tools = "0.1.0"
```

## Usage Examples

### Command Execution

```rust
use sublime_standard_tools::{execute, ComandResult};
use std::process::Output;
// Simple command execution
let git_version = execute("git", ".", ["--version"], |output, _| {
    Ok(output.to_string())
})?;
println!("Git version: {}", git_version);
// Executing npm commands
let dependencies = execute("npm", ".", ["list", "--json"], |output, _| {
    // Parse JSON output from npm
    let parsed: serde_json::Value = serde_json::from_str(output)?;
    
    // Extract dependency names
    let deps = parsed["dependencies"]
        .as_object()
        .map(|deps| deps.keys().cloned().collect())
        .unwrap_or_default();
        
    Ok::<_, serde_json::Error>(deps)
})?;
println!("Project dependencies: {:?}", dependencies);
// Command with custom error handling
let result = execute("node", ".", ["unavailable.js"], |output, proc_output| {
    // Check for specific error messages
    if output.contains("Error: Cannot find module") {
        println!("Module not found, installing...");
        return Ok("Installing module".to_string());
    }
    Ok(output.to_string())
});
// Handle command errors
match result {
    Ok(output) => println!("Success: {}", output),
    Err(err) => match err {
        sublime_standard_tools::CommandError::Failure { stdout, stderr } => {
            println!("Command failed with:\nOutput: {}\nError: {}", stdout, stderr);
        }
        _ => println!("Other error: {}", err),
    }
}
```

### Detecting Package Managers

```rust
use sublime_standard_tools::{detect_package_manager, CorePackageManager};
use std::path::Path;
// Detect package manager from a specific directory
let project_path = Path::new("./my-node-project");
let package_manager = detect_package_manager(project_path);
match package_manager {
    Some(CorePackageManager::Npm) => {
        println!("This project uses npm");
        // Execute npm-specific commands
        execute("npm", project_path, ["install"], |_, _| Ok(()))?;
    }
    Some(CorePackageManager::Yarn) => {
        println!("This project uses yarn");
        execute("yarn", project_path, ["install"], |_, _| Ok(()))?;
    }
    Some(CorePackageManager::Pnpm) => {
        println!("This project uses pnpm");
        execute("pnpm", project_path, ["install"], |_, _| Ok(()))?;
    }
    Some(CorePackageManager::Bun) => {
        println!("This project uses bun");
        execute("bun", project_path, ["install"], |_, _| Ok(()))?;
    }
    None => {
        println!("No package manager detected, defaulting to npm");
        execute("npm", project_path, ["install"], |_, _| Ok(()))?;
    }
}
// Converting strings to package manager enums
use std::convert::TryFrom;
let pm = CorePackageManager::try_from("yarn");
if let Ok(manager) = pm {
    println!("Using {}", manager); // Prints "Using yarn"
}
```

### Finding Project Roots

```rust
use sublime_standard_tools::get_project_root_path;
use std::path::PathBuf;
// Get the project root from the current directory
let root = get_project_root_path(None);
if let Some(path) = root {
    println!("Project root: {}", path.display());
    
    // Check if important project files exist at the root
    let package_json = path.join("package.json");
    if package_json.exists() {
        println!("Found package.json at project root");
    }
    
    let tsconfig = path.join("tsconfig.json");
    if tsconfig.exists() {
        println!("Found TypeScript configuration");
    }
}
// Find project root from a specific subdirectory
let specific_path = PathBuf::from("./src/components");
let project_root = get_project_root_path(Some(specific_path));
println!("Project root from subdirectory: {}", 
         project_root.unwrap_or_default().display());
```

### Building Complete Tools

```rust
use sublime_standard_tools::{
    execute, ComandResult, detect_package_manager, 
    CorePackageManager, get_project_root_path, strip_trailing_newline
};
use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Find the project root
    let root_path = get_project_root_path(None)
        .ok_or("Failed to find project root")?;
    println!("Project root: {}", root_path.display());
    
    // Detect the package manager
    let package_manager = detect_package_manager(&root_path)
        .unwrap_or(CorePackageManager::Npm);
    println!("Using package manager: {}", package_manager);
    
    // Build the appropriate install command
    let (cmd, args) = match package_manager {
        CorePackageManager::Npm => ("npm", vec!["install"]),
        CorePackageManager::Yarn => ("yarn", vec!["install"]),
        CorePackageManager::Pnpm => ("pnpm", vec!["install"]),
        CorePackageManager::Bun => ("bun", vec!["install"]),
    };
    
    // Execute the install command
    execute(cmd, &root_path, args, |output, _| {
        println!("Dependencies installed successfully");
        Ok(())
    })?;
    
    // Run a script defined in package.json
    let script_name = "build";
    let script_args = match package_manager {
        CorePackageManager::Npm | CorePackageManager::Pnpm => 
            vec!["run", script_name],
        CorePackageManager::Yarn => 
            vec![script_name],
        CorePackageManager::Bun => 
            vec!["run", script_name],
    };
    
    execute(package_manager.to_string().as_str(), &root_path, script_args, |output, _| {
        println!("Build output:\n{}", output);
        Ok(())
    })?;
    
    Ok(())
}
```

## Key Concepts

### Command Execution Pattern

The `execute` function provides a flexible pattern for running external commands:

```rust
execute(command, working_directory, arguments, output_processor)
```

The output processor is a closure that receives the command's stdout as a string and the complete `Output` struct, allowing you to handle the output however you need:

```rust
let result = execute("ls", ".", ["-la"], |stdout, output| {
    // Process the output
    let files: Vec<&str> = stdout.lines().collect();
    Ok(files)
})?;
```

### Error Handling

The crate uses a specialized `CommandError` enum for robust error handling:

```rust
match execute("ls", ".", ["non_existent_dir"], |stdout, _| Ok(stdout.to_string())) {
    Ok(output) => println!("Success: {}", output),
    Err(err) => match err {
        CommandError::Run(io_err) => {
            println!("Failed to run command: {}", io_err);
        },
        CommandError::Execution => {
            println!("Command execution failed");
        },
        CommandError::Failure { stdout, stderr } => {
            println!("Command returned non-zero exit code");
            println!("stdout: {}", stdout);
            println!("stderr: {}", stderr);
        }
    }
}
```

### Working with Package Managers

The `CorePackageManager` enum represents the supported package managers with useful methods:

```rust
// Convert a string to a package manager enum
let manager_str = "pnpm";
if let Ok(manager) = CorePackageManager::try_from(manager_str) {
    // Comparing package managers
    if manager == CorePackageManager::Pnpm {
        println!("Using pnpm!");
    }
    
    // Converting back to string for command execution
    let install_cmd = format!("{} install", manager);
    println!("Install command: {}", install_cmd);
}
```

## Best Practices

1. **Always handle command failures**: The `execute` function captures both stdout and stderr when a command fails, which provides valuable debugging information.

```rust
match execute("npm", ".", ["install"], |_, _| Ok(())) {
    Ok(_) => println!("Installation successful"),
    Err(CommandError::Failure { stdout, stderr }) => {
        eprintln!("Installation failed!");
        eprintln!("Output: {}", stdout);
        eprintln!("Error: {}", stderr);
    }
    Err(err) => eprintln!("Error: {}", err),
}
```

1. **Use the appropriate package manager commands**: Different package managers require different commands. Use the `CorePackageManager` enum to determine the correct commands.

```rust
fn get_script_args(manager: CorePackageManager, script: &str) -> Vec<&str> {
    match manager {
        CorePackageManager::Npm | CorePackageManager::Pnpm | CorePackageManager::Bun => 
            vec!["run", script],
        CorePackageManager::Yarn => 
            vec![script],
    }
}
```

1. **Process command output carefully**: The `strip_trailing_newline` utility helps clean up command output.

```rust
let version = execute("node", ".", ["--version"], |output, _| {
    let cleaned = strip_trailing_newline(&output.to_string());
    Ok(cleaned)
})?;
```

This crate provides a solid foundation for any Rust application that needs to interact with Node.js projects and their package managers, making it ideal for building developer tools, CI/CD pipelines, or cross-language bridges.

