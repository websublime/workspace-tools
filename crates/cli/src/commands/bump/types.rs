//! Types for version bump operations.
//!
//! **What**: Provides type definitions specific to version bump commands,
//! including changeset archive policies for controlling when changesets are
//! archived after version bumps.
//!
//! **How**: Defines policy enums and related types that control bump behavior
//! such as changeset archival decisions based on version types (prerelease vs stable).
//!
//! **Why**: To provide clear, type-safe control over changeset lifecycle management
//! during version bump operations, enabling different workflows (continuous prerelease,
//! gitflow, github flow, etc.) to handle changesets appropriately.
//!
//! # Examples
//!
//! ```rust
//! use sublime_cli_tools::commands::bump::types::ChangesetArchivePolicy;
//!
//! // Auto-decide based on version type (default)
//! let policy = ChangesetArchivePolicy::Auto;
//!
//! // Never archive (useful for continuous prerelease)
//! let policy = ChangesetArchivePolicy::Never;
//!
//! // Always archive (explicit control)
//! let policy = ChangesetArchivePolicy::Always;
//! ```

use serde::{Deserialize, Serialize};

/// Policy for archiving changesets after version bump.
///
/// Defines when changesets should be archived (moved to history) after applying
/// version bumps. Different workflows need different changeset management strategies.
///
/// # What
///
/// Controls the lifecycle of changesets after they've been used for version bumps:
/// - **Auto**: Smart decision based on version type (prerelease vs stable)
/// - **Never**: Keep changesets active (useful for testing, continuous prerelease)
/// - **Always**: Archive immediately regardless of version type
///
/// # Why
///
/// Different workflows need different changeset management:
/// - **Gitflow**: Prereleases in develop need reusable changesets, final release archives
/// - **GitHub Flow**: Direct to main, typically archive immediately
/// - **Continuous Prerelease**: Never archive, always iterate on same changeset
/// - **Testing**: Manual control over archival for validation
///
/// # How
///
/// The Auto policy intelligently decides based on version type:
/// - Prerelease versions (`1.3.0-beta.0`): **DON'T** archive (changeset reusable)
/// - Stable versions (`1.3.0`): **Archive** (changeset consumed)
///
/// This enables workflows like:
/// 1. Create changeset on feature branch
/// 2. Bump to 1.3.0-beta.0 (changeset kept)
/// 3. Bump to 1.3.0-beta.1 (changeset kept)
/// 4. Bump to 1.3.0 (changeset archived)
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::bump::types::ChangesetArchivePolicy;
///
/// // Default behavior: auto-decide
/// let auto = ChangesetArchivePolicy::Auto;
/// assert!(matches!(auto, ChangesetArchivePolicy::Auto));
///
/// // Explicit never archive
/// let never = ChangesetArchivePolicy::Never;
/// assert!(matches!(never, ChangesetArchivePolicy::Never));
///
/// // Explicit always archive
/// let always = ChangesetArchivePolicy::Always;
/// assert!(matches!(always, ChangesetArchivePolicy::Always));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangesetArchivePolicy {
    /// Auto-decide based on version type.
    ///
    /// Behavior:
    /// - Prerelease versions: DON'T archive (e.g., `1.3.0-beta.0`)
    /// - Stable versions: Archive (e.g., `1.3.0`)
    ///
    /// This is the default and recommended policy for most workflows.
    ///
    /// # Examples
    ///
    /// ```text
    /// // Prerelease bump
    /// workspace bump --execute --prerelease beta
    /// → 1.3.0-beta.0
    /// → Changesets KEPT (auto policy detects prerelease)
    ///
    /// // Stable bump
    /// workspace bump --execute
    /// → 1.3.0
    /// → Changesets ARCHIVED (auto policy detects stable)
    /// ```
    Auto,

    /// Never archive changesets.
    ///
    /// CLI flag: `--no-archive`
    ///
    /// Keeps changesets active after bump. Useful for:
    /// - Testing version bump behavior
    /// - Continuous prerelease workflows
    /// - Manual changeset management
    ///
    /// # Examples
    ///
    /// ```text
    /// workspace bump --execute --prerelease beta --no-archive
    /// → 1.3.0-beta.0
    /// → Changesets KEPT (explicit)
    /// ```
    Never,

    /// Always archive changesets.
    ///
    /// CLI flag: `--always-archive`
    ///
    /// Archives changesets regardless of version type. Useful for:
    /// - Single-use changesets
    /// - Workflows where each changeset represents a complete release
    /// - Explicit control over changeset lifecycle
    ///
    /// # Examples
    ///
    /// ```text
    /// workspace bump --execute --prerelease beta --always-archive
    /// → 1.3.0-beta.0
    /// → Changesets ARCHIVED (explicit)
    /// ```
    Always,
}

impl ChangesetArchivePolicy {
    /// Returns the string representation of the policy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::bump::types::ChangesetArchivePolicy;
    ///
    /// assert_eq!(ChangesetArchivePolicy::Auto.as_str(), "auto");
    /// assert_eq!(ChangesetArchivePolicy::Never.as_str(), "never");
    /// assert_eq!(ChangesetArchivePolicy::Always.as_str(), "always");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Never => "never",
            Self::Always => "always",
        }
    }

    /// Checks if this is the Auto policy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::bump::types::ChangesetArchivePolicy;
    ///
    /// assert!(ChangesetArchivePolicy::Auto.is_auto());
    /// assert!(!ChangesetArchivePolicy::Never.is_auto());
    /// ```
    #[must_use]
    pub fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }

    /// Checks if this is the Never policy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::bump::types::ChangesetArchivePolicy;
    ///
    /// assert!(ChangesetArchivePolicy::Never.is_never());
    /// assert!(!ChangesetArchivePolicy::Auto.is_never());
    /// ```
    #[must_use]
    pub fn is_never(&self) -> bool {
        matches!(self, Self::Never)
    }

    /// Checks if this is the Always policy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::bump::types::ChangesetArchivePolicy;
    ///
    /// assert!(ChangesetArchivePolicy::Always.is_always());
    /// assert!(!ChangesetArchivePolicy::Auto.is_always());
    /// ```
    #[must_use]
    pub fn is_always(&self) -> bool {
        matches!(self, Self::Always)
    }
}

impl std::fmt::Display for ChangesetArchivePolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for ChangesetArchivePolicy {
    /// Returns the default archive policy: Auto.
    fn default() -> Self {
        Self::Auto
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_policy_as_str() {
        assert_eq!(ChangesetArchivePolicy::Auto.as_str(), "auto");
        assert_eq!(ChangesetArchivePolicy::Never.as_str(), "never");
        assert_eq!(ChangesetArchivePolicy::Always.as_str(), "always");
    }

    #[test]
    fn test_archive_policy_display() {
        assert_eq!(format!("{}", ChangesetArchivePolicy::Auto), "auto");
        assert_eq!(format!("{}", ChangesetArchivePolicy::Never), "never");
        assert_eq!(format!("{}", ChangesetArchivePolicy::Always), "always");
    }

    #[test]
    fn test_archive_policy_default() {
        let default = ChangesetArchivePolicy::default();
        assert_eq!(default, ChangesetArchivePolicy::Auto);
    }

    #[test]
    fn test_archive_policy_is_auto() {
        assert!(ChangesetArchivePolicy::Auto.is_auto());
        assert!(!ChangesetArchivePolicy::Never.is_auto());
        assert!(!ChangesetArchivePolicy::Always.is_auto());
    }

    #[test]
    fn test_archive_policy_is_never() {
        assert!(!ChangesetArchivePolicy::Auto.is_never());
        assert!(ChangesetArchivePolicy::Never.is_never());
        assert!(!ChangesetArchivePolicy::Always.is_never());
    }

    #[test]
    fn test_archive_policy_is_always() {
        assert!(!ChangesetArchivePolicy::Auto.is_always());
        assert!(!ChangesetArchivePolicy::Never.is_always());
        assert!(ChangesetArchivePolicy::Always.is_always());
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_archive_policy_serialization() {
        let auto = ChangesetArchivePolicy::Auto;
        let json = serde_json::to_string(&auto).expect("Failed to serialize");
        assert_eq!(json, "\"auto\"");

        let deserialized: ChangesetArchivePolicy =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(auto, deserialized);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_all_policies_serialize_correctly() {
        let policies = vec![
            ChangesetArchivePolicy::Auto,
            ChangesetArchivePolicy::Never,
            ChangesetArchivePolicy::Always,
        ];

        for policy in policies {
            let json = serde_json::to_string(&policy).expect("Failed to serialize");
            let deserialized: ChangesetArchivePolicy =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(policy, deserialized);
        }
    }
}
