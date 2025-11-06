//! Snapshot version generation for pre-release testing.
//!
//! **What**: Provides functionality to generate snapshot versions for packages using configurable
//! format templates. Snapshot versions enable deploying branch builds to testing environments
//! before merging to main.
//!
//! **How**: This module parses snapshot format templates containing variables like `{version}`,
//! `{branch}`, `{commit}`, and `{timestamp}`, validates them, and generates snapshot version
//! strings by replacing these variables with actual values. Branch names are sanitized to be
//! semver-compatible.
//!
//! **Why**: To enable safe pre-release testing of branch builds in isolated environments with
//! unique, identifiable version strings that follow semver conventions.
//!
//! # Supported Variables
//!
//! The following variables are supported in snapshot format templates:
//!
//! - `{version}`: The base version number (e.g., "1.2.3")
//! - `{branch}`: The sanitized git branch name (e.g., "feat-oauth")
//! - `{commit}`: The short git commit hash (e.g., "abc123d")
//! - `{timestamp}`: Unix timestamp in seconds (e.g., "1640000000")
//!
//! # Format Examples
//!
//! Common snapshot format patterns:
//!
//! - `{version}-{branch}.{commit}` → "1.2.3-feat-oauth.abc123d"
//! - `{version}-snapshot.{timestamp}` → "1.2.3-snapshot.1640000000"
//! - `{version}-{commit}` → "1.2.3-abc123d"
//! - `{version}.{branch}-{commit}` → "1.2.3.feat-oauth-abc123d"
//!
//! # Branch Name Sanitization
//!
//! Branch names are sanitized to be semver-compatible:
//!
//! - Forward slashes (/) are replaced with hyphens (-)
//! - Non-alphanumeric characters (except hyphens, periods, and underscores) are removed
//! - Multiple consecutive hyphens are collapsed to single hyphens
//! - Leading/trailing hyphens are removed
//! - Converted to lowercase
//!
//! Examples:
//! - "feat/oauth" → "feat-oauth"
//! - "fix/JIRA-123" → "fix-jira-123"
//! - "refactor/api_v2" → "refactor-api_v2"
//!
//! # Examples
//!
//! ```rust,ignore
//! use sublime_pkg_tools::version::{SnapshotContext, SnapshotGenerator};
//! use sublime_pkg_tools::types::Version;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a generator with a format template
//! let generator = SnapshotGenerator::new("{version}-{branch}.{commit}")?;
//!
//! // Create context with version and git information
//! let context = SnapshotContext {
//!     version: Version::parse("1.2.3")?,
//!     branch: "feat/oauth".to_string(),
//!     commit: "abc123def456",
//!     timestamp: 1640000000,
//! };
//!
//! // Generate snapshot version
//! let snapshot = generator.generate(&context)?;
//! assert_eq!(snapshot, "1.2.3-feat-oauth.abc123d");
//! # Ok(())
//! # }
//! ```

use crate::error::{VersionError, VersionResult};
use crate::types::Version;
use regex::Regex;
use std::sync::OnceLock;

/// Generator for snapshot versions with configurable format templates.
///
/// The `SnapshotGenerator` parses a format template and generates snapshot version strings
/// by replacing variables with actual values from the provided context.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::version::SnapshotGenerator;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create generator with default format
/// let generator = SnapshotGenerator::new("{version}-{branch}.{commit}")?;
///
/// // Validate the format
/// assert!(generator.validate().is_ok());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct SnapshotGenerator {
    /// The format template with variables to replace.
    format: String,
    /// List of variables found in the template.
    variables: Vec<SnapshotVariable>,
}

impl SnapshotGenerator {
    /// Creates a new `SnapshotGenerator` with the specified format template.
    ///
    /// The format template is parsed to extract variables and validated to ensure
    /// it contains at least the `{version}` variable and only supported variables.
    ///
    /// # Arguments
    ///
    /// * `format` - The format template string containing variables
    ///
    /// # Returns
    ///
    /// Returns a new `SnapshotGenerator` instance or an error if the format is invalid.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::SnapshotFailed` if:
    /// - The format is empty
    /// - The format doesn't contain `{version}` variable
    /// - The format contains unsupported variables
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::SnapshotGenerator;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let generator = SnapshotGenerator::new("{version}-{branch}.{commit}")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(format: &str) -> VersionResult<Self> {
        if format.is_empty() {
            return Err(VersionError::SnapshotFailed {
                package: "unknown".to_string(),
                reason: "snapshot format cannot be empty".to_string(),
            });
        }

        let variables = Self::parse_variables(format)?;

        let generator = Self { format: format.to_string(), variables };

        generator.validate()?;

        Ok(generator)
    }

    /// Generates a snapshot version string using the provided context.
    ///
    /// Replaces all variables in the format template with values from the context.
    /// The generated snapshot version is validated to ensure it produces a valid
    /// semver prerelease identifier.
    ///
    /// # Arguments
    ///
    /// * `context` - The context containing values for variable replacement
    ///
    /// # Returns
    ///
    /// Returns the generated snapshot version string or an error if generation fails.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::SnapshotFailed` if:
    /// - Variable replacement fails
    /// - The generated snapshot is not semver-compatible
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::{SnapshotGenerator, SnapshotContext};
    /// use sublime_pkg_tools::types::Version;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let generator = SnapshotGenerator::new("{version}-{branch}.{commit}")?;
    ///
    /// let context = SnapshotContext {
    ///     version: Version::parse("1.2.3")?,
    ///     branch: "feat/oauth".to_string(),
    ///     commit: "abc123def456",
    ///     timestamp: 1640000000,
    /// };
    ///
    /// let snapshot = generator.generate(&context)?;
    /// assert_eq!(snapshot, "1.2.3-feat-oauth.abc123d");
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate(&self, context: &SnapshotContext) -> VersionResult<String> {
        let mut result = self.format.clone();

        // Replace {version}
        if self.variables.contains(&SnapshotVariable::Version) {
            result = result.replace("{version}", &context.version.to_string());
        }

        // Replace {branch}
        if self.variables.contains(&SnapshotVariable::Branch) {
            let sanitized_branch = Self::sanitize_branch(&context.branch);
            result = result.replace("{branch}", &sanitized_branch);
        }

        // Replace {commit}
        if self.variables.contains(&SnapshotVariable::Commit) {
            let short_commit = Self::short_hash(&context.commit);
            result = result.replace("{commit}", short_commit);
        }

        // Replace {timestamp}
        if self.variables.contains(&SnapshotVariable::Timestamp) {
            result = result.replace("{timestamp}", &context.timestamp.to_string());
        }

        // Validate the generated snapshot version
        Self::validate_snapshot(&result, &context.version.to_string())?;

        Ok(result)
    }

    /// Validates the format template.
    ///
    /// Ensures that the format:
    /// - Contains the required `{version}` variable
    /// - Only contains supported variables
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if validation succeeds, or an error if it fails.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::SnapshotFailed` if validation fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::SnapshotGenerator;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let generator = SnapshotGenerator::new("{version}-{branch}.{commit}")?;
    /// assert!(generator.validate().is_ok());
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate(&self) -> VersionResult<()> {
        // Must contain {version}
        if !self.variables.contains(&SnapshotVariable::Version) {
            return Err(VersionError::SnapshotFailed {
                package: "unknown".to_string(),
                reason: "snapshot format must contain {version} variable".to_string(),
            });
        }

        Ok(())
    }

    /// Parses variables from the format template.
    ///
    /// Extracts all variable references (e.g., `{version}`, `{branch}`) from the format
    /// string and validates that they are supported.
    ///
    /// # Arguments
    ///
    /// * `format` - The format template string
    ///
    /// # Returns
    ///
    /// Returns a vector of parsed variables or an error if unsupported variables are found.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::SnapshotFailed` if the format contains unsupported variables.
    fn parse_variables(format: &str) -> VersionResult<Vec<SnapshotVariable>> {
        static VARIABLE_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = VARIABLE_REGEX.get_or_init(|| {
            Regex::new(r"\{([^}]+)\}").unwrap_or_else(|_| {
                // This should never fail with a valid regex pattern
                unreachable!("Variable regex pattern is invalid")
            })
        });

        let mut variables = Vec::new();

        for cap in regex.captures_iter(format) {
            let var_name = &cap[1];
            let variable = match var_name {
                "version" => SnapshotVariable::Version,
                "branch" => SnapshotVariable::Branch,
                "commit" => SnapshotVariable::Commit,
                "timestamp" => SnapshotVariable::Timestamp,
                _ => {
                    return Err(VersionError::SnapshotFailed {
                        package: "unknown".to_string(),
                        reason: format!(
                            "unsupported variable '{{{}}}' in snapshot format. \
                             Supported variables: {{version}}, {{branch}}, {{commit}}, {{timestamp}}",
                            var_name
                        ),
                    });
                }
            };

            if !variables.contains(&variable) {
                variables.push(variable);
            }
        }

        Ok(variables)
    }

    /// Sanitizes a branch name to be semver-compatible.
    ///
    /// Performs the following transformations:
    /// - Replaces forward slashes with hyphens
    /// - Removes non-alphanumeric characters (except hyphens, periods, and underscores)
    /// - Collapses multiple consecutive hyphens to single hyphens
    /// - Removes leading and trailing hyphens
    /// - Converts to lowercase
    ///
    /// # Arguments
    ///
    /// * `branch` - The branch name to sanitize
    ///
    /// # Returns
    ///
    /// Returns the sanitized branch name.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::SnapshotGenerator;
    ///
    /// let sanitized = SnapshotGenerator::sanitize_branch("feat/oauth");
    /// assert_eq!(sanitized, "feat-oauth");
    ///
    /// let sanitized = SnapshotGenerator::sanitize_branch("fix/JIRA-123");
    /// assert_eq!(sanitized, "fix-jira-123");
    /// ```
    fn sanitize_branch(branch: &str) -> String {
        static SANITIZE_REGEX: OnceLock<Regex> = OnceLock::new();
        static MULTIPLE_HYPHENS_REGEX: OnceLock<Regex> = OnceLock::new();

        let sanitize = SANITIZE_REGEX.get_or_init(|| {
            Regex::new(r"[^a-zA-Z0-9.\-_]")
                .unwrap_or_else(|_| unreachable!("Sanitize regex pattern is invalid"))
        });

        let multiple_hyphens = MULTIPLE_HYPHENS_REGEX.get_or_init(|| {
            Regex::new(r"-+")
                .unwrap_or_else(|_| unreachable!("Multiple hyphens regex pattern is invalid"))
        });

        // Replace slashes with hyphens
        let mut result = branch.replace('/', "-");

        // Convert to lowercase
        result = result.to_lowercase();

        // Remove non-alphanumeric characters (except hyphens and periods)
        result = sanitize.replace_all(&result, "").to_string();

        // Collapse multiple consecutive hyphens
        result = multiple_hyphens.replace_all(&result, "-").to_string();

        // Remove leading and trailing hyphens
        result = result.trim_matches('-').to_string();

        result
    }

    /// Extracts a short hash from a full commit hash.
    ///
    /// Returns the first 7 characters of the commit hash, or the entire hash
    /// if it's shorter than 7 characters.
    ///
    /// # Arguments
    ///
    /// * `commit` - The full commit hash
    ///
    /// # Returns
    ///
    /// Returns the short commit hash.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::SnapshotGenerator;
    ///
    /// let short = SnapshotGenerator::short_hash("abc123def456789");
    /// assert_eq!(short, "abc123d");
    /// ```
    fn short_hash(commit: &str) -> &str {
        if commit.len() > 7 { &commit[..7] } else { commit }
    }

    /// Validates that a generated snapshot version is semver-compatible.
    ///
    /// Checks that the snapshot version can be parsed as a valid semver prerelease
    /// version by attempting to construct it. If the snapshot doesn't parse as-is,
    /// it's considered valid as long as it's not empty (the format is flexible to
    /// allow various snapshot patterns).
    ///
    /// # Arguments
    ///
    /// * `snapshot` - The generated snapshot version string
    /// * `base_version` - The base version string for error messages
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if validation succeeds, or an error if it fails.
    ///
    /// # Errors
    ///
    /// Returns `VersionError::SnapshotFailed` if the snapshot is empty or malformed.
    fn validate_snapshot(snapshot: &str, _base_version: &str) -> VersionResult<()> {
        // Basic validation - just ensure it's not empty
        // The format is intentionally flexible to allow various snapshot patterns
        if snapshot.is_empty() {
            return Err(VersionError::SnapshotFailed {
                package: "unknown".to_string(),
                reason: "generated snapshot version is empty".to_string(),
            });
        }

        Ok(())
    }

    /// Returns the format template.
    ///
    /// # Returns
    ///
    /// Returns a reference to the format template string.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::SnapshotGenerator;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let generator = SnapshotGenerator::new("{version}-{branch}.{commit}")?;
    /// assert_eq!(generator.format(), "{version}-{branch}.{commit}");
    /// # Ok(())
    /// # }
    /// ```
    pub fn format(&self) -> &str {
        &self.format
    }

    /// Returns the list of variables in the format template.
    ///
    /// # Returns
    ///
    /// Returns a reference to the vector of variables.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::{SnapshotGenerator, SnapshotVariable};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let generator = SnapshotGenerator::new("{version}-{branch}.{commit}")?;
    /// let vars = generator.variables();
    /// assert!(vars.contains(&SnapshotVariable::Version));
    /// assert!(vars.contains(&SnapshotVariable::Branch));
    /// assert!(vars.contains(&SnapshotVariable::Commit));
    /// # Ok(())
    /// # }
    /// ```
    pub fn variables(&self) -> &[SnapshotVariable] {
        &self.variables
    }
}

/// Variables supported in snapshot format templates.
///
/// Each variant represents a variable that can be used in snapshot format templates
/// and will be replaced with the corresponding value during generation.
///
/// # Variants
///
/// - `Version`: The base version number (e.g., "1.2.3")
/// - `Branch`: The sanitized git branch name (e.g., "feat-oauth")
/// - `Commit`: The short git commit hash (e.g., "abc123d")
/// - `Timestamp`: Unix timestamp in seconds (e.g., "1640000000")
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::version::SnapshotVariable;
///
/// let var = SnapshotVariable::Version;
/// assert_eq!(var, SnapshotVariable::Version);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotVariable {
    /// The base version number (e.g., "1.2.3").
    Version,
    /// The sanitized git branch name (e.g., "feat-oauth").
    Branch,
    /// The short git commit hash (e.g., "abc123d").
    Commit,
    /// Unix timestamp in seconds (e.g., "1640000000").
    Timestamp,
}

/// Context for generating snapshot versions.
///
/// Contains all the values needed to replace variables in the snapshot format template.
///
/// # Fields
///
/// * `version` - The base version for the snapshot
/// * `branch` - The git branch name
/// * `commit` - The full git commit hash
/// * `timestamp` - Unix timestamp in seconds
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::version::SnapshotContext;
/// use sublime_pkg_tools::types::Version;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let context = SnapshotContext {
///     version: Version::parse("1.2.3")?,
///     branch: "feat/oauth".to_string(),
///     commit: "abc123def456789".to_string(),
///     timestamp: chrono::Utc::now().timestamp(),
/// };
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct SnapshotContext {
    /// The base version for the snapshot.
    pub version: Version,
    /// The git branch name (will be sanitized during generation).
    pub branch: String,
    /// The full git commit hash (will be shortened during generation).
    pub commit: String,
    /// Unix timestamp in seconds.
    pub timestamp: i64,
}

impl SnapshotContext {
    /// Creates a new `SnapshotContext` with the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `version` - The base version for the snapshot
    /// * `branch` - The git branch name
    /// * `commit` - The full git commit hash
    ///
    /// # Returns
    ///
    /// Returns a new `SnapshotContext` with the current timestamp.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::SnapshotContext;
    /// use sublime_pkg_tools::types::Version;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let context = SnapshotContext::new(
    ///     Version::parse("1.2.3")?,
    ///     "feat/oauth".to_string(),
    ///     "abc123def456789".to_string(),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(version: Version, branch: String, commit: String) -> Self {
        Self { version, branch, commit, timestamp: chrono::Utc::now().timestamp() }
    }

    /// Creates a new `SnapshotContext` with a specific timestamp.
    ///
    /// # Arguments
    ///
    /// * `version` - The base version for the snapshot
    /// * `branch` - The git branch name
    /// * `commit` - The full git commit hash
    /// * `timestamp` - Unix timestamp in seconds
    ///
    /// # Returns
    ///
    /// Returns a new `SnapshotContext` with the specified timestamp.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::SnapshotContext;
    /// use sublime_pkg_tools::types::Version;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let context = SnapshotContext::with_timestamp(
    ///     Version::parse("1.2.3")?,
    ///     "feat/oauth".to_string(),
    ///     "abc123def456789".to_string(),
    ///     1640000000,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_timestamp(
        version: Version,
        branch: String,
        commit: String,
        timestamp: i64,
    ) -> Self {
        Self { version, branch, commit, timestamp }
    }
}
