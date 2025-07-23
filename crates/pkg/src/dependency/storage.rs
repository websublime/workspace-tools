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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::{Arc as StdArc, Barrier};

    #[test]
    fn test_new_storage_is_empty() {
        let storage = DependencyStorage::new();
        let (count, memory) = storage.stats();
        assert_eq!(count, 0);
        assert_eq!(memory, 0);
    }

    #[test]
    fn test_get_or_insert_new_dependency() {
        let storage = DependencyStorage::new();
        let dep = storage.get_or_insert("react", "^17.0.0").unwrap();
        assert_eq!(dep.name(), "react");
        assert_eq!(dep.version().to_string(), "^17.0.0");
        
        let (count, _) = storage.stats();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_get_or_insert_existing_dependency() {
        let storage = DependencyStorage::new();
        
        // Insert initial version
        let dep1 = storage.get_or_insert("react", "^17.0.0").unwrap();
        assert_eq!(dep1.version().to_string(), "^17.0.0");
        
        // Get same version - should return same
        let dep2 = storage.get_or_insert("react", "^17.0.0").unwrap();
        assert_eq!(dep2.version().to_string(), "^17.0.0");
        
        let (count, _) = storage.stats();
        assert_eq!(count, 1); // Still only one dependency
    }

    #[test]
    fn test_version_update_logic() {
        let storage = DependencyStorage::new();
        
        // Insert initial version
        storage.get_or_insert("react", "^17.0.0").unwrap();
        
        // Try to insert higher version - should update
        let dep = storage.get_or_insert("react", "^18.0.0").unwrap();
        assert_eq!(dep.version().to_string(), "^18.0.0");
        
        // Try to insert lower version - should keep higher
        let dep = storage.get_or_insert("react", "^16.0.0").unwrap();
        assert_eq!(dep.version().to_string(), "^18.0.0");
        
        // Verify using get method
        let dep = storage.get("react").unwrap();
        assert_eq!(dep.version().to_string(), "^18.0.0");
    }

    #[test]
    fn test_get_nonexistent_dependency() {
        let storage = DependencyStorage::new();
        assert!(storage.get("missing").is_none());
    }

    #[test]
    fn test_update_existing_dependency() {
        let storage = DependencyStorage::new();
        storage.get_or_insert("lodash", "^4.17.0").unwrap();
        
        // Update existing dependency
        storage.update("lodash", "^4.17.21").unwrap();
        let dep = storage.get("lodash").unwrap();
        assert_eq!(dep.version().to_string(), "^4.17.21");
    }

    #[test]
    fn test_update_nonexistent_dependency() {
        let storage = DependencyStorage::new();
        let result = storage.update("missing", "1.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_all_dependencies() {
        let storage = DependencyStorage::new();
        storage.get_or_insert("react", "^17.0.0").unwrap();
        storage.get_or_insert("lodash", "^4.17.0").unwrap();
        storage.get_or_insert("vue", "^3.0.0").unwrap();
        
        let all_deps = storage.all_dependencies();
        assert_eq!(all_deps.len(), 3);
        
        // Verify all dependencies are present
        let names: Vec<String> = all_deps.iter().map(|(name, _)| name.clone()).collect();
        assert!(names.contains(&"react".to_string()));
        assert!(names.contains(&"lodash".to_string()));
        assert!(names.contains(&"vue".to_string()));
    }

    #[test]
    fn test_batch_updates_success() {
        let storage = DependencyStorage::new();
        storage.get_or_insert("react", "^16.0.0").unwrap();
        storage.get_or_insert("lodash", "^4.17.0").unwrap();
        
        let updates = vec![
            ("react".to_string(), "^17.0.0".to_string()),
            ("lodash".to_string(), "^4.17.21".to_string()),
        ];
        
        storage.apply_batch_updates(&updates).unwrap();
        
        // Verify all updates applied
        let react = storage.get("react").unwrap();
        assert_eq!(react.version().to_string(), "^17.0.0");
        let lodash = storage.get("lodash").unwrap();
        assert_eq!(lodash.version().to_string(), "^4.17.21");
    }

    #[test]
    fn test_batch_updates_atomicity() {
        let storage = DependencyStorage::new();
        storage.get_or_insert("react", "^16.0.0").unwrap();
        
        // Include one invalid update - should fail and not apply any
        let updates = vec![
            ("react".to_string(), "^17.0.0".to_string()),
            ("missing".to_string(), "^1.0.0".to_string()), // This will fail
        ];
        
        let result = storage.apply_batch_updates(&updates);
        assert!(result.is_err());
        
        // Verify no updates were applied
        let react = storage.get("react").unwrap();
        assert_eq!(react.version().to_string(), "^16.0.0"); // Unchanged
    }

    #[test]
    fn test_clear() {
        let storage = DependencyStorage::new();
        storage.get_or_insert("react", "^17.0.0").unwrap();
        storage.get_or_insert("lodash", "^4.17.0").unwrap();
        
        let (count, _) = storage.stats();
        assert_eq!(count, 2);
        
        storage.clear().unwrap();
        let (count, _) = storage.stats();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_concurrent_access() {
        let storage = StdArc::new(DependencyStorage::new());
        let barrier = StdArc::new(Barrier::new(3));
        let mut handles = vec![];
        
        // Spawn 3 threads that will access storage concurrently
        for i in 0..3 {
            let storage_clone = StdArc::clone(&storage);
            let barrier_clone = StdArc::clone(&barrier);
            
            let handle = thread::spawn(move || {
                barrier_clone.wait(); // Synchronize start
                
                // Each thread inserts different dependencies
                for j in 0..10 {
                    let dep_name = format!("dep-{}-{}", i, j);
                    storage_clone.get_or_insert(&dep_name, "^1.0.0").unwrap();
                }
                
                // Read operations
                for j in 0..10 {
                    let dep_name = format!("dep-{}-{}", i, j);
                    assert!(storage_clone.get(&dep_name).is_some());
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify all dependencies were inserted correctly
        let (count, _) = storage.stats();
        assert_eq!(count, 30); // 3 threads * 10 dependencies each
    }

    #[test]
    fn test_concurrent_version_updates() {
        let storage = StdArc::new(DependencyStorage::new());
        
        // Insert initial dependency
        storage.get_or_insert("react", "^16.0.0").unwrap();
        
        let barrier = StdArc::new(Barrier::new(2));
        let mut handles = vec![];
        
        // Thread 1: Update to 17.0.0
        {
            let storage_clone = StdArc::clone(&storage);
            let barrier_clone = StdArc::clone(&barrier);
            let handle = thread::spawn(move || {
                barrier_clone.wait();
                storage_clone.get_or_insert("react", "^17.0.0").unwrap();
            });
            handles.push(handle);
        }
        
        // Thread 2: Update to 18.0.0 (higher version)
        {
            let storage_clone = StdArc::clone(&storage);
            let barrier_clone = StdArc::clone(&barrier);
            let handle = thread::spawn(move || {
                barrier_clone.wait();
                storage_clone.get_or_insert("react", "^18.0.0").unwrap();
            });
            handles.push(handle);
        }
        
        // Wait for both threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // The final version should be the highest one (^18.0.0)
        let dep = storage.get("react").unwrap();
        assert_eq!(dep.version().to_string(), "^18.0.0");
    }

    #[test]
    fn test_version_comparison_edge_cases() {
        let storage = DependencyStorage::new();
        
        // Test with versions without prefix (semver interprets as caret by default)
        storage.get_or_insert("test", "1.0.0").unwrap();
        storage.get_or_insert("test", "1.0.1").unwrap(); // Higher
        let dep = storage.get("test").unwrap();
        assert_eq!(dep.version().to_string(), "^1.0.1"); // semver default behavior
        
        // Test with exact versions using = prefix
        storage.get_or_insert("test_exact", "=1.0.0").unwrap();
        storage.get_or_insert("test_exact", "=1.0.1").unwrap(); // Higher
        let dep = storage.get("test_exact").unwrap();
        assert_eq!(dep.version().to_string(), "=1.0.1");
        
        // Test with mixed prefixes
        storage.get_or_insert("test2", "^1.0.0").unwrap();
        storage.get_or_insert("test2", "~1.0.5").unwrap(); // Higher patch version
        let dep = storage.get("test2").unwrap();
        assert_eq!(dep.version().to_string(), "~1.0.5");
        
        // Test with major version differences
        storage.get_or_insert("test3", "^1.5.0").unwrap();
        storage.get_or_insert("test3", "^2.0.0").unwrap(); // Higher major
        let dep = storage.get("test3").unwrap();
        assert_eq!(dep.version().to_string(), "^2.0.0");
    }

    #[test]
    fn test_invalid_version_handling() {
        let storage = DependencyStorage::new();
        
        // Test invalid version format
        let result = storage.get_or_insert("test", "invalid-version");
        assert!(result.is_err());
        
        // Verify storage wasn't corrupted
        let (count, _) = storage.stats();
        assert_eq!(count, 0);
    }
}