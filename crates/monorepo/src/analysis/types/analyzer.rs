//! Analyzer types for monorepo analysis
//!
//! This module contains type definitions for the monorepo analyzer.

use crate::core::interfaces::{WorkspaceProvider, PackageDiscoveryProvider, EnhancedConfigProvider};
use crate::core::MonorepoProject;
use std::sync::Arc;

/// Analyzer for comprehensive monorepo analysis
pub struct MonorepoAnalyzer {
    /// Package provider for accessing package information
    pub(crate) package_provider: Box<dyn crate::core::PackageProvider>,

    /// Configuration provider for accessing configuration settings
    pub(crate) config_provider: Box<dyn crate::core::ConfigProvider>,

    /// File system provider for file operations
    pub(crate) file_system_provider: Box<dyn crate::core::FileSystemProvider>,

    /// Git provider for repository operations
    pub(crate) git_provider: Box<dyn crate::core::GitProvider>,

    /// Registry provider for package registry operations
    pub(crate) registry_provider: Box<dyn crate::core::RegistryProvider>,
    
    /// Workspace provider for workspace patterns and configuration
    pub(crate) workspace_provider: Box<dyn WorkspaceProvider>,
    
    /// Package discovery provider for comprehensive package metadata
    pub(crate) package_discovery_provider: Box<dyn PackageDiscoveryProvider>,
    
    /// Enhanced configuration provider for workspace management
    pub(crate) enhanced_config_provider: Box<dyn EnhancedConfigProvider>,
    
    /// Optional reference to the source project for creating derived analyzers
    pub(crate) source_project: Option<Arc<MonorepoProject>>,
}