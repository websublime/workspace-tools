# GitHub Merge Strategy Configuration

## Required Configuration

This repository **MUST** use **"Rebase and merge"** as the default merge strategy.

## How to Configure

### 1. Repository Settings (Admin/Maintainer Only)

1. Go to repository **Settings** → **General**
2. Scroll down to **"Pull Requests"** section
3. Configure merge options:
   - ✅ **Allow rebase merging** (ENABLED)
   - ❌ **Allow squash merging** (DISABLED)
   - ❌ **Allow merge commits** (OPTIONAL - can be disabled)

### 2. Why This Matters

Our monorepo uses `release-plz` + `git-cliff` for automated changelog generation:

- **Each crate needs its own changelog** (`crates/standard/CHANGELOG.md`, `crates/git/CHANGELOG.md`, etc.)
- **Git-cliff filters commits by path** using `--include-path crates/<name>/**/*`
- **Squash merge breaks this** because all changes are combined into one commit
- **Result**: Empty changelogs for packages not directly modified in the squashed commit

### 3. Branch Protection Rules (Recommended)

Consider adding these branch protection rules for `main`:

1. Go to **Settings** → **Branches** → **Branch protection rules**
2. Add rule for `main`:
   - ✅ Require status checks to pass before merging
   - ✅ Require branches to be up to date before merging
   - ✅ Require linear history (prevents merge commits)

## For Contributors

When merging a PR, always select **"Rebase and merge"** from the merge button dropdown.

If the option is not available, contact a repository maintainer to update the repository settings.
