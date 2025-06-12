//! Hook manager type definitions

use super::{HookDefinition, HookInstaller, HookType, HookValidator};
use crate::core::MonorepoProject;
use std::collections::HashMap;
use std::sync::Arc;

/// Central manager for Git hook installation, execution, and validation
pub struct HookManager {
    /// Reference to the monorepo project
    #[allow(dead_code)] // Will be used when full integration is implemented
    pub(crate) project: Arc<MonorepoProject>,

    /// Hook installer for setting up Git hooks
    pub(crate) installer: HookInstaller,

    /// Hook validator for checking conditions and requirements
    pub(crate) validator: HookValidator,

    /// Registry of custom hook definitions
    pub(crate) custom_hooks: HashMap<HookType, HookDefinition>,

    /// Whether hooks are currently enabled
    pub(crate) enabled: bool,
}