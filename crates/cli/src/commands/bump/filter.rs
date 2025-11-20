//! Package filtering for selective version bumping.
//!
//! # What
//!
//! Provides the `PackageFilter` type for filtering which packages should receive
//! version bumps based on user selection. This enables selective releases, testing,
//! and emergency hotfixes by allowing users to bump only specific packages in a monorepo.
//!
//! # How
//!
//! The `PackageFilter` filters packages through:
//! - Package name matching (exact string match)
//! - Changeset filtering (intersection of user selection with changeset packages)
//! - Validation against available workspace packages
//! - Optional dependency inclusion (for future enhancement)
//!
//! # Why
//!
//! Package filtering enables:
//! - Selective releases in monorepos (bump only specific packages)
//! - Emergency hotfixes (bump single package without others)
//! - Testing version bumps on subset of packages
//! - Partial releases (frontend vs backend in stages)
//!
//! # Examples
//!
//! ```rust
//! use sublime_cli_tools::commands::bump::filter::PackageFilter;
//! use sublime_pkg_tools::types::Changeset;
//! use sublime_pkg_tools::types::VersionBump;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create filter for specific packages
//! let filter = PackageFilter::new(vec!["@org/core".to_string()], false);
//!
//! // Check if a package should bump
//! assert!(filter.should_bump("@org/core"));
//! assert!(!filter.should_bump("@org/utils"));
//!
//! // Apply filter to changeset
//! let mut changeset = Changeset::new("main", VersionBump::Minor, vec!["prod".to_string()]);
//! changeset.add_package("@org/core");
//! changeset.add_package("@org/utils");
//!
//! let filtered = filter.apply_to_changeset(&changeset);
//! assert_eq!(filtered.packages.len(), 1);
//! assert!(filtered.packages.contains(&"@org/core".to_string()));
//! # Ok(())
//! # }
//! ```

use crate::error::{CliError, Result};
use std::collections::HashSet;
use sublime_pkg_tools::types::Changeset;

/// Package filter for selective version bumping.
///
/// Filters which packages should receive version bumps based on user
/// selection, respecting versioning strategy and dependency relationships.
///
/// # What
///
/// The `PackageFilter` maintains a set of package names that should be bumped
/// and provides methods to check if packages match the filter and to apply
/// the filter to changesets.
///
/// # How
///
/// - Uses `HashSet` for O(1) package name lookups
/// - Filters changesets by intersecting their packages with the filter
/// - Validates filter packages against available workspace packages
/// - Supports optional dependency inclusion (for future enhancement)
///
/// # Why
///
/// Enables selective releases, testing, and emergency hotfixes by allowing
/// users to bump only specific packages in a monorepo.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::bump::filter::PackageFilter;
///
/// let filter = PackageFilter::new(vec!["@org/core".to_string()], false);
/// let should_bump = filter.should_bump("@org/core");  // true
/// let should_bump = filter.should_bump("@org/utils"); // false
/// ```
#[derive(Debug, Clone)]
pub struct PackageFilter {
    /// Set of package names to include in version bumps.
    ///
    /// Uses `HashSet` for O(1) lookup performance when checking if a package
    /// should be bumped.
    packages: HashSet<String>,

    /// Whether to include dependencies of filtered packages.
    ///
    /// When `true`, packages that depend on the filtered packages will also
    /// be bumped. This is a future enhancement and is currently not used.
    ///
    /// Default: `false` (strict filtering)
    include_dependencies: bool,
}

impl PackageFilter {
    /// Creates a new package filter.
    ///
    /// # What
    ///
    /// Constructs a `PackageFilter` from a list of package names and a flag
    /// indicating whether to include dependencies.
    ///
    /// # How
    ///
    /// Converts the package name vector into a `HashSet` for efficient lookup.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package names to include
    /// * `include_dependencies` - Whether to include dependencies
    ///
    /// # Returns
    ///
    /// A new `PackageFilter` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::bump::filter::PackageFilter;
    ///
    /// // Create strict filter (no dependencies)
    /// let filter = PackageFilter::new(
    ///     vec!["@org/core".to_string()],
    ///     false,
    /// );
    ///
    /// // Create filter with dependencies (future enhancement)
    /// let filter_with_deps = PackageFilter::new(
    ///     vec!["@org/core".to_string()],
    ///     true,
    /// );
    /// ```
    #[must_use]
    pub fn new(packages: Vec<String>, include_dependencies: bool) -> Self {
        Self { packages: packages.into_iter().collect(), include_dependencies }
    }

    /// Checks if a package should be bumped based on the filter.
    ///
    /// # What
    ///
    /// Determines if the given package name matches the filter criteria.
    ///
    /// # How
    ///
    /// Performs an O(1) lookup in the internal `HashSet` to check if the
    /// package name is in the filter.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Package name to check
    ///
    /// # Returns
    ///
    /// `true` if the package should be bumped based on filter rules, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::bump::filter::PackageFilter;
    ///
    /// let filter = PackageFilter::new(vec!["@org/core".to_string()], false);
    /// assert!(filter.should_bump("@org/core"));
    /// assert!(!filter.should_bump("@org/utils"));
    /// ```
    #[must_use]
    pub fn should_bump(&self, package_name: &str) -> bool {
        self.packages.contains(package_name)
    }

    /// Applies filter to a changeset.
    ///
    /// # What
    ///
    /// Creates a new changeset with only the filtered packages.
    ///
    /// # How
    ///
    /// Clones the original changeset and filters its packages list to only
    /// include packages that match the filter criteria. All other changeset
    /// properties (branch, bump type, environments, commits) are preserved.
    ///
    /// # Arguments
    ///
    /// * `changeset` - Original changeset
    ///
    /// # Returns
    ///
    /// Filtered changeset with subset of packages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::bump::filter::PackageFilter;
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    ///
    /// # fn example() {
    /// let filter = PackageFilter::new(vec!["@org/core".to_string()], false);
    ///
    /// let mut changeset = Changeset::new("main", VersionBump::Minor, vec!["prod".to_string()]);
    /// changeset.add_package("@org/core");
    /// changeset.add_package("@org/utils");
    ///
    /// let filtered = filter.apply_to_changeset(&changeset);
    /// assert_eq!(filtered.packages.len(), 1);
    /// assert!(filtered.packages.contains(&"@org/core".to_string()));
    /// # }
    /// ```
    #[must_use]
    pub fn apply_to_changeset(&self, changeset: &Changeset) -> Changeset {
        let mut filtered = changeset.clone();

        filtered.packages =
            changeset.packages.iter().filter(|pkg| self.should_bump(pkg)).cloned().collect();

        filtered
    }

    /// Validates that all filter packages exist in workspace.
    ///
    /// # What
    ///
    /// Checks that every package in the filter actually exists in the workspace.
    /// This helps catch typos and configuration errors early.
    ///
    /// # How
    ///
    /// Compares the filter's package set against the available workspace packages.
    /// Returns an error if any filter package is not found in the workspace.
    ///
    /// # Arguments
    ///
    /// * `available_packages` - List of package names in workspace
    ///
    /// # Errors
    ///
    /// Returns `CliError::validation` if any filter package doesn't exist in workspace.
    /// The error message includes:
    /// - The name of the missing package
    /// - A list of all available packages for reference
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::bump::filter::PackageFilter;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let filter = PackageFilter::new(vec!["@org/core".to_string()], false);
    /// let available = vec!["@org/core".to_string(), "@org/utils".to_string()];
    ///
    /// // Validation passes
    /// filter.validate(&available)?;
    ///
    /// // Validation fails for non-existent package
    /// let bad_filter = PackageFilter::new(vec!["@org/nonexistent".to_string()], false);
    /// assert!(bad_filter.validate(&available).is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate(&self, available_packages: &[String]) -> Result<()> {
        let available: HashSet<_> = available_packages.iter().collect();

        for pkg in &self.packages {
            if !available.contains(pkg) {
                return Err(CliError::validation(format!(
                    "Package '{}' not found in workspace. Available packages: {}",
                    pkg,
                    available_packages.join(", ")
                )));
            }
        }

        Ok(())
    }

    /// Returns whether the filter includes dependencies.
    ///
    /// # What
    ///
    /// Returns the value of the `include_dependencies` flag.
    ///
    /// # Returns
    ///
    /// `true` if dependencies should be included, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::bump::filter::PackageFilter;
    ///
    /// let filter = PackageFilter::new(vec!["@org/core".to_string()], true);
    /// assert!(filter.includes_dependencies());
    /// ```
    #[must_use]
    pub fn includes_dependencies(&self) -> bool {
        self.include_dependencies
    }

    /// Returns the number of packages in the filter.
    ///
    /// # What
    ///
    /// Returns the count of packages in the filter set.
    ///
    /// # Returns
    ///
    /// The number of packages in the filter.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::bump::filter::PackageFilter;
    ///
    /// let filter = PackageFilter::new(vec!["@org/core".to_string(), "@org/utils".to_string()], false);
    /// assert_eq!(filter.package_count(), 2);
    /// ```
    #[must_use]
    pub fn package_count(&self) -> usize {
        self.packages.len()
    }

    /// Returns an iterator over the package names in the filter.
    ///
    /// # What
    ///
    /// Provides an iterator over the package names in the filter.
    ///
    /// # Returns
    ///
    /// An iterator over package name strings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::bump::filter::PackageFilter;
    ///
    /// let filter = PackageFilter::new(vec!["@org/core".to_string()], false);
    /// for package in filter.packages() {
    ///     println!("Package: {}", package);
    /// }
    /// ```
    pub fn packages(&self) -> impl Iterator<Item = &String> {
        self.packages.iter()
    }
}
