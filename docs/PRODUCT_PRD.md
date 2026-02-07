# Product Requirements Document: workspace-node-tools v2

## Document Information

| Field | Value |
|-------|-------|
| **Product Name** | `workspace-node-tools` |
| **Version** | `2.0.0` |
| **Status** | Ready |
| **Created** | 2026-02-07 |
| **Last Updated** | 2026-02-07 |
| **Architecture** | Library-first (Rust crates + NAPI bridge + Bun+Ink CLI) |
| **MSRV** | Rust 1.90+, Edition 2024 |

---

## 1. Product Vision & Goals

### 1.1 What

A Rust library ecosystem for JavaScript/TypeScript workspace management -- changeset-based versioning, monorepo detection, changelog generation, dependency upgrades, and health auditing.

### 1.2 Why

The library-first architecture enables multiple consumers from a single codebase:

| Consumer | Technology | How |
|----------|------------|-----|
| **CLI** | Bun + Ink (TypeScript) | Interactive terminal UI via NAPI bindings |
| **NAPI bindings** | napi-rs (Rust cdylib) | Programmatic Node.js/Bun/Deno usage |
| **Rust crates** | Direct library dependency | Native Rust consumption |
| **Future: WASM** | wasm-pack | Browser-based tooling |

### 1.3 Target Users

- **Monorepo maintainers** -- managing multi-package JavaScript/TypeScript repositories
- **CI/CD systems** -- automated versioning, changelog generation, and publishing
- **Package authors** -- tracking changes, auditing health, managing upgrades

### 1.4 Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Product model | Library-first | Rust crates = library. CLI = Bun+Ink (TS). NAPI = bridge |
| Async model | Async-first (tokio) | All I/O async |
| Async traits | Native (Rust 2024 edition) | Drop `async-trait` crate |
| Crate granularity | Fine-grained (9 Rust crates) | Each concern standalone |
| Git library | `git2` (libgit2) | Battle-tested, same as old product |
| Changeset storage | Git-backed (worktree + orphan branch) | Eliminates JSON merge conflicts |
| Configuration format | TOML only | Simpler, Rust ecosystem standard |
| workspace-core scope | Detection only | Config loading deferred to `workspace-config` |
| Error handling | `snafu` per-crate | Each crate owns its Error enum |
| CLI framework | Bun + Ink (TypeScript) | NOT a Rust crate. Lives in `packages/workspace-cli/` |
| CLI package | `@websublime/workspace-cli` | Binary: `workspace` |
| Removed commands | `changes`, `execute` | `changes` redundant; `execute` not core (turborepo/nx) |
| Enhanced commands | `status` | Richer workspace information |

---

## 2. Architecture Overview

### 2.1 System Architecture

```
+-------------------------------------------------------------+
|  Consumers (TypeScript / packages/)                         |
|  +-----------------+  +------------------------------+      |
|  |  CLI (Bun+Ink)  |  |  @websublime/workspace-tools |      |
|  |  packages/      |  |  packages/workspace-tools/   |      |
|  |  workspace-cli/ |  |                              |      |
|  +--------+--------+  +--------------+---------------+      |
|           |                          |                       |
|           +----------+---------------+                       |
|                      v                                       |
|  +--------------------------------------------------+       |
|  |  workspace-napi (crates/napi/)                   |       |
|  |  Rust cdylib -- napi-rs bridge                   |       |
|  +----------------------+---------------------------+       |
+-------------------------+-------------------------------+
                          v
+-------------------------------------------------------------+
|  Rust Library Crates (crates/)                              |
|                                                             |
|  Layer 3 -- Features:                                       |
|  +------------+ +---------+ +----------+ +---------+        |
|  | changeset  | | version | |changelog | | upgrade |        |
|  +-----+------+ +----+----+ +----+-----+ +----+----+        |
|        |             |           |             |             |
|  +-----+-------------+-----------+-------------+-----+      |
|  |                    audit                          |      |
|  +----------------------+---------------------------+      |
|                         |                                    |
|  Layer 2 -- Operations: |                                    |
|  +----------------------+---------------------------+      |
|  |  workspace-git (git2/libgit2)                    |      |
|  +----------------------+---------------------------+      |
|                         |                                    |
|  Layer 1 -- Core:       |                                    |
|  +----------+  +--------+-----+                              |
|  |  config  |  |    core      |                              |
|  +----+-----+  +------+-------+                              |
|       |               |                                      |
|  Layer 0 -- Foundation:|                                     |
|  +---------------------+----------------------------+       |
|  |  workspace-fs (tokio::fs + mock)                 |       |
|  +--------------------------------------------------+       |
+-------------------------------------------------------------+
```

### 2.2 Layer Summary

| Layer | Name | Crates | Purpose |
|-------|------|--------|---------|
| 0 | Foundation | workspace-fs | Async filesystem abstraction |
| 1 | Core | workspace-core, workspace-config | Detection + configuration |
| 2 | Operations | workspace-git | Git operations via git2 |
| 3 | Features | workspace-changeset, workspace-version, workspace-changelog, workspace-upgrade, workspace-audit | Business logic |
| Bridge | NAPI | workspace-napi | cdylib for Node.js/Bun/Deno |
| Consumers | TypeScript | packages/workspace-tools, packages/workspace-cli | npm package + CLI |

---

## 3. Crate Architecture (9 Rust Library Crates + 1 NAPI Bridge)

### 3.1 Crate Registry

```
Layer 0 -- Foundation:
  workspace-fs           crates/filesystem/   Async filesystem abstraction

Layer 1 -- Core:
  workspace-core         crates/core/         Detection: repo type, PM, monorepo, packages
  workspace-config       crates/config/       TOML config loading, per-crate config sections

Layer 2 -- Operations:
  workspace-git          crates/git/          Git operations via git2 (repo, commits, tags, diff, worktrees)

Layer 3 -- Features:
  workspace-changeset    crates/changeset/    Git-backed changeset management
  workspace-version      crates/version/      Version resolution + dependency propagation
  workspace-changelog    crates/changelog/    Changelog generation (conventional commits)
  workspace-upgrade      crates/upgrade/      Dependency upgrade detection + application
  workspace-audit        crates/audit/        Workspace health checks + scoring

Bridge:
  workspace-napi         crates/napi/         NAPI bindings (cdylib, excluded from workspace)
```

### 3.2 Dependency Graph

```
workspace-fs
  ^
workspace-core -----------------------------------------------+
  ^                                                           |
workspace-config                                              |
  ^                                                           |
workspace-git <-- workspace-fs, workspace-core, git2          |
  ^                                                           |
+-- workspace-changeset <-- core, config, git                 |
+-- workspace-version   <-- core, config                      |
+-- workspace-changelog <-- core, git                         |
+-- workspace-upgrade   <-- core, config (+ reqwest)          |
+-- workspace-audit     <-- core, git, version, upgrade       |
                                                              |
workspace-napi <-- ALL feature crates + napi-rs --------------+
```

**Invariant**: No dependency cycles. Each layer depends only on layers below it.

### 3.3 Old-to-New Crate Mapping

| Old Crate | Old Path | New Crate(s) | What Migrates |
|-----------|----------|--------------|---------------|
| `sublime_standard_tools` | `crates/standard/` | **workspace-fs** + **workspace-core** + **workspace-config** | filesystem/ -> workspace-fs; node/ + project/ + monorepo/ -> workspace-core; config/ -> workspace-config |
| `sublime_git_tools` | `crates/git/` | **workspace-git** | Repo, commits, tags, diff, status, push/pull |
| `sublime_pkg_tools` | `crates/pkg/` | **workspace-changeset** + **workspace-version** + **workspace-changelog** + **workspace-upgrade** + **workspace-audit** | changeset/ -> workspace-changeset; version/ -> workspace-version; changelog/ -> workspace-changelog; upgrade/ -> workspace-upgrade; audit/ -> workspace-audit |
| `sublime_cli_tools` | `crates/cli/` | **packages/workspace-cli** (TypeScript) | All CLI commands migrated to Bun+Ink |
| `sublime_node_tools` | `crates/node/` | **workspace-napi** | All NAPI bindings |

---

## 4. Dependency Graph (Detailed)

### 4.1 External Dependencies Per Crate

| Crate | External Dependencies |
|-------|-----------------------|
| **workspace-fs** | `snafu`, `tokio` (fs, sync), `log` |
| **workspace-core** | `snafu`, `serde`, `serde_json`, `serde_yaml_ng`, `log`, `semver`, `glob` |
| **workspace-config** | `snafu`, `serde`, `toml`, `log` |
| **workspace-git** | `snafu`, `git2`, `log` |
| **workspace-changeset** | `snafu`, `serde`, `serde_json`, `chrono`, `log` |
| **workspace-version** | `snafu`, `semver`, `log` |
| **workspace-changelog** | `snafu`, `chrono`, `log` |
| **workspace-upgrade** | `snafu`, `semver`, `reqwest`, `serde_json`, `log` |
| **workspace-audit** | `snafu`, `log` |
| **workspace-napi** | `napi`, `napi-derive`, `tokio` |

### 4.2 Development Dependencies (All Crates)

| Crate | Dev Dependencies |
|-------|-----------------|
| `tempfile` | Integration tests with real filesystem |
| `tokio` (rt-multi-thread, macros) | Async test runtime |

---

## 5. Per-Crate Functional Requirements

### 5.1 workspace-fs (Layer 0 -- Foundation)

**Crate**: `workspace-fs`
**Path**: `crates/filesystem/`
**Depends on**: Nothing (foundational)
**Old source**: `crates/standard/src/filesystem/`
**Detailed PRD**: [`crates/filesystem/PRD.md`](../crates/filesystem/PRD.md)

#### Key Types

| Type | Purpose |
|------|---------|
| `FileSystem` | Async trait defining all filesystem operations |
| `RealFileSystem` | Production implementation using `tokio::fs` |
| `MockFileSystem` | In-memory implementation for testing |
| `DirEntry` | Directory entry abstraction |
| `Metadata` | File metadata (size, type, permissions) |
| `FileType` | Enum: `File`, `Dir`, `Symlink` |
| `PathExt` | Trait extending `std::path::Path` with `normalize()` |
| `FileSystemConfig` | Timeout settings with builder pattern |
| `Error` | Unified error enum with path context |

#### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| FS-FR-1 | `FileSystem` trait SHALL define async methods for read, write, metadata, directory, file, path, symlink, and traversal operations | P0 |
| FS-FR-2 | `FileSystem` trait SHALL use native async fn (edition 2024, no `async-trait` crate) | P0 |
| FS-FR-3 | `FileSystem` trait SHALL be `Send + Sync` | P0 |
| FS-FR-4 | Assess `dyn FileSystem` compatibility with native async traits; use `impl FileSystem` pattern if object safety is not achievable | P0 |
| FS-FR-5 | `RealFileSystem` SHALL implement all `FileSystem` methods using `tokio::fs` | P0 |
| FS-FR-6 | `RealFileSystem` SHALL respect configurable timeouts (read, write, operation) | P0 |
| FS-FR-7 | `MockFileSystem` SHALL store files in `HashMap<PathBuf, Vec<u8>>` with `tokio::sync::RwLock` | P0 |
| FS-FR-8 | `MockFileSystem` SHALL NOT enforce timeouts | P0 |
| FS-FR-9 | `walk_dir` SHALL traverse recursively, follow symlinks, sort entries deterministically, exclude root | P0 |
| FS-FR-10 | All errors SHALL include the path and operation that failed | P0 |
| FS-FR-11 | `PathExt::normalize()` SHALL resolve `.` and `..` without I/O | P0 |

#### Revision Notes (from existing PRD)

- **Drop `async-trait` dependency**: Use native async fn in trait definitions (Rust edition 2024)
- **`traits.rs`**: Replace `#[async_trait]` with direct `async fn` in trait definition
- **Object safety**: Native async traits may not support `dyn FileSystem`. Assess and document whether to use `impl FileSystem` generics or boxing strategy. If `dyn` is not feasible, prefer `impl FileSystem` pattern and update consuming crates accordingly.
- **Rest of PRD is solid**: All functional requirements, error variants, config builder, mock setup methods remain valid.

---

### 5.2 workspace-core (Layer 1 -- Core)

**Crate**: `workspace-core`
**Path**: `crates/core/`
**Depends on**: `workspace-fs`
**Old source**: `crates/standard/src/node/` + `crates/standard/src/monorepo/` + `crates/standard/src/project/` + `crates/pkg/src/types/`
**Detailed PRD**: [`crates/core/PRD.md`](../crates/core/PRD.md)

#### Key Types

| Type | Purpose |
|------|---------|
| `RepoType` | Enum: `Node`, `Deno`, `Bun` |
| `RepoKind` | Enum: `Simple`, `Monorepo` |
| `PackageManagerKind` | Enum: `Npm`, `Yarn`, `Pnpm`, `Bun`, `Deno` |
| `Package` | Unified package representation (name, version, paths, manifest, dependencies) |
| `Dependency` | Single dependency entry (name, version spec, internal flag) |
| `PackageDependencies` | Categorized dependencies (deps, dev, peer, optional) |
| `Project` | Unified project (root path, repo type, PM, repo kind, packages) |
| `DetectionConfig` | Configurable detection behavior with builder pattern |

#### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| CORE-FR-1 | SHALL detect `RepoType` by checking files in priority order: Deno (`deno.json`/`deno.jsonc`) > Bun (`bunfig.toml`/`bun.lockb`) > Node (`package.json`) | P0 |
| CORE-FR-2 | SHALL detect `PackageManagerKind` via: (1) `packageManager` field in `package.json`, (2) lock file presence, (3) optional env var, (4) configurable fallback | P0 |
| CORE-FR-3 | SHALL detect `RepoKind` by workspace config: `workspaces` in `package.json`, `pnpm-workspace.yaml`, or `workspace`/`workspaces` in `deno.json` | P0 |
| CORE-FR-4 | SHALL discover all workspace packages matching workspace glob patterns | P0 |
| CORE-FR-5 | SHALL parse and categorize dependencies (production, dev, peer, optional) distinguishing internal (workspace) vs external | P0 |
| CORE-FR-6 | SHALL detect workspace protocol specifiers (`workspace:*`, `workspace:^`, `workspace:~`) | P0 |
| CORE-FR-7 | `Project` SHALL provide `dependents_of(package_name)` returning all packages that depend on a given package | P0 |
| CORE-FR-8 | SHALL find project root from any subdirectory by walking up the directory tree | P0 |
| CORE-FR-9 | All path parameters SHALL be explicit `&Path` -- NO fallback to current directory | P0 |
| CORE-FR-10 | `DetectionConfig` SHALL support builder pattern with defaults for detection order, exclusion patterns, search depth | P0 |
| CORE-FR-11 | SHALL report error if `packageManager` field conflicts with detected lock file | P0 |

#### Revision Notes (from existing PRD)

- **Switch from sync-first to async-first**: The existing PRD specifies sync-first detection. Since `workspace-fs` is async-first, detection APIs SHALL be async.
- **Remove configuration loading responsibility**: Config loading is delegated to `workspace-config`. This crate only receives `DetectionConfig` programmatically.
- **Keep package/dependency representation**: `Package`, `Dependency`, `PackageDependencies` types remain in this crate.
- **Evaluate `package-json` crate**: Check if the `package-json` crate (v0.5.0) is still maintained. If not, implement a minimal `PackageJson` parser using `serde_json`.
- **Old PM kinds simplified**: The old product had `Npm`, `Yarn`, `YarnBerry`, `Pnpm`, `Bun`, `Lerna`, `Nx`, `Turbo`, `Rush`. The new product simplifies to `Npm`, `Yarn`, `Pnpm`, `Bun`, `Deno` (Lerna/Nx/Turbo/Rush are meta-tools, not package managers).

---

### 5.3 workspace-config (Layer 1 -- Core)

**Crate**: `workspace-config`
**Path**: `crates/config/`
**Depends on**: `workspace-fs`, `workspace-core`
**Old source**: `crates/standard/src/config/` + `crates/pkg/src/config/`

#### Key Types

| Type | Purpose |
|------|---------|
| `WorkspaceConfig` | Top-level config container, loaded from `repo.config.toml` |
| `ChangesetConfig` | Changeset behavior (environments, storage mode) |
| `VersionConfig` | Versioning strategy (independent/unified), snapshot format |
| `DependencyConfig` | Propagation rules (which dep types, max depth, circular behavior) |
| `UpgradeConfig` | Upgrade behavior (auto-changeset, registry settings) |
| `ChangelogConfig` | Changelog format, links, repository URL |
| `AuditConfig` | Audit sections, severity thresholds |

#### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| CFG-FR-1 | SHALL load configuration from `repo.config.toml` at workspace root | P0 |
| CFG-FR-2 | SHALL parse TOML into strongly-typed config structs | P0 |
| CFG-FR-3 | SHALL validate configuration (required fields, valid values, cross-field constraints) | P0 |
| CFG-FR-4 | SHALL provide sensible defaults for all optional fields | P0 |
| CFG-FR-5 | SHALL distribute config sections to per-crate config types (e.g., `ChangesetConfig` for workspace-changeset) | P0 |
| CFG-FR-6 | Config format SHALL be TOML only (JSON and YAML removed from old product) | P0 |
| CFG-FR-7 | SHALL support environment variable overrides with configurable prefix (default: `WNT_`) | P1 |

#### Configuration Schema

```toml
# repo.config.toml

[changeset]
available_environments = ["development", "staging", "production"]
default_environments = ["production"]

[version]
strategy = "independent"  # "independent" | "unified"
default_bump = "patch"
snapshot_format = "{version}-{branch}.{timestamp}"

[version.dependency]
propagation_bump = "patch"
propagate_dependencies = true
propagate_dev_dependencies = false
propagate_peer_dependencies = true
max_depth = 10
fail_on_circular = true

[upgrade]
auto_changeset = true
changeset_bump = "patch"

[upgrade.registry]
url = "https://registry.npmjs.org"
timeout_secs = 30
retry_attempts = 3

[changelog]
enabled = true
format = "keep-a-changelog"  # "keep-a-changelog" | "conventional-commits"
include_commit_links = true
repository_url = ""

[audit]
enabled = true
min_severity = "info"  # "critical" | "high" | "medium" | "low" | "info"
```

---

### 5.4 workspace-git (Layer 2 -- Operations)

**Crate**: `workspace-git`
**Path**: `crates/git/`
**Depends on**: `workspace-fs`, `workspace-core`, `git2`
**Old source**: `crates/git/`

#### Key Types

| Type | Purpose |
|------|---------|
| `Repo` | Repository handle wrapping `git2::Repository` |
| `RepoCommit` | Commit info (hash, author, date, message) |
| `RepoTags` | Tag info (hash, tag name) |
| `GitChangedFile` | Changed file with path and status |
| `GitFileStatus` | Enum: `Added`, `Modified`, `Deleted`, `Untracked` |
| `GitDiffStats` | Diff statistics (lines added/deleted) |

#### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| GIT-FR-1 | SHALL provide repository open, create, and clone operations | P0 |
| GIT-FR-2 | SHALL provide branch operations: create, checkout, list, current, delete | P0 |
| GIT-FR-3 | SHALL provide commit operations: stage, commit, get history since ref | P0 |
| GIT-FR-4 | SHALL provide change detection: changed files since SHA, since branch, staged, unstaged, working directory | P0 |
| GIT-FR-5 | SHALL provide tag operations: create annotated tag, list tags, get last tag | P0 |
| GIT-FR-6 | SHALL provide remote operations: push, push tags, pull | P0 |
| GIT-FR-7 | SHALL provide status operations: is clean, has staged changes | P0 |
| GIT-FR-8 | SHALL provide diff statistics (lines added/deleted) | P0 |
| GIT-FR-9 | SHALL provide worktree operations: add, remove, list (required for git-backed changesets) | P0 |
| GIT-FR-10 | SHALL provide orphan branch operations: create orphan branch, checkout orphan (required for `_changesets` branch) | P0 |
| GIT-FR-11 | All operations SHALL use `git2` (libgit2) | P0 |
| GIT-FR-12 | Change detection SHALL differentiate staged vs unstaged changes | P0 |
| GIT-FR-13 | Change detection SHALL support package-specific filtering | P0 |

#### Old API Surface (to migrate)

From the old `sublime_git_tools`:

| Operation Group | Methods |
|-----------------|---------|
| Repository | `create`, `open`, `clone` |
| Branch | `create_branch`, `checkout`, `list_branches`, `current_branch`, `delete_branch` |
| Commit | `add`, `add_all`, `commit`, `commit_changes`, `get_commits_since` |
| Changes | `get_all_files_changed_since_sha`, `get_all_files_changed_since_sha_with_status`, `get_all_files_changed_since_branch`, `get_changed_files`, `get_staged_files`, `get_unstaged_files` |
| Tags | `create_tag`, `get_last_tag`, `list_tags` |
| Remote | `push`, `push_tags`, `pull` |
| Status | `get_status`, `is_clean`, `has_staged_changes` |

#### New Operations (not in old product)

| Operation | Purpose |
|-----------|---------|
| `worktree_add` | Add a git worktree (for changeset storage) |
| `worktree_remove` | Remove a git worktree |
| `worktree_list` | List all worktrees |
| `create_orphan_branch` | Create an orphan branch (for `_changesets`) |

---

### 5.5 workspace-changeset (Layer 3 -- Features)

**Crate**: `workspace-changeset`
**Path**: `crates/changeset/`
**Depends on**: `workspace-core`, `workspace-config`, `workspace-git`
**Old source**: `crates/pkg/src/changeset/`, `crates/pkg/src/types/changeset.rs`

#### Key Types

| Type | Purpose |
|------|---------|
| `Changeset` | Core changeset: branch, bump, environments, packages, changes, timestamps |
| `ArchivedChangeset` | Released changeset with `ReleaseInfo` |
| `ReleaseInfo` | Release metadata: timestamp, applier, git commit, version map |
| `ChangesetManager` | High-level operations (create, update, list, show, delete, archive, history) |
| `GitWorktreeStorage` | Git-backed storage using worktree + orphan branch `_changesets` |
| `UpdateSummary` | Result of adding commits to a changeset |
| `ArchiveResult` | Result of archiving a changeset |

#### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| CS-FR-1 | SHALL store changesets as JSON files in a git worktree backed by orphan branch `_changesets` | P0 |
| CS-FR-2 | `create` SHALL create a new changeset with branch, bump type, environments, packages, and commit references | P0 |
| CS-FR-3 | `update` SHALL add commits, packages, or modify bump type of an existing changeset | P0 |
| CS-FR-4 | `list` SHALL return all pending changesets with filtering (by package, bump, environment) and sorting (date, bump, branch) | P0 |
| CS-FR-5 | `show` SHALL return full details of a specific changeset by branch name or ID | P0 |
| CS-FR-6 | `delete` SHALL remove a pending changeset | P0 |
| CS-FR-7 | `archive` SHALL move a changeset from `pending/` to `archived/` with `ReleaseInfo`, committing to `_changesets` branch | P0 |
| CS-FR-8 | `history` SHALL query archived changesets with filtering (package, env, bump, date range) and limiting | P0 |
| CS-FR-9 | `check` SHALL verify if a changeset exists for a given branch (for git hooks) | P0 |
| CS-FR-10 | All storage operations SHALL commit to the `_changesets` orphan branch | P0 |
| CS-FR-11 | Branch names SHALL be sanitized for use as filenames (replace `/\:*?"<>\|` with `-`) | P0 |
| CS-FR-12 | SHALL provide initialization: create orphan branch, add worktree, create `pending/` and `archived/` dirs | P0 |
| CS-FR-13 | SHALL provide migration tool to convert old file-based `.changesets/` to git-backed storage | P1 |

#### Git-Backed Changeset Architecture

```
Initialization:
  git checkout --orphan _changesets
  git reset --hard
  mkdir -p pending archived
  git add . && git commit -m "Initialize changeset storage"
  git checkout <original-branch>
  git worktree add .changesets _changesets

Runtime structure:
  .changesets/              <-- worktree (branch: _changesets)
  +-- pending/
  |   +-- feat-oauth.json   <-- active changeset
  +-- archived/
      +-- 2026-01-15-v1.2.0.json  <-- released changeset

Operations:
  - Create: write JSON to pending/, commit to _changesets
  - Update: modify JSON, commit
  - Archive: move pending -> archived, commit
  - History: git log on _changesets branch
  - Sync: git push origin _changesets
```

**Benefits over old file-based approach**:

| Aspect | Old (file-based) | New (git-backed) |
|--------|-------------------|-------------------|
| Merge conflicts | JSON conflicts in main branch | No conflicts -- separate branch |
| Audit trail | File timestamps only | Full `git log _changesets` |
| Visibility | Hidden in working tree | Branch visible in GitHub/GitLab UI |
| Sync | Manual copy/commit | Native `git push origin _changesets` |
| Parallel work | Conflict-prone | Isolated by branch |

#### Changeset Data Format

**Pending changeset** (`pending/feat-oauth.json`):
```json
{
  "branch": "feature/oauth-integration",
  "bump": "Minor",
  "environments": ["production", "staging"],
  "packages": ["@myorg/auth", "@myorg/core"],
  "changes": ["abc123def456", "789ghijklm"],
  "created_at": "2026-01-15T10:30:45Z",
  "updated_at": "2026-01-15T14:22:10Z"
}
```

**Archived changeset** (`archived/2026-01-15-v2.0.0.json`):
```json
{
  "changeset": {
    "branch": "feature/oauth",
    "bump": "Minor",
    "environments": ["production"],
    "packages": ["@myorg/auth"],
    "changes": ["abc123"],
    "created_at": "2026-01-15T10:30:45Z",
    "updated_at": "2026-01-15T14:22:10Z"
  },
  "release_info": {
    "applied_at": "2026-01-15T15:00:00Z",
    "applied_by": "ci-bot@example.com",
    "git_commit": "def456ghi789",
    "versions": {
      "@myorg/auth": "2.0.0",
      "@myorg/core": "1.5.0"
    }
  }
}
```

---

### 5.6 workspace-version (Layer 3 -- Features)

**Crate**: `workspace-version`
**Path**: `crates/version/`
**Depends on**: `workspace-core`, `workspace-config`
**Old source**: `crates/pkg/src/version/`, `crates/pkg/src/types/version.rs`

#### Key Types

| Type | Purpose |
|------|---------|
| `VersionResolver` | Resolves version bumps across packages respecting strategy |
| `VersionResolution` | Resolution result: per-package version changes |
| `PackageUpdate` | Single package version change (from -> to) |
| `DependencyGraph` | Internal dependency graph for propagation |
| `VersionBump` | Enum: `Major`, `Minor`, `Patch`, `None` |
| `VersioningStrategy` | Enum: `Independent` (per-package), `Unified` (all same version) |

#### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| VER-FR-1 | SHALL resolve version bumps in `Independent` mode: each package bumped independently per changeset | P0 |
| VER-FR-2 | SHALL resolve version bumps in `Unified` mode: all packages share a single version, highest bump wins | P0 |
| VER-FR-3 | `resolve` SHALL accept pending changesets and return a `VersionResolution` with per-package updates | P0 |
| VER-FR-4 | SHALL propagate version bumps through internal dependency graph (configurable: deps, devDeps, peerDeps) | P0 |
| VER-FR-5 | SHALL detect and report circular dependencies during propagation | P0 |
| VER-FR-6 | `apply` SHALL write resolved versions to `package.json` files and update internal dependency ranges | P0 |
| VER-FR-7 | `preview` SHALL return a dry-run resolution without writing changes | P0 |
| VER-FR-8 | `snapshot` SHALL generate snapshot versions using configurable format template: `{version}`, `{branch}`, `{short_commit}`, `{commit}`, `{timestamp}` | P0 |
| VER-FR-9 | SHALL support prerelease versions: create, increment, promote modes | P0 |
| VER-FR-10 | Propagation depth SHALL be configurable with a maximum limit (default: 10) | P0 |

#### Old CLI Parameters (to expose via API)

From the old `bump` command:

| Parameter | Maps to API |
|-----------|-------------|
| `--dry-run` | `preview()` |
| `--execute` | `apply()` |
| `--snapshot` | `snapshot()` |
| `--snapshot-format` | `SnapshotConfig.format` |
| `--prerelease TAG` | `PrereleaseConfig` |
| `--packages LIST` | `resolve()` package filter |
| `--git-tag` | Post-apply: `workspace-git` tag creation |
| `--git-push` | Post-apply: `workspace-git` push |
| `--git-commit` | Post-apply: `workspace-git` commit |
| `--no-changelog` | Caller skips changelog generation |
| `--no-archive` / `--always-archive` | `ArchivePolicy` |
| `--show-diff` | `preview()` with diff output |

---

### 5.7 workspace-changelog (Layer 3 -- Features)

**Crate**: `workspace-changelog`
**Path**: `crates/changelog/`
**Depends on**: `workspace-core`, `workspace-git`
**Old source**: `crates/pkg/src/changelog/`

#### Key Types

| Type | Purpose |
|------|---------|
| `ChangelogGenerator` | Generates changelog content from commits/changesets |
| `ConventionalCommit` | Parsed conventional commit (type, scope, description, breaking) |
| `ChangelogEntry` | Single changelog entry (version, date, changes by category) |
| `ChangelogFormat` | Enum: `KeepAChangelog`, `ConventionalCommits`, `Custom(template)` |

#### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| CL-FR-1 | `generate` SHALL produce changelog content from git commits since a given reference | P0 |
| CL-FR-2 | `generate_from_changeset` SHALL produce changelog content from archived changesets | P0 |
| CL-FR-3 | SHALL parse conventional commit messages (type, optional scope, description, optional body, optional footers) | P0 |
| CL-FR-4 | SHALL detect breaking changes from `!` suffix or `BREAKING CHANGE:` footer | P0 |
| CL-FR-5 | SHALL support `KeepAChangelog` format: Added, Changed, Deprecated, Removed, Fixed, Security | P0 |
| CL-FR-6 | SHALL support `ConventionalCommits` format: feat, fix, docs, style, refactor, perf, test, chore | P0 |
| CL-FR-7 | SHALL optionally include commit links when `repository_url` is configured | P0 |
| CL-FR-8 | SHALL generate per-package changelogs for monorepos | P0 |
| CL-FR-9 | SHALL append to existing `CHANGELOG.md` (prepend new entries, preserve existing content) | P0 |

---

### 5.8 workspace-upgrade (Layer 3 -- Features)

**Crate**: `workspace-upgrade`
**Path**: `crates/upgrade/`
**Depends on**: `workspace-core`, `workspace-config`
**Old source**: `crates/pkg/src/upgrade/`

#### Key Types

| Type | Purpose |
|------|---------|
| `UpgradeManager` | Orchestrates upgrade detection and application |
| `RegistryClient` | Queries npm registry for latest versions |
| `BackupManager` | Creates/restores package.json backups before upgrades |
| `UpgradePreview` | Preview of available upgrades (current -> latest, bump type) |
| `UpgradeFilter` | Filter: include/exclude major, minor, patch, dev deps |

#### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| UPG-FR-1 | `check` SHALL query the npm registry and return available upgrades for all packages | P0 |
| UPG-FR-2 | `check` SHALL classify upgrades by bump type (major, minor, patch) | P0 |
| UPG-FR-3 | `check` SHALL support filtering: exclude major, minor, patch, or dev dependencies | P0 |
| UPG-FR-4 | `check` SHALL support package-specific filtering | P0 |
| UPG-FR-5 | `check` SHALL support custom registry URL override | P0 |
| UPG-FR-6 | `apply` SHALL update `package.json` dependency versions | P0 |
| UPG-FR-7 | `apply` SHALL create a backup of `package.json` files before modification | P0 |
| UPG-FR-8 | `apply` SHALL optionally auto-create a changeset for the upgrade | P0 |
| UPG-FR-9 | `apply` SHALL support scope restrictions: `patch-only`, `minor-and-patch` (non-breaking only) | P0 |
| UPG-FR-10 | `rollback` SHALL restore `package.json` files from backup | P0 |
| UPG-FR-11 | Backup management: `backup_list`, `backup_restore(id)`, `backup_clean(keep_n)` | P0 |
| UPG-FR-12 | Registry queries SHALL respect configurable timeout and retry settings | P0 |
| UPG-FR-13 | SHALL include peer dependencies when requested | P1 |

---

### 5.9 workspace-audit (Layer 3 -- Features)

**Crate**: `workspace-audit`
**Path**: `crates/audit/`
**Depends on**: `workspace-core`, `workspace-git`, `workspace-version`, `workspace-upgrade`
**Old source**: `crates/pkg/src/audit/`

#### Key Types

| Type | Purpose |
|------|---------|
| `AuditManager` | Orchestrates all audit sections |
| `AuditReport` | Complete audit results |
| `HealthScore` | Numeric health score (0-100) with breakdown |
| `AuditIssue` | Single issue with severity, category, description, remediation |
| `AuditSection` | Enum: `Upgrades`, `Dependencies`, `VersionConsistency`, `BreakingChanges` |
| `Severity` | Enum: `Critical`, `High`, `Medium`, `Low`, `Info` |

#### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| AUD-FR-1 | `run_audit` SHALL execute configurable audit sections | P0 |
| AUD-FR-2 | **Upgrades section**: detect outdated dependencies with available upgrades | P0 |
| AUD-FR-3 | **Dependencies section**: detect missing, unused, or undeclared dependencies; analyze internal dependency health | P0 |
| AUD-FR-4 | **Version consistency section**: detect version mismatches across workspace packages (same dep at different versions) | P0 |
| AUD-FR-5 | **Breaking changes section**: detect potential breaking changes since last release | P0 |
| AUD-FR-6 | SHALL produce a `HealthScore` (0-100) with per-section breakdowns | P0 |
| AUD-FR-7 | SHALL support minimum severity filtering | P0 |
| AUD-FR-8 | SHALL support configurable verbosity: minimal, normal, detailed | P0 |
| AUD-FR-9 | SHALL support export formats: HTML, Markdown | P1 |

---

### 5.10 workspace-napi (Bridge)

**Crate**: `workspace-napi`
**Path**: `crates/napi/`
**Depends on**: All feature crates + `napi-rs`
**Old source**: `crates/node/`
**Note**: Excluded from Cargo workspace (cdylib)

#### Key Types

| Type | Purpose |
|------|---------|
| `ApiResponse<T>` | Unified response wrapper: `{ success, data?, error? }` |
| `ErrorInfo` | Error object: `{ code, message }` |
| Per-command param structs | napi-rs `#[napi(object)]` input types |
| Per-command response structs | napi-rs `#[napi(object)]` output types |

#### NAPI Functions

All functions return `ApiResponse<T>`. Error codes follow Node.js conventions.

| # | Function | Purpose | Maps to Crate |
|---|----------|---------|---------------|
| 1 | `status` | Workspace status overview | core |
| 2 | `init` | Initialize workspace config | config |
| 3 | `changesetAdd` | Create changeset | changeset |
| 4 | `changesetUpdate` | Update changeset | changeset |
| 5 | `changesetList` | List changesets | changeset |
| 6 | `changesetShow` | Show changeset details | changeset |
| 7 | `changesetRemove` | Delete changeset | changeset |
| 8 | `changesetHistory` | Query history | changeset |
| 9 | `changesetCheck` | Check existence | changeset |
| 10 | `bumpPreview` | Preview version bumps | version |
| 11 | `bumpApply` | Apply version bumps | version |
| 12 | `bumpSnapshot` | Generate snapshot versions | version |
| 13 | `configShow` | Show configuration | config |
| 14 | `configValidate` | Validate configuration | config |
| 15 | `upgradeCheck` | Check for upgrades | upgrade |
| 16 | `upgradeApply` | Apply upgrades | upgrade |
| 17 | `upgradeBackupList` | List backups | upgrade |
| 18 | `upgradeBackupRestore` | Restore backup | upgrade |
| 19 | `upgradeBackupClean` | Clean old backups | upgrade |
| 20 | `auditRun` | Run health audit | audit |
| 21 | `changelogGenerate` | Generate changelog | changelog |
| 22 | `publishPackages` | Publish packages to registry | version, git |
| 23 | `getVersion` | Get library version | -- |

**Removed from old product**: `execute` (not core; use turborepo/nx).

#### Error Codes

| Code | Meaning |
|------|---------|
| `EVALIDATION` | Input validation failed |
| `EGIT` | Git operation error |
| `EIO` | I/O error |
| `ENOENT` | Not found |
| `EEXIST` | Already exists |
| `EPERM` | Permission denied |
| `ECONFIG` | Configuration error |
| `EREGISTRY` | Registry query error |
| `EUNKNOWN` | Unknown/unexpected error |

#### NAPI Build Configuration

```json
{
  "binaryName": "workspace-tools",
  "targets": [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-gnu",
    "aarch64-unknown-linux-musl",
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc"
  ]
}
```

#### Runtime Management

- The NAPI crate SHALL manage a dedicated tokio runtime for async operations
- The runtime SHALL be lazily initialized on first NAPI call
- The runtime SHALL be shared across all NAPI function calls
- The runtime SHALL handle graceful shutdown on Node.js process exit

---

## 6. Cross-Cutting Concerns

### 6.1 Error Handling

| Aspect | Approach |
|--------|----------|
| Framework | `snafu` per-crate |
| Granularity | Each crate owns its own `Error` enum |
| Context | Every error includes path/operation context |
| Chaining | Source errors wrapped, not replaced |
| Display | `#[snafu(display("..."))]` for actionable messages |
| Result alias | `pub type Result<T> = std::result::Result<T, Error>` per crate |
| NAPI mapping | Crate errors map to `ErrorInfo { code, message }` in workspace-napi |

### 6.2 Logging

| Aspect | Approach |
|--------|----------|
| Facade | `log` crate |
| Initialization | Consumer responsibility (CLI uses `tracing-subscriber`) |
| Activation | `RUST_LOG` environment variable |
| Levels | `error` (fatal), `warn` (ambiguous/inconsistent), `info` (high-level ops), `debug` (detection decisions), `trace` (operation entry/exit) |

### 6.3 Configuration

| Aspect | Approach |
|--------|----------|
| Format | TOML only (`repo.config.toml`) |
| Crate | `workspace-config` loads and distributes |
| Validation | At load time, before distribution |
| Env override | `WNT_` prefix (configurable) |
| Per-crate | Each feature crate receives its typed config section |

### 6.4 Async Model

| Aspect | Approach |
|--------|----------|
| Runtime | tokio |
| Traits | Native async fn (Rust edition 2024) -- no `async-trait` crate |
| I/O | All filesystem and network I/O is async |
| Computation | Pure computation remains sync |
| NAPI | Dedicated tokio runtime managed by NAPI layer |

### 6.5 Testing Strategy

| Test Type | Tool | Purpose |
|-----------|------|---------|
| Unit tests | `MockFileSystem` | Fast, deterministic, no disk I/O |
| Integration tests | `tempfile` crate | Real filesystem operations |
| E2E tests | Full stack | CLI -> NAPI -> Rust crates |
| Platform CI | GitHub Actions | Windows, macOS, Linux |

### 6.6 Platform Support

| Platform | Status |
|----------|--------|
| macOS (arm64, x64) | Supported |
| Linux (arm64, x64, gnu, musl) | Supported |
| Windows (arm64, x64) | Supported |

### 6.7 Code Quality Standards

| Standard | Enforcement |
|----------|-------------|
| No `unsafe` code | `#![forbid(unsafe_code)]` |
| No `unwrap()`/`expect()` | `#![deny(clippy::unwrap_used, clippy::expect_used)]` |
| All public items documented | `#![warn(missing_docs)]` |
| Clippy clean | CI gate with deny settings |
| MSRV | Rust 1.90+, Edition 2024 |

---

## 7. Bun + Ink CLI Architecture

### 7.1 Overview

The CLI is NOT a Rust crate. It is a TypeScript project using Bun as runtime and Ink (React for terminals) as the UI framework.

| Field | Value |
|-------|-------|
| **Package name** | `@websublime/workspace-cli` |
| **Binary name** | `workspace` |
| **Runtime** | Bun |
| **UI framework** | Ink (React for terminals) |
| **Location** | `packages/workspace-cli/` |

### 7.2 Package Structure

```
packages/
+-- workspace-tools/     <-- NAPI npm package (@websublime/workspace-tools)
|   +-- package.json
|   +-- src/
|   |   +-- index.ts     <-- re-exports from binding
|   |   +-- binding.js   <-- auto-generated by napi-rs
|   |   +-- binding.d.ts <-- auto-generated TypeScript defs
|   +-- npm/             <-- platform-specific binaries
|       +-- darwin-arm64/
|       +-- linux-x64-gnu/
|       +-- ...
|
+-- workspace-cli/       <-- NEW: Bun + Ink CLI
    +-- package.json     <-- bin: { "workspace": "./dist/index.js" }
    +-- src/
    |   +-- index.tsx    <-- Ink app entry point
    |   +-- commands/    <-- CLI command components
    |   +-- ui/          <-- Shared UI components
    +-- tsconfig.json
```

### 7.3 CLI Command Inventory

#### Global Parameters

| Parameter | Short | Purpose |
|-----------|-------|---------|
| `--root PATH` | `-r` | Workspace root override |
| `--log-level LEVEL` | `-l` | Verbosity (silent/error/warn/info/debug/trace) |
| `--format FORMAT` | `-f` | Output format (human/json/json-compact/quiet) |
| `--no-color` | -- | Disable ANSI colors |
| `--config PATH` | `-c` | Config file path override |
| `--quiet` | `-q` | Alias for `--log-level=silent` |
| `--verbose` | `-v` | Alias for `--log-level=debug` |

#### Commands

| Command | Purpose | NAPI Function | Crate |
|---------|---------|---------------|-------|
| `status` | Enhanced workspace overview | `status` | core, changeset, version |
| `init` | Initialize workspace config | `init` | config |
| `changeset create` | Create changeset | `changesetAdd` | changeset |
| `changeset update` | Update changeset | `changesetUpdate` | changeset |
| `changeset list` | List changesets | `changesetList` | changeset |
| `changeset show` | Show changeset details | `changesetShow` | changeset |
| `changeset edit` | Edit changeset in $EDITOR | `changesetShow` + `changesetUpdate` | changeset |
| `changeset delete` | Delete changeset | `changesetRemove` | changeset |
| `changeset history` | Query archived history | `changesetHistory` | changeset |
| `changeset check` | Check changeset exists (git hooks) | `changesetCheck` | changeset |
| `version preview` | Preview version bumps (dry-run) | `bumpPreview` | version |
| `version apply` | Apply version bumps | `bumpApply` | version |
| `version snapshot` | Generate snapshot versions | `bumpSnapshot` | version |
| `changelog` | Generate changelogs | `changelogGenerate` | changelog |
| `upgrade check` | Check for upgrades | `upgradeCheck` | upgrade |
| `upgrade apply` | Apply upgrades | `upgradeApply` | upgrade |
| `upgrade backups list` | List backups | `upgradeBackupList` | upgrade |
| `upgrade backups restore` | Restore backup | `upgradeBackupRestore` | upgrade |
| `upgrade backups clean` | Clean old backups | `upgradeBackupClean` | upgrade |
| `audit` | Workspace health audit | `auditRun` | audit |
| `publish` | Publish packages to registry | `publishPackages` | version, git |
| `config show` | Display configuration | `configShow` | config |
| `config validate` | Validate configuration | `configValidate` | config |

#### Removed Commands

| Command | Reason |
|---------|--------|
| ~~`changes`~~ | Redundant: changeset workflow + git log cover this use case |
| ~~`execute`~~ | Not core: tools like turborepo/nx handle task orchestration |
| ~~`clone`~~ | Can be done with `git clone` + `workspace init` |

#### Enhanced Commands

| Command | Enhancement |
|---------|-------------|
| `status` | Richer output: packages list, pending changesets, version state, health summary, uncommitted changes, dependency status, recent tags |

#### Subcommand Parameter Review

All subcommand parameters from the old product need review for clarity and consistency during Phase 6 (CLI implementation). Known issues:
- Parameters that were confusing or had poor visibility in the old CLI
- Overlapping or redundant flags across subcommands
- Naming inconsistencies (e.g., `--filter-package` vs `--packages`)

This review happens iteratively during CLI implementation with user feedback.

---

## 8. NAPI Strategy

### 8.1 Pattern

Same pattern as old product:

1. `crates/napi/` -- Rust cdylib with napi-rs (excluded from Cargo workspace due to cdylib)
2. `packages/workspace-tools/` -- npm package with platform-specific binaries
3. Auto-generated TypeScript definitions from `#[napi(object)]` structs
4. `ApiResponse<T>` wrapper for consistent error handling

### 8.2 Platform Targets (8 targets)

| Target | OS | Arch |
|--------|-----|------|
| `aarch64-apple-darwin` | macOS | ARM64 |
| `x86_64-apple-darwin` | macOS | x64 |
| `x86_64-unknown-linux-gnu` | Linux | x64 (glibc) |
| `x86_64-unknown-linux-musl` | Linux | x64 (musl) |
| `aarch64-unknown-linux-gnu` | Linux | ARM64 (glibc) |
| `aarch64-unknown-linux-musl` | Linux | ARM64 (musl) |
| `x86_64-pc-windows-msvc` | Windows | x64 |
| `aarch64-pc-windows-msvc` | Windows | ARM64 |

### 8.3 npm Package Structure

```
packages/workspace-tools/
+-- package.json           <-- @websublime/workspace-tools
+-- src/
|   +-- index.ts           <-- re-exports from binding
|   +-- binding.js         <-- auto-generated by napi-rs
|   +-- binding.d.ts       <-- auto-generated TypeScript defs
+-- npm/                   <-- platform-specific binaries
    +-- darwin-arm64/      <-- @websublime/workspace-tools-darwin-arm64
    +-- darwin-x64/        <-- @websublime/workspace-tools-darwin-x64
    +-- linux-arm64-gnu/   <-- @websublime/workspace-tools-linux-arm64-gnu
    +-- linux-arm64-musl/  <-- @websublime/workspace-tools-linux-arm64-musl
    +-- linux-x64-gnu/     <-- @websublime/workspace-tools-linux-x64-gnu
    +-- linux-x64-musl/    <-- @websublime/workspace-tools-linux-x64-musl
    +-- win32-arm64-msvc/  <-- @websublime/workspace-tools-win32-arm64-msvc
    +-- win32-x64-msvc/    <-- @websublime/workspace-tools-win32-x64-msvc
```

---

## 9. Old Command Mapping (Completeness Verification)

### 9.1 All Old Commands Accounted For

| Old Command | Old Crate | New Location | Status |
|-------------|-----------|--------------|--------|
| `init` | cli | CLI: `init` / NAPI: `init` | Retained |
| `config show` | cli | CLI: `config show` / NAPI: `configShow` | Retained |
| `config validate` | cli | CLI: `config validate` / NAPI: `configValidate` | Retained |
| `changeset create` | cli | CLI: `changeset create` / NAPI: `changesetAdd` | Retained |
| `changeset update` | cli | CLI: `changeset update` / NAPI: `changesetUpdate` | Retained |
| `changeset list` | cli | CLI: `changeset list` / NAPI: `changesetList` | Retained |
| `changeset show` | cli | CLI: `changeset show` / NAPI: `changesetShow` | Retained |
| `changeset edit` | cli | CLI: `changeset edit` | Retained (CLI only) |
| `changeset delete` | cli | CLI: `changeset delete` / NAPI: `changesetRemove` | Retained |
| `changeset history` | cli | CLI: `changeset history` / NAPI: `changesetHistory` | Retained |
| `changeset check` | cli | CLI: `changeset check` / NAPI: `changesetCheck` | Retained |
| `bump` (preview) | cli | CLI: `version preview` / NAPI: `bumpPreview` | Renamed |
| `bump` (apply) | cli | CLI: `version apply` / NAPI: `bumpApply` | Renamed |
| `bump` (snapshot) | cli | CLI: `version snapshot` / NAPI: `bumpSnapshot` | Renamed |
| `upgrade check` | cli | CLI: `upgrade check` / NAPI: `upgradeCheck` | Retained |
| `upgrade apply` | cli | CLI: `upgrade apply` / NAPI: `upgradeApply` | Retained |
| `upgrade backups list` | cli | CLI: `upgrade backups list` / NAPI: `upgradeBackupList` | Retained |
| `upgrade backups restore` | cli | CLI: `upgrade backups restore` / NAPI: `upgradeBackupRestore` | Retained |
| `upgrade backups clean` | cli | CLI: `upgrade backups clean` / NAPI: `upgradeBackupClean` | Retained |
| `audit` | cli | CLI: `audit` / NAPI: `auditRun` | Retained |
| `status` | cli | CLI: `status` / NAPI: `status` | Enhanced |
| `version` (info) | cli | CLI: `--version` flag | Simplified |
| `changes` | cli | -- | **Removed** (redundant) |
| `execute` | cli | -- | **Removed** (not core) |
| `clone` | cli | -- | **Removed** (`git clone` + `init`) |

### 9.2 New Commands (not in old product)

| Command | Purpose |
|---------|---------|
| `changelog` | Standalone changelog generation (was embedded in `bump`) |
| `publish` | Publish packages to registry (was partially in `bump`) |

---

## 10. Old Module Mapping (Completeness Verification)

### 10.1 sublime_standard_tools -> workspace-fs + workspace-core + workspace-config

| Old Module | New Crate | What Migrates |
|------------|-----------|---------------|
| `filesystem/` | **workspace-fs** | FileSystem trait, RealFileSystem, MockFileSystem, DirEntry, Metadata, FileType, PathExt |
| `node/package_managers.rs` | **workspace-core** | PackageManagerKind, lock file detection |
| `node/repo_type.rs` | **workspace-core** | RepoType detection |
| `project/` | **workspace-core** | Project detection, root finding |
| `monorepo/` | **workspace-core** | Monorepo detection, workspace package discovery |
| `config/` | **workspace-config** | Configuration loading and validation |
| `command/` | -- | **Removed** (execute command removed) |

### 10.2 sublime_git_tools -> workspace-git

| Old Module | New Location | What Migrates |
|------------|--------------|---------------|
| `repo.rs` | **workspace-git** | All repository operations |
| `types.rs` | **workspace-git** | Repo, RepoCommit, RepoTags, GitChangedFile, GitFileStatus, GitDiffStats |
| `env.rs` | **workspace-git** | Environment provider abstraction |

### 10.3 sublime_pkg_tools -> 5 Feature Crates

| Old Module | New Crate | What Migrates |
|------------|-----------|---------------|
| `changeset/` | **workspace-changeset** | ChangesetManager, storage (rewritten as git-backed), history |
| `types/changeset.rs` | **workspace-changeset** | Changeset, ArchivedChangeset, ReleaseInfo |
| `version/` | **workspace-version** | VersionResolver, resolution, application |
| `types/version.rs` | **workspace-version** | Version, VersionBump, VersioningStrategy |
| `types/prerelease.rs` | **workspace-version** | PrereleaseMode, PrereleaseConfig |
| `changelog/` | **workspace-changelog** | ChangelogGenerator, ConventionalCommit |
| `upgrade/` | **workspace-upgrade** | UpgradeManager, RegistryClient, BackupManager |
| `audit/` | **workspace-audit** | AuditManager, HealthScore |
| `types/package.rs` | **workspace-core** | PackageInfo, DependencyType |
| `types/dependency.rs` | **workspace-core** | DependencyUpdate, VersionProtocol |
| `config/` | **workspace-config** | Per-feature config sections |
| `changes/` | -- | **Removed** (redundant with changeset + git log) |

### 10.4 sublime_cli_tools -> packages/workspace-cli (TypeScript)

| Old Module | New Location | Notes |
|------------|--------------|-------|
| `cli/` | `packages/workspace-cli/src/` | Bun+Ink command parsing |
| `commands/` | `packages/workspace-cli/src/commands/` | React components per command |
| `output/` | `packages/workspace-cli/src/ui/` | Ink UI components |
| `interactive/` | `packages/workspace-cli/src/ui/` | Ink interactive components |
| `error/` | `packages/workspace-cli/src/` | TypeScript error handling |

### 10.5 sublime_node_tools -> workspace-napi

| Old Module | New Location | Notes |
|------------|--------------|-------|
| `commands/` | `crates/napi/src/commands/` | NAPI function implementations |
| `types/` | `crates/napi/src/types/` | `#[napi(object)]` struct conversions |
| `error.rs` | `crates/napi/src/error.rs` | ErrorInfo + error codes |
| `response.rs` | `crates/napi/src/response.rs` | ApiResponse<T> |

---

## 11. Implementation Roadmap

### 11.1 Phase Overview

| Phase | Crates | Blocked By | Estimated Scope |
|-------|--------|------------|-----------------|
| **Phase 1** | workspace-fs (complete), workspace-core | Nothing | Foundation + detection |
| **Phase 2** | workspace-config, workspace-git | Phase 1 | Configuration + git operations |
| **Phase 3** | workspace-changeset, workspace-version | Phase 2 | Core features |
| **Phase 4** | workspace-changelog, workspace-upgrade, workspace-audit | Phases 2-3 | Remaining features |
| **Phase 5** | workspace-napi + packages/workspace-tools | Phases 1-4 | NAPI bridge + npm package |
| **Phase 6** | packages/workspace-cli (Bun + Ink) | Phase 5 | CLI |

### 11.2 Phase 1: Foundation + Detection

**Crates**: workspace-fs (remaining ~40%), workspace-core

**workspace-fs remaining work**:
- `traits.rs`: FileSystem trait with native async fn (no async-trait)
- `real.rs`: RealFileSystem using tokio::fs
- `mock.rs`: MockFileSystem with HashMap + RwLock
- Integration tests

**workspace-core**:
- Full implementation per PRD (async-first revision)
- RepoType, RepoKind, PackageManagerKind detection
- Package, Dependency, PackageDependencies types
- Project detection and monorepo analysis
- DetectionConfig with builder pattern

### 11.3 Phase 2: Configuration + Git

**Crates**: workspace-config, workspace-git

**workspace-config**:
- TOML loading and validation
- Per-crate config type distribution
- Environment variable overrides

**workspace-git**:
- Full git2 wrapper (migrate from old sublime_git_tools)
- New worktree and orphan branch operations
- Change detection with package filtering

### 11.4 Phase 3: Core Features

**Crates**: workspace-changeset, workspace-version

**workspace-changeset**:
- Git-backed storage implementation (worktree + orphan branch)
- All changeset CRUD operations
- Archive and history
- Migration tool from old file-based format

**workspace-version**:
- Independent and unified resolution strategies
- Dependency propagation
- Snapshot and prerelease support
- Preview and apply operations

### 11.5 Phase 4: Remaining Features

**Crates**: workspace-changelog, workspace-upgrade, workspace-audit

These can be developed in parallel once Phase 2-3 are complete.

### 11.6 Phase 5: NAPI Bridge

**Crates**: workspace-napi
**Packages**: packages/workspace-tools

- 23+ NAPI functions
- TypeScript type generation
- Platform builds (8 targets)
- npm package with optional dependencies

### 11.7 Phase 6: CLI

**Packages**: packages/workspace-cli

- Bun + Ink TypeScript project
- All CLI commands as React components
- Interactive prompts and UI
- Parameter review with user iteration

---

## Appendix A: Native Async Traits Impact Assessment

### A.1 Background

Rust edition 2024 supports native async fn in traits, eliminating the need for the `async-trait` proc macro. This impacts `workspace-fs` and all consuming crates.

### A.2 Impact on workspace-fs

| Aspect | `async-trait` (old) | Native (new) |
|--------|---------------------|--------------|
| Trait definition | `#[async_trait] trait FileSystem { async fn read(...) }` | `trait FileSystem { async fn read(...) }` |
| Object safety | Supported (`dyn FileSystem`) | Not directly supported for async traits |
| Boxing | Implicit (async-trait boxes futures) | Explicit if needed |
| Performance | Heap allocation per call | Zero-cost if using generics |

### A.3 Recommended Approach

1. **Prefer generics**: Use `impl FileSystem` or `fn foo<F: FileSystem>(fs: &F)` throughout the codebase
2. **Type erasure when needed**: If `dyn FileSystem` is required (e.g., storing heterogeneous implementations), use a boxing wrapper or the `trait-variant` crate
3. **Assess during Phase 1**: During workspace-fs implementation, validate that all consuming crate patterns work with the chosen approach
4. **Document the pattern**: Once decided, document in CLAUDE.md for consistency across all crates

### A.4 Consuming Crate Impact

All crates that accept a `FileSystem` parameter need to use the same pattern:

```rust
// Preferred: generics (zero-cost)
pub async fn detect_project<F: FileSystem>(fs: &F, path: &Path) -> Result<Project> { ... }

// Alternative: boxed trait object (if runtime polymorphism needed)
pub async fn detect_project(fs: &dyn FileSystem, path: &Path) -> Result<Project> { ... }
```

---

## Appendix B: Glossary

| Term | Definition |
|------|------------|
| **Changeset** | A structured record of changes to one or more packages, including bump type and affected packages |
| **Monorepo** | Repository containing multiple packages managed together via workspace configuration |
| **Package Manager** | Tool for managing JavaScript dependencies (npm, yarn, pnpm, bun, deno) |
| **Lock File** | File recording exact dependency versions (package-lock.json, yarn.lock, etc.) |
| **Workspace Package** | Individual package within a monorepo workspace |
| **RepoType** | The runtime ecosystem (Node, Deno, Bun) |
| **RepoKind** | The repository structure (Simple, Monorepo) |
| **PackageManagerKind** | The specific package manager tool |
| **NAPI** | Node API -- native addon interface for Node.js |
| **napi-rs** | Rust framework for building NAPI modules |
| **Ink** | React-based framework for building terminal UIs |
| **Orphan Branch** | Git branch with no parent commits, independent history |
| **Worktree** | Additional working directory attached to a git repository |
| **MSRV** | Minimum Supported Rust Version |
| **Edition 2024** | Rust language edition enabling native async traits |

---

## Appendix C: References

### Product References

- [workspace-fs PRD](../crates/filesystem/PRD.md) -- Detailed filesystem crate requirements
- [workspace-core PRD](../crates/core/PRD.md) -- Detailed core crate requirements
- [Old product source](../temp/workspace-tools-main/) -- Legacy implementation

### External References

- [npm Workspaces](https://docs.npmjs.com/cli/v7/using-npm/workspaces)
- [Yarn Workspaces](https://yarnpkg.com/features/workspaces)
- [pnpm Workspaces](https://pnpm.io/workspaces)
- [Bun Workspaces](https://bun.sh/docs/install/workspaces)
- [Deno Workspaces](https://deno.land/manual/workspaces)
- [napi-rs](https://napi.rs/)
- [Ink (React for CLIs)](https://github.com/vadimdemedes/ink)
- [git2-rs](https://docs.rs/git2/)
- [snafu](https://docs.rs/snafu/)
- [tokio](https://tokio.rs/)
