//! Configuration management for monorepo tools
//!
//! This module provides a comprehensive configuration system for managing monorepo settings,
//! including workspace patterns, versioning strategies, task management, changelogs, git hooks,
//! changesets, and plugin configurations.
//!
//! ## Overview
//!
//! The configuration system is designed to be flexible and extensible, supporting multiple
//! file formats (TOML, JSON, YAML) and providing sensible defaults for all settings. The
//! configuration is structured into logical sections, each managing a specific aspect of
//! monorepo operations.
//!
//! ## Configuration File
//!
//! By default, the configuration is stored in `monorepo.toml` at the repository root.
//! The system will automatically search for configuration files in the following order:
//! - `monorepo.toml`
//! - `monorepo.json`
//! - `monorepo.yaml` / `monorepo.yml`
//! - `.monorepo/config.toml`
//! - `.monorepo/config.json`
//! - `.monorepo/config.yaml` / `.monorepo/config.yml`
//!
//! ## Main Configuration Sections
//!
//! ### Workspace Configuration
//!
//! Manages workspace patterns for package discovery and organization:
//!
//! ```toml
//! [[workspace.patterns]]
//! pattern = "packages/*"
//! description = "Core packages"
//! priority = 100
//! enabled = true
//! package_managers = ["npm", "pnpm"]
//! environments = ["development", "production"]
//!
//! [[workspace.patterns]]
//! pattern = "apps/**"
//! description = "Application packages"
//! priority = 90
//! enabled = true
//!
//! [workspace.patterns.options]
//! include_nested = true
//! max_depth = 3
//! exclude_patterns = ["**/node_modules"]
//! ```
//!
//! ### Versioning Configuration
//!
//! Controls version management and propagation:
//!
//! ```toml
//! [versioning]
//! default_bump = "patch"
//! propagate_changes = true
//! snapshot_format = "{version}-snapshot.{sha}"
//! tag_prefix = "v"
//! auto_tag = true
//! ```
//!
//! ### Tasks Configuration
//!
//! Defines task execution settings and performance parameters:
//!
//! ```toml
//! [tasks]
//! default_tasks = ["test", "lint"]
//! parallel = true
//! max_concurrent = 4
//! timeout = 300
//!
//! [tasks.performance]
//! hook_timeout = 300
//! version_planning_per_package = 5
//! cache_duration = 300
//!
//! [tasks.performance.large_project]
//! max_concurrent = 8
//! timeout = 600
//! ```
//!
//! ### Hooks Configuration
//!
//! Manages git hooks for automated workflows:
//!
//! ```toml
//! [hooks]
//! enabled = true
//!
//! [hooks.pre_commit]
//! enabled = true
//! validate_changeset = true
//! run_tasks = ["lint"]
//!
//! [hooks.pre_push]
//! enabled = true
//! run_tasks = ["test", "build"]
//! ```
//!
//! ## Usage Examples
//!
//! ### Loading and Saving Configuration
//!
//! ```rust,no_run
//! use sublime_monorepo_tools::config::{ConfigManager, VersionBumpType};
//!
//! // Load configuration from file
//! let manager = ConfigManager::load_from_file("monorepo.toml")?;
//!
//! // Access configuration sections
//! let versioning = manager.get_versioning();
//! let workspace = manager.get_workspace();
//!
//! // Update configuration
//! let updated_manager = manager.with_update(|config| {
//!     config.versioning.default_bump = VersionBumpType::Minor;
//! })?;
//!
//! // Save configuration
//! updated_manager.save_to_file("monorepo.toml")?;
//! # Ok::<(), sublime_monorepo_tools::error::Error>(())
//! ```
//!
//! ### Working with Workspace Patterns
//!
//! ```rust,no_run
//! use sublime_monorepo_tools::config::{ConfigManager, WorkspacePattern, PackageManagerType};
//!
//! let mut manager = ConfigManager::new();
//!
//! // Add a workspace pattern
//! let pattern = WorkspacePattern {
//!     pattern: "services/*".to_string(),
//!     description: Some("Microservices".to_string()),
//!     enabled: true,
//!     priority: 85,
//!     ..Default::default()
//! };
//! manager.add_workspace_pattern(pattern)?;
//!
//! // Get filtered patterns
//! let npm_patterns = manager.get_workspace_patterns(
//!     Some(PackageManagerType::Npm),
//!     None
//! );
//!
//! // Match patterns against packages
//! let matches = manager.pattern_matches_package("services/*", "services/api");
//! # Ok::<(), sublime_monorepo_tools::error::Error>(())
//! ```
//!
//! ### Package Manager Configuration
//!
//! ```rust,no_run
//! use sublime_monorepo_tools::config::{ConfigManager, PackageManagerType};
//!
//! let manager = ConfigManager::new();
//!
//! // Get patterns for specific package manager
//! let pnpm_patterns = manager.get_package_manager_patterns(
//!     PackageManagerType::Pnpm
//! );
//!
//! // Get effective workspace patterns with auto-detection
//! let auto_detected = vec!["packages/*".to_string()];
//! let effective = manager.get_effective_workspace_patterns(
//!     auto_detected,
//!     Some(PackageManagerType::Npm),
//!     None
//! );
//! ```
//!
//! ### Configuration Validation
//!
//! ```rust,no_run
//! use sublime_monorepo_tools::config::{ConfigManager, MonorepoConfig};
//!
//! let manager = ConfigManager::new();
//! let config = MonorepoConfig::default();
//!
//! // Validate configuration
//! manager.validate_config(&config)?;
//!
//! // Validate workspace patterns
//! let existing_packages = vec![
//!     "packages/core".to_string(),
//!     "apps/web".to_string(),
//! ];
//! let errors = manager.validate_workspace_config(&existing_packages);
//! if !errors.is_empty() {
//!     for error in errors {
//!         eprintln!("Validation error: {}", error);
//!     }
//! }
//! # Ok::<(), sublime_monorepo_tools::error::Error>(())
//! ```
//!
//! ### Auto-save Functionality
//!
//! ```rust,no_run
//! use sublime_monorepo_tools::config::ConfigManager;
//!
//! let mut manager = ConfigManager::load_from_file("monorepo.toml")?;
//!
//! // Enable auto-save
//! manager.set_auto_save(true);
//!
//! // Any updates will now be automatically saved
//! manager.update(|config| {
//!     config.versioning.tag_prefix = "release-".to_string();
//! })?; // Automatically saved to file
//! # Ok::<(), sublime_monorepo_tools::error::Error>(())
//! ```
//!
//! ## Configuration Discovery
//!
//! The system provides automatic configuration file discovery:
//!
//! ```rust,no_run
//! use sublime_monorepo_tools::config::ConfigManager;
//! use std::env;
//!
//! // Find configuration file starting from current directory
//! if let Some(config_path) = ConfigManager::find_config_file(env::current_dir()?) {
//!     let manager = ConfigManager::load_from_file(config_path)?;
//!     // Use the configuration
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Default Configuration
//!
//! If no configuration file exists, the system provides sensible defaults:
//! - Workspace patterns: Empty (relies on auto-detection)
//! - Versioning: Patch bumps, with version propagation enabled
//! - Tasks: Run "test" and "lint" in parallel with 4 concurrent tasks
//! - Hooks: Pre-commit runs lint, pre-push runs test and build
//! - Changesets: Stored in `.changesets` directory
//! - Plugins: Empty (no plugins enabled by default)
//!
//! ## Environment-specific Configuration
//!
//! The configuration supports environment-specific settings:
//!
//! ```toml
//! environments = ["development", "staging", "production"]
//!
//! [[workspace.patterns]]
//! pattern = "packages/*"
//! environments = ["development", "production"]
//!
//! [tasks.deployment_tasks]
//! development = ["test", "build:dev"]
//! staging = ["test", "build:staging", "deploy:staging"]
//! production = ["test", "build:prod", "deploy:prod"]
//! ```
//!
//! ## Complete Configuration Example
//!
//! Here's a comprehensive example showing all available configuration options:
//!
//! ```toml
//! # Monorepo Configuration - All Available Options
//! # This example shows every configuration option with explanatory comments
//!
//! # Define deployment environments used throughout the configuration
//! environments = ["development", "staging", "integration", "production"]
//!
//! # ============================================================================
//! # WORKSPACE CONFIGURATION
//! # ============================================================================
//! [workspace]
//! # Whether to merge custom patterns with auto-detected ones
//! merge_with_detected = true
//!
//! # Workspace pattern definitions
//! [[workspace.patterns]]
//! pattern = "packages/*"
//! description = "Core library packages"
//! enabled = true
//! priority = 100  # Higher priority patterns are processed first
//! package_managers = ["npm", "pnpm", "yarn"]  # Optional: limit to specific managers
//! environments = ["development", "production"]  # Optional: limit to specific environments
//!
//! [workspace.patterns.options]
//! include_nested = false  # Don't include nested packages
//! max_depth = 1  # Maximum directory depth for package discovery
//! exclude_patterns = ["**/dist", "**/node_modules", "**/.turbo"]
//! follow_symlinks = false
//! override_detection = false  # If true, overrides auto-detection
//!
//! [[workspace.patterns]]
//! pattern = "apps/**"
//! description = "Application packages with nested structure"
//! enabled = true
//! priority = 90
//!
//! [workspace.patterns.options]
//! include_nested = true
//! max_depth = 3
//! exclude_patterns = ["**/node_modules", "**/build", "**/coverage"]
//!
//! [[workspace.patterns]]
//! pattern = "@company/*"
//! description = "Scoped packages"
//! enabled = true
//! priority = 95
//! package_managers = ["npm", "pnpm"]
//!
//! # Package discovery settings
//! [workspace.discovery]
//! auto_detect = true
//! scan_common_patterns = true
//! common_patterns = ["packages/*", "apps/*", "libs/*", "services/*", "tools/*"]
//! exclude_directories = ["node_modules", ".git", "dist", "build", "coverage", ".next", ".nuxt"]
//! max_scan_depth = 3
//! cache_results = true
//! cache_duration = 300  # 5 minutes
//!
//! # Workspace validation rules
//! [workspace.validation]
//! require_pattern_matches = true  # Ensure all patterns match at least one package
//! warn_orphaned_packages = true
//! validate_naming = true
//! naming_patterns = ["@company/*", "company-*", "*-lib", "*-app"]
//! validate_structure = true
//! required_files = ["package.json", "README.md"]
//!
//! # Package manager specific configurations
//! [workspace.package_manager_configs.npm]
//! workspaces_override = ["packages/*", "apps/*"]  # Override package.json workspaces
//! use_workspaces = true
//!
//! [workspace.package_manager_configs.yarn]
//! workspaces_override = ["packages/*", "apps/*", "tools/*"]
//! nohoist_patterns = ["**/react-native", "**/react-native/**"]
//! use_workspaces = true
//! version = "berry"  # "classic", "berry", or "auto"
//!
//! [workspace.package_manager_configs.pnpm]
//! packages_override = ["packages/**", "apps/**", "!**/test/**"]
//! use_workspaces = true
//! filter_options = ["--filter", "{packages/**}", "--filter", "{apps/**}"]
//!
//! [workspace.package_manager_configs.bun]
//! workspaces_override = ["packages/*", "apps/*"]
//! use_workspaces = true
//!
//! # Package manager commands configuration
//! [workspace.package_manager_commands]
//! default_manager = "npm"
//! extra_args_separator = "--"
//!
//! [workspace.package_manager_commands.commands]
//! npm = "npm"
//! yarn = "yarn"
//! pnpm = "pnpm"
//! bun = "bun"
//!
//! [workspace.package_manager_commands.version_args]
//! npm = ["--version"]
//! yarn = ["--version"]
//! pnpm = ["--version"]
//! bun = ["--version"]
//!
//! [workspace.package_manager_commands.script_run_args]
//! npm = ["run"]
//! yarn = ["run"]
//! pnpm = ["run"]
//! bun = ["run"]
//!
//! # File pattern configurations
//! [workspace.file_patterns]
//! package_files = ["package.json", "package-lock.json", "yarn.lock", "pnpm-lock.yaml", "bun.lockb"]
//! lock_files = ["package-lock.json", "yarn.lock", "pnpm-lock.yaml", "bun.lockb"]
//! source_patterns = ["src/**/*.{ts,js,tsx,jsx}", "lib/**/*.{ts,js,tsx,jsx}"]
//! test_patterns = ["**/*.{test,spec}.{ts,js,tsx,jsx}", "**/__tests__/**/*", "**/test/**/*"]
//! docs_patterns = ["**/*.md", "**/docs/**/*", "**/README*", "**/CHANGELOG*"]
//! config_patterns = ["**/*.config.{js,ts,json}", "**/.eslintrc*", "**/tsconfig*.json"]
//! build_output_patterns = ["dist", "build", "coverage", "tmp", ".next", ".nuxt", ".turbo"]
//!
//! # Tool configurations
//! [workspace.tool_configs]
//! config_file_patterns = [
//!     ".monorepo/config.toml",
//!     ".monorepo/config.json",
//!     ".monorepo/config.yaml",
//!     "monorepo.config.toml"
//! ]
//!
//! [workspace.tool_configs.registry_patterns]
//! "registry.npmjs.org" = "npm"
//! "npm.pkg.github.com" = "github"
//! "pkgs.dev.azure.com" = "azure"
//! "gitlab.com" = "gitlab"
//!
//! [workspace.tool_configs.auth_env_vars]
//! npm = ["NPM_TOKEN"]
//! github = ["GITHUB_TOKEN", "NPM_TOKEN"]
//! azure = ["AZURE_TOKEN"]
//! gitlab = ["GITLAB_TOKEN"]
//!
//! [workspace.tool_configs.default_task_groups]
//! quality = ["lint", "typecheck", "test"]
//! build = ["clean", "compile", "bundle"]
//! release = ["quality", "build", "docs"]
//!
//! # ============================================================================
//! # VERSIONING CONFIGURATION
//! # ============================================================================
//! [versioning]
//! default_bump = "patch"  # "major", "minor", "patch", or "snapshot"
//! propagate_changes = true  # Update dependent packages when versions change
//! snapshot_format = "{version}-snapshot.{sha}"  # Format for snapshot versions
//! tag_prefix = "v"  # Prefix for version tags
//! auto_tag = true  # Automatically create git tags
//! version_constraint = "^\\d+\\.\\d+\\.\\d+$"  # Optional: regex for version validation
//!
//! # ============================================================================
//! # TASK MANAGEMENT CONFIGURATION
//! # ============================================================================
//! [tasks]
//! default_tasks = ["test", "lint", "typecheck"]
//! parallel = true
//! max_concurrent = 4
//! timeout = 300  # 5 minutes
//!
//! # Task groups for common workflows
//! [tasks.groups]
//! ci = ["lint", "typecheck", "test", "build"]
//! release = ["ci", "docs", "publish"]
//! dev = ["lint", "test:watch"]
//!
//! # Environment-specific deployment tasks
//! [tasks.deployment_tasks]
//! development = ["test", "build:dev", "deploy:dev"]
//! staging = ["test", "build:staging", "e2e:staging", "deploy:staging"]
//! production = ["test", "build:prod", "e2e:prod", "deploy:prod", "smoke-test"]
//!
//! # Performance configuration
//! [tasks.performance]
//! hook_timeout = 300  # 5 minutes for hook execution
//! version_planning_per_package = 5  # Seconds per package for version planning
//! cache_duration = 300  # 5 minutes cache duration
//!
//! # Large project optimizations
//! [tasks.performance.large_project]
//! max_concurrent = 8  # More parallelism for large projects
//! timeout = 600  # 10 minutes timeout
//!
//! # Impact thresholds for workflow decisions
//! [tasks.performance.impact_thresholds]
//! medium_impact_files = 5
//! high_impact_files = 15
//!
//! # ============================================================================
//! # CHANGELOG CONFIGURATION
//! # ============================================================================
//! [changelog]
//! grouping = "type"  # "type", "scope", or "none"
//! output_format = "markdown"  # "markdown", "text", or "json"
//! include_breaking_changes = true
//!
//! # Conventional commit type mappings
//! [changelog.conventional_commit_types]
//! feat = "Features"
//! fix = "Bug Fixes"
//! docs = "Documentation"
//! style = "Styles"
//! refactor = "Code Refactoring"
//! perf = "Performance Improvements"
//! test = "Tests"
//! build = "Build System"
//! ci = "Continuous Integration"
//! chore = "Chores"
//! revert = "Reverts"
//!
//! # Changelog templates
//! [changelog.template]
//! header_template = """
//! # Changelog
//!
//! All notable changes to this project will be documented in this file.
//!
//! The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
//! and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
//!
//! """
//! section_template = "## [{version}] - {date}\n\n"
//! commit_template = "- {description} ([{hash}]({url})) by @{author}\n"
//! footer_template = "\n---\n\nGenerated by [Sublime Monorepo Tools](https://github.com/websublime/workspace-node-tools)\n"
//!
//! # ============================================================================
//! # GIT HOOKS CONFIGURATION
//! # ============================================================================
//! [hooks]
//! enabled = true
//! hooks_dir = ".monorepo/hooks"  # Optional: custom hooks directory
//!
//! [hooks.pre_commit]
//! enabled = true
//! validate_changeset = true  # Ensure changeset exists for changes
//! run_tasks = ["lint", "typecheck"]
//! custom_script = ".monorepo/scripts/pre-commit.sh"  # Optional: custom script
//!
//! [hooks.pre_push]
//! enabled = true
//! validate_changeset = false
//! run_tasks = ["test", "build"]
//! custom_script = ".monorepo/scripts/pre-push.sh"
//!
//! [hooks.post_merge]
//! enabled = true
//! validate_changeset = false
//! run_tasks = ["install", "build"]
//!
//! # ============================================================================
//! # CHANGESETS CONFIGURATION
//! # ============================================================================
//! [changesets]
//! enabled = true
//! changeset_dir = ".changesets"
//! commit_message = "chore: add changeset for {packages}"
//! required_approval_count = 2  # Number of approvals needed
//! auto_merge = true  # Automatically merge approved changesets
//! require_changeset_for_all = false  # Require changesets for all changes
//!
//! # Required fields in changesets
//! required_fields = ["summary", "type", "packages"]
//!
//! # Allowed change types
//! allowed_types = ["major", "minor", "patch", "none"]
//!
//! # Templates for different change types
//! [changesets.templates]
//! major = """
//! ---
//! "{package}": major
//! ---
//!
//! ### Breaking Changes
//!
//! {summary}
//!
//! #### Migration Guide
//!
//! {migration}
//! """
//!
//! minor = """
//! ---
//! "{package}": minor
//! ---
//!
//! ### New Features
//!
//! {summary}
//! """
//!
//! patch = """
//! ---
//! "{package}": patch
//! ---
//!
//! ### Bug Fixes
//!
//! {summary}
//! """
//!
//! # ============================================================================
//! # PLUGINS CONFIGURATION
//! # ============================================================================
//! [plugins]
//! # List of enabled plugins
//! enabled = ["analyzer", "generator", "validator", "configurator"]
//!
//! # Plugin-specific configurations
//! [plugins.analyzer]
//! enabled = true
//! ignore_patterns = ["**/test/**", "**/dist/**"]
//! max_depth = 5
//!
//! [plugins.generator]
//! enabled = true
//! templates_dir = ".monorepo/templates"
//! default_license = "MIT"
//!
//! [plugins.validator]
//! enabled = true
//! strict_mode = true
//! rules = ["no-circular-deps", "consistent-naming", "required-files"]
//!
//! [plugins.configurator]
//! enabled = true
//! auto_configure = true
//! config_presets = ["typescript", "react", "node"]
//!
//! # Custom plugin configurations
//! [plugins.custom]
//! my-custom-plugin = { path = "./plugins/my-custom-plugin", config = { key = "value" } }
//!
//! # ============================================================================
//! # GIT CONFIGURATION
//! # ============================================================================
//! [git]
//! auto_commit = false
//! auto_push = false
//! sign_commits = false
//! gpg_key = ""  # Optional: GPG key for signing
//!
//! # Branch configurations per environment
//! [git.branches]
//! main = { protected = true, allow_direct_push = false }
//! develop = { protected = true, allow_direct_push = false }
//! staging = { protected = true, allow_direct_push = false }
//!
//! # Repository host configurations
//! [git.repository_host]
//! type = "github"  # "github", "gitlab", "bitbucket", "custom"
//! api_endpoint = "https://api.github.com"
//! ssh_url = "git@github.com:org/repo.git"
//! https_url = "https://github.com/org/repo.git"
//!
//! # ============================================================================
//! # VALIDATION CONFIGURATION
//! # ============================================================================
//! [validation]
//! # Dependency validation rules
//! [validation.dependencies]
//! allow_wildcards = false
//! allow_workspace_protocol = true
//! allow_file_protocol = false
//! check_peer_dependencies = true
//! enforce_pnpm_workspace_protocol = true
//!
//! # Version validation rules
//! [validation.versions]
//! enforce_semver = true
//! allow_prerelease = true
//! version_pattern = "^\\d+\\.\\d+\\.\\d+(-[a-zA-Z0-9\\.]+)?(\\+[a-zA-Z0-9\\.]+)?$"
//!
//! # File validation rules
//! [validation.files]
//! required_files = ["README.md", "LICENSE", "package.json"]
//! forbidden_files = [".env", "*.log", "node_modules"]
//!
//! # Code quality gates
//! [validation.quality_gates]
//! min_coverage = 80.0
//! max_complexity = 10
//! max_duplicates = 5.0
//!
//! # Security validation
//! [validation.security]
//! check_vulnerabilities = true
//! audit_level = "moderate"  # "low", "moderate", "high", "critical"
//! allow_dev_dependencies_vulnerabilities = true
//! ```

pub mod components;
mod defaults;
mod manager;
#[cfg(test)]
mod tests;
pub mod types;

// Re-export main configuration struct
/// The main configuration structure containing all monorepo settings
pub use types::MonorepoConfig;

// Re-export configuration manager
/// The primary interface for loading, saving, and managing configurations
pub use types::ConfigManager;

// Re-export pattern matcher type
/// Type alias for pattern matching closures used in workspace operations
pub use types::PatternMatcher;

// Re-export environment enum
/// Deployment environment specification (Development, Staging, Production, etc.)
pub use types::Environment;

// Additional re-exports organized by category

// Workspace configuration types
/// Package discovery settings for auto-detection
pub use types::PackageDiscoveryConfig;
/// Configuration for workspace discovery and management
pub use types::WorkspaceConfig;
/// Individual workspace pattern definition with filtering options
pub use types::WorkspacePattern;
/// Options for workspace pattern behavior
pub use types::WorkspacePatternOptions;
/// Validation rules for workspace configuration
pub use types::WorkspaceValidationConfig;

// Package manager specific configurations
/// Bun-specific workspace configuration
pub use types::BunWorkspaceConfig;
/// npm-specific workspace configuration
pub use types::NpmWorkspaceConfig;
/// Container for all package manager specific configurations
pub use types::PackageManagerConfigs;
/// Supported package manager types (npm, yarn, pnpm, bun, custom)
pub use types::PackageManagerType;
/// pnpm-specific workspace configuration
pub use types::PnpmWorkspaceConfig;
/// Yarn version specification (classic vs berry)
pub use types::YarnVersion;
/// Yarn-specific workspace configuration
pub use types::YarnWorkspaceConfig;

// Versioning configuration
/// Type of version bump (major, minor, patch, snapshot)
pub use types::VersionBumpType;
/// Version management and propagation settings
pub use types::VersioningConfig;

// Task management configuration
/// Task execution and performance settings
pub use types::TasksConfig;

// Changelog configuration
/// Changelog generation settings and templates
pub use types::ChangelogConfig;
/// Changelog output format options
pub use types::ChangelogFormat;
/// Customizable changelog templates
pub use types::ChangelogTemplate;
/// How commits are grouped in changelogs
pub use types::CommitGrouping;

// Git and hooks configuration
/// Branch configuration for different environments
pub use types::BranchConfig;
/// Branch type specification (main, develop, feature, etc.)
pub use types::BranchType;
/// Git-related settings including branch management
pub use types::GitConfig;
/// Individual hook configuration
pub use types::HookConfig;
/// Git hooks automation settings
pub use types::HooksConfig;

// Changesets configuration
/// Changeset management settings
pub use types::ChangesetsConfig;

// Plugins configuration
/// Plugin system configuration
pub use types::PluginsConfig;

// Re-export components
/// Component-based configuration system interface
pub use components::ConfigComponents;
/// Handles loading and saving configuration files in various formats
pub use components::ConfigPersistence;
/// Read-only access to configuration data
pub use components::ConfigReader;
/// Write operations for configuration updates
pub use components::ConfigWriter;
/// Efficient multi-pattern matching for workspace operations
pub use components::MultiPatternMatcher;
/// Manages workspace patterns with filtering and validation
pub use components::WorkspacePatternManager;
