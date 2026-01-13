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
