# wnt Git Hooks

Automated changeset management through git hooks for seamless developer workflow.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Available Hooks](#available-hooks)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage Examples](#usage-examples)
- [Troubleshooting](#troubleshooting)
- [Integration with Other Tools](#integration-with-other-tools)

## Overview

The wnt git hooks automate changeset management throughout your development workflow:

- **pre-commit**: Automatically updates changesets when you commit
- **post-checkout**: Creates changesets when you start new feature branches
- **pre-push**: Validates changesets before pushing to remote
- **prepare-commit-msg**: Enhances commit messages with changeset info

### Benefits

✅ **Zero Friction**: Changesets are managed automatically  
✅ **Consistency**: Everyone on the team follows the same process  
✅ **Fewer Mistakes**: Never forget to update your changeset  
✅ **Better Commits**: Commit messages include changeset context  
✅ **Validation**: Catch issues before they reach remote

## Quick Start

### Installation

```bash
# From the project root
./scripts/install-hooks.sh
```

That's it! The hooks are now active.

### Basic Workflow

```bash
# 1. Create feature branch
git checkout -b feature/new-thing
# → Hook prompts to create changeset

# 2. Work and commit
git add file.js
git commit -m "feat: add new feature"
# → Hook automatically updates changeset

# 3. Push
git push origin feature/new-thing
# → Hook validates changeset before pushing
```

## Available Hooks

### pre-commit

**Purpose**: Auto-update changeset on every commit

**What it does:**
1. Detects current branch
2. Finds changeset for branch
3. Updates changeset with new commit info
4. Stages updated changeset file
5. Includes changeset in commit

**When it runs**: Before each `git commit`

**Example output:**
```
📝 Updating changeset...
✓ Changeset updated and staged
```

**Skip once:**
```bash
WNT_SKIP_HOOKS=1 git commit -m "message"
```

### post-checkout

**Purpose**: Create changeset for new feature branches

**What it does:**
1. Detects branch checkout
2. Checks if it's a feature branch
3. Checks if changeset exists
4. Prompts to create changeset (or creates automatically)

**When it runs**: After `git checkout -b` or `git checkout`

**Example output:**
```
📝 New feature branch detected: feature/my-branch
ℹ  No changeset found for this branch

? Create changeset now? [Y/n]
```

**Skip once:**
```bash
WNT_SKIP_HOOKS=1 git checkout -b feature/branch
```

### pre-push

**Purpose**: Validate changeset before pushing

**What it does:**
1. Detects current branch
2. Validates changeset exists
3. Validates changeset is properly formatted
4. Blocks push if validation fails

**When it runs**: Before `git push`

**Example output (success):**
```
🔍 Validating changeset...
✓ Changeset validation passed
```

**Example output (failure):**
```
✗ Changeset validation failed

Validation errors:
  - Missing package information
  - Invalid bump type

How to fix:
  1. Check your changeset: wnt changeset show
  2. Update if needed:     wnt changeset update
  3. Validate changes:     wnt changeset validate
  4. Commit and push:      git commit -m 'chore: update changeset' && git push
```

**Skip once:**
```bash
WNT_SKIP_HOOKS=1 git push
```

### prepare-commit-msg

**Purpose**: Enhance commit messages with changeset info

**What it does:**
1. Reads changeset for current branch
2. Appends changeset metadata to commit message
3. Shows affected packages and bump type

**When it runs**: When preparing commit message

**Example addition to commit:**
```
# ──────────────────────────────────────────────────────────────
# Changeset Info (added by wnt)
# Branch: feature/my-branch
# Packages: @myorg/api, @myorg/ui
# Bump: minor
# ──────────────────────────────────────────────────────────────
```

**Skip once:**
```bash
WNT_SKIP_HOOKS=1 git commit -m "message"
```

## Installation

### Install All Hooks (Recommended)

```bash
./scripts/install-hooks.sh
```

### Install Specific Hooks

```bash
# Only pre-commit
./scripts/install-hooks.sh --pre-commit

# Only validation (pre-push)
./scripts/install-hooks.sh --pre-push

# Multiple specific hooks
./scripts/install-hooks.sh --pre-commit --pre-push
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

Add to `.wnt.toml`:

```toml
[git_hooks]
# Enable/disable hooks
enabled = true

# Auto-update changeset on commit
auto_update_on_commit = true

# Prompt for changeset creation on checkout
prompt_for_changeset = true

# Validate on push
validate_on_push = true

# Enhance commit messages
enhance_commit_messages = true

# Strict validation (fail on warnings)
strict_validation = false
```

### Environment Variables

```bash
# Disable all hooks temporarily
export WNT_SKIP_HOOKS=1

# Disable for single command
WNT_SKIP_HOOKS=1 git commit -m "message"
```

### Per-Repository Settings

```bash
# Disable hooks for this repo only
git config hooks.wnt.enabled false

# Enable strict validation
git config hooks.wnt.strict true
```

## Usage Examples

### Scenario 1: Start New Feature

```bash
# Create branch
git checkout -b feature/add-login

# → Hook prompts:
📝 New feature branch detected: feature/add-login
ℹ  No changeset found for this branch

? Create changeset now? [Y/n] y

? Select packages (space to select):
  ❯ ◯ @myorg/api
    ◯ @myorg/ui
    ◯ @myorg/auth

? Version bump type: › minor

? Target environments:
  ❯ ☑ development
    ☑ staging
    ☐ production

✓ Changeset created: .changesets/feature-add-login.json
```

### Scenario 2: Regular Development

```bash
# Work on feature
vim src/login.js

# Commit changes
git add src/login.js
git commit -m "feat: implement login form"

# → Hook automatically:
📝 Updating changeset...
✓ Changeset updated and staged

# Result: Single commit with code + updated changeset
```

### Scenario 3: Push to Remote

```bash
git push origin feature/add-login

# → Hook validates:
🔍 Validating changeset...
✓ Changeset validation passed

# Push proceeds
```

### Scenario 4: Validation Fails

```bash
git push

# → Hook detects issue:
✗ Changeset validation failed

Validation errors:
  - Changeset out of sync with commits

How to fix:
  1. Update: wnt changeset update
  2. Commit: git commit -m 'chore: update changeset'
  3. Push:   git push

# Fix it:
wnt changeset update
git add .changesets/
git commit -m "chore: update changeset"
git push
```

## Troubleshooting

### Hook Not Running

**Check if hooks are installed:**
```bash
ls -la .git/hooks/pre-commit
ls -la .git/hooks/post-checkout
ls -la .git/hooks/pre-push
```

**Reinstall hooks:**
```bash
./scripts/install-hooks.sh --force
```

### wnt Command Not Found

**Install wnt:**
```bash
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh
```

**Or use full path:**
```bash
# Edit hook file and use absolute path
/usr/local/bin/wnt changeset update
```

### Hook Fails on Commit

**Check what's wrong:**
```bash
wnt changeset validate
```

**Skip hook temporarily:**
```bash
WNT_SKIP_HOOKS=1 git commit -m "message"
```

**Disable permanently:**
```bash
./scripts/uninstall-hooks.sh
```

### Permission Denied

**Make hooks executable:**
```bash
chmod +x .git/hooks/pre-commit
chmod +x .git/hooks/post-checkout
chmod +x .git/hooks/pre-push
chmod +x .git/hooks/prepare-commit-msg
```

### Hooks Too Slow

**Disable prepare-commit-msg (optional):**
```bash
rm .git/hooks/prepare-commit-msg
```

**Use --quiet flag:**
```bash
# Edit hook files, add --quiet:
wnt changeset update --quiet
```

**Disable for large repos:**
```toml
# .wnt.toml
[git_hooks]
auto_update_on_commit = false  # Use manual updates instead
```

## Integration with Other Tools

### Husky

If you're using Husky:

```bash
# .husky/pre-commit
#!/usr/bin/env sh
. "$(dirname -- "$0")/_/husky.sh"

# Run wnt changeset update
wnt changeset update --auto --quiet
git add .changesets/*.json 2>/dev/null || true
```

### Lefthook

If you're using Lefthook:

```yaml
# lefthook.yml
pre-commit:
  commands:
    changeset-update:
      run: wnt changeset update --auto --quiet && git add .changesets/*.json

pre-push:
  commands:
    changeset-validate:
      run: wnt changeset validate
```

### Git Aliases

Useful aliases:

```bash
# Skip hooks for one commit
git config --global alias.commit-skip 'commit --no-verify'

# Force update changeset
git config --global alias.cs-update '!wnt changeset update'

# Validate changeset
git config --global alias.cs-validate '!wnt changeset validate'

# Usage:
git commit-skip -m "wip"
git cs-update
git cs-validate
```

### CI/CD Integration

GitHub Actions example:

```yaml
name: Validate Changeset

on:
  pull_request:
    branches: [main]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install wnt
        run: curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh
      
      - name: Validate changeset
        run: wnt changeset validate --strict
```

## Best Practices

### ✅ Do

- Install hooks for all team members
- Commit the `.wnt.toml` configuration
- Use `--quiet` flags for faster hooks
- Document hook behavior in project README
- Test hooks before deploying to team

### ❌ Don't

- Don't commit `.git/hooks` directory
- Don't override hooks without understanding them
- Don't use hooks for heavy operations (keep them fast)
- Don't skip validation hooks regularly
- Don't disable hooks without team agreement

## FAQ

### Q: Can I customize the hooks?

**A:** Yes! The hook scripts are in `scripts/git-hooks/`. Copy and modify them, or configure behavior via `.wnt.toml`.

### Q: Do hooks work on Windows?

**A:** Yes, via Git Bash or WSL. Native Windows support coming soon.

### Q: Can I disable hooks temporarily?

**A:** Yes:
```bash
WNT_SKIP_HOOKS=1 git commit
```

### Q: What if wnt is not installed?

**A:** Hooks gracefully skip if wnt is not found. They won't break your workflow.

### Q: Can I use hooks with Husky?

**A:** Yes! See [Integration with Other Tools](#integration-with-other-tools).

### Q: Do hooks affect performance?

**A:** Minimal impact (<500ms typically). Use `--quiet` flags for faster execution.

## Support

- **Documentation**: https://github.com/websublime/workspace-node-tools
- **Issues**: https://github.com/websublime/workspace-node-tools/issues
- **Discussions**: https://github.com/websublime/workspace-node-tools/discussions

## License

MIT or Apache-2.0
