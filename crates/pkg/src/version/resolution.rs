//! Version resolution types and logic for calculating next package versions.
//!
//! **What**: Provides types and logic for resolving package versions based on changesets,
//! including `VersionResolution`, `PackageUpdate`, `UpdateReason`, and the core resolution
//! algorithm that determines which packages need version updates.
//!
//! **How**: Takes a changeset with a version bump type and a list of affected packages,
//! then calculates the next version for each package based on their current version and
//! the bump type. Returns a complete resolution with all updates and their reasons.
//!
//! **Why**: To provide a clear, type-safe representation of version changes before they
//! are applied, enabling dry-run previews, validation, and safe version updates with
//! full traceability of why each package is being updated.
//!
//! # Resolution Process
//!
//! The resolution process follows these steps:
//!
//! 1. **Validation**: Verify all packages in changeset exist and have valid versions
//! 2. **Direct Resolution**: Calculate next versions for packages in the changeset
//! 3. **Update Creation**: Create `PackageUpdate` entries with reasons
//! 4. **Result Assembly**: Return complete `VersionResolution` with all updates
//!
//! # Examples
//!
//! ## Basic Resolution
//!
//! ```rust,ignore
//! use sublime_pkg_tools::version::resolution::{resolve_versions, VersionResolution};
//! use sublime_pkg_tools::types::{Changeset, VersionBump, PackageInfo};
//! use std::collections::HashMap;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut changeset = Changeset::new("main", VersionBump::Minor, vec!["production".to_string()]);
//! changeset.add_package("@myorg/core");
//! changeset.add_package("@myorg/utils");
//!
//! let packages = HashMap::new(); // Load from filesystem
//!
//! let resolution = resolve_versions(&changeset, &packages).await?;
//!
//! for update in &resolution.updates {
//!     println!("{}: {} -> {}",
//!         update.name,
//!         update.current_version,
//!         update.next_version
//!     );
//! }
//! # Ok(())
//! # }
//! ```

use crate::error::{VersionError, VersionResult};
use crate::types::{
    Changeset, CircularDependency, DependencyUpdate, PackageInfo, UpdateReason, Version,
    VersioningStrategy,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Result of version resolution operation.
///
/// Contains all packages that will be updated with their new versions and the reasons
/// for each update. This type provides a complete preview of version changes before
/// they are applied to package.json files.
///
/// # Fields
///
/// * `updates` - All packages to be updated with their version changes
/// * `circular_dependencies` - Circular dependencies detected during resolution (if any)
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::version::resolution::VersionResolution;
///
/// # fn example(resolution: VersionResolution) {
/// println!("Packages to update: {}", resolution.updates.len());
///
/// for update in &resolution.updates {
///     println!("  {}: {} -> {}",
///         update.name,
///         update.current_version,
///         update.next_version
///     );
/// }
///
/// if !resolution.circular_dependencies.is_empty() {
///     println!("Warning: {} circular dependencies detected",
///         resolution.circular_dependencies.len()
///     );
/// }
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionResolution {
    /// All packages to be updated.
    ///
    /// Each entry contains the package name, current version, next version,
    /// and the reason for the update.
    pub updates: Vec<PackageUpdate>,

    /// Circular dependencies detected (if any).
    ///
    /// Contains cycles in the dependency graph. Empty if no circular
    /// dependencies were found.
    pub circular_dependencies: Vec<CircularDependency>,
}

impl VersionResolution {
    /// Creates a new empty version resolution.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::VersionResolution;
    ///
    /// let resolution = VersionResolution::new();
    /// assert!(resolution.updates.is_empty());
    /// assert!(resolution.circular_dependencies.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { updates: Vec::new(), circular_dependencies: Vec::new() }
    }

    /// Returns whether any packages will be updated.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::VersionResolution;
    ///
    /// let resolution = VersionResolution::new();
    /// assert!(!resolution.has_updates());
    /// ```
    #[must_use]
    pub fn has_updates(&self) -> bool {
        !self.updates.is_empty()
    }

    /// Returns the number of packages to be updated.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::VersionResolution;
    ///
    /// let resolution = VersionResolution::new();
    /// assert_eq!(resolution.update_count(), 0);
    /// ```
    #[must_use]
    pub fn update_count(&self) -> usize {
        self.updates.len()
    }

    /// Returns whether circular dependencies were detected.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::version::VersionResolution;
    ///
    /// let resolution = VersionResolution::new();
    /// assert!(!resolution.has_circular_dependencies());
    /// ```
    #[must_use]
    pub fn has_circular_dependencies(&self) -> bool {
        !self.circular_dependencies.is_empty()
    }

    /// Adds a package update to the resolution.
    ///
    /// This is an internal helper method used during resolution building.
    ///
    /// # Arguments
    ///
    /// * `update` - The package update to add
    pub(crate) fn add_update(&mut self, update: PackageUpdate) {
        self.updates.push(update);
    }

    /// Adds circular dependencies to the resolution.
    ///
    /// This is an internal helper method used during resolution building.
    ///
    /// # Arguments
    ///
    /// * `circular` - The circular dependencies to add
    ///
    /// # Note
    ///
    /// This method is used to add circular dependencies detected during dependency graph
    /// analysis. Circular dependencies are detected by Story 5.3 implementation.
    #[allow(dead_code)]
    pub(crate) fn add_circular_dependencies(&mut self, circular: Vec<CircularDependency>) {
        self.circular_dependencies.extend(circular);
    }
}

impl Default for VersionResolution {
    fn default() -> Self {
        Self::new()
    }
}

/// A single package version update in the resolution.
///
/// Contains all information about a package that will receive a version update,
/// including the current version, next version, and why the update is happening.
///
/// # Fields
///
/// * `name` - Package name (e.g., "@myorg/core")
/// * `path` - Absolute path to package directory
/// * `current_version` - Current version from package.json
/// * `next_version` - Next version after applying the bump
/// * `reason` - Why this package is being updated
/// * `dependency_updates` - Dependency version updates in this package (populated by propagation)
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::version::resolution::{PackageUpdate, UpdateReason};
/// use sublime_pkg_tools::types::Version;
/// use std::path::PathBuf;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let update = PackageUpdate {
///     name: "@myorg/core".to_string(),
///     path: PathBuf::from("/workspace/packages/core"),
///     current_version: Version::parse("1.2.3")?,
///     next_version: Version::parse("1.3.0")?,
///     reason: UpdateReason::DirectChange,
///     dependency_updates: vec![],
/// };
///
/// println!("{}: {} -> {}",
///     update.name,
///     update.current_version,
///     update.next_version
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageUpdate {
    /// Package name (e.g., "@myorg/core").
    pub name: String,

    /// Absolute path to package directory.
    pub path: PathBuf,

    /// Current version (from package.json).
    pub current_version: Version,

    /// Next version after bump.
    pub next_version: Version,

    /// Why this package is being updated.
    pub reason: UpdateReason,

    /// Dependency version updates in this package.
    ///
    /// This field is populated during dependency propagation (Story 5.5).
    /// It contains updates to dependency version specs in package.json.
    pub dependency_updates: Vec<DependencyUpdate>,
}

impl PackageUpdate {
    /// Creates a new package update.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name
    /// * `path` - Absolute path to package directory
    /// * `current_version` - Current version from package.json
    /// * `next_version` - Next version after bump
    /// * `reason` - Why this package is being updated
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::resolution::{PackageUpdate, UpdateReason};
    /// use sublime_pkg_tools::types::Version;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let update = PackageUpdate::new(
    ///     "@myorg/core".to_string(),
    ///     PathBuf::from("/workspace/packages/core"),
    ///     Version::parse("1.2.3")?,
    ///     Version::parse("1.3.0")?,
    ///     UpdateReason::DirectChange,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn new(
        name: String,
        path: PathBuf,
        current_version: Version,
        next_version: Version,
        reason: UpdateReason,
    ) -> Self {
        Self { name, path, current_version, next_version, reason, dependency_updates: Vec::new() }
    }

    /// Returns whether this is a direct change from the changeset.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::resolution::{PackageUpdate, UpdateReason};
    ///
    /// # fn example(update: PackageUpdate) {
    /// if update.is_direct_change() {
    ///     println!("Direct change: {}", update.name);
    /// } else {
    ///     println!("Propagated change: {}", update.name);
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn is_direct_change(&self) -> bool {
        matches!(self.reason, UpdateReason::DirectChange)
    }

    /// Returns whether this is a propagated change from a dependency.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::resolution::PackageUpdate;
    ///
    /// # fn example(update: PackageUpdate) {
    /// if update.is_propagated() {
    ///     println!("Propagated from dependency update");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn is_propagated(&self) -> bool {
        matches!(self.reason, UpdateReason::DependencyPropagation { .. })
    }

    /// Adds a dependency update to this package update.
    ///
    /// This is used during dependency propagation to track changes in dependency
    /// version specifications when a package is updated due to propagation.
    ///
    /// # Arguments
    ///
    /// * `dep_update` - The dependency update to add
    pub(crate) fn add_dependency_update(&mut self, dep_update: DependencyUpdate) {
        self.dependency_updates.push(dep_update);
    }
}

/// Resolves versions for packages in a changeset.
///
/// This is the core resolution function that calculates next versions for all packages
/// in the changeset based on their current versions and the bump type.
///
/// # Arguments
///
/// * `changeset` - The changeset containing packages and bump type
/// * `packages` - Map of package name to package info (with current versions)
/// * `strategy` - Versioning strategy (independent or unified)
///
/// # Returns
///
/// Returns a `VersionResolution` containing all package updates or an error if:
/// - A package in the changeset doesn't exist in the packages map
/// - A package has an invalid current version
/// - Version bump fails for any package
///
/// # Errors
///
/// Returns `VersionError::PackageNotFound` if a package in the changeset is not found.
/// Returns `VersionError::InvalidVersion` if a package version cannot be parsed.
/// Returns `VersionError::BumpError` if version bump fails.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::version::resolution::resolve_versions;
/// use sublime_pkg_tools::types::{Changeset, VersionBump, VersioningStrategy};
/// use std::collections::HashMap;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut changeset = Changeset::new("main", VersionBump::Minor, vec!["production".to_string()]);
/// changeset.add_package("@myorg/core");
///
/// let packages = HashMap::new(); // Load from filesystem
///
/// let resolution = resolve_versions(
///     &changeset,
///     &packages,
///     VersioningStrategy::Independent,
/// ).await?;
///
/// for update in &resolution.updates {
///     println!("{}: {} -> {}",
///         update.name,
///         update.current_version,
///         update.next_version
///     );
/// }
/// # Ok(())
/// # }
/// ```
pub async fn resolve_versions(
    changeset: &Changeset,
    packages: &HashMap<String, PackageInfo>,
    strategy: VersioningStrategy,
) -> VersionResult<VersionResolution> {
    // Validate all packages exist
    validate_packages_exist(changeset, packages)?;

    // Resolve based on strategy
    match strategy {
        VersioningStrategy::Independent => resolve_independent(changeset, packages).await,
        VersioningStrategy::Unified => resolve_unified(changeset, packages).await,
    }
}

/// Validates that all packages in the changeset exist in the packages map.
///
/// # Arguments
///
/// * `changeset` - The changeset to validate
/// * `packages` - Map of available packages
///
/// # Errors
///
/// Returns `VersionError::PackageNotFound` if any package in the changeset is not found.
/// Returns error if version parsing or bumping fails.
fn validate_packages_exist(
    changeset: &Changeset,
    packages: &HashMap<String, PackageInfo>,
) -> VersionResult<()> {
    for package_name in &changeset.packages {
        if !packages.contains_key(package_name) {
            return Err(VersionError::PackageNotFound {
                name: package_name.clone(),
                workspace_root: PathBuf::from("."),
            });
        }
    }
    Ok(())
}

/// Resolves versions using independent strategy.
///
/// Each package is bumped independently based on the changeset bump type.
///
/// # Arguments
///
/// * `changeset` - The changeset containing packages and bump type
/// * `packages` - Map of package info
///
/// # Returns
///
/// Returns a `VersionResolution` with updates for all packages in the changeset.
///
/// # Errors
///
/// Returns error if version parsing or bumping fails.
async fn resolve_independent(
    changeset: &Changeset,
    packages: &HashMap<String, PackageInfo>,
) -> VersionResult<VersionResolution> {
    let mut resolution = VersionResolution::new();

    for package_name in &changeset.packages {
        let package_info =
            packages.get(package_name).ok_or_else(|| VersionError::PackageNotFound {
                name: package_name.clone(),
                workspace_root: PathBuf::from("."),
            })?;

        let current_version = package_info.version();
        let next_version = current_version.bump(changeset.bump)?;

        let update = PackageUpdate::new(
            package_name.clone(),
            package_info.path().to_path_buf(),
            current_version,
            next_version,
            UpdateReason::DirectChange,
        );

        resolution.add_update(update);
    }

    Ok(resolution)
}

/// Resolves versions using unified strategy.
///
/// All packages are bumped to the same version, which is the highest next version
/// among all packages after applying the bump.
///
/// # Arguments
///
/// * `changeset` - The changeset containing packages and bump type
/// * `packages` - Map of package info
///
/// # Returns
///
/// Returns a `VersionResolution` with updates for all packages in the changeset,
/// all using the same next version.
///
/// # Errors
///
/// Returns error if version parsing or bumping fails.
async fn resolve_unified(
    changeset: &Changeset,
    packages: &HashMap<String, PackageInfo>,
) -> VersionResult<VersionResolution> {
    let mut resolution = VersionResolution::new();

    // Find the highest current version across ALL workspace packages
    let mut highest_version: Option<Version> = None;

    for package_info in packages.values() {
        let current_version = package_info.version();

        highest_version = match highest_version {
            Some(ref existing) => {
                if &current_version > existing {
                    Some(current_version)
                } else {
                    highest_version
                }
            }
            None => Some(current_version),
        };
    }

    // Bump the highest version
    let unified_next_version = if let Some(version) = highest_version {
        version.bump(changeset.bump)?
    } else {
        // No packages - shouldn't happen due to validation, but handle gracefully
        return Ok(resolution);
    };

    // Apply unified version to ALL workspace packages (not just those in changeset)
    // This is the core principle of unified strategy: all packages move together
    for (package_name, package_info) in packages {
        let current_version = package_info.version();

        // Determine update reason: packages in changeset are direct changes,
        // others are unified strategy propagation
        let reason = if changeset.packages.contains(package_name) {
            UpdateReason::DirectChange
        } else {
            UpdateReason::UnifiedStrategy
        };

        let update = PackageUpdate::new(
            package_name.clone(),
            package_info.path().to_path_buf(),
            current_version,
            unified_next_version.clone(),
            reason,
        );

        resolution.add_update(update);
    }

    Ok(resolution)
}
