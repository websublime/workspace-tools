//! Configuration types organized by functionality

// Core configuration types
pub mod core;
pub use core::{MonorepoConfig, Environment};

// Implementation structs (moved from main modules)
pub mod manager;
pub use manager::{ConfigManager, PatternMatcher};

// Versioning configuration types
pub mod versioning;
pub use versioning::{VersioningConfig, VersionBumpType};

// Task configuration types
pub mod tasks;
pub use tasks::TasksConfig;

// Changelog configuration types
pub mod changelog;
pub use changelog::{
    ChangelogConfig, ChangelogTemplate, CommitGrouping, ChangelogFormat,
};

// Git hooks configuration types
pub mod hooks;
pub use hooks::{HooksConfig, HookConfig};

// Changesets configuration types
pub mod changesets;
pub use changesets::ChangesetsConfig;

// Plugin configuration types
pub mod plugins;
pub use plugins::PluginsConfig;

// Workspace configuration types
pub mod workspace;
pub use workspace::{
    WorkspaceConfig, WorkspacePattern, WorkspacePatternOptions,
    PackageManagerType, PackageManagerConfigs,
    NpmWorkspaceConfig, YarnWorkspaceConfig, YarnVersion,
    PnpmWorkspaceConfig, BunWorkspaceConfig,
    WorkspaceValidationConfig, PackageDiscoveryConfig,
};

// Git configuration types
pub mod git;
pub use git::{GitConfig, BranchConfig, BranchType};

// Validation configuration types
pub mod validation;
pub use validation::{
    ValidationConfig, TaskPriorityConfig, ChangeDetectionRulesConfig,
    VersionBumpRulesConfig, DependencyAnalysisConfig, PatternScoringConfig,
    ValidationPatternsConfig, QualityGatesConfig,
};