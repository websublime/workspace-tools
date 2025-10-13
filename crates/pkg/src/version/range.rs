use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{error::VersionError, version::versioning::Version};

/// Version range specification for dependency constraints.
///
/// Supports semantic version range operators commonly used in package managers:
/// - Exact: `1.2.3` - matches exactly 1.2.3
/// - Caret: `^1.2.3` - compatible within major version (>=1.2.3 <2.0.0)
/// - Tilde: `~1.2.3` - reasonably close to version (>=1.2.3 <1.3.0)
/// - Greater: `>1.2.3` - strictly greater than 1.2.3
/// - GreaterOrEqual: `>=1.2.3` - greater than or equal to 1.2.3
/// - Less: `<1.2.3` - strictly less than 1.2.3
/// - LessOrEqual: `<=1.2.3` - less than or equal to 1.2.3
/// - Wildcard: `1.2.*` - matches any patch version in 1.2.x
/// - Range: `1.2.3 - 2.0.0` - inclusive range
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::version::{VersionRange, Version};
/// use std::str::FromStr;
///
/// let range = VersionRange::from_str("^1.2.3")?;
/// let version = Version::from_str("1.5.0")?;
///
/// assert!(range.matches(&version));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VersionRange {
    /// Exact version match: `1.2.3`
    Exact(Version),
    /// Caret range: `^1.2.3` (compatible within major version)
    Caret(Version),
    /// Tilde range: `~1.2.3` (reasonably close to version)
    Tilde(Version),
    /// Greater than: `>1.2.3`
    Greater(Version),
    /// Greater than or equal: `>=1.2.3`
    GreaterOrEqual(Version),
    /// Less than: `<1.2.3`
    Less(Version),
    /// Less than or equal: `<=1.2.3`
    LessOrEqual(Version),
    /// Wildcard pattern: `1.2.*`
    Wildcard {
        /// Major version component
        major: u64,
        /// Minor version component (None for major wildcards)
        minor: Option<u64>,
    },
    /// Inclusive range: `1.2.3 - 2.0.0`
    Range {
        /// Lower bound (inclusive)
        min: Version,
        /// Upper bound (inclusive)
        max: Version,
    },
    /// Any version: `*`
    Any,
}

impl FromStr for VersionRange {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        // Handle any/wildcard
        if trimmed == "*" {
            return Ok(Self::Any);
        }

        // Handle range (contains " - ")
        if let Some(dash_pos) = trimmed.find(" - ") {
            let min_str = &trimmed[..dash_pos].trim();
            let max_str = &trimmed[dash_pos + 3..].trim();

            let min = Version::from_str(min_str).map_err(|e| VersionError::InvalidFormat {
                version: s.to_string(),
                reason: format!("Invalid minimum version in range: {}", e),
            })?;
            let max = Version::from_str(max_str).map_err(|e| VersionError::InvalidFormat {
                version: s.to_string(),
                reason: format!("Invalid maximum version in range: {}", e),
            })?;

            return Ok(Self::Range { min, max });
        }

        // Handle comparison operators
        if let Some(stripped) = trimmed.strip_prefix(">=") {
            let version_str = stripped.trim();
            let version =
                Version::from_str(version_str).map_err(|e| VersionError::InvalidFormat {
                    version: s.to_string(),
                    reason: format!("Invalid version after '>=': {}", e),
                })?;
            return Ok(Self::GreaterOrEqual(version));
        }

        if let Some(stripped) = trimmed.strip_prefix("<=") {
            let version_str = stripped.trim();
            let version =
                Version::from_str(version_str).map_err(|e| VersionError::InvalidFormat {
                    version: s.to_string(),
                    reason: format!("Invalid version after '<=': {}", e),
                })?;
            return Ok(Self::LessOrEqual(version));
        }

        if let Some(stripped) = trimmed.strip_prefix('>') {
            let version_str = stripped.trim();
            let version =
                Version::from_str(version_str).map_err(|e| VersionError::InvalidFormat {
                    version: s.to_string(),
                    reason: format!("Invalid version after '>': {}", e),
                })?;
            return Ok(Self::Greater(version));
        }

        if let Some(stripped) = trimmed.strip_prefix('<') {
            let version_str = stripped.trim();
            let version =
                Version::from_str(version_str).map_err(|e| VersionError::InvalidFormat {
                    version: s.to_string(),
                    reason: format!("Invalid version after '<': {}", e),
                })?;
            return Ok(Self::Less(version));
        }

        // Handle caret range
        if let Some(stripped) = trimmed.strip_prefix('^') {
            let version_str = stripped.trim();
            let version =
                Version::from_str(version_str).map_err(|e| VersionError::InvalidFormat {
                    version: s.to_string(),
                    reason: format!("Invalid version after '^': {}", e),
                })?;
            return Ok(Self::Caret(version));
        }

        // Handle tilde range
        if let Some(stripped) = trimmed.strip_prefix('~') {
            let version_str = stripped.trim();
            let version =
                Version::from_str(version_str).map_err(|e| VersionError::InvalidFormat {
                    version: s.to_string(),
                    reason: format!("Invalid version after '~': {}", e),
                })?;
            return Ok(Self::Tilde(version));
        }

        // Check for wildcard patterns
        if trimmed.contains('*') {
            return Self::parse_wildcard(trimmed);
        }

        // Default to exact version
        let version = Version::from_str(trimmed).map_err(|e| VersionError::InvalidFormat {
            version: s.to_string(),
            reason: format!("Invalid exact version: {}", e),
        })?;
        Ok(Self::Exact(version))
    }
}

impl fmt::Display for VersionRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exact(v) => write!(f, "{}", v),
            Self::Caret(v) => write!(f, "^{}", v),
            Self::Tilde(v) => write!(f, "~{}", v),
            Self::Greater(v) => write!(f, ">{}", v),
            Self::GreaterOrEqual(v) => write!(f, ">={}", v),
            Self::Less(v) => write!(f, "<{}", v),
            Self::LessOrEqual(v) => write!(f, "<={}", v),
            Self::Wildcard { major, minor: Some(minor) } => write!(f, "{}.{}.*", major, minor),
            Self::Wildcard { major, minor: None } => write!(f, "{}.*", major),
            Self::Range { min, max } => write!(f, "{} - {}", min, max),
            Self::Any => write!(f, "*"),
        }
    }
}

impl VersionRange {
    /// Checks if a version satisfies this range.
    ///
    /// # Arguments
    ///
    /// * `version` - The version to check against this range
    ///
    /// # Returns
    ///
    /// `true` if the version satisfies the range, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{VersionRange, Version};
    /// use std::str::FromStr;
    ///
    /// let caret_range = VersionRange::from_str("^1.2.3")?;
    /// let version_match = Version::from_str("1.5.0")?;
    /// let version_no_match = Version::from_str("2.0.0")?;
    ///
    /// assert!(caret_range.matches(&version_match));
    /// assert!(!caret_range.matches(&version_no_match));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn matches(&self, version: &Version) -> bool {
        match self {
            Self::Exact(v) => version == v,
            Self::Caret(v) => self.matches_caret(version, v),
            Self::Tilde(v) => self.matches_tilde(version, v),
            Self::Greater(v) => version > v,
            Self::GreaterOrEqual(v) => version >= v,
            Self::Less(v) => version < v,
            Self::LessOrEqual(v) => version <= v,
            Self::Wildcard { major, minor } => self.matches_wildcard(version, *major, *minor),
            Self::Range { min, max } => version >= min && version <= max,
            Self::Any => true,
        }
    }

    /// Creates a caret range from a version.
    ///
    /// # Arguments
    ///
    /// * `version` - The base version for the caret range
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{VersionRange, Version};
    /// use std::str::FromStr;
    ///
    /// let version = Version::from_str("1.2.3")?;
    /// let range = VersionRange::caret(version);
    ///
    /// assert_eq!(range.to_string(), "^1.2.3");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn caret(version: Version) -> Self {
        Self::Caret(version)
    }

    /// Creates a tilde range from a version.
    ///
    /// # Arguments
    ///
    /// * `version` - The base version for the tilde range
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{VersionRange, Version};
    /// use std::str::FromStr;
    ///
    /// let version = Version::from_str("1.2.3")?;
    /// let range = VersionRange::tilde(version);
    ///
    /// assert_eq!(range.to_string(), "~1.2.3");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn tilde(version: Version) -> Self {
        Self::Tilde(version)
    }

    /// Creates an exact version range.
    ///
    /// # Arguments
    ///
    /// * `version` - The exact version to match
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{VersionRange, Version};
    /// use std::str::FromStr;
    ///
    /// let version = Version::from_str("1.2.3")?;
    /// let range = VersionRange::exact(version);
    ///
    /// assert_eq!(range.to_string(), "1.2.3");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn exact(version: Version) -> Self {
        Self::Exact(version)
    }

    /// Creates a range between two versions (inclusive).
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum version (inclusive)
    /// * `max` - Maximum version (inclusive)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{VersionRange, Version};
    /// use std::str::FromStr;
    ///
    /// let min = Version::from_str("1.0.0")?;
    /// let max = Version::from_str("2.0.0")?;
    /// let range = VersionRange::range(min, max);
    ///
    /// assert_eq!(range.to_string(), "1.0.0 - 2.0.0");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn range(min: Version, max: Version) -> Self {
        Self::Range { min, max }
    }

    /// Creates a wildcard range.
    ///
    /// # Arguments
    ///
    /// * `major` - Major version component
    /// * `minor` - Optional minor version component
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::VersionRange;
    ///
    /// let patch_wildcard = VersionRange::wildcard(1, Some(2)); // 1.2.*
    /// let minor_wildcard = VersionRange::wildcard(1, None);    // 1.*
    ///
    /// assert_eq!(patch_wildcard.to_string(), "1.2.*");
    /// assert_eq!(minor_wildcard.to_string(), "1.*");
    /// ```
    #[must_use]
    pub fn wildcard(major: u64, minor: Option<u64>) -> Self {
        Self::Wildcard { major, minor }
    }

    /// Creates an "any" range that matches all versions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{VersionRange, Version};
    /// use std::str::FromStr;
    ///
    /// let any_range = VersionRange::any();
    /// let version = Version::from_str("1.2.3")?;
    ///
    /// assert!(any_range.matches(&version));
    /// assert_eq!(any_range.to_string(), "*");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn any() -> Self {
        Self::Any
    }

    /// Checks if this range is exact (matches only one specific version).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{VersionRange, Version};
    /// use std::str::FromStr;
    ///
    /// let exact = VersionRange::from_str("1.2.3")?;
    /// let caret = VersionRange::from_str("^1.2.3")?;
    ///
    /// assert!(exact.is_exact());
    /// assert!(!caret.is_exact());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn is_exact(&self) -> bool {
        matches!(self, Self::Exact(_))
    }

    /// Checks if this range is a wildcard.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::VersionRange;
    /// use std::str::FromStr;
    ///
    /// let wildcard = VersionRange::from_str("1.2.*")?;
    /// let exact = VersionRange::from_str("1.2.3")?;
    ///
    /// assert!(wildcard.is_wildcard());
    /// assert!(!exact.is_wildcard());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn is_wildcard(&self) -> bool {
        matches!(self, Self::Wildcard { .. } | Self::Any)
    }

    /// Gets the minimum version that could satisfy this range.
    ///
    /// # Returns
    ///
    /// The minimum version, or `None` if the range has no lower bound
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{VersionRange, Version};
    /// use std::str::FromStr;
    ///
    /// let caret = VersionRange::from_str("^1.2.3")?;
    /// let min_version = caret.min_version().unwrap();
    ///
    /// assert_eq!(min_version.to_string(), "1.2.3");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn min_version(&self) -> Option<&Version> {
        match self {
            Self::Exact(v) | Self::Caret(v) | Self::Tilde(v) | Self::GreaterOrEqual(v) => Some(v),
            Self::Range { min, .. } => Some(min),
            Self::Greater(_)
            | Self::Less(_)
            | Self::LessOrEqual(_)
            | Self::Wildcard { .. }
            | Self::Any => None,
        }
    }

    /// Gets the maximum version that could satisfy this range.
    ///
    /// # Returns
    ///
    /// The maximum version, or `None` if the range has no upper bound
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{VersionRange, Version};
    /// use std::str::FromStr;
    ///
    /// let range = VersionRange::from_str("1.0.0 - 2.0.0")?;
    /// let max_version = range.max_version().unwrap();
    ///
    /// assert_eq!(max_version.to_string(), "2.0.0");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn max_version(&self) -> Option<&Version> {
        match self {
            Self::Exact(v) | Self::Less(v) | Self::LessOrEqual(v) => Some(v),
            Self::Range { max, .. } => Some(max),
            Self::Caret(_)
            | Self::Tilde(_)
            | Self::Greater(_)
            | Self::GreaterOrEqual(_)
            | Self::Wildcard { .. }
            | Self::Any => None,
        }
    }

    /// Parses a wildcard pattern like "1.2.*" or "1.*".
    fn parse_wildcard(input: &str) -> Result<Self, VersionError> {
        let parts: Vec<&str> = input.split('.').collect();

        if parts.len() < 2 || parts.len() > 3 {
            return Err(VersionError::InvalidFormat {
                version: input.to_string(),
                reason: "Wildcard must be in format 'major.*' or 'major.minor.*'".to_string(),
            });
        }

        let major = parts[0].parse::<u64>().map_err(|_| VersionError::InvalidFormat {
            version: input.to_string(),
            reason: "Invalid major version in wildcard".to_string(),
        })?;

        if parts.len() == 2 {
            // Format: "1.*"
            if parts[1] != "*" {
                return Err(VersionError::InvalidFormat {
                    version: input.to_string(),
                    reason: "Expected '*' after major version".to_string(),
                });
            }
            return Ok(Self::Wildcard { major, minor: None });
        }

        // Format: "1.2.*"
        let minor = parts[1].parse::<u64>().map_err(|_| VersionError::InvalidFormat {
            version: input.to_string(),
            reason: "Invalid minor version in wildcard".to_string(),
        })?;

        if parts[2] != "*" {
            return Err(VersionError::InvalidFormat {
                version: input.to_string(),
                reason: "Expected '*' for patch version".to_string(),
            });
        }

        Ok(Self::Wildcard { major, minor: Some(minor) })
    }

    /// Checks if a version matches a caret range.
    fn matches_caret(&self, version: &Version, base: &Version) -> bool {
        // Caret allows patch-level and minor-level changes if major > 0
        // For 0.x.y, only patch-level changes are allowed
        // For 0.0.x, no changes are allowed (equivalent to exact)

        if base.major() > 0 {
            // ^1.2.3 := >=1.2.3 <2.0.0
            version >= base && version.major() == base.major()
        } else if base.minor() > 0 {
            // ^0.2.3 := >=0.2.3 <0.3.0
            version >= base && version.major() == 0 && version.minor() == base.minor()
        } else {
            // ^0.0.3 := >=0.0.3 <0.0.4
            version >= base
                && version.major() == 0
                && version.minor() == 0
                && version.patch() == base.patch()
        }
    }

    /// Checks if a version matches a tilde range.
    fn matches_tilde(&self, version: &Version, base: &Version) -> bool {
        // Tilde allows patch-level changes within the same minor version
        // ~1.2.3 := >=1.2.3 <1.3.0
        version >= base && version.major() == base.major() && version.minor() == base.minor()
    }

    /// Checks if a version matches a wildcard pattern.
    fn matches_wildcard(&self, version: &Version, major: u64, minor: Option<u64>) -> bool {
        match minor {
            Some(minor_ver) => {
                // 1.2.* - matches major.minor.any_patch
                version.major() == major && version.minor() == minor_ver
            }
            None => {
                // 1.* - matches major.any_minor.any_patch
                version.major() == major
            }
        }
    }
}

/// Version range constraint for dependency analysis.
///
/// Represents a constraint that can be evaluated against versions to determine
/// compatibility for dependency resolution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionConstraint {
    /// The version range specification
    pub range: VersionRange,
    /// Optional constraint name for debugging
    pub name: Option<String>,
}

impl VersionConstraint {
    /// Creates a new version constraint.
    ///
    /// # Arguments
    ///
    /// * `range` - The version range for this constraint
    /// * `name` - Optional name for debugging purposes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{VersionConstraint, VersionRange};
    /// use std::str::FromStr;
    ///
    /// let range = VersionRange::from_str("^1.2.3")?;
    /// let constraint = VersionConstraint::new(range, Some("my-dependency".to_string()));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn new(range: VersionRange, name: Option<String>) -> Self {
        Self { range, name }
    }

    /// Checks if a version satisfies this constraint.
    ///
    /// # Arguments
    ///
    /// * `version` - The version to check
    ///
    /// # Returns
    ///
    /// `true` if the version satisfies the constraint
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{VersionConstraint, VersionRange, Version};
    /// use std::str::FromStr;
    ///
    /// let range = VersionRange::from_str("^1.2.3")?;
    /// let constraint = VersionConstraint::new(range, None);
    /// let version = Version::from_str("1.5.0")?;
    ///
    /// assert!(constraint.satisfies(&version));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn satisfies(&self, version: &Version) -> bool {
        self.range.matches(version)
    }

    /// Gets the constraint name if available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{VersionConstraint, VersionRange};
    /// use std::str::FromStr;
    ///
    /// let range = VersionRange::from_str("^1.2.3")?;
    /// let constraint = VersionConstraint::new(range, Some("my-dep".to_string()));
    ///
    /// assert_eq!(constraint.name(), Some("my-dep"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Gets the version range.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::{VersionConstraint, VersionRange};
    /// use std::str::FromStr;
    ///
    /// let range = VersionRange::from_str("^1.2.3")?;
    /// let constraint = VersionConstraint::new(range.clone(), None);
    ///
    /// assert_eq!(constraint.range(), &range);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn range(&self) -> &VersionRange {
        &self.range
    }
}

impl fmt::Display for VersionConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{}: {}", name, self.range),
            None => write!(f, "{}", self.range),
        }
    }
}

impl FromStr for VersionConstraint {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let range = VersionRange::from_str(s)?;
        Ok(Self::new(range, None))
    }
}

impl From<VersionRange> for VersionConstraint {
    fn from(range: VersionRange) -> Self {
        Self::new(range, None)
    }
}
