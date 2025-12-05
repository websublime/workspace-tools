# Parameter Analysis: CLI vs Node.js Documentation

**Date**: 2025-01-20  
**Purpose**: Compare CLI command parameters with Node.js documentation to identify gaps and inconsistencies.  
**Files Analyzed**:
- `crates/cli/src/cli/commands.rs` (Source of Truth)
- `crates/node/docs/NAPI_RESEARCH.md`
- `crates/node/docs/PLAN_NODE.md`
- `crates/node/docs/STORY_MAP_NODE.md`

---

## Executive Summary

### Issues Found and Resolution Status

| Category | Issue | Severity | Status |
|----------|-------|----------|--------|
| BumpArgs | Missing `always_archive` parameter | High | ✅ FIXED |
| BumpArgs | Prerelease/Snapshot not fully documented for Node params | High | ✅ FIXED |
| ChangesetCreateArgs | Missing `force` parameter | Medium | ✅ FIXED |
| Documentation | Incomplete TypeScript interface definitions | Medium | ✅ FIXED |
| Story Map | Missing detailed parameter specifications | Low | ✅ FIXED |

### Documents Updated

- `NAPI_RESEARCH.md`: Added `always_archive` to BumpArgs, `force` to ChangesetCreateArgs, added full bump function specifications (Section 11.3)
- `PLAN_NODE.md`: Added Phase 4 detailed specifications with prerelease/snapshot params, updated mapping table
- `STORY_MAP_NODE.md`: Updated Story 4.1, 4.2, 5.1-5.6 with detailed parameter specifications

---

## 1. Bump Command Analysis

### CLI Source (commands.rs)

```rust
pub struct BumpArgs {
    pub dry_run: bool,
    pub execute: bool,
    pub snapshot: bool,
    pub snapshot_format: Option<String>,
    pub prerelease: Option<String>,
    pub packages: Option<Vec<String>>,
    pub git_tag: bool,
    pub git_push: bool,
    pub git_commit: bool,
    pub no_changelog: bool,
    pub no_archive: bool,
    pub always_archive: bool,  // ⚠️ MISSING IN DOCS
    pub force: bool,
    pub show_diff: bool,
}
```

### NAPI_RESEARCH.md (Section 4.3) - ✅ UPDATED

```rust
pub struct BumpArgs {
    pub dry_run: bool,
    pub execute: bool,
    pub snapshot: bool,
    pub snapshot_format: Option<String>,
    pub prerelease: Option<String>,
    pub packages: Option<Vec<String>>,
    pub git_tag: bool,
    pub git_push: bool,
    pub git_commit: bool,
    pub no_changelog: bool,
    pub no_archive: bool,
    pub always_archive: bool,  // ✅ ADDED - Forces archiving even for prerelease versions
    pub force: bool,
    pub show_diff: bool,
}
```

**Additional Updates:**
- Added Section 11.3 with complete `bumpPreview`, `bumpApply`, `bumpSnapshot` specifications
- Added TypeScript interfaces for all bump params and response types
- Added prerelease/snapshot validators
- Added usage examples

### Required Node.js Parameters

Based on CLI analysis, the Node.js bindings should expose:

#### BumpPreviewParams

```typescript
interface BumpPreviewParams {
  root?: string;
  configPath?: string;
  packages?: string[];
  showDiff?: boolean;
}
```

#### BumpApplyParams

```typescript
interface BumpApplyParams {
  root?: string;
  configPath?: string;
  packages?: string[];
  
  // Git operations
  gitCommit?: boolean;
  gitTag?: boolean;
  gitPush?: boolean;
  
  // Prerelease support (NEW - not documented)
  prerelease?: string;  // alpha, beta, rc, or custom tag
  
  // Changelog/Archive control
  noChangelog?: boolean;
  noArchive?: boolean;
  alwaysArchive?: boolean;  // ⚠️ MISSING IN ALL DOCS
  
  // Behavior
  force?: boolean;
}
```

#### BumpSnapshotParams

```typescript
interface BumpSnapshotParams {
  root?: string;
  configPath?: string;
  packages?: string[];
  
  // Snapshot format
  format?: string;  // Template: {version}-snapshot.{short_commit}
}
```

### Gap Analysis - Bump (✅ ALL FIXED)

| Parameter | CLI | NAPI_RESEARCH | PLAN_NODE | STORY_MAP | Status |
|-----------|-----|---------------|-----------|-----------|--------|
| dry_run | ✅ | ✅ | N/A (internal) | N/A | OK |
| execute | ✅ | ✅ | N/A (internal) | N/A | OK |
| snapshot | ✅ | ✅ | ✅ | ✅ | OK |
| snapshot_format | ✅ | ✅ | ✅ | ✅ | OK |
| prerelease | ✅ | ✅ | ✅ | ✅ | ✅ FIXED |
| packages | ✅ | ✅ | ✅ | ✅ | OK |
| git_tag | ✅ | ✅ | ✅ | ✅ | OK |
| git_push | ✅ | ✅ | ✅ | ✅ | OK |
| git_commit | ✅ | ✅ | ✅ | ✅ | OK |
| no_changelog | ✅ | ✅ | ✅ | ✅ | ✅ FIXED |
| no_archive | ✅ | ✅ | ✅ | ✅ | ✅ FIXED |
| always_archive | ✅ | ✅ | ✅ | ✅ | ✅ FIXED |
| force | ✅ | ✅ | ✅ | ✅ | ✅ FIXED |
| show_diff | ✅ | ✅ | ✅ | ✅ | ✅ FIXED |

---

## 2. Changeset Commands Analysis

### CLI Source - ChangesetCreateArgs

```rust
pub struct ChangesetCreateArgs {
    pub bump: Option<String>,
    pub env: Option<Vec<String>>,
    pub branch: Option<String>,
    pub message: Option<String>,
    pub packages: Option<Vec<String>>,
    pub non_interactive: bool,
    pub force: bool,  // ⚠️ MISSING IN DOCS
}
```

### NAPI_RESEARCH.md - ✅ UPDATED

```rust
pub struct ChangesetCreateArgs {
    pub bump: Option<String>,
    pub env: Option<Vec<String>>,
    pub branch: Option<String>,
    pub message: Option<String>,
    pub packages: Option<Vec<String>>,
    pub non_interactive: bool,
    pub force: bool,  // ✅ ADDED - Overwrites existing changeset for the branch
}
```

### Required Node.js Parameters

#### ChangesetAddParams

```typescript
interface ChangesetAddParams {
  root?: string;
  configPath?: string;
  
  bump?: 'major' | 'minor' | 'patch';
  environments?: string[];
  branch?: string;
  message?: string;
  packages?: string[];
  force?: boolean;  // ⚠️ ADD - overwrites existing changeset
}
```

#### ChangesetUpdateParams

```typescript
interface ChangesetUpdateParams {
  root?: string;
  configPath?: string;
  
  id?: string;  // Changeset ID or branch name
  commit?: string;
  packages?: string[];
  bump?: 'major' | 'minor' | 'patch';
  environments?: string[];
}
```

#### ChangesetListParams

```typescript
interface ChangesetListParams {
  root?: string;
  configPath?: string;
  
  filterPackage?: string;
  filterBump?: 'major' | 'minor' | 'patch';
  filterEnv?: string;
  sort?: 'date' | 'bump' | 'branch';
}
```

#### ChangesetShowParams

```typescript
interface ChangesetShowParams {
  root?: string;
  configPath?: string;
  
  branch: string;  // Required
}
```

#### ChangesetRemoveParams

```typescript
interface ChangesetRemoveParams {
  root?: string;
  configPath?: string;
  
  branch: string;  // Required
  force?: boolean;
}
```

#### ChangesetHistoryParams

```typescript
interface ChangesetHistoryParams {
  root?: string;
  configPath?: string;
  
  filterPackage?: string;
  filterEnv?: string;
  filterBump?: 'major' | 'minor' | 'patch';
  since?: string;  // ISO 8601 date
  until?: string;  // ISO 8601 date
  limit?: number;
}
```

#### ChangesetCheckParams

```typescript
interface ChangesetCheckParams {
  root?: string;
  configPath?: string;
  
  branch?: string;
}
```

### Gap Analysis - Changeset (✅ ALL FIXED)

| Command | Parameter | CLI | NAPI_RESEARCH | PLAN_NODE | Status |
|---------|-----------|-----|---------------|-----------|--------|
| Create | force | ✅ | ✅ | ✅ | ✅ FIXED |
| Delete | force | ✅ | ✅ | ✅ | ✅ FIXED |

---

## 3. Upgrade Commands Analysis

### CLI Source - UpgradeCheckArgs

```rust
pub struct UpgradeCheckArgs {
    pub no_major: bool,
    pub no_minor: bool,
    pub no_patch: bool,
    pub no_dev: bool,
    pub peer: bool,
    pub packages: Option<Vec<String>>,
    pub registry: Option<String>,
}
```

### CLI Source - UpgradeApplyArgs

```rust
pub struct UpgradeApplyArgs {
    pub dry_run: bool,
    pub patch_only: bool,
    pub minor_and_patch: bool,
    pub packages: Option<Vec<String>>,
    pub auto_changeset: bool,
    pub changeset_bump: String,
    pub no_backup: bool,
    pub force: bool,
}
```

### Required Node.js Parameters

#### UpgradeCheckParams

```typescript
interface UpgradeCheckParams {
  root?: string;
  configPath?: string;
  
  // Exclusion flags
  noMajor?: boolean;
  noMinor?: boolean;
  noPatch?: boolean;
  noDev?: boolean;
  
  // Inclusion flags
  peer?: boolean;
  
  // Filters
  packages?: string[];
  registry?: string;
}
```

#### UpgradeApplyParams

```typescript
interface UpgradeApplyParams {
  root?: string;
  configPath?: string;
  
  // Scope restriction
  patchOnly?: boolean;
  minorAndPatch?: boolean;
  
  // Filters
  packages?: string[];
  
  // Changeset integration
  autoChangeset?: boolean;
  changesetBump?: 'major' | 'minor' | 'patch';
  
  // Backup control
  noBackup?: boolean;
  
  // Behavior
  force?: boolean;
}
```

#### BackupRestoreParams

```typescript
interface BackupRestoreParams {
  root?: string;
  configPath?: string;
  
  id: string;  // Required
  force?: boolean;
}
```

#### BackupCleanParams

```typescript
interface BackupCleanParams {
  root?: string;
  configPath?: string;
  
  keep?: number;  // Default: 5
  force?: boolean;
}
```

---

## 4. Audit Command Analysis

### CLI Source - AuditArgs

```rust
pub struct AuditArgs {
    pub sections: Vec<String>,
    pub output: Option<PathBuf>,
    pub min_severity: String,
    pub verbosity: String,
    pub no_health_score: bool,
    pub export: Option<String>,
    pub export_file: Option<PathBuf>,
}
```

### Required Node.js Parameters

```typescript
interface AuditParams {
  root?: string;
  configPath?: string;
  
  sections?: ('all' | 'upgrades' | 'dependencies' | 'version-consistency' | 'breaking-changes')[];
  minSeverity?: 'critical' | 'high' | 'medium' | 'low' | 'info';
  verbosity?: 'minimal' | 'normal' | 'detailed';
  noHealthScore?: boolean;
  
  // Note: output/export options may not be needed for API
  // as the data is returned directly
}
```

---

## 5. Changes Command Analysis

### CLI Source - ChangesArgs

```rust
pub struct ChangesArgs {
    pub since: Option<String>,
    pub until: Option<String>,
    pub branch: Option<String>,
    pub staged: bool,
    pub unstaged: bool,
    pub packages: Option<Vec<String>>,
}
```

### Required Node.js Parameters

```typescript
interface ChangesParams {
  root?: string;
  configPath?: string;
  
  since?: string;  // Git ref
  until?: string;  // Git ref
  branch?: string;
  
  staged?: boolean;
  unstaged?: boolean;
  
  packages?: string[];
}
```

---

## 6. Execute Command Analysis

### CLI Source - ExecuteArgs

```rust
pub struct ExecuteArgs {
    pub cmd: String,
    pub filter_package: Option<Vec<String>>,
    pub affected: bool,
    pub since: Option<String>,
    pub until: Option<String>,
    pub branch: Option<String>,
    pub parallel: bool,
    pub args: Vec<String>,
}
```

### NAPI_RESEARCH.md - ExecuteArgs

```rust
pub struct ExecuteArgs {
    pub cmd: String,
    pub filter_package: Option<Vec<String>>,
    pub affected: bool,
    pub since: Option<String>,
    pub until: Option<String>,
    pub branch: Option<String>,
    pub parallel: bool,
    pub args: Vec<String>,
}
```

**Status**: ✅ Correctly documented

### Required Node.js Parameters (with timeout extension)

```typescript
interface ExecuteParams {
  root?: string;
  configPath?: string;
  
  cmd: string;  // Required
  filterPackage?: string[];
  
  affected?: boolean;
  since?: string;
  until?: string;
  branch?: string;
  
  parallel?: boolean;
  args?: string[];
  
  // Timeout extensions (from config or parameter)
  timeoutSecs?: number;
  perPackageTimeoutSecs?: number;
}
```

---

## 7. Init Command Analysis

### CLI Source - InitArgs

```rust
pub struct InitArgs {
    pub changeset_path: PathBuf,
    pub environments: Option<Vec<String>>,
    pub default_env: Option<Vec<String>>,
    pub strategy: Option<String>,
    pub registry: String,
    pub config_format: Option<String>,
    pub force: bool,
    pub non_interactive: bool,
}
```

### Required Node.js Parameters

```typescript
interface InitParams {
  root?: string;
  
  changesetPath?: string;  // Default: .changesets
  environments?: string[];
  defaultEnv?: string[];
  strategy?: 'independent' | 'unified';
  registry?: string;  // Default: https://registry.npmjs.org
  configFormat?: 'json' | 'toml' | 'yaml';
  force?: boolean;
  // Note: non_interactive is always true for API
}
```

---

## 8. Clone Command Analysis

### CLI Source - CloneArgs

```rust
pub struct CloneArgs {
    pub url: String,
    pub destination: Option<PathBuf>,
    pub changeset_path: Option<String>,
    pub environments: Option<Vec<String>>,
    pub default_env: Option<Vec<String>>,
    pub strategy: Option<String>,
    pub registry: Option<String>,
    pub config_format: Option<String>,
    pub non_interactive: bool,
    pub skip_validation: bool,
    pub force: bool,
    pub depth: Option<u32>,
}
```

### Required Node.js Parameters

```typescript
interface CloneParams {
  url: string;  // Required
  destination?: string;
  
  // Init options (used if no config found)
  changesetPath?: string;
  environments?: string[];
  defaultEnv?: string[];
  strategy?: 'independent' | 'unified';
  registry?: string;
  configFormat?: 'json' | 'toml' | 'yaml';
  
  // Behavior
  skipValidation?: boolean;
  force?: boolean;
  depth?: number;  // Shallow clone depth
}
```

---

## 9. Status Command Analysis

### CLI Source - StatusArgs

```rust
pub struct StatusArgs {
    // No additional arguments - uses global options
}
```

### Required Node.js Parameters

```typescript
interface StatusParams {
  root?: string;
  configPath?: string;
}
```

**Status**: ✅ Correctly documented

---

## 10. Config Commands Analysis

### CLI Source

```rust
pub struct ConfigShowArgs {
    // No additional args
}

pub struct ConfigValidateArgs {
    // No additional args
}
```

### Required Node.js Parameters

```typescript
interface ConfigShowParams {
  root?: string;
  configPath?: string;
}

interface ConfigValidateParams {
  root?: string;
  configPath?: string;
}
```

**Status**: ✅ Correctly documented

---

## Action Items

### High Priority - ✅ COMPLETED

1. **Update NAPI_RESEARCH.md** ✅
   - ✅ Added `always_archive` to BumpArgs (Section 4.3)
   - ✅ Added `force` to ChangesetCreateArgs (Section 4.3)
   - ✅ Added complete TypeScript interface definitions (Section 11.3)
   - ✅ Added bump function implementations and validators

2. **Update PLAN_NODE.md** ✅
   - ✅ Added `prerelease` parameter to BumpApplyParams (Phase 4)
   - ✅ Added `noChangelog`, `noArchive`, `alwaysArchive`, `force` to BumpApplyParams
   - ✅ Added `force` to ChangesetAddParams
   - ✅ Documented all TypeScript interfaces
   - ✅ Updated mapping table to use specific Bump params

3. **Update STORY_MAP_NODE.md** ✅
   - ✅ Added detailed parameter specifications to Story 5.1 (Bump Types)
   - ✅ Added `showDiff` to acceptance criteria
   - ✅ Added Story 5.5 for prerelease/snapshot validators
   - ✅ Added detailed test scenarios to Story 5.6
   - ✅ Updated Story 4.1 and 4.2 with force parameter
   - ✅ Updated story count (51 total)

### Medium Priority - PENDING (Implementation Phase)

4. **Create types/bump.rs with correct params**
   - BumpPreviewParams
   - BumpApplyParams (with prerelease, noChangelog, noArchive, alwaysArchive, force)
   - BumpSnapshotParams

5. **Update validation.rs**
   - Add prerelease tag validator
   - Add snapshot format validator

### Low Priority - ✅ COMPLETED

6. **Update all interface definitions in docs** ✅ - consistent naming applied
7. **Add examples for prerelease workflow** ✅ - added in NAPI_RESEARCH.md Section 11.3

---

## Prerelease & Snapshot Functionality

### Prerelease Mode

The CLI supports official pre-release versions via `--prerelease <TAG>`:

```bash
# Create beta prerelease
workspace bump --execute --prerelease beta

# Result: 1.2.3 → 1.3.0-beta.0
```

**Node.js Equivalent**:

```typescript
const result = await bumpApply({
  root: '.',
  prerelease: 'beta',  // Creates 1.3.0-beta.0
  gitCommit: true,
  gitTag: true,
});
```

### Snapshot Mode

The CLI supports temporary snapshot versions via `--snapshot`:

```bash
# Create snapshot version
workspace bump --snapshot --snapshot-format "{version}-snapshot.{short_commit}"

# Result: 1.2.3-snapshot.abc123f
```

**Node.js Equivalent**:

```typescript
const result = await bumpSnapshot({
  root: '.',
  format: '{version}-snapshot.{short_commit}',
});

if (result.success) {
  for (const pkg of result.data.packages) {
    console.log(`${pkg.name}: ${pkg.snapshotVersion}`);
  }
}
```

### Key Differences

| Aspect | Prerelease | Snapshot |
|--------|------------|----------|
| SemVer Compliant | ✅ Yes | ❌ No |
| Persisted | ✅ Yes (changesets archived) | ❌ No |
| Use Case | Staging/Beta releases | Testing/CI preview |
| Example | `1.3.0-beta.0` | `1.2.3-snapshot.abc123f` |
| Changelogs | Generated | Not generated |
| Git Tags | Optional | Not created |

---

## Validation Requirements

### Bump Type Validation

Valid bump types for changesets:
- `major`
- `minor`
- `patch`

**Note**: `none` is used internally but not for user input.

### Prerelease Tag Validation

Valid prerelease tags must:
- Contain only ASCII alphanumerics and hyphens `[0-9A-Za-z-]`
- Common values: `alpha`, `beta`, `rc`
- Custom tags allowed

### Snapshot Format Validation

Valid format variables:
- `{version}` - Base version
- `{branch}` - Git branch name (sanitized)
- `{commit}` - Full commit hash
- `{short_commit}` - Short commit hash (7 chars)
- `{timestamp}` - Unix timestamp

---

## Conclusion

All documentation gaps have been addressed. The Node.js bindings documentation now fully matches the CLI capabilities:

1. ✅ **`always_archive`** added to BumpArgs in all docs
2. ✅ **`force`** added to ChangesetCreateArgs in all docs
3. ✅ **Complete prerelease/snapshot** parameter documentation added
4. ✅ **TypeScript interface definitions** added to PLAN_NODE.md and STORY_MAP_NODE.md

The documentation is now ready for implementation of the bump commands with full prerelease and snapshot support.

### Files Updated

| File | Changes |
|------|---------|
| `NAPI_RESEARCH.md` | Section 4.3 BumpArgs, Section 11.3 Bump functions |
| `PLAN_NODE.md` | Phase 4 specifications, Appendix A mapping table |
| `STORY_MAP_NODE.md` | Stories 4.1, 4.2, 5.1-5.6 with detailed params |
| `PARAMETER_ANALYSIS.md` | This analysis document |