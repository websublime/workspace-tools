# Product Requirements Document: workspace-fs

## Document Information

| Field | Value |
|-------|-------|
| **Crate Name** | `workspace-fs` |
| **Version** | `0.1.0` |
| **Status** | Ready |
| **Created** | 2026-01-13 |
| **Last Updated** | 2026-01-13 |

---

## 1. Executive Summary

### 1.1 Purpose

The `workspace-fs` crate provides a unified filesystem abstraction layer for all crates within the workspace-node-tools ecosystem. It serves as the single point of contact for all filesystem operations, enabling consistent behavior, comprehensive testing through mocking, and high-performance asynchronous I/O essential for large monorepos.

### 1.2 Scope

This crate focuses exclusively on:

- **Filesystem Abstraction**: Async trait-based interface for all filesystem operations
- **Real Filesystem Implementation**: Production-ready implementation using `tokio::fs`
- **Mock Filesystem Implementation**: In-memory implementation for testing
- **Path Utilities**: Common path operations and normalization
- **Directory Traversal**: Abstracted recursive directory walking
- **Error Standardization**: Unified error types for filesystem operations

### 1.3 Out of Scope

The following concerns are explicitly **not** part of this crate:

- Git operations (delegated to `workspace-git` crate)
- File watching/notification (future consideration)
- Network filesystem protocols (NFS, SMB, etc.)
- Archive handling (zip, tar, etc.)
- Temporary file management beyond basic creation

### 1.4 Dependencies

#### 1.4.1 Internal Dependencies

None. This is a foundational crate with no internal dependencies.

#### 1.4.2 External Dependencies

| Crate | Version | Category | Purpose |
|-------|---------|----------|---------|
| `snafu` | `0.8.9` | dep | Error handling with context |
| `tokio` | `1.49.0` | dep | Async runtime and filesystem operations (features: `fs`, `sync`) |
| `async-trait` | `0.1.89` | dep | Async trait support for object-safe traits |
| `log` | `0.4` | dep | Logging facade |

#### 1.4.3 Development Dependencies

| Crate | Version | Category | Purpose |
|-------|---------|----------|---------|
| `tempfile` | `3.24.0` | dev-dep | Temporary directories for integration tests |
| `tokio` | `1.49.0` | dev-dep | Async runtime for tests (features: `rt-multi-thread`, `macros`) |

---

## 2. Problem Statement

### 2.1 Current Challenges

When building tools that interact with the filesystem from Rust, developers face several challenges:

1. **Testing Complexity**: Direct filesystem operations are difficult to test in isolation. Tests become slow, flaky, and dependent on filesystem state.

2. **Inconsistent Error Handling**: Different parts of the codebase may handle filesystem errors inconsistently, leading to poor user experiences.

3. **Platform Differences**: Windows, macOS, and Linux have subtle differences in filesystem behavior that can cause bugs.

4. **Performance in Large Monorepos**: Synchronous filesystem operations become a bottleneck when processing thousands of files in large monorepos.

5. **Scattered Logic**: Without centralization, filesystem logic gets scattered across modules, making it hard to maintain consistent behavior.

### 2.2 Solution

The `workspace-fs` crate provides:

- An async trait-based abstraction (`FileSystem`) that defines all filesystem operations
- A production implementation (`RealFileSystem`) using `tokio::fs` for high-performance async I/O
- A mock implementation (`MockFileSystem`) for fast, deterministic unit testing
- Standardized error types with rich context
- Platform-agnostic path handling
- Centralized logging for filesystem operations

---

## 3. Conceptual Model

### 3.1 Core Concepts

```
┌─────────────────────────────────────────────────────────────────┐
│                       Consuming Crates                          │
│  (workspace-core, workspace-git, workspace-executor, etc.)     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                  FileSystem Trait (async)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │    Read      │  │    Write     │  │   Metadata   │          │
│  │  Operations  │  │  Operations  │  │  Operations  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Directory   │  │    Path      │  │   Symlink    │          │
│  │  Operations  │  │  Operations  │  │  Operations  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
┌─────────────────────────┐     ┌─────────────────────────┐
│    RealFileSystem       │     │    MockFileSystem       │
│    (Production)         │     │    (Testing)            │
│                         │     │                         │
│  Uses tokio::fs         │     │  In-memory with RwLock  │
└─────────────────────────┘     └─────────────────────────┘
```

### 3.2 Concept Definitions

| Concept | Definition |
|---------|------------|
| **FileSystem** | Async trait defining all filesystem operations required by consuming crates |
| **RealFileSystem** | Lightweight struct implementing `FileSystem` via `tokio::fs`, holds `FileSystemConfig` |
| **MockFileSystem** | In-memory implementation storing files as `HashMap<PathBuf, Vec<u8>>` with `RwLock` |
| **DirEntry** | Abstraction over directory entries (files, directories, symlinks) |
| **FileType** | Enum representing file type (file, directory, symlink) |
| **Metadata** | File metadata abstraction (size, type, permissions) |

### 3.3 Operation Categories

| Category | Operations |
|----------|------------|
| **Read** | `read_to_string`, `read_bytes` |
| **Write** | `write_string`, `write_bytes`, `append_string`, `append_bytes` |
| **Metadata** | `exists`, `is_file`, `is_dir`, `is_symlink`, `metadata` |
| **Directory** | `create_dir`, `create_dir_all`, `read_dir`, `remove_dir`, `remove_dir_all` |
| **File** | `remove_file`, `copy_file`, `rename` |
| **Path** | `canonicalize`, `absolute` |
| **Symlink** | `read_link`, `symlink_metadata` |
| **Traversal** | `walk_dir` |

---

## 4. User Personas

### 4.1 Primary Users

| Persona | Description | Needs |
|---------|-------------|-------|
| **workspace-core Developer** | Implements project detection | Path existence checks, file reading, directory traversal |
| **workspace-git Developer** | Implements git operations | File reading/writing, directory operations |
| **Test Author** | Writes unit tests for consuming crates | Mock filesystem with predefined content |
| **Performance-Critical User** | Works with large monorepos | Non-blocking async I/O for thousands of files |

### 4.2 Use Cases

#### UC-1: Read Configuration File

**Actor**: workspace-core  
**Description**: Read and parse a `package.json` file  
**Preconditions**: Path to file is known  
**Flow**:
1. Call `fs.read_to_string(path).await`
2. Handle `Result` - either content or error
3. Parse content as JSON

**Postconditions**: File content returned or descriptive error

#### UC-2: Check File Existence

**Actor**: workspace-core  
**Description**: Check if a lock file exists to detect package manager  
**Preconditions**: Path to check is known  
**Flow**:
1. Call `fs.exists(path).await`
2. Use boolean result for detection logic

**Postconditions**: Boolean indicating existence

#### UC-3: Discover Workspace Packages

**Actor**: workspace-core  
**Description**: Walk directory tree to find all `package.json` files  
**Preconditions**: Root path and glob patterns are known  
**Flow**:
1. Call `fs.walk_dir(root).await`
2. Filter entries by pattern
3. Collect matching paths

**Postconditions**: List of package paths

#### UC-4: Unit Test with Mock Filesystem

**Actor**: Test Author  
**Description**: Test project detection without real files  
**Preconditions**: Test fixtures defined in code  
**Flow**:
1. Create `MockFileSystem::new()`
2. Add files: `mock.add_file("package.json", content).await`
3. Pass mock to code under test
4. Assert expected behavior

**Postconditions**: Test runs in memory, no disk I/O

#### UC-5: Write Generated File

**Actor**: Future crate (e.g., changeset generator)  
**Description**: Write a generated changeset file  
**Preconditions**: Content and destination path known  
**Flow**:
1. Call `fs.write_string(path, content).await`
2. Handle `Result`

**Postconditions**: File written or descriptive error

---

## 5. Functional Requirements

### 5.1 FileSystem Trait (`trait FileSystem`)

#### FR-1.1: Trait Definition

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1.1 | Trait SHALL be named `FileSystem` | P0 |
| FR-1.1.2 | Trait SHALL use `#[async_trait]` for async method support | P0 |
| FR-1.1.3 | Trait SHALL be object-safe (usable as `dyn FileSystem`) | P0 |
| FR-1.1.4 | Trait SHALL be `Send + Sync` for thread safety | P0 |
| FR-1.1.5 | All fallible methods SHALL return `Result<T, Error>` | P0 |
| FR-1.1.6 | All path parameters SHALL use `&Path` type | P0 |

#### FR-1.2: Read Operations

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.2.1 | `async fn read_to_string(&self, path: &Path) -> Result<String>` SHALL read entire file as UTF-8 string | P0 |
| FR-1.2.2 | `async fn read_bytes(&self, path: &Path) -> Result<Vec<u8>>` SHALL read entire file as bytes | P0 |
| FR-1.2.3 | Read operations SHALL fail with appropriate error if file not found | P0 |
| FR-1.2.4 | Read operations SHALL fail with appropriate error if path is a directory | P0 |

#### FR-1.3: Write Operations

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.3.1 | `async fn write_string(&self, path: &Path, content: &str) -> Result<()>` SHALL write string as UTF-8 | P0 |
| FR-1.3.2 | `async fn write_bytes(&self, path: &Path, content: &[u8]) -> Result<()>` SHALL write raw bytes | P0 |
| FR-1.3.3 | Write operations SHALL create the file if it doesn't exist | P0 |
| FR-1.3.4 | Write operations SHALL truncate existing file before writing | P0 |
| FR-1.3.5 | Write operations SHALL fail if parent directory doesn't exist | P0 |
| FR-1.3.6 | `async fn append_string(&self, path: &Path, content: &str) -> Result<()>` SHALL append to file | P0 |
| FR-1.3.7 | `async fn append_bytes(&self, path: &Path, content: &[u8]) -> Result<()>` SHALL append to file | P0 |

#### FR-1.4: Metadata Operations

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.4.1 | `async fn exists(&self, path: &Path) -> bool` SHALL return true if path exists (follows symlinks) | P0 |
| FR-1.4.2 | `async fn is_file(&self, path: &Path) -> bool` SHALL return true if path is a regular file | P0 |
| FR-1.4.3 | `async fn is_dir(&self, path: &Path) -> bool` SHALL return true if path is a directory | P0 |
| FR-1.4.4 | `async fn is_symlink(&self, path: &Path) -> bool` SHALL return true if path is a symbolic link | P0 |
| FR-1.4.5 | `async fn metadata(&self, path: &Path) -> Result<Metadata>` SHALL return file metadata | P0 |
| FR-1.4.6 | `async fn symlink_metadata(&self, path: &Path) -> Result<Metadata>` SHALL return metadata without following symlinks | P0 |

#### FR-1.5: Directory Operations

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.5.1 | `async fn create_dir(&self, path: &Path) -> Result<()>` SHALL create a single directory | P0 |
| FR-1.5.2 | `async fn create_dir_all(&self, path: &Path) -> Result<()>` SHALL create directory and all parents | P0 |
| FR-1.5.3 | `async fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>>` SHALL list directory contents | P0 |
| FR-1.5.4 | `async fn remove_dir(&self, path: &Path) -> Result<()>` SHALL remove empty directory | P0 |
| FR-1.5.5 | `async fn remove_dir_all(&self, path: &Path) -> Result<()>` SHALL remove directory recursively | P0 |
| FR-1.5.6 | Directory operations SHALL fail with appropriate error if path is a file | P0 |

#### FR-1.6: File Operations

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.6.1 | `async fn remove_file(&self, path: &Path) -> Result<()>` SHALL delete a file | P0 |
| FR-1.6.2 | `async fn copy_file(&self, src: &Path, dst: &Path) -> Result<()>` SHALL copy file content | P0 |
| FR-1.6.3 | `async fn rename(&self, src: &Path, dst: &Path) -> Result<()>` SHALL rename/move a file or directory | P0 |

#### FR-1.7: Path Operations

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.7.1 | `async fn canonicalize(&self, path: &Path) -> Result<PathBuf>` SHALL return absolute path with symlinks resolved | P0 |
| FR-1.7.2 | `fn absolute(&self, path: &Path) -> Result<PathBuf>` SHALL return absolute path without resolving symlinks (sync, no I/O) | P0 |
| FR-1.7.3 | `canonicalize` SHALL fail if path doesn't exist | P0 |

#### FR-1.8: Symlink Operations

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.8.1 | `async fn read_link(&self, path: &Path) -> Result<PathBuf>` SHALL return symlink target | P0 |
| FR-1.8.2 | All read/write operations SHALL follow symlinks by default | P0 |

#### FR-1.9: Directory Traversal

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.9.1 | `async fn walk_dir(&self, path: &Path) -> Result<Vec<DirEntry>>` SHALL traverse recursively | P0 |
| FR-1.9.2 | Walk SHALL follow symlinks to directories | P0 |
| FR-1.9.3 | Walk SHALL yield entries in sorted order for determinism | P0 |
| FR-1.9.4 | Walk SHALL NOT include the root path in results | P0 |

### 5.2 Configuration (`FileSystemConfig`)

#### FR-2.1: Configuration Struct

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | `FileSystemConfig` SHALL provide `read_timeout: Duration` for read operations | P0 |
| FR-2.1.2 | `FileSystemConfig` SHALL provide `write_timeout: Duration` for write operations | P0 |
| FR-2.1.3 | `FileSystemConfig` SHALL provide `operation_timeout: Duration` for other operations | P0 |
| FR-2.1.4 | `FileSystemConfig` SHALL implement `Default` with sensible timeout values | P0 |
| FR-2.1.5 | `FileSystemConfig` SHALL implement `Debug + Clone` | P0 |

#### FR-2.2: Default Timeout Values

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | Default `read_timeout` SHALL be 30 seconds | P0 |
| FR-2.2.2 | Default `write_timeout` SHALL be 30 seconds | P0 |
| FR-2.2.3 | Default `operation_timeout` SHALL be 60 seconds | P0 |

#### FR-2.3: Builder Pattern

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | `FileSystemConfig::builder()` SHALL return a `FileSystemConfigBuilder` | P0 |
| FR-2.3.2 | Builder SHALL provide `with_read_timeout(Duration)` method | P0 |
| FR-2.3.3 | Builder SHALL provide `with_write_timeout(Duration)` method | P0 |
| FR-2.3.4 | Builder SHALL provide `with_operation_timeout(Duration)` method | P0 |
| FR-2.3.5 | Builder SHALL provide `build() -> FileSystemConfig` method | P0 |

### 5.3 RealFileSystem Implementation

#### FR-3.1: Implementation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1.1 | `RealFileSystem` SHALL store a `FileSystemConfig` instance | P0 |
| FR-3.1.2 | `RealFileSystem::new()` SHALL create instance with default config | P0 |
| FR-3.1.3 | `RealFileSystem::with_config(config)` SHALL create instance with custom config | P0 |
| FR-3.1.4 | `RealFileSystem` SHALL implement all `FileSystem` methods using `tokio::fs` | P0 |
| FR-3.1.5 | `RealFileSystem::walk_dir` SHALL use recursive async implementation | P0 |
| FR-3.1.6 | `RealFileSystem` SHALL be `Send + Sync + Clone` | P0 |
| FR-3.1.7 | All operations SHALL respect configured timeouts | P0 |
| FR-3.1.8 | Timeout errors SHALL return `Error::Timeout` variant | P0 |

### 5.4 MockFileSystem Implementation

#### FR-4.1: Implementation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.1.1 | `MockFileSystem` SHALL store files in `HashMap<PathBuf, Vec<u8>>` | P0 |
| FR-4.1.2 | `MockFileSystem` SHALL track directories separately in `HashSet<PathBuf>` | P0 |
| FR-4.1.3 | `MockFileSystem` SHALL implement all `FileSystem` methods | P0 |
| FR-4.1.4 | `MockFileSystem` SHALL be `Send + Sync` via interior mutability (`tokio::sync::RwLock`) | P0 |
| FR-4.1.5 | `MockFileSystem` SHALL implement `Clone` (clones the Arc, shared state) | P0 |
| FR-4.1.6 | `MockFileSystem` SHALL provide `fn deep_clone(&self) -> Self` for isolated copies | P0 |

#### FR-4.2: Setup Methods

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.2.1 | `async fn add_file(&self, path: impl AsRef<Path>, content: impl AsRef<[u8]>)` SHALL add a file | P0 |
| FR-4.2.2 | `async fn add_file_string(&self, path: impl AsRef<Path>, content: impl AsRef<str>)` SHALL add a UTF-8 file | P0 |
| FR-4.2.3 | `async fn add_dir(&self, path: impl AsRef<Path>)` SHALL add a directory | P0 |
| FR-4.2.4 | Setup methods SHALL automatically create parent directories | P0 |
| FR-4.2.5 | `MockFileSystem::new()` SHALL create empty filesystem | P0 |
| FR-4.2.6 | `MockFileSystem` SHALL NOT enforce timeouts (instant operations) | P0 |

### 5.5 Entry Types

#### FR-5.1: DirEntry

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-5.1.1 | `DirEntry` SHALL provide `fn path(&self) -> &Path` | P0 |
| FR-5.1.2 | `DirEntry` SHALL provide `fn file_name(&self) -> &OsStr` | P0 |
| FR-5.1.3 | `DirEntry` SHALL provide `fn file_type(&self) -> FileType` | P0 |
| FR-5.1.4 | `DirEntry` SHALL implement `Debug + Clone` | P0 |

#### FR-5.2: FileType

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-5.2.1 | `FileType` SHALL be an enum with variants `File`, `Dir`, `Symlink` | P0 |
| FR-5.2.2 | `FileType` SHALL provide `fn is_file(&self) -> bool` | P0 |
| FR-5.2.3 | `FileType` SHALL provide `fn is_dir(&self) -> bool` | P0 |
| FR-5.2.4 | `FileType` SHALL provide `fn is_symlink(&self) -> bool` | P0 |
| FR-5.2.5 | `FileType` SHALL implement `Debug + Clone + Copy + PartialEq + Eq` | P0 |

#### FR-5.3: Metadata

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-5.3.1 | `Metadata` SHALL provide `fn len(&self) -> u64` (file size) | P0 |
| FR-5.3.2 | `Metadata` SHALL provide `fn is_file(&self) -> bool` | P0 |
| FR-5.3.3 | `Metadata` SHALL provide `fn is_dir(&self) -> bool` | P0 |
| FR-5.3.4 | `Metadata` SHALL provide `fn is_symlink(&self) -> bool` | P0 |
| FR-5.3.5 | `Metadata` SHALL provide `fn file_type(&self) -> FileType` | P0 |
| FR-5.3.6 | `Metadata` SHALL implement `Debug + Clone` | P0 |

### 5.6 Error Handling (`error`)

#### FR-6.1: Error Type

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-6.1.1 | Single unified `Error` enum for all filesystem errors | P0 |
| FR-6.1.2 | Error SHALL use `snafu` for context and chaining | P0 |
| FR-6.1.3 | Error SHALL implement `std::error::Error` | P0 |
| FR-6.1.4 | Error SHALL be `Send + Sync` | P0 |

#### FR-6.2: Error Variants

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-6.2.1 | `NotFound { path }` - path does not exist | P0 |
| FR-6.2.2 | `PermissionDenied { path }` - insufficient permissions | P0 |
| FR-6.2.3 | `AlreadyExists { path }` - path already exists | P0 |
| FR-6.2.4 | `NotAFile { path }` - expected file but found directory | P0 |
| FR-6.2.5 | `NotADirectory { path }` - expected directory but found file | P0 |
| FR-6.2.6 | `NotEmpty { path }` - directory not empty | P0 |
| FR-6.2.7 | `InvalidUtf8 { path }` - file content is not valid UTF-8 | P0 |
| FR-6.2.8 | `Io { path, operation, source }` - wrapped I/O error | P0 |
| FR-6.2.9 | `Timeout { path, operation, duration }` - operation timed out | P0 |

### 5.7 Path Extension Trait (`PathExt`)

#### FR-7.1: PathExt Trait

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-7.1.1 | `PathExt` SHALL extend `std::path::Path` with utility methods | P0 |
| FR-7.1.2 | `fn normalize(&self) -> PathBuf` SHALL resolve `.` and `..` components without I/O | P0 |
| FR-7.1.3 | `PathExt` methods SHALL be synchronous (no I/O) | P0 |

---

## 6. Non-Functional Requirements

### 6.1 Performance

| ID | Requirement | Measurement |
|----|-------------|-------------|
| NFR-1.1 | Async operations SHALL NOT block the runtime | Code review, no blocking calls |
| NFR-1.2 | MockFileSystem operations < 1μs average | Benchmark |
| NFR-1.3 | `RealFileSystem` SHALL be lightweight (only config storage) | Code review |
| NFR-1.4 | Directory walks SHALL be concurrent-safe | Stress tests |

### 6.2 Reliability

| ID | Requirement | Measurement |
|----|-------------|-------------|
| NFR-2.1 | All errors SHALL preserve original I/O error | Error chain inspection |
| NFR-2.2 | MockFileSystem SHALL be deterministic | Identical results for identical setup |
| NFR-2.3 | Operations SHALL be cancellation-safe | Tokio cancellation tests |

### 6.3 Compatibility

| ID | Requirement | Measurement |
|----|-------------|-------------|
| NFR-3.1 | Support Windows, macOS, and Linux | CI tests on all platforms |
| NFR-3.2 | Handle platform-specific path separators | Path normalization tests |
| NFR-3.3 | Support symlinks on all platforms (where OS supports) | Symlink tests |

### 6.4 Code Quality

| ID | Requirement | Measurement |
|----|-------------|-------------|
| NFR-4.1 | No `unsafe` code | Code review, `#![forbid(unsafe_code)]` |
| NFR-4.2 | No `unwrap()` or `expect()` in production code | Clippy lints |
| NFR-4.3 | Documentation on all public items | `cargo doc` warnings |
| NFR-4.4 | Clippy clean with deny settings | CI gate |

### 6.5 Logging

#### 6.5.1 Logging Levels

| Level | Usage |
|-------|-------|
| `trace` | Every filesystem operation entry/exit |
| `debug` | Filesystem operation results |
| `warn` | Recoverable issues (e.g., permission denied on non-critical path) |
| `error` | Unrecoverable errors |

#### 6.5.2 Logging Examples

```rust
// trace: operation entry
log::trace!("read_to_string: entering with path={:?}", path);

// debug: operation success
log::debug!("read_to_string: read {} bytes from {:?}", content.len(), path);

// debug: operation failure (non-existence is expected sometimes)
log::debug!("exists: path {:?} does not exist", path);

// warn: unexpected but non-fatal
log::warn!("canonicalize: path {:?} contains non-UTF8 components", path);

// error: operation failed
log::error!("write_string: failed to write to {:?}: {}", path, error);
```

#### 6.5.3 Activation

- Logging is via the `log` crate facade
- Consuming crates/applications initialize the logger implementation
- No logger initialization in this crate

---

## 7. Architecture Overview

### 7.1 Module Structure

```
workspace-fs/                    # Directory: crates/filesystem/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Crate root, re-exports
│   ├── error.rs                # Error enum and Result alias
│   ├── config.rs               # FileSystemConfig and FileSystemConfigBuilder
│   ├── types.rs                # DirEntry, FileType, Metadata
│   ├── traits.rs               # FileSystem trait definition with async_trait
│   ├── path_ext.rs             # PathExt trait for Path utilities
│   ├── real.rs                 # RealFileSystem implementation (tokio::fs)
│   ├── mock.rs                 # MockFileSystem implementation
│   └── tests.rs                # Unit tests: mod tests { mod error { } mod config { } mod types { } ... }
└── tests/
    ├── real_fs_e2e.rs          # E2E tests with real filesystem
    └── mock_fs_e2e.rs          # E2E tests with mock filesystem
```

### 7.2 Dependency Graph (Internal Modules)

```
                         ┌──────────┐
                         │  lib.rs  │
                         │(re-exports)│
                         └────┬─────┘
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
         ▼                    ▼                    ▼
    ┌──────────┐        ┌──────────┐        ┌──────────┐
    │  traits  │        │   real   │        │   mock   │
    └────┬─────┘        └────┬─────┘        └────┬─────┘
         │                   │                   │
         │                   ▼                   │
         │              ┌──────────┐             │
         │              │  config  │             │
         │              └────┬─────┘             │
         │                   │                   │
         ▼                   ▼                   ▼
    ┌──────────────────────────────────────────────┐
    │                    types                      │
    │         (DirEntry, FileType, Metadata)       │
    └──────────────────────┬───────────────────────┘
                           │
                           ▼
                    ┌──────────┐
                    │  error   │
                    └──────────┘

    ┌──────────┐
    │ path_ext │  (standalone, no dependencies)
    └──────────┘
```

### 7.3 Key Design Principles

1. **Async-First API**: All I/O operations are async using `tokio::fs` for optimal performance in large monorepos.

2. **Configurable Timeouts**: All operations respect configurable timeouts to prevent hanging on slow/unresponsive filesystems.

3. **Testability First**: `MockFileSystem` enables fast, deterministic tests without disk I/O (no timeouts enforced).

4. **Error Context**: Every error includes the path and operation that failed.

5. **Platform Agnostic API**: Consumers don't need to handle platform differences.

6. **No Global State**: No global filesystem instance; always passed as parameter.

7. **Object-Safe Traits**: `FileSystem` can be used as `dyn FileSystem` for runtime polymorphism.

8. **Sensible Defaults**: Default configuration works well for most use cases; customization available when needed.

---

## 8. API Design Principles

### 8.1 Naming Conventions

- **Types**: PascalCase (e.g., `FileSystem`, `RealFileSystem`, `MockFileSystem`)
- **Methods**: snake_case (e.g., `read_to_string`, `create_dir_all`)
- **Constants**: SCREAMING_SNAKE_CASE (if any)
- **Modules**: snake_case (e.g., `traits`, `error`)

### 8.2 Path Handling

- All methods take `&Path` for paths
- No assumption about path format (relative vs absolute)
- Paths are used as-is; canonicalization is explicit via `canonicalize()`
- No implicit current directory operations

### 8.3 Error Handling

- All fallible operations return `Result<T, Error>`
- Type alias: `pub type Result<T> = std::result::Result<T, Error>`
- Errors always include the path that failed
- I/O errors are wrapped, not replaced

### 8.4 Documentation Standards

- All public items documented
- Module-level docs with What/How/Why
- Examples for all public functions
- Cross-references between related items

---

## 9. Success Criteria

### 9.1 Acceptance Criteria

| Criterion | Measurement |
|-----------|-------------|
| All P0 requirements implemented | 100% coverage |
| Unit test coverage | > 80% |
| Integration tests passing | 100% |
| Documentation complete | All public APIs |
| Clippy clean | Zero warnings |

### 9.2 Quality Gates

1. **Code Review**: All code reviewed by at least one other developer
2. **CI/CD**: All tests pass on Windows, macOS, and Linux
3. **Documentation**: Generated docs reviewed for completeness
4. **Performance**: Async operations verified non-blocking

---

## 10. Future Considerations

### 10.1 Potential Extensions (Not in Scope)

- **File Watching**: `watch_dir` for file change notifications
- **Temp Files**: Managed temporary file/directory creation
- **Atomic Writes**: Write-to-temp-then-rename pattern
- **Compression**: Read/write compressed files directly
- **Memory Mapping**: Memory-mapped file access
- **Timeout Configuration**: Configurable operation timeouts (like legacy code)

### 10.2 Design for Extension

The current design supports future extensions:

1. Add new methods to `FileSystem` trait with default implementations
2. `MockFileSystem` can simulate any new behavior
3. Consuming crates can define extension traits if needed

---

## 11. Glossary

| Term | Definition |
|------|------------|
| **Mock** | Test double that simulates real behavior in a controlled way |
| **DI** | Dependency Injection - passing dependencies as parameters |
| **Facade** | Interface that hides complexity (e.g., `log` crate) |
| **async_trait** | Macro that enables async methods in traits (object-safe) |

---

## 12. References

- [Rust std::fs module](https://doc.rust-lang.org/std/fs/)
- [tokio::fs module](https://docs.rs/tokio/latest/tokio/fs/)
- [async-trait crate](https://docs.rs/async-trait/)
- [snafu crate](https://docs.rs/snafu/)
- [workspace-core PRD](../core/PRD.md)
- [Legacy filesystem implementation](../../temp/wnt-stable/crates/standard/src/filesystem/)