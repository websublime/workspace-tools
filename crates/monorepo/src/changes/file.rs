//! File-based change store implementation.
//!
//! This module provides a file-based implementation of the `ChangeStore` trait,
//! allowing changes and changesets to be persisted to and loaded from the filesystem.
//! Each changeset is stored as a separate JSON file.

use crate::{Change, ChangeError, ChangeId, ChangeResult, ChangeStore, Changeset};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

/// File-based implementation of `ChangeStore`.
///
/// This implementation stores changesets as individual JSON files in a specified directory.
/// It provides in-memory caching to improve performance while ensuring changes are
/// safely persisted to disk.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use sublime_monorepo_tools::{FileChangeStore, ChangeStore, Change, ChangeType, Changeset};
///
/// // Create a file-based store
/// let mut store = FileChangeStore::new(Path::new(".changes")).unwrap();
///
/// // Create and store a change
/// let change = Change::new("ui", ChangeType::Feature, "Add button", false);
/// let changeset = Changeset::new::<String>(None, vec![change]);
/// store.store_changeset(&changeset).unwrap();
///
/// // Retrieve the change later
/// let retrieved = store.get_changeset(&changeset.id).unwrap().unwrap();
/// ```
pub struct FileChangeStore {
    /// Path to the directory containing changesets.
    changeset_dir: PathBuf,
    /// Cache of loaded changesets.
    changesets: HashMap<ChangeId, Changeset>,
}

impl FileChangeStore {
    /// Creates a new file-based change store.
    ///
    /// # Arguments
    ///
    /// * `changeset_dir` - Path to the directory where changesets will be stored
    ///
    /// # Returns
    ///
    /// A new `FileChangeStore` instance, or an error if the directory cannot be created.
    ///
    /// # Errors
    ///
    /// Returns an error if the changeset directory cannot be created.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use sublime_monorepo_tools::FileChangeStore;
    ///
    /// // Create store in .changes directory
    /// let store = FileChangeStore::new(Path::new(".changes")).unwrap();
    ///
    /// // Create store in a custom directory
    /// let custom_store = FileChangeStore::new(Path::new("custom/path/to/changes")).unwrap();
    /// ```
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
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error if loading fails.
    ///
    /// # Errors
    ///
    /// Returns an error if reading the directory or parsing any changeset file fails.
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
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the changeset file
    ///
    /// # Returns
    ///
    /// The loaded `Changeset`, or an error if reading or parsing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if parsing the JSON fails.
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
    ///
    /// # Arguments
    ///
    /// * `changeset` - The changeset to save
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error if writing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or if serializing the changeset fails.
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
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the changeset to remove
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error if removing the file fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be removed.
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
    /// Returns an error if accessing the store fails.
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
    /// `Ok(())` if successful, or an error if storing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if writing the changeset to disk fails.
    fn store_changeset(&mut self, changeset: &Changeset) -> ChangeResult<()> {
        // Save to file
        self.save_changeset_to_file(changeset)?;

        // Update in-memory cache
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
    /// `Ok(())` if successful, or an error if removal fails.
    ///
    /// # Errors
    ///
    /// Returns an error if removing the changeset file fails.
    fn remove_changeset(&mut self, id: &ChangeId) -> ChangeResult<()> {
        // Remove from file system
        self.remove_changeset_file(id)?;

        // Remove from in-memory cache
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
    /// Returns an error if accessing the store fails.
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
    /// Returns an error if accessing the store fails.
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
    /// Unreleased changes are grouped under the key "unreleased".
    ///
    /// # Errors
    ///
    /// Returns an error if accessing the store fails.
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
    /// Returns an error if accessing or updating the store fails.
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
