//! Hook manager type definitions
//!
//! Follows direct borrowing patterns instead of trait objects.

use super::{HookDefinition, HookInstaller, HookType, HookValidator};
use crate::events::EventBus;
use crate::core::MonorepoPackageInfo;
use crate::config::MonorepoConfig;
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::FileSystemManager;
use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;

/// Central manager for Git hook installation, execution, and validation
/// 
/// Uses direct borrowing from MonorepoProject components instead of trait objects.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct HookManager<'a> {
    /// Hook installer for setting up Git hooks
    pub(crate) installer: HookInstaller<'a>,

    /// Hook validator for checking conditions and requirements
    pub(crate) validator: HookValidator<'a>,

    /// Registry of custom hook definitions
    pub(crate) custom_hooks: HashMap<HookType, HookDefinition>,
    
    /// Default hook definitions (owned instead of static)
    pub(crate) default_hooks: HashMap<HookType, HookDefinition>,

    /// Whether hooks are currently enabled
    pub(crate) enabled: bool,

    /// Event bus for communicating with other components
    pub(crate) event_bus: Option<Arc<EventBus>>,

    /// Direct reference to configuration
    pub(crate) config: &'a MonorepoConfig,

    /// Direct reference to git repository
    pub(crate) repository: &'a Repo,

    /// Direct reference to file system manager
    pub(crate) file_system: &'a FileSystemManager,

    /// Direct reference to packages
    pub(crate) packages: &'a [MonorepoPackageInfo],

    /// Direct reference to root path
    pub(crate) root_path: &'a Path,

    /// Synchronous task executor for hook validation
    pub(crate) sync_task_executor: crate::hooks::sync_task_executor::SyncTaskExecutor<'a>,
}