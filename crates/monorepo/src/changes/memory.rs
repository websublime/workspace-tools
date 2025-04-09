//! In-memory change store implementation.
//!
//! This module provides an in-memory implementation of the `ChangeStore` trait,
//! useful for testing, temporary storage, and scenarios where persistence isn't needed.

use crate::{Change, ChangeId, ChangeResult, ChangeStore, Changeset};
use std::collections::HashMap;

/// Memory-based implementation of ChangeStore.
///
/// This implementation stores changesets in memory, making it useful for
/// testing, temporary storage, or scenarios where file persistence isn't needed.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::{MemoryChangeStore, ChangeStore, Change, ChangeType, Changeset};
///
/// // Create an in-memory store
/// let mut store = MemoryChangeStore::new();
///
/// // Create and store a change
/// let change = Change::new("ui", ChangeType::Feature, "Add button", false);
/// let changeset = Changeset::new::<String>(None, vec![change]);
/// store.store_changeset(&changeset).unwrap();
///
/// // Retrieve the change
/// let retrieved = store.get_changeset(&changeset.id).unwrap().unwrap();
/// ``
pub struct MemoryChangeStore {
    /// In-memory storage for changesets.
    changesets: HashMap<ChangeId, Changeset>,
}

impl MemoryChangeStore {
    /// Creates a new memory-based change store.
    ///
    /// # Returns
    ///
    /// A new empty `MemoryChangeStore`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::MemoryChangeStore;
    ///
    /// let store = MemoryChangeStore::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { changesets: HashMap::new() }
    }
}

impl Default for MemoryChangeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ChangeStore for MemoryChangeStore {
    /// Gets a changeset by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the changeset to retrieve
    ///
    /// # Returns
    ///
    /// The changeset if found, or `None` if not found.
    ///
    /// # Errors
    ///
    /// This implementation never returns an error.
    fn get_changeset(&self, id: &ChangeId) -> ChangeResult<Option<Changeset>> {
        Ok(self.changesets.get(id).cloned())
    }

    /// Gets all changesets.
    ///
    /// # Returns
    ///
    /// A vector of all changesets in the store.
    ///
    /// # Errors
    ///
    /// This implementation never returns an error.
    fn get_all_changesets(&self) -> ChangeResult<Vec<Changeset>> {
        Ok(self.changesets.values().cloned().collect())
    }

    /// Stores a changeset.
    ///
    /// # Arguments
    ///
    /// * `changeset` - The changeset to store
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful.
    ///
    /// # Errors
    ///
    /// This implementation never returns an error.
    fn store_changeset(&mut self, changeset: &Changeset) -> ChangeResult<()> {
        self.changesets.insert(changeset.id.clone(), changeset.clone());
        Ok(())
    }

    /// Removes a changeset.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the changeset to remove
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful.
    ///
    /// # Errors
    ///
    /// This implementation never returns an error.
    fn remove_changeset(&mut self, id: &ChangeId) -> ChangeResult<()> {
        self.changesets.remove(id);
        Ok(())
    }

    /// Gets all unreleased changes for a package.
    ///
    /// # Arguments
    ///
    /// * `package` - Name of the package
    ///
    /// # Returns
    ///
    /// A vector of unreleased changes for the specified package.
    ///
    /// # Errors
    ///
    /// This implementation never returns an error.
    fn get_unreleased_changes(&self, package: &str) -> ChangeResult<Vec<Change>> {
        // Collect all unreleased changes for the specified package
        let changes: Vec<Change> = self
            .changesets
            .values()
            .flat_map(|cs| {
                cs.changes
                    .iter()
                    .filter(|c| c.package == package && c.release_version.is_none())
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect();

        Ok(changes)
    }

    /// Gets all released changes for a package.
    ///
    /// # Arguments
    ///
    /// * `package` - Name of the package
    ///
    /// # Returns
    ///
    /// A vector of released changes for the specified package.
    ///
    /// # Errors
    ///
    /// This implementation never returns an error.
    fn get_released_changes(&self, package: &str) -> ChangeResult<Vec<Change>> {
        // Collect all released changes for the specified package
        let changes: Vec<Change> = self
            .changesets
            .values()
            .flat_map(|cs| {
                cs.changes
                    .iter()
                    .filter(|c| c.package == package && c.release_version.is_some())
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect();

        Ok(changes)
    }

    /// Gets all changes for a package grouped by version.
    ///
    /// # Arguments
    ///
    /// * `package` - Name of the package
    ///
    /// # Returns
    ///
    /// A hashmap where keys are version strings and values are vectors of changes.
    ///
    /// # Errors
    ///
    /// This implementation never returns an error.
    fn get_changes_by_version(&self, package: &str) -> ChangeResult<HashMap<String, Vec<Change>>> {
        let mut result: HashMap<String, Vec<Change>> = HashMap::new();

        // Add "unreleased" group
        let unreleased = self.get_unreleased_changes(package)?;
        if !unreleased.is_empty() {
            result.insert("unreleased".to_string(), unreleased);
        }

        // Add groups by version
        for changeset in self.changesets.values() {
            for change in &changeset.changes {
                if change.package == package && change.release_version.is_some() {
                    let version = change.release_version.clone().unwrap();
                    result.entry(version).or_default().push(change.clone());
                }
            }
        }

        Ok(result)
    }

    /// Marks changes as released.
    ///
    /// # Arguments
    ///
    /// * `package` - Name of the package to mark changes for
    /// * `version` - Version string to assign to the changes
    /// * `dry_run` - If true, only preview changes without applying them
    ///
    /// # Returns
    ///
    /// A vector of changes that were or would be marked as released.
    ///
    /// # Errors
    ///
    /// This implementation never returns an error.
    fn mark_changes_as_released(
        &mut self,
        package: &str,
        version: &str,
        dry_run: bool,
    ) -> ChangeResult<Vec<Change>> {
        let mut updated_changes = Vec::new();

        // Skip processing if this is a dry run
        if dry_run {
            // Only collect changes that would be marked
            for changeset in self.changesets.values() {
                for change in &changeset.changes {
                    if change.package == package && change.release_version.is_none() {
                        updated_changes.push(change.clone());
                    }
                }
            }
            return Ok(updated_changes);
        }

        // Find all changesets with unreleased changes for this package
        let changeset_ids: Vec<ChangeId> = self
            .changesets
            .iter()
            .filter(|(_, cs)| {
                cs.changes.iter().any(|c| c.package == package && c.release_version.is_none())
            })
            .map(|(id, _)| id.clone())
            .collect();

        // Update each changeset
        for id in changeset_ids {
            if let Some(mut changeset) = self.changesets.get(&id).cloned() {
                for change in &mut changeset.changes {
                    if change.package == package && change.release_version.is_none() {
                        change.release_version = Some(version.to_string());
                        updated_changes.push(change.clone());
                    }
                }
                self.changesets.insert(id, changeset);
            }
        }

        Ok(updated_changes)
    }

    /// Gets all changes grouped by package.
    ///
    /// # Returns
    ///
    /// A hashmap where keys are package names and values are vectors of all changes
    /// for that package.
    ///
    /// # Errors
    ///
    /// This implementation never returns an error.
    fn get_all_changes_by_package(&self) -> ChangeResult<HashMap<String, Vec<Change>>> {
        let mut result: HashMap<String, Vec<Change>> = HashMap::new();

        // Collect all changes across all changesets, grouped by package
        for changeset in self.changesets.values() {
            for change in &changeset.changes {
                result.entry(change.package.clone()).or_default().push(change.clone());
            }
        }

        Ok(result)
    }
}
