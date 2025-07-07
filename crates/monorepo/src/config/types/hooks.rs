//! Git hooks configuration types
//!
//! This module provides configuration for Git hooks with support for both native Git hooks
//! and Husky integration. The system can automatically detect the best hook strategy
//! based on the project setup (presence of Husky, package.json, etc.) or be manually configured.
//!
//! ## What
//! Provides comprehensive hook configuration including:
//! - Native Git hooks generation (.git/hooks/)
//! - Husky integration (.husky/ directory)
//! - Automatic detection and intelligent defaults
//! - Hybrid configurations for complex projects
//! - Task execution and changeset validation
//!
//! ## How
//! Uses strategy pattern to determine hook implementation:
//! - Auto: Automatically detects and chooses best strategy
//! - Native: Uses traditional Git hooks in .git/hooks/
//! - Husky: Integrates with Husky for Node.js projects
//! - Hybrid: Uses both systems for maximum compatibility
//!
//! ## Why
//! Enables seamless integration with existing developer workflows while providing
//! flexibility for different project types and team preferences.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Git hooks configuration with Husky support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    /// Whether hooks are enabled
    pub enabled: bool,

    /// Hook strategy to use (auto-detection, native Git, Husky, or hybrid)
    #[serde(default)]
    pub strategy: HookStrategy,

    /// Pre-commit hook configuration
    pub pre_commit: HookConfig,

    /// Pre-push hook configuration
    pub pre_push: HookConfig,

    /// Post-merge hook configuration
    pub post_merge: Option<HookConfig>,

    /// Custom hooks directory (for native Git hooks)
    pub hooks_dir: Option<PathBuf>,

    /// Husky-specific configuration
    #[serde(default)]
    pub husky: HuskyConfig,

    /// Auto-detection settings
    #[serde(default)]
    pub auto_detection: AutoDetectionConfig,
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strategy: HookStrategy::Auto,
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
            husky: HuskyConfig::default(),
            auto_detection: AutoDetectionConfig::default(),
        }
    }
}

/// Hook implementation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HookStrategy {
    /// Automatically detect and choose the best strategy
    Auto,
    /// Use native Git hooks (.git/hooks/)
    Native,
    /// Use Husky (.husky/ directory)
    Husky,
    /// Use both systems (hybrid approach)
    Hybrid,
}

impl Default for HookStrategy {
    fn default() -> Self {
        Self::Auto
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

/// Husky-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuskyConfig {
    /// Husky directory path (default: .husky)
    pub husky_dir: PathBuf,

    /// Whether to use npm/yarn/pnpm scripts instead of direct commands
    pub use_package_scripts: bool,

    /// Package manager to use for script execution (npm, yarn, pnpm, bun)
    pub package_manager: Option<String>,

    /// Additional arguments to pass to package manager
    pub package_manager_args: Vec<String>,

    /// Whether to generate husky install command in package.json prepare script
    pub auto_install: bool,

    /// Custom husky initialization command
    pub init_command: Option<String>,
}

impl Default for HuskyConfig {
    fn default() -> Self {
        Self {
            husky_dir: PathBuf::from(".husky"),
            use_package_scripts: true,
            package_manager: None, // Auto-detect
            package_manager_args: Vec::new(),
            auto_install: true,
            init_command: None,
        }
    }
}

/// Auto-detection configuration for hook strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoDetectionConfig {
    /// Whether to enable auto-detection of hook strategy
    pub enabled: bool,

    /// Prefer Husky if both Husky and native Git capabilities are detected
    pub prefer_husky: bool,

    /// Whether to check package.json for Husky configuration
    pub check_package_json: bool,

    /// Whether to check for .husky directory
    pub check_husky_dir: bool,

    /// Whether to check for existing Git hooks
    pub check_git_hooks: bool,

    /// File patterns that indicate Node.js project (triggers Husky preference)
    pub nodejs_indicators: Vec<String>,
}

impl Default for AutoDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefer_husky: true,
            check_package_json: true,
            check_husky_dir: true,
            check_git_hooks: true,
            nodejs_indicators: vec![
                "package.json".to_string(),
                "yarn.lock".to_string(),
                "package-lock.json".to_string(),
                "pnpm-lock.yaml".to_string(),
                "bun.lockb".to_string(),
                "node_modules".to_string(),
            ],
        }
    }
}
