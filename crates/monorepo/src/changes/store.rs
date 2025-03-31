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

    /// Gets changes for a package that apply to a specific environment.
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
