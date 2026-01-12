# Implementation Plan: workspace-core

## Document Information

| Field | Value |
|-------|-------|
| **Crate Name** | `workspace-core` |
| **PRD Reference** | [PRD.md](./PRD.md) |
| **Status** | Draft |
| **Created** | 2026-01-12 |
| **Last Updated** | 2026-01-12 |

---

## 1. Overview

This document details the implementation plan for the `workspace-core` crate. The implementation is divided into epics, each containing multiple tasks. Tasks are designed to be atomic units of work that can be completed and committed independently.

---

## 2. Implementation Phases

### Phase 0: Project Setup
Foundation and scaffolding for the crate.

### Phase 1: Error Module
Core error types that all other modules depend on.

### Phase 2: Configuration Module
Configuration types for detection behavior.

### Phase 3: Node Module
Package manager abstractions and repository types.

### Phase 4: Monorepo Module
Monorepo types, detection, and workspace analysis.

### Phase 5: Project Module
Unified project detection and management.

### Phase 6: Integration & Polish
End-to-end tests, documentation review, and optimization.

---

## 3. Epic Breakdown

---

### Epic 0: Project Setup

**Goal**: Establish the crate structure, dependencies, and development configuration.

#### Task 0.1: Create Crate Skeleton

**Description**: Initialize the crate with Cargo.toml and basic structure.

**Acceptance Criteria**:
- [ ] `Cargo.toml` created with proper metadata
- [ ] `src/lib.rs` created with clippy lints and crate-level documentation
- [ ] Crate compiles with `cargo check`

**Implementation Details**:

```toml
# Cargo.toml
[package]
name = "workspace-core"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
description = "Core detection and abstractions for JavaScript/TypeScript projects"
license = "MIT"
repository = "https://github.com/user/workspace-node-tools"
keywords = ["nodejs", "monorepo", "workspace", "package-manager"]
categories = ["development-tools"]

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
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
walkdir = "2"
glob = "0.3"

[dev-dependencies]
tempfile = "3"
```

**Files to Create**:
- `crates/core/Cargo.toml`
- `crates/core/src/lib.rs`

**Estimated Effort**: 1 hour

---

#### Task 0.2: Create Module Structure

**Description**: Create the directory structure and empty module files.

**Acceptance Criteria**:
- [ ] All module directories created
- [ ] All `mod.rs` files created with module declarations
- [ ] Crate compiles with empty modules

**Files to Create**:
- `crates/core/src/error/mod.rs`
- `crates/core/src/config/mod.rs`
- `crates/core/src/node/mod.rs`
- `crates/core/src/project/mod.rs`
- `crates/core/src/monorepo/mod.rs`

**Estimated Effort**: 30 minutes

---

### Epic 1: Error Module

**Goal**: Implement comprehensive error types for all modules.

#### Task 1.1: Define Core Error Enum

**Description**: Create the main `Error` enum that aggregates all error types.

**Acceptance Criteria**:
- [ ] `Error` enum defined with variants for each module
- [ ] `std::error::Error` implemented
- [ ] `Display` implemented with descriptive messages
- [ ] Unit tests for error creation and display

**Implementation Details**:

```rust
// src/error/mod.rs

//! # Error Module
//!
//! ## What
//! Comprehensive error types for the workspace-core crate.
//!
//! ## How
//! Uses thiserror for derive macros and provides specific error
//! types for each failure mode.
//!
//! ## Why
//! Structured errors enable proper error handling and provide
//! actionable information to users.

mod types;

pub use types::*;

/// Result type alias for workspace-core operations.
pub type Result<T> = std::result::Result<T, Error>;
```

**Files to Create/Modify**:
- `crates/core/src/error/mod.rs`
- `crates/core/src/error/types.rs`
- `crates/core/src/error/tests.rs`

**Estimated Effort**: 2 hours

---

#### Task 1.2: Define PackageManagerError

**Description**: Create error types specific to package manager operations.

**Acceptance Criteria**:
- [ ] `PackageManagerError` enum defined
- [ ] Variants: `NotFound`, `DetectionFailed`, `InvalidConfiguration`
- [ ] Context information included (path, reason)
- [ ] Unit tests for each variant

**Implementation Details**:

```rust
/// Errors related to package manager detection and operations.
#[derive(Debug, thiserror::Error)]
pub enum PackageManagerError {
    /// No package manager could be detected at the given path.
    #[error("no package manager found at '{path}'")]
    NotFound {
        /// The path where detection was attempted.
        path: PathBuf,
    },

    /// Package manager detection failed due to an error.
    #[error("package manager detection failed at '{path}': {reason}")]
    DetectionFailed {
        /// The path where detection was attempted.
        path: PathBuf,
        /// The reason for the failure.
        reason: String,
    },

    /// Invalid package manager configuration.
    #[error("invalid package manager configuration: {reason}")]
    InvalidConfiguration {
        /// The reason the configuration is invalid.
        reason: String,
    },
}
```

**Estimated Effort**: 1 hour

---

#### Task 1.3: Define MonorepoError

**Description**: Create error types specific to monorepo operations.

**Acceptance Criteria**:
- [ ] `MonorepoError` enum defined
- [ ] Variants for detection, config parsing, package discovery
- [ ] Context information included
- [ ] Unit tests for each variant

**Implementation Details**:

```rust
/// Errors related to monorepo detection and analysis.
#[derive(Debug, thiserror::Error)]
pub enum MonorepoError {
    /// Not a monorepo root.
    #[error("'{path}' is not a monorepo root")]
    NotMonorepo {
        /// The path that was checked.
        path: PathBuf,
    },

    /// Failed to detect monorepo type.
    #[error("failed to detect monorepo type at '{path}': {reason}")]
    DetectionFailed {
        /// The path where detection was attempted.
        path: PathBuf,
        /// The reason for the failure.
        reason: String,
    },

    /// Failed to parse workspace configuration.
    #[error("failed to parse workspace config at '{path}': {source}")]
    ConfigParseFailed {
        /// The path to the configuration file.
        path: PathBuf,
        /// The underlying parse error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Workspace package not found.
    #[error("package '{name}' not found in workspace")]
    PackageNotFound {
        /// The name of the missing package.
        name: String,
    },

    /// Circular dependency detected.
    #[error("circular dependency detected: {}", packages.join(" -> "))]
    CircularDependency {
        /// The packages involved in the cycle.
        packages: Vec<String>,
    },
}
```

**Estimated Effort**: 1 hour

---

#### Task 1.4: Define ProjectError

**Description**: Create error types specific to project operations.

**Acceptance Criteria**:
- [ ] `ProjectError` enum defined
- [ ] Variants for detection, validation, missing files
- [ ] Context information included
- [ ] Unit tests for each variant

**Implementation Details**:

```rust
/// Errors related to project detection and management.
#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    /// No project found at the given path.
    #[error("no project found at '{path}'")]
    NotFound {
        /// The path where detection was attempted.
        path: PathBuf,
    },

    /// Project root could not be determined.
    #[error("could not find project root from '{start_path}'")]
    RootNotFound {
        /// The starting path for the search.
        start_path: PathBuf,
    },

    /// Missing package.json file.
    #[error("package.json not found at '{path}'")]
    MissingPackageJson {
        /// The expected path of the package.json.
        path: PathBuf,
    },

    /// Invalid package.json content.
    #[error("invalid package.json at '{path}': {reason}")]
    InvalidPackageJson {
        /// The path to the package.json.
        path: PathBuf,
        /// The reason it's invalid.
        reason: String,
    },

    /// Project validation failed.
    #[error("project validation failed: {}", errors.join(", "))]
    ValidationFailed {
        /// List of validation errors.
        errors: Vec<String>,
    },
}
```

**Estimated Effort**: 1 hour

---

#### Task 1.5: Define IoError Wrapper

**Description**: Create wrapper for std::io::Error with path context.

**Acceptance Criteria**:
- [ ] `IoError` struct defined with path and operation context
- [ ] Conversion from std::io::Error implemented
- [ ] Unit tests for error wrapping

**Implementation Details**:

```rust
/// I/O error with path context.
#[derive(Debug, thiserror::Error)]
#[error("{operation} failed for '{path}': {source}")]
pub struct IoError {
    /// The path involved in the operation.
    pub path: PathBuf,
    /// The operation that failed.
    pub operation: &'static str,
    /// The underlying I/O error.
    #[source]
    pub source: std::io::Error,
}

impl IoError {
    /// Creates a new IoError for a read operation.
    pub fn read(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self {
            path: path.into(),
            operation: "read",
            source,
        }
    }

    /// Creates a new IoError for a directory listing operation.
    pub fn read_dir(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self {
            path: path.into(),
            operation: "read directory",
            source,
        }
    }
}
```

**Estimated Effort**: 1 hour

---

### Epic 2: Configuration Module

**Goal**: Implement configuration types for detection behavior.

#### Task 2.1: Define PackageManagerConfig

**Description**: Create configuration for package manager detection.

**Acceptance Criteria**:
- [ ] `PackageManagerConfig` struct defined
- [ ] Sensible defaults implemented
- [ ] Serialization/deserialization support
- [ ] Unit tests for defaults and custom config

**Implementation Details**:

```rust
// src/config/mod.rs

//! # Configuration Module
//!
//! ## What
//! Configuration types for controlling detection behavior.
//!
//! ## How
//! Provides structs with serde support and Default implementations.
//!
//! ## Why
//! Allows customization of detection logic without code changes.

mod detection;
#[cfg(test)]
mod tests;

pub use detection::*;
```

```rust
// src/config/detection.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for package manager detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManagerConfig {
    /// Order in which to check for package managers.
    /// First match wins.
    #[serde(default = "PackageManagerConfig::default_detection_order")]
    pub detection_order: Vec<String>,

    /// Custom lock file names for each package manager.
    #[serde(default)]
    pub custom_lock_files: HashMap<String, String>,

    /// Whether to check environment variables for package manager hint.
    #[serde(default)]
    pub detect_from_env: bool,

    /// Environment variable name to check for package manager hint.
    #[serde(default = "PackageManagerConfig::default_env_var")]
    pub env_var_name: String,

    /// Fallback package manager if none detected.
    #[serde(default)]
    pub fallback: Option<String>,
}

impl PackageManagerConfig {
    fn default_detection_order() -> Vec<String> {
        vec![
            "pnpm".to_string(),
            "yarn".to_string(),
            "bun".to_string(),
            "deno".to_string(),
            "npm".to_string(),
        ]
    }

    fn default_env_var() -> String {
        "WORKSPACE_PACKAGE_MANAGER".to_string()
    }
}

impl Default for PackageManagerConfig {
    fn default() -> Self {
        Self {
            detection_order: Self::default_detection_order(),
            custom_lock_files: HashMap::new(),
            detect_from_env: false,
            env_var_name: Self::default_env_var(),
            fallback: None,
        }
    }
}
```

**Files to Create/Modify**:
- `crates/core/src/config/mod.rs`
- `crates/core/src/config/detection.rs`
- `crates/core/src/config/tests.rs`

**Estimated Effort**: 1.5 hours

---

#### Task 2.2: Define MonorepoConfig

**Description**: Create configuration for monorepo detection.

**Acceptance Criteria**:
- [ ] `MonorepoConfig` struct defined
- [ ] Workspace patterns configurable
- [ ] Exclusion patterns configurable
- [ ] Search depth configurable
- [ ] Unit tests for configuration

**Implementation Details**:

```rust
/// Configuration for monorepo detection and analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonorepoConfig {
    /// Additional workspace directory patterns to search.
    #[serde(default)]
    pub workspace_patterns: Vec<String>,

    /// Additional directories to check for packages.
    #[serde(default)]
    pub package_directories: Vec<String>,

    /// Patterns to exclude from package detection.
    #[serde(default = "MonorepoConfig::default_exclude")]
    pub exclude_patterns: Vec<String>,

    /// Maximum depth for recursive package search.
    #[serde(default = "MonorepoConfig::default_search_depth")]
    pub max_search_depth: usize,

    /// Whether to follow symlinks during search.
    #[serde(default)]
    pub follow_symlinks: bool,
}

impl MonorepoConfig {
    /// Default search depth.
    pub const DEFAULT_SEARCH_DEPTH: usize = 5;

    fn default_exclude() -> Vec<String> {
        vec![
            "**/node_modules/**".to_string(),
            "**/.*/**".to_string(),
            "**/dist/**".to_string(),
            "**/build/**".to_string(),
        ]
    }

    fn default_search_depth() -> usize {
        Self::DEFAULT_SEARCH_DEPTH
    }
}

impl Default for MonorepoConfig {
    fn default() -> Self {
        Self {
            workspace_patterns: Vec::new(),
            package_directories: Vec::new(),
            exclude_patterns: Self::default_exclude(),
            max_search_depth: Self::DEFAULT_SEARCH_DEPTH,
            follow_symlinks: false,
        }
    }
}
```

**Estimated Effort**: 1 hour

---

#### Task 2.3: Define DetectionConfig

**Description**: Create unified detection configuration.

**Acceptance Criteria**:
- [ ] `DetectionConfig` struct combining all configs
- [ ] Builder pattern for ergonomic construction
- [ ] Unit tests for builder and defaults

**Implementation Details**:

```rust
/// Unified configuration for all detection operations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DetectionConfig {
    /// Package manager detection configuration.
    #[serde(default)]
    pub package_manager: PackageManagerConfig,

    /// Monorepo detection configuration.
    #[serde(default)]
    pub monorepo: MonorepoConfig,
}

impl DetectionConfig {
    /// Creates a new builder for DetectionConfig.
    pub fn builder() -> DetectionConfigBuilder {
        DetectionConfigBuilder::default()
    }
}

/// Builder for DetectionConfig.
#[derive(Debug, Default)]
pub struct DetectionConfigBuilder {
    config: DetectionConfig,
}

impl DetectionConfigBuilder {
    /// Sets the package manager configuration.
    pub fn package_manager(mut self, config: PackageManagerConfig) -> Self {
        self.config.package_manager = config;
        self
    }

    /// Sets the monorepo configuration.
    pub fn monorepo(mut self, config: MonorepoConfig) -> Self {
        self.config.monorepo = config;
        self
    }

    /// Builds the configuration.
    pub fn build(self) -> DetectionConfig {
        self.config
    }
}
```

**Estimated Effort**: 1 hour

---

### Epic 3: Node Module

**Goal**: Implement package manager abstractions and repository types.

#### Task 3.1: Define PackageManagerKind Enum

**Description**: Create the enumeration for package manager types.

**Acceptance Criteria**:
- [ ] `PackageManagerKind` enum with Npm, Yarn, Pnpm, Bun, Deno variants
- [ ] Methods: `command()`, `lock_file()`, `name()`, `supports_workspaces()`
- [ ] `FromStr` and `Display` implementations
- [ ] Serialization support
- [ ] Comprehensive unit tests

**Implementation Details**:

```rust
// src/node/mod.rs

//! # Node Module
//!
//! ## What
//! Abstractions for Node.js package managers and repository types.
//!
//! ## How
//! Provides enums and structs that model the Node.js ecosystem.
//!
//! ## Why
//! Creates a type-safe foundation for working with Node.js projects.

mod package_manager;
mod repository;
mod types;
#[cfg(test)]
mod tests;

pub use package_manager::*;
pub use repository::*;
pub use types::*;
```

```rust
// src/node/types.rs

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents the type of package manager used in a Node.js project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageManagerKind {
    /// npm - the default Node.js package manager.
    Npm,
    /// Yarn - fast, reliable package manager.
    Yarn,
    /// pnpm - efficient disk space package manager.
    Pnpm,
    /// Bun - fast all-in-one runtime and package manager.
    Bun,
    /// Deno - secure runtime with built-in tooling.
    Deno,
}

impl PackageManagerKind {
    /// Returns the command name for this package manager.
    ///
    /// # Examples
    ///
    /// ```
    /// use workspace_core::node::PackageManagerKind;
    ///
    /// assert_eq!(PackageManagerKind::Npm.command(), "npm");
    /// assert_eq!(PackageManagerKind::Pnpm.command(), "pnpm");
    /// ```
    #[must_use]
    pub const fn command(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Yarn => "yarn",
            Self::Pnpm => "pnpm",
            Self::Bun => "bun",
            Self::Deno => "deno",
        }
    }

    /// Returns the lock file name for this package manager.
    ///
    /// # Examples
    ///
    /// ```
    /// use workspace_core::node::PackageManagerKind;
    ///
    /// assert_eq!(PackageManagerKind::Npm.lock_file(), "package-lock.json");
    /// assert_eq!(PackageManagerKind::Yarn.lock_file(), "yarn.lock");
    /// ```
    #[must_use]
    pub const fn lock_file(self) -> &'static str {
        match self {
            Self::Npm => "package-lock.json",
            Self::Yarn => "yarn.lock",
            Self::Pnpm => "pnpm-lock.yaml",
            Self::Bun => "bun.lockb",
            Self::Deno => "deno.lock",
        }
    }

    /// Returns a human-readable name for this package manager.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Yarn => "yarn",
            Self::Pnpm => "pnpm",
            Self::Bun => "bun",
            Self::Deno => "deno",
        }
    }

    /// Returns whether this package manager supports workspaces natively.
    #[must_use]
    pub const fn supports_workspaces(self) -> bool {
        match self {
            Self::Npm => true,
            Self::Yarn => true,
            Self::Pnpm => true,
            Self::Bun => true,
            Self::Deno => true,
        }
    }

    /// Returns the workspace config file for this package manager, if any.
    #[must_use]
    pub const fn workspace_config_file(self) -> Option<&'static str> {
        match self {
            Self::Npm => None, // Uses package.json workspaces field
            Self::Yarn => None, // Uses package.json workspaces field
            Self::Pnpm => Some("pnpm-workspace.yaml"),
            Self::Bun => None, // Uses package.json workspaces field
            Self::Deno => Some("deno.json"),
        }
    }

    /// Returns all available package manager kinds.
    #[must_use]
    pub const fn all() -> &'static [PackageManagerKind] {
        &[
            Self::Npm,
            Self::Yarn,
            Self::Pnpm,
            Self::Bun,
            Self::Deno,
        ]
    }
}

impl fmt::Display for PackageManagerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for PackageManagerKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "npm" => Ok(Self::Npm),
            "yarn" => Ok(Self::Yarn),
            "pnpm" => Ok(Self::Pnpm),
            "bun" => Ok(Self::Bun),
            "deno" => Ok(Self::Deno),
            _ => Err(format!("unknown package manager: {s}")),
        }
    }
}
```

**Files to Create/Modify**:
- `crates/core/src/node/mod.rs`
- `crates/core/src/node/types.rs`
- `crates/core/src/node/tests.rs`

**Estimated Effort**: 2 hours

---

#### Task 3.2: Define PackageManager Struct

**Description**: Create the struct representing a detected package manager.

**Acceptance Criteria**:
- [ ] `PackageManager` struct with kind and root path
- [ ] Constructor and accessor methods
- [ ] Derived paths (lock file, workspace config)
- [ ] Unit tests

**Implementation Details**:

```rust
// src/node/package_manager.rs

use std::path::{Path, PathBuf};
use crate::error::{PackageManagerError, Result};
use crate::config::PackageManagerConfig;
use super::PackageManagerKind;

/// Represents a detected package manager in a Node.js project.
///
/// # Examples
///
/// ```
/// use workspace_core::node::{PackageManager, PackageManagerKind};
/// use std::path::Path;
///
/// let pm = PackageManager::new(PackageManagerKind::Pnpm, "/path/to/project");
/// assert_eq!(pm.kind(), PackageManagerKind::Pnpm);
/// assert_eq!(pm.command(), "pnpm");
/// ```
#[derive(Debug, Clone)]
pub struct PackageManager {
    kind: PackageManagerKind,
    root: PathBuf,
}

impl PackageManager {
    /// Creates a new PackageManager instance.
    ///
    /// # Arguments
    ///
    /// * `kind` - The type of package manager
    /// * `root` - The root directory where the package manager was detected
    pub fn new(kind: PackageManagerKind, root: impl Into<PathBuf>) -> Self {
        Self {
            kind,
            root: root.into(),
        }
    }

    /// Detects the package manager at the given path using default configuration.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to search for a package manager
    ///
    /// # Errors
    ///
    /// Returns an error if no package manager could be detected.
    pub fn detect(path: impl AsRef<Path>) -> Result<Self> {
        Self::detect_with_config(path, &PackageManagerConfig::default())
    }

    /// Detects the package manager at the given path using custom configuration.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to search for a package manager
    /// * `config` - Configuration for detection behavior
    ///
    /// # Errors
    ///
    /// Returns an error if no package manager could be detected.
    pub fn detect_with_config(
        path: impl AsRef<Path>,
        config: &PackageManagerConfig,
    ) -> Result<Self> {
        let path = path.as_ref();

        // Check environment variable first if enabled
        if config.detect_from_env {
            if let Ok(env_pm) = std::env::var(&config.env_var_name) {
                if let Ok(kind) = env_pm.parse::<PackageManagerKind>() {
                    return Ok(Self::new(kind, path));
                }
            }
        }

        // Check for lock files in configured order
        for pm_name in &config.detection_order {
            let kind: PackageManagerKind = match pm_name.parse() {
                Ok(k) => k,
                Err(_) => continue,
            };

            let lock_file = config
                .custom_lock_files
                .get(pm_name)
                .map(String::as_str)
                .unwrap_or_else(|| kind.lock_file());

            if path.join(lock_file).exists() {
                return Ok(Self::new(kind, path));
            }
        }

        // Try fallback if configured
        if let Some(fallback) = &config.fallback {
            if let Ok(kind) = fallback.parse::<PackageManagerKind>() {
                return Ok(Self::new(kind, path));
            }
        }

        Err(PackageManagerError::NotFound {
            path: path.to_path_buf(),
        }.into())
    }

    /// Returns the kind of package manager.
    #[must_use]
    pub fn kind(&self) -> PackageManagerKind {
        self.kind
    }

    /// Returns the root directory path.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the command name for this package manager.
    #[must_use]
    pub fn command(&self) -> &'static str {
        self.kind.command()
    }

    /// Returns the lock file name for this package manager.
    #[must_use]
    pub fn lock_file(&self) -> &'static str {
        self.kind.lock_file()
    }

    /// Returns the full path to the lock file.
    #[must_use]
    pub fn lock_file_path(&self) -> PathBuf {
        self.root.join(self.lock_file())
    }

    /// Returns whether this package manager supports workspaces.
    #[must_use]
    pub fn supports_workspaces(&self) -> bool {
        self.kind.supports_workspaces()
    }

    /// Returns the workspace config file path if applicable.
    #[must_use]
    pub fn workspace_config_path(&self) -> Option<PathBuf> {
        self.kind
            .workspace_config_file()
            .map(|f| self.root.join(f))
    }
}
```

**Files to Create/Modify**:
- `crates/core/src/node/package_manager.rs`

**Estimated Effort**: 2 hours

---

#### Task 3.3: Define RepoKind Enum

**Description**: Create the enumeration for repository types.

**Acceptance Criteria**:
- [ ] `RepoKind` enum with Simple and Monorepo variants
- [ ] Methods: `is_monorepo()`, `monorepo_kind()`, `name()`
- [ ] Unit tests for all methods

**Implementation Details**:

```rust
// Add to src/node/types.rs

use crate::monorepo::MonorepoKind;

/// Represents the type of Node.js repository.
///
/// # Examples
///
/// ```
/// use workspace_core::node::RepoKind;
/// use workspace_core::monorepo::MonorepoKind;
///
/// let simple = RepoKind::Simple;
/// assert!(!simple.is_monorepo());
///
/// let mono = RepoKind::Monorepo(MonorepoKind::Pnpm);
/// assert!(mono.is_monorepo());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoKind {
    /// Simple repository with a single package.json.
    Simple,
    /// Monorepo with multiple packages.
    Monorepo(MonorepoKind),
}

impl RepoKind {
    /// Returns a human-readable name for the repository kind.
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Simple => "simple".to_string(),
            Self::Monorepo(kind) => format!("{} monorepo", kind.name()),
        }
    }

    /// Returns whether this is a monorepo.
    #[must_use]
    pub fn is_monorepo(&self) -> bool {
        matches!(self, Self::Monorepo(_))
    }

    /// Returns the monorepo kind if this is a monorepo.
    #[must_use]
    pub fn monorepo_kind(&self) -> Option<&MonorepoKind> {
        match self {
            Self::Simple => None,
            Self::Monorepo(kind) => Some(kind),
        }
    }

    /// Checks if this matches a specific monorepo kind.
    #[must_use]
    pub fn is_monorepo_kind(&self, kind: &MonorepoKind) -> bool {
        match self {
            Self::Simple => false,
            Self::Monorepo(k) => k == kind,
        }
    }
}

impl fmt::Display for RepoKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
```

**Estimated Effort**: 1 hour

---

#### Task 3.4: Define RepositoryInfo Trait

**Description**: Create the trait for repository information.

**Acceptance Criteria**:
- [ ] `RepositoryInfo` trait defined
- [ ] Methods: `repo_kind()`, `name()`, `is_monorepo()`, `root()`
- [ ] Unit tests with mock implementation

**Implementation Details**:

```rust
// src/node/repository.rs

use std::path::Path;
use super::RepoKind;

/// Trait providing information about repository characteristics.
///
/// This trait enables polymorphic access to repository information
/// regardless of the concrete project type.
pub trait RepositoryInfo {
    /// Returns the repository kind.
    fn repo_kind(&self) -> &RepoKind;

    /// Returns the repository name.
    fn name(&self) -> String;

    /// Returns whether this is a monorepo.
    fn is_monorepo(&self) -> bool {
        self.repo_kind().is_monorepo()
    }

    /// Returns the root directory of the repository.
    fn root(&self) -> &Path;
}
```

**Estimated Effort**: 1 hour

---

### Epic 4: Monorepo Module

**Goal**: Implement monorepo types, detection, and workspace analysis.

#### Task 4.1: Define MonorepoKind Enum

**Description**: Create the enumeration for monorepo types.

**Acceptance Criteria**:
- [ ] `MonorepoKind` enum with Npm, Yarn, Pnpm, Bun, Deno, Custom variants
- [ ] Methods: `name()`, `config_file()`
- [ ] Serialization support
- [ ] Unit tests

**Implementation Details**:

```rust
// src/monorepo/mod.rs

//! # Monorepo Module
//!
//! ## What
//! Types and detection for monorepo structures.
//!
//! ## How
//! Provides enums and structs for representing monorepo configurations
//! and detecting workspace packages.
//!
//! ## Why
//! Enables comprehensive monorepo analysis and package discovery.

mod types;
mod detector;
mod descriptor;
mod workspace;
#[cfg(test)]
mod tests;

pub use types::*;
pub use detector::*;
pub use descriptor::*;
pub use workspace::*;
```

```rust
// src/monorepo/types.rs

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the type of monorepo system being used.
///
/// # Examples
///
/// ```
/// use workspace_core::monorepo::MonorepoKind;
///
/// let pnpm = MonorepoKind::Pnpm;
/// assert_eq!(pnpm.name(), "pnpm");
/// assert_eq!(pnpm.config_file(), "pnpm-workspace.yaml");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MonorepoKind {
    /// npm workspaces (package.json workspaces field).
    Npm,
    /// Yarn workspaces (package.json workspaces field).
    Yarn,
    /// pnpm workspaces (pnpm-workspace.yaml).
    Pnpm,
    /// Bun workspaces (package.json workspaces field).
    Bun,
    /// Deno workspaces (deno.json).
    Deno,
    /// Custom monorepo configuration.
    Custom {
        /// Name of the custom monorepo system.
        name: String,
        /// Configuration file for the system.
        config_file: String,
    },
}

impl MonorepoKind {
    /// Returns a human-readable name for this monorepo kind.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Npm => "npm",
            Self::Yarn => "yarn",
            Self::Pnpm => "pnpm",
            Self::Bun => "bun",
            Self::Deno => "deno",
            Self::Custom { name, .. } => name,
        }
    }

    /// Returns the primary configuration file for this monorepo kind.
    #[must_use]
    pub fn config_file(&self) -> &str {
        match self {
            Self::Npm | Self::Yarn | Self::Bun => "package.json",
            Self::Pnpm => "pnpm-workspace.yaml",
            Self::Deno => "deno.json",
            Self::Custom { config_file, .. } => config_file,
        }
    }

    /// Creates a custom monorepo kind.
    #[must_use]
    pub fn custom(name: impl Into<String>, config_file: impl Into<String>) -> Self {
        Self::Custom {
            name: name.into(),
            config_file: config_file.into(),
        }
    }

    /// Returns all standard monorepo kinds.
    #[must_use]
    pub const fn all_standard() -> &'static [MonorepoKind] {
        &[
            Self::Npm,
            Self::Yarn,
            Self::Pnpm,
            Self::Bun,
            Self::Deno,
        ]
    }
}

impl fmt::Display for MonorepoKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
```

**Files to Create/Modify**:
- `crates/core/src/monorepo/mod.rs`
- `crates/core/src/monorepo/types.rs`

**Estimated Effort**: 1.5 hours

---

#### Task 4.2: Define WorkspacePackage Struct

**Description**: Create the struct representing a package in a workspace.

**Acceptance Criteria**:
- [ ] `WorkspacePackage` struct with name, version, location, dependencies
- [ ] Methods for workspace dependency queries
- [ ] Serialization support
- [ ] Unit tests

**Implementation Details**:

```rust
// Add to src/monorepo/types.rs

use std::path::PathBuf;
use std::collections::HashSet;

/// Represents a single package within a monorepo workspace.
///
/// # Examples
///
/// ```
/// use workspace_core::monorepo::WorkspacePackage;
/// use std::path::PathBuf;
///
/// let pkg = WorkspacePackage::new(
///     "my-package",
///     "1.0.0",
///     PathBuf::from("packages/my-package"),
///     PathBuf::from("/repo/packages/my-package"),
/// );
/// assert_eq!(pkg.name(), "my-package");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePackage {
    /// Name of the package.
    name: String,
    /// Version of the package.
    version: String,
    /// Location relative to the monorepo root.
    location: PathBuf,
    /// Absolute path to the package.
    absolute_path: PathBuf,
    /// Names of workspace packages this depends on.
    workspace_dependencies: HashSet<String>,
    /// Names of workspace packages this dev-depends on.
    workspace_dev_dependencies: HashSet<String>,
}

impl WorkspacePackage {
    /// Creates a new WorkspacePackage.
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        location: PathBuf,
        absolute_path: PathBuf,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            location,
            absolute_path,
            workspace_dependencies: HashSet::new(),
            workspace_dev_dependencies: HashSet::new(),
        }
    }

    /// Returns the package name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the package version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the location relative to the monorepo root.
    #[must_use]
    pub fn location(&self) -> &Path {
        &self.location
    }

    /// Returns the absolute path.
    #[must_use]
    pub fn absolute_path(&self) -> &Path {
        &self.absolute_path
    }

    /// Returns the workspace dependencies.
    #[must_use]
    pub fn workspace_dependencies(&self) -> &HashSet<String> {
        &self.workspace_dependencies
    }

    /// Returns the workspace dev dependencies.
    #[must_use]
    pub fn workspace_dev_dependencies(&self) -> &HashSet<String> {
        &self.workspace_dev_dependencies
    }

    /// Adds a workspace dependency.
    pub fn add_workspace_dependency(&mut self, name: impl Into<String>) {
        self.workspace_dependencies.insert(name.into());
    }

    /// Adds a workspace dev dependency.
    pub fn add_workspace_dev_dependency(&mut self, name: impl Into<String>) {
        self.workspace_dev_dependencies.insert(name.into());
    }

    /// Checks if this package depends on another workspace package.
    #[must_use]
    pub fn depends_on(&self, package_name: &str) -> bool {
        self.workspace_dependencies.contains(package_name)
            || self.workspace_dev_dependencies.contains(package_name)
    }

    /// Returns all workspace dependencies (prod and dev).
    #[must_use]
    pub fn all_workspace_dependencies(&self) -> HashSet<&str> {
        self.workspace_dependencies
            .iter()
            .chain(self.workspace_dev_dependencies.iter())
            .map(String::as_str)
            .collect()
    }
}
```

**Estimated Effort**: 1.5 hours

---

#### Task 4.3: Define MonorepoDescriptor Struct

**Description**: Create the struct representing a complete monorepo.

**Acceptance Criteria**:
- [ ] `MonorepoDescriptor` struct with kind, root, packages
- [ ] Methods for package lookup and dependency graph
- [ ] Implements `RepositoryInfo` trait
- [ ] Unit tests

**Implementation Details**:

```rust
// src/monorepo/descriptor.rs

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde_json::Value as JsonValue;
use crate::node::{PackageManager, RepositoryInfo, RepoKind};
use super::{MonorepoKind, WorkspacePackage};

/// Describes a complete monorepo structure.
///
/// # Examples
///
/// ```ignore
/// use workspace_core::monorepo::{MonorepoDescriptor, MonorepoKind};
///
/// let descriptor = MonorepoDescriptor::new(
///     MonorepoKind::Pnpm,
///     PathBuf::from("/path/to/repo"),
///     vec![/* packages */],
///     None,
///     None,
/// );
/// assert_eq!(descriptor.kind(), &MonorepoKind::Pnpm);
/// ```
#[derive(Debug)]
pub struct MonorepoDescriptor {
    kind: MonorepoKind,
    root: PathBuf,
    packages: Vec<WorkspacePackage>,
    package_index: HashMap<String, usize>,
    package_manager: Option<PackageManager>,
    package_json: Option<JsonValue>,
}

impl MonorepoDescriptor {
    /// Creates a new MonorepoDescriptor.
    pub fn new(
        kind: MonorepoKind,
        root: PathBuf,
        packages: Vec<WorkspacePackage>,
        package_manager: Option<PackageManager>,
        package_json: Option<JsonValue>,
    ) -> Self {
        let package_index = packages
            .iter()
            .enumerate()
            .map(|(i, p)| (p.name().to_string(), i))
            .collect();

        Self {
            kind,
            root,
            packages,
            package_index,
            package_manager,
            package_json,
        }
    }

    /// Returns the monorepo kind.
    #[must_use]
    pub fn kind(&self) -> &MonorepoKind {
        &self.kind
    }

    /// Returns the root path.
    #[must_use]
    pub fn root_path(&self) -> &Path {
        &self.root
    }

    /// Returns all packages.
    #[must_use]
    pub fn packages(&self) -> &[WorkspacePackage] {
        &self.packages
    }

    /// Returns the number of packages.
    #[must_use]
    pub fn package_count(&self) -> usize {
        self.packages.len()
    }

    /// Gets a package by name.
    #[must_use]
    pub fn get_package(&self, name: &str) -> Option<&WorkspacePackage> {
        self.package_index.get(name).map(|&i| &self.packages[i])
    }

    /// Returns the package manager if detected.
    #[must_use]
    pub fn package_manager(&self) -> Option<&PackageManager> {
        self.package_manager.as_ref()
    }

    /// Returns the root package.json content if available.
    #[must_use]
    pub fn package_json(&self) -> Option<&JsonValue> {
        self.package_json.as_ref()
    }

    /// Generates a dependency graph.
    ///
    /// Returns a map where keys are package names and values are
    /// lists of packages that depend on them.
    #[must_use]
    pub fn dependency_graph(&self) -> HashMap<&str, Vec<&WorkspacePackage>> {
        let mut graph: HashMap<&str, Vec<&WorkspacePackage>> = HashMap::new();

        for package in &self.packages {
            for dep in package.all_workspace_dependencies() {
                graph.entry(dep).or_default().push(package);
            }
        }

        graph
    }

    /// Finds packages that depend on a given package.
    #[must_use]
    pub fn dependents_of(&self, package_name: &str) -> Vec<&WorkspacePackage> {
        self.packages
            .iter()
            .filter(|p| p.depends_on(package_name))
            .collect()
    }

    /// Finds the package containing a given path.
    #[must_use]
    pub fn find_package_for_path(&self, path: &Path) -> Option<&WorkspacePackage> {
        // Normalize the path
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };

        self.packages.iter().find(|p| path.starts_with(p.absolute_path()))
    }

    /// Returns an iterator over package names.
    pub fn package_names(&self) -> impl Iterator<Item = &str> {
        self.packages.iter().map(|p| p.name())
    }
}

impl RepositoryInfo for MonorepoDescriptor {
    fn repo_kind(&self) -> &RepoKind {
        // Note: This requires storing RepoKind or computing it
        // For now, we'll need to adjust the design
        todo!("Will be implemented with project integration")
    }

    fn name(&self) -> String {
        self.package_json
            .as_ref()
            .and_then(|pj| pj.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unnamed")
            .to_string()
    }

    fn root(&self) -> &Path {
        &self.root
    }
}
```

**Estimated Effort**: 2 hours

---

#### Task 4.4: Implement MonorepoDetector

**Description**: Create the detector for monorepo structures.

**Acceptance Criteria**:
- [ ] `MonorepoDetector` struct with detection methods
- [ ] Detects npm/yarn/bun workspaces from package.json
- [ ] Detects pnpm workspaces from pnpm-workspace.yaml
- [ ] Detects deno workspaces from deno.json
- [ ] Unit tests for each monorepo type

**Implementation Details**:

```rust
// src/monorepo/detector.rs

use std::path::{Path, PathBuf};
use std::fs;
use serde_json::Value as JsonValue;
use crate::config::MonorepoConfig;
use crate::error::{MonorepoError, Result, IoError};
use crate::node::{PackageManager, PackageManagerKind};
use super::{MonorepoKind, MonorepoDescriptor, WorkspacePackage};

/// Detects and analyzes monorepo structures.
///
/// # Examples
///
/// ```ignore
/// use workspace_core::monorepo::MonorepoDetector;
/// use std::path::Path;
///
/// let detector = MonorepoDetector::new();
/// if let Some(kind) = detector.detect_kind(Path::new("."))? {
///     println!("Found {} monorepo", kind);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MonorepoDetector {
    config: MonorepoConfig,
}

impl MonorepoDetector {
    /// Creates a new detector with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: MonorepoConfig::default(),
        }
    }

    /// Creates a new detector with custom configuration.
    #[must_use]
    pub fn with_config(config: MonorepoConfig) -> Self {
        Self { config }
    }

    /// Detects the monorepo kind at a path.
    ///
    /// Returns `None` if the path is not a monorepo root.
    pub fn detect_kind(&self, path: impl AsRef<Path>) -> Result<Option<MonorepoKind>> {
        let path = path.as_ref();

        // Check for pnpm-workspace.yaml first (most specific)
        if path.join("pnpm-workspace.yaml").exists() {
            return Ok(Some(MonorepoKind::Pnpm));
        }

        // Check for deno.json with workspace config
        if let Some(kind) = self.check_deno_workspace(path)? {
            return Ok(Some(kind));
        }

        // Check for package.json workspaces field
        if let Some(kind) = self.check_package_json_workspaces(path)? {
            return Ok(Some(kind));
        }

        Ok(None)
    }

    /// Checks if a path is a monorepo root.
    pub fn is_monorepo_root(&self, path: impl AsRef<Path>) -> Result<bool> {
        Ok(self.detect_kind(path)?.is_some())
    }

    /// Detects and analyzes a complete monorepo structure.
    pub fn detect(&self, path: impl AsRef<Path>) -> Result<MonorepoDescriptor> {
        let path = path.as_ref();

        let kind = self.detect_kind(path)?.ok_or_else(|| MonorepoError::NotMonorepo {
            path: path.to_path_buf(),
        })?;

        let package_manager = PackageManager::detect(path).ok();

        let package_json = self.read_package_json(path)?;

        let workspace_patterns = self.get_workspace_patterns(path, &kind)?;
        let packages = self.discover_packages(path, &workspace_patterns)?;

        Ok(MonorepoDescriptor::new(
            kind,
            path.to_path_buf(),
            packages,
            package_manager,
            package_json,
        ))
    }

    /// Finds the monorepo root by walking up from a path.
    pub fn find_root(&self, start: impl AsRef<Path>) -> Result<Option<(PathBuf, MonorepoKind)>> {
        let start = start.as_ref();
        let mut current = if start.is_absolute() {
            start.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| IoError::read_dir(start, e))?
                .join(start)
        };

        loop {
            if let Some(kind) = self.detect_kind(&current)? {
                return Ok(Some((current, kind)));
            }

            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => return Ok(None),
            }
        }
    }

    // Private helper methods

    fn check_deno_workspace(&self, path: &Path) -> Result<Option<MonorepoKind>> {
        let deno_json = path.join("deno.json");
        if !deno_json.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&deno_json)
            .map_err(|e| IoError::read(&deno_json, e))?;

        let json: JsonValue = serde_json::from_str(&content)
            .map_err(|e| MonorepoError::ConfigParseFailed {
                path: deno_json.clone(),
                source: Box::new(e),
            })?;

        if json.get("workspace").is_some() || json.get("workspaces").is_some() {
            return Ok(Some(MonorepoKind::Deno));
        }

        Ok(None)
    }

    fn check_package_json_workspaces(&self, path: &Path) -> Result<Option<MonorepoKind>> {
        let package_json_path = path.join("package.json");
        if !package_json_path.exists() {
            return Ok(None);
        }

        let json = self.read_package_json(path)?;
        let json = match json {
            Some(j) => j,
            None => return Ok(None),
        };

        // Check for workspaces field
        if json.get("workspaces").is_none() {
            return Ok(None);
        }

        // Determine specific kind based on lock file
        if path.join("pnpm-lock.yaml").exists() {
            return Ok(Some(MonorepoKind::Pnpm));
        }
        if path.join("yarn.lock").exists() {
            return Ok(Some(MonorepoKind::Yarn));
        }
        if path.join("bun.lockb").exists() {
            return Ok(Some(MonorepoKind::Bun));
        }

        // Default to npm workspaces
        Ok(Some(MonorepoKind::Npm))
    }

    fn read_package_json(&self, path: &Path) -> Result<Option<JsonValue>> {
        let package_json_path = path.join("package.json");
        if !package_json_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&package_json_path)
            .map_err(|e| IoError::read(&package_json_path, e))?;

        let json: JsonValue = serde_json::from_str(&content)
            .map_err(|e| MonorepoError::ConfigParseFailed {
                path: package_json_path,
                source: Box::new(e),
            })?;

        Ok(Some(json))
    }

    fn get_workspace_patterns(
        &self,
        root: &Path,
        kind: &MonorepoKind,
    ) -> Result<Vec<String>> {
        match kind {
            MonorepoKind::Pnpm => self.get_pnpm_patterns(root),
            MonorepoKind::Deno => self.get_deno_patterns(root),
            MonorepoKind::Npm | MonorepoKind::Yarn | MonorepoKind::Bun => {
                self.get_package_json_patterns(root)
            }
            MonorepoKind::Custom { .. } => {
                // Use configured patterns for custom monorepos
                Ok(self.config.workspace_patterns.clone())
            }
        }
    }

    fn get_pnpm_patterns(&self, root: &Path) -> Result<Vec<String>> {
        let config_path = root.join("pnpm-workspace.yaml");
        let content = fs::read_to_string(&config_path)
            .map_err(|e| IoError::read(&config_path, e))?;

        #[derive(serde::Deserialize)]
        struct PnpmWorkspace {
            packages: Vec<String>,
        }

        let config: PnpmWorkspace = serde_yaml::from_str(&content)
            .map_err(|e| MonorepoError::ConfigParseFailed {
                path: config_path,
                source: Box::new(e),
            })?;

        Ok(config.packages)
    }

    fn get_deno_patterns(&self, root: &Path) -> Result<Vec<String>> {
        let config_path = root.join("deno.json");
        let content = fs::read_to_string(&config_path)
            .map_err(|e| IoError::read(&config_path, e))?;

        let json: JsonValue = serde_json::from_str(&content)
            .map_err(|e| MonorepoError::ConfigParseFailed {
                path: config_path.clone(),
                source: Box::new(e),
            })?;

        // Deno supports both "workspace" and "workspaces"
        let patterns = json
            .get("workspace")
            .or_else(|| json.get("workspaces"))
            .and_then(|w| w.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        Ok(patterns)
    }

    fn get_package_json_patterns(&self, root: &Path) -> Result<Vec<String>> {
        let json = self.read_package_json(root)?
            .ok_or_else(|| MonorepoError::NotMonorepo { path: root.to_path_buf() })?;

        let workspaces = json.get("workspaces");

        let patterns = match workspaces {
            // Array format: ["packages/*", "apps/*"]
            Some(JsonValue::Array(arr)) => {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            }
            // Object format: { "packages": ["packages/*"] }
            Some(JsonValue::Object(obj)) => {
                obj.get("packages")
                    .and_then(|p| p.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(String::from)
                            .collect()
                    })
                    .unwrap_or_default()
            }
            _ => Vec::new(),
        };

        Ok(patterns)
    }

    fn discover_packages(
        &self,
        root: &Path,
        patterns: &[String],
    ) -> Result<Vec<WorkspacePackage>> {
        let mut packages = Vec::new();
        let all_package_names: std::collections::HashSet<String>;

        // First pass: discover all packages
        for pattern in patterns {
            // Skip exclusion patterns
            if pattern.starts_with('!') {
                continue;
            }

            let package_dirs = self.expand_pattern(root, pattern)?;
            for dir in package_dirs {
                if let Some(pkg) = self.read_workspace_package(root, &dir)? {
                    packages.push(pkg);
                }
            }
        }

        // Collect all package names for dependency resolution
        all_package_names = packages.iter().map(|p| p.name().to_string()).collect();

        // Second pass: resolve workspace dependencies
        for package in &mut packages {
            self.resolve_workspace_dependencies(package, &all_package_names)?;
        }

        Ok(packages)
    }

    fn expand_pattern(&self, root: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
        let full_pattern = root.join(pattern);
        let pattern_str = full_pattern.to_string_lossy();

        let mut dirs = Vec::new();

        for entry in glob::glob(&pattern_str).map_err(|e| MonorepoError::DetectionFailed {
            path: root.to_path_buf(),
            reason: format!("invalid glob pattern: {e}"),
        })? {
            match entry {
                Ok(path) => {
                    if path.is_dir() && path.join("package.json").exists() {
                        dirs.push(path);
                    }
                }
                Err(e) => {
                    // Log but continue - some paths may be inaccessible
                    log::warn!("Failed to access path in glob: {e}");
                }
            }
        }

        Ok(dirs)
    }

    fn read_workspace_package(
        &self,
        root: &Path,
        package_dir: &Path,
    ) -> Result<Option<WorkspacePackage>> {
        let package_json_path = package_dir.join("package.json");
        if !package_json_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&package_json_path)
            .map_err(|e| IoError::read(&package_json_path, e))?;

        let json: JsonValue = serde_json::from_str(&content)
            .map_err(|e| MonorepoError::ConfigParseFailed {
                path: package_json_path,
                source: Box::new(e),
            })?;

        let name = json
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unnamed")
            .to_string();

        let version = json
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();

        let location = package_dir
            .strip_prefix(root)
            .unwrap_or(package_dir)
            .to_path_buf();

        Ok(Some(WorkspacePackage::new(
            name,
            version,
            location,
            package_dir.to_path_buf(),
        )))
    }

    fn resolve_workspace_dependencies(
        &self,
        package: &mut WorkspacePackage,
        all_package_names: &std::collections::HashSet<String>,
    ) -> Result<()> {
        let package_json_path = package.absolute_path().join("package.json");
        let content = fs::read_to_string(&package_json_path)
            .map_err(|e| IoError::read(&package_json_path, e))?;

        let json: JsonValue = serde_json::from_str(&content)
            .map_err(|e| MonorepoError::ConfigParseFailed {
                path: package_json_path,
                source: Box::new(e),
            })?;

        // Check dependencies
        if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
            for dep_name in deps.keys() {
                if all_package_names.contains(dep_name) {
                    package.add_workspace_dependency(dep_name.clone());
                }
            }
        }

        // Check devDependencies
        if let Some(deps) = json.get("devDependencies").and_then(|d| d.as_object()) {
            for dep_name in deps.keys() {
                if all_package_names.contains(dep_name) {
                    package.add_workspace_dev_dependency(dep_name.clone());
                }
            }
        }

        Ok(())
    }
}

impl Default for MonorepoDetector {
    fn default() -> Self {
        Self::new()
    }
}
```

**Files to Create/Modify**:
- `crates/core/src/monorepo/detector.rs`

**Estimated Effort**: 4 hours

---

#### Task 4.5: Implement Workspace Configuration Parsing

**Description**: Create parsers for different workspace configuration formats.

**Acceptance Criteria**:
- [ ] Parse npm/yarn/bun workspaces from package.json
- [ ] Parse pnpm workspaces from pnpm-workspace.yaml
- [ ] Parse deno workspaces from deno.json
- [ ] Handle both array and object workspace formats
- [ ] Unit tests for each format

**Implementation Details**:

```rust
// src/monorepo/workspace.rs

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Workspace configuration from package.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PackageJsonWorkspaces {
    /// Simple array of patterns.
    Array(Vec<String>),
    /// Object with packages and optional nohoist.
    Object {
        /// Package patterns.
        packages: Vec<String>,
        /// Patterns to exclude from hoisting (yarn specific).
        #[serde(default)]
        nohoist: Vec<String>,
    },
}

impl PackageJsonWorkspaces {
    /// Returns the package patterns.
    #[must_use]
    pub fn patterns(&self) -> &[String] {
        match self {
            Self::Array(patterns) => patterns,
            Self::Object { packages, .. } => packages,
        }
    }
}

/// PNPM workspace configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnpmWorkspaceConfig {
    /// Package patterns.
    pub packages: Vec<String>,
}

/// Deno workspace configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenoWorkspaceConfig {
    /// Workspace members (can be "workspace" or "workspaces" field).
    #[serde(alias = "workspaces")]
    pub workspace: Option<Vec<String>>,
}
```

**Estimated Effort**: 2 hours

---

### Epic 5: Project Module

**Goal**: Implement unified project detection and management.

#### Task 5.1: Define ProjectKind Enum

**Description**: Create the enumeration for project types.

**Acceptance Criteria**:
- [ ] `ProjectKind` enum wrapping `RepoKind`
- [ ] Methods: `is_monorepo()`, `repo_kind()`, `name()`
- [ ] Unit tests

**Implementation Details**:

```rust
// src/project/mod.rs

//! # Project Module
//!
//! ## What
//! Unified project detection and management for Node.js projects.
//!
//! ## How
//! Provides a single entry point for detecting and working with
//! any type of Node.js project.
//!
//! ## Why
//! Simplifies project detection by providing a unified API
//! regardless of project structure.

mod types;
mod detector;
mod project;
#[cfg(test)]
mod tests;

pub use types::*;
pub use detector::*;
pub use project::*;
```

```rust
// src/project/types.rs

use crate::node::RepoKind;
use crate::monorepo::MonorepoKind;
use std::fmt;

/// Represents the type of Node.js project.
///
/// # Examples
///
/// ```
/// use workspace_core::project::ProjectKind;
/// use workspace_core::node::RepoKind;
///
/// let kind = ProjectKind::from(RepoKind::Simple);
/// assert!(!kind.is_monorepo());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectKind {
    /// A repository-based project.
    Repository(RepoKind),
}

impl ProjectKind {
    /// Returns a human-readable name for the project kind.
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Repository(repo) => repo.name(),
        }
    }

    /// Returns whether this is a monorepo project.
    #[must_use]
    pub fn is_monorepo(&self) -> bool {
        match self {
            Self::Repository(repo) => repo.is_monorepo(),
        }
    }

    /// Returns the repository kind.
    #[must_use]
    pub fn repo_kind(&self) -> &RepoKind {
        match self {
            Self::Repository(repo) => repo,
        }
    }

    /// Returns the monorepo kind if this is a monorepo.
    #[must_use]
    pub fn monorepo_kind(&self) -> Option<&MonorepoKind> {
        self.repo_kind().monorepo_kind()
    }
}

impl From<RepoKind> for ProjectKind {
    fn from(repo: RepoKind) -> Self {
        Self::Repository(repo)
    }
}

impl fmt::Display for ProjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Status of project validation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ValidationStatus {
    /// Project has not been validated.
    #[default]
    NotValidated,
    /// Project is valid.
    Valid,
    /// Project has warnings but is valid.
    Warning(Vec<String>),
    /// Project has errors and is invalid.
    Error(Vec<String>),
}

impl ValidationStatus {
    /// Returns whether the project is valid (no errors).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !matches!(self, Self::Error(_))
    }

    /// Returns whether there are warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        matches!(self, Self::Warning(_))
    }

    /// Returns whether there are errors.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        matches!(self, Self::Error(_))
    }
}
```

**Files to Create/Modify**:
- `crates/core/src/project/mod.rs`
- `crates/core/src/project/types.rs`

**Estimated Effort**: 1.5 hours

---

#### Task 5.2: Define Project Struct

**Description**: Create the struct representing a detected project.

**Acceptance Criteria**:
- [ ] `Project` struct with kind, root, package manager, package.json
- [ ] Implements `ProjectInfo` trait
- [ ] Access to dependencies
- [ ] Unit tests

**Implementation Details**:

```rust
// src/project/project.rs

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde_json::Value as JsonValue;
use crate::node::{PackageManager, RepoKind, RepositoryInfo};
use super::{ProjectKind, ValidationStatus};

/// Trait providing information about a project.
pub trait ProjectInfo {
    /// Returns the project root path.
    fn root(&self) -> &Path;

    /// Returns the package manager if detected.
    fn package_manager(&self) -> Option<&PackageManager>;

    /// Returns the package.json content if available.
    fn package_json(&self) -> Option<&JsonValue>;

    /// Returns the validation status.
    fn validation_status(&self) -> &ValidationStatus;

    /// Returns the project kind.
    fn kind(&self) -> &ProjectKind;
}

/// Represents a detected Node.js project.
///
/// # Examples
///
/// ```ignore
/// use workspace_core::project::Project;
///
/// let project = Project::detect(".")?;
/// println!("Project root: {}", project.root().display());
/// ```
#[derive(Debug)]
pub struct Project {
    root: PathBuf,
    kind: ProjectKind,
    package_manager: Option<PackageManager>,
    package_json: Option<JsonValue>,
    validation_status: ValidationStatus,
}

impl Project {
    /// Creates a new Project.
    pub(crate) fn new(
        root: PathBuf,
        kind: ProjectKind,
        package_manager: Option<PackageManager>,
        package_json: Option<JsonValue>,
    ) -> Self {
        Self {
            root,
            kind,
            package_manager,
            package_json,
            validation_status: ValidationStatus::NotValidated,
        }
    }

    /// Returns the project name from package.json.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.package_json
            .as_ref()
            .and_then(|pj| pj.get("name"))
            .and_then(|n| n.as_str())
    }

    /// Returns the project version from package.json.
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.package_json
            .as_ref()
            .and_then(|pj| pj.get("version"))
            .and_then(|v| v.as_str())
    }

    /// Returns whether this is a monorepo.
    #[must_use]
    pub fn is_monorepo(&self) -> bool {
        self.kind.is_monorepo()
    }

    /// Returns dependencies from package.json.
    #[must_use]
    pub fn dependencies(&self) -> HashMap<String, String> {
        self.extract_deps("dependencies")
    }

    /// Returns dev dependencies from package.json.
    #[must_use]
    pub fn dev_dependencies(&self) -> HashMap<String, String> {
        self.extract_deps("devDependencies")
    }

    /// Returns optional dependencies from package.json.
    #[must_use]
    pub fn optional_dependencies(&self) -> HashMap<String, String> {
        self.extract_deps("optionalDependencies")
    }

    /// Returns peer dependencies from package.json.
    #[must_use]
    pub fn peer_dependencies(&self) -> HashMap<String, String> {
        self.extract_deps("peerDependencies")
    }

    fn extract_deps(&self, field: &str) -> HashMap<String, String> {
        self.package_json
            .as_ref()
            .and_then(|pj| pj.get(field))
            .and_then(|d| d.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| {
                        v.as_str().map(|s| (k.clone(), s.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Sets the validation status.
    pub fn set_validation_status(&mut self, status: ValidationStatus) {
        self.validation_status = status;
    }
}

impl ProjectInfo for Project {
    fn root(&self) -> &Path {
        &self.root
    }

    fn package_manager(&self) -> Option<&PackageManager> {
        self.package_manager.as_ref()
    }

    fn package_json(&self) -> Option<&JsonValue> {
        self.package_json.as_ref()
    }

    fn validation_status(&self) -> &ValidationStatus {
        &self.validation_status
    }

    fn kind(&self) -> &ProjectKind {
        &self.kind
    }
}

impl RepositoryInfo for Project {
    fn repo_kind(&self) -> &RepoKind {
        self.kind.repo_kind()
    }

    fn name(&self) -> String {
        self.name().unwrap_or("unnamed").to_string()
    }

    fn root(&self) -> &Path {
        &self.root
    }
}
```

**Estimated Effort**: 2 hours

---

#### Task 5.3: Implement ProjectDetector

**Description**: Create the unified project detector.

**Acceptance Criteria**:
- [ ] `ProjectDetector` struct with detection methods
- [ ] Detects simple and monorepo projects
- [ ] Finds project root from subdirectories
- [ ] Configurable detection behavior
- [ ] Unit tests

**Implementation Details**:

```rust
// src/project/detector.rs

use std::path::{Path, PathBuf};
use std::fs;
use serde_json::Value as JsonValue;
use crate::config::DetectionConfig;
use crate::error::{ProjectError, Result, IoError};
use crate::node::{PackageManager, RepoKind};
use crate::monorepo::MonorepoDetector;
use super::{Project, ProjectKind};

/// Detects Node.js projects.
///
/// # Examples
///
/// ```ignore
/// use workspace_core::project::ProjectDetector;
///
/// let detector = ProjectDetector::new();
/// let project = detector.detect(".")?;
/// println!("Found {} project", project.kind());
/// ```
#[derive(Debug, Clone)]
pub struct ProjectDetector {
    config: DetectionConfig,
    monorepo_detector: MonorepoDetector,
}

impl ProjectDetector {
    /// Creates a new detector with default configuration.
    #[must_use]
    pub fn new() -> Self {
        let config = DetectionConfig::default();
        Self {
            monorepo_detector: MonorepoDetector::with_config(config.monorepo.clone()),
            config,
        }
    }

    /// Creates a new detector with custom configuration.
    #[must_use]
    pub fn with_config(config: DetectionConfig) -> Self {
        Self {
            monorepo_detector: MonorepoDetector::with_config(config.monorepo.clone()),
            config,
        }
    }

    /// Detects a project at the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to detect the project at
    ///
    /// # Errors
    ///
    /// Returns an error if no valid project is found.
    pub fn detect(&self, path: impl AsRef<Path>) -> Result<Project> {
        let path = path.as_ref();

        // Check for package.json
        let package_json_path = path.join("package.json");
        if !package_json_path.exists() {
            return Err(ProjectError::MissingPackageJson {
                path: package_json_path,
            }.into());
        }

        // Read package.json
        let package_json = self.read_package_json(path)?;

        // Detect package manager
        let package_manager = PackageManager::detect_with_config(
            path,
            &self.config.package_manager,
        ).ok();

        // Check if it's a monorepo
        let kind = if self.monorepo_detector.is_monorepo_root(path)? {
            let monorepo_kind = self.monorepo_detector
                .detect_kind(path)?
                .ok_or_else(|| ProjectError::NotFound {
                    path: path.to_path_buf(),
                })?;
            ProjectKind::Repository(RepoKind::Monorepo(monorepo_kind))
        } else {
            ProjectKind::Repository(RepoKind::Simple)
        };

        Ok(Project::new(
            path.to_path_buf(),
            kind,
            package_manager,
            package_json,
        ))
    }

    /// Detects the project kind at a path.
    ///
    /// This is faster than full detection as it doesn't parse package.json.
    pub fn detect_kind(&self, path: impl AsRef<Path>) -> Result<ProjectKind> {
        let path = path.as_ref();

        if !path.join("package.json").exists() {
            return Err(ProjectError::MissingPackageJson {
                path: path.join("package.json"),
            }.into());
        }

        if self.monorepo_detector.is_monorepo_root(path)? {
            let monorepo_kind = self.monorepo_detector
                .detect_kind(path)?
                .ok_or_else(|| ProjectError::NotFound {
                    path: path.to_path_buf(),
                })?;
            Ok(ProjectKind::Repository(RepoKind::Monorepo(monorepo_kind)))
        } else {
            Ok(ProjectKind::Repository(RepoKind::Simple))
        }
    }

    /// Finds the project root by walking up from a starting path.
    ///
    /// # Arguments
    ///
    /// * `start` - The starting path to search from
    ///
    /// # Returns
    ///
    /// The project root path and kind, or None if no project found.
    pub fn find_root(&self, start: impl AsRef<Path>) -> Result<Option<(PathBuf, ProjectKind)>> {
        let start = start.as_ref();
        let mut current = if start.is_absolute() {
            start.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| IoError::read_dir(start, e))?
                .join(start)
        };

        loop {
            if current.join("package.json").exists() {
                let kind = self.detect_kind(&current)?;
                return Ok(Some((current, kind)));
            }

            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => return Ok(None),
            }
        }
    }

    /// Checks if a path is a valid project root.
    pub fn is_valid_project(&self, path: impl AsRef<Path>) -> bool {
        path.as_ref().join("package.json").exists()
    }

    fn read_package_json(&self, path: &Path) -> Result<Option<JsonValue>> {
        let package_json_path = path.join("package.json");
        if !package_json_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&package_json_path)
            .map_err(|e| IoError::read(&package_json_path, e))?;

        let json: JsonValue = serde_json::from_str(&content)
            .map_err(|e| ProjectError::InvalidPackageJson {
                path: package_json_path,
                reason: e.to_string(),
            })?;

        Ok(Some(json))
    }
}

impl Default for ProjectDetector {
    fn default() -> Self {
        Self::new()
    }
}
```

**Files to Create/Modify**:
- `crates/core/src/project/detector.rs`

**Estimated Effort**: 3 hours

---

### Epic 6: Integration & Polish

**Goal**: End-to-end tests, documentation review, and final polish.

#### Task 6.1: Create Integration Tests

**Description**: Create comprehensive E2E tests for the crate.

**Acceptance Criteria**:
- [ ] Test fixtures for simple project
- [ ] Test fixtures for npm/yarn/pnpm/bun/deno monorepos
- [ ] E2E tests for project detection
- [ ] E2E tests for monorepo analysis
- [ ] All tests pass on CI

**Test Fixtures Structure**:

```
tests/
├── fixtures/
│   ├── simple_npm/
│   │   ├── package.json
│   │   └── package-lock.json
│   ├── simple_yarn/
│   │   ├── package.json
│   │   └── yarn.lock
│   ├── simple_pnpm/
│   │   ├── package.json
│   │   └── pnpm-lock.yaml
│   ├── monorepo_npm/
│   │   ├── package.json
│   │   ├── package-lock.json
│   │   └── packages/
│   │       ├── pkg-a/package.json
│   │       └── pkg-b/package.json
│   ├── monorepo_pnpm/
│   │   ├── package.json
│   │   ├── pnpm-lock.yaml
│   │   ├── pnpm-workspace.yaml
│   │   └── packages/
│   │       ├── pkg-a/package.json
│   │       └── pkg-b/package.json
│   └── monorepo_deno/
│       ├── deno.json
│       └── packages/
│           ├── pkg-a/deno.json
│           └── pkg-b/deno.json
└── integration/
    ├── package_manager_e2e.rs
    ├── project_detection_e2e.rs
    └── monorepo_analysis_e2e.rs
```

**Estimated Effort**: 4 hours

---

#### Task 6.2: Documentation Review

**Description**: Review and complete all documentation.

**Acceptance Criteria**:
- [ ] Crate-level documentation complete
- [ ] All public items documented
- [ ] Examples compile and run
- [ ] README.md created for crate
- [ ] CHANGELOG.md initialized

**Estimated Effort**: 2 hours

---

#### Task 6.3: Final Cleanup and Release Preparation

**Description**: Final code review and release preparation.

**Acceptance Criteria**:
- [ ] All clippy warnings addressed
- [ ] All tests pass
- [ ] Documentation builds without warnings
- [ ] Version set correctly
- [ ] License file in place

**Estimated Effort**: 1 hour

---

## 4. Timeline Summary

| Epic | Description | Estimated Hours |
|------|-------------|-----------------|
| Epic 0 | Project Setup | 1.5 |
| Epic 1 | Error Module | 6 |
| Epic 2 | Configuration Module | 3.5 |
| Epic 3 | Node Module | 6 |
| Epic 4 | Monorepo Module | 11 |
| Epic 5 | Project Module | 6.5 |
| Epic 6 | Integration & Polish | 7 |
| **Total** | | **41.5 hours** |

---

## 5. Dependencies Between Tasks

```
Epic 0 (Setup)
    └── Epic 1 (Error)
        └── Epic 2 (Config)
            └── Epic 3 (Node)
                ├── Task 3.1-3.2 (PackageManager)
                │   └── Epic 4 (Monorepo)
                │       └── Epic 5 (Project)
                │           └── Epic 6 (Polish)
                └── Task 3.3-3.4 (RepoKind)
                    └── Epic 4 (Monorepo)
```

---

## 6. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Glob pattern edge cases | Medium | Medium | Comprehensive tests with real-world examples |
| Cross-platform path handling | Low | High | Use Path/PathBuf consistently, test on all OSes |
| Performance with large monorepos | Low | Medium | Benchmark early, optimize if needed |
| Deno workspace format changes | Medium | Low | Abstract format parsing, version check |

---

## 7. Success Metrics

| Metric | Target |
|--------|--------|
| Code coverage | > 80% |
| All P0 requirements | 100% implemented |
| All P1 requirements | 100% implemented |
| E2E tests | All passing |
| Documentation | All public items |
| Clippy | Zero warnings |
| Build time | < 30 seconds |

---

## 8. Appendix: File Structure

```
crates/core/
├── Cargo.toml
├── README.md
├── CHANGELOG.md
├── PRD.md
├── PLAN.md
├── src/
│   ├── lib.rs
│   ├── error/
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   └── tests.rs
│   ├── config/
│   │   ├── mod.rs
│   │   ├── detection.rs
│   │   └── tests.rs
│   ├── node/
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   ├── package_manager.rs
│   │   ├── repository.rs
│   │   └── tests.rs
│   ├── monorepo/
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   ├── detector.rs
│   │   ├── descriptor.rs
│   │   ├── workspace.rs
│   │   └── tests.rs
│   └── project/
│       ├── mod.rs
│       ├── types.rs
│       ├── project.rs
│       ├── detector.rs
│       └── tests.rs
└── tests/
    ├── fixtures/
    │   └── (test project fixtures)
    └── integration/
        ├── package_manager_e2e.rs
        ├── project_detection_e2e.rs
        └── monorepo_analysis_e2e.rs
```