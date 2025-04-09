//! Scoped package name parsing and utilities
//!
//! This module provides utilities for parsing and manipulating scoped package names
//! in the format @scope/name, with optional version and path components.

/// Metadata for a scoped package
///
/// Contains the parsed components of a scoped package identifier, including
/// the full string, package name, version, and optional path.
///
/// # Examples
///
/// ```
/// use sublime_package_tools::PackageScopeMetadata;
///
/// let metadata = PackageScopeMetadata {
///     full: "@scope/package@1.0.0".to_string(),
///     name: "@scope/package".to_string(),
///     version: "1.0.0".to_string(),
///     path: None,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct PackageScopeMetadata {
    /// Original full package string (e.g., "@scope/package@1.0.0")
    pub full: String,
    /// Package name with scope (e.g., "@scope/package")
    pub name: String,
    /// Package version (e.g., "1.0.0")
    pub version: String,
    /// Optional subpath within the package
    pub path: Option<String>,
}

/// Parse package scope, name, and version from a string
///
/// This function parses a scoped package identifier, which can be in one of these formats:
/// - `@scope/name`
/// - `@scope/name@version`
/// - `@scope/name@version@path`
/// - `@scope/name:version`
///
/// # Arguments
///
/// * `pkg_name` - The package identifier string to parse
///
/// # Returns
///
/// `Some(PackageScopeMetadata)` if the string is a valid scoped package identifier,
/// or `None` if the string is not a scoped package.
///
/// # Examples
///
/// ```
/// use sublime_package_tools::package_scope_name_version;
///
/// // Parse a simple scoped package
/// let metadata = package_scope_name_version("@scope/package").unwrap();
/// assert_eq!(metadata.name, "@scope/package");
/// assert_eq!(metadata.version, "latest"); // Default version
///
/// // Parse with version
/// let metadata = package_scope_name_version("@scope/package@1.2.3").unwrap();
/// assert_eq!(metadata.name, "@scope/package");
/// assert_eq!(metadata.version, "1.2.3");
///
/// // Parse with version and path
/// let metadata = package_scope_name_version("@scope/package@1.2.3@lib/index.js").unwrap();
/// assert_eq!(metadata.name, "@scope/package");
/// assert_eq!(metadata.version, "1.2.3");
/// assert_eq!(metadata.path, Some("lib/index.js".to_string()));
///
/// // Parse with colon version separator
/// let metadata = package_scope_name_version("@scope/package:1.2.3").unwrap();
/// assert_eq!(metadata.name, "@scope/package");
/// assert_eq!(metadata.version, "1.2.3");
///
/// // Not a scoped package
/// assert!(package_scope_name_version("regular-package").is_none());
/// ```
pub fn package_scope_name_version(pkg_name: &str) -> Option<PackageScopeMetadata> {
    // Must start with @ to be a scoped package
    if !pkg_name.starts_with('@') {
        return None;
    }

    let full = pkg_name.to_string();
    let mut name = String::new();
    let mut version = "latest".to_string();
    let mut path = None;

    // First check for colon format: @scope/name:version
    if pkg_name.contains(':') {
        let parts: Vec<&str> = pkg_name.split(':').collect();
        name = parts[0].to_string();
        if parts.len() > 1 {
            version = parts[1].to_string();
        }
    }
    // Handle @ format: @scope/name@version or @scope/name@version@path
    else {
        let parts: Vec<&str> = pkg_name.split('@').collect();

        // First part is empty because it starts with @
        if parts.len() >= 2 {
            // Format: @scope/name
            name = format!("@{}", parts[1]);

            // Check if there's a version
            if parts.len() >= 3 {
                // Format: @scope/name@version
                version = parts[2].to_string();

                // Check if there's a path
                if parts.len() >= 4 {
                    // Format: @scope/name@version@path
                    path = Some(parts[3].to_string());
                }
            }
        }
    }

    Some(PackageScopeMetadata { full, name, version, path })
}
