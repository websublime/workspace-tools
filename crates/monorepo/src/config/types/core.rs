//! Core configuration types for monorepo tools

use serde::{Deserialize, Serialize};
use super::{
    VersioningConfig, TasksConfig, ChangelogConfig, HooksConfig,
    ChangesetsConfig, PluginsConfig, WorkspaceConfig, GitConfig,
    ValidationConfig,
};

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

    /// Git configuration
    pub git: GitConfig,

    /// Validation rules and quality gates configuration
    pub validation: ValidationConfig,

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
            git: GitConfig::default(),
            validation: ValidationConfig::default(),
            environments: vec![
                Environment::Development,
                Environment::Staging,
                Environment::Production,
            ],
        }
    }
}

/// Deployment environment
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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