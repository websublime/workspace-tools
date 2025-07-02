//! Configuration management for monorepo tools
//!
//! This module provides types and utilities for managing monorepo configurations,
//! including versioning, tasks, changelogs, hooks, changesets, and plugins.

pub mod components;
mod defaults;
mod manager;
pub mod types;

// Explicit re-exports from types module
pub use types::{
    BranchConfig,
    BranchType,
    BunWorkspaceConfig,
    // Changelog
    ChangelogConfig,
    ChangelogFormat,
    ChangelogTemplate,
    // Changesets
    ChangesetsConfig,
    CommitGrouping,
    // Manager
    ConfigManager,
    Environment,
    // Git
    GitConfig,
    HookConfig,
    // Hooks
    HooksConfig,
    // Core
    MonorepoConfig,
    NpmWorkspaceConfig,
    PackageDiscoveryConfig,
    PackageManagerConfigs,
    PackageManagerType,
    PatternMatcher,
    // Plugins
    PluginsConfig,
    PnpmWorkspaceConfig,
    // Tasks
    TasksConfig,
    VersionBumpType,
    // Versioning
    VersioningConfig,
    // Workspace
    WorkspaceConfig,
    WorkspacePattern,
    WorkspacePatternOptions,
    WorkspaceValidationConfig,
    YarnVersion,
    YarnWorkspaceConfig,
};

// Re-export components
pub use components::{
    ConfigComponents, ConfigPersistence, ConfigReader, ConfigWriter, MultiPatternMatcher,
    WorkspacePatternManager,
};
