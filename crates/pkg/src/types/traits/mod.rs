//! Common traits for shared behavior patterns across package types.
//!
//! **What**: Defines reusable traits that express common capabilities shared by multiple
//! types in the package management system, such as having a name, version, or dependencies.
//!
//! **How**: Provides trait definitions with clear contracts that types can implement to
//! indicate they support specific operations. This enables generic programming and
//! consistent API patterns across the crate.
//!
//! **Why**: To reduce code duplication, improve type safety, and create a more maintainable
//! codebase where related types share common interfaces and behaviors.
//!
//! # Examples
//!
//! ## Named Trait
//!
//! ```rust,ignore
//! use sublime_pkg_tools::types::traits::Named;
//!
//! fn print_name<T: Named>(item: &T) {
//!     println!("Name: {}", item.name());
//! }
//! ```
//!
//! ## Versionable Trait
//!
//! ```rust,ignore
//! use sublime_pkg_tools::types::traits::Versionable;
//! use sublime_pkg_tools::types::Version;
//!
//! fn compare_versions<T: Versionable>(a: &T, b: &T) -> bool {
//!     a.version() > b.version()
//! }
//! ```

use crate::types::{PackageName, Version};

/// Trait for types that have a name.
///
/// This trait provides a consistent interface for accessing the name of packages,
/// dependencies, and other named entities in the system.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::types::traits::Named;
/// use sublime_pkg_tools::types::PackageInfo;
///
/// fn print_package_name<T: Named>(pkg: &T) {
///     println!("Package: {}", pkg.name());
/// }
/// ```
pub trait Named {
    /// Returns the name of this entity.
    ///
    /// # Returns
    ///
    /// A string slice containing the name.
    fn name(&self) -> &str;
}

/// Trait for types that have a version.
///
/// This trait provides a consistent interface for accessing version information
/// from packages and other versioned entities.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::types::traits::Versionable;
/// use sublime_pkg_tools::types::{Version, PackageInfo};
///
/// fn check_version<T: Versionable>(item: &T, required: &Version) -> bool {
///     item.version() >= required
/// }
/// ```
pub trait Versionable {
    /// Returns the current version of this entity.
    ///
    /// # Returns
    ///
    /// A reference to the Version.
    fn version(&self) -> &Version;
}

/// Trait for types that can be identified by both name and version.
///
/// This trait combines `Named` and `Versionable` to represent entities that
/// have both a name and a version, which is common in package management.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::types::traits::Identifiable;
///
/// fn format_identifier<T: Identifiable>(item: &T) -> String {
///     format!("{}@{}", item.name(), item.version())
/// }
/// ```
pub trait Identifiable: Named + Versionable {
    /// Returns a formatted identifier string combining name and version.
    ///
    /// The default implementation returns "name@version" format.
    ///
    /// # Returns
    ///
    /// A String in the format "name@version".
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::types::traits::Identifiable;
    ///
    /// # fn example<T: Identifiable>(pkg: &T) {
    /// let id = pkg.identifier();
    /// assert_eq!(id, format!("{}@{}", pkg.name(), pkg.version()));
    /// # }
    /// ```
    fn identifier(&self) -> String {
        format!("{}@{}", self.name(), self.version())
    }
}

/// Trait for types that can have dependencies.
///
/// This trait provides a consistent interface for accessing dependency information
/// from packages and other entities that declare dependencies.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::types::traits::HasDependencies;
/// use std::collections::HashMap;
///
/// fn count_dependencies<T: HasDependencies>(item: &T) -> usize {
///     item.dependencies().len()
/// }
/// ```
pub trait HasDependencies {
    /// Returns the dependencies of this entity.
    ///
    /// # Returns
    ///
    /// A reference to a map of dependency names to version specifications.
    fn dependencies(&self) -> &std::collections::HashMap<PackageName, String>;

    /// Returns the development dependencies of this entity.
    ///
    /// # Returns
    ///
    /// A reference to a map of dev dependency names to version specifications.
    fn dev_dependencies(&self) -> &std::collections::HashMap<PackageName, String>;

    /// Returns the peer dependencies of this entity.
    ///
    /// # Returns
    ///
    /// A reference to a map of peer dependency names to version specifications.
    fn peer_dependencies(&self) -> &std::collections::HashMap<PackageName, String>;

    /// Returns all dependencies (regular, dev, and peer) combined.
    ///
    /// The default implementation merges all dependency types.
    ///
    /// # Returns
    ///
    /// A HashMap containing all dependencies.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::types::traits::HasDependencies;
    ///
    /// # fn example<T: HasDependencies>(pkg: &T) {
    /// let all_deps = pkg.all_dependencies();
    /// println!("Total dependencies: {}", all_deps.len());
    /// # }
    /// ```
    fn all_dependencies(&self) -> std::collections::HashMap<PackageName, String> {
        let mut all = std::collections::HashMap::new();
        all.extend(self.dependencies().clone());
        all.extend(self.dev_dependencies().clone());
        all.extend(self.peer_dependencies().clone());
        all
    }
}

#[cfg(test)]
mod tests;
