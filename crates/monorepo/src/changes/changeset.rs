//! Changeset implementation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{Change, ChangeId};

/// Collection of related changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changeset {
    /// Unique identifier
    #[serde(default = "ChangeId::new")]
    pub id: ChangeId,

    /// Summary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Changes in this set
    pub changes: Vec<Change>,

    /// Creation timestamp
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

impl Changeset {
    /// Creates a new changeset.
    #[must_use]
    pub fn new<S: Into<String>>(summary: Option<S>, changes: Vec<Change>) -> Self {
        Self {
            id: ChangeId::new(),
            summary: summary.map(Into::into),
            changes,
            created_at: Utc::now(),
        }
    }

    /// Checks if all changes in the changeset are released.
    #[must_use]
    pub fn is_released(&self) -> bool {
        !self.changes.is_empty() && self.changes.iter().all(Change::is_released)
    }

    /// Gets the package names included in this changeset.
    #[must_use]
    pub fn package_names(&self) -> Vec<String> {
        self.changes.iter().map(|c| c.package.clone()).collect::<HashSet<_>>().into_iter().collect()
    }
}
