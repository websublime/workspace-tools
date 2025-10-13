use std::path::PathBuf;

use crate::{changeset::Changeset, error::ChangesetError, PackageResult};

/// Changeset manager service.
///
/// Provides high-level operations for changeset management.
#[allow(dead_code)]
pub struct ChangesetManager {
    pub(crate) changeset_path: PathBuf,
    pub(crate) history_path: PathBuf,
}

impl ChangesetManager {
    /// Creates a new changeset manager.
    ///
    /// # Arguments
    ///
    /// * `changeset_path` - Directory where changesets are stored
    /// * `history_path` - Directory where applied changesets are archived
    pub fn new(changeset_path: PathBuf, history_path: PathBuf) -> Self {
        Self { changeset_path, history_path }
    }

    /// Lists all pending changesets.
    pub async fn list_pending(&self) -> PackageResult<Vec<String>> {
        // TODO: Implement in future stories
        Ok(vec![])
    }

    /// Lists all applied changesets from history.
    pub async fn list_history(&self) -> PackageResult<Vec<String>> {
        // TODO: Implement in future stories
        Ok(vec![])
    }

    /// Creates a new changeset.
    ///
    /// # Arguments
    ///
    /// * `changeset` - The changeset to create
    pub async fn create(&self, _changeset: &Changeset) -> PackageResult<String> {
        // TODO: Implement in future stories
        Err(ChangesetError::CreationFailed {
            branch: "unknown".to_string(),
            reason: "Not implemented yet".to_string(),
        }
        .into())
    }

    /// Applies a changeset.
    ///
    /// # Arguments
    ///
    /// * `changeset_id` - ID of the changeset to apply
    pub async fn apply(&self, _changeset_id: &str) -> PackageResult<()> {
        // TODO: Implement in future stories
        Err(ChangesetError::ApplicationFailed {
            changeset_id: "unknown".to_string(),
            reason: "Not implemented yet".to_string(),
        }
        .into())
    }
}
