//! Versioning configuration types

use serde::{Deserialize, Serialize};

/// Versioning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersioningConfig {
    /// Default bump type when not specified
    pub default_bump: VersionBumpType,

    /// Whether to propagate version changes to dependents
    pub propagate_changes: bool,

    /// Snapshot version format
    pub snapshot_format: String,

    /// Version tag prefix
    pub tag_prefix: String,

    /// Whether to create tags automatically
    pub auto_tag: bool,
}

impl Default for VersioningConfig {
    fn default() -> Self {
        Self {
            default_bump: VersionBumpType::Patch,
            propagate_changes: true,
            snapshot_format: "{version}-snapshot.{sha}".to_string(),
            tag_prefix: "v".to_string(),
            auto_tag: true,
        }
    }
}

/// Type of version bump
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionBumpType {
    /// Major version bump (x.0.0)
    Major,
    /// Minor version bump (0.x.0)
    Minor,
    /// Patch version bump (0.0.x)
    Patch,
    /// Snapshot version with commit SHA
    Snapshot,
}