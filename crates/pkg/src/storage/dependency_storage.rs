//! # Dependency Registry Facade
//!
//! Enterprise-grade dependency registry implementing facade pattern with SRP-compliant services.
//!
//! ## Architecture Overview
//!
//! The `Registry` serves as a unified facade over three specialized services:
//! - **DependencyStorage**: Thread-safe dependency data management
//! - **PackageRegistryClient**: External registry communication
//! - **ConflictResolver**: Intelligent version conflict resolution
//!
//! This design ensures separation of concerns while maintaining a simple, consistent API
//! for dependency management operations.
//!
//! ## Key Features
//!
//! - **Thread Safety**: All operations are safe for concurrent access
//! - **Registry Integration**: Optional external registry support (npm, yarn, etc.)
//! - **Conflict Resolution**: Advanced algorithms for version conflict resolution
//! - **Atomic Operations**: Batch updates with rollback capabilities
//! - **Enterprise Patterns**: Follows enterprise architecture principles
//!
//! ## Usage Examples
//!
//! ```rust
//! use sublime_package_tools::{Dependency, Registry, Package};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a registry
//! let mut registry = Registry::new();
//!
//! // Get dependencies from the registry
//! let react_dep = registry.get_or_create("react", "^17.0.2")?;
//! let router_dep = registry.get_or_create("react-router", "^6.0.0")?;
//!
//! // Create a package using these dependencies
//! let pkg = Package::new_with_registry(
//!     "my-app",
//!     "1.0.0",
//!     Some(vec![("react", "^17.0.2"), ("react-router", "^6.0.0")]),
//!     &mut registry
//! )?;
//!
//! // The dependencies in the package will be consistently managed
//! # Ok(())
//! # }
//! ```

use crate::{
    dependency::{
        storage::DependencyStorage,
        conflict_resolver::ConflictResolver,
        resolution::ResolutionResult,
    },
    external::{
        package_registry_client::PackageRegistryClient,
        npm_client::PackageRegistryClone,
    },
    errors::{PackageRegistryError, VersionError},
    Dependency,
};
use semver::VersionReq;

/// Enterprise-grade dependency registry facade with SRP-compliant architecture
///
/// The `Registry` serves as a unified interface over three specialized services,
/// providing a clean API while ensuring separation of concerns and maintainability.
///
/// # Architecture Components
///
/// - **DependencyStorage**: Thread-safe storage service for dependency data
/// - **PackageRegistryClient**: External registry communication service  
/// - **ConflictResolver**: Business logic service for conflict resolution
///
/// # Thread Safety
///
/// All operations are thread-safe through service composition:
/// - Storage uses Arc<RwLock<HashMap>> for concurrent access
/// - Registry client uses Arc<> for safe sharing
/// - Conflict resolver coordinates both services safely
///
/// # Examples
///
/// ```rust
/// use sublime_package_tools::{Registry, Package};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut registry = Registry::new();
///
/// // When we create packages with the same dependencies
/// let pkg1 = Package::new_with_registry(
///     "pkg1",
///     "1.0.0",
///     Some(vec![("react", "^17.0.0")]),
///     &mut registry
/// )?;
///
/// let pkg2 = Package::new_with_registry(
///     "pkg2",
///     "1.0.0",
///     Some(vec![("react", "^17.0.0")]),
///     &mut registry
/// )?;
///
/// // The registry ensures consistent dependency management
/// # Ok(())
/// # }
/// ```
///
/// ## With Package Registry
///
/// ```rust
/// use sublime_package_tools::{Registry, NpmRegistry};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create registry with npm package registry for enhanced version resolution
/// let npm_registry = NpmRegistry::new("https://registry.npmjs.org".to_string());
/// let mut dependency_registry = Registry::with_package_registry(
///     Box::new(npm_registry)
/// );
///
/// // Now the registry can query npm for available versions when resolving conflicts
/// let react_dep = dependency_registry.get_or_create("react", "^17.0.0")?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Registry {
    /// Storage service for dependency data management
    ///
    /// Provides thread-safe access to dependency data with intelligent
    /// version resolution and atomic batch operations.
    storage: DependencyStorage,
    
    /// Registry client for external package queries
    ///
    /// Enables enhanced resolution using real-world package data,
    /// version availability, and compatibility information.
    registry_client: PackageRegistryClient,
    
    /// Conflict resolver for business logic operations
    ///
    /// Encapsulates all complex algorithms for resolving dependency conflicts
    /// while maintaining clear separation from data and network concerns.
    conflict_resolver: ConflictResolver,
}

impl Clone for Registry {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            registry_client: self.registry_client.clone(),
            conflict_resolver: self.conflict_resolver.clone_resolver(),
        }
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    /// Creates a new dependency registry with default services
    ///
    /// Initializes the registry with:
    /// - Empty dependency storage
    /// - Registry client without external registry
    /// - Conflict resolver with local-only capabilities
    ///
    /// # Returns
    ///
    /// A new Registry ready for dependency management operations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::Registry;
    ///
    /// let registry = Registry::new();
    /// assert!(!registry.has_package_registry());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        let storage = DependencyStorage::new();
        let registry_client = PackageRegistryClient::new();
        let conflict_resolver = ConflictResolver::new(storage.clone(), registry_client.clone());
        
        Self {
            storage,
            registry_client,
            conflict_resolver,
        }
    }

    /// Creates a new dependency registry with external package registry capabilities
    ///
    /// When a package registry is provided, the registry can query external sources
    /// (like npm, yarn, etc.) for enhanced version resolution and conflict detection.
    ///
    /// # Arguments
    ///
    /// * `package_registry` - A boxed package registry implementation for querying external sources
    ///
    /// # Returns
    ///
    /// A new Registry with enhanced resolution capabilities
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{Registry, NpmRegistry};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let npm_registry = NpmRegistry::new("https://registry.npmjs.org".to_string());
    /// let registry = Registry::with_package_registry(Box::new(npm_registry));
    /// assert!(registry.has_package_registry());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn with_package_registry(package_registry: Box<dyn PackageRegistryClone>) -> Self {
        let storage = DependencyStorage::new();
        let registry_client = PackageRegistryClient::with_registry(package_registry);
        let conflict_resolver = ConflictResolver::new(storage.clone(), registry_client.clone());
        
        Self {
            storage,
            registry_client,
            conflict_resolver,
        }
    }

    /// Sets the package registry for enhanced version resolution capabilities
    ///
    /// This allows adding external registry functionality to an existing registry,
    /// enabling enhanced conflict resolution and version queries.
    ///
    /// # Arguments
    ///
    /// * `package_registry` - A boxed package registry implementation for querying external sources
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{Registry, NpmRegistry};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = Registry::new();
    /// let npm_registry = NpmRegistry::new("https://registry.npmjs.org".to_string());
    /// registry.set_package_registry(Box::new(npm_registry));
    /// assert!(registry.has_package_registry());
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_package_registry(&mut self, package_registry: Box<dyn PackageRegistryClone>) {
        // Update the registry client with the new package registry
        self.registry_client.set_registry(package_registry);
        
        // Create a new conflict resolver with the updated registry client
        self.conflict_resolver = ConflictResolver::new(
            self.storage.clone(),
            self.registry_client.clone(),
        );
    }

    /// Gets an existing dependency or creates a new one using storage service
    ///
    /// This method delegates to the DependencyStorage service, which handles
    /// intelligent version resolution when conflicts occur.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the dependency
    /// * `version` - The version requirement string
    ///
    /// # Returns
    ///
    /// A `Dependency` instance from storage
    ///
    /// # Errors
    ///
    /// Returns a `VersionError` if the version string is invalid or storage operation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::Registry;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = Registry::new();
    ///
    /// // First call creates a new dependency
    /// let dep1 = registry.get_or_create("react", "^17.0.2")?;
    /// assert_eq!(dep1.name(), "react");
    /// assert_eq!(dep1.version().to_string(), "^17.0.2");
    ///
    /// // Second call returns existing or intelligently resolved dependency
    /// let dep2 = registry.get_or_create("react", "^17.0.2")?;
    /// assert_eq!(dep2.name(), "react");
    ///
    /// // With a higher version, storage service handles resolution
    /// let dep3 = registry.get_or_create("react", "^18.0.0")?;
    /// assert_eq!(dep3.version().to_string(), "^18.0.0");
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_or_create(&mut self, name: &str, version: &str) -> Result<Dependency, VersionError> {
        self.storage.get_or_insert(name, version)
    }

    /// Gets an existing dependency by name from storage service
    ///
    /// This method delegates to the DependencyStorage service for consistent
    /// dependency retrieval across the system.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the dependency to retrieve
    ///
    /// # Returns
    ///
    /// A `Dependency` instance if found, or `None` if not found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::Registry;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = Registry::new();
    ///
    /// // Add a dependency
    /// registry.get_or_create("react", "^17.0.2")?;
    ///
    /// // Retrieve the dependency
    /// if let Some(dep) = registry.get("react") {
    ///     println!("Found: {}", dep.name());
    /// }
    ///
    /// // Non-existent dependency
    /// assert!(registry.get("missing").is_none());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn get(&self, name: &str) -> Option<Dependency> {
        self.storage.get(name)
    }

    /// Resolves version conflicts using advanced conflict resolution algorithms
    ///
    /// This method delegates to the ConflictResolver service, which implements
    /// sophisticated algorithms for finding optimal solutions to version conflicts.
    ///
    /// # Returns
    ///
    /// A `ResolutionResult` containing resolved versions and required updates
    ///
    /// # Errors
    ///
    /// Returns `VersionError` if conflict resolution fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::Registry;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = Registry::new();
    ///
    /// // Add dependencies with potential conflicts
    /// registry.get_or_create("lodash", "^4.17.0")?;
    /// registry.get_or_create("lodash", "^4.17.21")?;
    ///
    /// // Resolve conflicts using advanced algorithms
    /// let result = registry.resolve_version_conflicts().await?;
    ///
    /// // Check resolved versions
    /// if let Some(version) = result.resolved_versions.get("lodash") {
    ///     println!("Resolved lodash to: {}", version);
    /// }
    ///
    /// // Check updates needed
    /// for update in result.updates_required {
    ///     println!("Update required: {} from {} to {}",
    ///              update.dependency_name,
    ///              update.current_version,
    ///              update.new_version);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn resolve_version_conflicts(&self) -> Result<ResolutionResult, VersionError> {
        self.conflict_resolver.resolve_version_conflicts().await
    }

    /// Gets all versions of a package from external registry using registry client
    ///
    /// This method delegates to the PackageRegistryClient service for external
    /// package version queries.
    ///
    /// # Arguments
    ///
    /// * `package_name` - The name of the package to query
    ///
    /// # Returns
    ///
    /// A list of available versions or an error if the query fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{Registry, NpmRegistry};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let npm_registry = NpmRegistry::new("https://registry.npmjs.org".to_string());
    /// let registry = Registry::with_package_registry(Box::new(npm_registry));
    ///
    /// if let Ok(versions) = registry.get_package_versions("react").await {
    ///     println!("Found {} versions of react", versions.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_package_versions(
        &self,
        package_name: &str,
    ) -> Result<Vec<String>, PackageRegistryError> {
        self.registry_client.get_package_versions(package_name).await
    }

    /// Checks if the registry has external package registry capabilities
    ///
    /// This method delegates to the PackageRegistryClient service to determine
    /// if external registry functionality is available.
    ///
    /// # Returns
    ///
    /// `true` if external registry is configured, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{Registry, NpmRegistry};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = Registry::new();
    /// assert!(!registry.has_package_registry());
    ///
    /// let npm_registry = NpmRegistry::new("https://registry.npmjs.org".to_string());
    /// registry.set_package_registry(Box::new(npm_registry));
    /// assert!(registry.has_package_registry());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn has_package_registry(&self) -> bool {
        self.registry_client.has_registry()
    }

    /// Find highest version that is compatible with all requirements.
    ///
    /// This method first checks existing dependencies in the registry, then falls back
    /// to querying the package registry (if available) to find all available versions
    /// and select the highest one that satisfies all requirements.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the dependency
    /// * `requirements` - A list of version requirements to satisfy
    ///
    /// # Returns
    ///
    /// A string representing the highest version that satisfies all requirements.
    ///
    /// # Errors
    ///
    /// Returns `PackageRegistryError` if querying the package registry fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Registry, Dependency};
    /// use semver::VersionReq;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = Registry::new();
    /// registry.get_or_create("react", "^17.0.0")?;
    ///
    /// // Find a version compatible with both requirements
    /// let req1 = VersionReq::parse("^17.0.0")?;
    /// let req2 = VersionReq::parse("^17.0.2")?;
    /// let version = registry.find_highest_compatible_version(
    ///     "react",
    ///     &[&req1, &req2]
    /// )?;
    ///
    /// // The method will return the highest compatible version
    /// assert!(!version.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## With Package Registry
    ///
    /// ```no_run
    /// use sublime_package_tools::{Registry, NpmRegistry};
    /// use semver::VersionReq;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create registry with npm for enhanced resolution
    /// let npm_registry = NpmRegistry::new("https://registry.npmjs.org".to_string());
    /// let registry = Registry::with_package_registry(Box::new(npm_registry));
    ///
    /// let req = VersionReq::parse("^17.0.0")?;
    ///
    /// // This will query npm to find the highest version matching the requirement
    /// let version = registry.find_highest_compatible_version("react", &[&req])?;
    /// println!("Found compatible version: {}", version);
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::unnecessary_wraps)]
    pub async fn find_highest_compatible_version(
        &self,
        name: &str,
        requirements: &[&VersionReq],
    ) -> Result<String, PackageRegistryError> {
        self.conflict_resolver.find_highest_compatible_version(name, requirements).await
    }

    /// Applies resolution result updates using conflict resolver service
    ///
    /// This method delegates to the ConflictResolver service, which ensures
    /// atomic application of all updates with rollback capabilities.
    ///
    /// # Arguments
    ///
    /// * `result` - The resolution result containing resolved versions and updates
    ///
    /// # Returns
    ///
    /// `Ok(())` if all updates applied successfully
    ///
    /// # Errors
    ///
    /// Returns `VersionError` if any update fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::{Registry, Update, ResolutionResult};
    /// use std::collections::HashMap;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = Registry::new();
    /// registry.get_or_create("react", "^16.0.0")?;
    ///
    /// // First resolve conflicts
    /// let result = registry.resolve_version_conflicts().await?;
    ///
    /// // Apply updates atomically
    /// registry.apply_resolution_result(&result).await?;
    ///
    /// // Verify updates were applied
    /// if let Some(dep) = registry.get("react") {
    ///     println!("Updated to: {}", dep.version());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn apply_resolution_result(
        &mut self,
        result: &ResolutionResult,
    ) -> Result<(), VersionError> {
        self.conflict_resolver.apply_resolution(result).await
    }
}

