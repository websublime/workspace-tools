# Product Research: CLI Architecture & Git-Backed Changesets

**Version:** 1.0.0
**Date:** 2025
**Status:** Research & Proposal

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [CLI Command Mapping Analysis](#cli-command-mapping-analysis)
3. [Crate Evaluation](#crate-evaluation)
4. [Critical Findings](#critical-findings)
5. [Git-Backed Changesets Proposal](#git-backed-changesets-proposal)
6. [Recommendations](#recommendations)

---

## Executive Summary

This document provides a comprehensive analysis of the current CLI implementation against the PRD requirements, evaluates the base crates (`git`, `standard`, `pkg`), and proposes a new architecture for changeset storage using Git worktrees instead of file-based JSON storage.

### Key Findings

1. **Overall Architecture**: Well-designed and modular
2. **git crate**: Excellent condition ✅
3. **standard crate**: Excellent condition ✅
4. **pkg crate**: Good condition with one identified gap ⚠️
5. **CLI Implementation**: Missing automatic git integration in changeset update command
6. **PRD Compliance**: ~95% - Only changeset update auto-detect is missing

### Primary Bug Identified

The CLI's `workspace changeset update` command doesn't automatically detect affected packages from git, which is explicitly required in PRD F-011.

---

## CLI Command Mapping Analysis

### Command Implementation Status

| PRD Command | CLI Implementation | pkg Crate Module | Status |
|-------------|-------------------|------------------|--------|
| `workspace init` | `commands/init.rs` | `config` module | ✅ Complete |
| `workspace config show/validate` | `commands/config.rs` | `config` module | ✅ Complete |
| `workspace changeset create` | `commands/changeset/add.rs` | `changeset.ChangesetManager.create()` | ✅ Complete |
| `workspace changeset update` | `commands/changeset/update.rs` | `changeset.ChangesetManager.update()` | ⚠️ Partial |
| `workspace changeset list` | `commands/changeset/list.rs` | `changeset.ChangesetManager.list_pending()` | ✅ Complete |
| `workspace changeset show` | `commands/changeset/show.rs` | `changeset.ChangesetManager.load()` | ✅ Complete |
| `workspace changeset delete` | `commands/changeset/remove.rs` | `changeset.ChangesetManager.delete()` | ✅ Complete |
| `workspace changeset history` | `commands/changeset/history.rs` | `changeset.ChangesetHistory` | ⚠️ Needs verification |
| `workspace changeset check` | `commands/changeset/check.rs` | `changeset.ChangesetManager.exists()` | ✅ Complete |
| `workspace bump` | `commands/bump/` | `version.VersionResolver` | ✅ Complete |
| `workspace upgrade check` | `commands/upgrade/` | `upgrade.detect_upgrades()` | ✅ Complete |
| `workspace upgrade apply` | `commands/upgrade/` | `upgrade.apply_upgrades()` | ✅ Complete |
| `workspace upgrade backups` | `commands/upgrade/` | `upgrade.BackupManager` | ✅ Complete |
| `workspace audit` | `commands/audit/` | `audit.AuditManager` | ✅ Complete |
| `workspace changes` | `commands/changes.rs` | `changes.ChangesAnalyzer` | ✅ Complete |
| `workspace clone` | `commands/clone.rs` | git crate + init flow | ✅ Complete |
| `workspace version` | `commands/version.rs` | - | ✅ Complete |

### Global Options Support

All commands properly support global options:

| Flag | Short | Description | Output Stream |
|------|-------|-------------|---------------|
| `--root <PATH>` | `-r` | Project root directory | N/A |
| `--log-level <LEVEL>` | `-l` | Log level (silent\|error\|warn\|info\|debug\|trace) | stderr |
| `--format <FORMAT>` | `-f` | Output format (text\|json\|json-compact) | stdout |
| `--no-color` | | Disable colored output | both |
| `--config <PATH>` | `-c` | Path to config file | N/A |

---

## Crate Evaluation

### git crate - ✅ Excellent

**Location:** `crates/git/`

The git crate provides comprehensive Git operations:

- Repository management (create, open, clone)
- Branch operations (create, checkout, list, detect current)
- Commit operations (add, commit, history)
- Tag operations (create, list)
- File status and change detection
- Remote operations (push, pull, fetch)

**API Quality:** Well-defined, matches SPEC.md, comprehensive error handling.

### standard crate - ✅ Excellent

**Location:** `crates/standard/`

The standard crate provides foundational utilities:

- Configuration management with multiple sources (TOML, JSON, YAML)
- Filesystem abstraction (`AsyncFileSystem` trait)
- Command execution with queues
- Error handling infrastructure
- Path utilities

**API Quality:** Robust, flexible, enterprise-grade.

### pkg crate - ⚠️ Good with Minor Gaps

**Location:** `crates/pkg/`

| Module | Status | Notes |
|--------|--------|-------|
| `config` | ✅ Complete | Full configuration support |
| `types` | ✅ Complete | All data structures defined |
| `changeset` | ⚠️ Minor gap | `add_commits_from_git` not used by CLI |
| `version` | ✅ Complete | Both strategies, prerelease, snapshots |
| `changes` | ✅ Complete | Working directory and commit range analysis |
| `changelog` | ✅ Complete | Multiple formats, conventional commits |
| `upgrade` | ✅ Complete | Detection, application, backup |
| `audit` | ✅ Complete | All 4 sections, health score |

#### Version Resolution - Correctly Implemented

**Independent Strategy** (only bumps packages in changesets):
```rust
async fn resolve_independent(
    changeset: &Changeset,
    packages: &HashMap<String, PackageInfo>,
) -> VersionResult<VersionResolution> {
    let mut resolution = VersionResolution::new();

    for package_name in &changeset.packages {
        // Only packages in changeset get bumped
        let package_info = packages.get(package_name)?;
        let next_version = package_info.version().bump(changeset.bump)?;
        resolution.add_update(PackageUpdate::new(...));
    }

    Ok(resolution)
}
```

**Unified Strategy** (bumps ALL packages to same version):
```rust
async fn resolve_unified(
    changeset: &Changeset,
    packages: &HashMap<String, PackageInfo>,
) -> VersionResult<VersionResolution> {
    // Find highest version across ALL packages
    let unified_next_version = highest_version.bump(changeset.bump)?;

    // Apply to ALL packages (not just those in changeset)
    for (package_name, package_info) in packages {
        let reason = if changeset.packages.contains(package_name) {
            UpdateReason::DirectChange
        } else {
            UpdateReason::UnifiedStrategy
        };
        resolution.add_update(...);
    }

    Ok(resolution)
}
```

---

## Critical Findings

### Issue 1: Changeset Update Missing Auto-Detect

**Location:** `crates/cli/src/commands/changeset/update.rs`

**PRD Requirement (F-011):**
> - Analyze git diff to determine affected packages
> - Add commit hash to changeset
> - Add newly affected packages to changeset

**Current Implementation Problem:**

The CLI's `execute_update` function only adds packages/commits that the user explicitly provides:

```rust
// Current: Only adds what user explicitly provides
if let Some(packages) = &args.packages {
    for package in packages {
        changeset.add_package(package);
    }
}
```

**Missing:** The `ChangesetManager` has `add_commits_from_git()` that automatically detects affected packages:

```rust
// This method EXISTS but is NOT CALLED by CLI
pub async fn add_commits_from_git(&self, branch: &str) -> ChangesetResult<UpdateSummary> {
    // Gets commits from Git and auto-detects affected packages
    let detector = PackageDetector::new_with_config(...);
    let new_commits = detector.get_commits_since(since_commit)?;
    let affected_packages = detector.detect_affected_packages(&commit_ids).await?;
    // ...
}
```

**Recommendation:** The CLI `update` command should call `add_commits_from_git()` when no explicit `--commit` or `--packages` flags are provided.

### Issue 2: Bump Command - Complete ✅

The bump execute command properly orchestrates all steps:

1. Git repository validation
2. Configuration loading
3. Changeset loading and merging
4. Package filtering
5. Prerelease version support
6. Version resolution with strategy handling
7. **Changelog generation** ✅
8. **Changeset archival** ✅
9. **Git operations** (commit, tag, push) ✅

### Issue 3: Dependency Propagation - Implemented ✅

The `DependencyPropagator` is correctly called in `VersionResolver.resolve_versions()`:

```rust
if let Some(graph) = graph {
    let propagator = DependencyPropagator::new(&graph, &packages, &self.config.dependency);
    propagator.propagate(&mut resolution)?;
}
```

---

## Git-Backed Changesets Proposal

### Current Architecture (File-based)

```
.changesets/
├── feature-new-api.json
├── hotfix-security.json
└── history/
    └── archived-2024-01-15-abc123.json
```

**Problems:**
- JSON merge conflicts when multiple developers create changesets
- No history of changeset modifications
- Manual synchronization needed
- Pollutes main branch with metadata files

### Proposed Architecture Options

#### Option A: Git Refs Customizadas (Most Elegant)

Each changeset is a **commit** pointed to by a **custom ref**:

```
refs/changesets/
├── pending/
│   ├── feature/new-api     → commit SHA
│   └── hotfix/security     → commit SHA
└── archived/
    └── 2024-01-15-abc123   → commit SHA

Commit structure:
tree/
  └── changeset.json    # Serialized changeset content
  
Commit message: "Changeset: feature/new-api (minor)"
Parent: none (orphan) or previous changeset commit
```

**Advantages:**
- Refs are lightweight and fast
- History of changes to each changeset (each update = new commit)
- Doesn't pollute code history
- Selective push/fetch of changesets
- Atomic operations (no merge conflicts)

**Disadvantages:**
- Custom refs don't appear in GitHub/GitLab UI
- More complex to implement
- Harder to debug without tooling

#### Option B: Worktree + Orphan Branch (Most Practical)

An orphan branch `_changesets` with a worktree for access:

```
/project/
├── .git/                      ← shared repository
├── .changesets/               ← WORKTREE (branch _changesets)
│   ├── .git                   ← file pointing to ../.git
│   ├── pending/
│   │   └── feature-api.json
│   └── archived/
├── src/                       ← normal code (branch main)
└── package.json
```

**Advantages:**
- Real files you can see/edit
- Branch visible in GitHub
- Single history for all changeset changes
- Existing tools work (VS Code, etc.)
- Merge conflicts handled by Git (not in code)

**Disadvantages:**
- Requires worktree setup (automatable)
- More disk space

#### Option C: Git Notes

Attach metadata to existing commits:

```bash
git notes --ref=changesets add -m '{"bump":"minor","packages":[...]}' HEAD
```

**Disadvantages:**
- Notes not pushed by default
- Less flexible for changeset lifecycle
- Harder to manage

### Recommended: Option B - Worktree + Orphan Branch

#### Understanding Git Concepts

**Branch:** Simply a pointer to a commit (a reference).

**Ref (Reference):** Files in `.git/refs/` storing commit SHAs. Branches are one type of ref.

**Worktree:** An additional working directory linked to the same repository. Allows having multiple branches checked out simultaneously.

```
/project/                    ← main worktree (main checked out)
├── .git/
├── src/
└── package.json

/project/.changesets/        ← additional worktree (_changesets checked out)
├── pending/
│   └── feature-api.json
└── archived/
```

#### Setup Process

```bash
# 1. Create orphan branch for changesets
git checkout --orphan _changesets
git reset --hard
mkdir -p pending archived
echo '{}' > pending/.gitkeep
echo '{}' > archived/.gitkeep
git add .
git commit -m "Initialize changesets storage"
git push origin _changesets

# 2. Return to working branch
git checkout main

# 3. Create worktree
git worktree add .changesets _changesets

# 4. Add to .gitignore of main
echo ".changesets" >> .gitignore
```

**Result:**

```
/project/
├── .git/
├── .gitignore          ← contains ".changesets"
├── .changesets/        ← WORKTREE (real directory!)
│   ├── .git            ← text file: "gitdir: ../.git/worktrees/.changesets"
│   ├── pending/
│   │   └── feature-api.json
│   └── archived/
├── src/
└── package.json
```

#### Operation Flows

**Create Changeset:**

```
1. User: workspace changeset create --bump minor
2. CLI: Detects current branch (feature/new-api)
3. pkg: ChangesetManager.create("feature/new-api", Minor, ["production"])
4. storage: Write to .changesets/pending/feature-api.json
5. storage: cd .changesets && git add && git commit && git push
6. Done: Changeset exists as file in _changesets branch
```

**Update Changeset:**

```
1. User: workspace changeset update (or git hook post-commit)
2. CLI: Detects current branch
3. pkg: ChangesetManager.add_commits_from_git("feature/new-api")
   → PackageDetector analyzes commits since last update
   → Detects affected packages
   → Adds commits and packages to changeset
4. storage: Update JSON, commit in _changesets branch
5. Done: Changeset updated with new packages/commits
```

**Bump and Archive:**

```
1. User: workspace bump --execute
2. CLI: Load all pending changesets from .changesets/pending/
3. pkg: VersionResolver resolves versions
4. pkg: Apply versions to package.json files
5. pkg: Move files from pending/ to archived/
6. storage: Commit in _changesets branch
7. CLI: Git commit, tag, push (in main branch)
8. Done: Changesets archived, versions bumped
```

#### Synchronization Between Developers

```bash
# Developer A creates changeset
workspace changeset create  # → commit in _changesets, push

# Developer B fetches
git fetch origin _changesets:_changesets

# If worktree doesn't exist, create it:
git worktree add .changesets _changesets

# Now sees Developer A's changesets
ls .changesets/pending/
```

**Automatic in ChangesetManager:**

```rust
async fn ensure_worktree(workspace_root: &Path) -> Result<PathBuf> {
    let worktree_path = workspace_root.join(".changesets");
    
    if !worktree_path.exists() {
        let repo = Repo::open(workspace_root)?;
        
        if !repo.branch_exists("_changesets")? {
            create_orphan_changesets_branch(&repo)?;
        }
        
        repo.create_worktree(".changesets", "_changesets")?;
    }
    
    // Pull to get updated changesets
    let worktree_repo = Repo::open(&worktree_path)?;
    worktree_repo.pull("origin", "_changesets")?;
    
    Ok(worktree_path)
}
```

### Implementation Changes Required

#### git crate

New module: `worktree.rs`

```rust
impl Repo {
    /// Creates a new worktree
    pub fn create_worktree(&self, path: &str, branch: &str) -> Result<(), RepoError>;
    
    /// Lists existing worktrees
    pub fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>, RepoError>;
    
    /// Removes a worktree
    pub fn remove_worktree(&self, path: &str) -> Result<(), RepoError>;
    
    /// Creates an orphan branch
    pub fn create_orphan_branch(&self, name: &str) -> Result<(), RepoError>;
}
```

#### pkg crate

New storage implementation:

```rust
/// Git worktree-backed changeset storage
pub struct GitWorktreeChangesetStorage {
    worktree_path: PathBuf,
    worktree_repo: Repo,
    pending_dir: PathBuf,
    archived_dir: PathBuf,
}

#[async_trait]
impl ChangesetStorage for GitWorktreeChangesetStorage {
    async fn save(&self, changeset: &Changeset) -> ChangesetResult<()> {
        // 1. Write JSON to pending/
        let file_path = self.pending_dir.join(format!("{}.json", changeset.branch));
        fs::write(&file_path, serde_json::to_string_pretty(changeset)?)?;
        
        // 2. Commit in worktree
        self.worktree_repo.add("pending/")?;
        self.worktree_repo.commit(&format!("Update changeset: {}", changeset.branch))?;
        
        // 3. Push
        self.worktree_repo.push("origin", None)?;
        
        Ok(())
    }
    
    async fn load(&self, branch: &str) -> ChangesetResult<Changeset> {
        let file_path = self.pending_dir.join(format!("{}.json", branch));
        let content = fs::read_to_string(&file_path)?;
        Ok(serde_json::from_str(&content)?)
    }
    
    async fn list_pending(&self) -> ChangesetResult<Vec<String>> {
        let entries = fs::read_dir(&self.pending_dir)?;
        Ok(entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension() == Some("json".as_ref()))
            .filter_map(|e| e.path().file_stem()?.to_str().map(String::from))
            .collect())
    }
    
    async fn archive(&self, changeset: &Changeset, release_info: ReleaseInfo) -> ChangesetResult<()> {
        let archived = ArchivedChangeset::new(changeset.clone(), release_info);
        
        // Move from pending/ to archived/
        let from = self.pending_dir.join(format!("{}.json", changeset.branch));
        let archive_id = format!("{}-{}", Utc::now().format("%Y-%m-%d"), &changeset.id[..8]);
        let to = self.archived_dir.join(format!("{}.json", archive_id));
        
        fs::write(&to, serde_json::to_string_pretty(&archived)?)?;
        fs::remove_file(&from)?;
        
        // Commit
        self.worktree_repo.add(".")?;
        self.worktree_repo.commit(&format!("Archive changeset: {}", changeset.branch))?;
        self.worktree_repo.push("origin", None)?;
        
        Ok(())
    }
}
```

Configuration update:

```rust
pub enum StorageBackend {
    File,        // Current - JSON files in .changesets/
    GitWorktree, // New - Git worktree with orphan branch
}

pub struct ChangesetConfig {
    pub path: PathBuf,
    pub history_path: PathBuf,
    pub available_environments: Vec<String>,
    pub default_environments: Vec<String>,
    pub storage_backend: StorageBackend, // NEW
}
```

#### cli crate

Minimal changes - uses same `ChangesetManager` API:

```rust
// Factory based on config
fn create_changeset_storage(config: &PackageToolsConfig, repo: &Repo) -> Box<dyn ChangesetStorage> {
    match config.changeset.storage_backend {
        StorageBackend::File => Box::new(FileBasedChangesetStorage::new(...)),
        StorageBackend::GitWorktree => Box::new(GitWorktreeChangesetStorage::new(...)),
    }
}
```

### Comparison: Worktree vs Refs vs Files

| Aspect | File-based | Worktree + Orphan | Refs Customizadas |
|--------|-----------|-------------------|-------------------|
| **Visibility** | Files in `.changesets/` | Files in worktree | Invisible, only via `git show` |
| **GitHub/GitLab** | Files in repo | Branch visible in UI | Refs don't appear in UI |
| **Complexity** | Low | Medium | High |
| **Conflicts** | JSON merge conflicts | Git handles merges | None (atomic refs) |
| **Debug** | Easy (`cat file.json`) | Easy (`cat file.json`) | Hard (`git show refs/...`) |
| **History** | Overwrites file | One branch, many commits | Each changeset has own history |
| **Sync** | Pull/push main branch | Pull/push `_changesets` | Push/fetch custom refs |
| **Offline** | ✅ Works | ✅ Works | ✅ Works |
| **CI/CD** | ✅ Read files | ✅ Read files | ⚠️ Configure ref fetch |

---

## Recommendations

### Immediate Actions

1. **Fix changeset update command** to call `add_commits_from_git()`:
   ```rust
   // In CLI, changeset update should call:
   if args.commit.is_none() && args.packages.is_none() {
       manager.add_commits_from_git(&branch).await?;
   }
   ```

2. **Add integration tests** for complete CLI flows

3. **Verify changeset history queries** work correctly

### For Bun + Ink CLI Redesign

If redesigning CLI with Bun + Ink:

1. **Keep Rust crates as-is** - They provide core business logic

2. **Create NAPI bindings** to expose:
   - `ChangesetManager`
   - `VersionResolver`
   - `UpgradeManager`
   - `AuditManager`
   - `ChangesAnalyzer`
   - `ChangelogGenerator`

3. **The pkg crate is the primary interface** - All features flow through it

### For Git-Backed Storage

**Phased Implementation:**

1. **Phase 1:** git crate - Add worktree support
2. **Phase 2:** pkg crate - Implement `GitWorktreeChangesetStorage`
3. **Phase 3:** Configuration - Add `storage_backend` option
4. **Phase 4:** CLI - Support for new backend
5. **Phase 5:** Migration tooling (file → git)
6. **Phase 6:** Documentation and setup automation

### Final Recommendation

**Implement Worktree + Orphan Branch approach** because:

1. ✅ Maintains `ChangesetStorage` interface - transparent change
2. ✅ Complete history of each changeset modification
3. ✅ Native synchronization with git push/fetch
4. ✅ No JSON merge conflicts in main branch
5. ✅ Free audit trail via git log
6. ✅ Trivial rollback with git reset
7. ✅ Works offline
8. ✅ Visible in GitHub/GitLab UI
9. ✅ Easy to debug (real files)
10. ✅ Compatible with CI/CD

---

## Appendix: Additional Resources

- [PRD Documentation](../crates/cli/docs/PRD.md)
- [pkg crate SPEC](../crates/pkg/SPEC.md)
- [git crate SPEC](../crates/git/SPEC.md)
- [standard crate SPEC](../crates/standard/SPEC.md)
- [Git Worktrees Documentation](https://git-scm.com/docs/git-worktree)