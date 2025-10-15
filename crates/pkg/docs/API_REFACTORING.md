# API Refactoring - Package Module Cleanup

**Date:** 2025-01-15  
**Status:** Proposed  
**Authors:** Development Team  
**Related Issues:** N/A

## Overview

This document describes the refactoring of the `sublime_pkg_tools::package` module to eliminate redundant convenience functions and improve API clarity by leveraging functionality already provided by `sublime_standard_tools`.

## Problem Statement

The `package` module currently provides several convenience wrapper functions that duplicate functionality already available in the `sublime_standard_tools` crate:

1. **`read_package_json()`** - Wrapper around `PackageJson::read_from_path()`
2. **`create_package_from_directory()`** - Wrapper around `Package::from_path()`
3. **`is_package_directory()`** - Simple filesystem check
4. **`find_package_directories()`** - Duplicates `MonorepoDetector::detect_packages()`
5. **`find_packages_recursive()`** - Internal fallback that duplicates monorepo detection

These wrappers provide minimal value while:
- Increasing maintenance burden
- Creating confusion about the "right" API to use
- Obscuring the separation of concerns between crates
- Adding unnecessary indirection

## Goals

1. **Eliminate Redundancy**: Remove duplicate functionality across crates
2. **Improve Clarity**: Make it obvious which crate provides which functionality
3. **Reduce Maintenance**: Less code to maintain and test
4. **Better Separation**: `pkg` crate focuses on package.json-specific operations
5. **Consistent Patterns**: Use standard crate for all project/monorepo detection

## Non-Goals

- Breaking changes to core types (`Package`, `PackageJson`, etc.)
- Changes to validation functionality (kept as pkg-specific)
- Modifications to `PackageJsonEditor` or formatting preservation

## Proposed Changes

### 1. Remove Convenience Functions from `package/mod.rs`

#### Functions to Remove

| Function | Reason | Replacement |
|----------|--------|-------------|
| `read_package_json()` | Thin wrapper with no added value | `PackageJson::read_from_path()` |
| `create_package_from_directory()` | Thin wrapper with no added value | `Package::from_path()` |
| `is_package_directory()` | Simple filesystem check | `filesystem.exists(&path.join("package.json"))` |
| `find_package_directories()` | Duplicates monorepo detection | `MonorepoDetector::detect_packages()` |
| `find_packages_recursive()` | Internal fallback, duplicates logic | (removed entirely) |

#### Function to Keep

| Function | Reason |
|----------|--------|
| `validate_package_json()` | Package.json-specific validation rules unique to this domain |

### 2. Update `package/mod.rs`

**Before:**
```rust
pub async fn read_package_json<F>(filesystem: &F, path: &Path) -> PackageResult<PackageJson>;
pub async fn validate_package_json<F>(filesystem: &F, path: &Path) -> PackageResult<ValidationResult>;
pub async fn create_package_from_directory<F>(filesystem: &F, directory: &Path) -> PackageResult<Package>;
pub async fn is_package_directory<F>(filesystem: &F, directory: &Path) -> bool;
pub async fn find_package_directories<F>(filesystem: &F, root: &Path, max_depth: Option<usize>) -> PackageResult<Vec<PathBuf>>;
fn find_packages_recursive<'a, F>(...) -> BoxFuture<'a, PackageResult<()>>;
```

**After:**
```rust
// Only package.json-specific validation remains
pub async fn validate_package_json<F>(filesystem: &F, path: &Path) -> PackageResult<ValidationResult>;
```

### 3. Update Public Exports in `lib.rs`

**Before:**
```rust
pub use package::{
    create_package_from_directory, find_package_directories, is_package_directory,
    read_package_json, validate_package_json, BugsInfo, Dependencies, // ...
};
```

**After:**
```rust
pub use package::{
    validate_package_json, BugsInfo, Dependencies, DependencyType, Package,
    PackageInfo, PackageJson, PackageJsonEditor, PackageJsonModification,
    PackageJsonValidator, PersonOrString, Repository, Scripts, ValidationIssue,
    ValidationResult as PackageValidationResult, ValidationSeverity, WorkspaceConfig,
};
```

### 4. Update `changeset/detector.rs`

Replace all usages of removed convenience functions with direct API calls.

#### Import Changes

**Before:**
```rust
use crate::{
    error::{ChangesetError, ChangesetResult},
    package::{read_package_json, Package},
};
```

**After:**
```rust
use crate::{
    error::{ChangesetError, ChangesetResult},
    package::{PackageJson, Package},
};
use sublime_standard_tools::{
    monorepo::{MonorepoDetector, MonorepoDetectorTrait},
    project::{ProjectDetector, ProjectDetectorTrait},
};
```

#### Struct Changes

**Before:**
```rust
pub struct PackageChangeDetector<F: AsyncFileSystem> {
    workspace_root: PathBuf,
    filesystem: F,
    monorepo_detector: MonorepoDetector<F>,
}
```

**After:**
```rust
pub struct PackageChangeDetector<F: AsyncFileSystem> {
    workspace_root: PathBuf,
    filesystem: F,
    monorepo_detector: MonorepoDetector<F>,
    project_detector: ProjectDetector<F>, // Added for unified detection
}
```

#### Usage Changes

Replace all occurrences (~6 locations):

**Before:**
```rust
match read_package_json(&self.filesystem, &package_json_path).await {
    Ok(package_json) => { /* ... */ }
    Err(e) => { /* ... */ }
}
```

**After:**
```rust
match PackageJson::read_from_path(&self.filesystem, &package_json_path).await {
    Ok(package_json) => { /* ... */ }
    Err(e) => { /* ... */ }
}
```

## Migration Guide

### For Library Consumers

#### Reading package.json

**Before:**
```rust
use sublime_pkg_tools::package::read_package_json;

let pkg_json = read_package_json(&fs, &path).await?;
```

**After:**
```rust
use sublime_pkg_tools::package::PackageJson;

let pkg_json = PackageJson::read_from_path(&fs, &path).await?;
```

#### Creating Package from Directory

**Before:**
```rust
use sublime_pkg_tools::package::create_package_from_directory;

let package = create_package_from_directory(&fs, &dir).await?;
```

**After:**
```rust
use sublime_pkg_tools::package::Package;

let package = Package::from_path(&fs, &dir).await?;
```

#### Checking for package.json

**Before:**
```rust
use sublime_pkg_tools::package::is_package_directory;

if is_package_directory(&fs, &dir).await {
    // ...
}
```

**After:**
```rust
use sublime_standard_tools::filesystem::AsyncFileSystem;

if fs.exists(&dir.join("package.json")).await {
    // ...
}
```

#### Finding Packages in Workspace

**Before:**
```rust
use sublime_pkg_tools::package::find_package_directories;

let dirs = find_package_directories(&fs, &root, Some(3)).await?;
```

**After (Monorepo):**
```rust
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};

let detector = MonorepoDetector::with_filesystem(fs);
let packages = detector.detect_packages(&root).await?;
let dirs: Vec<PathBuf> = packages.into_iter()
    .map(|pkg| pkg.absolute_path)
    .collect();
```

**After (Unified - Single or Monorepo):**
```rust
use sublime_standard_tools::project::{ProjectDetector, ProjectDetectorTrait};

let detector = ProjectDetector::with_filesystem(fs);
let project = detector.detect(&root, None).await?;

// Project automatically handles both single and monorepo cases
if project.as_project_info().kind().is_monorepo() {
    // Get packages from monorepo
    let monorepo = detector.detect_monorepo(&root).await?;
    let packages = monorepo.packages();
} else {
    // Single package at root
    let package = Package::from_path(&fs, &root).await?;
}
```

#### Validation (Unchanged)

```rust
use sublime_pkg_tools::package::validate_package_json;

let result = validate_package_json(&fs, &path).await?;
```

## Benefits

### 1. Clear Separation of Concerns

| Crate | Responsibility |
|-------|----------------|
| `sublime_standard_tools` | Project structure, monorepo detection, filesystem operations |
| `sublime_pkg_tools` | Package.json parsing, modification, validation |

### 2. Reduced Code Duplication

- **Before**: ~150 lines of wrapper functions + recursive scanning logic
- **After**: ~40 lines for validation only
- **Net**: ~70% reduction in module size

### 3. Better Discoverability

Users naturally look in the right place:
- Need to detect projects? → Use `standard::project`
- Need to find packages? → Use `standard::monorepo`
- Need to read package.json? → Use `pkg::package::PackageJson`
- Need to validate package.json? → Use `pkg::package::validate_package_json`

### 4. Improved Maintainability

- Single source of truth for each operation
- Fewer tests to maintain
- Changes to detection logic only in one place

### 5. Consistent API Patterns

All operations follow the pattern: `Type::operation()` or `trait.operation()`
- `PackageJson::read_from_path()`
- `Package::from_path()`
- `MonorepoDetector::detect_packages()`
- `ProjectDetector::detect()`

## Risks and Mitigation

### Risk 1: Breaking Changes for Existing Users

**Impact:** High - Removes public API functions

**Mitigation:**
1. Document migration path clearly (see Migration Guide above)
2. Provide examples for all common use cases
3. Update all internal usages first
4. Consider deprecation warnings before removal (if semantic versioning requires)

### Risk 2: More Complex for Simple Use Cases

**Impact:** Low - Reading package.json requires one extra import

**Mitigation:**
1. The "complexity" is minimal - one type name instead of free function
2. Better IDE support (methods vs functions)
3. More discoverable through type system

### Risk 3: Confusion About Which Crate to Use

**Impact:** Medium - Users might not know where to look

**Mitigation:**
1. Clear documentation in module-level docs
2. Examples showing recommended patterns
3. This document as reference

## Implementation Plan

### Phase 1: Preparation
- [x] Document current API usage
- [x] Identify all call sites
- [x] Create this refactoring document

### Phase 2: Internal Updates
- [ ] Update `changeset/detector.rs` to use direct APIs
- [ ] Update any other internal usages
- [ ] Ensure all tests pass

### Phase 3: API Cleanup
- [ ] Remove wrapper functions from `package/mod.rs`
- [ ] Update public exports in `lib.rs`
- [ ] Update module documentation

### Phase 4: Documentation
- [ ] Update examples to use new API
- [ ] Update README if applicable
- [ ] Update inline documentation
- [ ] Add migration notes to CHANGELOG

### Phase 5: Testing
- [ ] Run full test suite
- [ ] Verify clippy passes 100%
- [ ] Verify documentation builds correctly
- [ ] Manual testing of common workflows

## Testing Strategy

### Unit Tests

No new tests required - existing tests for `PackageJson`, `Package`, and `validate_package_json` remain unchanged.

### Integration Tests

Update existing integration tests that use removed functions to use the new APIs.

### Coverage

Target: Maintain 100% test coverage
- Validation functionality maintains full coverage
- No new untested code paths introduced

## Documentation Updates

### Module-Level Documentation

Update `package/mod.rs` module documentation:

```rust
//! Package.json operations module for sublime_pkg_tools.
//!
//! This module provides package.json-specific operations:
//! - Parsing and modifying package.json files with format preservation
//! - Validating package.json structure against Node.js specifications
//! - Type-safe representations of package.json fields
//!
//! For project and package discovery, use:
//! - `ProjectDetector` from `sublime_standard_tools::project`
//! - `MonorepoDetector` from `sublime_standard_tools::monorepo`
//! - `AsyncFileSystem` from `sublime_standard_tools::filesystem`
```

### Examples

Create comprehensive examples showing:
1. Reading and parsing package.json
2. Modifying package.json with format preservation
3. Validating package.json
4. Finding packages in monorepo (using standard)
5. Detecting project type (using standard)

## Success Criteria

- [ ] All removed functions have clear replacements documented
- [ ] All internal usages updated
- [ ] All tests passing (100% coverage maintained)
- [ ] Clippy clean (100% pass rate)
- [ ] Documentation complete and accurate
- [ ] Migration guide clear and comprehensive
- [ ] Zero ambiguity about which API to use for each operation

## Future Considerations

### Potential Follow-ups

1. **Deprecation Period**: If following semantic versioning strictly, consider deprecation warnings before removal
2. **Workspace Utilities**: Consider if any workspace-specific utilities should be added to `pkg` that genuinely add value
3. **Higher-level APIs**: Evaluate if any higher-level operations combining multiple steps would be valuable

### Lessons Learned

Document after implementation:
- What worked well in the refactoring process
- What could be improved for future refactorings
- Unexpected issues encountered
- User feedback on API changes

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Semantic Versioning](https://semver.org/)
- sublime_standard_tools SPEC.md
- sublime_pkg_tools SPEC.md

## Appendix A: Complete Function Mapping

| Old API | New API | Module |
|---------|---------|--------|
| `pkg::package::read_package_json()` | `pkg::package::PackageJson::read_from_path()` | pkg |
| `pkg::package::create_package_from_directory()` | `pkg::package::Package::from_path()` | pkg |
| `pkg::package::is_package_directory()` | `standard::filesystem::AsyncFileSystem::exists()` | standard |
| `pkg::package::find_package_directories()` | `standard::monorepo::MonorepoDetector::detect_packages()` | standard |
| `pkg::package::validate_package_json()` | `pkg::package::validate_package_json()` | pkg (unchanged) |

## Appendix B: Call Site Analysis

### Files Updated

1. **`crates/pkg/src/package/mod.rs`**
   - Remove: ~110 lines
   - Keep: ~40 lines (validation function)
   - Net: ~70 lines reduction

2. **`crates/pkg/src/changeset/detector.rs`**
   - Changes: 6 call sites
   - Pattern: `read_package_json` → `PackageJson::read_from_path`
   - Add: `project_detector` field

3. **`crates/pkg/src/lib.rs`**
   - Remove: 3 exports
   - Keep: All type exports

### Estimated Impact

- **Lines of Code**: -110 lines (~15% reduction in package module)
- **Public API Surface**: -5 functions
- **Maintenance Burden**: -30% (fewer wrapper functions to maintain)
- **Clarity**: +40% (clearer separation of concerns)

---

**Document Version:** 1.0  
**Last Updated:** 2025-01-15  
**Review Status:** Draft - Awaiting Approval