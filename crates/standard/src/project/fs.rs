//! Filesystem operations for Node.js projects.
//!
//! What:
//! This module provides a safe abstraction layer for filesystem operations
//! specifically designed for Node.js projects. It handles common operations
//! like reading/writing files, directory traversal, and path validation.
//!
//! Who:
//! Used by developers who need to:
//! - Perform filesystem operations safely within Node.js projects
//! - Handle file content as strings or raw bytes
//! - Traverse directories recursively
//! - Validate path existence and types
//!
//! Why:
//! Safe filesystem operations are essential for:
//! - Preventing data loss or corruption
//! - Providing consistent error handling
//! - Ensuring cross-platform compatibility
//! - Maintaining filesystem integrity during operations

use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

use crate::error::FileSystemError;

/// Trait defining filesystem operations for Node.js projects.
///
/// Provides a comprehensive set of operations for interacting with the filesystem
/// in a type-safe and error-handled way.
///
/// # Examples
///
/// ```no_run
/// use sublime_standard_tools::project::{FileSystem, FileSystemManager};
/// use std::path::Path;
///
/// let fs_manager = FileSystemManager::new();
/// if fs_manager.exists(Path::new("package.json")) {
///     let content = fs_manager.read_file_string(Path::new("package.json"))
///         .expect("Failed to read package.json");
///     println!("Package.json content: {}", content);
/// }
/// ```
pub trait FileSystem: Send + Sync {
    /// Reads a file as raw bytes.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to read
    ///
    /// # Returns
    ///
    /// Raw file contents as bytes or a filesystem error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_standard_tools::project::{FileSystem, FileSystemManager};
    /// # use std::path::Path;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let data = fs.read_file(Path::new("config.json"))?;
    /// # Ok(())
    /// # }
    /// ```
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, FileSystemError>;

    /// Writes raw bytes to a file.
    ///
    /// Creates parent directories if they don't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to write
    /// * `contents` - Raw byte content to write
    ///
    /// # Returns
    ///
    /// Success or a filesystem error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_standard_tools::project::{FileSystem, FileSystemManager};
    /// # use std::path::Path;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// fs.write_file(Path::new("data.bin"), &[0, 1, 2, 3])?;
    /// # Ok(())
    /// # }
    /// ```
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), FileSystemError>;

    /// Reads a file as a UTF-8 string.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to read
    ///
    /// # Returns
    ///
    /// File contents as a string or a filesystem error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_standard_tools::project::{FileSystem, FileSystemManager};
    /// # use std::path::Path;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let content = fs.read_file_string(Path::new("package.json"))?;
    /// println!("Package name: {}", content);
    /// # Ok(())
    /// # }
    /// ```
    fn read_file_string(&self, path: &Path) -> Result<String, FileSystemError>;

    /// Writes a string to a file using UTF-8 encoding.
    ///
    /// Creates parent directories if they don't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to write
    /// * `contents` - String content to write
    ///
    /// # Returns
    ///
    /// Success or a filesystem error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_standard_tools::project::{FileSystem, FileSystemManager};
    /// # use std::path::Path;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// fs.write_file_string(Path::new("README.md"), "# Project Documentation")?;
    /// # Ok(())
    /// # }
    /// ```
    fn write_file_string(&self, path: &Path, contents: &str) -> Result<(), FileSystemError>;

    /// Creates a directory and all parent directories if they don't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory to create
    ///
    /// # Returns
    ///
    /// Success or a filesystem error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_standard_tools::project::{FileSystem, FileSystemManager};
    /// # use std::path::Path;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// fs.create_dir_all(Path::new("dist/assets"))?;
    /// # Ok(())
    /// # }
    /// ```
    fn create_dir_all(&self, path: &Path) -> Result<(), FileSystemError>;

    /// Removes a file or directory.
    ///
    /// If the path is a directory, it will be removed recursively.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file or directory to remove
    ///
    /// # Returns
    ///
    /// Success or a filesystem error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_standard_tools::project::{FileSystem, FileSystemManager};
    /// # use std::path::Path;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// fs.remove(Path::new("temp.log"))?;
    /// # Ok(())
    /// # }
    /// ```
    fn remove(&self, path: &Path) -> Result<(), FileSystemError>;

    /// Checks if a path exists.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check
    ///
    /// # Returns
    ///
    /// True if the path exists, false otherwise
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_standard_tools::project::{FileSystem, FileSystemManager};
    /// # use std::path::Path;
    /// let fs = FileSystemManager::new();
    /// if fs.exists(Path::new("node_modules")) {
    ///     println!("Dependencies are installed");
    /// }
    /// ```
    fn exists(&self, path: &Path) -> bool;

    /// Lists the contents of a directory (non-recursively).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory to read
    ///
    /// # Returns
    ///
    /// A list of paths in the directory or a filesystem error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_standard_tools::project::{FileSystem, FileSystemManager};
    /// # use std::path::Path;
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

    /// Recursively traverses a directory.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory to traverse
    ///
    /// # Returns
    ///
    /// A list of all paths in the directory tree or a filesystem error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_standard_tools::project::{FileSystem, FileSystemManager};
    /// # use std::path::Path;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let all_files = fs.walk_dir(Path::new("."))?;
    /// println!("Found {} files and directories", all_files.len());
    /// # Ok(())
    /// # }
    /// ```
    fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FileSystemError>;
}

/// Manager for safe filesystem operations within Node.js projects.
///
/// Provides implementations of filesystem operations with proper error handling
/// and validation.
///
/// # Examples
///
/// ```no_run
/// use sublime_standard_tools::project::{FileSystem, FileSystemManager};
/// use std::path::Path;
///
/// let fs = FileSystemManager::new();
/// let package_json = fs.read_file_string(Path::new("package.json"));
/// match package_json {
///     Ok(content) => println!("Successfully read package.json"),
///     Err(e) => eprintln!("Error reading package.json: {}", e),
/// }
/// ```
#[derive(Debug, Default)]
pub struct FileSystemManager;

impl FileSystemManager {
    /// Creates a new FileSystemManager.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::FileSystemManager;
    ///
    /// let fs = FileSystemManager::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    /// Validates that a path exists.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to validate
    ///
    /// # Returns
    ///
    /// Self reference for method chaining or a filesystem error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_standard_tools::project::FileSystemManager;
    /// # use std::path::Path;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// fs.validate_path(Path::new("package.json"))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let fs_manager = FileSystemManager::new();

        let test_file = temp_dir.path().join("test.txt");
        fs_manager.write_file_string(&test_file, "test content").unwrap();
        let content = fs_manager.read_file_string(&test_file).unwrap();
        assert_eq!(content, "test content");
        assert!(fs_manager.exists(&test_file));
        fs_manager.remove(&test_file).unwrap();
        assert!(!fs_manager.exists(&test_file));
    }

    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_directory_operations() {
        let temp_dir = TempDir::new().unwrap();
        let fs_manager = FileSystemManager::new();

        let test_dir = temp_dir.path().join("test_dir");
        fs_manager.create_dir_all(&test_dir).unwrap();
        assert!(test_dir.is_dir());

        let file1 = test_dir.join("file1.txt");
        fs_manager.write_file_string(&file1, "content1").unwrap();
        let file2 = test_dir.join("file2.txt");
        fs_manager.write_file_string(&file2, "content2").unwrap();

        let entries = fs_manager.read_dir(&test_dir).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&file1));
        assert!(entries.contains(&file2));

        let all_paths_result = fs_manager.walk_dir(temp_dir.path());
        assert!(all_paths_result.is_ok());
        let all_paths = all_paths_result.unwrap();
        assert!(all_paths.contains(&temp_dir.path().to_path_buf()));
        assert!(all_paths.contains(&test_dir));
        assert!(all_paths.contains(&file1));
        assert!(all_paths.contains(&file2));
    }

    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_error_handling() {
        let fs_manager = FileSystemManager::new();

        let non_existent_path = Path::new("/nonexistent/path/that/surely/does/not/exist");
        let result_read = fs_manager.read_file(non_existent_path);
        assert!(matches!(result_read.unwrap_err(), FileSystemError::NotFound { .. }));

        let result_validate = fs_manager.validate_path(non_existent_path);
        assert!(matches!(result_validate.unwrap_err(), FileSystemError::NotFound { .. }));

        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();
        let result_read_dir_as_file = fs_manager.read_file(dir_path);
        assert!(result_read_dir_as_file.is_err()); // OS specific error, likely IsADirectory

        let file_path = dir_path.join("a_file");
        fs_manager.write_file_string(&file_path, "hello").unwrap();
        let result_read_dir_on_file = fs_manager.read_dir(&file_path);
        assert!(matches!(
            result_read_dir_on_file.unwrap_err(),
            FileSystemError::NotADirectory { .. }
        ));

        let result_walk_dir_on_file = fs_manager.walk_dir(&file_path);
        assert!(matches!(
            result_walk_dir_on_file.unwrap_err(),
            FileSystemError::NotADirectory { .. }
        ));
    }
}
