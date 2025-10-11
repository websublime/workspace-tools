# Design Decisions for sublime_pkg_tools

**Last Updated**: 2024-01-15  
**Status**: Finalized

---

## Overview

This document captures all key design decisions made during the planning phase of `sublime_pkg_tools`. These decisions prioritize simplicity, safety, and maintainability while providing powerful version management capabilities.

---

## 1. Snapshot Versions: Always on Branches

### Decision

**Snapshot versions are ALWAYS used on non-main branches, regardless of commit type.**

### Rationale

- Allows multiple pushes to the same branch with unique versions
- Enables continuous deployment to development environments
- Each commit gets a unique identifier: `1.2.3-abc123d.snapshot`
- Prevents version pollution in package.json during development
- Actual version bumps only happen on merge to main

### Behavior

```
Branch: feat/user-auth
├─ Commit 1 (abc123d): 1.2.3-abc123d.snapshot → Deploy to dev
├─ Commit 2 (def456g): 1.2.3-def456g.snapshot → Deploy to dev
└─ Commit 3 (ghi789j): 1.2.3-ghi789j.snapshot → Deploy to dev

Merge to main:
└─ Apply changeset: 1.2.3 → 1.3.0 → Deploy to prod
```

### Impact

- `package.json` never modified on branches
- Snapshot versions calculated at runtime
- Version bumps are controlled and reviewable
- Safe continuous deployment to dev environments

---

## 2. Version Bumps: Only on Merge to Main

### Decision

**Version bumps from conventional commits are ONLY applied when merging to main.**

### Rationale

- Separates development (ephemeral) from releases (permanent)
- Changesets review and aggregate all changes before bumping
- Prevents accidental version increases during development
- Allows feature branches to accumulate multiple commits
- Single, reviewable version change per feature

### Behavior

```
On branch feat/user-auth:
├─ feat: add OAuth2           → snapshot (not minor yet)
├─ fix: resolve memory leak   → snapshot (not patch yet)
└─ docs: update README        → snapshot

On merge to main:
└─ Changeset analyzed → minor bump (feat wins) → 1.2.3 → 1.3.0
```

### Impact

- Conventional commits guide bump type but don't apply it immediately
- Changeset aggregates intent across multiple commits
- Version history is clean and intentional

---

## 3. Dependency Updates: Patch Bump Default

### Decision

**When a package is updated, all dependent packages receive a patch bump by default.**

### Rationale

- Semantic versioning: dependencies changed = implementation changed
- Safe default (patch is smallest bump)
- Can be overridden if dependent package has direct changes
- Maintains version consistency across monorepo

### Behavior

```
Package A: feat: add OAuth2 → minor bump (1.2.3 → 1.3.0)
Package B: depends on A     → patch bump (2.0.0 → 2.0.1)
Package C: depends on B     → patch bump (1.5.0 → 1.5.1)

If Package B also has direct changes:
Package B: fix: resolve bug → patch bump (direct changes win)
```

### Impact

- Dependency propagation is predictable
- All affected packages get version updates
- Configurable via `dependency_update_bump = "patch"`

---

## 4. Simplified Changeset Structure

### Decision

**Use simple array for releases instead of complex HashMap with tracking.**

### Rationale

- Easier to understand and modify
- Reduces initial complexity
- All packages released to all specified environments
- Can be extended later if needed

### Structure

```json
{
  "branch": "feat/user-auth",
  "created_at": "2024-01-15T10:30:00Z",
  "author": "developer@example.com",
  "releases": ["dev", "qa"],
  "packages": [...]
}
```

### Impact

- Simple array: `"releases": ["dev", "qa"]`
- Not: `"releases": { "dev": { "packages": [...], "released": false } }`
- Environments configured in `repo.config.toml`
- All packages deployed to all specified environments

---

## 5. Filename as Identity

### Decision

**Changeset filenames follow format: `{branch}-{datetime}.json`**

### Rationale

- Self-documenting: branch and timestamp visible in filename
- Naturally unique (datetime ensures uniqueness)
- Sortable by creation time
- Git-friendly (no special characters)
- No separate `id` field needed

### Examples

```
feat/user-auth       → feat-user-auth-20240115T103000Z.json
bugfix/memory-leak   → bugfix-memory-leak-20240115T144530Z.json
feature/oauth2       → feature-oauth2-20240115T091500Z.json
```

### Impact

- Filename = identity
- Latest changeset for branch = latest file with matching branch prefix
- Easy to find, sort, and manage

---

## 6. Two Versioning Strategies

### Decision

**Support Independent and Unified versioning strategies.**

### Independent Strategy (Default)

Each package maintains its own version independently.

```
Before: pkg-a@1.2.0, pkg-b@2.5.1, pkg-c@0.8.0
After:  pkg-a@1.3.0, pkg-b@2.5.2, pkg-c@0.8.0
```

**Use Cases**:
- Packages have different release cycles
- Different maturity levels (v0.x vs v2.x)
- Independent evolution

### Unified Strategy

All packages share the same version.

```
Before: pkg-a@1.2.0, pkg-b@1.2.0, pkg-c@1.2.0
After:  pkg-a@1.3.0, pkg-b@1.3.0, pkg-c@1.3.0
```

**Use Cases**:
- Tightly coupled packages
- Simpler version management
- All packages released together

### Configuration

```toml
[package_tools.release]
strategy = "independent"  # or "unified"
```

### Impact

- Configurable behavior
- Independent is default
- Unified simplifies version tracking for coupled packages

---

## 7. Full Dry Run Support

### Decision

**All version bump and release operations support dry-run mode.**

### Rationale

- Safety: preview before applying changes
- Validation: catch errors early
- Transparency: see exactly what will happen
- CI/CD friendly: validate in pipelines

### Capabilities

- Show packages to be updated
- Display current and next versions
- List files to be modified
- Show Git tags to be created
- Display commands to be executed
- **Zero writes**: no filesystem, Git, or registry changes

### Example Output

```
🔍 DRY RUN MODE - No changes will be made

Packages to be updated:
  @myorg/auth-service: 1.2.3 → 1.3.0 (feat: add OAuth2)
  @myorg/user-service: 2.5.1 → 2.5.2 (dependency update)

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

### Impact

- `release_mgr.dry_run(plan)` API
- CLI flag: `--dry-run`
- Safe previews before any operation

---

## 8. Configuration via Standard System

### Decision

**Use `repo.config.{toml,yml,yaml,json}` from sublime_standard_tools.**

### Rationale

- Consistency across all sublime tools
- Single configuration file
- Reuse existing infrastructure
- Environment variable overrides
- No separate config files

### Configuration Locations

1. `repo.config.toml` (project root) - highest priority
2. `repo.config.yml/yaml` (project root)
3. `repo.config.json` (project root)
4. `~/.config/sublime/config.toml` (user config)
5. Environment variables: `SUBLIME_PACKAGE_TOOLS_*`

### Structure

```toml
[package_tools]

[package_tools.changeset]
available_environments = ["dev", "test", "qa", "staging", "prod"]
default_environments = ["dev"]

[package_tools.release]
strategy = "independent"
dry_run_by_default = false

[package_tools.dependency]
dependency_update_bump = "patch"
```

### Impact

- Nested under `[package_tools]` key
- Leverages standard config loading
- Validation via `Configurable` trait
- Merge strategies handled automatically

---

## 9. Maximum Reuse of Existing Crates

### Decision

**Leverage sublime_standard_tools and sublime_git_tools for all operations.**

### Rationale

- Avoid reimplementing functionality
- Battle-tested implementations
- Consistency with other tools
- Reduced maintenance burden

### Usage

- **FileSystemManager**: All file operations
- **ConfigManager**: Configuration loading
- **Repo**: All Git operations
- **MonorepoDetector**: Project detection
- **CommandExecutor**: Shell commands (npm publish, etc.)

### Impact

- Minimal code duplication
- Consistent error handling
- Shared configuration system
- Cross-platform support handled by standard tools

---

## 10. Snapshot Versions Never Persisted

### Decision

**Snapshot versions are calculated dynamically at runtime and NEVER written to package.json.**

### Rationale

- Prevents Git pollution
- Avoids merge conflicts
- Snapshot versions are ephemeral
- Only release versions are committed
- Each commit has unique identifier without changes

### Implementation

```rust
// VersionResolver calculates on-the-fly
let version = resolver.resolve_current_version(package_path).await?;

// On branch: 1.2.3-abc123d.snapshot (calculated)
// On main:   1.2.3 (from package.json)
```

### Impact

- `package.json` only contains release versions
- Snapshot format: `{version}-{commit}.snapshot`
- Zero git history pollution from version changes

---

## 11. Changeset History Tracking

### Decision

**When a changeset is applied (release), it MUST be moved from `.changesets/` to `.changesets/history/` with release metadata.**

### Rationale

- Maintains complete audit trail of all releases
- Separates pending changesets from applied changesets
- Enables historical analysis and troubleshooting
- Prevents accidental reapplication of changesets
- Provides clear visibility into what's pending vs. completed

### Structure

```
.changesets/
├── feat-user-auth-20240115T103000Z.json    (pending)
├── fix-memory-leak-20240115T144530Z.json   (pending)
└── history/
    ├── feat-oauth2-20240114T091500Z.json   (applied)
    └── bugfix-security-20240113T120000Z.json (applied)
```

### Metadata Added on Archive

When moved to history, add release metadata:

```json
{
  "branch": "feat/user-auth",
  "created_at": "2024-01-15T10:30:00Z",
  "author": "developer@example.com",
  "releases": ["dev", "qa"],
  "packages": [...],
  "release_info": {
    "applied_at": "2024-01-15T14:45:00Z",
    "applied_by": "developer@example.com",
    "git_commit": "def456a",
    "environments_released": {
      "dev": {
        "released_at": "2024-01-15T14:45:00Z",
        "tag": "v1.3.0-dev"
      },
      "qa": {
        "released_at": "2024-01-15T14:50:00Z",
        "tag": "v1.3.0-qa"
      }
    }
  }
}
```

### Behavior

1. **Before Release**: Changeset exists in `.changesets/`
2. **During Release**: Changeset is read and applied
3. **After Release**: 
   - Add `release_info` section
   - Move to `.changesets/history/`
   - Original filename preserved
4. **History Query**: All historical releases available for audit

### Impact

- Clear separation of pending vs. completed
- Complete traceability of releases
- Enables rollback analysis
- Supports compliance requirements
- History can be queried for reports

---

## 12. Environment Configuration

### Decision

**Available environments configured in repo.config.toml, changesets reference them by name.**

### Rationale

- Single source of truth
- Prevents typos
- Easy to add/remove environments globally
- Validation at config level

### Configuration

```toml
[package_tools.changeset]
available_environments = ["dev", "test", "qa", "staging", "prod"]
default_environments = ["dev"]
```

### Impact

- Changeset validates environment names
- Global environment list
- Easy to extend

---

## 13. Changeset History Management API

### Decision

**Provide APIs to query, list, and analyze changeset history.**

### Capabilities

```rust
// List all pending changesets
let pending = changeset_mgr.list_pending().await?;

// List all applied changesets (history)
let history = changeset_mgr.list_history().await?;

// Get specific changeset from history
let changeset = changeset_mgr.get_from_history("feat-auth-20240115T103000Z").await?;

// Query history by date range
let recent = changeset_mgr.query_history_by_date(start, end).await?;

// Query history by package
let pkg_history = changeset_mgr.query_history_by_package("@myorg/auth").await?;
```

### Impact

- History is first-class citizen
- Easy to audit and analyze releases
- Supports reporting and compliance
- Foundation for advanced features (rollback, analytics)

---

## Summary of Key Principles

| Principle | Implementation |
|-----------|---------------|
| **Safety First** | Snapshot versions on branches, dry-run support |
| **Simplicity** | Simple changeset structure, filename-based identity |
| **Flexibility** | Two versioning strategies, configurable behavior |
| **Consistency** | Reuse standard tools, shared configuration |
| **Transparency** | Dry-run shows everything, clear reasoning |
| **Semantic Versioning** | Conventional commits, dependency propagation |
| **Traceability** | Complete history tracking, audit trail maintained |

---

## Complete Workflow Summary

```
1. Feature Branch Development
   ├─ Create branch: feat/user-auth
   ├─ Commit changes (snapshot versions always)
   ├─ Push and deploy to dev (unique snapshot per commit)
   └─ Continue iterating

2. Prepare Release
   ├─ Create changeset (analyze commits)
   ├─ Specify target environments: ["dev", "qa"]
   ├─ Calculate dependency propagation
   └─ Save: feat-user-auth-20240115T103000Z.json

3. Preview Changes
   ├─ Run dry-run
   ├─ Review packages, versions, files, tags
   └─ Confirm or abort

4. Merge to Main
   ├─ Apply changeset version bumps
   ├─ Update package.json files
   ├─ Commit version changes
   └─ Create Git tags

5. Release to Environments
   ├─ Dev: 1.3.0-dev
   ├─ QA: 1.3.0-qa
   └─ Prod: 1.3.0 (latest)

6. Archive Changeset
   ├─ Add release_info metadata
   ├─ Move to .changesets/history/
   └─ Maintain complete audit trail
```

---

## Open for Future Consideration

These items are explicitly **not** part of the initial implementation but can be added later:

- Rollback support (history tracking enables this)
- Changeset merging strategies
- Approval workflows
- Interactive CLI
- Web UI
- Advanced dependency visualization

---

**These decisions provide a solid foundation for a simple, safe, and maintainable package version management system.**