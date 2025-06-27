//! Configuration management for monorepo tools
//! 
//! This module provides types and utilities for managing monorepo configurations,
//! including versioning, tasks, changelogs, hooks, changesets, and plugins.

pub mod types;
pub mod components;
mod manager;
mod defaults;

#[cfg(test)]
mod tests;

// Explicit re-exports from types module
pub use types::{
    // Core
    MonorepoConfig, Environment,
    // Manager
    ConfigManager, PatternMatcher,
    // Versioning
    VersioningConfig, VersionBumpType,
    // Tasks
    TasksConfig,
    // Changelog
    ChangelogConfig, ChangelogTemplate, CommitGrouping, ChangelogFormat,
    // Hooks
    HooksConfig, HookConfig,
    // Changesets
    ChangesetsConfig,
    // Plugins
    PluginsConfig,
    // Workspace
    WorkspaceConfig, WorkspacePattern, WorkspacePatternOptions,
    PackageManagerType, PackageManagerConfigs,
    NpmWorkspaceConfig, YarnWorkspaceConfig, YarnVersion,
    PnpmWorkspaceConfig, BunWorkspaceConfig,
    WorkspaceValidationConfig, PackageDiscoveryConfig,
    // Git
    GitConfig, BranchConfig, BranchType,
};

// Re-export components
pub use components::{
    ConfigComponents, ConfigPersistence, ConfigReader, ConfigWriter,
    WorkspacePatternManager, MultiPatternMatcher,
};