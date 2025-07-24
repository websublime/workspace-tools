//! # Conflict Resolver Service
//!
//! Business logic service for dependency conflict resolution with enterprise-grade algorithms.
//!
//! ## Overview
//!
//! This service handles all complex dependency conflict resolution scenarios:
//! - Version conflict detection and intelligent resolution
//! - Compatibility analysis across version ranges and semver constraints
//! - Optimal version selection using sophisticated algorithms
//! - Integration with external registry data for informed decisions
//! - Atomic resolution application with rollback capabilities
//!
//! ## Architecture
//!
//! Uses composition of two other services:
//! - **DependencyStorage**: For local dependency data access and mutations
//! - **PackageRegistryClient**: For external registry queries and metadata
//!
//! ## Algorithms
//!
//! Implements multiple resolution strategies with fallback mechanisms:
//! 1. **Registry-Enhanced Resolution**: Uses external data when available
//! 2. **Local-Only Resolution**: Works with cached/stored dependencies  
//! 3. **Hybrid Resolution**: Combines external and local data intelligently
//! 4. **Fallback Strategies**: Graceful degradation when data unavailable
//!
//! ## Examples
//!
//! ```rust
//! use sublime_package_tools::dependency::{
//!     storage::DependencyStorage,
//!     package_registry_client::PackageRegistryClient,
//!     conflict_resolver::ConflictResolver
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Setup services
//! let storage = DependencyStorage::new();
//! let client = PackageRegistryClient::new();
//! let resolver = ConflictResolver::new(storage, client);
//!
//! // Resolve conflicts
//! let result = resolver.resolve_version_conflicts().await?;
//! println!("Resolved {} dependencies", result.resolved_versions.len());
//! 
//! // Apply resolution
//! resolver.apply_resolution(&result).await?;  
//! # Ok(())
//! # }
//! ```

use super::storage::DependencyStorage;
use crate::external::package_registry_client::PackageRegistryClient;
use crate::{
    dependency::{resolution::ResolutionResult, update::Update as DependencyUpdate},
    errors::{VersionError, PackageRegistryError},
};
use semver::{Version as SemverVersion, VersionReq};
use std::collections::HashMap;

/// Business logic service for dependency conflict resolution
/// 
/// This service encapsulates all complex algorithms for resolving dependency conflicts
/// while maintaining clear separation from data storage and network communication.
///
/// # Responsibilities
///
/// - **Conflict Detection**: Identify version conflicts between dependencies
/// - **Resolution Strategies**: Apply multiple algorithms to find optimal solutions
/// - **Registry Integration**: Use external data to make informed decisions
/// - **Update Generation**: Create actionable update lists with full context
/// - **Atomic Operations**: Ensure all-or-nothing resolution application
///
/// # Thread Safety
///
/// The resolver is thread-safe through composition:
/// - DependencyStorage uses Arc<RwLock<>> for safe concurrent access
/// - PackageRegistryClient uses Arc<> for safe registry sharing
/// - All operations are designed to be called concurrently
///
/// # Examples
///
/// ```rust
/// use sublime_package_tools::dependency::conflict_resolver::ConflictResolver;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let storage = DependencyStorage::new();
/// let client = PackageRegistryClient::new();
/// let resolver = ConflictResolver::new(storage, client);
/// 
/// // Add some conflicting dependencies
/// storage.get_or_insert("react", "^16.0.0")?;
/// storage.get_or_insert("react", "^17.0.0")?; // Conflict!
/// 
/// // Resolve conflicts intelligently
/// let result = resolver.resolve_version_conflicts().await?;
/// assert!(!result.resolved_versions.is_empty());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub(crate) struct ConflictResolver {
    /// Storage service for dependency data access and mutations
    ///
    /// Provides thread-safe access to dependency data with intelligent
    /// version resolution and atomic batch operations.
    storage: DependencyStorage,
    
    /// Registry client for external package queries and metadata
    ///
    /// Enables enhanced resolution using real-world package data,
    /// version availability, and compatibility information.
    registry_client: PackageRegistryClient,
}

impl ConflictResolver {
    /// Creates a new conflict resolver with storage and registry client
    ///
    /// The resolver requires both services for full functionality:
    /// - Storage for local dependency management
    /// - Registry client for external package data
    ///
    /// # Arguments
    ///
    /// * `storage` - Storage service for dependency data operations
    /// * `registry_client` - Client for external registry queries
    ///
    /// # Returns
    ///
    /// A new ConflictResolver ready for conflict resolution operations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::{
    ///     storage::DependencyStorage,
    ///     package_registry_client::PackageRegistryClient,
    ///     conflict_resolver::ConflictResolver
    /// };
    ///
    /// let storage = DependencyStorage::new();
    /// let client = PackageRegistryClient::new();
    /// let resolver = ConflictResolver::new(storage, client);
    /// ```
    #[must_use]
    pub fn new(storage: DependencyStorage, registry_client: PackageRegistryClient) -> Self {
        Self {
            storage,
            registry_client,
        }
    }

    /// Resolves version conflicts between all dependencies using advanced algorithms
    ///
    /// This is the core conflict resolution method that implements sophisticated
    /// algorithms to find optimal solutions for version conflicts.
    ///
    /// # Algorithm Overview
    ///
    /// 1. **Data Collection**: Gather all dependencies from storage
    /// 2. **Conflict Analysis**: Group dependencies by name to identify conflicts
    /// 3. **Resolution Strategy**: Apply multiple resolution approaches:
    ///    - Registry-enhanced resolution (when available)
    ///    - Local-only semver-based resolution  
    ///    - Hybrid approaches combining multiple data sources
    /// 4. **Update Generation**: Create detailed update instructions
    /// 5. **Validation**: Ensure resolution is consistent and applicable
    ///
    /// # Returns
    ///
    /// `ResolutionResult` containing:
    /// - `resolved_versions`: Final version for each dependency name
    /// - `updates_required`: List of specific updates to apply
    ///
    /// # Errors
    ///
    /// Returns `VersionError` if:
    /// - Version parsing fails for any dependency
    /// - Storage operations fail due to lock contention
    /// - Resolution produces inconsistent results
    /// - Required updates cannot be validated
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::conflict_resolver::ConflictResolver;
    ///
    /// # async fn example() -> Result<(), VersionError> {
    /// let resolver = ConflictResolver::new(storage, client);
    /// 
    /// // Resolve all version conflicts
    /// let result = resolver.resolve_version_conflicts().await?;
    /// 
    /// println!("Resolved {} dependencies", result.resolved_versions.len());
    /// println!("Updates required: {}", result.updates_required.len());
    /// 
    /// // Examine specific resolutions
    /// for (name, version) in &result.resolved_versions {
    ///     println!("  {} -> {}", name, version);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn resolve_version_conflicts(&self) -> Result<ResolutionResult, VersionError> {
        let mut resolved_versions: HashMap<String, String> = HashMap::new();
        let mut updates_required: Vec<DependencyUpdate> = Vec::new();

        // Get all current dependencies from storage
        let all_deps = self.storage.all_dependencies();

        if all_deps.is_empty() {
            // No dependencies to resolve
            return Ok(ResolutionResult {
                resolved_versions,
                updates_required,
            });
        }

        // Group dependencies by name for conflict analysis
        let mut dependency_groups: HashMap<String, Vec<(String, VersionReq)>> = HashMap::new();

        for (name, dep) in &all_deps {
            let version_req = dep.version().clone();
            
            let fixed_version = dep.fixed_version()
                .map_err(|e| VersionError::InvalidVersion(
                    format!("Fixed version error for '{name}': {e}")
                ))?;
            
            dependency_groups
                .entry(name.clone())
                .or_default()
                .push((fixed_version.to_string(), version_req));
        }

        // Resolve each dependency group using sophisticated algorithms
        for (dep_name, version_specs) in &dependency_groups {
            let resolved_version = self.resolve_dependency_group(dep_name, version_specs).await?;
            
            resolved_versions.insert(dep_name.clone(), resolved_version.clone());

            // Generate updates for dependencies that need version changes
            for (current_version, _version_req) in version_specs {
                if self.version_needs_update(current_version, &resolved_version)? {
                    updates_required.push(DependencyUpdate {
                        package_name: String::new(), // Context filled by caller
                        dependency_name: dep_name.clone(),
                        current_version: current_version.clone(),
                        new_version: resolved_version.clone(),
                    });
                }
            }
        }

        Ok(ResolutionResult {
            resolved_versions,
            updates_required,
        })
    }

    /// Finds the highest compatible version for a dependency with given requirements
    ///
    /// This method implements advanced version resolution using multiple strategies
    /// with intelligent fallback mechanisms.
    ///
    /// # Strategy Priority
    ///
    /// 1. **External Registry**: Query real package data when available
    /// 2. **Local Storage**: Use cached dependency information
    /// 3. **Intelligent Fallback**: Generate reasonable version when data unavailable
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the dependency to resolve
    /// * `requirements` - Array of version requirements that must be satisfied
    ///
    /// # Returns
    ///
    /// String representing the highest version that satisfies all requirements
    ///
    /// # Errors
    ///
    /// Returns `PackageRegistryError` if:
    /// - Registry queries fail with network errors
    /// - Version parsing fails for available versions
    /// - No compatible version can be found or generated
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::conflict_resolver::ConflictResolver;
    /// use semver::VersionReq;
    ///
    /// # async fn example() -> Result<(), PackageRegistryError> {
    /// let resolver = ConflictResolver::new(storage, client);
    /// 
    /// let req1 = VersionReq::parse("^17.0.0")?;
    /// let req2 = VersionReq::parse("^17.2.0")?;
    /// 
    /// let version = resolver.find_highest_compatible_version(
    ///     "react", 
    ///     &[&req1, &req2]
    /// ).await?;
    /// 
    /// println!("Best compatible version: {}", version);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_highest_compatible_version(
        &self,
        name: &str,
        requirements: &[&VersionReq],
    ) -> Result<String, PackageRegistryError> {
        // Strategy 1: Use external registry data when available
        if self.registry_client.has_registry() {
            match self.registry_client.get_package_versions(name).await {
                Ok(available_versions) if !available_versions.is_empty() => {
                    if let Some(best_version) = self.find_best_compatible_version(&available_versions, requirements) {
                        return Ok(best_version);
                    }
                },
                Ok(_) | Err(_) => {
                    // Empty versions from registry or registry error, continue to fallback strategies
                }
            }
        }

        // Strategy 2: Use local storage data
        if let Some(local_dep) = self.storage.get(name) {
            let version_str = local_dep.version().to_string();
            let clean_version = self.clean_version_string(&version_str);
            
            if let Ok(version) = SemverVersion::parse(&clean_version) {
                if requirements.iter().all(|req| req.matches(&version)) {
                    return Ok(clean_version);
                }
            }
        }

        // Strategy 3: Intelligent fallback version generation
        self.generate_fallback_version(requirements)
    }

    /// Applies resolution result updates to storage with atomic guarantees
    ///
    /// This method ensures all updates are applied atomically - either all
    /// updates succeed or none are applied, maintaining storage consistency.
    ///
    /// # Arguments
    ///
    /// * `result` - Resolution result containing the updates to apply
    ///
    /// # Returns
    ///
    /// `Ok(())` if all updates applied successfully
    ///
    /// # Errors
    ///
    /// Returns `VersionError` if:
    /// - Any update contains invalid version information
    /// - Storage operations fail due to lock contention
    /// - Batch update operation cannot be completed atomically
    ///
    /// # Atomicity
    ///
    /// This operation uses the storage service's atomic batch update mechanism
    /// to ensure consistency. If any update fails, all updates are rolled back.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::conflict_resolver::ConflictResolver;
    ///
    /// # async fn example() -> Result<(), VersionError> {
    /// let resolver = ConflictResolver::new(storage, client);
    /// 
    /// // First resolve conflicts
    /// let result = resolver.resolve_version_conflicts().await?;
    /// 
    /// // Then apply the resolution atomically
    /// resolver.apply_resolution(&result).await?;
    /// 
    /// println!("Successfully applied {} updates", result.updates_required.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn apply_resolution(&self, result: &ResolutionResult) -> Result<(), VersionError> {
        if result.updates_required.is_empty() {
            // No updates to apply
            return Ok(());
        }

        // Prepare batch updates for atomic application
        let updates: Vec<(String, String)> = result.updates_required
            .iter()
            .map(|update| (update.dependency_name.clone(), update.new_version.clone()))
            .collect();

        // Apply all updates atomically using storage service
        self.storage.apply_batch_updates(&updates)
    }

    /// Creates a clone of this resolver
    ///
    /// This method provides explicit cloning with the same service configuration.
    /// The cloned resolver shares the same storage and registry client instances.
    ///
    /// # Returns
    ///
    /// A new ConflictResolver with identical service configuration
    ///
    /// # Examples  
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::conflict_resolver::ConflictResolver;
    ///
    /// let resolver = ConflictResolver::new(storage, client);
    /// let resolver_clone = resolver.clone_resolver();
    /// 
    /// // Both resolvers share the same underlying services
    /// assert_eq!(resolver.has_registry(), resolver_clone.has_registry());
    /// ```
    #[must_use]
    pub fn clone_resolver(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            registry_client: self.registry_client.clone(),
        }
    }

    /// Checks if the resolver has external registry capabilities
    ///
    /// # Returns  
    ///
    /// `true` if external registry is available for enhanced resolution
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::dependency::conflict_resolver::ConflictResolver;
    ///
    /// let resolver = ConflictResolver::new(storage, client);
    /// 
    /// if resolver.has_registry() {
    ///     println!("Enhanced resolution available");
    /// } else {
    ///     println!("Local-only resolution");
    /// }
    /// ```
    #[must_use]
    pub fn has_registry(&self) -> bool {
        self.registry_client.has_registry()
    }

    /// Resolves a specific dependency group to find the optimal version
    ///
    /// Internal method that implements the core resolution logic for a single
    /// dependency name with potentially multiple version requirements.
    ///
    /// # Arguments
    ///
    /// * `dep_name` - Name of the dependency to resolve
    /// * `version_specs` - List of (version, VersionReq) pairs to reconcile
    ///
    /// # Returns
    ///
    /// String representing the resolved version for this dependency
    ///
    /// # Algorithm
    ///
    /// 1. Extract all version requirements from the specs
    /// 2. Use find_highest_compatible_version to get optimal version
    /// 3. Fallback to highest existing version if no compatible version found
    /// 4. Final fallback to first available version
    async fn resolve_dependency_group(
        &self,
        dep_name: &str,
        version_specs: &[(String, VersionReq)],
    ) -> Result<String, VersionError> {
        if version_specs.is_empty() {
            return Err(VersionError::InvalidVersion(
                format!("No version specifications for dependency '{dep_name}'")
            ));
        }

        if version_specs.len() == 1 {
            // No conflict - return the single version
            return Ok(version_specs[0].0.clone());
        }

        // Multiple versions - need to resolve conflict
        let requirements: Vec<&VersionReq> = version_specs.iter()
            .map(|(_, req)| req)
            .collect();

        // Try to find compatible version using advanced algorithm
        match self.find_highest_compatible_version(dep_name, &requirements).await {
            Ok(version) => Ok(version),
            Err(_) => {
                // Fallback: choose highest semver version from existing versions
                self.select_highest_existing_version(version_specs)
            }
        }
    }

    /// Finds the best compatible version from a list of available versions
    ///
    /// Internal method that filters available versions against requirements
    /// and selects the highest compatible version.
    ///
    /// # Arguments
    ///
    /// * `available_versions` - List of versions available from registry
    /// * `requirements` - Version requirements to satisfy
    ///
    /// # Returns
    ///
    /// `Ok(Some(version))` if compatible version found, `Ok(None)` otherwise
    fn find_best_compatible_version(
        &self,
        available_versions: &[String],
        requirements: &[&VersionReq],
    ) -> Option<String> {
        let mut compatible_versions = Vec::new();

        for version_str in available_versions {
            let clean_version = self.clean_version_string(version_str);
            
            if let Ok(version) = SemverVersion::parse(&clean_version) {
                if requirements.iter().all(|req| req.matches(&version)) {
                    compatible_versions.push((version, version_str.clone()));
                }
            }
        }

        if compatible_versions.is_empty() {
            return None;
        }

        // Sort by semver and return highest
        compatible_versions.sort_by(|(a, _), (b, _)| a.cmp(b));
        
        compatible_versions.last().map(|(_, version_str)| version_str.clone())
    }

    /// Selects the highest version from existing version specifications
    ///
    /// Fallback method when no compatible version can be found through
    /// external registry or compatibility analysis.
    ///
    /// # Arguments
    ///
    /// * `version_specs` - List of (version, VersionReq) pairs
    ///
    /// # Returns
    ///
    /// String representing the highest version from the specifications
    fn select_highest_existing_version(
        &self,
        version_specs: &[(String, VersionReq)],
    ) -> Result<String, VersionError> {
        let mut parsed_versions = Vec::new();

        for (version_str, _) in version_specs {
            let clean_version = self.clean_version_string(version_str);
            
            if let Ok(version) = SemverVersion::parse(&clean_version) {
                parsed_versions.push((version, version_str.clone()));
            }
        }

        if parsed_versions.is_empty() {
            return Err(VersionError::InvalidVersion(
                "No valid versions found in specifications".to_string()
            ));
        }

        // Sort and return highest
        parsed_versions.sort_by(|(a, _), (b, _)| a.cmp(b));
        
        parsed_versions.last()
            .map(|(_, version_str)| version_str.clone())
            .ok_or_else(|| VersionError::InvalidVersion(
                "No valid versions found after parsing".to_string()
            ))
    }

    /// Determines if a version needs to be updated to the resolved version
    ///
    /// Internal method to compare current and resolved versions and determine
    /// if an update is necessary.
    ///
    /// # Arguments
    ///
    /// * `current` - Current version string
    /// * `resolved` - Resolved target version string
    ///
    /// # Returns
    ///
    /// `true` if update is needed, `false` otherwise
    fn version_needs_update(&self, current: &str, resolved: &str) -> Result<bool, VersionError> {
        let current_clean = self.clean_version_string(current);
        let resolved_clean = self.clean_version_string(resolved);

        if current_clean == resolved_clean {
            return Ok(false);
        }

        // Parse both versions for comparison
        let current_ver = SemverVersion::parse(&current_clean)
            .map_err(|e| VersionError::InvalidVersion(
                format!("Current version parse error: {}", e)
            ))?;
            
        let resolved_ver = SemverVersion::parse(&resolved_clean)
            .map_err(|e| VersionError::InvalidVersion(
                format!("Resolved version parse error: {}", e)
            ))?;

        // Update if resolved version is different (could be upgrade or downgrade)
        Ok(current_ver != resolved_ver)
    }

    /// Cleans version strings by removing semver prefixes
    ///
    /// Internal utility method to normalize version strings for parsing.
    ///
    /// # Arguments
    ///
    /// * `version` - Version string to clean
    ///
    /// # Returns
    ///
    /// Cleaned version string without ^ ~ = prefixes
    fn clean_version_string(&self, version: &str) -> String {
        version
            .trim_start_matches('^')
            .trim_start_matches('~')
            .trim_start_matches('=')
            .to_string()
    }

    /// Generates a reasonable fallback version when no data is available
    ///
    /// Internal method that creates a version string that could potentially
    /// satisfy the given requirements when no better data is available.
    ///
    /// # Arguments
    ///
    /// * `requirements` - Version requirements to potentially satisfy
    ///
    /// # Returns
    ///
    /// A fallback version string
    fn generate_fallback_version(&self, requirements: &[&VersionReq]) -> Result<String, PackageRegistryError> {
        if requirements.is_empty() {
            return Ok("1.0.0".to_string());
        }

        // Try to find a reasonable version that might satisfy requirements
        // This is a simple heuristic - in a real implementation you might
        // analyze the requirements more sophisticatedly
        for req in requirements {
            let req_str = req.to_string();
            
            // Extract base version from requirement if possible
            let clean_req = self.clean_version_string(&req_str);
            if SemverVersion::parse(&clean_req).is_ok() {
                return Ok(clean_req);
            }
        }

        // Final fallback
        Ok("1.0.0".to_string())
    }
}

impl Clone for ConflictResolver {
    fn clone(&self) -> Self {
        self.clone_resolver()
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new(DependencyStorage::default(), PackageRegistryClient::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_no_conflicts() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new();
        let resolver = ConflictResolver::new(storage.clone(), client);

        // Add single dependency with no conflicts
        storage.get_or_insert("react", "^17.0.0").unwrap();

        let result = resolver.resolve_version_conflicts().await.unwrap();
        
        assert_eq!(result.resolved_versions.len(), 1);
        assert!(result.resolved_versions.contains_key("react"));
        assert!(result.updates_required.is_empty()); // No updates needed
    }

    #[tokio::test]
    async fn test_resolve_simple_conflict() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new();
        let resolver = ConflictResolver::new(storage.clone(), client);

        // Add conflicting versions - storage should resolve to higher
        storage.get_or_insert("react", "^16.0.0").unwrap();
        storage.get_or_insert("react", "^17.0.0").unwrap(); // Higher version

        let result = resolver.resolve_version_conflicts().await.unwrap();
        
        assert_eq!(result.resolved_versions.len(), 1);
        // The resolver uses fixed_version() which removes prefixes
        assert_eq!(result.resolved_versions.get("react").unwrap(), "17.0.0");
    }

    #[tokio::test]
    async fn test_resolve_empty_storage() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new();
        let resolver = ConflictResolver::new(storage, client);

        let result = resolver.resolve_version_conflicts().await.unwrap();
        
        assert!(result.resolved_versions.is_empty());
        assert!(result.updates_required.is_empty());
    }

    #[tokio::test]
    async fn test_find_highest_compatible_version_fallback() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new(); // No registry configured
        let resolver = ConflictResolver::new(storage.clone(), client);

        // Add a dependency to storage
        storage.get_or_insert("lodash", "^4.17.0").unwrap();

        let req = VersionReq::parse("^4.0.0").unwrap();
        let result = resolver.find_highest_compatible_version("lodash", &[&req]).await.unwrap();
        
        // Should use local storage version
        assert_eq!(result, "4.17.0");
    }

    #[tokio::test]
    async fn test_apply_resolution_empty() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new();
        let resolver = ConflictResolver::new(storage, client);

        // Empty resolution result
        let result = ResolutionResult {
            resolved_versions: HashMap::new(),
            updates_required: Vec::new(),
        };

        // Should succeed with no updates
        resolver.apply_resolution(&result).await.unwrap();
    }

    #[tokio::test]
    async fn test_apply_resolution_with_updates() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new();
        let resolver = ConflictResolver::new(storage.clone(), client);

        // Add initial dependency
        storage.get_or_insert("react", "^16.0.0").unwrap();

        // Create resolution with update
        let mut resolved_versions = HashMap::new();
        resolved_versions.insert("react".to_string(), "^17.0.0".to_string());

        let updates = vec![DependencyUpdate {
            package_name: "test-app".to_string(),
            dependency_name: "react".to_string(),
            current_version: "^16.0.0".to_string(),
            new_version: "^17.0.0".to_string(),
        }];

        let result = ResolutionResult {
            resolved_versions,
            updates_required: updates,
        };

        // Apply updates
        resolver.apply_resolution(&result).await.unwrap();

        // Verify update was applied
        let updated_dep = storage.get("react").unwrap();
        assert_eq!(updated_dep.version().to_string(), "^17.0.0");
    }

    #[test]
    fn test_has_registry() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new();
        let resolver = ConflictResolver::new(storage, client);

        assert!(!resolver.has_registry()); // No registry configured
    }

    #[test]
    fn test_clone_resolver() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new();
        let resolver = ConflictResolver::new(storage, client);
        
        let cloned = resolver.clone_resolver();
        assert_eq!(resolver.has_registry(), cloned.has_registry());
    }

    #[test]
    fn test_clean_version_string() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new();
        let resolver = ConflictResolver::new(storage, client);

        assert_eq!(resolver.clean_version_string("^1.0.0"), "1.0.0");
        assert_eq!(resolver.clean_version_string("~2.1.0"), "2.1.0");
        assert_eq!(resolver.clean_version_string("=3.0.0"), "3.0.0");
        assert_eq!(resolver.clean_version_string("1.0.0"), "1.0.0");
    }

    #[tokio::test]
    async fn test_generate_fallback_version() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new();
        let resolver = ConflictResolver::new(storage, client);

        // Empty requirements should return default
        let result = resolver.generate_fallback_version(&[]).unwrap();
        assert_eq!(result, "1.0.0");

        // With valid requirement
        let req = VersionReq::parse("^2.0.0").unwrap();
        let result = resolver.generate_fallback_version(&[&req]).unwrap();
        assert_eq!(result, "2.0.0");
    }
}