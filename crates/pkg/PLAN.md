# sublime_pkg_tools - Implementation Plan

**Version:** 1.0  
**Last Updated:** 2024-01-15  
**Status:** Planning Phase

---

## Table of Contents

- [Design Decisions](#design-decisions)
- [Overview](#overview)
- [Goals and Objectives](#goals-and-objectives)
- [Core Concepts](#core-concepts)
- [Architecture](#architecture)
- [Module Structure](#module-structure)
- [Key Features](#key-features)
- [Integration with Existing Crates](#integration-with-existing-crates)
- [Configuration System](#configuration-system)
- [Implementation Phases](#implementation-phases)
- [Data Structures](#data-structures)
- [Workflows](#workflows)
- [Examples](#examples)
- [Testing Strategy](#testing-strategy)
- [Documentation Requirements](#documentation-requirements)

---

## Design Decisions

This section documents key design decisions made during planning to keep the implementation simple and maintainable.

### 1. Simplified Changeset Structure

**Decision**: Use a simple array for releases instead of complex HashMap with tracking.

**Rationale**:
- Easier to understand and maintain
- Reduces complexity in initial implementation
- Can be extended later if needed
- All packages released to all specified environments (no per-environment package selection)

**Impact**:
```json
// Simple array
"releases": ["dev", "qa"]

// Instead of complex structure with tracking
"releases": {
  "dev": { "packages": [...], "released": false, "released_at": null }
}
```

### 2. Filename as Identity

**Decision**: Use `{branch}-{datetime}.json` format for changeset filenames instead of separate ID field.

**Rationale**:
- Self-documenting: branch and timestamp visible in filename
- Naturally unique (datetime ensures uniqueness)
- Sortable by creation time
- Git-friendly (no special characters)
- One less field to manage in JSON

**Impact**:
- Filename: `.changesets/feat-user-auth-20240115T103000Z.json`
- No separate `id` field in JSON
- Load by branch finds latest changeset for that branch

### 3. Snapshot Versions Never Persisted

**Decision**: Snapshot versions are calculated dynamically at runtime, never written to package.json. **Snapshot versions are ALWAYS used on non-main branches**, regardless of commit type.

**Rationale**:
- Avoids polluting Git history with version changes
- Prevents merge conflicts on version field
- Snapshot versions are ephemeral by nature
- Allows multiple pushes/deploys to dev environments with unique versions
- Only release versions (after merge to main) should be committed
- Each push to a branch gets a unique snapshot version based on commit SHA

**Impact**:
- `VersionResolver` calculates versions on-the-fly
- package.json only contains release versions
- Snapshot format: `1.2.3-abc123d.snapshot`
- On branches: ALWAYS snapshot (even for `feat:` or `fix:` commits)
- On main/master: Apply changeset bump (minor, patch, major)

### 4. Configuration via Standard Config System

**Decision**: Use `repo.config.{toml,yml,yaml,json}` from sublime_standard_tools instead of separate config files.

**Rationale**:
- Consistency across all sublime tools
- Reuse existing configuration infrastructure
- Single source of configuration
- Environment variable overrides with `SUBLIME_PACKAGE_TOOLS_` prefix

**Impact**:
- Configuration nested under `[package_tools]` key
- No separate config files needed
- Leverages standard config loading and merging

### 5. Version Bump Strategy

**Decision**: 
- **On branches**: ALWAYS snapshot version (regardless of commit type)
- **On merge to main**: Apply changeset bump based on conventional commits
- **Dependency updates**: Default to patch bump

**Rationale**:
- Development needs unique versions per push for continuous testing
- Snapshot allows multiple deploys to dev environments
- Actual version bumps only happen on merge to main
- Dependencies getting updated should at minimum bump patch

**Impact**:
- On `feat/user-auth` branch: `1.2.3-abc123d.snapshot` (always)
- On merge to main with `feat:` commits → minor bump (1.2.3 → 1.3.0)
- On merge to main with `fix:` commits → patch bump (1.2.3 → 1.2.4)
- Package depending on updated package → patch bump (minimum)

### 6. Environment Configuration

**Decision**: Available environments configured in `repo.config.toml`, changesets reference them by name.

**Rationale**:
- Single source of truth for valid environments
- Prevents typos in changeset files
- Easy to add/remove environments globally
- Validation happens at config level

**Impact**:
```toml
[package_tools.changeset]
available_environments = ["dev", "test", "qa", "staging", "prod"]
default_environments = ["dev"]
```

### 7. Versioning Strategies

**Decision**: Support two versioning strategies - Independent and Unified.

**Rationale**:
- **Independent**: Each package maintains its own version (default for monorepos)
- **Unified**: All packages share the same version (simpler, but less flexible)
- Different projects have different needs
- Configuration-driven selection

**Impact**:
```toml
[package_tools.release]
strategy = "independent"  # or "unified"
```

**Behavior**:
- **Independent**: `pkg-a@1.2.0`, `pkg-b@2.5.1`, `pkg-c@0.3.0`
- **Unified**: `pkg-a@1.2.0`, `pkg-b@1.2.0`, `pkg-c@1.2.0`

### 8. Dry Run Support

**Decision**: Support dry-run mode for all version bump and release operations.

**Rationale**:
- Preview changes before applying them
- Validate changeset without side effects
- Useful for CI/CD pipelines
- Helps catch errors early

**Impact**:
- Shows what would be changed without making changes
- Displays: packages, current versions, next versions, files affected
- No writes to filesystem or Git
- Flag-based: `--dry-run` or programmatic API

### 9. Maximum Reuse of Existing Crates

**Decision**: Leverage sublime_standard_tools and sublime_git_tools for all operations.

**Rationale**:
- Avoid reimplementing functionality
- Consistency with other tools
- Battle-tested implementations
- Reduces maintenance burden

**Impact**:
- `FileSystemManager` for all file operations
- `ConfigManager` for configuration
- `Repo` for all Git operations
- `MonorepoDetector` for project detection
- `CommandExecutor` for shell commands

---

## Overview

`sublime_pkg_tools` is a comprehensive toolkit for Node.js package and version management with changeset support. It provides high-level abstractions for managing packages in both monorepos and single repositories, with a focus on semantic versioning, dependency propagation, and multi-environment releases.

### What

This crate provides:
- **Version Management**: Semantic versioning with snapshot support (calculated at runtime)
- **Changeset System**: Track changes and manage releases across multiple environments
- **Dependency Analysis**: Build dependency graphs, detect circular dependencies, propagate updates
- **Registry Integration**: Interface with npm and enterprise registries
- **Release Management**: Coordinate releases across multiple environments
- **Changelog Generation**: Automatic changelog from conventional commits

### How

The crate integrates deeply with:
- `sublime_git_tools` for Git operations
- `sublime_standard_tools` for filesystem, configuration, and project detection

### Why

Managing versions and releases in monorepos requires coordinated operations across Git, filesystem, and package registries. This crate provides a unified interface that handles all these concerns with proper error handling and cross-platform support.

---

## Goals and Objectives

### Primary Goals

1. **Version Management**
   - Support semantic versioning (major.minor.patch)
   - Calculate snapshot versions dynamically at runtime (never written to package.json)
   - Version bump determination from conventional commits

2. **Changeset Management**
   - Track changes per branch
   - Support multiple release environments (dev, test, qa, staging, prod)
   - Store changesets in `.changesets/` directory as JSON files
   - Track release status per environment

3. **Dependency Propagation**
   - Automatically detect dependent packages
   - Propagate version updates through dependency chain
   - Support both dependencies and devDependencies
   - Maintain topological order for updates

4. **Multi-Environment Releases**
   - Release same changeset to different environments
   - Track release status per environment
   - Support independent package versioning
   - Create environment-specific Git tags

5. **Conventional Commits**
   - Parse conventional commit messages
   - Determine version bumps from commit types
   - Generate changelogs automatically
   - Default to snapshot bump for non-conventional commits

6. **Enterprise Support**
   - Support private/enterprise npm registries
   - Token and basic authentication
   - .npmrc integration

### Non-Goals

- **Automatic testing/linting**: Use only validations from `sublime_standard_tools`
- **Build orchestration**: This is handled by other tools
- **Deployment**: Only handles versioning and registry publishing

---

## Core Concepts

### 1. Snapshot Versions

**Definition**: A snapshot version is a dynamically calculated version that includes the base version, commit hash, and a snapshot marker. **Always used on non-main branches**.

**Format**: `{version}-{commit}.snapshot` (e.g., `1.2.3-abc123d.snapshot`)

**Key Points**:
- **Never written to package.json**
- Calculated at runtime based on current commit
- **ALWAYS used on non-main branches** (feat/, bugfix/, etc.)
- Allows multiple pushes with unique versions for continuous testing
- Each push to branch gets unique snapshot: `1.2.3-abc123d.snapshot`, then `1.2.3-def456.snapshot`
- Actual version bumps only applied on merge to main

**Example - On Branch**:
```
package.json has: "version": "1.2.3"
Current commit: abc123def456
Current branch: feat/user-auth

Resolved version: 1.2.3-abc123d.snapshot

After new commit: def456ghi789
Resolved version: 1.2.3-def456g.snapshot
```

**Example - After Merge to Main**:
```
On main branch after merge:
- Changeset specifies: minor bump
- package.json updated: "1.2.3" → "1.3.0"
- Resolved version: 1.3.0 (no snapshot)
```

### 2. Changesets

**Definition**: A changeset tracks all changes in a branch and defines how packages should be released to different environments.

**Structure**:
```json
{
  "branch": "feat/user-auth",
  "created_at": "2024-01-15T10:30:00Z",
  "author": "developer@example.com",
  "releases": ["dev", "qa"],
  "packages": [
    {
      "name": "@myorg/auth-service",
      "bump": "minor",
      "current_version": "1.2.3",
      "next_version": "1.3.0",
      "reason": "direct_changes",
      "changes": [
        {
          "type": "feat",
          "description": "Add OAuth2 authentication",
          "breaking": false,
          "commit": "abc123def456"
        },
        {
          "type": "feat",
          "description": "Add JWT token support",
          "breaking": false,
          "commit": "def456ghi789"
        }
      ]
    },
    {
      "name": "@myorg/user-service",
      "bump": "patch",
      "current_version": "2.1.0",
      "next_version": "2.1.1",
      "reason": "dependency_update",
      "dependency": "@myorg/auth-service",
      "changes": [
        {
          "type": "chore",
          "description": "Update auth-service dependency",
          "breaking": false,
          "commit": "auto-generated"
        }
      ]
    }
  ]
}
```

**Filename**: `.changesets/feat-user-auth-20240115T103000Z.json`

**Key Points**:
- Stored in `.changesets/` directory as `{branch}-{datetime}.json`
- Filename format: Replace `/` in branch with `-`, append ISO datetime
- Example: `feat-user-auth-20240115T103000Z.json`
- One changeset per branch (latest overwrites previous)
- Simple array of target environments: `["dev", "qa"]`
- Environments must be defined in `repo.config.toml` under `available_environments`
- All packages will be released to all specified environments

### 7. Versioning Strategies

**Definition**: Two strategies for managing versions across packages in a monorepo.

#### Independent Strategy (Default)

Each package maintains its own version independently.

**Example**:
```
Before:
- @myorg/auth-service: 1.2.3
- @myorg/user-service: 2.5.1
- @myorg/api-gateway: 0.8.0

After changes and release:
- @myorg/auth-service: 1.3.0 (minor bump from feat)
- @myorg/user-service: 2.5.2 (patch bump from dependency)
- @myorg/api-gateway: 0.8.0 (no changes)
```

**Use Cases**:
- Packages have different release cycles
- Independent evolution of packages
- Different maturity levels (v0.x vs v2.x)

#### Unified Strategy

All packages share the same version and are bumped together.

**Example**:
```
Before (all at):
- @myorg/auth-service: 1.2.3
- @myorg/user-service: 1.2.3
- @myorg/api-gateway: 1.2.3

After changes and release (all bumped to):
- @myorg/auth-service: 1.3.0
- @myorg/user-service: 1.3.0
- @myorg/api-gateway: 1.3.0
```

**Use Cases**:
- Tightly coupled packages
- Simpler version management
- All packages released together
- Single version number for entire project

**Configuration**:
```toml
[package_tools.release]
strategy = "independent"  # or "unified"
```

### 8. Dry Run Mode

**Definition**: Preview mode that shows what would happen without making any changes.

**Capabilities**:
- Show which packages would be updated
- Display current and next versions
- List files that would be modified
- Show Git tags that would be created
- Display commands that would be executed
- No writes to filesystem, Git, or registry

**Example Output**:
```
🔍 DRY RUN MODE - No changes will be made

Packages to be updated:
  @myorg/auth-service
    Current: 1.2.3
    Next:    1.3.0
    Reason:  Direct changes (feat: add OAuth2)
    
  @myorg/user-service
    Current: 2.5.1
    Next:    2.5.2
    Reason:  Dependency update (@myorg/auth-service)

Files to be modified:
  - packages/auth-service/package.json
  - packages/user-service/package.json

Git tags to be created:
  - @myorg/auth-service@1.3.0-dev
  - @myorg/user-service@2.5.2-dev

Commands to be executed:
  - npm publish packages/auth-service --tag dev
  - npm publish packages/user-service --tag dev

✅ Dry run complete. Use --execute to apply changes.
```

**Usage**:
```rust
// Programmatic API
let plan = release_mgr.plan_release(&changeset, "dev").await?;
let dry_run_result = release_mgr.dry_run(&plan).await?;

// Display results
println!("{}", dry_run_result.summary());

// Execute if desired
if confirm("Apply changes?") {
    release_mgr.execute(&plan).await?;
}
```

### 9. Changeset Filename Generation

**Definition**: Changesets are stored as JSON files with a specific naming convention based on the branch name and creation timestamp.

**Format**: `{branch}-{datetime}.json`

**Rules**:
- Branch name processing:
  - Replace all `/` with `-` (e.g., `feat/user-auth` → `feat-user-auth`)
  - Keep alphanumeric characters, hyphens, and underscores
  - Remove or replace other special characters
- DateTime format: ISO 8601 format `YYYYMMDDTHHmmssZ`
  - Example: `20240115T103000Z`
- Resulting filename: `feat-user-auth-20240115T103000Z.json`

**Examples**:
```
Branch: feat/user-auth
DateTime: 2024-01-15T10:30:00Z
Filename: feat-user-auth-20240115T103000Z.json

Branch: bugfix/memory-leak
DateTime: 2024-01-15T14:45:30Z
Filename: bugfix-memory-leak-20240115T144530Z.json

Branch: feature/oauth2-support
DateTime: 2024-01-15T09:15:00Z
Filename: feature-oauth2-support-20240115T091500Z.json
```

**Benefits**:
- Unique filenames (datetime ensures uniqueness)
- Sortable by creation time
- Human-readable
- Git-friendly (no spaces or special characters)
- One changeset per branch (latest overwrites if branch name matches)

### 4. Dependency Propagation

**Definition**: When package A changes and package B depends on A, package B must also be updated.

**Rules**:
- Direct changes: Package has commits affecting its files → bump per conventional commits
- Dependency update: Package depends on another package that changed → **patch bump (minimum)**
- DevDependency update: Package has dev dependency on changed package → **patch bump (minimum)**

**Example**:
```
Structure:
- Package A (changed directly with feat: commit)
- Package B (depends on A)
- Package C (depends on B)

Result (on merge to main):
- A: minor bump (1.2.0 → 1.3.0) - direct changes with feat:
- B: patch bump (2.0.0 → 2.0.1) - dependency update
- C: patch bump (1.5.0 → 1.5.1) - transitive dependency update

On branch (before merge):
- A: 1.2.0-abc123d.snapshot
- B: 2.0.0-abc123d.snapshot
- C: 1.5.0-abc123d.snapshot
```

### 5. Multi-Environment Releases

**Definition**: A single changeset can be released to multiple environments independently.

**Flow**:
1. Developer creates changeset with environments: `["dev", "qa"]`
2. Release to dev → packages deployed, tagged as `pkg@1.3.0-dev`
3. Later, release to qa → same packages, tagged as `pkg@1.3.0-qa`

**Key Points**:
- Releases are simple array of environment names
- Environments must be defined in configuration
- All packages in changeset released to specified environments
- Each release creates environment-specific Git tags
- On branches: Deploy snapshot versions to dev environments
- On main: Deploy release versions based on changeset bumps

### 6. Conventional Commits

**Format**: `<type>(<scope>): <description>`

**Types** (applied only on merge to main):
- `feat`: New feature → minor bump
- `fix`: Bug fix → patch bump
- `perf`: Performance improvement → patch bump
- `breaking` or `!`: Breaking change → major bump
- Other types (`docs`, `chore`, etc.) → no bump (or patch if dependencies changed)

**On Branches**: All commits result in snapshot version, regardless of type

**Breaking Changes**: Indicated by `!` after type or `BREAKING CHANGE:` in footer

---

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    sublime_pkg_tools                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   Version    │  │  Changeset   │  │  Dependency  │    │
│  │  Management  │  │  Management  │  │   Analysis   │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   Release    │  │   Registry   │  │  Changelog   │    │
│  │  Management  │  │ Integration  │  │  Generation  │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐                       │
│  │ Conventional │  │   Package    │                       │
│  │   Commits    │  │     JSON     │                       │
│  └──────────────┘  └──────────────┘                       │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│              Integration Layer                              │
├─────────────────────────────────────────────────────────────┤
│  sublime_git_tools        │    sublime_standard_tools       │
│  - Git operations         │    - Filesystem operations      │
│  - Commit history         │    - Configuration management   │
│  - Tags & branches        │    - Project detection          │
│  - File changes           │    - Command execution          │
└─────────────────────────────────────────────────────────────┘
```

### Component Interaction

```
User Request (e.g., "Create changeset")
    ↓
ReleaseManager
    ↓
    ├→ ChangesetManager (load/save changesets)
    │   └→ FileSystemManager (from standard_tools)
    │
    ├→ DependencyAnalyzer (analyze dependencies)
    │   ├→ MonorepoDetector (from standard_tools)
    │   └→ DependencyGraph
    │
    ├→ ConventionalCommitParser (parse commits)
    │   └→ Repo (from git_tools)
    │
    └→ VersionResolver (calculate versions)
        └→ Repo (from git_tools)
```

---

## Module Structure

```
crates/pkg/
├── Cargo.toml
├── README.md
├── SPEC.md
├── PLAN.md (this file)
├── src/
│   ├── lib.rs                      # Main crate entry with re-exports
│   │
│   ├── version/                    # Version management
│   │   ├── mod.rs
│   │   ├── version.rs              # Version type (semantic versioning)
│   │   ├── bump.rs                 # VersionBump enum and logic
│   │   ├── range.rs                # Version ranges (^1.0.0, ~1.0.0)
│   │   ├── snapshot.rs             # SnapshotVersion (dynamic calculation)
│   │   ├── resolver.rs             # VersionResolver (runtime resolution)
│   │   └── parser.rs               # Version parsing utilities
│   │
│   ├── changeset/                  # Changeset management
│   │   ├── mod.rs
│   │   ├── changeset.rs            # Changeset type
│   │   ├── manager.rs              # ChangesetManager (CRUD operations)
│   │   ├── storage.rs              # Storage implementation (JSON files)
│   │   ├── entry.rs                # ChangesetEntry, ChangesetPackage
│   │   ├── filename.rs             # Filename generation ({branch}-{datetime}.json)
│   │   ├── history.rs              # History management (archival, queries)
│   │   └── query.rs                # History query utilities (date, package filters)
│   │
│   ├── dependency/                 # Dependency analysis
│   │   ├── mod.rs
│   │   ├── graph.rs                # DependencyGraph (build & query)
│   │   ├── analyzer.rs             # DependencyAnalyzer
│   │   ├── propagator.rs           # DependencyPropagator (update propagation)
│   │   ├── node.rs                 # DependencyNode
│   │   └── circular.rs             # Circular dependency detection
│   │
│   ├── registry/                   # Package registry integration
│   │   ├── mod.rs
│   │   ├── client.rs               # RegistryClient (HTTP client)
│   │   ├── metadata.rs             # PackageMetadata types
│   │   ├── auth.rs                 # Authentication (token, basic, .npmrc)
│   │   └── publish.rs              # Package publishing
│   │
│   ├── upgrade/                    # Dependency upgrades
│   │   ├── mod.rs
│   │   ├── manager.rs              # UpgradeManager
│   │   ├── strategy.rs             # UpgradeStrategy (latest, compatible, exact)
│   │   ├── plan.rs                 # UpgradePlan
│   │   └── detector.rs             # Outdated dependency detection
│   │
│   ├── release/                    # Release management
│   │   ├── mod.rs
│   │   ├── manager.rs              # ReleaseManager (orchestrates releases)
│   │   ├── plan.rs                 # ReleasePlan
│   │   ├── executor.rs             # ReleaseExecutor (execute release plan)
│   │   └── tagger.rs               # Git tag creation
│   │
│   ├── package/                    # Package.json manipulation
│   │   ├── mod.rs
│   │   ├── package.rs              # Package type
│   │   ├── json.rs                 # PackageJson structure
│   │   ├── editor.rs               # PackageJsonEditor (modify package.json)
│   │   └── validator.rs            # Package validation
│   │
│   ├── changelog/                  # Changelog generation
│   │   ├── mod.rs
│   │   ├── generator.rs            # ChangelogGenerator
│   │   ├── entry.rs                # ChangelogEntry
│   │   ├── formatter.rs            # Markdown formatter
│   │   └── template.rs             # Changelog templates
│   │
│   ├── conventional/               # Conventional commits
│   │   ├── mod.rs
│   │   ├── parser.rs               # ConventionalCommitParser
│   │   ├── commit.rs               # ConventionalCommit type
│   │   └── types.rs                # CommitType enum
│   │
│   ├── config/                     # Configuration (integrated with standard_tools)
│   │   ├── mod.rs
│   │   ├── config.rs               # PackageToolsConfig
│   │   ├── changeset.rs            # ChangesetConfig
│   │   ├── registry.rs             # RegistryConfig
│   │   ├── release.rs              # ReleaseConfig
│   │   ├── version.rs              # VersionConfig
│   │   └── loader.rs               # Config loading utilities
│   │
│   └── errors/                     # Error handling
│       ├── mod.rs
│       └── errors.rs               # Error types and Result aliases
│
├── tests/
│   ├── integration_tests.rs
│   ├── version_tests.rs
│   ├── changeset_tests.rs
│   ├── dependency_tests.rs
│   ├── release_tests.rs
│   └── conventional_tests.rs
│
└── examples/
    ├── 01_basic_version.rs
    ├── 02_changeset_workflow.rs
    ├── 03_dependency_analysis.rs
    ├── 04_release_process.rs
    ├── 05_multi_env_release.rs
    └── 06_changelog_generation.rs
```

---

## Key Features

### 1. Version Management

**Capabilities**:
- Parse semantic versions
- Compare versions
- Bump versions (major, minor, patch)
- Calculate snapshot versions at runtime
- Validate version strings

**Implementation**:
- Use `semver` crate for semantic versioning
- Custom `SnapshotVersion` type
- `VersionResolver` for runtime version calculation

### 2. Changeset Management

**Capabilities**:
- Create changesets from Git changes
- Save/load changesets from `.changesets/` directory with `{branch}-{datetime}.json` format
- Support multiple target environments as simple array
- Specify which environments to release to

**Implementation**:
- JSON storage in `.changesets/` with format: `{branch}-{datetime}.json`
- Uses `FileSystemManager` from `sublime_standard_tools`
- Integration with Git for commit analysis
- Environments configured in `repo.config.toml`

### 3. Dependency Analysis

**Capabilities**:
- Build dependency graph from monorepo
- Detect circular dependencies
- Calculate dependency propagation
- Topological sorting for update order
- Find affected packages from file changes

**Implementation**:
- Uses `petgraph` for graph algorithms
- Integration with `MonorepoDetector` from `sublime_standard_tools`
- Recursive dependency traversal

### 4. Registry Integration

**Capabilities**:
- Fetch package metadata from registries
- Publish packages to registries
- Support npm, enterprise, and custom registries
- Multiple authentication methods (token, basic, .npmrc)
- Version listing and tagging

**Implementation**:
- Uses `reqwest` for HTTP operations
- Parse .npmrc for authentication
- Support for custom registry URLs

### 5. Release Management

**Capabilities**:
- Plan releases across multiple environments
- Execute release plans (bump versions, create tags, publish)
- Multi-environment release workflow
- Create Git tags for releases
- Update package.json files
- Generate changelogs

**Implementation**:
- Orchestrates all components
- Uses `sublime_git_tools` for Git operations
- Transactional release process

### 6. Changelog Generation

**Capabilities**:
- Parse conventional commits
- Group commits by type
- Generate markdown changelogs
- Append to existing CHANGELOG.md
- Support custom templates

**Implementation**:
- Parse commits using conventional commit format
- Markdown formatting
- Template-based generation

### 7. Conventional Commits

**Capabilities**:
- Parse conventional commit messages
- Extract type, scope, description
- Detect breaking changes
- Determine version bumps

**Implementation**:
- Regex-based parsing
- Support for multiple conventional commit formats
- Breaking change detection from `!` or footer

---

## Integration with Existing Crates

### sublime_git_tools Integration

**Used For**:
- Getting current branch
- Fetching commit history
- Getting current commit SHA
- Detecting changed files
- Creating Git tags
- Getting branch information

**Example**:
```rust
use sublime_git_tools::Repo;

let repo = Repo::open("./")?;
let current_branch = repo.get_current_branch()?;
let commits = repo.get_commits_since(Some("main".to_string()), &None)?;
let changed_files = repo.get_all_files_changed_since_branch(&paths, "main")?;
```

### sublime_standard_tools Integration

**Used For**:
- Filesystem operations (read/write files)
- Configuration management
- Project and monorepo detection
- Command execution (npm publish, etc.)
- Path utilities

**Example**:
```rust
use sublime_standard_tools::{
    FileSystemManager,
    ConfigBuilder,
    MonorepoDetector,
    DefaultCommandExecutor,
};

// Filesystem
let fs = FileSystemManager::new();
let content = fs.read_to_string("package.json").await?;

// Config
let config: PackageToolsConfig = ConfigBuilder::new()
    .with_defaults(PackageToolsConfig::default())
    .with_file_optional("repo.config.toml")
    .with_env_prefix("SUBLIME_PACKAGE_TOOLS")
    .build()
    .await?
    .load()
    .await?;

// Monorepo detection
let detector = MonorepoDetector::new();
let monorepo = detector.detect_monorepo("./").await?;

// Command execution
let executor = DefaultCommandExecutor::new();
let result = executor.execute(Command::new("npm").arg("publish")).await?;
```

---

## Configuration System

### Configuration Files

Configuration is loaded from the standard locations (in order of precedence):

1. `repo.config.toml` (project root) - **highest priority**
2. `repo.config.yml` (project root)
3. `repo.config.yaml` (project root)
4. `repo.config.json` (project root)
5. `~/.config/sublime/config.toml` (user config)
6. Environment variables with `SUBLIME_PACKAGE_TOOLS_` prefix

### Configuration Structure

```toml
[package_tools]

[package_tools.changeset]
    path = ".changesets"
    history_path = ".changesets/history"
    available_environments = ["dev", "test", "qa", "staging", "prod"]
    default_environments = ["dev"]
    filename_format = "{branch}-{datetime}.json"

[package_tools.version]
snapshot_format = "{version}-{commit}.snapshot"
commit_hash_length = 7
allow_snapshot_on_main = false

[package_tools.registry]
url = "https://registry.npmjs.org"
timeout = 30
retry_attempts = 3
use_npmrc = true

[package_tools.registry.registries.enterprise]
url = "https://npm.company.com"
auth_type = "token"

[package_tools.release]
# Versioning strategy: "independent" or "unified"
strategy = "independent"

# Git tag format
tag_format = "{package}@{version}"
env_tag_format = "{package}@{version}-{env}"
create_tags = true
push_tags = true

# Changelog
create_changelog = true
changelog_file = "CHANGELOG.md"

# Commit message format
commit_message = "chore(release): {package}@{version}"

# Dry run by default
dry_run_by_default = false

[package_tools.conventional]
# Bump types for conventional commits (applied on merge to main)
[package_tools.conventional.types]
feat = "minor"
fix = "patch"
perf = "patch"
breaking = "major"

# Note: On branches, all commits result in snapshot versions

[package_tools.dependency]
propagate_updates = true
propagate_dev_dependencies = false
max_propagation_depth = 0
detect_circular = true
fail_on_circular = false

# Default bump for dependency updates
dependency_update_bump = "patch"

[package_tools.changelog]
include_commit_hash = true
include_authors = false
group_by_type = true
include_date = true

[package_tools.upgrade]
default_strategy = "compatible"
check_outdated = true
include_prerelease = false
interactive = true
```

### Environment Variables

```bash
# Changeset
export SUBLIME_PACKAGE_TOOLS_CHANGESET_PATH=".changesets"
export SUBLIME_PACKAGE_TOOLS_CHANGESET_AVAILABLE_ENVIRONMENTS="dev,test,qa,staging,prod"
export SUBLIME_PACKAGE_TOOLS_CHANGESET_DEFAULT_ENVIRONMENTS="dev"
export SUBLIME_PACKAGE_TOOLS_CHANGESET_FILENAME_FORMAT="{branch}-{datetime}.json"

# Version
export SUBLIME_PACKAGE_TOOLS_VERSION_SNAPSHOT_FORMAT="{version}-{commit}.snapshot"
export SUBLIME_PACKAGE_TOOLS_VERSION_COMMIT_HASH_LENGTH="7"

# Registry
export SUBLIME_PACKAGE_TOOLS_REGISTRY_URL="https://registry.npmjs.org"
export SUBLIME_PACKAGE_TOOLS_REGISTRY_TIMEOUT="30"

# Release
export SUBLIME_PACKAGE_TOOLS_RELEASE_STRATEGY="independent"
export SUBLIME_PACKAGE_TOOLS_RELEASE_CREATE_TAGS="true"
export SUBLIME_PACKAGE_TOOLS_RELEASE_DRY_RUN_BY_DEFAULT="false"

# Dependency
export SUBLIME_PACKAGE_TOOLS_DEPENDENCY_PROPAGATE_UPDATES="true"
export SUBLIME_PACKAGE_TOOLS_DEPENDENCY_UPDATE_BUMP="patch"
```

---

## Implementation Phases

### Phase 1: Foundation (Weeks 1-2)

**Objective**: Set up core infrastructure and basic types

**Tasks**:
1. Create crate structure
2. Set up Cargo.toml with dependencies
3. Implement error types (`errors` module)
4. Implement configuration system (`config` module)
5. Implement basic version types (`version` module)
   - `Version` struct
   - `VersionBump` enum
   - `SnapshotVersion` struct
   - Version parsing and comparison

**Deliverables**:
- Functional error handling system
- Configuration loading from repo.config files
- Basic version types with tests
- Documentation for core types

**Success Criteria**:
- 100% Clippy compliance
- 100% test coverage for implemented modules
- All functions documented with examples

### Phase 2: Core Functionality (Weeks 3-4)

**Objective**: Implement core package and dependency management

**Tasks**:
1. Implement `package` module
   - `Package` and `PackageJson` types
   - `PackageJsonEditor` for modifications
   - Package validation
2. Implement `conventional` module
   - `ConventionalCommitParser`
   - `CommitType` enum
   - Breaking change detection
3. Implement `dependency` module
   - `DependencyGraph` construction
   - `DependencyAnalyzer`
   - Circular dependency detection
4. Integrate with `sublime_standard_tools` for filesystem ops
5. Integrate with `sublime_git_tools` for commit parsing

**Deliverables**:
- Package.json parsing and editing
- Conventional commit parsing
- Dependency graph construction
- Circular dependency detection

**Success Criteria**:
- Can parse and modify package.json files
- Can parse conventional commits from Git history
- Can build dependency graph from monorepo
- Can detect circular dependencies

### Phase 3: Changeset System (Weeks 5-6)

**Objective**: Implement changeset management with multi-environment support and history tracking

**Tasks**:
1. Implement `changeset` module
   - `Changeset` type with simple releases array and optional `release_info`
   - `ChangesetManager` for CRUD operations
   - `ChangesetStorage` for JSON persistence
   - Filename generation: `{branch}-{datetime}.json`
   - History management (archival and queries)
   - Query utilities for filtering by date, package, environment
2. Implement `VersionResolver` for dynamic snapshot calculation
3. Implement `DependencyPropagator` for update propagation
4. Integration with Git for change detection

**Deliverables**:
- Changeset creation from Git changes
- Save/load changesets from `.changesets/` with `{branch}-{datetime}.json` format
- Simple array-based environment targeting
- Dependency propagation logic
- Snapshot version calculation
- Changeset archival to `.changesets/history/` with release metadata
- History query APIs (by date, package, environment)

**Success Criteria**:
- Can create changeset for current branch
- Can save/load changesets as JSON with proper filename format
- Can specify target environments as simple array
- Can calculate which packages need updates
- Snapshot versions calculated without writing to disk
- Can archive applied changesets to history with release metadata
- Can query historical changesets by various criteria

### Phase 4: Release Management (Weeks 7-8)

**Objective**: Implement release orchestration and execution with history tracking

**Tasks**:
1. Implement `release` module
   - `ReleaseManager` orchestration
   - `ReleasePlan` creation
   - `ReleaseExecutor` for execution
   - Git tag creation
   - Integration with changeset archival
2. Implement `registry` module
   - `RegistryClient` for HTTP operations
   - Authentication handling
   - Package publishing
   - Metadata fetching
3. Multi-environment release workflow
4. Automatic changeset archival after successful release
   - Add `release_info` metadata
   - Move to `.changesets/history/`
   - Track per-environment release details

**Deliverables**:
- Complete release workflow
- Git tag creation with environment markers
- Package publishing to registries
- Multi-environment release support
- Automatic changeset archival with complete metadata

**Success Criteria**:
- Can plan and execute releases
- Can publish to npm registries
- Can create environment-specific Git tags
- Can release same changeset to multiple environments
- Automatically archives changesets after release with complete metadata
- History maintains audit trail of all releases

### Phase 5: Changelog and Extras (Weeks 9-10)

**Objective**: Implement changelog generation and optional features

**Tasks**:
1. Implement `changelog` module
   - `ChangelogGenerator`
   - Markdown formatting
   - Template support
2. Implement `upgrade` module (optional)
   - `UpgradeManager`
   - `UpgradeStrategy` types
   - Outdated dependency detection
3. Polish and optimization
4. Comprehensive examples
5. Complete documentation

**Deliverables**:
- Automatic changelog generation
- Dependency upgrade support (optional)
- Complete examples for all workflows
- Full API documentation
- README with usage guide

**Success Criteria**:
- Can generate changelog from conventional commits
- Can detect and upgrade outdated dependencies
- All examples run successfully
- Documentation complete with examples

### Phase 6: Testing and Refinement (Weeks 11-12)

**Objective**: Comprehensive testing and refinement

**Tasks**:
1. Write integration tests
2. Write unit tests for all modules
3. Performance testing
4. Cross-platform testing (Windows, macOS, Linux)
5. Edge case handling
6. Documentation review
7. Code review and cleanup

**Deliverables**:
- 100% test coverage
- Integration test suite
- Performance benchmarks
- Cross-platform verification
- Final documentation

**Success Criteria**:
- All tests pass on all platforms
- 100% Clippy compliance
- 100% test coverage
- Zero warnings
- Documentation reviewed and complete

---

## Data Structures

### Core Types

#### Version Types

```rust
/// Semantic version (major.minor.patch)
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
    pre_release: Option<String>,
    build_metadata: Option<String>,
}

/// Snapshot version (calculated at runtime)
pub struct SnapshotVersion {
    base_version: Version,
    commit_id: String,
}

/// Version bump types
pub enum VersionBump {
    Major,
    Minor,
    Patch,
    /// No bump (used internally)
    None,
}

/// Note: Snapshot versions are handled by SnapshotVersion type,
/// not as a VersionBump variant, since they're always used on branches

/// Resolved version (either release or snapshot)
pub enum ResolvedVersion {
    Release(Version),
    Snapshot(SnapshotVersion),
}
```

#### Changeset Types

```rust
/// Complete changeset with multi-environment support
pub struct Changeset {
    pub branch: String,
    pub created_at: String,
    pub author: String,
    pub releases: Vec<String>,
    pub packages: Vec<ChangesetPackage>,
    pub release_info: Option<ReleaseInfo>,
}

/// Release information added when changeset is applied
pub struct ReleaseInfo {
    pub applied_at: String,
    pub applied_by: String,
    pub git_commit: String,
    pub environments_released: HashMap<String, EnvironmentRelease>,
}

pub struct EnvironmentRelease {
    pub released_at: String,
    pub tag: String,
}

/// Package entry in changeset
pub struct ChangesetPackage {
    pub name: String,
    pub bump: VersionBump,
    pub current_version: String,
    pub next_version: String,
    pub reason: ChangeReason,
    pub dependency: Option<String>,
    pub changes: Vec<ChangeEntry>,
}

/// Individual change entry
pub struct ChangeEntry {
    pub change_type: String,
    pub description: String,
    pub breaking: bool,
    pub commit: String,
}

/// Reason for package inclusion
pub enum ChangeReason {
    DirectChanges,
    DependencyUpdate,
    DevDependencyUpdate,
}
```

#### Dependency Types

```rust
/// Dependency graph
pub struct DependencyGraph {
    graph: DiGraph<DependencyNode, EdgeType>,
    package_index: HashMap<String, NodeIndex>,
}

/// Dependency node
pub struct DependencyNode {
    pub name: String,
    pub version: Version,
    pub path: PathBuf,
}

/// Propagated update information
pub struct PropagatedUpdate {
    pub package_name: String,
    pub reason: PropagationReason,
    pub suggested_bump: VersionBump,
}

/// Reason for propagation
pub enum PropagationReason {
    DirectChanges { commits: Vec<ConventionalCommit> },
    DependencyUpdate { dependency: String, old_version: Version, new_version: Version },
    DevDependencyUpdate { dependency: String, old_version: Version, new_version: Version },
}
```

#### Conventional Commit Types

```rust
/// Parsed conventional commit
pub struct ConventionalCommit {
    pub commit_type: CommitType,
    pub scope: Option<String>,
    pub breaking: bool,
    pub description: String,
    pub body: Option<String>,
    pub footer: Option<String>,
    pub hash: String,
    pub author: String,
    pub date: String,
}

/// Commit types
pub enum CommitType {
    Feat,
    Fix,
    Breaking,
    Docs,
    Style,
    Refactor,
    Perf,
    Test,
    Build,
    Ci,
    Chore,
    Revert,
}
```

#### Release Types

```rust
/// Release plan
pub struct ReleasePlan {
    pub changeset_id: String,
    pub environment: String,
    pub packages: Vec<PackageRelease>,
    pub version_tag: String,
    pub create_tags: bool,
    pub push_tags: bool,
    pub create_changelog: bool,
    pub strategy: ReleaseStrategy,
}

/// Package release information
pub struct PackageRelease {
    pub name: String,
    pub current_version: Version,
    pub next_version: Version,
    pub path: PathBuf,
    pub publish: bool,
    pub reason: PropagationReason,
}

/// Release strategy
pub enum ReleaseStrategy {
    /// Each package has independent version
    Independent,
    /// All packages share same version
    Unified,
}

/// Dry run result
pub struct DryRunResult {
    pub packages: Vec<PackageRelease>,
    pub files_to_modify: Vec<PathBuf>,
    pub tags_to_create: Vec<String>,
    pub commands: Vec<String>,
    pub summary: String,
}
```

---

## Workflows

### Workflow 1: Create Changeset

```
1. Developer creates feature branch
   └→ git checkout -b feat/user-auth

2. Developer makes changes and commits
   └→ git commit -m "feat: add OAuth2 authentication"
   └→ git commit -m "fix: resolve memory leak"

1. Create changeset
   └→ Call: ChangesetManager::create_from_git()
   └→ Parse conventional commits
   └→ Detect changed files
   └→ Build dependency graph
   └→ Calculate propagation
   └→ Determine version bumps
   └→ Create changeset with releases: ["dev", "qa"]

4. Save changeset
   └→ Call: ChangesetManager::save()
   └→ Write to .changesets/feat-user-auth-20240115T103000Z.json

Result: Changeset created with packages and environments
```

### Workflow 2: Multi-Environment Release

```
1. Load changeset for branch
   └→ Call: ChangesetManager::load_for_branch("feat/user-auth")
   └→ Changeset has releases: ["dev", "qa"]

2. Release to dev environment
   └→ Call: ReleaseManager::plan_release(changeset, "dev")
   └→ Create ReleasePlan for all packages
   └→ Call: ReleaseManager::execute(plan)
   └→ Bump versions in package.json
   └→ Create Git tag (pkg@1.3.0-dev)
   └→ Commit changes
   └→ Push to Git
   └→ Publish to registry with tag "dev"

3. Later: Release to qa environment
   └→ Call: ReleaseManager::plan_release(changeset, "qa")
   └→ Execute release for qa
   └→ Create Git tag (pkg@1.3.0-qa)
   └→ Publish to registry with tag "qa"
   └→ Generate changelog

4. Archive changeset to history
   └→ Call: ChangesetManager::archive(changeset)
   └→ Add release_info metadata:
      - applied_at: current timestamp
      - applied_by: current user
      - git_commit: current commit hash
      - environments_released: {"dev": {...}, "qa": {...}}
   └→ Move from .changesets/ to .changesets/history/
   └→ Preserve original filename

Result: Same changeset released to multiple environments and archived with complete metadata
```

### Workflow 3: Dependency Propagation

```
1. Detect changed packages
   └→ Get changed files from Git
   └→ Map files to packages
   └→ Result: [pkg-a]

2. Build dependency graph
   └→ Call: DependencyGraph::build_from_monorepo()
   └→ Parse all package.json files
   └→ Construct graph

3. Calculate propagation
   └→ Call: DependencyPropagator::calculate_propagation([pkg-a])
   └→ Find packages depending on pkg-a
   └→ Recursively traverse dependents
   └→ Result: [pkg-a, pkg-b, pkg-c]

4. Create changeset with all affected packages
   └→ pkg-a: minor bump (direct changes)
   └→ pkg-b: patch bump (dependency on pkg-a)
   └→ pkg-c: patch bump (dependency on pkg-b)

Result: All affected packages included in changeset
```

### Workflow 4: Snapshot Version Resolution

```
1. Developer working on branch feat/user-auth
   └→ Current commit: abc123def456
   └→ package.json has: "version": "1.2.3"

2. Get current version (ALWAYS snapshot on branches)
   └→ Call: VersionResolver::resolve_current_version(package_path)
   └→ Check current branch (feat/user-auth - not main)
   └→ Calculate snapshot version
   └→ Result: 1.2.3-abc123d.snapshot

3. Developer makes another commit and pushes
   └→ New commit: def456ghi789
   └→ package.json still has: "version": "1.2.3"
   └→ Resolved version: 1.2.3-def456g.snapshot
   └→ Can deploy to dev with unique version

4. After merge to main
   └→ Changeset applied: minor bump (feat: commits)
   └→ package.json updated: "1.2.3" → "1.3.0"
   └→ Resolved version on main: 1.3.0 (no snapshot)

5. Version usage:
   └→ On branches: Deploy snapshot versions to dev environments
   └→ On main: Deploy release versions to prod environments
   └→ Snapshot versions NEVER written to package.json

Result: Unique snapshot per commit, release version on merge
```

### Workflow 5: Dry Run Before Release

```
1. Load changeset for branch
   └→ Call: ChangesetManager::load_for_branch("feat/user-auth")

2. Plan release to dev environment
   └→ Call: ReleaseManager::plan_release(changeset, "dev")
   └→ Create ReleasePlan

3. Run in dry-run mode
   └→ Call: ReleaseManager::dry_run(plan)
   └→ Calculate all changes without applying
   └→ Display:
      - Packages to update
      - Current and next versions
      - Files to modify
      - Git tags to create
      - Commands to execute

4. Review output and confirm
   └→ User reviews dry-run output
   └→ Validates expected changes
   └→ Confirms or aborts

5. Execute release if approved
   └→ Call: ReleaseManager::execute(plan)
   └→ Apply all changes
   └→ Create Git tags
   └→ Publish to registry

Result: Safe preview before making changes
```

### Workflow 6: Changelog Generation

```
1. Get commits since last release
   └→ Call: Repo::get_commits_since("v1.2.0")

2. Parse conventional commits
   └→ Call: ConventionalCommitParser::parse()
   └→ Extract type, scope, description
   └→ Detect breaking changes

3. Group by type
   └→ Features: [commit1, commit2]
   └→ Bug Fixes: [commit3, commit4]
   └→ Breaking Changes: [commit5]

4. Generate markdown
   └→ Call: ChangelogGenerator::generate()
   └→ Format commits by type
   └→ Add links to commits
   └→ Include breaking changes section

5. Append to CHANGELOG.md
   └→ Call: ChangelogGenerator::append_to_file()
   └→ Prepend new entry to existing file

Result: CHANGELOG.md updated with new version
```

---

### Workflow 7: Changeset History Management

**Goal**: Query and analyze historical releases.

**Steps**:

1. List all pending changesets
2. List all applied changesets (history)
3. Query history by date range
4. Query history by package name
5. Get details of specific historical changeset

**Example**:

```
Query: Show all releases for @myorg/auth-service in last 30 days

Steps:
1. Scan .changesets/history/
2. Filter by package name
3. Filter by date range
4. Return sorted list with metadata

Result:
- feat-oauth2-20240114T091500Z.json
  - Applied: 2024-01-14T14:30:00Z
  - Environments: dev, qa, prod
  - Version: 1.2.0 → 1.3.0
  
- fix-security-20240110T120000Z.json
  - Applied: 2024-01-10T15:00:00Z
  - Environments: dev, qa
  - Version: 1.1.5 → 1.1.6
```

---

## Examples

### Example 1: Basic Version Operations

```rust
use sublime_pkg_tools::{Version, VersionBump};

fn main() -> Result<()> {
    // Parse version
    let version = Version::parse("1.2.3")?;
    println!("Version: {}", version);
    
    // Bump version
    let minor = version.bump(VersionBump::Minor)?;
    println!("After minor bump: {}", minor); // 1.3.0
    
    let major = version.bump(VersionBump::Major)?;
    println!("After major bump: {}", major); // 2.0.0
    
    Ok(())
}
```

### Example 2: Create and Save Changeset

```rust
use sublime_pkg_tools::{ChangesetManager, ReleaseType};
use sublime_git_tools::Repo;

#[tokio::main]
async fn main() -> Result<()> {
    let repo = Repo::open("./")?;
    let manager = ChangesetManager::new("./").await?;
    
    let current_branch = repo.get_current_branch()?;
    let releases = vec!["dev".to_string(), "qa".to_string()];
    
    // Create changeset from Git changes
    let changeset = manager.create_from_git(
        &repo,
        &current_branch,
        releases,
    ).await?;
    
    println!("Created changeset for branch: {}", changeset.branch);
    println!("Packages affected: {}", changeset.packages.len());
    println!("Target environments: {:?}", changeset.releases);
    
    // Save changeset
    let path = manager.save(&changeset).await?;
    println!("Saved to: {:?}", path);
    
    Ok(())
}
```

### Example 3: Dependency Propagation

```rust
use sublime_pkg_tools::{DependencyGraph, DependencyPropagator};
use sublime_git_tools::Repo;

#[tokio::main]
async fn main() -> Result<()> {
    let repo = Repo::open("./")?;
    
    // Build dependency graph
    let graph = DependencyGraph::build_from_monorepo("./").await?;
    
    // Get changed files from Git
    let changed_files = repo.get_all_files_changed_since_branch(
        &graph.package_paths(),
        "main",
    )?;
    
    // Get directly changed packages
    let directly_changed = graph.packages_from_files(&changed_files)?;
    println!("Directly changed: {:?}", directly_changed);
    
    // Calculate propagation
    let propagator = DependencyPropagator::new(graph);
    let all_updates = propagator.calculate_propagation(&directly_changed)?;
    
    for update in all_updates {
        println!("Package: {}", update.package_name);
        println!("  Reason: {:?}", update.reason);
        println!("  Bump: {:?}", update.suggested_bump);
    }
    
    Ok(())
}
```

### Example 4: Multi-Environment Release

```rust
use sublime_pkg_tools::{ChangesetManager, ReleaseManager};
use sublime_git_tools::Repo;

#[tokio::main]
async fn main() -> Result<()> {
    let repo = Repo::open("./")?;
    let changeset_mgr = ChangesetManager::new("./").await?;
    let release_mgr = ReleaseManager::new(repo);
    
    // Load changeset
    let changeset = changeset_mgr.load_for_branch("feat/user-auth").await?
        .expect("Changeset not found");
    
    println!("Target environments: {:?}", changeset.releases);
    
    // Release to dev (if in changeset.releases)
    if changeset.releases.contains(&"dev".to_string()) {
        println!("🚀 Releasing to dev...");
        let dev_plan = release_mgr.plan_release(&changeset, "dev").await?;
        release_mgr.execute(&dev_plan).await?;
        println!("✅ Released to dev: {}", dev_plan.version_tag);
    }
    
    // Later: Release to qa (if in changeset.releases)
    if changeset.releases.contains(&"qa".to_string()) {
        println!("🚀 Releasing to qa...");
        let qa_plan = release_mgr.plan_release(&changeset, "qa").await?;
        release_mgr.execute(&qa_plan).await?;
        println!("✅ Released to qa: {}", qa_plan.version_tag);
    }
    
    Ok(())
}
```

### Example 5: Snapshot Version Resolution (Always on Branches)

```rust
use sublime_pkg_tools::VersionResolver;
use sublime_git_tools::Repo;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    let repo = Repo::open("./")?;
    let resolver = VersionResolver::new(repo);
    
    // Get current version (ALWAYS snapshot on non-main branches)
    let current = resolver.resolve_current_version(
        Path::new("packages/my-pkg")
    ).await?;
    
    println!("Current version: {}", current.to_string());
    // On feature branch: 1.2.3-abc123d.snapshot (ALWAYS)
    // On main branch: 1.2.3 (release version)
    
    // Multiple commits on same branch get different snapshots
    // Commit 1: 1.2.3-abc123d.snapshot
    // Commit 2: 1.2.3-def456g.snapshot
    // Commit 3: 1.2.3-ghi789j.snapshot
    
    // Check if current version is snapshot
    if current.is_snapshot() {
        println!("Development build - can deploy to dev environment");
    } else {
        println!("Release build - can deploy to prod environment");
    }
    
    Ok(())
}
```

### Example 6: Dry Run Release

```rust
use sublime_pkg_tools::{ReleaseManager, ChangesetManager, DryRunResult};
use sublime_git_tools::Repo;

#[tokio::main]
async fn main() -> Result<()> {
    let repo = Repo::open("./")?;
    let changeset_mgr = ChangesetManager::new("./").await?;
    let release_mgr = ReleaseManager::new(repo);
    
    // Load changeset
    let changeset = changeset_mgr.load_for_branch("feat/user-auth").await?
        .expect("Changeset not found");
    
    // Plan release
    let plan = release_mgr.plan_release(&changeset, "dev").await?;
    
    // Run in dry-run mode (no changes made)
    println!("🔍 Running dry-run...\n");
    let dry_run = release_mgr.dry_run(&plan).await?;
    
    // Display what would happen
    println!("Packages to update:");
    for pkg in &dry_run.packages {
        println!("  {} : {} → {}", pkg.name, pkg.current_version, pkg.next_version);
        println!("    Reason: {:?}", pkg.reason);
    }
    
    println!("\nFiles to modify:");
    for file in &dry_run.files_to_modify {
        println!("  - {}", file.display());
    }
    
    println!("\nGit tags to create:");
    for tag in &dry_run.tags_to_create {
        println!("  - {}", tag);
    }
    
    println!("\nCommands to execute:");
    for cmd in &dry_run.commands {
        println!("  - {}", cmd);
    }
    
    // Ask for confirmation
    print!("\nApply these changes? (y/n): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() == "y" {
        println!("✅ Executing release...");
        release_mgr.execute(&plan).await?;
        println!("✅ Release complete!");
    } else {
        println!("❌ Release cancelled");
    }
    
    Ok(())
}
```

### Example 7: Unified vs Independent Versioning

```rust
use sublime_pkg_tools::{ReleaseManager, ReleaseStrategy};
use sublime_git_tools::Repo;

#[tokio::main]
async fn main() -> Result<()> {
    let repo = Repo::open("./")?;
    
    // Example 1: Independent Strategy (default)
    println!("=== Independent Strategy ===");
    let config = PackageToolsConfig {
        release: ReleaseConfig {
            strategy: ReleaseStrategy::Independent,
            ..Default::default()
        },
        ..Default::default()
    };
    
    let release_mgr = ReleaseManager::with_config(repo.clone(), config);
    
    // Packages before:
    // @myorg/auth-service: 1.2.3
    // @myorg/user-service: 2.5.1
    // @myorg/api-gateway: 0.8.0
    
    // After release (auth-service has feat, user-service has dependency update):
    // @myorg/auth-service: 1.3.0 (minor bump)
    // @myorg/user-service: 2.5.2 (patch bump - dependency)
    // @myorg/api-gateway: 0.8.0 (no changes)
    
    // Example 2: Unified Strategy
    println!("\n=== Unified Strategy ===");
    let config = PackageToolsConfig {
        release: ReleaseConfig {
            strategy: ReleaseStrategy::Unified,
            ..Default::default()
        },
        ..Default::default()
    };
    
    let release_mgr = ReleaseManager::with_config(repo.clone(), config);
    
    // Packages before (all at same version):
    // @myorg/auth-service: 1.2.3
    // @myorg/user-service: 1.2.3
    // @myorg/api-gateway: 1.2.3
    
    // After release (highest bump wins - minor in this case):
    // @myorg/auth-service: 1.3.0
    // @myorg/user-service: 1.3.0
    // @myorg/api-gateway: 1.3.0
    
    Ok(())
}
```

### Example 8: Generate Changelog

```rust
use sublime_pkg_tools::{ChangelogGenerator, ChangelogConfig, ConventionalCommitParser};
use sublime_git_tools::Repo;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let repo = Repo::open("./")?;
    
    // Get commits since last release
    let commits = repo.get_commits_since(Some("v1.2.0".to_string()), &None)?;
    
    // Parse conventional commits
    let parser = ConventionalCommitParser;
    let conventional: Vec<_> = commits
        .iter()
        .filter_map(|c| parser.parse(&c.message, &c.hash, &c.author_name, &c.author_date).ok())
        .collect();
    
    // Generate changelog
    let config = ChangelogConfig {
        path: PathBuf::from("CHANGELOG.md"),
        include_commit_hash: true,
        include_authors: false,
        group_by_type: true,
        include_date: true,
    };
    
    let generator = ChangelogGenerator::new(config);
    let version = Version::parse("1.3.0")?;
    let entry = generator.generate(conventional, &version).await?;
    
    // Append to CHANGELOG.md
    generator.append_to_file(&entry).await?;
    
    println!("✅ Changelog updated");
    
    Ok(())
}
```

### Example 9: Changeset History Management

```rust
use sublime_pkg_tools::changeset::ChangesetManager;

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let changeset_mgr = ChangesetManager::new(".changesets")?;

    // List pending changesets
    let pending = changeset_mgr.list_pending().await?;
    println!("Pending changesets: {}", pending.len());
    
    // List historical changesets
    let history = changeset_mgr.list_history().await?;
    println!("Historical changesets: {}", history.len());
    
    // Query history by date range
    let start = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")?;
    let end = DateTime::parse_from_rfc3339("2024-01-31T23:59:59Z")?;
    let january_releases = changeset_mgr
        .query_history_by_date(start, end)
        .await?;
    
    // Query history by package
    let auth_history = changeset_mgr
        .query_history_by_package("@myorg/auth-service")
        .await?;
    
    println!("Auth service releases in January: {}", auth_history.len());
    
    // Get specific changeset from history
    let changeset = changeset_mgr
        .get_from_history("feat-oauth2-20240114T091500Z")
        .await?;
    
    if let Some(release_info) = &changeset.release_info {
        println!("Applied at: {}", release_info.applied_at);
        println!("Applied by: {}", release_info.applied_by);
        println!("Git commit: {}", release_info.git_commit);
        
        for (env, info) in &release_info.environments_released {
            println!("  {}: {} at {}", env, info.tag, info.released_at);
        }
    }

    Ok(())
}
```

---

## Testing Strategy

### Unit Tests

**Coverage**: Each module must have unit tests covering:
- Happy path scenarios
- Error cases
- Edge cases
- Boundary conditions

**Example Structure**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_parse() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }
    
    #[test]
    fn test_version_parse_invalid() {
        assert!(Version::parse("invalid").is_err());
    }
    
    #[test]
    fn test_version_bump_major() {
        let v = Version::parse("1.2.3").unwrap();
        let bumped = v.bump(VersionBump::Major).unwrap();
        assert_eq!(bumped.to_string(), "2.0.0");
    }
}
```

### Integration Tests

**Location**: `tests/` directory

**Coverage**:
- End-to-end workflows
- Multi-component interactions
- Filesystem operations
- Git operations
- Configuration loading

**Example**:
```rust
// tests/integration_tests.rs

use sublime_pkg_tools::*;
use tempfile::TempDir;

#[tokio::test]
async fn test_complete_changeset_workflow() {
    let temp_dir = TempDir::new().unwrap();
    
    // Setup test repository
    // ...
    
    // Create changeset
    let manager = ChangesetManager::new(temp_dir.path()).await.unwrap();
    let changeset = manager.create_from_git(/* ... */).await.unwrap();
    
    // Verify changeset
    assert!(!changeset.packages.is_empty());
    assert!(changeset.releases.contains(&"dev".to_string()));
    
    // Save and reload
    let path = manager.save(&changeset).await.unwrap();
    let filename = path.file_name().unwrap().to_str().unwrap();
    
    // Verify filename format: {branch}-{datetime}.json
    assert!(filename.contains(&changeset.branch.replace("/", "-")));
    assert!(filename.ends_with(".json"));
    
    // Load by branch
    let loaded = manager.load_for_branch(&changeset.branch).await.unwrap()
        .expect("Changeset should exist");
    assert_eq!(loaded.branch, changeset.branch);
}
```

### Property-Based Tests

**Use**: For complex algorithms (dependency propagation, version comparison)

**Tool**: `proptest` crate

**Example**:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_version_ordering(v1 in version_strategy(), v2 in version_strategy()) {
        let cmp1 = v1.cmp(&v2);
        let cmp2 = v2.cmp(&v1);
        
        // Ordering should be antisymmetric
        match cmp1 {
            Ordering::Less => assert_eq!(cmp2, Ordering::Greater),
            Ordering::Greater => assert_eq!(cmp2, Ordering::Less),
            Ordering::Equal => assert_eq!(cmp2, Ordering::Equal),
        }
    }
}
```

### Test Coverage Requirements

**Mandatory**:
- **100% line coverage** for all modules
- **100% branch coverage** for critical paths
- **Edge case coverage** for error handling

**Tools**:
- `cargo tarpaulin` for coverage reports
- CI/CD integration for automated testing

---

## Documentation Requirements

### Module-Level Documentation

**Requirements**:
- Describe what the module does
- Explain how it works
- Explain why it's structured this way
- Provide usage examples

**Example**:
```rust
//! Version management module.
//!
//! ## What
//! This module provides semantic versioning support with snapshot version
//! calculation. It handles version parsing, comparison, and bumping.
//!
//! ## How
//! Version types are based on the semver crate with custom extensions for
//! snapshot versions. Snapshot versions are calculated dynamically at runtime
//! based on the current Git commit, never written to package.json.
//!
//! ## Why
//! Snapshot versions allow development builds to have unique identifiers
//! without polluting the Git history with version bumps in package.json.
//!
//! ## Examples
//!
//! ```rust
//! use sublime_pkg_tools::{Version, VersionBump};
//!
//! let version = Version::parse("1.2.3")?;
//! let bumped = version.bump(VersionBump::Minor)?;
//! assert_eq!(bumped.to_string(), "1.3.0");
//! ```
```

### Struct/Enum Documentation

**Requirements**:
- Describe the type
- Explain when to use it
- Provide examples

**Example**:
```rust
/// Represents a semantic version (major.minor.patch).
///
/// Versions can be parsed from strings, compared, and bumped using
/// the `VersionBump` enum.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::Version;
///
/// let v1 = Version::parse("1.2.3")?;
/// let v2 = Version::parse("1.3.0")?;
///
/// assert!(v1 < v2);
/// ```
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
    pre_release: Option<String>,
    build_metadata: Option<String>,
}
```

### Function Documentation

**Requirements**:
- Brief description
- Parameters explanation
- Return value explanation
- Error conditions
- Examples

**Example**:
```rust
/// Bumps the version according to the specified bump type.
///
/// # Arguments
///
/// * `bump` - The type of version bump to apply (Major, Minor, Patch, etc.)
///
/// # Returns
///
/// A new `Version` with the bumped version number.
///
/// # Errors
///
/// Returns an error if the bump type is invalid for the current version state.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::{Version, VersionBump};
///
/// let version = Version::parse("1.2.3")?;
/// let bumped = version.bump(VersionBump::Minor)?;
/// assert_eq!(bumped.to_string(), "1.3.0");
/// ```
pub fn bump(&self, bump: VersionBump) -> Result<Version>;
```

### README.md

**Structure**:
1. Brief introduction
2. Features overview
3. Installation
4. Quick start
5. Common workflows
6. Configuration
7. API reference (link to SPEC.md)
8. Examples
9. Contributing
10. License

### SPEC.md

**Structure**:
1. API reference for all public types
2. Method signatures with detailed documentation
3. Examples for each major function
4. Error types and handling
5. Configuration options

---

## Dependencies

### Cargo.toml

```toml
[package]
name = "sublime_pkg_tools"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "Package and version management toolkit for Node.js projects"
repository = "https://github.com/your-org/workspace-node-tools"
documentation = "https://docs.rs/sublime_pkg_tools"

[dependencies]
# Internal dependencies
sublime_git_tools = { path = "../git" }
sublime_standard_tools = { path = "../standard" }

# Async runtime
tokio = { version = "1.40", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Semantic versioning
semver = "1.0"

# HTTP client for registry
reqwest = { version = "0.12", features = ["json"] }

# Error handling
thiserror = "1.0"

# Date/time for changesets
chrono = { version = "0.4", features = ["serde"] }

# Graph algorithms for dependency analysis
petgraph = "0.6"

# Regex for conventional commit parsing
regex = "1.10"

# Logging
log = "0.4"

[dev-dependencies]
tempfile = "3.8"
proptest = "1.4"
tokio-test = "0.4"
```

---

## Clippy Rules

**Mandatory clippy rules** (from project requirements):

```rust
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]
```

**Additional recommended rules**:

```rust
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::cargo)]
```

---

## Success Metrics

### Code Quality

- ✅ 100% Clippy compliance (no warnings)
- ✅ 100% test coverage
- ✅ Zero `unwrap()`, `expect()`, `todo!()`, `unimplemented!()`, or `panic!()`
- ✅ All functions documented with examples
- ✅ All modules have comprehensive documentation

### Functionality

- ✅ Can create and manage changesets
- ✅ Can calculate dependency propagation
- ✅ Can release to multiple environments
- ✅ Can generate changelogs
- ✅ Snapshot versions calculated dynamically
- ✅ Integration with Git and standard tools works seamlessly

### Performance

- ✅ Dependency graph builds in < 1s for 100 packages
- ✅ Changeset operations complete in < 100ms
- ✅ Version resolution is instant

### Cross-Platform

- ✅ Tests pass on Windows, macOS, Linux
- ✅ No platform-specific code paths (use standard tools abstractions)

---

## Open Questions and Future Considerations

### Decisions Made

1. **Snapshot Versions on Branches**: ✅ Always use snapshot versions on non-main branches
   - Allows multiple pushes with unique versions
   - Deploy to dev environments continuously
   - Actual bumps only on merge to main

2. **Version Bump for Dependencies**: ✅ Default to patch bump
   - When package A changes, dependent package B gets patch bump (minimum)
   - Safe and semantic

3. **Versioning Strategies**: ✅ Support Independent and Unified
   - Independent: Each package has own version (default)
   - Unified: All packages share same version
   - Configured in repo.config.toml

4. **Dry Run**: ✅ Full dry-run support
   - Preview all changes before applying
   - Show packages, versions, files, tags, commands
   - Programmatic and CLI support

### Open Questions

1. **Changeset Merging**: What happens when two branches both have changesets and are merged?
   - **Answer needed**: Should changesets be merged automatically or require manual intervention?

2. **Version Conflicts in Unified Strategy**: In unified mode, what if one package needs major bump and another needs patch?
   - **Answer needed**: Take highest bump? Force all to same? Error?

3. **Rollback**: How to rollback a release if something goes wrong?
   - **Answer needed**: Should we track previous versions and provide rollback functionality?

4. **Audit Trail**: Should we maintain an audit trail of all releases?
   - **Answer needed**: Store in `.changesets/history/` or separate system?

### Future Enhancements

1. **Workspace Protocol**: Support for npm workspace protocol (`workspace:*`)
2. **Yarn Berry**: Full support for Yarn 2+ (Plug'n'Play)
3. **pnpm Patches**: Support for pnpm patches
4. **Automated Dependency Updates**: Dependabot-like functionality
5. **Release Notes**: Generate release notes from changelogs
6. **Slack/Discord Integration**: Notifications for releases
7. **Approval Workflow**: Require approval before production releases
8. **Interactive CLI**: Rich CLI with prompts and selection
9. **Web UI**: Visual interface for managing releases
10. **Rollback Support**: Revert to previous versions
11. **Release History**: Track all releases in `.changesets/history/`
12. **Advanced Dry Run**: Include dependency tree visualization

---

## Conclusion

This plan provides a comprehensive roadmap for implementing `sublime_pkg_tools`. The phased approach ensures steady progress with clear deliverables and success criteria at each stage.

**Key Principles**:
- Maximum reuse of `sublime_standard_tools` and `sublime_git_tools`
- **Snapshot versions ALWAYS on branches** (never written to disk)
- **Version bumps only on merge to main** (based on changeset)
- **Dependency updates default to patch bump**
- Multi-environment releases at changeset level
- **Support for Independent and Unified versioning strategies**
- **Full dry-run support for safe previews**
- Dependency propagation with clear reasoning
- 100% test coverage and Clippy compliance
- Comprehensive documentation with examples

**Next Steps**:
1. Review and approve this plan
2. Begin Phase 1 implementation (Foundation)
3. Iterate and refine as needed
4. Regular reviews at phase boundaries

---

---

### Summary of All Design Clarifications

### Key Changes from Initial Design

#### Changeset Structure

| Aspect | Initial Design | **Simplified Design** |
|--------|---------------|----------------------|
| **Changeset ID** | Separate `id` field with custom format | ❌ Removed - filename serves as ID |
| **Filename** | `{id}.json` | ✅ `{branch}-{datetime}.json` |
| **Releases Structure** | `HashMap<String, ReleaseEnvironment>` with tracking | ✅ Simple `Vec<String>` array |
| **Release Tracking** | `released`, `released_at`, `version` per env | ❌ Removed - simplified for now |
| **Environment Config** | `environments`, `default_environment` | ✅ `available_environments`, `default_environments` (array) |
| **Package Selection** | Different packages per environment | ✅ All packages released to all specified environments |

### Example: Before vs After

#### Before (Complex)
```json
{
  "id": "feat-user-auth-20240115",
  "branch": "feat/user-auth",
  "releases": {
    "dev": {
      "packages": ["pkg-a", "pkg-b"],
      "released": false,
      "released_at": null,
      "version": null
    },
    "qa": {
      "packages": ["pkg-a"],
      "released": false,
      "released_at": null,
      "version": null
    }
  },
  "packages": [...]
}
```

#### After (Simplified)
```json
{
  "branch": "feat/user-auth",
  "created_at": "2024-01-15T10:30:00Z",
  "author": "developer@example.com",
  "releases": ["dev", "qa"],
  "packages": [...]
}
```

**Filename**: `.changesets/feat-user-auth-20240115T103000Z.json`

### Benefits of Simplification

✅ **Simpler Structure**: Fewer nested objects, easier to understand  
✅ **Self-Documenting Filenames**: Branch and timestamp visible in filename  
✅ **Less State Management**: No need to track release status in changeset  
✅ **Easier Configuration**: Simple array of environments  
✅ **Clear Intent**: Releases array shows exactly which environments to target  
✅ **Git-Friendly**: Filename format works well with Git  

### Configuration Impact

**Before**:
```toml
[package_tools.changeset]
environments = ["dev", "test", "qa"]
default_environment = "dev"

[package_tools.conventional]
default_bump = "snapshot"
```

**After**:
```toml
[package_tools.changeset]
available_environments = ["dev", "test", "qa", "staging", "prod"]
default_environments = ["dev"]
filename_format = "{branch}-{datetime}.json"

[package_tools.release]
strategy = "independent"  # or "unified"
dry_run_by_default = false

[package_tools.dependency]
dependency_update_bump = "patch"  # default bump for dependency updates

# Note: Snapshot versions always used on branches
# Conventional commit bumps only applied on merge to main
```

#### Version Management

| Aspect | Initial Design | **Final Design** |
|--------|---------------|------------------|
| **Snapshot on Branches** | Used for non-conventional commits | ✅ **ALWAYS used on non-main branches** |
| **When Bump Applied** | Per commit | ✅ **Only on merge to main** based on changeset |
| **Dependency Update Bump** | Not specified | ✅ **Patch bump (default)** |
| **Branch Deploy Strategy** | One version per branch | ✅ **Unique snapshot per commit** for continuous testing |

**Impact**: Each push to branch gets unique snapshot version:
- Commit 1: `1.2.3-abc123d.snapshot`
- Commit 2: `1.2.3-def456g.snapshot`
- Commit 3: `1.2.3-ghi789j.snapshot`
- After merge to main: `1.3.0` (changeset bump applied)

#### Versioning Strategies

| Strategy | Description | Use Case | Example |
|----------|-------------|----------|---------|
| **Independent** | Each package maintains own version | Default for monorepos | pkg-a@1.2.0, pkg-b@2.5.1 |
| **Unified** | All packages share same version | Tightly coupled packages | pkg-a@1.2.0, pkg-b@1.2.0 |

**Configuration**:
```toml
[package_tools.release]
strategy = "independent"  # or "unified"
```

#### Dry Run Support

| Feature | Description | Benefit |
|---------|-------------|---------|
| **Preview Mode** | Show changes without applying | Validate before release |
| **Displays** | Packages, versions, files, tags, commands | Complete visibility |
| **Safe** | No writes to filesystem, Git, or registry | Risk-free preview |
| **Programmatic API** | `release_mgr.dry_run(plan)` | Integrate into workflows |

**Example Output**:
```
🔍 DRY RUN MODE
Packages: @myorg/auth-service: 1.2.3 → 1.3.0
Files: packages/auth-service/package.json
Tags: @myorg/auth-service@1.3.0-dev
Commands: npm publish packages/auth-service --tag dev
```

### Complete Workflow: Branch to Production

```
1. On Feature Branch (feat/user-auth)
   ├─ Developer commits (feat: add OAuth2)
   ├─ Version: 1.2.3-abc123d.snapshot
   ├─ Push to branch
   ├─ Deploy to dev environment (snapshot version)
   │
   ├─ More commits (fix: resolve issue)
   ├─ Version: 1.2.3-def456g.snapshot
   ├─ Push to branch
   └─ Deploy to dev environment (new snapshot)

2. Create Changeset
   ├─ Analyze commits (feat + fix)
   ├─ Determine bump: minor (feat wins)
   ├─ Check dependencies
   ├─ Calculate propagation (dependents get patch)
   └─ Save: .changesets/feat-user-auth-20240115T103000Z.json

3. Dry Run Release
   ├─ Load changeset
   ├─ Plan release to test environment
   ├─ Run dry-run
   ├─ Review: packages, versions, files, tags
   └─ Confirm or abort

4. Merge to Main
   ├─ Apply changeset bumps
   ├─ Update package.json: 1.2.3 → 1.3.0
   ├─ Commit version changes
   ├─ Create Git tags
   └─ Push to main

5. Release to Environments
   ├─ Test: Deploy 1.3.0 with tag "test"
   ├─ QA: Deploy 1.3.0 with tag "qa"
   └─ Prod: Deploy 1.3.0 with tag "latest"

6. Archive Changeset
   ├─ Add release_info metadata:
   │  - applied_at: timestamp
   │  - applied_by: user
   │  - git_commit: commit hash
   │  - environments_released: {test, qa, prod}
   ├─ Move from .changesets/ to .changesets/history/
   ├─ Preserve filename: feat-user-auth-20240115T103000Z.json
   └─ Complete audit trail maintained
```

### Key Principles Summary

✅ **Snapshots ALWAYS on branches** - unique version per commit for continuous testing  
✅ **Version bumps ONLY on merge to main** - controlled, reviewable changes  
✅ **Dependency updates = patch bump** - safe default for propagated changes  
✅ **Two strategies supported** - Independent (flexible) and Unified (simple)  
✅ **Dry run for safety** - preview before applying changes  
✅ **Configuration-driven** - all behavior configurable via repo.config.toml  
✅ **Multi-environment releases** - same changeset, multiple targets  
✅ **History tracking** - complete audit trail of all releases via .changesets/history/  

---

**Document Version**: 2.0  
**Last Updated**: 2024-01-15  
**Status**: Ready for Implementation

---

## Design Complete ✅

This plan incorporates all key clarifications:

1. ✅ **Simplified Changeset Structure** - Simple releases array, filename-based identity
2. ✅ **Snapshot Versions Always on Branches** - Unique version per commit for continuous testing
3. ✅ **Version Bumps Only on Merge** - Changeset bumps applied when merging to main
4. ✅ **Dependency Updates = Patch Bump** - Safe default for propagated changes
5. ✅ **Changeset History Tracking** - Automatic archival to .changesets/history/ with release metadata
5. ✅ **Two Versioning Strategies** - Independent (default) and Unified
6. ✅ **Full Dry Run Support** - Preview all changes before applying
7. ✅ **Configuration Integration** - Uses repo.config.toml from sublime_standard_tools
8. ✅ **Maximum Reuse** - Leverages sublime_git_tools and sublime_standard_tools

**Ready to proceed with implementation!** 🚀