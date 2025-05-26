# Sublime Monorepo Tools

A comprehensive toolkit for managing JavaScript/TypeScript monorepos with advanced dependency management, task execution, and versioning capabilities.

## Features

- **Workspace Management**
  - Automatic package discovery
  - Dependency graph analysis
  - Cycle detection and visualization
  - Package validation

- **Task System**
  - Define tasks with dependencies
  - Package-specific task execution
  - Task filtering and selection
  - Parallel execution

- **Change Tracking**
  - Track changes across packages
  - Store changes in JSON format
  - Associate changes with packages
  - Mark changes as released

- **Versioning**
  - Version bump suggestions based on changes
  - Multiple versioning strategies
  - Dependency-aware version bumps
  - Cycle-aware version harmonization
  - Changelog generation

## Getting Started

```rust
use sublime_monorepo_tools::{
    DiscoveryOptions, Task, TaskRunner, Workspace, WorkspaceManager
};

// Discover a workspace
let manager = WorkspaceManager::new();
let options = DiscoveryOptions::default();
let workspace = manager.discover_workspace(".", &options)?;

// Create a task runner
let mut runner = TaskRunner::new(&workspace);

// Define tasks
runner.add_tasks(vec![
    Task::new("build", "npm run build"),
    Task::new("test", "npm test").with_dependency("build"),
]);

// Run tasks
let results = runner.run_task("test")?;
```

## Task System

Define and execute tasks with dependencies:

```rust
use sublime_monorepo_tools::{Task, TaskFilter, TaskRunner};

// Define tasks
let build_task = Task::new("build", "npm run build")
    .with_package("ui")
    .with_cwd("packages/ui");

let test_task = Task::new("test", "npm test")
    .with_dependency("build")
    .with_package("ui")
    .with_timeout(std::time::Duration::from_secs(60));

// Add tasks to runner
let mut runner = TaskRunner::new(&workspace);
runner.add_tasks(vec![build_task, test_task]);

// Run tasks by name
runner.run_tasks(&["test"])?;

// Run tasks with a filter
let filter = TaskFilter::new()
    .with_include(vec!["*test*"])
    .with_packages(vec!["ui"]);
runner.run_filtered(&filter)?;
```

## Change Tracking

Track and manage changes across packages:

```rust
use sublime_monorepo_tools::{
    Change, ChangeTracker, ChangeType, FileChangeStore
};
use std::rc::Rc;

// Create a change store
let store = FileChangeStore::new(".changes")?;
let mut tracker = ChangeTracker::new(Rc::new(workspace), Box::new(store));

// Record a change
let change = Change::new("ui", ChangeType::Feature, "Add new button component", false);
tracker.record_change(change)?;

// Detect changes between Git references
let changes = tracker.detect_changes_between("v1.0.0", None)?;

// Get unreleased changes
let unreleased = tracker.unreleased_changes()?;

// Mark changes as released
tracker.mark_released("ui", "1.1.0", false)?;
```

## Versioning

Manage versions across packages:

```rust
use sublime_monorepo_tools::{
    ChangelogOptions, VersionBumpStrategy, VersionManager
};

// Create version manager
let manager = VersionManager::new(&workspace, Some(&tracker));

// Define a strategy
let strategy = VersionBumpStrategy::Independent {
    major_if_breaking: true,
    minor_if_feature: true, 
    patch_otherwise: true,
};

// Preview version bumps
let preview = manager.preview_bumps(&strategy)?;

// Apply version bumps
let changes = manager.apply_bumps(&strategy, false)?;

// Generate changelogs
let options = ChangelogOptions::default();
manager.generate_changelogs(&options, false)?;
```

## Cycle Handling

The library includes special handling for circular dependencies:

```rust
// Check for cycles
if manager.has_cycles() {
    println!("Cycles detected:");
    println!("{}", manager.visualize_cycles());
}

// Get cycle groups
let cycle_groups = manager.get_cycle_groups();

// Check if a package is in a cycle
if manager.is_in_cycle("package-name") {
    println!("Package is in a dependency cycle");
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.