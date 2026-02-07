# Product Requirements Document: workspace-core

## Document Information

| Field | Value |
|-------|-------|
| **Crate Name** | `workspace-core` |
| **Version** | `0.1.0` |
| **Status** | Ready |
| **Created** | 2026-01-12 |
| **Last Updated** | 2026-01-13 |

---

## 1. Executive Summary

### 1.1 Purpose

The `workspace-core` crate provides the foundational abstractions and detection mechanisms for working with JavaScript/TypeScript projects from Rust. It serves as the core building block for the workspace-node-tools ecosystem, enabling reliable detection of project types (single-package or monorepo), repository types (Node, Deno, Bun), and package managers.

### 1.2 Scope

This crate focuses exclusively on:

- **Repository Type Detection**: Identifying the runtime ecosystem (Node, Deno, Bun) based on characteristic files
- **Package Manager Detection**: Identifying which package manager (npm, yarn, pnpm, bun, deno) is used in a project
- **Repository Kind Detection**: Determining if a project is a simple single-package repository or a monorepo
- **Monorepo Analysis**: Detecting workspace configuration, discovering workspace packages, and analyzing internal dependencies
- **Core Abstractions**: Providing reusable types and traits for the ecosystem

### 1.3 Out of Scope

The following concerns are explicitly **not** part of this crate:

- Command execution (delegated to a separate `workspace-executor` crate)
- Filesystem operations beyond detection (delegated to `workspace-fs` crate)
- Git operations (delegated to a separate `workspace-git` crate)
- Version management and changesets (delegated to higher-level crates)
- CLI interfaces (delegated to the `workspace` CLI crate)

### 1.4 Dependencies

#### 1.4.1 Internal Dependencies

| Crate | Category | Purpose |
|-------|----------|---------|
| `workspace-fs` | dep | Required for all filesystem operations |

#### 1.4.2 External Dependencies

| Crate | Version | Category | Purpose |
|-------|---------|----------|---------|
| `snafu` | `0.8.9` | dep | Error handling with context |
| `serde` | `1.0` | dep | Serialization framework (with `derive` feature) |
| `serde_json` | `1.0` | dep | JSON parsing for `package.json` |
| `serde_yaml_ng` | `0.10.0` | dep | YAML parsing for `pnpm-workspace.yaml` |
| `log` | `0.4` | dep | Logging facade |
| `semver` | `1.0` | dep | Semantic versioning parsing and comparison |
| `package-json` | `0.5.0` | dep | Type-safe `package.json` parsing |
| `walkdir` | `2.0` | dep | Recursive directory traversal |
| `glob` | `0.3` | dep | Glob pattern matching |

#### 1.4.3 Development Dependencies

| Crate | Version | Category | Purpose |
|-------|---------|----------|---------|
| `tempfile` | `3.0` | dev-dep | Temporary directories for tests |

---

## 2. Problem Statement

### 2.1 Current Challenges

When building tools that interact with JavaScript/TypeScript projects from Rust, developers face several challenges:

1. **Runtime Fragmentation**: The JavaScript ecosystem now includes multiple runtimes (Node.js, Deno, Bun), each with different configuration files and conventions.

2. **Package Manager Fragmentation**: Multiple package managers exist (npm, yarn, pnpm, bun, deno), each with different lock files, commands, and workspace configurations.

3. **Monorepo Complexity**: Monorepos can use different workspace implementations, requiring format-specific detection logic.

4. **Inconsistent Detection**: Without a unified approach, tools often implement ad-hoc detection that fails in edge cases or doesn't handle all scenarios.

5. **Type Safety**: Rust's type system can prevent many errors, but only if the abstractions are well-designed and comprehensive.

### 2.2 Solution

The `workspace-core` crate provides:

- A unified detection API that works across all major runtimes and package managers
- Clear separation of concerns: RepoType (runtime), PackageManagerKind (tooling), RepoKind (structure)
- Type-safe abstractions for project and repository concepts
- Configurable detection with sensible defaults
- Workspace package discovery and dependency analysis
- Validation with clear error reporting for inconsistencies

---

## 3. Conceptual Model

### 3.1 Core Concepts

```
┌─────────────────────────────────────────────────────────────────┐
│                        PROJECT                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────┐    ┌──────────────────┐                   │
│  │   RepoKind       │    │   RepoType       │                   │
│  ├──────────────────┤    ├──────────────────┤                   │
│  │ • Simple         │    │ • Deno           │                   │
│  │ • Monorepo       │    │ • Bun            │                   │
│  └──────────────────┘    │ • Node           │                   │
│                          └──────────────────┘                   │
│                                                                  │
│  ┌──────────────────────────────────────────┐                   │
│  │         PackageManagerKind               │                   │
│  ├──────────────────────────────────────────┤                   │
│  │ • Npm                                    │                   │
│  │ • Yarn                                   │                   │
│  │ • Pnpm                                   │                   │
│  │ • Bun                                    │                   │
│  │ • Deno                                   │                   │
│  └──────────────────────────────────────────┘                   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Concept Definitions

| Concept | Description | Values |
|---------|-------------|--------|
| **RepoKind** | Structure of the repository | `Simple`, `Monorepo` |
| **RepoType** | Runtime/ecosystem type (detected by characteristic files) | `Node`, `Deno`, `Bun` |
| **PackageManagerKind** | Package manager used for dependency management | `Npm`, `Yarn`, `Pnpm`, `Bun`, `Deno` |

### 3.3 Detection Rules

#### RepoType Detection (Priority Order)

| Priority | RepoType | Detection Criteria |
|----------|----------|-------------------|
| 1 | **Deno** | Presence of `deno.json` or `deno.jsonc` |
| 2 | **Bun** | Presence of `bunfig.toml` **OR** `bun.lockb` |
| 3 | **Node** | Presence of `package.json` (fallback) |

#### PackageManagerKind Detection (Priority Order)

| Priority | Method | Description |
|----------|--------|-------------|
| 1 | `packageManager` field | Parse `package.json` field (e.g., `"packageManager": "pnpm@9.0.0"`) - extract name only |
| 2 | Lock file | Detect by lock file presence |
| 3 | Environment variable | Optional, configurable (e.g., `WORKSPACE_PACKAGE_MANAGER`) |
| 4 | Fallback | Configurable default (default: `Npm`) |

#### Lock File Mapping

| PackageManagerKind | Lock File |
|--------------------|-----------|
| **Npm** | `package-lock.json` |
| **Yarn** | `yarn.lock` |
| **Pnpm** | `pnpm-lock.yaml` |
| **Bun** | `bun.lockb` |
| **Deno** | `deno.lock` |

#### RepoKind Detection

| RepoKind | Detection Criteria |
|----------|-------------------|
| **Monorepo** | `workspaces` field in `package.json`, or `pnpm-workspace.yaml`, or `workspace`/`workspaces` in `deno.json` |
| **Simple** | No workspace configuration found |

### 3.4 Validation Rules

| Rule | Behavior |
|------|----------|
| `packageManager` field ≠ detected lock file | **Error**: Report inconsistency |
| Multiple lock files present | **Warning**: Report ambiguity, use priority order |
| No lock file and no `packageManager` field | Use fallback (configurable) |

---

## 4. User Personas

### 4.1 Primary Users

| Persona | Description | Needs |
|---------|-------------|-------|
| **Rust Library Developer** | Building tools that interact with Node.js/Deno/Bun projects | Reliable detection, clear APIs, good documentation |
| **CLI Tool Author** | Creating command-line tools for JavaScript workflows | Simple integration, configurable behavior, accurate detection |
| **Monorepo Tool Builder** | Building specialized monorepo management tools | Complete workspace analysis, dependency graphs, package discovery |

### 4.2 Use Cases

#### UC-1: Repository Type Detection

**Actor**: Any user  
**Goal**: Determine if a project uses Node.js, Deno, or Bun  
**Precondition**: A valid project directory exists  
**Flow**:
1. Caller provides an explicit path to a project directory (required, no fallback)
2. System checks for characteristic files in priority order
3. System returns the detected repository type

#### UC-2: Package Manager Detection

**Actor**: Any user  
**Goal**: Determine which package manager is used in a project  
**Precondition**: A valid project directory exists  
**Flow**:
1. Caller provides an explicit path to a project directory (required, no fallback)
2. System checks `packageManager` field in `package.json`
3. If not found, system checks for lock files
4. System validates consistency between declaration and lock file
5. System returns the detected package manager or error if inconsistent

#### UC-3: Project Type Detection

**Actor**: Any user  
**Goal**: Determine if a project is a simple repository or monorepo  
**Precondition**: A valid project directory exists  
**Flow**:
1. Caller provides an explicit path to a project directory (required, no fallback)
2. System detects the repository type
3. System checks for workspace configuration
4. System returns the repository kind (simple or monorepo)

#### UC-4: Monorepo Analysis

**Actor**: Monorepo Tool Builder  
**Goal**: Get complete information about a monorepo structure  
**Precondition**: A valid monorepo root directory exists  
**Flow**:
1. Caller provides an explicit path to a monorepo root (required, no fallback)
2. System validates it is a monorepo
3. System discovers all workspace packages
4. System analyzes internal dependencies between packages
5. System returns a complete monorepo descriptor

#### UC-5: Find Project Root

**Actor**: Any user  
**Goal**: Find the root of a project from any subdirectory  
**Precondition**: A valid path inside a project exists  
**Flow**:
1. Caller provides an explicit starting path (required, no fallback to current directory)
2. System walks up the directory tree
3. System returns the project root and type

---

## 5. Functional Requirements

### 5.1 Repository Type Module (`repo`)

#### FR-1.1: RepoType Enumeration

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1.1 | System SHALL support Node repository type detection | P0 |
| FR-1.1.2 | System SHALL support Deno repository type detection | P0 |
| FR-1.1.3 | System SHALL support Bun repository type detection | P0 |
| FR-1.1.4 | System SHALL detect RepoType by checking files in priority order (Deno > Bun > Node) | P0 |

#### FR-1.2: RepoKind Enumeration

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.2.1 | System SHALL distinguish between Simple and Monorepo repository kinds | P0 |
| FR-1.2.2 | System SHALL provide methods to query repository kind characteristics | P0 |

### 5.2 Package Manager Module (`package_manager`)

#### FR-2.1: PackageManagerKind Enumeration

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | System SHALL support npm package manager detection | P0 |
| FR-2.1.2 | System SHALL support yarn package manager detection | P0 |
| FR-2.1.3 | System SHALL support pnpm package manager detection | P0 |
| FR-2.1.4 | System SHALL support bun package manager detection | P0 |
| FR-2.1.5 | System SHALL support deno package manager detection | P0 |

#### FR-2.2: Package Manager Detection

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.2.1 | System SHALL detect package manager from `packageManager` field in `package.json` (name only) | P0 |
| FR-2.2.2 | System SHALL detect package manager by lock file presence | P0 |
| FR-2.2.3 | System SHALL report error if `packageManager` field conflicts with lock file | P0 |
| FR-2.2.4 | System SHALL support configurable detection order | P1 |
| FR-2.2.5 | System SHALL support environment variable override for package manager | P1 |
| FR-2.2.6 | System SHALL provide configurable fallback package manager | P1 |

#### FR-2.3: Package Manager Metadata

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.3.1 | System SHALL provide command name for each package manager | P0 |
| FR-2.3.2 | System SHALL provide lock file name for each package manager | P0 |
| FR-2.3.3 | System SHALL indicate workspace support for each package manager | P0 |
| FR-2.3.4 | System SHALL provide workspace config file path when applicable | P1 |

### 5.3 Package Module (`package`)

#### FR-3.1: Package Representation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1.1 | System SHALL provide a unified `Package` struct for both single-repo and monorepo packages | P0 |
| FR-3.1.2 | System SHALL provide package name from `package.json` | P0 |
| FR-3.1.3 | System SHALL provide package version from `package.json` | P0 |
| FR-3.1.4 | System SHALL provide relative path (from project root) | P0 |
| FR-3.1.5 | System SHALL provide absolute path to package directory | P0 |
| FR-3.1.6 | System SHALL provide parsed `package.json` manifest using `package-json` crate | P0 |
| FR-3.1.7 | System SHALL provide categorized dependencies (`PackageDependencies`) | P0 |

#### FR-3.2: Package Data Model

```rust
/// Represents a single package (works for both single-repo and monorepo).
/// In a single-repo, there is exactly one Package (the root).
/// In a monorepo, there are multiple Packages (workspace members).
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

impl Package {
    // Getters
    fn name(&self) -> &str;
    fn version(&self) -> &Version;
    fn relative_path(&self) -> &Path;
    fn absolute_path(&self) -> &Path;
    fn manifest(&self) -> &PackageJson;
    fn dependencies(&self) -> &PackageDependencies;
    
    // Dependency queries
    fn internal_dependencies(&self) -> impl Iterator<Item = &Dependency>;
    fn external_dependencies(&self) -> impl Iterator<Item = &Dependency>;
    fn depends_on(&self, package_name: &str) -> bool;
}
```

### 5.4 Dependency Module (`dependency`)

#### FR-4.1: Dependency Parsing

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.1.1 | System SHALL parse production dependencies from `dependencies` field | P0 |
| FR-4.1.2 | System SHALL parse development dependencies from `devDependencies` field | P0 |
| FR-4.1.3 | System SHALL parse peer dependencies from `peerDependencies` field | P1 |
| FR-4.1.4 | System SHALL parse optional dependencies from `optionalDependencies` field | P1 |

#### FR-4.2: Dependency Version Handling

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.2.1 | System SHALL use `semver` crate to parse version specifiers | P0 |
| FR-4.2.2 | System SHALL preserve raw version strings (e.g., `"^4.17.21"`, `"workspace:*"`) | P0 |
| FR-4.2.3 | System SHALL detect workspace protocol specifiers (`workspace:*`, `workspace:^`, `workspace:~`) | P0 |
| FR-4.2.4 | System SHALL flag dependencies using workspace protocol as internal | P0 |

#### FR-4.3: Dependency Categorization

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.3.1 | System SHALL categorize dependencies as internal (workspace) or external | P0 |
| FR-4.3.2 | System SHALL identify internal dependencies by matching names against workspace package list | P0 |
| FR-4.3.3 | System SHALL provide methods to query internal dependencies only | P0 |
| FR-4.3.4 | System SHALL provide methods to query external dependencies only | P0 |
| FR-4.3.5 | System SHALL handle only direct dependencies (no transitive resolution) | P0 |

#### FR-4.4: Dependency Data Model

```rust
/// A single dependency entry
struct Dependency {
    name: String,
    version_spec: String,              // Raw: "^4.17.21", "workspace:*"
    parsed_version: Option<VersionReq>, // Parsed semver (None for workspace protocol)
    is_internal: bool,                  // True if workspace protocol or matches workspace package
}

/// Categorized dependencies from package.json
struct PackageDependencies {
    dependencies: Vec<Dependency>,
    dev_dependencies: Vec<Dependency>,
    peer_dependencies: Vec<Dependency>,
    optional_dependencies: Vec<Dependency>,
}

impl PackageDependencies {
    fn all(&self) -> impl Iterator<Item = &Dependency>;
    fn internal(&self) -> impl Iterator<Item = &Dependency>;
    fn external(&self) -> impl Iterator<Item = &Dependency>;
}
```

### 5.5 Project Module (`project`)

#### FR-5.1: Project Detection

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-5.1.1 | System SHALL detect projects from any valid project path | P0 |
| FR-5.1.2 | System SHALL find project root from any subdirectory | P0 |
| FR-5.1.3 | System SHALL validate project structure | P1 |
| FR-5.1.4 | System SHALL support detection with custom configuration | P1 |

#### FR-5.2: Project Information

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-5.2.1 | System SHALL provide project root path | P0 |
| FR-5.2.2 | System SHALL provide detected repository type (`RepoType`) | P0 |
| FR-5.2.3 | System SHALL provide detected package manager (`PackageManagerKind`) | P0 |
| FR-5.2.4 | System SHALL provide repository kind (`RepoKind`: simple/monorepo) | P0 |
| FR-5.2.5 | System SHALL provide root package (`Package`) | P0 |
| FR-5.2.6 | System SHALL provide workspace packages for monorepos (`Vec<Package>`) | P0 |
| FR-5.2.7 | System SHALL provide method to query packages that depend on a given package (`dependents_of`) | P0 |

#### FR-5.3: Project Data Model

```rust
/// Unified project representation (single-repo or monorepo)
struct Project {
    // Project info
    root_path: PathBuf,
    repo_type: RepoType,
    package_manager: PackageManagerKind,
    repo_kind: RepoKind,
    
    // Packages
    root_package: Package,
    workspace_packages: Vec<Package>,  // Empty for single-repo
    package_index: HashMap<String, usize>,  // Fast lookup by name
}

impl Project {
    // Accessors
    fn root_path(&self) -> &Path;
    fn repo_type(&self) -> RepoType;
    fn package_manager(&self) -> PackageManagerKind;
    fn repo_kind(&self) -> &RepoKind;
    fn is_monorepo(&self) -> bool;
    
    // Package access
    fn root_package(&self) -> &Package;
    fn workspace_packages(&self) -> &[Package];
    fn all_packages(&self) -> impl Iterator<Item = &Package>;  // root + workspace
    fn get_package(&self, name: &str) -> Option<&Package>;
    fn find_package_for_path(&self, path: &Path) -> Option<&Package>;
    
    // Dependency queries (cross-package)
    /// Returns all packages that depend on the given package name.
    fn dependents_of(&self, package_name: &str) -> Vec<&Package>;
}
```

### 5.6 Monorepo Module (`monorepo`)

#### FR-6.1: Monorepo Detection

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-6.1.1 | System SHALL detect npm/yarn/bun workspaces from `package.json` `workspaces` field | P0 |
| FR-6.1.2 | System SHALL detect pnpm workspaces from `pnpm-workspace.yaml` | P0 |
| FR-6.1.3 | System SHALL detect deno workspaces from `deno.json` `workspace`/`workspaces` field | P0 |

#### FR-6.2: Workspace Package Discovery

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-6.2.1 | System SHALL discover all packages matching workspace patterns | P0 |
| FR-6.2.2 | System SHALL respect exclusion patterns | P0 |
| FR-6.2.3 | System SHALL provide package name and version | P0 |
| FR-6.2.4 | System SHALL provide relative and absolute paths | P0 |
| FR-6.2.5 | System SHALL support configurable search depth | P1 |

#### FR-6.3: Workspace Dependency Analysis

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-6.3.1 | System SHALL identify internal workspace dependencies | P0 |
| FR-6.3.2 | System SHALL identify internal dev dependencies | P0 |
| FR-6.3.3 | System SHALL generate dependency graph between packages | P1 |
| FR-6.3.4 | System SHALL detect circular dependencies | P2 |

#### FR-6.4: Monorepo Information

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-6.4.1 | System SHALL provide repository type | P0 |
| FR-6.4.2 | System SHALL provide root path | P0 |
| FR-6.4.3 | System SHALL provide list of all packages | P0 |
| FR-6.4.4 | System SHALL provide package lookup by name | P0 |
| FR-6.4.5 | System SHALL find package containing a given path | P1 |

### 5.7 Configuration Module (`config`)

#### FR-7.1: Configuration Design

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-7.1.1 | System SHALL provide a `DetectionConfig` struct for all detection operations | P0 |
| FR-7.1.2 | System SHALL use the Builder Pattern for `DetectionConfig` construction | P0 |
| FR-7.1.3 | System SHALL provide sensible defaults for all configuration fields | P0 |
| FR-7.1.4 | System SHALL implement `Serialize`/`Deserialize` for `DetectionConfig` | P0 |
| FR-7.1.5 | Configuration SHALL be passed programmatically (crate does NOT read config files) | P0 |
| FR-7.1.6 | System SHALL keep `DetectionConfig` fields private with getter methods | P0 |

#### FR-7.2: Configuration Fields

The `DetectionConfig` struct SHALL contain the following fields:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `detection_order` | `Vec<PackageManagerKind>` | `[Pnpm, Yarn, Bun, Deno, Npm]` | Order to check for package managers (first match wins) |
| `detect_from_env` | `bool` | `false` | Whether to check environment variable for PM hint |
| `env_var_name` | `String` | `"WORKSPACE_PACKAGE_MANAGER"` | Environment variable name to check |
| `additional_workspace_patterns` | `Vec<String>` | `[]` | Extra patterns to merge with those from config files |
| `exclude_patterns` | `Vec<String>` | `["**/node_modules/**", "**/.*/**", "**/dist/**", "**/build/**"]` | Patterns to exclude from package detection |
| `max_search_depth` | `usize` | `5` | Maximum depth for recursive package search |

#### FR-7.3: Configuration Behavior

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-7.3.1 | System SHALL NOT provide a fallback package manager (error if not found) | P0 |
| FR-7.3.2 | System SHALL always follow symlinks during detection (not configurable) | P0 |
| FR-7.3.3 | System SHALL merge `additional_workspace_patterns` with patterns from config files | P0 |
| FR-7.3.4 | System SHALL ensure merged workspace patterns contain no duplicates | P0 |
| FR-7.3.5 | System SHALL use `workspace-fs` for all filesystem operations | P0 |

#### FR-7.4: Builder Pattern

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-7.4.1 | System SHALL provide `DetectionConfig::builder()` method returning `DetectionConfigBuilder` | P0 |
| FR-7.4.2 | Builder SHALL provide fluent setter methods for each field | P0 |
| FR-7.4.3 | Builder SHALL provide `build()` method returning `DetectionConfig` | P0 |
| FR-7.4.4 | Builder SHALL implement `Default` with all default values | P0 |

#### FR-7.5: Builder Usage Example

```rust
// All defaults
let config = DetectionConfig::builder().build();

// Custom configuration
let config = DetectionConfig::builder()
    .detection_order(vec![
        PackageManagerKind::Yarn,
        PackageManagerKind::Npm,
    ])
    .exclude_patterns(vec![
        "**/node_modules/**".to_string(),
        "**/vendor/**".to_string(),
    ])
    .max_search_depth(10)
    .build();
```

### 5.8 Error Handling (`error`)

#### FR-8.1: Error Types

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-8.1.1 | System SHALL provide a single unified `Error` enum for the entire crate using `snafu` | P0 |
| FR-8.1.2 | System SHALL include context information (paths, reasons) in error variants | P0 |
| FR-8.1.3 | System SHALL implement `std::error::Error` for the `Error` type | P0 |
| FR-8.1.4 | System SHALL implement `AsRef<str>` for the `Error` type, returning the qualified variant name as a static string (e.g., `"Error::PackageManagerNotFound"`) | P0 |
| FR-8.1.5 | System SHALL provide actionable error messages via `Display` | P1 |
| FR-8.1.6 | System SHALL provide a `Result<T>` type alias for `std::result::Result<T, Error>` | P0 |

#### FR-8.2: Error Variants

The unified `Error` enum SHALL include variants for:

| Category | Variants |
|----------|----------|
| **Repository** | `RepoTypeDetection`, `RepoTypeUnknown` |
| **Package Manager** | `PackageManagerNotFound`, `PackageManagerMismatch` |
| **Configuration** | `ConfigParse`, `ConfigInvalid` |
| **Project** | `ProjectRootNotFound`, `ProjectNotFound` |
| **Monorepo** | `MonorepoNotDetected`, `WorkspacePackageNotFound`, `CircularDependency` |
| **I/O** | `Io` (wrapping errors from `workspace-fs`) |

---

## 6. Non-Functional Requirements

### 6.1 Performance

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-1.1 | Repository type detection | < 5ms for typical project |
| NFR-1.2 | Package manager detection | < 10ms for typical project |
| NFR-1.3 | Project type detection | < 50ms for typical project |
| NFR-1.4 | Full monorepo analysis | < 500ms for 100 packages |
| NFR-1.5 | Memory usage | < 50MB for 100-package monorepo |

### 6.2 Reliability

| ID | Requirement |
|----|-------------|
| NFR-2.1 | System SHALL handle missing files gracefully |
| NFR-2.2 | System SHALL handle permission errors appropriately |
| NFR-2.3 | System SHALL handle malformed configuration files |
| NFR-2.4 | System SHALL handle symlinks according to configuration |

### 6.3 Compatibility

| ID | Requirement |
|----|-------------|
| NFR-3.1 | System SHALL work on Windows, macOS, and Linux |
| NFR-3.2 | System SHALL handle platform-specific path separators |
| NFR-3.3 | System SHALL support Rust stable (MSRV: 1.90+) |
| NFR-3.4 | System SHALL use Rust edition 2024 |

### 6.4 Code Quality

| ID | Requirement |
|----|-------------|
| NFR-4.1 | All public APIs SHALL be documented with examples |
| NFR-4.2 | Code coverage SHALL exceed 80% |
| NFR-4.3 | All clippy warnings SHALL be addressed |
| NFR-4.4 | No unsafe code without explicit justification |

### 6.5 Logging

| ID | Requirement |
|----|-------------|
| NFR-5.1 | System SHALL use the `log` crate for logging facade |
| NFR-5.2 | Logging SHALL be activated via presence of `RUST_LOG` environment variable |
| NFR-5.3 | System SHALL log all detection operations and decisions |
| NFR-5.4 | System SHALL NOT require any specific logging implementation (facade pattern) |

#### 6.5.1 Logging Levels

| Level | What to Log |
|-------|-------------|
| **Error** | Fatal errors that prevent operation completion |
| **Warn** | Ambiguous situations, inconsistencies, potential issues (e.g., multiple lock files, packageManager mismatch) |
| **Info** | High-level operations (e.g., "Detecting repository type at /path", "Found 15 workspace packages") |
| **Debug** | Detailed detection decisions (e.g., "Checking for deno.json", "Lock file pnpm-lock.yaml found", "Package manager priority order: [Pnpm, Yarn, ...]") |
| **Trace** | Very detailed operation traces (e.g., file access patterns, pattern matching results, iteration over workspace patterns) |

#### 6.5.2 Logging Examples

```rust
// Info level - high-level operations
log::info!("Detecting repository type at '{}'", path.display());
log::info!("Found {} workspace packages", packages.len());

// Debug level - detection decisions
log::debug!("Checking for characteristic file: deno.json");
log::debug!("Package manager detected: {} (via lock file)", kind);
log::debug!("Workspace patterns from config: {:?}", patterns);

// Warn level - ambiguous situations
log::warn!("Multiple lock files found: {:?}", lock_files);
log::warn!("packageManager field '{}' conflicts with lock file '{}'", declared, detected);

// Trace level - detailed traces
log::trace!("Checking path: {}", file_path.display());
log::trace!("Pattern match result: {} matches {}", path, pattern);
```

#### 6.5.3 Activation

- Logging is controlled by the standard `RUST_LOG` environment variable
- Examples:
  - `RUST_LOG=workspace_core=debug` - Debug level for this crate
  - `RUST_LOG=workspace_core=trace` - Trace level for this crate
  - `RUST_LOG=debug` - Debug level for all crates
- The consuming application is responsible for initializing a logging implementation (e.g., `env_logger`, `tracing-subscriber`)

---

## 7. Architecture Overview

### 7.1 Module Structure

```
workspace-core/
├── src/
│   ├── lib.rs              # Crate root with re-exports
│   ├── error.rs            # Unified error type (single Error enum)
│   ├── repo/               # Repository type and kind abstractions
│   │   ├── mod.rs
│   │   ├── repo_type.rs    # RepoType enum (Node, Deno, Bun)
│   │   ├── repo_kind.rs    # RepoKind enum (Simple, Monorepo)
│   │   └── tests.rs
│   ├── package_manager/    # Package manager abstractions
│   │   ├── mod.rs
│   │   ├── kind.rs         # PackageManagerKind enum
│   │   ├── detector.rs     # Detection logic
│   │   └── tests.rs
│   ├── package/            # Package identity and location
│   │   ├── mod.rs
│   │   ├── package.rs      # Package struct
│   │   └── tests.rs
│   ├── dependency/         # Dependency analysis
│   │   ├── mod.rs
│   │   ├── types.rs        # Dependency, PackageDependencies structs
│   │   ├── parser.rs       # Parse dependencies from PackageJson
│   │   ├── categorizer.rs  # Categorize internal vs external
│   │   └── tests.rs
│   ├── project/            # Unified project (uses package + dependency)
│   │   ├── mod.rs
│   │   ├── detector.rs
│   │   ├── project.rs      # Project struct
│   │   └── tests.rs
│   ├── monorepo/           # Monorepo-specific functionality
│   │   ├── mod.rs
│   │   ├── detector.rs
│   │   ├── workspace.rs    # Workspace config parsing
│   │   └── tests.rs
│   └── config/             # Configuration types
│       ├── mod.rs
│       ├── detection.rs    # DetectionConfig struct
│       ├── builder.rs      # DetectionConfigBuilder
│       └── tests.rs
└── tests/
    └── integration/        # E2E tests
```

### 7.2 Dependency Graph (Internal Modules)

```
┌─────────────────────────────────────────────────────────────────┐
│                      workspace-core                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│                      ┌───────────┐                              │
│                      │  project  │                              │
│                      └─────┬─────┘                              │
│                            │                                     │
│              ┌─────────────┼─────────────┐                      │
│              ▼             ▼             ▼                      │
│       ┌──────────┐  ┌───────────┐  ┌──────────┐                │
│       │ monorepo │  │  package  │  │  config  │                │
│       └────┬─────┘  └─────┬─────┘  └──────────┘                │
│            │              │                                      │
│            │              ▼                                      │
│            │       ┌────────────┐                               │
│            └──────►│ dependency │                               │
│                    └──────┬─────┘                               │
│                           │                                      │
│              ┌────────────┼────────────┐                        │
│              ▼            ▼            ▼                        │
│       ┌──────────┐  ┌──────────┐  ┌───────────────┐            │
│       │   repo   │  │  error   │  │package_manager│            │
│       └──────────┘  └──────────┘  └───────────────┘            │
│                                                                  │
│  ┌──────────┐    ┌──────────────────┐    ┌─────────────────┐   │
│  │  error   │◄───│  package_manager │◄───│    project      │   │
│  └──────────┘    └──────────────────┘    └─────────────────┘   │
│       ▲                 ▲                        ▲              │
│       │                 │                        │              │
│       │          ┌──────┴──────┐                 │              │
│       │          │    repo     │                 │              │
│       │          └─────────────┘                 │              │
│       │                 ▲                        │              │
│       │                 │                        │              │
│       └─────────────────┼────────────────────────┘              │
│                         │                                        │
│                  ┌──────┴──────┐                                │
│                  │  monorepo   │                                │
│                  └─────────────┘                                │
│                         ▲                                        │
│                         │                                        │
│                  ┌──────┴──────┐                                │
│                  │   config    │                                │
│                  └─────────────┘                                │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│                    External Dependencies                         │
│  ┌─────────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐    │
│  │workspace-fs │  │  snafu  │  │  serde  │  │     log     │    │
│  └─────────────┘  └─────────┘  └─────────┘  └─────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### 7.3 Key Design Principles

1. **Sync-First API**: Detection operations use synchronous I/O for simplicity. Async wrappers can be provided by consuming crates if needed.

2. **Zero Unsafe**: No unsafe code in this crate.

3. **Minimal Dependencies**: Only essential dependencies (snafu, serde, serde_json, serde_yaml, log, workspace-fs).

4. **Trait-Based Abstractions**: Core behaviors defined as traits for testability and extensibility.

5. **Builder Pattern**: Complex types constructed via builders for ergonomic APIs.

6. **Filesystem Abstraction**: All filesystem operations go through `workspace-fs`, enabling testing and future async support.

---

## 8. API Design Principles

### 8.1 Naming Conventions

- **Types**: PascalCase (e.g., `PackageManagerKind`, `MonorepoDescriptor`, `RepoType`)
- **Functions**: snake_case (e.g., `detect_package_manager`, `find_workspace_packages`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `DEFAULT_SEARCH_DEPTH`)
- **Modules**: snake_case (e.g., `package_manager`, `project`)

### 8.2 Path Handling

- All functions requiring a path take an explicit `&Path` parameter (required)
- NO fallback to current directory - caller is responsible for resolving paths
- This crate does NOT call `std::env::current_dir()` or similar
- Path resolution (including current directory fallback) is the responsibility of higher-level crates (e.g., CLI)

### 8.3 Error Handling

- All fallible operations return `Result<T>` (alias for `std::result::Result<T, Error>`)
- Single unified `Error` enum per crate (Rust idiom: one crate = one error type)
- Specific error variants for each failure mode within the enum
- Context preserved through error chain using `snafu`
- `#[snafu(display("..."))]` for actionable error messages

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
| All P1 requirements implemented | 100% coverage |
| Unit test coverage | > 80% |
| Integration tests passing | 100% |
| Documentation complete | All public APIs |
| Clippy clean | Zero warnings |

### 9.2 Quality Gates

1. **Code Review**: All code reviewed by at least one other developer
2. **CI/CD**: All tests pass on Windows, macOS, and Linux
3. **Documentation**: Generated docs reviewed for completeness
4. **Performance**: Benchmarks meet NFR targets

---

## 10. Future Considerations

### 10.1 Potential Extensions (Not in Scope)

- Turbo monorepo support
- Nx monorepo support
- Lerna monorepo support
- Rush monorepo support
- Custom monorepo configurations (P2)
- Project scaffolding

### 10.2 Migration Path

The crate is designed to eventually replace the functionality in `temp/wnt-stable/crates/standard`, but with:

- Cleaner module boundaries
- Sync-first APIs (vs async-first in the old crate)
- Better separation of concerns (RepoType, RepoKind, PackageManagerKind)
- Improved error handling with `snafu`
- Filesystem abstraction via `workspace-fs`

---

## 11. Glossary

| Term | Definition |
|------|------------|
| **Package Manager** | Tool for managing JavaScript dependencies (npm, yarn, pnpm, bun, deno) |
| **Lock File** | File that records exact dependency versions (package-lock.json, yarn.lock, etc.) |
| **Monorepo** | Repository containing multiple packages managed together |
| **Workspace** | Package manager feature for managing multiple packages in one repository |
| **Workspace Package** | Individual package within a monorepo workspace |
| **Project Root** | Directory containing the root package.json or deno.json |
| **RepoType** | The runtime ecosystem (Node, Deno, Bun) |
| **RepoKind** | The repository structure (Simple, Monorepo) |
| **PackageManagerKind** | The specific package manager tool used |

---

## 12. References

- [npm Workspaces Documentation](https://docs.npmjs.com/cli/v7/using-npm/workspaces)
- [Yarn Workspaces Documentation](https://yarnpkg.com/features/workspaces)
- [pnpm Workspaces Documentation](https://pnpm.io/workspaces)
- [Bun Workspaces Documentation](https://bun.sh/docs/install/workspaces)
- [Deno Workspaces Documentation](https://deno.land/manual/workspaces)
- [Node.js Corepack - packageManager field](https://nodejs.org/api/corepack.html)
- [snafu Error Handling](https://docs.rs/snafu/0.8.9/snafu/)
- [Original Crate Specification](../../temp/wnt-stable/crates/standard/SPEC.md)
- [Product PRD v2](../../docs/PRODUCT_PRD.md)

---

## Appendix: v2 Revision Notes

> Added 2026-02-07 as part of workspace-node-tools v2 Product PRD. See [PRODUCT_PRD.md](../../docs/PRODUCT_PRD.md) Section 5.2 for full context.

### R1: Switch from Sync-First to Async-First

**Change**: The existing PRD specifies sync-first detection (Section 7.3, Principle 1: "Sync-First API"). Since `workspace-fs` is async-first with `tokio::fs`, detection APIs SHALL be async.

**Impact on this PRD**:
- Section 7.3 Principle 1: Replace "Sync-First API" with "Async-First API: Detection operations use async I/O via workspace-fs."
- All detection functions (`detect_repo_type`, `detect_package_manager`, `detect_project`, `find_workspace_packages`) become `async fn`.
- Section 10.2 Migration Path: Remove "Sync-first APIs (vs async-first in the old crate)" -- both are now async.

### R2: Remove Configuration Loading Responsibility

**Change**: Configuration loading is delegated to `workspace-config` crate (new in v2).

**Impact on this PRD**:
- Section 5.7 (Configuration Module) remains as `DetectionConfig` -- this is **detection-specific configuration**, not file/TOML loading.
- This crate receives `DetectionConfig` programmatically from callers. It does NOT read `repo.config.toml`.
- The old `config/` module in `sublime_standard_tools` moves to `workspace-config`, NOT to this crate.

### R3: Package Manager Kind Simplification

**Change**: Old product had 9 PM kinds (`Npm`, `Yarn`, `YarnBerry`, `Pnpm`, `Bun`, `Lerna`, `Nx`, `Turbo`, `Rush`). New product simplifies to 5: `Npm`, `Yarn`, `Pnpm`, `Bun`, `Deno`.

**Rationale**: Lerna, Nx, Turbo, and Rush are meta-tools/task runners, not package managers. YarnBerry is merged into `Yarn` (version detection handled internally). Deno added as first-class PM.

**Impact on this PRD**:
- FR-2.1 (PackageManagerKind): 5 variants instead of 9.
- FR-2.2 (Detection): Simplified detection matrix.

### R4: Evaluate `package-json` Crate

**Action required during Phase 1 implementation**: Check if the `package-json` crate (v0.5.0, listed in dependencies) is still actively maintained. If not, implement a minimal `PackageJson` parser using `serde` + `serde_json` with only the fields needed:
- `name`, `version`, `private`, `workspaces`
- `dependencies`, `devDependencies`, `peerDependencies`, `optionalDependencies`
- `packageManager`

### R5: Edition 2024

**Change**: MSRV updated to Rust 1.90+, edition 2024.

**Impact**: Update `Cargo.toml` edition field. NFR-3.3 and NFR-3.4 already specify this.

### R6: No Other Changes

All other functional requirements (FR-1 through FR-8), non-functional requirements, architecture, and testing strategy remain valid as specified.