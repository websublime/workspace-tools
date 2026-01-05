//! Config command type definitions for Node.js bindings.
//!
//! # What
//!
//! This module defines all NAPI-compatible type structures for the config commands
//! (`configShow` and `configValidate`), including input parameters and response data
//! types. These types enable JavaScript and TypeScript consumers to interact with
//! workspace configuration in a type-safe manner.
//!
//! # How
//!
//! Types are defined with the `#[napi(object)]` attribute to be automatically
//! exposed as JavaScript objects. The module provides:
//!
//! - **Input Parameters**:
//!   - `ConfigShowParams`: Parameters for the `configShow` command
//!   - `ConfigValidateParams`: Parameters for the `configValidate` command
//!
//! - **Response Data**:
//!   - `ConfigShowData`: Response containing the loaded configuration and its path
//!   - `ConfigValidateData`: Response containing validation results
//!
//! - **Configuration Structures** (exposed as nested objects in responses):
//!   - `ConfigData`: Main configuration container with all sections
//!   - `ChangesetConfigInfo`: Changeset management configuration
//!   - `VersionConfigInfo`: Version resolution configuration
//!   - `DependencyConfigInfo`: Dependency propagation configuration
//!   - `UpgradeConfigInfo`: Upgrade detection and application configuration
//!   - `RegistryConfigInfo`: NPM registry configuration
//!   - `BackupConfigInfo`: Backup and rollback configuration
//!   - `ChangelogConfigInfo`: Changelog generation configuration
//!   - `AuditConfigInfo`: Audit and health check configuration
//!   - `GitConfigInfo`: Git integration configuration
//!   - `ExecuteConfigInfo`: Command execution configuration
//!
//! - **Validation Types**:
//!   - `ConfigValidationIssue`: Individual validation issue with severity
//!
//! All types implement `Clone`, `Debug`, and `Serialize` for flexibility in
//! testing and serialization scenarios.
//!
//! # Why
//!
//! The config commands allow users to inspect and validate the workspace
//! configuration (`repo.config.json`, `repo.config.toml`, or `repo.config.yaml`)
//! programmatically. These types provide:
//!
//! - **Type safety**: Strong typing for JavaScript/TypeScript consumers
//! - **Documentation**: Self-documenting API through TypeScript definitions
//! - **Consistency**: Matches the CLI configuration structure for compatibility
//! - **Validation**: Enables parameter validation before CLI execution
//!
//! # Examples
//!
//! ## TypeScript Usage
//!
//! ```typescript
//! import { configShow, configValidate } from '@websublime/workspace-tools';
//! import type { ConfigShowParams, ConfigValidateParams } from '@websublime/workspace-tools';
//!
//! // Show configuration
//! const showParams: ConfigShowParams = { root: '.' };
//! const showResult = await configShow(showParams);
//!
//! if (showResult.success) {
//!   console.log(`Config loaded from: ${showResult.data.configPath}`);
//!   console.log(`Versioning strategy: ${showResult.data.config.version.strategy}`);
//!   console.log(`Changeset path: ${showResult.data.config.changeset.path}`);
//!
//!   // Access nested configuration sections
//!   const { changeset, version, dependency, execute } = showResult.data.config;
//!   console.log(`Default bump type: ${version.defaultBump}`);
//!   console.log(`Propagate dependencies: ${dependency.propagateDependencies}`);
//!   console.log(`Execute timeout: ${execute.timeoutSecs}s`);
//! }
//!
//! // Validate configuration
//! const validateParams: ConfigValidateParams = { root: '.' };
//! const validateResult = await configValidate(validateParams);
//!
//! if (validateResult.success) {
//!   console.log(`Configuration valid: ${validateResult.data.valid}`);
//!
//!   if (validateResult.data.errors.length > 0) {
//!     console.error('Errors:');
//!     for (const error of validateResult.data.errors) {
//!       console.error(`  [${error.severity}] ${error.message} (field: ${error.field})`);
//!     }
//!   }
//!
//!   if (validateResult.data.warnings.length > 0) {
//!     console.warn('Warnings:');
//!     for (const warning of validateResult.data.warnings) {
//!       console.warn(`  [${warning.severity}] ${warning.message}`);
//!     }
//!   }
//! }
//! ```
//!
//! ## Rust Usage (Internal)
//!
//! ```rust,ignore
//! use sublime_node_tools::types::config::{
//!     ConfigShowParams, ConfigShowData, ConfigData,
//!     ChangesetConfigInfo, VersionConfigInfo
//! };
//!
//! // Creating params for validation
//! let params = ConfigShowParams::new(".".to_string());
//!
//! // Constructing response data
//! let config_data = ConfigData::default();
//! let show_data = ConfigShowData::new(
//!     "repo.config.json".to_string(),
//!     config_data,
//! );
//! ```

use napi_derive::napi;
use serde::Serialize;

use crate::error::ErrorInfo;

// ============================================================================
// Constants
// ============================================================================

/// Valid versioning strategy values.
///
/// These are the allowed values for the `strategy` field in `VersionConfigInfo`.
// Allow dead_code - used for semantic validation documentation, not directly in code
#[allow(dead_code)]
pub const VALID_STRATEGIES: [&str; 2] = ["independent", "unified"];

/// Valid bump type values.
///
/// These are the allowed values for default bump type fields.
// Allow dead_code - used for semantic validation documentation, not directly in code
#[allow(dead_code)]
pub const VALID_BUMP_TYPES: [&str; 4] = ["major", "minor", "patch", "none"];

/// Valid changelog format values.
///
/// These are the allowed values for the `format` field in `ChangelogConfigInfo`.
// Allow dead_code - used for semantic validation documentation, not directly in code
#[allow(dead_code)]
pub const VALID_CHANGELOG_FORMATS: [&str; 3] =
    ["keep-a-changelog", "conventional-commits", "custom"];

/// Valid monorepo mode values.
///
/// These are the allowed values for the `monorepoMode` field in `ChangelogConfigInfo`.
// Allow dead_code - used for semantic validation documentation, not directly in code
#[allow(dead_code)]
pub const VALID_MONOREPO_MODES: [&str; 3] = ["per-package", "root", "both"];

/// Valid severity levels for validation issues.
///
/// These are the allowed values for the `severity` field in `ConfigValidationIssue`.
// Allow dead_code - used for semantic validation documentation, not directly in code
#[allow(dead_code)]
pub const VALID_SEVERITY_LEVELS: [&str; 3] = ["error", "warning", "info"];

// ============================================================================
// Input Parameters
// ============================================================================

/// Input parameters for the `configShow` command.
///
/// This structure defines the parameters that can be passed to the `configShow`
/// function from JavaScript/TypeScript. The root path is required, while
/// the config path is optional.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ConfigShowParams {
///   // Workspace root directory path
///   root: string;
///   // Optional custom config file path
///   configPath?: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Minimal params with just root
/// const params: ConfigShowParams = { root: '.' };
///
/// // With custom config path
/// const paramsWithConfig: ConfigShowParams = {
///   root: '/path/to/workspace',
///   configPath: '/path/to/custom/repo.config.json'
/// };
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ConfigShowParams {
    /// Workspace root directory path.
    ///
    /// This is the absolute or relative path to the root of the workspace.
    /// The configuration file will be searched for in this directory unless
    /// a custom `configPath` is provided.
    pub root: String,

    /// Optional custom configuration file path.
    ///
    /// If not provided, the command will search for configuration files
    /// in standard locations (`repo.config.json`, `repo.config.toml`,
    /// `repo.config.yaml`) within the workspace root.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
}

/// Input parameters for the `configValidate` command.
///
/// This structure defines the parameters that can be passed to the `configValidate`
/// function from JavaScript/TypeScript. The root path is required, while
/// the config path is optional.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ConfigValidateParams {
///   // Workspace root directory path
///   root: string;
///   // Optional custom config file path
///   configPath?: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Minimal params with just root
/// const params: ConfigValidateParams = { root: '.' };
///
/// // With custom config path
/// const paramsWithConfig: ConfigValidateParams = {
///   root: '/path/to/workspace',
///   configPath: '/path/to/custom/repo.config.json'
/// };
/// ```
// ConfigValidateParams is used by story 7.3 (configValidate command)
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ConfigValidateParams {
    /// Workspace root directory path.
    ///
    /// This is the absolute or relative path to the root of the workspace.
    /// The configuration file will be searched for in this directory unless
    /// a custom `configPath` is provided.
    pub root: String,

    /// Optional custom configuration file path.
    ///
    /// If not provided, the command will search for configuration files
    /// in standard locations (`repo.config.json`, `repo.config.toml`,
    /// `repo.config.yaml`) within the workspace root.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
}

// ============================================================================
// Configuration Structures
// ============================================================================

/// Changeset configuration information.
///
/// Contains settings for changeset management, including paths and
/// environment configuration.
///
/// # Fields
///
/// - `path`: Path to store active changesets
/// - `history_path`: Path to store archived changesets
/// - `available_environments`: List of valid environment names
/// - `default_environments`: Default environments for new changesets
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangesetConfigInfo {
///   // Path to store active changesets (default: ".changesets")
///   path: string;
///   // Path to store archived changesets
///   historyPath: string;
///   // List of valid environment names
///   availableEnvironments: string[];
///   // Default environments for new changesets
///   defaultEnvironments: string[];
/// }
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangesetConfigInfo {
    /// Path to store active changesets.
    ///
    /// This is the directory where pending changeset files are stored.
    /// Default value is `.changesets`.
    pub path: String,

    /// Path to store archived changesets.
    ///
    /// This is the directory where consumed changeset files are moved
    /// after a version bump operation. Typically a subdirectory of `path`.
    pub history_path: String,

    /// List of valid environment names.
    ///
    /// These are the environments that changesets can target. Common
    /// examples include "production", "staging", "development".
    pub available_environments: Vec<String>,

    /// Default environments for new changesets.
    ///
    /// These environments are automatically assigned to new changesets
    /// if not explicitly specified.
    pub default_environments: Vec<String>,
}

/// Version configuration information.
///
/// Contains settings for version resolution and management.
///
/// # Fields
///
/// - `strategy`: Versioning strategy ("independent" or "unified")
/// - `default_bump`: Default version bump type
/// - `snapshot_format`: Format template for snapshot versions
///
/// # TypeScript Definition
///
/// ```typescript
/// interface VersionConfigInfo {
///   // Versioning strategy: "independent" or "unified"
///   strategy: string;
///   // Default version bump type: "major", "minor", "patch", or "none"
///   defaultBump: string;
///   // Format template for snapshot versions
///   snapshotFormat: string;
/// }
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct VersionConfigInfo {
    /// Versioning strategy.
    ///
    /// Determines how package versions are managed:
    /// - `"independent"`: Each package has its own version
    /// - `"unified"`: All packages share the same version
    pub strategy: String,

    /// Default version bump type.
    ///
    /// Used when no explicit bump type is specified:
    /// - `"major"`: Breaking changes
    /// - `"minor"`: New features
    /// - `"patch"`: Bug fixes
    /// - `"none"`: No version change
    pub default_bump: String,

    /// Format template for snapshot versions.
    ///
    /// Template string for generating snapshot version identifiers.
    /// Supports placeholders like `{version}`, `{branch}`, `{commit}`,
    /// `{shortCommit}`, and `{timestamp}`.
    pub snapshot_format: String,
}

/// Dependency configuration information.
///
/// Contains settings for dependency propagation during version bumps.
///
/// # Fields
///
/// - `propagation_bump`: Version bump type for dependency updates
/// - `propagate_dependencies`: Whether to propagate regular dependencies
/// - `propagate_dev_dependencies`: Whether to propagate dev dependencies
/// - `propagate_peer_dependencies`: Whether to propagate peer dependencies
/// - `max_depth`: Maximum propagation depth
/// - `fail_on_circular`: Whether to fail on circular dependencies
/// - `skip_workspace_protocol`: Skip workspace: protocol dependencies
/// - `skip_file_protocol`: Skip file: protocol dependencies
/// - `skip_link_protocol`: Skip link: protocol dependencies
/// - `skip_portal_protocol`: Skip portal: protocol dependencies
///
/// # TypeScript Definition
///
/// ```typescript
/// interface DependencyConfigInfo {
///   // Version bump type for dependency updates
///   propagationBump: string;
///   // Whether to propagate regular dependencies
///   propagateDependencies: boolean;
///   // Whether to propagate dev dependencies
///   propagateDevDependencies: boolean;
///   // Whether to propagate peer dependencies
///   propagatePeerDependencies: boolean;
///   // Maximum propagation depth
///   maxDepth: number;
///   // Whether to fail on circular dependencies
///   failOnCircular: boolean;
///   // Skip workspace: protocol dependencies
///   skipWorkspaceProtocol: boolean;
///   // Skip file: protocol dependencies
///   skipFileProtocol: boolean;
///   // Skip link: protocol dependencies
///   skipLinkProtocol: boolean;
///   // Skip portal: protocol dependencies
///   skipPortalProtocol: boolean;
/// }
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
// Allow struct_excessive_bools - this matches the pkg crate's DependencyConfig structure
#[allow(dead_code, clippy::struct_excessive_bools)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct DependencyConfigInfo {
    /// Version bump type for dependency updates.
    ///
    /// When a package is updated, this determines how dependent packages
    /// have their versions bumped. Values: "major", "minor", "patch", "none".
    pub propagation_bump: String,

    /// Whether to propagate regular dependencies.
    ///
    /// If `true`, packages that depend on updated packages will also
    /// be considered for version updates.
    pub propagate_dependencies: bool,

    /// Whether to propagate dev dependencies.
    ///
    /// If `true`, packages that have the updated package as a dev
    /// dependency will also be considered for version updates.
    pub propagate_dev_dependencies: bool,

    /// Whether to propagate peer dependencies.
    ///
    /// If `true`, packages that have the updated package as a peer
    /// dependency will also be considered for version updates.
    pub propagate_peer_dependencies: bool,

    /// Maximum propagation depth.
    ///
    /// Limits how deep dependency propagation can traverse the
    /// dependency graph. Prevents excessive updates in large monorepos.
    pub max_depth: u32,

    /// Whether to fail on circular dependencies.
    ///
    /// If `true`, the operation fails when circular dependencies are
    /// detected. If `false`, circular dependencies are handled gracefully.
    pub fail_on_circular: bool,

    /// Skip workspace: protocol dependencies.
    ///
    /// If `true`, dependencies using `workspace:` protocol are not
    /// propagated. These are typically handled differently in monorepos.
    pub skip_workspace_protocol: bool,

    /// Skip file: protocol dependencies.
    ///
    /// If `true`, dependencies using `file:` protocol are not propagated.
    pub skip_file_protocol: bool,

    /// Skip link: protocol dependencies.
    ///
    /// If `true`, dependencies using `link:` protocol are not propagated.
    pub skip_link_protocol: bool,

    /// Skip portal: protocol dependencies.
    ///
    /// If `true`, dependencies using `portal:` protocol are not propagated.
    pub skip_portal_protocol: bool,
}

/// Registry configuration information.
///
/// Contains settings for NPM registry access.
///
/// # Fields
///
/// - `default_registry`: Default npm registry URL
/// - `scoped_registries`: Map of scopes to registry URLs
/// - `timeout_secs`: Request timeout in seconds
/// - `retry_attempts`: Number of retry attempts for failed requests
/// - `read_npmrc`: Whether to read .npmrc for registry configuration
///
/// # TypeScript Definition
///
/// ```typescript
/// interface RegistryConfigInfo {
///   // Default npm registry URL
///   defaultRegistry: string;
///   // Map of scopes to registry URLs (e.g., {"@myorg": "https://npm.myorg.com"})
///   scopedRegistries: Record<string, string>;
///   // Request timeout in seconds
///   timeoutSecs: number;
///   // Number of retry attempts for failed requests
///   retryAttempts: number;
///   // Whether to read .npmrc for registry configuration
///   readNpmrc: boolean;
/// }
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct RegistryConfigInfo {
    /// Default npm registry URL.
    ///
    /// The registry URL to use for packages without a specific scope
    /// configuration. Default is "https://registry.npmjs.org".
    pub default_registry: String,

    /// Map of scopes to registry URLs.
    ///
    /// Allows configuring different registries for different npm scopes.
    /// Keys are scope names (e.g., "@myorg"), values are registry URLs.
    #[napi(ts_type = "Record<string, string>")]
    pub scoped_registries: Vec<ScopedRegistryEntry>,

    /// Request timeout in seconds.
    ///
    /// How long to wait for registry requests before timing out.
    pub timeout_secs: u32,

    /// Number of retry attempts for failed requests.
    ///
    /// How many times to retry a failed registry request before giving up.
    pub retry_attempts: u32,

    /// Whether to read .npmrc for registry configuration.
    ///
    /// If `true`, the tool will read `.npmrc` files for additional
    /// registry configuration and authentication tokens.
    pub read_npmrc: bool,
}

/// Scoped registry entry.
///
/// Represents a mapping from an npm scope to a registry URL.
///
/// # Fields
///
/// - `scope`: The npm scope (e.g., "@myorg")
/// - `registry`: The registry URL for this scope
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ScopedRegistryEntry {
///   // The npm scope (e.g., "@myorg")
///   scope: string;
///   // The registry URL for this scope
///   registry: string;
/// }
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ScopedRegistryEntry {
    /// The npm scope.
    ///
    /// The scope name including the `@` prefix (e.g., "@myorg").
    pub scope: String,

    /// The registry URL for this scope.
    ///
    /// The full URL of the npm registry to use for this scope.
    pub registry: String,
}

/// Backup configuration information.
///
/// Contains settings for backup and rollback functionality.
///
/// # Fields
///
/// - `enabled`: Whether backup is enabled
/// - `path`: Path to store backups
/// - `keep_count`: Number of backups to keep
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BackupConfigInfo {
///   // Whether backup is enabled
///   enabled: boolean;
///   // Path to store backups
///   path: string;
///   // Number of backups to keep
///   keepCount: number;
/// }
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BackupConfigInfo {
    /// Whether backup is enabled.
    ///
    /// If `true`, backups are created before operations that modify
    /// package files, allowing rollback if needed.
    pub enabled: bool,

    /// Path to store backups.
    ///
    /// The directory where backup files are stored. This should be
    /// outside the workspace to avoid being affected by operations.
    pub path: String,

    /// Number of backups to keep.
    ///
    /// Older backups beyond this count are automatically deleted.
    pub keep_count: u32,
}

/// Upgrade configuration information.
///
/// Contains settings for upgrade detection and application.
///
/// # Fields
///
/// - `auto_changeset`: Automatically create changesets for upgrades
/// - `changeset_bump`: Version bump type for upgrade changesets
/// - `registry`: Registry configuration
/// - `backup`: Backup configuration
///
/// # TypeScript Definition
///
/// ```typescript
/// interface UpgradeConfigInfo {
///   // Automatically create changesets for upgrades
///   autoChangeset: boolean;
///   // Version bump type for upgrade changesets
///   changesetBump: string;
///   // Registry configuration
///   registry: RegistryConfigInfo;
///   // Backup configuration
///   backup: BackupConfigInfo;
/// }
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct UpgradeConfigInfo {
    /// Automatically create changesets for upgrades.
    ///
    /// If `true`, a changeset is automatically created when
    /// dependency upgrades are applied.
    pub auto_changeset: bool,

    /// Version bump type for upgrade changesets.
    ///
    /// The bump type to use when creating changesets for upgrades.
    /// Values: "major", "minor", "patch", "none".
    pub changeset_bump: String,

    /// Registry configuration.
    ///
    /// Settings for accessing npm registries to check for updates.
    pub registry: RegistryConfigInfo,

    /// Backup configuration.
    ///
    /// Settings for backup and rollback functionality.
    pub backup: BackupConfigInfo,
}

/// Changelog configuration information.
///
/// Contains settings for changelog generation.
///
/// # Fields
///
/// - `enabled`: Whether changelog generation is enabled
/// - `format`: Changelog format ("keep-a-changelog", "conventional-commits", "custom")
/// - `include_commit_links`: Whether to include commit links
/// - `repository_url`: Repository URL for generating links
/// - `conventional`: Whether to use conventional commits parsing
/// - `template`: Custom template path
/// - `exclude`: Patterns to exclude from changelog
/// - `monorepo_mode`: How to handle changelogs in monorepos
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ChangelogConfigInfo {
///   // Whether changelog generation is enabled
///   enabled: boolean;
///   // Changelog format: "keep-a-changelog", "conventional-commits", or "custom"
///   format: string;
///   // Whether to include commit links
///   includeCommitLinks: boolean;
///   // Repository URL for generating links
///   repositoryUrl?: string;
///   // Whether to use conventional commits parsing
///   conventional: boolean;
///   // Custom template path
///   template?: string;
///   // Patterns to exclude from changelog
///   exclude: string[];
///   // How to handle changelogs in monorepos: "per-package", "root", or "both"
///   monorepoMode: string;
/// }
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ChangelogConfigInfo {
    /// Whether changelog generation is enabled.
    ///
    /// If `false`, no changelog files are generated or updated.
    pub enabled: bool,

    /// Changelog format.
    ///
    /// The format to use for changelog entries:
    /// - `"keep-a-changelog"`: Keep a Changelog format
    /// - `"conventional-commits"`: Conventional Commits format
    /// - `"custom"`: Custom template-based format
    pub format: String,

    /// Whether to include commit links.
    ///
    /// If `true`, changelog entries include links to the relevant commits.
    pub include_commit_links: bool,

    /// Repository URL for generating links.
    ///
    /// Used to generate links to commits, comparisons, and issues
    /// in the changelog. Example: "https://github.com/org/repo".
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_url: Option<String>,

    /// Whether to use conventional commits parsing.
    ///
    /// If `true`, commit messages are parsed using conventional commits
    /// specification to categorize changes.
    pub conventional: bool,

    /// Custom template path.
    ///
    /// Path to a custom template file for changelog generation.
    /// Only used when `format` is `"custom"`.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,

    /// Patterns to exclude from changelog.
    ///
    /// Commit messages or files matching these patterns are excluded
    /// from changelog generation.
    pub exclude: Vec<String>,

    /// How to handle changelogs in monorepos.
    ///
    /// Determines where changelog files are created:
    /// - `"per-package"`: Each package has its own CHANGELOG.md
    /// - `"root"`: Single CHANGELOG.md at the repository root
    /// - `"both"`: Both per-package and root changelogs
    pub monorepo_mode: String,
}

/// Audit configuration information.
///
/// Contains settings for audit and health check functionality.
///
/// # Fields
///
/// - `enabled`: Whether audit is enabled
/// - `min_severity`: Minimum severity level to report
/// - `sections`: Which audit sections to run
/// - `health_score_weights`: Weights for health score calculation
///
/// # TypeScript Definition
///
/// ```typescript
/// interface AuditConfigInfo {
///   // Whether audit is enabled
///   enabled: boolean;
///   // Minimum severity level to report: "critical", "high", "medium", "low", "info"
///   minSeverity: string;
///   // Which audit sections to run
///   sections: AuditSectionsConfigInfo;
///   // Weights for health score calculation
///   healthScoreWeights: HealthScoreWeightsInfo;
/// }
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct AuditConfigInfo {
    /// Whether audit is enabled.
    ///
    /// If `false`, audit commands are skipped.
    pub enabled: bool,

    /// Minimum severity level to report.
    ///
    /// Only issues at or above this severity are reported:
    /// - `"critical"`: Only critical issues
    /// - `"high"`: High and above
    /// - `"medium"`: Medium and above
    /// - `"low"`: Low and above
    /// - `"info"`: All issues including informational
    pub min_severity: String,

    /// Which audit sections to run.
    ///
    /// Allows selectively enabling or disabling specific audit checks.
    pub sections: AuditSectionsConfigInfo,

    /// Weights for health score calculation.
    ///
    /// Determines how different factors contribute to the overall
    /// health score.
    pub health_score_weights: HealthScoreWeightsInfo,
}

/// Audit sections configuration.
///
/// Contains flags for enabling/disabling specific audit sections.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface AuditSectionsConfigInfo {
///   // Check for available upgrades
///   upgrades: boolean;
///   // Analyze dependencies
///   dependencies: boolean;
///   // Check version consistency
///   versionConsistency: boolean;
///   // Detect breaking changes
///   breakingChanges: boolean;
/// }
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
// Allow struct_excessive_bools - this matches the pkg crate's AuditConfig structure
#[allow(dead_code, clippy::struct_excessive_bools)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct AuditSectionsConfigInfo {
    /// Check for available upgrades.
    ///
    /// Analyzes dependencies for available updates.
    pub upgrades: bool,

    /// Analyze dependencies.
    ///
    /// Checks for circular dependencies, missing dependencies, etc.
    pub dependencies: bool,

    /// Check version consistency.
    ///
    /// Verifies that dependency versions are consistent across packages.
    pub version_consistency: bool,

    /// Detect breaking changes.
    ///
    /// Identifies potential breaking changes based on commits and changelogs.
    pub breaking_changes: bool,
}

/// Health score weights configuration.
///
/// Contains weights for calculating the overall health score.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface HealthScoreWeightsInfo {
///   // Weight for upgrade score (0.0-1.0)
///   upgradesWeight: number;
///   // Weight for dependencies score (0.0-1.0)
///   dependenciesWeight: number;
///   // Weight for version consistency score (0.0-1.0)
///   versionConsistencyWeight: number;
///   // Weight for breaking changes score (0.0-1.0)
///   breakingChangesWeight: number;
/// }
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
// Allow struct_field_names - the _weight suffix is intentional for clarity in JavaScript
#[allow(dead_code, clippy::struct_field_names)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct HealthScoreWeightsInfo {
    /// Weight for upgrade score.
    ///
    /// How much the upgrade status contributes to the health score.
    /// Value between 0.0 and 1.0.
    pub upgrades_weight: f64,

    /// Weight for dependencies score.
    ///
    /// How much the dependency health contributes to the health score.
    /// Value between 0.0 and 1.0.
    pub dependencies_weight: f64,

    /// Weight for version consistency score.
    ///
    /// How much version consistency contributes to the health score.
    /// Value between 0.0 and 1.0.
    pub version_consistency_weight: f64,

    /// Weight for breaking changes score.
    ///
    /// How much breaking changes impact the health score.
    /// Value between 0.0 and 1.0.
    pub breaking_changes_weight: f64,
}

/// Git configuration information.
///
/// Contains settings for Git integration.
///
/// # Fields
///
/// - `branch_base`: Base branch for comparisons
/// - `detect_affected_packages`: Whether to auto-detect affected packages
///
/// # TypeScript Definition
///
/// ```typescript
/// interface GitConfigInfo {
///   // Base branch for comparisons (e.g., "main", "master")
///   branchBase: string;
///   // Whether to auto-detect affected packages from Git changes
///   detectAffectedPackages: boolean;
/// }
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct GitConfigInfo {
    /// Base branch for comparisons.
    ///
    /// The branch used as the base for determining changes.
    /// Common values: "main", "master", "develop".
    pub branch_base: String,

    /// Whether to auto-detect affected packages.
    ///
    /// If `true`, packages affected by Git changes are automatically
    /// detected based on file changes.
    pub detect_affected_packages: bool,
}

/// Execute configuration information.
///
/// Contains settings for command execution with timeout and parallelism.
///
/// # Fields
///
/// - `timeout_secs`: Overall timeout in seconds
/// - `per_package_timeout_secs`: Per-package timeout in seconds
/// - `max_parallel`: Maximum number of parallel executions
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ExecuteConfigInfo {
///   // Overall timeout in seconds (0 = no timeout)
///   timeoutSecs: number;
///   // Per-package timeout in seconds (0 = no timeout)
///   perPackageTimeoutSecs: number;
///   // Maximum number of parallel executions
///   maxParallel: number;
/// }
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ExecuteConfigInfo {
    /// Overall timeout in seconds.
    ///
    /// Maximum time allowed for the entire execute command.
    /// A value of 0 means no timeout.
    pub timeout_secs: u32,

    /// Per-package timeout in seconds.
    ///
    /// Maximum time allowed for executing the command on each package.
    /// A value of 0 means no timeout.
    pub per_package_timeout_secs: u32,

    /// Maximum number of parallel executions.
    ///
    /// How many packages can have commands running simultaneously.
    /// Higher values can speed up execution but increase resource usage.
    pub max_parallel: u32,
}

/// Main configuration data structure.
///
/// Contains all configuration sections from the `repo.config` file.
/// This is the root structure that holds all workspace tool settings.
///
/// # Fields
///
/// - `changeset`: Changeset management configuration
/// - `version`: Version resolution configuration
/// - `dependency`: Dependency propagation configuration
/// - `upgrade`: Upgrade detection and application configuration
/// - `changelog`: Changelog generation configuration
/// - `audit`: Audit and health check configuration
/// - `git`: Git integration configuration
/// - `execute`: Command execution configuration
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ConfigData {
///   // Changeset management configuration
///   changeset: ChangesetConfigInfo;
///   // Version resolution configuration
///   version: VersionConfigInfo;
///   // Dependency propagation configuration
///   dependency: DependencyConfigInfo;
///   // Upgrade detection and application configuration
///   upgrade: UpgradeConfigInfo;
///   // Changelog generation configuration
///   changelog: ChangelogConfigInfo;
///   // Audit and health check configuration
///   audit: AuditConfigInfo;
///   // Git integration configuration
///   git: GitConfigInfo;
///   // Command execution configuration
///   execute: ExecuteConfigInfo;
/// }
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize, Default)]
pub struct ConfigData {
    /// Changeset management configuration.
    ///
    /// Settings for managing changesets including paths and environments.
    pub changeset: ChangesetConfigInfo,

    /// Version resolution configuration.
    ///
    /// Settings for version management including strategy and defaults.
    pub version: VersionConfigInfo,

    /// Dependency propagation configuration.
    ///
    /// Settings for how dependency updates propagate through the workspace.
    pub dependency: DependencyConfigInfo,

    /// Upgrade detection and application configuration.
    ///
    /// Settings for checking and applying dependency upgrades.
    pub upgrade: UpgradeConfigInfo,

    /// Changelog generation configuration.
    ///
    /// Settings for generating and formatting changelog files.
    pub changelog: ChangelogConfigInfo,

    /// Audit and health check configuration.
    ///
    /// Settings for workspace health auditing.
    pub audit: AuditConfigInfo,

    /// Git integration configuration.
    ///
    /// Settings for Git-related operations.
    pub git: GitConfigInfo,

    /// Command execution configuration.
    ///
    /// Settings for running commands across packages.
    pub execute: ExecuteConfigInfo,
}

// ============================================================================
// Response Data Types
// ============================================================================

/// Response data for the `configShow` command.
///
/// Contains the loaded configuration and the path where it was found.
///
/// # Fields
///
/// - `config_path`: Path to the loaded configuration file
/// - `config_format`: Format of the configuration file (json, toml, yaml)
/// - `config`: The loaded configuration data
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ConfigShowData {
///   // Path to the loaded configuration file
///   configPath: string;
///   // Format of the configuration file
///   configFormat: string;
///   // The loaded configuration data
///   config: ConfigData;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const result = await configShow({ root: '.' });
/// if (result.success) {
///   console.log(`Loaded from: ${result.data.configPath}`);
///   console.log(`Format: ${result.data.configFormat}`);
///   console.log(`Strategy: ${result.data.config.version.strategy}`);
/// }
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ConfigShowData {
    /// Path to the loaded configuration file.
    ///
    /// The absolute or relative path where the configuration was found.
    /// Examples: "repo.config.json", "/path/to/repo.config.toml".
    pub config_path: String,

    /// Format of the configuration file.
    ///
    /// The detected format based on file extension:
    /// - `"json"`: JSON format
    /// - `"toml"`: TOML format
    /// - `"yaml"`: YAML format
    pub config_format: String,

    /// The loaded configuration data.
    ///
    /// Contains all configuration sections parsed from the file.
    pub config: ConfigData,
}

/// Validation issue information.
///
/// Represents a single validation issue found during configuration validation.
///
/// # Fields
///
/// - `severity`: Issue severity ("error", "warning", or "info")
/// - `field`: The configuration field with the issue
/// - `message`: Human-readable description of the issue
/// - `suggestion`: Optional suggestion for fixing the issue
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ConfigValidationIssue {
///   // Issue severity: "error", "warning", or "info"
///   severity: string;
///   // The configuration field with the issue
///   field: string;
///   // Human-readable description of the issue
///   message: string;
///   // Optional suggestion for fixing the issue
///   suggestion?: string;
/// }
/// ```
// ConfigValidationIssue is used by story 7.3 (configValidate command)
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ConfigValidationIssue {
    /// Issue severity.
    ///
    /// Indicates the importance of the issue:
    /// - `"error"`: Critical issue that must be fixed
    /// - `"warning"`: Potential problem that should be addressed
    /// - `"info"`: Informational note for improvement
    pub severity: String,

    /// The configuration field with the issue.
    ///
    /// Dot-notation path to the field, e.g., "version.strategy" or
    /// "changeset.path".
    pub field: String,

    /// Human-readable description of the issue.
    ///
    /// Explains what is wrong with the configuration.
    pub message: String,

    /// Optional suggestion for fixing the issue.
    ///
    /// Provides guidance on how to resolve the issue.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

/// Response data for the `configValidate` command.
///
/// Contains validation results including whether the configuration is valid
/// and any issues found.
///
/// # Fields
///
/// - `valid`: Whether the configuration is valid (no errors)
/// - `config_path`: Path to the validated configuration file
/// - `errors`: List of validation errors
/// - `warnings`: List of validation warnings
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ConfigValidateData {
///   // Whether the configuration is valid (no errors)
///   valid: boolean;
///   // Path to the validated configuration file
///   configPath: string;
///   // List of validation errors
///   errors: ConfigValidationIssue[];
///   // List of validation warnings
///   warnings: ConfigValidationIssue[];
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const result = await configValidate({ root: '.' });
/// if (result.success) {
///   if (result.data.valid) {
///     console.log('Configuration is valid!');
///   } else {
///     console.error(`Found ${result.data.errors.length} errors`);
///     for (const error of result.data.errors) {
///       console.error(`  - [${error.field}]: ${error.message}`);
///     }
///   }
/// }
/// ```
// ConfigValidateData is used by story 7.3 (configValidate command)
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ConfigValidateData {
    /// Whether the configuration is valid.
    ///
    /// `true` if no errors were found (warnings are allowed),
    /// `false` if there are any validation errors.
    pub valid: bool,

    /// Path to the validated configuration file.
    ///
    /// The path where the configuration file was found and validated.
    pub config_path: String,

    /// List of validation errors.
    ///
    /// Critical issues that must be fixed for the configuration to be valid.
    pub errors: Vec<ConfigValidationIssue>,

    /// List of validation warnings.
    ///
    /// Non-critical issues that should be addressed but don't prevent
    /// the configuration from being used.
    pub warnings: Vec<ConfigValidationIssue>,
}

// ============================================================================
// API Response Types
// ============================================================================

/// API response wrapper for the `configShow` command.
///
/// This structure wraps the `configShow` response with success/failure status
/// and consistent error handling, following the pattern used across all
/// NAPI commands.
///
/// # Fields
///
/// - `success`: Whether the operation succeeded
/// - `data`: The config show data (present when success is true)
/// - `error`: Error information (present when success is false)
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ConfigShowApiResponse {
///   // Whether the operation succeeded
///   success: boolean;
///   // The config show data (present when success is true)
///   data?: ConfigShowData;
///   // Error information (present when success is false)
///   error?: ErrorInfo;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const result = await configShow({ root: '.' });
///
/// if (result.success) {
///   // result.data is ConfigShowData
///   console.log(result.data.config.version.strategy);
/// } else {
///   // result.error is ErrorInfo
///   console.error(`[${result.error.code}] ${result.error.message}`);
/// }
/// ```
// Allow dead_code - will be used in story 7.2 (configShow command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ConfigShowApiResponse {
    /// Whether the operation succeeded.
    ///
    /// - `true`: Operation completed successfully, `data` field will be present
    /// - `false`: Operation failed, `error` field will be present
    pub success: bool,

    /// The config show data (only present when `success` is `true`).
    ///
    /// Contains the loaded configuration and its path.
    #[napi(ts_type = "ConfigShowData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ConfigShowData>,

    /// Error information (only present when `success` is `false`).
    ///
    /// Contains structured error information with a Node.js-style error code,
    /// message, optional context, and error kind.
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

/// API response wrapper for the `configValidate` command.
///
/// This structure wraps the `configValidate` response with success/failure status
/// and consistent error handling, following the pattern used across all
/// NAPI commands.
///
/// # Fields
///
/// - `success`: Whether the operation succeeded
/// - `data`: The config validate data (present when success is true)
/// - `error`: Error information (present when success is false)
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ConfigValidateApiResponse {
///   // Whether the operation succeeded
///   success: boolean;
///   // The config validate data (present when success is true)
///   data?: ConfigValidateData;
///   // Error information (present when success is false)
///   error?: ErrorInfo;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const result = await configValidate({ root: '.' });
///
/// if (result.success) {
///   // result.data is ConfigValidateData
///   console.log(`Valid: ${result.data.valid}`);
///   console.log(`Errors: ${result.data.errors.length}`);
/// } else {
///   // result.error is ErrorInfo
///   console.error(`[${result.error.code}] ${result.error.message}`);
/// }
/// ```
// ConfigValidateApiResponse is used by story 7.3 (configValidate command)
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ConfigValidateApiResponse {
    /// Whether the operation succeeded.
    ///
    /// - `true`: Operation completed successfully, `data` field will be present
    /// - `false`: Operation failed, `error` field will be present
    pub success: bool,

    /// The config validate data (only present when `success` is `true`).
    ///
    /// Contains validation results including whether the config is valid
    /// and any issues found.
    #[napi(ts_type = "ConfigValidateData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ConfigValidateData>,

    /// Error information (only present when `success` is `false`).
    ///
    /// Contains structured error information with a Node.js-style error code,
    /// message, optional context, and error kind.
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

// ============================================================================
// Implementations
// ============================================================================

#[allow(dead_code)]
impl ConfigShowParams {
    /// Creates a new `ConfigShowParams` with the specified root path.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    ///
    /// # Returns
    ///
    /// A new `ConfigShowParams` instance with default optional values.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::config::ConfigShowParams;
    ///
    /// let params = ConfigShowParams::new(".".to_string());
    /// assert_eq!(params.root, ".");
    /// assert!(params.config_path.is_none());
    /// ```
    #[must_use]
    pub fn new(root: String) -> Self {
        Self { root, config_path: None }
    }

    /// Creates a new `ConfigShowParams` with a custom config path.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    /// * `config_path` - Custom configuration file path
    ///
    /// # Returns
    ///
    /// A new `ConfigShowParams` instance with the specified config path.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::config::ConfigShowParams;
    ///
    /// let params = ConfigShowParams::with_config(
    ///     ".".to_string(),
    ///     "custom/repo.config.json".to_string(),
    /// );
    /// assert_eq!(params.config_path, Some("custom/repo.config.json".to_string()));
    /// ```
    #[must_use]
    pub fn with_config(root: String, config_path: String) -> Self {
        Self { root, config_path: Some(config_path) }
    }
}

#[allow(dead_code)]
impl ConfigValidateParams {
    /// Creates a new `ConfigValidateParams` with the specified root path.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    ///
    /// # Returns
    ///
    /// A new `ConfigValidateParams` instance with default optional values.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::config::ConfigValidateParams;
    ///
    /// let params = ConfigValidateParams::new(".".to_string());
    /// assert_eq!(params.root, ".");
    /// assert!(params.config_path.is_none());
    /// ```
    #[must_use]
    pub fn new(root: String) -> Self {
        Self { root, config_path: None }
    }

    /// Creates a new `ConfigValidateParams` with a custom config path.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    /// * `config_path` - Custom configuration file path
    ///
    /// # Returns
    ///
    /// A new `ConfigValidateParams` instance with the specified config path.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::config::ConfigValidateParams;
    ///
    /// let params = ConfigValidateParams::with_config(
    ///     ".".to_string(),
    ///     "custom/repo.config.json".to_string(),
    /// );
    /// assert_eq!(params.config_path, Some("custom/repo.config.json".to_string()));
    /// ```
    #[must_use]
    pub fn with_config(root: String, config_path: String) -> Self {
        Self { root, config_path: Some(config_path) }
    }
}

#[allow(dead_code)]
impl ChangesetConfigInfo {
    /// Creates a new `ChangesetConfigInfo` with the specified values.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to store active changesets
    /// * `history_path` - Path to store archived changesets
    /// * `available_environments` - List of valid environment names
    /// * `default_environments` - Default environments for new changesets
    ///
    /// # Returns
    ///
    /// A new `ChangesetConfigInfo` instance.
    #[must_use]
    pub fn new(
        path: String,
        history_path: String,
        available_environments: Vec<String>,
        default_environments: Vec<String>,
    ) -> Self {
        Self { path, history_path, available_environments, default_environments }
    }
}

#[allow(dead_code)]
impl Default for ChangesetConfigInfo {
    fn default() -> Self {
        Self {
            path: ".changesets".to_string(),
            history_path: ".changesets/history".to_string(),
            available_environments: vec![],
            default_environments: vec![],
        }
    }
}

#[allow(dead_code)]
impl VersionConfigInfo {
    /// Creates a new `VersionConfigInfo` with the specified values.
    ///
    /// # Arguments
    ///
    /// * `strategy` - Versioning strategy
    /// * `default_bump` - Default version bump type
    /// * `snapshot_format` - Format template for snapshot versions
    ///
    /// # Returns
    ///
    /// A new `VersionConfigInfo` instance.
    #[must_use]
    pub fn new(strategy: String, default_bump: String, snapshot_format: String) -> Self {
        Self { strategy, default_bump, snapshot_format }
    }
}

#[allow(dead_code)]
impl Default for VersionConfigInfo {
    fn default() -> Self {
        Self {
            strategy: "independent".to_string(),
            default_bump: "patch".to_string(),
            snapshot_format: "{version}-{branch}.{commit}".to_string(),
        }
    }
}

#[allow(dead_code)]
impl DependencyConfigInfo {
    /// Creates a new `DependencyConfigInfo` with the specified values.
    #[must_use]
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    pub fn new(
        propagation_bump: String,
        propagate_dependencies: bool,
        propagate_dev_dependencies: bool,
        propagate_peer_dependencies: bool,
        max_depth: u32,
        fail_on_circular: bool,
        skip_workspace_protocol: bool,
        skip_file_protocol: bool,
        skip_link_protocol: bool,
        skip_portal_protocol: bool,
    ) -> Self {
        Self {
            propagation_bump,
            propagate_dependencies,
            propagate_dev_dependencies,
            propagate_peer_dependencies,
            max_depth,
            fail_on_circular,
            skip_workspace_protocol,
            skip_file_protocol,
            skip_link_protocol,
            skip_portal_protocol,
        }
    }
}

#[allow(dead_code)]
impl Default for DependencyConfigInfo {
    fn default() -> Self {
        Self {
            propagation_bump: "patch".to_string(),
            propagate_dependencies: true,
            propagate_dev_dependencies: false,
            propagate_peer_dependencies: false,
            max_depth: 10,
            fail_on_circular: false,
            skip_workspace_protocol: true,
            skip_file_protocol: true,
            skip_link_protocol: true,
            skip_portal_protocol: true,
        }
    }
}

#[allow(dead_code)]
impl RegistryConfigInfo {
    /// Creates a new `RegistryConfigInfo` with the specified values.
    #[must_use]
    pub fn new(
        default_registry: String,
        scoped_registries: Vec<ScopedRegistryEntry>,
        timeout_secs: u32,
        retry_attempts: u32,
        read_npmrc: bool,
    ) -> Self {
        Self { default_registry, scoped_registries, timeout_secs, retry_attempts, read_npmrc }
    }
}

#[allow(dead_code)]
impl Default for RegistryConfigInfo {
    fn default() -> Self {
        Self {
            default_registry: "https://registry.npmjs.org".to_string(),
            scoped_registries: vec![],
            timeout_secs: 30,
            retry_attempts: 3,
            read_npmrc: true,
        }
    }
}

#[allow(dead_code)]
impl ScopedRegistryEntry {
    /// Creates a new `ScopedRegistryEntry`.
    #[must_use]
    pub fn new(scope: String, registry: String) -> Self {
        Self { scope, registry }
    }
}

#[allow(dead_code)]
impl BackupConfigInfo {
    /// Creates a new `BackupConfigInfo` with the specified values.
    #[must_use]
    pub fn new(enabled: bool, path: String, keep_count: u32) -> Self {
        Self { enabled, path, keep_count }
    }
}

#[allow(dead_code)]
impl Default for BackupConfigInfo {
    fn default() -> Self {
        Self { enabled: true, path: ".backups".to_string(), keep_count: 5 }
    }
}

#[allow(dead_code)]
impl UpgradeConfigInfo {
    /// Creates a new `UpgradeConfigInfo` with the specified values.
    #[must_use]
    pub fn new(
        auto_changeset: bool,
        changeset_bump: String,
        registry: RegistryConfigInfo,
        backup: BackupConfigInfo,
    ) -> Self {
        Self { auto_changeset, changeset_bump, registry, backup }
    }
}

#[allow(dead_code)]
impl Default for UpgradeConfigInfo {
    fn default() -> Self {
        Self {
            auto_changeset: true,
            changeset_bump: "patch".to_string(),
            registry: RegistryConfigInfo::default(),
            backup: BackupConfigInfo::default(),
        }
    }
}

#[allow(dead_code)]
impl ChangelogConfigInfo {
    /// Creates a new `ChangelogConfigInfo` with the specified values.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        enabled: bool,
        format: String,
        include_commit_links: bool,
        repository_url: Option<String>,
        conventional: bool,
        template: Option<String>,
        exclude: Vec<String>,
        monorepo_mode: String,
    ) -> Self {
        Self {
            enabled,
            format,
            include_commit_links,
            repository_url,
            conventional,
            template,
            exclude,
            monorepo_mode,
        }
    }
}

#[allow(dead_code)]
impl Default for ChangelogConfigInfo {
    fn default() -> Self {
        Self {
            enabled: true,
            format: "keep-a-changelog".to_string(),
            include_commit_links: true,
            repository_url: None,
            conventional: true,
            template: None,
            exclude: vec![],
            monorepo_mode: "per-package".to_string(),
        }
    }
}

#[allow(dead_code)]
impl AuditSectionsConfigInfo {
    /// Creates a new `AuditSectionsConfigInfo` with the specified values.
    #[must_use]
    #[allow(clippy::fn_params_excessive_bools)]
    pub fn new(
        upgrades: bool,
        dependencies: bool,
        version_consistency: bool,
        breaking_changes: bool,
    ) -> Self {
        Self { upgrades, dependencies, version_consistency, breaking_changes }
    }
}

#[allow(dead_code)]
impl Default for AuditSectionsConfigInfo {
    fn default() -> Self {
        Self {
            upgrades: true,
            dependencies: true,
            version_consistency: true,
            breaking_changes: true,
        }
    }
}

#[allow(dead_code)]
impl HealthScoreWeightsInfo {
    /// Creates a new `HealthScoreWeightsInfo` with the specified values.
    #[must_use]
    pub fn new(
        upgrades_weight: f64,
        dependencies_weight: f64,
        version_consistency_weight: f64,
        breaking_changes_weight: f64,
    ) -> Self {
        Self {
            upgrades_weight,
            dependencies_weight,
            version_consistency_weight,
            breaking_changes_weight,
        }
    }
}

#[allow(dead_code)]
impl Default for HealthScoreWeightsInfo {
    fn default() -> Self {
        Self {
            upgrades_weight: 0.25,
            dependencies_weight: 0.25,
            version_consistency_weight: 0.25,
            breaking_changes_weight: 0.25,
        }
    }
}

#[allow(dead_code)]
impl AuditConfigInfo {
    /// Creates a new `AuditConfigInfo` with the specified values.
    #[must_use]
    pub fn new(
        enabled: bool,
        min_severity: String,
        sections: AuditSectionsConfigInfo,
        health_score_weights: HealthScoreWeightsInfo,
    ) -> Self {
        Self { enabled, min_severity, sections, health_score_weights }
    }
}

#[allow(dead_code)]
impl Default for AuditConfigInfo {
    fn default() -> Self {
        Self {
            enabled: true,
            min_severity: "low".to_string(),
            sections: AuditSectionsConfigInfo::default(),
            health_score_weights: HealthScoreWeightsInfo::default(),
        }
    }
}

#[allow(dead_code)]
impl GitConfigInfo {
    /// Creates a new `GitConfigInfo` with the specified values.
    #[must_use]
    pub fn new(branch_base: String, detect_affected_packages: bool) -> Self {
        Self { branch_base, detect_affected_packages }
    }
}

#[allow(dead_code)]
impl Default for GitConfigInfo {
    fn default() -> Self {
        Self { branch_base: "main".to_string(), detect_affected_packages: true }
    }
}

#[allow(dead_code)]
impl ExecuteConfigInfo {
    /// Creates a new `ExecuteConfigInfo` with the specified values.
    #[must_use]
    pub fn new(timeout_secs: u32, per_package_timeout_secs: u32, max_parallel: u32) -> Self {
        Self { timeout_secs, per_package_timeout_secs, max_parallel }
    }
}

#[allow(dead_code)]
impl Default for ExecuteConfigInfo {
    fn default() -> Self {
        Self { timeout_secs: 300, per_package_timeout_secs: 60, max_parallel: 4 }
    }
}

#[allow(dead_code)]
impl ConfigData {
    /// Creates a new `ConfigData` with the specified values.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        changeset: ChangesetConfigInfo,
        version: VersionConfigInfo,
        dependency: DependencyConfigInfo,
        upgrade: UpgradeConfigInfo,
        changelog: ChangelogConfigInfo,
        audit: AuditConfigInfo,
        git: GitConfigInfo,
        execute: ExecuteConfigInfo,
    ) -> Self {
        Self { changeset, version, dependency, upgrade, changelog, audit, git, execute }
    }
}

#[allow(dead_code)]
impl ConfigShowData {
    /// Creates a new `ConfigShowData` with the specified values.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the loaded configuration file
    /// * `config_format` - Format of the configuration file
    /// * `config` - The loaded configuration data
    ///
    /// # Returns
    ///
    /// A new `ConfigShowData` instance.
    #[must_use]
    pub fn new(config_path: String, config_format: String, config: ConfigData) -> Self {
        Self { config_path, config_format, config }
    }
}

#[allow(dead_code)]
impl ConfigValidationIssue {
    /// Creates a new validation error.
    ///
    /// # Arguments
    ///
    /// * `field` - The configuration field with the issue
    /// * `message` - Human-readable description of the issue
    ///
    /// # Returns
    ///
    /// A new `ConfigValidationIssue` with severity "error".
    #[must_use]
    pub fn error(field: String, message: String) -> Self {
        Self { severity: "error".to_string(), field, message, suggestion: None }
    }

    /// Creates a new validation error with a suggestion.
    ///
    /// # Arguments
    ///
    /// * `field` - The configuration field with the issue
    /// * `message` - Human-readable description of the issue
    /// * `suggestion` - Suggestion for fixing the issue
    ///
    /// # Returns
    ///
    /// A new `ConfigValidationIssue` with severity "error" and a suggestion.
    #[must_use]
    pub fn error_with_suggestion(field: String, message: String, suggestion: String) -> Self {
        Self { severity: "error".to_string(), field, message, suggestion: Some(suggestion) }
    }

    /// Creates a new validation warning.
    ///
    /// # Arguments
    ///
    /// * `field` - The configuration field with the issue
    /// * `message` - Human-readable description of the issue
    ///
    /// # Returns
    ///
    /// A new `ConfigValidationIssue` with severity "warning".
    #[must_use]
    pub fn warning(field: String, message: String) -> Self {
        Self { severity: "warning".to_string(), field, message, suggestion: None }
    }

    /// Creates a new validation warning with a suggestion.
    ///
    /// # Arguments
    ///
    /// * `field` - The configuration field with the issue
    /// * `message` - Human-readable description of the issue
    /// * `suggestion` - Suggestion for fixing the issue
    ///
    /// # Returns
    ///
    /// A new `ConfigValidationIssue` with severity "warning" and a suggestion.
    #[must_use]
    pub fn warning_with_suggestion(field: String, message: String, suggestion: String) -> Self {
        Self { severity: "warning".to_string(), field, message, suggestion: Some(suggestion) }
    }

    /// Creates a new informational validation issue.
    ///
    /// # Arguments
    ///
    /// * `field` - The configuration field with the issue
    /// * `message` - Human-readable description of the issue
    ///
    /// # Returns
    ///
    /// A new `ConfigValidationIssue` with severity "info".
    #[must_use]
    pub fn info(field: String, message: String) -> Self {
        Self { severity: "info".to_string(), field, message, suggestion: None }
    }

    /// Creates a new `ConfigValidationIssue` with all fields specified.
    ///
    /// # Arguments
    ///
    /// * `severity` - Issue severity ("error", "warning", or "info")
    /// * `field` - The configuration field with the issue
    /// * `message` - Human-readable description of the issue
    /// * `suggestion` - Optional suggestion for fixing the issue
    ///
    /// # Returns
    ///
    /// A new `ConfigValidationIssue` instance.
    #[must_use]
    pub fn new(
        severity: String,
        field: String,
        message: String,
        suggestion: Option<String>,
    ) -> Self {
        Self { severity, field, message, suggestion }
    }

    /// Returns whether this issue is an error.
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.severity == "error"
    }

    /// Returns whether this issue is a warning.
    #[must_use]
    pub fn is_warning(&self) -> bool {
        self.severity == "warning"
    }

    /// Returns whether this issue is informational.
    #[must_use]
    pub fn is_info(&self) -> bool {
        self.severity == "info"
    }
}

#[allow(dead_code)]
impl ConfigValidateData {
    /// Creates a new `ConfigValidateData` with the specified values.
    ///
    /// # Arguments
    ///
    /// * `valid` - Whether the configuration is valid
    /// * `config_path` - Path to the validated configuration file
    /// * `errors` - List of validation errors
    /// * `warnings` - List of validation warnings
    ///
    /// # Returns
    ///
    /// A new `ConfigValidateData` instance.
    #[must_use]
    pub fn new(
        valid: bool,
        config_path: String,
        errors: Vec<ConfigValidationIssue>,
        warnings: Vec<ConfigValidationIssue>,
    ) -> Self {
        Self { valid, config_path, errors, warnings }
    }

    /// Creates a valid configuration result.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the validated configuration file
    ///
    /// # Returns
    ///
    /// A new `ConfigValidateData` with `valid = true` and no errors or warnings.
    #[must_use]
    pub fn valid(config_path: String) -> Self {
        Self { valid: true, config_path, errors: vec![], warnings: vec![] }
    }

    /// Creates a valid configuration result with warnings.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the validated configuration file
    /// * `warnings` - List of validation warnings
    ///
    /// # Returns
    ///
    /// A new `ConfigValidateData` with `valid = true` and the specified warnings.
    #[must_use]
    pub fn valid_with_warnings(config_path: String, warnings: Vec<ConfigValidationIssue>) -> Self {
        Self { valid: true, config_path, errors: vec![], warnings }
    }

    /// Creates an invalid configuration result.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the validated configuration file
    /// * `errors` - List of validation errors
    ///
    /// # Returns
    ///
    /// A new `ConfigValidateData` with `valid = false` and the specified errors.
    #[must_use]
    pub fn invalid(config_path: String, errors: Vec<ConfigValidationIssue>) -> Self {
        Self { valid: false, config_path, errors, warnings: vec![] }
    }

    /// Creates an invalid configuration result with both errors and warnings.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the validated configuration file
    /// * `errors` - List of validation errors
    /// * `warnings` - List of validation warnings
    ///
    /// # Returns
    ///
    /// A new `ConfigValidateData` with `valid = false` and the specified issues.
    #[must_use]
    pub fn invalid_with_warnings(
        config_path: String,
        errors: Vec<ConfigValidationIssue>,
        warnings: Vec<ConfigValidationIssue>,
    ) -> Self {
        Self { valid: false, config_path, errors, warnings }
    }

    /// Returns the total number of issues (errors + warnings).
    #[must_use]
    pub fn total_issues(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }

    /// Returns whether there are any errors.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns whether there are any warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

#[allow(dead_code)]
impl ConfigShowApiResponse {
    /// Creates a successful config show response with data.
    ///
    /// # Arguments
    ///
    /// * `data` - The config show data to include
    ///
    /// # Returns
    ///
    /// A new `ConfigShowApiResponse` with `success = true` and the provided data.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::config::{
    ///     ConfigShowApiResponse, ConfigShowData, ConfigData
    /// };
    ///
    /// let data = ConfigShowData::new(
    ///     "repo.config.json".to_string(),
    ///     "json".to_string(),
    ///     ConfigData::default(),
    /// );
    /// let response = ConfigShowApiResponse::success(data);
    /// assert!(response.success);
    /// assert!(response.data.is_some());
    /// ```
    #[must_use]
    pub fn success(data: ConfigShowData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failed config show response with error information.
    ///
    /// # Arguments
    ///
    /// * `error` - The error information to include
    ///
    /// # Returns
    ///
    /// A new `ConfigShowApiResponse` with `success = false` and the provided error.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::config::ConfigShowApiResponse;
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::not_found("Config file not found", Some("repo.config.json"));
    /// let response = ConfigShowApiResponse::failure(error);
    /// assert!(!response.success);
    /// assert!(response.error.is_some());
    /// ```
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns whether this response represents a success.
    ///
    /// # Returns
    ///
    /// `true` if the operation succeeded, `false` otherwise.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns whether this response represents a failure.
    ///
    /// # Returns
    ///
    /// `true` if the operation failed, `false` otherwise.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

#[allow(dead_code)]
impl ConfigValidateApiResponse {
    /// Creates a successful config validate response with data.
    ///
    /// # Arguments
    ///
    /// * `data` - The config validate data to include
    ///
    /// # Returns
    ///
    /// A new `ConfigValidateApiResponse` with `success = true` and the provided data.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::config::{
    ///     ConfigValidateApiResponse, ConfigValidateData
    /// };
    ///
    /// let data = ConfigValidateData::valid("repo.config.json".to_string());
    /// let response = ConfigValidateApiResponse::success(data);
    /// assert!(response.success);
    /// assert!(response.data.is_some());
    /// ```
    #[must_use]
    pub fn success(data: ConfigValidateData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failed config validate response with error information.
    ///
    /// # Arguments
    ///
    /// * `error` - The error information to include
    ///
    /// # Returns
    ///
    /// A new `ConfigValidateApiResponse` with `success = false` and the provided error.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::config::ConfigValidateApiResponse;
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::not_found("Config file not found", Some("repo.config.json"));
    /// let response = ConfigValidateApiResponse::failure(error);
    /// assert!(!response.success);
    /// assert!(response.error.is_some());
    /// ```
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns whether this response represents a success.
    ///
    /// # Returns
    ///
    /// `true` if the operation succeeded, `false` otherwise.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns whether this response represents a failure.
    ///
    /// # Returns
    ///
    /// `true` if the operation failed, `false` otherwise.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}
