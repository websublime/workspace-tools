//! Workspace configuration types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::Environment;

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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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