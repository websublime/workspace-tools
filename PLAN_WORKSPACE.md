# Plan: Rename Binary from `wnt` to `workspace`

## Overview

Rename the CLI binary from `wnt` to `workspace` for better clarity and intuitiveness.

**Scope**: Direct rename without migration path or backward compatibility.

**Package Strategy**:
- Keep namespace: `sublime_*` packages
- Binary name: `wnt` → `workspace`

---

## Availability Check ✅

All desired names are available on crates.io:
- ✅ `workspace`
- ✅ `workspace-tools`
- ✅ `workspace-cli`

---

## Changes Required

### Summary Statistics

- **Total files**: ~80+
- **Total occurrences**: ~967+
  - CLI crate: ~60 files
  - Scripts: 356 occurrences in 15 files
  - GitHub Actions: 7 occurrences in 2 files
  - Documentation: 604 occurrences in 11 files

---

## Phase 1: CLI Crate Changes

### 1.1. Cargo.toml

**File**: `crates/cli/Cargo.toml`

```toml
# Keep package name with sublime namespace
[package]
name = "sublime_cli_tools"

# Change binary name
[[bin]]
name = "workspace"  # ← Changed from "wnt"
path = "src/main.rs"
```

### 1.2. Branding

**File**: `crates/cli/src/cli/branding.rs`

Changes:
- Redesign ASCII logo for "WORKSPACE"
- Update `SHORT_NAME = "workspace"` (from "WNT")
- Update `FULL_NAME = "Workspace Tools"` (simplify from "Workspace Node Tools")

```rust
pub const ASCII_LOGO: &str = r"
░█░█░█▀█░█▀▄░█░█░█▀▀░█▀█░█▀█░█▀▀░█▀▀
░█▄█░█░█░█▀▄░█▀▄░▀▀█░█▀▀░█▀█░█░█░█▀▀
░▀░▀░▀▀▀░▀░▀░▀░▀░▀▀▀░▀░░░▀░▀░▀▀▀░▀▀▀
";

pub const SHORT_NAME: &str = "workspace";  // ← Changed
pub const FULL_NAME: &str = "Workspace Tools";  // ← Changed
```

### 1.3. Build Script

**File**: `crates/cli/build.rs`

Update comments:
- "Shell completions for the `wnt` CLI tool" → "Shell completions for the `workspace` CLI tool"

### 1.4. Completions

**File**: `crates/cli/src/cli/completions.rs`

Global replacements in documentation and examples:
- `wnt` → `workspace`
- `/wnt` → `/workspace`
- `_wnt` → `_workspace`
- `wnt.fish` → `workspace.fish`
- `wnt.elv` → `workspace.elv`

**Locations** (~20 occurrences):
- Line 37, 44, 46, 49, 51, 54, 57: Installation examples
- Line 74, 86, 99: Function documentation
- Line 197, 201, 213, 217, 230, 233: Installation instructions
- Line 254, 257, 258: Elvish completions

### 1.5. All Command Files

**Files**: `crates/cli/src/commands/**/*.rs` (~50 files)

Global find & replace in all documentation examples:
- `` `wnt <command>` `` → `` `workspace <command>` ``

**Regex pattern**:
```regex
Find: `wnt ([a-z\-\s]+)`
Replace: `workspace $1`
```

**Affected files**:
- `commands/changeset/add.rs`
- `commands/changeset/update.rs`
- `commands/changeset/remove.rs`
- `commands/changeset/show.rs`
- `commands/changeset/list.rs`
- `commands/changeset/edit.rs`
- `commands/changeset/common.rs`
- `commands/bump/preview.rs`
- `commands/bump/execute.rs`
- `commands/upgrade/apply.rs`
- `commands/upgrade/check.rs`
- `commands/upgrade/rollback.rs`
- `commands/audit/breaking.rs`
- `commands/audit/upgrades.rs`
- `commands/audit/report.rs`
- `commands/changes.rs`
- `commands/init.rs`
- `commands/config.rs`
- `commands/version.rs`
- All test files in `commands/tests.rs` and `commands/*/tests.rs`

### 1.6. Main Files

**Files**:
- `crates/cli/src/main.rs`
- `crates/cli/src/lib.rs`
- `crates/cli/src/cli/mod.rs`
- `crates/cli/src/cli/commands.rs`
- `crates/cli/src/cli/tests.rs`
- `crates/cli/src/error/display.rs`

Replace `wnt` with `workspace` in comments and documentation.

### 1.7. CLI Documentation

**Files**:
- `crates/cli/README.md` (17 occurrences)
- `crates/cli/PLAN.md` (47 occurrences)
- `crates/cli/PRD.md` (237 occurrences)
- `crates/cli/CLI.md` (7 occurrences)
- `crates/cli/STORY_MAP.md` (35 occurrences)

Global find & replace: `wnt` → `workspace`

---

## Phase 2: Scripts Changes

### 2.1. Installation Script

**File**: `scripts/install.sh`

**Critical changes**:

```bash
# Line 39
readonly BINARY_NAME="workspace"  # ← Changed from "wnt"

# Line 4 (header comment)
# Official installation script for Workspace Tools
# (changed from "wnt (Workspace Node Tools)")

# Update all user-facing messages
# "wnt" → "workspace" throughout the file
```

**Total occurrences**: 4+ direct references plus many in messages and examples

### 2.2. Uninstallation Script

**File**: `scripts/uninstall.sh`

```bash
# Header comment
# Official uninstallation script for Workspace Tools

# Binary name constant
readonly BINARY_NAME="workspace"
```

**Total occurrences**: 18

### 2.3. Development Scripts

**File**: `scripts/install-dev.sh`
- Update BINARY_NAME
- Update messages
- **Occurrences**: 12

**File**: `scripts/test-in-demo.sh`
- Update binary references in test commands
- **Occurrences**: 9

### 2.4. Hook Scripts

**File**: `scripts/install-hooks.sh`
- Update references to binary in installation messages
- **Occurrences**: 11

**File**: `scripts/uninstall-hooks.sh`
- Update references in messages
- **Occurrences**: 1

### 2.5. Git Hooks

All hooks in `scripts/git-hooks/`:

**File**: `scripts/git-hooks/pre-commit`
```bash
# Change binary checks
if command -v workspace >/dev/null 2>&1; then  # ← Changed from wnt
    workspace changeset validate
```
**Occurrences**: 9

**File**: `scripts/git-hooks/pre-push`
- Same pattern as pre-commit
- **Occurrences**: 9

**File**: `scripts/git-hooks/post-commit`
- Same pattern
- **Occurrences**: 8

**File**: `scripts/git-hooks/post-checkout`
- Same pattern
- **Occurrences**: 11

**File**: `scripts/git-hooks/prepare-commit-msg`
- Same pattern
- **Occurrences**: 7

### 2.6. Scripts Documentation

**File**: `scripts/INSTALLATION.md`
- Replace all command examples
- **Occurrences**: 126

**File**: `scripts/TESTING.md`
- Replace all command examples
- **Occurrences**: 59

**File**: `scripts/README.md`
- Replace all references
- **Occurrences**: 42

**File**: `scripts/git-hooks/README.md`
- Replace all references
- **Occurrences**: 30

---

## Phase 3: GitHub Actions Changes

### 3.1. Build Binaries Workflow

**File**: `.github/workflows/build-binaries.yml`

**Changes** (6 occurrences):

```yaml
# Line ~121: Strip binary
- name: Strip binary (Linux/macOS)
  run: |
    strip target/${{ matrix.target }}/release/workspace || true  # ← Changed
    ls -lh target/${{ matrix.target }}/release/workspace  # ← Changed

# Line ~128: Windows binary
- name: Get binary info (Windows)
  run: |
    ls -lh target/${{ matrix.target }}/release/workspace.exe  # ← Changed

# Line ~135: Archive name
- name: Create archive
  run: |
    ARCHIVE_NAME="workspace-${TAG}-${{ matrix.target }}"  # ← Changed
    
    if [ "${{ matrix.os }}" = "windows-latest" ]; then
      7z a "${ARCHIVE_NAME}.zip" workspace.exe  # ← Changed
    else
      tar czf "${ARCHIVE_NAME}.tar.gz" workspace  # ← Changed
```

### 3.2. Release Please Workflow

**File**: `.github/workflows/release-please.yml`

Check and update any references to binary name.
**Occurrences**: 1

---

## Phase 4: Documentation Changes

### 4.1. Root Documentation

**File**: `RELEASE.md`
- Update release instructions
- **Occurrences**: 2

### 4.2. Package Documentation

**File**: `crates/pkg/docs/guides/configuration.md`
- Update CLI command examples
- **Occurrences**: 2

**Files**: `crates/pkg/examples/*.toml`
- Update comments if they reference the CLI
- Check: `basic-config.toml`, `monorepo-config.toml`

**Files**: Configuration and upgrade modules
- `crates/pkg/src/config/upgrade.rs`
- `crates/pkg/src/config/tests.rs`
- `crates/pkg/src/upgrade/backup/mod.rs`
- `crates/pkg/src/upgrade/backup/tests.rs`

Replace `wnt` with `workspace` in comments and documentation.

---

## Validation Checklist

### Build & Compilation
- [ ] `cargo build --release`
- [ ] Binary exists at `target/release/workspace`
- [ ] Binary runs: `./target/release/workspace --version`
- [ ] Binary runs: `./target/release/workspace --help`

### Tests
- [ ] `cargo test --all`
- [ ] `cargo test --doc`
- [ ] `cargo clippy --all -- -D warnings`
- [ ] `cargo fmt --all -- --check`

### Scripts
- [ ] `shellcheck scripts/*.sh`
- [ ] `shellcheck scripts/git-hooks/*`
- [ ] Test `scripts/install-dev.sh` execution
- [ ] Verify completions generation works

### Commands
- [ ] `workspace --version` outputs correct version
- [ ] `workspace --help` displays help
- [ ] `workspace init --help` works
- [ ] `workspace changeset add --help` works
- [ ] `workspace completions bash > /tmp/test.bash` works

### Documentation
- [ ] No remaining references to `wnt` in user-facing docs
- [ ] All examples use `workspace` command
- [ ] README files are consistent

### GitHub Actions
- [ ] Workflow syntax is valid
- [ ] Archive names use `workspace-` prefix
- [ ] Binary paths reference `workspace` not `wnt`

---

## Execution Steps

### Step 1: Prepare

```bash
# Create working branch
git checkout -b refactor/WOR-TSK-125-RENAME-TO-WORKSPACE

# Verify current state
cargo build --release
cargo test --all
```

### Step 2: Execute Changes

Execute all phases in order:
1. Phase 1: CLI Crate (use find & replace carefully)
2. Phase 2: Scripts (update each file)
3. Phase 3: GitHub Actions (update workflows)
4. Phase 4: Documentation (global replace)

### Step 3: Validate

Run all validation checks from the checklist above.

### Step 4: Final Verification

```bash
# Build fresh
cargo clean
cargo build --release

# Verify binary
ls -la target/release/workspace
./target/release/workspace --version
./target/release/workspace --help

# Run all tests
cargo test --all --verbose

# Check for any remaining "wnt" references
git grep -i "wnt" | grep -v "PLAN_WORKSPACE.md" | grep -v ".git/"
```

### Step 5: Commit

```bash
git add .
git commit -m "refactor(WOR-TSK-125): rename binary from wnt to workspace

Changes:
- Binary name: wnt → workspace
- Update Cargo.toml bin configuration
- Redesign ASCII logo and branding
- Update all CLI command examples and documentation
- Update installation and development scripts
- Update all git hooks to use new binary name
- Update GitHub Actions workflows
- Update all user-facing documentation

The package name remains 'sublime_cli_tools' to maintain namespace consistency.

Closes WOR-TSK-125"
```

---

## Find & Replace Patterns

### Pattern 1: Command Examples in Rust Docs
```regex
Find: `wnt ([a-z][a-z0-9\-\s]*)`
Replace: `workspace $1`
```

### Pattern 2: Shell Script Binary Variable
```regex
Find: BINARY_NAME="wnt"
Replace: BINARY_NAME="workspace"
```

### Pattern 3: Completion File Names
```regex
Find: (/)wnt([\.])
Replace: $1workspace$2
```

### Pattern 4: Simple Text Replace
```
Find: wnt
Replace: workspace
```

Note: Use case-sensitive search. Review each replacement in context.

---

## Files Requiring Manual Review

Some files may need manual review for context:

1. **Git history references** (`.git/logs/*`) - No changes needed
2. **Changelog entries** - May want to keep historical `wnt` references
3. **ASCII logo design** - Requires manual redesign for aesthetics
4. **Package.json** (if exists) - May have references in scripts

---

## Estimated Effort

| Phase | Files | Time |
|-------|-------|------|
| 1. CLI Crate | ~60 | 4-5h |
| 2. Scripts | 15 | 3-4h |
| 3. GitHub Actions | 2 | 1h |
| 4. Documentation | 11 | 2h |
| Validation | - | 2h |
| **Total** | **~88** | **12-14h** |

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Missed reference | Final `git grep` before commit |
| Broken scripts | Test each script after changes |
| Failed CI | Validate workflows syntax locally |
| Wrong replacements | Review all changes in git diff |

---

## Post-Execution Verification

After all changes:

```bash
# 1. Search for any remaining "wnt" references
git grep -i "\bwnt\b" | grep -v "PLAN_WORKSPACE.md"

# 2. Check binary exists with new name
ls -la target/release/workspace

# 3. Test basic functionality
./target/release/workspace --version
./target/release/workspace init --help
./target/release/workspace changeset --help

# 4. Verify scripts
bash -n scripts/install.sh
bash -n scripts/uninstall.sh

# 5. Check GitHub Actions syntax
# Use: https://github.com/rhysd/actionlint
actionlint .github/workflows/*.yml
```

---

## Success Criteria

- [ ] All files updated with new binary name
- [ ] No references to `wnt` in user-facing content
- [ ] All tests pass
- [ ] All clippy checks pass
- [ ] Binary builds successfully as `workspace`
- [ ] Scripts execute without errors
- [ ] GitHub Actions syntax is valid
- [ ] Documentation is consistent

---

## Notes

- This is a direct replacement without backward compatibility
- No migration guide needed
- Package namespace (`sublime_*`) remains unchanged
- Only the binary name changes: `wnt` → `workspace`
