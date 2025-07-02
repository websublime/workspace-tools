//! File system service implementation
//!
//! Provides centralized file system operations for the monorepo with proper
//! error handling, path resolution, and monorepo-specific functionality.

use crate::error::Result;
use std::path::{Path, PathBuf};
use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};

/// File system operations service
///
/// Provides monorepo-aware file system operations including path resolution,
/// file reading/writing, and directory operations. All paths are resolved
/// relative to the monorepo root and include proper error handling.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::core::services::FileSystemService;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs_service = FileSystemService::new("/path/to/monorepo")?;
///
/// // Read a file relative to monorepo root
/// let content = fs_service.read_file_string("package.json")?;
///
/// // Check if a path exists
/// if fs_service.exists("src/main.rs") {
///     println!("Main source file found");
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub(crate) struct FileSystemService {
    /// Underlying file system manager
    file_system: FileSystemManager,

    /// Root path of the monorepo
    root_path: PathBuf,
}

#[allow(dead_code)]
impl FileSystemService {
    /// Create a new file system service
    ///
    /// Initializes the file system service with the specified monorepo root path.
    /// All subsequent operations will be relative to this root path.
    ///
    /// # Arguments
    ///
    /// * `root_path` - Root path of the monorepo
    ///
    /// # Returns
    ///
    /// A new file system service.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Root path does not exist
    /// - Root path is not accessible
    /// - File system manager cannot be created
    pub fn new<P: AsRef<Path>>(root_path: P) -> Result<Self> {
        let root_path = root_path.as_ref().to_path_buf();

        // Ensure root path exists
        if !root_path.exists() {
            return Err(crate::error::Error::filesystem(format!(
                "Monorepo root path does not exist: {}",
                root_path.display()
            )));
        }

        let file_system = FileSystemManager::new();

        Ok(Self { file_system, root_path })
    }

    /// Get the underlying file system manager
    ///
    /// Provides access to the underlying FileSystemManager for operations
    /// that require the raw manager interface.
    ///
    /// # Returns
    ///
    /// Reference to the file system manager.
    pub fn manager(&self) -> &FileSystemManager {
        &self.file_system
    }

    /// Get the root path
    ///
    /// Returns the root path of the monorepo that this service manages.
    ///
    /// # Returns
    ///
    /// Reference to the root path.
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Resolve a relative path to absolute path
    ///
    /// Converts a path relative to the monorepo root into an absolute path.
    /// Handles both relative and already-absolute paths correctly.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - Path relative to monorepo root
    ///
    /// # Returns
    ///
    /// Absolute path resolved from the monorepo root.
    pub fn resolve_path<P: AsRef<Path>>(&self, relative_path: P) -> PathBuf {
        let path = relative_path.as_ref();
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root_path.join(path)
        }
    }

    /// Check if a path exists
    ///
    /// Checks whether a file or directory exists at the specified path
    /// relative to the monorepo root.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - Path relative to monorepo root
    ///
    /// # Returns
    ///
    /// True if the path exists, false otherwise.
    pub fn exists<P: AsRef<Path>>(&self, relative_path: P) -> bool {
        self.resolve_path(relative_path).exists()
    }

    /// Check if a path is a directory
    ///
    /// Checks whether the specified path is a directory.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - Path relative to monorepo root
    ///
    /// # Returns
    ///
    /// True if the path exists and is a directory, false otherwise.
    pub fn is_dir<P: AsRef<Path>>(&self, relative_path: P) -> bool {
        self.resolve_path(relative_path).is_dir()
    }

    /// Check if a path is a file
    ///
    /// Checks whether the specified path is a file.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - Path relative to monorepo root
    ///
    /// # Returns
    ///
    /// True if the path exists and is a file, false otherwise.
    pub fn is_file<P: AsRef<Path>>(&self, relative_path: P) -> bool {
        self.resolve_path(relative_path).is_file()
    }

    /// Read file as string
    ///
    /// Reads the entire contents of a file as a UTF-8 string.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - Path to file relative to monorepo root
    ///
    /// # Returns
    ///
    /// File contents as a string.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File does not exist
    /// - File cannot be read
    /// - File contents are not valid UTF-8
    pub fn read_file_string<P: AsRef<Path>>(&self, relative_path: P) -> Result<String> {
        let absolute_path = self.resolve_path(relative_path);
        self.file_system.read_file_string(&absolute_path).map_err(|e| {
            crate::error::Error::filesystem(format!(
                "Failed to read file {}: {}",
                absolute_path.display(),
                e
            ))
        })
    }

    /// Read file as bytes
    ///
    /// Reads the entire contents of a file as raw bytes.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - Path to file relative to monorepo root
    ///
    /// # Returns
    ///
    /// File contents as bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File does not exist
    /// - File cannot be read
    pub fn read_file<P: AsRef<Path>>(&self, relative_path: P) -> Result<Vec<u8>> {
        let absolute_path = self.resolve_path(relative_path);
        self.file_system.read_file(&absolute_path).map_err(|e| {
            crate::error::Error::filesystem(format!(
                "Failed to read file {}: {}",
                absolute_path.display(),
                e
            ))
        })
    }

    /// Write string to file
    ///
    /// Writes a string to the specified file, creating directories as needed.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - Path to file relative to monorepo root
    /// * `content` - Content to write to the file
    ///
    /// # Returns
    ///
    /// Success if file was written successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory cannot be created
    /// - File cannot be written
    /// - Insufficient permissions
    pub fn write_file_string<P: AsRef<Path>>(&self, relative_path: P, content: &str) -> Result<()> {
        let absolute_path = self.resolve_path(relative_path);
        self.file_system.write_file_string(&absolute_path, content).map_err(|e| {
            crate::error::Error::filesystem(format!(
                "Failed to write file {}: {}",
                absolute_path.display(),
                e
            ))
        })
    }

    /// Write bytes to file
    ///
    /// Writes raw bytes to the specified file, creating directories as needed.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - Path to file relative to monorepo root
    /// * `content` - Content to write to the file
    ///
    /// # Returns
    ///
    /// Success if file was written successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory cannot be created
    /// - File cannot be written
    /// - Insufficient permissions
    pub fn write_file<P: AsRef<Path>>(&self, relative_path: P, content: &[u8]) -> Result<()> {
        let absolute_path = self.resolve_path(relative_path);
        self.file_system.write_file(&absolute_path, content).map_err(|e| {
            crate::error::Error::filesystem(format!(
                "Failed to write file {}: {}",
                absolute_path.display(),
                e
            ))
        })
    }

    /// Create directory
    ///
    /// Creates a directory and all necessary parent directories.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - Path to directory relative to monorepo root
    ///
    /// # Returns
    ///
    /// Success if directory was created successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory cannot be created
    /// - Insufficient permissions
    pub fn create_dir_all<P: AsRef<Path>>(&self, relative_path: P) -> Result<()> {
        let absolute_path = self.resolve_path(relative_path);
        self.file_system.create_dir_all(&absolute_path).map_err(|e| {
            crate::error::Error::filesystem(format!(
                "Failed to create directory {}: {}",
                absolute_path.display(),
                e
            ))
        })
    }

    /// List directory contents
    ///
    /// Lists all files and directories in the specified directory.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - Path to directory relative to monorepo root
    ///
    /// # Returns
    ///
    /// Vector of directory entries.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory does not exist
    /// - Directory cannot be read
    /// - Insufficient permissions
    pub fn list_directory<P: AsRef<Path>>(&self, relative_path: P) -> Result<Vec<PathBuf>> {
        let absolute_path = self.resolve_path(relative_path);

        let entries = std::fs::read_dir(&absolute_path).map_err(|e| {
            crate::error::Error::filesystem(format!(
                "Failed to read directory {}: {}",
                absolute_path.display(),
                e
            ))
        })?;

        let mut paths = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| {
                crate::error::Error::filesystem(format!(
                    "Failed to read directory entry in {}: {}",
                    absolute_path.display(),
                    e
                ))
            })?;
            paths.push(entry.path());
        }

        Ok(paths)
    }
}
