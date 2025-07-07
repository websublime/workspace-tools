//! Configuration types organized by functionality

// Core configuration types
pub mod core;
pub use core::{Environment, MonorepoConfig};

// Implementation structs (moved from main modules)
pub mod manager;
pub use manager::{ConfigManager, PatternMatcher};

// Versioning configuration types
pub mod versioning;
pub use versioning::{VersionBumpType, VersioningConfig};

// Task configuration types
pub mod tasks;
pub use tasks::TasksConfig;

// Changelog configuration types
pub mod changelog;
pub use changelog::{ChangelogConfig, ChangelogFormat, ChangelogTemplate, CommitGrouping};

// Git hooks configuration types
pub mod hooks;
pub use hooks::{
    AutoDetectionConfig, HookConfig, HooksConfig, HookStrategy, HuskyConfig,
};

// Changesets configuration types
pub mod changesets;
pub use changesets::ChangesetsConfig;

// Plugin configuration types
pub mod plugins;
pub use plugins::PluginsConfig;

// Workspace configuration types
pub mod workspace;
pub use workspace::{
    BunWorkspaceConfig, NpmWorkspaceConfig, PackageDiscoveryConfig, PackageManagerConfigs,
    PackageManagerType, PnpmWorkspaceConfig, WorkspaceConfig, WorkspacePattern,
    WorkspacePatternOptions, WorkspaceValidationConfig, YarnVersion, YarnWorkspaceConfig,
};

// Git configuration types
pub mod git;
pub use git::{
    BranchConfig, BranchType, GitConfig, RepositoryHostConfig, RepositoryProvider, SshConversion,
    UrlPatterns,
};

// Validation configuration types
pub mod validation;
pub use validation::{
    ChangeDetectionRulesConfig, DependencyAnalysisConfig, PatternScoringConfig, QualityGatesConfig,
    TaskPriorityConfig, ValidationConfig, ValidationPatternsConfig, VersionBumpRulesConfig,
};
