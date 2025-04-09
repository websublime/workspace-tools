//! Change store interface for persistently tracking changes.
//!
//! This module defines the `ChangeStore` trait, which is the interface for
//! storing, retrieving, and managing changes across implementations.
//! Different storage backends (file-based, memory-based, database, etc.)
//! can implement this trait to provide consistent access to change data.

use crate::{Change, ChangeId, ChangeResult, Changeset};
use std::collections::HashMap;

/// Interface for storing and retrieving changes and changesets.
///
/// This trait defines the operations that any change storage implementation
/// must support, allowing for different storage backends while maintaining
/// a consistent interface.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::{
///     ChangeStore, Change, ChangeType, Changeset, MemoryChangeStore
/// };
///
/// // Create any store implementing ChangeStore
/// let mut store: Box<dyn ChangeStore> = Box::new(MemoryChangeStore::new());
///
/// // Create and store a change
/// let change = Change::new("ui", ChangeType::Feature, "Add button", false);
/// let changeset = Changeset::new::<String>(None, vec![change]);
/// store.store_changeset(&changeset).unwrap();
///
/// // Get unreleased changes for a package
/// let unreleased = store.get_unreleased_changes("ui").unwrap();
/// ```
pub trait ChangeStore {
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
    /// Returns an error if accessing the store fails.
    fn get_changeset(&self, id: &ChangeId) -> ChangeResult<Option<Changeset>>;

    /// Gets all changesets.
    ///
    /// # Returns
    ///
    /// A vector of all changesets in the store.
    ///
    /// # Errors
    ///
    /// Returns an error if accessing the store fails.
    fn get_all_changesets(&self) -> ChangeResult<Vec<Changeset>>;

    /// Stores a changeset.
    ///
    /// # Arguments
    ///
    /// * `changeset` - The changeset to store
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error if storing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if storing the changeset fails.
    fn store_changeset(&mut self, changeset: &Changeset) -> ChangeResult<()>;

    /// Removes a changeset.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the changeset to remove
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error if removal fails.
    ///
    /// # Errors
    ///
    /// Returns an error if removing the changeset fails.
    fn remove_changeset(&mut self, id: &ChangeId) -> ChangeResult<()>;

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
    /// Returns an error if accessing the store fails.
    fn get_unreleased_changes(&self, package: &str) -> ChangeResult<Vec<Change>>;

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
    /// Returns an error if accessing the store fails.
    fn get_released_changes(&self, package: &str) -> ChangeResult<Vec<Change>>;

    /// Gets all changes for a package grouped by version.
    ///
    /// # Arguments
    ///
    /// * `package` - Name of the package
    ///
    /// # Returns
    ///
    /// A hashmap where keys are version strings and values are vectors of changes.
    /// Unreleased changes are grouped under the key "unreleased".
    ///
    /// # Errors
    ///
    /// Returns an error if accessing the store fails.
    fn get_changes_by_version(&self, package: &str) -> ChangeResult<HashMap<String, Vec<Change>>>;

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
    /// Returns an error if accessing or updating the store fails.
    fn mark_changes_as_released(
        &mut self,
        package: &str,
        version: &str,
        dry_run: bool,
    ) -> ChangeResult<Vec<Change>>;

    /// Gets all changes grouped by package.
    ///
    /// # Returns
    ///
    /// A hashmap where keys are package names and values are vectors of all changes
    /// for that package.
    ///
    /// # Errors
    ///
    /// Returns an error if accessing the store fails.
    fn get_all_changes_by_package(&self) -> ChangeResult<HashMap<String, Vec<Change>>>;

    /// Gets changes for a package that apply to a specific environment.
    ///
    /// # Arguments
    ///
    /// * `package` - Name of the package
    /// * `environment` - Name of the environment (e.g., "production", "staging")
    ///
    /// # Returns
    ///
    /// A vector of changes for the specified package that apply to the given environment.
    ///
    /// # Errors
    ///
    /// Returns an error if accessing the store fails.
    fn get_changes_for_environment(
        &self,
        package: &str,
        environment: &str,
    ) -> ChangeResult<Vec<Change>> {
        let all_changes =
            self.get_all_changes_by_package()?.get(package).cloned().unwrap_or_default();

        // Filter changes that apply to this environment
        Ok(all_changes.into_iter().filter(|c| c.applies_to_environment(environment)).collect())
    }

    /// Gets unreleased changes for a package that apply to a specific environment.
    ///
    /// # Arguments
    ///
    /// * `package` - Name of the package
    /// * `environment` - Name of the environment (e.g., "production", "staging")
    ///
    /// # Returns
    ///
    /// A vector of unreleased changes for the specified package that apply to the given environment.
    ///
    /// # Errors
    ///
    /// Returns an error if accessing the store fails.
    fn get_unreleased_changes_for_environment(
        &self,
        package: &str,
        environment: &str,
    ) -> ChangeResult<Vec<Change>> {
        let unreleased = self.get_unreleased_changes(package)?;

        // Filter unreleased changes that apply to this environment
        Ok(unreleased.into_iter().filter(|c| c.applies_to_environment(environment)).collect())
    }

    /// Gets all changes by environment.
    ///
    /// # Arguments
    ///
    /// * `environment` - Name of the environment (e.g., "production", "staging")
    ///
    /// # Returns
    ///
    /// A hashmap where keys are package names and values are vectors of changes
    /// that apply to the specified environment.
    ///
    /// # Errors
    ///
    /// Returns an error if accessing the store fails.
    fn get_changes_by_environment(
        &self,
        environment: &str,
    ) -> ChangeResult<HashMap<String, Vec<Change>>> {
        let all_by_package = self.get_all_changes_by_package()?;
        let mut by_env = HashMap::new();

        for (package, changes) in all_by_package {
            let env_changes: Vec<Change> =
                changes.into_iter().filter(|c| c.applies_to_environment(environment)).collect();

            if !env_changes.is_empty() {
                by_env.insert(package, env_changes);
            }
        }

        Ok(by_env)
    }
}
