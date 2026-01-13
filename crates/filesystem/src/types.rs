//! # Types Module
//!
//! Core data types for representing filesystem entries and their metadata.
//!
//! ## What
//!
//! This module provides three fundamental types used throughout the crate:
//! - [`FileType`]: Discriminates between files, directories, and symlinks
//! - [`DirEntry`]: Represents a single entry when listing directory contents
//! - [`Metadata`]: Provides file/directory metadata (size, timestamps, permissions)
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
//! use workspace_fs::{DirEntry, FileType, Metadata};
//!
//! async fn list_files(fs: &impl FileSystem, dir: &Path) -> Result<Vec<DirEntry>> {
//!     let entries = fs.read_dir(dir).await?;
//!     Ok(entries.into_iter()
//!         .filter(|e| e.file_type() == FileType::File)
//!         .collect())
//! }
//! ```

// TODO: will be implemented on epic workspace-node-tools-3q8 (Types Module)
#![allow(clippy::todo)]
