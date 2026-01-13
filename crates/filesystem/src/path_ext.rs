//! # Path Extension Module
//!
//! Provides utility extensions for working with filesystem paths.
//!
//! ## What
//!
//! This module defines the [`PathExt`] trait, which adds convenience methods to
//! [`std::path::Path`] for common path manipulation tasks. These are pure functions
//! that operate on path data without performing any I/O operations.
//!
//! ## How
//!
//! The trait is implemented for `Path` and provides methods such as:
//! - Path normalization (resolving `.` and `..` components)
//! - Cross-platform path handling
//!
//! All methods are synchronous because they operate only on in-memory path data,
//! not on the filesystem. This makes them safe to use in any context without
//! blocking or requiring async runtimes.
//!
//! ## Why
//!
//! Path extension utilities are valuable for:
//! - **Normalization**: Ensure consistent path representation across platforms
//! - **Safety**: Validate paths before filesystem operations
//! - **Convenience**: Common operations available as methods, not functions
//! - **Portability**: Abstract over platform-specific path separators
//! - **No I/O**: Pure functions that can be called without filesystem access
//!
//! ## Example
//!
//! ```rust
//! use workspace_fs::PathExt;
//! use std::path::Path;
//!
//! let path = Path::new("/foo/bar/../baz/./qux");
//!
//! // Normalize path without touching filesystem
//! let normalized = path.normalize();
//! assert_eq!(normalized, Path::new("/foo/baz/qux"));
//!
//! // Works with relative paths too
//! let relative = Path::new("./a/./b/../c");
//! assert_eq!(relative.normalize(), Path::new("a/c"));
//!
//! // Preserves parent directory references when necessary
//! let up_path = Path::new("../../config");
//! assert_eq!(up_path.normalize(), Path::new("../../config"));
//! ```

use std::path::{Component, Path, PathBuf};

/// Extension trait for [`Path`] with utility methods.
///
/// This trait provides additional methods for path manipulation that operate
/// purely on the path string without performing any I/O operations. All methods
/// are synchronous and can be safely called from any context.
///
/// The trait is automatically implemented for all types that can be dereferenced
/// to a [`Path`], including [`PathBuf`], `&Path`, and `String` paths.
///
/// # Thread Safety
///
/// All methods are thread-safe as they operate on immutable path data and
/// produce new [`PathBuf`] instances rather than modifying in place.
///
/// # Examples
///
/// ```rust
/// use workspace_fs::PathExt;
/// use std::path::Path;
///
/// // Normalize an absolute path with parent references
/// let path = Path::new("/home/user/../admin/./config");
/// assert_eq!(path.normalize(), Path::new("/home/admin/config"));
///
/// // Normalize a relative path
/// let rel = Path::new("src/../lib/./utils");
/// assert_eq!(rel.normalize(), Path::new("lib/utils"));
/// ```
pub trait PathExt {
    /// Normalizes the path by resolving `.` (current directory) and `..` (parent directory) components.
    ///
    /// This method does **NOT** perform any I/O operations. It processes path
    /// components logically without accessing the filesystem. This means:
    /// - Symlinks are NOT followed (they would require I/O)
    /// - Path existence is NOT verified
    /// - Permissions are NOT checked
    ///
    /// # Algorithm
    ///
    /// The normalization follows these rules:
    ///
    /// 1. **Current directory (`.`)**: Removed entirely
    /// 2. **Parent directory (`..`)** with a preceding normal component: Both are removed
    /// 3. **Parent directory (`..`)** at the start of a relative path: Preserved
    /// 4. **Parent directory (`..`)** after root: Ignored (can't go above root)
    /// 5. **Root directory (`/`)**: Clears all preceding components
    /// 6. **Prefix (Windows drive like `C:`)**: Preserved at the start
    /// 7. **Empty path**: Returns `.` (current directory)
    ///
    /// # Platform Considerations
    ///
    /// - On **Windows**, drive prefixes (e.g., `C:`) are preserved
    /// - On **Unix**, paths starting with `/` are absolute
    /// - Multiple consecutive separators are normalized (handled by [`Path::components`])
    ///
    /// # Examples
    ///
    /// ## Absolute paths
    ///
    /// ```rust
    /// use workspace_fs::PathExt;
    /// use std::path::Path;
    ///
    /// // Parent directory resolution
    /// assert_eq!(Path::new("/a/b/../c").normalize(), Path::new("/a/c"));
    ///
    /// // Current directory removal
    /// assert_eq!(Path::new("/a/./b/./c").normalize(), Path::new("/a/b/c"));
    ///
    /// // Combined resolution
    /// assert_eq!(Path::new("/a/b/c/../../d").normalize(), Path::new("/a/d"));
    ///
    /// // Parent at root is ignored
    /// assert_eq!(Path::new("/../a").normalize(), Path::new("/a"));
    /// ```
    ///
    /// ## Relative paths
    ///
    /// ```rust
    /// use workspace_fs::PathExt;
    /// use std::path::Path;
    ///
    /// // Current directory at start
    /// assert_eq!(Path::new("./a/./b").normalize(), Path::new("a/b"));
    ///
    /// // Parent directory resolution
    /// assert_eq!(Path::new("a/b/../../c").normalize(), Path::new("c"));
    ///
    /// // Preserved parent directories
    /// assert_eq!(Path::new("../a").normalize(), Path::new("../a"));
    /// assert_eq!(Path::new("../../a/b").normalize(), Path::new("../../a/b"));
    /// assert_eq!(Path::new("a/../../b").normalize(), Path::new("../b"));
    /// ```
    ///
    /// ## Edge cases
    ///
    /// ```rust
    /// use workspace_fs::PathExt;
    /// use std::path::Path;
    ///
    /// // Empty path becomes current directory
    /// assert_eq!(Path::new("").normalize(), Path::new("."));
    ///
    /// // Just dots
    /// assert_eq!(Path::new(".").normalize(), Path::new("."));
    /// assert_eq!(Path::new("..").normalize(), Path::new(".."));
    /// assert_eq!(Path::new("../..").normalize(), Path::new("../.."));
    ///
    /// // Root stays root
    /// assert_eq!(Path::new("/").normalize(), Path::new("/"));
    /// ```
    fn normalize(&self) -> PathBuf;
}

impl PathExt for Path {
    fn normalize(&self) -> PathBuf {
        let mut components: Vec<Component<'_>> = Vec::new();

        for component in self.components() {
            match component {
                // Prefix is Windows-specific (e.g., C:, \\?\, \\.\)
                // Always preserve it at the beginning
                Component::Prefix(prefix) => {
                    components.push(Component::Prefix(prefix));
                }

                // Root directory (/) clears everything except prefix and starts fresh
                Component::RootDir => {
                    // Keep only the prefix if present, then add root
                    let prefix =
                        components.first().filter(|c| matches!(c, Component::Prefix(_))).copied();
                    components.clear();
                    if let Some(p) = prefix {
                        components.push(p);
                    }
                    components.push(Component::RootDir);
                }

                // Current directory (.) is always removed - it's a no-op
                Component::CurDir => {
                    // Skip entirely - current directory adds no information
                }

                // Parent directory (..) requires careful handling
                Component::ParentDir => {
                    match components.last() {
                        // If the last component is a normal path segment, we can resolve
                        // the parent by removing that segment
                        Some(Component::Normal(_)) => {
                            components.pop();
                        }

                        // If we have no components, the last is already a ParentDir, or
                        // CurDir (which shouldn't happen since we skip it, but for safety),
                        // we're building a relative path that goes up - preserve it
                        None | Some(Component::ParentDir | Component::CurDir) => {
                            components.push(component);
                        }

                        // If we're at root (RootDir) or a Windows prefix, we can't go
                        // higher - silently ignore the parent reference
                        Some(Component::RootDir | Component::Prefix(_)) => {
                            // Cannot go above root - ignore this component
                        }
                    }
                }

                // Normal components (regular file/directory names) are always added
                Component::Normal(name) => {
                    components.push(Component::Normal(name));
                }
            }
        }

        // Handle the empty result case - return current directory marker
        if components.is_empty() { PathBuf::from(".") } else { components.iter().collect() }
    }
}

// Also implement for PathBuf for convenience
impl PathExt for PathBuf {
    /// Normalizes the path by delegating to the [`Path`] implementation.
    ///
    /// See [`PathExt::normalize`] for full documentation.
    #[inline]
    fn normalize(&self) -> PathBuf {
        self.as_path().normalize()
    }
}
