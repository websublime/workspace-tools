# Prerelease Versions - Implementation Plan

**Version**: 1.0  
**Date**: 2025-01-19  
**Status**: Planning Complete  
**Author**: Development Team

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current System State](#current-system-state)
3. [SemVer 2.0.0 Prerelease Specification](#semver-200-prerelease-specification)
4. [Proposed Architecture](#proposed-architecture)
5. [Implementation Details](#implementation-details)
6. [Changeset Archive Policies](#changeset-archive-policies)
7. [Use Cases - Different Workflows](#use-cases---different-workflows)
8. [Implementation Checklist](#implementation-checklist)
9. [Risks and Mitigations](#risks-and-mitigations)
10. [Summary](#summary)

---

## Executive Summary

### Problem Statement

The `workspace bump` command currently ignores the `--prerelease` flag, which exists in the CLI arguments but has no implementation. This prevents users from creating prerelease versions (e.g., `1.3.0-beta.0`, `1.3.0-rc.1`) and forces them to either:

1. Use `--snapshot` for temporary versions (not suitable for controlled prereleases)
2. Use `--no-archive` manually to prevent changeset consumption
3. Skip prerelease workflows entirely

### Solution Overview

Implement full prerelease support with a **workflow-agnostic** design that works for:
- GitHub Flow (feature → main)
- Gitflow (feature → develop → main)
- Custom workflows (feature → staging → production)
- Any other branching strategy

### Key Features

✅ **Flexible Prerelease Modes**: Create, Increment, Promote  
✅ **Smart Archive Policy**: Auto-detect when to archive changesets  
✅ **SemVer 2.0.0 Compliant**: Follows specification exactly  
✅ **Backward Compatible**: No breaking changes to existing workflows  
✅ **Explicit Opt-in**: Requires `--prerelease` flag to activate  

---

## Current System State

### 1.1 Infrastructure Already in Place

#### ✅ Version Type (`crates/pkg/src/types/version.rs`)

**Prerelease support ALREADY EXISTS:**

```rust
pub struct Version {
    inner: semver::Version,  // ← Uses semver::Version with prerelease support!
}

impl Version {
    pub fn prerelease(&self) -> &str;      // ← Returns prerelease string
    pub fn is_prerelease(&self) -> bool;   // ← Checks if prerelease
    pub fn build(&self) -> &str;           // ← Build metadata
    
    // ❌ BUT: bump() REMOVES prerelease
    pub fn bump(&self, bump_type: VersionBump) -> VersionResult<Self> {
        // Current implementation creates semver::Version::new() without preserving prerelease
    }
}
```

**Capabilities:**
- ✅ Parse prerelease versions: `Version::parse("1.2.0-beta.1")` works
- ✅ Comparison respects semver precedence
- ✅ Serialization/deserialization
- ❌ Bump **does not support** prerelease (always removes it)
- ✅ Snapshot uses prerelease: `1.2.0-snapshot-{timestamp}-{hash}`

### 1.2 VersionResolver (`crates/pkg/src/version/resolver.rs`)

**Current Flow:**

```rust
pub async fn resolve_versions(&self, changeset: &Changeset) -> VersionResult<VersionResolution> {
    // 1. Discover packages
    // 2. Resolve versions by strategy (Independent | Unified)
    // 3. Apply dependency propagation
    
    // ❌ Does not accept prerelease parameter
}
```

**Intervention Point** (`crates/pkg/src/version/resolution.rs:486`):

```rust
let current_version = package_info.version();
let next_version = current_version.bump(changeset.bump)?;  // ← HERE!
```

### 1.3 CLI Bump Command

**Parameter exists but is IGNORED:**

```rust
// crates/cli/src/cli/commands.rs:495
pub struct BumpArgs {
    pub prerelease: Option<String>,  // ← Exists but not used!
}
```

**Execute flow** (`crates/cli/src/commands/bump/execute.rs`):

```rust
// Line 209: Resolve versions
let resolution = resolver.resolve_versions(&merged_changeset).await?;
// ❌ args.prerelease is not passed!
```

---

## SemVer 2.0.0 Prerelease Specification

### 2.1 Format

```
MAJOR.MINOR.PATCH-PRERELEASE+BUILD

Examples:
- 1.0.0-alpha
- 1.0.0-alpha.1
- 1.0.0-beta.2
- 1.0.0-rc.1
- 1.0.0-rc.1+build.123
```

### 2.2 Rules

1. **Identifiers**: Only ASCII alphanumerics and hyphens `[0-9A-Za-z-]`
2. **Separation**: Dot-separated identifiers (`.`)
3. **Leading Zeros**: Numeric identifiers CANNOT have leading zeros
4. **Precedence**: `alpha < alpha.1 < beta < beta.2 < rc.1 < 1.0.0`

### 2.3 Bump Semantics

**Expected Behavior:**

| Current Version | Bump Type | Without Prerelease | With Prerelease `beta` |
|-----------------|-----------|-------------------|------------------------|
| `1.2.3` | Minor | `1.3.0` | `1.3.0-beta.0` |
| `1.2.3-beta.0` | Patch | `1.2.4` | `1.2.4-beta.0` |
| `1.2.3-beta.0` | Prerelease | N/A | `1.2.3-beta.1` |
| `1.2.3-alpha.5` | Minor (promote) | `1.3.0` | `1.3.0-beta.0` |

**Prerelease Bump Types:**

1. **Normal Bump + Prerelease**: `1.2.3` → `1.3.0-beta.0`
2. **Prerelease Increment**: `1.3.0-beta.0` → `1.3.0-beta.1`
3. **Promotion (remove prerelease)**: `1.3.0-rc.1` → `1.3.0`

---

## Proposed Architecture

### 3.1 Design Principle

**❌ AVOID**: Assuming a specific workflow (feature→develop→main)

**✅ USE**: Generic mechanisms that work for ANY workflow:
- feature → main (GitHub flow)
- feature → develop → main (Gitflow)
- feature → staging → production (Custom)
- Any other...

### 3.2 Solution Components

```
┌──────────────────────────────────────────────────────────────┐
│                    CLI Bump Command                           │
│  workspace bump --execute --prerelease beta                   │
└─────────────────────┬────────────────────────────────────────┘
                      │
                      ↓
        ┌─────────────────────────────┐
        │  Prerelease Config          │
        │  - tag: String (beta, rc)   │
        │  - mode: PrereleaseMode     │
        └──────────┬──────────────────┘
                   │
                   ↓
        ┌──────────────────────────────┐
        │  VersionResolver             │
        │  resolve_with_prerelease()   │
        └──────────┬───────────────────┘
                   │
                   ↓
        ┌──────────────────────────────┐
        │  Version::bump_with_prerelease│
        │  - Normal bump + tag          │
        │  - Increment prerelease       │
        │  - Promote to stable          │
        └──────────┬───────────────────┘
                   │
                   ↓
        ┌──────────────────────────────┐
        │  Changeset Archive Policy    │
        │  - Auto (default)            │
        │  - Never (--no-archive)      │
        │  - Always (--always-archive) │
        └──────────────────────────────┘
```

### 3.3 New Types

#### **PrereleaseConfig**

```rust
/// Configuration for prerelease version bumping.
///
/// # What
///
/// Defines the prerelease tag (e.g., "alpha", "beta", "rc") and the behavior mode
/// (create new prerelease, increment existing, or promote to stable).
///
/// # Why
///
/// Provides explicit control over prerelease version generation while maintaining
/// flexibility across different branching workflows.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::types::prerelease::{PrereleaseConfig, PrereleaseMode};
///
/// // Create new beta prerelease
/// let config = PrereleaseConfig {
///     tag: "beta".to_string(),
///     mode: PrereleaseMode::Create,
/// };
///
/// // Increment existing beta
/// let config = PrereleaseConfig {
///     tag: "beta".to_string(),
///     mode: PrereleaseMode::Increment,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrereleaseConfig {
    /// Prerelease tag (e.g., "alpha", "beta", "rc").
    pub tag: String,
    
    /// Behavior mode.
    pub mode: PrereleaseMode,
}

/// Prerelease version bump mode.
///
/// # Variants
///
/// - **Create**: Generate new prerelease from stable version
///   - Example: `1.2.3` → `1.3.0-beta.0`
/// - **Increment**: Increment existing prerelease number
///   - Example: `1.3.0-beta.0` → `1.3.0-beta.1`
/// - **Promote**: Remove prerelease tag (promote to stable)
///   - Example: `1.3.0-rc.1` → `1.3.0`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrereleaseMode {
    /// Create new prerelease from stable: 1.2.3 → 1.3.0-beta.0
    Create,
    
    /// Increment existing prerelease: 1.3.0-beta.0 → 1.3.0-beta.1
    Increment,
    
    /// Promote to stable (remove prerelease): 1.3.0-rc.1 → 1.3.0
    Promote,
}
```

#### **ChangesetArchivePolicy**

```rust
/// Policy for archiving changesets after version bump.
///
/// # What
///
/// Defines when changesets should be archived (moved to history) after applying
/// version bumps.
///
/// # Why
///
/// Different workflows need different changeset management:
/// - Prereleases often need multiple bumps before final release
/// - Stable releases typically archive changesets immediately
/// - Some workflows want explicit control
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::types::ChangesetArchivePolicy;
///
/// // Auto-decide based on version type (default)
/// let policy = ChangesetArchivePolicy::Auto;
///
/// // Never archive
/// let policy = ChangesetArchivePolicy::Never;
///
/// // Always archive
/// let policy = ChangesetArchivePolicy::Always;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangesetArchivePolicy {
    /// Auto-decide based on version type:
    /// - Prerelease versions: DON'T archive
    /// - Stable versions: Archive
    Auto,
    
    /// Never archive changesets (--no-archive).
    Never,
    
    /// Always archive regardless of version type (--always-archive).
    Always,
}
```

---

## Implementation Details

### 4.1 Modify Version::bump

**File**: `crates/pkg/src/types/version.rs`

**Add new method:**

```rust
impl Version {
    /// Bumps version with optional prerelease support.
    ///
    /// # What
    ///
    /// Provides flexible version bumping that supports standard semver bumps
    /// (major, minor, patch) as well as prerelease version creation, increment,
    /// and promotion to stable.
    ///
    /// # How
    ///
    /// - If `prerelease_config` is None: Standard bump (removes prerelease)
    /// - If `prerelease_config` is Some:
    ///   - Create mode: Bump + add prerelease (1.2.3 → 1.3.0-beta.0)
    ///   - Increment mode: Increment prerelease (1.3.0-beta.0 → 1.3.0-beta.1)
    ///   - Promote mode: Remove prerelease (1.3.0-rc.1 → 1.3.0)
    ///
    /// # Why
    ///
    /// Enables controlled prerelease workflows while maintaining backward
    /// compatibility with existing version bump behavior.
    ///
    /// # Arguments
    ///
    /// * `bump_type` - Type of version bump (Major, Minor, Patch, None)
    /// * `prerelease_config` - Optional prerelease configuration
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Version overflow would occur
    /// - Invalid prerelease format
    /// - Attempting to increment prerelease on stable version
    /// - Prerelease tag mismatch when incrementing
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::{Version, VersionBump};
    /// use sublime_pkg_tools::types::prerelease::{PrereleaseConfig, PrereleaseMode};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let v = Version::parse("1.2.3")?;
    ///
    /// // Normal bump (backward compatible)
    /// let bumped = v.bump_with_prerelease(VersionBump::Minor, None)?;
    /// assert_eq!(bumped.to_string(), "1.3.0");
    ///
    /// // Create prerelease
    /// let config = PrereleaseConfig {
    ///     tag: "beta".to_string(),
    ///     mode: PrereleaseMode::Create,
    /// };
    /// let beta = v.bump_with_prerelease(VersionBump::Minor, Some(&config))?;
    /// assert_eq!(beta.to_string(), "1.3.0-beta.0");
    ///
    /// // Increment prerelease
    /// let config = PrereleaseConfig {
    ///     tag: "beta".to_string(),
    ///     mode: PrereleaseMode::Increment,
    /// };
    /// let beta1 = beta.bump_with_prerelease(VersionBump::None, Some(&config))?;
    /// assert_eq!(beta1.to_string(), "1.3.0-beta.1");
    ///
    /// // Promote to stable
    /// let config = PrereleaseConfig {
    ///     tag: "beta".to_string(),
    ///     mode: PrereleaseMode::Promote,
    /// };
    /// let stable = beta1.bump_with_prerelease(VersionBump::None, Some(&config))?;
    /// assert_eq!(stable.to_string(), "1.3.0");
    /// # Ok(())
    /// # }
    /// ```
    pub fn bump_with_prerelease(
        &self,
        bump_type: VersionBump,
        prerelease_config: Option<&PrereleaseConfig>,
    ) -> VersionResult<Self> {
        match prerelease_config {
            None => {
                // Standard bump - maintains current behavior
                self.bump(bump_type)
            }
            Some(config) => match config.mode {
                PrereleaseMode::Create => {
                    // Bump version + add prerelease tag
                    let bumped = self.bump(bump_type)?;
                    bumped.with_prerelease(&format!("{}.0", config.tag))
                }
                PrereleaseMode::Increment => {
                    // Increment existing prerelease number
                    self.increment_prerelease(&config.tag)
                }
                PrereleaseMode::Promote => {
                    // Remove prerelease (promote to stable)
                    self.remove_prerelease()
                }
            },
        }
    }

    /// Sets or replaces prerelease tag.
    ///
    /// # Arguments
    ///
    /// * `tag` - Prerelease tag to set (e.g., "beta.0", "rc.1")
    ///
    /// # Errors
    ///
    /// Returns error if tag format is invalid per SemVer 2.0.0 spec.
    fn with_prerelease(&self, tag: &str) -> VersionResult<Self> {
        let mut new_version = self.inner.clone();
        new_version.pre = semver::Prerelease::new(tag)
            .map_err(|e| VersionError::InvalidVersion {
                version: tag.to_string(),
                reason: format!("invalid prerelease tag: {}", e),
            })?;
        Ok(Self { inner: new_version })
    }

    /// Increments prerelease number (e.g., beta.0 → beta.1).
    ///
    /// # Arguments
    ///
    /// * `expected_tag` - Expected prerelease tag base (e.g., "beta")
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Current version is not a prerelease
    /// - Prerelease format is invalid
    /// - Tag mismatch (trying to increment beta when current is alpha)
    fn increment_prerelease(&self, expected_tag: &str) -> VersionResult<Self> {
        let current_pre = self.prerelease();
        if current_pre.is_empty() {
            return Err(VersionError::InvalidVersion {
                version: self.to_string(),
                reason: "cannot increment prerelease on stable version".to_string(),
            });
        }

        // Parse current prerelease: "beta.0" → ("beta", 0)
        let parts: Vec<&str> = current_pre.split('.').collect();
        if parts.len() != 2 {
            return Err(VersionError::InvalidVersion {
                version: self.to_string(),
                reason: format!("invalid prerelease format: {}", current_pre),
            });
        }

        let tag = parts[0];
        let num: u64 = parts[1].parse().map_err(|_| VersionError::InvalidVersion {
            version: self.to_string(),
            reason: format!("invalid prerelease number: {}", parts[1]),
        })?;

        // Validate tag matches
        if tag != expected_tag {
            return Err(VersionError::InvalidVersion {
                version: self.to_string(),
                reason: format!(
                    "prerelease tag mismatch: expected '{}', found '{}'",
                    expected_tag, tag
                ),
            });
        }

        // Increment
        let new_pre = format!("{}.{}", tag, num + 1);
        self.with_prerelease(&new_pre)
    }

    /// Removes prerelease tag (promotes to stable).
    ///
    /// # What
    ///
    /// Creates a stable version by removing the prerelease tag while preserving
    /// major.minor.patch and build metadata.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::types::Version;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let prerelease = Version::parse("1.3.0-rc.1")?;
    /// let stable = prerelease.remove_prerelease()?;
    /// assert_eq!(stable.to_string(), "1.3.0");
    /// # Ok(())
    /// # }
    /// ```
    fn remove_prerelease(&self) -> VersionResult<Self> {
        let mut new_version = semver::Version::new(
            self.inner.major,
            self.inner.minor,
            self.inner.patch,
        );
        // Copy build metadata if present
        new_version.build = self.inner.build.clone();
        Ok(Self { inner: new_version })
    }
}
```

### 4.2 Modify VersionResolver

**File**: `crates/pkg/src/version/resolver.rs`

**Add new method:**

```rust
impl<F: AsyncFileSystem + Clone + Send + Sync + 'static> VersionResolver<F> {
    /// Resolves versions with optional prerelease support.
    ///
    /// # What
    ///
    /// Extends the standard version resolution to support prerelease versions,
    /// allowing controlled creation and increment of prerelease versions.
    ///
    /// # How
    ///
    /// Follows the same flow as `resolve_versions()` but passes prerelease
    /// configuration down to the resolution logic, which then uses
    /// `Version::bump_with_prerelease()` instead of standard bump.
    ///
    /// # Why
    ///
    /// Enables prerelease workflows while maintaining the existing resolution
    /// logic for dependency propagation and strategy handling.
    ///
    /// # Arguments
    ///
    /// * `changeset` - Changeset containing packages and bump type
    /// * `prerelease_config` - Optional prerelease configuration
    ///
    /// # Errors
    ///
    /// Returns error if version resolution or propagation fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::VersionResolver;
    /// use sublime_pkg_tools::types::prerelease::{PrereleaseConfig, PrereleaseMode};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let resolver = VersionResolver::new(workspace_root, config).await?;
    ///
    /// // Without prerelease (standard behavior)
    /// let resolution = resolver.resolve_versions_with_prerelease(&changeset, None).await?;
    ///
    /// // With prerelease
    /// let config = PrereleaseConfig {
    ///     tag: "beta".to_string(),
    ///     mode: PrereleaseMode::Create,
    /// };
    /// let resolution = resolver.resolve_versions_with_prerelease(&changeset, Some(&config)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn resolve_versions_with_prerelease(
        &self,
        changeset: &Changeset,
        prerelease_config: Option<&PrereleaseConfig>,
    ) -> VersionResult<VersionResolution> {
        // Discover all packages in the workspace
        let package_list = self.discover_packages().await?;

        // Build dependency graph for propagation
        let (graph, circular_deps) = if self.config.dependency.propagation_bump != "none" {
            let g = DependencyGraph::from_packages(&package_list)?;
            let cycles = g.detect_cycles();
            (Some(g), cycles)
        } else {
            (None, Vec::new())
        };

        // Build package map
        let mut packages = HashMap::new();
        for package_info in package_list {
            let name = package_info.name().to_string();
            packages.insert(name, package_info);
        }

        // Step 1: Resolve direct version changes with prerelease support
        let mut resolution = resolve_versions_with_prerelease(
            changeset,
            &packages,
            self.strategy,
            prerelease_config,
        ).await?;

        // Step 2: Add circular dependencies
        resolution.circular_dependencies = circular_deps;

        // Step 3: Apply dependency propagation
        if let Some(graph) = graph {
            let propagator = DependencyPropagator::new(&graph, &packages, &self.config.dependency);
            propagator.propagate(&mut resolution)?;
        }

        Ok(resolution)
    }
}
```

**File**: `crates/pkg/src/version/resolution.rs`

**Add new function:**

```rust
/// Resolves versions with optional prerelease support.
///
/// # Arguments
///
/// * `changeset` - Changeset containing packages and bump type
/// * `packages` - Map of package info
/// * `strategy` - Versioning strategy (Independent or Unified)
/// * `prerelease_config` - Optional prerelease configuration
///
/// # Errors
///
/// Returns error if version parsing or bumping fails.
pub async fn resolve_versions_with_prerelease(
    changeset: &Changeset,
    packages: &HashMap<String, PackageInfo>,
    strategy: VersioningStrategy,
    prerelease_config: Option<&PrereleaseConfig>,
) -> VersionResult<VersionResolution> {
    validate_packages_exist(changeset, packages)?;

    match strategy {
        VersioningStrategy::Independent => {
            resolve_independent_with_prerelease(changeset, packages, prerelease_config).await
        }
        VersioningStrategy::Unified => {
            resolve_unified_with_prerelease(changeset, packages, prerelease_config).await
        }
    }
}

/// Resolves versions using independent strategy with prerelease support.
async fn resolve_independent_with_prerelease(
    changeset: &Changeset,
    packages: &HashMap<String, PackageInfo>,
    prerelease_config: Option<&PrereleaseConfig>,
) -> VersionResult<VersionResolution> {
    let mut resolution = VersionResolution::new();

    for package_name in &changeset.packages {
        let package_info = packages.get(package_name)
            .ok_or_else(|| VersionError::PackageNotFound {
                name: package_name.clone(),
                workspace_root: PathBuf::from("."),
            })?;

        let current_version = package_info.version();
        
        // ✅ Use new method with prerelease support
        let next_version = current_version.bump_with_prerelease(
            changeset.bump,
            prerelease_config,
        )?;

        let update = PackageUpdate::new(
            package_name.clone(),
            package_info.path().to_path_buf(),
            current_version,
            next_version,
            UpdateReason::DirectChange,
        );

        resolution.add_update(update);
    }

    Ok(resolution)
}

/// Resolves versions using unified strategy with prerelease support.
async fn resolve_unified_with_prerelease(
    changeset: &Changeset,
    packages: &HashMap<String, PackageInfo>,
    prerelease_config: Option<&PrereleaseConfig>,
) -> VersionResult<VersionResolution> {
    // Similar to independent but applies same version to all packages
    // Implementation details follow same pattern as resolve_unified()
    // but uses bump_with_prerelease() instead of bump()
    
    // TODO: Implement unified strategy with prerelease support
    todo!("TODO: will be implemented - unified strategy with prerelease")
}
```

### 4.3 Modify CLI Bump Command

**File**: `crates/cli/src/commands/bump/execute.rs`

**Modify execute_bump_apply:**

```rust
pub async fn execute_bump_apply(
    args: &BumpArgs,
    output: &Output,
    root: &Path,
    config_path: Option<&Path>,
) -> Result<()> {
    let workspace_root = root;
    info!("Executing bump apply in workspace: {}", workspace_root.display());

    // Step 1: Validate Git repository state if git operations requested
    let git_repo = if args.git_commit || args.git_tag || args.git_push {
        // ... existing git validation code ...
        Some(repo)
    } else {
        None
    };

    // Step 2: Load configuration
    let config = load_config(workspace_root, config_path).await?;

    // Step 3: Load all pending changesets
    let fs = FileSystemManager::new();
    let manager = ChangesetManager::new(workspace_root.to_path_buf(), fs.clone(), config.clone())
        .await
        .map_err(|e| CliError::execution(format!("Failed to create changeset manager: {e}")))?;

    let loaded_changesets = manager.list_pending().await
        .map_err(|e| CliError::execution(format!("Failed to load changesets: {e}")))?;

    if loaded_changesets.is_empty() {
        // ... handle empty changesets ...
        return Ok(());
    }

    // ✅ Step 4: Parse prerelease configuration
    let prerelease_config = parse_prerelease_args(args, &loaded_changesets)?;

    // Step 5: Create VersionResolver
    let resolver = VersionResolver::new(workspace_root.to_path_buf(), config.clone())
        .await
        .map_err(|e| CliError::execution(format!("Failed to create version resolver: {e}")))?;

    let merged_changeset = merge_changesets(&loaded_changesets)?;

    // ✅ Step 6: Resolve versions WITH prerelease support
    let resolution = resolver
        .resolve_versions_with_prerelease(&merged_changeset, prerelease_config.as_ref())
        .await
        .map_err(|e| CliError::execution(format!("Failed to resolve versions: {e}")))?;

    if resolution.updates.is_empty() {
        // ... handle no updates ...
        return Ok(());
    }

    // Step 7: Show confirmation prompt
    if !args.force && !output.format().is_json() {
        // ... existing confirmation code ...
    }

    // Step 8: Apply version updates
    let apply_result = resolver.apply_versions(&merged_changeset, false).await
        .map_err(|e| CliError::execution(format!("Failed to apply version updates: {e}")))?;

    let mut modified_files: Vec<PathBuf> = apply_result.resolution.updates
        .iter()
        .map(|u| u.path.join("package.json"))
        .collect();

    // Step 9: Generate changelogs
    if !args.no_changelog && config.changelog.enabled {
        // ... existing changelog generation code ...
    }

    // ✅ Step 10: Determine archive policy and conditionally archive
    let archive_policy = determine_archive_policy(args, &resolution);
    let mut archived_count = 0;

    if should_archive(&archive_policy, &resolution) {
        info!("Archiving changesets");

        let commit_sha = if let Some(ref repo) = git_repo {
            get_current_commit_sha(repo).unwrap_or_else(|_| "unknown".to_string())
        } else {
            "unknown".to_string()
        };

        let mut versions_map = HashMap::new();
        for update in &apply_result.resolution.updates {
            versions_map.insert(update.name.clone(), update.next_version.to_string());
        }

        let release_info = ReleaseInfo::new("workspace-cli", commit_sha.as_str(), versions_map);

        for changeset in &loaded_changesets {
            manager.archive(&changeset.branch, release_info.clone()).await
                .map_err(|e| CliError::execution(format!("Failed to archive changeset: {}", e)))?;
            archived_count += 1;
        }

        info!("Archived {} changeset(s)", archived_count);
    } else {
        debug!("Changeset archival skipped by policy: {:?}", archive_policy);
    }

    // Step 11: Git operations
    // ... existing git code ...

    // Step 12: Build and display result
    // ... existing result code ...

    Ok(())
}

/// Parses prerelease arguments and determines mode.
///
/// # What
///
/// Validates the --prerelease flag value and creates a PrereleaseConfig
/// with the appropriate mode.
///
/// # Arguments
///
/// * `args` - Bump command arguments
/// * `changesets` - Loaded changesets (for auto-detection)
///
/// # Errors
///
/// Returns error if prerelease tag is invalid.
fn parse_prerelease_args(
    args: &BumpArgs,
    _changesets: &[Changeset],
) -> Result<Option<PrereleaseConfig>> {
    let Some(tag) = &args.prerelease else {
        return Ok(None);
    };

    // Validate tag
    if !matches!(tag.as_str(), "alpha" | "beta" | "rc") {
        return Err(CliError::validation(format!(
            "Invalid prerelease tag: '{}'. Valid values: alpha, beta, rc",
            tag
        )));
    }

    // For now, default to Create mode
    // TODO: Auto-detect mode based on current package versions (future enhancement)
    let mode = PrereleaseMode::Create;

    Ok(Some(PrereleaseConfig {
        tag: tag.clone(),
        mode,
    }))
}

/// Determines changeset archive policy based on arguments and resolution.
///
/// # What
///
/// Decides whether changesets should be archived based on user flags
/// and version types in the resolution.
///
/// # Arguments
///
/// * `args` - Bump command arguments
/// * `resolution` - Version resolution result
///
/// # Returns
///
/// The archive policy to use.
fn determine_archive_policy(
    args: &BumpArgs,
    _resolution: &VersionResolution,
) -> ChangesetArchivePolicy {
    if args.no_archive {
        return ChangesetArchivePolicy::Never;
    }

    if args.always_archive {
        return ChangesetArchivePolicy::Always;
    }

    // Default to Auto policy
    ChangesetArchivePolicy::Auto
}

/// Determines whether to archive changesets based on policy and resolution.
///
/// # What
///
/// Applies the archive policy to decide if changesets should be archived.
///
/// # How
///
/// - Auto: Archives only if ALL versions are stable (no prerelease)
/// - Never: Never archives
/// - Always: Always archives
///
/// # Arguments
///
/// * `policy` - Archive policy
/// * `resolution` - Version resolution result
///
/// # Returns
///
/// True if changesets should be archived.
fn should_archive(
    policy: &ChangesetArchivePolicy,
    resolution: &VersionResolution,
) -> bool {
    match policy {
        ChangesetArchivePolicy::Always => true,
        ChangesetArchivePolicy::Never => false,
        ChangesetArchivePolicy::Auto => {
            // Auto: only archive if ALL versions are stable (no prerelease)
            resolution.updates.iter().all(|u| !u.next_version.is_prerelease())
        }
    }
}
```

**Add new flag to BumpArgs:**

```rust
// crates/cli/src/cli/commands.rs

#[derive(Debug, Args)]
pub struct BumpArgs {
    // ... existing fields ...

    /// Always archive changesets.
    ///
    /// Forces changesets to be archived even for prerelease versions.
    /// Overrides auto-detection behavior.
    #[arg(long, conflicts_with = "no_archive")]
    pub always_archive: bool,
}
```

---

## Changeset Archive Policies

### 6.1 Policy Options

| Policy | Behavior | CLI Flag | Use Case |
|--------|----------|----------|----------|
| **Auto** | Archive only stable releases | *(default)* | Standard workflow with prereleases |
| **Never** | Never archive changesets | `--no-archive` | Testing, continuous prereleases |
| **Always** | Always archive | `--always-archive` | Explicit control, single-use changesets |

### 6.2 Auto Policy Rationale

**Why Auto is the default:**

The Auto policy intelligently decides based on version type:
- **Prerelease versions** (`1.3.0-beta.0`): **DON'T** archive
- **Stable versions** (`1.3.0`): **Archive**

**Benefits:**

✅ Changesets available for multiple prerelease iterations  
✅ Automatic final archive on stable release  
✅ No manual intervention needed  
✅ Works for any workflow

**Example Flow:**

```bash
# Prerelease 1
workspace bump --execute --prerelease beta
# → 1.3.0-beta.0
# → Changesets KEPT (auto policy detects prerelease)

# Prerelease 2
workspace bump --execute --prerelease beta
# → 1.3.0-beta.1
# → Changesets KEPT

# Final release
workspace bump --execute  # NO --prerelease
# → 1.3.0
# → Changesets ARCHIVED (auto policy detects stable)
```

### 6.3 Override Options

**Never Archive** (testing, continuous prereleases):

```bash
workspace bump --execute --prerelease beta --no-archive
# → 1.3.0-beta.0
# → Changesets KEPT (explicit)
```

**Always Archive** (single-use changesets):

```bash
workspace bump --execute --prerelease beta --always-archive
# → 1.3.0-beta.0
# → Changesets ARCHIVED (explicit)
```

---

## Use Cases - Different Workflows

### 7.1 GitHub Flow (feature → main)

**Workflow**: Direct merge to main, no prereleases needed.

```bash
# Feature branch
git checkout -b feature/new-api
# ... development ...
workspace changeset create --bump minor

# Merge to main
git checkout main
git merge feature/new-api

# Release directly
workspace bump --execute
# → 1.3.0 (stable)
# → Changesets archived ✅
```

**Prerelease NOT needed**: Simple, direct workflow.

---

### 7.2 Gitflow (feature → develop → main)

**Workflow**: Prereleases in develop, final release in main.

```bash
# Feature branch
git checkout -b feature/new-api
workspace changeset create --bump minor

# Merge to develop
git checkout develop
git merge feature/new-api

# Beta release in develop
workspace bump --execute --prerelease beta
# → 1.3.0-beta.0
# → Changesets KEPT (auto policy) ✅

# Bug fixes in develop
workspace changeset create --bump patch
workspace bump --execute --prerelease beta
# → 1.3.0-beta.1
# → Changesets KEPT ✅

# Release candidate
workspace bump --execute --prerelease rc
# → 1.3.0-rc.1
# → Changesets KEPT ✅

# Merge to main
git checkout main
git merge develop

# Final release
workspace bump --execute
# → 1.3.0 (stable)
# → Changesets ARCHIVED ✅
```

---

### 7.3 Custom Flow (feature → staging → production)

**Workflow**: Alpha in staging, stable in production.

```bash
# Feature
git checkout -b feature/X
workspace changeset create --bump minor

# Merge to staging
git checkout staging
git merge feature/X

# Alpha release for internal testing
workspace bump --execute --prerelease alpha
# → 1.3.0-alpha.0
# → Changesets KEPT ✅

# Production release
git checkout production
git merge staging
workspace bump --execute
# → 1.3.0
# → Changesets ARCHIVED ✅
```

---

### 7.4 Continuous Prerelease (always beta)

**Workflow**: Never release stable, always beta.

```bash
# Always maintain beta releases
workspace bump --execute --prerelease beta --no-archive
# → 1.3.0-beta.0
# → Changesets KEPT (explicit --no-archive) ✅

# Next bump
workspace bump --execute --prerelease beta --no-archive
# → 1.3.0-beta.1
# → Changesets KEPT ✅
```

---

## Implementation Checklist

### Phase 1: Types and Infrastructure
- [ ] Create `PrereleaseConfig` struct in `crates/pkg/src/types/prerelease.rs`
- [ ] Create `PrereleaseMode` enum in `crates/pkg/src/types/prerelease.rs`
- [ ] Create `ChangesetArchivePolicy` enum in `crates/cli/src/types/archive.rs`
- [ ] Add `Version::bump_with_prerelease()` method
- [ ] Add `Version::with_prerelease()` helper
- [ ] Add `Version::increment_prerelease()` helper
- [ ] Add `Version::remove_prerelease()` helper
- [ ] Unit tests for `Version::bump_with_prerelease()` (all modes)
- [ ] Unit tests for prerelease increment edge cases
- [ ] Unit tests for tag validation

### Phase 2: VersionResolver
- [ ] Add `resolve_versions_with_prerelease()` in `version/resolver.rs`
- [ ] Add `resolve_versions_with_prerelease()` in `version/resolution.rs`
- [ ] Implement `resolve_independent_with_prerelease()`
- [ ] Implement `resolve_unified_with_prerelease()`
- [ ] Unit tests for independent strategy with prerelease
- [ ] Unit tests for unified strategy with prerelease
- [ ] Integration tests for version resolution

### Phase 3: CLI Integration
- [ ] Modify `execute_bump_apply()` to use prerelease config
- [ ] Implement `parse_prerelease_args()`
- [ ] Implement `determine_archive_policy()`
- [ ] Implement `should_archive()`
- [ ] Add `--always-archive` flag to `BumpArgs`
- [ ] Update help text for `--prerelease` flag
- [ ] Update help text for archive flags
- [ ] E2E tests for prerelease bumps (create mode)
- [ ] E2E tests for prerelease bumps (increment mode)
- [ ] E2E tests for prerelease bumps (promote mode)
- [ ] E2E tests for archive policies (Auto, Never, Always)

### Phase 4: Edge Cases & Polish
- [ ] Auto-detect mode (Create vs Increment) based on current version
- [ ] Validation for prerelease tags (alpha, beta, rc)
- [ ] Clear error messages for invalid operations
- [ ] Human output shows prerelease info
- [ ] JSON output includes prerelease metadata
- [ ] Changelog generation works with prereleases
- [ ] Git tags work with prerelease versions
- [ ] Test tag conflict scenarios (alpha → beta transition)
- [ ] Test promotion scenarios (rc.1 → stable)
- [ ] Performance tests for large monorepos

### Phase 5: Documentation
- [ ] Update `crates/pkg/SPEC.md` with prerelease types
- [ ] Update CLI help text and examples
- [ ] Add usage examples to README
- [ ] Document archive policy behavior
- [ ] Add workflow examples (Gitflow, GitHub Flow, etc.)
- [ ] Update CHANGELOG with new features
- [ ] Create migration guide if needed
- [ ] Add troubleshooting section

### Phase 6: Final Validation
- [ ] All unit tests passing (100% coverage goal)
- [ ] All integration tests passing
- [ ] All E2E tests passing
- [ ] Clippy warnings resolved
- [ ] Format check passing
- [ ] Manual testing on real monorepo
- [ ] Test all documented workflows
- [ ] Performance validation
- [ ] Security review (input validation)
- [ ] Code review completed

---

## Risks and Mitigations

| Risk | Probability | Impact | Mitigation |
|------|------------|---------|-----------|
| Incorrect SemVer precedence | Medium | High | Use `semver::Version` directly, extensive tests |
| Tag conflicts (alpha vs beta) | Low | Medium | Validation on input, clear error messages |
| Archive policy confusion | Medium | Medium | Clear documentation, sensible defaults |
| Increment on stable version | High | Low | Validation + clear error message |
| Breaking change in internal API | High | Low | Only internal APIs affected, well-tested |
| Mode detection errors | Medium | Medium | Conservative defaults, allow explicit override |
| Dependency propagation issues | Low | High | Reuse existing propagation logic, isolated tests |
| Git tag format issues | Low | Medium | Follow existing tag format patterns |
| Performance regression | Very Low | Low | Minimal new logic, benchmark if needed |
| Workflow incompatibility | Low | High | Design is workflow-agnostic, documented examples |

---

## Summary

### 9.1 Key Features

✅ **Workflow-Agnostic**: Works with any branching strategy  
✅ **SemVer 2.0.0 Compliant**: Follows specification exactly  
✅ **Smart Defaults**: Auto archive policy handles common cases  
✅ **Explicit Control**: Flags available for override when needed  
✅ **Backward Compatible**: No breaking changes to existing workflows  
✅ **Well-Tested**: Comprehensive test coverage planned  
✅ **Clear Documentation**: Examples for different workflows  

### 9.2 Success Metrics

- [ ] All documented use cases work correctly
- [ ] 100% test coverage for new code
- [ ] Zero clippy warnings
- [ ] Complete documentation
- [ ] Manual validation on different workflows
- [ ] Performance acceptable (< 5% overhead)
- [ ] Security validated (input sanitization)
- [ ] Team approval and code review

### 9.3 Implementation Timeline

**Estimated Effort**: 3-5 days

- **Phase 1**: Types & Infrastructure (0.5 day)
- **Phase 2**: VersionResolver (1 day)
- **Phase 3**: CLI Integration (1 day)
- **Phase 4**: Edge Cases & Polish (1 day)
- **Phase 5**: Documentation (0.5 day)
- **Phase 6**: Final Validation (0.5-1 day)

---

## Next Steps

1. **Review this plan** with the team
2. **Approve architecture** and approach
3. **Begin Phase 1** implementation
4. **Iterate** with regular reviews
5. **Test thoroughly** at each phase
6. **Document** as you go
7. **Validate** with real workflows

---

**Status**: Ready for Implementation 🚀

This solution provides **robust, flexible prerelease support** without assuming any specific workflow. Each component has clear responsibilities and can be tested independently.
