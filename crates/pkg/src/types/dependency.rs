//! Dependency-related types and utilities.
//!
//! **What**: Provides types and utilities for working with package dependencies,
//! including protocol detection, dependency updates, circular dependency tracking,
//! and dependency relationship classification.
//!
//! **How**: This module defines enums for protocol types (workspace, file, link, portal),
//! structures for tracking dependency updates and circular dependencies, and helper
//! functions for protocol detection in version specifications.
//!
//! **Why**: To support dependency management operations like version propagation,
//! protocol filtering, and circular dependency detection in monorepo workflows.
//!
//! # Core Types
//!
//! ## Protocol Types
//!
//! - [`VersionProtocol`]: Enum representing different version specification protocols
//! - [`LocalLinkType`]: Specific types of local file protocols
//!
//! ## Dependency Tracking
//!
//! - [`DependencyUpdate`]: Represents a change to a dependency version
//! - [`CircularDependency`]: Represents a circular dependency cycle
//! - [`UpdateReason`]: Why a package is being updated (direct or propagated)
//!
//! # Protocol Detection
//!
//! The module provides helper functions to detect and categorize version specifications:
//!
//! ```rust
//! use sublime_pkg_tools::types::dependency::{is_workspace_protocol, is_local_protocol};
//!
//! assert!(is_workspace_protocol("workspace:*"));
//! assert!(is_workspace_protocol("workspace:^1.0.0"));
//! assert!(is_local_protocol("file:../local-lib"));
//! assert!(is_local_protocol("link:./shared"));
//! ```
//!
//! # Examples
//!
//! ## Working with Dependency Updates
//!
//! ```rust,ignore
//! use sublime_pkg_tools::types::{DependencyType, dependency::DependencyUpdate};
//!
//! let update = DependencyUpdate {
//!     dependency_name: "my-package".to_string(),
//!     dependency_type: DependencyType::Regular,
//!     old_version_spec: "^1.0.0".to_string(),
//!     new_version_spec: "^2.0.0".to_string(),
//! };
//! ```
//!
//! ## Detecting Circular Dependencies
//!
//! ```rust,ignore
//! use sublime_pkg_tools::types::dependency::CircularDependency;
//!
//! let cycle = CircularDependency {
//!     cycle: vec![
//!         "package-a".to_string(),
//!         "package-b".to_string(),
//!         "package-a".to_string(),
//!     ],
//! };
//! ```

use crate::types::DependencyType;
use serde::{Deserialize, Serialize};

/// Version specification protocol type.
///
/// Categorizes the protocol used in a dependency version specification.
/// This is used to determine how version specifications should be handled
/// during dependency resolution and propagation.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::dependency::VersionProtocol;
///
/// let protocol = VersionProtocol::Workspace;
/// assert_eq!(protocol.as_str(), "workspace:");
///
/// let local = VersionProtocol::Local(
///     sublime_pkg_tools::types::dependency::LocalLinkType::File
/// );
/// assert_eq!(local.prefix(), "file:");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VersionProtocol {
    /// Workspace protocol (workspace:*, workspace:^1.0.0, etc.)
    Workspace,
    /// Local file/link protocol (file:, link:, portal:)
    Local(LocalLinkType),
    /// Standard semantic version or range
    Semver,
}

impl VersionProtocol {
    /// Returns the string prefix for this protocol.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::dependency::{VersionProtocol, LocalLinkType};
    ///
    /// assert_eq!(VersionProtocol::Workspace.as_str(), "workspace:");
    /// assert_eq!(
    ///     VersionProtocol::Local(LocalLinkType::File).prefix(),
    ///     "file:"
    /// );
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Workspace => "workspace:",
            Self::Local(link_type) => link_type.as_str(),
            Self::Semver => "",
        }
    }

    /// Returns the protocol prefix (alias for as_str for consistency).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::dependency::VersionProtocol;
    ///
    /// assert_eq!(VersionProtocol::Workspace.prefix(), "workspace:");
    /// ```
    #[must_use]
    pub fn prefix(&self) -> &'static str {
        self.as_str()
    }

    /// Parses a version specification string to determine its protocol.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::dependency::{VersionProtocol, LocalLinkType};
    ///
    /// assert_eq!(
    ///     VersionProtocol::parse("workspace:*"),
    ///     VersionProtocol::Workspace
    /// );
    /// assert_eq!(
    ///     VersionProtocol::parse("file:../lib"),
    ///     VersionProtocol::Local(LocalLinkType::File)
    /// );
    /// assert_eq!(
    ///     VersionProtocol::parse("^1.0.0"),
    ///     VersionProtocol::Semver
    /// );
    /// ```
    #[must_use]
    pub fn parse(version_spec: &str) -> Self {
        if version_spec.starts_with("workspace:") {
            Self::Workspace
        } else if version_spec.starts_with("file:") {
            Self::Local(LocalLinkType::File)
        } else if version_spec.starts_with("link:") {
            Self::Local(LocalLinkType::Link)
        } else if version_spec.starts_with("portal:") {
            Self::Local(LocalLinkType::Portal)
        } else {
            Self::Semver
        }
    }

    /// Returns `true` if this protocol should be skipped during version resolution.
    ///
    /// Workspace and local protocols are typically skipped because they refer
    /// to packages within the same workspace or local filesystem.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::dependency::VersionProtocol;
    ///
    /// assert!(VersionProtocol::Workspace.should_skip());
    /// assert!(!VersionProtocol::Semver.should_skip());
    /// ```
    #[must_use]
    pub fn should_skip(&self) -> bool {
        !matches!(self, Self::Semver)
    }
}

impl std::fmt::Display for VersionProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Workspace => write!(f, "workspace"),
            Self::Local(link_type) => write!(f, "{}", link_type),
            Self::Semver => write!(f, "semver"),
        }
    }
}

/// Type of local link protocol.
///
/// Represents different ways to reference local packages or files
/// in package.json dependencies.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::dependency::LocalLinkType;
///
/// let file_link = LocalLinkType::File;
/// assert_eq!(file_link.as_str(), "file:");
///
/// let portal_link = LocalLinkType::Portal;
/// assert_eq!(portal_link.as_str(), "portal:");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LocalLinkType {
    /// File protocol (file:../path)
    File,
    /// Link protocol (link:./path)
    Link,
    /// Portal protocol (portal:./path)
    Portal,
}

impl LocalLinkType {
    /// Returns the protocol prefix string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::dependency::LocalLinkType;
    ///
    /// assert_eq!(LocalLinkType::File.as_str(), "file:");
    /// assert_eq!(LocalLinkType::Link.as_str(), "link:");
    /// assert_eq!(LocalLinkType::Portal.as_str(), "portal:");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::File => "file:",
            Self::Link => "link:",
            Self::Portal => "portal:",
        }
    }
}

impl std::fmt::Display for LocalLinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::File => "file",
            Self::Link => "link",
            Self::Portal => "portal",
        })
    }
}

/// Represents an update to a dependency version.
///
/// Used during version resolution to track what dependency versions
/// need to be updated in package.json files.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::types::{DependencyType, dependency::DependencyUpdate};
///
/// let update = DependencyUpdate {
///     dependency_name: "@myorg/core".to_string(),
///     dependency_type: DependencyType::Regular,
///     old_version_spec: "^1.0.0".to_string(),
///     new_version_spec: "^2.0.0".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyUpdate {
    /// Name of the dependency being updated
    pub dependency_name: String,

    /// Type of dependency (regular, dev, peer, optional)
    pub dependency_type: DependencyType,

    /// Previous version specification
    pub old_version_spec: String,

    /// New version specification
    pub new_version_spec: String,
}

impl DependencyUpdate {
    /// Creates a new dependency update.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{DependencyType, dependency::DependencyUpdate};
    ///
    /// let update = DependencyUpdate::new(
    ///     "my-package",
    ///     DependencyType::Regular,
    ///     "^1.0.0",
    ///     "^2.0.0",
    /// );
    ///
    /// assert_eq!(update.dependency_name, "my-package");
    /// assert_eq!(update.old_version_spec, "^1.0.0");
    /// ```
    #[must_use]
    pub fn new(
        dependency_name: impl Into<String>,
        dependency_type: DependencyType,
        old_version_spec: impl Into<String>,
        new_version_spec: impl Into<String>,
    ) -> Self {
        Self {
            dependency_name: dependency_name.into(),
            dependency_type,
            old_version_spec: old_version_spec.into(),
            new_version_spec: new_version_spec.into(),
        }
    }

    /// Returns `true` if this update involves a workspace protocol.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{DependencyType, dependency::DependencyUpdate};
    ///
    /// let update = DependencyUpdate::new(
    ///     "my-package",
    ///     DependencyType::Regular,
    ///     "workspace:*",
    ///     "workspace:^2.0.0",
    /// );
    ///
    /// assert!(update.is_workspace_protocol());
    /// ```
    #[must_use]
    pub fn is_workspace_protocol(&self) -> bool {
        is_workspace_protocol(&self.old_version_spec)
            || is_workspace_protocol(&self.new_version_spec)
    }

    /// Returns `true` if this update involves a local protocol.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{DependencyType, dependency::DependencyUpdate};
    ///
    /// let update = DependencyUpdate::new(
    ///     "my-package",
    ///     DependencyType::Regular,
    ///     "file:../lib",
    ///     "file:../lib",
    /// );
    ///
    /// assert!(update.is_local_protocol());
    /// ```
    #[must_use]
    pub fn is_local_protocol(&self) -> bool {
        is_local_protocol(&self.old_version_spec) || is_local_protocol(&self.new_version_spec)
    }
}

/// Represents a circular dependency cycle.
///
/// Contains the chain of package names that form a circular dependency.
/// The last element typically equals the first element to complete the cycle.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::dependency::CircularDependency;
///
/// let cycle = CircularDependency::new(vec![
///     "package-a".to_string(),
///     "package-b".to_string(),
///     "package-c".to_string(),
///     "package-a".to_string(),
/// ]);
///
/// assert_eq!(cycle.len(), 4);
/// assert!(cycle.involves("package-b"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircularDependency {
    /// Packages involved in the circular dependency chain
    pub cycle: Vec<String>,
}

impl CircularDependency {
    /// Creates a new circular dependency.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::dependency::CircularDependency;
    ///
    /// let cycle = CircularDependency::new(vec![
    ///     "pkg-a".to_string(),
    ///     "pkg-b".to_string(),
    ///     "pkg-a".to_string(),
    /// ]);
    ///
    /// assert_eq!(cycle.len(), 3);
    /// ```
    #[must_use]
    pub fn new(cycle: Vec<String>) -> Self {
        Self { cycle }
    }

    /// Returns the number of packages in the cycle.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::dependency::CircularDependency;
    ///
    /// let cycle = CircularDependency::new(vec![
    ///     "a".to_string(),
    ///     "b".to_string(),
    ///     "a".to_string(),
    /// ]);
    ///
    /// assert_eq!(cycle.len(), 3);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.cycle.len()
    }

    /// Returns `true` if the cycle is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::dependency::CircularDependency;
    ///
    /// let empty = CircularDependency::new(vec![]);
    /// assert!(empty.is_empty());
    ///
    /// let cycle = CircularDependency::new(vec!["a".to_string(), "b".to_string()]);
    /// assert!(!cycle.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cycle.is_empty()
    }

    /// Returns `true` if the given package is involved in this cycle.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::dependency::CircularDependency;
    ///
    /// let cycle = CircularDependency::new(vec![
    ///     "pkg-a".to_string(),
    ///     "pkg-b".to_string(),
    /// ]);
    ///
    /// assert!(cycle.involves("pkg-a"));
    /// assert!(cycle.involves("pkg-b"));
    /// assert!(!cycle.involves("pkg-c"));
    /// ```
    #[must_use]
    pub fn involves(&self, package_name: &str) -> bool {
        self.cycle.iter().any(|p| p == package_name)
    }

    /// Returns a formatted string representation of the cycle.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::dependency::CircularDependency;
    ///
    /// let cycle = CircularDependency::new(vec![
    ///     "a".to_string(),
    ///     "b".to_string(),
    ///     "a".to_string(),
    /// ]);
    ///
    /// assert_eq!(cycle.display_cycle(), "a -> b -> a");
    /// ```
    #[must_use]
    pub fn display_cycle(&self) -> String {
        self.cycle.join(" -> ")
    }
}

impl std::fmt::Display for CircularDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Circular dependency: {}", self.display_cycle())
    }
}

/// Reason why a package is being updated.
///
/// Used during version resolution to track whether a package is being
/// updated due to direct changes or as a result of dependency propagation.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::dependency::UpdateReason;
///
/// let direct = UpdateReason::DirectChange;
/// assert!(direct.is_direct());
///
/// let propagated = UpdateReason::DependencyPropagation {
///     triggered_by: "core-package".to_string(),
///     depth: 2,
/// };
/// assert!(!propagated.is_direct());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateReason {
    /// Package has direct changes (in changeset)
    DirectChange,

    /// Package is updated due to dependency propagation
    DependencyPropagation {
        /// Package that triggered this update
        triggered_by: String,
        /// Depth in the dependency chain (1 = direct dependent)
        depth: usize,
    },
}

impl UpdateReason {
    /// Returns `true` if this is a direct change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::dependency::UpdateReason;
    ///
    /// assert!(UpdateReason::DirectChange.is_direct());
    /// assert!(!UpdateReason::DependencyPropagation {
    ///     triggered_by: "pkg".to_string(),
    ///     depth: 1,
    /// }.is_direct());
    /// ```
    #[must_use]
    pub fn is_direct(&self) -> bool {
        matches!(self, Self::DirectChange)
    }

    /// Returns `true` if this is a propagated update.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::dependency::UpdateReason;
    ///
    /// assert!(!UpdateReason::DirectChange.is_propagated());
    /// assert!(UpdateReason::DependencyPropagation {
    ///     triggered_by: "pkg".to_string(),
    ///     depth: 1,
    /// }.is_propagated());
    /// ```
    #[must_use]
    pub fn is_propagated(&self) -> bool {
        matches!(self, Self::DependencyPropagation { .. })
    }

    /// Returns the depth of propagation, or 0 for direct changes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::dependency::UpdateReason;
    ///
    /// assert_eq!(UpdateReason::DirectChange.depth(), 0);
    /// assert_eq!(
    ///     UpdateReason::DependencyPropagation {
    ///         triggered_by: "pkg".to_string(),
    ///         depth: 3,
    ///     }.depth(),
    ///     3
    /// );
    /// ```
    #[must_use]
    pub fn depth(&self) -> usize {
        match self {
            Self::DirectChange => 0,
            Self::DependencyPropagation { depth, .. } => *depth,
        }
    }
}

impl std::fmt::Display for UpdateReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DirectChange => write!(f, "direct change"),
            Self::DependencyPropagation { triggered_by, depth } => {
                write!(f, "dependency propagation (triggered by {}, depth {})", triggered_by, depth)
            }
        }
    }
}

/// Information about a package update during version resolution.
///
/// # Note
///
/// This type will be fully implemented in story 5.4 (Version Resolution Logic).
/// The structure is defined here as part of the dependency types foundation.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::types::dependency::{PackageUpdate, UpdateReason};
/// use sublime_pkg_tools::types::Version;
/// use std::path::PathBuf;
///
/// let update = PackageUpdate {
///     name: "@myorg/core".to_string(),
///     path: PathBuf::from("/workspace/packages/core"),
///     current_version: Version::parse("1.0.0")?,
///     next_version: Version::parse("1.1.0")?,
///     reason: UpdateReason::DirectChange,
///     dependency_updates: vec![],
/// };
/// ```
// PackageUpdate is now defined in version::resolution module to avoid duplication.
// It is re-exported from the types module for convenience.
// See: src/version/resolution.rs for the canonical definition.

// Helper functions for protocol detection

/// Checks if a version specification uses the workspace protocol.
///
/// Returns `true` if the version spec starts with "workspace:".
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::dependency::is_workspace_protocol;
///
/// assert!(is_workspace_protocol("workspace:*"));
/// assert!(is_workspace_protocol("workspace:^1.0.0"));
/// assert!(is_workspace_protocol("workspace:~2.3.0"));
/// assert!(!is_workspace_protocol("^1.0.0"));
/// assert!(!is_workspace_protocol("file:../lib"));
/// ```
#[must_use]
pub fn is_workspace_protocol(version_spec: &str) -> bool {
    version_spec.starts_with("workspace:")
}

/// Checks if a version specification uses a local file protocol.
///
/// Returns `true` if the version spec starts with "file:", "link:", or "portal:".
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::dependency::is_local_protocol;
///
/// assert!(is_local_protocol("file:../lib"));
/// assert!(is_local_protocol("link:./shared"));
/// assert!(is_local_protocol("portal:./packages/core"));
/// assert!(!is_local_protocol("workspace:*"));
/// assert!(!is_local_protocol("^1.0.0"));
/// ```
#[must_use]
pub fn is_local_protocol(version_spec: &str) -> bool {
    version_spec.starts_with("file:")
        || version_spec.starts_with("link:")
        || version_spec.starts_with("portal:")
}

/// Checks if a version specification should be skipped during version resolution.
///
/// Returns `true` if the version spec uses workspace or local protocols.
/// These types of dependencies are typically not updated during version bumps
/// as they reference packages within the same workspace or local filesystem.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::dependency::should_skip_protocol;
///
/// assert!(should_skip_protocol("workspace:*"));
/// assert!(should_skip_protocol("file:../lib"));
/// assert!(should_skip_protocol("link:./shared"));
/// assert!(should_skip_protocol("portal:./pkg"));
/// assert!(!should_skip_protocol("^1.0.0"));
/// assert!(!should_skip_protocol("~2.3.4"));
/// ```
#[must_use]
pub fn should_skip_protocol(version_spec: &str) -> bool {
    is_workspace_protocol(version_spec) || is_local_protocol(version_spec)
}

/// Extracts the path from a local protocol specification.
///
/// Returns the path portion after the protocol prefix, or the original
/// string if no recognized protocol is found.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::dependency::extract_protocol_path;
///
/// assert_eq!(extract_protocol_path("file:../lib"), "../lib");
/// assert_eq!(extract_protocol_path("link:./shared"), "./shared");
/// assert_eq!(extract_protocol_path("portal:./pkg"), "./pkg");
/// assert_eq!(extract_protocol_path("workspace:*"), "*");
/// assert_eq!(extract_protocol_path("^1.0.0"), "^1.0.0");
/// ```
#[must_use]
pub fn extract_protocol_path(version_spec: &str) -> &str {
    if let Some(stripped) = version_spec.strip_prefix("file:") {
        stripped
    } else if let Some(stripped) = version_spec.strip_prefix("link:") {
        stripped
    } else if let Some(stripped) = version_spec.strip_prefix("portal:") {
        stripped
    } else if let Some(stripped) = version_spec.strip_prefix("workspace:") {
        stripped
    } else {
        version_spec
    }
}

/// Parses the protocol type from a version specification.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::dependency::{parse_protocol, VersionProtocol, LocalLinkType};
///
/// assert_eq!(parse_protocol("workspace:*"), VersionProtocol::Workspace);
/// assert_eq!(parse_protocol("file:../lib"), VersionProtocol::Local(LocalLinkType::File));
/// assert_eq!(parse_protocol("link:./shared"), VersionProtocol::Local(LocalLinkType::Link));
/// assert_eq!(parse_protocol("portal:./pkg"), VersionProtocol::Local(LocalLinkType::Portal));
/// assert_eq!(parse_protocol("^1.0.0"), VersionProtocol::Semver);
/// ```
#[must_use]
pub fn parse_protocol(version_spec: &str) -> VersionProtocol {
    VersionProtocol::parse(version_spec)
}
