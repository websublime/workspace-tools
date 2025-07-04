//! Monorepo project type definitions
//!
//! This module defines the MonorepoProject structure that directly uses base crates
//! for CLI/daemon consumption. Removes service abstraction layers for direct access patterns.

/// Main project structure for CLI and daemon consumption
///
/// This structure uses direct access patterns with base crates to provide
/// simple, focused functionality for monorepo operations. Eliminates service
/// abstractions in favor of direct base crate usage.
///
/// # Architecture
///
/// The project directly uses base crates:
/// - `sublime_standard_tools`: File system operations and monorepo detection
/// - `sublime_git_tools`: Git repository operations  
/// - `sublime_package_tools`: Package management and dependencies
///
/// # CLI/Daemon Focus
///
/// Designed for CLI and daemon consumption with direct borrowing patterns
/// and minimal abstractions for optimal performance.
pub struct MonorepoProject {
    /// All packages in the monorepo with enhanced information  
    pub(crate) packages: Vec<super::MonorepoPackageInfo>,

    /// Direct access to root path (pub(crate) field)
    pub(crate) root_path: std::path::PathBuf,

    /// Direct access to configuration (pub(crate) field)
    pub(crate) config: crate::config::MonorepoConfig,

    /// Direct access to file system manager (pub(crate) field)
    pub(crate) file_system: sublime_standard_tools::filesystem::FileSystemManager,

    /// Direct access to repository (pub(crate) field)
    pub(crate) repository: sublime_git_tools::Repo,
}
