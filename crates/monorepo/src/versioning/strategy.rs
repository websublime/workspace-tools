//! Version bump strategies for monorepo versioning.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Strategy for determining how package versions should be bumped.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum VersionBumpStrategy {
    /// All packages get the same version
    #[serde(rename_all = "camelCase")]
    Synchronized { version: String },

    /// Each package is bumped according to its changes
    #[serde(rename_all = "camelCase")]
    Independent { major_if_breaking: bool, minor_if_feature: bool, patch_otherwise: bool },

    /// Use conventional commit messages to determine bump types
    #[serde(rename_all = "camelCase")]
    ConventionalCommits { from_ref: Option<String> },

    /// Manually specified versions
    Manual(HashMap<String, String>),
}

impl Default for VersionBumpStrategy {
    fn default() -> Self {
        Self::Independent { major_if_breaking: true, minor_if_feature: true, patch_otherwise: true }
    }
}

/// Type of version bump to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BumpType {
    /// Major version bump (x.0.0)
    Major,
    /// Minor version bump (0.x.0)
    Minor,
    /// Patch version bump (0.0.x)
    Patch,
    /// Development snapshot
    Snapshot,
    /// No bump needed
    None,
}

impl From<BumpType> for sublime_package_tools::Version {
    fn from(bump_type: BumpType) -> Self {
        match bump_type {
            BumpType::Major => sublime_package_tools::Version::Major,
            BumpType::Minor => sublime_package_tools::Version::Minor,
            BumpType::Snapshot => sublime_package_tools::Version::Snapshot,
            BumpType::Patch | BumpType::None => sublime_package_tools::Version::Patch, // Default to patch if "None" needs conversion
        }
    }
}

impl std::fmt::Display for BumpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BumpType::Major => write!(f, "major"),
            BumpType::Minor => write!(f, "minor"),
            BumpType::Patch => write!(f, "patch"),
            BumpType::Snapshot => write!(f, "snapshot"),
            BumpType::None => write!(f, "none"),
        }
    }
}

/// Reason for a version bump.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BumpReason {
    /// Breaking change
    Breaking(String),
    /// New feature
    Feature(String),
    /// Bug fix
    Fix(String),
    /// Other change
    Other(String),
    /// Dependency update
    DependencyUpdate(String),
    /// Manual specification
    Manual,
}

/// Represents the result of a version bump for a package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageVersionChange {
    /// Package name
    pub package_name: String,
    /// Previous version
    pub previous_version: String,
    /// New version
    pub new_version: String,
    /// Type of bump performed
    pub bump_type: BumpType,
    /// Whether this was a dependency-only update
    pub is_dependency_update: bool,
    /// Whether this update was due to being in a cycle
    #[serde(default)]
    pub is_cycle_update: bool,
    /// Which cycle group this package belongs to (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle_group: Option<Vec<String>>,
}

// Implement default for backward compatibility
impl Default for PackageVersionChange {
    fn default() -> Self {
        Self {
            package_name: String::new(),
            previous_version: String::new(),
            new_version: String::new(),
            bump_type: BumpType::None,
            is_dependency_update: false,
            is_cycle_update: false,
            cycle_group: None,
        }
    }
}

/// Settings for changelog generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogOptions {
    /// Whether to update existing changelog files
    #[serde(default = "default_true")]
    pub update_existing: bool,

    /// Base filename for changelog files
    #[serde(default = "default_changelog_filename")]
    pub filename: String,

    /// Whether to include version details in the changelog
    #[serde(default = "default_true")]
    pub include_version_details: bool,

    /// Whether to include release date in the changelog
    #[serde(default = "default_true")]
    pub include_release_date: bool,

    /// Header template for the changelog
    #[serde(default = "default_header_template")]
    pub header_template: String,

    /// Template for individual changes
    #[serde(default = "default_change_template")]
    pub change_template: String,
}

impl Default for ChangelogOptions {
    fn default() -> Self {
        Self {
            update_existing: true,
            filename: default_changelog_filename(),
            include_version_details: true,
            include_release_date: true,
            header_template: default_header_template(),
            change_template: default_change_template(),
        }
    }
}

impl ChangelogOptions {
    /// Create a new changelog options object with defaults.
    pub fn new() -> Self {
        Self::default()
    }
}

fn default_true() -> bool {
    true
}

fn default_changelog_filename() -> String {
    "CHANGELOG.md".to_string()
}

fn default_header_template() -> String {
    "# Changelog\n\n".to_string()
}

fn default_change_template() -> String {
    "- {type}: {description} {breaking}\n".to_string()
}
