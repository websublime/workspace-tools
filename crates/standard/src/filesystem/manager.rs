//! # Async FileSystem Manager Implementation - Unified
//!
//! ## What
//! This file implements the `AsyncFileSystem` trait for the `FileSystemManager` struct,
//! providing async filesystem operations using tokio::fs for maximum performance.
//! All sync operations have been removed for architectural clarity.
//!
//! ## How
//! The implementation uses tokio::fs functions for all filesystem operations, providing
//! non-blocking I/O operations. All operations include proper error handling and
//! timeout configuration.
//!
//! ## Why
//! Async filesystem operations are essential for performance in large monorepos where
//! thousands of files need to be processed. This unified async-only approach eliminates
//! confusion and provides the foundation for concurrent operations.

use super::types::{AsyncFileSystem, AsyncFileSystemConfig};
use crate::error::{Error, FileSystemError, Result};
use async_trait::async_trait;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{fs, time::timeout};

/// Async manager for filesystem operations.
///
/// This struct provides a concrete implementation of the `AsyncFileSystem` trait
/// using tokio::fs for high-performance async filesystem operations.
///
/// # Examples
///
/// ```rust
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = FileSystemManager::new();
/// if fs.exists(Path::new("Cargo.toml")).await {
///     println!("Cargo.toml exists");
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct FileSystemManager {
    config: AsyncFileSystemConfig,
}

impl Default for FileSystemManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystemManager {
    /// Creates a new `FileSystemManager` instance with default configuration.
    ///
    /// # Returns
    ///
    /// A new `FileSystemManager` instance ready for use.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    ///
    /// let fs_manager = FileSystemManager::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: AsyncFileSystemConfig::default(),
        }
    }

    /// Creates a new `FileSystemManager` instance with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to use for filesystem operations
    ///
    /// # Returns
    ///
    /// A new `FileSystemManager` instance with the specified configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::filesystem::{FileSystemManager, AsyncFileSystemConfig};
    /// use std::time::Duration;
    ///
    /// let config = AsyncFileSystemConfig::new()
    ///     .with_operation_timeout(Duration::from_secs(60));
    /// let fs_manager = FileSystemManager::with_config(config);
    /// ```
    #[must_use]
    pub fn with_config(config: AsyncFileSystemConfig) -> Self {
        Self { config }
    }

    /// Gets the current configuration.
    ///
    /// # Returns
    ///
    /// A reference to the current configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    ///
    /// let fs = FileSystemManager::new();
    /// let config = fs.config();
    /// println!("Operation timeout: {:?}", config.operation_timeout);
    /// ```
    #[must_use]
    pub fn config(&self) -> &AsyncFileSystemConfig {
        &self.config
    }

    /// Asynchronously validates that a path exists, returning an error if it doesn't.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the path exists
    /// * `Err(FileSystemError)` - If the path does not exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// fs.validate_path(Path::new("Cargo.toml")).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the path does not exist.
    async fn validate_path(&self, path: &Path) -> Result<()> {
        if !self.exists(path).await {
            return Err(Error::FileSystem(FileSystemError::NotFound {
                path: path.to_path_buf(),
            }));
        }
        Ok(())
    }

    /// Executes an async operation with a timeout.
    ///
    /// # Arguments
    ///
    /// * `operation` - The async operation to execute
    /// * `timeout_duration` - The timeout duration
    ///
    /// # Returns
    ///
    /// The result of the operation or a timeout error.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation times out or fails.
    async fn with_timeout<T, F>(&self, operation: F, timeout_duration: Duration) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        match timeout(timeout_duration, operation).await {
            Ok(result) => result,
            Err(_) => Err(Error::FileSystem(FileSystemError::Operation(format!(
                "Operation timed out after {timeout_duration:?}"
            )))),
        }
    }
}

#[async_trait]
impl AsyncFileSystem for FileSystemManager {
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        let operation = async {
            self.validate_path(path).await?;
            fs::read(path)
                .await
                .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, path)))
        };

        self.with_timeout(operation, self.config.read_timeout).await
    }

    async fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()> {
        let operation = async {
            // Create parent directories if they don't exist
            if let Some(parent) = path.parent() {
                if !self.exists(parent).await {
                    self.create_dir_all(parent).await?;
                }
            }

            fs::write(path, contents)
                .await
                .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, path)))?;

            Ok(())
        };

        self.with_timeout(operation, self.config.write_timeout).await
    }

    async fn read_file_string(&self, path: &Path) -> Result<String> {
        let operation = async {
            self.validate_path(path).await?;
            fs::read_to_string(path)
                .await
                .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, path)))
        };

        self.with_timeout(operation, self.config.read_timeout).await
    }

    async fn write_file_string(&self, path: &Path, contents: &str) -> Result<()> {
        let operation = async {
            // Create parent directories if they don't exist
            if let Some(parent) = path.parent() {
                if !self.exists(parent).await {
                    self.create_dir_all(parent).await?;
                }
            }

            fs::write(path, contents)
                .await
                .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, path)))?;

            Ok(())
        };

        self.with_timeout(operation, self.config.write_timeout).await
    }

    async fn create_dir_all(&self, path: &Path) -> Result<()> {
        let operation = async {
            fs::create_dir_all(path)
                .await
                .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, path)))?;

            Ok(())
        };

        self.with_timeout(operation, self.config.operation_timeout).await
    }

    async fn remove(&self, path: &Path) -> Result<()> {
        let operation = async {
            self.validate_path(path).await?;
            
            let metadata = fs::metadata(path)
                .await
                .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, path)))?;

            if metadata.is_dir() {
                fs::remove_dir_all(path)
                    .await
                    .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, path)))?;
            } else {
                fs::remove_file(path)
                    .await
                    .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, path)))?;
            }

            Ok(())
        };

        self.with_timeout(operation, self.config.operation_timeout).await
    }

    async fn exists(&self, path: &Path) -> bool {
        // tokio::fs doesn't have an exists method, so we use try_exists or metadata
        fs::try_exists(path).await.unwrap_or(false)
    }

    async fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let operation = async {
            self.validate_path(path).await?;
            let metadata = fs::metadata(path)
                .await
                .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, path)))?;
            if !metadata.is_dir() {
                return Err(Error::FileSystem(FileSystemError::NotADirectory {
                    path: path.to_path_buf(),
                }));
            }

            let mut entries = Vec::new();
            let mut read_dir = fs::read_dir(path)
                .await
                .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, path)))?;

            while let Some(entry) = read_dir
                .next_entry()
                .await
                .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, path)))?
            {
                entries.push(entry.path());
            }

            // Sort entries for consistent ordering
            entries.sort();
            Ok(entries)
        };

        self.with_timeout(operation, self.config.operation_timeout).await
    }

    async fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let operation = async {
            self.validate_path(path).await?;
            let mut paths = Vec::new();

            Self::walk_recursive(path, &mut paths, self).await?;

            // Sort all paths for consistent ordering
            paths.sort();
            Ok(paths)
        };

        self.with_timeout(operation, self.config.operation_timeout).await
    }

    async fn metadata(&self, path: &Path) -> Result<std::fs::Metadata> {
        let operation = async {
            self.validate_path(path).await?;
            fs::metadata(path)
                .await
                .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, path)))
        };

        self.with_timeout(operation, self.config.operation_timeout).await
    }
}

impl FileSystemManager {
    /// Recursively walks directory tree
    fn walk_recursive<'a>(
        path: &'a Path,
        paths: &'a mut Vec<PathBuf>,
        fs_manager: &'a FileSystemManager,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let entries = fs_manager.read_dir(path).await?;
            for entry in entries {
                paths.push(entry.clone());
                let metadata = fs::metadata(&entry)
                    .await
                    .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, &entry)))?;
                if metadata.is_dir() {
                    Self::walk_recursive(&entry, paths, fs_manager).await?;
                }
            }
            Ok(())
        })
    }
}