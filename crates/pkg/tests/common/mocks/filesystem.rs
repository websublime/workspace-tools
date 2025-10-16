//! # Mock Filesystem Implementation
//!
//! This module provides an in-memory mock filesystem implementation for testing.
//!
//! ## What
//!
//! `MockFileSystem` is an in-memory implementation of the `AsyncFileSystem` trait
//! that allows tests to run without touching the real filesystem.
//!
//! ## How
//!
//! Files are stored in a `HashMap` with their paths as keys and contents as values.
//! All operations are synchronous but wrapped in async functions to match the trait.
//!
//! ## Why
//!
//! Mock filesystem provides:
//! - Fast test execution without I/O overhead
//! - Predictable test behavior
//! - Easy setup and teardown
//! - Ability to test error conditions

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use sublime_standard_tools::error::{FileSystemError, Result};
use sublime_standard_tools::filesystem::AsyncFileSystem;

/// In-memory mock filesystem for testing
///
/// This struct maintains an in-memory representation of a filesystem,
/// allowing tests to run without touching the real filesystem.
///
/// # Examples
///
/// ```rust,ignore
/// use crate::common::mocks::MockFileSystem;
///
/// #[tokio::test]
/// async fn test_read_write() {
///     let fs = MockFileSystem::new();
///     let path = Path::new("/test/file.txt");
///
///     fs.write_string(path, "content").await.unwrap();
///     let content = fs.read_to_string(path).await.unwrap();
///
///     assert_eq!(content, "content");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MockFileSystem {
    /// Internal storage for files and directories
    files: Arc<Mutex<HashMap<PathBuf, FileEntry>>>,
}

/// Represents an entry in the mock filesystem
#[derive(Debug, Clone)]
enum FileEntry {
    /// A file with its contents
    File(Vec<u8>),
    /// A directory
    Directory,
}

impl MockFileSystem {
    /// Creates a new empty mock filesystem
    ///
    /// # Returns
    ///
    /// A new `MockFileSystem` instance
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let fs = MockFileSystem::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { files: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// Creates a new mock filesystem with pre-populated files
    ///
    /// # Arguments
    ///
    /// * `files` - A map of file paths to their contents
    ///
    /// # Returns
    ///
    /// A new `MockFileSystem` instance with the specified files
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut files = HashMap::new();
    /// files.insert(PathBuf::from("/test.txt"), "content".to_string());
    /// let fs = MockFileSystem::with_files(files);
    /// ```
    #[must_use]
    pub fn with_files(files: HashMap<PathBuf, String>) -> Self {
        let mut entries = HashMap::new();
        for (path, content) in files {
            // Ensure all parent directories exist
            let mut current = PathBuf::new();
            for component in path.parent().unwrap_or(Path::new("")).components() {
                current.push(component);
                if !entries.contains_key(&current) {
                    entries.insert(current.clone(), FileEntry::Directory);
                }
            }
            entries.insert(path, FileEntry::File(content.into_bytes()));
        }

        Self { files: Arc::new(Mutex::new(entries)) }
    }

    /// Adds a file to the mock filesystem
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the file should be created
    /// * `contents` - The contents of the file
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let fs = MockFileSystem::new();
    /// fs.add_file("/test.txt", "content");
    /// ```
    pub fn add_file(&self, path: impl AsRef<Path>, contents: impl Into<Vec<u8>>) {
        let path = path.as_ref().to_path_buf();
        let contents = contents.into();

        // Ensure parent directories exist
        if let Some(parent) = path.parent() {
            self.ensure_dir_exists(parent);
        }

        let mut files = self.files.lock().unwrap();
        files.insert(path, FileEntry::File(contents));
    }

    /// Adds a directory to the mock filesystem
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the directory should be created
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let fs = MockFileSystem::new();
    /// fs.add_dir("/test");
    /// ```
    pub fn add_dir(&self, path: impl AsRef<Path>) {
        self.ensure_dir_exists(path.as_ref());
    }

    /// Ensures a directory and all its parents exist
    fn ensure_dir_exists(&self, path: &Path) {
        let mut files = self.files.lock().unwrap();
        let mut current = PathBuf::new();

        for component in path.components() {
            current.push(component);
            files.entry(current.clone()).or_insert(FileEntry::Directory);
        }
    }

    /// Clears all files from the mock filesystem
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let fs = MockFileSystem::new();
    /// fs.add_file("/test.txt", "content");
    /// fs.clear();
    /// // Filesystem is now empty
    /// ```
    pub fn clear(&self) {
        let mut files = self.files.lock().unwrap();
        files.clear();
    }

    /// Gets the number of files in the filesystem
    ///
    /// # Returns
    ///
    /// The total number of entries (files and directories)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let fs = MockFileSystem::new();
    /// assert_eq!(fs.len(), 0);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        let files = self.files.lock().unwrap();
        files.len()
    }

    /// Checks if the filesystem is empty
    /// Checks if a path is a file
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    ///
    /// # Returns
    ///
    /// `true` if the path is a file, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let fs = MockFileSystem::new();
    /// let path = Path::new("/test.txt");
    /// fs.add_file(path, "content");
    /// assert!(fs.is_file(path));
    /// ```
    #[must_use]
    pub fn is_file(&self, path: &Path) -> bool {
        let files = self.files.lock().unwrap();
        matches!(files.get(path), Some(FileEntry::File(_)))
    }

    /// Checks if a path is a directory
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    ///
    /// # Returns
    ///
    /// `true` if the path is a directory, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let fs = MockFileSystem::new();
    /// let path = Path::new("/test");
    /// fs.add_dir(path);
    /// assert!(fs.is_dir(path));
    /// ```
    #[must_use]
    pub fn is_dir(&self, path: &Path) -> bool {
        let files = self.files.lock().unwrap();
        matches!(files.get(path), Some(FileEntry::Directory))
    }

    /// Checks if the filesystem is empty
    ///
    /// # Returns
    ///
    /// `true` if there are no entries, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let fs = MockFileSystem::new();
    /// assert!(fs.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        let files = self.files.lock().unwrap();
        files.is_empty()
    }

    /// Lists all files in the filesystem
    ///
    /// # Returns
    ///
    /// A vector of all file paths (not including directories)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let fs = MockFileSystem::new();
    /// fs.add_file("/test.txt", "content");
    /// let files = fs.list_files();
    /// assert_eq!(files.len(), 1);
    /// ```
    #[must_use]
    pub fn list_files(&self) -> Vec<PathBuf> {
        let files = self.files.lock().unwrap();
        files
            .iter()
            .filter_map(
                |(path, entry)| {
                    if matches!(entry, FileEntry::File(_)) {
                        Some(path.clone())
                    } else {
                        None
                    }
                },
            )
            .collect()
    }

    /// Writes a string to a file (convenience wrapper)
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the file should be written
    /// * `contents` - The string contents to write
    ///
    /// # Returns
    ///
    /// A result indicating success or failure
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let fs = MockFileSystem::new();
    /// fs.write_file_string(Path::new("/test.txt"), "content").await.unwrap();
    /// ```
    pub async fn write_file_string(&self, path: &Path, contents: &str) -> Result<()> {
        AsyncFileSystem::write_file_string(self, path, contents).await
    }

    /// Reads a file as a string (convenience wrapper)
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the file to read
    ///
    /// # Returns
    ///
    /// The file contents as a string
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let fs = MockFileSystem::new();
    /// let content = fs.read_file_string(Path::new("/test.txt")).await.unwrap();
    /// ```
    pub async fn read_file_string(&self, path: &Path) -> Result<String> {
        AsyncFileSystem::read_file_string(self, path).await
    }

    /// Checks if a path exists (convenience wrapper)
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    ///
    /// # Returns
    ///
    /// `true` if the path exists, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let fs = MockFileSystem::new();
    /// if fs.exists(Path::new("/test.txt")).await {
    ///     println!("File exists");
    /// }
    /// ```
    pub async fn exists(&self, path: &Path) -> bool {
        AsyncFileSystem::exists(self, path).await
    }

    /// Removes a file from the mock filesystem (convenience method)
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the file to remove
    ///
    /// # Returns
    ///
    /// A result indicating success or failure
    ///
    /// # Errors
    ///
    /// Returns an error if the path is not a file or does not exist
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let fs = MockFileSystem::new();
    /// fs.add_file("/test.txt", "content");
    /// fs.remove_file(Path::new("/test.txt")).await.unwrap();
    /// ```
    pub async fn remove_file(&self, path: &Path) -> Result<()> {
        let mut files = self.files.lock().unwrap();
        match files.get(path) {
            Some(FileEntry::File(_)) => {
                files.remove(path);
                Ok(())
            }
            Some(FileEntry::Directory) => {
                Err(FileSystemError::NotAFile { path: path.to_path_buf() }.into())
            }
            None => Err(FileSystemError::NotFound { path: path.to_path_buf() }.into()),
        }
    }
}

impl Default for MockFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AsyncFileSystem for MockFileSystem {
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        let files = self.files.lock().unwrap();
        match files.get(path) {
            Some(FileEntry::File(contents)) => Ok(contents.clone()),
            Some(FileEntry::Directory) => {
                Err(FileSystemError::NotAFile { path: path.to_path_buf() }.into())
            }
            None => Err(FileSystemError::NotFound { path: path.to_path_buf() }.into()),
        }
    }

    async fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            self.ensure_dir_exists(parent);
        }

        let mut files = self.files.lock().unwrap();
        files.insert(path.to_path_buf(), FileEntry::File(contents.to_vec()));
        Ok(())
    }

    async fn read_file_string(&self, path: &Path) -> Result<String> {
        let bytes = self.read_file(path).await?;
        String::from_utf8(bytes).map_err(|e| {
            FileSystemError::Utf8Decode {
                path: path.to_path_buf(),
                message: format!("Invalid UTF-8: {}", e),
            }
            .into()
        })
    }

    async fn write_file_string(&self, path: &Path, contents: &str) -> Result<()> {
        self.write_file(path, contents.as_bytes()).await
    }

    async fn create_dir_all(&self, path: &Path) -> Result<()> {
        self.ensure_dir_exists(path);
        Ok(())
    }

    async fn remove(&self, path: &Path) -> Result<()> {
        let mut files = self.files.lock().unwrap();

        // Check if path exists
        if !files.contains_key(path) {
            return Err(FileSystemError::NotFound { path: path.to_path_buf() }.into());
        }

        // Remove the path and all its children
        files.retain(|p, _| {
            // Keep only paths that are not the target and not descendants of it
            p != path && !p.starts_with(path)
        });

        Ok(())
    }

    async fn exists(&self, path: &Path) -> bool {
        let files = self.files.lock().unwrap();
        files.contains_key(path)
    }

    async fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let files = self.files.lock().unwrap();

        // Check if path exists and is a directory
        match files.get(path) {
            Some(FileEntry::Directory) => {}
            Some(FileEntry::File(_)) => {
                return Err(FileSystemError::NotADirectory { path: path.to_path_buf() }.into())
            }
            None => return Err(FileSystemError::NotFound { path: path.to_path_buf() }.into()),
        }

        let mut entries = Vec::new();

        for (p, _) in files.iter() {
            // Check if this path is a direct child of the given path
            if let Ok(relative) = p.strip_prefix(path) {
                // Count the number of components in the relative path
                // Direct children should have exactly 1 component
                if relative.components().count() == 1 {
                    entries.push(p.clone());
                }
            }
        }

        Ok(entries)
    }

    async fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let files = self.files.lock().unwrap();

        // Check if path exists and is a directory
        match files.get(path) {
            Some(FileEntry::Directory) => {}
            Some(FileEntry::File(_)) => {
                return Err(FileSystemError::NotADirectory { path: path.to_path_buf() }.into())
            }
            None => return Err(FileSystemError::NotFound { path: path.to_path_buf() }.into()),
        }

        let mut entries = Vec::new();

        for (p, _) in files.iter() {
            // Include the directory itself and all its descendants
            if p == path || p.starts_with(path) {
                entries.push(p.clone());
            }
        }

        Ok(entries)
    }

    async fn metadata(&self, _path: &Path) -> Result<std::fs::Metadata> {
        // Mock implementation doesn't support full metadata
        // This is a limitation for testing
        Err(FileSystemError::Operation("metadata not supported in MockFileSystem".to_string())
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_filesystem_is_empty() {
        let fs = MockFileSystem::new();
        assert!(fs.is_empty());
        assert_eq!(fs.len(), 0);
    }

    #[tokio::test]
    async fn test_add_and_read_file() {
        let fs = MockFileSystem::new();
        let path = Path::new("/test/file.txt");

        fs.add_file(path, "test content");

        let content = fs.read_file_string(path).await.unwrap();
        assert_eq!(content, "test content");
    }

    #[tokio::test]
    async fn test_write_and_read() {
        let fs = MockFileSystem::new();
        let path = Path::new("/test.txt");

        fs.write_file_string(path, "content").await.unwrap();
        let content = fs.read_file_string(path).await.unwrap();

        assert_eq!(content, "content");
    }

    #[tokio::test]
    async fn test_file_exists() {
        let fs = MockFileSystem::new();
        let path = Path::new("/test.txt");

        assert!(!fs.exists(path).await);

        fs.add_file(path, "content");

        assert!(fs.exists(path).await);
    }

    #[tokio::test]
    async fn test_directory_operations() {
        let fs = MockFileSystem::new();
        let dir = Path::new("/test/dir");

        fs.create_dir_all(dir).await.unwrap();

        assert!(fs.exists(dir).await);
    }

    #[tokio::test]
    async fn test_remove_file() {
        let fs = MockFileSystem::new();
        let path = Path::new("/test.txt");

        fs.add_file(path, "content");
        assert!(fs.exists(path).await);

        fs.remove_file(path).await.unwrap();
        assert!(!fs.exists(path).await);
    }

    #[tokio::test]
    async fn test_read_dir() {
        let fs = MockFileSystem::new();

        fs.add_dir(Path::new("/test"));
        fs.add_file("/test/file1.txt", "content1");
        fs.add_file("/test/file2.txt", "content2");
        fs.add_file("/test/sub/file3.txt", "content3");

        let entries = fs.read_dir(Path::new("/test")).await.unwrap();

        // Should only return direct children
        assert_eq!(entries.len(), 3); // file1.txt, file2.txt, sub/
    }

    #[tokio::test]
    async fn test_copy_file() {
        let fs = MockFileSystem::new();
        let from = Path::new("/source.txt");
        let to = Path::new("/dest.txt");

        fs.add_file(from, "content");
        let content = fs.read_file(from).await.unwrap();
        fs.write_file(to, &content).await.unwrap();

        let content = fs.read_file_string(to).await.unwrap();
        assert_eq!(content, "content");
    }

    #[tokio::test]
    async fn test_rename_file() {
        let fs = MockFileSystem::new();
        let from = Path::new("/old.txt");
        let to = Path::new("/new.txt");

        fs.add_file(from, "content");
        let content = fs.read_file(from).await.unwrap();
        fs.remove(from).await.unwrap();
        fs.write_file(to, &content).await.unwrap();

        assert!(!fs.exists(from).await);
        assert!(fs.exists(to).await);

        let content = fs.read_file_string(to).await.unwrap();
        assert_eq!(content, "content");
    }

    #[tokio::test]
    async fn test_list_files() {
        let fs = MockFileSystem::new();

        fs.add_dir(Path::new("/test"));
        fs.add_file("/test/file1.txt", "content1");
        fs.add_file("/test/file2.txt", "content2");

        let files = fs.list_files();
        assert_eq!(files.len(), 2);
    }

    #[tokio::test]
    async fn test_clear() {
        let fs = MockFileSystem::new();

        fs.add_file("/test1.txt", "content1");
        fs.add_file("/test2.txt", "content2");

        // len() includes both files and directories (root "/" is created)
        assert!(fs.len() >= 2);
        assert_eq!(fs.list_files().len(), 2);

        fs.clear();

        assert!(fs.is_empty());
        assert_eq!(fs.len(), 0);
    }
}
