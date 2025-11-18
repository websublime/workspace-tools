# workspace Git Hooks

Automated changeset management through a single, simple git hook.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [How It Works](#how-it-works)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage Examples](#usage-examples)
- [Troubleshooting](#troubleshooting)
- [FAQ](#faq)

## Overview

The workspace git hook provides automatic changeset synchronization with **one simple hook**:

- **pre-push**: Syncs all branch commits to changeset before pushing

### Key Benefits

✅ **Zero Manual Work**: All commits are synced automatically before push  
✅ **Simple**: Just one hook to understand  
✅ **No Loops**: Sync commits are maintenance-only, don't need tracking  
✅ **Clean**: Minimal commits, maximum efficiency  
✅ **Flexible**: Commit as many times as you want, sync happens once on push

### How It Works

```
Developer Workflow:
  1. Create changeset once:  workspace changeset create
  2. Make commits freely:    git commit (as many as you want)
  3. Push when ready:        git push
     → pre-push syncs ALL commits to changeset
     → Creates "chore: sync changeset" commit if needed
     → Push proceeds with everything
```

**Key Insight**: Sync commits (`chore: sync changeset for <branch>`) are **maintenance commits** and don't need to be in the changeset themselves. Only your feature/fix commits are tracked.

## Quick Start

### Installation

```bash
# From the project root
./scripts/install-hooks.sh
```

### Basic Workflow

```bash
# 1. Create feature branch
git checkout -b feature/amazing-feature

# 2. Create changeset (one time)
workspace changeset create
# → Interactive prompts for packages, version bump, etc.

# 3. Make commits (as many as you want, whenever you want)
git commit -m "feat: add core functionality"
git commit -m "feat: add validation"
git commit -m "test: add tests"
git commit -m "docs: update documentation"
git commit -m "refactor: improve performance"
# ... commit freely, no sync needed yet!

# 4. Push (one time or whenever ready)
git push origin feature/amazing-feature
# → pre-push hook runs:
#    ✓ Syncs all 5 commits to changeset
#    ✓ Creates "chore: sync changeset for feature/amazing-feature" commit
#    ✓ Pushes everything (your 5 commits + sync commit)
```

**That's it!** No manual sync, no per-commit overhead, simple and efficient.

## How It Works

### The pre-push Hook

**When it runs**: Before every `git push`

**What it does**:
1. Detects current branch
2. Skips main/master branches
3. Checks if changeset exists (blocks if missing)
4. Gets all commits from branch (excluding main/master commits)
5. Filters out previous sync commits (they don't need tracking)
6. Adds each commit SHA to changeset (workspace skips duplicates)
7. If changeset was modified:
   - Stages the changeset file
   - Creates commit: `chore: sync changeset for <branch>`
   - This commit is included in the push automatically
8. Allows push to proceed

**Example output (first push)**:
```
🔍 Syncing changeset...
✓ Changeset exists for branch feature/my-branch
📊 Syncing 5 commit(s)...
✓ Commits synced to changeset
📦 Creating sync commit...
✓ Sync commit created
ℹ This commit will be included in the push
✓ Ready to push
```

**Example output (subsequent push, no new commits)**:
```
🔍 Syncing changeset...
✓ Changeset exists for branch feature/my-branch
ℹ No commits to sync
✓ Ready to push
```

**Example output (no changeset)**:
```
✗ No changeset found for branch feature/my-branch

How to fix:
  1. Create changeset:     workspace changeset create
  2. Verify it was created: workspace changeset show feature/my-branch
  3. Push again:           git push

To skip this check once:
  WORKSPACE_SKIP_HOOKS=1 git push
```

**Skip once:**
```bash
WORKSPACE_SKIP_HOOKS=1 git push
```

### Why Sync Commits Don't Need Tracking

Sync commits (`chore: sync changeset for <branch>`) are **meta-commits** - they exist to update the changeset file, not to add features or fix bugs. Including them in the changeset would be redundant:

- They don't change the package code
- They only update the changeset JSON file
- They're created automatically by the hook
- Tracking them would pollute the changeset with noise

**Your feature commits ARE tracked**, which is what matters for versioning and changelogs.

## Installation

### Install Hook

```bash
./scripts/install-hooks.sh
```

### Force Reinstall

```bash
./scripts/install-hooks.sh --force
```

### Uninstall

```bash
./scripts/uninstall-hooks.sh
# or
./scripts/install-hooks.sh --uninstall
```

## Configuration

### Project Configuration

Add to `.workspace.toml`:

```toml
[git_hooks]
# Enable/disable hook globally
enabled = true

# Sync changeset before push
sync_on_push = true
```

### Environment Variables

```bash
# Disable hook temporarily
export WORKSPACE_SKIP_HOOKS=1

# Disable for single command
WORKSPACE_SKIP_HOOKS=1 git push
```

### Per-Repository Settings

```bash
# Disable hook for this repo only
git config hooks.workspace.enabled false
```

## Usage Examples

### Scenario 1: Normal Feature Development

```bash
# Create branch
git checkout -b feature/user-authentication

# Create changeset (one time)
workspace changeset create
# → Interactive prompts...
# ✓ Changeset created: .changesets/feature-user-authentication.json

# Develop feature (multiple commits over days/weeks)
git commit -m "feat: add login form"
git commit -m "feat: add password validation"  
git commit -m "test: add authentication tests"
git commit -m "docs: document authentication flow"
git commit -m "refactor: extract validation logic"
git commit -m "fix: handle edge case"

# Push when ready (maybe days later)
git push origin feature/user-authentication
# → pre-push:
#    ✓ Syncs all 6 commits to changeset
#    ✓ Creates "chore: sync changeset for feature/user-authentication"
#    ✓ Pushes 7 commits total (6 feature + 1 sync)
```

### Scenario 2: Multiple Pushes

```bash
# First push
git commit -m "feat: add feature part 1"
git commit -m "feat: add feature part 2"
git push
# → Syncs 2 commits, creates sync commit, pushes 3 commits

# Continue working
git commit -m "feat: add feature part 3"
git commit -m "test: add more tests"
git push
# → Syncs 2 NEW commits (skips already-synced ones), creates sync commit, pushes 3 commits

# Another push with no new commits
git push
# → No commits to sync, no sync commit created, push proceeds
```

### Scenario 3: Starting From Existing Branch (No Changeset Yet)

```bash
# You already have commits but no changeset
git checkout feature/existing-branch

# Install hook
./scripts/install-hooks.sh

# Try to push
git push
# → ✗ No changeset found for branch feature/existing-branch
# → Hook blocks push with instructions

# Create changeset now
workspace changeset create

# Push again
git push
# → pre-push:
#    ✓ Syncs ALL existing commits to changeset
#    ✓ Creates sync commit
#    ✓ Pushes everything
```

### Scenario 4: Emergency Hotfix (Skip Hook)

```bash
# Create hotfix branch
git checkout -b hotfix/critical-bug

# Make emergency fix (skip changeset for speed)
git commit -m "fix: critical security issue"

# Push immediately (skip hook)
WORKSPACE_SKIP_HOOKS=1 git push origin hotfix/critical-bug

# After emergency is resolved, create changeset
workspace changeset create
git push
# → Hook syncs the hotfix commit to changeset retroactively
```

### Scenario 5: Amending or Rebasing Commits

```bash
# Make commits
git commit -m "feat: add feature"
git commit -m "feat: add another feature"

# Oops, need to amend
git commit --amend -m "feat: add improved feature"

# Or rebase
git rebase -i HEAD~2

# Push
git push
# → Hook syncs the NEW commit SHAs (after amend/rebase)
# → Old SHAs remain in changeset (no harm, will be ignored)
# → If needed, manually clean up: workspace changeset update <branch> --remove-commit <old-sha>
```

## Troubleshooting

### Hook Not Running

**Check if hook is installed:**
```bash
ls -la .git/hooks/pre-push
```

**Reinstall hook:**
```bash
./scripts/install-hooks.sh --force
```

### workspace Command Not Found

**Install workspace:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/websublime/workspace-tools/releases/latest/download/sublime_cli_tools-installer.sh | sh
```

**Verify installation:**
```bash
workspace --version
```

### Push Blocked (No Changeset)

```
✗ No changeset found for branch feature/my-branch
```

**Fix**:
```bash
# Create changeset
workspace changeset create

# Push again
git push
```

### Hook Hangs or Freezes

**Possible causes**:
- `workspace` command waiting for input
- Git lock file present

**Fix**:
```bash
# Cancel with Ctrl+C
# Check for lock files
rm -f .git/index.lock

# Try again, or skip hook
WORKSPACE_SKIP_HOOKS=1 git push
```

### Permission Denied

**Make hook executable:**
```bash
chmod +x .git/hooks/pre-push
```

### Too Many Commits (Slow)

For branches with hundreds of commits:

```bash
# Sync manually first (faster)
git log main..HEAD --pretty=%H --grep="^chore: sync changeset" --invert-grep | \
  xargs -I {} workspace changeset update $(git branch --show-current) --commit {}

# Commit the changeset
git add .changesets/
git commit -m "chore: bulk sync changeset"

# Push (hook will skip, already synced)
git push
```

## Best Practices

### ✅ Do

- **Create changeset early** - Right after creating the branch
- **Commit freely** - Don't worry about syncing until push
- **Push when ready** - The hook handles everything
- **Install for all team members** - Ensures consistency
- **Document in project README** - Explain the workflow to new devs

### ❌ Don't

- **Don't use `--no-verify` routinely** - Only for emergencies
- **Don't manually edit changeset files** - Use `workspace changeset update`
- **Don't commit `.git/hooks`** - Hooks are installed per-developer
- **Don't track sync commits in changeset** - They're maintenance-only

### Recommended Workflow

1. **Branch creation**: `git checkout -b feature/name`
2. **Changeset creation**: `workspace changeset create` (immediately)
3. **Development**: `git commit` (as many times as needed, over days/weeks)
4. **Push**: `git push` (whenever ready - hook syncs everything)
5. **More development**: `git commit` (more commits)
6. **Push again**: `git push` (hook syncs only new commits)
7. **PR creation**: Create pull request
8. **Code review**: Review includes complete changeset
9. **Merge**: Merge or squash (changeset is complete)

## FAQ

### Q: Why only one hook now?

**A:** Simplicity and reliability. Previous approaches with multiple hooks (post-commit, etc.) caused issues:
- Infinite loops with `git commit --amend`
- Timing problems (last commit always missing)
- Complexity and confusion

The single pre-push hook is simple, reliable, and efficient.

### Q: Why don't sync commits need to be in the changeset?

**A:** Sync commits (`chore: sync changeset`) are meta-commits that only update the changeset file. They don't:
- Change package code
- Add features
- Fix bugs
- Need to appear in changelogs

Your actual feature/fix commits ARE tracked, which is what matters.

### Q: What if I push multiple times?

**A:** The hook is smart:
- First push: Syncs all commits
- Subsequent pushes: Syncs only NEW commits
- No new commits: No sync commit created

### Q: What if I have hundreds of commits?

**A:** The hook might be slow. Options:
1. Bulk sync manually first (see Troubleshooting)
2. Push more frequently (sync fewer commits each time)
3. Use `WORKSPACE_SKIP_HOOKS=1` and sync manually

### Q: Can I create the changeset after making commits?

**A:** Yes! Create the changeset anytime, then push. The hook will sync all existing commits automatically.

### Q: What if I amend or rebase commits?

**A:** The hook syncs the current commit SHAs. Old SHAs remain in changeset but are harmless (they're not in the branch anymore, so they're ignored).

### Q: Do hooks work on Windows?

**A:** Yes, via Git Bash or WSL.

### Q: What if workspace is not installed?

**A:** Hook shows a warning with installation instructions but doesn't block your workflow.

### Q: Can I disable the hook temporarily?

**A:** Yes:
```bash
WORKSPACE_SKIP_HOOKS=1 git push
```

### Q: How do I see what's in my changeset?

**A:**
```bash
workspace changeset show <branch-name>
```

### Q: What happens if I force-push?

**A:** The hook still runs before force-push. It syncs commits based on the current branch state.

### Q: Can I use this with rebasing workflows?

**A:** Yes! Rebase/squash your commits as needed, then push. The hook syncs the final commit SHAs.

## Architecture

### Why One Hook?

**Pre-push is the perfect sync point**:
- Runs once per push (efficient)
- All commits are finalized (no more changes)
- Can create sync commit and include it in the same push
- No loops or timing issues
- Simple to understand and debug

### Hook Execution Flow

```
Developer: git push origin feature/my-branch
  ↓
1. pre-push hook runs (before push to remote)
  ↓
2. Get all commits: main..HEAD (excluding sync commits)
   Example: [abc123, def456, ghi789]
  ↓
3. Add each to changeset:
   workspace changeset update feature/my-branch --commit abc123
   workspace changeset update feature/my-branch --commit def456
   workspace changeset update feature/my-branch --commit ghi789
  ↓
4. If changeset changed:
   git add .changesets/feature-my-branch.json
   git commit -m "chore: sync changeset for feature/my-branch" --no-verify
   → Creates commit jkl012
  ↓
5. Push proceeds with all commits:
   - abc123 (your commit)
   - def456 (your commit)
   - ghi789 (your commit)
   - jkl012 (sync commit - NOT in changeset, and that's OK!)
```

**Key insight**: Sync commit `jkl012` is NOT added to the changeset. It's a maintenance commit. Next push will NOT try to add it (filtered by `--grep --invert-grep`).

## Support

- **Documentation**: https://github.com/websublime/workspace-tools
- **Issues**: https://github.com/websublime/workspace-tools/issues
- **Discussions**: https://github.com/websublime/workspace-tools/discussions

## License

MIT or Apache-2.0
