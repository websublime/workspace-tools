use std::collections::HashMap;

use crate::{Change, ChangeId, ChangeResult, Changeset};

pub trait ChangeStore {
    /// Gets a changeset by ID.
    fn get_changeset(&self, id: &ChangeId) -> ChangeResult<Option<Changeset>>;

    /// Gets all changesets.
    fn get_all_changesets(&self) -> ChangeResult<Vec<Changeset>>;

    /// Stores a changeset.
    fn store_changeset(&mut self, changeset: &Changeset) -> ChangeResult<()>;

    /// Removes a changeset.
    fn remove_changeset(&mut self, id: &ChangeId) -> ChangeResult<()>;

    /// Gets all unreleased changes for a package.
    fn get_unreleased_changes(&self, package: &str) -> ChangeResult<Vec<Change>>;

    /// Gets all released changes for a package.
    fn get_released_changes(&self, package: &str) -> ChangeResult<Vec<Change>>;

    /// Gets all changes for a package grouped by version.
    fn get_changes_by_version(&self, package: &str) -> ChangeResult<HashMap<String, Vec<Change>>>;

    /// Marks changes as released.
    fn mark_changes_as_released(
        &mut self,
        package: &str,
        version: &str,
        dry_run: bool,
    ) -> ChangeResult<Vec<Change>>;

    /// Gets all changes grouped by package.
    fn get_all_changes_by_package(&self) -> ChangeResult<HashMap<String, Vec<Change>>>;
}
