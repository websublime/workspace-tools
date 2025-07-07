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

Create a `monorepo.toml` file in your repository root:

```toml
#####################################
# Versioning Configuration
#####################################
[versioning]
# Default version bump type when not specified
default_bump = "patch"                # "major" | "minor" | "patch" | "snapshot"

# Whether to propagate version changes to dependents
propagate_changes = true

# Snapshot version format
snapshot_format = "{version}-snapshot.{sha}"

# Version tag prefix
tag_prefix = "v"

# Whether to create tags automatically
auto_tag = true

# Optional version constraint for validating version formats
version_constraint = "^1.0.0"

#####################################
# Task Configuration
#####################################
[tasks]
# Default tasks to run on changes
default_tasks = ["test", "lint"]

# Whether to run tasks in parallel
parallel = true

# Maximum concurrent tasks
max_concurrent = 4

# Task timeout in seconds
timeout = 300

# Task groups
[tasks.groups]
quality = ["lint", "typecheck", "test"]
build = ["clean", "compile", "bundle"]
release = ["quality", "build", "docs"]

# Deployment tasks for each environment
[tasks.deployment_tasks]
development = ["build", "deploy-dev"]
staging = ["build", "test", "deploy-staging"]
production = ["build", "test", "security-scan", "deploy-prod"]

# Performance and timing configuration
[tasks.performance]
# Hook execution timeout in seconds
hook_timeout = 300

# Version planning estimation per package in seconds
version_planning_per_package = 5

# Cache duration for task results in seconds
cache_duration = 300

# Large project configuration overrides
[tasks.performance.large_project]
max_concurrent = 8
timeout = 600

# Workflow impact thresholds
[tasks.performance.impact_thresholds]
medium_impact_files = 5
high_impact_files = 15

#####################################
# Changelog Configuration
#####################################
[changelog]
# How to group commits
grouping = "type"                     # "type" | "scope" | "none"

# Output format
output_format = "markdown"            # "markdown" | "text" | "json"

# Whether to include breaking changes section
include_breaking_changes = true

# Conventional commit type mappings
[changelog.conventional_commit_types]
feat = "Features"
fix = "Bug Fixes"
docs = "Documentation"
style = "Styles"
refactor = "Code Refactoring"
perf = "Performance Improvements"
test = "Tests"
build = "Build System"
ci = "Continuous Integration"
chore = "Chores"
revert = "Reverts"

# Changelog template configuration
[changelog.template]
header_template = "# Changelog\n\nAll notable changes to this project will be documented in this file.\n\n"
section_template = "## [{version}] - {date}\n\n"
commit_template = "- {description} ([{hash}]({url}))\n"
footer_template = "\n---\n\nGenerated by [Sublime Monorepo Tools](https://github.com/websublime/workspace-node-tools)\n"

#####################################
# Git Hooks Configuration
#####################################
[hooks]
# Whether hooks are enabled
enabled = true

# Custom hooks directory
hooks_dir = ".git/hooks"

# Pre-commit hook configuration
[hooks.pre_commit]
enabled = true
validate_changeset = true
run_tasks = ["lint"]
custom_script = "./scripts/pre-commit.sh"

# Pre-push hook configuration
[hooks.pre_push]
enabled = true
validate_changeset = false
run_tasks = ["test", "build"]
custom_script = "./scripts/pre-push.sh"

# Post-merge hook configuration
[hooks.post_merge]
enabled = true
validate_changeset = false
run_tasks = ["install"]
custom_script = "./scripts/post-merge.sh"

#####################################
# Changesets Configuration
#####################################
[changesets]
# Whether changesets are required
required = true

# Changeset directory
changeset_dir = ".changesets"

# Default environments for new changesets
default_environments = ["development", "staging"]

# Whether to auto-deploy to environments
auto_deploy = false

# Changeset filename format
filename_format = "{timestamp}-{branch}-{hash}.json"

#####################################
# Plugin System Configuration
#####################################
[plugins]
# Enabled plugins
enabled = ["semantic-release", "conventional-changelog"]

# Plugin directories
plugin_dirs = ["./plugins", "./node_modules/.plugins"]

# Plugin-specific configurations
[plugins.configs]
semantic-release = { branches = ["main"] }
conventional-changelog = { preset = "angular" }

#####################################
# Workspace Configuration
#####################################
[workspace]
# Whether to merge custom patterns with auto-detected ones
merge_with_detected = true

# Custom workspace patterns
[[workspace.patterns]]
pattern = "packages/*"
description = "Main packages directory"
enabled = true
priority = 100
package_managers = ["npm", "yarn", "pnpm"]
environments = ["development", "staging", "production"]

[workspace.patterns.options]
include_nested = true
max_depth = 2
exclude_patterns = ["**/node_modules", "**/dist"]
follow_symlinks = false
override_detection = false

# Workspace validation rules
[workspace.validation]
require_pattern_matches = false
warn_orphaned_packages = true
validate_naming = false
naming_patterns = ["@company/*"]
validate_structure = false
required_files = ["package.json"]

# Package discovery settings
[workspace.discovery]
auto_detect = true
scan_common_patterns = true
common_patterns = ["packages/*", "apps/*", "libs/*", "services/*", "tools/*", "modules/*"]
exclude_directories = ["node_modules", ".git", "dist", "build", "coverage"]
max_scan_depth = 3
cache_results = true
cache_duration = 300

# Package manager specific configurations
[workspace.package_manager_configs]
# npm configuration
[workspace.package_manager_configs.npm]
workspaces_override = ["packages/*"]
use_workspaces = true
options = { fund = false }

# Yarn configuration
[workspace.package_manager_configs.yarn]
workspaces_override = ["packages/*"]
nohoist_patterns = ["**/react-native", "**/react-native/**"]
use_workspaces = true
version = "classic"                   # "classic" | "berry" | "auto"
options = { silent = true }

# pnpm configuration
[workspace.package_manager_configs.pnpm]
packages_override = ["packages/*"]
use_workspaces = true
filter_options = ["--filter", "!docs"]
options = { shamefully_hoist = false }

# bun configuration
[workspace.package_manager_configs.bun]
workspaces_override = ["packages/*"]
use_workspaces = true
options = { install_optional = false }

#####################################
# Enterprise Registry Configuration  
#####################################
[workspace.tool_configs]
# Registry URL patterns and their types
[workspace.tool_configs.registry_patterns]
"registry.npmjs.org" = "npm"
"npm.pkg.github.com" = "github"
"pkgs.dev.azure.com" = "azure"
"gitlab.com" = "gitlab"
"npm.company.com" = "company"

# Authentication environment variables for each registry
[workspace.tool_configs.auth_env_vars]
npm = ["NPM_TOKEN"]
github = ["GITHUB_TOKEN", "NPM_TOKEN"]
azure = ["AZURE_TOKEN"]
gitlab = ["GITLAB_TOKEN"]
company = ["COMPANY_NPM_TOKEN"]

#####################################
# Git Configuration
#####################################
[git]
# Default reference for change detection
default_since_ref = "HEAD~1"

# Default target for comparisons
default_until_ref = "HEAD"

# Remote name for push operations
default_remote = "origin"

# Branch configuration
[git.branches]
main_branches = ["main", "master", "trunk", "develop"]
develop_branches = ["develop", "dev", "development"]
release_prefixes = ["release/", "releases/", "rel/"]
feature_prefixes = ["feature/", "feat/", "features/"]
hotfix_prefixes = ["hotfix/", "fix/", "bugfix/"]
default_base_branch = "main"

# Repository hosting configuration
[git.repository]
provider = "github"                   # "github" | "gitlab" | "bitbucket" | "azure_devops" | "custom"
base_url = "github.com"
auto_detect = true
url_override = "https://github.com/company/repo"

[git.repository.url_patterns]
commit_url = "https://{base_url}/{owner}/{repo}/commit/{hash}"
compare_url = "https://{base_url}/{owner}/{repo}/compare/{from}...{to}"

#####################################
# Validation Configuration
#####################################
[validation]
# Task priority configuration
[validation.task_priorities]
low = 0
normal = 50
high = 100
critical = 200

# Change detection rules
[validation.change_detection_rules]
dependency_changes_priority = 100
source_code_changes_priority = 80
configuration_changes_priority = 70
test_changes_priority = 60
documentation_changes_priority = 50

# Version bump rules
[validation.version_bump_rules]
breaking_changes_priority = 100
feature_changes_priority = 80
dependency_changes_priority = 70
patch_changes_priority = 60

# Quality gates
[validation.quality_gates]
min_test_coverage = 80.0
max_cyclomatic_complexity = 10
max_file_size_bytes = 100000
max_lines_per_file = 1000
max_dependencies_per_package = 50
min_documentation_coverage = 70.0

#####################################
# Environment Configuration
#####################################
# Define deployment environments
environments = ["development", "staging", "integration", "production"]
```

## Configuration Sections Explained

### 🔄 Versioning Configuration

Manages how versions are bumped and propagated:

- **`default_bump`**: Default version bump type (`major`, `minor`, `patch`, `snapshot`)
- **`propagate_changes`**: Whether to propagate version changes to dependents
- **`snapshot_format`**: Template for snapshot version generation
- **`tag_prefix`**: Prefix for Git tags (e.g., "v" for v1.0.0)
- **`auto_tag`**: Whether to create Git tags automatically

### 🎯 Task Configuration

Controls task execution across packages:

- **`default_tasks`**: Tasks to run when packages change
- **`parallel`**: Whether to run tasks in parallel
- **`max_concurrent`**: Maximum number of concurrent tasks
- **`timeout`**: Global timeout for task execution
- **`groups`**: Logical groupings of related tasks
- **`deployment_tasks`**: Environment-specific task definitions
- **`performance`**: Timing and performance optimizations

### 📝 Changelog Configuration

Controls automated changelog generation:

- **`grouping`**: How to group commits (`type`, `scope`, `none`)
- **`output_format`**: Output format (`markdown`, `text`, `json`)
- **`include_breaking_changes`**: Whether to include breaking changes section
- **`conventional_commit_types`**: Mapping of commit types to display names
- **`template`**: Customizable templates for changelog sections

### 🪝 Hooks Configuration

Git hooks integration for automated quality gates:

- **`enabled`**: Whether hooks are globally enabled
- **`hooks_dir`**: Directory for custom hook scripts
- **`pre_commit`**: Pre-commit validation and task execution
- **`pre_push`**: Pre-push testing and validation
- **`post_merge`**: Post-merge automation and cleanup

### 📦 Changesets Configuration

Manages the changeset lifecycle:

- **`required`**: Whether changesets are mandatory for changes
- **`changeset_dir`**: Directory to store changeset files
- **`default_environments`**: Default environments for new changesets
- **`auto_deploy`**: Whether to automatically deploy changesets
- **`filename_format`**: Template for changeset filenames

### 🔌 Plugin System Configuration

Simple plugin system for extensibility:

- **`enabled`**: List of enabled plugins
- **`plugin_dirs`**: Directories to search for plugins
- **`configs`**: Plugin-specific configuration objects

### 📁 Workspace Configuration

Controls package discovery and workspace management:

- **`patterns`**: Custom workspace patterns for package discovery
- **`merge_with_detected`**: Whether to combine custom patterns with auto-detected ones
- **`validation`**: Rules for validating workspace structure
- **`discovery`**: Settings for automatic package discovery
- **`package_manager_configs`**: Package manager specific configurations
- **`tool_configs`**: Enterprise registry and authentication configuration

### 🏢 Enterprise Registry Configuration

Configure custom npm registries and authentication for enterprise environments:

- **`registry_patterns`**: Map registry URLs to registry types for proper authentication
- **`auth_env_vars`**: Environment variables for each registry type's authentication
- **`config_file_patterns`**: Configuration file discovery patterns
- **`default_task_groups`**: Default task groups for different operations

**Example enterprise setup:**
```toml
[workspace.tool_configs.registry_patterns]
"npm.company.com" = "company"
"registry.internal.dev" = "internal"

[workspace.tool_configs.auth_env_vars]
company = ["COMPANY_NPM_TOKEN", "COMPANY_AUTH_TOKEN"]
internal = ["INTERNAL_REGISTRY_TOKEN"]
```

### 🔧 Git Configuration

Git operations and repository management:

- **`default_since_ref`**: Default reference for change detection
- **`default_until_ref`**: Default target for comparisons
- **`default_remote`**: Remote name for push operations
- **`branches`**: Branch classification and naming patterns
- **`repository`**: Repository hosting provider configuration

### 🔒 Validation Configuration

Quality gates and validation rules:

- **`task_priorities`**: Priority levels for different task types
- **`change_detection_rules`**: Rules for analyzing change significance
- **`version_bump_rules`**: Rules for determining version bump types
- **`quality_gates`**: Minimum quality standards that must be met

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