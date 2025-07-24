//! # Change Set Management for Batch Version Operations
//!
//! ## What
//! This module provides structures for managing batch version bumping operations
//! across packages in both single repository and monorepo contexts. It implements
//! Rust idiomático patterns for representing change sets with strong type safety
//! and zero-cost abstractions.
//!
//! ## How  
//! The module uses owned data structures with explicit ownership semantics,
//! avoiding Java-like abstractions in favor of concrete Rust implementations.
//! All operations are designed for async contexts with efficient borrowing patterns.
//!
//! ## Why
//! Batch operations require careful coordination of multiple package changes,
//! preview functionality, and context-aware execution. This module provides
//! the foundational structures for enterprise-grade cascade version bumping
//! while maintaining Rust performance characteristics.

use crate::version::version::BumpStrategy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::SystemTime,
};

/// Execution mode for version bumping operations
///
/// This enum controls whether version bump operations are executed as preview
/// (no filesystem changes) or applied (real filesystem modifications).
/// 
/// # Examples
///
/// ```rust
/// use sublime_package_tools::version::BumpExecutionMode;
///
/// // Preview mode for safe analysis
/// let preview_mode = BumpExecutionMode::Preview;
/// assert!(!preview_mode.modifies_filesystem());
///
/// // Apply mode for real changes
/// let apply_mode = BumpExecutionMode::Apply;
/// assert!(apply_mode.modifies_filesystem());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BumpExecutionMode {
    /// Generate VersionBumpReport without making filesystem changes
    /// 
    /// This mode simulates all operations and produces a complete report
    /// of what would happen, including affected packages, version changes,
    /// and dependency reference updates, without modifying any files.
    Preview,
    
    /// Execute real changes on the filesystem
    ///
    /// This mode performs actual version bumping operations, modifying
    /// package.json files and updating dependency references as needed.
    /// Should only be used after preview analysis confirms the changes.
    Apply,
}

impl BumpExecutionMode {
    /// Check if this execution mode modifies the filesystem
    ///
    /// # Returns
    ///
    /// `true` if the mode will modify files, `false` for preview-only modes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::BumpExecutionMode;
    ///
    /// assert!(!BumpExecutionMode::Preview.modifies_filesystem());
    /// assert!(BumpExecutionMode::Apply.modifies_filesystem());
    /// ```
    #[must_use]
    pub const fn modifies_filesystem(self) -> bool {
        matches!(self, Self::Apply)
    }
    
    /// Check if this execution mode is preview-only
    ///
    /// # Returns
    ///
    /// `true` if the mode is preview-only, `false` if it modifies files
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::BumpExecutionMode;
    ///
    /// assert!(BumpExecutionMode::Preview.is_preview());
    /// assert!(!BumpExecutionMode::Apply.is_preview());
    /// ```
    #[must_use]
    pub const fn is_preview(self) -> bool {
        matches!(self, Self::Preview)
    }
}

impl Default for BumpExecutionMode {
    /// Default execution mode is Preview for safety
    ///
    /// Preview mode is the default to prevent accidental filesystem
    /// modifications. Users must explicitly choose Apply mode.
    fn default() -> Self {
        Self::Preview
    }
}

/// Batch change set for version bumping operations
///
/// Represents a collection of package version changes to be applied together,
/// with execution context and metadata. This structure uses owned data to
/// avoid lifetime complications in async contexts.
///
/// # Design Principles
///
/// - **Owned Data**: All fields are owned to work seamlessly with async
/// - **Type Safety**: Uses strong types to prevent invalid operations  
/// - **Zero-Cost**: Designed for zero-cost abstraction patterns
/// - **Explicit**: All behavior is explicit, no hidden magic
///
/// # Examples
///
/// ```rust
/// use sublime_package_tools::version::{ChangeSet, BumpExecutionMode, BumpStrategy};
/// use std::collections::HashMap;
///
/// // Create a change set for multiple packages
/// let mut target_packages = HashMap::new();
/// target_packages.insert("package-a".to_string(), BumpStrategy::Minor);
/// target_packages.insert("package-b".to_string(), BumpStrategy::Patch);
///
/// let change_set = ChangeSet::new(
///     target_packages,
///     "Feature release with bug fixes".to_string(),
/// );
///
/// assert_eq!(change_set.target_packages().len(), 2);
/// assert!(change_set.execution_mode().is_preview());
/// ```
#[derive(Debug, Clone)]
pub struct ChangeSet {
    /// Packages that require version changes with their bump strategies
    ///
    /// Each entry represents a package that has been modified and needs
    /// a version bump. The BumpStrategy determines how the version should
    /// be incremented (Major, Minor, Patch, etc.).
    target_packages: HashMap<String, BumpStrategy>,
    
    /// Human-readable reason for the changes
    ///
    /// This field provides context for why the version bump is happening,
    /// useful for logging, reporting, and debugging purposes.
    reason: String,
    
    /// Timestamp when the change set was created
    ///
    /// Used for ordering operations and providing audit trail information.
    /// Stored as SystemTime for maximum compatibility across platforms.
    timestamp: SystemTime,
    
    /// Whether this is a preview or real execution
    ///
    /// Controls the execution behavior of the cascade bumping operation.
    /// Preview mode generates reports without filesystem changes.
    execution_mode: BumpExecutionMode,
}

impl ChangeSet {
    /// Create a new change set with preview execution mode
    ///
    /// # Arguments
    ///
    /// * `target_packages` - Map of package names to their bump strategies
    /// * `reason` - Human-readable description of why the changes are needed
    ///
    /// # Returns
    ///
    /// A new ChangeSet configured for preview mode with current timestamp
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{ChangeSet, BumpStrategy};
    /// use std::collections::HashMap;
    ///
    /// let mut packages = HashMap::new();
    /// packages.insert("my-package".to_string(), BumpStrategy::Minor);
    ///
    /// let change_set = ChangeSet::new(packages, "Add new feature".to_string());
    /// assert!(change_set.execution_mode().is_preview());
    /// ```
    #[must_use]
    pub fn new(target_packages: HashMap<String, BumpStrategy>, reason: String) -> Self {
        Self {
            target_packages,
            reason,
            timestamp: SystemTime::now(),
            execution_mode: BumpExecutionMode::default(),
        }
    }
    
    /// Create a new change set with explicit execution mode
    ///
    /// # Arguments
    ///
    /// * `target_packages` - Map of package names to their bump strategies
    /// * `reason` - Human-readable description of why the changes are needed
    /// * `execution_mode` - Whether to preview or apply changes
    ///
    /// # Returns
    ///
    /// A new ChangeSet with the specified execution mode
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{ChangeSet, BumpStrategy, BumpExecutionMode};
    /// use std::collections::HashMap;
    ///
    /// let mut packages = HashMap::new();
    /// packages.insert("my-package".to_string(), BumpStrategy::Major);
    ///
    /// let change_set = ChangeSet::with_execution_mode(
    ///     packages,
    ///     "Breaking change release".to_string(),
    ///     BumpExecutionMode::Apply,
    /// );
    /// assert!(!change_set.execution_mode().is_preview());
    /// ```
    #[must_use]
    pub fn with_execution_mode(
        target_packages: HashMap<String, BumpStrategy>,
        reason: String,
        execution_mode: BumpExecutionMode,
    ) -> Self {
        Self {
            target_packages,
            reason,
            timestamp: SystemTime::now(),
            execution_mode,
        }
    }
    
    /// Get reference to target packages and their bump strategies
    ///
    /// # Returns
    ///
    /// Immutable reference to the packages HashMap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{ChangeSet, BumpStrategy};
    /// use std::collections::HashMap;
    ///
    /// let mut packages = HashMap::new();
    /// packages.insert("test-pkg".to_string(), BumpStrategy::Patch);
    /// let change_set = ChangeSet::new(packages, "Bug fix".to_string());
    ///
    /// assert_eq!(change_set.target_packages().len(), 1);
    /// assert_eq!(
    ///     change_set.target_packages().get("test-pkg"),
    ///     Some(&BumpStrategy::Patch)
    /// );
    /// ```
    #[must_use]
    pub fn target_packages(&self) -> &HashMap<String, BumpStrategy> {
        &self.target_packages
    }
    
    /// Get mutable reference to target packages for modification
    ///
    /// # Returns
    ///
    /// Mutable reference to the packages HashMap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{ChangeSet, BumpStrategy};
    /// use std::collections::HashMap;
    ///
    /// let mut change_set = ChangeSet::new(HashMap::new(), "Update".to_string());
    /// change_set.target_packages_mut().insert(
    ///     "new-pkg".to_string(),
    ///     BumpStrategy::Minor,
    /// );
    ///
    /// assert_eq!(change_set.target_packages().len(), 1);
    /// ```
    pub fn target_packages_mut(&mut self) -> &mut HashMap<String, BumpStrategy> {
        &mut self.target_packages
    }
    
    /// Get the reason for these changes
    ///
    /// # Returns
    ///
    /// String slice containing the human-readable reason
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::ChangeSet;
    /// use std::collections::HashMap;
    ///
    /// let change_set = ChangeSet::new(HashMap::new(), "Bug fixes".to_string());
    /// assert_eq!(change_set.reason(), "Bug fixes");
    /// ```
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
    
    /// Get the timestamp when this change set was created
    ///
    /// # Returns
    ///
    /// SystemTime representing when the change set was created
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::ChangeSet;
    /// use std::collections::HashMap;
    /// use std::time::SystemTime;
    ///
    /// let before = SystemTime::now();
    /// let change_set = ChangeSet::new(HashMap::new(), "Test".to_string());
    /// let after = SystemTime::now();
    ///
    /// assert!(change_set.timestamp() >= before);
    /// assert!(change_set.timestamp() <= after);
    /// ```
    #[must_use]
    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
    
    /// Get the current execution mode
    ///
    /// # Returns
    ///
    /// The current BumpExecutionMode
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{ChangeSet, BumpExecutionMode};
    /// use std::collections::HashMap;
    ///
    /// let change_set = ChangeSet::new(HashMap::new(), "Test".to_string());
    /// assert_eq!(change_set.execution_mode(), BumpExecutionMode::Preview);
    /// ```
    #[must_use]
    pub fn execution_mode(&self) -> BumpExecutionMode {
        self.execution_mode
    }
    
    /// Set the execution mode for this change set
    ///
    /// # Arguments
    ///
    /// * `mode` - The new execution mode to set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{ChangeSet, BumpExecutionMode};
    /// use std::collections::HashMap;
    ///
    /// let mut change_set = ChangeSet::new(HashMap::new(), "Test".to_string());
    /// assert!(change_set.execution_mode().is_preview());
    ///
    /// change_set.set_execution_mode(BumpExecutionMode::Apply);
    /// assert!(change_set.execution_mode().modifies_filesystem());
    /// ```
    pub fn set_execution_mode(&mut self, mode: BumpExecutionMode) {
        self.execution_mode = mode;
    }
    
    /// Convert this change set to preview mode
    ///
    /// Returns a new ChangeSet identical to this one but in preview mode,
    /// leaving the original unchanged.
    ///
    /// # Returns
    ///
    /// A new ChangeSet in preview mode
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{ChangeSet, BumpExecutionMode};
    /// use std::collections::HashMap;
    ///
    /// let mut original = ChangeSet::new(HashMap::new(), "Test".to_string());
    /// original.set_execution_mode(BumpExecutionMode::Apply);
    ///
    /// let preview = original.as_preview();
    /// assert!(original.execution_mode().modifies_filesystem());
    /// assert!(preview.execution_mode().is_preview());
    /// ```
    #[must_use]
    pub fn as_preview(&self) -> Self {
        Self {
            target_packages: self.target_packages.clone(),
            reason: self.reason.clone(),
            timestamp: self.timestamp,
            execution_mode: BumpExecutionMode::Preview,
        }
    }
    
    /// Convert this change set to apply mode
    ///
    /// Returns a new ChangeSet identical to this one but in apply mode,
    /// leaving the original unchanged.
    ///
    /// # Returns
    ///
    /// A new ChangeSet in apply mode
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{ChangeSet, BumpExecutionMode};
    /// use std::collections::HashMap;
    ///
    /// let original = ChangeSet::new(HashMap::new(), "Test".to_string());
    /// let apply = original.as_apply();
    ///
    /// assert!(original.execution_mode().is_preview());
    /// assert!(apply.execution_mode().modifies_filesystem());
    /// ```
    #[must_use]
    pub fn as_apply(&self) -> Self {
        Self {
            target_packages: self.target_packages.clone(),
            reason: self.reason.clone(),
            timestamp: self.timestamp,
            execution_mode: BumpExecutionMode::Apply,
        }
    }
    
    /// Check if this change set is empty (no target packages)
    ///
    /// # Returns
    ///
    /// `true` if there are no packages to bump, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{ChangeSet, BumpStrategy};
    /// use std::collections::HashMap;
    ///
    /// let empty = ChangeSet::new(HashMap::new(), "Empty".to_string());
    /// assert!(empty.is_empty());
    ///
    /// let mut packages = HashMap::new();
    /// packages.insert("pkg".to_string(), BumpStrategy::Patch);
    /// let non_empty = ChangeSet::new(packages, "Non-empty".to_string());
    /// assert!(!non_empty.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.target_packages.is_empty()
    }
    
    /// Get the number of packages in this change set
    ///
    /// # Returns
    ///
    /// The number of packages that will be affected
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{ChangeSet, BumpStrategy};
    /// use std::collections::HashMap;
    ///
    /// let mut packages = HashMap::new();
    /// packages.insert("pkg1".to_string(), BumpStrategy::Major);
    /// packages.insert("pkg2".to_string(), BumpStrategy::Minor);
    ///
    /// let change_set = ChangeSet::new(packages, "Multi-package".to_string());
    /// assert_eq!(change_set.len(), 2);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.target_packages.len()
    }
    
    /// Add a package to this change set
    ///
    /// If the package already exists, its bump strategy will be updated.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to add
    /// * `strategy` - Bump strategy for this package
    ///
    /// # Returns
    ///
    /// The previous bump strategy if the package was already present
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{ChangeSet, BumpStrategy};
    /// use std::collections::HashMap;
    ///
    /// let mut change_set = ChangeSet::new(HashMap::new(), "Test".to_string());
    /// assert!(change_set.add_package("pkg".to_string(), BumpStrategy::Minor).is_none());
    /// assert_eq!(change_set.len(), 1);
    ///
    /// // Update existing package
    /// let previous = change_set.add_package("pkg".to_string(), BumpStrategy::Major);
    /// assert_eq!(previous, Some(BumpStrategy::Minor));
    /// assert_eq!(change_set.len(), 1);
    /// ```
    pub fn add_package(&mut self, package_name: String, strategy: BumpStrategy) -> Option<BumpStrategy> {
        self.target_packages.insert(package_name, strategy)
    }
    
    /// Remove a package from this change set
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to remove
    ///
    /// # Returns
    ///
    /// The bump strategy of the removed package, if it existed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{ChangeSet, BumpStrategy};
    /// use std::collections::HashMap;
    ///
    /// let mut packages = HashMap::new();
    /// packages.insert("pkg".to_string(), BumpStrategy::Patch);
    /// let mut change_set = ChangeSet::new(packages, "Test".to_string());
    ///
    /// assert_eq!(change_set.remove_package("pkg"), Some(BumpStrategy::Patch));
    /// assert!(change_set.is_empty());
    /// assert_eq!(change_set.remove_package("nonexistent"), None);
    /// ```
    pub fn remove_package(&mut self, package_name: &str) -> Option<BumpStrategy> {
        self.target_packages.remove(package_name)
    }
    
    /// Check if a package is included in this change set
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to check
    ///
    /// # Returns
    ///
    /// `true` if the package is in the change set, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::version::{ChangeSet, BumpStrategy};
    /// use std::collections::HashMap;
    ///
    /// let mut packages = HashMap::new();
    /// packages.insert("existing".to_string(), BumpStrategy::Minor);
    /// let change_set = ChangeSet::new(packages, "Test".to_string());
    ///
    /// assert!(change_set.contains_package("existing"));
    /// assert!(!change_set.contains_package("missing"));
    /// ```
    #[must_use]
    pub fn contains_package(&self, package_name: &str) -> bool {
        self.target_packages.contains_key(package_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod unit_tests {
        use super::*;

        #[test]
        fn test_bump_execution_mode_methods() {
            assert!(BumpExecutionMode::Preview.is_preview());
            assert!(!BumpExecutionMode::Preview.modifies_filesystem());
            
            assert!(!BumpExecutionMode::Apply.is_preview());
            assert!(BumpExecutionMode::Apply.modifies_filesystem());
        }

        #[test]
        fn test_bump_execution_mode_default() {
            assert_eq!(BumpExecutionMode::default(), BumpExecutionMode::Preview);
        }

        #[test]
        fn test_change_set_new() {
            let mut packages = HashMap::new();
            packages.insert("test-pkg".to_string(), BumpStrategy::Minor);
            
            let change_set = ChangeSet::new(packages, "Test reason".to_string());
            
            assert_eq!(change_set.target_packages().len(), 1);
            assert_eq!(change_set.reason(), "Test reason");
            assert_eq!(change_set.execution_mode(), BumpExecutionMode::Preview);
            assert!(!change_set.is_empty());
            assert_eq!(change_set.len(), 1);
        }

        #[test]
        fn test_change_set_with_execution_mode() {
            let packages = HashMap::new();
            let change_set = ChangeSet::with_execution_mode(
                packages,
                "Apply mode".to_string(),
                BumpExecutionMode::Apply,
            );
            
            assert_eq!(change_set.execution_mode(), BumpExecutionMode::Apply);
            assert!(change_set.execution_mode().modifies_filesystem());
        }

        #[test]
        fn test_change_set_mode_conversion() {
            let mut original = ChangeSet::new(HashMap::new(), "Test".to_string());
            original.set_execution_mode(BumpExecutionMode::Apply);
            
            let preview = original.as_preview();
            let apply = original.as_apply();
            
            assert_eq!(original.execution_mode(), BumpExecutionMode::Apply);
            assert_eq!(preview.execution_mode(), BumpExecutionMode::Preview);
            assert_eq!(apply.execution_mode(), BumpExecutionMode::Apply);
        }

        #[test]
        fn test_change_set_package_operations() {
            let mut change_set = ChangeSet::new(HashMap::new(), "Test".to_string());
            
            // Add packages
            assert!(change_set.add_package("pkg1".to_string(), BumpStrategy::Major).is_none());
            assert!(change_set.add_package("pkg2".to_string(), BumpStrategy::Minor).is_none());
            assert_eq!(change_set.len(), 2);
            
            // Update existing package
            let previous = change_set.add_package("pkg1".to_string(), BumpStrategy::Patch);
            assert_eq!(previous, Some(BumpStrategy::Major));
            assert_eq!(change_set.len(), 2);
            
            // Check contains
            assert!(change_set.contains_package("pkg1"));
            assert!(change_set.contains_package("pkg2"));
            assert!(!change_set.contains_package("nonexistent"));
            
            // Remove package
            assert_eq!(change_set.remove_package("pkg1"), Some(BumpStrategy::Patch));
            assert_eq!(change_set.len(), 1);
            assert!(!change_set.contains_package("pkg1"));
        }

        #[test]
        fn test_change_set_empty() {
            let empty = ChangeSet::new(HashMap::new(), "Empty".to_string());
            assert!(empty.is_empty());
            assert_eq!(empty.len(), 0);
            
            let mut non_empty = HashMap::new();
            non_empty.insert("pkg".to_string(), BumpStrategy::Patch);
            let non_empty = ChangeSet::new(non_empty, "Non-empty".to_string());
            assert!(!non_empty.is_empty());
            assert_eq!(non_empty.len(), 1);
        }

        #[test]
        fn test_change_set_timestamp() {
            use std::time::Duration;
            
            let before = SystemTime::now();
            let change_set = ChangeSet::new(HashMap::new(), "Test".to_string());
            let after = SystemTime::now();
            
            assert!(change_set.timestamp() >= before);
            assert!(change_set.timestamp() <= after);
            
            // Timestamp should be reasonably recent (within 1 second)
            let duration = after.duration_since(change_set.timestamp()).unwrap_or(Duration::ZERO);
            assert!(duration.as_secs() < 1);
        }
    }
}