use semver::Version;
use serde::{Deserialize, Serialize};

use crate::ChangeType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyChange {
    /// Name of the dependency
    pub name: String,
    /// Previous version (None if newly added)
    pub previous_version: Option<String>,
    /// Current version (None if removed)
    pub current_version: Option<String>,
    /// Type of change
    pub change_type: ChangeType,
    /// Whether this is a breaking change based on semver
    pub breaking: bool,
}

impl DependencyChange {
    /// Creates a new dependency change
    pub fn new(
        name: &str,
        previous_version: Option<&str>,
        current_version: Option<&str>,
        change_type: ChangeType,
    ) -> Self {
        // Determine if this is a breaking change based on semver
        let breaking = match (previous_version, current_version) {
            (Some(prev), Some(curr)) => {
                // Clean up the version strings
                let clean_prev = prev.trim_start_matches('^').trim_start_matches('~');
                let clean_curr = curr.trim_start_matches('^').trim_start_matches('~');

                if let (Ok(prev_ver), Ok(curr_ver)) =
                    (Version::parse(clean_prev), Version::parse(clean_curr))
                {
                    // Breaking if major version increases
                    curr_ver.major > prev_ver.major
                } else {
                    // If we can't parse the version, conservatively assume it might be breaking
                    true
                }
            }
            // Only removals are considered breaking changes, not additions
            (Some(_), None) => true, // Removed dependency is breaking
            (None, Some(_) | None) => false, // Added dependency is not breaking
        };

        Self {
            name: name.to_string(),
            previous_version: previous_version.map(String::from),
            current_version: current_version.map(String::from),
            change_type,
            breaking,
        }
    }
}
