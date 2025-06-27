//! Hook manager type definitions

use super::{HookDefinition, HookInstaller, HookType, HookValidator};
use crate::events::EventBus;
use std::collections::HashMap;
use std::sync::Arc;

/// Central manager for Git hook installation, execution, and validation
pub struct HookManager {
    /// Hook installer for setting up Git hooks
    pub(crate) installer: HookInstaller,

    /// Hook validator for checking conditions and requirements
    pub(crate) validator: HookValidator,

    /// Registry of custom hook definitions
    pub(crate) custom_hooks: HashMap<HookType, HookDefinition>,
    
    /// Default hook definitions (owned instead of static)
    pub(crate) default_hooks: HashMap<HookType, HookDefinition>,

    /// Whether hooks are currently enabled
    pub(crate) enabled: bool,

    /// Event bus for communicating with other components
    pub(crate) event_bus: Option<Arc<EventBus>>,

    /// Configuration provider for accessing configuration settings
    pub(crate) config_provider: Box<dyn crate::core::ConfigProvider>,

    /// Git provider for repository operations
    pub(crate) git_provider: Box<dyn crate::core::GitProvider>,

    /// File system provider for file operations
    pub(crate) file_system_provider: Box<dyn crate::core::FileSystemProvider>,

    /// Package provider for package discovery and management
    pub(crate) package_provider: Box<dyn crate::core::PackageProvider>,

    /// Synchronous task executor for hook validation
    pub(crate) sync_task_executor: crate::hooks::sync_task_executor::SyncTaskExecutor,
}