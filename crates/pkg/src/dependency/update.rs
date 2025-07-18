//! # Dependency Update Module
//!
//! This module defines structures for representing dependency updates.
//!
//! When resolving dependency conflicts or performing upgrades, the system
//! needs to track which dependencies need to be updated, in which packages,
//! and to what versions. The `Update` struct captures this information.

/// Represents a required update to a dependency.
///
/// When resolving conflicts or performing upgrades, this structure tracks
/// the details of what needs to be updated and where.
///
/// # Examples
///
/// ```
/// use sublime_package_tools::Update;
///
/// // Define an update that needs to be applied
/// let update = Update {
///     package_name: "my-app".to_string(),
///     dependency_name: "react".to_string(),
///     current_version: "^16.0.0".to_string(),
///     new_version: "^17.0.0".to_string(),
/// };
///
/// // Use the update information
/// println!(
///     "Update {} in {} from {} to {}",
///     update.dependency_name,
///     update.package_name,
///     update.current_version,
///     update.new_version
/// );
/// ```
#[derive(Debug)]
pub struct Update {
    /// Package where the dependency needs to be updated
    ///
    /// The name of the package containing the dependency that needs to be updated.
    pub package_name: String,
    /// Name of the dependency that needs updating
    ///
    /// The name of the dependency to update.
    pub dependency_name: String,
    /// Current version requirement
    ///
    /// The current version requirement string (e.g., "^16.0.0").
    pub current_version: String,
    /// New version requirement to update to
    ///
    /// The new version requirement string (e.g., "^17.0.0").
    pub new_version: String,
}

