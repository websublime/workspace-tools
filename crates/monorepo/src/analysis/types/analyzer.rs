//! Analyzer types for monorepo analysis - simplified for CLI consumption
//!
//! This module contains streamlined type definitions for the monorepo analyzer.
//! Focuses on essential CLI operations with direct base crate integration.

use crate::config::MonorepoConfig;
use crate::core::MonorepoPackageInfo;
use std::path::Path;
use sublime_standard_tools::filesystem::FileSystemManager;

/// Simplified analyzer for essential monorepo analysis
///
/// Streamlined for CLI consumption with direct base crate integration.
/// Focuses on dependency graph, change detection, and package classification.
pub struct MonorepoAnalyzer<'a> {
    /// Direct reference to packages for analysis
    pub(crate) packages: &'a [MonorepoPackageInfo],

    /// Direct reference to configuration for essential analysis
    pub(crate) config: &'a MonorepoConfig,

    /// Direct reference to file system for file operations
    pub(crate) file_system: &'a FileSystemManager,

    /// Direct reference to root path
    pub(crate) root_path: &'a Path,
}
