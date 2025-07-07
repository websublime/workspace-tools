# Sublime Monorepo Tools API Specification

This document provides a comprehensive specification of the public API for the `sublime_monorepo_tools` crate, a streamlined library for CLI and daemon consumption that provides essential Node.js monorepo functionality.

## Table of Contents

- [Overview](#overview)
- [Core Project Management](#core-project-management)
  - [MonorepoProject](#monorepoproject)
- [Analysis Module](#analysis-module)
  - [MonorepoAnalyzer](#monorepoanalyzer)
  - [ChangeAnalysis](#changeanalysis)
- [Configuration Module](#configuration-module)
  - [MonorepoConfig](#monorepoconfig)
  - [Environment](#environment)
  - [VersionBumpType](#versionbumptype)
  - [Workspace Configuration](#workspace-configuration)
  - [Task Configuration](#task-configuration)
  - [Hooks Configuration](#hooks-configuration)
  - [Changelog Configuration](#changelog-configuration)
  - [Changesets Configuration](#changesets-configuration)
- [Version Management](#version-management)
  - [VersionManager](#versionmanager)
  - [VersioningResult](#versioningresult)
- [Change Detection](#change-detection)
  - [ChangeDetector](#changedetector)
  - [PackageChange](#packagechange)
- [Task Management](#task-management)
  - [TaskManager](#taskmanager)
  - [Task Execution](#task-execution)
- [Changeset Management](#changeset-management)
  - [ChangesetManager](#changesetmanager)
  - [Changeset Types](#changeset-types)
- [Changelog Generation](#changelog-generation)
  - [ChangelogManager](#changelogmanager)
  - [Changelog Types](#changelog-types)
- [Error Types](#error-types)
  - [Error](#error)
  - [Result](#result)

## Overview

The `sublime_monorepo_tools` crate provides a comprehensive set of utilities for working with Node.js monorepos from Rust applications. It's designed for CLI and daemon usage patterns with sub-second performance targets.

### Core Architecture

Built on foundational base crates:
- `sublime_git_tools`: Git operations and repository management
- `sublime_standard_tools`: File system, command execution, and monorepo detection  
- `sublime_package_tools`: Package management, dependencies, and version handling

### Essential API (12 Types)

The public API is intentionally minimal for CLI/daemon usage:

```rust
// Core project management (1 type)
pub use crate::core::MonorepoProject;

// Core analysis (2 types)
pub use crate::analysis::{ChangeAnalysis, MonorepoAnalyzer};

// Configuration (3 types)
pub use crate::config::{Environment, MonorepoConfig, VersionBumpType};

// Version management (2 types)
pub use crate::core::{VersionManager, VersioningResult};

// Change detection (2 types)  
pub use crate::changes::{ChangeDetector, PackageChange};

// Error handling (2 types)
pub use crate::error::{Error, Result};
```

## Core Project Management

### MonorepoProject

The main entry point for monorepo operations, providing direct integration with base crates.

```rust
pub struct MonorepoProject {
    pub config: MonorepoConfig,
    pub packages: Vec<MonorepoPackageInfo>,
    pub repository: sublime_git_tools::Repo,
    pub file_system: sublime_standard_tools::filesystem::FileSystemManager,
    pub root_path: PathBuf,
}

impl MonorepoProject {
    /// Creates a new MonorepoProject instance from the specified path
    pub fn new(path: impl AsRef<Path>) -> Result<Self>;
    
    /// Returns the root path of the monorepo
    pub fn root_path(&self) -> &Path;
    
    /// Returns the configuration for this monorepo
    pub fn config(&self) -> &MonorepoConfig;
    
    /// Returns all packages in the monorepo
    pub fn packages(&self) -> &[MonorepoPackageInfo];
    
    /// Returns all internal packages (non-external dependencies)
    pub fn internal_packages(&self) -> Vec<&MonorepoPackageInfo>;
    
    /// Gets a specific package by name
    pub fn get_package(&self, name: &str) -> Option<&MonorepoPackageInfo>;
    
    /// Returns the Git repository instance
    pub fn repository(&self) -> &sublime_git_tools::Repo;
    
    /// Returns the file system manager
    pub fn file_system(&self) -> &sublime_standard_tools::filesystem::FileSystemManager;
}

/// Information about a package within the monorepo
#[derive(Debug, Clone)]
pub struct MonorepoPackageInfo {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub dependencies_internal: Vec<String>,
    pub dependencies_external: Vec<String>,
    pub dev_dependencies_internal: Vec<String>,
    pub dev_dependencies_external: Vec<String>,
    pub package_manager: PackageManagerType,
}

impl MonorepoPackageInfo {
    /// Returns the package name
    pub fn name(&self) -> &str;
    
    /// Returns the package version
    pub fn version(&self) -> &str;
    
    /// Returns the package path relative to monorepo root
    pub fn path(&self) -> &Path;
    
    /// Returns internal dependencies within the monorepo
    pub fn internal_dependencies(&self) -> &[String];
    
    /// Returns external dependencies from registries
    pub fn external_dependencies(&self) -> &[String];
}
```

## Analysis Module

### MonorepoAnalyzer

Provides analysis capabilities for detecting changes and understanding package relationships.

```rust
pub struct MonorepoAnalyzer<'a> {
    config: &'a MonorepoConfig,
    packages: &'a [MonorepoPackageInfo],
    repository: &'a sublime_git_tools::Repo,
    root_path: &'a Path,
}

impl<'a> MonorepoAnalyzer<'a> {
    /// Creates a new analyzer instance
    pub fn new(project: &'a MonorepoProject) -> Self;
    
    /// Detects changes between Git references
    pub fn detect_changes_since(&self, since_ref: &str, until_ref: Option<&str>) -> Result<ChangeAnalysis>;
    
    /// Compares changes between two branches
    pub fn compare_branches(&self, base_branch: &str, target_branch: &str) -> Result<BranchComparison>;
    
    /// Builds a dependency graph for the monorepo
    pub fn build_dependency_graph(&self) -> Result<DependencyGraph>;
    
    /// Analyzes which packages are affected by specific file changes
    pub fn analyze_affected_packages(&self, changed_files: &[String]) -> Result<Vec<String>>;
    
    /// Gets packages that depend on the specified package
    pub fn get_dependent_packages(&self, package_name: &str) -> Result<Vec<String>>;
}

/// Represents a dependency graph with nodes and edges
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub node_count: usize,
    pub edge_count: usize,
    pub has_cycles: bool,
}

impl DependencyGraph {
    /// Creates a new dependency graph
    pub fn new() -> Self;
    
    /// Returns the number of nodes in the graph
    pub fn node_count(&self) -> usize;
    
    /// Returns the number of edges in the graph
    pub fn edge_count(&self) -> usize;
    
    /// Returns true if the graph contains cycles
    pub fn has_cycles(&self) -> bool;
}

/// Result of comparing two branches
#[derive(Debug, Clone)]
pub struct BranchComparison {
    pub base_branch: String,
    pub target_branch: String,
    pub changed_files: Vec<String>,
    pub affected_packages: Vec<String>,
    pub new_packages: Vec<String>,
    pub deleted_packages: Vec<String>,
}
```

### ChangeAnalysis

Contains the results of change detection analysis.

```rust
#[derive(Debug, Clone)]
pub struct ChangeAnalysis {
    /// Git reference range analyzed
    pub from_ref: String,
    pub to_ref: String,
    
    /// All changed files in the analysis
    pub changed_files: Vec<String>,
    
    /// Package-specific changes
    pub package_changes: Vec<PackageChange>,
    
    /// Packages directly affected by changes
    pub directly_affected: Vec<String>,
    
    /// Packages affected through dependencies
    pub dependents_affected: Vec<String>,
    
    /// Analysis timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ChangeAnalysis {
    /// Returns all affected packages (direct + dependents)
    pub fn all_affected_packages(&self) -> Vec<&str>;
    
    /// Returns true if any packages were affected
    pub fn has_changes(&self) -> bool;
    
    /// Gets changes for a specific package
    pub fn get_package_changes(&self, package_name: &str) -> Option<&PackageChange>;
}
```

## Configuration Module

### MonorepoConfig

Main configuration structure for monorepo tools.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonorepoConfig {
    /// Versioning configuration
    pub versioning: VersioningConfig,
    
    /// Task management configuration
    pub tasks: TasksConfig,
    
    /// Changelog generation configuration
    pub changelog: ChangelogConfig,
    
    /// Git hooks configuration
    pub hooks: HooksConfig,
    
    /// Changesets configuration
    pub changesets: ChangesetsConfig,
    
    /// Plugin system configuration
    pub plugins: PluginsConfig,
    
    /// Workspace configuration
    pub workspace: WorkspaceConfig,
    
    /// Git configuration
    pub git: GitConfig,
    
    /// Validation rules and quality gates configuration
    pub validation: ValidationConfig,
    
    /// Deployment environments
    pub environments: Vec<Environment>,
}

impl Default for MonorepoConfig {
    fn default() -> Self;
}
```

### Environment

Represents deployment environments.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Environment {
    /// Development environment
    Development,
    /// Staging environment
    Staging,
    /// Integration environment
    Integration,
    /// Production environment
    Production,
    /// Custom environment
    Custom(String),
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}
```

### VersionBumpType

Defines the type of version bump to apply.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionBumpType {
    /// Major version bump (breaking changes)
    Major,
    /// Minor version bump (new features)
    Minor,
    /// Patch version bump (bug fixes)
    Patch,
    /// Snapshot version for feature branches
    Snapshot,
}

impl VersionBumpType {
    /// Returns true if this is a breaking change
    pub fn is_breaking(&self) -> bool;
    
    /// Returns the string representation
    pub fn as_str(&self) -> &'static str;
}
```

### Workspace Configuration

Configuration for workspace patterns and package discovery.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Custom workspace patterns for package discovery
    pub patterns: Vec<WorkspacePattern>,
    
    /// Whether to merge custom patterns with auto-detected ones
    pub merge_with_detected: bool,
    
    /// Package manager specific configurations
    pub package_manager_configs: PackageManagerConfigs,
    
    /// Workspace validation rules
    pub validation: WorkspaceValidationConfig,
    
    /// Package discovery settings
    pub discovery: PackageDiscoveryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePattern {
    /// Glob pattern for package discovery (e.g., "packages/*")
    pub pattern: String,
    
    /// Human-readable description of this pattern
    pub description: Option<String>,
    
    /// Whether this pattern is enabled
    pub enabled: bool,
    
    /// Priority for pattern matching (higher = first)
    pub priority: i32,
    
    /// Package managers this pattern applies to
    pub package_managers: Option<Vec<PackageManagerType>>,
    
    /// Environments this pattern applies to
    pub environments: Option<Vec<Environment>>,
    
    /// Additional options for this pattern
    pub options: WorkspacePatternOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePatternOptions {
    /// Include nested packages within matched directories
    pub include_nested: bool,
    
    /// Maximum depth to search for packages
    pub max_depth: Option<usize>,
    
    /// Patterns to exclude from matching
    pub exclude_patterns: Vec<String>,
    
    /// Whether to follow symbolic links
    pub follow_symlinks: bool,
    
    /// Override auto-detection for this pattern
    pub override_detection: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageManagerType {
    Npm,
    Yarn,
    Pnpm,
    Bun,
    Deno,
}
```

### Task Configuration

Configuration for task execution and management.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksConfig {
    /// Global task timeout in seconds
    pub default_timeout: u64,
    
    /// Maximum concurrent tasks
    pub max_concurrent: usize,
    
    /// Whether to continue on task failures
    pub continue_on_failure: bool,
    
    /// Task execution mode
    pub execution_mode: TaskExecutionMode,
    
    /// Environment variables for tasks
    pub environment_variables: std::collections::HashMap<String, String>,
    
    /// Task-specific configurations
    pub task_configs: std::collections::HashMap<String, TaskConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskExecutionMode {
    /// Execute tasks in parallel where possible
    Parallel,
    /// Execute tasks sequentially
    Sequential,
    /// Execute based on dependency graph
    Dependency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    /// Command to execute
    pub command: String,
    
    /// Arguments for the command
    pub args: Vec<String>,
    
    /// Working directory (relative to package root)
    pub working_directory: Option<String>,
    
    /// Task timeout in seconds
    pub timeout: Option<u64>,
    
    /// Whether this task can run in parallel
    pub parallel: bool,
    
    /// Dependencies that must run before this task
    pub dependencies: Vec<String>,
    
    /// Conditions for running this task
    pub conditions: TaskConditions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConditions {
    /// Only run if files matching these patterns have changed
    pub on_file_changes: Vec<String>,
    
    /// Only run in these environments
    pub environments: Vec<Environment>,
    
    /// Only run for these package types
    pub package_types: Vec<String>,
}
```

### Hooks Configuration

Configuration for Git hooks integration.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    /// Whether hooks are enabled globally
    pub enabled: bool,
    
    /// Pre-commit hook configuration
    pub pre_commit: PreCommitConfig,
    
    /// Pre-push hook configuration
    pub pre_push: PrePushConfig,
    
    /// Post-merge hook configuration
    pub post_merge: PostMergeConfig,
    
    /// Custom hook configurations
    pub custom_hooks: std::collections::HashMap<String, CustomHookConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreCommitConfig {
    /// Whether pre-commit hook is enabled
    pub enabled: bool,
    
    /// Validate that changesets are present for changed packages
    pub validate_changeset: bool,
    
    /// Tasks to run on pre-commit
    pub run_tasks: Vec<String>,
    
    /// Whether to block commit on task failures
    pub block_on_failure: bool,
    
    /// Whether to run tasks only on changed packages
    pub affected_packages_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrePushConfig {
    /// Whether pre-push hook is enabled
    pub enabled: bool,
    
    /// Tasks to run on pre-push
    pub run_tasks: Vec<String>,
    
    /// Whether to block push on task failures
    pub block_on_failure: bool,
    
    /// Whether to run tasks only on changed packages
    pub affected_packages_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMergeConfig {
    /// Whether post-merge hook is enabled
    pub enabled: bool,
    
    /// Whether to apply changesets automatically after merge
    pub auto_apply_changesets: bool,
    
    /// Whether to generate changelogs after merge
    pub auto_generate_changelogs: bool,
    
    /// Tasks to run after merge
    pub run_tasks: Vec<String>,
}
```

### Changelog Configuration

Configuration for changelog generation.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogConfig {
    /// Whether changelog generation is enabled
    pub enabled: bool,
    
    /// Output format for changelogs
    pub format: ChangelogFormat,
    
    /// Template for changelog generation
    pub template: Option<String>,
    
    /// Whether to include all commits or only conventional commits
    pub include_all_commits: bool,
    
    /// Sections to include in the changelog
    pub sections: Vec<ChangelogSection>,
    
    /// Custom commit types and their configurations
    pub commit_types: std::collections::HashMap<String, CommitTypeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangelogFormat {
    Markdown,
    Json,
    Html,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogSection {
    /// Section title
    pub title: String,
    
    /// Commit types to include in this section
    pub commit_types: Vec<String>,
    
    /// Whether this section is hidden if empty
    pub hide_if_empty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitTypeConfig {
    /// Display name for this commit type
    pub display_name: String,
    
    /// Whether to include in changelog
    pub include_in_changelog: bool,
    
    /// Whether this type represents a breaking change
    pub breaking: bool,
    
    /// Section this commit type belongs to
    pub section: Option<String>,
}
```

### Changesets Configuration

Configuration for changeset management.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesetsConfig {
    /// Whether changesets are required for changes
    pub required: bool,
    
    /// Whether to automatically deploy changesets
    pub auto_deploy: bool,
    
    /// Directory to store changeset files
    pub changesets_dir: String,
    
    /// Whether to validate changeset format
    pub validate_format: bool,
    
    /// Template for changeset files
    pub template: Option<String>,
    
    /// Environments that require changesets
    pub required_environments: Vec<Environment>,
}
```

## Version Management

### VersionManager

Manages package version bumping and propagation.

```rust
pub struct VersionManager<'a> {
    config: &'a MonorepoConfig,
    packages: &'a [MonorepoPackageInfo],
    root_path: &'a Path,
    strategy: Box<dyn VersioningStrategy + 'a>,
}

impl<'a> VersionManager<'a> {
    /// Creates a new version manager
    pub fn new(project: &'a MonorepoProject) -> Self;
    
    /// Bumps the version of a specific package
    pub fn bump_package_version(
        &self,
        package_name: &str,
        bump_type: VersionBumpType,
        commit_sha: Option<&str>
    ) -> Result<VersioningResult>;
    
    /// Propagates version changes to dependent packages
    pub fn propagate_version_changes(&self, package_name: &str) -> Result<PropagationResult>;
    
    /// Gets the current version of a package
    pub fn get_package_version(&self, package_name: &str) -> Result<String>;
    
    /// Calculates the next version for a given bump type
    pub fn calculate_next_version(&self, current_version: &str, bump_type: VersionBumpType) -> Result<String>;
}

/// Trait for version bumping strategies
pub trait VersioningStrategy {
    /// Bumps a package version according to the strategy
    fn bump_version(&self, package: &MonorepoPackageInfo, bump_type: VersionBumpType) -> Result<String>;
    
    /// Determines if dependents should be bumped when this package changes
    fn should_propagate(&self, package: &MonorepoPackageInfo, bump_type: VersionBumpType) -> bool;
}

/// Result of propagating version changes
#[derive(Debug, Clone)]
pub struct PropagationResult {
    /// Packages that were updated
    pub updated_packages: Vec<String>,
    
    /// Version changes applied
    pub version_changes: std::collections::HashMap<String, String>,
    
    /// Packages that were skipped
    pub skipped_packages: Vec<String>,
}
```

### VersioningResult

Result of a version bump operation.

```rust
#[derive(Debug, Clone)]
pub struct VersioningResult {
    /// Name of the package that was versioned
    pub package_name: String,
    
    /// Previous version
    pub old_version: String,
    
    /// New version after bump
    pub new_version: String,
    
    /// Type of bump that was applied
    pub bump_type: VersionBumpType,
    
    /// Packages that were updated due to propagation
    pub propagated_packages: Vec<String>,
    
    /// Commit SHA if the change was committed
    pub commit_sha: Option<String>,
    
    /// Timestamp of the version change
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl VersioningResult {
    /// Returns true if any propagation occurred
    pub fn has_propagation(&self) -> bool;
    
    /// Returns the total number of packages affected
    pub fn total_affected_packages(&self) -> usize;
}
```

## Change Detection

### ChangeDetector

Detects changes in packages and files.

```rust
pub struct ChangeDetector<'a> {
    config: &'a MonorepoConfig,
    packages: &'a [MonorepoPackageInfo],
    repository: &'a sublime_git_tools::Repo,
    root_path: &'a Path,
}

impl<'a> ChangeDetector<'a> {
    /// Creates a new change detector
    pub fn new(project: &'a MonorepoProject) -> Self;
    
    /// Detects changes in files between Git references
    pub fn detect_file_changes(&self, since: &str, until: Option<&str>) -> Result<Vec<String>>;
    
    /// Maps file changes to affected packages
    pub fn map_files_to_packages(&self, changed_files: &[String]) -> Result<Vec<String>>;
    
    /// Detects changes since the last release
    pub fn detect_changes_since_release(&self, package_name: &str) -> Result<Vec<PackageChange>>;
    
    /// Analyzes the significance of changes
    pub fn analyze_change_significance(&self, changes: &[String]) -> Result<ChangeSeverity>;
}

/// Severity of detected changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeSeverity {
    /// No significant changes
    None,
    /// Documentation or comment changes only
    Documentation,
    /// Non-breaking changes
    Minor,
    /// Potentially breaking changes
    Major,
}
```

### PackageChange

Represents a change detected in a package.

```rust
#[derive(Debug, Clone)]
pub struct PackageChange {
    /// Name of the affected package
    pub package_name: String,
    
    /// Type of change detected
    pub change_type: ChangeType,
    
    /// Significance of the change
    pub significance: ChangeSeverity,
    
    /// Suggested version bump type
    pub suggested_version_bump: VersionBumpType,
    
    /// Files that were changed in this package
    pub changed_files: Vec<String>,
    
    /// Commit SHAs associated with the changes
    pub commit_shas: Vec<String>,
    
    /// Timestamp of the change detection
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

impl PackageChange {
    /// Returns true if this change requires a version bump
    pub fn requires_version_bump(&self) -> bool;
    
    /// Returns true if this is a breaking change
    pub fn is_breaking_change(&self) -> bool;
}

/// Type of change detected in a package
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    /// Source code changes
    Source,
    /// Dependency changes
    Dependencies,
    /// Configuration changes
    Configuration,
    /// Documentation changes
    Documentation,
    /// Test changes
    Tests,
    /// Build configuration changes
    Build,
}
```

## Task Management

### TaskManager

Manages task execution across packages.

```rust
pub struct TaskManager<'a> {
    config: &'a MonorepoConfig,
    packages: &'a [MonorepoPackageInfo],
    repository: &'a sublime_git_tools::Repo,
    root_path: &'a Path,
}

impl<'a> TaskManager<'a> {
    /// Creates a new task manager
    pub fn new(project: &'a MonorepoProject) -> Result<Self>;
    
    /// Executes tasks for affected packages
    pub fn execute_tasks_for_affected_packages(&self, affected_packages: &[String]) -> Result<Vec<TaskExecutionResult>>;
    
    /// Executes a batch of tasks
    pub fn execute_tasks_batch(&self, task_names: &[String]) -> Result<Vec<TaskExecutionResult>>;
    
    /// Executes a single task across specified packages
    pub fn execute_task(&self, task_name: &str, packages: Option<&[String]>) -> Result<TaskExecutionResult>;
    
    /// Registers a new task definition
    pub fn register_task(&mut self, task: TaskDefinition) -> Result<()>;
    
    /// Gets available tasks for a package
    pub fn get_available_tasks(&self, package_name: &str) -> Result<Vec<String>>;
}

/// Definition of a task that can be executed
#[derive(Debug, Clone)]
pub struct TaskDefinition {
    /// Unique name for the task
    pub name: String,
    
    /// Human-readable description
    pub description: String,
    
    /// Command to execute
    pub command: String,
    
    /// Arguments for the command
    pub args: Vec<String>,
    
    /// Task scope (which packages to run on)
    pub scope: TaskScope,
    
    /// Conditions for when to run this task
    pub conditions: TaskConditions,
}

impl TaskDefinition {
    /// Creates a new task definition
    pub fn new(name: &str, description: &str) -> Self;
    
    /// Sets the package script to execute
    pub fn with_package_script(mut self, script_name: &str) -> Self;
    
    /// Sets the task scope
    pub fn with_scope(mut self, scope: TaskScope) -> Self;
    
    /// Adds a condition for running the task
    pub fn with_condition(mut self, condition: TaskCondition) -> Self;
}

/// Scope for task execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskScope {
    /// Run on all packages
    AllPackages,
    /// Run only on affected packages
    AffectedPackages,
    /// Run only on specific packages
    SpecificPackages(Vec<String>),
    /// Run at the root level only
    Root,
}

/// Condition for when a task should run
#[derive(Debug, Clone)]
pub enum TaskCondition {
    /// Run only if specific files have changed
    FileChanges(Vec<String>),
    /// Run only in specific environments
    Environment(Vec<Environment>),
    /// Run only for specific package types
    PackageType(Vec<String>),
    /// Custom condition
    Custom(String),
}

/// Result of task execution
#[derive(Debug, Clone)]
pub struct TaskExecutionResult {
    /// Name of the executed task
    pub task_name: String,
    
    /// Execution status
    pub status: TaskExecutionStatus,
    
    /// Packages that were processed
    pub packages_processed: Vec<String>,
    
    /// Execution statistics
    pub stats: TaskExecutionStats,
    
    /// Output from task execution
    pub output: TaskOutput,
    
    /// Execution duration
    pub duration: std::time::Duration,
}

/// Status of task execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskExecutionStatus {
    /// All tasks completed successfully
    Success,
    /// Some tasks failed but execution continued
    PartialFailure,
    /// Task execution failed completely
    Failure,
    /// Task execution was cancelled
    Cancelled,
}

/// Statistics from task execution
#[derive(Debug, Clone)]
pub struct TaskExecutionStats {
    /// Total number of packages processed
    pub packages_processed: usize,
    
    /// Number of commands executed
    pub commands_executed: usize,
    
    /// Number of successful executions
    pub successful_executions: usize,
    
    /// Number of failed executions
    pub failed_executions: usize,
}

/// Output from task execution
#[derive(Debug, Clone)]
pub struct TaskOutput {
    /// Standard output
    pub stdout: String,
    
    /// Standard error
    pub stderr: String,
    
    /// Exit codes from commands
    pub exit_codes: Vec<i32>,
}
```

## Changeset Management

### ChangesetManager

Manages changeset creation, validation, and application.

```rust
pub struct ChangesetManager<'a> {
    storage: ChangesetStorage,
    task_manager: TaskManager<'a>,
    config: &'a MonorepoConfig,
    packages: &'a [MonorepoPackageInfo],
    repository: &'a sublime_git_tools::Repo,
}

impl<'a> ChangesetManager<'a> {
    /// Creates a new changeset manager from a project
    pub fn from_project(project: &'a MonorepoProject) -> Result<Self>;
    
    /// Creates a new changeset
    pub fn create_changeset(&self, spec: ChangesetSpec) -> Result<Changeset>;
    
    /// Lists changesets matching the filter
    pub fn list_changesets(&self, filter: &ChangesetFilter) -> Result<Vec<Changeset>>;
    
    /// Applies changesets on merge
    pub fn apply_changesets_on_merge(&self, branch: &str) -> Result<Vec<ChangesetApplication>>;
    
    /// Validates a changeset
    pub fn validate_changeset(&self, changeset: &Changeset) -> Result<ValidationResult>;
    
    /// Deletes a changeset
    pub fn delete_changeset(&self, changeset_id: &str) -> Result<()>;
}

/// Specification for creating a changeset
#[derive(Debug, Clone)]
pub struct ChangesetSpec {
    /// Package this changeset applies to
    pub package: String,
    
    /// Type of version bump
    pub version_bump: VersionBumpType,
    
    /// Description of the changes
    pub description: String,
    
    /// Environments to deploy to
    pub development_environments: Vec<Environment>,
    
    /// Whether to deploy to production
    pub production_deployment: bool,
    
    /// Author of the changeset
    pub author: Option<String>,
}

/// Represents a changeset
#[derive(Debug, Clone)]
pub struct Changeset {
    /// Unique identifier
    pub id: String,
    
    /// Package this changeset applies to
    pub package: String,
    
    /// Type of version bump
    pub version_bump: VersionBumpType,
    
    /// Description of the changes
    pub description: String,
    
    /// Current status
    pub status: ChangesetStatus,
    
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Author information
    pub author: Option<String>,
}

/// Status of a changeset
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangesetStatus {
    /// Changeset is pending application
    Pending,
    /// Changeset has been applied
    Applied,
    /// Changeset was cancelled
    Cancelled,
}

/// Filter for listing changesets
#[derive(Debug, Clone, Default)]
pub struct ChangesetFilter {
    /// Filter by package name
    pub package: Option<String>,
    
    /// Filter by status
    pub status: Option<ChangesetStatus>,
    
    /// Filter by author
    pub author: Option<String>,
    
    /// Filter by date range
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Filter by date range
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
}

/// Result of applying a changeset
#[derive(Debug, Clone)]
pub struct ChangesetApplication {
    /// The applied changeset
    pub changeset: Changeset,
    
    /// Version bump result
    pub version_result: VersioningResult,
    
    /// Whether the application was successful
    pub success: bool,
    
    /// Error message if application failed
    pub error: Option<String>,
}

/// Result of changeset validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the changeset is valid
    pub is_valid: bool,
    
    /// Validation errors
    pub errors: Vec<String>,
    
    /// Validation warnings
    pub warnings: Vec<String>,
}
```

## Changelog Generation

### ChangelogManager

Manages changelog generation from Git history and changesets.

```rust
pub struct ChangelogManager {
    project: std::rc::Rc<MonorepoProject>,
}

impl ChangelogManager {
    /// Creates a new changelog manager from a project
    pub fn from_project(project: &std::rc::Rc<MonorepoProject>) -> Self;
    
    /// Creates a new changelog manager
    pub fn new(project: &MonorepoProject) -> Self;
    
    /// Generates a changelog based on the request
    pub fn generate_changelog(&self, request: ChangelogRequest) -> Result<ChangelogResult>;
    
    /// Generates a changelog for a specific package
    pub fn generate_package_changelog(&self, package_name: &str, version: &str) -> Result<ChangelogResult>;
    
    /// Generates a changelog for the entire monorepo
    pub fn generate_monorepo_changelog(&self, version: &str) -> Result<ChangelogResult>;
}

/// Request for changelog generation
#[derive(Debug, Clone)]
pub struct ChangelogRequest {
    /// Package name (None for monorepo-wide changelog)
    pub package_name: Option<String>,
    
    /// Version for the changelog
    pub version: String,
    
    /// Starting Git reference
    pub since: Option<String>,
    
    /// Ending Git reference
    pub until: Option<String>,
    
    /// Whether to write to file
    pub write_to_file: bool,
    
    /// Include all commits or only conventional commits
    pub include_all_commits: bool,
    
    /// Output file path
    pub output_path: Option<String>,
}

impl Default for ChangelogRequest {
    fn default() -> Self;
}

/// Result of changelog generation
#[derive(Debug, Clone)]
pub struct ChangelogResult {
    /// Generated changelog content
    pub content: String,
    
    /// Number of commits included
    pub commit_count: usize,
    
    /// Whether breaking changes were found
    pub has_breaking_changes: bool,
    
    /// Output file path if written to file
    pub output_path: Option<String>,
    
    /// Generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl ChangelogResult {
    /// Returns true if changelog has content
    pub fn has_content(&self) -> bool;
    
    /// Returns the number of lines in the changelog
    pub fn line_count(&self) -> usize;
}
```

## Error Types

### Error

Main error type for the crate.

```rust
#[derive(ThisError, Debug)]
pub enum Error {
    /// Git operation error
    #[error("Git error: {0}")]
    Git(#[from] sublime_git_tools::Error),
    
    /// File system operation error
    #[error("File system error: {0}")]
    FileSystem(#[from] sublime_standard_tools::Error),
    
    /// Package operation error
    #[error("Package error: {0}")]
    Package(#[from] sublime_package_tools::Error),
    
    /// Configuration error
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    /// Validation error
    #[error("Validation error: {message}")]
    Validation { message: String },
    
    /// Task execution error
    #[error("Task execution error: {message}")]
    TaskExecution { message: String },
    
    /// Changeset error
    #[error("Changeset error: {message}")]
    Changeset { message: String },
    
    /// Version management error
    #[error("Version management error: {message}")]
    VersionManagement { message: String },
    
    /// Generic error with custom message
    #[error("Error: {0}")]
    Generic(String),
}

impl Error {
    /// Creates a new generic error
    pub fn generic(message: impl Into<String>) -> Self;
    
    /// Creates a new configuration error
    pub fn configuration(message: impl Into<String>) -> Self;
    
    /// Creates a new validation error
    pub fn validation(message: impl Into<String>) -> Self;
    
    /// Creates a new task execution error
    pub fn task_execution(message: impl Into<String>) -> Self;
    
    /// Creates a new changeset error
    pub fn changeset(message: impl Into<String>) -> Self;
    
    /// Creates a new version management error
    pub fn version_management(message: impl Into<String>) -> Self;
}
```

### Result

Type alias for results using the crate's Error type.

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

## Usage Examples

### Basic Project Analysis

```rust
use sublime_monorepo_tools::{MonorepoProject, MonorepoAnalyzer, Result};

fn analyze_monorepo() -> Result<()> {
    // Initialize project
    let project = MonorepoProject::new(".")?;
    
    // Create analyzer
    let analyzer = MonorepoAnalyzer::new(&project);
    
    // Detect changes since last commit
    let changes = analyzer.detect_changes_since("HEAD~1", None)?;
    
    println!("Found {} affected packages", changes.package_changes.len());
    for change in &changes.package_changes {
        println!("- {}: {:?}", change.package_name, change.change_type);
    }
    
    Ok(())
}
```

### Version Management

```rust
use sublime_monorepo_tools::{MonorepoProject, VersionManager, VersionBumpType, Result};

fn bump_package_version() -> Result<()> {
    let mut project = MonorepoProject::new(".")?;
    let mut version_manager = VersionManager::new(&mut project);
    
    let result = version_manager.bump_package_version(
        "my-package",
        VersionBumpType::Minor,
        None
    )?;
    
    println!("Bumped {} from {} to {}", 
        result.package_name, 
        result.old_version, 
        result.new_version
    );
    
    Ok(())
}
```

### Task Execution

```rust
use sublime_monorepo_tools::{MonorepoProject, Result};
use sublime_monorepo_tools::tasks::{TaskManager, TaskDefinition, TaskScope};

fn execute_tasks() -> Result<()> {
    let project = MonorepoProject::new(".")?;
    let mut task_manager = TaskManager::new(&project)?;
    
    // Define a task
    let task = TaskDefinition::new("test", "Run tests for affected packages")
        .with_package_script("test")
        .with_scope(TaskScope::AffectedPackages);
    
    // Register and execute
    task_manager.register_task(task)?;
    let result = task_manager.execute_task("test", None)?;
    
    println!("Task completed with status: {:?}", result.status);
    
    Ok(())
}
```

This specification provides comprehensive documentation of the `sublime_monorepo_tools` API, covering all public types, methods, and usage patterns for effective monorepo management in Rust applications.