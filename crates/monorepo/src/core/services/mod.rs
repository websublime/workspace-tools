//! Core services for monorepo operations
//!
//! This module contains the service layer that breaks down the MonorepoProject
//! god object into focused, single-responsibility services while maintaining
//! backward compatibility through the facade pattern.

pub mod config_service;
pub mod file_system_service;
pub mod git_service;
pub mod package_service;
pub mod dependency_service;

pub use config_service::ConfigurationService;
pub use file_system_service::FileSystemService;
pub use git_service::GitOperationsService;
pub use package_service::PackageDiscoveryService;
pub use dependency_service::DependencyAnalysisService;

use crate::error::Result;
use std::path::Path;

/// Container for all monorepo services
///
/// This structure aggregates all the focused services that were previously
/// handled by the MonorepoProject god object. It follows the Service Container
/// pattern to provide centralized access to all services while maintaining
/// separation of concerns.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::core::services::MonorepoServices;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let services = MonorepoServices::new("/path/to/monorepo")?;
/// 
/// // Access specific services
/// let packages = services.package_service().discover_packages()?;
/// let config = services.config_service().get_configuration();
/// let repo_status = services.git_service().get_status()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct MonorepoServices {
    /// Configuration management service
    config_service: ConfigurationService,
    
    /// File system operations service
    file_system_service: FileSystemService,
    
    /// Git operations service
    git_service: GitOperationsService,
    
    /// Package discovery and management service
    package_service: PackageDiscoveryService,
    
    /// Dependency analysis service
    dependency_service: DependencyAnalysisService,
}

impl MonorepoServices {
    /// Create a new service container for the monorepo
    ///
    /// Initializes all services with the provided root path and ensures
    /// they are properly configured and ready for use.
    ///
    /// # Arguments
    ///
    /// * `root_path` - Root path of the monorepo
    ///
    /// # Returns
    ///
    /// A new service container with all services initialized.
    ///
    /// # Errors
    ///
    /// Returns an error if any service cannot be initialized due to:
    /// - Invalid root path
    /// - Missing configuration files
    /// - Git repository not found
    /// - File system access issues
    pub fn new<P: AsRef<Path>>(root_path: P) -> Result<Self> {
        let root_path = root_path.as_ref();
        
        // Initialize services in dependency order
        let file_system_service = FileSystemService::new(root_path)?;
        let config_service = ConfigurationService::new(root_path, &file_system_service)?;
        let git_service = GitOperationsService::new(root_path)?;
        let package_service = PackageDiscoveryService::new(
            root_path, 
            &file_system_service,
            config_service.get_configuration()
        )?;
        let dependency_service = DependencyAnalysisService::new(
            &package_service,
            config_service.get_configuration()
        )?;
        
        Ok(Self {
            config_service,
            file_system_service,
            git_service,
            package_service,
            dependency_service,
        })
    }
    
    /// Get the configuration service
    ///
    /// Provides access to configuration management operations including
    /// loading, validating, and accessing monorepo configuration settings.
    ///
    /// # Returns
    ///
    /// Reference to the configuration service.
    pub fn config_service(&self) -> &ConfigurationService {
        &self.config_service
    }
    
    /// Get the file system service
    ///
    /// Provides access to file system operations with proper error handling
    /// and path resolution for monorepo contexts.
    ///
    /// # Returns
    ///
    /// Reference to the file system service.
    pub fn file_system_service(&self) -> &FileSystemService {
        &self.file_system_service
    }
    
    /// Get the Git operations service
    ///
    /// Provides access to Git repository operations including status checking,
    /// commit history analysis, and branch management.
    ///
    /// # Returns
    ///
    /// Reference to the Git operations service.
    pub fn git_service(&self) -> &GitOperationsService {
        &self.git_service
    }
    
    /// Get the package discovery service
    ///
    /// Provides access to package discovery, metadata parsing, and package
    /// relationship analysis within the monorepo.
    ///
    /// # Returns
    ///
    /// Reference to the package discovery service.
    pub fn package_service(&self) -> &PackageDiscoveryService {
        &self.package_service
    }
    
    /// Get the dependency analysis service
    ///
    /// Provides access to dependency graph analysis, conflict detection,
    /// and dependency relationship mapping.
    ///
    /// # Returns
    ///
    /// Reference to the dependency analysis service.
    pub fn dependency_service(&self) -> &DependencyAnalysisService {
        &self.dependency_service
    }

    /// Get mutable access to the dependency analysis service
    ///
    /// Required for operations that modify the dependency graph state.
    /// This breaks the immutable service pattern but is needed for backward compatibility.
    ///
    /// # Returns
    ///
    /// Mutable reference to the dependency analysis service.
    pub fn dependency_service_mut(&mut self) -> &mut DependencyAnalysisService {
        &mut self.dependency_service
    }

    /// Get mutable access to the package discovery service
    ///
    /// Required for operations that refresh package discovery state.
    /// This breaks the immutable service pattern but is needed for backward compatibility.
    ///
    /// # Returns
    ///
    /// Mutable reference to the package discovery service.
    pub fn package_service_mut(&mut self) -> &mut PackageDiscoveryService {
        &mut self.package_service
    }
}