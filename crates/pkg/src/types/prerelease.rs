//! Prerelease version support types.
//!
//! **What**: Provides types and configurations for managing prerelease versions
//! (e.g., "1.3.0-beta.0", "2.0.0-rc.1") according to SemVer 2.0.0 specification.
//!
//! **How**: Defines configuration types (`PrereleaseConfig`) and behavior modes
//! (`PrereleaseMode`) that control how prerelease versions are created, incremented,
//! and promoted to stable releases.
//!
//! **Why**: To enable controlled prerelease workflows (alpha, beta, RC) while maintaining
//! flexibility across different branching strategies (GitHub Flow, Gitflow, custom workflows).
//!
//! # SemVer 2.0.0 Prerelease Specification
//!
//! Prerelease versions follow the format:
//! - `MAJOR.MINOR.PATCH-PRERELEASE` (e.g., `1.0.0-beta.1`)
//! - Identifiers: Only ASCII alphanumerics and hyphens `[0-9A-Za-z-]`
//! - Separation: Dot-separated identifiers (`.`)
//! - Precedence: `alpha < alpha.1 < beta < beta.2 < rc.1 < 1.0.0`
//!
//! # Prerelease Modes
//!
//! ## Create Mode
//! Generate new prerelease from stable version:
//! ```text
//! 1.2.3 → 1.3.0-beta.0
//! ```
//!
//! ## Increment Mode
//! Increment existing prerelease number:
//! ```text
//! 1.3.0-beta.0 → 1.3.0-beta.1
//! ```
//!
//! ## Promote Mode
//! Remove prerelease tag (promote to stable):
//! ```text
//! 1.3.0-rc.1 → 1.3.0
//! ```
//!
//! # Examples
//!
//! ```rust
//! use sublime_pkg_tools::types::prerelease::{PrereleaseConfig, PrereleaseMode};
//!
//! // Create new beta prerelease
//! let config = PrereleaseConfig {
//!     tag: "beta".to_string(),
//!     mode: PrereleaseMode::Create,
//! };
//!
//! // Increment existing beta
//! let config = PrereleaseConfig {
//!     tag: "beta".to_string(),
//!     mode: PrereleaseMode::Increment,
//! };
//!
//! // Promote to stable (remove prerelease)
//! let config = PrereleaseConfig {
//!     tag: "rc".to_string(),
//!     mode: PrereleaseMode::Promote,
//! };
//! ```

use serde::{Deserialize, Serialize};

/// Configuration for prerelease version bumping.
///
/// Defines the prerelease tag (e.g., "alpha", "beta", "rc") and the behavior mode
/// (create new prerelease, increment existing, or promote to stable).
///
/// # What
///
/// Provides explicit control over prerelease version generation while maintaining
/// flexibility across different branching workflows.
///
/// # Why
///
/// Different workflows need different prerelease behaviors:
/// - GitHub Flow: Simple workflow, may not use prereleases
/// - Gitflow: Beta in develop, RC before merge to main
/// - Custom: Alpha in staging, stable in production
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::prerelease::{PrereleaseConfig, PrereleaseMode};
///
/// // Create new beta prerelease
/// let config = PrereleaseConfig {
///     tag: "beta".to_string(),
///     mode: PrereleaseMode::Create,
/// };
///
/// assert_eq!(config.tag, "beta");
/// assert_eq!(config.mode, PrereleaseMode::Create);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrereleaseConfig {
    /// Prerelease tag (e.g., "alpha", "beta", "rc").
    ///
    /// Valid tags follow SemVer 2.0.0 specification:
    /// - Only ASCII alphanumerics and hyphens `[0-9A-Za-z-]`
    /// - Common conventions: "alpha", "beta", "rc" (release candidate)
    pub tag: String,

    /// Behavior mode for prerelease version bumping.
    ///
    /// Determines how the version will be modified:
    /// - `Create`: Generate new prerelease from stable
    /// - `Increment`: Increment existing prerelease number
    /// - `Promote`: Remove prerelease (promote to stable)
    pub mode: PrereleaseMode,
}

impl PrereleaseConfig {
    /// Creates a new `PrereleaseConfig` with the specified tag and mode.
    ///
    /// # Arguments
    ///
    /// * `tag` - Prerelease tag (e.g., "alpha", "beta", "rc")
    /// * `mode` - Behavior mode
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::prerelease::{PrereleaseConfig, PrereleaseMode};
    ///
    /// let config = PrereleaseConfig::new("beta".to_string(), PrereleaseMode::Create);
    /// assert_eq!(config.tag, "beta");
    /// ```
    #[must_use]
    pub fn new(tag: String, mode: PrereleaseMode) -> Self {
        Self { tag, mode }
    }

    /// Creates a configuration for creating a new prerelease.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::prerelease::{PrereleaseConfig, PrereleaseMode};
    ///
    /// let config = PrereleaseConfig::create("beta".to_string());
    /// assert_eq!(config.mode, PrereleaseMode::Create);
    /// ```
    #[must_use]
    pub fn create(tag: String) -> Self {
        Self::new(tag, PrereleaseMode::Create)
    }

    /// Creates a configuration for incrementing an existing prerelease.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::prerelease::{PrereleaseConfig, PrereleaseMode};
    ///
    /// let config = PrereleaseConfig::increment("beta".to_string());
    /// assert_eq!(config.mode, PrereleaseMode::Increment);
    /// ```
    #[must_use]
    pub fn increment(tag: String) -> Self {
        Self::new(tag, PrereleaseMode::Increment)
    }

    /// Creates a configuration for promoting to stable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::prerelease::{PrereleaseConfig, PrereleaseMode};
    ///
    /// let config = PrereleaseConfig::promote("rc".to_string());
    /// assert_eq!(config.mode, PrereleaseMode::Promote);
    /// ```
    #[must_use]
    pub fn promote(tag: String) -> Self {
        Self::new(tag, PrereleaseMode::Promote)
    }

    /// Validates the prerelease tag format.
    ///
    /// Ensures the tag contains only ASCII alphanumerics and hyphens.
    ///
    /// # Returns
    ///
    /// `true` if the tag is valid, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::prerelease::{PrereleaseConfig, PrereleaseMode};
    ///
    /// let valid = PrereleaseConfig::new("beta".to_string(), PrereleaseMode::Create);
    /// assert!(valid.is_valid_tag());
    ///
    /// let invalid = PrereleaseConfig::new("beta@1".to_string(), PrereleaseMode::Create);
    /// assert!(!invalid.is_valid_tag());
    /// ```
    #[must_use]
    pub fn is_valid_tag(&self) -> bool {
        !self.tag.is_empty() && self.tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    }
}

/// Prerelease version bump mode.
///
/// Defines the behavior when bumping a version with prerelease support.
///
/// # Variants
///
/// - **Create**: Generate new prerelease from stable version
///   - Example: `1.2.3` → `1.3.0-beta.0`
/// - **Increment**: Increment existing prerelease number
///   - Example: `1.3.0-beta.0` → `1.3.0-beta.1`
/// - **Promote**: Remove prerelease tag (promote to stable)
///   - Example: `1.3.0-rc.1` → `1.3.0`
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::prerelease::PrereleaseMode;
///
/// // Create a new prerelease
/// let create = PrereleaseMode::Create;
///
/// // Increment existing prerelease
/// let increment = PrereleaseMode::Increment;
///
/// // Promote to stable
/// let promote = PrereleaseMode::Promote;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrereleaseMode {
    /// Create new prerelease from stable: `1.2.3` → `1.3.0-beta.0`.
    Create,

    /// Increment existing prerelease: `1.3.0-beta.0` → `1.3.0-beta.1`.
    Increment,

    /// Promote to stable (remove prerelease): `1.3.0-rc.1` → `1.3.0`.
    Promote,
}

impl PrereleaseMode {
    /// Returns the string representation of the mode.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::prerelease::PrereleaseMode;
    ///
    /// assert_eq!(PrereleaseMode::Create.as_str(), "create");
    /// assert_eq!(PrereleaseMode::Increment.as_str(), "increment");
    /// assert_eq!(PrereleaseMode::Promote.as_str(), "promote");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Increment => "increment",
            Self::Promote => "promote",
        }
    }
}

impl std::fmt::Display for PrereleaseMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prerelease_config_new() {
        let config = PrereleaseConfig::new("beta".to_string(), PrereleaseMode::Create);
        assert_eq!(config.tag, "beta");
        assert_eq!(config.mode, PrereleaseMode::Create);
    }

    #[test]
    fn test_prerelease_config_create() {
        let config = PrereleaseConfig::create("alpha".to_string());
        assert_eq!(config.tag, "alpha");
        assert_eq!(config.mode, PrereleaseMode::Create);
    }

    #[test]
    fn test_prerelease_config_increment() {
        let config = PrereleaseConfig::increment("beta".to_string());
        assert_eq!(config.tag, "beta");
        assert_eq!(config.mode, PrereleaseMode::Increment);
    }

    #[test]
    fn test_prerelease_config_promote() {
        let config = PrereleaseConfig::promote("rc".to_string());
        assert_eq!(config.tag, "rc");
        assert_eq!(config.mode, PrereleaseMode::Promote);
    }

    #[test]
    fn test_is_valid_tag() {
        // Valid tags
        assert!(PrereleaseConfig::create("alpha".to_string()).is_valid_tag());
        assert!(PrereleaseConfig::create("beta".to_string()).is_valid_tag());
        assert!(PrereleaseConfig::create("rc".to_string()).is_valid_tag());
        assert!(PrereleaseConfig::create("beta-1".to_string()).is_valid_tag());
        assert!(PrereleaseConfig::create("RC1".to_string()).is_valid_tag());

        // Invalid tags
        assert!(!PrereleaseConfig::create("".to_string()).is_valid_tag());
        assert!(!PrereleaseConfig::create("beta@1".to_string()).is_valid_tag());
        assert!(!PrereleaseConfig::create("beta.1".to_string()).is_valid_tag());
        assert!(!PrereleaseConfig::create("beta 1".to_string()).is_valid_tag());
    }

    #[test]
    fn test_prerelease_mode_as_str() {
        assert_eq!(PrereleaseMode::Create.as_str(), "create");
        assert_eq!(PrereleaseMode::Increment.as_str(), "increment");
        assert_eq!(PrereleaseMode::Promote.as_str(), "promote");
    }

    #[test]
    fn test_prerelease_mode_display() {
        assert_eq!(format!("{}", PrereleaseMode::Create), "create");
        assert_eq!(format!("{}", PrereleaseMode::Increment), "increment");
        assert_eq!(format!("{}", PrereleaseMode::Promote), "promote");
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_prerelease_config_serialization() {
        let config = PrereleaseConfig::create("beta".to_string());
        let json = serde_json::to_string(&config).expect("Failed to serialize");
        let deserialized: PrereleaseConfig =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(config, deserialized);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_prerelease_mode_serialization() {
        let mode = PrereleaseMode::Create;
        let json = serde_json::to_string(&mode).expect("Failed to serialize");
        assert_eq!(json, "\"create\"");

        let deserialized: PrereleaseMode =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(mode, deserialized);
    }
}
