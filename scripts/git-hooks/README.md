# workspace Git Hooks

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

The workspace git hooks automate changeset management throughout your development workflow:

- **pre-commit**: Validates changeset exists, prompts to create if missing
- **post-checkout**: Creates changesets when you start new feature branches
- **pre-push**: Adds all branch commits and creates a commit with updated changeset
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

# 2. Make first commit
git add file.js
git commit -m "feat: add new feature"
# → pre-commit: "No changeset found. Create one? [Y/n]"
# → User: Y (creates changeset interactively)
# → Commit proceeds

# 3. Make more commits (as many as you want)
git add another.js  
git commit -m "feat: improve feature"
# → pre-commit: ✓ Changeset exists

# 4. Push (commits are added here)
git push origin feature/new-thing
# → pre-push: Adds ALL branch commits to changeset
# → pre-push: Creates commit "chore: update changeset for feature/new-thing"
# → Push includes all your commits + changeset commit
```

## Available Hooks

### pre-commit

**Purpose**: Ensure changeset exists, prompting to create interactively if missing

**What it does:**
1. Detects current branch
2. Skips main/master branches
3. Checks if changeset exists for the branch
4. If missing (interactive mode):
   - Prompts: "Create changeset now? [Y/n]"
   - If yes (default): Runs `workspace changeset create` interactively
   - If no: Asks if user wants to continue without changeset
5. If missing (non-interactive mode): Warns but allows commit
6. Allows commit to proceed

**When it runs**: Before each `git commit`

**Example output (changeset exists):**
```
✓ Changeset found for branch feature/my-branch
```

**Example output (no changeset - user creates):**
```
⚠ No changeset found for branch feature/my-branch

? Create changeset now? [Y/n] 

📝 Creating changeset...
[Interactive prompts...]
✓ Changeset created successfully
ℹ Continuing with commit...
```

**Example output (user declines):**
```
⚠ No changeset found for branch feature/my-branch

? Create changeset now? [Y/n] n
? Continue commit without changeset? [y/N] y
⚠ Proceeding without changeset
```

**Skip once:**
```bash
WORKSPACE_SKIP_HOOKS=1 git commit -m "message"
```

### post-commit

**Purpose**: Automatically add commit SHA to changeset after commit is created

**What it does:**
1. Detects current branch
2. Skips main/master branches
3. Gets the SHA of the just-created commit
4. Adds the SHA to the changeset's changes array
5. Amends the commit to include the updated changeset

**When it runs**: After each `git commit`

**Example output:**
```
📝 Adding commit to changeset...
✓ Changeset updated and included in commit
```

**Why amend?**: The commit is amended so that the changeset update is included in the same commit, keeping the history clean without extra "update changeset" commits.

**Skip once:**
```bash
WORKSPACE_SKIP_HOOKS=1 git commit -m "message"
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
WORKSPACE_SKIP_HOOKS=1 git checkout -b feature/branch
```

### pre-push

**Purpose**: Add all branch commits to changeset and create a commit with the update before pushing

**What it does:**
1. Detects current branch
2. Skips main/master branches
3. Checks if changeset exists (blocks push if missing)
4. Gets all commits from branch that aren't in main
5. Adds each commit SHA to the changeset's changes array (bulk update)
6. If changeset was modified:
   - Stages the changeset file
   - Creates a commit: `chore: update changeset for <branch-name>`
   - This commit is included automatically in the push
7. Allows push to continue

**When it runs**: Before `git push`

**Example output (with updates):**
```
🔍 Checking changeset...
✓ Changeset exists for branch feature/my-branch
📝 Adding branch commits to changeset...
ℹ Found 3 commit(s)
✓ Commits added to changeset
📦 Committing updated changeset...
✓ Changeset committed
✓ Ready to push
```

**Example output (already up-to-date):**
```
🔍 Checking changeset...
✓ Changeset exists for branch feature/my-branch
📝 Adding branch commits to changeset...
ℹ No commits to add
ℹ Changeset already up-to-date
✓ Ready to push
```

**Example output (no changeset):**
```
✗ No changeset found for branch feature/my-branch

How to fix:
  1. Create changeset:     workspace changeset create
  2. Verify it was created: workspace changeset show feature/my-branch
  3. Push again:           git push

To skip this check once:
  WORKSPACE_SKIP_HOOKS=1 git push
```

**Why bulk update on push?**: Adding commits in bulk before push (instead of per-commit) avoids issues with git commit amending and keeps the workflow simple. You can make multiple commits freely, and they're all tracked when you push. The automatic commit ensures the changeset is always included in the push.

**Skip once:**
```bash
WORKSPACE_SKIP_HOOKS=1 git push
```

### prepare-commit-msg

**Purpose**: Enhance commit messages with changeset info

**What it does:**
1. Detects current branch
2. Checks if changeset exists for the branch
3. Appends changeset reference to commit message

**When it runs**: When preparing commit message

**Example addition to commit:**
```
# ──────────────────────────────────────────────────────────────
# Changeset Info (added by workspace)
# Branch: feature/my-branch
# Run 'workspace changeset show feature/my-branch' to see details
# ──────────────────────────────────────────────────────────────
```

**Skip once:**
```bash
WORKSPACE_SKIP_HOOKS=1 git commit -m "message"
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

# Only post-commit
./scripts/install-hooks.sh --post-commit

# Only validation (pre-push)
./scripts/install-hooks.sh --pre-push

# Multiple specific hooks
./scripts/install-hooks.sh --pre-commit --post-commit --pre-push
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
# Enable/disable hooks
enabled = true

# Require changeset before allowing commit (pre-commit)
require_changeset = true

# Prompt for changeset creation on checkout (post-checkout)
prompt_for_changeset = true

# Validate changeset exists before push (pre-push)
validate_on_push = true

# Enhance commit messages with changeset info (prepare-commit-msg)
enhance_commit_messages = true
```

### Environment Variables

```bash
# Disable all hooks temporarily
export WORKSPACE_SKIP_HOOKS=1

# Disable for single command
WORKSPACE_SKIP_HOOKS=1 git commit -m "message"
```

### Per-Repository Settings

```bash
# Disable hooks for this repo only
git config hooks.workspace.enabled false

# Enable strict validation
git config hooks.workspace.strict true
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
  1. Update: workspace changeset update
  2. Commit: git commit -m 'chore: update changeset'
  3. Push:   git push

# Fix it:
workspace changeset update
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

### workspace Command Not Found

**Install workspace:**
```bash
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh
```

**Or use full path:**
```bash
# Edit hook file and use absolute path
/usr/local/bin/workspace changeset update
```

### Hook Fails on Commit

**Check what's wrong:**
```bash
workspace changeset validate
```

**Skip hook temporarily:**
```bash
WORKSPACE_SKIP_HOOKS=1 git commit -m "message"
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
workspace changeset update --quiet
```

**Disable for large repos:**
```toml
# .workspace.toml
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

# Run workspace changeset update
workspace changeset update --auto --quiet
git add .changesets/*.json 2>/dev/null || true
```

### Lefthook

If you're using Lefthook:

```yaml
# lefthook.yml
pre-commit:
  commands:
    changeset-update:
      run: workspace changeset update --auto --quiet && git add .changesets/*.json

pre-push:
  commands:
    changeset-validate:
      run: workspace changeset validate
```

### Git Aliases

Useful aliases:

```bash
# Skip hooks for one commit
git config --global alias.commit-skip 'commit --no-verify'

# Force update changeset
git config --global alias.cs-update '!workspace changeset update'

# Validate changeset
git config --global alias.cs-validate '!workspace changeset validate'

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
      
      - name: Install workspace
        run: curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh
      
      - name: Validate changeset
        run: workspace changeset validate --strict
```

## Best Practices

### ✅ Do

- Install hooks for all team members
- Commit the `.workspace.toml` configuration
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

**A:** Yes! The hook scripts are in `scripts/git-hooks/`. Copy and modify them, or configure behavior via `.workspace.toml`.

### Q: Do hooks work on Windows?

**A:** Yes, via Git Bash or WSL. Native Windows support coming soon.

### Q: Can I disable hooks temporarily?

**A:** Yes:
```bash
WORKSPACE_SKIP_HOOKS=1 git commit
```

### Q: What if workspace is not installed?

**A:** Hooks gracefully skip if workspace is not found. They won't break your workflow.

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
