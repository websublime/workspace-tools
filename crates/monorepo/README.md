# Sublime Monorepo Tools

A streamlined library for CLI and daemon consumption that provides essential Node.js monorepo functionality including project management, analysis, versioning, and change detection.

## Features

- **Project Management**: Direct integration with base crates for optimal performance
- **Analysis**: Dependency graph analysis and change detection with < 1s performance
- **Versioning**: Major, Minor, Patch with automatic propagation to dependents  
- **Change Detection**: Efficient package change analysis for CLI operations
- **Configuration**: Streamlined monorepo configuration management
- **Changelogs**: Conventional commits parsing with customizable templates
- **Tasks**: Synchronous task execution optimized for CLI responsiveness

## Core API (12 Types)

The public API is intentionally minimal and focused for CLI/daemon usage:

- **Project Management**: `MonorepoProject`
- **Analysis**: `MonorepoAnalyzer`, `ChangeAnalysis`
- **Configuration**: `Environment`, `MonorepoConfig`, `VersionBumpType`
- **Version Management**: `VersionManager`, `VersioningResult`
- **Change Detection**: `ChangeDetector`, `PackageChange`
- **Error Handling**: `Error`, `Result`

## Quick Start

```rust
use sublime_monorepo_tools::{MonorepoProject, MonorepoAnalyzer, Result};

fn main() -> Result<()> {
    // Initialize project with direct base crate integration
    let project = MonorepoProject::new(".")?;
    
    // Perform analysis with sub-second performance
    let analyzer = MonorepoAnalyzer::new(&project);
    let changes = analyzer.detect_changes_since("HEAD~1", None)?;
    
    println!("Found {} affected packages", changes.package_changes.len());
    for change in &changes.package_changes {
        println!("- {}: {:?}", change.package_name, change.change_type);
    }
    
    Ok(())
}
```

## Project Management

Initialize and work with monorepo projects:

```rust
use sublime_monorepo_tools::{MonorepoProject, MonorepoConfig, Result};

fn example() -> Result<()> {
    // Create a new project instance
    let project = MonorepoProject::new("/path/to/monorepo")?;
    
    // Access project information
    println!("Root path: {}", project.root_path().display());
    println!("Found {} packages", project.packages.len());
    
    // Get specific package information
    if let Some(package) = project.get_package("my-package") {
        println!("Package path: {}", package.path().display());
        println!("Dependencies: {:?}", package.dependencies_internal);
    }
    
    // Work with configuration
    let config = project.config();
    println!("Default environment: {:?}", config.environments.default);
    
    Ok(())
}
```

## Change Analysis

Analyze changes between Git references:

```rust
use sublime_monorepo_tools::{MonorepoProject, MonorepoAnalyzer, Result};

fn analyze_changes() -> Result<()> {
    let project = MonorepoProject::new(".")?;
    let analyzer = MonorepoAnalyzer::new(&project);
    
    // Detect changes since last commit
    let changes = analyzer.detect_changes_since("HEAD~1", None)?;
    
    println!("Analysis from {} to {}", changes.from_ref, changes.to_ref);
    println!("Changed files: {}", changes.changed_files.len());
    
    // Analyze affected packages
    for change in &changes.package_changes {
        println!("Package: {}", change.package_name);
        println!("  Change type: {:?}", change.change_type);
        println!("  Significance: {:?}", change.significance);
        println!("  Suggested bump: {:?}", change.suggested_version_bump);
        println!("  Files changed: {}", change.changed_files.len());
    }
    
    // Check directly vs transitively affected packages
    println!("Directly affected: {:?}", changes.directly_affected);
    println!("Dependents affected: {:?}", changes.dependents_affected);
    
    Ok(())
}
```

## Version Management

Manage package versions with dependency propagation:

```rust
use sublime_monorepo_tools::{MonorepoProject, VersionManager, VersionBumpType, Result};

fn manage_versions() -> Result<()> {
    let mut project = MonorepoProject::new(".")?;
    let mut version_manager = VersionManager::new(&mut project);
    
    // Bump a specific package version
    let result = version_manager.bump_package_version("my-package", VersionBumpType::Minor)?;
    
    println!("Version bump result:");
    println!("  Package: {}", result.package_name);
    println!("  Old version: {}", result.old_version);
    println!("  New version: {}", result.new_version);
    println!("  Propagated to {} dependents", result.propagated_packages.len());
    
    Ok(())
}
```

## Changelog Generation

Generate changelogs from conventional commits:

```rust
use sublime_monorepo_tools::changelog::{ChangelogManager, ChangelogRequest};

fn generate_changelog() -> Result<()> {
    let project = MonorepoProject::new(".")?;
    let manager = ChangelogManager::new(&project);
    
    // Generate changelog for specific package
    let request = ChangelogRequest {
        package_name: Some("my-package".to_string()),
        version: "1.2.0".to_string(),
        since: Some("v1.1.0".to_string()),
        write_to_file: true,
        ..Default::default()
    };
    
    let result = manager.generate_changelog(request)?;
    
    println!("Generated changelog with {} commits", result.commit_count);
    if result.has_breaking_changes {
        println!("⚠️  Contains breaking changes");
    }
    
    if let Some(path) = result.output_path {
        println!("Written to: {}", path);
    }
    
    Ok(())
}
```

## Task Execution

Execute tasks with condition checking:

```rust
use sublime_monorepo_tools::tasks::{TaskManager, TaskDefinition, TaskScope};

fn execute_tasks() -> Result<()> {
    let project = MonorepoProject::new(".")?;
    let mut task_manager = TaskManager::new(&project)?;
    
    // Define a task
    let task = TaskDefinition::new("test", "Run tests for changed packages")
        .with_package_script("test")
        .with_scope(TaskScope::AffectedPackages);
    
    // Register and execute
    task_manager.register_task(task)?;
    let result = task_manager.execute_task("test", None)?;
    
    println!("Task execution completed:");
    println!("  Status: {:?}", result.status);
    println!("  Packages processed: {}", result.stats.packages_processed);
    println!("  Commands executed: {}", result.stats.commands_executed);
    
    Ok(())
}
```

## CLI/Daemon Usage

This library is optimized for CLI and daemon usage patterns:

```rust
// Fast startup and analysis for CLI responsiveness
let project = MonorepoProject::new(".")?;  // < 100ms target
let analyzer = MonorepoAnalyzer::new(&project);
let changes = analyzer.detect_changes_since("HEAD~1", None)?;  // < 1s target

// Direct base crate access (no service abstractions)
let file_system = project.file_system();
let repository = &project.repository;

// Synchronous execution (no async complexity)
let task_manager = TaskManager::new(&project)?;
let result = task_manager.execute_task("build", None)?;  // Sync execution
```

## Advanced Features

Advanced functionality is available through module paths:

```rust
// Changesets for CI/CD integration
use sublime_monorepo_tools::changesets::ChangesetManager;

// Detailed configuration management  
use sublime_monorepo_tools::config::{ConfigManager, WorkspaceConfig};

// Change detection engines
use sublime_monorepo_tools::changes::{ChangeEngine, ChangeDetector};

// Direct access to base crate functionality
use sublime_monorepo_tools::core::MonorepoProject;
```

## Performance

- **Startup Time**: < 100ms for CLI responsiveness
- **Analysis Time**: < 1s for real-time feedback
- **Memory Usage**: Efficient direct borrowing patterns
- **Dependencies**: Minimal footprint, no async overhead

## Architecture

Built on foundational base crates:
- `sublime_git_tools`: Git operations and repository management
- `sublime_standard_tools`: File system, command execution, and monorepo detection  
- `sublime_package_tools`: Package management, dependencies, and version handling

## License

This project is licensed under the MIT License - see the LICENSE file for details.