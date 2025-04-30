//! # FileSystem Manager Implementation
//!
//! ## What
//! This file implements the `FileSystem` trait for the `FileSystemManager` struct,
//! providing concrete filesystem operations using the standard library and the
//! `walkdir` crate.
//!
//! ## How
//! The implementation uses a combination of standard library functions from the
//! `std::fs` module and external crates like `walkdir` for recursive directory
//! traversal. All operations are wrapped with proper error handling to provide
//! consistent error types and messages.
//!
//! ## Why
//! This implementation provides a real filesystem interface that can be used
//! in production code, while still adhering to the trait contract that allows
//! for testing with mock implementations if needed.

use super::{FileSystem, FileSystemManager};
use crate::error::FileSystemError;
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

impl FileSystemManager {
    /// Creates a new FileSystemManager instance.
    ///
    /// # Returns
    ///
    /// A new `FileSystemManager` instance ready for use.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    ///
    /// let fs_manager = FileSystemManager::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    /// Validates that a path exists, returning an error if it doesn't.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to validate
    ///
    /// # Returns
    ///
    /// * `Ok(&Self)` - If the path exists
    /// * `Err(FileSystemError)` - If the path does not exist
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # use sublime_standard_tools::filesystem::FileSystemManager;
    /// #
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// // This would succeed if Cargo.toml exists
    /// let _ = fs.validate_path(Path::new("Cargo.toml"))?;
    /// # Ok(())
    /// # }
    /// ```
    fn validate_path(&self, path: &Path) -> Result<&Self, FileSystemError> {
        if !path.exists() {
            return Err(FileSystemError::NotFound { path: path.to_path_buf() });
        }
        Ok(self)
    }
}

impl FileSystem for FileSystemManager {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, FileSystemError> {
        self.validate_path(path)?;
        let mut file = File::open(path).map_err(|e| FileSystemError::from_io(e, path))?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).map_err(|e| FileSystemError::from_io(e, path))?;
        Ok(contents)
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), FileSystemError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| FileSystemError::from_io(e, parent))?;
        }
        let mut file = File::create(path).map_err(|e| FileSystemError::from_io(e, path))?;
        file.write_all(contents).map_err(|e| FileSystemError::from_io(e, path))?;
        Ok(())
    }

    fn read_file_string(&self, path: &Path) -> Result<String, FileSystemError> {
        let bytes = self.read_file(path)?;
        String::from_utf8(bytes)
            .map_err(|e| FileSystemError::Utf8Decode { path: path.to_path_buf(), source: e })
    }

    fn write_file_string(&self, path: &Path, contents: &str) -> Result<(), FileSystemError> {
        self.write_file(path, contents.as_bytes())
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), FileSystemError> {
        fs::create_dir_all(path).map_err(|e| FileSystemError::from_io(e, path))
    }

    fn remove(&self, path: &Path) -> Result<(), FileSystemError> {
        self.validate_path(path)?;
        if path.is_dir() { fs::remove_dir_all(path) } else { fs::remove_file(path) }
            .map_err(|e| FileSystemError::from_io(e, path))
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FileSystemError> {
        self.validate_path(path)?;
        if !path.is_dir() {
            return Err(FileSystemError::NotADirectory { path: path.to_path_buf() });
        }
        fs::read_dir(path)
            .map_err(|e| FileSystemError::from_io(e, path))?
            .map(|res| res.map(|e| e.path()).map_err(|e| FileSystemError::from_io(e, path)))
            .collect()
    }

    fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FileSystemError> {
        self.validate_path(path)?;
        if !path.is_dir() {
            return Err(FileSystemError::NotADirectory { path: path.to_path_buf() });
        }
        WalkDir::new(path)
            .into_iter()
            .map(|entry_result| {
                entry_result.map(|entry| entry.path().to_path_buf()).map_err(|e| {
                    let path_context = e.path().unwrap_or(path).to_path_buf();
                    FileSystemError::from_io(
                        e.into_io_error().unwrap_or_else(|| {
                            std::io::Error::new(std::io::ErrorKind::Other, "walkdir error")
                        }),
                        path_context, // Provide path context if available
                    )
                })
            })
            .collect() // Collect results, propagating the first error
    }
}
