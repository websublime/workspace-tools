---
type: Definition
title: 'API Specification: `sublime_monorepo_tools` crate'
tags: [workspace-tools, rust]
---

# API Specification: `sublime_monorepo_tools` crate

[Workspace Tools](https://app.capacities.io/f8d3f900-cfe2-4c6d-b44c-906946be3e3c/52c8c019-b006-462b-8c5d-6b518afa0d64)

## Table of Contents

- Module: workspace (#module-workspace)

- Module: changes (#module-changes)

- Module: versioning (#module-versioning)

- Module: tasks (#module-tasks)

## Module: workspace

### Structs

#### `Workspace`

A representation of a monorepo workspace containing packages and their relationships.

**Methods:**

- `new(root_path: PathBuf, config: WorkspaceConfig, git_repo: Option<Repo>) -> Result<Self, WorkspaceError>` - Creates a new workspace

- `get_circular_dependencies(&self) -> Vec<Vec<String>>` - Returns information about circular dependencies

- `is_in_cycle(&self, package_name: &str) -> bool` - Checks if a package is part of a circular dependency

- `get_cycle_for_package(&self, package_name: &str) -> Option<Vec<String>>` - Gets the cycle group containing a package

- `get_cycle_membership(&self) -> HashMap<String, usize>` - Returns a mapping of packages to cycle groups

- `discover_packages_with_options(&mut self, options: &DiscoveryOptions) -> Result<&Self, WorkspaceError>` - Discovers packages with custom options

- `analyze_dependencies(&self) -> Result<WorkspaceGraph, WorkspaceError>` - Analyzes workspace dependencies

- `get_package(&self, name: &str) -> Option<Rc<RefCell<PackageInfo>>>` - Gets a package by name

- `sorted_packages(&self) -> Vec<Rc<RefCell<PackageInfo>>>` - Gets packages in topological order

- `get_sorted_packages_with_circulars(&self) -> SortedPackages` - Gets packages sorted with cycle information

- `affected_packages(&self, changed_packages: &[&str], check_circular: Option<bool>) -> Vec<Rc<RefCell<PackageInfo>>>` - Gets packages affected by changes

- `dependents_of(&self, package_name: &str) -> Vec<Rc<RefCell<PackageInfo>>>` - Gets packages that depend on a specific package

- `dependencies_of(&self, package_name: &str) -> Vec<Rc<RefCell<PackageInfo>>>` - Gets direct dependencies of a package

- `root_path(&self) -> &Path` - Gets workspace root path

- `git_repo(&self) -> Option<&Repo>` - Gets Git repository reference

- `package_manager(&self) -> &Option<CorePackageManager>` - Gets package manager

- `is_empty(&self) -> bool` - Checks if the workspace is empty

- `write_changes(&self) -> Result<(), WorkspaceError>` - Writes package changes to disk

- `validate(&self) -> Result<ValidationReport, WorkspaceError>` - Validates workspace consistency

- `validate_with_options(&self, options: &ValidationOptions) -> Result<ValidationReport, WorkspaceError>` - Validates workspace with custom options

#### `SortedPackages`

Result of topologically sorting workspace packages.

**Fields:**

- `sorted: Vec<Rc<RefCell<PackageInfo>>>` - Packages without cycles

- `circular: Vec<Vec<Rc<RefCell<PackageInfo>>>>` - Groups of packages forming cycles

#### `WorkspaceManager`

Main entry point for workspace operations.

**Methods:**

- `new() -> Self` - Creates a new workspace manager

- `discover_workspace(&self, path: impl AsRef<Path>, options: &DiscoveryOptions) -> Result<Workspace, WorkspaceError>` - Discovers a workspace from a directory

- `load_workspace(&self, config: WorkspaceConfig) -> Result<Workspace, WorkspaceError>` - Loads a workspace from configuration

- `analyze_workspace(&self, workspace: &Workspace) -> Result<WorkspaceAnalysis, WorkspaceError>` - Analyzes a workspace for issues

#### `WorkspaceConfig`

Workspace configuration.

**Fields:**

- `root_path: PathBuf` - Path to the workspace root

- `packages: Vec<String>` - Package patterns to include

- `package_manager: Option<String>` - Package manager to use

- `config: HashMap<String, serde_json::Value>` - Additional configuration

**Methods:**

- `new(root_path: PathBuf) -> Self` - Creates a new workspace configuration

- `with_packages<I, S>(mut self, packages: I) -> Self` - Sets the package patterns

- `with_package_manager<S: Into<String>>(mut self, package_manager: Option<S>) -> Self` - Sets the package manager

- `with_config<K, V>(mut self, key: K, value: V) -> Self` - Adds additional configuration

#### `DiscoveryOptions`

Options for discovering a workspace.

**Fields:**

- `auto_detect_root: bool` - Whether to auto-detect the project root

- `detect_package_manager: bool` - Whether to detect the package manager

- `include_patterns: Vec<String>` - Package patterns to include

- `exclude_patterns: Vec<String>` - Package patterns to exclude

- `max_depth: usize` - Maximum depth to search for packages

- `include_private: bool` - Whether to discover private packages

- `additional_package_paths: Vec<PathBuf>` - Custom package.json paths to include

**Methods:**

- `new() -> Self` - Creates new discovery options with defaults

- `auto_detect_root(mut self, value: bool) -> Self` - Sets whether to auto-detect the root

- `detect_package_manager(mut self, value: bool) -> Self` - Sets whether to detect the package manager

- `include_patterns<I, S>(mut self, patterns: I) -> Self` - Sets package patterns to include

- `exclude_patterns<I, S>(mut self, patterns: I) -> Self` - Sets package patterns to exclude

- `max_depth(mut self, depth: usize) -> Self` - Sets the maximum depth to search

- `include_private(mut self, value: bool) -> Self` - Sets whether to include private packages

- `additional_package_paths<I, P>(mut self, paths: I) -> Self` - Adds additional package paths

#### `ValidationOptions`

Options for validating workspace consistency.

**Fields:**

- `treat_unresolved_as_external: bool` - Whether to treat unresolved dependencies as external

- `internal_dependencies: Vec<String>` - List of dependency names to consider as internal

**Methods:**

- `new() -> Self` - Creates new validation options with defaults

- `treat_unresolved_as_external(mut self, value: bool) -> Self` - Sets whether to treat unresolved dependencies as external

- `with_internal_dependencies<I, S>(mut self, deps: I) -> Self` - Sets the list of dependencies to consider internal

#### `WorkspaceGraph`

Abstraction of a workspace dependency graph.

**Fields:**

- `cycles_detected: bool` - Whether cycles were detected in the graph

- `cycles: Vec<Vec<String>>` - The cycle groups identified in the graph

- `external_dependencies: Vec<String>` - External dependencies

- `version_conflicts: HashMap<String, Vec<String>>` - Version conflicts

- `validation: Option<ValidationReport>` - Validation report

#### `WorkspaceAnalysis`

Results from analyzing a workspace.

**Fields:**

- `cycles: Vec<Vec<String>>` - Cycle groups detected in the dependency graph

- `external_dependencies: Vec<String>` - External dependencies

- `version_conflicts: HashMap<String, Vec<String>>` - Version conflicts in the workspace

- `validation_issues: bool` - Whether there are validation issues

### Enums

#### `WorkspaceError`

Errors that can occur during workspace operations.

**Variants:**

- `RootNotFound` - Failed to find workspace root

- `ManifestReadError { path: PathBuf, error: io::Error }` - Failed to read workspace manifest

- `ManifestParseError { path: PathBuf, error: serde_json::Error }` - Failed to parse workspace manifest

- `NoPackagesFound(PathBuf)` - No packages found in workspace

- `PackageNotFound(String)` - Package not found in workspace

- `GitError(RepoError)` - Git repository error

- `PackageError(PackageError)` - Package error

- `VersionError(VersionError)` - Version error

- `DependencyResolutionError(DependencyResolutionError)` - Dependency resolution error

- `IoError(io::Error)` - IO error

- `InvalidConfiguration(String)` - Invalid workspace configuration

- `CycleDetected(String)` - Cycle in workspace packages

**Impls:**

- `impl AsRef<str> for WorkspaceError`

- `impl std::fmt::Display for WorkspaceError`

- `impl std::error::Error for WorkspaceError`

## Module: changes

### Structs

#### `Change`

A single change record.

**Fields:**

- `id: ChangeId` - Unique identifier for the change

- `package: String` - Package name

- `change_type: ChangeType` - Change type (feature, fix, etc.)

- `description: String` - Description

- `breaking: bool` - Whether this is breaking

- `timestamp: DateTime<Utc>` - Creation timestamp

- `author: Option<String>` - Author

- `issues: Vec<String>` - Related issues

- `release_version: Option<String>` - Release version (None if unreleased)

- `environments: Vec<String>` - Target environments

**Methods:**

- `new<S: Into<String>>(package: S, change_type: ChangeType, description: S, breaking: bool) -> Self` - Creates a new change record

- `with_author<S: Into<String>>(mut self, author: S) -> Self` - Sets the author of the change

- `with_release_version<S: Into<String>>(mut self, version: S) -> Self` - Sets the release version

- `with_issues<I, S>(mut self, issues: I) -> Self` - Adds related issues

- `with_environments<I, S>(mut self, environments: I) -> Self` - Sets the target environments

- `is_released(&self) -> bool` - Checks if the change is released

- `applies_to_environment(&self, environment: &str) -> bool` - Checks if change applies to an environment

- `summary(&self) -> String` - Gets a summary of the change

#### `ChangeId`

Unique identifier for a change or changeset.

**Methods:**

- `new() -> Self` - Creates a new random change ID

- `uuid(&self) -> &Uuid` - Gets the underlying UUID

**Impls:**

- `impl Default for ChangeId`

- `impl fmt::Display for ChangeId`

- `impl FromStr for ChangeId`

#### `Changeset`

Collection of related changes.

**Fields:**

- `id: ChangeId` - Unique identifier

- `summary: Option<String>` - Summary

- `changes: Vec<Change>` - Changes in this set

- `created_at: DateTime<Utc>` - Creation timestamp

**Methods:**

- `new<S: Into<String>>(summary: Option<S>, changes: Vec<Change>) -> Self` - Creates a new changeset

- `is_released(&self) -> bool` - Checks if all changes are released

- `package_names(&self) -> Vec<String>` - Gets the package names included in this changeset

#### `FileChangeStore`

File-based implementation of `ChangeStore`.

**Methods:**

- `new<P: AsRef<Path>>(changeset_dir: P) -> ChangeResult<Self>` - Creates a new file-based change store

#### `MemoryChangeStore`

Memory-based implementation of ChangeStore.

**Methods:**

- `new() -> Self` - Creates a new memory-based change store

**Impls:**

- `impl Default for MemoryChangeStore`

#### `ChangeTracker`

Change tracking system.

**Methods:**

- `new(workspace: Rc<Workspace>, store: Box<dyn ChangeStore>) -> Self` - Creates a new change tracker

- `with_git_user<S: Into<String>>(mut self, name: Option<S>, email: Option<S>) -> Self` - Sets the Git user information

- `clear_cache(&mut self)` - Clears the scope cache

- `map_file_to_scope(&mut self, file_path: &str) -> ChangeResult<ChangeScope>` - Maps a file to its change scope

- `get_workspace_root_path(&self) -> &Path` - Gets the workspace root path

- `detect_changes_between(&mut self, from_ref: &str, to_ref: Option<&str>) -> ChangeResult<Vec<Change>>` - Detects changes between Git references

- `record_change(&mut self, change: Change) -> ChangeResult<()>` - Records a change manually

- `create_changeset(&mut self, summary: Option<String>, changes: Vec<Change>) -> ChangeResult<Changeset>` - Creates and records a changeset

- `unreleased_changes(&self) -> ChangeResult<HashMap<String, Vec<Change>>>` - Gets unreleased changes for all packages

- `mark_released(&mut self, package: &str, version: &str, dry_run: bool) -> ChangeResult<Vec<Change>>` - Marks changes as released

- `unreleased_changes_for_environment(&self, environment: &str) -> ChangeResult<HashMap<String, Vec<Change>>>` - Gets unreleased changes for environment

- `mark_released_for_environment(&mut self, package: &str, version: &str, environment: &str, dry_run: bool) -> ChangeResult<Vec<Change>>` - Marks changes as released for environment

- `mark_specific_changes_as_released(&mut self, package: &str, version: &str, change_ids: &[ChangeId], dry_run: bool) -> ChangeResult<Vec<Change>>` - Marks specific changes as released

- `store(&self) -> &dyn ChangeStore` - Gets the change store

- `store_mut(&mut self) -> &mut dyn ChangeStore` - Gets a mutable reference to the change store

- `generate_changes_report(&self, with_cycle_info: bool) -> ChangeResult<String>` - Generate a report of package changes

- `visualize_dependency_graph(&self, include_cycles: bool) -> ChangeResult<String>` - Visualize the dependency graph

### Enums

#### `ChangeType`

Types of changes.

**Variants:**

- `Feature` - New feature or enhancement

- `Fix` - Bug fix

- `Documentation` - Documentation changes

- `Performance` - Performance improvements

- `Refactor` - Code refactoring without behavior change

- `Test` - Test changes

- `Chore` - Chore (e.g., build system, dependencies)

- `Build` - Build system changes

- `CI` - CI/CD changes

- `Revert` - Reverts a previous change

- `Style` - Code style changes

- `Custom(String)` - Custom change type

- `Unknown` - Unknown change type

- `Breaking` - Breaking change type

**Impls:**

- `impl fmt::Display for ChangeType`

#### `ChangeScope`

Represents the scope of a change in the repository.

**Variants:**

- `Package(String)` - Change specific to a package

- `Monorepo` - Change affecting the monorepo infrastructure

- `Root` - Change at the root level

#### `ChangeError`

Errors that can occur during change tracking operations.

**Variants:**

- `ReadError { path: PathBuf, error: io::Error }` - Failed to read a changeset file

- `WriteError { path: PathBuf, error: io::Error }` - Failed to write a changeset file

- `ParseError { path: PathBuf, error: serde_json::Error }` - Failed to parse a changeset file

- `SerializeError(serde_json::Error)` - Failed to serialize a changeset

- `DirectoryCreationError { path: PathBuf, error: io::Error }` - Failed to create changeset directory

- `ListError { path: PathBuf, error: io::Error }` - Failed to list changeset files

- `NoGitRepository` - No Git repository found

- `DetectionError(String)` - Failed to detect changes

- `InvalidReference(String)` - Invalid reference

- `WorkspaceError(WorkspaceError)` - Workspace error

- `GitError(sublime_git_tools::RepoError)` - Git error

- `NoChangesFound` - No changes found

- `InvalidChangeset(String)` - Invalid changeset

- `InvalidPackage(String)` - Invalid package

- `IoError(io::Error)` - IO error

**Impls:**

- `impl AsRef<str> for ChangeError`

- `impl std::fmt::Display for ChangeError`

- `impl std::error::Error for ChangeError`

### Traits

#### `ChangeStore`

Interface for storing and retrieving changes and changesets.

**Methods:**

- `get_changeset(&self, id: &ChangeId) -> ChangeResult<Option<Changeset>>` - Gets a changeset by ID

- `get_all_changesets(&self) -> ChangeResult<Vec<Changeset>>` - Gets all changesets

- `store_changeset(&mut self, changeset: &Changeset) -> ChangeResult<()>` - Stores a changeset

- `remove_changeset(&mut self, id: &ChangeId) -> ChangeResult<()>` - Removes a changeset

- `get_unreleased_changes(&self, package: &str) -> ChangeResult<Vec<Change>>` - Gets all unreleased changes for a package

- `get_released_changes(&self, package: &str) -> ChangeResult<Vec<Change>>` - Gets all released changes for a package

- `get_changes_by_version(&self, package: &str) -> ChangeResult<HashMap<String, Vec<Change>>>` - Gets all changes for a package grouped by version

- `mark_changes_as_released(&mut self, package: &str, version: &str, dry_run: bool) -> ChangeResult<Vec<Change>>` - Marks changes as released

- `get_all_changes_by_package(&self) -> ChangeResult<HashMap<String, Vec<Change>>>` - Gets all changes grouped by package

- `get_changes_for_environment(&self, package: &str, environment: &str) -> ChangeResult<Vec<Change>>` - Gets changes for a specific environment

- `get_unreleased_changes_for_environment(&self, package: &str, environment: &str) -> ChangeResult<Vec<Change>>` - Gets unreleased changes for environment

- `get_changes_by_environment(&self, environment: &str) -> ChangeResult<HashMap<String, Vec<Change>>>` - Gets all changes by environment

### Type Aliases

- `type ChangeResult<T> = Result<T, ChangeError>` - Type alias for Change operation results

## Module: versioning

### Structs

#### `VersionManager<'a>`

Manages version operations across a workspace.

**Methods:**

- `new(workspace: &'a Workspace, change_tracker: Option<&'a ChangeTracker>) -> Self` - Create a new version manager

- `suggest_bumps(&self, strategy: &VersionBumpStrategy) -> VersioningResult<HashMap<String, VersionSuggestion>>` - Suggest version bumps based on changes

- `suggest_bumps_with_options(&self, strategy: &VersionBumpStrategy, harmonize_cycles: bool) -> VersioningResult<HashMap<String, VersionSuggestion>>` - Suggests version bumps with cycle options

- `preview_bumps(&self, strategy: &VersionBumpStrategy) -> VersioningResult<VersionBumpPreview>` - Preview version bumps

- `apply_bumps(&self, strategy: &VersionBumpStrategy, dry_run: bool) -> VersioningResult<Vec<PackageVersionChange>>` - Apply version bumps

- `mark_changes_as_released(change_tracker: &mut ChangeTracker, version_changes: &[PackageVersionChange], dry_run: bool) -> VersioningResult<()>` - Mark changes as released

- `validate_versions(&self) -> VersioningResult<VersionValidation>` - Validate version consistency

- `get_workspace(&self) -> &'a Workspace` - Get the workspace reference

- `get_change_tracker(&self) -> VersioningResult<&'a ChangeTracker>` - Get the change tracker reference

- `get_cycle_groups(&self) -> Vec<Vec<String>>` - Gets all cycle groups in the workspace

- `has_cycles(&self) -> bool` - Checks if the workspace has cyclic dependencies

- `visualize_cycles(&self) -> String` - Visualizes cycles in the dependency graph

- `generate_version_report(&self, changes: &[PackageVersionChange]) -> String` - Generate a detailed version report

- `generate_changelogs(&self, options: &ChangelogOptions, dry_run: bool) -> VersioningResult<HashMap<String, String>>` - Generate changelogs for packages

#### `VersionValidation`

Validation result for workspace version consistency.

**Fields:**

- `has_cycles: bool` - Whether the dependency graph has cycles

- `inconsistencies: Vec<VersionInconsistency>` - List of version inconsistencies

#### `VersionInconsistency`

Represents an inconsistency between a package's dependency and the actual package version.

**Fields:**

- `package_name: String` - The package with the inconsistent dependency

- `dependency_name: String` - The dependency package name

- `required_version: String` - The version required by the package

- `actual_version: String` - The actual version of the dependency package

#### `PackageVersionChange`

Represents the result of a version bump for a package.

**Fields:**

- `package_name: String` - Package name

- `previous_version: String` - Previous version

- `new_version: String` - New version

- `bump_type: BumpType` - Type of bump performed

- `is_dependency_update: bool` - Whether this was a dependency-only update

- `is_cycle_update: bool` - Whether this update was due to being in a cycle

- `cycle_group: Option<Vec<String>>` - Which cycle group this package belongs to

**Impls:**

- `impl Default for PackageVersionChange`

#### `ChangelogOptions`

Settings for changelog generation.

**Fields:**

- `update_existing: bool` - Whether to update existing changelog files

- `filename: String` - Base filename for changelog files

- `include_version_details: bool` - Whether to include version details in the changelog

- `include_release_date: bool` - Whether to include release date in the changelog

- `header_template: String` - Header template for the changelog

- `change_template: String` - Template for individual changes

**Methods:**

- `new() -> Self` - Create a new changelog options object with defaults

**Impls:**

- `impl Default for ChangelogOptions`

#### `VersionSuggestion`

Version bump suggestion for a package.

**Fields:**

- `package_name: String` - Package name

- `current_version: String` - Current version

- `suggested_version: String` - Suggested next version

- `bump_type: BumpType` - Type of bump

- `reasons: Vec<BumpReason>` - Reasons for bump

- `cycle_group: Option<Vec<String>>` - Cycle group this package belongs to

**Methods:**

- `new(package_name: String, current_version: String, suggested_version: String, bump_type: BumpType) -> Self` - Create a new version suggestion

- `with_reason(mut self, reason: BumpReason) -> Self` - Add a reason for the bump

- `with_reasons(mut self, reasons: Vec<BumpReason>) -> Self` - Add multiple reasons

- `with_cycle_group(mut self, group: Vec<String>) -> Self` - Set the cycle group information

#### `VersionBumpPreview`

Preview of version bumps to be applied.

**Fields:**

- `changes: Vec<VersionSuggestion>` - Version changes to be applied

- `cycle_detected: bool` - Cycle detected in dependencies preventing some strategies

- `cycle_groups: Vec<Vec<String>>` - Groups of packages forming cycles

**Impls:**

- `impl Default for VersionBumpPreview`

### Enums

#### `VersionBumpStrategy`

Strategy for determining how package versions should be bumped.

**Variants:**

- `Synchronized { version: String }` - All packages get the same version

- `Independent { major_if_breaking: bool, minor_if_feature: bool, patch_otherwise: bool }` - Each package is bumped according to its changes

- `ConventionalCommits { from_ref: Option<String> }` - Use conventional commit messages to determine bump types

- `Manual(HashMap<String, String>)` - Manually specified versions

**Impls:**

- `impl Default for VersionBumpStrategy`

#### `BumpType`

Type of version bump to perform.

**Variants:**

- `Major` - Major version bump (x.0.0)

- `Minor` - Minor version bump (0.x.0)

- `Patch` - Patch version bump (0.0.x)

- `Snapshot` - Development snapshot

- `None` - No bump needed

**Impls:**

- `impl From<BumpType> for sublime_package_tools::Version`

- `impl std::fmt::Display for BumpType`

#### `BumpReason`

Reason for a version bump.

**Variants:**

- `Breaking(String)` - Breaking change

- `Feature(String)` - New feature

- `Fix(String)` - Bug fix

- `Other(String)` - Other change

- `DependencyUpdate(String)` - Dependency update

- `Manual` - Manual specification

#### `VersioningError`

Errors that can occur during versioning operations.

**Variants:**

- `WorkspaceError(WorkspaceError)` - Failed to load or parse workspace data

- `ChangeError(ChangeError)` - Error in change tracking operations

- `VersionError(VersionError)` - Error in version handling

- `PackageError(PackageError)` - Error with package operations

- `DependencyResolutionError(DependencyResolutionError)` - Error with dependency resolution

- `IoError(io::Error)` - IO error during changelog generation

- `NoChangesFound(String)` - No changes found for version bump

- `InvalidBumpStrategy(String)` - Invalid bump strategy

- `PackageNotFound(String)` - Missing package in workspace

- `NoVersionSuggestion(String, String)` - No version suggestion possible

- `CyclicDependencies(String)` - Cyclic dependencies prevent synchronized versioning

- `NoVersionFile(String)` - No version file found

**Impls:**

- `impl AsRef<str> for VersioningError`

- `impl std::fmt::Display for VersioningError`

- `impl std::error::Error for VersioningError`

### Functions

- `determine_bump_type_from_change(change: &Change, strategy: &VersionBumpStrategy) -> BumpType` - Determine the bump type based on a change

- `suggest_version_bumps(workspace: &Workspace, change_tracker: &ChangeTracker, strategy: &VersionBumpStrategy) -> VersioningResult<HashMap<String, VersionSuggestion>>` - Generate version suggestions

- `suggest_version_bumps_with_options(workspace: &Workspace, change_tracker: &ChangeTracker, strategy: &VersionBumpStrategy, harmonize_cycles: bool) -> VersioningResult<HashMap<String, VersionSuggestion>>` - Generate version suggestions with options

- `print_version_bump_preview(preview: &VersionBumpPreview)` - Prints a version bump preview to stdout

### Type Aliases

- `type VersioningResult<T> = Result<T, VersioningError>` - Type alias for versioning results

## Module: tasks

### Structs

#### `Task`

Definition of a task to be executed.

**Fields:**

- `name: String` - Task name

- `command: String` - Command to execute

- `package: Option<String>` - Package context (optional)

- `dependencies: Vec<String>` - Task dependencies (task names)

- `config: TaskConfig` - Task configuration

**Methods:**

- `new(name: impl Into<String>, command: impl Into<String>) -> Self` - Create a new task

- `with_package(mut self, package: impl Into<String>) -> Self` - Set the package for this task

- `with_dependency(mut self, dependency: impl Into<String>) -> Self` - Add a dependency

- `with_dependencies(mut self, dependencies: Vec<impl Into<String>>) -> Self` - Add multiple dependencies

- `with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self` - Set working directory

- `with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self` - Add environment variable

- `with_timeout(mut self, timeout: Duration) -> Self` - Set timeout

- `ignore_error(mut self, ignore: bool) -> Self` - Set whether to ignore errors

- `live_output(mut self, live: bool) -> Self` - Set whether to show live output

#### `TaskConfig`

Configuration for a task.

**Fields:**

- `cwd: Option<PathBuf>` - Working directory for the task

- `env: HashMap<String, String>` - Environment variables

- `timeout: Option<Duration>` - Timeout for the task

- `ignore_error: bool` - Whether to ignore errors

- `live_output: bool` - Whether to log output to console in real-time

**Impls:**

- `impl Default for TaskConfig`

#### `TaskExecution`

Results from task execution.

**Fields:**

- `exit_code: i32` - Exit code

- `stdout: String` - Standard output

- `stderr: String` - Standard error

- `duration: Duration` - Execution duration

- `status: TaskStatus` - Final task status

#### `TaskResultInfo`

Result of executing a specific task with full context.

**Fields:**

- `task: Task` - The task that was executed

- `execution: TaskExecution` - The execution details

**Methods:**

- `new(task: Task, execution: TaskExecution) -> Self` - Create a new task result

- `is_success(&self) -> bool` - Check if the task succeeded

- `is_failure(&self) -> bool` - Check if the task failed

- `duration(&self) -> Duration` - Get the duration of the task

- `exit_code(&self) -> i32` - Get the exit code

- `name(&self) -> &str` - Get the task name

**Impls:**

- `impl fmt::Display for TaskResultInfo`

#### `TaskRunner<'a>`

Task runner for executing tasks in a workspace.

**Methods:**

- `new(workspace: &'a Workspace) -> Self` - Create a new task runner

- `add_task(&mut self, task: Task) -> &mut Self` - Add a task to the runner

- `add_tasks(&mut self, tasks: Vec<Task>) -> &mut Self` - Add multiple tasks

- `get_tasks(&self) -> &[Task]` - Get all registered tasks

- `get_task(&self, name: &str) -> Option<Task>` - Get a task by name

- `load_tasks_from_config(&mut self, path: &Path) -> TaskResult<&mut Self>` - Load tasks from config

- `run_task(&self, task_name: &str) -> TaskResult<TaskResultInfo>` - Execute a single task

- `run_tasks(&self, task_names: &[&str]) -> TaskResult<Vec<TaskResultInfo>>` - Run multiple tasks

- `run_filtered(&self, filter: &TaskFilter) -> TaskResult<Vec<TaskResultInfo>>` - Run tasks matching a filter

- `build_task_graph(&self) -> TaskResult<TaskGraph>` - Build task graph for visualization

- `execute_task(&self, task: &Task) -> TaskResult<TaskExecution>` - Execute a task

**Impls:**

- `impl<'a> Clone for TaskRunner<'a>`

#### `TaskGraph`

Represents a task dependency graph.

**Fields:**

- `graph: DiGraph<String, ()>` - The internal graph structure

- `node_indices: HashMap<String, petgraph::graph::NodeIndex>` - Mapping from task names to node indices

- `tasks: HashMap<String, Task>` - All tasks by name

**Methods:**

- `from_tasks(tasks: &[Task]) -> TaskResult<Self>` - Create a task graph from tasks

- `sorted_tasks(&self, mode: TaskSortMode) -> TaskResult<Vec<Task>>` - Get sorted tasks

- `task_levels(&self) -> Vec<Vec<Task>>` - Get task dependency levels

- `dependencies_of(&self, task_name: &str) -> TaskResult<Vec<Task>>` - Get direct dependencies

- `dependents_of(&self, task_name: &str) -> TaskResult<Vec<Task>>` - Get direct dependents

- `all_tasks(&self) -> Vec<Task>` - Get all tasks in the graph

- `get_task(&self, name: &str) -> Option<Task>` - Get a task by name

- `task_count(&self) -> usize` - Get the number of tasks in the graph

#### `TaskFilter`

Filter for selecting tasks to run.

**Fields:**

- `include: Vec<String>` - Include tasks matching these patterns

- `exclude: Vec<String>` - Exclude tasks matching these patterns

- `packages: Vec<String>` - Only package-specific tasks in these packages

- `include_dependencies: bool` - Whether to include dependencies of matched tasks

- `include_dependents: bool` - Whether to include dependents of matched tasks

**Methods:**

- `new() -> Self` - Create a new empty task filter

- `with_include(mut self, patterns: Vec<impl Into<String>>) -> Self` - Add patterns to include

- `with_exclude(mut self, patterns: Vec<impl Into<String>>) -> Self` - Add patterns to exclude

- `with_packages(mut self, packages: Vec<impl Into<String>>) -> Self` - Add packages to filter by

- `include_dependencies(mut self, include: bool) -> Self` - Set whether to include dependencies

- `include_dependents(mut self, include: bool) -> Self` - Set whether to include dependents

- `apply(&self, tasks: &[Task]) -> TaskResult<Vec<Task>>` - Apply the filter to a list of tasks

**Impls:**

- `impl Default for TaskFilter`

#### `ParallelExecutionConfig`

Configuration for parallel task execution.

**Fields:**

- `max_parallel: usize` - Maximum number of parallel tasks

- `fail_fast: bool` - Whether to stop on first failure

- `show_progress: bool` - Whether to output progress

**Impls:**

- `impl Default for ParallelExecutionConfig`

#### `ParallelExecutor<'a>`

Execution engine for tasks.

**Methods:**

- `new(task_runner: &'a TaskRunner<'a>, config: ParallelExecutionConfig) -> Self` - Create a new executor

- `execute(&self, tasks: &[Task]) -> TaskResult<Vec<TaskResultInfo>>` - Execute tasks in sequence

### Enums

#### `TaskStatus`

Status of a task.

**Variants:**

- `Pending` - Task has not started

- `Running` - Task is currently running

- `Success` - Task completed successfully

- `Failed` - Task failed

- `Skipped` - Task was skipped

- `Timeout` - Task timed out

- `Cancelled` - Task was cancelled

#### `TaskSortMode`

Task sorting approach.

**Variants:**

- `Topological` - Topological sort (dependencies first)

- `Parallel` - Parallel sort (optimize for parallelism)

- `Random` - Random sort (no particular order)

#### `TaskError`

Errors that can occur during task operations.

**Variants:**

- `ExecutionFailed(String)` - Task execution failed

- `Timeout(Duration)` - Task timed out

- `Cancelled` - Task was cancelled

- `DependencyFailed(String)` - Dependency failed

- `CircularDependency(String)` - Circular dependency detected

- `TaskNotFound(String)` - Task not found

- `FilterError(String)` - Filter error

- `GraphError(String)` - Task graph error

- `IoError(std::io::Error)` - IO error

- `CommandError(String)` - Command execution error

- `StandardToolsError(String)` - Standard tools error

- `Other(String)` - Other error

**Impls:**

- `impl AsRef<str> for TaskError`

- `impl std::fmt::Display for TaskError`

- `impl std::error::Error for TaskError`

### Functions

- `default_parallel_config() -> ParallelExecutionConfig` - Creates a default parallel executor config

- `parallel_config_with_concurrency(concurrency: usize) -> ParallelExecutionConfig` - Creates a parallel config with specific concurrency

- `fail_fast_parallel_config() -> ParallelExecutionConfig` - Creates a fail-fast parallel config

### Type Aliases

- `type TaskResult<T> = Result<T, TaskError>` - Result type for task operations

## Examples

### Workspace Discovery and Analysis

```rust
use std::path::Path;
use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceManager, ValidationOptions};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a workspace manager
    let manager = WorkspaceManager::new();
    // Configure discovery options
    let options = DiscoveryOptions::new()
        .include_patterns(vec!["packages/*/package.json"])
        .exclude_patterns(vec!["**/node_modules/**"])
        .include_private(true);
    // Discover workspace
    let workspace = manager.discover_workspace(".", &options)?;
    
    println!("Found {} packages", workspace.sorted_packages().len());
    
    // Analyze workspace for issues
    let analysis = manager.analyze_workspace(&workspace)?;
    
    // Check for dependency cycles
    if !analysis.cycles.is_empty() {
        println!("Found {} dependency cycles:", analysis.cycles.len());
        for (i, cycle) in analysis.cycles.iter().enumerate() {
            println!("Cycle {}: {}", i+1, cycle.join(" → "));
        }
    }
    
    // Validate with custom options
    let validation_options = ValidationOptions::new()
        .treat_unresolved_as_external(true);
    
    let validation = workspace.validate_with_options(&validation_options)?;
    
    if validation.has_issues() {
        println!("Validation issues found:");
        for issue in validation.issues() {
            println!("- {}", issue);
        }
    }
    
    Ok(())
}
```

### Change Tracking

```rust
use std::rc::Rc;
use std::path::Path;
use sublime_monorepo_tools::{
    Change, ChangeType, ChangeTracker, FileChangeStore, Workspace, WorkspaceConfig
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a workspace
    let config = WorkspaceConfig::new(Path::new(".").to_path_buf());
    let workspace = Rc::new(Workspace::new(Path::new(".").to_path_buf(), config, None)?);
    
    // Create a change store
    let store = Box::new(FileChangeStore::new(".changes")?);
    
    // Create a change tracker
    let mut tracker = ChangeTracker::new(workspace, store);
    
    // Record manual changes
    let feature_change = Change::new(
        "ui-components",
        ChangeType::Feature,
        "Add new button component",
        false
    ).with_author("John Doe");
    
    tracker.record_change(feature_change)?;
    
    // Detect changes since a Git tag
    let detected_changes = tracker.detect_changes_between("v1.0.0", None)?;
    println!("Detected {} changes", detected_changes.len());
    
    // Create a changeset from detected changes
    let changeset = tracker.create_changeset(
        Some("Release preparation".to_string()),
        detected_changes
    )?;
    
    // Get all unreleased changes
    let unreleased = tracker.unreleased_changes()?;
    
    for (package, changes) in &unreleased {
        println!("Package {} has {} unreleased changes", package, changes.len());
        for change in changes {
            println!("  - {}", change.summary());
        }
    }
    
    // Generate report
    let report = tracker.generate_changes_report(true)?;
    println!("{}", report);
    
    Ok(())
}
```

### Version Management

```rust
use std::rc::Rc;
use std::path::Path;
use sublime_monorepo_tools::{
    ChangeTracker, FileChangeStore, VersionManager, VersionBumpStrategy,
    ChangelogOptions, Workspace, WorkspaceConfig
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create workspace and change tracker
    let config = WorkspaceConfig::new(Path::new(".").to_path_buf());
    let workspace = Rc::new(Workspace::new(Path::new(".").to_path_buf(), config, None)?);
    
    let store = Box::new(FileChangeStore::new(".changes")?);
    let tracker = ChangeTracker::new(workspace.clone(), store);
    
    // Create version manager
    let manager = VersionManager::new(&workspace, Some(&tracker));
    
    // Check for cycles (important for version bumping)
    if manager.has_cycles() {
        println!("Dependency cycles detected:");
        println!("{}", manager.visualize_cycles());
    }
    
    // Define version bump strategy
    let strategy = VersionBumpStrategy::Independent {
        major_if_breaking: true,
        minor_if_feature: true,
        patch_otherwise: true,
    };
    
    // Preview version bumps
    let preview = manager.preview_bumps(&strategy)?;
    println!("Would bump {} packages", preview.changes.len());
    
    // Apply version bumps (dry run first)
    let dry_run_changes = manager.apply_bumps(&strategy, true)?;
    println!("Would update {} packages:", dry_run_changes.len());
    
    for change in &dry_run_changes {
        println!("  {} {} → {}", change.package_name, change.previous_version, change.new_version);
    }
    
    // Apply actual changes
    let changes = manager.apply_bumps(&strategy, false)?;
    
    // Generate a report
    let report = manager.generate_version_report(&changes);
    println!("{}", report);
    
    // Generate changelogs
    let changelog_options = ChangelogOptions::new();
    let changelogs = manager.generate_changelogs(&changelog_options, false)?;
    
    println!("Generated {} changelogs", changelogs.len());
    
    Ok(())
}
```

### Task Running

```rust
use sublime_monorepo_tools::{
    Task, TaskRunner, TaskFilter, ParallelExecutionConfig, ParallelExecutor, Workspace
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get workspace (or create it, as shown in previous examples)
    let workspace = create_workspace()?;
    
    // Create task runner
    let mut runner = TaskRunner::new(&workspace);
    
    // Define tasks with dependencies
    runner.add_tasks(vec![
        Task::new("build", "npm run build"),
        Task::new("test", "npm run test")
            .with_dependency("build"),
        Task::new("lint", "npm run lint"),
        Task::new("deploy", "npm run deploy")
            .with_dependencies(vec!["build", "test", "lint"])
            .with_env("NODE_ENV", "production")
    ]);
    
    // Run a specific task (and its dependencies)
    let result = runner.run_task("test")?;
    println!("Task {} completed with status: {:?}", result.name(), result.execution.status);
    
    // Run multiple tasks
    let results = runner.run_tasks(&["lint", "build"])?;
    
    // Run tasks with a filter
    let filter = TaskFilter::new()
        .with_include(vec!["build*"])
        .with_exclude(vec!["*:slow"])
        .include_dependencies(true);
    
    let filtered_results = runner.run_filtered(&filter)?;
    
    // Run tasks in parallel (with parallel executor config)
    let tasks = runner.get_tasks().to_vec();
    let config = ParallelExecutionConfig {
        max_parallel: 4,
        fail_fast: true,
        show_progress: true,
    };
    
    let executor = ParallelExecutor::new(&runner, config);
    let results = executor.execute(&tasks)?;
    
    // Build and inspect task graph
    let graph = runner.build_task_graph()?;
    let levels = graph.task_levels();
    
    println!("Task graph has {} levels:", levels.len());
    for (i, level) in levels.iter().enumerate() {
        println!("Level {}: {} tasks", i, level.len());
        for task in level {
            println!("  {}", task.name);
        }
    }
    
    Ok(())
}
fn create_workspace() -> Result<Workspace, Box<dyn std::error::Error>> {
    // Implementation omitted for brevity
    // You'd typically use WorkspaceManager here
    unimplemented!()
}
```

## Common Usage Patterns

### Detecting and Handling Cycles

```rust
use sublime_monorepo_tools::{Workspace, WorkspaceManager, DiscoveryOptions};
fn detect_cycles(workspace: &Workspace) {
    // Get cycle information
    let cycles = workspace.get_circular_dependencies();
    
    if cycles.is_empty() {
        println!("No cycles detected - dependency graph is a DAG");
        return;
    }
    
    println!("Found {} cycle groups:", cycles.len());
    for (i, cycle) in cycles.iter().enumerate() {
        println!("Cycle {}: {}", i+1, cycle.join(" → "));
    }
    
    // Check if a specific package is in a cycle
    let package_name = "ui-components";
    if workspace.is_in_cycle(package_name) {
        if let Some(cycle_group) = workspace.get_cycle_for_package(package_name) {
            println!(
                "Package '{}' is in cycle: {}",
                package_name,
                cycle_group.join(" → ")
            );
        }
    }
}
```

### Change Classification and Changelog Generation

```rust
use sublime_monorepo_tools::{
    VersionManager, ChangelogOptions, Workspace, ChangeTracker
};
fn generate_release_artifacts(
    workspace: &Workspace,
    tracker: &ChangeTracker,
    version: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let manager = VersionManager::new(workspace, Some(tracker));
    
    // Get all unreleased changes
    let unreleased = tracker.unreleased_changes()?;
    
    // First, mark the changes as released
    for (package, _) in &unreleased {
        tracker.mark_released(package, version, false)?;
    }
    
    // Generate changelogs with custom options
    let options = ChangelogOptions {
        update_existing: true,
        filename: "CHANGELOG.md".to_string(),
        include_version_details: true,
        include_release_date: true,
        header_template: "# Changelog\n\nAll notable changes to this project will be documented in this file.\n\n".to_string(),
        change_template: "- **{type}**: {description} {breaking}\n".to_string(),
    };
    
    let changelogs = manager.generate_changelogs(&options, false)?;
    println!("Generated {} changelogs", changelogs.len());
    
    Ok(())
}
```

### Task Dependency Visualization

```rust
use sublime_monorepo_tools::{Task, TaskGraph, TaskRunner, Workspace};
use std::collections::HashMap;
fn visualize_task_dependencies(workspace: &Workspace, tasks: &[Task]) -> Result<String, Box<dyn std::error::Error>> {
    // Build the task graph
    let graph = TaskGraph::from_tasks(tasks)?;
    
    // Get task levels (for parallelization potential)
    let levels = graph.task_levels();
    
    let mut output = String::from("Task Dependency Graph:\n\n");
    
    // Build a visualization
    let mut level_map: HashMap<&str, usize> = HashMap::new();
    
    // Assign levels to tasks
    for (level_idx, level) in levels.iter().enumerate() {
        for task in level {
            level_map.insert(&task.name, level_idx);
        }
    }
    
    // Print the visualization
    for (level_idx, level) in levels.iter().enumerate() {
        output.push_str(&format!("Level {} (can run in parallel):\n", level_idx));
        
        for task in level {
            output.push_str(&format!("  {}", task.name));
            
            // Print dependencies
            let deps = match graph.dependencies_of(&task.name) {
                Ok(deps) if !deps.is_empty() => {
                    let names: Vec<String> = deps.iter().map(|d| d.name.clone()).collect();
                    format!(" (depends on: {})", names.join(", "))
                },
                _ => String::new()
            };
            
            output.push_str(&format!("{}\n", deps));
        }
        
        output.push('\n');
    }
    
    Ok(output)
}
```

## Extension Methods and Best Practices

### Efficient Package Graph Navigation

```rust
use sublime_monorepo_tools::{Workspace};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;
// Extension function to efficiently get the complete dependency tree
fn get_full_dependency_tree(
    workspace: &Workspace,
    package_name: &str
) -> HashSet<String> {
    let mut result = HashSet::new();
    let mut stack = vec![package_name.to_string()];
    let mut visited = HashSet::new();
    
    while let Some(pkg) = stack.pop() {
        if !visited.insert(pkg.clone()) {
            continue; // Skip if already visited
        }
        
        result.insert(pkg.clone());
        
        // Add direct dependencies
        for dep_pkg in workspace.dependencies_of(&pkg) {
            let dep_name = dep_pkg.borrow().package.borrow().name().to_string();
            if !visited.contains(&dep_name) {
                stack.push(dep_name);
            }
        }
    }
    
    result.remove(package_name); // Remove self from result
    result
}
// Extension function to find the common ancestor packages
fn find_common_ancestors(
    workspace: &Workspace,
    packages: &[&str]
) -> HashSet<String> {
    if packages.is_empty() {
        return HashSet::new();
    }
    
    // Get dependents for each package
    let mut all_dependents: Vec<HashSet<String>> = Vec::new();
    
    for &pkg in packages {
        let mut pkg_dependents = HashSet::new();
        let mut stack = vec![pkg.to_string()];
        let mut visited = HashSet::new();
        
        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            
            pkg_dependents.insert(current.clone());
            
            // Add direct dependents
            for dep in workspace.dependents_of(&current) {
                let dep_name = dep.borrow().package.borrow().name().to_string();
                if !visited.contains(&dep_name) {
                    stack.push(dep_name);
                }
            }
        }
        
        all_dependents.push(pkg_dependents);
    }
    
    // Find the intersection of all dependents
    let mut common = all_dependents[0].clone();
    for deps in &all_dependents[1..] {
        common = common.intersection(deps).cloned().collect();
    }
    
    common
}
```

### Managing Changes with Transaction-Like Patterns

```rust
use std::rc::Rc;
use sublime_monorepo_tools::{
    Change, Changeset, ChangeTracker, ChangeType, Workspace
};
struct ChangeTransaction<'a> {
    tracker: &'a mut ChangeTracker,
    changes: Vec<Change>,
    summary: Option<String>,
    committed: bool,
}
impl<'a> ChangeTransaction<'a> {
    fn new(tracker: &'a mut ChangeTracker) -> Self {
        Self {
            tracker,
            changes: Vec::new(),
            summary: None,
            committed: false,
        }
    }
    
    fn with_summary(mut self, summary: &str) -> Self {
        self.summary = Some(summary.to_string());
        self
    }
    
    fn add_change(&mut self, change: Change) {
        self.changes.push(change);
    }
    
    fn add_feature(&mut self, package: &str, description: &str, breaking: bool) {
        let change = Change::new(
            package,
            ChangeType::Feature,
            description,
            breaking
        );
        self.changes.push(change);
    }
    
    fn add_fix(&mut self, package: &str, description: &str) {
        let change = Change::new(
            package,
            ChangeType::Fix,
            description,
            false
        );
        self.changes.push(change);
    }
    
    fn commit(&mut self) -> Result<Changeset, Box<dyn std::error::Error>> {
        if self.committed {
            return Err("Transaction already committed".into());
        }
        
        if self.changes.is_empty() {
            return Err("No changes to commit".into());
        }
        
        let changeset = self.tracker.create_changeset(
            self.summary.clone(),
            self.changes.drain(..).collect()
        )?;
        
        self.committed = true;
        Ok(changeset)
    }
}
impl<'a> Drop for ChangeTransaction<'a> {
    fn drop(&mut self) {
        if !self.committed && !self.changes.is_empty() {
            eprintln!("Warning: ChangeTransaction dropped without committing {} changes", self.changes.len());
        }
    }
}
```

### Workspace Package Graph Pre-processing for Performance

```rust
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use sublime_monorepo_tools::{Workspace, WorkspaceManager, DiscoveryOptions};
// Caching package dependency structure for repeated operations
struct PackageGraphCache<'a> {
    workspace: &'a Workspace,
    dependencies_cache: HashMap<String, HashSet<String>>,
    dependents_cache: HashMap<String, HashSet<String>>,
    initialized: bool,
}
impl<'a> PackageGraphCache<'a> {
    fn new(workspace: &'a Workspace) -> Self {
        Self {
            workspace,
            dependencies_cache: HashMap::new(),
            dependents_cache: HashMap::new(),
            initialized: false,
        }
    }
    
    fn initialize(&mut self) {
        if self.initialized {
            return;
        }
        
        // Build the complete dependency graph in memory
        for pkg_info in self.workspace.sorted_packages() {
            let pkg_name = pkg_info.borrow().package.borrow().name().to_string();
            
            // Cache direct dependencies
            let deps = self.workspace.dependencies_of(&pkg_name);
            let dep_names: HashSet<String> = deps
                .iter()
                .map(|d| d.borrow().package.borrow().name().to_string())
                .collect();
            
            self.dependencies_cache.insert(pkg_name.clone(), dep_names.clone());
            
            // Update dependents cache
            for dep_name in dep_names {
                self.dependents_cache
                    .entry(dep_name)
                    .or_insert_with(HashSet::new)
                    .insert(pkg_name.clone());
            }
        }
        
        self.initialized = true;
    }
    
    fn dependencies(&mut self, package: &str) -> HashSet<String> {
        self.initialize();
        self.dependencies_cache
            .get(package)
            .cloned()
            .unwrap_or_default()
    }
    
    fn dependents(&mut self, package: &str) -> HashSet<String> {
        self.initialize();
        self.dependents_cache
            .get(package)
            .cloned()
            .unwrap_or_default()
    }
    
    fn all_dependencies(&mut self, package: &str) -> HashSet<String> {
        self.initialize();
        
        let mut result = HashSet::new();
        let mut stack = vec![package.to_string()];
        let mut visited = HashSet::new();
        
        while let Some(pkg) = stack.pop() {
            if visited.contains(&pkg) {
                continue;
            }
            visited.insert(pkg.clone());
            
            if let Some(deps) = self.dependencies_cache.get(&pkg) {
                for dep in deps {
                    if !visited.contains(dep) {
                        result.insert(dep.clone());
                        stack.push(dep.clone());
                    }
                }
            }
        }
        
        result
    }
    
    fn all_dependents(&mut self, package: &str) -> HashSet<String> {
        self.initialize();
        
        let mut result = HashSet::new();
        let mut stack = vec![package.to_string()];
        let mut visited = HashSet::new();
        
        while let Some(pkg) = stack.pop() {
            if visited.contains(&pkg) {
                continue;
            }
            visited.insert(pkg.clone());
            
            if let Some(deps) = self.dependents_cache.get(&pkg) {
                for dep in deps {
                    if !visited.contains(dep) {
                        result.insert(dep.clone());
                        stack.push(dep.clone());
                    }
                }
            }
        }
        
        result
    }
}
```

## License

This crate is licensed under [LICENSE INFORMATION].

