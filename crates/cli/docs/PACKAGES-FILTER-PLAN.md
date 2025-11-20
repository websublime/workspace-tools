# Package Filtering in Bump Command - Implementation Plan

**Version**: 1.0  
**Date**: 2025-01-20  
**Status**: Planning Complete  
**Author**: Development Team

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current System State](#current-system-state)
3. [Problem Analysis](#problem-analysis)
4. [Proposed Architecture](#proposed-architecture)
5. [Implementation Details](#implementation-details)
6. [Use Cases](#use-cases)
7. [Implementation Checklist](#implementation-checklist)
8. [Risks and Mitigations](#risks-and-mitigations)
9. [Summary](#summary)

---

## Executive Summary

### Problem Statement

The `workspace bump` command currently ignores the `--packages` flag, which exists in the CLI arguments but has no implementation. This prevents users from:

1. Bumping only specific packages while leaving others unchanged
2. Creating partial releases in monorepos
3. Testing version bumps on subset of packages
4. Performing emergency hotfixes on individual packages

**Current Behavior:**
```bash
workspace bump --execute --packages @org/core,@org/utils
# ❌ Flag is silently ignored
# ✅ ALL packages from changesets are bumped
```

**Expected Behavior:**
```bash
workspace bump --execute --packages @org/core,@org/utils
# ✅ Only @org/core and @org/utils are bumped
# ✅ Other packages remain unchanged
# ✅ Dependency propagation still applies (optional)
```

### Solution Overview

Implement package filtering with support for:

✅ **Explicit Package Selection**: Specify exact packages to bump  
✅ **Changeset Intersection**: Filter packages within changesets  
✅ **Dependency-Aware Mode**: Optionally include dependencies  
✅ **Strategy Compatibility**: Works with Independent and Unified strategies  
✅ **Validation**: Clear errors for non-existent packages  
✅ **Backward Compatible**: No breaking changes when flag is not used  

---

## Current System State

### 2.1 CLI Argument Exists But Is Unused

**File**: `crates/cli/src/cli/commands.rs:264`

```rust
#[derive(Debug, Args)]
pub struct BumpArgs {
    // ... other fields ...
    
    /// Comma-separated list of packages to bump.
    ///
    /// Overrides changeset packages.
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    pub packages: Option<Vec<String>>,  // ❌ Defined but NEVER used!
    
    // ... other fields ...
}
```

**Usage in Implementation** (`crates/cli/src/commands/bump/execute.rs`):

```rust
pub async fn execute_bump_apply(
    args: &BumpArgs,  // ← args.packages exists here
    output: &Output,
    root: &Path,
    config_path: Option<&Path>,
) -> Result<()> {
    // ... code ...
    
    // Load changesets
    let loaded_changesets = manager.list_pending().await?;
    
    // Merge changesets
    let merged_changeset = merge_changesets(&loaded_changesets)?;
    
    // Resolve versions
    let resolution = resolver.resolve_versions(&merged_changeset).await?;
    // ❌ args.packages is NEVER referenced or used!
    
    // ... rest of code ...
}
```

### 2.2 Current Versioning Strategy Behavior

#### Independent Strategy

**Current Behavior:**
- Only packages listed in `changeset.packages` are bumped
- Other workspace packages remain unchanged
- Dependency propagation may bump additional packages

**With `--packages` Filter:**
- Should filter the changeset packages further
- Only bump packages that are BOTH in changeset AND in filter
- Preserve dependency propagation behavior

#### Unified Strategy

**Current Behavior:**
- ALL workspace packages receive the same version bump
- All packages bumped together regardless of changeset

**With `--packages` Filter:**
- Should override unified behavior
- Only bump specified packages
- Effectively converts Unified → Independent for this operation

### 2.3 Dependency Propagation

**Current Behavior** (`crates/pkg/src/version/propagation.rs`):

```rust
impl DependencyPropagator {
    pub fn propagate(&self, resolution: &mut VersionResolution) -> VersionResult<()> {
        // Automatically bumps packages whose dependencies changed
        // Based on config.dependency.propagation_bump setting
    }
}
```

**Interaction with Package Filter:**
- Should propagation include filtered-out packages?
- Should propagation be completely disabled when filtering?
- Should there be a flag to control this?

---

## Problem Analysis

### 3.1 Core Questions

**Q1: What does `--packages` mean?**

**A**: It should specify the EXACT packages to bump, overriding changeset packages.

**Q2: How does it interact with versioning strategies?**

**A**: 
- **Independent**: Filters changeset packages (intersection)
- **Unified**: Overrides unified behavior (becomes independent-like)

**Q3: What about dependency propagation?**

**A**: Two modes:
- **Strict**: Only bump specified packages (ignore dependencies)
- **Include Dependencies**: Bump specified packages + their dependencies

**Q4: Should it work with `--prerelease`?**

**A**: Yes, both flags should be orthogonal and work together.

**Q5: What if a specified package isn't in any changeset?**

**A**: 
- **Independent**: Error (no changeset defines bump for this package)
- **Unified**: Allow (changeset defines bump type for all packages)

### 3.2 Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Filter Mode** | Intersection with changesets | Respects changeset workflow |
| **Unified Override** | Allow filtering | Enables partial releases |
| **Dependency Mode** | Default = strict, optional flag for deps | Clear, predictable behavior |
| **Validation** | Fail on non-existent packages | Catch typos early |
| **Prerelease Compat** | Full compatibility | Orthogonal features |
| **Archive Policy** | Same as non-filtered | Consistent behavior |

---

## Proposed Architecture

### 4.1 High-Level Flow

```
┌────────────────────────────────────────────────────────────┐
│  CLI: workspace bump --execute --packages @org/core        │
└─────────────────────┬──────────────────────────────────────┘
                      │
                      ↓
        ┌─────────────────────────────┐
        │  Parse --packages argument   │
        │  ["@org/core"]               │
        └──────────┬──────────────────┘
                   │
                   ↓
        ┌──────────────────────────────┐
        │  Load Changesets             │
        │  Changeset has:              │
        │  [@org/core, @org/utils]     │
        └──────────┬───────────────────┘
                   │
                   ↓
        ┌──────────────────────────────┐
        │  Apply Package Filter        │
        │  Intersection:               │
        │  [@org/core]                 │
        └──────────┬───────────────────┘
                   │
                   ↓
        ┌──────────────────────────────┐
        │  Create Filtered Changeset   │
        │  packages: [@org/core]       │
        │  bump: Minor                 │
        └──────────┬───────────────────┘
                   │
                   ↓
        ┌──────────────────────────────┐
        │  VersionResolver             │
        │  resolve_versions()          │
        └──────────┬───────────────────┘
                   │
                   ↓
        ┌──────────────────────────────┐
        │  Apply Versions              │
        │  Update package.json files   │
        └──────────────────────────────┘
```

### 4.2 New Components

#### **PackageFilter**

```rust
/// Package filter for selective version bumping.
///
/// # What
///
/// Filters which packages should receive version bumps based on user
/// selection, respecting versioning strategy and dependency relationships.
///
/// # Why
///
/// Enables selective releases, testing, and emergency hotfixes by allowing
/// users to bump only specific packages in a monorepo.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::types::PackageFilter;
///
/// let filter = PackageFilter::new(vec!["@org/core".to_string()]);
/// let should_bump = filter.should_bump("@org/core");  // true
/// let should_bump = filter.should_bump("@org/utils"); // false
/// ```
#[derive(Debug, Clone)]
pub struct PackageFilter {
    /// List of package names to include.
    packages: HashSet<String>,
    
    /// Whether to include dependencies of filtered packages.
    include_dependencies: bool,
}

impl PackageFilter {
    /// Creates a new package filter.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package names to include
    /// * `include_dependencies` - Whether to include dependencies
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::types::PackageFilter;
    ///
    /// let filter = PackageFilter::new(
    ///     vec!["@org/core".to_string()],
    ///     false, // strict mode
    /// );
    /// ```
    pub fn new(packages: Vec<String>, include_dependencies: bool) -> Self {
        Self {
            packages: packages.into_iter().collect(),
            include_dependencies,
        }
    }

    /// Checks if a package should be bumped.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Package name to check
    ///
    /// # Returns
    ///
    /// True if the package should be bumped based on filter rules.
    pub fn should_bump(&self, package_name: &str) -> bool {
        self.packages.contains(package_name)
    }

    /// Applies filter to a changeset.
    ///
    /// # What
    ///
    /// Creates a new changeset with only the filtered packages.
    ///
    /// # Arguments
    ///
    /// * `changeset` - Original changeset
    ///
    /// # Returns
    ///
    /// Filtered changeset with subset of packages.
    pub fn apply_to_changeset(&self, changeset: &Changeset) -> Changeset {
        let mut filtered = changeset.clone();
        
        filtered.packages = changeset
            .packages
            .iter()
            .filter(|pkg| self.should_bump(pkg))
            .cloned()
            .collect();
        
        filtered
    }

    /// Validates that all filter packages exist in workspace.
    ///
    /// # Arguments
    ///
    /// * `available_packages` - List of package names in workspace
    ///
    /// # Errors
    ///
    /// Returns error if any filter package doesn't exist in workspace.
    pub fn validate(&self, available_packages: &[String]) -> Result<()> {
        let available: HashSet<_> = available_packages.iter().collect();
        
        for pkg in &self.packages {
            if !available.contains(pkg) {
                return Err(CliError::validation(format!(
                    "Package '{}' not found in workspace. Available packages: {}",
                    pkg,
                    available_packages.join(", ")
                )));
            }
        }
        
        Ok(())
    }
}
```

---

## Implementation Details

### 5.1 Modify CLI Bump Command

**File**: `crates/cli/src/commands/bump/execute.rs`

**Add package filtering:**

```rust
pub async fn execute_bump_apply(
    args: &BumpArgs,
    output: &Output,
    root: &Path,
    config_path: Option<&Path>,
) -> Result<()> {
    let workspace_root = root;
    info!("Executing bump apply in workspace: {}", workspace_root.display());

    // ... existing git validation code ...

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

    // Step 4: Create VersionResolver
    let resolver = VersionResolver::new(workspace_root.to_path_buf(), config.clone())
        .await
        .map_err(|e| CliError::execution(format!("Failed to create version resolver: {e}")))?;

    // ✅ Step 5: Apply package filter if specified
    let filtered_changesets = if let Some(ref package_list) = args.packages {
        debug!("Applying package filter: {:?}", package_list);
        
        // Discover all workspace packages for validation
        let all_packages = resolver.discover_packages().await
            .map_err(|e| CliError::execution(format!("Failed to discover packages: {e}")))?;
        
        let package_names: Vec<String> = all_packages
            .iter()
            .map(|p| p.name().to_string())
            .collect();
        
        // Create and validate filter
        let filter = PackageFilter::new(package_list.clone(), false);
        filter.validate(&package_names)?;
        
        // Apply filter to changesets
        let mut filtered = Vec::new();
        for changeset in &loaded_changesets {
            let filtered_cs = filter.apply_to_changeset(changeset);
            
            // Only include if filter left some packages
            if !filtered_cs.packages.is_empty() {
                filtered.push(filtered_cs);
            }
        }
        
        if filtered.is_empty() {
            return Err(CliError::validation(
                "No packages match the filter. No packages to bump.".to_string()
            ));
        }
        
        info!(
            "Package filter applied: {} package(s) selected from {} changeset(s)",
            package_list.len(),
            filtered.len()
        );
        
        filtered
    } else {
        loaded_changesets
    };

    // Step 6: Merge changesets (now using filtered changesets)
    let merged_changeset = merge_changesets(&filtered_changesets)?;

    // Step 7: Parse prerelease configuration
    let prerelease_config = parse_prerelease_args(args, &filtered_changesets)?;

    // Step 8: Resolve versions
    let resolution = resolver
        .resolve_versions_with_prerelease(&merged_changeset, prerelease_config.as_ref())
        .await
        .map_err(|e| CliError::execution(format!("Failed to resolve versions: {e}")))?;

    if resolution.updates.is_empty() {
        // ... handle no updates ...
        return Ok(());
    }

    // ... rest of implementation (confirmation, apply, changelog, archive, git) ...

    Ok(())
}
```

### 5.2 Add Dependency-Aware Mode (Future Enhancement)

**Add new flag to BumpArgs:**

```rust
// crates/cli/src/cli/commands.rs

#[derive(Debug, Args)]
pub struct BumpArgs {
    // ... existing fields ...

    /// Comma-separated list of packages to bump.
    ///
    /// Overrides changeset packages. Only specified packages will be bumped.
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    pub packages: Option<Vec<String>>,

    /// Include dependencies of filtered packages.
    ///
    /// When using --packages, also bump packages that depend on the filtered
    /// packages. Only applies when --packages is specified.
    #[arg(long, requires = "packages")]
    pub include_dependencies: bool,
}
```

**Modify filtering logic:**

```rust
// Create filter with dependency mode
let filter = PackageFilter::new(
    package_list.clone(),
    args.include_dependencies,  // ✅ Use flag
);
```

### 5.3 Update Preview and Snapshot Commands

**File**: `crates/cli/src/commands/bump/preview.rs`

Apply same filtering logic:

```rust
pub async fn execute_bump_preview(
    args: &BumpArgs,
    output: &Output,
    root: &Path,
    config_path: Option<&Path>,
) -> Result<()> {
    // ... existing code ...

    // Load changesets
    let changesets = manager.list_pending().await?;

    // ✅ Apply package filter if specified
    let filtered_changesets = if let Some(ref package_list) = args.packages {
        let all_packages = resolver.discover_packages().await?;
        let package_names: Vec<String> = all_packages
            .iter()
            .map(|p| p.name().to_string())
            .collect();
        
        let filter = PackageFilter::new(package_list.clone(), args.include_dependencies);
        filter.validate(&package_names)?;
        
        changesets
            .iter()
            .map(|cs| filter.apply_to_changeset(cs))
            .filter(|cs| !cs.packages.is_empty())
            .collect()
    } else {
        changesets
    };

    // ... rest of preview logic ...
}
```

**File**: `crates/cli/src/commands/bump/snapshot.rs`

Same filtering pattern applies to snapshot generation.

---

## Use Cases

### 6.1 Emergency Hotfix (Single Package)

**Scenario**: Critical bug in `@org/core`, need to release only that package.

```bash
# Current changeset has multiple packages
workspace changeset list
# Changeset: feature/bug-fix
#   Packages: @org/core, @org/utils, @org/api
#   Bump: patch

# Bump only @org/core
workspace bump --execute --packages @org/core
# ✅ Only @org/core bumped: 1.2.3 → 1.2.4
# ✅ @org/utils and @org/api remain unchanged
# ✅ Changeset archived (only @org/core removed from it)
```

### 6.2 Staged Release (Multiple Packages)

**Scenario**: Release frontend packages first, backend later.

```bash
# Changeset has all packages
workspace changeset show feature/new-ui
# Packages: @org/web, @org/mobile, @org/api, @org/database

# Release frontend packages first
workspace bump --execute --packages @org/web,@org/mobile
# ✅ @org/web and @org/mobile bumped
# ✅ @org/api and @org/database unchanged

# Later: release backend
workspace bump --execute --packages @org/api,@org/database
# ✅ @org/api and @org/database bumped
```

### 6.3 Testing Version Bump (Preview)

**Scenario**: Preview what would happen if only specific packages bumped.

```bash
# Preview full changeset
workspace bump --dry-run
# Shows all 5 packages will bump

# Preview filtered
workspace bump --dry-run --packages @org/core
# Shows only @org/core will bump
# ✅ Safe to test without applying
```

### 6.4 Unified Strategy Override

**Scenario**: Workspace uses Unified strategy, but need to bump only one package.

```bash
# Unified strategy normally bumps ALL packages
workspace bump --execute
# Bumps all 10 packages together

# Override with filter
workspace bump --execute --packages @org/core
# ✅ Only @org/core bumped
# ✅ Temporarily bypasses unified strategy
```

### 6.5 Prerelease + Filter Combination

**Scenario**: Create prerelease for subset of packages.

```bash
# Beta release for specific packages
workspace bump --execute --prerelease beta --packages @org/web,@org/api
# ✅ @org/web: 1.2.3 → 1.3.0-beta.0
# ✅ @org/api: 2.0.0 → 2.1.0-beta.0
# ✅ Other packages unchanged
```

---

## Implementation Checklist

### Phase 1: Core Infrastructure
- [ ] Create `PackageFilter` struct in `crates/cli/src/types/filter.rs`
- [ ] Implement `PackageFilter::new()`
- [ ] Implement `PackageFilter::should_bump()`
- [ ] Implement `PackageFilter::apply_to_changeset()`
- [ ] Implement `PackageFilter::validate()`
- [ ] Unit tests for `PackageFilter` (all methods)
- [ ] Unit tests for edge cases (empty filter, all filtered out, etc.)

### Phase 2: CLI Integration
- [ ] Modify `execute_bump_apply()` to use package filter
- [ ] Add package filter logic to `execute_bump_preview()`
- [ ] Add package filter logic to `execute_bump_snapshot()`
- [ ] Update help text for `--packages` flag
- [ ] Add validation error messages
- [ ] E2E test: bump with single package filter
- [ ] E2E test: bump with multiple package filter
- [ ] E2E test: filter with non-existent package (should error)
- [ ] E2E test: filter with no matching packages (should error)

### Phase 3: Strategy Compatibility
- [ ] Test filter with Independent strategy
- [ ] Test filter with Unified strategy (override behavior)
- [ ] Integration test: Independent + filter
- [ ] Integration test: Unified + filter
- [ ] Verify dependency propagation behavior
- [ ] Document strategy interaction

### Phase 4: Advanced Features (Optional)
- [ ] Add `--include-dependencies` flag to `BumpArgs`
- [ ] Implement dependency-aware filtering in `PackageFilter`
- [ ] Tests for dependency inclusion
- [ ] Document dependency mode

### Phase 5: Compatibility Testing
- [ ] Test `--packages` + `--prerelease` combination
- [ ] Test `--packages` + `--snapshot` combination
- [ ] Test `--packages` + `--no-archive` combination
- [ ] Test `--packages` + `--force` combination
- [ ] Verify all flag combinations work correctly

### Phase 6: Edge Cases & Polish
- [ ] Handle empty filter results gracefully
- [ ] Clear error for typos in package names
- [ ] Warning if filter excludes all changeset packages
- [ ] Human output shows filter applied
- [ ] JSON output includes filter metadata
- [ ] Performance test with large package lists
- [ ] Test case-sensitivity handling

### Phase 7: Documentation
- [ ] Update `crates/cli/SPEC.md` with package filter
- [ ] Update CLI help text with examples
- [ ] Add usage examples to README
- [ ] Document strategy interaction
- [ ] Add troubleshooting section
- [ ] Update CHANGELOG

### Phase 8: Final Validation
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] All E2E tests passing
- [ ] Clippy warnings resolved
- [ ] Format check passing
- [ ] Manual testing on real monorepo
- [ ] Code review completed

---

## Risks and Mitigations

| Risk | Probability | Impact | Mitigation |
|------|------------|---------|-----------|
| Filter excludes all packages | Medium | Medium | Validation + clear error message |
| Typo in package name | High | Low | Validation against workspace packages |
| Confusion with unified strategy | Medium | Medium | Clear documentation, warning messages |
| Dependency propagation issues | Low | Medium | Maintain existing propagation logic |
| Partial changeset archival | Low | High | Document behavior clearly |
| Breaking change concerns | Very Low | Low | Purely additive, backward compatible |
| Performance with large filters | Very Low | Low | HashSet for O(1) lookup |
| Flag interaction bugs | Medium | Medium | Comprehensive E2E tests |

---

## Summary

### 9.1 Key Features

✅ **Explicit Package Selection**: Users specify exactly which packages to bump  
✅ **Changeset Integration**: Works within existing changeset workflow  
✅ **Strategy Compatible**: Works with both Independent and Unified  
✅ **Validation**: Catches errors early (typos, non-existent packages)  
✅ **Backward Compatible**: No breaking changes when flag not used  
✅ **Orthogonal**: Works with prerelease, snapshot, and other flags  

### 9.2 Success Metrics

- [ ] All documented use cases work correctly
- [ ] Clear error messages for all failure scenarios
- [ ] 100% test coverage for new code
- [ ] Zero clippy warnings
- [ ] Complete documentation
- [ ] Manual validation on monorepo
- [ ] Team approval

### 9.3 Implementation Timeline

**Estimated Effort**: 2-3 days

- **Phase 1**: Core Infrastructure (0.5 day)
- **Phase 2**: CLI Integration (1 day)
- **Phase 3**: Strategy Compatibility (0.5 day)
- **Phase 4**: Advanced Features (0.5 day, optional)
- **Phase 5**: Compatibility Testing (0.5 day)
- **Phase 6**: Edge Cases & Polish (0.5 day)
- **Phase 7**: Documentation (0.5 day)
- **Phase 8**: Final Validation (0.5 day)

---

## Next Steps

1. **Review plan** with team
2. **Approve approach** and design decisions
3. **Begin Phase 1** implementation
4. **Test incrementally** at each phase
5. **Document** as you go
6. **Validate** with real workflows

---

**Status**: Ready for Implementation 🚀

This solution provides **flexible package filtering** that respects existing workflows while enabling selective releases for emergency situations and staged rollouts.
