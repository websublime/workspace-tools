//! Configuration types for monorepo tools

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main configuration for monorepo tools
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

    /// Deployment environments
    pub environments: Vec<Environment>,
}

impl Default for MonorepoConfig {
    fn default() -> Self {
        Self {
            versioning: VersioningConfig::default(),
            tasks: TasksConfig::default(),
            changelog: ChangelogConfig::default(),
            hooks: HooksConfig::default(),
            changesets: ChangesetsConfig::default(),
            plugins: PluginsConfig::default(),
            workspace: WorkspaceConfig::default(),
            environments: vec![
                Environment::Development,
                Environment::Staging,
                Environment::Production,
            ],
        }
    }
}

/// Versioning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersioningConfig {
    /// Default bump type when not specified
    pub default_bump: VersionBumpType,

    /// Whether to propagate version changes to dependents
    pub propagate_changes: bool,

    /// Snapshot version format
    pub snapshot_format: String,

    /// Version tag prefix
    pub tag_prefix: String,

    /// Whether to create tags automatically
    pub auto_tag: bool,
}

impl Default for VersioningConfig {
    fn default() -> Self {
        Self {
            default_bump: VersionBumpType::Patch,
            propagate_changes: true,
            snapshot_format: "{version}-snapshot.{sha}".to_string(),
            tag_prefix: "v".to_string(),
            auto_tag: true,
        }
    }
}

/// Type of version bump
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionBumpType {
    /// Major version bump (x.0.0)
    Major,
    /// Minor version bump (0.x.0)
    Minor,
    /// Patch version bump (0.0.x)
    Patch,
    /// Snapshot version with commit SHA
    Snapshot,
}

/// Task management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksConfig {
    /// Default tasks to run on changes
    pub default_tasks: Vec<String>,

    /// Task groups
    pub groups: HashMap<String, Vec<String>>,

    /// Whether to run tasks in parallel
    pub parallel: bool,

    /// Maximum concurrent tasks
    pub max_concurrent: usize,

    /// Task timeout in seconds
    pub timeout: u64,
}

impl Default for TasksConfig {
    fn default() -> Self {
        Self {
            default_tasks: vec!["test".to_string(), "lint".to_string()],
            groups: HashMap::new(),
            parallel: true,
            max_concurrent: 4,
            timeout: 300, // 5 minutes
        }
    }
}

/// Changelog configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogConfig {
    /// Changelog template
    pub template: ChangelogTemplate,

    /// How to group commits
    pub grouping: CommitGrouping,

    /// Output format
    pub output_format: ChangelogFormat,

    /// Whether to include breaking changes section
    pub include_breaking_changes: bool,

    /// Conventional commit type mappings
    pub conventional_commit_types: HashMap<String, String>,
}

impl Default for ChangelogConfig {
    fn default() -> Self {
        let mut types = HashMap::new();
        types.insert("feat".to_string(), "Features".to_string());
        types.insert("fix".to_string(), "Bug Fixes".to_string());
        types.insert("docs".to_string(), "Documentation".to_string());
        types.insert("style".to_string(), "Styles".to_string());
        types.insert("refactor".to_string(), "Code Refactoring".to_string());
        types.insert("perf".to_string(), "Performance Improvements".to_string());
        types.insert("test".to_string(), "Tests".to_string());
        types.insert("build".to_string(), "Build System".to_string());
        types.insert("ci".to_string(), "Continuous Integration".to_string());
        types.insert("chore".to_string(), "Chores".to_string());
        types.insert("revert".to_string(), "Reverts".to_string());

        Self {
            template: ChangelogTemplate::default(),
            grouping: CommitGrouping::Type,
            output_format: ChangelogFormat::Markdown,
            include_breaking_changes: true,
            conventional_commit_types: types,
        }
    }
}

/// Changelog template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogTemplate {
    /// Header template
    pub header_template: String,

    /// Section template
    pub section_template: String,

    /// Commit template
    pub commit_template: String,

    /// Footer template
    pub footer_template: String,
}

impl Default for ChangelogTemplate {
    fn default() -> Self {
        Self {
            header_template: "# Changelog\n\nAll notable changes to this project will be documented in this file.\n\n".to_string(),
            section_template: "## [{version}] - {date}\n\n".to_string(),
            commit_template: "- {description} ([{hash}]({url}))\n".to_string(),
            footer_template: "\n---\n\nGenerated by [Sublime Monorepo Tools](https://github.com/websublime/workspace-node-tools)\n".to_string(),
        }
    }
}

/// How to group commits in changelog
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitGrouping {
    /// Group by commit type
    Type,
    /// Group by scope
    Scope,
    /// No grouping
    None,
}

/// Changelog output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangelogFormat {
    /// Markdown format
    Markdown,
    /// Plain text format
    Text,
    /// JSON format
    Json,
}

/// Git hooks configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    /// Whether hooks are enabled
    pub enabled: bool,

    /// Pre-commit hook configuration
    pub pre_commit: HookConfig,

    /// Pre-push hook configuration
    pub pre_push: HookConfig,

    /// Post-merge hook configuration
    pub post_merge: Option<HookConfig>,

    /// Custom hooks directory
    pub hooks_dir: Option<PathBuf>,
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            pre_commit: HookConfig {
                enabled: true,
                validate_changeset: true,
                run_tasks: vec!["lint".to_string()],
                custom_script: None,
            },
            pre_push: HookConfig {
                enabled: true,
                validate_changeset: false,
                run_tasks: vec!["test".to_string(), "build".to_string()],
                custom_script: None,
            },
            post_merge: None,
            hooks_dir: None,
        }
    }
}

/// Individual hook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// Whether this hook is enabled
    pub enabled: bool,

    /// Whether to validate changeset exists
    pub validate_changeset: bool,

    /// Tasks to run
    pub run_tasks: Vec<String>,

    /// Custom script to execute
    pub custom_script: Option<PathBuf>,
}

/// Changesets configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesetsConfig {
    /// Whether changesets are required
    pub required: bool,

    /// Changeset directory
    pub changeset_dir: PathBuf,

    /// Default environments for new changesets
    pub default_environments: Vec<Environment>,

    /// Whether to auto-deploy to environments
    pub auto_deploy: bool,

    /// Changeset filename format
    pub filename_format: String,
}

impl Default for ChangesetsConfig {
    fn default() -> Self {
        Self {
            required: true,
            changeset_dir: PathBuf::from(".changesets"),
            default_environments: vec![Environment::Development, Environment::Staging],
            auto_deploy: false,
            filename_format: "{timestamp}-{branch}-{hash}.json".to_string(),
        }
    }
}

/// Plugin system configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginsConfig {
    /// Enabled plugins
    pub enabled: Vec<String>,

    /// Plugin directories
    pub plugin_dirs: Vec<PathBuf>,

    /// Plugin-specific configurations
    pub configs: HashMap<String, serde_json::Value>,
}

/// Deployment environment
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Staging => write!(f, "staging"),
            Environment::Integration => write!(f, "integration"),
            Environment::Production => write!(f, "production"),
            Environment::Custom(name) => write!(f, "{name}"),
        }
    }
}

/// Workspace configuration for monorepo structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Custom workspace patterns to be used for package discovery
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

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            merge_with_detected: true,
            package_manager_configs: PackageManagerConfigs::default(),
            validation: WorkspaceValidationConfig::default(),
            discovery: PackageDiscoveryConfig::default(),
        }
    }
}

/// Workspace pattern configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePattern {
    /// The glob pattern for finding packages
    pub pattern: String,

    /// Description of what this pattern matches
    pub description: Option<String>,

    /// Whether this pattern is enabled
    pub enabled: bool,

    /// Priority when multiple patterns match (higher = preferred)
    pub priority: u32,

    /// Package manager types this pattern applies to
    pub package_managers: Option<Vec<PackageManagerType>>,

    /// Environments where this pattern is used
    pub environments: Option<Vec<Environment>>,

    /// Additional pattern options
    pub options: WorkspacePatternOptions,
}

impl Default for WorkspacePattern {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            description: None,
            enabled: true,
            priority: 100,
            package_managers: None,
            environments: None,
            options: WorkspacePatternOptions::default(),
        }
    }
}

/// Package manager type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageManagerType {
    /// npm package manager
    Npm,
    /// Yarn package manager (classic)
    Yarn,
    /// Yarn Berry (v2+)
    YarnBerry,
    /// pnpm package manager
    Pnpm,
    /// bun package manager
    Bun,
    /// Custom package manager
    Custom(String),
}

/// Workspace pattern options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePatternOptions {
    /// Whether to include nested packages
    pub include_nested: bool,

    /// Maximum depth for nested package discovery
    pub max_depth: Option<u32>,

    /// Patterns to exclude
    pub exclude_patterns: Vec<String>,

    /// Whether to follow symlinks
    pub follow_symlinks: bool,

    /// Whether this pattern should override auto-detection
    pub override_detection: bool,
}

impl Default for WorkspacePatternOptions {
    fn default() -> Self {
        Self {
            include_nested: true,
            max_depth: None,
            exclude_patterns: Vec::new(),
            follow_symlinks: false,
            override_detection: false,
        }
    }
}

/// Package manager specific configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManagerConfigs {
    /// npm specific configuration
    pub npm: Option<NpmWorkspaceConfig>,

    /// Yarn specific configuration
    pub yarn: Option<YarnWorkspaceConfig>,

    /// pnpm specific configuration
    pub pnpm: Option<PnpmWorkspaceConfig>,

    /// bun specific configuration
    pub bun: Option<BunWorkspaceConfig>,

    /// Custom package manager configurations
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for PackageManagerConfigs {
    fn default() -> Self {
        Self { npm: None, yarn: None, pnpm: None, bun: None, custom: HashMap::new() }
    }
}

/// npm specific workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmWorkspaceConfig {
    /// Override workspaces field in package.json
    pub workspaces_override: Option<Vec<String>>,

    /// Whether to use npm workspaces feature
    pub use_workspaces: bool,

    /// Additional npm workspace options
    pub options: HashMap<String, serde_json::Value>,
}

/// Yarn specific workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YarnWorkspaceConfig {
    /// Override workspaces field in package.json
    pub workspaces_override: Option<Vec<String>>,

    /// Nohoist patterns
    pub nohoist_patterns: Vec<String>,

    /// Whether to use yarn workspaces
    pub use_workspaces: bool,

    /// Yarn version (classic vs berry)
    pub version: YarnVersion,

    /// Additional yarn workspace options
    pub options: HashMap<String, serde_json::Value>,
}

/// Yarn version specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum YarnVersion {
    /// Yarn classic (v1.x)
    Classic,
    /// Yarn Berry (v2+)
    Berry,
    /// Auto-detect yarn version
    Auto,
}

/// pnpm specific workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnpmWorkspaceConfig {
    /// Override packages field in pnpm-workspace.yaml
    pub packages_override: Option<Vec<String>>,

    /// Whether to use pnpm workspaces
    pub use_workspaces: bool,

    /// pnpm workspace filtering options
    pub filter_options: Vec<String>,

    /// Additional pnpm workspace options
    pub options: HashMap<String, serde_json::Value>,
}

/// bun specific workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BunWorkspaceConfig {
    /// Override workspaces field in package.json
    pub workspaces_override: Option<Vec<String>>,

    /// Whether to use bun workspaces
    pub use_workspaces: bool,

    /// Additional bun workspace options
    pub options: HashMap<String, serde_json::Value>,
}

/// Workspace validation configuration
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceValidationConfig {
    /// Whether to validate that all patterns match at least one package
    pub require_pattern_matches: bool,

    /// Whether to warn about orphaned packages
    pub warn_orphaned_packages: bool,

    /// Whether to validate package naming conventions
    pub validate_naming: bool,

    /// Naming convention patterns
    pub naming_patterns: Vec<String>,

    /// Whether to validate package structure
    pub validate_structure: bool,

    /// Required files in each package
    pub required_files: Vec<String>,
}

impl Default for WorkspaceValidationConfig {
    fn default() -> Self {
        Self {
            require_pattern_matches: false,
            warn_orphaned_packages: true,
            validate_naming: false,
            naming_patterns: Vec::new(),
            validate_structure: false,
            required_files: vec!["package.json".to_string()],
        }
    }
}

/// Package discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDiscoveryConfig {
    /// Whether to enable auto-detection of workspace patterns
    pub auto_detect: bool,

    /// Whether to scan for common workspace patterns
    pub scan_common_patterns: bool,

    /// Common patterns to scan for
    pub common_patterns: Vec<String>,

    /// Directories to exclude from scanning
    pub exclude_directories: Vec<String>,

    /// Maximum depth for package discovery
    pub max_scan_depth: u32,

    /// Whether to cache discovery results
    pub cache_results: bool,

    /// Cache validity duration in seconds
    pub cache_duration: u64,
}

impl Default for PackageDiscoveryConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            scan_common_patterns: true,
            common_patterns: vec![
                "packages/*".to_string(),
                "apps/*".to_string(),
                "libs/*".to_string(),
                "services/*".to_string(),
                "tools/*".to_string(),
                "modules/*".to_string(),
            ],
            exclude_directories: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "dist".to_string(),
                "build".to_string(),
                "coverage".to_string(),
            ],
            max_scan_depth: 3,
            cache_results: true,
            cache_duration: 300, // 5 minutes
        }
    }
}
