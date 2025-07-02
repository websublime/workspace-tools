//! Version manager type definitions

use super::VersioningStrategy;
use crate::config::MonorepoConfig;
use crate::core::MonorepoPackageInfo;

/// Manager for package versioning with dependency propagation
/// 
/// Uses direct borrowing from MonorepoProject components instead of Arc.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct VersionManager<'a> {
    /// Direct reference to configuration
    pub(crate) config: &'a MonorepoConfig,
    /// Direct reference to packages
    pub(crate) packages: &'a [MonorepoPackageInfo],
    /// Direct reference to root path
    pub(crate) root_path: &'a std::path::Path,
    /// Versioning strategy to use
    pub(crate) strategy: Box<dyn VersioningStrategy + 'a>,
}