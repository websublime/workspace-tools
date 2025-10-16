//! Package information and metadata types.
//!
//! **What**: Provides types for representing Node.js package information, including
//! package.json data, workspace context, and dependency relationships.
//!
//! **How**: Wraps the `package-json` crate's `PackageJson` type along with workspace
//! metadata from `sublime_standard_tools` to provide a unified view of a package's
//! information. Implements helper methods for accessing common fields and filtering
//! dependencies based on version spec protocols (workspace:, file:, link:, portal:).
//!
//! **Why**: To provide a consistent, type-safe way to work with package metadata across
//! the package tools system, enabling version resolution, dependency analysis, and
//! changeset management operations.
//!
//! # Core Types
//!
//! ## PackageInfo
//!
//! The main type representing a package's complete information, including:
//! - Package.json metadata (name, version, dependencies, etc.)
//! - Optional workspace context (for monorepo packages)
//! - Filesystem location
//!
//! ## DependencyType
//!
//! Enum categorizing the type of dependency relationship.
//!
//! # Examples
//!
//! ## Creating PackageInfo
//!
//! ```rust
//! use sublime_pkg_tools::types::PackageInfo;
//! use std::path::PathBuf;
//! use package_json::PackageJson;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load package.json from a file
//! let package_json = PackageJson::load("path/to/package.json")?;
//! let package_info = PackageInfo::new(
//!     package_json,
//!     None, // No workspace context
//!     PathBuf::from("path/to/package")
//! );
//!
//! println!("Package: {}", package_info.name());
//! println!("Version: {}", package_info.version());
//! # Ok(())
//! # }
//! ```
//!
//! ## Accessing Dependencies
//!
//! ```rust
//! use sublime_pkg_tools::types::{PackageInfo, DependencyType};
//! use package_json::PackageJson;
//! use std::path::PathBuf;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let package_json = PackageJson::load("path/to/package.json")?;
//! let package_info = PackageInfo::new(package_json, None, PathBuf::from("."));
//!
//! // Get all dependencies (filters out workspace: and local protocols)
//! let all_deps = package_info.all_dependencies();
//! for (name, version, dep_type) in all_deps {
//!     println!("{} @ {} ({:?})", name, version, dep_type);
//! }
//!
//! // Get only production dependencies
//! let prod_deps = package_info.dependencies();
//! # Ok(())
//! # }
//! ```
//!
//! ## Filtering Internal Dependencies
//!
//! ```rust
//! use sublime_pkg_tools::types::PackageInfo;
//! use package_json::PackageJson;
//! use std::path::PathBuf;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let package_json = PackageJson::load("path/to/package.json")?;
//! let package_info = PackageInfo::new(package_json, None, PathBuf::from("."));
//!
//! // Check if package is internal (has workspace dependencies)
//! if package_info.is_internal() {
//!     println!("Package has workspace dependencies");
//! }
//!
//! // Get internal dependencies
//! let internal = package_info.internal_dependencies();
//! println!("Internal dependencies: {:?}", internal);
//!
//! // Get external dependencies
//! let external = package_info.external_dependencies();
//! println!("External dependencies: {:?}", external);
//! # Ok(())
//! # }
//! ```

use package_json::PackageJson;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use sublime_standard_tools::monorepo::WorkspacePackage;

use crate::types::Version;

/// Information about a package in the workspace.
///
/// This type aggregates all relevant information about a Node.js package,
/// including its package.json metadata, optional workspace context (for monorepos),
/// and filesystem location.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::PackageInfo;
/// use package_json::PackageJson;
/// use std::path::PathBuf;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let package_json = PackageJson::load("package.json")?;
/// let info = PackageInfo::new(
///     package_json,
///     None,
///     PathBuf::from("/path/to/package")
/// );
///
/// println!("Package: {}", info.name());
/// println!("Version: {}", info.version());
/// println!("Path: {}", info.path().display());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct PackageInfo {
    /// Package metadata from package.json
    package_json: PackageJson,

    /// Workspace metadata (if in monorepo)
    workspace: Option<WorkspacePackage>,

    /// Absolute path to package directory
    path: PathBuf,
}

impl PackageInfo {
    /// Creates a new `PackageInfo` instance.
    ///
    /// # Arguments
    ///
    /// * `package_json` - The parsed package.json data
    /// * `workspace` - Optional workspace package information (for monorepos)
    /// * `path` - Absolute path to the package directory
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::PackageInfo;
    /// use package_json::PackageJson;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let package_json = PackageJson::load("package.json")?;
    /// let info = PackageInfo::new(
    ///     package_json,
    ///     None,
    ///     PathBuf::from("/path/to/package")
    /// );
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn new(
        package_json: PackageJson,
        workspace: Option<WorkspacePackage>,
        path: PathBuf,
    ) -> Self {
        Self { package_json, workspace, path }
    }

    /// Returns the package name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::PackageInfo;
    /// use package_json::PackageJson;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let package_json = PackageJson::load("package.json")?;
    /// let info = PackageInfo::new(package_json, None, PathBuf::from("."));
    /// println!("Package name: {}", info.name());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.package_json.name
    }

    /// Returns the current version of the package.
    ///
    /// If the version string in package.json cannot be parsed as a valid semantic
    /// version, returns a default version of `0.0.0`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{PackageInfo, Version};
    /// use package_json::PackageJson;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let package_json = PackageJson::load("package.json")?;
    /// let info = PackageInfo::new(package_json, None, PathBuf::from("."));
    /// let version = info.version();
    /// println!("Current version: {}", version);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn version(&self) -> Version {
        Version::parse(&self.package_json.version).unwrap_or_else(|_| Version::new(0, 0, 0))
    }

    /// Returns a reference to the package.json data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::PackageInfo;
    /// use package_json::PackageJson;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let package_json = PackageJson::load("package.json")?;
    /// let info = PackageInfo::new(package_json, None, PathBuf::from("."));
    /// let pkg_json = info.package_json();
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn package_json(&self) -> &PackageJson {
        &self.package_json
    }

    /// Returns a reference to the workspace package information, if available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::PackageInfo;
    /// use package_json::PackageJson;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let package_json = PackageJson::load("package.json")?;
    /// let info = PackageInfo::new(package_json, None, PathBuf::from("."));
    /// if let Some(workspace) = info.workspace() {
    ///     println!("Workspace package: {}", workspace.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn workspace(&self) -> Option<&WorkspacePackage> {
        self.workspace.as_ref()
    }

    /// Returns the absolute path to the package directory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::PackageInfo;
    /// use package_json::PackageJson;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let package_json = PackageJson::load("package.json")?;
    /// let info = PackageInfo::new(package_json, None, PathBuf::from("/path/to/package"));
    /// println!("Package path: {}", info.path().display());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns all dependencies (production, dev, and peer), excluding workspace
    /// and local protocol dependencies.
    ///
    /// This method filters out dependencies with version specs that start with:
    /// - `workspace:*` - workspace protocol
    /// - `file:` - file protocol
    /// - `link:` - link protocol
    /// - `portal:` - portal protocol
    ///
    /// # Returns
    ///
    /// A vector of tuples containing (dependency_name, version_spec, dependency_type).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{PackageInfo, DependencyType};
    /// use package_json::PackageJson;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let package_json = PackageJson::load("package.json")?;
    /// let info = PackageInfo::new(package_json, None, PathBuf::from("."));
    ///
    /// for (name, version, dep_type) in info.all_dependencies() {
    ///     match dep_type {
    ///         DependencyType::Regular => println!("dep: {} @ {}", name, version),
    ///         DependencyType::Dev => println!("devDep: {} @ {}", name, version),
    ///         DependencyType::Peer => println!("peerDep: {} @ {}", name, version),
    ///         DependencyType::Optional => println!("optionalDep: {} @ {}", name, version),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn all_dependencies(&self) -> Vec<(String, String, DependencyType)> {
        let mut deps = Vec::new();

        // Add regular dependencies
        if let Some(dependencies) = &self.package_json.dependencies {
            for (name, version) in dependencies {
                if !Self::is_skipped_version_spec(version) {
                    deps.push((name.clone(), version.clone(), DependencyType::Regular));
                }
            }
        }

        // Add dev dependencies
        if let Some(dev_dependencies) = &self.package_json.dev_dependencies {
            for (name, version) in dev_dependencies {
                if !Self::is_skipped_version_spec(version) {
                    deps.push((name.clone(), version.clone(), DependencyType::Dev));
                }
            }
        }

        // Add peer dependencies
        if let Some(peer_dependencies) = &self.package_json.peer_dependencies {
            for (name, version) in peer_dependencies {
                if !Self::is_skipped_version_spec(version) {
                    deps.push((name.clone(), version.clone(), DependencyType::Peer));
                }
            }
        }

        // Add optional dependencies
        if let Some(optional_dependencies) = &self.package_json.optional_dependencies {
            for (name, version) in optional_dependencies {
                if !Self::is_skipped_version_spec(version) {
                    deps.push((name.clone(), version.clone(), DependencyType::Optional));
                }
            }
        }

        deps
    }

    /// Returns only production dependencies, excluding workspace and local protocols.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::PackageInfo;
    /// use package_json::PackageJson;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let package_json = PackageJson::load("package.json")?;
    /// let info = PackageInfo::new(package_json, None, PathBuf::from("."));
    /// let prod_deps = info.dependencies();
    /// println!("Production dependencies: {:?}", prod_deps);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn dependencies(&self) -> HashMap<String, String> {
        self.package_json
            .dependencies
            .as_ref()
            .map(|deps| {
                deps.iter()
                    .filter(|(_, version)| !Self::is_skipped_version_spec(version))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns only development dependencies, excluding workspace and local protocols.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::PackageInfo;
    /// use package_json::PackageJson;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let package_json = PackageJson::load("package.json")?;
    /// let info = PackageInfo::new(package_json, None, PathBuf::from("."));
    /// let dev_deps = info.dev_dependencies();
    /// println!("Dev dependencies: {:?}", dev_deps);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn dev_dependencies(&self) -> HashMap<String, String> {
        self.package_json
            .dev_dependencies
            .as_ref()
            .map(|deps| {
                deps.iter()
                    .filter(|(_, version)| !Self::is_skipped_version_spec(version))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns only peer dependencies, excluding workspace and local protocols.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::PackageInfo;
    /// use package_json::PackageJson;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let package_json = PackageJson::load("package.json")?;
    /// let info = PackageInfo::new(package_json, None, PathBuf::from("."));
    /// let peer_deps = info.peer_dependencies();
    /// println!("Peer dependencies: {:?}", peer_deps);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn peer_dependencies(&self) -> HashMap<String, String> {
        self.package_json
            .peer_dependencies
            .as_ref()
            .map(|deps| {
                deps.iter()
                    .filter(|(_, version)| !Self::is_skipped_version_spec(version))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns only optional dependencies, excluding workspace and local protocols.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::PackageInfo;
    /// use package_json::PackageJson;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let package_json = PackageJson::load("package.json")?;
    /// let info = PackageInfo::new(package_json, None, PathBuf::from("."));
    /// let optional_deps = info.optional_dependencies();
    /// println!("Optional dependencies: {:?}", optional_deps);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn optional_dependencies(&self) -> HashMap<String, String> {
        self.package_json
            .optional_dependencies
            .as_ref()
            .map(|deps| {
                deps.iter()
                    .filter(|(_, version)| !Self::is_skipped_version_spec(version))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Checks if this package has internal (workspace) dependencies.
    ///
    /// Returns `true` if the package has workspace dependencies based on
    /// the workspace metadata, if available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::PackageInfo;
    /// use package_json::PackageJson;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let package_json = PackageJson::load("package.json")?;
    /// let info = PackageInfo::new(package_json, None, PathBuf::from("."));
    ///
    /// if info.is_internal() {
    ///     println!("Package has workspace dependencies");
    /// } else {
    ///     println!("Package has no workspace dependencies");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn is_internal(&self) -> bool {
        self.workspace
            .as_ref()
            .map(|ws| {
                !ws.workspace_dependencies.is_empty() || !ws.workspace_dev_dependencies.is_empty()
            })
            .unwrap_or(false)
    }

    /// Returns the list of internal (workspace) dependency names.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::PackageInfo;
    /// use package_json::PackageJson;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let package_json = PackageJson::load("package.json")?;
    /// let info = PackageInfo::new(package_json, None, PathBuf::from("."));
    ///
    /// let internal_deps = info.internal_dependencies();
    /// println!("Internal dependencies: {:?}", internal_deps);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn internal_dependencies(&self) -> Vec<String> {
        self.workspace
            .as_ref()
            .map(|ws| {
                let mut deps = ws.workspace_dependencies.clone();
                deps.extend(ws.workspace_dev_dependencies.clone());
                deps
            })
            .unwrap_or_default()
    }

    /// Returns external (non-workspace) dependencies as a map.
    ///
    /// This filters all dependencies to return only those that are not
    /// workspace dependencies (don't use workspace: protocol).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::PackageInfo;
    /// use package_json::PackageJson;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let package_json = PackageJson::load("package.json")?;
    /// let info = PackageInfo::new(package_json, None, PathBuf::from("."));
    ///
    /// let external_deps = info.external_dependencies();
    /// for (name, version) in external_deps {
    ///     println!("External dep: {} @ {}", name, version);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn external_dependencies(&self) -> HashMap<String, String> {
        let internal_deps: Vec<String> = self.internal_dependencies();
        let internal_set: std::collections::HashSet<_> = internal_deps.iter().collect();

        self.all_dependencies()
            .into_iter()
            .filter(|(name, _, _)| !internal_set.contains(name))
            .map(|(name, version, _)| (name, version))
            .collect()
    }

    /// Checks if a version specification should be skipped.
    ///
    /// Returns `true` if the version spec uses a protocol that should be filtered out:
    /// - `workspace:*` - workspace protocol
    /// - `file:` - file protocol
    /// - `link:` - link protocol
    /// - `portal:` - portal protocol
    ///
    /// These are typically internal references in monorepos or local development
    /// setups and should not be considered as external dependencies.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::PackageInfo;
    /// use package_json::PackageJson;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // These are internal/local protocols that should be skipped
    /// assert_eq!(PackageInfo::is_skipped_version_spec("workspace:*"), true);
    /// assert_eq!(PackageInfo::is_skipped_version_spec("file:../local"), true);
    /// assert_eq!(PackageInfo::is_skipped_version_spec("link:../linked"), true);
    /// assert_eq!(PackageInfo::is_skipped_version_spec("portal:../portal"), true);
    ///
    /// // Normal version specs should not be skipped
    /// assert_eq!(PackageInfo::is_skipped_version_spec("^1.2.3"), false);
    /// assert_eq!(PackageInfo::is_skipped_version_spec(">=2.0.0"), false);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn is_skipped_version_spec(version_spec: &str) -> bool {
        version_spec.starts_with("workspace:")
            || version_spec.starts_with("file:")
            || version_spec.starts_with("link:")
            || version_spec.starts_with("portal:")
    }
}

/// Type of dependency relationship.
///
/// Categorizes dependencies based on their role in the package:
/// - Regular: Production dependencies required at runtime
/// - Dev: Development dependencies needed only during development
/// - Peer: Peer dependencies expected to be provided by the consumer
/// - Optional: Optional dependencies that enhance functionality if present
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::DependencyType;
///
/// let dep_type = DependencyType::Regular;
/// assert_eq!(dep_type.as_str(), "dependencies");
///
/// let dev_type = DependencyType::Dev;
/// assert_eq!(dev_type.as_str(), "devDependencies");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyType {
    /// Production dependency (from `dependencies` field)
    Regular,
    /// Development dependency (from `devDependencies` field)
    Dev,
    /// Peer dependency (from `peerDependencies` field)
    Peer,
    /// Optional dependency (from `optionalDependencies` field)
    Optional,
}

impl DependencyType {
    /// Returns the string representation of the dependency type.
    ///
    /// This matches the field name in package.json.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::DependencyType;
    ///
    /// assert_eq!(DependencyType::Regular.as_str(), "dependencies");
    /// assert_eq!(DependencyType::Dev.as_str(), "devDependencies");
    /// assert_eq!(DependencyType::Peer.as_str(), "peerDependencies");
    /// assert_eq!(DependencyType::Optional.as_str(), "optionalDependencies");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Regular => "dependencies",
            Self::Dev => "devDependencies",
            Self::Peer => "peerDependencies",
            Self::Optional => "optionalDependencies",
        }
    }

    /// Returns `true` if this is a production dependency.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::DependencyType;
    ///
    /// assert!(DependencyType::Regular.is_production());
    /// assert!(!DependencyType::Dev.is_production());
    /// ```
    #[must_use]
    pub fn is_production(&self) -> bool {
        matches!(self, Self::Regular)
    }

    /// Returns `true` if this is a development dependency.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::DependencyType;
    ///
    /// assert!(DependencyType::Dev.is_development());
    /// assert!(!DependencyType::Regular.is_development());
    /// ```
    #[must_use]
    pub fn is_development(&self) -> bool {
        matches!(self, Self::Dev)
    }

    /// Returns `true` if this is a peer dependency.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::DependencyType;
    ///
    /// assert!(DependencyType::Peer.is_peer());
    /// assert!(!DependencyType::Regular.is_peer());
    /// ```
    #[must_use]
    pub fn is_peer(&self) -> bool {
        matches!(self, Self::Peer)
    }

    /// Returns `true` if this is an optional dependency.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::DependencyType;
    ///
    /// assert!(DependencyType::Optional.is_optional());
    /// assert!(!DependencyType::Regular.is_optional());
    /// ```
    #[must_use]
    pub fn is_optional(&self) -> bool {
        matches!(self, Self::Optional)
    }
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
