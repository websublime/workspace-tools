//! Git hooks configuration types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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