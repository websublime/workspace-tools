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

    /// Package manager command configurations
    pub package_manager_commands: PackageManagerCommandConfig,

    /// File pattern configurations for change detection
    pub file_patterns: FilePatternConfig,

    /// Registry and tool configurations
    pub tool_configs: ToolConfig,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            merge_with_detected: true,
            package_manager_configs: PackageManagerConfigs::default(),
            validation: WorkspaceValidationConfig::default(),
            discovery: PackageDiscoveryConfig::default(),
            package_manager_commands: PackageManagerCommandConfig::default(),
            file_patterns: FilePatternConfig::default(),
            tool_configs: ToolConfig::default(),
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// Package manager command configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManagerCommandConfig {
    /// Command name for each package manager
    pub commands: HashMap<PackageManagerType, String>,

    /// Version check arguments for each package manager
    pub version_args: HashMap<PackageManagerType, Vec<String>>,

    /// Default package manager when none is detected
    pub default_manager: PackageManagerType,

    /// Script execution arguments (e.g., "run" for npm run)
    pub script_run_args: HashMap<PackageManagerType, Vec<String>>,

    /// Additional arguments for script execution
    pub extra_args_separator: String,
}

impl Default for PackageManagerCommandConfig {
    fn default() -> Self {
        let mut commands = HashMap::new();
        commands.insert(PackageManagerType::Npm, "npm".to_string());
        commands.insert(PackageManagerType::Yarn, "yarn".to_string());
        commands.insert(PackageManagerType::YarnBerry, "yarn".to_string());
        commands.insert(PackageManagerType::Pnpm, "pnpm".to_string());
        commands.insert(PackageManagerType::Bun, "bun".to_string());

        let mut version_args = HashMap::new();
        version_args.insert(PackageManagerType::Npm, vec!["--version".to_string()]);
        version_args.insert(PackageManagerType::Yarn, vec!["--version".to_string()]);
        version_args.insert(PackageManagerType::YarnBerry, vec!["--version".to_string()]);
        version_args.insert(PackageManagerType::Pnpm, vec!["--version".to_string()]);
        version_args.insert(PackageManagerType::Bun, vec!["--version".to_string()]);

        let mut script_run_args = HashMap::new();
        script_run_args.insert(PackageManagerType::Npm, vec!["run".to_string()]);
        script_run_args.insert(PackageManagerType::Yarn, vec!["run".to_string()]);
        script_run_args.insert(PackageManagerType::YarnBerry, vec!["run".to_string()]);
        script_run_args.insert(PackageManagerType::Pnpm, vec!["run".to_string()]);
        script_run_args.insert(PackageManagerType::Bun, vec!["run".to_string()]);

        Self {
            commands,
            version_args,
            default_manager: PackageManagerType::Npm,
            script_run_args,
            extra_args_separator: "--".to_string(),
        }
    }
}

/// File pattern configuration for change detection and workspace operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePatternConfig {
    /// Package definition file patterns
    pub package_files: Vec<String>,

    /// Lock file patterns for package managers
    pub lock_files: Vec<String>,

    /// Source code file patterns
    pub source_patterns: Vec<String>,

    /// Test file patterns
    pub test_patterns: Vec<String>,

    /// Documentation file patterns
    pub docs_patterns: Vec<String>,

    /// Configuration file patterns
    pub config_patterns: Vec<String>,

    /// Build output directory patterns to exclude
    pub build_output_patterns: Vec<String>,
}

impl Default for FilePatternConfig {
    fn default() -> Self {
        Self {
            package_files: vec![
                "package.json".to_string(),
                "package-lock.json".to_string(),
                "yarn.lock".to_string(),
                "pnpm-lock.yaml".to_string(),
                "bun.lockb".to_string(),
            ],
            lock_files: vec![
                "package-lock.json".to_string(),
                "yarn.lock".to_string(),
                "pnpm-lock.yaml".to_string(),
                "bun.lockb".to_string(),
            ],
            source_patterns: vec![
                "src/**/*.{ts,js,tsx,jsx}".to_string(),
                "lib/**/*.{ts,js,tsx,jsx}".to_string(),
                "**/*.{ts,js,tsx,jsx}".to_string(),
            ],
            test_patterns: vec![
                "**/*.{test,spec}.{ts,js,tsx,jsx}".to_string(),
                "**/__tests__/**/*.{ts,js,tsx,jsx}".to_string(),
                "**/test/**/*.{ts,js,tsx,jsx}".to_string(),
                "**/tests/**/*.{ts,js,tsx,jsx}".to_string(),
            ],
            docs_patterns: vec![
                "**/*.md".to_string(),
                "**/docs/**/*".to_string(),
                "**/README*".to_string(),
                "**/CHANGELOG*".to_string(),
            ],
            config_patterns: vec![
                "**/*.config.{js,ts,json}".to_string(),
                "**/.eslintrc*".to_string(),
                "**/tsconfig*.json".to_string(),
                "**/.gitignore".to_string(),
                "**/.env*".to_string(),
            ],
            build_output_patterns: vec![
                "dist".to_string(),
                "build".to_string(),
                "coverage".to_string(),
                "tmp".to_string(),
                ".next".to_string(),
                ".nuxt".to_string(),
            ],
        }
    }
}

/// Tool and registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Registry URL patterns and their types
    pub registry_patterns: HashMap<String, String>,

    /// Environment variable names for authentication
    pub auth_env_vars: HashMap<String, Vec<String>>,

    /// Configuration file search patterns
    pub config_file_patterns: Vec<String>,

    /// Default task groups and their commands
    pub default_task_groups: HashMap<String, Vec<String>>,
}

impl Default for ToolConfig {
    fn default() -> Self {
        let mut registry_patterns = HashMap::new();
        registry_patterns.insert("registry.npmjs.org".to_string(), "npm".to_string());
        registry_patterns.insert("npm.pkg.github.com".to_string(), "github".to_string());
        registry_patterns.insert("pkgs.dev.azure.com".to_string(), "azure".to_string());
        registry_patterns.insert("gitlab.com".to_string(), "gitlab".to_string());

        let mut auth_env_vars = HashMap::new();
        auth_env_vars.insert("npm".to_string(), vec!["NPM_TOKEN".to_string()]);
        auth_env_vars.insert("github".to_string(), vec!["GITHUB_TOKEN".to_string(), "NPM_TOKEN".to_string()]);
        auth_env_vars.insert("azure".to_string(), vec!["AZURE_TOKEN".to_string()]);
        auth_env_vars.insert("gitlab".to_string(), vec!["GITLAB_TOKEN".to_string()]);

        let mut default_task_groups = HashMap::new();
        default_task_groups.insert("quality".to_string(), vec![
            "lint".to_string(), 
            "typecheck".to_string(), 
            "test".to_string()
        ]);
        default_task_groups.insert("build".to_string(), vec![
            "clean".to_string(), 
            "compile".to_string(), 
            "bundle".to_string()
        ]);
        default_task_groups.insert("release".to_string(), vec![
            "quality".to_string(), 
            "build".to_string(), 
            "docs".to_string()
        ]);

        Self {
            registry_patterns,
            auth_env_vars,
            config_file_patterns: vec![
                ".monorepo/config.json".to_string(),
                ".monorepo/config.toml".to_string(),
                ".monorepo/config.yaml".to_string(),
                ".monorepo/config.yml".to_string(),
                "monorepo.config.json".to_string(),
                "monorepo.config.toml".to_string(),
                "monorepo.config.yaml".to_string(),
                "monorepo.config.yml".to_string(),
            ],
            default_task_groups,
        }
    }
}

impl PackageManagerCommandConfig {
    /// Get the command for a specific package manager
    #[must_use]
    pub fn get_command(&self, pm_type: &PackageManagerType) -> &str {
        self.commands.get(pm_type)
            .map(String::as_str)
            .unwrap_or_else(|| self.commands.get(&self.default_manager).unwrap())
    }

    /// Get version check arguments for a package manager
    #[must_use]
    pub fn get_version_args(&self, pm_type: &PackageManagerType) -> &[String] {
        self.version_args.get(pm_type)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Get script run arguments for a package manager
    #[must_use]
    pub fn get_script_run_args(&self, pm_type: &PackageManagerType) -> &[String] {
        self.script_run_args.get(pm_type)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

impl FilePatternConfig {
    /// Check if a file path matches any package file pattern
    #[must_use]
    pub fn is_package_file(&self, path: &str) -> bool {
        self.package_files.iter().any(|pattern| {
            // Simple contains check for exact matches, could be extended with glob matching
            path.ends_with(pattern)
        })
    }

    /// Check if a file path matches any source code pattern
    #[must_use]
    pub fn is_source_file(&self, path: &str) -> bool {
        self.source_patterns.iter().any(|pattern| {
            // Simplified pattern matching - would use proper glob library in production
            let ext_patterns = ["ts", "js", "tsx", "jsx"];
            ext_patterns.iter().any(|ext| path.ends_with(&format!(".{ext}")))
        })
    }

    /// Check if a file path matches any test pattern
    #[must_use]
    pub fn is_test_file(&self, path: &str) -> bool {
        self.test_patterns.iter().any(|_pattern| {
            // Simplified test file detection
            path.contains("test") || path.contains("spec") || path.contains("__tests__")
        })
    }
}

impl ToolConfig {
    /// Get registry type from URL
    #[must_use]
    pub fn get_registry_type(&self, url: &str) -> &str {
        for (pattern, registry_type) in &self.registry_patterns {
            if url.contains(pattern) {
                return registry_type;
            }
        }
        "custom"
    }

    /// Get authentication environment variables for a registry type
    #[must_use]
    pub fn get_auth_env_vars(&self, registry_type: &str) -> &[String] {
        self.auth_env_vars.get(registry_type)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Get default task group commands
    #[must_use]
    pub fn get_task_group(&self, group_name: &str) -> Option<&[String]> {
        self.default_task_groups.get(group_name).map(Vec::as_slice)
    }
}