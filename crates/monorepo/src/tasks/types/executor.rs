//! Task executor type definitions

use crate::core::{PackageProvider, ConfigProvider};

/// Executor for running tasks with various scopes and configurations
pub struct TaskExecutor {
    /// Package operations provider
    pub(crate) package_provider: Box<dyn PackageProvider>,
    
    /// Configuration provider
    pub(crate) config_provider: Box<dyn ConfigProvider>,
}