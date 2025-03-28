use std::collections::HashMap;

use crate::{Change, ChangeId, ChangeResult, ChangeStore, Changeset};

/// Memory-based implementation of ChangeStore.
pub struct MemoryChangeStore {
    /// In-memory storage for changesets.
    changesets: HashMap<ChangeId, Changeset>,
}

impl MemoryChangeStore {
    /// Creates a new memory-based change store.
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
    fn get_changeset(&self, id: &ChangeId) -> ChangeResult<Option<Changeset>> {
        Ok(self.changesets.get(id).cloned())
    }

    fn get_all_changesets(&self) -> ChangeResult<Vec<Changeset>> {
        Ok(self.changesets.values().cloned().collect())
    }

    fn store_changeset(&mut self, changeset: &Changeset) -> ChangeResult<()> {
        self.changesets.insert(changeset.id.clone(), changeset.clone());
        Ok(())
    }

    fn remove_changeset(&mut self, id: &ChangeId) -> ChangeResult<()> {
        self.changesets.remove(id);
        Ok(())
    }

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
