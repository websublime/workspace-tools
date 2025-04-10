---
type: Page
title: '`sublime_monorepo_tools` API Design'
description: The `sublime_monorepo_tools` crate provides a comprehensive suite of utilities for managing JavaScript/TypeScript monorepos in Rust. It supports package discovery, dependency analysis, change tracking, versioning, and task execution with a focus on maintainability and automation.
icon: 🔧
createdAt: '2025-04-09T22:39:58.645Z'
creationDate: 2025-04-09 23:39
modificationDate: 2025-04-10 00:19
tags: [workspace-tools, rust]
coverImage: null
---

[Workspace Tools](https://app.capacities.io/f8d3f900-cfe2-4c6d-b44c-906946be3e3c/52c8c019-b006-462b-8c5d-6b518afa0d64)

## Installation

Add the crate to your `Cargo.toml`:

```text
[dependencies]
sublime_monorepo_tools = "0.1.0"
```

## Core Components

The crate is organized into four main modules:

1. **Workspace Management** - Discovering and managing packages in a monorepo

2. **Change Tracking** - Tracking and recording changes to packages

3. **Versioning** - Managing version bumps across packages

4. **Task Management** - Defining and executing tasks with dependency resolution

## 1. Workspace Management API

### `Workspace`

Central struct representing a monorepo workspace.

```rust
// Create a workspace from a path
let workspace = Workspace::new(
    PathBuf::from("."),
    WorkspaceConfig::default(),
    None
)?;
// Discover packages
workspace.discover_packages_with_options(&DiscoveryOptions::default())?;
// Access packages
let packages = workspace.sorted_packages();
for pkg_info in packages {
    let pkg = pkg_info.borrow().package.borrow();
    println!("Found package: {} v{}", pkg.name(), pkg.version_str());
}
// Get a specific package
if let Some(pkg) = workspace.get_package("ui-components") {
    let info = pkg.borrow();
    println!("Package path: {}", info.package_path);
}
// Check for dependency cycles
let cycles = workspace.get_circular_dependencies();
if !cycles.is_empty() {
    println!("Found circular dependencies!");
}
// Analyze dependencies
let analysis = workspace.analyze_dependencies()?;
println!("Found {} external dependencies", analysis.external_dependencies.len());
```

### `WorkspaceManager`

High-level interface for creating and working with workspaces.

```rust
// Create a manager
let manager = WorkspaceManager::new();
// Discover a workspace with custom options
let options = DiscoveryOptions::new()
    .include_patterns(vec!["packages/*/package.json"])
    .exclude_patterns(vec!["**/node_modules/**"]);
let workspace = manager.discover_workspace(".", &options)?;
// Analyze the workspace
let analysis = manager.analyze_workspace(&workspace)?;
```

### `WorkspaceConfig`

Configuration for a workspace.

```rust
// Create custom workspace config
let config = WorkspaceConfig::new(PathBuf::from("."))
    .with_packages(vec!["packages/*", "apps/*"])
    .with_package_manager(Some("pnpm"));
```

### `DiscoveryOptions`

Options for discovering packages in a workspace.

```rust
// Configure package discovery
let options = DiscoveryOptions::new()
    .auto_detect_root(true)
    .include_patterns(vec!["packages/*/package.json", "apps/*/package.json"])
    .exclude_patterns(vec!["**/node_modules/**", "**/dist/**"])
    .max_depth(5)
    .include_private(false);
```

### `ValidationOptions`

Options for validating a workspace.

```rust
// Configure validation
let options = ValidationOptions::new()
    .treat_unresolved_as_external(true)
    .with_internal_dependencies(vec!["core", "shared"]);
// Validate with custom options
let report = workspace.validate_with_options(&options)?;
```

### `WorkspaceAnalysis`

Results from analyzing a workspace.

```rust
// Access analysis results
let analysis = manager.analyze_workspace(&workspace)?;
// Check for cycles
if !analysis.cycles.is_empty() {
    println!("Detected {} cycle groups", analysis.cycles.len());
    for cycle in &analysis.cycles {
        println!("Cycle: {}", cycle.join(" → "));
    }
}
// Check for external dependencies
if !analysis.external_dependencies.is_empty() {
    println!("External dependencies: {}", analysis.external_dependencies.join(", "));
}
// Check for version conflicts
for (package, conflicts) in &analysis.version_conflicts {
    println!("Package {} has version conflicts", package);
}
```

### `WorkspaceError`

Error types for workspace operations.

```rust
match result {
    Err(WorkspaceError::RootNotFound) => 
        println!("Could not find workspace root"),
    Err(WorkspaceError::NoPackagesFound(path)) => 
        println!("No packages found in {}", path.display()),
    Err(WorkspaceError::PackageNotFound(name)) => 
        println!("Package '{}' not found in workspace", name),
    Err(WorkspaceError::CycleDetected(cycle)) => 
        println!("Cycle detected: {}", cycle),
    // Handle other errors...
    Ok(_) => println!("Operation succeeded"),
}
```

## 2. Change Tracking API

### `Change` and `ChangeType`

Represents a single change to a package.

```rust
// Create a change
let change = Change::new(
    "ui-components",
    ChangeType::Feature,
    "Add new button component",
    false
);
// Create a breaking change
let breaking_change = Change::new(
    "api",
    ChangeType::Breaking,
    "Refactor authentication API",
    true
)
.with_author("Jane Doe")
.with_issues(vec!["#123", "JIRA-456"])
.with_environments(vec!["production", "staging"]);
// Check properties
println!("Summary: {}", breaking_change.summary());
println!("Is released: {}", breaking_change.is_released());
```

### `ChangeId`

Unique identifier for a change.

```rust
// Generate a new ID
let id = ChangeId::new();
// Convert to string
let id_str = id.to_string();
// Parse from string
let parsed_id = ChangeId::from_str(&id_str)?;
```

### `Changeset`

Collection of related changes.

```rust
// Create a changeset with multiple changes
let changeset = Changeset::new(
    Some("PR #123: UI improvements"),
    vec![change1, change2]
);
// Access properties
println!("Changes: {}", changeset.changes.len());
println!("Package names: {:?}", changeset.package_names());
println!("Is released: {}", changeset.is_released());
```

### `ChangeStore`

Interface for storing and retrieving changes.

```rust
// Create a file-based change store
let store = FileChangeStore::new(".changes")?;
// Create an in-memory change store
let memory_store = MemoryChangeStore::new();
// Get unreleased changes for a package
let unreleased = store.get_unreleased_changes("ui-components")?;
// Get changes by version
let changes_by_version = store.get_changes_by_version("api")?;
// Mark changes as released
let released = store.mark_changes_as_released("core", "1.2.0", false)?;
```

### `ChangeTracker`

High-level API for tracking changes.

```rust
// Create a tracker
let tracker = ChangeTracker::new(Rc::new(workspace), Box::new(store));
// Detect changes between Git refs
let changes = tracker.detect_changes_between("v1.0.0", None)?;
// Record a change manually
tracker.record_change(change)?;
// Create a changeset
let changeset = tracker.create_changeset(
    Some("New features".to_string()),
    vec![change1, change2]
)?;
// Get unreleased changes
let unreleased = tracker.unreleased_changes()?;
// Mark changes as released
let released = tracker.mark_released("api", "2.0.0", false)?;
// Generate a changes report
let report = tracker.generate_changes_report(true)?;
println!("{}", report);
```

### `ChangeError`

Error types for change operations.

```rust
match result {
    Err(ChangeError::ReadError { path, error }) => 
        println!("Failed to read changeset at {}: {}", path.display(), error),
    Err(ChangeError::NoChangesFound) => 
        println!("No changes found"),
    Err(ChangeError::CircularDependency(cycle)) => 
        println!("Circular dependency detected: {}", cycle),
    // Handle other errors...
    Ok(_) => println!("Operation succeeded"),
}
```

## 3. Versioning API

### `VersionManager`

Central component for managing versions across packages.

```rust
// Create a version manager
let manager = VersionManager::new(&workspace, Some(&change_tracker));
// Check for cycles
if manager.has_cycles() {
    println!("{}", manager.visualize_cycles());
}
// Create a version bump strategy
let strategy = VersionBumpStrategy::Independent {
    major_if_breaking: true,
    minor_if_feature: true,
    patch_otherwise: true,
};
// Preview version bumps
let preview = manager.preview_bumps(&strategy)?;
println!("Would bump {} packages", preview.changes.len());
// Apply version bumps
let changes = manager.apply_bumps(&strategy, false)?;
println!("Applied {} version bumps", changes.len());
// Mark changes as released
VersionManager::mark_changes_as_released(&mut tracker, &changes, false)?;
// Validate version consistency
let validation = manager.validate_versions()?;
if validation.has_cycles {
    println!("Dependency cycles detected");
}
// Generate detailed report
let report = manager.generate_version_report(&changes);
println!("{}", report);
```

### `VersionBumpStrategy`

Defines how package versions should be bumped.

```rust
// Independent versioning based on changes
let independent = VersionBumpStrategy::Independent {
    major_if_breaking: true,
    minor_if_feature: true,
    patch_otherwise: true,
};
// All packages get the same version
let synchronized = VersionBumpStrategy::Synchronized {
    version: "1.2.0".to_string(),
};
// Use conventional commits to determine bump type
let conventional = VersionBumpStrategy::ConventionalCommits {
    from_ref: Some("v1.0.0".to_string()),
};
// Manually specified versions
let mut versions = HashMap::new();
versions.insert("ui".to_string(), "2.0.0".to_string());
versions.insert("api".to_string(), "1.5.0".to_string());
let manual = VersionBumpStrategy::Manual(versions);
```

### `ChangelogOptions`

Configuration for generating changelogs.

```rust
// Create custom changelog options
let options = ChangelogOptions {
    update_existing: true,
    filename: "CHANGELOG.md".to_string(),
    include_version_details: true,
    include_release_date: true,
    header_template: "# Changelog\n\n".to_string(),
    change_template: "- {type}: {description} {breaking}\n".to_string(),
};
// Generate changelogs
let changelogs = manager.generate_changelogs(&options, false)?;
```

### `VersionSuggestion`

Suggested version change for a package.

```rust
// Create a version suggestion
let suggestion = VersionSuggestion::new(
    "ui".to_string(),
    "1.0.0".to_string(),
    "1.1.0".to_string(),
    BumpType::Minor
)
.with_reason(BumpReason::Feature("Add button component".to_string()));
```

### `VersionBumpPreview`

Preview of version changes to be applied.

```rust
// Generate preview
let preview = manager.preview_bumps(&strategy)?;
// Check for cycles
if preview.cycle_detected {
    println!("Cycle detected - harmonizing versions");
    for group in &preview.cycle_groups {
        println!("Cycle group: {}", group.join(" → "));
    }
}
// Print changes
for change in &preview.changes {
    println!("{}: {} → {}",
        change.package_name,
        change.current_version,
        change.suggested_version
    );
}
```

### `VersioningError`

Error types for versioning operations.

```rust
match result {
    Err(VersioningError::NoChangesFound(pkg)) => 
        println!("No changes found for package {}", pkg),
    Err(VersioningError::CyclicDependencies(desc)) => 
        println!("Cyclic dependencies prevent versioning: {}", desc),
    Err(VersioningError::PackageNotFound(pkg)) => 
        println!("Package {} not found in workspace", pkg),
    // Handle other errors...
    Ok(_) => println!("Operation succeeded"),
}
```

## 4. Task Management API

### `Task` and `TaskConfig`

Definition of a task to be executed.

```rust
// Create a simple task
let build = Task::new("build", "npm run build");
// Create a task with dependencies and configuration
let test = Task::new("test", "npm test")
    .with_package("ui-components")
    .with_dependency("build")
    .with_cwd("/path/to/dir")
    .with_env("NODE_ENV", "test")
    .with_timeout(Duration::from_secs(60))
    .ignore_error(true)
    .live_output(true);
```

### `TaskRunner`

Manages task execution within a workspace.

```rust
// Create a runner
let mut runner = TaskRunner::new(&workspace);
// Add tasks
runner.add_task(build);
runner.add_task(test);
// Load tasks from config file
runner.load_tasks_from_config(Path::new("tasks.json"))?;
// Run a single task
let result = runner.run_task("build")?;
println!("{} exited with code {}", result.name(), result.exit_code());
// Run multiple tasks
let results = runner.run_tasks(&["build", "test"])?;
// Apply a filter
let filter = TaskFilter::new()
    .with_include(vec!["test*"])
    .with_exclude(vec!["*:slow"])
    .with_packages(vec!["ui"])
    .include_dependencies(true);
let filtered_results = runner.run_filtered(&filter)?;
// Build task graph
let graph = runner.build_task_graph()?;
println!("Task graph has {} tasks", graph.task_count());
```

### `TaskGraph`

Represents dependencies between tasks.

```rust
// Create graph from tasks
let graph = TaskGraph::from_tasks(&tasks)?;
// Get sorted tasks
let sorted = graph.sorted_tasks(TaskSortMode::Topological)?;
// Get tasks organized by execution level
let levels = graph.task_levels();
for (i, level) in levels.iter().enumerate() {
    println!("Level {}: {} tasks", i, level.len());
}
// Get task dependencies/dependents
let deps = graph.dependencies_of("build")?;
let dependents = graph.dependents_of("core")?;
```

### `TaskFilter`

Filter for selecting tasks to run.

```rust
// Create filters
let build_filter = TaskFilter::new()
    .with_include(vec!["build:*"])
    .with_packages(vec!["ui"])
    .include_dependencies(true);
let test_filter = TaskFilter::new()
    .with_include(vec!["test:*"])
    .with_exclude(vec!["*:coverage"])
    .include_dependencies(true)
    .include_dependents(false);
// Apply filter to tasks
let filtered_tasks = filter.apply(&all_tasks)?;
```

### `ParallelExecutor` and `ParallelExecutionConfig`

Handles parallel execution of tasks.

```rust
// Create execution config
let config = ParallelExecutionConfig {
    max_parallel: 4,
    fail_fast: true,
    show_progress: true,
};
// Or use helper functions
let default_config = default_parallel_config();
let fail_fast = fail_fast_parallel_config();
let custom_concurrency = parallel_config_with_concurrency(8);
// Create executor
let executor = ParallelExecutor::new(&runner, config);
// Execute tasks
let results = executor.execute(&tasks)?;
```

### `TaskStatus` and `TaskExecution`

Status and execution results of tasks.

```rust
// Create task execution result
let execution = TaskExecution {
    exit_code: 0,
    stdout: "Task completed successfully".to_string(),
    stderr: "".to_string(),
    duration: Duration::from_secs(2),
    status: TaskStatus::Success,
};
// Check status
match execution.status {
    TaskStatus::Success => println!("Task succeeded"),
    TaskStatus::Failed => println!("Task failed with code {}", execution.exit_code),
    TaskStatus::Timeout => println!("Task timed out after {:?}", execution.duration),
    TaskStatus::Cancelled => println!("Task was cancelled"),
    TaskStatus::Skipped => println!("Task was skipped"),
    // Handle other statuses...
}
```

### `TaskResultInfo`

Combines task definition with execution results.

```rust
// Create task result info
let result = TaskResultInfo::new(task, execution);
// Check properties
println!("Task: {}", result.name());
println!("Duration: {:?}", result.duration());
println!("Exit code: {}", result.exit_code());
println!("Success: {}", result.is_success());
println!("Failure: {}", result.is_failure());
```

### `TaskError`

Error types for task operations.

```rust
match result {
    Err(TaskError::TaskNotFound(name)) => 
        println!("Task {} not found", name),
    Err(TaskError::Timeout(duration)) => 
        println!("Task timed out after {:?}", duration),
    Err(TaskError::ExecutionFailed(reason)) => 
        println!("Task failed: {}", reason),
    Err(TaskError::CircularDependency(cycle)) => 
        println!("Circular dependency detected: {}", cycle),
    // Handle other errors...
    Ok(_) => println!("Operation succeeded"),
}
```

## Complete Example Workflows

### Setting up a workspace and analyzing dependencies

```rust
use sublime_monorepo_tools::{
    DiscoveryOptions, WorkspaceManager, ValidationOptions
};
// Initialize workspace manager
let manager = WorkspaceManager::new();
// Discover workspace
let options = DiscoveryOptions::new()
    .include_patterns(vec!["packages/*/package.json"])
    .exclude_patterns(vec!["**/node_modules/**", "**/dist/**"]);
let workspace = manager.discover_workspace(".", &options)?;
// Analyze dependencies
let analysis = manager.analyze_workspace(&workspace)?;
// Check for issues
if !analysis.cycles.is_empty() {
    println!("Found {} dependency cycles", analysis.cycles.len());
    for cycle in &analysis.cycles {
        println!("Cycle: {}", cycle.join(" → "));
    }
}
// Validate with custom options
let validation_options = ValidationOptions::new()
    .treat_unresolved_as_external(true);
let report = workspace.validate_with_options(&validation_options)?;
if report.has_issues() {
    println!("Validation found {} issues", report.issues().len());
}
```

### Tracking changes and bumping versions

```rust
use sublime_monorepo_tools::{
    ChangeTracker, FileChangeStore, VersionManager, 
    VersionBumpStrategy, ChangelogOptions
};
use std::rc::Rc;
// Create change store and tracker
let store = FileChangeStore::new(".changes")?;
let tracker = ChangeTracker::new(Rc::new(workspace), Box::new(store));
// Detect changes since last tag
let changes = tracker.detect_changes_between("v1.0.0", None)?;
println!("Detected {} changes", changes.len());
// Create a changeset
let changeset = tracker.create_changeset(
    Some("Feature updates".to_string()),
    changes
)?;
// Create version manager
let version_manager = VersionManager::new(&workspace, Some(&tracker));
// Define version strategy
let strategy = VersionBumpStrategy::Independent {
    major_if_breaking: true,
    minor_if_feature: true,
    patch_otherwise: true,
};
// Preview and apply version bumps
let preview = version_manager.preview_bumps(&strategy)?;
let changes = version_manager.apply_bumps(&strategy, false)?;
println!("Updated {} package versions", changes.len());
// Mark changes as released
VersionManager::mark_changes_as_released(&mut tracker, &changes, false)?;
// Generate changelogs
let changelog_options = ChangelogOptions::default();
let changelogs = version_manager.generate_changelogs(&changelog_options, false)?;
```

### Running tasks with dependencies

```rust
use sublime_monorepo_tools::{
    Task, TaskRunner, TaskFilter, TaskGraph,
    ParallelExecutor, default_parallel_config
};
use std::time::Duration;
// Create task runner
let mut runner = TaskRunner::new(&workspace);
// Define tasks with dependencies
runner.add_tasks(vec![
    Task::new("clean", "rm -rf dist"),
    Task::new("build", "npm run build").with_dependency("clean"),
    Task::new("test", "npm test").with_dependency("build"),
    Task::new("lint", "npm run lint"),
    Task::new("deploy", "npm run deploy")
        .with_dependencies(vec!["build", "test", "lint"])
        .with_timeout(Duration::from_secs(300))
]);
// Create task graph
let graph = runner.build_task_graph()?;
// View task levels
let levels = graph.task_levels();
for (i, level) in levels.iter().enumerate() {
    println!("Level {}: {} tasks", i, level.len());
    for task in level {
        println!("  - {}", task.name);
    }
}
// Run tasks with a filter
let filter = TaskFilter::new()
    .with_include(vec!["test", "lint"])
    .include_dependencies(true);
// Execute in parallel
let config = default_parallel_config();
let executor = ParallelExecutor::new(&runner, config);
let results = executor.execute(&filter.apply(&runner.get_tasks())?)?;
// Process results
for result in results {
    if result.is_success() {
        println!("✅ {} completed in {:?}", result.name(), result.duration());
    } else {
        println!("❌ {} failed with code {}", result.name(), result.exit_code());
    }
}
```

## Error Handling

The crate provides specific error types for each module, each implementing the `std::error::Error` trait:

- `WorkspaceError` - Errors in workspace operations

- `ChangeError` - Errors in change tracking operations

- `VersioningError` - Errors in versioning operations

- `TaskError` - Errors in task execution operations

Each error type provides detailed information about what went wrong, and corresponding result type aliases:

- `type ChangeResult<T> = Result<T, ChangeError>`

- `type VersioningResult<T> = Result<T, VersioningError>`

- `type TaskResult<T> = Result<T, TaskError>`

Proper error handling should use pattern matching to handle specific error cases:

```rust
match workspace.analyze_dependencies() {
    Ok(analysis) => {
        // Process analysis results
    },
    Err(WorkspaceError::CycleDetected(cycle)) => {
        println!("Circular dependency detected: {}", cycle);
        // Special handling for cycles
    },
    Err(err) => {
        eprintln!("Analysis failed: {}", err);
        // Generic error handling
    }
}
```

## Conclusion

The `sublime_monorepo_tools` crate provides comprehensive functionality for managing JavaScript/TypeScript monorepos in Rust. It handles package discovery, dependency analysis, change tracking, versioning, and task execution, with robust error handling and a flexible API.

By combining these components, you can automate various monorepo workflows such as dependency analysis, version bumping, changelog generation, and task orchestration, all with strong Rust-based type safety and error handling.

