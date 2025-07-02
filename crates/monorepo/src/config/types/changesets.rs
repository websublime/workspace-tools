//! Changesets configuration types

use super::Environment;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Changesets configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesetsConfig {
    /// Whether changesets are required
    pub required: bool,

    /// Changeset directory
    pub changeset_dir: PathBuf,

    /// Default environments for new changesets
    pub default_environments: Vec<Environment>,

    /// Whether to auto-deploy to environments
    pub auto_deploy: bool,

    /// Changeset filename format
    pub filename_format: String,
}

impl Default for ChangesetsConfig {
    fn default() -> Self {
        Self {
            required: true,
            changeset_dir: PathBuf::from(".changesets"),
            default_environments: vec![Environment::Development, Environment::Staging],
            auto_deploy: false,
            filename_format: "{timestamp}-{branch}-{hash}.json".to_string(),
        }
    }
}
