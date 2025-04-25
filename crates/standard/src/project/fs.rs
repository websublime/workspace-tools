//! Filesystem operations for Node.js projects.

use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

use crate::error::FileSystemError;

/// Trait defining filesystem operations
pub trait FileSystem: Send + Sync {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, FileSystemError>;
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), FileSystemError>;
    fn read_file_string(&self, path: &Path) -> Result<String, FileSystemError>;
    fn write_file_string(&self, path: &Path, contents: &str) -> Result<(), FileSystemError>;
    fn create_dir_all(&self, path: &Path) -> Result<(), FileSystemError>;
    fn remove(&self, path: &Path) -> Result<(), FileSystemError>;
    fn exists(&self, path: &Path) -> bool;
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FileSystemError>;
    fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FileSystemError>;
}

/// Manager for safe filesystem operations
#[derive(Debug, Default)]
pub struct FileSystemManager;

impl FileSystemManager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates that a path exists.
    fn validate_path(&self, path: &Path) -> Result<(), FileSystemError> {
        if !path.exists() {
            return Err(FileSystemError::NotFound { path: path.to_path_buf() });
        }
        Ok(())
    }
}

impl FileSystem for FileSystemManager {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, FileSystemError> {
        self.validate_path(path)?;
        let mut file = File::open(path).map_err(|e| FileSystemError::from_io(e, path))?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .map_err(|e| FileSystemError::from_io(e, path))?;
        Ok(contents)
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), FileSystemError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| FileSystemError::from_io(e, parent))?;
        }
        let mut file = File::create(path).map_err(|e| FileSystemError::from_io(e, path))?;
        file.write_all(contents)
            .map_err(|e| FileSystemError::from_io(e, path))?;
        Ok(())
    }

    fn read_file_string(&self, path: &Path) -> Result<String, FileSystemError> {
        let bytes = self.read_file(path)?;
        String::from_utf8(bytes).map_err(|e| FileSystemError::Utf8Decode {
            path: path.to_path_buf(),
            source: e,
        })
    }

    fn write_file_string(&self, path: &Path, contents: &str) -> Result<(), FileSystemError> {
        self.write_file(path, contents.as_bytes())
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), FileSystemError> {
        fs::create_dir_all(path).map_err(|e| FileSystemError::from_io(e, path))
    }

    fn remove(&self, path: &Path) -> Result<(), FileSystemError> {
        self.validate_path(path)?;
        if path.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        }
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
                entry_result
                    .map(|entry| entry.path().to_path_buf())
                    .map_err(|e| {
                        let path_context = e.path().unwrap_or(path).to_path_buf();
                        FileSystemError::from_io(
                            e.into_io_error().unwrap_or_else(|| std::io::Error::new(std::io::ErrorKind::Other,"walkdir error")),
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
         assert!(matches!(result_read_dir_on_file.unwrap_err(), FileSystemError::NotADirectory { .. }));

         let result_walk_dir_on_file = fs_manager.walk_dir(&file_path);
         assert!(matches!(result_walk_dir_on_file.unwrap_err(), FileSystemError::NotADirectory { .. }));
    }
}

// Removed duplicate content below this line
    /// * `path` - Path to create
    fn create_dir_all(&self, path: &Path) -> Result<(), FileSystemError>;

    /// Removes a file or directory
    ///
    /// # Arguments
    ///
    /// * `path` - Path to remove
    fn remove(&self, path: &Path) -> Result<(), FileSystemError> {
        self.validate_path(path)?; // Use correct validation method name
        if path.is_dir() {
            fs::remove_dir_all(path)
    /// # Arguments
    ///
    /// * `path` - Path to check
    fn exists(&self, path: &Path) -> bool;

    /// Lists contents of a directory
    ///
    /// # Arguments
    ///
    /// * `path` - Directory to list
    ///
    /// # Returns
    ///
    /// Iterator over directory entries
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FileSystemError> {
        self.validate_path(path)?; // Use correct validation method name
        if !path.is_dir() {
            return Err(FileSystemError::NotADirectory { path: path.to_path_buf() });
    /// # Arguments
    ///
    /// * `path` - Directory to walk
    ///
    /// # Returns
    ///
    /// Iterator over all files in directory and subdirectories
    fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FileSystemError> {
        self.validate_path(path)?; // Use correct validation method name
        if !path.is_dir() {
            return Err(FileSystemError::NotADirectory { path: path.to_path_buf() });
#[derive(Debug, Default)]
pub struct FileSystemManager {
    // Configuration could be added here in the future
#[async_trait::async_trait] // Add async_trait if async methods are needed later
impl FileSystem for FileSystemManager {
    /// Creates a new FileSystemManager
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::project::FileSystemManager;
    ///
    /// let fs = FileSystemManager::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates that a path exists.
    fn validate_path(&self, path: &Path) -> Result<(), FileSystemError> {
        if !path.exists() {
            return Err(FileSystemError::NotFound { path: path.to_path_buf() });
        }
        Ok(())
    }
}
        if !path.exists() {
            .into_iter()
             .map(|entry_result| {
                 entry_result
                     .map(|entry| entry.path().to_path_buf())
                     .map_err(|e| FileSystemError::from_io(e.into_io_error().unwrap_or_else(|| std::io::Error::new(std::io::ErrorKind::Other,"walkdir error")), path))
             })
             .collect::<Result<Vec<_>, _>>()? // Propagate errors from walkdir
        )
    }
}
}

impl FileSystem for FileSystemManager {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, FileSystemError> {
        self.validate_path(path)?;
        let result = fs_manager.validate_path(Path::new("/nonexistent/path")); // Use correct validation method name
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FileSystemError::NotFound { .. }));
    }
}

    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), FileSystemError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| FileSystemError::from_io(e, parent))?;
        }

        let mut file = File::create(path).map_err(|e| FileSystemError::from_io(e, path))?;
        file.write_all(contents)
            .map_err(|e| FileSystemError::from_io(e, path))?;
        Ok(())
    }

    fn read_file_string(&self, path: &Path) -> Result<String, FileSystemError> {
        let bytes = self.read_file(path)?;
        String::from_utf8(bytes).map_err(|e| FileSystemError::Utf8Decode {
            path: path.to_path_buf(),
            source: e,
        })
    }

    fn write_file_string(&self, path: &Path, contents: &str) -> Result<(), FileSystemError> {
        self.write_file(path, contents.as_bytes())
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), FileSystemError> {
        fs::create_dir_all(path).map_err(|e| FileSystemError::from_io(e, path))
    }

    fn remove(&self, path: &Path) -> Result<(), FileSystemError> {
        self.validate_path(path)?;
        if path.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        }
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
        Ok(WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .map(|entry| entry.path().to_path_buf())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_error_handling() {
        let fs = FileSystemManager::new();

        // Test reading non-existent file
        let result = fs.read_file(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }

    // Remove duplicate test function
}
        // Test validation of non-existent path
        let result = fs.validate_path(Path::new("/nonexistent/path"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FileSystemError::NotFound { .. }));
    }

    #[test]
    fn test_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let fs = FileSystemManager::new();

        // Test file writing
        let test_file = temp_dir.path().join("test.txt");
        fs.write_file_string(&test_file, "test content").unwrap();

        // Test file reading
        let content = fs.read_file_string(&test_file).unwrap();
        assert_eq!(content, "test content");

        // Test file existence
        assert!(fs.exists(&test_file));

        // Test file removal
        fs.remove(&test_file).unwrap();
        assert!(!fs.exists(&test_file));
    }

    #[test]
    fn test_directory_operations() {
        let temp_dir = TempDir::new().unwrap();
        let fs = FileSystemManager::new();

        // Test directory creation
        let test_dir = temp_dir.path().join("test_dir");
        fs.create_dir_all(&test_dir).unwrap();
        assert!(test_dir.is_dir());

        // Create some test files
        fs.write_file_string(&test_dir.join("file1.txt"), "content1").unwrap();
        fs.write_file_string(&test_dir.join("file2.txt"), "content2").unwrap();

        // Test directory listing
        let entries = fs.read_dir(&test_dir).unwrap();
        assert_eq!(entries.len(), 2);

        // Test directory walking
        let all_files = fs.walk_dir(&test_dir).unwrap();
        assert_eq!(all_files.len(), 2);
    }

    #[test]
    fn test_error_handling() {
        let fs = FileSystemManager::new();

        // Test reading non-existent file
        let result = fs.read_file(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());

        // Test invalid path validation
        let result = fs.validate_path(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}

