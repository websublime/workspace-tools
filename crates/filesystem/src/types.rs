//! # Types Module
//!
//! Core data types for representing filesystem entries and their metadata.
//!
//! ## What
//!
//! This module provides three fundamental types used throughout the crate:
//! - [`FileType`]: Discriminates between files, directories, and symlinks
//! - [`DirEntry`]: Represents a single entry when listing directory contents
//! - [`Metadata`]: Provides file/directory metadata (size, type)
//!
//! ## How
//!
//! These types abstract over platform-specific filesystem representations:
//! - `FileType` is a simple enum that normalizes the different ways OSes report types
//! - `DirEntry` wraps the essential information needed when traversing directories
//! - `Metadata` provides a platform-agnostic view of file attributes
//!
//! All types implement common traits (`Debug`, `Clone`, `PartialEq`, `Eq`) for
//! ergonomic use in tests and application code.
//!
//! ## Why
//!
//! Custom types (rather than re-exporting `std::fs` types) provide:
//! - **Testability**: `MockFileSystem` can construct these without touching disk
//! - **Consistency**: Same types work for both real and mock implementations
//! - **Extensibility**: Can add custom fields without breaking API
//! - **Documentation**: Clear documentation specific to this crate's semantics
//!
//! ## Example
//!
//! ```rust,ignore
//! use workspace_fs::{FileType, DirEntry, Metadata};
//!
//! let file_type = FileType::File;
//! assert!(file_type.is_file());
//! assert!(!file_type.is_dir());
//! assert!(!file_type.is_symlink());
//!
//! let dir_type = FileType::Dir;
//! assert!(dir_type.is_dir());
//! ```

// =============================================================================
// FileType Enum
// =============================================================================

/// Represents the type of a filesystem entry.
///
/// This enum provides a simple, unified representation of filesystem entry types
/// across all platforms. It normalizes the differences between how various
/// operating systems report file types.
///
/// # Variants
///
/// | Variant | Description |
/// |---------|-------------|
/// | [`File`][Self::File] | A regular file containing data |
/// | [`Dir`][Self::Dir] | A directory that can contain other entries |
/// | [`Symlink`][Self::Symlink] | A symbolic link pointing to another path |
///
/// # Trait Implementations
///
/// - [`Debug`]: Formats the variant name for debugging
/// - [`Clone`] and [`Copy`]: Allows copying by value (zero-cost)
/// - [`PartialEq`] and [`Eq`]: Enables equality comparisons
/// - [`From<std::fs::FileType>`]: Converts from the standard library type
///
/// # Example
///
/// ```rust,ignore
/// use workspace_fs::FileType;
///
/// // Create and check file types
/// let file = FileType::File;
/// assert!(file.is_file());
///
/// let dir = FileType::Dir;
/// assert!(dir.is_dir());
///
/// let symlink = FileType::Symlink;
/// assert!(symlink.is_symlink());
///
/// // Types are comparable
/// assert_eq!(FileType::File, FileType::File);
/// assert_ne!(FileType::File, FileType::Dir);
///
/// // Types are copyable
/// let copy = file;
/// assert_eq!(file, copy);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// A regular file containing data.
    ///
    /// This variant represents any file that is not a directory or symbolic link.
    /// It includes text files, binary files, executables, and any other regular
    /// file type.
    File,

    /// A directory that can contain other filesystem entries.
    ///
    /// Directories are containers for files, other directories, and symbolic links.
    /// They form the hierarchical structure of the filesystem.
    Dir,

    /// A symbolic link pointing to another path.
    ///
    /// Symbolic links are special filesystem entries that reference another path.
    /// The target path may or may not exist, and may be a file, directory, or
    /// another symbolic link.
    Symlink,
}

impl FileType {
    /// Returns `true` if this is a regular file.
    ///
    /// This method checks whether the entry is a regular file (not a directory
    /// or symbolic link).
    ///
    /// # Returns
    ///
    /// - `true` if the type is [`FileType::File`]
    /// - `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileType;
    ///
    /// assert!(FileType::File.is_file());
    /// assert!(!FileType::Dir.is_file());
    /// assert!(!FileType::Symlink.is_file());
    /// ```
    #[must_use]
    pub fn is_file(self) -> bool {
        matches!(self, Self::File)
    }

    /// Returns `true` if this is a directory.
    ///
    /// This method checks whether the entry is a directory.
    ///
    /// # Returns
    ///
    /// - `true` if the type is [`FileType::Dir`]
    /// - `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileType;
    ///
    /// assert!(FileType::Dir.is_dir());
    /// assert!(!FileType::File.is_dir());
    /// assert!(!FileType::Symlink.is_dir());
    /// ```
    #[must_use]
    pub fn is_dir(self) -> bool {
        matches!(self, Self::Dir)
    }

    /// Returns `true` if this is a symbolic link.
    ///
    /// This method checks whether the entry is a symbolic link.
    ///
    /// # Returns
    ///
    /// - `true` if the type is [`FileType::Symlink`]
    /// - `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileType;
    ///
    /// assert!(FileType::Symlink.is_symlink());
    /// assert!(!FileType::File.is_symlink());
    /// assert!(!FileType::Dir.is_symlink());
    /// ```
    #[must_use]
    pub fn is_symlink(self) -> bool {
        matches!(self, Self::Symlink)
    }
}

// =============================================================================
// Conversions
// =============================================================================

/// Converts from [`std::fs::FileType`] to [`FileType`].
///
/// This implementation allows seamless conversion from the standard library's
/// file type representation to this crate's abstraction.
///
/// # Conversion Logic
///
/// The conversion follows this priority order:
/// 1. If [`std::fs::FileType::is_symlink()`] returns `true` → [`FileType::Symlink`]
/// 2. If [`std::fs::FileType::is_dir()`] returns `true` → [`FileType::Dir`]
/// 3. Otherwise → [`FileType::File`]
///
/// The symlink check comes first because symbolic links can sometimes appear
/// as files or directories depending on how metadata is retrieved (with or
/// without following links).
///
/// # Example
///
/// ```rust,ignore
/// use workspace_fs::FileType;
/// use std::fs;
///
/// let metadata = fs::metadata("some_file.txt")?;
/// let file_type: FileType = metadata.file_type().into();
/// ```
impl From<std::fs::FileType> for FileType {
    fn from(ft: std::fs::FileType) -> Self {
        if ft.is_symlink() {
            Self::Symlink
        } else if ft.is_dir() {
            Self::Dir
        } else {
            Self::File
        }
    }
}

// =============================================================================
// Metadata Struct
// =============================================================================

/// Metadata information about a filesystem entry.
///
/// This struct provides a platform-agnostic representation of file metadata,
/// including the file type and size. It abstracts over the differences between
/// how various operating systems report file metadata.
///
/// # Fields
///
/// The struct contains the following private fields, accessible through getter methods:
/// - `file_type`: The type of the filesystem entry ([`FileType`])
/// - `len`: The size of the file in bytes (always 0 for directories)
///
/// # Trait Implementations
///
/// - [`Debug`]: Formats the metadata for debugging
/// - [`Clone`]: Allows cloning the metadata
/// - [`From<std::fs::Metadata>`]: Converts from the standard library type
///
/// # Example
///
/// ```rust,ignore
/// use workspace_fs::{Metadata, FileType};
///
/// // Create metadata for a 1024-byte file
/// let metadata = Metadata::new(FileType::File, 1024);
/// assert!(metadata.is_file());
/// assert_eq!(metadata.len(), 1024);
/// assert!(!metadata.is_empty());
///
/// // Create metadata for an empty directory
/// let dir_metadata = Metadata::new(FileType::Dir, 0);
/// assert!(dir_metadata.is_dir());
/// assert!(dir_metadata.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct Metadata {
    /// The type of the filesystem entry.
    file_type: FileType,
    /// The size of the file in bytes.
    ///
    /// For directories, this value is typically 0 or represents the
    /// directory's metadata size (platform-dependent).
    len: u64,
}

impl Metadata {
    /// Creates a new `Metadata` instance.
    ///
    /// # Arguments
    ///
    /// * `file_type` - The type of the filesystem entry
    /// * `len` - The size of the file in bytes
    ///
    /// # Returns
    ///
    /// A new `Metadata` instance with the specified file type and length.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::{Metadata, FileType};
    ///
    /// let metadata = Metadata::new(FileType::File, 2048);
    /// assert!(metadata.is_file());
    /// assert_eq!(metadata.len(), 2048);
    /// ```
    #[must_use]
    pub fn new(file_type: FileType, len: u64) -> Self {
        Self { file_type, len }
    }

    /// Returns the size of the file in bytes.
    ///
    /// For regular files, this returns the actual file size. For directories,
    /// the value is platform-dependent and typically represents the directory
    /// metadata size (often 0).
    ///
    /// # Returns
    ///
    /// The size of the file in bytes as a `u64`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::{Metadata, FileType};
    ///
    /// let metadata = Metadata::new(FileType::File, 1024);
    /// assert_eq!(metadata.len(), 1024);
    ///
    /// let empty_file = Metadata::new(FileType::File, 0);
    /// assert_eq!(empty_file.len(), 0);
    /// ```
    #[must_use]
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Returns `true` if the file size is zero.
    ///
    /// This is useful for quickly checking if a file is empty without
    /// reading its contents.
    ///
    /// # Returns
    ///
    /// - `true` if the file size is 0
    /// - `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::{Metadata, FileType};
    ///
    /// let empty = Metadata::new(FileType::File, 0);
    /// assert!(empty.is_empty());
    ///
    /// let not_empty = Metadata::new(FileType::File, 100);
    /// assert!(!not_empty.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the file type.
    ///
    /// # Returns
    ///
    /// The [`FileType`] of this filesystem entry.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::{Metadata, FileType};
    ///
    /// let metadata = Metadata::new(FileType::Dir, 0);
    /// assert_eq!(metadata.file_type(), FileType::Dir);
    /// ```
    #[must_use]
    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    /// Returns `true` if this is a regular file.
    ///
    /// This is a convenience method equivalent to calling
    /// `self.file_type().is_file()`.
    ///
    /// # Returns
    ///
    /// - `true` if the entry is a regular file
    /// - `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::{Metadata, FileType};
    ///
    /// let file_meta = Metadata::new(FileType::File, 1024);
    /// assert!(file_meta.is_file());
    ///
    /// let dir_meta = Metadata::new(FileType::Dir, 0);
    /// assert!(!dir_meta.is_file());
    /// ```
    #[must_use]
    pub fn is_file(&self) -> bool {
        self.file_type.is_file()
    }

    /// Returns `true` if this is a directory.
    ///
    /// This is a convenience method equivalent to calling
    /// `self.file_type().is_dir()`.
    ///
    /// # Returns
    ///
    /// - `true` if the entry is a directory
    /// - `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::{Metadata, FileType};
    ///
    /// let dir_meta = Metadata::new(FileType::Dir, 0);
    /// assert!(dir_meta.is_dir());
    ///
    /// let file_meta = Metadata::new(FileType::File, 1024);
    /// assert!(!file_meta.is_dir());
    /// ```
    #[must_use]
    pub fn is_dir(&self) -> bool {
        self.file_type.is_dir()
    }

    /// Returns `true` if this is a symbolic link.
    ///
    /// This is a convenience method equivalent to calling
    /// `self.file_type().is_symlink()`.
    ///
    /// # Returns
    ///
    /// - `true` if the entry is a symbolic link
    /// - `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::{Metadata, FileType};
    ///
    /// let symlink_meta = Metadata::new(FileType::Symlink, 0);
    /// assert!(symlink_meta.is_symlink());
    ///
    /// let file_meta = Metadata::new(FileType::File, 1024);
    /// assert!(!file_meta.is_symlink());
    /// ```
    #[must_use]
    pub fn is_symlink(&self) -> bool {
        self.file_type.is_symlink()
    }
}

/// Converts from [`std::fs::Metadata`] to [`Metadata`].
///
/// This implementation allows seamless conversion from the standard library's
/// metadata type to this crate's abstraction.
///
/// # Conversion Details
///
/// - `file_type`: Converted using the `From<std::fs::FileType>` implementation
/// - `len`: Obtained from [`std::fs::Metadata::len()`]
///
/// # Example
///
/// ```rust,ignore
/// use workspace_fs::Metadata;
/// use std::fs;
///
/// let std_metadata = fs::metadata("some_file.txt")?;
/// let metadata: Metadata = std_metadata.into();
/// println!("File size: {} bytes", metadata.len());
/// ```
impl From<std::fs::Metadata> for Metadata {
    fn from(meta: std::fs::Metadata) -> Self {
        Self { file_type: meta.file_type().into(), len: meta.len() }
    }
}

// =============================================================================
// DirEntry Struct
// =============================================================================

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Represents an entry in a directory.
///
/// This struct provides a unified representation of directory entries across
/// all platforms. It contains the full path to the entry and its file type,
/// which are the essential pieces of information needed when listing directory
/// contents.
///
/// # Fields
///
/// The struct contains the following private fields, accessible through getter methods:
/// - `path`: The full path to the filesystem entry ([`PathBuf`])
/// - `file_type`: The type of the entry ([`FileType`])
///
/// # Trait Implementations
///
/// - [`Debug`]: Formats the entry for debugging
/// - [`Clone`]: Allows cloning the entry
///
/// # Example
///
/// ```rust,ignore
/// use workspace_fs::{DirEntry, FileType};
/// use std::path::PathBuf;
///
/// // Create a directory entry for a file
/// let entry = DirEntry::new(PathBuf::from("/home/user/file.txt"), FileType::File);
/// assert_eq!(entry.path().to_str(), Some("/home/user/file.txt"));
/// assert_eq!(entry.file_name().to_str(), Some("file.txt"));
/// assert!(entry.file_type().is_file());
///
/// // Create a directory entry for a directory
/// let dir_entry = DirEntry::new(PathBuf::from("/home/user/docs"), FileType::Dir);
/// assert!(dir_entry.file_type().is_dir());
/// ```
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// The full path to this filesystem entry.
    path: PathBuf,
    /// The type of this filesystem entry.
    file_type: FileType,
}

impl DirEntry {
    /// Creates a new `DirEntry` instance.
    ///
    /// # Arguments
    ///
    /// * `path` - The full path to the filesystem entry
    /// * `file_type` - The type of the entry (file, directory, or symlink)
    ///
    /// # Returns
    ///
    /// A new `DirEntry` instance with the specified path and file type.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::{DirEntry, FileType};
    /// use std::path::PathBuf;
    ///
    /// let entry = DirEntry::new(PathBuf::from("/tmp/test.txt"), FileType::File);
    /// assert!(entry.file_type().is_file());
    /// ```
    #[must_use]
    pub fn new(path: PathBuf, file_type: FileType) -> Self {
        Self { path, file_type }
    }

    /// Returns the full path of this entry.
    ///
    /// This returns a reference to the complete path, including all parent
    /// directories and the file name.
    ///
    /// # Returns
    ///
    /// A reference to the [`Path`] of this entry.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::{DirEntry, FileType};
    /// use std::path::PathBuf;
    ///
    /// let entry = DirEntry::new(PathBuf::from("/home/user/docs/file.txt"), FileType::File);
    /// assert_eq!(entry.path().to_str(), Some("/home/user/docs/file.txt"));
    /// ```
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the file name of this entry.
    ///
    /// This extracts just the final component of the path (the file or directory
    /// name without the parent path). For paths that don't have a file name
    /// component (like `/` or `..`), this returns an empty [`OsStr`].
    ///
    /// # Returns
    ///
    /// A reference to the file name as an [`OsStr`]. Returns an empty `OsStr`
    /// if the path has no file name component.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::{DirEntry, FileType};
    /// use std::path::PathBuf;
    ///
    /// let entry = DirEntry::new(PathBuf::from("/home/user/document.pdf"), FileType::File);
    /// assert_eq!(entry.file_name().to_str(), Some("document.pdf"));
    ///
    /// let dir_entry = DirEntry::new(PathBuf::from("/var/log"), FileType::Dir);
    /// assert_eq!(dir_entry.file_name().to_str(), Some("log"));
    /// ```
    #[must_use]
    pub fn file_name(&self) -> &OsStr {
        self.path.file_name().unwrap_or_else(|| OsStr::new(""))
    }

    /// Returns the file type of this entry.
    ///
    /// # Returns
    ///
    /// The [`FileType`] of this entry (file, directory, or symlink).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::{DirEntry, FileType};
    /// use std::path::PathBuf;
    ///
    /// let file_entry = DirEntry::new(PathBuf::from("/tmp/file.txt"), FileType::File);
    /// assert_eq!(file_entry.file_type(), FileType::File);
    /// assert!(file_entry.file_type().is_file());
    ///
    /// let dir_entry = DirEntry::new(PathBuf::from("/tmp/subdir"), FileType::Dir);
    /// assert_eq!(dir_entry.file_type(), FileType::Dir);
    /// assert!(dir_entry.file_type().is_dir());
    /// ```
    #[must_use]
    pub fn file_type(&self) -> FileType {
        self.file_type
    }
}
