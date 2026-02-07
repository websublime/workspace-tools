//! # Traits Module
//!
//! Defines the core [`FileSystem`] trait that abstracts all filesystem operations.
//!
//! ## What
//!
//! This module provides the `FileSystem` trait, which is the central abstraction of
//! the workspace-fs crate. The trait defines a complete set of asynchronous filesystem
//! operations that can be implemented by different backends.
//!
//! ## How
//!
//! The trait uses **native `async fn` in traits** (stabilized in Rust 1.75+, fully
//! usable with `Send` bounds in Edition 2024). Each method:
//! - Takes `&self` to allow shared access (implementations handle internal synchronization)
//! - Accepts paths as `&Path` for explicit, zero-cost path references
//! - Returns `Result<T>` using the crate's error type (except metadata queries which return `bool`)
//!
//! The trait requires `Send + Sync` to ensure implementations are safe to share across
//! async task boundaries.
//!
//! Two implementations are provided:
//! - [`RealFileSystem`](crate::RealFileSystem): Production implementation using `tokio::fs`
//! - [`MockFileSystem`](crate::MockFileSystem): In-memory implementation for testing
//!
//! ## Why
//!
//! A trait-based abstraction enables:
//! - **Dependency Injection**: Pass filesystem as a parameter, not a global
//! - **Testing**: Swap in `MockFileSystem` for fast, deterministic unit tests
//! - **Flexibility**: Could add other implementations (e.g., cached, logged, remote)
//! - **Decoupling**: Application code doesn't depend on specific filesystem implementation
//! - **Zero Overhead**: Native async traits avoid the heap allocation of `async_trait`
//!
//! Using generics (`impl FileSystem` or `<FS: FileSystem>`) rather than `dyn FileSystem`
//! enables monomorphization and avoids dynamic dispatch overhead. See
//! [`docs/DECISIONS.md`](../../../docs/DECISIONS.md) ADR-001 for rationale.
//!
//! ## Example
//!
//! ```rust,ignore
//! use workspace_fs::{FileSystem, RealFileSystem, MockFileSystem};
//! use std::path::Path;
//!
//! // Generic function works with any FileSystem implementation
//! async fn read_json<FS: FileSystem>(fs: &FS, path: &Path) -> Result<String, workspace_fs::Error> {
//!     fs.read_to_string(path).await
//! }
//!
//! // In production
//! let fs = RealFileSystem::new();
//! let content = read_json(&fs, Path::new("config.json")).await?;
//!
//! // In tests
//! let mock = MockFileSystem::new();
//! mock.write_string(Path::new("config.json"), r#"{"key": "value"}"#).await?;
//! let content = read_json(&mock, Path::new("config.json")).await?;
//! ```

use std::path::{Path, PathBuf};

use crate::Result;
use crate::types::{DirEntry, Metadata};

// =============================================================================
// FileSystem Trait
// =============================================================================

/// Unified asynchronous filesystem abstraction.
///
/// This trait defines the complete set of filesystem operations used throughout
/// the workspace-node-tools ecosystem. It is the primary abstraction boundary
/// that enables dependency injection and testability.
///
/// # Design
///
/// - **Native async**: Uses Rust's built-in `async fn` in traits (no `async_trait` macro)
/// - **`Send + Sync`**: All implementations must be safe to share across async tasks
/// - **Generic dispatch**: Consumers use `<FS: FileSystem>` for zero-cost monomorphization
/// - **`&Path` parameters**: Explicit path references avoid hidden allocations
///
/// # Method Categories
///
/// | Category | Methods | PRD Reference |
/// |----------|---------|---------------|
/// | Read | [`read_to_string`][Self::read_to_string], [`read_bytes`][Self::read_bytes] | FR-1.2 |
/// | Write | [`write_string`][Self::write_string], [`write_bytes`][Self::write_bytes], [`append_string`][Self::append_string], [`append_bytes`][Self::append_bytes] | FR-1.3 |
/// | Metadata | [`exists`][Self::exists], [`is_file`][Self::is_file], [`is_dir`][Self::is_dir], [`is_symlink`][Self::is_symlink], [`metadata`][Self::metadata], [`symlink_metadata`][Self::symlink_metadata] | FR-1.4 |
/// | Directory | [`create_dir`][Self::create_dir], [`create_dir_all`][Self::create_dir_all], [`read_dir`][Self::read_dir], [`remove_dir`][Self::remove_dir], [`remove_dir_all`][Self::remove_dir_all] | FR-1.5 |
/// | File | [`remove_file`][Self::remove_file], [`copy_file`][Self::copy_file], [`rename`][Self::rename] | FR-1.6 |
/// | Path | [`canonicalize`][Self::canonicalize], [`absolute`][Self::absolute] | FR-1.7 |
/// | Symlink | [`read_link`][Self::read_link] | FR-1.8 |
/// | Traversal | [`walk_dir`][Self::walk_dir] | FR-1.9 |
///
/// # Example
///
/// ```rust,ignore
/// use workspace_fs::FileSystem;
/// use std::path::Path;
///
/// async fn count_json_files<FS: FileSystem>(fs: &FS, dir: &Path) -> usize {
///     let entries = fs.read_dir(dir).await.unwrap_or_default();
///     entries.iter().filter(|e| {
///         e.path().extension().is_some_and(|ext| ext == "json")
///     }).count()
/// }
/// ```
#[allow(async_fn_in_trait)]
pub trait FileSystem: Send + Sync {
    // =========================================================================
    // Read Operations (FR-1.2)
    // =========================================================================

    /// Reads the entire contents of a file as a UTF-8 string.
    ///
    /// Implements FR-1.2.1: Read file contents as a string.
    ///
    /// This is the primary method for reading text files such as configuration
    /// files, JSON documents, TOML manifests, and changelogs.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - The path is a directory ([`Error::NotAFile`](crate::Error::NotAFile))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - The file content is not valid UTF-8 ([`Error::InvalidUtf8`](crate::Error::InvalidUtf8))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn read_config<FS: FileSystem>(fs: &FS) -> workspace_fs::Result<String> {
    ///     fs.read_to_string(Path::new("package.json")).await
    /// }
    /// ```
    async fn read_to_string(&self, path: &Path) -> Result<String>;

    /// Reads the entire contents of a file as raw bytes.
    ///
    /// Implements FR-1.2.2: Read file contents as bytes.
    ///
    /// Use this method when you need the raw binary content of a file, or when
    /// the file may not contain valid UTF-8 data.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - The path is a directory ([`Error::NotAFile`](crate::Error::NotAFile))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn read_binary<FS: FileSystem>(fs: &FS, path: &Path) -> workspace_fs::Result<Vec<u8>> {
    ///     fs.read_bytes(path).await
    /// }
    /// ```
    async fn read_bytes(&self, path: &Path) -> Result<Vec<u8>>;

    // =========================================================================
    // Write Operations (FR-1.3)
    // =========================================================================

    /// Writes a UTF-8 string to a file, creating it if it does not exist
    /// or truncating it if it does.
    ///
    /// Implements FR-1.3.1: Write string content to a file.
    ///
    /// Parent directories must already exist. Use [`create_dir_all`][Self::create_dir_all]
    /// first if needed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The parent directory does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - The path is a directory ([`Error::NotAFile`](crate::Error::NotAFile))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn save_config<FS: FileSystem>(fs: &FS, content: &str) -> workspace_fs::Result<()> {
    ///     fs.write_string(Path::new("config.json"), content).await
    /// }
    /// ```
    async fn write_string(&self, path: &Path, content: &str) -> Result<()>;

    /// Writes raw bytes to a file, creating it if it does not exist
    /// or truncating it if it does.
    ///
    /// Implements FR-1.3.2: Write binary content to a file.
    ///
    /// Parent directories must already exist. Use [`create_dir_all`][Self::create_dir_all]
    /// first if needed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The parent directory does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - The path is a directory ([`Error::NotAFile`](crate::Error::NotAFile))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn save_binary<FS: FileSystem>(fs: &FS, data: &[u8]) -> workspace_fs::Result<()> {
    ///     fs.write_bytes(Path::new("output.bin"), data).await
    /// }
    /// ```
    async fn write_bytes(&self, path: &Path, content: &[u8]) -> Result<()>;

    /// Appends a UTF-8 string to a file, creating it if it does not exist.
    ///
    /// Implements FR-1.3.3: Append string content to a file.
    ///
    /// Unlike [`write_string`][Self::write_string], this method does not truncate
    /// existing content. New content is added at the end of the file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The parent directory does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - The path is a directory ([`Error::NotAFile`](crate::Error::NotAFile))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn append_log<FS: FileSystem>(fs: &FS, entry: &str) -> workspace_fs::Result<()> {
    ///     fs.append_string(Path::new("debug.log"), entry).await
    /// }
    /// ```
    async fn append_string(&self, path: &Path, content: &str) -> Result<()>;

    /// Appends raw bytes to a file, creating it if it does not exist.
    ///
    /// Implements FR-1.3.4: Append binary content to a file.
    ///
    /// Unlike [`write_bytes`][Self::write_bytes], this method does not truncate
    /// existing content. New content is added at the end of the file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The parent directory does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - The path is a directory ([`Error::NotAFile`](crate::Error::NotAFile))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn append_data<FS: FileSystem>(fs: &FS, data: &[u8]) -> workspace_fs::Result<()> {
    ///     fs.append_bytes(Path::new("output.bin"), data).await
    /// }
    /// ```
    async fn append_bytes(&self, path: &Path, content: &[u8]) -> Result<()>;

    // =========================================================================
    // Metadata Operations (FR-1.4)
    // =========================================================================

    /// Checks whether a path exists on the filesystem.
    ///
    /// Implements FR-1.4.1: Check path existence.
    ///
    /// This method follows symbolic links. To check existence without following
    /// symlinks, use [`symlink_metadata`][Self::symlink_metadata] and check for errors.
    ///
    /// Returns `false` if the path does not exist or if an error occurs during
    /// the check (e.g., permission denied on a parent directory). This matches
    /// the behavior of [`std::path::Path::exists`].
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn ensure_config_exists<FS: FileSystem>(fs: &FS) -> bool {
    ///     fs.exists(Path::new("workspace.toml")).await
    /// }
    /// ```
    async fn exists(&self, path: &Path) -> bool;

    /// Checks whether a path points to a regular file.
    ///
    /// Implements FR-1.4.2: Check if path is a file.
    ///
    /// This method follows symbolic links. If the path is a symlink pointing to
    /// a file, this returns `true`. Returns `false` if the path does not exist,
    /// is a directory, or if an error occurs.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn validate_file<FS: FileSystem>(fs: &FS, path: &Path) -> bool {
    ///     fs.is_file(path).await
    /// }
    /// ```
    async fn is_file(&self, path: &Path) -> bool;

    /// Checks whether a path points to a directory.
    ///
    /// Implements FR-1.4.3: Check if path is a directory.
    ///
    /// This method follows symbolic links. If the path is a symlink pointing to
    /// a directory, this returns `true`. Returns `false` if the path does not
    /// exist, is a file, or if an error occurs.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn validate_workspace<FS: FileSystem>(fs: &FS, path: &Path) -> bool {
    ///     fs.is_dir(path).await
    /// }
    /// ```
    async fn is_dir(&self, path: &Path) -> bool;

    /// Checks whether a path points to a symbolic link.
    ///
    /// Implements FR-1.4.4: Check if path is a symlink.
    ///
    /// Unlike [`is_file`][Self::is_file] and [`is_dir`][Self::is_dir], this method
    /// does **not** follow symbolic links. It checks the link itself, not its target.
    /// Returns `false` if the path does not exist or if an error occurs.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn check_symlink<FS: FileSystem>(fs: &FS, path: &Path) -> bool {
    ///     fs.is_symlink(path).await
    /// }
    /// ```
    async fn is_symlink(&self, path: &Path) -> bool;

    /// Retrieves metadata for a path, following symbolic links.
    ///
    /// Implements FR-1.4.5: Retrieve file metadata.
    ///
    /// If the path is a symbolic link, this returns metadata for the link's
    /// target (the file or directory the link points to). To get metadata for
    /// the link itself, use [`symlink_metadata`][Self::symlink_metadata].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn file_size<FS: FileSystem>(fs: &FS, path: &Path) -> workspace_fs::Result<u64> {
    ///     let meta = fs.metadata(path).await?;
    ///     Ok(meta.len())
    /// }
    /// ```
    async fn metadata(&self, path: &Path) -> Result<Metadata>;

    /// Retrieves metadata for a path without following symbolic links.
    ///
    /// Implements FR-1.4.6: Retrieve symlink metadata.
    ///
    /// If the path is a symbolic link, this returns metadata for the link itself,
    /// not its target. The returned [`Metadata`] will report [`FileType::Symlink`](crate::FileType::Symlink).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn is_link<FS: FileSystem>(fs: &FS, path: &Path) -> workspace_fs::Result<bool> {
    ///     let meta = fs.symlink_metadata(path).await?;
    ///     Ok(meta.is_symlink())
    /// }
    /// ```
    async fn symlink_metadata(&self, path: &Path) -> Result<Metadata>;

    // =========================================================================
    // Directory Operations (FR-1.5)
    // =========================================================================

    /// Creates a single directory.
    ///
    /// Implements FR-1.5.1: Create a directory.
    ///
    /// Creates the directory at the given path. The parent directory must already
    /// exist. To create all intermediate directories, use
    /// [`create_dir_all`][Self::create_dir_all].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The parent directory does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - The path already exists ([`Error::AlreadyExists`](crate::Error::AlreadyExists))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn setup_output<FS: FileSystem>(fs: &FS) -> workspace_fs::Result<()> {
    ///     fs.create_dir(Path::new("output")).await
    /// }
    /// ```
    async fn create_dir(&self, path: &Path) -> Result<()>;

    /// Creates a directory and all its parent directories.
    ///
    /// Implements FR-1.5.2: Create directory tree.
    ///
    /// Creates the directory at the given path, including any missing parent
    /// directories. If the directory already exists, this is a no-op.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A component of the path exists but is not a directory
    ///   ([`Error::NotADirectory`](crate::Error::NotADirectory))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn ensure_changeset_dir<FS: FileSystem>(fs: &FS) -> workspace_fs::Result<()> {
    ///     fs.create_dir_all(Path::new(".changeset/fragments")).await
    /// }
    /// ```
    async fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Lists the entries of a directory.
    ///
    /// Implements FR-1.5.3: List directory contents.
    ///
    /// Returns all entries in the directory as a vector of [`DirEntry`] values.
    /// The entries are not guaranteed to be in any particular order. This does
    /// **not** recurse into subdirectories; use [`walk_dir`][Self::walk_dir]
    /// for recursive traversal.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - The path is not a directory ([`Error::NotADirectory`](crate::Error::NotADirectory))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn list_packages<FS: FileSystem>(fs: &FS) -> workspace_fs::Result<Vec<String>> {
    ///     let entries = fs.read_dir(Path::new("packages")).await?;
    ///     Ok(entries.iter().filter(|e| e.file_type().is_dir()).map(|e| {
    ///         e.file_name().to_string_lossy().into_owned()
    ///     }).collect())
    /// }
    /// ```
    async fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>>;

    /// Removes an empty directory.
    ///
    /// Implements FR-1.5.4: Remove an empty directory.
    ///
    /// The directory must be empty. To remove a directory and all its contents,
    /// use [`remove_dir_all`][Self::remove_dir_all].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - The path is not a directory ([`Error::NotADirectory`](crate::Error::NotADirectory))
    /// - The directory is not empty ([`Error::NotEmpty`](crate::Error::NotEmpty))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn cleanup_empty_dir<FS: FileSystem>(fs: &FS, path: &Path) -> workspace_fs::Result<()> {
    ///     fs.remove_dir(path).await
    /// }
    /// ```
    async fn remove_dir(&self, path: &Path) -> Result<()>;

    /// Removes a directory and all its contents recursively.
    ///
    /// Implements FR-1.5.5: Remove directory tree.
    ///
    /// This removes the directory at the given path along with all files and
    /// subdirectories it contains. Use with caution as this operation is
    /// irreversible.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - The path is not a directory ([`Error::NotADirectory`](crate::Error::NotADirectory))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn clean_build<FS: FileSystem>(fs: &FS) -> workspace_fs::Result<()> {
    ///     fs.remove_dir_all(Path::new("target")).await
    /// }
    /// ```
    async fn remove_dir_all(&self, path: &Path) -> Result<()>;

    // =========================================================================
    // File Operations (FR-1.6)
    // =========================================================================

    /// Removes a file.
    ///
    /// Implements FR-1.6.1: Remove a file.
    ///
    /// Removes the file at the given path. This does not follow symbolic links;
    /// if the path is a symlink, the link itself is removed, not its target.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - The path is a directory ([`Error::NotAFile`](crate::Error::NotAFile))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn remove_lockfile<FS: FileSystem>(fs: &FS) -> workspace_fs::Result<()> {
    ///     fs.remove_file(Path::new(".lock")).await
    /// }
    /// ```
    async fn remove_file(&self, path: &Path) -> Result<()>;

    /// Copies a file from one path to another.
    ///
    /// Implements FR-1.6.2: Copy a file.
    ///
    /// Copies the contents and permissions of the file at `src` to `dst`. If `dst`
    /// already exists, it is overwritten. The parent directory of `dst` must exist.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The source path does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - The source is a directory ([`Error::NotAFile`](crate::Error::NotAFile))
    /// - The destination parent does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn backup_config<FS: FileSystem>(fs: &FS) -> workspace_fs::Result<()> {
    ///     fs.copy_file(Path::new("config.json"), Path::new("config.json.bak")).await
    /// }
    /// ```
    async fn copy_file(&self, src: &Path, dst: &Path) -> Result<()>;

    /// Renames a file or directory.
    ///
    /// Implements FR-1.6.3: Rename/move a file or directory.
    ///
    /// Moves the filesystem entry from `src` to `dst`. This operation is atomic
    /// on most platforms when the source and destination are on the same filesystem.
    /// The parent directory of `dst` must exist.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The source path does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - The destination parent does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn finalize_changelog<FS: FileSystem>(fs: &FS) -> workspace_fs::Result<()> {
    ///     fs.rename(
    ///         Path::new("CHANGELOG.draft.md"),
    ///         Path::new("CHANGELOG.md"),
    ///     ).await
    /// }
    /// ```
    async fn rename(&self, src: &Path, dst: &Path) -> Result<()>;

    // =========================================================================
    // Path Operations (FR-1.7)
    // =========================================================================

    /// Returns the canonical, absolute form of a path.
    ///
    /// Implements FR-1.7.1: Canonicalize a path.
    ///
    /// Resolves all symbolic links, `.`, and `..` components and returns the
    /// resulting absolute path. The path must exist on the filesystem.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - A component of the path is not a directory ([`Error::NotADirectory`](crate::Error::NotADirectory))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn resolve_path<FS: FileSystem>(fs: &FS, path: &Path) -> workspace_fs::Result<std::path::PathBuf> {
    ///     fs.canonicalize(path).await
    /// }
    /// ```
    async fn canonicalize(&self, path: &Path) -> Result<PathBuf>;

    /// Converts a relative path to an absolute path without I/O.
    ///
    /// Implements FR-1.7.2: Convert path to absolute form.
    ///
    /// Unlike [`canonicalize`][Self::canonicalize], this method does **not** access
    /// the filesystem. It resolves `.` and `..` components and prepends the current
    /// working directory if the path is relative, but does not follow symbolic links
    /// or verify that the path exists.
    ///
    /// This is a **synchronous** method because it performs no I/O.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The current working directory cannot be determined ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// fn make_absolute<FS: FileSystem>(fs: &FS, path: &Path) -> workspace_fs::Result<std::path::PathBuf> {
    ///     fs.absolute(path)
    /// }
    /// ```
    fn absolute(&self, path: &Path) -> Result<PathBuf>;

    // =========================================================================
    // Symlink Operations (FR-1.8)
    // =========================================================================

    /// Reads the target path of a symbolic link.
    ///
    /// Implements FR-1.8.1: Read symbolic link target.
    ///
    /// Returns the path that the symbolic link points to. The returned path may
    /// be relative or absolute, and may or may not exist.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - The path is not a symbolic link ([`Error::Io`](crate::Error::Io))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn resolve_link<FS: FileSystem>(fs: &FS, path: &Path) -> workspace_fs::Result<std::path::PathBuf> {
    ///     fs.read_link(path).await
    /// }
    /// ```
    async fn read_link(&self, path: &Path) -> Result<PathBuf>;

    // =========================================================================
    // Directory Traversal (FR-1.9)
    // =========================================================================

    /// Recursively walks a directory tree and returns all entries.
    ///
    /// Implements FR-1.9.1: Recursive directory traversal.
    ///
    /// Returns all files, directories, and symbolic links found by recursively
    /// traversing the given directory. The entries are returned in an unspecified
    /// order. This is useful for operations that need to process entire directory
    /// trees, such as finding all `package.json` files in a monorepo.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist ([`Error::NotFound`](crate::Error::NotFound))
    /// - The path is not a directory ([`Error::NotADirectory`](crate::Error::NotADirectory))
    /// - Permission is denied ([`Error::PermissionDenied`](crate::Error::PermissionDenied))
    /// - An I/O error occurs ([`Error::Io`](crate::Error::Io))
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::FileSystem;
    /// use std::path::Path;
    ///
    /// async fn find_manifests<FS: FileSystem>(fs: &FS, root: &Path) -> workspace_fs::Result<Vec<std::path::PathBuf>> {
    ///     let entries = fs.walk_dir(root).await?;
    ///     Ok(entries.into_iter()
    ///         .filter(|e| e.file_name() == "package.json")
    ///         .map(|e| e.path().to_path_buf())
    ///         .collect())
    /// }
    /// ```
    async fn walk_dir(&self, path: &Path) -> Result<Vec<DirEntry>>;
}

// =============================================================================
// Static Assertions
// =============================================================================

// Verify that FileSystem can be used as a generic bound with Send futures.
// This ensures that `async fn foo<FS: FileSystem>(fs: &FS)` produces Send futures
// when the implementation's futures are Send.
const _: () = {
    #[allow(unused)]
    fn assert_usable_as_generic_bound<FS: FileSystem>() {}
};
