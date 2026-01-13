# Implementation Plan: workspace-core

## Document Information

| Field | Value |
|-------|-------|
| **Crate Name** | `workspace-core` |
| **PRD Reference** | [PRD.md](./PRD.md) |
| **Status** | Ready |
| **Created** | 2026-01-12 |
| **Last Updated** | 2026-01-13 |

---

## 1. Overview

This document details the implementation plan for the `workspace-core` crate. The implementation is divided into epics, each containing multiple tasks. Tasks are designed to be atomic units of work that can be completed and committed independently.

### 1.1 Key Architectural Decisions

The following decisions were finalized during PRD development and must be followed during implementation:

| Decision | Choice | PRD Reference |
|----------|--------|---------------|
| Error handling | Single unified `Error` enum using `snafu` | §5.8, §8.3 |
| Logging | `log` crate facade, consumer initializes impl | §6.5 |
| Configuration | `DetectionConfig` with builder pattern | §5.7 |
| Path handling | Explicit `&Path` required, NO fallback to cwd | §8.2 |
| Filesystem | All ops via `workspace-fs` crate | §1.4.1 |
| Symlinks | Always follow (not configurable) | §5.7 FR-7.3.2 |
| Package Manager fallback | None - error if not detected | §5.7 FR-7.3.1 |
| packageManager vs lockfile mismatch | Error (not warning) | §3.4 |
| Module separation | Separate `package/` and `dependency/` modules | §7.1 |

### 1.2 External Dependencies

From PRD §1.4.2:

| Crate | Version | Purpose |
|-------|---------|---------|
| `snafu` | `0.8.9` | Error handling with context |
| `serde` | `1.0` | Serialization (with `derive` feature) |
| `serde_json` | `1.0` | JSON parsing |
| `serde_yaml_ng` | `0.10.0` | YAML parsing (note: `serde_yaml` is deprecated) |
| `log` | `0.4` | Logging facade |
| `semver` | `1.0` | Semantic versioning |
| `package-json` | `0.5.0` | Type-safe `package.json` parsing |
| `walkdir` | `2.0` | Directory traversal |
| `glob` | `0.3` | Glob pattern matching |

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
Single unified error type using `snafu` (PRD §5.8).

### Phase 2: Configuration Module
`DetectionConfig` with builder pattern (PRD §5.7).

### Phase 3: Repository Module
`RepoType` and `RepoKind` enums (PRD §5.1).

### Phase 4: Package Manager Module
`PackageManagerKind` and detection logic (PRD §5.2).

### Phase 5: Package & Dependency Modules
Package representation and dependency analysis (PRD §5.3, §5.4).

### Phase 6: Monorepo Module
Workspace detection and package discovery (PRD §5.6).

### Phase 7: Project Module
Unified project detection and representation (PRD §5.5).

### Phase 8: Integration & Polish
End-to-end tests, documentation review, and optimization.

---

## 3. Epic Breakdown

---

### Epic 0: Project Setup

**Goal**: Establish the crate structure, dependencies, and development configuration.

**PRD Context**: §1.4 Dependencies, §6.3 Compatibility (MSRV 1.90+, edition 2024), §6.4 Code Quality

---

#### Task 0.1: Create Crate Skeleton

**Description**: Initialize the crate with `Cargo.toml` and basic structure following the external dependencies defined in PRD §1.4.2.

**PRD References**:
- §1.4.1: Internal dependency on `workspace-fs`
- §1.4.2: External dependencies list (snafu, serde, etc.)
- §1.4.3: Development dependencies (tempfile)
- §6.3: Compatibility requirements (MSRV 1.90+, edition 2024)
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
name = "workspace-core"
version = "0.1.0"
edition = "2024"
rust-version = "1.90"
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
# Error handling (PRD §1.4.2)
snafu = "0.8.9"

# Serialization (PRD §1.4.2)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml_ng = "0.10.0"

# Logging (PRD §1.4.2, §6.5)
log = "0.4"

# Versioning (PRD §1.4.2)
semver = "1.0"

# Package.json parsing (PRD §1.4.2)
package-json = "0.5.0"

# Filesystem traversal (PRD §1.4.2)
walkdir = "2.0"
glob = "0.3"

# Internal dependency (PRD §1.4.1)
workspace-fs = { path = "../fs" }

[dev-dependencies]
# PRD §1.4.3
tempfile = "3.0"
```

**Files to Create**:
- `crates/core/Cargo.toml`
- `crates/core/src/lib.rs`

**Estimated Effort**: 1 hour

---

#### Task 0.2: Create Module Structure

**Description**: Create the directory structure and empty module files matching the architecture defined in PRD §7.1.

**PRD References**:
- §7.1: Module Structure (complete directory layout)
- §7.2: Dependency Graph (module dependencies)

**Acceptance Criteria**:
- [ ] All module directories created matching PRD §7.1
- [ ] All `mod.rs` files created with module declarations
- [ ] Crate compiles with empty modules
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Module Structure** (from PRD §7.1):

```
src/
├── lib.rs
├── error.rs              # Single unified Error enum (§5.8)
├── repo/                 # §5.1
│   ├── mod.rs
│   ├── repo_type.rs
│   ├── repo_kind.rs
│   └── tests.rs
├── package_manager/      # §5.2
│   ├── mod.rs
│   ├── kind.rs
│   ├── detector.rs
│   └── tests.rs
├── package/              # §5.3
│   ├── mod.rs
│   ├── package.rs
│   └── tests.rs
├── dependency/           # §5.4
│   ├── mod.rs
│   ├── types.rs
│   ├── parser.rs
│   ├── categorizer.rs
│   └── tests.rs
├── project/              # §5.5
│   ├── mod.rs
│   ├── detector.rs
│   ├── project.rs
│   └── tests.rs
├── monorepo/             # §5.6
│   ├── mod.rs
│   ├── detector.rs
│   ├── workspace.rs
│   └── tests.rs
└── config/               # §5.7
    ├── mod.rs
    ├── detection.rs
    ├── builder.rs
    └── tests.rs
```

**Files to Create**: All directories and `mod.rs` files as listed above.

**Estimated Effort**: 30 minutes

---

### Epic 1: Error Module

**Goal**: Implement a single unified `Error` enum using `snafu` with all error variants.

**PRD Context**: 
- §5.8: Error Handling requirements
- §8.3: Error Handling design principles
- FR-8.1.1 through FR-8.1.6: Specific requirements

**Key Requirements**:
- Single unified `Error` enum (one crate = one error type)
- Uses `snafu` for derive and context
- Implements `std::error::Error`, `Display`
- Implements `AsRef<str>` returning qualified variant name (e.g., `"Error::PackageManagerNotFound"`)
- Provides `Result<T>` type alias

---

#### Task 1.1: Define Unified Error Enum

**Description**: Create the single unified `Error` enum with all variants defined in PRD §5.8 FR-8.2.

**PRD References**:
- §5.8 FR-8.1.1: Single unified `Error` enum using `snafu`
- §5.8 FR-8.1.2: Context information (paths, reasons) in variants
- §5.8 FR-8.1.3: Implement `std::error::Error`
- §5.8 FR-8.1.4: Implement `AsRef<str>` returning qualified variant name
- §5.8 FR-8.1.5: Actionable error messages via `Display`
- §5.8 FR-8.1.6: `Result<T>` type alias
- §5.8 FR-8.2: Error variants table

**Error Variants** (from PRD §5.8 FR-8.2):

| Category | Variants |
|----------|----------|
| Repository | `RepoTypeDetection`, `RepoTypeUnknown` |
| Package Manager | `PackageManagerNotFound`, `PackageManagerMismatch` |
| Configuration | `ConfigParse`, `ConfigInvalid` |
| Project | `ProjectRootNotFound`, `ProjectNotFound` |
| Monorepo | `MonorepoNotDetected`, `WorkspacePackageNotFound`, `CircularDependency` |
| I/O | `Io` (wrapping errors from `workspace-fs`) |

**Acceptance Criteria**:
- [ ] `Error` enum defined with all variants from PRD §5.8 FR-8.2
- [ ] Uses `snafu` derive macros with `#[snafu(display("..."))]`
- [ ] All variants include context information (paths, reasons)
- [ ] `std::error::Error` implemented via snafu
- [ ] `AsRef<str>` implemented returning `"Error::VariantName"`
- [ ] `Result<T>` type alias defined
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Implementation Details**:

```rust
// src/error.rs

//! # Error Module
//!
//! ## What
//! Single unified error type for the workspace-core crate.
//!
//! ## How
//! Uses `snafu` for ergonomic error handling with context.
//! Implements `AsRef<str>` for variant name introspection.
//!
//! ## Why
//! Rust idiom: one crate = one error type. Structured errors
//! enable proper error handling and provide actionable information.

use snafu::Snafu;
use std::path::PathBuf;

/// Result type alias for workspace-core operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Unified error type for all workspace-core operations.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    // Repository errors
    #[snafu(display("failed to detect repository type at '{path}': {reason}"))]
    RepoTypeDetection {
        path: PathBuf,
        reason: String,
    },

    #[snafu(display("unknown repository type at '{path}'"))]
    RepoTypeUnknown {
        path: PathBuf,
    },

    // Package Manager errors
    #[snafu(display("no package manager found at '{path}'"))]
    PackageManagerNotFound {
        path: PathBuf,
    },

    #[snafu(display("package manager mismatch at '{path}': declared '{declared}' but found lock file for '{detected}'"))]
    PackageManagerMismatch {
        path: PathBuf,
        declared: String,
        detected: String,
    },

    // Configuration errors
    #[snafu(display("failed to parse configuration at '{path}': {reason}"))]
    ConfigParse {
        path: PathBuf,
        reason: String,
    },

    #[snafu(display("invalid configuration: {reason}"))]
    ConfigInvalid {
        reason: String,
    },

    // Project errors
    #[snafu(display("project root not found starting from '{start_path}'"))]
    ProjectRootNotFound {
        start_path: PathBuf,
    },

    #[snafu(display("no project found at '{path}'"))]
    ProjectNotFound {
        path: PathBuf,
    },

    // Monorepo errors
    #[snafu(display("'{path}' is not a monorepo"))]
    MonorepoNotDetected {
        path: PathBuf,
    },

    #[snafu(display("workspace package '{name}' not found"))]
    WorkspacePackageNotFound {
        name: String,
    },

    #[snafu(display("circular dependency detected: {}", packages.join(" -> ")))]
    CircularDependency {
        packages: Vec<String>,
    },

    // I/O errors (wrapping workspace-fs errors)
    #[snafu(display("{operation} failed for '{path}': {source}"))]
    Io {
        path: PathBuf,
        operation: String,
        source: std::io::Error,
    },
}

impl AsRef<str> for Error {
    fn as_ref(&self) -> &str {
        match self {
            Error::RepoTypeDetection { .. } => "Error::RepoTypeDetection",
            Error::RepoTypeUnknown { .. } => "Error::RepoTypeUnknown",
            Error::PackageManagerNotFound { .. } => "Error::PackageManagerNotFound",
            Error::PackageManagerMismatch { .. } => "Error::PackageManagerMismatch",
            Error::ConfigParse { .. } => "Error::ConfigParse",
            Error::ConfigInvalid { .. } => "Error::ConfigInvalid",
            Error::ProjectRootNotFound { .. } => "Error::ProjectRootNotFound",
            Error::ProjectNotFound { .. } => "Error::ProjectNotFound",
            Error::MonorepoNotDetected { .. } => "Error::MonorepoNotDetected",
            Error::WorkspacePackageNotFound { .. } => "Error::WorkspacePackageNotFound",
            Error::CircularDependency { .. } => "Error::CircularDependency",
            Error::Io { .. } => "Error::Io",
        }
    }
}
```

**Test Requirements**:
- Test `Display` output for each variant
- Test `AsRef<str>` returns correct qualified name for each variant
- Test error creation with context
- Test `std::error::Error` trait implementation

**Files to Create/Modify**:
- `crates/core/src/error.rs`

**Estimated Effort**: 2 hours

---

### Epic 2: Configuration Module

**Goal**: Implement `DetectionConfig` with builder pattern.

**PRD Context**:
- §5.7: Configuration Module requirements
- FR-7.1 through FR-7.5: Specific requirements

**Key Requirements**:
- Builder pattern for construction (FR-7.1.2)
- Private fields with getter methods (FR-7.1.6)
- `Serialize`/`Deserialize` support (FR-7.1.4)
- Sensible defaults (FR-7.1.3)
- No fallback package manager (FR-7.3.1)

---

#### Task 2.1: Define DetectionConfig Struct

**Description**: Create the `DetectionConfig` struct with private fields, getter methods, and serde support.

**PRD References**:
- §5.7 FR-7.1.1: Provide `DetectionConfig` struct
- §5.7 FR-7.1.3: Sensible defaults
- §5.7 FR-7.1.4: `Serialize`/`Deserialize`
- §5.7 FR-7.1.5: Passed programmatically (no file reading)
- §5.7 FR-7.1.6: Private fields with getter methods
- §5.7 FR-7.2: Configuration fields table

**Configuration Fields** (from PRD §5.7 FR-7.2):

| Field | Type | Default |
|-------|------|---------|
| `detection_order` | `Vec<PackageManagerKind>` | `[Pnpm, Yarn, Bun, Deno, Npm]` |
| `detect_from_env` | `bool` | `false` |
| `env_var_name` | `String` | `"WORKSPACE_PACKAGE_MANAGER"` |
| `additional_workspace_patterns` | `Vec<String>` | `[]` |
| `exclude_patterns` | `Vec<String>` | `["**/node_modules/**", ...]` |
| `max_search_depth` | `usize` | `5` |

**Acceptance Criteria**:
- [ ] `DetectionConfig` struct with all fields from PRD §5.7 FR-7.2
- [ ] All fields private with getter methods
- [ ] `Serialize`/`Deserialize` derived
- [ ] `Default` implemented with values from PRD table
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Implementation Details**:

```rust
// src/config/detection.rs

use serde::{Deserialize, Serialize};
use crate::package_manager::PackageManagerKind;

/// Configuration for project detection operations.
///
/// Use [`DetectionConfig::builder()`] to construct with custom values,
/// or [`DetectionConfig::default()`] for sensible defaults.
///
/// # PRD Reference
/// See PRD §5.7 for configuration requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionConfig {
    detection_order: Vec<PackageManagerKind>,
    detect_from_env: bool,
    env_var_name: String,
    additional_workspace_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    max_search_depth: usize,
}

impl DetectionConfig {
    /// Returns a new builder for constructing `DetectionConfig`.
    pub fn builder() -> DetectionConfigBuilder {
        DetectionConfigBuilder::default()
    }

    // Getter methods
    pub fn detection_order(&self) -> &[PackageManagerKind] {
        &self.detection_order
    }

    pub fn detect_from_env(&self) -> bool {
        self.detect_from_env
    }

    pub fn env_var_name(&self) -> &str {
        &self.env_var_name
    }

    pub fn additional_workspace_patterns(&self) -> &[String] {
        &self.additional_workspace_patterns
    }

    pub fn exclude_patterns(&self) -> &[String] {
        &self.exclude_patterns
    }

    pub fn max_search_depth(&self) -> usize {
        self.max_search_depth
    }
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            detection_order: vec![
                PackageManagerKind::Pnpm,
                PackageManagerKind::Yarn,
                PackageManagerKind::Bun,
                PackageManagerKind::Deno,
                PackageManagerKind::Npm,
            ],
            detect_from_env: false,
            env_var_name: "WORKSPACE_PACKAGE_MANAGER".to_string(),
            additional_workspace_patterns: vec![],
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/.*/**".to_string(),
                "**/dist/**".to_string(),
                "**/build/**".to_string(),
            ],
            max_search_depth: 5,
        }
    }
}
```

**Test Requirements**:
- Test default values match PRD specification
- Test all getter methods return correct values
- Test serialization/deserialization roundtrip

**Files to Create/Modify**:
- `crates/core/src/config/detection.rs`
- `crates/core/src/config/mod.rs`

**Estimated Effort**: 1.5 hours

---

#### Task 2.2: Implement DetectionConfigBuilder

**Description**: Create the builder for `DetectionConfig` with fluent API.

**PRD References**:
- §5.7 FR-7.1.2: Builder Pattern
- §5.7 FR-7.4.1: `DetectionConfig::builder()` method
- §5.7 FR-7.4.2: Fluent setter methods
- §5.7 FR-7.4.3: `build()` method
- §5.7 FR-7.4.4: Builder implements `Default`
- §5.7 FR-7.5: Builder usage example

**Acceptance Criteria**:
- [ ] `DetectionConfigBuilder` struct
- [ ] `Default` implementation
- [ ] Fluent setter methods for each field
- [ ] `build()` method returning `DetectionConfig`
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Implementation Details**:

```rust
// src/config/builder.rs

use super::DetectionConfig;
use crate::package_manager::PackageManagerKind;

/// Builder for [`DetectionConfig`].
///
/// # Example (from PRD §5.7 FR-7.5)
///
/// ```ignore
/// use workspace_core::config::DetectionConfig;
/// use workspace_core::package_manager::PackageManagerKind;
///
/// // All defaults
/// let config = DetectionConfig::builder().build();
///
/// // Custom configuration
/// let config = DetectionConfig::builder()
///     .detection_order(vec![
///         PackageManagerKind::Yarn,
///         PackageManagerKind::Npm,
///     ])
///     .exclude_patterns(vec![
///         "**/node_modules/**".to_string(),
///         "**/vendor/**".to_string(),
///     ])
///     .max_search_depth(10)
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct DetectionConfigBuilder {
    config: DetectionConfig,
}

impl DetectionConfigBuilder {
    pub fn detection_order(mut self, order: Vec<PackageManagerKind>) -> Self {
        self.config.detection_order = order;
        self
    }

    pub fn detect_from_env(mut self, enabled: bool) -> Self {
        self.config.detect_from_env = enabled;
        self
    }

    pub fn env_var_name(mut self, name: impl Into<String>) -> Self {
        self.config.env_var_name = name.into();
        self
    }

    pub fn additional_workspace_patterns(mut self, patterns: Vec<String>) -> Self {
        self.config.additional_workspace_patterns = patterns;
        self
    }

    pub fn exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.config.exclude_patterns = patterns;
        self
    }

    pub fn max_search_depth(mut self, depth: usize) -> Self {
        self.config.max_search_depth = depth;
        self
    }

    pub fn build(self) -> DetectionConfig {
        self.config
    }
}
```

**Test Requirements**:
- Test builder with all defaults
- Test builder with custom values for each field
- Test fluent API chaining
- Test example from PRD §5.7 FR-7.5

**Files to Create/Modify**:
- `crates/core/src/config/builder.rs`
- `crates/core/src/config/mod.rs` (add re-export)

**Estimated Effort**: 1 hour

---

### Epic 3: Repository Module

**Goal**: Implement `RepoType` and `RepoKind` enums with detection logic.

**PRD Context**:
- §3.1: Core Concepts diagram
- §3.2: Concept Definitions table
- §3.3: Detection Rules
- §5.1: Repository Type Module requirements

---

#### Task 3.1: Define RepoType Enum

**Description**: Create the `RepoType` enum representing runtime ecosystems (Node, Deno, Bun).

**PRD References**:
- §3.2: Concept Definitions - RepoType
- §3.3: RepoType Detection (Priority Order) table
- §5.1 FR-1.1.1: Node support
- §5.1 FR-1.1.2: Deno support
- §5.1 FR-1.1.3: Bun support
- §5.1 FR-1.1.4: Detection priority order (Deno > Bun > Node)

**Detection Priority** (from PRD §3.3):

| Priority | RepoType | Detection Criteria |
|----------|----------|-------------------|
| 1 | **Deno** | Presence of `deno.json` or `deno.jsonc` |
| 2 | **Bun** | Presence of `bunfig.toml` **OR** `bun.lockb` |
| 3 | **Node** | Presence of `package.json` (fallback) |

**Acceptance Criteria**:
- [ ] `RepoType` enum with `Node`, `Deno`, `Bun` variants
- [ ] Derive `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`
- [ ] Detection function following priority order from PRD §3.3
- [ ] Helper methods (e.g., `characteristic_files()`)
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Implementation Details**:

```rust
// src/repo/repo_type.rs

use serde::{Deserialize, Serialize};

/// The runtime ecosystem type of a repository.
///
/// Detection follows strict priority order (PRD §3.3):
/// 1. Deno (if `deno.json` or `deno.jsonc` present)
/// 2. Bun (if `bunfig.toml` or `bun.lockb` present)
/// 3. Node (if `package.json` present - fallback)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RepoType {
    /// Node.js runtime (detected via `package.json`)
    Node,
    /// Deno runtime (detected via `deno.json` or `deno.jsonc`)
    Deno,
    /// Bun runtime (detected via `bunfig.toml` or `bun.lockb`)
    Bun,
}

impl RepoType {
    /// Returns the characteristic files used to detect this repo type.
    pub fn characteristic_files(&self) -> &'static [&'static str] {
        match self {
            RepoType::Node => &["package.json"],
            RepoType::Deno => &["deno.json", "deno.jsonc"],
            RepoType::Bun => &["bunfig.toml", "bun.lockb"],
        }
    }
}
```

**Test Requirements**:
- Test characteristic_files() for each variant
- Test detection priority order
- Test serialization/deserialization

**Files to Create/Modify**:
- `crates/core/src/repo/repo_type.rs`
- `crates/core/src/repo/mod.rs`

**Estimated Effort**: 1.5 hours

---

#### Task 3.2: Define RepoKind Enum

**Description**: Create the `RepoKind` enum representing repository structure (Simple, Monorepo).

**PRD References**:
- §3.2: Concept Definitions - RepoKind
- §3.3: RepoKind Detection table
- §5.1 FR-1.2.1: Distinguish Simple/Monorepo
- §5.1 FR-1.2.2: Query methods

**Detection Criteria** (from PRD §3.3):

| RepoKind | Detection Criteria |
|----------|-------------------|
| Monorepo | `workspaces` in `package.json`, or `pnpm-workspace.yaml`, or `workspace`/`workspaces` in `deno.json` |
| Simple | No workspace configuration found |

**Acceptance Criteria**:
- [ ] `RepoKind` enum with `Simple`, `Monorepo` variants
- [ ] Derive `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`
- [ ] Helper methods (e.g., `is_monorepo()`)
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Test Requirements**:
- Test is_monorepo() returns correct value
- Test serialization/deserialization

**Files to Create/Modify**:
- `crates/core/src/repo/repo_kind.rs`
- `crates/core/src/repo/mod.rs`

**Estimated Effort**: 1 hour

---

### Epic 4: Package Manager Module

**Goal**: Implement `PackageManagerKind` enum and detection logic.

**PRD Context**:
- §3.2: Concept Definitions
- §3.3: PackageManagerKind Detection (Priority Order)
- §3.3: Lock File Mapping table
- §3.4: Validation Rules
- §5.2: Package Manager Module requirements

---

#### Task 4.1: Define PackageManagerKind Enum

**Description**: Create the `PackageManagerKind` enum with metadata methods.

**PRD References**:
- §5.2 FR-2.1.1-5: Support for npm, yarn, pnpm, bun, deno
- §5.2 FR-2.3.1: Command name for each PM
- §5.2 FR-2.3.2: Lock file name for each PM
- §5.2 FR-2.3.3: Workspace support indication
- §5.2 FR-2.3.4: Workspace config file path
- §3.3: Lock File Mapping table

**Lock File Mapping** (from PRD §3.3):

| PackageManagerKind | Lock File |
|--------------------|-----------|
| Npm | `package-lock.json` |
| Yarn | `yarn.lock` |
| Pnpm | `pnpm-lock.yaml` |
| Bun | `bun.lockb` |
| Deno | `deno.lock` |

**Acceptance Criteria**:
- [ ] `PackageManagerKind` enum with `Npm`, `Yarn`, `Pnpm`, `Bun`, `Deno` variants
- [ ] `command_name() -> &'static str` method
- [ ] `lock_file() -> &'static str` method
- [ ] `supports_workspaces() -> bool` method
- [ ] `workspace_config_file() -> Option<&'static str>` method
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Implementation Details**:

```rust
// src/package_manager/kind.rs

use serde::{Deserialize, Serialize};

/// The package manager used for dependency management.
///
/// # PRD Reference
/// See PRD §5.2 for package manager requirements and §3.3 for lock file mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageManagerKind {
    Npm,
    Yarn,
    Pnpm,
    Bun,
    Deno,
}

impl PackageManagerKind {
    /// Returns the CLI command name for this package manager.
    pub fn command_name(&self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Yarn => "yarn",
            Self::Pnpm => "pnpm",
            Self::Bun => "bun",
            Self::Deno => "deno",
        }
    }

    /// Returns the lock file name for this package manager (PRD §3.3).
    pub fn lock_file(&self) -> &'static str {
        match self {
            Self::Npm => "package-lock.json",
            Self::Yarn => "yarn.lock",
            Self::Pnpm => "pnpm-lock.yaml",
            Self::Bun => "bun.lockb",
            Self::Deno => "deno.lock",
        }
    }

    /// Returns true if this package manager supports workspaces.
    pub fn supports_workspaces(&self) -> bool {
        true // All supported PMs have workspace support
    }

    /// Returns the workspace config file path, if any.
    pub fn workspace_config_file(&self) -> Option<&'static str> {
        match self {
            Self::Pnpm => Some("pnpm-workspace.yaml"),
            _ => None, // Others use package.json workspaces field
        }
    }
}
```

**Test Requirements**:
- Test command_name() for each variant
- Test lock_file() matches PRD §3.3 table
- Test supports_workspaces() for each variant
- Test workspace_config_file() for each variant

**Files to Create/Modify**:
- `crates/core/src/package_manager/kind.rs`
- `crates/core/src/package_manager/mod.rs`

**Estimated Effort**: 1 hour

---

#### Task 4.2: Implement Package Manager Detection

**Description**: Implement detection logic following PRD priority order with mismatch validation.

**PRD References**:
- §3.3: PackageManagerKind Detection (Priority Order)
- §3.4: Validation Rules
- §5.2 FR-2.2.1: Detect from `packageManager` field (name only)
- §5.2 FR-2.2.2: Detect by lock file presence
- §5.2 FR-2.2.3: **Error if `packageManager` field conflicts with lock file**
- §5.2 FR-2.2.4: Configurable detection order
- §5.2 FR-2.2.5: Environment variable override
- §5.7 FR-7.3.1: **No fallback package manager (error if not found)**

**Detection Priority** (from PRD §3.3):

| Priority | Method | Description |
|----------|--------|-------------|
| 1 | `packageManager` field | Parse field, extract name only |
| 2 | Lock file | Detect by lock file presence |
| 3 | Environment variable | Optional, configurable |
| 4 | **Error** | No fallback - return error |

**Critical Validation** (from PRD §3.4):
- `packageManager` field ≠ detected lock file → **Error** (not warning)
- No lock file and no `packageManager` field → **Error** (no fallback)

**Acceptance Criteria**:
- [ ] `detect_package_manager(path: &Path, config: &DetectionConfig) -> Result<PackageManagerKind>`
- [ ] Priority 1: Check `packageManager` field in `package.json`
- [ ] Priority 2: Check lock files in configured order
- [ ] Priority 3: Check environment variable if enabled
- [ ] Return `Error::PackageManagerMismatch` if field conflicts with lock file
- [ ] Return `Error::PackageManagerNotFound` if nothing detected (no fallback!)
- [ ] Log at appropriate levels (PRD §6.5)
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Implementation Details**:

```rust
// src/package_manager/detector.rs

use std::path::Path;
use crate::config::DetectionConfig;
use crate::error::{Result, Error};
use super::PackageManagerKind;

/// Detects the package manager for a project.
///
/// # Detection Order (PRD §3.3)
/// 1. `packageManager` field in `package.json`
/// 2. Lock file presence (in config-specified order)
/// 3. Environment variable (if enabled)
/// 4. **Error** (no fallback!)
///
/// # Errors
/// - `PackageManagerMismatch`: `packageManager` field conflicts with lock file
/// - `PackageManagerNotFound`: No package manager could be detected
///
/// # PRD Reference
/// See PRD §5.2 FR-2.2 and §3.4 for validation rules.
pub fn detect_package_manager(
    path: &Path,
    config: &DetectionConfig,
) -> Result<PackageManagerKind> {
    log::debug!("Detecting package manager at '{}'", path.display());

    // Priority 1: Check packageManager field
    let declared = detect_from_package_manager_field(path)?;
    
    // Priority 2: Check lock files
    let from_lock = detect_from_lock_file(path, config)?;

    // Validate consistency (PRD §3.4: mismatch = ERROR)
    if let (Some(decl), Some(lock)) = (&declared, &from_lock) {
        if decl != lock {
            log::warn!(
                "packageManager field '{}' conflicts with lock file '{}'",
                decl.command_name(),
                lock.command_name()
            );
            return Err(Error::PackageManagerMismatch {
                path: path.to_path_buf(),
                declared: decl.command_name().to_string(),
                detected: lock.command_name().to_string(),
            });
        }
    }

    // Return first detected (priority order)
    if let Some(pm) = declared.or(from_lock) {
        log::info!("Package manager detected: {}", pm.command_name());
        return Ok(pm);
    }

    // Priority 3: Check environment variable
    if config.detect_from_env() {
        if let Some(pm) = detect_from_env(config.env_var_name())? {
            log::info!("Package manager from env: {}", pm.command_name());
            return Ok(pm);
        }
    }

    // NO FALLBACK (PRD §5.7 FR-7.3.1)
    log::error!("No package manager found at '{}'", path.display());
    Err(Error::PackageManagerNotFound {
        path: path.to_path_buf(),
    })
}
```

**Test Requirements**:
- Test detection from packageManager field
- Test detection from lock file
- Test detection from environment variable
- Test mismatch returns Error::PackageManagerMismatch
- Test not found returns Error::PackageManagerNotFound (no fallback)
- Test priority order is respected

**Files to Create/Modify**:
- `crates/core/src/package_manager/detector.rs`
- `crates/core/src/package_manager/mod.rs`

**Estimated Effort**: 3 hours

---

### Epic 5: Package & Dependency Modules

**Goal**: Implement `Package`, `Dependency`, and `PackageDependencies` types.

**PRD Context**:
- §5.3: Package Module requirements
- §5.4: Dependency Module requirements

---

#### Task 5.1: Define Dependency Types

**Description**: Create `Dependency` and `PackageDependencies` structs.

**PRD References**:
- §5.4 FR-4.1: Dependency Parsing (all dep types)
- §5.4 FR-4.2: Dependency Version Handling (semver, workspace protocol)
- §5.4 FR-4.3: Dependency Categorization (internal vs external)
- §5.4 FR-4.4: Dependency Data Model (structs)

**Data Model** (from PRD §5.4 FR-4.4):

```rust
struct Dependency {
    name: String,
    version_spec: String,              // Raw: "^4.17.21", "workspace:*"
    parsed_version: Option<VersionReq>, // Parsed semver (None for workspace protocol)
    is_internal: bool,                  // True if workspace protocol or matches workspace package
}

struct PackageDependencies {
    dependencies: Vec<Dependency>,
    dev_dependencies: Vec<Dependency>,
    peer_dependencies: Vec<Dependency>,
    optional_dependencies: Vec<Dependency>,
}
```

**Acceptance Criteria**:
- [ ] `Dependency` struct with fields from PRD §5.4 FR-4.4
- [ ] `PackageDependencies` struct with all dependency types
- [ ] `PackageDependencies::all()` iterator
- [ ] `PackageDependencies::internal()` iterator
- [ ] `PackageDependencies::external()` iterator
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Test Requirements**:
- Test Dependency construction
- Test PackageDependencies iterators (all, internal, external)
- Test is_internal flag behavior

**Files to Create/Modify**:
- `crates/core/src/dependency/types.rs`
- `crates/core/src/dependency/mod.rs`

**Estimated Effort**: 2 hours

---

#### Task 5.2: Implement Dependency Parser

**Description**: Parse dependencies from `package.json` using `package-json` crate.

**PRD References**:
- §5.4 FR-4.1: Parse all dependency types
- §5.4 FR-4.2.1: Use `semver` crate
- §5.4 FR-4.2.2: Preserve raw version strings
- §5.4 FR-4.2.3: Detect workspace protocol
- §1.4.2: Use `package-json` crate

**Acceptance Criteria**:
- [ ] Parse `dependencies`, `devDependencies`, `peerDependencies`, `optionalDependencies`
- [ ] Use `semver` crate for `VersionReq` parsing
- [ ] Preserve raw version spec strings
- [ ] Detect `workspace:*`, `workspace:^`, `workspace:~` protocols
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Test Requirements**:
- Test parsing standard version specs (^, ~, *, ranges)
- Test parsing workspace protocol versions
- Test parsing from package-json crate types
- Test raw version string preservation

**Files to Create/Modify**:
- `crates/core/src/dependency/parser.rs`

**Estimated Effort**: 2 hours

---

#### Task 5.3: Implement Dependency Categorizer

**Description**: Categorize dependencies as internal or external.

**PRD References**:
- §5.4 FR-4.3.1: Categorize internal vs external
- §5.4 FR-4.3.2: Match names against workspace package list
- §5.4 FR-4.2.4: Flag workspace protocol as internal
- §5.4 FR-4.3.5: Direct dependencies only (no transitive)

**Acceptance Criteria**:
- [ ] Detect workspace protocol as internal
- [ ] Match dependency names against workspace package list
- [ ] Handle edge cases (self-reference, etc.)
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Test Requirements**:
- Test workspace protocol detection
- Test name matching against package list
- Test self-reference handling

**Files to Create/Modify**:
- `crates/core/src/dependency/categorizer.rs`

**Estimated Effort**: 1.5 hours

---

#### Task 5.4: Define Package Struct

**Description**: Create the unified `Package` struct.

**PRD References**:
- §5.3 FR-3.1: Package Representation requirements
- §5.3 FR-3.2: Package Data Model
- §1.4.2: Use `package-json` crate for manifest

**Data Model** (from PRD §5.3 FR-3.2):

```rust
struct Package {
    // Identity
    name: String,
    version: Version,
    
    // Location
    relative_path: PathBuf,    // "." for root, "packages/utils" for workspace
    absolute_path: PathBuf,
    
    // Manifest
    manifest: PackageJson,     // From package-json crate
    
    // Dependencies
    dependencies: PackageDependencies,
}
```

**Methods** (from PRD §5.3 FR-3.2):
- `name() -> &str`
- `version() -> &Version`
- `relative_path() -> &Path`
- `absolute_path() -> &Path`
- `manifest() -> &PackageJson`
- `dependencies() -> &PackageDependencies`
- `internal_dependencies() -> impl Iterator<Item = &Dependency>`
- `external_dependencies() -> impl Iterator<Item = &Dependency>`
- `depends_on(package_name: &str) -> bool`

**Acceptance Criteria**:
- [ ] `Package` struct with all fields from PRD §5.3 FR-3.2
- [ ] All getter methods implemented
- [ ] `depends_on()` method
- [ ] `internal_dependencies()` / `external_dependencies()` iterators
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Test Requirements**:
- Test all getter methods
- Test depends_on() logic
- Test internal/external dependency iterators

**Files to Create/Modify**:
- `crates/core/src/package/package.rs`
- `crates/core/src/package/mod.rs`

**Estimated Effort**: 2 hours

---

### Epic 6: Monorepo Module

**Goal**: Implement monorepo detection and workspace package discovery.

**PRD Context**:
- §5.6: Monorepo Module requirements
- §3.3: RepoKind Detection

---

#### Task 6.1: Implement Workspace Config Parsing

**Description**: Parse workspace configuration from various sources.

**PRD References**:
- §5.6 FR-6.1.1: npm/yarn/bun workspaces from `package.json`
- §5.6 FR-6.1.2: pnpm workspaces from `pnpm-workspace.yaml`
- §5.6 FR-6.1.3: deno workspaces from `deno.json`
- §1.4.2: Use `serde_yaml_ng` for YAML

**Acceptance Criteria**:
- [ ] Parse `workspaces` field from `package.json` (string array or object with `packages` field)
- [ ] Parse `packages` from `pnpm-workspace.yaml`
- [ ] Parse `workspace`/`workspaces` from `deno.json`
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Test Requirements**:
- Test package.json array format
- Test package.json object format with packages field
- Test pnpm-workspace.yaml parsing
- Test deno.json workspace/workspaces parsing

**Files to Create/Modify**:
- `crates/core/src/monorepo/workspace.rs`

**Estimated Effort**: 2.5 hours

---

#### Task 6.2: Implement Workspace Package Discovery

**Description**: Discover all packages matching workspace patterns.

**PRD References**:
- §5.6 FR-6.2.1: Discover packages matching patterns
- §5.6 FR-6.2.2: Respect exclusion patterns
- §5.6 FR-6.2.3: Provide name and version
- §5.6 FR-6.2.4: Provide relative and absolute paths
- §5.6 FR-6.2.5: Configurable search depth
- §5.7 FR-7.3.3: Merge `additional_workspace_patterns` with config
- §5.7 FR-7.3.4: Deduplicate merged patterns

**Acceptance Criteria**:
- [ ] Discover packages using glob patterns
- [ ] Merge `additional_workspace_patterns` with patterns from files
- [ ] Remove duplicates from merged patterns
- [ ] Respect `exclude_patterns` from config
- [ ] Respect `max_search_depth` from config
- [ ] Return `Package` for each discovered workspace package
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Test Requirements**:
- Test pattern matching with various glob patterns
- Test pattern merging and deduplication
- Test exclusion pattern filtering
- Test max search depth limit

**Files to Create/Modify**:
- `crates/core/src/monorepo/detector.rs`

**Estimated Effort**: 3 hours

---

#### Task 6.3: Implement Dependency Graph Analysis

**Description**: Analyze dependencies between workspace packages.

**PRD References**:
- §5.6 FR-6.3.1: Identify internal workspace dependencies
- §5.6 FR-6.3.2: Identify internal dev dependencies
- §5.6 FR-6.3.3: Generate dependency graph (P1)
- §5.6 FR-6.3.4: Detect circular dependencies (P2)
- §5.5 FR-5.2.7: `dependents_of(package_name)` method

**Acceptance Criteria**:
- [ ] Build dependency graph between packages
- [ ] `dependents_of(package_name)` returns packages that depend on given package
- [ ] Detect circular dependencies (return `Error::CircularDependency`)
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Test Requirements**:
- Test dependency graph construction
- Test dependents_of() with various graph structures
- Test circular dependency detection

**Files to Create/Modify**:
- `crates/core/src/monorepo/detector.rs` (extend)

**Estimated Effort**: 2.5 hours

---

### Epic 7: Project Module

**Goal**: Implement unified `Project` detection and representation.

**PRD Context**:
- §5.5: Project Module requirements

---

#### Task 7.1: Define Project Struct

**Description**: Create the unified `Project` struct.

**PRD References**:
- §5.5 FR-5.1: Project Detection requirements
- §5.5 FR-5.2: Project Information requirements
- §5.5 FR-5.3: Project Data Model

**Data Model** (from PRD §5.5 FR-5.3):

```rust
struct Project {
    root_path: PathBuf,
    repo_type: RepoType,
    package_manager: PackageManagerKind,
    repo_kind: RepoKind,
    root_package: Package,
    workspace_packages: Vec<Package>,  // Empty for single-repo
    package_index: HashMap<String, usize>,  // Fast lookup by name
}
```

**Methods** (from PRD §5.5 FR-5.3):
- Accessors: `root_path()`, `repo_type()`, `package_manager()`, `repo_kind()`, `is_monorepo()`
- Package access: `root_package()`, `workspace_packages()`, `all_packages()`, `get_package(name)`, `find_package_for_path(path)`
- Dependency queries: `dependents_of(package_name) -> Vec<&Package>`

**Acceptance Criteria**:
- [ ] `Project` struct with all fields from PRD §5.5 FR-5.3
- [ ] All accessor methods
- [ ] All package access methods
- [ ] `dependents_of(package_name)` method
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Test Requirements**:
- Test all accessor methods
- Test package lookup methods
- Test dependents_of() integration

**Files to Create/Modify**:
- `crates/core/src/project/project.rs`
- `crates/core/src/project/mod.rs`

**Estimated Effort**: 2 hours

---

#### Task 7.2: Implement Project Detection

**Description**: Implement unified project detection from any path.

**PRD References**:
- §5.5 FR-5.1.1: Detect from any valid project path
- §5.5 FR-5.1.2: Find project root from subdirectory
- §5.5 FR-5.1.3: Validate project structure
- §5.5 FR-5.1.4: Support custom configuration
- §8.2: **Explicit `&Path` required, NO fallback to cwd**

**Use Case UC-5** (from PRD §4.2):
1. Caller provides explicit starting path (required)
2. System walks up directory tree
3. System returns project root and type

**Acceptance Criteria**:
- [ ] `detect_project(path: &Path, config: &DetectionConfig) -> Result<Project>`
- [ ] Walk up directory tree to find project root
- [ ] Detect `RepoType`, `PackageManagerKind`, `RepoKind`
- [ ] Build `Package` for root
- [ ] Discover workspace packages if monorepo
- [ ] **NO fallback to current directory** (PRD §8.2)
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Test Requirements**:
- Test detection from project root
- Test detection from subdirectory (walk up)
- Test simple repo detection
- Test monorepo detection
- Test no fallback to cwd behavior

**Files to Create/Modify**:
- `crates/core/src/project/detector.rs`

**Estimated Effort**: 3 hours

---

### Epic 8: Integration & Polish

**Goal**: End-to-end tests, documentation, and optimization.

**PRD Context**:
- §6.1: Performance requirements
- §6.4: Code Quality requirements
- §8.4: Documentation Standards
- §9: Success Criteria

---

#### Task 8.1: Create Integration Tests

**Description**: Create end-to-end tests for common scenarios.

**PRD References**:
- §4.2: Use Cases (UC-1 through UC-5)
- §9.1: Acceptance Criteria

**Test Scenarios**:
- Single-repo Node.js project
- Single-repo Deno project
- Single-repo Bun project
- npm monorepo
- yarn monorepo
- pnpm monorepo
- Mixed scenarios (packageManager field + lock file mismatch)
- Edge cases (symlinks, deep nesting, circular deps)

**Acceptance Criteria**:
- [ ] Integration test for each major use case
- [ ] Test fixtures created
- [ ] All tests passing on CI
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Files to Create/Modify**:
- `crates/core/tests/integration/*.rs`

**Estimated Effort**: 4 hours

---

#### Task 8.2: Performance Validation

**Description**: Validate performance meets NFR targets.

**PRD References**:
- §6.1: Performance requirements

**Performance Targets** (from PRD §6.1):

| Operation | Target |
|-----------|--------|
| Repository type detection | < 5ms |
| Package manager detection | < 10ms |
| Project type detection | < 50ms |
| Full monorepo analysis (100 packages) | < 500ms |
| Memory usage (100 packages) | < 50MB |

**Acceptance Criteria**:
- [ ] Benchmark tests created
- [ ] Performance meets targets
- [ ] Performance documented
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Files to Create/Modify**:
- `crates/core/benches/detection.rs`

**Estimated Effort**: 2 hours

---

#### Task 8.3: Documentation Review

**Description**: Ensure all documentation meets standards.

**PRD References**:
- §8.4: Documentation Standards
- §9.1: Acceptance Criteria

**Acceptance Criteria**:
- [ ] All public items documented
- [ ] Module-level docs with What/How/Why pattern
- [ ] Examples for all public functions
- [ ] Cross-references between related items
- [ ] `cargo doc` generates without warnings
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Files to Modify**:
- All `*.rs` files (add/improve documentation)

**Estimated Effort**: 2 hours

---

#### Task 8.4: Final Quality Gates

**Description**: Verify all quality gates pass.

**PRD References**:
- §9.2: Quality Gates

**Quality Gates** (from PRD §9.2):
1. Code review by at least one developer
2. All tests pass on Windows, macOS, Linux
3. Generated docs reviewed for completeness
4. Benchmarks meet NFR targets

**Acceptance Criteria**:
- [ ] `cargo clippy` passes with zero warnings
- [ ] `cargo test` passes
- [ ] Code coverage > 80%
- [ ] PR reviewed and approved
- [ ] CI passes on all platforms
- [ ] Clippy 100%
- [ ] Fmt 100%
- [ ] Docs 100%
- [ ] Unit tests 100%
- [ ] Build success
- [ ] Ask for review implementation in a new session

**Estimated Effort**: 2 hours

---

## 4. Task Dependency Graph

```
Epic 0: Setup
  ├── Task 0.1: Crate Skeleton
  └── Task 0.2: Module Structure
        │
        ▼
Epic 1: Error Module
  └── Task 1.1: Unified Error Enum
        │
        ├───────────────────┐
        ▼                   ▼
Epic 2: Config         Epic 3: Repo
  ├── Task 2.1: Config   ├── Task 3.1: RepoType
  └── Task 2.2: Builder  └── Task 3.2: RepoKind
        │                       │
        └───────────┬───────────┘
                    ▼
             Epic 4: Package Manager
               ├── Task 4.1: Kind Enum
               └── Task 4.2: Detection
                       │
                       ▼
             Epic 5: Package & Dependency
               ├── Task 5.1: Dep Types
               ├── Task 5.2: Dep Parser
               ├── Task 5.3: Categorizer
               └── Task 5.4: Package Struct
                       │
                       ▼
             Epic 6: Monorepo
               ├── Task 6.1: Workspace Config
               ├── Task 6.2: Package Discovery
               └── Task 6.3: Dep Graph
                       │
                       ▼
             Epic 7: Project
               ├── Task 7.1: Project Struct
               └── Task 7.2: Detection
                       │
                       ▼
             Epic 8: Integration
               ├── Task 8.1: Integration Tests
               ├── Task 8.2: Performance
               ├── Task 8.3: Documentation
               └── Task 8.4: Quality Gates
```

---

## 5. Estimated Effort Summary

| Epic | Estimated Hours |
|------|-----------------|
| Epic 0: Setup | 1.5 |
| Epic 1: Error | 2 |
| Epic 2: Config | 2.5 |
| Epic 3: Repo | 2.5 |
| Epic 4: Package Manager | 4 |
| Epic 5: Package & Dependency | 7.5 |
| Epic 6: Monorepo | 8 |
| Epic 7: Project | 5 |
| Epic 8: Integration | 10 |
| **Total** | **~43 hours** |

---

## 6. Logging Guidelines

All tasks should implement logging following PRD §6.5:

| Level | What to Log |
|-------|-------------|
| **Error** | Fatal errors preventing operation completion |
| **Warn** | Ambiguities, inconsistencies (e.g., multiple lock files) |
| **Info** | High-level operations (e.g., "Detecting repository type at X") |
| **Debug** | Detection decisions (e.g., "Lock file pnpm-lock.yaml found") |
| **Trace** | Detailed traces (e.g., file access patterns) |

---

## 7. Testing Guidelines

Each module should include `tests.rs` following PRD testing convention:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod feature_a {
        use super::*;
        
        #[test]
        fn test_specific_behavior() { ... }
    }

    mod feature_b {
        use super::*;
        
        #[test]
        fn test_another_behavior() { ... }
    }
}
```

---

## 8. References

- [PRD.md](./PRD.md) - Product Requirements Document
- [workspace-fs](../fs/README.md) - Filesystem abstraction crate
- [snafu documentation](https://docs.rs/snafu/0.8.9/snafu/)
- [package-json crate](https://docs.rs/package-json/0.5.0/package_json/)