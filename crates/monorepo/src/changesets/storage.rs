//! Changeset storage implementation
//!
//! This module provides persistent storage for changesets using the `FileSystemManager`
//! from standard-tools. Changesets are stored as JSON files in the configured
//! changeset directory with structured naming for easy querying.

use serde_json;
use std::path::PathBuf;

use super::types::{Changeset, ChangesetFilter, ChangesetStorage};
use crate::error::Error;
use sublime_standard_tools::filesystem::FileSystem;

impl<'a> ChangesetStorage<'a> {
    /// Creates a new changeset storage instance
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to the monorepo project
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use std::path::Path;
    /// use sublime_monorepo_tools::{ChangesetStorage, MonorepoProject};
    ///
    /// let project = Arc::new(MonorepoProject::new(Path::new("/project"))?);
    /// let storage = ChangesetStorage::new(project);
    /// ```
    /// Creates a new changeset storage with direct borrowing from project
    ///
    /// Uses borrowing instead of trait objects to eliminate Arc proliferation
    /// and work with Rust ownership principles.
    ///
    /// # Arguments
    ///
    /// * `config` - Changeset configuration
    /// * `file_system` - Direct reference to file system manager
    /// * `root_path` - Direct reference to root path
    ///
    /// # Returns
    ///
    /// A new changeset storage instance
    #[must_use]
    pub fn new(
        config: crate::config::types::ChangesetsConfig,
        file_system: &'a sublime_standard_tools::filesystem::FileSystemManager,
        root_path: &'a std::path::Path,
    ) -> Self {
        Self { config, file_system, root_path }
    }

    /// Saves a changeset to storage
    ///
    /// Serializes the changeset to JSON and saves it to the configured
    /// changeset directory with a filename based on the configured format.
    ///
    /// # Arguments
    ///
    /// * `changeset` - The changeset to save
    ///
    /// # Returns
    ///
    /// Result indicating success or failure of the save operation.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The changeset directory cannot be created
    /// - The changeset cannot be serialized
    /// - The file cannot be written
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use sublime_monorepo_tools::{ChangesetStorage, Changeset};
    /// # let storage = create_test_storage();
    /// # let changeset = create_test_changeset();
    /// storage.save(&changeset).await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn save(&self, changeset: &Changeset) -> Result<(), Error> {
        // Ensure changeset directory exists
        let changeset_dir = self.get_changeset_directory();
        self.file_system
            .create_dir_all(&changeset_dir)
            .map_err(|e| Error::changeset(format!("Failed to create changeset directory: {e}")))?;

        // Generate filename
        let filename = self.generate_filename(changeset);
        let filepath = changeset_dir.join(filename);

        // Serialize changeset
        let content = serde_json::to_string_pretty(changeset)
            .map_err(|e| Error::changeset(format!("Failed to serialize changeset: {e}")))?;

        // Write to file
        self.file_system
            .write_file(&filepath, content.as_bytes())
            .map_err(|e| Error::changeset(format!("Failed to write changeset file: {e}")))?;

        Ok(())
    }

    /// Loads a changeset by ID
    ///
    /// Searches for a changeset file with the given ID and loads it.
    ///
    /// # Arguments
    ///
    /// * `id` - The changeset ID to load
    ///
    /// # Returns
    ///
    /// The changeset if found, or None if not found.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or parsed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use sublime_monorepo_tools::ChangesetStorage;
    /// # let storage = create_test_storage();
    /// if let Some(changeset) = storage.load("abc123").await? {
    ///     println!("Found changeset: {}", changeset.description);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load(&self, id: &str) -> Result<Option<Changeset>, Error> {
        let changeset_dir = self.get_changeset_directory();

        // Find file with this ID
        if !self.file_system.exists(&changeset_dir) {
            // Directory doesn't exist, so no changesets
            return Ok(None);
        }

        let files = self
            .file_system
            .walk_dir(&changeset_dir)
            .map_err(|e| Error::changeset(format!("Failed to list changeset files: {e}")))?
            .into_iter()
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
            .collect::<Vec<_>>();

        for file in files {
            if let Some(filename) = file.file_name().and_then(|n| n.to_str()) {
                // Try to match either the full ID or the short hash (first 8 characters)
                let short_id = if id.len() > 8 { &id[..8] } else { id };
                if filename.contains(id) || filename.contains(short_id) {
                    let content = self.file_system.read_file(&file).map_err(|e| {
                        Error::changeset(format!("Failed to read changeset file: {e}"))
                    })?;
                    let content = String::from_utf8(content).map_err(|e| {
                        Error::changeset(format!("Invalid UTF-8 in changeset file: {e}"))
                    })?;

                    let changeset: Changeset = serde_json::from_str(&content)
                        .map_err(|e| Error::changeset(format!("Failed to parse changeset: {e}")))?;

                    if changeset.id == id {
                        return Ok(Some(changeset));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Lists all changesets matching the given filter
    ///
    /// Scans the changeset directory and returns all changesets that match
    /// the provided filter criteria.
    ///
    /// # Arguments
    ///
    /// * `filter` - Filter criteria for changesets
    ///
    /// # Returns
    ///
    /// Vector of changesets matching the filter.
    ///
    /// # Errors
    ///
    /// Returns an error if the changeset directory cannot be read or
    /// if changeset files cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use sublime_monorepo_tools::{ChangesetStorage, ChangesetFilter};
    /// # let storage = create_test_storage();
    /// let filter = ChangesetFilter {
    ///     package: Some("@test/core".to_string()),
    ///     ..Default::default()
    /// };
    /// let changesets = storage.list(&filter).await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn list(&self, filter: &ChangesetFilter) -> Result<Vec<Changeset>, Error> {
        let changeset_dir = self.get_changeset_directory();

        // Get all JSON files in the changeset directory
        if !self.file_system.exists(&changeset_dir) {
            // Directory doesn't exist, so no changesets
            return Ok(Vec::new());
        }

        let files = self
            .file_system
            .walk_dir(&changeset_dir)
            .map_err(|e| Error::changeset(format!("Failed to list changeset files: {e}")))?
            .into_iter()
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
            .collect::<Vec<_>>();

        let mut changesets = Vec::new();

        for file in files {
            let content = self
                .file_system
                .read_file_string(&file)
                .map_err(|e| Error::changeset(format!("Failed to read changeset file: {e}")))?;

            let changeset: Changeset = serde_json::from_str(&content)
                .map_err(|e| Error::changeset(format!("Failed to parse changeset: {e}")))?;

            if self.matches_filter(&changeset, filter) {
                changesets.push(changeset);
            }
        }

        // Sort by creation date (newest first)
        changesets.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(changesets)
    }

    /// Deletes a changeset by ID
    ///
    /// Removes the changeset file from storage.
    ///
    /// # Arguments
    ///
    /// * `id` - The changeset ID to delete
    ///
    /// # Returns
    ///
    /// True if the changeset was found and deleted, false if not found.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be deleted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use sublime_monorepo_tools::ChangesetStorage;
    /// # let storage = create_test_storage();
    /// let deleted = storage.delete("abc123").await?;
    /// if deleted {
    ///     println!("Changeset deleted successfully");
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn delete(&self, id: &str) -> Result<bool, Error> {
        let changeset_dir = self.get_changeset_directory();

        // Find file with this ID
        if !self.file_system.exists(&changeset_dir) {
            // Directory doesn't exist, so changeset doesn't exist
            return Ok(false);
        }

        let files = self
            .file_system
            .walk_dir(&changeset_dir)
            .map_err(|e| Error::changeset(format!("Failed to list changeset files: {e}")))?
            .into_iter()
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
            .collect::<Vec<_>>();

        for file in files {
            if let Some(filename) = file.file_name().and_then(|n| n.to_str()) {
                // Try to match either the full ID or the short hash (first 8 characters)
                let short_id = if id.len() > 8 { &id[..8] } else { id };
                if filename.contains(id) || filename.contains(short_id) {
                    // Verify this is the correct changeset
                    let content = self.file_system.read_file(&file).map_err(|e| {
                        Error::changeset(format!("Failed to read changeset file: {e}"))
                    })?;
                    let content = String::from_utf8(content).map_err(|e| {
                        Error::changeset(format!("Invalid UTF-8 in changeset file: {e}"))
                    })?;

                    let changeset: Changeset = serde_json::from_str(&content)
                        .map_err(|e| Error::changeset(format!("Failed to parse changeset: {e}")))?;

                    if changeset.id == id {
                        self.file_system.remove(&file).map_err(|e| {
                            Error::changeset(format!("Failed to delete changeset file: {e}"))
                        })?;
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Gets the full path to the changeset directory
    fn get_changeset_directory(&self) -> PathBuf {
        self.root_path.join(&self.config.changeset_dir)
    }

    /// Generates a filename for a changeset based on the configured format
    fn generate_filename(&self, changeset: &Changeset) -> String {
        let timestamp = changeset.created_at.timestamp();
        let short_hash = &changeset.id[..8]; // Use first 8 characters of ID as hash

        self.config
            .filename_format
            .replace("{timestamp}", &timestamp.to_string())
            .replace("{branch}", &changeset.branch.replace('/', "-"))
            .replace("{hash}", short_hash)
    }

    /// Checks if a changeset matches the given filter
    #[allow(clippy::unused_self)]
    fn matches_filter(&self, changeset: &Changeset, filter: &ChangesetFilter) -> bool {
        if let Some(ref package) = filter.package {
            if changeset.package != *package {
                return false;
            }
        }

        if let Some(ref status) = filter.status {
            if changeset.status != *status {
                return false;
            }
        }

        if let Some(ref environment) = filter.environment {
            if !changeset.development_environments.contains(environment) {
                return false;
            }
        }

        if let Some(ref branch) = filter.branch {
            if changeset.branch != *branch {
                return false;
            }
        }

        if let Some(ref author) = filter.author {
            if changeset.author != *author {
                return false;
            }
        }

        true
    }
}
