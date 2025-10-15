# sublime_pkg_tools - Concept Document

**Status**: ✅ DONE - Ready for Implementation  
**Last Updated**: 2024-01-15  
**Version**: 1.0  
**Purpose**: Complete specification for changeset-based package version management library

---

## Overview

`sublime_pkg_tools` is a **library crate** for changeset-based version management in Node.js projects (single-package and monorepos). It provides data structures and APIs to create, update, archive, and query changesets that explicitly declare what packages are being released and why.

**This is a library crate, not a CLI tool.** CLI tools, Git hooks, and interactive workflows are built **on top** of this library.

---

## Core Philosophy

### 1. Changeset as Source of Truth

The changeset is an explicit, reviewable declaration of release intent:
- Branch being worked on
- Version bump to apply (minor, major, patch)
- Target deployment environments
- Packages affected
- Commits included (by ID)

### 2. Simple Data Model

Changesets are simple JSON files stored in `.changesets/`:
```json
{
  "branch": "feat/oauth-integration",
  "bump": "minor",
  "environments": ["int", "stage", "test", "prod"],
  "packages": ["@myorg/auth", "@myorg/core"],
  "changes": ["abc123def", "456ghi789"],
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T14:20:00Z"
}
```

### 3. Library Not CLI

This crate provides:
- ✅ Data structures (`Changeset`, etc)
- ✅ Storage operations (save, load, archive)
- ✅ Core APIs (create, update, query)
- ✅ Git integration helper (add commits automatically)
- ❌ CLI commands
- ❌ Interactive prompts
- ❌ Git hooks

External tools (CLIs, CI/CD, hooks) use this library.

### 4. Minimal Git Integration

The only Git integration is a helper function that:
- Uses `sublime_git_tools` to get new commits
- Detects affected packages
- Adds commit IDs to the changeset

All other Git operations are external to this crate.

---

## Data Structures

### Changeset

Core structure representing a set of changes to be released.

```rust
/// Represents a changeset for a branch.
pub struct Changeset {
    /// Branch name (e.g., "feat/oauth-integration")
    pub branch: String,
    
    /// Version bump to apply to all affected packages
    pub bump: VersionBump,
    
    /// Target deployment environments
    pub environments: Vec<String>,
    
    /// Package names affected (e.g., ["@myorg/auth", "@myorg/core"])
    pub packages: Vec<String>,
    
    /// Commit IDs included in this changeset
    pub changes: Vec<String>,
    
    /// When changeset was created
    pub created_at: DateTime<Utc>,
    
    /// When changeset was last updated
    pub updated_at: DateTime<Utc>,
}
```

### VersionBump

Simple enum for version bump types.

```rust
/// Type of version bump.
pub enum VersionBump {
    /// Major version bump (1.0.0 → 2.0.0)
    Major,
    
    /// Minor version bump (1.0.0 → 1.1.0)
    Minor,
    
    /// Patch version bump (1.0.0 → 1.0.1)
    Patch,
    
    /// No version change
    None,
}
```

### ArchivedChangeset

Changeset with release metadata after being applied.

```rust
/// Changeset after being released and archived.
pub struct ArchivedChangeset {
    /// Original changeset data
    pub changeset: Changeset,
    
    /// Release metadata
    pub release_info: ReleaseInfo,
}

/// Release metadata added when changeset is archived.
pub struct ReleaseInfo {
    /// When release was applied
    pub applied_at: DateTime<Utc>,
    
    /// Who applied the release (e.g., "ci-bot", "user@example.com")
    pub applied_by: String,
    
    /// Git commit hash of the release commit
    pub git_commit: String,
    
    /// Actual versions released per package
    pub versions: HashMap<String, String>,
}
```

---

## Core APIs

### ChangesetManager

Main interface for changeset operations.

```rust
/// Manager for changeset operations.
pub struct ChangesetManager {
    storage: Box<dyn ChangesetStorage>,
    git_repo: Repo,  // from sublime_git_tools
}

impl ChangesetManager {
    /// Create a new changeset.
    pub async fn create(
        &self,
        branch: String,
        bump: VersionBump,
        environments: Vec<String>,
    ) -> Result<Changeset, ChangesetError>;
    
    /// Load an existing changeset by branch name.
    pub async fn load(&self, branch: &str) -> Result<Changeset, ChangesetError>;
    
    /// Update changeset properties.
    pub async fn update(
        &self,
        changeset: &mut Changeset,
    ) -> Result<(), ChangesetError>;
    
    /// Add commit IDs to changeset (manual).
    pub async fn add_commits(
        &self,
        changeset: &mut Changeset,
        commit_ids: Vec<String>,
    ) -> Result<(), ChangesetError>;
    
    /// Add commits from Git automatically.
    /// Detects new commits since last update and affected packages.
    pub async fn add_commits_from_git(
        &self,
        changeset: &mut Changeset,
    ) -> Result<UpdateSummary, ChangesetError>;
    
    /// Archive changeset to history.
    pub async fn archive(
        &self,
        changeset: &Changeset,
        release_info: ReleaseInfo,
    ) -> Result<(), ChangesetError>;
    
    /// List all pending changesets.
    pub async fn list_pending(&self) -> Result<Vec<Changeset>, ChangesetError>;
    
    /// Delete a pending changeset.
    pub async fn delete(&self, branch: &str) -> Result<(), ChangesetError>;
}
```

### History Query API

Query archived changesets.

```rust
/// Query interface for changeset history.
pub struct ChangesetHistory {
    storage: Box<dyn ChangesetStorage>,
}

impl ChangesetHistory {
    /// List all archived changesets.
    pub async fn list_all(&self) -> Result<Vec<ArchivedChangeset>, ChangesetError>;
    
    /// Get archived changeset by branch name.
    pub async fn get(&self, branch: &str) -> Result<ArchivedChangeset, ChangesetError>;
    
    /// Query changesets by date range.
    pub async fn query_by_date(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<ArchivedChangeset>, ChangesetError>;
    
    /// Query changesets by package name.
    pub async fn query_by_package(
        &self,
        package: &str,
    ) -> Result<Vec<ArchivedChangeset>, ChangesetError>;
    
    /// Query changesets by environment.
    pub async fn query_by_environment(
        &self,
        environment: &str,
    ) -> Result<Vec<ArchivedChangeset>, ChangesetError>;
    
    /// Query changesets by version bump type.
    pub async fn query_by_bump(
        &self,
        bump: VersionBump,
    ) -> Result<Vec<ArchivedChangeset>, ChangesetError>;
}
```

### UpdateSummary

Information returned when adding commits from Git.

```rust
/// Summary of what was updated when adding commits from Git.
pub struct UpdateSummary {
    /// Number of commits added
    pub commits_added: usize,
    
    /// Commit IDs that were added
    pub commit_ids: Vec<String>,
    
    /// Packages that were newly detected
    pub new_packages: Vec<String>,
    
    /// Packages that already existed
    pub existing_packages: Vec<String>,
}
```

---

## Storage

### ChangesetStorage Trait

Abstraction for changeset persistence.

```rust
#[async_trait]
pub trait ChangesetStorage: Send + Sync {
    /// Save a changeset (create or update).
    async fn save(&self, changeset: &Changeset) -> Result<(), ChangesetError>;
    
    /// Load a changeset by branch name.
    async fn load(&self, branch: &str) -> Result<Changeset, ChangesetError>;
    
    /// Check if changeset exists for branch.
    async fn exists(&self, branch: &str) -> Result<bool, ChangesetError>;
    
    /// Delete a changeset.
    async fn delete(&self, branch: &str) -> Result<(), ChangesetError>;
    
    /// List all pending changesets.
    async fn list_pending(&self) -> Result<Vec<Changeset>, ChangesetError>;
    
    /// Archive a changeset (move to history).
    async fn archive(
        &self,
        changeset: &Changeset,
        release_info: ReleaseInfo,
    ) -> Result<(), ChangesetError>;
    
    /// Load archived changeset.
    async fn load_archived(&self, branch: &str) -> Result<ArchivedChangeset, ChangesetError>;
    
    /// List all archived changesets.
    async fn list_archived(&self) -> Result<Vec<ArchivedChangeset>, ChangesetError>;
}
```

### FileBasedChangesetStorage

Default filesystem implementation.

```rust
/// Filesystem-based changeset storage.
pub struct FileBasedChangesetStorage {
    root_path: PathBuf,
    changeset_dir: PathBuf,      // .changesets/
    history_dir: PathBuf,         // .changesets/history/
    fs: FileSystemManager,        // from sublime_standard_tools
}

impl FileBasedChangesetStorage {
    /// Create new storage at given root path.
    pub fn new(
        root_path: PathBuf,
        fs: FileSystemManager,
    ) -> Self;
    
    /// Get path for a changeset file.
    fn changeset_path(&self, branch: &str) -> PathBuf {
        // .changesets/{sanitized_branch}.json
    }
    
    /// Get path for archived changeset.
    fn archive_path(&self, branch: &str) -> PathBuf {
        // .changesets/history/{sanitized_branch}.json
    }
}
```

**File Structure**:
```
.changesets/
├── feat-oauth-integration.json      (pending)
├── fix-memory-leak.json             (pending)
└── history/
    ├── feat-user-auth.json          (archived)
    └── fix-security-bug.json        (archived)
```

---

## Git Integration

### Package Detection

Helper to detect which packages are affected by commits.

```rust
/// Detects packages affected by Git commits.
pub struct PackageDetector {
    workspace_root: PathBuf,
    repo: Repo,  // from sublime_git_tools
}

impl PackageDetector {
    /// Detect packages affected by given commits.
    pub async fn detect_affected_packages(
        &self,
        commit_ids: &[String],
    ) -> Result<Vec<String>, PackageError>;
    
    /// Detect if workspace is monorepo.
    pub async fn is_monorepo(&self) -> Result<bool, PackageError>;
    
    /// List all packages in workspace.
    pub async fn list_packages(&self) -> Result<Vec<String>, PackageError>;
}
```

### Add Commits Helper

Implementation of `add_commits_from_git`.

```rust
impl ChangesetManager {
    pub async fn add_commits_from_git(
        &self,
        changeset: &mut Changeset,
    ) -> Result<UpdateSummary, ChangesetError> {
        // 1. Get last commit ID in changeset (or None if empty)
        let since_commit = changeset.changes.last();
        
        // 2. Get new commits from Git using sublime_git_tools
        let new_commits = if let Some(last) = since_commit {
            self.git_repo.get_commits_since(last)?
        } else {
            // Get all commits on current branch
            self.git_repo.get_commits_on_branch(&changeset.branch)?
        };
        
        if new_commits.is_empty() {
            return Ok(UpdateSummary::empty());
        }
        
        // 3. Extract commit IDs
        let commit_ids: Vec<String> = new_commits
            .iter()
            .map(|c| c.hash.clone())
            .collect();
        
        // 4. Detect affected packages
        let detector = PackageDetector::new(
            self.workspace_root.clone(),
            self.git_repo.clone(),
        );
        
        let affected_packages = detector
            .detect_affected_packages(&commit_ids)
            .await?;
        
        // 5. Update changeset
        let new_packages: Vec<String> = affected_packages
            .into_iter()
            .filter(|pkg| !changeset.packages.contains(pkg))
            .collect();
        
        changeset.packages.extend(new_packages.clone());
        changeset.changes.extend(commit_ids.clone());
        changeset.updated_at = Utc::now();
        
        // 6. Save
        self.storage.save(changeset).await?;
        
        // 7. Return summary
        Ok(UpdateSummary {
            commits_added: commit_ids.len(),
            commit_ids,
            new_packages,
            existing_packages: changeset.packages.clone(),
        })
    }
}
```

---

## Configuration

### PackageToolsConfig

Configuration structure (uses `sublime_standard_tools` config system).

```toml
# repo.config.toml

[package_tools]

[package_tools.changeset]
# Directory for changesets
path = ".changesets"

# Directory for archived changesets
history_path = ".changesets/history"

# Available deployment environments
available_environments = ["int", "stage", "test", "prod"]

# Default environments when creating changeset
default_environments = ["int"]

[package_tools.version]
# Default bump type if not specified
default_bump = "patch"
```

```rust
/// Configuration for package tools.
pub struct PackageToolsConfig {
    pub changeset: ChangesetConfig,
    pub version: VersionConfig,
}

pub struct ChangesetConfig {
    pub path: PathBuf,
    pub history_path: PathBuf,
    pub available_environments: Vec<String>,
    pub default_environments: Vec<String>,
}

pub struct VersionConfig {
    pub default_bump: VersionBump,
}
```

---

## Usage Examples

### Example 1: Create Changeset

```rust
use sublime_pkg_tools::{ChangesetManager, VersionBump};

let manager = ChangesetManager::new(storage, repo);

let changeset = manager.create(
    "feat/oauth-integration".to_string(),
    VersionBump::Minor,
    vec!["int".to_string(), "stage".to_string()],
).await?;

println!("Created changeset for branch: {}", changeset.branch);
```

### Example 2: Add Commits from Git

```rust
// Load existing changeset
let mut changeset = manager.load("feat/oauth-integration").await?;

// Add new commits automatically
let summary = manager.add_commits_from_git(&mut changeset).await?;

println!("Added {} commits", summary.commits_added);
println!("New packages detected: {:?}", summary.new_packages);
```

### Example 3: Update Changeset Properties

```rust
// Load changeset
let mut changeset = manager.load("feat/oauth-integration").await?;

// Update properties
changeset.bump = VersionBump::Major;  // Change to major
changeset.environments.push("prod".to_string());  // Add prod

// Save changes
manager.update(&mut changeset).await?;
```

### Example 4: Archive Changeset

```rust
// Load changeset
let changeset = manager.load("feat/oauth-integration").await?;

// Create release info
let release_info = ReleaseInfo {
    applied_at: Utc::now(),
    applied_by: "ci-bot".to_string(),
    git_commit: "abc123def456".to_string(),
    versions: HashMap::from([
        ("@myorg/auth".to_string(), "1.3.0".to_string()),
        ("@myorg/core".to_string(), "2.1.0".to_string()),
    ]),
};

// Archive
manager.archive(&changeset, release_info).await?;

println!("Changeset archived to history");
```

### Example 5: Query History

```rust
use sublime_pkg_tools::ChangesetHistory;

let history = ChangesetHistory::new(storage);

// Query by package
let changesets = history
    .query_by_package("@myorg/auth")
    .await?;

println!("Found {} changesets for @myorg/auth", changesets.len());

// Query by date range
let from = Utc.ymd(2024, 1, 1).and_hms(0, 0, 0);
let to = Utc::now();
let changesets = history.query_by_date(from, to).await?;

println!("Changesets in 2024: {}", changesets.len());
```

---

## Module Structure

```
sublime_pkg_tools/
├── changeset/
│   ├── mod.rs              # Public exports
│   ├── models.rs           # Changeset, ArchivedChangeset, ReleaseInfo
│   ├── manager.rs          # ChangesetManager
│   ├── history.rs          # ChangesetHistory query API
│   ├── storage.rs          # Storage trait and FileBasedChangesetStorage
│   └── detector.rs         # PackageDetector
│
├── version/
│   ├── mod.rs              # Public exports
│   ├── models.rs           # VersionBump, Version
│   └── bumper.rs           # Version bump logic
│
├── config/
│   └── models.rs           # PackageToolsConfig
│
├── error/
│   └── models.rs           # ChangesetError, PackageError
│
└── lib.rs                  # Crate root
```

---

## Dependencies

### Internal
- `sublime_git_tools` - Git operations (get commits, detect changes)
- `sublime_standard_tools` - Filesystem, configuration, monorepo detection

### External
- `serde` / `serde_json` - Serialization
- `chrono` - Date/time handling
- `thiserror` - Error types
- `async-trait` - Async traits
- `tokio` - Async runtime

---

## Design Principles

### 1. Library First
This is a library crate. No CLI, no interactivity, no assumptions about how it's used.

### 2. Simple Data Model
Changesets are simple JSON files with minimal structure.

### 3. Single Responsibility
Each component has one clear job:
- Manager: orchestrate operations
- Storage: persist data
- Detector: find affected packages
- History: query archives

### 4. Testability
All components are mockable and testable in isolation.

### 5. No Opinionated Workflow
The library doesn't enforce how/when changesets are created or updated. That's up to the caller.

---

## Non-Goals

1. **CLI tool** - External concern
2. **Git hooks** - External concern
3. **Interactive prompts** - External concern
4. **Conventional commit parsing** - Not needed (only commit IDs stored)
5. **Version bump calculation** - Bump is explicit in changeset
6. **Package.json modification** - External concern (release tooling)
7. **Publishing** - External concern
8. **Changelog generation** - Could be separate feature, not core

---

## Success Criteria

### Functional
- [x] Create changeset with branch, bump, environments
- [x] Update changeset properties (bump, environments, packages, commits)
- [x] Add commits manually (by ID)
- [x] Add commits from Git automatically (detect affected packages)
- [x] Archive changeset with release metadata
- [x] Query history by package, date, environment, bump type
- [x] Store as JSON files in `.changesets/` and `.changesets/history/`

### Quality
- [x] 100% test coverage
- [x] All Clippy rules passing
- [x] Comprehensive documentation
- [x] Cross-platform support

### API Design
- [x] Clear, minimal API surface
- [x] Mockable/testable components
- [x] Async/await throughout
- [x] Proper error handling

---

## Open Questions

1. **Bump per package or global?**  
   → **Decision**: Global (simpler, covers most cases)

2. **Changeset filename format?**  
   → Proposal: `{sanitized-branch}.json`  
   → Example: `feat-oauth-integration.json`

3. **How to handle multiple changesets for same branch?**  
   → One changeset per branch (update existing, don't create new)

4. **Should we validate environments against config?**  
   → Yes, validate on create/update

5. **Should detector be part of this crate or separate?**  
   → Part of this crate (core functionality)

---

## Versioning & Dependency Propagation

This section details how version bumps are calculated, applied, and propagated through dependency chains.

---

### Overview

When a changeset is applied:
1. **Direct packages** receive the bump specified in the changeset
2. **Dependent packages** are automatically updated via propagation
3. **Dependencies are updated** to reference new versions
4. All changes can be **previewed** before applying (dry-run)

### Versioning Strategy

Configured globally in `repo.config.toml`:

```toml
[package_tools.version]
strategy = "independent"  # or "unified"
```

#### Independent Strategy (Default)

Each package maintains its own version independently.

**Example**:
```
Before: @myorg/auth@1.2.3, @myorg/core@2.5.1, @myorg/api@0.8.0
After:  @myorg/auth@1.3.0, @myorg/core@2.5.2, @myorg/api@0.8.1
```

Packages are bumped based on:
- Direct changes (from changeset)
- Dependency propagation

#### Unified Strategy

All packages share the same version, bumped together.

**Example**:
```
Before: @myorg/auth@1.2.3, @myorg/core@1.2.3, @myorg/api@1.2.3
After:  @myorg/auth@1.3.0, @myorg/core@1.3.0, @myorg/api@1.3.0
```

Even if only one package has changes, all packages are bumped.

---

### Dependency Propagation

When a package version changes, all packages that depend on it must also be updated.

#### Propagation Rules

1. **Trigger**: Package A is updated (from changeset)
2. **Detect**: Find all packages that depend on A
3. **Update**: Apply propagation bump to dependent packages
4. **Cascade**: Repeat for newly updated packages (A → B → C → D)
5. **Update deps**: Update dependency references in package.json

#### Dependency Types

Propagation affects all dependency types:
- `dependencies`
- `devDependencies`
- `peerDependencies`

Configurable per type:
```toml
[package_tools.dependency]
propagate_dependencies = true
propagate_dev_dependencies = true
propagate_peer_dependencies = true
```

#### Version Spec Skipping

Dependency version specifications using workspace protocols or local references are **skipped** and not updated:

**Skipped patterns**:
- `workspace:*` - Workspace protocol (pnpm, yarn)
- `workspace:^` - Workspace protocol with caret
- `workspace:~` - Workspace protocol with tilde
- `file:*` - Local file references
- `link:*` - Local link references
- `portal:*` - Portal references (yarn)

**Example**:
```json
{
  "dependencies": {
    "@myorg/core": "workspace:*",      // SKIPPED - not updated
    "@myorg/utils": "^1.2.3",          // UPDATED normally
    "local-lib": "file:../local-lib",  // SKIPPED - not updated
    "external-lib": "^2.0.0"           // UPDATED normally
  }
}
```

**Rationale**: These patterns represent workspace-managed or local dependencies that should not be version-bumped as they're resolved by the package manager or are local filesystem references.

#### Propagation Bump

Configurable bump type for propagated updates:

```toml
[package_tools.dependency]
propagation_bump = "patch"  # or "minor", "major", "none"
```

Default: `patch`

#### Example: Propagation Chain

**Setup**:
```
@myorg/core (no dependencies)
@myorg/auth (depends on core)
@myorg/api (depends on auth)
@myorg/web (depends on api)
```

**Changeset**:
```json
{
  "branch": "feat/oauth",
  "bump": "minor",
  "packages": ["@myorg/core"],
  "changes": ["abc123"]
}
```

**Resolution** (with `propagation_bump = "patch"`):

```
@myorg/core
  1.2.3 → 1.3.0 (minor - from changeset)
  
@myorg/auth (depends on core)
  2.1.0 → 2.1.1 (patch - propagated)
  dependencies: { "@myorg/core": "^1.2.3" → "^1.3.0" }
  
@myorg/api (depends on auth)
  3.5.0 → 3.5.1 (patch - propagated)
  dependencies: { "@myorg/auth": "^2.1.0" → "^2.1.1" }
  
@myorg/web (depends on api)
  0.9.0 → 0.9.1 (patch - propagated)
  dependencies: { "@myorg/api": "^3.5.0" → "^3.5.1" }
```

#### Circular Dependencies

Circular dependencies are **detected** but do **not** prevent updates:

```
@myorg/auth (depends on utils)
@myorg/utils (depends on auth)
```

Both packages are updated in the same pass, avoiding infinite loops.

---

### Snapshot Versions

Snapshot versions enable deploying branches to environments before merging.

#### Format

```
{base_version}-{sanitized_branch}.{commit_short}
```

**Examples**:
```
1.2.3-feat-oauth.abc123d
2.5.0-fix-memory-leak.def456a
0.8.0-refactor-api.ghi789b
```

#### Generation

Snapshots are generated on-demand:

```rust
let snapshots = version_resolver
    .generate_snapshots(&changeset, "abc123def456")
    .await?;

for snapshot in snapshots {
    println!("{}: {}", snapshot.package_name, snapshot);
}
```

**Output**:
```
@myorg/auth: 1.2.3-feat-oauth.abc123d
@myorg/core: 2.5.0-feat-oauth.abc123d
```

#### Use Case

1. Developer works on `feat/oauth` branch
2. Commits changes
3. CI generates snapshot versions
4. Deploys to `int` environment with snapshot versions
5. Testing happens with `@myorg/auth@1.2.3-feat-oauth.abc123d`
6. After merge, releases with real version `1.3.0`

---

### Version Resolution (Dry Run)

Preview what will happen before applying changes.

#### API

```rust
let resolution = version_resolver
    .resolve_versions(&changeset)
    .await?;

// Preview
for update in &resolution.updates {
    println!(
        "{}: {} → {} ({})",
        update.name,
        update.current_version,
        update.next_version,
        match &update.reason {
            UpdateReason::DirectChange => "direct".to_string(),
            UpdateReason::DependencyPropagation { triggered_by, depth } => 
                format!("depends on {} (depth {})", triggered_by, depth),
        }
    );
    
    for dep_update in &update.dependency_updates {
        println!(
            "  {} {} → {}",
            dep_update.dependency_name,
            dep_update.old_version_spec,
            dep_update.new_version_spec
        );
    }
}
```

**Example Output**:
```
@myorg/core: 1.2.3 → 1.3.0 (direct)

@myorg/auth: 2.1.0 → 2.1.1 (depends on @myorg/core, depth 1)
  @myorg/core: ^1.2.3 → ^1.3.0

@myorg/api: 3.5.0 → 3.5.1 (depends on @myorg/auth, depth 2)
  @myorg/auth: ^2.1.0 → ^2.1.1

@myorg/web: 0.9.0 → 0.9.1 (depends on @myorg/api, depth 3)
  @myorg/api: ^3.5.0 → ^3.5.1
```

#### Resolution Result

```rust
pub struct VersionResolution {
    /// All packages to be updated
    pub updates: Vec<PackageUpdate>,
    
    /// Circular dependencies detected (if any)
    pub circular_dependencies: Vec<CircularDependency>,
}

pub struct PackageUpdate {
    /// Package name
    pub name: String,
    
    /// Path to package directory
    pub path: PathBuf,
    
    /// Current version (from package.json)
    pub current_version: Version,
    
    /// Next version after bump
    pub next_version: Version,
    
    /// Why this package is being updated
    pub reason: UpdateReason,
    
    /// Dependency version updates in this package
    pub dependency_updates: Vec<DependencyUpdate>,
}

pub enum UpdateReason {
    /// Direct change (in changeset)
    DirectChange,
    
    /// Propagated due to dependency update
    DependencyPropagation {
        /// Package that triggered this update
        triggered_by: String,
        /// Depth in dependency chain (1 = direct dependent)
        depth: usize,
    },
}

pub struct DependencyUpdate {
    /// Dependency package name
    pub dependency_name: String,
    
    /// Dependency type
    pub dependency_type: DependencyType,
    
    /// Old version specification
    pub old_version_spec: String,
    
    /// New version specification
    pub new_version_spec: String,
}

pub enum DependencyType {
    Regular,      // dependencies
    Dev,          // devDependencies
    Peer,         // peerDependencies
}

pub struct CircularDependency {
    /// Packages involved in the cycle
    pub cycle: Vec<String>,
}
```

---

### Applying Versions

Apply version changes to package.json files.

#### API with Dry-Run Flag

```rust
pub struct VersionResolver {
    // ...
}

impl VersionResolver {
    /// Apply version changes from changeset.
    /// 
    /// # Arguments
    /// 
    /// * `changeset` - The changeset to apply
    /// * `dry_run` - If true, only preview changes (default: true)
    /// 
    /// # Returns
    /// 
    /// Result with applied changes summary
    pub async fn apply_versions(
        &self,
        changeset: &Changeset,
        dry_run: bool,
    ) -> Result<ApplyResult, VersionError>;
}
```

#### Apply Result

```rust
pub struct ApplyResult {
    /// Dry run mode (true = no changes made)
    pub dry_run: bool,
    
    /// Version resolution details
    pub resolution: VersionResolution,
    
    /// Files modified (if not dry run)
    pub modified_files: Vec<PathBuf>,
    
    /// Summary of changes
    pub summary: ApplySummary,
}

pub struct ApplySummary {
    /// Number of packages updated
    pub packages_updated: usize,
    
    /// Number of direct updates (from changeset)
    pub direct_updates: usize,
    
    /// Number of propagated updates
    pub propagated_updates: usize,
    
    /// Number of dependency references updated
    pub dependency_updates: usize,
    
    /// Circular dependencies detected
    pub circular_dependencies: usize,
}
```

#### Usage Example

```rust
// Preview (dry run = true)
let result = version_resolver
    .apply_versions(&changeset, true)
    .await?;

println!("Dry Run Summary:");
println!("  Packages updated: {}", result.summary.packages_updated);
println!("  Direct updates: {}", result.summary.direct_updates);
println!("  Propagated updates: {}", result.summary.propagated_updates);
println!("  Dependency updates: {}", result.summary.dependency_updates);

if !result.resolution.circular_dependencies.is_empty() {
    println!("  ⚠️  Circular dependencies detected: {}", 
        result.summary.circular_dependencies);
}

// Apply (dry run = false) - WRITES to package.json files
let result = version_resolver
    .apply_versions(&changeset, false)
    .await?;

println!("\nApplied {} changes to:", result.modified_files.len());
for file in &result.modified_files {
    println!("  - {}", file.display());
}
```

**Important**: When `dry_run = false`, the `apply_versions` method:
1. **Writes new versions** to all affected `package.json` files
2. **Updates dependency references** in `dependencies`, `devDependencies`, `peerDependencies`
3. Uses `FileSystemManager` from `sublime_standard_tools` for all file operations
4. Handles both **monorepo** (multiple package.json files) and **single-package** (one package.json) projects automatically
5. **Skips** workspace protocols and local references (workspace:*, file:, link:, portal:)

---

### Configuration

Configuration is managed by `sublime_standard_tools` config system. This crate **extends** the `StandardConfig` with package-specific settings.

#### Configuration File Structure

```toml
# repo.config.toml (or .yml, .yaml, .json)

# Standard tools configuration (from sublime_standard_tools)
[standard]
version = "1.0"

[standard.package_managers]
detection_order = ["pnpm", "yarn", "npm"]

[standard.monorepo]
workspace_patterns = ["packages/*", "apps/*"]

# Package tools configuration (this crate - extends standard)
[package_tools]

[package_tools.changeset]
path = ".changesets"
history_path = ".changesets/history"
available_environments = ["int", "stage", "test", "prod"]
default_environments = ["int"]

[package_tools.version]
# Versioning strategy: "independent" or "unified"
strategy = "independent"

# Default bump if not specified
default_bump = "patch"

# Snapshot version format
snapshot_format = "{version}-{branch}.{commit}"

[package_tools.dependency]
# Bump type for propagated updates
propagation_bump = "patch"

# Which dependency types to propagate through
propagate_dependencies = true
propagate_dev_dependencies = true
propagate_peer_dependencies = true

# Maximum propagation depth (0 = unlimited)
max_depth = 0

# Fail on circular dependencies (if false, just warns)
fail_on_circular = false

# Skip workspace protocols and local references
skip_workspace_protocol = true  # workspace:*
skip_file_protocol = true       # file:
skip_link_protocol = true       # link:
skip_portal_protocol = true     # portal:

[package_tools.git]
# Merge commit message template for releases
# Available variables:
#   {version} - New version
#   {previous_version} - Previous version
#   {package_name} - Package name (monorepo only)
#   {bump_type} - Major, Minor, or Patch
#   {date} - Release date (YYYY-MM-DD)
#   {breaking_changes_count} - Number of breaking changes
#   {features_count} - Number of new features
#   {fixes_count} - Number of bug fixes
#   {changelog_summary} - Brief summary from changelog
#   {author} - Current git user

# Single package merge commit template
merge_commit_template = """chore(release): {version}

Release version {version}

{changelog_summary}
"""

# Monorepo package merge commit template
monorepo_merge_commit_template = """chore(release): {package_name}@{version}

Release {package_name} version {version}

{changelog_summary}
"""

# Include breaking changes warning in merge commit
include_breaking_warning = true

# Breaking changes warning template
breaking_warning_template = """
⚠️  BREAKING CHANGES: {breaking_changes_count}
"""
```

#### Configuration Loading

Uses `ConfigManager` from `sublime_standard_tools`:

```rust
use sublime_standard_tools::config::{ConfigManager, Configurable};
use sublime_pkg_tools::config::PackageToolsConfig;

// Load configuration from project
let config_manager = ConfigManager::<PackageToolsConfig>::builder()
    .with_defaults()
    .with_file("repo.config.toml")
    .with_env_prefix("SUBLIME_PACKAGE_TOOLS")
    .build()?;

let config = config_manager.load().await?;

println!("Strategy: {:?}", config.version.strategy);
println!("Propagation bump: {:?}", config.dependency.propagation_bump);
```

#### Environment Variable Overrides

All configuration can be overridden via environment variables:

```bash
# Version strategy
export SUBLIME_PACKAGE_TOOLS_VERSION_STRATEGY=unified

# Propagation bump
export SUBLIME_PACKAGE_TOOLS_DEPENDENCY_PROPAGATION_BUMP=minor

# Changeset path
export SUBLIME_PACKAGE_TOOLS_CHANGESET_PATH=.releases

# Skip protocols
export SUBLIME_PACKAGE_TOOLS_DEPENDENCY_SKIP_WORKSPACE_PROTOCOL=false
```

#### PackageToolsConfig Structure

```rust
use serde::{Deserialize, Serialize};
use sublime_standard_tools::config::traits::Configurable;

/// Configuration for package tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageToolsConfig {
    pub changeset: ChangesetConfig,
    pub version: VersionConfig,
    pub dependency: DependencyConfig,
    pub git: GitConfig,
}

impl Default for PackageToolsConfig {
    fn default() -> Self {
        Self {
            changeset: ChangesetConfig::default(),
            version: VersionConfig::default(),
            dependency: DependencyConfig::default(),
            git: GitConfig::default(),
        }
    }
}

impl Configurable for PackageToolsConfig {
    fn validate(&self) -> ConfigResult<()> {
        // Validate environments
        if self.changeset.available_environments.is_empty() {
            return Err("available_environments cannot be empty".into());
        }
        
        // Validate propagation depth
        if self.dependency.max_depth > 1000 {
            return Err("max_depth cannot exceed 1000".into());
        }
        
        Ok(())
    }
    
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.changeset.merge_with(other.changeset)?;
        self.version.merge_with(other.version)?;
        self.dependency.merge_with(other.dependency)?;
        Ok(())
    }
}

/// Changeset configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesetConfig {
    pub path: PathBuf,
    pub history_path: PathBuf,
    pub available_environments: Vec<String>,
    pub default_environments: Vec<String>,
}

/// Version configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConfig {
    pub strategy: VersioningStrategy,
    pub default_bump: VersionBump,
    pub snapshot_format: String,
}

/// Dependency configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConfig {
    pub propagation_bump: VersionBump,
    pub propagate_dependencies: bool,
    pub propagate_dev_dependencies: bool,
    pub propagate_peer_dependencies: bool,
    pub max_depth: usize,
    pub fail_on_circular: bool,
    pub skip_workspace_protocol: bool,
    pub skip_file_protocol: bool,
    pub skip_link_protocol: bool,
    pub skip_portal_protocol: bool,
}

pub struct GitConfig {
    pub merge_commit_template: String,
    pub monorepo_merge_commit_template: String,
    pub include_breaking_warning: bool,
    pub breaking_warning_template: String,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            merge_commit_template: "chore(release): {version}\n\nRelease version {version}\n\n{changelog_summary}".to_string(),
            monorepo_merge_commit_template: "chore(release): {package_name}@{version}\n\nRelease {package_name} version {version}\n\n{changelog_summary}".to_string(),
            include_breaking_warning: true,
            breaking_warning_template: "\n⚠️  BREAKING CHANGES: {breaking_changes_count}\n".to_string(),
        }
    }
}
```

#### Usage with Standard Config

Access both standard and package-specific configuration:

```rust
use sublime_standard_tools::config::{ConfigManager, StandardConfig};
use sublime_pkg_tools::config::PackageToolsConfig;

// Load standard config (filesystem, monorepo, etc)
let standard_config = ConfigManager::<StandardConfig>::builder()
    .with_defaults()
    .with_file("repo.config.toml")
    .build()?
    .load()
    .await?;

// Load package tools config (versioning, changesets, etc)
let package_config = ConfigManager::<PackageToolsConfig>::builder()
    .with_defaults()
    .with_file("repo.config.toml")
    .build()?
    .load()
    .await?;

// Use both
let fs = FileSystemManager::new_with_config(&standard_config.filesystem);
let version_resolver = VersionResolver::new(
    workspace_root,
    package_config.version,
)?;
```

**Key Points**:
- ✅ Reuses `ConfigManager` from `sublime_standard_tools`
- ✅ Extends configuration, doesn't modify standard
- ✅ Same file can contain both `[standard]` and `[package_tools]` sections
- ✅ Environment variable overrides supported
- ✅ Validation and merging built-in via `Configurable` trait

---

### Data Structures

#### Version Types

```rust
/// Semantic version.
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl Version {
    pub fn bump(&self, bump: &VersionBump) -> Version {
        match bump {
            VersionBump::Major => Version::new(self.major + 1, 0, 0),
            VersionBump::Minor => Version::new(self.major, self.minor + 1, 0),
            VersionBump::Patch => Version::new(self.major, self.minor, self.patch + 1),
            VersionBump::None => self.clone(),
        }
    }
}

/// Type of version bump.
pub enum VersionBump {
    Major,
    Minor,
    Patch,
    None,
}

/// Versioning strategy.
pub enum VersioningStrategy {
    /// Each package has independent version
    Independent,
    /// All packages share same version
    Unified,
}
```

#### Package Information

Reuse from `sublime_standard_tools` and `package-json` crate:

```rust
use package_json::PackageJson;  // External crate
use sublime_standard_tools::monorepo::WorkspacePackage;

/// Extended package info for versioning.
pub struct PackageInfo {
    /// Package metadata from package.json
    pub package_json: PackageJson,
    
    /// Workspace metadata (if in monorepo)
    pub workspace: Option<WorkspacePackage>,
    
    /// Absolute path to package directory
    pub path: PathBuf,
}

impl PackageInfo {
    /// Get package name.
    pub fn name(&self) -> &str {
        &self.package_json.name
    }
    
    /// Get current version.
    pub fn version(&self) -> Version {
        Version::parse(&self.package_json.version)
            .unwrap_or_else(|_| Version::new(0, 0, 0))
    }
    
    /// Get all dependencies (including dev and peer).
    /// Filters out workspace protocols and local references.
    pub fn all_dependencies(&self) -> Vec<(String, String, DependencyType)> {
        let mut deps = Vec::new();
        
        if let Some(deps_map) = &self.package_json.dependencies {
            for (name, version) in deps_map.iter() {
                if !Self::is_skipped_version_spec(version) {
                    deps.push((name.clone(), version.clone(), DependencyType::Regular));
                }
            }
        }
        
        if let Some(dev_deps) = &self.package_json.dev_dependencies {
            for (name, version) in dev_deps.iter() {
                if !Self::is_skipped_version_spec(version) {
                    deps.push((name.clone(), version.clone(), DependencyType::Dev));
                }
            }
        }
        
        if let Some(peer_deps) = &self.package_json.peer_dependencies {
            for (name, version) in peer_deps.iter() {
                if !Self::is_skipped_version_spec(version) {
                    deps.push((name.clone(), version.clone(), DependencyType::Peer));
                }
            }
        }
        
        deps
    }
    
    /// Check if a version spec should be skipped (workspace:*, file:, etc).
    fn is_skipped_version_spec(version_spec: &str) -> bool {
        version_spec.starts_with("workspace:")
            || version_spec.starts_with("file:")
            || version_spec.starts_with("link:")
            || version_spec.starts_with("portal:")
    }
}
```

#### Dependency Graph

```rust
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// Dependency graph for packages.
pub struct DependencyGraph {
    /// Graph structure (package dependencies)
    graph: DiGraph<String, ()>,
    
    /// Map package name to node index
    node_map: HashMap<String, NodeIndex>,
}

impl DependencyGraph {
    /// Build graph from workspace packages.
    pub fn from_packages(packages: &[PackageInfo]) -> Self {
        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();
        
        // Add nodes
        for pkg in packages {
            let idx = graph.add_node(pkg.name().to_string());
            node_map.insert(pkg.name().to_string(), idx);
        }
        
        // Add edges
        for pkg in packages {
            let from_idx = node_map[pkg.name()];
            for (dep_name, _, _) in pkg.all_dependencies() {
                if let Some(&to_idx) = node_map.get(&dep_name) {
                    graph.add_edge(from_idx, to_idx, ());
                }
            }
        }
        
        Self { graph, node_map }
    }
    
    /// Get all packages that depend on given package.
    pub fn dependents(&self, package: &str) -> Vec<String> {
        if let Some(&idx) = self.node_map.get(package) {
            self.graph
                .neighbors_directed(idx, petgraph::Direction::Incoming)
                .map(|idx| self.graph[idx].clone())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Detect circular dependencies.
    pub fn detect_cycles(&self) -> Vec<CircularDependency> {
        use petgraph::algo::tarjan_scc;
        
        let sccs = tarjan_scc(&self.graph);
        
        sccs.into_iter()
            .filter(|scc| scc.len() > 1)
            .map(|scc| CircularDependency {
                cycle: scc.iter()
                    .map(|&idx| self.graph[idx].clone())
                    .collect(),
            })
            .collect()
    }
}
```

---

### Monorepo vs Single Package Detection

The version resolver automatically detects the project structure using `sublime_standard_tools`.

#### Detection

```rust
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};

pub struct VersionResolver {
    workspace_root: PathBuf,
    strategy: VersioningStrategy,
    fs: FileSystemManager,  // from sublime_standard_tools
}

impl VersionResolver {
    pub async fn new(workspace_root: PathBuf, config: VersionConfig) -> Result<Self> {
        let fs = FileSystemManager::new();
        
        // Detect if monorepo or single package
        let detector = MonorepoDetector::new();
        let is_monorepo = detector
            .is_monorepo_root(&workspace_root)
            .await?
            .is_some();
        
        // Strategy from config
        let strategy = config.strategy;
        
        Ok(Self { workspace_root, strategy, fs })
    }
}
```

#### Package Discovery

**Monorepo**:
```rust
// Detect all packages in monorepo
let detector = MonorepoDetector::new();
let monorepo = detector.detect_monorepo(&workspace_root).await?;

let packages: Vec<PackageInfo> = monorepo.packages()
    .iter()
    .map(|wp| PackageInfo::from_workspace_package(wp, &fs))
    .collect();
```

**Single Package**:
```rust
// Single package.json at root
let package_json_path = workspace_root.join("package.json");
let content = fs.read_file_string(&package_json_path).await?;
let pkg_json: PackageJson = serde_json::from_str(&content)?;

let package = PackageInfo {
    package_json: pkg_json,
    workspace: None,
    path: workspace_root.clone(),
};
```

#### Writing package.json Files

Uses `FileSystemManager` for all writes:

```rust
impl VersionResolver {
    async fn write_package_json(
        &self,
        package: &PackageInfo,
        new_version: &Version,
        dependency_updates: &[DependencyUpdate],
    ) -> Result<PathBuf> {
        // Read current package.json
        let package_json_path = package.path.join("package.json");
        let mut pkg_json = package.package_json.clone();
        
        // Update version
        pkg_json.version = new_version.to_string();
        
        // Update dependencies (skipping workspace protocols and local refs)
        for dep_update in dependency_updates {
            // Skip if this is a workspace protocol or local reference
            if Self::is_skipped_version_spec(&dep_update.old_version_spec) {
                continue;
            }
            
            match dep_update.dependency_type {
                DependencyType::Regular => {
                    if let Some(deps) = &mut pkg_json.dependencies {
                        deps.insert(
                            dep_update.dependency_name.clone(),
                            dep_update.new_version_spec.clone(),
                        );
                    }
                }
                DependencyType::Dev => {
                    if let Some(dev_deps) = &mut pkg_json.dev_dependencies {
                        dev_deps.insert(
                            dep_update.dependency_name.clone(),
                            dep_update.new_version_spec.clone(),
                        );
                    }
                }
                DependencyType::Peer => {
                    if let Some(peer_deps) = &mut pkg_json.peer_dependencies {
                        peer_deps.insert(
                            dep_update.dependency_name.clone(),
                            dep_update.new_version_spec.clone(),
                        );
                    }
                }
            }
        }
        
        // Serialize with pretty formatting
        let json = serde_json::to_string_pretty(&pkg_json)?;
        
        // Write using FileSystemManager (handles permissions, atomic writes, etc)
        self.fs.write_file_string(&package_json_path, &json).await?;
        
        Ok(package_json_path)
    }
    
    /// Check if a version spec should be skipped.
    fn is_skipped_version_spec(version_spec: &str) -> bool {
        version_spec.starts_with("workspace:")
            || version_spec.starts_with("file:")
            || version_spec.starts_with("link:")
            || version_spec.starts_with("portal:")
    }
}
```

**Key Points**:
- Uses `AsyncFileSystem` trait from `sublime_standard_tools`
- `FileSystemManager` handles all filesystem operations robustly
- Supports both monorepo (multiple package.json) and single-package (one package.json)
- Atomic writes, proper error handling, cross-platform support
- **Skips workspace protocols and local references** (workspace:*, file:, link:, portal:)

---

## Dependency Upgrades

### Overview

The **Dependency Upgrades** module provides APIs to:
- Detect external dependencies in package.json files
- Query registries (npm, private/enterprise) for available updates
- Preview upgrades with version type classification (major, minor, patch)
- Apply upgrades with dry-run support
- Rollback changes on failure
- Automatically create changesets for applied upgrades

This module follows the same principles as the rest of the crate:
- **Library-only**: No CLI, no interactive prompts
- **Dry-run first**: Always preview before applying
- **Rollback support**: Atomic operations with automatic rollback on failure
- **Registry agnostic**: Support for npm, private, and enterprise registries
- **Changeset integration**: Automatically creates patch changesets after upgrades

---

### Core Concepts

#### External vs Internal Dependencies

- **External dependencies**: Published to registries (npm, private registries)
- **Internal dependencies**: Workspace packages (workspace:*, file:, link:, etc.)
- Only external dependencies are eligible for upgrades
- Internal dependencies are managed by version resolution and propagation

#### Update Types

Following semver conventions:
- **Major**: Breaking changes (1.0.0 → 2.0.0)
- **Minor**: New features, backward compatible (1.0.0 → 1.1.0)
- **Patch**: Bug fixes, backward compatible (1.0.0 → 1.0.1)

#### Registry Support

- **npm registry** (default): https://registry.npmjs.org
- **Private registries**: Corporate/enterprise registries
- **Scoped packages**: Support for @scope/package with custom registries
- **.npmrc support**: Read registry configuration from .npmrc files

---

### Data Structures

#### UpgradeManager

Main entry point for dependency upgrade operations.

```rust
/// Manages dependency upgrades across packages
///
/// Provides APIs to detect, preview, and apply dependency upgrades
/// from configured registries.
pub struct UpgradeManager {
    workspace_root: PathBuf,
    registry_client: RegistryClient,
    fs: FileSystemManager,
}

impl UpgradeManager {
    /// Creates a new UpgradeManager with the given configuration
    ///
    /// # Arguments
    /// * `workspace_root` - Root directory of the workspace/package
    /// * `config` - Upgrade configuration (registries, timeout, etc.)
    ///
    /// # Examples
    /// ```rust
    /// let manager = UpgradeManager::new(
    ///     PathBuf::from("/workspace"),
    ///     UpgradeConfig::default()
    /// ).await?;
    /// ```
    pub async fn new(
        workspace_root: PathBuf,
        config: UpgradeConfig,
    ) -> Result<Self, UpgradeError>;

    /// Detects all external dependencies and checks for available updates
    ///
    /// Scans all package.json files in the workspace and queries registries
    /// for newer versions. Returns a preview of available upgrades.
    ///
    /// # Arguments
    /// * `options` - Detection options (filter by type, package name, etc.)
    ///
    /// # Returns
    /// `UpgradePreview` with all available upgrades
    pub async fn detect_upgrades(
        &self,
        options: DetectionOptions,
    ) -> Result<UpgradePreview, UpgradeError>;

    /// Applies upgrades to package.json files
    ///
    /// Updates package.json files with new dependency versions.
    /// Supports dry-run mode for preview without changes.
    /// Creates automatic backup and rollback on failure.
    ///
    /// # Arguments
    /// * `selection` - Which upgrades to apply
    /// * `dry_run` - If true, preview changes without writing files
    ///
    /// # Returns
    /// `UpgradeResult` with applied changes and summary
    pub async fn apply_upgrades(
        &self,
        selection: UpgradeSelection,
        dry_run: bool,
    ) -> Result<UpgradeResult, UpgradeError>;

    /// Rolls back the last upgrade operation
    ///
    /// Restores package.json files from automatic backup.
    /// Only available if the last operation failed or was interrupted.
    ///
    /// # Returns
    /// List of restored file paths
    pub async fn rollback_last(&self) -> Result<Vec<PathBuf>, UpgradeError>;
}
```

#### UpgradePreview

Preview of available upgrades before applying.

```rust
/// Preview of available dependency upgrades
///
/// Contains all detected external dependencies with available updates,
/// classified by upgrade type (major, minor, patch).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradePreview {
    /// Timestamp when detection was performed
    pub detected_at: DateTime<Utc>,

    /// All available upgrades grouped by package
    pub packages: Vec<PackageUpgrades>,

    /// Summary statistics
    pub summary: UpgradeSummary,
}

/// Available upgrades for a single package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageUpgrades {
    /// Package name (from package.json)
    pub package_name: String,

    /// Path to package.json file
    pub package_path: PathBuf,

    /// Current version in package.json
    pub current_version: Option<String>,

    /// List of available upgrades for dependencies in this package
    pub upgrades: Vec<DependencyUpgrade>,
}

/// Details of a single dependency upgrade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyUpgrade {
    /// Dependency name
    pub name: String,

    /// Current version spec in package.json
    pub current_version: String,

    /// Latest available version from registry
    pub latest_version: String,

    /// Type of upgrade (major, minor, patch)
    pub upgrade_type: UpgradeType,

    /// Dependency type (regular, dev, peer, optional)
    pub dependency_type: DependencyType,

    /// Registry where this package is published
    pub registry_url: String,

    /// Additional version information
    pub version_info: VersionInfo,
}

/// Type of version upgrade
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpgradeType {
    /// Major version bump (breaking changes)
    Major,
    /// Minor version bump (new features)
    Minor,
    /// Patch version bump (bug fixes)
    Patch,
}

/// Additional version information from registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// All available versions
    pub available_versions: Vec<String>,

    /// Latest stable version
    pub latest_stable: String,

    /// Latest pre-release version (if any)
    pub latest_prerelease: Option<String>,

    /// Deprecation warning (if deprecated)
    pub deprecated: Option<String>,

    /// Publication date of latest version
    pub published_at: Option<DateTime<Utc>>,
}

/// Summary statistics for upgrades
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeSummary {
    /// Total number of packages scanned
    pub packages_scanned: usize,

    /// Total number of external dependencies found
    pub total_dependencies: usize,

    /// Number of dependencies with available upgrades
    pub upgrades_available: usize,

    /// Breakdown by upgrade type
    pub major_upgrades: usize,
    pub minor_upgrades: usize,
    pub patch_upgrades: usize,

    /// Deprecated dependencies found
    pub deprecated_dependencies: usize,
}
```

#### UpgradeSelection

Specifies which upgrades to apply.

```rust
/// Selection criteria for applying upgrades
///
/// Allows filtering upgrades by type, package, or specific dependencies.
#[derive(Debug, Clone, Default)]
pub struct UpgradeSelection {
    /// Apply all available upgrades
    pub all: bool,

    /// Apply only patch upgrades
    pub patch_only: bool,

    /// Apply patch and minor upgrades
    pub minor_and_patch: bool,

    /// Specific packages to upgrade (filter by package name)
    pub packages: Option<Vec<String>>,

    /// Specific dependencies to upgrade (filter by dependency name)
    pub dependencies: Option<Vec<String>>,

    /// Maximum upgrade type to apply
    pub max_upgrade_type: Option<UpgradeType>,
}

impl UpgradeSelection {
    /// Select all upgrades
    pub fn all() -> Self {
        Self { all: true, ..Default::default() }
    }

    /// Select only patch upgrades
    pub fn patch_only() -> Self {
        Self { patch_only: true, ..Default::default() }
    }

    /// Select patch and minor upgrades (exclude major)
    pub fn minor_and_patch() -> Self {
        Self { minor_and_patch: true, ..Default::default() }
    }

    /// Select upgrades for specific packages
    pub fn packages(packages: Vec<String>) -> Self {
        Self { packages: Some(packages), ..Default::default() }
    }

    /// Select specific dependencies across all packages
    pub fn dependencies(deps: Vec<String>) -> Self {
        Self { dependencies: Some(deps), ..Default::default() }
    }
}
```

#### UpgradeResult

Result of applying upgrades.

```rust
/// Result of applying dependency upgrades
///
/// Contains information about what was changed, files modified,
/// and automatic changeset creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeResult {
    /// Whether this was a dry-run
    pub dry_run: bool,

    /// Applied upgrades
    pub applied: Vec<AppliedUpgrade>,

    /// Files modified (package.json paths)
    pub modified_files: Vec<PathBuf>,

    /// Backup location (for rollback)
    pub backup_path: Option<PathBuf>,

    /// Automatically created changeset (if not dry-run)
    pub changeset_id: Option<String>,

    /// Summary statistics
    pub summary: ApplySummary,
}

/// Details of a single applied upgrade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedUpgrade {
    /// Package path
    pub package_path: PathBuf,

    /// Dependency name
    pub dependency_name: String,

    /// Dependency type
    pub dependency_type: DependencyType,

    /// Old version
    pub old_version: String,

    /// New version
    pub new_version: String,

    /// Upgrade type
    pub upgrade_type: UpgradeType,
}

/// Summary of apply operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplySummary {
    /// Total packages modified
    pub packages_modified: usize,

    /// Total dependencies upgraded
    pub dependencies_upgraded: usize,

    /// Breakdown by upgrade type
    pub major_upgrades: usize,
    pub minor_upgrades: usize,
    pub patch_upgrades: usize,

    /// Operation timestamp
    pub applied_at: DateTime<Utc>,
}
```

#### Registry Client

Handles communication with package registries.

```rust
/// Client for querying package registries
///
/// Supports npm registry, private registries, and scoped packages.
/// Respects .npmrc configuration for registry URLs and authentication.
pub struct RegistryClient {
    config: RegistryConfig,
    http_client: HttpClient,
    npmrc: Option<NpmrcConfig>,
}

impl RegistryClient {
    /// Creates a new registry client
    pub async fn new(
        workspace_root: &Path,
        config: RegistryConfig,
    ) -> Result<Self, RegistryError>;

    /// Queries package metadata from registry
    ///
    /// Returns all available versions and metadata for a package.
    pub async fn get_package_info(
        &self,
        package_name: &str,
    ) -> Result<PackageMetadata, RegistryError>;

    /// Gets the latest version for a package
    pub async fn get_latest_version(
        &self,
        package_name: &str,
    ) -> Result<String, RegistryError>;

    /// Compares two versions and determines upgrade type
    pub fn compare_versions(
        &self,
        current: &str,
        latest: &str,
    ) -> Result<UpgradeType, RegistryError>;

    /// Resolves registry URL for a package
    ///
    /// Handles scoped packages and .npmrc configuration.
    fn resolve_registry_url(&self, package_name: &str) -> String;
}

/// Package metadata from registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    /// Package name
    pub name: String,

    /// All available versions
    pub versions: Vec<String>,

    /// Latest dist-tag
    pub latest: String,

    /// Deprecation notice
    pub deprecated: Option<String>,

    /// Time metadata (publication dates)
    pub time: HashMap<String, DateTime<Utc>>,

    /// Repository information
    pub repository: Option<RepositoryInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub type_: String,
    pub url: String,
}
```

#### Configuration

Configuration for upgrade operations and registries.

```rust
/// Configuration for dependency upgrades
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeConfig {
    /// Registry configuration
    pub registry: RegistryConfig,

    /// Automatic changeset creation
    pub auto_changeset: bool,

    /// Changeset bump type for upgrades (default: Patch)
    pub changeset_bump: VersionBump,

    /// Backup configuration
    pub backup: BackupConfig,
}

impl Default for UpgradeConfig {
    fn default() -> Self {
        Self {
            registry: RegistryConfig::default(),
            auto_changeset: true,
            changeset_bump: VersionBump::Patch,
            backup: BackupConfig::default(),
        }
    }
}

/// Registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Default registry URL (default: https://registry.npmjs.org)
    pub default_registry: String,

    /// Scoped registry mappings (@scope -> registry URL)
    pub scoped_registries: HashMap<String, String>,

    /// Authentication tokens (registry URL -> token)
    pub auth_tokens: HashMap<String, String>,

    /// HTTP timeout in seconds
    pub timeout_secs: u64,

    /// Retry configuration
    pub retry_attempts: usize,
    pub retry_delay_ms: u64,

    /// Read .npmrc file for configuration
    pub read_npmrc: bool,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            default_registry: "https://registry.npmjs.org".to_string(),
            scoped_registries: HashMap::new(),
            auth_tokens: HashMap::new(),
            timeout_secs: 30,
            retry_attempts: 3,
            retry_delay_ms: 1000,
            read_npmrc: true,
        }
    }
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Enable automatic backup before applying upgrades
    pub enabled: bool,

    /// Backup directory (default: .sublime/backups)
    pub backup_dir: PathBuf,

    /// Keep backups after successful operation
    pub keep_after_success: bool,

    /// Maximum number of backups to keep
    pub max_backups: usize,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backup_dir: PathBuf::from(".sublime/backups"),
            keep_after_success: false,
            max_backups: 5,
        }
    }
}
```

---

### Configuration File Integration

Extends the existing `package_tools` configuration section.

```toml
[package_tools.upgrade]
# Enable automatic changeset creation after upgrades
auto_changeset = true

# Changeset bump type for upgrades (Major, Minor, Patch)
changeset_bump = "Patch"

[package_tools.upgrade.registry]
# Default npm registry
default_registry = "https://registry.npmjs.org"

# HTTP timeout in seconds
timeout_secs = 30

# Retry configuration
retry_attempts = 3
retry_delay_ms = 1000

# Read .npmrc for registry configuration
read_npmrc = true

# Scoped registries (optional)
[package_tools.upgrade.registry.scoped]
"@myorg" = "https://npm.myorg.com"
"@internal" = "https://registry.internal.company.com"

# Authentication tokens (optional, prefer .npmrc or env vars)
# [package_tools.upgrade.registry.auth]
# "https://npm.myorg.com" = "${NPM_TOKEN}"

[package_tools.upgrade.backup]
# Enable automatic backup before applying upgrades
enabled = true

# Backup directory
backup_dir = ".sublime/backups"

# Keep backups after successful operation
keep_after_success = false

# Maximum number of backups to keep
max_backups = 5
```

#### Configuration Structure

```rust
/// Extended PackageToolsConfig with upgrade support
pub struct PackageToolsConfig {
    pub changeset: ChangesetConfig,
    pub version: VersionConfig,
    pub dependency: DependencyConfig,
    pub upgrade: UpgradeConfig,  // NEW
}

impl Default for PackageToolsConfig {
    fn default() -> Self {
        Self {
            changeset: ChangesetConfig::default(),
            version: VersionConfig::default(),
            dependency: DependencyConfig::default(),
            upgrade: UpgradeConfig::default(),  // NEW
        }
    }
}
```

#### Environment Variable Overrides

```bash
# Registry configuration
SUBLIME_PKG_UPGRADE_REGISTRY_URL="https://registry.npmjs.org"
SUBLIME_PKG_UPGRADE_TIMEOUT_SECS=30
SUBLIME_PKG_UPGRADE_RETRY_ATTEMPTS=3

# Authentication (preferred method)
NPM_TOKEN="your-token-here"
SUBLIME_PKG_UPGRADE_AUTH_TOKEN="your-token-here"

# Backup configuration
SUBLIME_PKG_UPGRADE_BACKUP_ENABLED=true
SUBLIME_PKG_UPGRADE_BACKUP_DIR=".sublime/backups"
```

---

### Detection Options

Fine-grained control over upgrade detection.

```rust
/// Options for detecting available upgrades
#[derive(Debug, Clone, Default)]
pub struct DetectionOptions {
    /// Include dependencies
    pub include_dependencies: bool,

    /// Include devDependencies
    pub include_dev_dependencies: bool,

    /// Include peerDependencies
    pub include_peer_dependencies: bool,

    /// Include optionalDependencies
    pub include_optional_dependencies: bool,

    /// Filter by package names (detect only these packages)
    pub package_filter: Option<Vec<String>>,

    /// Filter by dependency names (detect only these dependencies)
    pub dependency_filter: Option<Vec<String>>,

    /// Include pre-release versions
    pub include_prereleases: bool,

    /// Concurrency limit for registry queries
    pub concurrency: usize,
}

impl DetectionOptions {
    /// Default options: all dependency types, no filters
    pub fn all() -> Self {
        Self {
            include_dependencies: true,
            include_dev_dependencies: true,
            include_peer_dependencies: true,
            include_optional_dependencies: true,
            concurrency: 10,
            ..Default::default()
        }
    }

    /// Only production dependencies
    pub fn production_only() -> Self {
        Self {
            include_dependencies: true,
            concurrency: 10,
            ..Default::default()
        }
    }

    /// Only dev dependencies
    pub fn dev_only() -> Self {
        Self {
            include_dev_dependencies: true,
            concurrency: 10,
            ..Default::default()
        }
    }
}
```

---

### .npmrc Support

Reading and parsing .npmrc files for registry configuration.

```rust
/// Parsed .npmrc configuration
#[derive(Debug, Clone, Default)]
pub struct NpmrcConfig {
    /// Default registry
    pub registry: Option<String>,

    /// Scoped registries
    pub scoped_registries: HashMap<String, String>,

    /// Authentication tokens
    pub auth_tokens: HashMap<String, String>,

    /// Other configuration
    pub other: HashMap<String, String>,
}

impl NpmrcConfig {
    /// Parses .npmrc file from workspace root
    ///
    /// Looks for .npmrc in the workspace root and user home directory.
    /// Merges configuration with workspace .npmrc taking precedence.
    pub async fn from_workspace(
        workspace_root: &Path,
        fs: &FileSystemManager,
    ) -> Result<Self, NpmrcError>;

    /// Resolves registry URL for a package name
    ///
    /// Returns scoped registry if package is scoped, otherwise default.
    pub fn resolve_registry(&self, package_name: &str) -> Option<&str>;

    /// Gets authentication token for a registry URL
    pub fn get_auth_token(&self, registry_url: &str) -> Option<&str>;
}
```

**Example .npmrc**:

```text
registry=https://registry.npmjs.org
@myorg:registry=https://npm.myorg.com
//npm.myorg.com/:_authToken=${NPM_TOKEN}
```

---

### Usage Examples

#### Example 1: Detect Available Upgrades

```rust
use sublime_pkg_tools::upgrade::{UpgradeManager, DetectionOptions, UpgradeConfig};

// Create manager
let manager = UpgradeManager::new(
    PathBuf::from("/workspace"),
    UpgradeConfig::default()
).await?;

// Detect all upgrades
let preview = manager.detect_upgrades(DetectionOptions::all()).await?;

println!("Available upgrades: {}", preview.summary.upgrades_available);
println!("  Major: {}", preview.summary.major_upgrades);
println!("  Minor: {}", preview.summary.minor_upgrades);
println!("  Patch: {}", preview.summary.patch_upgrades);

// Print details
for package in preview.packages {
    println!("\n{} ({})", package.package_name, package.package_path.display());
    for upgrade in package.upgrades {
        println!(
            "  {} {} -> {} ({:?})",
            upgrade.name,
            upgrade.current_version,
            upgrade.latest_version,
            upgrade.upgrade_type
        );
    }
}
```

#### Example 2: Preview Upgrades (Dry Run)

```rust
// Detect upgrades
let preview = manager.detect_upgrades(DetectionOptions::all()).await?;

// Apply in dry-run mode (no actual changes)
let result = manager.apply_upgrades(
    UpgradeSelection::all(),
    true  // dry_run = true
).await?;

println!("DRY RUN - No files modified");
println!("Would modify {} packages", result.summary.packages_modified);
println!("Would upgrade {} dependencies", result.summary.dependencies_upgraded);

for applied in result.applied {
    println!(
        "  {} in {}: {} -> {}",
        applied.dependency_name,
        applied.package_path.display(),
        applied.old_version,
        applied.new_version
    );
}
```

#### Example 3: Apply Patch Upgrades Only

```rust
// Detect upgrades
let preview = manager.detect_upgrades(DetectionOptions::all()).await?;

// Apply only patch upgrades (safe, backward compatible)
let result = manager.apply_upgrades(
    UpgradeSelection::patch_only(),
    false  // dry_run = false, actually apply
).await?;

println!("Applied {} patch upgrades", result.summary.patch_upgrades);
println!("Modified files: {}", result.modified_files.len());

// Changeset automatically created
if let Some(changeset_id) = result.changeset_id {
    println!("Created changeset: {}", changeset_id);
}
```

#### Example 4: Apply Specific Dependencies

```rust
// Upgrade only specific dependencies across all packages
let result = manager.apply_upgrades(
    UpgradeSelection::dependencies(vec![
        "lodash".to_string(),
        "axios".to_string(),
    ]),
    false
).await?;

println!("Upgraded lodash and axios in {} packages", result.summary.packages_modified);
```

#### Example 5: Rollback on Failure

```rust
// Try to apply upgrades
let result = manager.apply_upgrades(
    UpgradeSelection::all(),
    false
).await;

match result {
    Ok(res) => {
        println!("Successfully upgraded {} dependencies", res.summary.dependencies_upgraded);
    }
    Err(e) => {
        eprintln!("Upgrade failed: {}", e);
        
        // Automatic rollback
        let restored = manager.rollback_last().await?;
        println!("Rolled back {} files", restored.len());
    }
}
```

#### Example 6: Private Registry with Authentication

```rust
use sublime_pkg_tools::upgrade::{UpgradeConfig, RegistryConfig};

let mut registry_config = RegistryConfig::default();
registry_config.default_registry = "https://npm.myorg.com".to_string();

// Auth token from environment variable
let auth_token = std::env::var("NPM_TOKEN")?;
registry_config.auth_tokens.insert(
    "https://npm.myorg.com".to_string(),
    auth_token,
);

let config = UpgradeConfig {
    registry: registry_config,
    ..Default::default()
};

let manager = UpgradeManager::new(
    PathBuf::from("/workspace"),
    config
).await?;

// Now detect and apply upgrades from private registry
let preview = manager.detect_upgrades(DetectionOptions::all()).await?;
```

#### Example 7: Scoped Packages with Different Registries

```rust
let mut registry_config = RegistryConfig::default();

// Public packages from npm
registry_config.default_registry = "https://registry.npmjs.org".to_string();

// @myorg packages from private registry
registry_config.scoped_registries.insert(
    "@myorg".to_string(),
    "https://npm.myorg.com".to_string(),
);

// @internal packages from internal registry
registry_config.scoped_registries.insert(
    "@internal".to_string(),
    "https://registry.internal.company.com".to_string(),
);

let config = UpgradeConfig {
    registry: registry_config,
    ..Default::default()
};

let manager = UpgradeManager::new(
    PathBuf::from("/workspace"),
    config
).await?;
```

#### Example 8: Integration with Changeset Workflow

```rust
// 1. Detect upgrades
let preview = manager.detect_upgrades(DetectionOptions::all()).await?;

// 2. Apply patch upgrades (automatic changeset created)
let upgrade_result = manager.apply_upgrades(
    UpgradeSelection::patch_only(),
    false
).await?;

// 3. Load the automatically created changeset
let changeset_manager = ChangesetManager::new(/* ... */);
let changeset = changeset_manager.load(&upgrade_result.changeset_id.unwrap()).await?;

println!("Changeset: {}", changeset.branch);
println!("Bump: {:?}", changeset.bump);  // Will be Patch
println!("Packages: {:?}", changeset.packages);

// 4. Optionally update the changeset (change bump type, environments, etc.)
changeset_manager.update(
    &changeset.branch,
    Some(VersionBump::Minor),  // Upgrade to minor
    Some(vec!["production".to_string()]),
    None,
).await?;

// 5. Continue with normal versioning workflow
// ... (version resolution, apply versions, archive changeset)
```

---

### Automatic Changeset Creation

When `auto_changeset = true` (default), the upgrade manager automatically creates a changeset after applying upgrades.

#### Changeset Details

```rust
// After applying upgrades, a changeset is created with:
Changeset {
    branch: current_git_branch,  // e.g., "feature/upgrade-deps"
    bump: VersionBump::Patch,    // Configurable via changeset_bump
    environments: vec![],         // Empty, user can update later
    packages: affected_packages,  // All packages that were modified
    changes: vec![],              // Empty, no commits yet
    created_at: now,
    updated_at: now,
}
```

#### Workflow Integration

1. **Detect upgrades**: `detect_upgrades()` returns preview
2. **Apply upgrades**: `apply_upgrades()` modifies package.json files
3. **Auto changeset**: Changeset created with affected packages and patch bump
4. **Git commit**: User commits the changes
5. **Add commits**: Use `changeset_manager.add_commits()` to link the commit
6. **Version and release**: Continue with normal versioning workflow

---

### Error Handling

```rust
/// Errors that can occur during upgrade operations
#[derive(Debug, thiserror::Error)]
pub enum UpgradeError {
    #[error("Registry error: {0}")]
    Registry(#[from] RegistryError),

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("No backup available for rollback")]
    NoBackup,

    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Changeset creation failed: {0}")]
    ChangesetCreation(String),

    #[error("Invalid version: {0}")]
    InvalidVersion(String),
}

/// Errors related to registry communication
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Package not found in registry: {0}")]
    PackageNotFound(String),

    #[error("Authentication failed for registry: {0}")]
    AuthenticationFailed(String),

    #[error("Timeout while querying registry")]
    Timeout,

    #[error("Invalid registry response: {0}")]
    InvalidResponse(String),
}

/// Errors related to .npmrc parsing
#[derive(Debug, thiserror::Error)]
pub enum NpmrcError {
    #[error("Failed to read .npmrc: {0}")]
    ReadError(String),

    #[error("Invalid .npmrc format: {0}")]
    ParseError(String),
}
```

---

### Rollback Mechanism

The upgrade manager provides automatic rollback support through atomic operations and backup files.

#### How Rollback Works

1. **Before applying upgrades**: All package.json files are backed up to `.sublime/backups/{timestamp}/`
2. **During upgrade**: Files are modified atomically using `FileSystemManager`
3. **On success**: Backup is optionally kept or deleted (configurable)
4. **On failure**: Automatic rollback restores all files from backup
5. **Manual rollback**: User can call `rollback_last()` to restore previous state

#### Backup Structure

```text
.sublime/backups/
├── 2024-01-15T10-30-45-upgrade/
│   ├── package.json
│   └── packages/
│       ├── core/package.json
│       └── utils/package.json
├── 2024-01-14T15-20-30-upgrade/
│   └── ...
└── metadata.json
```

#### Metadata File

```json
{
  "backups": [
    {
      "id": "2024-01-15T10-30-45-upgrade",
      "created_at": "2024-01-15T10:30:45Z",
      "operation": "upgrade",
      "files": [
        "/workspace/package.json",
        "/workspace/packages/core/package.json"
      ],
      "success": true
    }
  ]
}
```

---

### Concurrency and Performance

The upgrade manager uses concurrent registry queries to improve performance.

```rust
impl UpgradeManager {
    /// Detects upgrades concurrently
    ///
    /// Uses tokio tasks to query registry for multiple packages in parallel.
    /// Concurrency limit is configurable via DetectionOptions.
    async fn detect_upgrades_concurrent(
        &self,
        dependencies: Vec<(String, String)>,  // (name, current_version)
        concurrency: usize,
    ) -> Result<Vec<DependencyUpgrade>, UpgradeError> {
        use futures::stream::{self, StreamExt};

        let results = stream::iter(dependencies)
            .map(|(name, current_version)| async move {
                self.check_single_upgrade(&name, &current_version).await
            })
            .buffer_unordered(concurrency)
            .collect::<Vec<_>>()
            .await;

        // Collect results and handle errors
        // ...
    }
}
```

**Performance Considerations**:
- Default concurrency: 10 parallel requests
- HTTP timeout: 30 seconds (configurable)
- Retry logic: 3 attempts with exponential backoff
- Caching: Registry responses cached for session duration

---

### Module Structure

```text
crates/pkg/src/upgrade/
├── mod.rs                  # Public API exports
├── manager.rs              # UpgradeManager implementation
├── registry/
│   ├── mod.rs              # Registry client exports
│   ├── client.rs           # RegistryClient implementation
│   ├── npmrc.rs            # .npmrc parsing
│   └── metadata.rs         # PackageMetadata types
├── detection.rs            # Upgrade detection logic
├── apply.rs                # Apply upgrades logic
├── backup.rs               # Backup and rollback
├── selection.rs            # UpgradeSelection types
├── types.rs                # Core data structures
├── config.rs               # Configuration types
└── error.rs                # Error types
```

---

### Dependencies

#### New External Dependencies

```toml
[dependencies]
# HTTP client for registry queries
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }

# Async runtime (already available)
tokio = { version = "1", features = ["full"] }

# Async streams
futures = "0.3"

# Semver parsing and comparison
semver = "1.0"

# HTTP retry logic
reqwest-retry = "0.3"
reqwest-middleware = "0.2"

# Error handling (already available)
thiserror = "1.0"

# Serialization (already available)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Date/time (already available)
chrono = { version = "0.4", features = ["serde"] }
```

---

### Design Principles

1. **Library First**: No CLI, no interactive prompts - pure library API
2. **Dry-run Support**: Always preview before applying changes
3. **Atomic Operations**: All changes are atomic with automatic rollback on failure
4. **Registry Agnostic**: Support for npm, private, and enterprise registries
5. **Secure by Default**: Auth tokens from environment variables or .npmrc, never hardcoded
6. **Changeset Integration**: Seamless integration with existing changeset workflow
7. **Performance**: Concurrent registry queries with configurable limits
8. **Testability**: All components are independently testable
9. **Error Recovery**: Comprehensive error handling with rollback support

---

### Security Considerations

1. **Authentication Tokens**:
   - Never log or display auth tokens
   - Read from environment variables or .npmrc
   - Support for multiple registries with different tokens

2. **HTTPS Only**:
   - All registry communication over HTTPS
   - Certificate validation enabled by default

3. **Input Validation**:
   - Validate all package names (prevent injection)
   - Validate version strings
   - Sanitize registry URLs

4. **File Operations**:
   - Use atomic writes from `FileSystemManager`
   - Proper permissions on backup directories
   - No world-readable backups

---

### Testing Strategy

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_upgrades_empty_workspace() {
        // Test detection with no packages
    }

    #[tokio::test]
    async fn test_compare_versions() {
        // Test version comparison logic
        // 1.0.0 -> 2.0.0 = Major
        // 1.0.0 -> 1.1.0 = Minor
        // 1.0.0 -> 1.0.1 = Patch
    }

    #[tokio::test]
    async fn test_upgrade_selection_filtering() {
        // Test selection filters
    }

    #[tokio::test]
    async fn test_rollback_success() {
        // Test rollback mechanism
    }

    #[tokio::test]
    async fn test_npmrc_parsing() {
        // Test .npmrc file parsing
    }
}
```

#### Integration Tests

```rust
#[tokio::test]
async fn test_full_upgrade_workflow() {
    // 1. Setup test workspace
    // 2. Detect upgrades
    // 3. Apply upgrades (dry-run)
    // 4. Apply upgrades (real)
    // 5. Verify changeset created
    // 6. Verify package.json updated
}

#[tokio::test]
async fn test_private_registry() {
    // Test with mock private registry
}

#[tokio::test]
async fn test_scoped_packages() {
    // Test @scope/package handling
}
```

#### Mock Registry

```rust
/// Mock registry server for testing
pub struct MockRegistry {
    packages: HashMap<String, PackageMetadata>,
    port: u16,
}

impl MockRegistry {
    pub async fn start() -> Self {
        // Start HTTP server with mock responses
    }

    pub fn add_package(&mut self, name: &str, versions: Vec<&str>) {
        // Add mock package data
    }

    pub fn url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }
}
```

---

### Open Questions

1. **Pre-release Versions**: How to handle pre-release versions (alpha, beta, rc)?
   - **Answer**: By default, exclude pre-releases. Option to include via `DetectionOptions::include_prereleases`

2. **Version Range Updates**: Should we update version ranges (e.g., `^1.0.0` -> `^2.0.0`)?
   - **Answer**: Yes, preserve range operator and update version

3. **Peer Dependency Conflicts**: How to handle peer dependency conflicts after upgrades?
   - **Answer**: Detect and warn, but don't block. User responsibility to resolve.

4. **Multiple Registry Failures**: What if some registries are unreachable?
   - **Answer**: Continue with available registries, report failures in summary

5. **Large Monorepos**: Performance with hundreds of packages?
   - **Answer**: Concurrent queries with configurable limit, progress reporting

---

## Changes Analysis

### Overview

The **Changes Analysis** module provides comprehensive analysis of changes across packages by integrating with `sublime_git_tools`. It answers critical questions for versioning decisions:

- **Which packages were affected?** → Returns all `PackageInfo` for changed packages
- **What files changed?** → Lists all changed files, grouped by package
- **What commits affected each package?** → Associates commits with packages (one commit can affect multiple packages)
- **What are the current and next versions?** → Shows current version and calculated next version based on changeset bump

This module follows the same principles:
- **Library-only**: Pure API, no CLI or interactive prompts
- **Git integration**: Leverages `sublime_git_tools` for all git operations
- **Monorepo support**: Automatically handles single-package or monorepo projects
- **Changeset integration**: Calculates next versions based on active changeset

---

### Core Concepts

#### Change Detection

Changes are detected by:
1. **Git diff analysis**: Compare working tree, staging area, or commit ranges
2. **File-to-package mapping**: Each changed file is mapped to its owning package
3. **Commit-to-package association**: Commits are associated with affected packages
4. **Version calculation**: Next versions calculated from changeset bump type

#### Modes of Analysis

- **Working Directory**: Uncommitted changes (working tree + staging)
- **Commit Range**: Changes between two commits (e.g., `main..feature-branch`)
- **Single Commit**: Changes in a specific commit
- **Branch Comparison**: Changes between current branch and another branch

#### Package Association

In a monorepo:
- Files under `packages/core/` → `@myorg/core` package
- Files under `packages/utils/` → `@myorg/utils` package
- Root files (package.json, tsconfig.json) → Root package (if exists)

In single-package:
- All files → Single package

#### Commit Association

A single commit can affect multiple packages:
```
commit abc123
  Modified: packages/core/src/index.ts      → @myorg/core
  Modified: packages/utils/src/helper.ts    → @myorg/utils
```

This commit is associated with both `@myorg/core` and `@myorg/utils`.

---

### Data Structures

#### ChangesAnalyzer

Main entry point for analyzing changes.

```rust
/// Analyzes changes across packages using git integration
///
/// Provides APIs to detect which packages were affected by changes,
/// what files changed, and what commits are associated with each package.
pub struct ChangesAnalyzer {
    workspace_root: PathBuf,
    git_repo: GitRepository,
    monorepo_detector: MonorepoDetector,
    fs: FileSystemManager,
}

impl ChangesAnalyzer {
    /// Creates a new ChangesAnalyzer
    ///
    /// # Arguments
    /// * `workspace_root` - Root directory of the workspace
    ///
    /// # Examples
    /// ```rust
    /// let analyzer = ChangesAnalyzer::new(PathBuf::from("/workspace")).await?;
    /// ```
    pub async fn new(workspace_root: PathBuf) -> Result<Self, ChangesError>;

    /// Analyzes changes in the working directory (uncommitted changes)
    ///
    /// Detects all uncommitted changes (working tree + staging area)
    /// and maps them to affected packages.
    ///
    /// # Returns
    /// `ChangesReport` with all affected packages and their changes
    pub async fn analyze_working_directory(&self) -> Result<ChangesReport, ChangesError>;

    /// Analyzes changes in a commit range
    ///
    /// Compares two commits/branches and detects all changes between them.
    ///
    /// # Arguments
    /// * `base` - Base commit/branch (e.g., "main")
    /// * `head` - Head commit/branch (e.g., "feature-branch" or "HEAD")
    ///
    /// # Examples
    /// ```rust
    /// // Compare current branch with main
    /// let report = analyzer.analyze_commit_range("main", "HEAD").await?;
    ///
    /// // Compare two branches
    /// let report = analyzer.analyze_commit_range("main", "feature-branch").await?;
    /// ```
    pub async fn analyze_commit_range(
        &self,
        base: &str,
        head: &str,
    ) -> Result<ChangesReport, ChangesError>;

    /// Analyzes changes in a single commit
    ///
    /// # Arguments
    /// * `commit_ref` - Commit hash, tag, or reference
    ///
    /// # Examples
    /// ```rust
    /// let report = analyzer.analyze_single_commit("abc123def").await?;
    /// ```
    pub async fn analyze_single_commit(&self, commit_ref: &str) -> Result<ChangesReport, ChangesError>;

    /// Analyzes changes from a list of commit IDs
    ///
    /// Useful for analyzing changes from a changeset's commit list.
    ///
    /// # Arguments
    /// * `commit_ids` - List of commit hashes
    ///
    /// # Examples
    /// ```rust
    /// let changeset = changeset_manager.load("feature-branch").await?;
    /// let report = analyzer.analyze_commits(&changeset.changes).await?;
    /// ```
    pub async fn analyze_commits(&self, commit_ids: &[String]) -> Result<ChangesReport, ChangesError>;

    /// Analyzes changes with next version calculation
    ///
    /// Includes next version for each package based on changeset bump.
    ///
    /// # Arguments
    /// * `base` - Base commit/branch
    /// * `head` - Head commit/branch
    /// * `changeset` - Changeset with bump type
    ///
    /// # Returns
    /// `ChangesReport` with next_version calculated for each package
    pub async fn analyze_with_versions(
        &self,
        base: &str,
        head: &str,
        changeset: &Changeset,
    ) -> Result<ChangesReport, ChangesError>;

    /// Gets package info for all packages (with optional filter)
    ///
    /// # Arguments
    /// * `filter` - Optional filter to include only specific packages
    pub async fn get_all_packages(
        &self,
        filter: Option<&[String]>,
    ) -> Result<Vec<PackageInfo>, ChangesError>;
}
```

#### ChangesReport

Complete report of all changes across packages.

```rust
/// Complete report of changes across packages
///
/// Contains all information about which packages were affected,
/// what files changed, what commits are associated, and version information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesReport {
    /// Analysis timestamp
    pub analyzed_at: DateTime<Utc>,

    /// Analysis mode (working dir, commit range, etc.)
    pub analysis_mode: AnalysisMode,

    /// Base reference (for commit range)
    pub base_ref: Option<String>,

    /// Head reference (for commit range)
    pub head_ref: Option<String>,

    /// All affected packages
    pub packages: Vec<PackageChanges>,

    /// Summary statistics
    pub summary: ChangesSummary,

    /// Whether this is a monorepo
    pub is_monorepo: bool,
}

impl ChangesReport {
    /// Gets a specific package by name
    pub fn get_package(&self, name: &str) -> Option<&PackageChanges>;

    /// Gets all packages with changes
    pub fn packages_with_changes(&self) -> Vec<&PackageChanges>;

    /// Gets all packages without changes
    pub fn packages_without_changes(&self) -> Vec<&PackageChanges>;

    /// Filters packages by change type
    pub fn filter_by_change_type(&self, change_type: FileChangeType) -> Vec<&PackageChanges>;
}

/// Analysis mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisMode {
    /// Uncommitted changes (working dir + staging)
    WorkingDirectory,
    /// Changes between two commits/branches
    CommitRange,
    /// Changes in a single commit
    SingleCommit,
    /// Changes from a list of commits
    CommitList,
}
```

#### PackageChanges

Changes for a single package.

```rust
/// Changes for a single package
///
/// Contains all information about changes affecting this package,
/// including files, commits, and version information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageChanges {
    /// Package information (name, path, current version, dependencies)
    pub package_info: PackageInfo,

    /// Current version from package.json
    pub current_version: Option<Version>,

    /// Next version (calculated from changeset bump)
    pub next_version: Option<Version>,

    /// Bump type applied (if next_version is calculated)
    pub bump_type: Option<VersionBump>,

    /// All files changed in this package
    pub files: Vec<FileChange>,

    /// All commits affecting this package
    pub commits: Vec<CommitInfo>,

    /// Whether this package has changes
    pub has_changes: bool,

    /// Change statistics for this package
    pub stats: PackageChangeStats,
}

impl PackageChanges {
    /// Gets files by change type
    pub fn files_by_type(&self, change_type: FileChangeType) -> Vec<&FileChange>;

    /// Checks if package.json was modified
    pub fn package_json_modified(&self) -> bool;

    /// Groups files by directory
    pub fn files_by_directory(&self) -> HashMap<PathBuf, Vec<&FileChange>>;
}
```

#### FileChange

Details of a single file change.

```rust
/// Details of a single file change
///
/// Contains information about what happened to a file (added, modified, deleted)
/// and basic statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// File path relative to workspace root
    pub path: PathBuf,

    /// File path relative to package root
    pub package_relative_path: PathBuf,

    /// Type of change
    pub change_type: FileChangeType,

    /// Lines added (if available)
    pub lines_added: Option<usize>,

    /// Lines deleted (if available)
    pub lines_deleted: Option<usize>,

    /// Commits that modified this file
    pub commits: Vec<String>,  // Commit hashes
}

/// Type of file change
///
/// Maps directly to git status indicators (A, M, D, R, C).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeType {
    /// File was added
    Added,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File was renamed
    Renamed,
    /// File was copied
    Copied,
}
```

#### CommitInfo

Information about a commit affecting packages.

```rust
/// Information about a commit
///
/// Contains commit metadata and which packages it affects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    /// Commit hash
    pub hash: String,

    /// Short hash (first 7 characters)
    pub short_hash: String,

    /// Commit author
    pub author: String,

    /// Commit author email
    pub author_email: String,

    /// Commit date
    pub date: DateTime<Utc>,

    /// Commit message (first line)
    pub message: String,

    /// Full commit message
    pub full_message: String,

    /// Packages affected by this commit
    pub affected_packages: Vec<String>,

    /// Files changed in this commit
    pub files_changed: usize,

    /// Lines added in this commit
    pub lines_added: usize,

    /// Lines deleted in this commit
    pub lines_deleted: usize,
}

impl CommitInfo {
    /// Creates from sublime_git_tools::Commit
    pub fn from_git_commit(
        commit: &Commit,
        affected_packages: Vec<String>,
    ) -> Self;

    /// Checks if this commit is a merge commit
    pub fn is_merge_commit(&self) -> bool;

    /// Checks if this commit affects a specific package
    pub fn affects_package(&self, package_name: &str) -> bool;
}
```

#### Statistics

```rust
/// Summary statistics for changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesSummary {
    /// Total packages analyzed
    pub total_packages: usize,

    /// Packages with changes
    pub packages_with_changes: usize,

    /// Packages without changes
    pub packages_without_changes: usize,

    /// Total files changed
    pub total_files_changed: usize,

    /// Total commits analyzed
    pub total_commits: usize,

    /// Total lines added
    pub total_lines_added: usize,

    /// Total lines deleted
    pub total_lines_deleted: usize,
}

/// Statistics for a single package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageChangeStats {
    /// Total files changed
    pub files_changed: usize,

    /// Files added
    pub files_added: usize,

    /// Files modified
    pub files_modified: usize,

    /// Files deleted
    pub files_deleted: usize,

    /// Total commits affecting this package
    pub commits: usize,

    /// Lines added
    pub lines_added: usize,

    /// Lines deleted
    pub lines_deleted: usize,
}
```

---

### Integration with sublime_git_tools

The changes analyzer leverages git tools for all git operations.

```rust
use sublime_git_tools::{
    repository::{GitRepository, RepositoryTrait},
    commit::{Commit, CommitTrait},
    diff::{DiffEntry, DiffOptions, DiffTrait},
    status::{StatusEntry, StatusTrait},
};

impl ChangesAnalyzer {
    /// Analyzes working directory using git status
    async fn analyze_working_directory_impl(&self) -> Result<ChangesReport, ChangesError> {
        // Get git status
        let status = self.git_repo.status().await?;
        
        // Get all changed files
        let changed_files: Vec<StatusEntry> = status.entries();
        
        // Map files to packages
        let packages = self.map_files_to_packages(&changed_files).await?;
        
        // Build report
        Ok(ChangesReport {
            analyzed_at: Utc::now(),
            analysis_mode: AnalysisMode::WorkingDirectory,
            packages,
            // ...
        })
    }

    /// Analyzes commit range using git diff
    async fn analyze_commit_range_impl(
        &self,
        base: &str,
        head: &str,
    ) -> Result<ChangesReport, ChangesError> {
        // Get commits in range
        let commits = self.git_repo.log_range(base, head).await?;
        
        // Get diff between base and head
        let diff_options = DiffOptions::default();
        let diff = self.git_repo.diff_range(base, head, diff_options).await?;
        
        // Get all changed files
        let changed_files: Vec<DiffEntry> = diff.entries();
        
        // Map files to packages
        let packages = self.map_diff_to_packages(&changed_files).await?;
        
        // Associate commits with packages
        let packages = self.associate_commits(&packages, &commits).await?;
        
        // Build report
        Ok(ChangesReport {
            analyzed_at: Utc::now(),
            analysis_mode: AnalysisMode::CommitRange,
            base_ref: Some(base.to_string()),
            head_ref: Some(head.to_string()),
            packages,
            // ...
        })
    }

    /// Maps files to their owning packages
    async fn map_files_to_packages(
        &self,
        files: &[PathBuf],
    ) -> Result<HashMap<String, Vec<PathBuf>>, ChangesError> {
        // Detect monorepo
        let monorepo = self.monorepo_detector.detect_monorepo(&self.workspace_root).await?;
        
        if let Some(monorepo) = monorepo {
            // Monorepo: map each file to its package
            let mut package_files: HashMap<String, Vec<PathBuf>> = HashMap::new();
            
            for file in files {
                // Find which package owns this file
                if let Some(package) = self.find_owning_package(&monorepo, file).await? {
                    package_files
                        .entry(package.name.clone())
                        .or_default()
                        .push(file.clone());
                }
            }
            
            Ok(package_files)
        } else {
            // Single package: all files belong to root package
            let pkg_json_path = self.workspace_root.join("package.json");
            let pkg_json = self.read_package_json(&pkg_json_path).await?;
            let package_name = pkg_json.name.unwrap_or_else(|| "root".to_string());
            
            let mut package_files = HashMap::new();
            package_files.insert(package_name, files.to_vec());
            
            Ok(package_files)
        }
    }
}
```

---

### Version Calculation

Next versions are calculated based on changeset bump type.

```rust
impl ChangesAnalyzer {
    /// Calculates next version for a package
    fn calculate_next_version(
        &self,
        current_version: &Version,
        bump: VersionBump,
    ) -> Version {
        current_version.bump(bump)
    }

    /// Adds version information to package changes
    fn add_version_info(
        &self,
        package_changes: &mut PackageChanges,
        changeset: &Changeset,
    ) -> Result<(), ChangesError> {
        if let Some(current) = &package_changes.current_version {
            // Calculate next version based on changeset bump
            let next = self.calculate_next_version(current, changeset.bump);
            
            package_changes.next_version = Some(next);
            package_changes.bump_type = Some(changeset.bump);
        }
        
        Ok(())
    }
}
```

---

### Usage Examples

#### Example 1: Analyze Working Directory

```rust
use sublime_pkg_tools::changes::{ChangesAnalyzer, AnalysisMode};

// Create analyzer
let analyzer = ChangesAnalyzer::new(PathBuf::from("/workspace")).await?;

// Analyze uncommitted changes
let report = analyzer.analyze_working_directory().await?;

println!("Analysis Mode: {:?}", report.analysis_mode);
println!("Monorepo: {}", report.is_monorepo);
println!("Packages with changes: {}", report.summary.packages_with_changes);
println!("Total files changed: {}", report.summary.total_files_changed);

// List affected packages
for pkg_changes in report.packages_with_changes() {
    println!("\n{} ({})", 
        pkg_changes.package_info.name(),
        pkg_changes.package_info.path.display()
    );
    println!("  Files changed: {}", pkg_changes.stats.files_changed);
    println!("  Lines added: {}", pkg_changes.stats.lines_added);
    println!("  Lines deleted: {}", pkg_changes.stats.lines_deleted);
}
```

#### Example 2: Analyze Commit Range with Versions

```rust
// Load changeset
let changeset_manager = ChangesetManager::new(/* ... */);
let changeset = changeset_manager.load("feature-branch").await?;

// Analyze changes with version calculation
let report = analyzer.analyze_with_versions(
    "main",
    "HEAD",
    &changeset,
).await?;

// Show current and next versions
for pkg_changes in report.packages_with_changes() {
    println!("\n{}", pkg_changes.package_info.name());
    
    if let Some(current) = &pkg_changes.current_version {
        println!("  Current: {}", current);
    }
    
    if let Some(next) = &pkg_changes.next_version {
        println!("  Next:    {} ({:?})", next, pkg_changes.bump_type.unwrap());
    }
    
    println!("  Files changed: {}", pkg_changes.files.len());
    for file in &pkg_changes.files {
        println!("    {:?} {}", file.change_type, file.package_relative_path.display());
    }
}
```

#### Example 3: List Files by Package

```rust
let report = analyzer.analyze_commit_range("main", "feature-branch").await?;

for pkg_changes in report.packages_with_changes() {
    println!("\n{}", pkg_changes.package_info.name());
    
    // Group files by directory
    let by_dir = pkg_changes.files_by_directory();
    for (dir, files) in by_dir {
        println!("  {}/", dir.display());
        for file in files {
            println!("    {:?} {} (+{} -{})",
                file.change_type,
                file.package_relative_path.file_name().unwrap().to_string_lossy(),
                file.lines_added.unwrap_or(0),
                file.lines_deleted.unwrap_or(0),
            );
        }
    }
}
```

#### Example 4: Show Commits per Package

```rust
let report = analyzer.analyze_commit_range("main", "feature-branch").await?;

for pkg_changes in report.packages_with_changes() {
    println!("\n{} - {} commits", 
        pkg_changes.package_info.name(),
        pkg_changes.commits.len()
    );
    
    for commit in &pkg_changes.commits {
        println!("  {} {} (by {})",
            commit.short_hash,
            commit.message,
            commit.author,
        );
        
        // Show which other packages this commit affects
        if commit.affected_packages.len() > 1 {
            let other_packages: Vec<_> = commit.affected_packages.iter()
                .filter(|p| *p != pkg_changes.package_info.name())
                .collect();
            if !other_packages.is_empty() {
                println!("    Also affects: {}", other_packages.join(", "));
            }
        }
    }
}
```

#### Example 5: Filter by Change Type

```rust
let report = analyzer.analyze_working_directory().await?;

for pkg_changes in report.packages_with_changes() {
    println!("\n{}", pkg_changes.package_info.name());
    
    // Show added files
    let added_files = pkg_changes.files_by_type(FileChangeType::Added);
    if !added_files.is_empty() {
        println!("  Added files ({}):", added_files.len());
        for file in added_files {
            println!("    {}", file.package_relative_path.display());
        }
    }
    
    // Show modified files
    let modified_files = pkg_changes.files_by_type(FileChangeType::Modified);
    if !modified_files.is_empty() {
        println!("  Modified files ({}):", modified_files.len());
        for file in modified_files {
            println!("    {}", file.package_relative_path.display());
        }
    }
    
    // Check if package.json was modified
    if pkg_changes.package_json_modified() {
        println!("  ⚠️  package.json was modified");
    }
}
```

#### Example 6: Analyze Changeset Commits

```rust
// Load changeset
let changeset = changeset_manager.load("feature-branch").await?;

// Analyze the specific commits in the changeset
let report = analyzer.analyze_commits(&changeset.changes).await?;

println!("Analyzing {} commits from changeset", changeset.changes.len());

for pkg_changes in report.packages_with_changes() {
    println!("\n{}", pkg_changes.package_info.name());
    println!("  Commits: {}", pkg_changes.commits.len());
    println!("  Files: {}", pkg_changes.files.len());
    
    // Show commit messages
    for commit in &pkg_changes.commits {
        println!("    {} {}", commit.short_hash, commit.message);
    }
}
```

#### Example 7: Complete Workflow with All Information

```rust
// 1. Load changeset
let changeset = changeset_manager.load("feature-branch").await?;

// 2. Analyze changes with versions
let report = analyzer.analyze_with_versions(
    "main",
    "HEAD",
    &changeset,
).await?;

// 3. Print complete report
println!("=== CHANGES REPORT ===");
println!("Branch: {}", changeset.branch);
println!("Bump: {:?}", changeset.bump);
println!("Monorepo: {}", report.is_monorepo);
println!("\n=== SUMMARY ===");
println!("Packages analyzed: {}", report.summary.total_packages);
println!("Packages with changes: {}", report.summary.packages_with_changes);
println!("Total files changed: {}", report.summary.total_files_changed);
println!("Total commits: {}", report.summary.total_commits);
println!("Lines added: {}", report.summary.total_lines_added);
println!("Lines deleted: {}", report.summary.total_lines_deleted);

println!("\n=== PACKAGE DETAILS ===");
for pkg_changes in report.packages_with_changes() {
    println!("\n📦 {}", pkg_changes.package_info.name());
    println!("   Path: {}", pkg_changes.package_info.path.display());
    
    // Version info
    if let Some(current) = &pkg_changes.current_version {
        println!("   Version: {} → {}", 
            current, 
            pkg_changes.next_version.as_ref().unwrap()
        );
    }
    
    // Stats
    println!("   Stats:");
    println!("     Files: {} (+{} ~{} -{})",
        pkg_changes.stats.files_changed,
        pkg_changes.stats.files_added,
        pkg_changes.stats.files_modified,
        pkg_changes.stats.files_deleted,
    );
    println!("     Lines: +{} -{}",
        pkg_changes.stats.lines_added,
        pkg_changes.stats.lines_deleted,
    );
    println!("     Commits: {}", pkg_changes.commits.len());
    
    // Commits
    println!("   Commits:");
    for commit in &pkg_changes.commits {
        println!("     {} {} (by {}, {})",
            commit.short_hash,
            commit.message,
            commit.author,
            commit.date.format("%Y-%m-%d"),
        );
    }
    
    // Files
    println!("   Files:");
    for file in &pkg_changes.files {
        println!("     {:?} {} (+{} -{})",
            file.change_type,
            file.package_relative_path.display(),
            file.lines_added.unwrap_or(0),
            file.lines_deleted.unwrap_or(0),
        );
    }
}
```

---

### Integration with Changeset Workflow

The changes analyzer integrates seamlessly with the changeset workflow.

```rust
// Typical workflow:

// 1. Create or load changeset
let changeset = changeset_manager.create("feature-branch", VersionBump::Minor).await?;

// 2. Add commits from git
changeset_manager.add_commits_from_git(&changeset.branch, "main..HEAD").await?;

// 3. Analyze changes
let report = analyzer.analyze_with_versions("main", "HEAD", &changeset).await?;

// 4. Verify affected packages match changeset
for pkg_changes in report.packages_with_changes() {
    let pkg_name = pkg_changes.package_info.name();
    if !changeset.packages.contains(pkg_name) {
        println!("Warning: {} has changes but not in changeset", pkg_name);
    }
}

// 5. Show version preview
println!("\nVersion Preview:");
for pkg_changes in report.packages_with_changes() {
    if let (Some(current), Some(next)) = (&pkg_changes.current_version, &pkg_changes.next_version) {
        println!("{}: {} → {}", 
            pkg_changes.package_info.name(),
            current,
            next
        );
    }
}

// 6. Apply versions (dry-run first)
let version_resolver = VersionResolver::new(&workspace_root, config).await?;
let apply_result = version_resolver.apply_versions(&changeset, true).await?;

// 7. Apply versions (real)
let apply_result = version_resolver.apply_versions(&changeset, false).await?;

// 8. Archive changeset
changeset_manager.archive(&changeset.branch, release_info).await?;
```

---

### Configuration

Changes analysis configuration (optional, uses defaults if not specified).

```toml
[package_tools.changes]
# Include merge commits in analysis
include_merge_commits = true

# File patterns to exclude from analysis (gitignore-style)
exclude_patterns = [
    "*.lock",
    "node_modules/**",
    "dist/**",
    "build/**",
    ".next/**",
]
```

```rust
/// Configuration for changes analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesConfig {
    /// Include merge commits
    pub include_merge_commits: bool,

    /// File patterns to exclude (gitignore-style)
    pub exclude_patterns: Vec<String>,
}

impl Default for ChangesConfig {
    fn default() -> Self {
        Self {
            include_merge_commits: true,
            exclude_patterns: vec![
                "*.lock".to_string(),
                "node_modules/**".to_string(),
                "dist/**".to_string(),
                "build/**".to_string(),
            ],
        }
    }
}
```

---

### Error Handling

```rust
/// Errors that can occur during changes analysis
#[derive(Debug, thiserror::Error)]
pub enum ChangesError {
    #[error("Git error: {0}")]
    Git(#[from] GitError),

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Invalid commit reference: {0}")]
    InvalidCommitRef(String),

    #[error("No packages found")]
    NoPackagesFound,

    #[error("Failed to parse package.json: {0}")]
    PackageJsonParse(String),

    #[error("Monorepo detection failed: {0}")]
    MonorepoDetection(String),
}
```

---

### Module Structure

```text
crates/pkg/src/changes/
├── mod.rs                  # Public API exports
├── analyzer.rs             # ChangesAnalyzer implementation
├── report.rs               # ChangesReport and related types
├── package_changes.rs      # PackageChanges implementation
├── file_change.rs          # FileChange types
├── commit_info.rs          # CommitInfo types
├── mapping.rs              # File-to-package mapping logic
├── stats.rs                # Statistics calculation
├── version_calc.rs         # Version calculation logic
├── config.rs               # Configuration types
└── error.rs                # Error types
```

---

### Dependencies

No new external dependencies required. All functionality leverages existing crates:

```toml
[dependencies]
# Git operations
sublime_git_tools = { path = "../git" }

# Filesystem and monorepo detection
sublime_standard_tools = { path = "../standard" }

# Already available
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
```

---

### Design Principles

1. **Git-first**: All change detection through git, no manual file scanning
2. **Package-centric**: Everything organized by package
3. **Commit association**: Track which commits affect which packages
4. **Version preview**: Show current and next versions together
5. **Statistics**: Provide actionable metrics (lines changed, file counts, change types)
6. **Flexible queries**: Support multiple analysis modes (working dir, ranges, commits)
7. **Monorepo-aware**: Handle multi-package and single-package seamlessly
8. **Simple data**: FileChangeType from git (Added, Modified, Deleted, Renamed, Copied)
9. **Testability**: All components independently testable with mock git repos

---

### Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_single_package() {
        // Test changes analysis in single-package project
    }

    #[tokio::test]
    async fn test_analyze_monorepo() {
        // Test changes analysis in monorepo
    }

    #[tokio::test]
    async fn test_file_to_package_mapping() {
        // Test that files are correctly mapped to packages
    }

    #[tokio::test]
    async fn test_commit_association() {
        // Test that commits are associated with correct packages
    }

    #[tokio::test]
    async fn test_multi_package_commit() {
        // Test a commit that affects multiple packages
    }

    #[tokio::test]
    async fn test_version_calculation() {
        // Test next version calculation
    }

    #[tokio::test]
    async fn test_working_directory_analysis() {
        // Test analysis of uncommitted changes
    }

    #[tokio::test]
    async fn test_commit_range_analysis() {
        // Test analysis of commit range
    }
}
```

---

## Changelog Generation

### Overview

The **Changelog Generation** module provides comprehensive changelog management for both single-package and monorepo projects. It supports:

- **Conventional Commits** parsing and grouping
- **Custom formats** via configurable templates
- **Single package** and **monorepo** changelogs
- **Version-based generation** using git diff between versions
- **Incremental updates** to existing changelogs
- **Links** to commits, issues, and pull requests

This module follows the same principles:
- **Library-only**: Pure API, no CLI
- **Git integration**: Uses `sublime_git_tools` for version detection and commit history
- **Filesystem**: Uses `sublime_standard_tools` for file operations
- **Configurable**: Support for multiple formats and templates

---

### Core Concepts

#### Changelog Detection

Changes between versions are detected by:
1. **Git tag comparison**: Compare current version with previous version tag (e.g., `v1.0.0..v1.1.0`)
2. **Commit parsing**: Parse commit messages (conventional or custom)
3. **Section grouping**: Group commits by type (Features, Fixes, Breaking Changes, etc.)
4. **Package filtering**: In monorepo, filter commits by affected package

#### Changelog Formats

- **Keep a Changelog**: Standard format (https://keepachangelog.com)
- **Conventional Commits**: Automatic grouping by type (feat, fix, breaking, etc.)
- **Custom**: User-defined templates and sections

#### Monorepo Support

- **Per-package changelogs**: Each package has its own `CHANGELOG.md`
- **Root changelog**: Single changelog at workspace root
- **Configurable**: Choose per-package, root, or both

#### Conventional Commits

Parses commit messages following the conventional commits specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`

**Breaking changes**: `BREAKING CHANGE:` in footer or `!` after type/scope

---

### Data Structures

#### ChangelogGenerator

Main entry point for changelog generation.

```rust
/// Generates and manages changelogs for packages
///
/// Supports conventional commits, custom formats, and both
/// single-package and monorepo projects.
pub struct ChangelogGenerator {
    workspace_root: PathBuf,
    git_repo: GitRepository,
    fs: FileSystemManager,
    config: ChangelogConfig,
}

impl ChangelogGenerator {
    /// Creates a new ChangelogGenerator
    ///
    /// # Arguments
    /// * `workspace_root` - Root directory of the workspace
    /// * `config` - Changelog configuration
    ///
    /// # Examples
    /// ```rust
    /// let generator = ChangelogGenerator::new(
    ///     PathBuf::from("/workspace"),
    ///     ChangelogConfig::default()
    /// ).await?;
    /// ```
    pub async fn new(
        workspace_root: PathBuf,
        config: ChangelogConfig,
    ) -> Result<Self, ChangelogError>;

    /// Generates changelog for a specific version
    ///
    /// Compares current version with previous version tag and generates
    /// changelog entries from the commits in between.
    ///
    /// # Arguments
    /// * `package_name` - Package name (None for root changelog)
    /// * `version` - Version to generate changelog for
    /// * `previous_version` - Previous version (auto-detected if None)
    ///
    /// # Examples
    /// ```rust
    /// // Generate for specific package
    /// let changelog = generator.generate_for_version(
    ///     Some("@myorg/core"),
    ///     "1.1.0",
    ///     Some("1.0.0")
    /// ).await?;
    /// ```
    pub async fn generate_for_version(
        &self,
        package_name: Option<&str>,
        version: &str,
        previous_version: Option<&str>,
    ) -> Result<Changelog, ChangelogError>;

    /// Generates changelog from changeset
    ///
    /// Uses changeset information to determine which packages changed
    /// and generates changelogs accordingly.
    ///
    /// # Arguments
    /// * `changeset` - Changeset with version and commit information
    /// * `version_resolution` - Resolved versions for packages
    ///
    /// # Examples
    /// ```rust
    /// let changeset = changeset_manager.load("feature-branch").await?;
    /// let resolution = version_resolver.resolve_versions(&changeset).await?;
    /// 
    /// let changelogs = generator.generate_from_changeset(
    ///     &changeset,
    ///     &resolution
    /// ).await?;
    /// ```
    pub async fn generate_from_changeset(
        &self,
        changeset: &Changeset,
        version_resolution: &VersionResolution,
    ) -> Result<Vec<GeneratedChangelog>, ChangelogError>;

    /// Updates existing changelog with new version
    ///
    /// Reads existing CHANGELOG.md, adds new version section at the top.
    ///
    /// # Arguments
    /// * `package_path` - Path to package directory
    /// * `changelog` - New changelog to prepend
    /// * `dry_run` - If true, return content without writing
    ///
    /// # Returns
    /// Updated changelog content
    pub async fn update_changelog(
        &self,
        package_path: &Path,
        changelog: &Changelog,
        dry_run: bool,
    ) -> Result<String, ChangelogError>;

    /// Creates new changelog file
    ///
    /// Creates CHANGELOG.md with initial version.
    ///
    /// # Arguments
    /// * `package_path` - Path to package directory
    /// * `changelog` - Initial changelog
    pub async fn create_changelog(
        &self,
        package_path: &Path,
        changelog: &Changelog,
    ) -> Result<(), ChangelogError>;

    /// Parses existing changelog
    ///
    /// Reads and parses CHANGELOG.md file.
    pub async fn parse_existing(
        &self,
        changelog_path: &Path,
    ) -> Result<ParsedChangelog, ChangelogError>;

    /// Detects previous version from git tags
    ///
    /// # Arguments
    /// * `package_name` - Package name (None for root)
    /// * `current_version` - Current version
    ///
    /// # Returns
    /// Previous version tag or None if not found
    pub async fn detect_previous_version(
        &self,
        package_name: Option<&str>,
        current_version: &str,
    ) -> Result<Option<String>, ChangelogError>;
}
```

#### Changelog

Represents a changelog for a specific version.

```rust
/// Changelog for a specific version
///
/// Contains all entries grouped by section (Features, Fixes, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changelog {
    /// Package name (None for root changelog)
    pub package_name: Option<String>,

    /// Version this changelog is for
    pub version: String,

    /// Previous version (for comparison)
    pub previous_version: Option<String>,

    /// Release date
    pub date: DateTime<Utc>,

    /// Changelog sections (Features, Fixes, Breaking Changes, etc.)
    pub sections: Vec<ChangelogSection>,

    /// Metadata
    pub metadata: ChangelogMetadata,
}

impl Changelog {
    /// Renders changelog to markdown string
    pub fn to_markdown(&self, config: &ChangelogConfig) -> String;

    /// Gets all breaking changes
    pub fn breaking_changes(&self) -> Vec<&ChangelogEntry>;

    /// Checks if changelog has any entries
    pub fn is_empty(&self) -> bool;

    /// Gets entry count
    pub fn entry_count(&self) -> usize;
}

/// Changelog metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogMetadata {
    /// Git tag for this version
    pub tag: Option<String>,

    /// Commit range (e.g., "v1.0.0..v1.1.0")
    pub commit_range: Option<String>,

    /// Total commits in this version
    pub total_commits: usize,

    /// Repository URL (for links)
    pub repository_url: Option<String>,

    /// Bump type applied
    pub bump_type: Option<VersionBump>,
}
```

#### ChangelogSection

A section within a changelog (e.g., Features, Fixes).

```rust
/// Section within a changelog
///
/// Groups related entries together (e.g., all Features).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogSection {
    /// Section title (e.g., "Features", "Bug Fixes")
    pub title: String,

    /// Section type
    pub section_type: SectionType,

    /// Entries in this section
    pub entries: Vec<ChangelogEntry>,
}

impl ChangelogSection {
    /// Renders section to markdown
    pub fn to_markdown(&self, config: &ChangelogConfig) -> String;

    /// Checks if section is empty
    pub fn is_empty(&self) -> bool;
}

/// Type of changelog section
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionType {
    /// Breaking changes (highest priority)
    Breaking,
    /// New features
    Features,
    /// Bug fixes
    Fixes,
    /// Performance improvements
    Performance,
    /// Deprecations
    Deprecations,
    /// Documentation changes
    Documentation,
    /// Code refactoring
    Refactoring,
    /// Build system changes
    Build,
    /// CI/CD changes
    CI,
    /// Test changes
    Tests,
    /// Other changes
    Other,
}
```

#### ChangelogEntry

A single entry in the changelog.

```rust
/// Single entry in a changelog
///
/// Represents a commit or group of commits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    /// Entry description
    pub description: String,

    /// Commit hash
    pub commit_hash: String,

    /// Short commit hash (first 7 characters)
    pub short_hash: String,

    /// Commit type (from conventional commits)
    pub commit_type: Option<String>,

    /// Scope (from conventional commits)
    pub scope: Option<String>,

    /// Is this a breaking change?
    pub breaking: bool,

    /// Author name
    pub author: String,

    /// Related issues/PRs (e.g., ["#123", "#456"])
    pub references: Vec<String>,

    /// Commit date
    pub date: DateTime<Utc>,
}

impl ChangelogEntry {
    /// Renders entry to markdown
    pub fn to_markdown(&self, config: &ChangelogConfig) -> String;

    /// Generates commit link
    pub fn commit_link(&self, base_url: &str) -> String;

    /// Generates issue links
    pub fn issue_links(&self, base_url: &str) -> Vec<String>;
}
```

#### ConventionalCommit

Parsed conventional commit.

```rust
/// Parsed conventional commit
///
/// Follows the conventional commits specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConventionalCommit {
    /// Commit type (feat, fix, docs, etc.)
    pub commit_type: String,

    /// Optional scope
    pub scope: Option<String>,

    /// Breaking change indicator
    pub breaking: bool,

    /// Description (subject line)
    pub description: String,

    /// Optional body
    pub body: Option<String>,

    /// Footers (e.g., BREAKING CHANGE, Refs)
    pub footers: Vec<CommitFooter>,
}

impl ConventionalCommit {
    /// Parses a commit message
    ///
    /// # Examples
    /// ```rust
    /// let commit = ConventionalCommit::parse("feat(core)!: add new API")?;
    /// assert_eq!(commit.commit_type, "feat");
    /// assert_eq!(commit.scope, Some("core".to_string()));
    /// assert!(commit.breaking);
    /// ```
    pub fn parse(message: &str) -> Result<Self, ParseError>;

    /// Checks if this is a breaking change
    pub fn is_breaking(&self) -> bool;

    /// Gets section type for this commit
    pub fn section_type(&self) -> SectionType;

    /// Extracts issue references (#123, closes #456)
    pub fn extract_references(&self) -> Vec<String>;
}

/// Commit footer (key-value pair)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitFooter {
    pub key: String,
    pub value: String,
}
```

#### GeneratedChangelog

Result of generating a changelog.

```rust
/// Result of changelog generation
///
/// Contains the generated changelog and metadata about the operation.
#[derive(Debug, Clone)]
pub struct GeneratedChangelog {
    /// Package name (None for root)
    pub package_name: Option<String>,

    /// Package path
    pub package_path: PathBuf,

    /// Generated changelog
    pub changelog: Changelog,

    /// Rendered markdown content
    pub content: String,

    /// Whether changelog file already exists
    pub existing: bool,

    /// Path to changelog file
    pub changelog_path: PathBuf,
}

impl GeneratedChangelog {
    /// Writes changelog to file
    pub async fn write(&self, fs: &FileSystemManager) -> Result<(), ChangelogError>;

    /// Returns updated changelog content (prepends to existing)
    pub async fn merge_with_existing(
        &self,
        fs: &FileSystemManager,
    ) -> Result<String, ChangelogError>;

    /// Generates merge commit message from template
    ///
    /// Uses configured template and substitutes variables.
    ///
    /// # Arguments
    /// * `git_config` - Git configuration with templates
    /// * `previous_version` - Previous version (optional)
    ///
    /// # Examples
    /// ```rust
    /// let message = generated.generate_merge_commit_message(
    ///     &git_config,
    ///     Some("1.0.0")
    /// )?;
    /// ```
    pub fn generate_merge_commit_message(
        &self,
        git_config: &GitConfig,
        previous_version: Option<&str>,
    ) -> Result<String, ChangelogError>;
}
```

#### ParsedChangelog

Parsed existing changelog.

```rust
/// Parsed existing changelog
///
/// Result of parsing an existing CHANGELOG.md file.
#[derive(Debug, Clone)]
pub struct ParsedChangelog {
    /// Path to changelog file
    pub path: PathBuf,

    /// All versions found in changelog
    pub versions: Vec<ParsedVersion>,

    /// Raw content
    pub raw_content: String,
}

/// Single version section in parsed changelog
#[derive(Debug, Clone)]
pub struct ParsedVersion {
    /// Version string
    pub version: String,

    /// Release date (if found)
    pub date: Option<DateTime<Utc>>,

    /// Raw markdown content for this version
    pub content: String,

    /// Line number where this version starts
    pub line_start: usize,

    /// Line number where this version ends
    pub line_end: usize,
}
```

---

### Configuration

Changelog generation is highly configurable.

```toml
[package_tools.changelog]
# Enable changelog generation
enabled = true

# Changelog format: "keep-a-changelog", "conventional", "custom"
format = "conventional"

# Changelog filename
filename = "CHANGELOG.md"

# Include commit links
include_commit_links = true

# Include issue links (e.g., #123)
include_issue_links = true

# Include author attribution
include_authors = false

# Repository URL for links (auto-detected from git remote if not set)
repository_url = "https://github.com/myorg/myrepo"

# Monorepo mode: "per-package", "root", "both"
monorepo_mode = "per-package"

# Version tag format (for monorepo packages)
# {name} = package name, {version} = version
version_tag_format = "{name}@{version}"

# Root version tag format (for single package or root)
root_tag_format = "v{version}"

[package_tools.changelog.conventional]
# Enable conventional commits parsing
enabled = true

# Commit types to include
types = ["feat", "fix", "perf", "refactor", "docs", "build", "ci", "test", "chore"]

# Section titles for each type
[package_tools.changelog.conventional.sections]
feat = "Features"
fix = "Bug Fixes"
perf = "Performance Improvements"
refactor = "Code Refactoring"
docs = "Documentation"
build = "Build System"
ci = "Continuous Integration"
test = "Tests"
chore = "Chores"

# Breaking changes section title
breaking = "BREAKING CHANGES"

[package_tools.changelog.exclude]
# Exclude commits matching these patterns
patterns = [
    "^chore\\(release\\):",
    "^Merge branch",
    "^Merge pull request",
]

# Exclude commits by author
authors = []

[package_tools.changelog.template]
# Custom template (if format = "custom")
header = "# Changelog"
version_header = "## [{version}] - {date}"
section_header = "### {title}"
entry_format = "- {description} ({short_hash})"
```

#### Configuration Structure

```rust
/// Configuration for changelog generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogConfig {
    /// Enable changelog generation
    pub enabled: bool,

    /// Changelog format
    pub format: ChangelogFormat,

    /// Changelog filename
    pub filename: String,

    /// Include commit links
    pub include_commit_links: bool,

    /// Include issue links
    pub include_issue_links: bool,

    /// Include author attribution
    pub include_authors: bool,

    /// Repository URL (for links)
    pub repository_url: Option<String>,

    /// Monorepo mode
    pub monorepo_mode: MonorepoMode,

    /// Version tag format (monorepo packages)
    pub version_tag_format: String,

    /// Root version tag format (single package)
    pub root_tag_format: String,

    /// Conventional commits configuration
    pub conventional: ConventionalConfig,

    /// Exclusion rules
    pub exclude: ExcludeConfig,

    /// Custom template
    pub template: TemplateConfig,
}

impl Default for ChangelogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            format: ChangelogFormat::Conventional,
            filename: "CHANGELOG.md".to_string(),
            include_commit_links: true,
            include_issue_links: true,
            include_authors: false,
            repository_url: None,
            monorepo_mode: MonorepoMode::PerPackage,
            version_tag_format: "{name}@{version}".to_string(),
            root_tag_format: "v{version}".to_string(),
            conventional: ConventionalConfig::default(),
            exclude: ExcludeConfig::default(),
            template: TemplateConfig::default(),
        }
    }
}

/// Changelog format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangelogFormat {
    /// Keep a Changelog format
    KeepAChangelog,
    /// Conventional Commits format
    Conventional,
    /// Custom format
    Custom,
}

/// Monorepo mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonorepoMode {
    /// One changelog per package
    PerPackage,
    /// Single changelog at root
    Root,
    /// Both per-package and root
    Both,
}

/// Conventional commits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionalConfig {
    pub enabled: bool,
    pub types: Vec<String>,
    pub sections: HashMap<String, String>,
    pub breaking_section: String,
}

impl Default for ConventionalConfig {
    fn default() -> Self {
        let mut sections = HashMap::new();
        sections.insert("feat".to_string(), "Features".to_string());
        sections.insert("fix".to_string(), "Bug Fixes".to_string());
        sections.insert("perf".to_string(), "Performance Improvements".to_string());
        sections.insert("docs".to_string(), "Documentation".to_string());

        Self {
            enabled: true,
            types: vec![
                "feat".to_string(),
                "fix".to_string(),
                "perf".to_string(),
                "docs".to_string(),
            ],
            sections,
            breaking_section: "BREAKING CHANGES".to_string(),
        }
    }
}

/// Exclusion rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcludeConfig {
    pub patterns: Vec<String>,
    pub authors: Vec<String>,
}

impl Default for ExcludeConfig {
    fn default() -> Self {
        Self {
            patterns: vec![
                "^chore\\(release\\):".to_string(),
                "^Merge branch".to_string(),
            ],
            authors: vec![],
        }
    }
}

/// Custom template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub header: String,
    pub version_header: String,
    pub section_header: String,
    pub entry_format: String,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            header: "# Changelog".to_string(),
            version_header: "## [{version}] - {date}".to_string(),
            section_header: "### {title}".to_string(),
            entry_format: "- {description} ({short_hash})".to_string(),
        }
    }
}
```

---

### Usage Examples

#### Example 1: Generate Changelog for Single Package

```rust
use sublime_pkg_tools::changelog::{ChangelogGenerator, ChangelogConfig};

// Create generator
let generator = ChangelogGenerator::new(
    PathBuf::from("/workspace"),
    ChangelogConfig::default()
).await?;

// Generate changelog for new version
let changelog = generator.generate_for_version(
    None,  // Single package (no package name)
    "1.1.0",
    Some("1.0.0")
).await?;

println!("Changelog for v{}", changelog.version);
println!("Previous version: {:?}", changelog.previous_version);
println!("Sections: {}", changelog.sections.len());

// Render to markdown
let markdown = changelog.to_markdown(&ChangelogConfig::default());
println!("{}", markdown);

// Update CHANGELOG.md
generator.update_changelog(
    &PathBuf::from("/workspace"),
    &changelog,
    false  // not dry-run
).await?;
```

#### Example 2: Generate from Changeset (Monorepo)

```rust
// Load changeset
let changeset = changeset_manager.load("feature-branch").await?;

// Resolve versions
let version_resolver = VersionResolver::new(&workspace_root, config).await?;
let resolution = version_resolver.resolve_versions(&changeset).await?;

// Generate changelogs for all affected packages
let changelogs = generator.generate_from_changeset(
    &changeset,
    &resolution
).await?;

println!("Generated {} changelogs", changelogs.len());

for generated in &changelogs {
    println!("\n{}", generated.package_name.as_ref().unwrap_or(&"root".to_string()));
    println!("  Version: {}", generated.changelog.version);
    println!("  Entries: {}", generated.changelog.entry_count());
    println!("  Path: {}", generated.changelog_path.display());
    
    // Write to file
    generated.write(&fs).await?;
}
```

#### Example 3: Preview Changelog (Dry Run)

```rust
let changelog = generator.generate_for_version(
    Some("@myorg/core"),
    "2.0.0",
    Some("1.5.0")
).await?;

// Preview without writing
let updated_content = generator.update_changelog(
    &PathBuf::from("/workspace/packages/core"),
    &changelog,
    true  // dry-run
).await?;

println!("=== PREVIEW ===");
println!("{}", updated_content);
```

#### Example 4: Conventional Commits Grouping

```rust
let changelog = generator.generate_for_version(
    None,
    "1.2.0",
    Some("1.1.0")
).await?;

// Show sections
for section in &changelog.sections {
    println!("\n### {}", section.title);
    for entry in &section.entries {
        let prefix = if entry.breaking { "⚠️  " } else { "" };
        println!("  {}{}", prefix, entry.description);
        
        if let Some(scope) = &entry.scope {
            println!("    Scope: {}", scope);
        }
        
        if !entry.references.is_empty() {
            println!("    Refs: {}", entry.references.join(", "));
        }
    }
}
```

#### Example 5: Check Breaking Changes

```rust
let changelog = generator.generate_for_version(
    None,
    "2.0.0",
    Some("1.9.0")
).await?;

let breaking = changelog.breaking_changes();
if !breaking.is_empty() {
    println!("⚠️  {} BREAKING CHANGES:", breaking.len());
    for entry in breaking {
        println!("  - {}", entry.description);
        println!("    Commit: {}", entry.short_hash);
    }
}
```

#### Example 6: Parse Existing Changelog

```rust
let parsed = generator.parse_existing(
    &PathBuf::from("/workspace/CHANGELOG.md")
).await?;

println!("Found {} versions:", parsed.versions.len());
for version in &parsed.versions {
    println!("  {} ({})", 
        version.version,
        version.date.map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "unknown".to_string())
    );
}
```

#### Example 7: Complete Workflow with Versioning

```rust
// 1. Load changeset
let changeset = changeset_manager.load("feature-branch").await?;

// 2. Resolve versions
let resolution = version_resolver.resolve_versions(&changeset).await?;

// 3. Apply versions (dry-run first)
let apply_result = version_resolver.apply_versions(&changeset, true).await?;
println!("Packages to update: {}", apply_result.resolution.updates.len());

// 4. Generate changelogs
let changelogs = generator.generate_from_changeset(
    &changeset,
    &resolution
).await?;

// 5. Preview changelogs
for generated in &changelogs {
    println!("\n=== {} ===", 
        generated.package_name.as_ref().unwrap_or(&"root".to_string())
    );
    println!("{}", generated.content);
}

// User confirms...

// 6. Apply versions (real)
let apply_result = version_resolver.apply_versions(&changeset, false).await?;

// 7. Write changelogs
for generated in &changelogs {
    generated.write(&fs).await?;
    println!("✓ Updated {}", generated.changelog_path.display());
}

// 8. Git commit
// (User commits changes)

// 9. Archive changeset
changeset_manager.archive(&changeset.branch, release_info).await?;
```

#### Example 8: Custom Format

```rust
let mut config = ChangelogConfig::default();
config.format = ChangelogFormat::Custom;
config.template.entry_format = "* {description} by {author} ({short_hash})".to_string();
config.include_authors = true;

let generator = ChangelogGenerator::new(workspace_root, config).await?;

let changelog = generator.generate_for_version(
    None,
    "1.0.0",
    None
).await?;

// Will use custom template
let markdown = changelog.to_markdown(&generator.config);

// Generate merge commit message
let merge_message = generated.generate_merge_commit_message(
    &config.git,
    Some("0.9.0")
)?;
println!("Suggested commit message:\n{}", merge_message);
```

---

### Integration with Git Tools

The changelog generator uses `sublime_git_tools` for all git operations.

```rust
use sublime_git_tools::{
    repository::{GitRepository, RepositoryTrait},
    commit::{Commit, CommitTrait},
    tag::{Tag, TagTrait},
    log::{LogOptions, LogTrait},
};

impl ChangelogGenerator {
    /// Detects previous version from git tags
    async fn detect_previous_version_impl(
        &self,
        package_name: Option<&str>,
        current_version: &str,
    ) -> Result<Option<String>, ChangelogError> {
        // Get all tags
        let tags = self.git_repo.list_tags().await?;
        
        // Filter by package (if monorepo)
        let version_tags: Vec<_> = tags.iter()
            .filter_map(|tag| self.parse_version_tag(tag, package_name))
            .collect();
        
        // Find previous version
        let current = Version::parse(current_version)?;
        let previous = version_tags.iter()
            .filter(|(v, _)| v < &current)
            .max_by_key(|(v, _)| v)
            .map(|(_, tag)| tag.clone());
        
        Ok(previous)
    }

    /// Gets commits between two versions
    async fn get_commits_between(
        &self,
        previous_tag: Option<&str>,
        current_ref: &str,
    ) -> Result<Vec<Commit>, ChangelogError> {
        let range = if let Some(prev) = previous_tag {
            format!("{}..{}", prev, current_ref)
        } else {
            // No previous tag, get all commits to current
            current_ref.to_string()
        };
        
        let log_options = LogOptions::default();
        let commits = self.git_repo.log(&range, log_options).await?;
        
        Ok(commits)
    }

    /// Filters commits by package (for monorepo)
    async fn filter_commits_by_package(
        &self,
        commits: &[Commit],
        package_name: &str,
    ) -> Result<Vec<Commit>, ChangelogError> {
        let mut filtered = Vec::new();
        
        // Get package path
        let monorepo = self.monorepo_detector.detect_monorepo(&self.workspace_root).await?;
        let package = monorepo.packages().iter()
            .find(|p| p.name == package_name)
            .ok_or_else(|| ChangelogError::PackageNotFound(package_name.to_string()))?;
        
        for commit in commits {
            // Get files changed in this commit
            let diff = self.git_repo.diff_commit(&commit.hash).await?;
            let files: Vec<PathBuf> = diff.entries().iter()
                .map(|e| e.path.clone())
                .collect();
            
            // Check if any file is under package path
            let affects_package = files.iter()
                .any(|f| f.starts_with(&package.relative_path));
            
            if affects_package {
                filtered.push(commit.clone());
            }
        }
        
        Ok(filtered)
    }
}
```

---

### Integration with FileSystem

Uses `sublime_standard_tools` for all file operations.

```rust
use sublime_standard_tools::{
    filesystem::{AsyncFileSystem, FileSystemManager},
};

impl ChangelogGenerator {
    /// Updates changelog file
    async fn update_changelog_impl(
        &self,
        package_path: &Path,
        changelog: &Changelog,
        dry_run: bool,
    ) -> Result<String, ChangelogError> {
        let changelog_path = package_path.join(&self.config.filename);
        
        // Render new content
        let new_section = changelog.to_markdown(&self.config);
        
        // Read existing changelog (if exists)
        let existing_content = if self.fs.exists(&changelog_path).await? {
            self.fs.read_file_string(&changelog_path).await?
        } else {
            // Create header
            self.config.template.header.clone() + "\n\n"
        };
        
        // Prepend new section
        let updated_content = self.prepend_changelog(&existing_content, &new_section);
        
        // Write if not dry-run
        if !dry_run {
            self.fs.write_file_string(&changelog_path, &updated_content).await?;
        }
        
        Ok(updated_content)
    }

    /// Prepends new changelog section
    fn prepend_changelog(&self, existing: &str, new_section: &str) -> String {
        // Find where to insert (after header, before first version)
        let lines: Vec<&str> = existing.lines().collect();
        
        // Skip header and empty lines
        let insert_pos = lines.iter()
            .position(|line| line.starts_with("## "))
            .unwrap_or(lines.len());
        
        let mut result = String::new();
        
        // Add lines before insert position
        for line in &lines[..insert_pos] {
            result.push_str(line);
            result.push('\n');
        }
        
        // Add new section
        result.push_str(new_section);
        result.push_str("\n\n");
        
        // Add remaining lines
        for line in &lines[insert_pos..] {
            result.push_str(line);
            result.push('\n');
        }
        
        result
    }
}
```

---

### Conventional Commits Parser

Parses commit messages following conventional commits spec.

```rust
impl ConventionalCommit {
    /// Parses a commit message
    pub fn parse(message: &str) -> Result<Self, ParseError> {
        let lines: Vec<&str> = message.lines().collect();
        if lines.is_empty() {
            return Err(ParseError::EmptyMessage);
        }
        
        let first_line = lines[0];
        
        // Parse type, scope, breaking, description
        let (commit_type, scope, breaking, description) = Self::parse_subject(first_line)?;
        
        // Parse body and footers
        let (body, footers) = Self::parse_body_and_footers(&lines[1..]);
        
        // Check for BREAKING CHANGE in footers
        let breaking = breaking || footers.iter()
            .any(|f| f.key == "BREAKING CHANGE" || f.key == "BREAKING-CHANGE");
        
        Ok(Self {
            commit_type,
            scope,
            breaking,
            description,
            body,
            footers,
        })
    }

    /// Parses subject line
    fn parse_subject(line: &str) -> Result<(String, Option<String>, bool, String), ParseError> {
        // Regex: ^(\\w+)(\\(([^)]+)\\))?(!)?: (.+)$
        // Example: feat(core)!: add new API
        
        let re = regex::Regex::new(r"^(\w+)(\(([^)]+)\))?(!)?:\s*(.+)$")
            .map_err(|e| ParseError::RegexError(e.to_string()))?;
        
        let caps = re.captures(line)
            .ok_or_else(|| ParseError::InvalidFormat(line.to_string()))?;
        
        let commit_type = caps.get(1).unwrap().as_str().to_string();
        let scope = caps.get(3).map(|m| m.as_str().to_string());
        let breaking = caps.get(4).is_some();
        let description = caps.get(5).unwrap().as_str().to_string();
        
        Ok((commit_type, scope, breaking, description))
    }

    /// Parses body and footers
    fn parse_body_and_footers(lines: &[&str]) -> (Option<String>, Vec<CommitFooter>) {
        // Skip empty lines
        let lines: Vec<&str> = lines.iter()
            .skip_while(|l| l.is_empty())
            .copied()
            .collect();
        
        if lines.is_empty() {
            return (None, vec![]);
        }
        
        // Find footer start (line with "Key: value" or "Key #value")
        let footer_start = lines.iter()
            .position(|l| Self::is_footer_line(l))
            .unwrap_or(lines.len());
        
        let body = if footer_start > 0 {
            Some(lines[..footer_start].join("\n").trim().to_string())
        } else {
            None
        };
        
        let footers = if footer_start < lines.len() {
            Self::parse_footers(&lines[footer_start..])
        } else {
            vec![]
        };
        
        (body, footers)
    }

    /// Checks if line is a footer
    fn is_footer_line(line: &str) -> bool {
        line.contains(':') && !line.trim().is_empty()
    }

    /// Parses footers
    fn parse_footers(lines: &[&str]) -> Vec<CommitFooter> {
        let mut footers = Vec::new();
        let mut current: Option<CommitFooter> = None;
        
        for line in lines {
            if let Some(pos) = line.find(':') {
                // New footer
                if let Some(footer) = current.take() {
                    footers.push(footer);
                }
                
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();
                current = Some(CommitFooter { key, value });
            } else if let Some(ref mut footer) = current {
                // Continuation of previous footer
                footer.value.push(' ');
                footer.value.push_str(line.trim());
            }
        }
        
        if let Some(footer) = current {
            footers.push(footer);
        }
        
        footers
    }

    /// Gets section type for this commit
    pub fn section_type(&self) -> SectionType {
        if self.breaking {
            return SectionType::Breaking;
        }
        
        match self.commit_type.as_str() {
            "feat" => SectionType::Features,
            "fix" => SectionType::Fixes,
            "perf" => SectionType::Performance,
            "docs" => SectionType::Documentation,
            "refactor" => SectionType::Refactoring,
            "build" => SectionType::Build,
            "ci" => SectionType::CI,
            "test" => SectionType::Tests,
            _ => SectionType::Other,
        }
    }

    /// Extracts issue references
    pub fn extract_references(&self) -> Vec<String> {
        let mut refs = Vec::new();
        
        // Check description
        refs.extend(Self::find_refs_in_text(&self.description));
        
        // Check body
        if let Some(body) = &self.body {
            refs.extend(Self::find_refs_in_text(body));
        }
        
        // Check footers
        for footer in &self.footers {
            if footer.key.to_lowercase().contains("ref") 
                || footer.key.to_lowercase().contains("close")
                || footer.key.to_lowercase().contains("fix") {
                refs.extend(Self::find_refs_in_text(&footer.value));
            }
        }
        
        refs
    }

    /// Finds references in text (#123, GH-456, etc.)
    fn find_refs_in_text(text: &str) -> Vec<String> {
        let re = regex::Regex::new(r"#(\d+)").unwrap();
        re.captures_iter(text)
            .filter_map(|cap| cap.get(0).map(|m| m.as_str().to_string()))
            .collect()
    }
}
```

---

### Error Handling

```rust
/// Errors that can occur during changelog operations
#[derive(Debug, thiserror::Error)]
pub enum ChangelogError {
    #[error("Git error: {0}")]
    Git(#[from] GitError),

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Version not found: {0}")]
    VersionNotFound(String),

    #[error("Failed to parse version: {0}")]
    VersionParse(String),

    #[error("Failed to parse commit: {0}")]
    CommitParse(#[from] ParseError),

    #[error("Changelog not found: {0}")]
    ChangelogNotFound(PathBuf),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Errors related to parsing
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Empty commit message")]
    EmptyMessage,

    #[error("Invalid conventional commit format: {0}")]
    InvalidFormat(String),

    #[error("Regex error: {0}")]
    RegexError(String),
}
```

---

### Module Structure

```text
crates/pkg/src/changelog/
├── mod.rs                      # Public API exports
├── generator.rs                # ChangelogGenerator implementation
├── changelog.rs                # Changelog types
├── section.rs                  # ChangelogSection and SectionType
├── entry.rs                    # ChangelogEntry
├── conventional.rs             # ConventionalCommit parser
├── parser.rs                   # Existing changelog parser
├── formatter/
│   ├── mod.rs                  # Formatter exports
│   ├── keep_a_changelog.rs     # Keep a Changelog formatter
│   ├── conventional.rs         # Conventional Commits formatter
│   └── custom.rs               # Custom template formatter
├── config.rs                   # Configuration types
└── error.rs                    # Error types
```

---

### Dependencies

```toml
[dependencies]
# Git operations
sublime_git_tools = { path = "../git" }

# Filesystem operations
sublime_standard_tools = { path = "../standard" }

# Regex for conventional commits parsing
regex = "1.10"

# Semver for version comparison
semver = "1.0"

# Already available
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
```

---

### Design Principles

1. **Git-based**: All change detection via git diff/log, not file scanning
2. **Configurable**: Support multiple formats and templates
3. **Conventional commits**: First-class support with automatic grouping
4. **Monorepo-aware**: Per-package or root changelogs
5. **Incremental**: Update existing changelogs, don't replace
6. **Links**: Automatic linking to commits and issues
7. **Dry-run**: Preview before writing
8. **Testability**: All components independently testable
9. **Standard formats**: Follow Keep a Changelog and Conventional Commits specs

---

### Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_conventional_commit() {
        let commit = ConventionalCommit::parse("feat(core): add new API").unwrap();
        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, Some("core".to_string()));
        assert_eq!(commit.description, "add new API");
        assert!(!commit.breaking);
    }

    #[test]
    fn test_parse_breaking_change() {
        let commit = ConventionalCommit::parse("feat!: breaking change").unwrap();
        assert!(commit.breaking);
    }

    #[test]
    fn test_extract_references() {
        let commit = ConventionalCommit::parse(
            "fix: resolve issue\n\nCloses #123, fixes #456"
        ).unwrap();
        let refs = commit.extract_references();
        assert!(refs.contains(&"#123".to_string()));
        assert!(refs.contains(&"#456".to_string()));
    }

    #[tokio::test]
    async fn test_generate_changelog_single_package() {
        // Test changelog generation for single package
    }

    #[tokio::test]
    async fn test_generate_changelog_monorepo() {
        // Test changelog generation for monorepo
    }

    #[tokio::test]
    async fn test_update_existing_changelog() {
        // Test updating existing CHANGELOG.md
    }

    #[tokio::test]
    async fn test_detect_previous_version() {
        // Test version detection from git tags
    }

    #[tokio::test]
    async fn test_filter_commits_by_package() {
        // Test filtering commits for specific package
    }
}
```

---

## Audit & Health Checks

### Overview

The **Audit** module provides comprehensive health checks and analysis of the repository and its packages. It aggregates information from all other modules to provide a holistic view of:

- **Available upgrades** for external dependencies
- **Circular dependencies** between internal packages
- **Breaking changes** in packages
- **Dependency categorization** (internal, external, workspace links)
- **Package health** and potential issues
- **Suggested actions** to improve repository health

This module follows the same principles:
- **Library-only**: Pure API, no CLI
- **Aggregation**: Combines data from multiple modules
- **Actionable**: Provides clear issues and suggestions
- **Configurable**: Control what to audit and severity thresholds

---

### Core Concepts

#### Audit Scope

The audit can analyze:
1. **Upgrade opportunities**: External dependencies with available updates
2. **Dependency health**: Circular dependencies, missing dependencies, version conflicts
3. **Internal packages**: List and analyze workspace packages
4. **External packages**: List and categorize registry dependencies
5. **Workspace links**: Identify workspace:*, file:, link:, portal: references
6. **Breaking changes**: Identify packages with breaking changes (from changeset/changelog)
7. **Version consistency**: Check if internal dependencies use consistent versions

#### Issue Severity

Issues are classified by severity:
- **Critical**: Must be fixed (e.g., circular dependencies blocking builds)
- **Warning**: Should be fixed (e.g., major version upgrades available)
- **Info**: Nice to have (e.g., patch upgrades available)

#### Dependency Categories

Packages are categorized as:
- **Internal**: Workspace packages (part of the monorepo)
- **External**: Published to registries (npm, private registries)
- **Workspace Link**: Using workspace protocol (workspace:*, workspace:^, etc.)
- **Local Link**: Using local protocols (file:, link:, portal:)

---

### Data Structures

#### AuditManager

Main entry point for audit operations.

```rust
/// Performs comprehensive audits of repository and packages
///
/// Aggregates information from multiple modules to provide
/// a complete health check of the repository.
pub struct AuditManager {
    workspace_root: PathBuf,
    upgrade_manager: UpgradeManager,
    changes_analyzer: ChangesAnalyzer,
    fs: FileSystemManager,
    monorepo_detector: MonorepoDetector,
    config: AuditConfig,
}

impl AuditManager {
    /// Creates a new AuditManager
    ///
    /// # Arguments
    /// * `workspace_root` - Root directory of the workspace
    /// * `config` - Audit configuration
    ///
    /// # Examples
    /// ```rust
    /// let audit = AuditManager::new(
    ///     PathBuf::from("/workspace"),
    ///     AuditConfig::default()
    /// ).await?;
    /// ```
    pub async fn new(
        workspace_root: PathBuf,
        config: AuditConfig,
    ) -> Result<Self, AuditError>;

    /// Performs a complete audit
    ///
    /// Analyzes all aspects of the repository and returns
    /// a comprehensive report.
    ///
    /// # Examples
    /// ```rust
    /// let report = audit.run_audit().await?;
    /// println!("Issues found: {}", report.summary.total_issues);
    /// ```
    pub async fn run_audit(&self) -> Result<AuditReport, AuditError>;

    /// Audits upgrade opportunities
    ///
    /// Checks all external dependencies for available updates.
    pub async fn audit_upgrades(&self) -> Result<UpgradeAuditSection, AuditError>;

    /// Audits dependency graph
    ///
    /// Checks for circular dependencies, missing dependencies, etc.
    pub async fn audit_dependencies(&self) -> Result<DependencyAuditSection, AuditError>;

    /// Audits breaking changes
    ///
    /// Identifies packages with breaking changes based on
    /// changeset or changelog information.
    ///
    /// # Arguments
    /// * `changeset` - Optional changeset to analyze
    pub async fn audit_breaking_changes(
        &self,
        changeset: Option<&Changeset>,
    ) -> Result<BreakingChangesAuditSection, AuditError>;

    /// Categorizes all dependencies
    ///
    /// Returns lists of internal, external, and linked dependencies.
    pub async fn categorize_dependencies(&self) -> Result<DependencyCategorization, AuditError>;

    /// Audits version consistency
    ///
    /// Checks if internal dependencies use consistent versions across packages.
    pub async fn audit_version_consistency(&self) -> Result<VersionConsistencyAuditSection, AuditError>;
}
```

#### AuditReport

Complete audit report with all findings.

```rust
/// Complete audit report
///
/// Contains all audit sections and a summary of findings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    /// When the audit was performed
    pub audited_at: DateTime<Utc>,

    /// Workspace root
    pub workspace_root: PathBuf,

    /// Is this a monorepo?
    pub is_monorepo: bool,

    /// All audit sections
    pub sections: AuditSections,

    /// Summary of all findings
    pub summary: AuditSummary,

    /// Overall health score (0-100)
    pub health_score: u8,
}

impl AuditReport {
    /// Gets all issues by severity
    pub fn issues_by_severity(&self, severity: IssueSeverity) -> Vec<&AuditIssue>;

    /// Gets all critical issues
    pub fn critical_issues(&self) -> Vec<&AuditIssue>;

    /// Gets all warnings
    pub fn warnings(&self) -> Vec<&AuditIssue>;

    /// Checks if audit passed (no critical issues)
    pub fn passed(&self) -> bool;

    /// Renders report as markdown
    pub fn to_markdown(&self) -> String;

    /// Renders report as JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error>;
}

/// All audit sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSections {
    /// Upgrade opportunities
    pub upgrades: UpgradeAuditSection,

    /// Dependency graph analysis
    pub dependencies: DependencyAuditSection,

    /// Breaking changes
    pub breaking_changes: BreakingChangesAuditSection,

    /// Dependency categorization
    pub categorization: DependencyCategorization,

    /// Version consistency
    pub version_consistency: VersionConsistencyAuditSection,
}

/// Summary of audit findings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    /// Total packages analyzed
    pub packages_analyzed: usize,

    /// Total dependencies analyzed
    pub dependencies_analyzed: usize,

    /// Total issues found
    pub total_issues: usize,

    /// Critical issues
    pub critical_issues: usize,

    /// Warnings
    pub warnings: usize,

    /// Info items
    pub info_items: usize,

    /// Suggested actions
    pub suggested_actions: Vec<String>,
}
```

#### Audit Sections

##### UpgradeAuditSection

```rust
/// Audit section for upgrade opportunities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeAuditSection {
    /// Total upgrades available
    pub total_upgrades: usize,

    /// Major upgrades (breaking changes possible)
    pub major_upgrades: usize,

    /// Minor upgrades (new features)
    pub minor_upgrades: usize,

    /// Patch upgrades (bug fixes)
    pub patch_upgrades: usize,

    /// Deprecated packages found
    pub deprecated_packages: Vec<DeprecatedPackage>,

    /// Upgrades by package
    pub upgrades_by_package: HashMap<String, Vec<DependencyUpgrade>>,

    /// Issues found
    pub issues: Vec<AuditIssue>,
}

/// Deprecated package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecatedPackage {
    pub name: String,
    pub current_version: String,
    pub deprecation_message: String,
    pub alternative: Option<String>,
}
```

##### DependencyAuditSection

```rust
/// Audit section for dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAuditSection {
    /// Circular dependencies found
    pub circular_dependencies: Vec<CircularDependency>,

    /// Missing dependencies (used but not declared)
    pub missing_dependencies: Vec<MissingDependency>,

    /// Unused dependencies (declared but not used)
    pub unused_dependencies: Vec<UnusedDependency>,

    /// Version conflicts
    pub version_conflicts: Vec<VersionConflict>,

    /// Issues found
    pub issues: Vec<AuditIssue>,
}

/// Missing dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingDependency {
    pub package_name: String,
    pub dependency_name: String,
    pub used_in_files: Vec<PathBuf>,
}

/// Unused dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedDependency {
    pub package_name: String,
    pub dependency_name: String,
    pub dependency_type: DependencyType,
}

/// Version conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConflict {
    pub dependency_name: String,
    pub versions: Vec<VersionUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionUsage {
    pub package_name: String,
    pub version_spec: String,
}
```

##### BreakingChangesAuditSection

```rust
/// Audit section for breaking changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChangesAuditSection {
    /// Packages with breaking changes
    pub packages_with_breaking: Vec<PackageBreakingChanges>,

    /// Total breaking changes found
    pub total_breaking_changes: usize,

    /// Issues found
    pub issues: Vec<AuditIssue>,
}

/// Breaking changes for a package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageBreakingChanges {
    pub package_name: String,
    pub current_version: Option<Version>,
    pub next_version: Option<Version>,
    pub breaking_changes: Vec<BreakingChange>,
}

/// Single breaking change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    pub description: String,
    pub commit_hash: Option<String>,
    pub source: BreakingChangeSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakingChangeSource {
    /// From conventional commit (feat!, fix!, etc.)
    ConventionalCommit,
    /// From changelog
    Changelog,
    /// From changeset
    Changeset,
}
```

##### DependencyCategorization

```rust
/// Categorization of all dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyCategorization {
    /// Internal packages (workspace packages)
    pub internal_packages: Vec<InternalPackage>,

    /// External packages (from registries)
    pub external_packages: Vec<ExternalPackage>,

    /// Workspace links (workspace:*, workspace:^, etc.)
    pub workspace_links: Vec<WorkspaceLink>,

    /// Local links (file:, link:, portal:)
    pub local_links: Vec<LocalLink>,

    /// Summary statistics
    pub stats: CategorizationStats,
}

/// Internal package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalPackage {
    pub name: String,
    pub path: PathBuf,
    pub version: Option<Version>,
    pub used_by: Vec<String>,  // Package names that depend on this
}

/// External package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalPackage {
    pub name: String,
    pub version_spec: String,
    pub used_by: Vec<String>,  // Package names that use this
    pub is_deprecated: bool,
}

/// Workspace link information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLink {
    pub package_name: String,
    pub dependency_name: String,
    pub version_spec: String,  // e.g., "workspace:*", "workspace:^1.0.0"
}

/// Local link information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalLink {
    pub package_name: String,
    pub dependency_name: String,
    pub link_type: LocalLinkType,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalLinkType {
    File,    // file:
    Link,    // link:
    Portal,  // portal:
}

/// Categorization statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorizationStats {
    pub total_packages: usize,
    pub internal_packages: usize,
    pub external_packages: usize,
    pub workspace_links: usize,
    pub local_links: usize,
}
```

##### VersionConsistencyAuditSection

```rust
/// Audit section for version consistency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConsistencyAuditSection {
    /// Inconsistent versions found
    pub inconsistencies: Vec<VersionInconsistency>,

    /// Issues found
    pub issues: Vec<AuditIssue>,
}

/// Version inconsistency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInconsistency {
    /// Internal package name
    pub package_name: String,

    /// Different versions used across workspace
    pub versions_used: Vec<VersionUsage>,

    /// Recommended version (usually the latest)
    pub recommended_version: String,
}
```

#### AuditIssue

```rust
/// Single audit issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditIssue {
    /// Issue severity
    pub severity: IssueSeverity,

    /// Issue category
    pub category: IssueCategory,

    /// Issue title
    pub title: String,

    /// Detailed description
    pub description: String,

    /// Affected packages
    pub affected_packages: Vec<String>,

    /// Suggested action
    pub suggestion: Option<String>,

    /// Related data (for programmatic access)
    pub metadata: HashMap<String, String>,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    Warning,
    Info,
}

/// Issue category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueCategory {
    Upgrades,
    Dependencies,
    BreakingChanges,
    VersionConsistency,
    Security,
    Other,
}
```

---

### Configuration

```toml
[package_tools.audit]
# Enable audit functionality
enabled = true

# Minimum severity to report (Critical, Warning, Info)
min_severity = "Warning"

# Audit sections to include
[package_tools.audit.sections]
upgrades = true
dependencies = true
breaking_changes = true
categorization = true
version_consistency = true

# Upgrade audit configuration
[package_tools.audit.upgrades]
# Include patch upgrades in report
include_patch = false
# Include minor upgrades in report
include_minor = true
# Include major upgrades in report
include_major = true
# Report deprecated packages as critical
deprecated_as_critical = true

# Dependency audit configuration
[package_tools.audit.dependencies]
# Check for circular dependencies
check_circular = true
# Check for missing dependencies
check_missing = false  # Requires code analysis
# Check for unused dependencies
check_unused = false   # Requires code analysis
# Check for version conflicts
check_version_conflicts = true

# Breaking changes configuration
[package_tools.audit.breaking_changes]
# Check conventional commits for breaking changes
check_conventional_commits = true
# Check changelog for breaking changes
check_changelog = true

# Version consistency configuration
[package_tools.audit.version_consistency]
# Fail on inconsistent versions
fail_on_inconsistency = false
# Warn on inconsistent versions
warn_on_inconsistency = true
```

```rust
/// Configuration for audit operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub min_severity: IssueSeverity,
    pub sections: AuditSectionsConfig,
    pub upgrades: UpgradeAuditConfig,
    pub dependencies: DependencyAuditConfig,
    pub breaking_changes: BreakingChangesAuditConfig,
    pub version_consistency: VersionConsistencyAuditConfig,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_severity: IssueSeverity::Warning,
            sections: AuditSectionsConfig::default(),
            upgrades: UpgradeAuditConfig::default(),
            dependencies: DependencyAuditConfig::default(),
            breaking_changes: BreakingChangesAuditConfig::default(),
            version_consistency: VersionConsistencyAuditConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSectionsConfig {
    pub upgrades: bool,
    pub dependencies: bool,
    pub breaking_changes: bool,
    pub categorization: bool,
    pub version_consistency: bool,
}

impl Default for AuditSectionsConfig {
    fn default() -> Self {
        Self {
            upgrades: true,
            dependencies: true,
            breaking_changes: true,
            categorization: true,
            version_consistency: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeAuditConfig {
    pub include_patch: bool,
    pub include_minor: bool,
    pub include_major: bool,
    pub deprecated_as_critical: bool,
}

impl Default for UpgradeAuditConfig {
    fn default() -> Self {
        Self {
            include_patch: false,
            include_minor: true,
            include_major: true,
            deprecated_as_critical: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAuditConfig {
    pub check_circular: bool,
    pub check_missing: bool,
    pub check_unused: bool,
    pub check_version_conflicts: bool,
}

impl Default for DependencyAuditConfig {
    fn default() -> Self {
        Self {
            check_circular: true,
            check_missing: false,
            check_unused: false,
            check_version_conflicts: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChangesAuditConfig {
    pub check_conventional_commits: bool,
    pub check_changelog: bool,
}

impl Default for BreakingChangesAuditConfig {
    fn default() -> Self {
        Self {
            check_conventional_commits: true,
            check_changelog: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConsistencyAuditConfig {
    pub fail_on_inconsistency: bool,
    pub warn_on_inconsistency: bool,
}

impl Default for VersionConsistencyAuditConfig {
    fn default() -> Self {
        Self {
            fail_on_inconsistency: false,
            warn_on_inconsistency: true,
        }
    }
}
```

---

### Usage Examples

#### Example 1: Complete Audit

```rust
use sublime_pkg_tools::audit::{AuditManager, AuditConfig};

// Create audit manager
let audit = AuditManager::new(
    PathBuf::from("/workspace"),
    AuditConfig::default()
).await?;

// Run complete audit
let report = audit.run_audit().await?;

println!("=== AUDIT REPORT ===");
println!("Health Score: {}/100", report.health_score);
println!("Packages: {}", report.summary.packages_analyzed);
println!("Issues: {} (Critical: {}, Warnings: {})",
    report.summary.total_issues,
    report.summary.critical_issues,
    report.summary.warnings
);

// Check if passed
if !report.passed() {
    println!("\n⚠️  AUDIT FAILED - Critical issues found:");
    for issue in report.critical_issues() {
        println!("  - {}: {}", issue.title, issue.description);
    }
}

// Print markdown report
println!("\n{}", report.to_markdown());
```

#### Example 2: Audit Upgrades Only

```rust
let report = audit.audit_upgrades().await?;

println!("Available upgrades:");
println!("  Major: {} ⚠️", report.major_upgrades);
println!("  Minor: {}", report.minor_upgrades);
println!("  Patch: {}", report.patch_upgrades);

if !report.deprecated_packages.is_empty() {
    println!("\n❌ Deprecated packages:");
    for pkg in &report.deprecated_packages {
        println!("  - {}: {}", pkg.name, pkg.deprecation_message);
        if let Some(alt) = &pkg.alternative {
            println!("    → Use {} instead", alt);
        }
    }
}
```

#### Example 3: Check Circular Dependencies

```rust
let deps_audit = audit.audit_dependencies().await?;

if !deps_audit.circular_dependencies.is_empty() {
    println!("❌ Circular dependencies found:");
    for circular in &deps_audit.circular_dependencies {
        println!("  Cycle: {}", circular.cycle.join(" → "));
    }
}

if !deps_audit.version_conflicts.is_empty() {
    println!("\n⚠️  Version conflicts:");
    for conflict in &deps_audit.version_conflicts {
        println!("  {}", conflict.dependency_name);
        for usage in &conflict.versions {
            println!("    - {} uses {}", usage.package_name, usage.version_spec);
        }
    }
}
```

#### Example 4: Categorize Dependencies

```rust
let categorization = audit.categorize_dependencies().await?;

println!("=== DEPENDENCY CATEGORIZATION ===");
println!("\n📦 Internal Packages ({}):", categorization.stats.internal_packages);
for pkg in &categorization.internal_packages {
    println!("  - {} (v{})", pkg.name, pkg.version.as_ref().unwrap());
    if !pkg.used_by.is_empty() {
        println!("    Used by: {}", pkg.used_by.join(", "));
    }
}

println!("\n🌐 External Packages ({}):", categorization.stats.external_packages);
for pkg in &categorization.external_packages {
    let deprecated = if pkg.is_deprecated { " [DEPRECATED]" } else { "" };
    println!("  - {} ({}){}", pkg.name, pkg.version_spec, deprecated);
}

println!("\n🔗 Workspace Links ({}):", categorization.stats.workspace_links);
for link in &categorization.workspace_links {
    println!("  - {} → {} ({})", 
        link.package_name, 
        link.dependency_name, 
        link.version_spec
    );
}

println!("\n📁 Local Links ({}):", categorization.stats.local_links);
for link in &categorization.local_links {
    println!("  - {} → {} ({:?}: {})", 
        link.package_name, 
        link.dependency_name, 
        link.link_type,
        link.path
    );
}
```

#### Example 5: Check Breaking Changes

```rust
// Load changeset
let changeset = changeset_manager.load("feature-branch").await?;

// Audit breaking changes
let breaking_audit = audit.audit_breaking_changes(Some(&changeset)).await?;

if breaking_audit.total_breaking_changes > 0 {
    println!("⚠️  {} packages with breaking changes:", 
        breaking_audit.packages_with_breaking.len()
    );
    
    for pkg in &breaking_audit.packages_with_breaking {
        println!("\n  {}", pkg.package_name);
        println!("    {} → {}", 
            pkg.current_version.as_ref().unwrap(),
            pkg.next_version.as_ref().unwrap()
        );
        
        for change in &pkg.breaking_changes {
            println!("    - {} ({:?})", 
                change.description,
                change.source
            );
        }
    }
}
```

#### Example 6: Version Consistency Check

```rust
let consistency = audit.audit_version_consistency().await?;

if !consistency.inconsistencies.is_empty() {
    println!("⚠️  Version inconsistencies found:");
    
    for inconsistency in &consistency.inconsistencies {
        println!("\n  {}", inconsistency.package_name);
        println!("    Versions used:");
        for usage in &inconsistency.versions_used {
            println!("      - {} uses {}", 
                usage.package_name, 
                usage.version_spec
            );
        }
        println!("    Recommended: {}", inconsistency.recommended_version);
    }
}
```

#### Example 7: Filter by Severity

```rust
let report = audit.run_audit().await?;

// Get only critical issues
let critical = report.issues_by_severity(IssueSeverity::Critical);
println!("Critical issues: {}", critical.len());
for issue in critical {
    println!("  ❌ {}", issue.title);
    println!("     {}", issue.description);
    if let Some(suggestion) = &issue.suggestion {
        println!("     → {}", suggestion);
    }
}

// Get warnings
let warnings = report.issues_by_severity(IssueSeverity::Warning);
println!("\nWarnings: {}", warnings.len());
for issue in warnings {
    println!("  ⚠️  {}", issue.title);
}
```

#### Example 8: Export Report

```rust
let report = audit.run_audit().await?;

// Export as markdown
let markdown = report.to_markdown();
fs::write("AUDIT.md", markdown).await?;

// Export as JSON
let json = report.to_json()?;
fs::write("audit-report.json", json).await?;

println!("Reports written to AUDIT.md and audit-report.json");
```

#### Example 9: CI/CD Integration

```rust
// Run audit in CI/CD
let audit = AuditManager::new(workspace_root, config).await?;
let report = audit.run_audit().await?;

// Check health score
if report.health_score < 80 {
    eprintln!("❌ Health score below threshold: {}/100", report.health_score);
    std::process::exit(1);
}

// Check for critical issues
if !report.passed() {
    eprintln!("❌ Audit failed with {} critical issues", 
        report.summary.critical_issues
    );
    for issue in report.critical_issues() {
        eprintln!("  - {}", issue.title);
    }
    std::process::exit(1);
}

println!("✅ Audit passed - Health score: {}/100", report.health_score);
```

---

### Integration with Other Modules

The audit manager integrates with all other modules:

```rust
impl AuditManager {
    /// Audit upgrades using UpgradeManager
    async fn audit_upgrades_impl(&self) -> Result<UpgradeAuditSection, AuditError> {
        // Use upgrade manager to detect upgrades
        let preview = self.upgrade_manager.detect_upgrades(
            DetectionOptions::all()
        ).await?;
        
        // Convert to audit section
        let mut section = UpgradeAuditSection {
            total_upgrades: preview.summary.upgrades_available,
            major_upgrades: preview.summary.major_upgrades,
            minor_upgrades: preview.summary.minor_upgrades,
            patch_upgrades: preview.summary.patch_upgrades,
            deprecated_packages: vec![],
            upgrades_by_package: HashMap::new(),
            issues: vec![],
        };
        
        // Check for deprecated packages
        for pkg in &preview.packages {
            for upgrade in &pkg.upgrades {
                if let Some(deprecated) = &upgrade.version_info.deprecated {
                    section.deprecated_packages.push(DeprecatedPackage {
                        name: upgrade.name.clone(),
                        current_version: upgrade.current_version.clone(),
                        deprecation_message: deprecated.clone(),
                        alternative: None,
                    });
                    
                    // Add critical issue
                    section.issues.push(AuditIssue {
                        severity: IssueSeverity::Critical,
                        category: IssueCategory::Upgrades,
                        title: format!("Deprecated package: {}", upgrade.name),
                        description: deprecated.clone(),
                        affected_packages: vec![pkg.package_name.clone()],
                        suggestion: Some("Update to a non-deprecated package".to_string()),
                        metadata: HashMap::new(),
                    });
                }
            }
        }
        
        Ok(section)
    }

    /// Audit dependencies using DependencyGraph
    async fn audit_dependencies_impl(&self) -> Result<DependencyAuditSection, AuditError> {
        // Build dependency graph
        let packages = self.get_all_packages().await?;
        let graph = DependencyGraph::from_packages(&packages)?;
        
        // Detect circular dependencies
        let circular = graph.detect_cycles();
        
        // Create issues for circular dependencies
        let mut issues = vec![];
        for cycle in &circular {
            issues.push(AuditIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::Dependencies,
                title: "Circular dependency detected".to_string(),
                description: format!("Cycle: {}", cycle.cycle.join(" → ")),
                affected_packages: cycle.cycle.clone(),
                suggestion: Some("Refactor to break the circular dependency".to_string()),
                metadata: HashMap::new(),
            });
        }
        
        Ok(DependencyAuditSection {
            circular_dependencies: circular,
            missing_dependencies: vec![],
            unused_dependencies: vec![],
            version_conflicts: vec![],
            issues,
        })
    }

    /// Audit breaking changes using ChangesAnalyzer
    async fn audit_breaking_changes_impl(
        &self,
        changeset: Option<&Changeset>,
    ) -> Result<BreakingChangesAuditSection, AuditError> {
        let mut packages_with_breaking = vec![];
        
        if let Some(changeset) = changeset {
            // Analyze changes with versions
            let report = self.changes_analyzer.analyze_with_versions(
                "main",
                "HEAD",
                changeset,
            ).await?;
            
            // Check each package for breaking changes
            for pkg in report.packages_with_changes() {
                let mut breaking_changes = vec![];
                
                // Check conventional commits
                if self.config.breaking_changes.check_conventional_commits {
                    for commit in &pkg.commits {
                        if commit.message.contains("BREAKING CHANGE") || 
                           commit.message.contains('!') {
                            breaking_changes.push(BreakingChange {
                                description: commit.message.clone(),
                                commit_hash: Some(commit.hash.clone()),
                                source: BreakingChangeSource::ConventionalCommit,
                            });
                        }
                    }
                }
                
                if !breaking_changes.is_empty() {
                    packages_with_breaking.push(PackageBreakingChanges {
                        package_name: pkg.package_info.name().to_string(),
                        current_version: pkg.current_version.clone(),
                        next_version: pkg.next_version.clone(),
                        breaking_changes,
                    });
                }
            }
        }
        
        let total = packages_with_breaking.iter()
            .map(|p| p.breaking_changes.len())
            .sum();
        
        Ok(BreakingChangesAuditSection {
            packages_with_breaking,
            total_breaking_changes: total,
            issues: vec![],
        })
    }
}
```

---

### Error Handling

```rust
/// Errors that can occur during audit operations
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Upgrade error: {0}")]
    Upgrade(#[from] UpgradeError),

    #[error("Changes error: {0}")]
    Changes(#[from] ChangesError),

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Audit section disabled: {0}")]
    SectionDisabled(String),
}
```

---

### Module Structure

```text
crates/pkg/src/audit/
├── mod.rs                      # Public API exports
├── manager.rs                  # AuditManager implementation
├── report.rs                   # AuditReport and related types
├── sections/
│   ├── mod.rs                  # Section exports
│   ├── upgrades.rs             # UpgradeAuditSection
│   ├── dependencies.rs         # DependencyAuditSection
│   ├── breaking_changes.rs     # BreakingChangesAuditSection
│   ├── categorization.rs       # DependencyCategorization
│   └── version_consistency.rs  # VersionConsistencyAuditSection
├── issue.rs                    # AuditIssue types
├── formatter.rs                # Report formatting (markdown, JSON)
├── health_score.rs             # Health score calculation
├── config.rs                   # Configuration types
└── error.rs                    # Error types
```

---

### Dependencies

No new external dependencies. Reuses all existing crates:

```toml
[dependencies]
# Internal crates (already available)
sublime_git_tools = { path = "../git" }
sublime_standard_tools = { path = "../standard" }

# Already available
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
```

---

### Design Principles

1. **Aggregation**: Combines data from all modules into one report
2. **Actionable**: Provides clear issues and suggestions
3. **Severity-based**: Critical, Warning, Info for prioritization
4. **Configurable**: Control what to audit and thresholds
5. **Health score**: Single metric for overall repository health
6. **Export-friendly**: Markdown and JSON formats
7. **CI/CD ready**: Exit codes and clear pass/fail criteria
8. **Non-invasive**: Read-only operations, no modifications

---

### Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_single_package() {
        // Test audit in single-package project
    }

    #[tokio::test]
    async fn test_audit_monorepo() {
        // Test audit in monorepo
    }

    #[tokio::test]
    async fn test_detect_circular_dependencies() {
        // Test circular dependency detection
    }

    #[tokio::test]
    async fn test_categorize_dependencies() {
        // Test dependency categorization
    }

    #[tokio::test]
    async fn test_version_consistency() {
        // Test version consistency checks
    }

    #[tokio::test]
    async fn test_health_score_calculation() {
        // Test health score calculation
    }

    #[tokio::test]
    async fn test_markdown_export() {
        // Test markdown report generation
    }

    #[tokio::test]
    async fn test_json_export() {
        // Test JSON report generation
    }
}
```

---

## Integration with sublime_standard_tools

This crate heavily leverages `sublime_standard_tools` for:

### FileSystem Operations
- `AsyncFileSystem` trait - All file I/O
- `FileSystemManager` - Robust, cross-platform file operations
- Read/write package.json files
- Atomic writes, proper error handling

### Monorepo Detection
- `MonorepoDetector` - Detect monorepo vs single-package
- `MonorepoDetectorTrait` - API for detection
- `WorkspacePackage` - Package metadata in monorepos
- Automatic discovery of all packages

### Package Metadata
- `package-json` crate (already used by standard) - Parse package.json
- `PackageJson` struct with all fields (dependencies, devDependencies, peerDependencies, etc)

### Example Integration

```rust
use sublime_standard_tools::{
    filesystem::{AsyncFileSystem, FileSystemManager},
    monorepo::{MonorepoDetector, MonorepoDetectorTrait, WorkspacePackage},
};
use package_json::PackageJson;

// Detect project type
let fs = FileSystemManager::new();
let detector = MonorepoDetector::new();

if let Some(monorepo_kind) = detector.is_monorepo_root(root).await? {
    // Monorepo - detect all packages
    let monorepo = detector.detect_monorepo(root).await?;
    for package in monorepo.packages() {
        // Process each package
        let pkg_path = package.absolute_path.join("package.json");
        let content = fs.read_file_string(&pkg_path).await?;
        let pkg_json: PackageJson = serde_json::from_str(&content)?;
        // ...
    }
} else {
    // Single package
    let pkg_path = root.join("package.json");
    let content = fs.read_file_string(&pkg_path).await?;
    let pkg_json: PackageJson = serde_json::from_str(&content)?;
    // ...
}
```

---

## Next Steps

1. ✅ Validate this simplified concept
2. ⏳ Implement core data structures
3. ⏳ Implement storage layer
4. ⏳ Implement manager APIs
5. ⏳ Implement Git integration (add_commits_from_git)
6. ⏳ Implement version resolution (with dependency propagation)
7. ⏳ Implement snapshot version generation
8. ⏳ Implement apply versions (with dry-run and package.json writing)
9. ⏳ Implement monorepo/single-package detection
10. ⏳ Implement history query API
11. ⏳ Implement dependency upgrade detection
12. ⏳ Implement registry client with .npmrc support
13. ⏳ Implement upgrade apply with rollback
14. ⏳ Implement automatic changeset creation for upgrades
15. ⏳ Implement changes analyzer
16. ⏳ Implement file-to-package mapping
17. ⏳ Implement commit-to-package association
18. ⏳ Implement version preview calculation
19. ⏳ Implement changelog generator
20. ⏳ Implement conventional commits parser
21. ⏳ Implement changelog formatters (Keep a Changelog, Conventional, Custom)
22. ⏳ Implement existing changelog parser
23. ⏳ Implement audit manager
24. ⏳ Implement audit sections (upgrades, dependencies, breaking changes, categorization, consistency)
25. ⏳ Implement health score calculation
26. ⏳ Implement report formatters (markdown, JSON)
27. ⏳ Write comprehensive tests
28. ⏳ Document all public APIs

---

**Document Status**: ✅ DONE - All modules specified and ready for implementation

---

## Document Review Summary

### ✅ Modules Completed (6/6)

1. **Changesets** - Core changeset management with storage, history, and Git integration
2. **Versioning & Dependency Propagation** - Version resolution, propagation rules, snapshot versions, dry-run support
3. **Dependency Upgrades** - Registry integration, upgrade detection, dry-run, rollback, automatic changeset creation
4. **Changes Analysis** - Git-based change detection, file-to-package mapping, commit association, version preview
5. **Changelog Generation** - Conventional commits, Keep a Changelog, custom templates, monorepo support
6. **Audit & Health Checks** - Repository health analysis, dependency categorization, issue detection with severity

### ✅ Cross-Cutting Concerns

- **Configuration**: Global configuration with TOML/YAML/JSON support, environment variable overrides
- **Git Integration**: Full integration with `sublime_git_tools` for all git operations
- **FileSystem**: Leverages `sublime_standard_tools` for all file operations
- **Monorepo Support**: All modules handle single-package and monorepo seamlessly
- **Dry-Run**: Preview support across all mutation operations
- **Error Handling**: Comprehensive error types with context
- **Testing**: Test strategies defined for all modules

### ✅ Key Design Principles Maintained

1. **Library-only**: No CLI, no interactive prompts - pure API
2. **Git-based**: All change detection via git, not file scanning
3. **Configurable**: Extensive configuration options with sensible defaults
4. **Testable**: All components independently testable
5. **Documented**: Complete API documentation with examples
6. **Reusable**: Maximum reuse of `sublime_git_tools` and `sublime_standard_tools`
7. **Atomic**: Operations with rollback support where needed
8. **Monorepo-aware**: First-class support for both single and monorepo

### ✅ Integration Points Verified

- ✅ All modules integrate with `sublime_git_tools` for git operations
- ✅ All modules use `sublime_standard_tools` for filesystem and monorepo detection
- ✅ Configuration system extends standard config seamlessly
- ✅ Modules compose together (Audit uses all other modules)
- ✅ Data flows between modules (Changeset → Version Resolution → Changelog → Audit)

### ✅ Documentation Complete

- ✅ Overview and philosophy for each module
- ✅ Complete data structures with documentation
- ✅ API signatures with examples
- ✅ Configuration schemas (TOML)
- ✅ Usage examples (simple to complex)
- ✅ Integration examples between modules
- ✅ Error handling strategies
- ✅ Module structure and file organization
- ✅ Dependencies list
- ✅ Design principles
- ✅ Testing strategies

### 📋 Ready for Next Phase

This document is **complete and ready** for the implementation planning phase. All concepts are:

- **Well-defined**: Clear responsibilities and boundaries
- **Integrated**: Modules work together cohesively
- **Practical**: Real-world usage examples provided
- **Testable**: Testing approaches defined
- **Maintainable**: Simple, consistent patterns throughout

### 🎯 What's Next

The next iteration should produce a **PLAN.md** that defines:

1. **Implementation phases**: Order of development
2. **Dependencies**: Which modules depend on others
3. **Milestones**: Measurable completion criteria
4. **Testing strategy**: Unit, integration, and E2E tests
5. **Documentation plan**: API docs, guides, examples
6. **Success criteria**: How to know when done

---

**CONCEPT.md STATUS**: ✅ **DONE** - Approved for implementation planning