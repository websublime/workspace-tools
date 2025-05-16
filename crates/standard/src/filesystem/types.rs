//! # Filesystem Types
//!
//! ## What
//! This file defines the core types for filesystem operations, including the
//! `FileSystem` trait and the concrete `FileSystemManager` implementation.
//!
//! ## How
//! The `FileSystem` trait defines a comprehensive set of operations for file
//! and directory manipulation, while `FileSystemManager` provides a real
//! implementation using standard library functions.
//!
//! ## Why
//! Separating the trait from the implementation allows for better testability
//! through mocking and provides a clear contract for filesystem operations.

use crate::error::Result;
use std::path::{Path, PathBuf};

/// A manager for filesystem operations.
///
/// This struct provides a concrete implementation of the `FileSystem` trait
/// using the standard library and additional crates for enhanced functionality.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};
///
/// let fs = FileSystemManager::new();
/// if fs.exists(Path::new("Cargo.toml")) {
///     println!("Cargo.toml exists");
/// }
/// ```
#[derive(Debug, Default)]
pub struct FileSystemManager;

/// Trait defining filesystem operations.
///
/// This trait provides a comprehensive set of operations for interacting with
/// the filesystem, including reading and writing files, creating and removing
/// directories, and traversing directory structures.
///
/// Implementations must ensure thread safety through the `Send` and `Sync` bounds.
pub trait FileSystem: Send + Sync {
    /// Reads a file and returns its contents as a byte vector.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to read
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - The file contents as bytes
    /// * `Err(FileSystemError)` - If the file cannot be read or does not exist
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let contents = fs.read_file(Path::new("Cargo.toml"))?;
    /// println!("Read {} bytes", contents.len());
    /// # Ok(())
    /// # }
    /// ```
    fn read_file(&self, path: &Path) -> Result<Vec<u8>>;

    /// Writes data to a file, creating the file and any parent directories if they don't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to write
    /// * `contents` - The data to write to the file
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the write was successful
    /// * `Err(FileSystemError)` - If the file cannot be written
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let data = b"Hello, world!";
    /// fs.write_file(Path::new("example.txt"), data)?;
    /// # Ok(())
    /// # }
    /// ```
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()>;

    /// Reads a file and returns its contents as a UTF-8 encoded string.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to read
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The file contents as a string
    /// * `Err(FileSystemError)` - If the file cannot be read, does not exist, or contains invalid UTF-8
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let contents = fs.read_file_string(Path::new("Cargo.toml"))?;
    /// println!("First line: {}", contents.lines().next().unwrap_or_default());
    /// # Ok(())
    /// # }
    /// ```
    fn read_file_string(&self, path: &Path) -> Result<String>;

    /// Writes a string to a file, creating the file and any parent directories if they don't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to write
    /// * `contents` - The string to write to the file
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the write was successful
    /// * `Err(FileSystemError)` - If the file cannot be written
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// fs.write_file_string(Path::new("example.txt"), "Hello, world!")?;
    /// # Ok(())
    /// # }
    /// ```
    fn write_file_string(&self, path: &Path, contents: &str) -> Result<()>;

    /// Creates a directory and all of its parent directories if they don't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the directory to create
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the directory was created or already exists
    /// * `Err(FileSystemError)` - If the directory cannot be created
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// fs.create_dir_all(Path::new("nested/directory/structure"))?;
    /// # Ok(())
    /// # }
    /// ```
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Removes a file or directory (recursively if it's a directory).
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file or directory to remove
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the removal was successful
    /// * `Err(FileSystemError)` - If the path cannot be removed
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// // Create a file, then remove it
    /// fs.write_file_string(Path::new("temp.txt"), "temporary content")?;
    /// fs.remove(Path::new("temp.txt"))?;
    /// # Ok(())
    /// # }
    /// ```
    fn remove(&self, path: &Path) -> Result<()>;

    /// Checks if a path exists.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    ///
    /// # Returns
    ///
    /// * `true` - If the path exists
    /// * `false` - If the path does not exist
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};
    ///
    /// let fs = FileSystemManager::new();
    /// if fs.exists(Path::new("Cargo.toml")) {
    ///     println!("Cargo.toml exists");
    /// } else {
    ///     println!("Cargo.toml does not exist");
    /// }
    /// ```
    fn exists(&self, path: &Path) -> bool;

    /// Lists the contents of a directory.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the directory to read
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<PathBuf>)` - The paths of entries in the directory
    /// * `Err(FileSystemError)` - If the directory cannot be read or does not exist
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let entries = fs.read_dir(Path::new("src"))?;
    /// for entry in entries {
    ///     println!("Found: {}", entry.display());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;

    /// Walks a directory recursively, listing all files and directories.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the directory to walk
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<PathBuf>)` - The paths of all entries found recursively
    /// * `Err(FileSystemError)` - If the directory cannot be read or does not exist
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let all_entries = fs.walk_dir(Path::new("src"))?;
    /// println!("Found {} total entries", all_entries.len());
    /// # Ok(())
    /// # }
    /// ```
    fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
}

/// Represents common directory and file types in Node.js projects.
///
/// This enum provides a type-safe way to reference conventional Node.js
/// project paths like "node_modules", "src", etc.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use sublime_standard_tools::filesystem::NodePathKind;
///
/// let project_dir = Path::new("/project");
/// let node_modules = project_dir.join(NodePathKind::NodeModules.default_path());
/// assert_eq!(node_modules, Path::new("/project/node_modules"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodePathKind {
    /// Node modules directory
    NodeModules,
    /// Package configuration
    PackageJson,
    /// Source directory
    Src,
    /// Distribution directory
    Dist,
    /// Test directory
    Test,
}

/// Utility struct for file system path operations.
///
/// This struct provides static methods for common path operations
/// related to Node.js projects, such as finding project roots and
/// manipulating paths.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use sublime_standard_tools::filesystem::PathUtils;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Find the current directory
/// let current = PathUtils::current_dir()?;
///
/// // Find a project root (if in a Node.js project)
/// if let Some(root) = PathUtils::find_project_root(&current) {
///     println!("Project root found at: {}", root.display());
/// }
/// # Ok(())
/// # }
/// ```
pub struct PathUtils;

/// Extension trait for Path to provide Node.js-specific path utilities.
///
/// This trait adds methods to the standard Path type for working with Node.js
/// project directories, normalizing paths, and handling Node.js path conventions.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use sublime_standard_tools::filesystem::{NodePathKind, PathExt};
///
/// let path = Path::new("/some/path");
/// let node_modules = path.node_path(NodePathKind::NodeModules);
/// assert_eq!(node_modules, Path::new("/some/path/node_modules"));
///
/// let normalized = Path::new("/a/b/../c").normalize();
/// assert_eq!(normalized, Path::new("/a/c"));
/// ```
pub trait PathExt {
    /// Normalizes a path by resolving `.` and `..` components.
    ///
    /// This method processes the path components to:
    /// - Remove `.` (current directory) components
    /// - Resolve `..` (parent directory) components by removing the previous component
    /// - Preserve root and prefix components
    /// - Keep normal path components unchanged
    ///
    /// # Returns
    ///
    /// A normalized `PathBuf` with all `.` and `..` components resolved.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::PathExt;
    ///
    /// let path = Path::new("/home/user/../projects/./app");
    /// let normalized = path.normalize();
    /// assert_eq!(normalized, Path::new("/home/projects/app"));
    /// ```
    fn normalize(&self) -> PathBuf;

    /// Checks if this path is inside a Node.js project by looking for
    /// a package.json file in the current directory or any parent directory.
    ///
    /// # Returns
    ///
    /// `true` if the path is inside a Node.js project, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::PathExt;
    ///
    /// // Assuming the current directory is in a Node.js project
    /// let path = Path::new(".");
    /// if path.is_in_project() {
    ///     println!("This path is in a Node.js project");
    /// }
    /// ```
    fn is_in_project(&self) -> bool;

    /// Gets the path relative to the nearest Node.js project root.
    ///
    /// This method searches upward from the current path to find a directory
    /// containing a package.json file, then returns the path relative to that
    /// project root.
    ///
    /// # Returns
    ///
    /// * `Some(PathBuf)` - The path relative to the project root if a project root was found
    /// * `None` - If no project root was found
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::PathExt;
    ///
    /// // Assuming the current directory is in a Node.js project
    /// let path = Path::new("src/components/Button.js");
    /// if let Some(relative) = path.relative_to_project() {
    ///     println!("Path relative to project: {}", relative.display());
    /// }
    /// ```
    fn relative_to_project(&self) -> Option<PathBuf>;

    /// Joins a Node.js path kind to this path.
    ///
    /// This is a convenience method for joining paths like "node_modules",
    /// "package.json", etc. to the current path.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of Node.js path to join
    ///
    /// # Returns
    ///
    /// A new `PathBuf` with the Node.js path kind joined to this path.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::{NodePathKind, PathExt};
    ///
    /// let project_path = Path::new("/projects/my-app");
    /// let node_modules = project_path.node_path(NodePathKind::NodeModules);
    /// assert_eq!(node_modules, Path::new("/projects/my-app/node_modules"));
    /// ```
    fn node_path(&self, kind: NodePathKind) -> PathBuf;

    /// Canonicalizes a path, resolving symlinks if needed.
    ///
    /// If the path is a symlink, it resolves to the target path.
    /// Otherwise, it returns the path as-is.
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - The canonicalized path
    /// * `Err(FileSystemError)` - If the path cannot be canonicalized
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::filesystem::PathExt;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = Path::new("Cargo.toml");
    /// let canonical = path.canonicalize()?;
    /// println!("Canonical path: {}", canonical.display());
    /// # Ok(())
    /// # }
    /// ```
    fn canonicalize(&self) -> Result<PathBuf>;
}
