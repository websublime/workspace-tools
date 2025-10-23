# Type Relationships and Architecture

This document describes the relationships between the core types in `sublime_pkg_tools` and how they work together to provide comprehensive package management functionality.

## Overview

The type system in `sublime_pkg_tools` is organized around a few key concepts:

1. **Changesets** - The source of truth for what changes are being made
2. **Versions** - Semantic versions and version operations
3. **Packages** - Package metadata and information
4. **Dependencies** - Dependency relationships and updates
5. **Updates** - How packages are updated and why

## Core Type Hierarchy

```
┌─────────────────────────────────────────────────────────────┐
│                         Changeset                            │
│  (Source of Truth - What packages changed and how)           │
│                                                              │
│  - branch: BranchName                                        │
│  - bump: VersionBump                                         │
│  - environments: Vec<String>                                 │
│  - packages: HashSet<PackageName>                            │
│  - changes: Vec<CommitHash>                                  │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  │ drives
                  ↓
┌─────────────────────────────────────────────────────────────┐
│                    VersionResolution                         │
│        (Result of version resolution operation)              │
│                                                              │
│  - updates: Vec<PackageUpdate>                              │
│  - circular_dependencies: Vec<CircularDependency>           │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  │ contains
                  ↓
┌─────────────────────────────────────────────────────────────┐
│                     PackageUpdate                            │
│   (Single package version update with dependencies)          │
│                                                              │
│  - name: PackageName                                         │
│  - path: PathBuf                                             │
│  - current_version: Version                                  │
│  - next_version: Version                                     │
│  - reason: UpdateReason                                      │
│  - dependency_updates: Vec<DependencyUpdate>                │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  │ contains
                  ↓
┌─────────────────────────────────────────────────────────────┐
│                   DependencyUpdate                           │
│        (Dependency version specification update)             │
│                                                              │
│  - dependency_name: PackageName                              │
│  - dependency_type: DependencyType                           │
│  - old_version_spec: VersionSpec                             │
│  - new_version_spec: VersionSpec                             │
└─────────────────────────────────────────────────────────────┘
```

## Type Relationships by Domain

### 1. Version Management

**Primary Types:**
- `Version` - Semantic version (major.minor.patch)
- `VersionBump` - Type of version change (Major, Minor, Patch, None)
- `VersioningStrategy` - How versions are managed (Independent, Unified)

**Relationships:**
```
Version ──bumps with──> VersionBump ──produces──> Version
                                          ↓
                                    VersioningStrategy
                                    (determines how bump applies)
```

**Usage Flow:**
1. Start with a `Version` (e.g., "1.2.3")
2. Apply a `VersionBump` (e.g., Minor)
3. Get new `Version` (e.g., "1.3.0")
4. `VersioningStrategy` determines if all packages bump together (Unified) or independently (Independent)

### 2. Changeset Workflow

**Primary Types:**
- `Changeset` - Active changeset being worked on
- `ArchivedChangeset` - Released changeset with history
- `ReleaseInfo` - Metadata about when/how changeset was released
- `UpdateSummary` - Summary of changes made to a changeset

**Relationships:**
```
Changeset ──archives to──> ArchivedChangeset
    ↓                              ↓
packages: Set<PackageName>    contains: Changeset
changes: Vec<CommitHash>      + release_info: ReleaseInfo
    ↓
UpdateSummary (returned when modifying Changeset)
```

**Lifecycle:**
1. Create `Changeset` with branch, bump type, and environments
2. Add packages and commits (returns `UpdateSummary`)
3. Archive to `ArchivedChangeset` with `ReleaseInfo`
4. Query history of `ArchivedChangeset` instances

### 3. Package Information

**Primary Types:**
- `PackageInfo` - Complete package metadata
- `DependencyType` - Type of dependency (Regular, Dev, Peer, Optional)

**Relationships:**
```
PackageInfo
    ↓ contains
package_json: PackageJson ──has──> dependencies: HashMap<PackageName, VersionSpec>
    ↓                                      ↓
workspace: Option<WorkspacePackage>   categorized by DependencyType
    ↓
path: PathBuf
```

**Traits Implemented:**
- `Named` - Has a name (from package.json)
- `Versionable` - Has a version (from package.json)
- `Identifiable` - Has name@version identifier
- `HasDependencies` - Has dependencies, devDependencies, peerDependencies

### 4. Dependency Management

**Primary Types:**
- `DependencyUpdate` - Change to a dependency version spec
- `CircularDependency` - Detected circular dependency
- `UpdateReason` - Why a package is being updated
- `VersionProtocol` - Protocol prefix (workspace:, file:, etc.)
- `LocalLinkType` - Type of local link (File, Link, Portal)

**Relationships:**
```
PackageUpdate
    ↓ has
UpdateReason
    ├──> DirectChange (package in changeset)
    └──> DependencyPropagation { triggered_by, depth }
    ↓
dependency_updates: Vec<DependencyUpdate>
    ↓ each contains
    ├──> dependency_name: PackageName
    ├──> dependency_type: DependencyType
    ├──> old_version_spec: VersionSpec
    └──> new_version_spec: VersionSpec
         (may have VersionProtocol prefix)
```

**Protocol Handling:**
```
VersionSpec ──parsed by──> VersionProtocol
    ├──> Workspace (workspace:*)
    ├──> Local (file:/path, link:/path, portal:/path)
    │        └──> LocalLinkType
    └──> Semver (^1.2.3, >=2.0.0, etc.)
```

## Data Flow Patterns

### Pattern 1: Version Resolution Flow

```
Input: Changeset
    ↓
1. Load PackageInfo for all packages
    ↓
2. Build DependencyGraph
    ↓
3. Resolve versions based on VersioningStrategy
    ↓
4. Apply VersionBump to packages in changeset
    ↓
5. Propagate changes through dependency graph
    ↓
6. Detect CircularDependency instances
    ↓
Output: VersionResolution
    └──> contains: Vec<PackageUpdate>
                   └──> each has: Vec<DependencyUpdate>
```

### Pattern 2: Dependency Propagation Flow

```
PackageUpdate (Direct Change)
    ↓ triggers
Propagation Algorithm
    ↓ finds
Dependent Packages
    ↓ creates
PackageUpdate (DependencyPropagation)
    ↓ adds
DependencyUpdate (for updated dependency)
    ↓ may trigger
Further Propagation (recursive)
    └──> until: max_depth or no more dependents
```

### Pattern 3: Changeset Lifecycle

```
Create Changeset
    ├──> branch: BranchName
    ├──> bump: VersionBump
    └──> environments: Vec<String>
    ↓
Modify Changeset
    ├──> add_package() → UpdateSummary
    ├──> add_commits() → UpdateSummary
    └──> add_commits_from_git() → UpdateSummary
    ↓
Resolve Versions
    └──> VersionResolution → Vec<PackageUpdate>
    ↓
Apply Versions
    └──> ApplyResult
    ↓
Archive Changeset
    └──> ArchivedChangeset + ReleaseInfo
```

## Common Type Combinations

### 1. Creating a Version Update

```rust
// Start with a changeset
let changeset: Changeset = ...;

// Resolve versions
let resolution: VersionResolution = resolver.resolve_versions(&changeset).await?;

// Each update contains:
for update in resolution.updates {
    let name: PackageName = update.name;
    let current: Version = update.current_version;
    let next: Version = update.next_version;
    let reason: UpdateReason = update.reason;
    
    // Dependencies affected
    for dep_update in update.dependency_updates {
        let dep_name: PackageName = dep_update.dependency_name;
        let dep_type: DependencyType = dep_update.dependency_type;
        let old_spec: VersionSpec = dep_update.old_version_spec;
        let new_spec: VersionSpec = dep_update.new_version_spec;
    }
}
```

### 2. Working with Package Information

```rust
// Load package info
let package: PackageInfo = ...;

// Use trait methods
let name: &str = package.name();  // Named trait
let version: &Version = package.version();  // Versionable trait
let id: String = package.identifier();  // Identifiable trait

// Access dependencies
let deps = package.dependencies();  // HasDependencies trait
let dev_deps = package.dev_dependencies();
let peer_deps = package.peer_dependencies();
let all_deps = package.all_dependencies();
```

### 3. Checking Update Reasons

```rust
let update: PackageUpdate = ...;

match update.reason {
    UpdateReason::DirectChange => {
        println!("{} changed directly", update.name);
    }
    UpdateReason::DependencyPropagation { triggered_by, depth } => {
        println!("{} updated because {} changed (depth: {})", 
                 update.name, triggered_by, depth);
    }
}
```

## Type Aliases for Clarity

The crate provides type aliases for common string types:

- `PackageName` = `String` - Package names (e.g., "@myorg/core")
- `VersionSpec` = `String` - Version specifications (e.g., "^1.2.3")
- `CommitHash` = `String` - Git commit hashes
- `BranchName` = `String` - Git branch names

These improve code readability and self-documentation.

## Trait-Based Abstraction

Common capabilities are expressed through traits:

- **`Named`** - Has a name (`fn name(&self) -> &str`)
- **`Versionable`** - Has a version (`fn version(&self) -> &Version`)
- **`Identifiable`** - Has name and version (`fn identifier(&self) -> String`)
- **`HasDependencies`** - Has dependencies (`fn dependencies(&self) -> &HashMap<...>`)

This allows generic programming:

```rust
fn compare_versions<T: Versionable>(a: &T, b: &T) -> std::cmp::Ordering {
    a.version().cmp(b.version())
}

fn print_dependencies<T: HasDependencies>(pkg: &T) {
    for (name, spec) in pkg.all_dependencies() {
        println!("{}: {}", name, spec);
    }
}
```

## Best Practices

### 1. Use Type Aliases
```rust
// Good
fn update_package(name: PackageName, version: VersionSpec) -> Result<()> { ... }

// Less clear
fn update_package(name: String, version: String) -> Result<()> { ... }
```

### 2. Use Traits for Generic Code
```rust
// Good - works with any type implementing Identifiable
fn format_updates<T: Identifiable>(items: &[T]) -> Vec<String> {
    items.iter().map(|item| item.identifier()).collect()
}
```

### 3. Use the Prelude for Convenience
```rust
// Instead of multiple imports
use sublime_pkg_tools::types::{Version, VersionBump, Changeset};
use sublime_pkg_tools::types::{Named, Versionable};

// Use prelude
use sublime_pkg_tools::types::prelude::*;
```

### 4. Understand Update Reasons
```rust
// Check why a package is being updated
if update.is_direct_change() {
    // Handle direct changes
} else if update.is_propagated() {
    // Handle propagated changes
}
```

## See Also

- [API Documentation](https://docs.rs/sublime_pkg_tools)
- [CONCEPT.md](../CONCEPT.md) - High-level design concepts
- [PLAN.md](../PLAN.md) - Implementation plan and module structure
- [Examples](../examples/) - Code examples demonstrating common patterns