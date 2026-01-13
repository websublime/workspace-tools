# Implementation Plan: workspace-fs

## Document Information

| Field | Value |
|-------|-------|
| **Crate Name** | `workspace-fs` |
| **PRD Reference** | [PRD.md](./PRD.md) |
| **Status** | Ready |
| **Created** | 2026-01-13 |
| **Last Updated** | 2026-01-13 |

---

## 1. Overview

This document details the implementation plan for the `workspace-fs` crate. The implementation is divided into epics, each containing multiple tasks. Tasks are designed to be atomic units of work that can be completed and committed independently.

### 1.1 Key Architectural Decisions

The following decisions were finalized during PRD development and must be followed during implementation:

| Decision | Choice | PRD Reference |
|----------|--------|---------------|
| I/O Model | Async-first with `tokio::fs` | §1.2, §2.2 |
| Trait Definition | `#[async_trait]` for object-safety | §5.1 FR-1.1.2 |
| Timeout Handling | Configurable per-operation timeouts | §5.2 |
| Error Handling | Single unified `Error` enum using `snafu` | §5.6 |
| FileType | Custom enum (`File`, `Dir`, `Symlink`) | §5.5 FR-5.2 |
| MockFileSystem | `Arc<RwLock<State>>` with `deep_clone()` option | §5.4 FR-4.1.5, FR-4.1.6 |
| PathExt | Sync-only utilities, no I/O | §5.7 |

### 1.2 External Dependencies

From PRD §1.4.2:

| Crate | Version | Purpose |
|-------|---------|---------|
| `snafu` | `0.8.9` | Error handling with context |
| `tokio` | `1.49.0` | Async runtime and filesystem operations |
| `async-trait` | `0.1.89` | Async trait support |
| `log` | `0.4` | Logging facade |

### 1.3 Standard Acceptance Criteria

**All tasks MUST meet these criteria before completion:**

- [ ] **Clippy**: `cargo clippy` passes with zero warnings
- [ ] **Fmt**: `cargo fmt --check` passes
- [ ] **Docs**: All public items documented, `cargo doc` generates without warnings
- [ ] **Tests**: Unit tests written and passing (`cargo test`)
- [ ] **Build**: `cargo build` succeeds
- [ ] **Review**: Request implementation review in a new session for robust code and quality solution

---

## 2. Implementation Phases

### Phase 0: Project Setup
Foundation and scaffolding for the crate.

### Phase 1: Error Module
Single unified error type using `snafu` (PRD §5.6).

### Phase 2: Types Module
Core types: `FileType`, `DirEntry`, `Metadata` (PRD §5.5).

### Phase 3: Configuration Module
`FileSystemConfig` with builder pattern (PRD §5.2).

### Phase 4: PathExt Module
Path utility trait for synchronous operations (PRD §5.7).

### Phase 5: FileSystem Trait
Async trait definition with all operations (PRD §5.1).

### Phase 6: RealFileSystem Implementation
Production implementation using `tokio::fs` (PRD §5.3).

### Phase 7: MockFileSystem Implementation
In-memory implementation for testing (PRD §5.4).

### Phase 8: Integration & Polish
End-to-end tests, documentation review, and optimization.

---

## 3. Epic Breakdown

---

### Epic 0: Project Setup

**Goal**: Establish the crate structure, dependencies, and development configuration.

**PRD Context**: §1.4 Dependencies, §6.3 Compatibility, §6.4 Code Quality

---

#### Task 0.1: Create Crate Skeleton

**Description**: Initialize the crate with `Cargo.toml` and basic structure following the external dependencies defined in PRD §1.4.2.

**PRD References**:
- §1.4.2: External dependencies list (snafu, tokio, async-trait, log)
- §1.4.3: Development dependencies (tempfile, tokio test features)
- §6.3: Compatibility requirements
- §6.4: Code quality requirements (clippy, docs)

**Acceptance Criteria**:
- [ ] `Cargo.toml` created with proper metadata and all dependencies from PRD §1.4
- [ ] `src/lib.rs` created with clippy lints and crate-level documentation
- [ ] Crate compiles with `cargo check`
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Implementation Details**:

```toml
# Cargo.toml
[package]
name = "workspace-fs"
version = "0.1.0"
edition = "2024"
rust-version = "1.90"
description = "Async filesystem abstraction for workspace-node-tools"
license = "MIT"
repository = "https://github.com/user/workspace-node-tools"
keywords = ["filesystem", "async", "tokio", "mock"]
categories = ["filesystem", "asynchronous"]

[lints.rust]
missing_docs = "warn"
rustdoc-missing-crate-level-docs = "warn"
unused_must_use = "deny"

[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
todo = "deny"
unimplemented = "deny"
panic = "deny"

[dependencies]
# Error handling (PRD §1.4.2)
snafu = "0.8.9"

# Async runtime and filesystem (PRD §1.4.2)
tokio = { version = "1.49.0", features = ["fs", "sync", "time"] }
async-trait = "0.1.89"

# Logging (PRD §1.4.2)
log = "0.4"

[dev-dependencies]
# PRD §1.4.3
tempfile = "3.24.0"
tokio = { version = "1.49.0", features = ["rt-multi-thread", "macros"] }
```

**Files to Create**:
- `crates/filesystem/Cargo.toml`
- `crates/filesystem/src/lib.rs`

**Estimated Effort**: 1 hour

---

#### Task 0.2: Create Module Structure

**Description**: Create the module files matching the architecture defined in PRD §7.1.

**PRD References**:
- §7.1: Module Structure (complete file layout)
- §7.2: Dependency Graph (module dependencies)

**Acceptance Criteria**:
- [ ] All module files created matching PRD §7.1
- [ ] `lib.rs` declares all modules
- [ ] Crate compiles with empty modules
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Module Structure** (from PRD §7.1):

```
src/
├── lib.rs                  # Crate root, re-exports
├── error.rs                # Error enum and Result alias
├── config.rs               # FileSystemConfig and FileSystemConfigBuilder
├── types.rs                # DirEntry, FileType, Metadata
├── traits.rs               # FileSystem trait definition with async_trait
├── path_ext.rs             # PathExt trait for Path utilities
├── real.rs                 # RealFileSystem implementation (tokio::fs)
├── mock.rs                 # MockFileSystem implementation
└── tests.rs                # Unit tests
```

**Files to Create**: All files as listed above with minimal placeholder content.

**Estimated Effort**: 30 minutes

---

### Epic 1: Error Module

**Goal**: Implement a single unified `Error` enum using `snafu` with all error variants.

**PRD Context**: 
- §5.6: Error Handling requirements
- FR-6.1.1 through FR-6.2.9: Specific error variants

---

#### Task 1.1: Define Error Enum

**Description**: Create the unified `Error` enum with all variants defined in PRD §5.6.

**PRD References**:
- §5.6 FR-6.1.1: Single unified `Error` enum
- §5.6 FR-6.1.2: Use `snafu` for context and chaining
- §5.6 FR-6.1.3: Implement `std::error::Error`
- §5.6 FR-6.1.4: `Send + Sync`
- §5.6 FR-6.2.1-FR-6.2.9: All error variants

**Acceptance Criteria**:
- [ ] `Error` enum created with all variants from PRD §5.6 FR-6.2
- [ ] `Result<T>` type alias created
- [ ] Error implements `std::error::Error`, `Send`, `Sync`
- [ ] Each variant has descriptive `#[snafu(display(...))]` message
- [ ] Unit tests for error display messages
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Implementation Details**:

```rust
use snafu::Snafu;
use std::path::PathBuf;
use std::time::Duration;

/// Type alias for Results using the crate's Error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Unified error type for all filesystem operations.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    /// Path does not exist.
    #[snafu(display("path not found: {}", path.display()))]
    NotFound {
        path: PathBuf,
    },

    /// Insufficient permissions to access path.
    #[snafu(display("permission denied: {}", path.display()))]
    PermissionDenied {
        path: PathBuf,
    },

    /// Path already exists when it shouldn't.
    #[snafu(display("path already exists: {}", path.display()))]
    AlreadyExists {
        path: PathBuf,
    },

    /// Expected a file but found a directory.
    #[snafu(display("expected file, found directory: {}", path.display()))]
    NotAFile {
        path: PathBuf,
    },

    /// Expected a directory but found a file.
    #[snafu(display("expected directory, found file: {}", path.display()))]
    NotADirectory {
        path: PathBuf,
    },

    /// Directory is not empty.
    #[snafu(display("directory not empty: {}", path.display()))]
    NotEmpty {
        path: PathBuf,
    },

    /// File content is not valid UTF-8.
    #[snafu(display("invalid UTF-8 content in file: {}", path.display()))]
    InvalidUtf8 {
        path: PathBuf,
    },

    /// Wrapped I/O error with context.
    #[snafu(display("{} failed for '{}': {}", operation, path.display(), source))]
    Io {
        path: PathBuf,
        operation: String,
        source: std::io::Error,
    },

    /// Operation timed out.
    #[snafu(display("{} timed out after {:?} for '{}'", operation, duration, path.display()))]
    Timeout {
        path: PathBuf,
        operation: String,
        duration: Duration,
    },
}
```

**Estimated Effort**: 1 hour

---

### Epic 2: Types Module

**Goal**: Implement core types: `FileType`, `DirEntry`, and `Metadata`.

**PRD Context**: §5.5 Entry Types

---

#### Task 2.1: Define FileType Enum

**Description**: Create the `FileType` enum with variants for files, directories, and symlinks.

**PRD References**:
- §5.5 FR-5.2.1: Enum with `File`, `Dir`, `Symlink` variants
- §5.5 FR-5.2.2-FR-5.2.4: `is_file()`, `is_dir()`, `is_symlink()` methods
- §5.5 FR-5.2.5: Implement `Debug + Clone + Copy + PartialEq + Eq`

**Acceptance Criteria**:
- [ ] `FileType` enum created with `File`, `Dir`, `Symlink` variants
- [ ] `is_file()`, `is_dir()`, `is_symlink()` methods implemented
- [ ] Derives: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`
- [ ] Conversion from `std::fs::FileType` implemented
- [ ] Unit tests for all methods
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Implementation Details**:

```rust
/// Represents the type of a filesystem entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// Regular file.
    File,
    /// Directory.
    Dir,
    /// Symbolic link.
    Symlink,
}

impl FileType {
    /// Returns `true` if this is a regular file.
    #[must_use]
    pub fn is_file(self) -> bool {
        matches!(self, Self::File)
    }

    /// Returns `true` if this is a directory.
    #[must_use]
    pub fn is_dir(self) -> bool {
        matches!(self, Self::Dir)
    }

    /// Returns `true` if this is a symbolic link.
    #[must_use]
    pub fn is_symlink(self) -> bool {
        matches!(self, Self::Symlink)
    }
}

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
```

**Estimated Effort**: 30 minutes

---

#### Task 2.2: Define Metadata Struct

**Description**: Create the `Metadata` struct for file metadata abstraction.

**PRD References**:
- §5.5 FR-5.3.1-FR-5.3.5: `len()`, `is_file()`, `is_dir()`, `is_symlink()`, `file_type()` methods
- §5.5 FR-5.3.6: Implement `Debug + Clone`

**Acceptance Criteria**:
- [ ] `Metadata` struct created with necessary fields
- [ ] All methods from PRD §5.5 FR-5.3 implemented
- [ ] Derives: `Debug`, `Clone`
- [ ] Conversion from `std::fs::Metadata` implemented
- [ ] Unit tests for all methods
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Implementation Details**:

```rust
/// Metadata information about a filesystem entry.
#[derive(Debug, Clone)]
pub struct Metadata {
    file_type: FileType,
    len: u64,
}

impl Metadata {
    /// Creates a new `Metadata` instance.
    #[must_use]
    pub fn new(file_type: FileType, len: u64) -> Self {
        Self { file_type, len }
    }

    /// Returns the size of the file in bytes.
    #[must_use]
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Returns `true` if the file size is zero.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the file type.
    #[must_use]
    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    /// Returns `true` if this is a regular file.
    #[must_use]
    pub fn is_file(&self) -> bool {
        self.file_type.is_file()
    }

    /// Returns `true` if this is a directory.
    #[must_use]
    pub fn is_dir(&self) -> bool {
        self.file_type.is_dir()
    }

    /// Returns `true` if this is a symbolic link.
    #[must_use]
    pub fn is_symlink(&self) -> bool {
        self.file_type.is_symlink()
    }
}
```

**Estimated Effort**: 30 minutes

---

#### Task 2.3: Define DirEntry Struct

**Description**: Create the `DirEntry` struct for directory entry abstraction.

**PRD References**:
- §5.5 FR-5.1.1-FR-5.1.3: `path()`, `file_name()`, `file_type()` methods
- §5.5 FR-5.1.4: Implement `Debug + Clone`

**Acceptance Criteria**:
- [ ] `DirEntry` struct created with necessary fields
- [ ] All methods from PRD §5.5 FR-5.1 implemented
- [ ] Derives: `Debug`, `Clone`
- [ ] Unit tests for all methods
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Implementation Details**:

```rust
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Represents an entry in a directory.
#[derive(Debug, Clone)]
pub struct DirEntry {
    path: PathBuf,
    file_type: FileType,
}

impl DirEntry {
    /// Creates a new `DirEntry` instance.
    #[must_use]
    pub fn new(path: PathBuf, file_type: FileType) -> Self {
        Self { path, file_type }
    }

    /// Returns the full path of this entry.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the file name of this entry.
    #[must_use]
    pub fn file_name(&self) -> &OsStr {
        self.path.file_name().unwrap_or_else(|| OsStr::new(""))
    }

    /// Returns the file type of this entry.
    #[must_use]
    pub fn file_type(&self) -> FileType {
        self.file_type
    }
}
```

**Estimated Effort**: 30 minutes

---

### Epic 3: Configuration Module

**Goal**: Implement `FileSystemConfig` with builder pattern for timeout configuration.

**PRD Context**: §5.2 Configuration

---

#### Task 3.1: Define FileSystemConfig Struct

**Description**: Create the `FileSystemConfig` struct with timeout fields and default values.

**PRD References**:
- §5.2 FR-2.1.1-FR-2.1.5: Config struct requirements
- §5.2 FR-2.2.1-FR-2.2.3: Default timeout values

**Acceptance Criteria**:
- [ ] `FileSystemConfig` struct created with `read_timeout`, `write_timeout`, `operation_timeout`
- [ ] `Default` implementation with values from PRD §5.2 FR-2.2
- [ ] Implements `Debug + Clone`
- [ ] Getter methods for all fields
- [ ] Unit tests for default values
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Implementation Details**:

```rust
use std::time::Duration;

/// Configuration for filesystem operations.
#[derive(Debug, Clone)]
pub struct FileSystemConfig {
    read_timeout: Duration,
    write_timeout: Duration,
    operation_timeout: Duration,
}

impl FileSystemConfig {
    /// Returns a builder for creating a custom configuration.
    #[must_use]
    pub fn builder() -> FileSystemConfigBuilder {
        FileSystemConfigBuilder::default()
    }

    /// Returns the read operation timeout.
    #[must_use]
    pub fn read_timeout(&self) -> Duration {
        self.read_timeout
    }

    /// Returns the write operation timeout.
    #[must_use]
    pub fn write_timeout(&self) -> Duration {
        self.write_timeout
    }

    /// Returns the general operation timeout.
    #[must_use]
    pub fn operation_timeout(&self) -> Duration {
        self.operation_timeout
    }
}

impl Default for FileSystemConfig {
    fn default() -> Self {
        Self {
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(30),
            operation_timeout: Duration::from_secs(60),
        }
    }
}
```

**Estimated Effort**: 30 minutes

---

#### Task 3.2: Implement FileSystemConfigBuilder

**Description**: Create the builder for `FileSystemConfig`.

**PRD References**:
- §5.2 FR-2.3.1-FR-2.3.5: Builder pattern requirements

**Acceptance Criteria**:
- [ ] `FileSystemConfigBuilder` struct created
- [ ] `with_read_timeout()`, `with_write_timeout()`, `with_operation_timeout()` methods
- [ ] `build()` method returns `FileSystemConfig`
- [ ] Builder starts with default values
- [ ] Unit tests for builder pattern
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Implementation Details**:

```rust
/// Builder for creating a custom `FileSystemConfig`.
#[derive(Debug, Clone)]
pub struct FileSystemConfigBuilder {
    config: FileSystemConfig,
}

impl Default for FileSystemConfigBuilder {
    fn default() -> Self {
        Self {
            config: FileSystemConfig::default(),
        }
    }
}

impl FileSystemConfigBuilder {
    /// Sets the read operation timeout.
    #[must_use]
    pub fn with_read_timeout(mut self, timeout: Duration) -> Self {
        self.config.read_timeout = timeout;
        self
    }

    /// Sets the write operation timeout.
    #[must_use]
    pub fn with_write_timeout(mut self, timeout: Duration) -> Self {
        self.config.write_timeout = timeout;
        self
    }

    /// Sets the general operation timeout.
    #[must_use]
    pub fn with_operation_timeout(mut self, timeout: Duration) -> Self {
        self.config.operation_timeout = timeout;
        self
    }

    /// Builds the `FileSystemConfig`.
    #[must_use]
    pub fn build(self) -> FileSystemConfig {
        self.config
    }
}
```

**Estimated Effort**: 30 minutes

---

### Epic 4: PathExt Module

**Goal**: Implement `PathExt` trait for synchronous path utilities.

**PRD Context**: §5.7 Path Extension Trait

---

#### Task 4.1: Define PathExt Trait

**Description**: Create the `PathExt` trait with `normalize()` method.

**PRD References**:
- §5.7 FR-7.1.1: Extend `std::path::Path`
- §5.7 FR-7.1.2: `normalize()` method
- §5.7 FR-7.1.3: Synchronous, no I/O

**Acceptance Criteria**:
- [ ] `PathExt` trait created extending `Path`
- [ ] `normalize()` method resolves `.` and `..` without I/O
- [ ] Implementation for `Path`
- [ ] Unit tests for various path patterns
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Implementation Details**:

```rust
use std::path::{Component, Path, PathBuf};

/// Extension trait for `Path` with utility methods.
pub trait PathExt {
    /// Normalizes the path by resolving `.` and `..` components.
    ///
    /// This method does NOT perform any I/O operations.
    /// It only processes the path components logically.
    fn normalize(&self) -> PathBuf;
}

impl PathExt for Path {
    fn normalize(&self) -> PathBuf {
        let mut components = Vec::new();
        
        for component in self.components() {
            match component {
                Component::Prefix(p) => components.push(Component::Prefix(p)),
                Component::RootDir => {
                    components.clear();
                    components.push(Component::RootDir);
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    if let Some(Component::Normal(_)) = components.last() {
                        components.pop();
                    } else if components.is_empty() {
                        components.push(Component::ParentDir);
                    }
                }
                Component::Normal(name) => {
                    components.push(Component::Normal(name));
                }
            }
        }
        
        if components.is_empty() {
            PathBuf::from(".")
        } else {
            components.iter().collect()
        }
    }
}
```

**Estimated Effort**: 1 hour

---

### Epic 5: FileSystem Trait

**Goal**: Define the async `FileSystem` trait with all operations.

**PRD Context**: §5.1 FileSystem Trait

---

#### Task 5.1: Define FileSystem Trait

**Description**: Create the async `FileSystem` trait with all operations defined in PRD §5.1.

**PRD References**:
- §5.1 FR-1.1.1-FR-1.1.6: Trait definition requirements
- §5.1 FR-1.2-FR-1.9: All operation categories

**Acceptance Criteria**:
- [ ] `FileSystem` trait created with `#[async_trait]`
- [ ] All read operations (FR-1.2)
- [ ] All write operations (FR-1.3)
- [ ] All metadata operations (FR-1.4)
- [ ] All directory operations (FR-1.5)
- [ ] All file operations (FR-1.6)
- [ ] All path operations (FR-1.7)
- [ ] All symlink operations (FR-1.8)
- [ ] Directory traversal (FR-1.9)
- [ ] Trait is `Send + Sync`
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Implementation Details**:

```rust
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use crate::{DirEntry, Metadata, Result};

/// Async filesystem abstraction trait.
///
/// This trait defines all filesystem operations required by consuming crates.
/// It is object-safe and can be used as `dyn FileSystem`.
#[async_trait]
pub trait FileSystem: Send + Sync {
    // Read Operations (FR-1.2)
    
    /// Reads the entire contents of a file as a UTF-8 string.
    async fn read_to_string(&self, path: &Path) -> Result<String>;
    
    /// Reads the entire contents of a file as bytes.
    async fn read_bytes(&self, path: &Path) -> Result<Vec<u8>>;

    // Write Operations (FR-1.3)
    
    /// Writes a string to a file, creating it if it doesn't exist.
    async fn write_string(&self, path: &Path, content: &str) -> Result<()>;
    
    /// Writes bytes to a file, creating it if it doesn't exist.
    async fn write_bytes(&self, path: &Path, content: &[u8]) -> Result<()>;
    
    /// Appends a string to a file.
    async fn append_string(&self, path: &Path, content: &str) -> Result<()>;
    
    /// Appends bytes to a file.
    async fn append_bytes(&self, path: &Path, content: &[u8]) -> Result<()>;

    // Metadata Operations (FR-1.4)
    
    /// Returns `true` if the path exists.
    async fn exists(&self, path: &Path) -> bool;
    
    /// Returns `true` if the path is a file.
    async fn is_file(&self, path: &Path) -> bool;
    
    /// Returns `true` if the path is a directory.
    async fn is_dir(&self, path: &Path) -> bool;
    
    /// Returns `true` if the path is a symbolic link.
    async fn is_symlink(&self, path: &Path) -> bool;
    
    /// Returns metadata for the path.
    async fn metadata(&self, path: &Path) -> Result<Metadata>;
    
    /// Returns metadata for the path without following symlinks.
    async fn symlink_metadata(&self, path: &Path) -> Result<Metadata>;

    // Directory Operations (FR-1.5)
    
    /// Creates a directory.
    async fn create_dir(&self, path: &Path) -> Result<()>;
    
    /// Creates a directory and all parent directories.
    async fn create_dir_all(&self, path: &Path) -> Result<()>;
    
    /// Lists the contents of a directory.
    async fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>>;
    
    /// Removes an empty directory.
    async fn remove_dir(&self, path: &Path) -> Result<()>;
    
    /// Removes a directory and all its contents.
    async fn remove_dir_all(&self, path: &Path) -> Result<()>;

    // File Operations (FR-1.6)
    
    /// Removes a file.
    async fn remove_file(&self, path: &Path) -> Result<()>;
    
    /// Copies a file from src to dst.
    async fn copy_file(&self, src: &Path, dst: &Path) -> Result<()>;
    
    /// Renames/moves a file or directory.
    async fn rename(&self, src: &Path, dst: &Path) -> Result<()>;

    // Path Operations (FR-1.7)
    
    /// Returns the canonical, absolute path with symlinks resolved.
    async fn canonicalize(&self, path: &Path) -> Result<PathBuf>;
    
    /// Returns the absolute path without resolving symlinks.
    fn absolute(&self, path: &Path) -> Result<PathBuf>;

    // Symlink Operations (FR-1.8)
    
    /// Returns the target of a symbolic link.
    async fn read_link(&self, path: &Path) -> Result<PathBuf>;

    // Directory Traversal (FR-1.9)
    
    /// Recursively walks a directory tree.
    async fn walk_dir(&self, path: &Path) -> Result<Vec<DirEntry>>;
}
```

**Estimated Effort**: 2 hours

---

### Epic 6: RealFileSystem Implementation

**Goal**: Implement `FileSystem` trait using `tokio::fs` with timeout support.

**PRD Context**: §5.3 RealFileSystem Implementation

---

#### Task 6.1: Create RealFileSystem Struct

**Description**: Create the `RealFileSystem` struct with configuration.

**PRD References**:
- §5.3 FR-3.1.1: Store `FileSystemConfig`
- §5.3 FR-3.1.2: `new()` with default config
- §5.3 FR-3.1.3: `with_config()` for custom config
- §5.3 FR-3.1.6: `Send + Sync + Clone`

**Acceptance Criteria**:
- [ ] `RealFileSystem` struct created with `FileSystemConfig` field
- [ ] `new()` and `with_config()` constructors
- [ ] Implements `Clone`, `Send`, `Sync`
- [ ] Implements `Default`
- [ ] Unit tests for constructors
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Estimated Effort**: 30 minutes

---

#### Task 6.2: Implement Timeout Helper

**Description**: Create internal helper for executing operations with timeout.

**PRD References**:
- §5.3 FR-3.1.7: All operations respect configured timeouts
- §5.3 FR-3.1.8: Timeout errors return `Error::Timeout`

**Acceptance Criteria**:
- [ ] Helper function/method for wrapping async operations with timeout
- [ ] Returns `Error::Timeout` when timeout exceeded
- [ ] Uses appropriate timeout based on operation type
- [ ] Unit tests for timeout behavior
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Estimated Effort**: 1 hour

---

#### Task 6.3: Implement Read Operations

**Description**: Implement `read_to_string` and `read_bytes` using `tokio::fs`.

**PRD References**:
- §5.1 FR-1.2.1-FR-1.2.4: Read operation requirements

**Acceptance Criteria**:
- [ ] `read_to_string` implemented with timeout
- [ ] `read_bytes` implemented with timeout
- [ ] Proper error handling for not found, permission denied
- [ ] UTF-8 validation for `read_to_string`
- [ ] Unit tests (with MockFileSystem or integration tests)
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Estimated Effort**: 1 hour

---

#### Task 6.4: Implement Write Operations

**Description**: Implement `write_string`, `write_bytes`, `append_string`, `append_bytes`.

**PRD References**:
- §5.1 FR-1.3.1-FR-1.3.7: Write operation requirements

**Acceptance Criteria**:
- [ ] All write operations implemented with timeout
- [ ] Creates file if doesn't exist
- [ ] Truncates on write, appends on append
- [ ] Fails if parent directory doesn't exist
- [ ] Unit tests
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Estimated Effort**: 1.5 hours

---

#### Task 6.5: Implement Metadata Operations

**Description**: Implement `exists`, `is_file`, `is_dir`, `is_symlink`, `metadata`, `symlink_metadata`.

**PRD References**:
- §5.1 FR-1.4.1-FR-1.4.6: Metadata operation requirements

**Acceptance Criteria**:
- [ ] All metadata operations implemented
- [ ] `exists`, `is_file`, `is_dir`, `is_symlink` return bool (no error on not found)
- [ ] `metadata` and `symlink_metadata` return `Result<Metadata>`
- [ ] Proper conversion to custom `Metadata` and `FileType`
- [ ] Unit tests
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Estimated Effort**: 1 hour

---

#### Task 6.6: Implement Directory Operations

**Description**: Implement `create_dir`, `create_dir_all`, `read_dir`, `remove_dir`, `remove_dir_all`.

**PRD References**:
- §5.1 FR-1.5.1-FR-1.5.6: Directory operation requirements

**Acceptance Criteria**:
- [ ] All directory operations implemented with timeout
- [ ] `read_dir` returns `Vec<DirEntry>` sorted alphabetically
- [ ] Proper error handling for not a directory, not empty
- [ ] Unit tests
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Estimated Effort**: 1.5 hours

---

#### Task 6.7: Implement File and Path Operations

**Description**: Implement `remove_file`, `copy_file`, `rename`, `canonicalize`, `absolute`, `read_link`.

**PRD References**:
- §5.1 FR-1.6.1-FR-1.6.3: File operations
- §5.1 FR-1.7.1-FR-1.7.3: Path operations
- §5.1 FR-1.8.1: Symlink operations

**Acceptance Criteria**:
- [ ] All file operations implemented with timeout
- [ ] `canonicalize` resolves symlinks
- [ ] `absolute` does NOT resolve symlinks (sync operation)
- [ ] `read_link` returns symlink target
- [ ] Unit tests
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Estimated Effort**: 1.5 hours

---

#### Task 6.8: Implement Directory Traversal

**Description**: Implement `walk_dir` with recursive async traversal.

**PRD References**:
- §5.1 FR-1.9.1-FR-1.9.4: Walk requirements

**Acceptance Criteria**:
- [ ] `walk_dir` recursively traverses directories
- [ ] Follows symlinks to directories
- [ ] Returns entries sorted for determinism
- [ ] Does NOT include root path in results
- [ ] Respects operation timeout
- [ ] Unit tests
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Estimated Effort**: 2 hours

---

### Epic 7: MockFileSystem Implementation

**Goal**: Implement in-memory `FileSystem` for testing.

**PRD Context**: §5.4 MockFileSystem Implementation

---

#### Task 7.1: Create MockFileSystem Struct

**Description**: Create the `MockFileSystem` struct with in-memory storage.

**PRD References**:
- §5.4 FR-4.1.1: Store files in `HashMap<PathBuf, Vec<u8>>`
- §5.4 FR-4.1.2: Track directories in `HashSet<PathBuf>`
- §5.4 FR-4.1.4: `Send + Sync` via `tokio::sync::RwLock`
- §5.4 FR-4.1.5: `Clone` clones Arc (shared state)
- §5.4 FR-4.1.6: `deep_clone()` for isolated copies

**Acceptance Criteria**:
- [ ] `MockFileSystem` struct with internal state wrapped in `Arc<RwLock<_>>`
- [ ] `new()` creates empty filesystem
- [ ] `Clone` implementation shares state via Arc
- [ ] `deep_clone()` creates isolated copy
- [ ] Unit tests for clone behavior
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Estimated Effort**: 1 hour

---

#### Task 7.2: Implement Setup Methods

**Description**: Implement `add_file`, `add_file_string`, `add_dir` for test setup.

**PRD References**:
- §5.4 FR-4.2.1-FR-4.2.5: Setup method requirements
- §5.4 FR-4.2.6: No timeout enforcement

**Acceptance Criteria**:
- [ ] `add_file` adds bytes at path
- [ ] `add_file_string` adds UTF-8 content
- [ ] `add_dir` creates directory
- [ ] All methods auto-create parent directories
- [ ] Unit tests
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Estimated Effort**: 1 hour

---

#### Task 7.3: Implement FileSystem Trait for MockFileSystem

**Description**: Implement all `FileSystem` trait methods for `MockFileSystem`.

**PRD References**:
- §5.4 FR-4.1.3: Implement all `FileSystem` methods
- §5.4 FR-4.2.6: No timeout enforcement (instant operations)

**Acceptance Criteria**:
- [ ] All `FileSystem` methods implemented
- [ ] Operations work on in-memory storage
- [ ] Proper error handling matching real filesystem behavior
- [ ] `walk_dir` returns sorted entries
- [ ] No timeouts (operations are instant)
- [ ] Unit tests for all operations
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Estimated Effort**: 4 hours

---

### Epic 8: Integration & Polish

**Goal**: End-to-end tests, documentation review, and final quality checks.

**PRD Context**: §9 Success Criteria

---

#### Task 8.1: Create E2E Tests for RealFileSystem

**Description**: Create integration tests that use the real filesystem via `tempfile`.

**PRD References**:
- §9.1: Integration tests passing 100%

**Acceptance Criteria**:
- [ ] `tests/real_fs_e2e.rs` created
- [ ] Tests for all major operations using temp directories
- [ ] Tests for error conditions
- [ ] Tests for timeout behavior (if feasible)
- [ ] All tests passing
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Estimated Effort**: 3 hours

---

#### Task 8.2: Create E2E Tests for MockFileSystem

**Description**: Create integration tests verifying MockFileSystem behaves like RealFileSystem.

**PRD References**:
- §6.2 NFR-2.2: MockFileSystem deterministic

**Acceptance Criteria**:
- [ ] `tests/mock_fs_e2e.rs` created
- [ ] Tests verifying mock behavior matches real behavior
- [ ] Tests for `deep_clone()` isolation
- [ ] All tests passing
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Estimated Effort**: 2 hours

---

#### Task 8.3: Documentation Review

**Description**: Review and improve all documentation.

**PRD References**:
- §8.4: Documentation Standards
- §9.1: Documentation complete for all public APIs

**Acceptance Criteria**:
- [ ] All public items have documentation
- [ ] Module-level docs with What/How/Why
- [ ] Examples for key functions
- [ ] `cargo doc` generates without warnings
- [ ] README.md created for the crate
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Estimated Effort**: 2 hours

---

#### Task 8.4: Final Quality Gates

**Description**: Final verification of all quality requirements.

**PRD References**:
- §9.2: Quality Gates

**Acceptance Criteria**:
- [ ] All tests passing on local machine
- [ ] `cargo clippy` zero warnings
- [ ] `cargo fmt --check` passes
- [ ] `cargo doc` no warnings
- [ ] `cargo build --release` succeeds
- [ ] All PRD P0 requirements verified
- [ ] Ask for review implementation in a new session

**Estimated Effort**: 1 hour

---

## 4. Task Dependency Graph

```
Epic 0: Project Setup
├── Task 0.1: Create Crate Skeleton
└── Task 0.2: Create Module Structure
         │
         ▼
Epic 1: Error Module
└── Task 1.1: Define Error Enum
         │
         ▼
Epic 2: Types Module
├── Task 2.1: Define FileType Enum
├── Task 2.2: Define Metadata Struct
└── Task 2.3: Define DirEntry Struct
         │
         ▼
Epic 3: Configuration Module
├── Task 3.1: Define FileSystemConfig Struct
└── Task 3.2: Implement FileSystemConfigBuilder
         │
         ├─────────────────────┐
         ▼                     ▼
Epic 4: PathExt Module    Epic 5: FileSystem Trait
└── Task 4.1: PathExt     └── Task 5.1: Define Trait
                                   │
                   ┌───────────────┴───────────────┐
                   ▼                               ▼
    Epic 6: RealFileSystem              Epic 7: MockFileSystem
    ├── Task 6.1: Struct                ├── Task 7.1: Struct
    ├── Task 6.2: Timeout Helper        ├── Task 7.2: Setup Methods
    ├── Task 6.3: Read Ops              └── Task 7.3: FileSystem Impl
    ├── Task 6.4: Write Ops
    ├── Task 6.5: Metadata Ops
    ├── Task 6.6: Directory Ops
    ├── Task 6.7: File/Path Ops
    └── Task 6.8: Walk Dir
                   │                               │
                   └───────────────┬───────────────┘
                                   ▼
                    Epic 8: Integration & Polish
                    ├── Task 8.1: RealFS E2E Tests
                    ├── Task 8.2: MockFS E2E Tests
                    ├── Task 8.3: Documentation Review
                    └── Task 8.4: Final Quality Gates
```

---

## 5. Estimated Effort Summary

| Epic | Tasks | Estimated Hours |
|------|-------|-----------------|
| Epic 0: Project Setup | 2 | 1.5 |
| Epic 1: Error Module | 1 | 1 |
| Epic 2: Types Module | 3 | 1.5 |
| Epic 3: Configuration Module | 2 | 1 |
| Epic 4: PathExt Module | 1 | 1 |
| Epic 5: FileSystem Trait | 1 | 2 |
| Epic 6: RealFileSystem | 8 | 10 |
| Epic 7: MockFileSystem | 3 | 6 |
| Epic 8: Integration & Polish | 4 | 8 |
| **Total** | **25** | **~32 hours** |

---

## 6. Testing Guidelines

Each module should include tests in `tests.rs` following this convention:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod file_type {
        use super::*;
        
        #[test]
        fn test_is_file() {
            assert!(FileType::File.is_file());
            assert!(!FileType::Dir.is_file());
        }
    }

    mod metadata {
        use super::*;
        
        #[test]
        fn test_len() {
            let meta = Metadata::new(FileType::File, 100);
            assert_eq!(meta.len(), 100);
        }
    }

    mod config {
        use super::*;
        
        #[test]
        fn test_default_timeouts() {
            let config = FileSystemConfig::default();
            assert_eq!(config.read_timeout(), Duration::from_secs(30));
        }
    }
}
```

For async tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod real_fs {
        use super::*;
        
        #[tokio::test]
        async fn test_read_write() {
            // Use tempfile for real filesystem tests
        }
    }
}
```

---

## 7. References

- [PRD.md](./PRD.md) - Product Requirements Document
- [workspace-core PLAN](../core/PLAN.md) - Reference implementation plan
- [tokio::fs documentation](https://docs.rs/tokio/latest/tokio/fs/)
- [async-trait documentation](https://docs.rs/async-trait/)
- [snafu documentation](https://docs.rs/snafu/)