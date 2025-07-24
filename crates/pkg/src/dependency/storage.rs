//! # Dependency Storage Service
//!
//! Pure data persistence service for dependency storage with enterprise-grade thread safety.
//!
//! ## Overview
//!
//! This service handles all storage concerns for dependencies:
//! - Thread-safe HashMap operations using Arc<RwLock<>>  
//! - Intelligent version comparison and resolution
//! - Atomic batch operations for consistency
//! - Memory management and optimization
//!
//! ## Thread Safety
//!
//! Uses Arc<RwLock<HashMap>> for concurrent access:
//! - Multiple concurrent readers allowed
//! - Single writer with exclusive access
//! - Lock poisoning protection with graceful degradation
//!
//! ## Performance
//!
//! - RwLock optimized for read-heavy workloads
//! - Arc enables cheap cloning for service sharing
//! - Batch operations minimize lock contention
//! - Lazy evaluation where possible

use crate::{Dependency, errors::VersionError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Pure data persistence service for dependency storage
///
/// This service encapsulates all dependency storage operations with enterprise-grade
/// thread safety and intelligent version resolution.
///
/// # Examples
///
/// ```rust
/// use sublime_package_tools::dependency::storage::DependencyStorage;
///
/// let storage = DependencyStorage::new();
/// 
/// // Thread-safe operations
/// let dep = storage.get_or_insert("react", "^17.0.0").unwrap();
/// assert_eq!(dep.name(), "react");
/// 
/// // Intelligent version resolution - higher version wins
/// let updated_dep = storage.get_or_insert("react", "^18.0.0").unwrap();
/// assert_eq!(updated_dep.version().to_string(), "^18.0.0");
/// ```
#[derive(Debug, Clone)]
pub(crate) struct DependencyStorage {
    /// Thread-safe storage for dependencies
    /// - Arc enables sharing between services
    /// - RwLock allows concurrent reads, exclusive writes
    /// - HashMap provides O(1) average case lookup
    dependencies: Arc<RwLock<HashMap<String, Dependency>>>,
}

impl DependencyStorage {
    /// Creates a new empty dependency storage
    ///
    /// # Examples
    ///
    /// ```rust
    /// let storage = DependencyStorage::new();
    /// let (count, _) = storage.stats();
    /// assert_eq!(count, 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            dependencies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Gets a dependency by name (read-only operation)
    ///
    /// This operation uses a read lock, allowing concurrent access with other readers.
    /// Returns `None` if dependency not found or lock is poisoned.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name to lookup
    ///
    /// # Returns
    ///
    /// `Some(Dependency)` if found, `None` otherwise
    ///
    /// # Thread Safety
    ///
    /// Uses read lock - allows concurrent access with other readers
    ///
    /// # Examples
    ///
    /// ```rust
    /// let storage = DependencyStorage::new();
    /// storage.get_or_insert("lodash", "^4.17.0").unwrap();
    /// 
    /// let dep = storage.get("lodash").unwrap();
    /// assert_eq!(dep.name(), "lodash");
    /// 
    /// assert!(storage.get("missing").is_none());
    /// ```
    pub fn get(&self, name: &str) -> Option<Dependency> {
        let deps = self.dependencies.read().ok()?;
        deps.get(name).cloned()
    }

    /// Inserts or updates a dependency with intelligent version resolution
    ///
    /// This is the core operation that implements intelligent version resolution:
    /// - If dependency doesn't exist: creates new dependency
    /// - If dependency exists with same version: returns existing
    /// - If dependency exists with different version: keeps higher version
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name
    /// * `version` - The version string (supports ^, ~, exact versions)
    ///
    /// # Returns
    ///
    /// The final dependency (existing or newly created)
    ///
    /// # Errors
    ///
    /// Returns `VersionError` if:
    /// - Version string is invalid for semver parsing
    /// - Lock is poisoned (degrades gracefully)
    /// - Dependency creation fails
    ///
    /// # Thread Safety
    ///
    /// Uses write lock for atomic insert/update operations
    ///
    /// # Algorithm
    ///
    /// 1. Acquire write lock (exclusive access)
    /// 2. Check if dependency exists
    /// 3. If exists, compare versions intelligently
    /// 4. Update to higher version if necessary
    /// 5. Return final dependency
    ///
    /// # Examples
    ///
    /// ```rust
    /// let storage = DependencyStorage::new();
    /// 
    /// // Create new dependency
    /// let dep1 = storage.get_or_insert("react", "^17.0.0").unwrap();
    /// assert_eq!(dep1.version().to_string(), "^17.0.0");
    /// 
    /// // Same version - returns same dependency
    /// let dep2 = storage.get_or_insert("react", "^17.0.0").unwrap();
    /// assert_eq!(dep2.version().to_string(), "^17.0.0");
    /// 
    /// // Higher version - updates and returns new version
    /// let dep3 = storage.get_or_insert("react", "^18.0.0").unwrap();
    /// assert_eq!(dep3.version().to_string(), "^18.0.0");
    /// 
    /// // Lower version - keeps higher version
    /// let dep4 = storage.get_or_insert("react", "^16.0.0").unwrap();
    /// assert_eq!(dep4.version().to_string(), "^18.0.0");
    /// ```
    pub fn get_or_insert(&self, name: &str, version: &str) -> Result<Dependency, VersionError> {
        let mut deps = self.dependencies.write()
            .map_err(|_| VersionError::InvalidVersion("Storage lock poisoned".to_string()))?;

        if let Some(existing_dep) = deps.get_mut(name) {
            let current_version = existing_dep.version().to_string();

            // If versions differ, apply intelligent resolution
            if current_version != version {
                let should_update = self.should_update_version(&current_version, version)?;
                if should_update {
                    existing_dep.update_version(version)?;
                }
            }

            return Ok(existing_dep.clone());
        }

        // Create new dependency
        let dep = Dependency::new(name, version)?;
        deps.insert(name.to_string(), dep.clone());
        Ok(dep)
    }

    /// Updates an existing dependency version
    ///
    /// This operation updates an existing dependency to a new version.
    /// Fails if the dependency doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name
    /// * `new_version` - The new version string
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful
    ///
    /// # Errors
    ///
    /// Returns `VersionError` if:
    /// - Dependency not found
    /// - Version string is invalid
    /// - Lock is poisoned
    ///
    /// # Thread Safety
    ///
    /// Uses write lock for atomic update
    ///
    /// # Examples
    ///
    /// ```rust
    /// let storage = DependencyStorage::new();
    /// storage.get_or_insert("lodash", "^4.17.0").unwrap();
    /// 
    /// // Update existing dependency
    /// storage.update("lodash", "^4.17.21").unwrap();
    /// let dep = storage.get("lodash").unwrap();
    /// assert_eq!(dep.version().to_string(), "^4.17.21");
    /// 
    /// // Updating non-existent dependency fails
    /// assert!(storage.update("missing", "1.0.0").is_err());
    /// ```
    #[allow(dead_code)]
    pub fn update(&self, name: &str, new_version: &str) -> Result<(), VersionError> {
        let mut deps = self.dependencies.write()
            .map_err(|_| VersionError::InvalidVersion("Storage lock poisoned".to_string()))?;

        if let Some(dep) = deps.get_mut(name) {
            dep.update_version(new_version)?;
            Ok(())
        } else {
            Err(VersionError::InvalidVersion(format!("Dependency '{}' not found", name)))
        }
    }

    /// Gets all dependencies as a snapshot
    ///
    /// This operation creates a snapshot of all dependencies, avoiding holding
    /// the lock for extended periods. The returned vector is independent of
    /// the internal storage.
    ///
    /// # Returns
    ///
    /// Vector of (name, dependency) pairs
    ///
    /// # Thread Safety
    ///
    /// Uses read lock, creates snapshot to avoid holding lock
    ///
    /// # Examples
    ///
    /// ```rust
    /// let storage = DependencyStorage::new();
    /// storage.get_or_insert("react", "^17.0.0").unwrap();
    /// storage.get_or_insert("lodash", "^4.17.0").unwrap();
    /// 
    /// let all_deps = storage.all_dependencies();
    /// assert_eq!(all_deps.len(), 2);
    /// 
    /// // Snapshot is independent - storage can be modified
    /// storage.get_or_insert("vue", "^3.0.0").unwrap();
    /// assert_eq!(all_deps.len(), 2); // Snapshot unchanged
    /// ```
    pub fn all_dependencies(&self) -> Vec<(String, Dependency)> {
        let deps = match self.dependencies.read() {
            Ok(deps) => deps,
            Err(_) => {
                // Lock poisoned - return empty vec
                // In production, this should be logged as an error
                return Vec::new();
            }
        };

        deps.iter()
            .map(|(name, dep)| (name.clone(), dep.clone()))
            .collect()
    }

    /// Applies batch updates atomically
    ///
    /// This operation applies multiple updates in a single transaction.
    /// Either all updates succeed or none are applied (atomicity).
    ///
    /// # Arguments
    ///
    /// * `updates` - Vector of (name, new_version) pairs to update
    ///
    /// # Returns
    ///
    /// `Ok(())` if all updates successful
    ///
    /// # Errors
    ///
    /// Returns `VersionError` on first failure - no updates are applied
    ///
    /// # Atomicity
    ///
    /// Updates are validated first, then applied atomically.
    /// Uses single write lock for entire operation to ensure consistency.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let storage = DependencyStorage::new();
    /// storage.get_or_insert("react", "^16.0.0").unwrap();
    /// storage.get_or_insert("lodash", "^4.17.0").unwrap();
    /// 
    /// // Batch update multiple dependencies
    /// let updates = vec![
    ///     ("react".to_string(), "^17.0.0".to_string()),
    ///     ("lodash".to_string(), "^4.17.21".to_string()),
    /// ];
    /// 
    /// storage.apply_batch_updates(&updates).unwrap();
    /// 
    /// // Verify all updates applied
    /// let react = storage.get("react").unwrap();
    /// assert_eq!(react.version().to_string(), "^17.0.0");
    /// let lodash = storage.get("lodash").unwrap();
    /// assert_eq!(lodash.version().to_string(), "^4.17.21");
    /// ```
    pub fn apply_batch_updates(&self, updates: &[(String, String)]) -> Result<(), VersionError> {
        let mut deps = self.dependencies.write()
            .map_err(|_| VersionError::InvalidVersion("Storage lock poisoned".to_string()))?;

        // Phase 1: Validate all updates can be applied
        for (name, new_version) in updates {
            if !deps.contains_key(name) {
                return Err(VersionError::InvalidVersion(format!("Dependency '{}' not found", name)));
            }
            // Validate version format by attempting to create a dependency
            let _ = Dependency::new(name, new_version)?;
        }

        // Phase 2: Apply all updates atomically
        for (name, new_version) in updates {
            if let Some(dep) = deps.get_mut(name) {
                dep.update_version(new_version)?;
            }
        }

        Ok(())
    }

    /// Gets storage statistics for monitoring
    ///
    /// Returns statistics about the current storage state for monitoring
    /// and debugging purposes.
    ///
    /// # Returns
    ///
    /// (total_dependencies, estimated_memory_usage_bytes)
    ///
    /// # Examples
    ///
    /// ```rust
    /// let storage = DependencyStorage::new();
    /// storage.get_or_insert("react", "^17.0.0").unwrap();
    /// storage.get_or_insert("lodash", "^4.17.0").unwrap();
    /// 
    /// let (count, memory) = storage.stats();
    /// assert_eq!(count, 2);
    /// assert!(memory > 0);
    /// ```
    #[allow(dead_code)]
    pub fn stats(&self) -> (usize, usize) {
        let deps = match self.dependencies.read() {
            Ok(deps) => deps,
            Err(_) => return (0, 0), // Lock poisoned
        };

        let count = deps.len();
        let memory_estimate = count * std::mem::size_of::<(String, Dependency)>();

        (count, memory_estimate)
    }

    /// Clears all dependencies from storage
    ///
    /// This operation removes all dependencies from storage. Useful for
    /// testing and cleanup scenarios.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let storage = DependencyStorage::new();
    /// storage.get_or_insert("react", "^17.0.0").unwrap();
    /// 
    /// let (count, _) = storage.stats();
    /// assert_eq!(count, 1);
    /// 
    /// storage.clear();
    /// let (count, _) = storage.stats();
    /// assert_eq!(count, 0);
    /// ```
    #[allow(dead_code)]
    pub fn clear(&self) -> Result<(), VersionError> {
        let mut deps = self.dependencies.write()
            .map_err(|_| VersionError::InvalidVersion("Storage lock poisoned".to_string()))?;
        
        deps.clear();
        Ok(())
    }

    /// Helper method for intelligent version comparison
    ///
    /// Implements enhanced version comparison logic using proper semver parsing.
    /// Handles ^ and ~ prefixes correctly.
    ///
    /// # Arguments
    ///
    /// * `current` - Current version string
    /// * `new` - New version string to compare
    ///
    /// # Returns
    ///
    /// `true` if should update to new version, `false` otherwise
    ///
    /// # Algorithm
    ///
    /// 1. Clean both version strings (remove ^ ~ prefixes)
    /// 2. Parse using semver for proper comparison
    /// 3. Return true if new version is higher
    fn should_update_version(&self, current: &str, new: &str) -> Result<bool, VersionError> {
        use semver::Version;

        // Clean version strings (remove ^ ~ = prefixes)
        let current_clean = current
            .trim_start_matches('^')
            .trim_start_matches('~')
            .trim_start_matches('=');
        let new_clean = new
            .trim_start_matches('^')
            .trim_start_matches('~')
            .trim_start_matches('=');

        // Parse both versions using semver
        let current_ver = Version::parse(current_clean)
            .map_err(|e| VersionError::InvalidVersion(format!("Current version parse error: {}", e)))?;
        let new_ver = Version::parse(new_clean)
            .map_err(|e| VersionError::InvalidVersion(format!("New version parse error: {}", e)))?;

        // Update if new version is higher
        Ok(new_ver > current_ver)
    }
}

impl Default for DependencyStorage {
    fn default() -> Self {
        Self::new()
    }
}

