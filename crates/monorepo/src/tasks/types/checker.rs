//! Task condition checker type definitions

use crate::core::{GitProvider, ConfigProvider, PackageProvider, FileSystemProvider};

/// Checker for evaluating task execution conditions
pub struct ConditionChecker {
    /// Git operations provider
    pub(crate) git_provider: Box<dyn GitProvider>,
    
    /// Configuration provider
    pub(crate) config_provider: Box<dyn ConfigProvider>,
    
    /// Package operations provider
    pub(crate) package_provider: Box<dyn PackageProvider>,
    
    /// File system operations provider
    pub(crate) file_system_provider: Box<dyn FileSystemProvider>,
}