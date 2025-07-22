//! # Async Filesystem Trait
//!
//! ## What
//! This module defines the core `AsyncFileSystem` trait that provides
//! a comprehensive set of async operations for file and directory manipulation.
//!
//! ## How
//! The trait uses async/await patterns with proper error handling and
//! comprehensive documentation for all operations.
//!
//! ## Why
//! A unified async-only approach eliminates confusion between sync and async
//! operations and provides better performance for large repositories.

use crate::error::Result;
use async_trait::async_trait;
use std::path::Path;

/// Async trait defining filesystem operations.
///
/// This trait provides a comprehensive set of async operations for interacting with
/// the filesystem, including reading and writing files, creating and removing
/// directories, and traversing directory structures.
///
/// All operations are non-blocking and can be executed concurrently for maximum
/// performance when dealing with large repositories.
///
/// # Examples
///
/// ```rust
/// use sublime_standard_tools::filesystem::{AsyncFileSystem, AsyncFileSystemManager};
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = AsyncFileSystemManager::new();
/// let contents = fs.read_file_string(Path::new("Cargo.toml")).await?;
/// println!("Cargo.toml contents: {}", contents);
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait AsyncFileSystem: Send + Sync {
    /// Asynchronously reads a file and returns its contents as a byte vector.
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
    /// ```rust
    /// use sublime_standard_tools::filesystem::{AsyncFileSystem, AsyncFileSystemManager};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystemManager::new();
    /// let contents = fs.read_file(Path::new("Cargo.toml")).await?;
    /// println!("Read {} bytes", contents.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or does not exist.
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>>;

    /// Asynchronously writes data to a file, creating the file and any parent directories if they don't exist.
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
    /// ```rust
    /// use sublime_standard_tools::filesystem::{AsyncFileSystem, AsyncFileSystemManager};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystemManager::new();
    /// let data = b"Hello, world!";
    /// fs.write_file(Path::new("example.txt"), data).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    async fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()>;

    /// Asynchronously reads a file and returns its contents as a UTF-8 encoded string.
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
    /// ```rust
    /// use sublime_standard_tools::filesystem::{AsyncFileSystem, AsyncFileSystemManager};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystemManager::new();
    /// let contents = fs.read_file_string(Path::new("Cargo.toml")).await?;
    /// println!("First line: {}", contents.lines().next().unwrap_or_default());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, does not exist, or contains invalid UTF-8.
    async fn read_file_string(&self, path: &Path) -> Result<String>;

    /// Asynchronously writes a string to a file, creating the file and any parent directories if they don't exist.
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
    /// ```rust
    /// use sublime_standard_tools::filesystem::{AsyncFileSystem, AsyncFileSystemManager};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystemManager::new();
    /// fs.write_file_string(Path::new("example.txt"), "Hello, world!").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    async fn write_file_string(&self, path: &Path, contents: &str) -> Result<()>;

    /// Asynchronously creates a directory and all of its parent directories if they don't exist.
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
    /// ```rust
    /// use sublime_standard_tools::filesystem::{AsyncFileSystem, AsyncFileSystemManager};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystemManager::new();
    /// fs.create_dir_all(Path::new("nested/directory/structure")).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created.
    async fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Asynchronously removes a file or directory (recursively if it's a directory).
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
    /// ```rust
    /// use sublime_standard_tools::filesystem::{AsyncFileSystem, AsyncFileSystemManager};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystemManager::new();
    /// // Create a file, then remove it
    /// fs.write_file_string(Path::new("temp.txt"), "temporary content").await?;
    /// fs.remove(Path::new("temp.txt")).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be removed.
    async fn remove(&self, path: &Path) -> Result<()>;

    /// Asynchronously checks if a path exists.
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
    /// ```rust
    /// use sublime_standard_tools::filesystem::{AsyncFileSystem, AsyncFileSystemManager};
    /// use std::path::Path;
    ///
    /// # async fn example() {
    /// let fs = AsyncFileSystemManager::new();
    /// if fs.exists(Path::new("Cargo.toml")).await {
    ///     println!("Cargo.toml exists");
    /// } else {
    ///     println!("Cargo.toml does not exist");
    /// }
    /// # }
    /// ```
    async fn exists(&self, path: &Path) -> bool;

    /// Asynchronously lists the contents of a directory.
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
    /// ```rust
    /// use sublime_standard_tools::filesystem::{AsyncFileSystem, AsyncFileSystemManager};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystemManager::new();
    /// let entries = fs.read_dir(Path::new("src")).await?;
    /// for entry in entries {
    ///     println!("Found: {}", entry.display());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read or does not exist.
    async fn read_dir(&self, path: &Path) -> Result<Vec<std::path::PathBuf>>;

    /// Asynchronously walks a directory recursively, listing all files and directories.
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
    /// ```rust
    /// use sublime_standard_tools::filesystem::{AsyncFileSystem, AsyncFileSystemManager};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystemManager::new();
    /// let all_entries = fs.walk_dir(Path::new("src")).await?;
    /// println!("Found {} total entries", all_entries.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read or does not exist.
    async fn walk_dir(&self, path: &Path) -> Result<Vec<std::path::PathBuf>>;

    /// Asynchronously gets metadata for a path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to get metadata for
    ///
    /// # Returns
    ///
    /// * `Ok(std::fs::Metadata)` - The metadata for the path
    /// * `Err(FileSystemError)` - If the metadata cannot be read
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::filesystem::{AsyncFileSystem, AsyncFileSystemManager};
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = AsyncFileSystemManager::new();
    /// let metadata = fs.metadata(Path::new("Cargo.toml")).await?;
    /// println!("File size: {} bytes", metadata.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the metadata cannot be read.
    async fn metadata(&self, path: &Path) -> Result<std::fs::Metadata>;
}
