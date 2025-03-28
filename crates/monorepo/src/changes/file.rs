use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use crate::{Change, ChangeError, ChangeId, ChangeResult, ChangeStore, Changeset};

pub struct FileChangeStore {
    /// Path to the directory containing changesets.
    changeset_dir: PathBuf,
    /// Cache of loaded changesets.
    changesets: HashMap<ChangeId, Changeset>,
}

impl FileChangeStore {
    /// Creates a new file-based change store.
    ///
    /// # Errors
    /// Returns an error if the changeset directory cannot be created.
    pub fn new<P: AsRef<Path>>(changeset_dir: P) -> ChangeResult<Self> {
        let changeset_dir = changeset_dir.as_ref().to_path_buf();

        // Create directory if it doesn't exist
        if !changeset_dir.exists() {
            fs::create_dir_all(&changeset_dir).map_err(|error| {
                ChangeError::DirectoryCreationError { path: changeset_dir.clone(), error }
            })?;
        }

        let mut store = Self { changeset_dir, changesets: HashMap::new() };

        // Load existing changesets
        store.load_changesets()?;

        Ok(store)
    }

    /// Loads all changesets from the changeset directory.
    fn load_changesets(&mut self) -> ChangeResult<()> {
        // Clear the cache
        self.changesets.clear();

        // Check if directory exists
        if !self.changeset_dir.exists() {
            return Ok(());
        }

        // Read all .json files in the directory
        let entries = fs::read_dir(&self.changeset_dir)
            .map_err(|error| ChangeError::ListError { path: self.changeset_dir.clone(), error })?;

        for entry in entries {
            let entry = entry.map_err(|error| ChangeError::ListError {
                path: self.changeset_dir.clone(),
                error,
            })?;

            let path = entry.path();
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                // Load and parse the changeset
                let changeset = FileChangeStore::load_changeset_from_file(&path)?;
                self.changesets.insert(changeset.id.clone(), changeset);
            }
        }

        Ok(())
    }

    /// Loads a changeset from a file.
    fn load_changeset_from_file(path: &Path) -> ChangeResult<Changeset> {
        // Open the file
        let file = File::open(path)
            .map_err(|error| ChangeError::ReadError { path: path.to_path_buf(), error })?;

        // Parse the JSON
        let reader = BufReader::new(file);
        let changeset: Changeset = serde_json::from_reader(reader)
            .map_err(|error| ChangeError::ParseError { path: path.to_path_buf(), error })?;

        Ok(changeset)
    }

    /// Saves a changeset to a file.
    fn save_changeset_to_file(&self, changeset: &Changeset) -> ChangeResult<()> {
        // Create filename from changeset ID
        let filename = format!("{}.json", changeset.id);
        let path = self.changeset_dir.join(filename);

        // Create or overwrite the file
        let file = File::create(&path)
            .map_err(|error| ChangeError::WriteError { path: path.clone(), error })?;

        // Write the JSON
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, changeset).map_err(ChangeError::SerializeError)?;

        Ok(())
    }

    /// Removes a changeset file.
    fn remove_changeset_file(&self, id: &ChangeId) -> ChangeResult<()> {
        // Create filename from changeset ID
        let filename = format!("{id}.json");
        let path = self.changeset_dir.join(filename);

        // Remove the file if it exists
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|error| ChangeError::WriteError { path: path.clone(), error })?;
        }

        Ok(())
    }
}

impl ChangeStore for FileChangeStore {
    fn get_changeset(&self, id: &ChangeId) -> ChangeResult<Option<Changeset>> {
        Ok(self.changesets.get(id).cloned())
    }

    fn get_all_changesets(&self) -> ChangeResult<Vec<Changeset>> {
        Ok(self.changesets.values().cloned().collect())
    }

    fn store_changeset(&mut self, changeset: &Changeset) -> ChangeResult<()> {
        // Save to file
        self.save_changeset_to_file(changeset)?;

        // Update in-memory cache
        self.changesets.insert(changeset.id.clone(), changeset.clone());

        Ok(())
    }

    fn remove_changeset(&mut self, id: &ChangeId) -> ChangeResult<()> {
        // Remove from file system
        self.remove_changeset_file(id)?;

        // Remove from in-memory cache
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
        let mut updated_changesets = Vec::new();

        // Find all changesets with unreleased changes for this package
        for changeset in self.changesets.values() {
            let mut has_updates = false;
            let mut updated_changeset = changeset.clone();

            // Update changes within this changeset
            for change in &mut updated_changeset.changes {
                if change.package == package && change.release_version.is_none() {
                    change.release_version = Some(version.to_string());
                    updated_changes.push(change.clone());
                    has_updates = true;
                }
            }

            // If changes were updated, queue for storage
            if has_updates {
                updated_changesets.push(updated_changeset);
            }
        }

        // Store updated changesets if not a dry run
        if !dry_run {
            for changeset in updated_changesets {
                self.store_changeset(&changeset)?;
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
