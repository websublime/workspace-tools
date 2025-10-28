//! Version detection from Git tags for changelog generation.
//!
//! **What**: Provides functionality to detect and parse version tags from Git repositories,
//! supporting both monorepo (per-package tags) and single-package (root tags) scenarios.
//!
//! **How**: This module parses Git tags according to configurable formats, extracts version
//! information, and identifies the previous version for changelog generation. It supports
//! custom tag formats with placeholders for package names and versions.
//!
//! **Why**: To automatically detect version boundaries for changelog generation, enabling
//! the system to determine which commits belong to which version without manual intervention.
//!
//! # Tag Format Support
//!
//! This module supports two types of tag formats:
//!
//! ## Monorepo Tags
//!
//! Format: `{name}@{version}` (configurable via `version_tag_format`)
//! - Example: `@myorg/utils@1.2.3`
//! - Example: `pkg-core@2.0.0`
//!
//! ## Root Tags
//!
//! Format: `v{version}` (configurable via `root_tag_format`)
//! - Example: `v1.2.3`
//! - Example: `1.0.0` (when format is `{version}`)
//!
//! # Examples
//!
//! ```rust,ignore
//! use sublime_pkg_tools::changelog::version_detection::{VersionTag, parse_version_tag};
//!
//! // Parse monorepo tag
//! let tag = parse_version_tag("@myorg/utils@1.2.3", Some("@myorg/utils"), "{name}@{version}");
//! assert!(tag.is_some());
//! let tag = tag.unwrap();
//! assert_eq!(tag.version().to_string(), "1.2.3");
//! assert_eq!(tag.package_name(), Some("@myorg/utils"));
//!
//! // Parse root tag
//! let tag = parse_version_tag("v1.2.3", None, "v{version}");
//! assert!(tag.is_some());
//! let tag = tag.unwrap();
//! assert_eq!(tag.version().to_string(), "1.2.3");
//! assert_eq!(tag.package_name(), None);
//! ```

use crate::error::{ChangelogError, ChangelogResult};
use crate::types::Version;
use regex::Regex;
use std::cmp::Ordering;

/// Represents a parsed version tag from Git.
///
/// A version tag contains a version number and optionally a package name
/// (for monorepo scenarios). This structure allows comparing tags and
/// determining version ordering.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::version_detection::VersionTag;
/// use sublime_pkg_tools::types::Version;
///
/// let tag = VersionTag::new(
///     "mypackage@1.0.0".to_string(),
///     Version::parse("1.0.0").unwrap(),
///     Some("mypackage".to_string()),
/// );
///
/// assert_eq!(tag.tag_name(), "mypackage@1.0.0");
/// assert_eq!(tag.version().to_string(), "1.0.0");
/// assert_eq!(tag.package_name(), Some("mypackage"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionTag {
    /// The full tag name as it appears in Git.
    tag_name: String,

    /// The parsed semantic version.
    version: Version,

    /// The package name (for monorepo tags), or None for root tags.
    package_name: Option<String>,
}

impl VersionTag {
    /// Creates a new `VersionTag`.
    ///
    /// # Arguments
    ///
    /// * `tag_name` - The full tag name from Git
    /// * `version` - The parsed semantic version
    /// * `package_name` - Optional package name for monorepo tags
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::version_detection::VersionTag;
    /// use sublime_pkg_tools::types::Version;
    ///
    /// let tag = VersionTag::new(
    ///     "v1.0.0".to_string(),
    ///     Version::parse("1.0.0").unwrap(),
    ///     None,
    /// );
    /// ```
    #[must_use]
    pub fn new(tag_name: String, version: Version, package_name: Option<String>) -> Self {
        Self { tag_name, version, package_name }
    }

    /// Returns the full tag name.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changelog::version_detection::VersionTag;
    /// # use sublime_pkg_tools::types::Version;
    /// #
    /// let tag = VersionTag::new("v1.0.0".to_string(), Version::parse("1.0.0").unwrap(), None);
    /// assert_eq!(tag.tag_name(), "v1.0.0");
    /// ```
    #[must_use]
    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }

    /// Returns a reference to the parsed version.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changelog::version_detection::VersionTag;
    /// # use sublime_pkg_tools::types::Version;
    /// #
    /// let tag = VersionTag::new("v1.0.0".to_string(), Version::parse("1.0.0").unwrap(), None);
    /// assert_eq!(tag.version().to_string(), "1.0.0");
    /// ```
    #[must_use]
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Returns the package name if this is a monorepo tag.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changelog::version_detection::VersionTag;
    /// # use sublime_pkg_tools::types::Version;
    /// #
    /// let tag = VersionTag::new(
    ///     "pkg@1.0.0".to_string(),
    ///     Version::parse("1.0.0").unwrap(),
    ///     Some("pkg".to_string()),
    /// );
    /// assert_eq!(tag.package_name(), Some("pkg"));
    /// ```
    #[must_use]
    pub fn package_name(&self) -> Option<&str> {
        self.package_name.as_deref()
    }

    /// Checks if this tag matches the given package name.
    ///
    /// Returns `true` if:
    /// - This is a root tag (no package name) and `package_name` is `None`
    /// - This is a monorepo tag and the package names match
    ///
    /// # Arguments
    ///
    /// * `package_name` - The package name to match against, or None for root tags
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changelog::version_detection::VersionTag;
    /// # use sublime_pkg_tools::types::Version;
    /// #
    /// let tag = VersionTag::new(
    ///     "pkg@1.0.0".to_string(),
    ///     Version::parse("1.0.0").unwrap(),
    ///     Some("pkg".to_string()),
    /// );
    /// assert!(tag.matches_package(Some("pkg")));
    /// assert!(!tag.matches_package(Some("other")));
    /// assert!(!tag.matches_package(None));
    /// ```
    #[must_use]
    pub fn matches_package(&self, package_name: Option<&str>) -> bool {
        match (&self.package_name, package_name) {
            (None, None) => true,
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for VersionTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VersionTag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.version.as_semver().cmp(other.version.as_semver())
    }
}

/// Parses a Git tag string into a `VersionTag`.
///
/// This function attempts to parse a tag according to the provided format template.
/// It supports two types of formats:
/// - Monorepo format with `{name}` and `{version}` placeholders
/// - Root format with `{version}` placeholder only
///
/// # Arguments
///
/// * `tag` - The Git tag string to parse
/// * `expected_package` - The expected package name for monorepo tags, or None for root tags
/// * `format` - The tag format template with placeholders
///
/// # Returns
///
/// Returns `Some(VersionTag)` if the tag matches the format and contains a valid version,
/// otherwise returns `None`.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::version_detection::parse_version_tag;
///
/// // Parse monorepo tag
/// let tag = parse_version_tag("@myorg/utils@1.2.3", Some("@myorg/utils"), "{name}@{version}");
/// assert!(tag.is_some());
///
/// // Parse root tag
/// let tag = parse_version_tag("v1.2.3", None, "v{version}");
/// assert!(tag.is_some());
///
/// // Invalid tag
/// let tag = parse_version_tag("invalid", None, "v{version}");
/// assert!(tag.is_none());
/// ```
#[must_use]
pub(crate) fn parse_version_tag(
    tag: &str,
    expected_package: Option<&str>,
    format: &str,
) -> Option<VersionTag> {
    // Build regex from format template
    let regex_pattern = build_tag_regex(format)?;
    let re = Regex::new(&regex_pattern).ok()?;

    // Try to match the tag
    let captures = re.captures(tag)?;

    // Extract version
    let version_str = captures.name("version")?.as_str();
    let version = Version::parse(version_str).ok()?;

    // Extract package name if present in format
    let package_name = if format.contains("{name}") {
        captures.name("name").map(|m| m.as_str().to_string())
    } else {
        None
    };

    // Verify package matches if expected
    if let Some(expected) = expected_package
        && package_name.as_deref() != Some(expected) {
            return None;
        }

    Some(VersionTag::new(tag.to_string(), version, package_name))
}

/// Builds a regex pattern from a tag format template.
///
/// Converts placeholders to named capture groups:
/// - `{name}` -> `(?P<name>.+?)`
/// - `{version}` -> `(?P<version>\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?(?:\+[a-zA-Z0-9.-]+)?)`
///
/// # Arguments
///
/// * `format` - The format template string
///
/// # Returns
///
/// Returns `Some(String)` containing the regex pattern, or `None` if the format is invalid.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::version_detection::build_tag_regex;
///
/// let pattern = build_tag_regex("v{version}");
/// assert!(pattern.is_some());
///
/// let pattern = build_tag_regex("{name}@{version}");
/// assert!(pattern.is_some());
/// ```
#[must_use]
pub(crate) fn build_tag_regex(format: &str) -> Option<String> {
    if format.is_empty() {
        return None;
    }

    // Escape special regex characters except placeholders
    let mut pattern = regex::escape(format);

    // Replace escaped placeholders with capture groups
    // Version pattern supports semantic versioning with prerelease and build metadata
    pattern = pattern.replace(
        "\\{version\\}",
        r"(?P<version>\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?(?:\+[a-zA-Z0-9.-]+)?)",
    );

    // Name pattern captures any characters (non-greedy)
    pattern = pattern.replace("\\{name\\}", r"(?P<name>.+?)");

    // Ensure full match
    Some(format!("^{}$", pattern))
}

/// Finds all version tags in a list of Git tags.
///
/// Filters and parses tags according to the provided format, optionally filtering
/// by package name for monorepo scenarios.
///
/// # Arguments
///
/// * `tags` - List of Git tag strings
/// * `package_name` - Optional package name to filter monorepo tags
/// * `format` - The tag format template
///
/// # Returns
///
/// Returns a vector of `VersionTag` instances, sorted by version (newest first).
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::version_detection::find_version_tags;
///
/// let tags = vec!["v1.0.0", "v1.1.0", "v2.0.0", "other-tag"];
/// let version_tags = find_version_tags(&tags, None, "v{version}");
/// assert_eq!(version_tags.len(), 3);
/// assert_eq!(version_tags[0].version().to_string(), "2.0.0"); // Sorted newest first
/// ```
#[must_use]
pub(crate) fn find_version_tags(
    tags: &[String],
    package_name: Option<&str>,
    format: &str,
) -> Vec<VersionTag> {
    let mut version_tags: Vec<VersionTag> =
        tags.iter().filter_map(|tag| parse_version_tag(tag, package_name, format)).collect();

    // Sort by version (newest first)
    version_tags.sort_by(|a, b| b.cmp(a));

    version_tags
}

/// Finds the previous version tag before a given version.
///
/// Searches through a list of tags to find the most recent version that is
/// less than the current version. This is used to determine the commit range
/// for changelog generation.
///
/// # Arguments
///
/// * `tags` - List of Git tag strings
/// * `current_version` - The current version to compare against
/// * `package_name` - Optional package name for monorepo filtering
/// * `format` - The tag format template
///
/// # Returns
///
/// Returns `Ok(Some(VersionTag))` if a previous version is found,
/// `Ok(None)` if this is the first version,
/// or an error if the current version is invalid.
///
/// # Errors
///
/// Returns an error if:
/// - The current version string cannot be parsed
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::version_detection::find_previous_version;
///
/// let tags = vec!["v1.0.0".to_string(), "v1.1.0".to_string(), "v2.0.0".to_string()];
/// let previous = find_previous_version(&tags, "2.0.0", None, "v{version}").unwrap();
/// assert!(previous.is_some());
/// assert_eq!(previous.unwrap().version().to_string(), "1.1.0");
///
/// // First version
/// let previous = find_previous_version(&tags, "1.0.0", None, "v{version}").unwrap();
/// assert!(previous.is_none());
/// ```
pub(crate) fn find_previous_version(
    tags: &[String],
    current_version: &str,
    package_name: Option<&str>,
    format: &str,
) -> ChangelogResult<Option<VersionTag>> {
    // Parse current version
    let current = Version::parse(current_version).map_err(|_| ChangelogError::InvalidVersion {
        version: current_version.to_string(),
        reason: "Failed to parse version".to_string(),
    })?;

    // Find all version tags
    let version_tags = find_version_tags(tags, package_name, format);

    // Find the most recent version that is less than current
    let previous = version_tags
        .into_iter()
        .filter(|tag| tag.version().as_semver() < current.as_semver())
        .max_by(|a, b| a.version().as_semver().cmp(b.version().as_semver()));

    Ok(previous)
}

/// Detects if a tag format is for monorepo (contains `{name}` placeholder).
///
/// # Arguments
///
/// * `format` - The tag format template
///
/// # Returns
///
/// Returns `true` if the format contains a `{name}` placeholder, `false` otherwise.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::version_detection::is_monorepo_format;
///
/// assert!(is_monorepo_format("{name}@{version}"));
/// assert!(!is_monorepo_format("v{version}"));
/// ```
#[must_use]
#[allow(dead_code)]
pub(crate) fn is_monorepo_format(format: &str) -> bool {
    format.contains("{name}")
}

/// Formats a version tag string from a package name and version.
///
/// This is the inverse of `parse_version_tag`, creating a tag string from
/// components according to the format template.
///
/// # Arguments
///
/// * `package_name` - Optional package name for monorepo tags
/// * `version` - The version string
/// * `format` - The tag format template
///
/// # Returns
///
/// Returns the formatted tag string.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::version_detection::format_version_tag;
///
/// let tag = format_version_tag(Some("mypackage"), "1.0.0", "{name}@{version}");
/// assert_eq!(tag, "mypackage@1.0.0");
///
/// let tag = format_version_tag(None, "1.0.0", "v{version}");
/// assert_eq!(tag, "v1.0.0");
/// ```
#[must_use]
#[allow(dead_code)]
pub(crate) fn format_version_tag(
    package_name: Option<&str>,
    version: &str,
    format: &str,
) -> String {
    let mut result = format.to_string();

    if let Some(name) = package_name {
        result = result.replace("{name}", name);
    }

    result = result.replace("{version}", version);

    result
}
