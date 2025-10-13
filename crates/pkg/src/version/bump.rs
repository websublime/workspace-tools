use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::error::VersionError;

/// Version bump types for semantic versioning.
///
/// Defines the types of version increments that can be applied
/// based on conventional commits or explicit requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VersionBump {
    /// Major version bump (breaking changes)
    Major,
    /// Minor version bump (new features)
    Minor,
    /// Patch version bump (bug fixes)
    Patch,
    /// No version bump required
    None,
}

impl FromStr for VersionBump {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "major" => Ok(Self::Major),
            "minor" => Ok(Self::Minor),
            "patch" => Ok(Self::Patch),
            "none" => Ok(Self::None),
            _ => Err(VersionError::InvalidFormat {
                version: s.to_string(),
                reason: "Valid values are 'major', 'minor', 'patch', or 'none'".to_string(),
            }),
        }
    }
}

impl fmt::Display for VersionBump {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl VersionBump {
    /// Gets the string representation of the version bump.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Patch => "patch",
            Self::None => "none",
        }
    }

    /// Combines two version bumps, returning the higher precedence bump.
    ///
    /// Major > Minor > Patch > None
    ///
    /// # Arguments
    ///
    /// * `other` - The other version bump to combine with
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::VersionBump;
    ///
    /// let combined = VersionBump::Patch.combine(VersionBump::Minor);
    /// assert_eq!(combined, VersionBump::Minor);
    ///
    /// let combined = VersionBump::Major.combine(VersionBump::Patch);
    /// assert_eq!(combined, VersionBump::Major);
    /// ```
    #[must_use]
    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Major, _) | (_, Self::Major) => Self::Major,
            (Self::Minor, _) | (_, Self::Minor) => Self::Minor,
            (Self::Patch, _) | (_, Self::Patch) => Self::Patch,
            (Self::None, Self::None) => Self::None,
        }
    }

    /// Checks if this bump is higher precedence than another.
    ///
    /// # Arguments
    ///
    /// * `other` - The other version bump to compare against
    #[must_use]
    pub fn is_higher_than(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Major, Self::Minor | Self::Patch | Self::None)
                | (Self::Minor, Self::Patch | Self::None)
                | (Self::Patch, Self::None)
        )
    }
}
