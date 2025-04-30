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

use crate::error::FileSystemError;
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
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, FileSystemError>;

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
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), FileSystemError>;

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
    fn read_file_string(&self, path: &Path) -> Result<String, FileSystemError>;

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
    fn write_file_string(&self, path: &Path, contents: &str) -> Result<(), FileSystemError>;

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
    fn create_dir_all(&self, path: &Path) -> Result<(), FileSystemError>;

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
    fn remove(&self, path: &Path) -> Result<(), FileSystemError>;

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
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FileSystemError>;

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
    fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FileSystemError>;
}
