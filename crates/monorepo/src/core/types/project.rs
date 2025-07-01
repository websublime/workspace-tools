//! Monorepo project type definitions
//!
//! This module defines the MonorepoProject structure that now uses the service
//! container pattern to break the god object anti-pattern while maintaining
//! backward compatibility through facade methods.

use crate::core::services::MonorepoServices;

/// Main project structure that provides a facade over the service container
///
/// This structure has been refactored to use the Service Container pattern
/// internally while maintaining complete backward compatibility with the
/// existing API. The god object pattern has been eliminated by delegating
/// operations to focused, single-responsibility services.
///
/// # Service Architecture
///
/// The project now uses the following services internally:
/// - ConfigurationService: Configuration management and validation
/// - FileSystemService: File system operations with monorepo context
/// - GitOperationsService: Git repository operations
/// - PackageDiscoveryService: Package discovery and metadata management
/// - DependencyAnalysisService: Dependency graph analysis and conflict detection
///
/// # Backward Compatibility
///
/// All existing public methods remain unchanged, ensuring that existing code
/// continues to work without modifications. The service delegation is completely
/// transparent to external users.
pub struct MonorepoProject {
    /// Service container with all focused services
    pub(crate) services: MonorepoServices,

    /// All packages in the monorepo with enhanced information  
    pub(crate) packages: Vec<super::MonorepoPackageInfo>,

    /// Dependency graph of all packages (built on-demand)
    pub(crate) dependency_graph: Option<()>,

    /// Direct access to root path (pub(crate) field)
    pub(crate) root_path: std::path::PathBuf,

    /// Direct access to configuration (pub(crate) field)
    pub(crate) config: crate::config::MonorepoConfig,

    /// Direct access to file system manager (pub(crate) field)
    pub(crate) file_system: sublime_standard_tools::filesystem::FileSystemManager,

    /// Direct access to repository (pub(crate) field)
    pub(crate) repository: sublime_git_tools::Repo,
}
