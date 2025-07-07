# Sublime Monorepo Tools

A streamlined library for CLI and daemon consumption that provides essential Node.js monorepo functionality including project management, analysis, versioning, change detection, and comprehensive automation workflows.

## Features

- **🚀 Project Management**: Direct integration with base crates for optimal performance (< 100ms startup)
- **📊 Analysis**: Dependency graph analysis and change detection with < 1s performance
- **🔄 Versioning**: Major, Minor, Patch, Snapshot with automatic propagation to dependents  
- **🔍 Change Detection**: Efficient package change analysis for CLI operations
- **⚙️ Configuration**: Comprehensive monorepo configuration management with TOML support
- **📝 Changelogs**: Conventional commits parsing with customizable templates
- **🎯 Tasks**: Synchronous task execution optimized for CLI responsiveness
- **🔗 Changesets**: Complete changeset lifecycle management for version coordination
- **🪝 Hooks**: Git hooks integration for automated quality gates
- **🎨 Beautiful UI**: Rich terminal output with progress bars, tables, and colors

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

## Automated Workflow Example

See our comprehensive [automated workflow example](examples/automated_workflow_example.rs) that demonstrates:

- **Complete Development Lifecycle**: Feature branch → changes → commit → changeset → CI → merge → bump → cleanup
- **Beautiful Terminal UI**: Progress bars, styled tables, color-coded status indicators
- **Advanced Features**: Snapshot versions, dependency propagation, conflict resolution
- **Pre-push Hooks**: Automated task execution on changed packages
- **Realistic Demo**: 4 interconnected packages with circular dependencies

Run the example:

```bash
cargo run --example automated_workflow_example
```

## Complete Monorepo Configuration

Create a `monorepo.toml` file in your repository root with comprehensive configuration:

```toml
# Main Monorepo Configuration File
# This file controls all aspects of monorepo automation and tooling

#####################################
# Workspace Configuration
#####################################
[workspace]
# Custom workspace patterns for package discovery
# These patterns define where packages are located in your monorepo
[[workspace.patterns]]
pattern = "packages/*"                    # Glob pattern for package discovery
description = "Main packages directory"  # Human-readable description
enabled = true                          # Whether this pattern is active
priority = 100                          # Higher priority patterns are checked first
package_managers = ["npm", "yarn", "pnpm"]  # Which package managers this applies to
environments = ["development", "production"]  # Which environments this applies to

[workspace.patterns.options]
include_nested = true                    # Include packages within packages
max_depth = 2                           # Maximum depth to search
exclude_patterns = ["**/node_modules", "**/dist"]  # Patterns to exclude
follow_symlinks = false                 # Whether to follow symbolic links
override_detection = false              # Override auto-detection

# Whether to merge custom patterns with auto-detected ones
merge_with_detected = true

# Package discovery settings
[workspace.discovery]
auto_detect_patterns = true            # Automatically detect workspace patterns
scan_package_json = true               # Look for workspaces in package.json
scan_pnpm_workspace = true             # Look for pnpm-workspace.yaml
scan_rush_json = true                  # Look for rush.json
cache_discovery_results = true         # Cache discovery results for performance

# Package manager specific configurations
[workspace.package_manager_configs]
npm_install_command = "npm ci"          # Command for npm installs
yarn_install_command = "yarn install --frozen-lockfile"  # Command for yarn installs
pnpm_install_command = "pnpm install --frozen-lockfile"  # Command for pnpm installs

# Validation rules for workspace structure
[workspace.validation]
require_package_json = true            # All packages must have package.json
validate_dependencies = true           # Validate internal dependencies exist
check_circular_dependencies = true     # Detect circular dependencies
enforce_naming_convention = false      # Enforce package naming patterns
naming_pattern = "@company/*"          # Pattern for package names

#####################################
# Versioning Configuration
#####################################
[versioning]
# Default versioning strategy for packages
strategy = "independent"               # "independent" | "fixed" | "fixed-group"

# Whether to automatically bump dependent packages
auto_bump_dependents = true

# How to handle dependent package bumps
dependent_bump_strategy = "inherit"    # "inherit" | "patch" | "minor" | "none"

# Pre-release configuration
[versioning.prerelease]
enabled = true                         # Enable pre-release versions
prefix = "alpha"                       # Pre-release prefix (alpha, beta, rc)
include_commit_sha = true              # Include short commit SHA
include_timestamp = false              # Include timestamp in version

# Version constraints
[versioning.constraints]
min_major_version = 0                  # Minimum major version
max_major_version = 999                # Maximum major version
allow_major_zero = true                # Allow 0.x.x versions

#####################################
# Task Configuration
#####################################
[tasks]
# Global task execution settings
default_timeout = 300                 # Default timeout in seconds (5 minutes)
max_concurrent = 4                     # Maximum concurrent tasks
continue_on_failure = false           # Continue if tasks fail
execution_mode = "parallel"           # "parallel" | "sequential" | "dependency"

# Environment variables for all tasks
[tasks.environment_variables]
NODE_ENV = "development"
CI = "true"

# Task-specific configurations
[tasks.task_configs.lint]
command = "npm run lint"               # Command to execute
working_directory = "."                # Working directory (relative to package)
timeout = 120                         # Task-specific timeout in seconds
parallel = true                       # Whether this task can run in parallel
dependencies = []                      # Tasks that must run before this one

# Conditions for when to run this task
[tasks.task_configs.lint.conditions]
on_file_changes = ["**/*.js", "**/*.ts", "**/*.jsx", "**/*.tsx"]  # Run on these file changes
environments = ["development", "production"]  # Run in these environments
package_types = ["library", "application"]    # Run for these package types

[tasks.task_configs.test]
command = "npm test"
timeout = 600                         # 10 minutes for tests
parallel = true
dependencies = ["lint"]               # Must lint before testing

[tasks.task_configs.test.conditions]
on_file_changes = ["**/*.js", "**/*.ts", "**/*.test.*", "**/*.spec.*"]

[tasks.task_configs.build]
command = "npm run build"
timeout = 300                         # 5 minutes for build
parallel = false                      # Build serially to avoid conflicts
dependencies = ["lint", "test"]       # Must lint and test before building

#####################################
# Git Hooks Configuration
#####################################
[hooks]
enabled = true                        # Enable all hooks globally

# Pre-commit hook configuration
[hooks.pre_commit]
enabled = true                        # Enable pre-commit hook
validate_changeset = true             # Ensure changesets exist for changes
run_tasks = ["lint", "typecheck"]     # Tasks to run on pre-commit
block_on_failure = true               # Block commit if tasks fail
affected_packages_only = true         # Only run on affected packages

# Pre-push hook configuration
[hooks.pre_push]
enabled = true                        # Enable pre-push hook
run_tasks = ["lint", "test", "build"] # Tasks to run on pre-push
block_on_failure = true               # Block push if tasks fail
affected_packages_only = true         # Only run on affected packages

# Post-merge hook configuration
[hooks.post_merge]
enabled = true                        # Enable post-merge hook
auto_apply_changesets = true          # Automatically apply changesets after merge
auto_generate_changelogs = true       # Generate changelogs after merge
run_tasks = ["install"]               # Tasks to run after merge

#####################################
# Changelog Configuration
#####################################
[changelog]
enabled = true                        # Enable changelog generation
format = "markdown"                   # "markdown" | "json" | "html"
include_all_commits = false           # Include all commits or only conventional commits

# Changelog sections configuration
[[changelog.sections]]
title = "Breaking Changes"            # Section title
commit_types = ["feat"]               # Commit types to include
hide_if_empty = false                 # Hide section if no commits

[[changelog.sections]]
title = "Features"
commit_types = ["feat"]
hide_if_empty = true

[[changelog.sections]]
title = "Bug Fixes"
commit_types = ["fix"]
hide_if_empty = true

[[changelog.sections]]
title = "Documentation"
commit_types = ["docs"]
hide_if_empty = true

# Custom commit type configurations
[changelog.commit_types.feat]
display_name = "Features"             # Display name for this commit type
include_in_changelog = true           # Whether to include in changelog
breaking = false                      # Whether this type represents breaking changes
section = "Features"                  # Which section this belongs to

[changelog.commit_types.fix]
display_name = "Bug Fixes"
include_in_changelog = true
breaking = false
section = "Bug Fixes"

[changelog.commit_types.docs]
display_name = "Documentation"
include_in_changelog = true
breaking = false
section = "Documentation"

[changelog.commit_types.chore]
display_name = "Chores"
include_in_changelog = false          # Don't include chores in changelog
breaking = false

#####################################
# Changesets Configuration
#####################################
[changesets]
required = true                       # Require changesets for changes
auto_deploy = true                    # Automatically deploy changesets on merge
changesets_dir = ".changesets"        # Directory to store changeset files
validate_format = true                # Validate changeset format
required_environments = ["production"]  # Environments that require changesets

#####################################
# Git Configuration
#####################################
[git]
# Default branch configuration
default_branch = "main"               # Default branch name
protected_branches = ["main", "develop"]  # Branches that require special handling

# Commit message validation
[git.commit_validation]
enforce_conventional_commits = true   # Enforce conventional commit format
require_signed_commits = false        # Require GPG signed commits
max_subject_length = 72               # Maximum commit subject length
require_issue_reference = false       # Require issue/ticket reference

# Branch naming conventions
[git.branch_naming]
enforce_naming = true                 # Enforce branch naming conventions
feature_prefix = "feature/"           # Prefix for feature branches
bugfix_prefix = "fix/"                # Prefix for bugfix branches
release_prefix = "release/"           # Prefix for release branches

#####################################
# Validation Configuration
#####################################
[validation]
# Quality gates that must pass
[validation.quality_gates]
min_test_coverage = 80.0              # Minimum test coverage percentage
max_bundle_size_mb = 5.0              # Maximum bundle size in MB
max_dependencies = 50                 # Maximum number of dependencies per package
require_readme = true                 # Require README files
require_license = true                # Require LICENSE files

# Code quality rules
[validation.code_quality]
enforce_prettier = true               # Enforce Prettier formatting
enforce_eslint = true                 # Enforce ESLint rules
require_typescript = false            # Require TypeScript usage
max_cyclomatic_complexity = 10        # Maximum cyclomatic complexity

# Security validation
[validation.security]
audit_dependencies = true             # Audit dependencies for vulnerabilities
block_vulnerable_deps = true          # Block deployment with vulnerable deps
max_vulnerability_severity = "moderate"  # Maximum allowed vulnerability severity

#####################################
# Plugin System Configuration
#####################################
[plugins]
enabled = true                        # Enable plugin system

# Plugin configurations
[[plugins.plugin_configs]]
name = "semantic-release"             # Plugin name
enabled = true                        # Whether this plugin is enabled
config = { "branches" = ["main"] }    # Plugin-specific configuration

[[plugins.plugin_configs]]
name = "conventional-changelog"
enabled = true
config = { "preset" = "angular" }

#####################################
# Environment Configuration
#####################################
# Define deployment environments
environments = [
    "development",                    # Development environment
    "staging",                        # Staging environment  
    "integration",                    # Integration testing environment
    "production"                      # Production environment
]

# Environment-specific configurations
[environments.development]
auto_deploy = true                    # Auto-deploy to development
require_approval = false              # No approval required
run_tests = true                      # Run tests before deploy

[environments.staging]
auto_deploy = false                   # Manual deploy to staging
require_approval = true               # Require approval
run_tests = true                      # Run tests before deploy
approval_count = 1                    # Number of approvals required

[environments.production]
auto_deploy = false                   # Manual deploy to production
require_approval = true               # Require approval
run_tests = true                      # Run tests before deploy
approval_count = 2                    # Number of approvals required
require_changeset = true              # Require changeset for production
```

## Configuration Sections Explained

### 📁 Workspace Configuration

Controls package discovery and workspace management:

- **`patterns`**: Define where packages are located using glob patterns
- **`merge_with_detected`**: Whether to combine custom patterns with auto-detected ones
- **`discovery`**: Settings for automatic package discovery
- **`validation`**: Rules for validating workspace structure

### 🔄 Versioning Configuration

Manages how versions are bumped and propagated:

- **`strategy`**: How versions are managed across packages (`independent`, `fixed`, `fixed-group`)
- **`auto_bump_dependents`**: Whether dependent packages are automatically bumped
- **`dependent_bump_strategy`**: How dependent packages are bumped (`inherit`, `patch`, `minor`, `none`)
- **`prerelease`**: Configuration for pre-release versions

### 🎯 Task Configuration

Controls task execution across packages:

- **`default_timeout`**: Global timeout for all tasks
- **`max_concurrent`**: Number of tasks that can run simultaneously
- **`execution_mode`**: How tasks are executed (`parallel`, `sequential`, `dependency`)
- **`task_configs`**: Specific configuration for individual tasks
- **`conditions`**: When tasks should run (file changes, environments, package types)

### 🪝 Hooks Configuration

Git hooks integration for automated quality gates:

- **`pre_commit`**: Validates changesets and runs quality checks before commits
- **`pre_push`**: Runs comprehensive tests before pushing to remote
- **`post_merge`**: Automates changeset application and changelog generation after merges

### 📝 Changelog Configuration

Controls automated changelog generation:

- **`format`**: Output format (`markdown`, `json`, `html`)
- **`sections`**: How to organize changelog entries
- **`commit_types`**: Configuration for different types of commits
- **`include_all_commits`**: Whether to include all commits or only conventional ones

### 📦 Changesets Configuration

Manages the changeset lifecycle:

- **`required`**: Whether changesets are mandatory for changes
- **`auto_deploy`**: Whether to automatically deploy changesets on merge
- **`validate_format`**: Whether to validate changeset file format
- **`required_environments`**: Which environments require changesets

### 🔒 Validation Configuration

Quality gates and code standards:

- **`quality_gates`**: Minimum standards that must be met
- **`code_quality`**: Code formatting and linting requirements  
- **`security`**: Security validation and vulnerability management

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
    println!("Default environment: {:?}", config.environments);
    
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

## Changeset Management

Work with changesets for version coordination:

```rust
use sublime_monorepo_tools::changesets::{ChangesetManager, ChangesetSpec, ChangesetFilter, ChangesetStatus};
use sublime_monorepo_tools::{VersionBumpType, Environment};

fn manage_changesets() -> Result<()> {
    let project = MonorepoProject::new(".")?;
    let changeset_manager = ChangesetManager::from_project(&project)?;
    
    // Create a new changeset
    let spec = ChangesetSpec {
        package: "my-package".to_string(),
        version_bump: VersionBumpType::Minor,
        description: "Add new feature".to_string(),
        development_environments: vec![Environment::Development, Environment::Staging],
        production_deployment: true,
        author: Some("developer@company.com".to_string()),
    };
    
    let changeset = changeset_manager.create_changeset(spec)?;
    println!("Created changeset: {}", changeset.id);
    
    // List pending changesets
    let filter = ChangesetFilter {
        status: Some(ChangesetStatus::Pending),
        ..Default::default()
    };
    
    let pending = changeset_manager.list_changesets(&filter)?;
    println!("Found {} pending changesets", pending.len());
    
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

## API Documentation

For comprehensive API documentation, see [SPEC.md](SPEC.md) which provides detailed specifications of all public types, methods, and usage patterns.

## Examples

- **[Automated Workflow](examples/automated_workflow_example.rs)**: Complete development lifecycle with beautiful terminal UI
- **Terminal Components**: Progress bars, tables, and interactive prompts
- **Advanced Features**: Snapshot versions, dependency propagation, conflict resolution

## License

This project is licensed under the MIT License - see the LICENSE file for details.