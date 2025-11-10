# Workspace Tools - Command Reference

**Version:** 0.1.0  
**Last Updated:** 2025-11-07

---

## Table of Contents

1. [Introduction](#introduction)
2. [Global Options](#global-options)
3. [Commands](#commands)
   - [init](#init---initialize-project-configuration)
   - [config](#config---manage-configuration)
   - [changeset](#changeset---manage-changesets)
   - [bump](#bump---bump-package-versions)
   - [upgrade](#upgrade---manage-dependency-upgrades)
   - [audit](#audit---run-project-health-audit)
   - [changes](#changes---analyze-changes)
   - [version](#version---display-version-information)
4. [Quick Reference](#quick-reference)
5. [Common Patterns](#common-patterns)
6. [Exit Codes](#exit-codes)

---

## Introduction

This document provides a comprehensive reference for all `workspace` CLI commands, their options, and usage examples. Each command includes:

- **Synopsis**: Brief command syntax
- **Description**: What the command does
- **Options**: All available flags and their meanings
- **Examples**: Common usage patterns
- **Output Examples**: Sample command output in different formats

### Command Structure

```
workspace [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGS]
```

### Getting Help

For help on any command:

```bash
workspace --help                    # General help
workspace <command> --help          # Command-specific help
workspace <command> <subcommand> --help  # Subcommand-specific help
```

---

## Global Options

Global options apply to **ALL** commands and control behavior across the entire application.

### Key Principles

1. **Stream Separation**
   - **stderr**: Logs only (controlled by `--log-level`)
   - **stdout**: Command output only (controlled by `--format`)
   - These streams are completely independent

2. **Independence**
   - Logging and output format are independent
   - You can have JSON output with any log level
   - You can have text output with any log level

### Options

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--root <PATH>` | `-r` | Path | Current dir | Project root directory |
| `--log-level <LEVEL>` | `-l` | Enum | `info` | Logging verbosity (stderr) |
| `--format <FORMAT>` | `-f` | Enum | `human` | Output format (stdout) |
| `--no-color` | | Flag | false | Disable colored output |
| `--config <PATH>` | `-c` | Path | Auto-detect | Path to config file |
| `--help` | `-h` | Flag | | Show help |
| `--version` | `-V` | Flag | | Show version |

### Log Levels

Controls what logs are written to **stderr**:

- `silent`: No logs at all (clean output only)
- `error`: Only critical errors
- `warn`: Errors + warnings
- `info`: General progress (default)
- `debug`: Detailed operations
- `trace`: Very verbose debugging

### Output Formats

Controls format of command output written to **stdout**:

- `human`: Human-readable with colors and tables (default)
- `json`: Pretty-printed JSON
- `json-compact`: Compact JSON (single line)
- `quiet`: Minimal output

### Global Options Examples

```bash
# Change working directory
workspace --root /path/to/project init

# JSON output with NO logs (clean JSON for automation)
workspace --format json --log-level silent bump --dry-run

# JSON output WITH debug logs (logs to stderr, JSON to stdout)
workspace --format json --log-level debug bump --dry-run > output.json 2> debug.log

# Text output with no logs
workspace --log-level silent changeset list

# Debug logging with text output
workspace --log-level debug upgrade check

# Disable colors (for CI/CD or file redirection)
workspace --no-color audit > report.txt

# Use specific config file
workspace --config ./test-config.yaml init

# Separate output and logs to different files
workspace --format json --log-level debug bump --execute > output.json 2> process.log
```

---

## Commands

## `init` - Initialize Project Configuration

Initialize a new or existing project with workspace configuration.

### Synopsis

```bash
workspace init [OPTIONS]
```

### Description

Creates a configuration file for changeset-based version management. Supports both interactive and non-interactive modes. Detects project type (single package or monorepo) and package manager automatically.

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--changeset-path <PATH>` | Path | `.changesets` | Changeset directory path |
| `--environments <LIST>` | CSV | Prompt | Comma-separated environments |
| `--default-env <LIST>` | CSV | Prompt | Default environments |
| `--strategy <STRATEGY>` | String | Prompt | Versioning strategy (`independent` or `unified`) |
| `--registry <URL>` | URL | `https://registry.npmjs.org` | NPM registry URL |
| `--config-format <FORMAT>` | String | Prompt | Config format (`json`, `toml`, `yaml`) |
| `--force` | Flag | false | Overwrite existing config |
| `--non-interactive` | Flag | false | Skip prompts, use defaults/flags |

### Examples

```bash
# Interactive mode (recommended for first-time setup)
workspace init

# Non-interactive with all options
workspace init \
  --non-interactive \
  --strategy unified \
  --config-format yaml \
  --environments "dev,staging,prod" \
  --default-env "prod"

# Minimal non-interactive (uses defaults)
workspace init --non-interactive --config-format json

# Force overwrite existing config
workspace init --force

# JSON output for automation
workspace --format json init --non-interactive > init-result.json
```

### Output Example (Human Format)

```
✓ Configuration initialized successfully

  Config file: repo.config.yaml
  Strategy: independent
  Changesets: .changesets/
  Environments: dev, staging, production
  Default: production
  Registry: https://registry.npmjs.org

Next steps:
  1. Create a feature branch: git checkout -b feature/my-feature
  2. Create a changeset: workspace changeset create
  3. Make changes and commit
  4. Update changeset: workspace changeset update
```

### Output Example (JSON Format)

```json
{
  "success": true,
  "configFile": "repo.config.yaml",
  "configFormat": "yaml",
  "strategy": "independent",
  "changesetPath": ".changesets/",
  "environments": ["dev", "staging", "production"],
  "defaultEnvironments": ["production"],
  "registry": "https://registry.npmjs.org"
}
```

---

## `config` - Manage Configuration

View, validate, and manage project configuration.

### Subcommands

- `config show` - Display current configuration
- `config validate` - Validate configuration file

### Synopsis

```bash
workspace config show [OPTIONS]
workspace config validate [OPTIONS]
```

### Description

The `config` command manages workspace configuration files. It can display current settings, validate configuration integrity, and provide feedback on configuration issues.

### Options (config show)

No command-specific options beyond global options.

### Options (config validate)

No command-specific options beyond global options.

### Examples

```bash
# Show current configuration
workspace config show

# Show configuration as JSON
workspace --format json config show

# Validate configuration
workspace config validate

# Validate with JSON output
workspace --format json config validate

# Show config from specific file
workspace --config ./custom.yaml config show
```

### Output Example (config show, Human Format)

```
Configuration
━━━━━━━━━━━━━

Strategy: independent
Changeset Path: .changesets/
Environments: dev, staging, production
Default Environments: production
Registry: https://registry.npmjs.org

Version Settings:
  Default Bump: patch
  Snapshot Format: {version}-{branch}.{short_commit}

Changelog:
  Enabled: true
  Path: CHANGELOG.md
```

### Output Example (config show, JSON Format)

```json
{
  "success": true,
  "config": {
    "changeset": {
      "path": ".changesets/",
      "environments": ["dev", "staging", "production"],
      "defaultEnvironments": ["production"]
    },
    "version": {
      "strategy": "independent",
      "defaultBump": "patch",
      "snapshotFormat": "{version}-{branch}.{short_commit}"
    },
    "changelog": {
      "enabled": true,
      "path": "CHANGELOG.md"
    },
    "registry": "https://registry.npmjs.org"
  }
}
```

### Output Example (config validate, Human Format)

```
✓ Configuration is valid

All checks passed:
  ✓ Config file exists
  ✓ All required fields present
  ✓ Environments valid (no duplicates)
  ✓ Changeset directory exists
  ✓ Strategy is valid
  ✓ Registry URL is valid
```

### Output Example (config validate, JSON Format)

```json
{
  "success": true,
  "valid": true,
  "checks": [
    { "name": "Config file exists", "passed": true },
    { "name": "All required fields present", "passed": true },
    { "name": "Environments valid", "passed": true },
    { "name": "Changeset directory exists", "passed": true },
    { "name": "Strategy is valid", "passed": true },
    { "name": "Registry URL is valid", "passed": true }
  ]
}
```

---

## `changeset` - Manage Changesets

Create, update, list, and manage changesets for version control.

### Subcommands

- `changeset create` - Create a new changeset
- `changeset update [ID]` - Update an existing changeset
- `changeset list` - List all changesets
- `changeset show <BRANCH>` - Show changeset details
- `changeset edit [BRANCH]` - Edit changeset in editor
- `changeset delete <BRANCH>` - Delete a changeset
- `changeset history` - Query archived changesets
- `changeset check` - Check if changeset exists

### Synopsis

```bash
workspace changeset create [OPTIONS]
workspace changeset update [ID] [OPTIONS]
workspace changeset list [OPTIONS]
workspace changeset show <BRANCH>
workspace changeset edit [BRANCH]
workspace changeset delete <BRANCH> [OPTIONS]
workspace changeset history [OPTIONS]
workspace changeset check [OPTIONS]
```

---

### `changeset create`

Create a new changeset for the current branch.

#### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--bump <TYPE>` | String | Prompt | Bump type (`major`, `minor`, `patch`) |
| `--env <LIST>` | CSV | Prompt | Comma-separated environments |
| `--branch <NAME>` | String | Current branch | Branch name |
| `--message <TEXT>` | String | Empty | Changeset message |
| `--packages <LIST>` | CSV | Auto-detect | Comma-separated packages |
| `--non-interactive` | Flag | false | Skip prompts |

#### Examples

```bash
# Interactive mode (recommended)
workspace changeset create

# Non-interactive with all options
workspace changeset create \
  --bump minor \
  --env "staging,prod" \
  --message "Add new API endpoint"

# Create for specific packages
workspace changeset create \
  --bump patch \
  --packages "@org/core,@org/utils"

# JSON output
workspace --format json changeset create \
  --bump minor \
  --non-interactive
```

#### Output Example (Human Format)

```
✓ Changeset created successfully

  Branch: feature/new-api
  ID: cs_abc123def
  Bump: minor
  Environments: staging, production
  Packages: @org/core (auto-detected)
  File: .changesets/feature-new-api.json

Next steps:
  1. Make your changes and commit
  2. Run: workspace changeset update
```

#### Output Example (JSON Format)

```json
{
  "success": true,
  "changeset": {
    "id": "cs_abc123def",
    "branch": "feature/new-api",
    "bump": "minor",
    "environments": ["staging", "production"],
    "packages": ["@org/core"],
    "commits": [],
    "file": ".changesets/feature-new-api.json",
    "createdAt": "2025-11-07T10:00:00Z"
  }
}
```

---

### `changeset update`

Update an existing changeset with new commits and packages.

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<ID>` | No | Changeset ID or branch name (defaults to current branch) |

#### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--commit <HASH>` | String | | Add specific commit hash |
| `--packages <LIST>` | CSV | Auto-detect | Add specific packages |
| `--bump <TYPE>` | String | | Update bump type |
| `--env <LIST>` | CSV | | Add environments |

#### Examples

```bash
# Update current branch's changeset (auto-detects)
workspace changeset update

# Update specific changeset by branch name
workspace changeset update feature/my-feature

# Add specific commit
workspace changeset update --commit abc123

# Add specific packages
workspace changeset update --packages "@org/core,@org/cli"

# Change bump type
workspace changeset update --bump major

# JSON output
workspace --format json changeset update
```

#### Output Example (Human Format)

```
✓ Changeset updated successfully

  Branch: feature/new-api
  ID: cs_abc123def
  
  Added:
    Commits: 1 (abc123)
    Packages: @org/utils (detected from changes)
  
  Current state:
    Total commits: 3
    Total packages: @org/core, @org/utils
    Bump: minor
```

#### Output Example (JSON Format)

```json
{
  "success": true,
  "changeset": {
    "id": "cs_abc123def",
    "branch": "feature/new-api",
    "bump": "minor",
    "packages": ["@org/core", "@org/utils"],
    "commits": ["abc123", "def456", "ghi789"],
    "environments": ["staging", "production"],
    "updatedAt": "2025-11-07T14:30:00Z"
  },
  "added": {
    "commits": ["abc123"],
    "packages": ["@org/utils"]
  }
}
```

---

### `changeset list`

List all active changesets.

#### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--filter-package <NAME>` | String | | Filter by package name |
| `--filter-bump <TYPE>` | String | | Filter by bump type |
| `--filter-env <ENV>` | String | | Filter by environment |
| `--sort <FIELD>` | String | `date` | Sort by field (`date`, `bump`, `branch`) |

#### Examples

```bash
# List all changesets
workspace changeset list

# Filter by bump type
workspace changeset list --filter-bump major

# Filter by package
workspace changeset list --filter-package "@org/core"

# Sort by branch name
workspace changeset list --sort branch

# JSON output
workspace --format json changeset list
```

#### Output Example (Human Format)

```
Active Changesets
━━━━━━━━━━━━━━━━━

feature/new-api (minor)
  Packages: @org/core, @org/utils
  Environments: production, staging
  Commits: 5
  Created: 2025-11-07 10:00:00

hotfix/security (patch)
  Packages: @org/utils
  Environments: production
  Commits: 2
  Created: 2025-11-06 09:00:00

breaking/v2-api (major)
  Packages: @org/core, @org/cli
  Environments: production
  Commits: 12
  Created: 2025-11-05 14:00:00

Total: 3 changesets
```

#### Output Example (JSON Format)

```json
{
  "success": true,
  "changesets": [
    {
      "id": "cs_abc123",
      "branch": "feature/new-api",
      "bump": "minor",
      "packages": ["@org/core", "@org/utils"],
      "environments": ["production", "staging"],
      "commits": ["abc123", "def456", "ghi789", "jkl012", "mno345"],
      "commitCount": 5,
      "createdAt": "2025-11-07T10:00:00Z",
      "updatedAt": "2025-11-07T14:30:00Z"
    },
    {
      "id": "cs_def456",
      "branch": "hotfix/security",
      "bump": "patch",
      "packages": ["@org/utils"],
      "environments": ["production"],
      "commits": ["pqr678", "stu901"],
      "commitCount": 2,
      "createdAt": "2025-11-06T09:00:00Z",
      "updatedAt": "2025-11-06T09:15:00Z"
    },
    {
      "id": "cs_ghi789",
      "branch": "breaking/v2-api",
      "bump": "major",
      "packages": ["@org/core", "@org/cli"],
      "environments": ["production"],
      "commits": ["vwx234", "yz567", "..."],
      "commitCount": 12,
      "createdAt": "2025-11-05T14:00:00Z",
      "updatedAt": "2025-11-07T12:00:00Z"
    }
  ],
  "total": 3
}
```

---

### `changeset show`

Show detailed information about a specific changeset.

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<BRANCH>` | Yes | Branch name or changeset ID |

#### Examples

```bash
# Show changeset by branch name
workspace changeset show feature/new-api

# Show with JSON output
workspace --format json changeset show feature/new-api
```

#### Output Example (Human Format)

```
Changeset Details
━━━━━━━━━━━━━━━━━

Branch: feature/new-api
ID: cs_abc123def
Bump: minor
Environments: production, staging
Created: 2025-11-07 10:00:00
Updated: 2025-11-07 14:30:00

Packages (2):
  • @org/core
  • @org/utils

Commits (5):
  abc123 - feat: add new API endpoint (2025-11-07 10:15:00)
  def456 - refactor: improve error handling (2025-11-07 11:00:00)
  ghi789 - test: add API tests (2025-11-07 12:00:00)
  jkl012 - docs: update API documentation (2025-11-07 13:00:00)
  mno345 - fix: address review feedback (2025-11-07 14:30:00)

Message:
  Add comprehensive REST API with authentication and rate limiting.
  Includes full test coverage and documentation.
```

#### Output Example (JSON Format)

```json
{
  "success": true,
  "changeset": {
    "id": "cs_abc123def",
    "branch": "feature/new-api",
    "bump": "minor",
    "environments": ["production", "staging"],
    "packages": ["@org/core", "@org/utils"],
    "commits": [
      {
        "hash": "abc123",
        "message": "feat: add new API endpoint",
        "author": "John Doe",
        "timestamp": "2025-11-07T10:15:00Z"
      },
      {
        "hash": "def456",
        "message": "refactor: improve error handling",
        "author": "John Doe",
        "timestamp": "2025-11-07T11:00:00Z"
      }
    ],
    "message": "Add comprehensive REST API with authentication and rate limiting.\nIncludes full test coverage and documentation.",
    "createdAt": "2025-11-07T10:00:00Z",
    "updatedAt": "2025-11-07T14:30:00Z"
  }
}
```

---

### `changeset edit`

Open changeset file in the user's editor for manual editing.

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<BRANCH>` | No | Branch name (defaults to current branch) |

#### Examples

```bash
# Edit current branch's changeset
workspace changeset edit

# Edit specific branch's changeset
workspace changeset edit feature/my-feature
```

---

### `changeset delete`

Delete a changeset.

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<BRANCH>` | Yes | Branch name to delete changeset for |

#### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--force` | Flag | false | Skip confirmation prompt |

#### Examples

```bash
# Delete with confirmation
workspace changeset delete old-feature

# Force delete without confirmation
workspace changeset delete old-feature --force

# JSON output
workspace --format json changeset delete old-feature --force
```

---

### `changeset history`

Query archived changesets.

#### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--package <NAME>` | String | | Filter by package name |
| `--since <DATE>` | String | | Since date (ISO 8601) |
| `--until <DATE>` | String | | Until date (ISO 8601) |
| `--env <ENV>` | String | | Filter by environment |
| `--bump <TYPE>` | String | | Filter by bump type |
| `--limit <N>` | Number | | Limit number of results |

#### Examples

```bash
# Query all history
workspace changeset history

# Filter by package
workspace changeset history --package "@org/core"

# Filter by date range
workspace changeset history \
  --since "2025-01-01" \
  --until "2025-03-31"

# Limit results
workspace changeset history --limit 10

# JSON output
workspace --format json changeset history --package "@org/core"
```

---

### `changeset check`

Check if a changeset exists for the current or specified branch.

#### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--branch <NAME>` | String | Current branch | Branch name to check |

#### Examples

```bash
# Check current branch
workspace changeset check

# Check specific branch
workspace changeset check --branch feature/my-feature

# JSON output (useful for Git hooks)
workspace --format json changeset check
```

#### Output Example (JSON Format)

```json
{
  "success": true,
  "exists": true,
  "changeset": {
    "id": "cs_abc123",
    "branch": "feature/new-api"
  }
}
```

---

## `bump` - Bump Package Versions

Bump package versions based on changesets.

### Synopsis

```bash
workspace bump [OPTIONS]
```

### Description

Calculates and applies version bumps according to active changesets and the configured versioning strategy.

### Bump Modes

The `bump` command operates in one of three modes:

1. **Preview Mode** (default)
   - Activated when: No action flags provided, or `--dry-run` flag
   - Shows what would change without modifying any files
   - Safe to run anytime - no side effects
   - This is the **DEFAULT** behavior for safety

2. **Execute Mode**
   - Activated when: `--execute` flag
   - Applies version bumps and updates files
   - Requires explicit flag to prevent accidents
   - Can be combined with git flags for full release workflow

3. **Snapshot Mode**
   - Activated when: `--snapshot` flag
   - Generates snapshot versions for feature branches
   - Format: `{version}-{branch}.{shortcommit}`
   - Can be combined with `--execute` to apply snapshots

**Safety Design**: Preview mode is the default to prevent accidental version bumps.
You must explicitly use `--execute` to modify files.

**Behavior by Strategy:**
- **Single Repository**: Bumps the single package version
- **Monorepo (Independent)**: Bumps only packages listed in changesets
- **Monorepo (Unified)**: Bumps all workspace packages to the same version

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--dry-run` | Flag | false | Preview changes without applying |
| `--execute` | Flag | false | Apply version changes (required) |
| `--snapshot` | Flag | false | Generate snapshot versions |
| `--snapshot-format <FORMAT>` | String | From config | Snapshot format template |
| `--prerelease <TAG>` | String | | Pre-release tag (`alpha`, `beta`, `rc`) |
| `--packages <LIST>` | CSV | All affected | Only bump specific packages |
| `--git-tag` | Flag | false | Create Git tags for releases |
| `--git-push` | Flag | false | Push Git tags to remote |
| `--git-commit` | Flag | false | Commit version changes |
| `--no-changelog` | Flag | false | Don't update changelogs |
| `--no-archive` | Flag | false | Don't archive changesets |
| `--force` | Flag | false | Skip confirmations |
| `--show-diff` | Flag | false | Show detailed version diffs |

### Examples

```bash
# Preview version changes (DEFAULT - safest, no modifications)
workspace bump                    # Default preview mode (recommended)
workspace bump --dry-run          # Explicit preview (same as above)

# Preview with JSON output (for CI/CD info)
workspace --format json bump      # Default is preview

# Show detailed diffs in preview
workspace bump --show-diff

# Execute version bump (requires explicit flag)
workspace bump --execute

# Execute with git operations
workspace bump --execute --git-tag --git-push

# Full release workflow
workspace bump --execute --git-commit --git-tag --git-push

# Generate snapshot versions for feature branch
workspace bump --snapshot
workspace bump --snapshot --execute  # Apply snapshots

# Create pre-release versions
workspace bump --prerelease beta --execute

# Bump specific packages only
workspace bump --packages "@org/core,@org/utils" --execute

# CI/CD workflow (silent, JSON output)
workspace --format json --log-level silent bump --execute --force
```

💡 **Tip**: `workspace bump` without flags is safe - it only previews changes without modifying any files.

### Output Example (--dry-run, Independent Strategy)

```
Version Bump Preview
━━━━━━━━━━━━━━━━━━━━
Strategy: Independent

Packages to bump (from changesets):
  @org/core: 1.2.3 → 1.3.0 (minor, direct change)
  @org/utils: 2.0.1 → 2.1.0 (minor, dependency propagation)

Packages unchanged:
  @org/cli: 0.5.0 (no changeset)
  @org/docs: 1.0.0 (no changeset)

Changesets to process:
  ✓ feature/new-api (minor, 5 commits, packages: @org/core)
  ✓ feature/fix-bug (patch, 2 commits, packages: @org/utils)

Git tags to create:
  @org/core@1.3.0
  @org/utils@2.1.0

Run with --execute to apply changes.
```

### Output Example (--dry-run, Unified Strategy)

```
Version Bump Preview
━━━━━━━━━━━━━━━━━━━━
Strategy: Unified

All packages will be bumped to: 1.3.0
  @org/core: 1.2.3 → 1.3.0
  @org/utils: 2.0.1 → 1.3.0
  @org/cli: 0.5.0 → 1.3.0
  @org/docs: 1.0.0 → 1.3.0

Changesets to process:
  ✓ feature/new-api (minor, 5 commits, packages: @org/core)
  ✓ feature/fix-bug (patch, 2 commits, packages: @org/utils)

Highest bump type: minor (determines unified version)

Git tags to create:
  @org/core@1.3.0
  @org/utils@1.3.0
  @org/cli@1.3.0
  @org/docs@1.3.0

Run with --execute to apply changes.
```

### Output Example (JSON Format)

```json
{
  "success": true,
  "mode": "dry-run",
  "strategy": "independent",
  "packages": [
    {
      "name": "@org/core",
      "path": "packages/core",
      "currentVersion": "1.2.3",
      "nextVersion": "1.3.0",
      "bump": "minor",
      "reason": "direct",
      "willBump": true
    },
    {
      "name": "@org/utils",
      "path": "packages/utils",
      "currentVersion": "2.0.1",
      "nextVersion": "2.1.0",
      "bump": "minor",
      "reason": "dependency_propagation",
      "willBump": true
    },
    {
      "name": "@org/cli",
      "path": "packages/cli",
      "currentVersion": "0.5.0",
      "nextVersion": "0.5.0",
      "bump": "none",
      "reason": "no_changes",
      "willBump": false
    }
  ],
  "changesets": [
    {
      "id": "cs_abc123",
      "branch": "feature/new-api",
      "bump": "minor",
      "commits": 5,
      "packages": ["@org/core"]
    },
    {
      "id": "cs_def456",
      "branch": "feature/fix-bug",
      "bump": "patch",
      "commits": 2,
      "packages": ["@org/utils"]
    }
  ],
  "tags": ["@org/core@1.3.0", "@org/utils@2.1.0"],
  "summary": {
    "totalPackages": 3,
    "packagesToBump": 2,
    "packagesUnchanged": 1,
    "totalChangesets": 2,
    "totalTags": 2
  }
}
```

---

## `upgrade` - Manage Dependency Upgrades

Detect and apply dependency upgrades.

### Subcommands

- `upgrade check` - Detect available upgrades
- `upgrade apply` - Apply upgrades
- `upgrade backups list` - List all backups
- `upgrade backups restore <ID>` - Restore a backup
- `upgrade backups clean` - Clean old backups

### Synopsis

```bash
workspace upgrade check [OPTIONS]
workspace upgrade apply [OPTIONS]
workspace upgrade backups list
workspace upgrade backups restore <ID> [OPTIONS]
workspace upgrade backups clean [OPTIONS]
```

---

### `upgrade check`

Detect available dependency upgrades.

#### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--major` | Flag | true | Include major version upgrades |
| `--no-major` | Flag | | Exclude major version upgrades |
| `--minor` | Flag | true | Include minor version upgrades |
| `--no-minor` | Flag | | Exclude minor version upgrades |
| `--patch` | Flag | true | Include patch version upgrades |
| `--no-patch` | Flag | | Exclude patch version upgrades |
| `--dev` | Flag | true | Include dev dependencies |
| `--peer` | Flag | false | Include peer dependencies |
| `--packages <LIST>` | CSV | | Only check specific packages |
| `--registry <URL>` | URL | From config | Override registry URL |

#### Examples

```bash
# Check for all upgrades
workspace upgrade check

# Check with JSON output
workspace --format json upgrade check

# Check only patch and minor upgrades
workspace upgrade check --no-major

# Check specific packages
workspace upgrade check --packages "typescript,eslint"

# Check including peer dependencies
workspace upgrade check --peer
```

#### Output Example (Human Format)

```
Dependency Upgrades Available
━━━━━━━━━━━━━━━━━━━━━━━━━━━━

@org/core:
  ┌──────────────────┬─────────┬─────────┬────────┬──────────┐
  │ Package          │ Current │ Latest  │ Type   │ Breaking │
  ├──────────────────┼─────────┼─────────┼────────┼──────────┤
  │ typescript       │ 5.0.0   │ 5.3.3   │ minor  │ No       │
  │ eslint           │ 8.0.0   │ 9.0.0   │ major  │ Yes      │
  │ vitest           │ 1.0.0   │ 1.2.1   │ minor  │ No       │
  └──────────────────┴─────────┴─────────┴────────┴──────────┘

@org/utils:
  ┌──────────────────┬─────────┬─────────┬────────┬──────────┐
  │ Package          │ Current │ Latest  │ Type   │ Breaking │
  ├──────────────────┼─────────┼─────────┼────────┼──────────┤
  │ lodash           │ 4.17.20 │ 4.17.21 │ patch  │ No       │
  │ axios            │ 1.5.0   │ 1.6.2   │ minor  │ No       │
  └──────────────────┴─────────┴─────────┴────────┴──────────┘

Summary:
  Total upgrades: 5
  Major: 1
  Minor: 3
  Patch: 1
  Breaking: 1

Recommended:
  Apply patch and minor upgrades: workspace upgrade apply --minor-and-patch
  Review major upgrades: workspace upgrade apply --dry-run
```

#### Output Example (JSON Format)

```json
{
  "success": true,
  "packages": [
    {
      "name": "@org/core",
      "path": "packages/core",
      "upgrades": [
        {
          "package": "typescript",
          "currentVersion": "5.0.0",
          "latestVersion": "5.3.3",
          "type": "minor",
          "breaking": false
        },
        {
          "package": "eslint",
          "currentVersion": "8.0.0",
          "latestVersion": "9.0.0",
          "type": "major",
          "breaking": true
        },
        {
          "package": "vitest",
          "currentVersion": "1.0.0",
          "latestVersion": "1.2.1",
          "type": "minor",
          "breaking": false
        }
      ]
    }
  ],
  "summary": {
    "totalUpgrades": 5,
    "major": 1,
    "minor": 3,
    "patch": 1,
    "breaking": 1
  }
}
```

---

### `upgrade apply`

Apply dependency upgrades.

#### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--dry-run` | Flag | false | Preview without applying |
| `--patch-only` | Flag | false | Only apply patch upgrades |
| `--minor-and-patch` | Flag | false | Only minor and patch upgrades |
| `--packages <LIST>` | CSV | All detected | Only upgrade specific packages |
| `--auto-changeset` | Flag | false | Automatically create changeset |
| `--changeset-bump <TYPE>` | String | `patch` | Changeset bump type |
| `--no-backup` | Flag | false | Skip backup creation |
| `--force` | Flag | false | Skip confirmations |

#### Examples

```bash
# Preview all upgrades
workspace upgrade apply --dry-run

# Apply all patch upgrades
workspace upgrade apply --patch-only

# Apply minor and patch upgrades with auto-changeset
workspace upgrade apply --minor-and-patch --auto-changeset

# Apply specific packages
workspace upgrade apply --packages "@types/node,typescript"

# Apply with JSON output
workspace --format json upgrade apply --patch-only

# CI/CD safe upgrade (no major versions)
workspace upgrade apply --minor-and-patch --force --auto-changeset
```

#### Output Example (Human Format)

```
Applying Upgrades
━━━━━━━━━━━━━━━━━

Backup created: backup_20251107_143045

Upgraded packages:
  ✓ @org/core: typescript 5.0.0 → 5.3.3 (minor)
  ✓ @org/core: vitest 1.0.0 → 1.2.1 (minor)
  ✓ @org/utils: lodash 4.17.20 → 4.17.21 (patch)
  ✓ @org/utils: axios 1.5.0 → 1.6.2 (minor)

Skipped packages (major versions):
  • eslint 8.0.0 → 9.0.0 (major, breaking)

Summary:
  Applied: 4 upgrades
  Skipped: 1 upgrade
  
Changeset created: cs_upgrade_20251107

Next steps:
  1. Review changes in package.json files
  2. Run tests: npm test
  3. If issues occur, restore backup: workspace upgrade backups restore backup_20251107_143045
```

#### Output Example (JSON Format)

```json
{
  "success": true,
  "applied": [
    {
      "package": "typescript",
      "workspace": "@org/core",
      "from": "5.0.0",
      "to": "5.3.3",
      "type": "minor"
    },
    {
      "package": "vitest",
      "workspace": "@org/core",
      "from": "1.0.0",
      "to": "1.2.1",
      "type": "minor"
    },
    {
      "package": "lodash",
      "workspace": "@org/utils",
      "from": "4.17.20",
      "to": "4.17.21",
      "type": "patch"
    },
    {
      "package": "axios",
      "workspace": "@org/utils",
      "from": "1.5.0",
      "to": "1.6.2",
      "type": "minor"
    }
  ],
  "skipped": [
    {
      "package": "eslint",
      "workspace": "@org/core",
      "currentVersion": "8.0.0",
      "latestVersion": "9.0.0",
      "reason": "major_version",
      "breaking": true
    }
  ],
  "summary": {
    "totalApplied": 4,
    "totalSkipped": 1,
    "backupId": "backup_20251107_143045",
    "changesetId": "cs_upgrade_20251107"
  }
}
```

---

### `upgrade backups`

Manage upgrade backups.

#### Examples

```bash
# List all backups
workspace upgrade backups list

# Restore a backup
workspace upgrade backups restore backup_20251107_143045

# Force restore without confirmation
workspace upgrade backups restore backup_20251107_143045 --force

# Clean old backups (keep last 5)
workspace upgrade backups clean --keep 5

# Clean with force (no confirmation)
workspace upgrade backups clean --keep 3 --force
```

---

## `audit` - Run Project Health Audit

Analyze project health including upgrades, dependencies, version consistency, and breaking changes.

### Synopsis

```bash
workspace audit [OPTIONS]
```

### Description

Executes a comprehensive project health audit with customizable sections and verbosity levels. Calculates an overall health score and provides actionable recommendations.

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--sections <LIST>` | CSV | `all` | Sections to audit |
| `--output <PATH>` | Path | stdout | Write output to file |
| `--min-severity <LEVEL>` | String | `info` | Minimum severity level |
| `--verbosity <LEVEL>` | String | `normal` | Detail level |
| `--no-health-score` | Flag | false | Skip health score calculation |
| `--export <FORMAT>` | String | | Export format (`html`, `markdown`) |
| `--export-file <PATH>` | Path | | File path for exported report |

**Audit Sections:**
- `all` - All sections
- `upgrades` - Available dependency upgrades
- `dependencies` - Dependency health (circular, missing, deprecated)
- `version-consistency` - Version consistency across monorepo
- `breaking-changes` - Breaking changes detection

**Severity Levels:**
- `critical` - Critical issues only
- `high` - Critical and high issues
- `medium` - Medium and above
- `low` - Low and above
- `info` - All issues (default)

**Verbosity Levels:**
- `minimal` - Summary only
- `normal` - Standard detail (default)
- `detailed` - Full details with recommendations

### Examples

```bash
# Full audit
workspace audit

# Specific sections
workspace audit --sections upgrades,dependencies

# Generate markdown report
workspace audit --output audit-report.md

# JSON for CI/CD
workspace --format json audit

# JSON compact for CI/CD
workspace --format json-compact audit

# Only critical and high severity
workspace audit --min-severity high

# Detailed output
workspace audit --verbosity detailed

# Export as HTML
workspace audit --export html --export-file report.html

# CI/CD health check (JSON, silent logs)
workspace --format json --log-level silent audit --sections upgrades,dependencies
```

### Output Example (Human Format)

```
Project Health Audit
━━━━━━━━━━━━━━━━━━━━

Health Score: 78/100 (Good)
  Upgrades:           85/100
  Dependencies:       72/100
  Version Consistency: 80/100
  Breaking Changes:   75/100

Summary:
  Total Issues: 12
  Critical: 0
  High: 2
  Medium: 5
  Low: 3
  Info: 2

━━━━━━━━━━━━━━━━━━━━
Upgrades (Score: 85/100)
━━━━━━━━━━━━━━━━━━━━

Available Upgrades: 15 total
  Major: 3 (1 breaking)
  Minor: 8
  Patch: 4

Critical Upgrades:
  • None

Recommended Upgrades:
  ✓ Apply 12 non-breaking upgrades
  ⚠ Review 3 major upgrades manually

━━━━━━━━━━━━━━━━━━━━
Dependencies (Score: 72/100)
━━━━━━━━━━━━━━━━━━━━

Issues Found: 8 total

Circular Dependencies: 2
  ⚠ @org/core → @org/utils → @org/core
  ⚠ @org/cli → @org/api → @org/cli

Deprecated Packages: 3
  ⚠ request@2.88.2 (use axios or node-fetch)
  ⚠ babel-eslint@10.1.0 (use @babel/eslint-parser)
  ⚠ mkdirp@0.5.5 (native fs.mkdir is sufficient)

Missing Dependencies: 1
  ⚠ @types/node referenced but not in package.json (@org/cli)

Phantom Dependencies: 2
  ℹ lodash used but not declared (@org/utils)
  ℹ chalk used but not declared (@org/cli)

━━━━━━━━━━━━━━━━━━━━
Version Consistency (Score: 80/100)
━━━━━━━━━━━━━━━━━━━━

Inconsistencies Found: 4

Different Versions:
  ⚠ typescript: 3 versions (5.0.0, 5.2.2, 5.3.3)
  ⚠ eslint: 2 versions (8.0.0, 8.54.0)

Recommendations:
  • Align typescript to 5.3.3 (latest)
  • Align eslint to 8.54.0 (latest)

━━━━━━━━━━━━━━━━━━━━
Breaking Changes (Score: 75/100)
━━━━━━━━━━━━━━━━━━━━

Potential Breaking Changes: 3

From Commits:
  ⚠ feat!: remove deprecated API endpoints (@org/core)
  ⚠ chore!: drop Node 14 support (@org/utils)

From Dependencies:
  ℹ eslint 8.0.0 → 9.0.0 (breaking)

━━━━━━━━━━━━━━━━━━━━
Recommendations
━━━━━━━━━━━━━━━━━━━━

High Priority:
  1. Fix circular dependencies in @org/core and @org/cli
  2. Replace deprecated packages (request, babel-eslint, mkdirp)
  3. Align typescript versions across workspace

Medium Priority:
  4. Add missing @types/node to @org/cli
  5. Declare phantom dependencies (lodash, chalk)
  6. Apply 12 non-breaking dependency upgrades

Low Priority:
  7. Review major version upgrades
  8. Document breaking changes in CHANGELOG.md
```

### Output Example (JSON Format)

```json
{
  "success": true,
  "healthScore": 78,
  "scores": {
    "upgrades": 85,
    "dependencies": 72,
    "versionConsistency": 80,
    "breakingChanges": 75
  },
  "summary": {
    "totalIssues": 12,
    "critical": 0,
    "high": 2,
    "medium": 5,
    "low": 3,
    "info": 2
  },
  "sections": {
    "upgrades": {
      "score": 85,
      "availableUpgrades": 15,
      "major": 3,
      "minor": 8,
      "patch": 4,
      "breaking": 1
    },
    "dependencies": {
      "score": 72,
      "issues": [
        {
          "type": "circular",
          "severity": "high",
          "cycle": ["@org/core", "@org/utils", "@org/core"]
        },
        {
          "type": "deprecated",
          "severity": "medium",
          "package": "request",
          "version": "2.88.2",
          "alternative": "axios or node-fetch"
        }
      ]
    },
    "versionConsistency": {
      "score": 80,
      "inconsistencies": [
        {
          "package": "typescript",
          "versions": ["5.0.0", "5.2.2", "5.3.3"],
          "recommended": "5.3.3"
        }
      ]
    },
    "breakingChanges": {
      "score": 75,
      "changes": [
        {
          "type": "commit",
          "package": "@org/core",
          "message": "feat!: remove deprecated API endpoints"
        }
      ]
    }
  },
  "recommendations": [
    {
      "priority": "high",
      "action": "Fix circular dependencies in @org/core and @org/cli"
    },
    {
      "priority": "high",
      "action": "Replace deprecated packages"
    }
  ]
}
```

---

## `changes` - Analyze Changes

Analyze changes in repository to detect affected packages.

### Synopsis

```bash
workspace changes [OPTIONS]
```

### Description

Detects which packages are affected by changes in the working directory or between commits. Useful for CI/CD pipelines to determine which packages need testing or deployment.

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--since <REF>` | String | | Since commit/branch/tag |
| `--until <REF>` | String | HEAD | Until commit/branch/tag |
| `--branch <NAME>` | String | | Compare against branch |
| `--staged` | Flag | false | Only staged changes |
| `--unstaged` | Flag | false | Only unstaged changes |
| `--packages <LIST>` | CSV | | Filter by packages |

### Examples

```bash
# Analyze working directory changes
workspace changes

# Analyze changes since HEAD~1
workspace changes --since HEAD~1

# Analyze changes between branches
workspace changes --branch main

# Only staged changes
workspace changes --staged

# Specific commit range
workspace changes --since abc123 --until def456

# Filter by packages
workspace changes --packages "@org/core,@org/cli"

# JSON output for CI/CD
workspace --format json changes --since main
```

### Output Example (Human Format)

```
Repository Changes
━━━━━━━━━━━━━━━━━━

Analyzing: Working directory
Reference: main

Affected Packages (2):

@org/core (packages/core/)
  Modified files: 5
    • src/api/routes.ts
    • src/api/handlers.ts
    • src/types/index.ts
    • tests/api.test.ts
    • package.json
  
  Lines changed: +127 -45

@org/utils (packages/utils/)
  Modified files: 2
    • src/string-utils.ts
    • tests/string-utils.test.ts
  
  Lines changed: +38 -12

Unaffected Packages (2):
  • @org/cli (no changes)
  • @org/docs (no changes)

Summary:
  Total files changed: 7
  Affected packages: 2 / 4
  Lines added: 165
  Lines removed: 57
```

### Output Example (JSON Format)

```json
{
  "success": true,
  "analysis": {
    "type": "working-directory",
    "reference": "main"
  },
  "affectedPackages": [
    {
      "name": "@org/core",
      "path": "packages/core",
      "filesChanged": 5,
      "linesAdded": 127,
      "linesRemoved": 45,
      "files": [
        "src/api/routes.ts",
        "src/api/handlers.ts",
        "src/types/index.ts",
        "tests/api.test.ts",
        "package.json"
      ]
    },
    {
      "name": "@org/utils",
      "path": "packages/utils",
      "filesChanged": 2,
      "linesAdded": 38,
      "linesRemoved": 12,
      "files": [
        "src/string-utils.ts",
        "tests/string-utils.test.ts"
      ]
    }
  ],
  "unaffectedPackages": [
    "@org/cli",
    "@org/docs"
  ],
  "summary": {
    "totalFilesChanged": 7,
    "affectedPackages": 2,
    "unaffectedPackages": 2,
    "totalPackages": 4,
    "linesAdded": 165,
    "linesRemoved": 57
  }
}
```

---

## `version` - Display Version Information

Show CLI version and optionally detailed build information.

### Synopsis

```bash
workspace version [OPTIONS]
```

### Description

Displays the version of the workspace CLI tool. With the `--verbose` flag, shows additional build information including Rust version and dependencies.

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--verbose` | Flag | false | Show detailed version information |

### Examples

```bash
# Show version
workspace version

# Show detailed version information
workspace version --verbose

# JSON output
workspace --format json version
```

### Output Example (Human Format)

```
workspace 0.1.0
```

### Output Example (Human Format, --verbose)

```
workspace 0.1.0

Build Information:
  Rust: 1.75.0
  Target: aarch64-apple-darwin
  Profile: release
  Features: default

Dependencies:
  clap: 4.4.11
  tokio: 1.35.1
  serde: 1.0.193
  sublime-package-tools: 0.1.0
  sublime-git-tools: 0.1.0
  sublime-standard-tools: 0.1.0
```

### Output Example (JSON Format)

```json
{
  "success": true,
  "version": "0.1.0",
  "buildInfo": {
    "rustVersion": "1.75.0",
    "target": "aarch64-apple-darwin",
    "profile": "release",
    "features": ["default"]
  }
}
```

---

## Quick Reference

### Common Command Patterns

```bash
# Project Setup
workspace init                              # Initialize configuration
workspace config show                       # View configuration
workspace config validate                   # Validate configuration

# Daily Workflow
git checkout -b feature/new-feature         # Create feature branch
workspace changeset create                  # Create changeset
# ... make changes ...
git commit -m "feat: add feature"           # Commit changes
workspace changeset update                  # Update changeset
workspace bump --dry-run                    # Preview versions

# Release Workflow
workspace bump --execute                    # Apply version bumps
workspace bump --execute --git-tag --git-push  # Full release

# Maintenance
workspace upgrade check                     # Check for upgrades
workspace upgrade apply --minor-and-patch   # Apply safe upgrades
workspace audit                             # Run health audit

# CI/CD
workspace --format json --log-level silent bump --dry-run  # Get version info
workspace --format json --log-level silent changes --since main  # Detect changes
workspace --format json --log-level silent audit --sections upgrades  # Health check
```

### Flags Cheat Sheet

| Task | Command |
|------|---------|
| Preview without changes | `--dry-run` |
| Execute changes | `--execute` |
| JSON output | `--format json` |
| No logs | `--log-level silent` |
| Debug logs | `--log-level debug` |
| Skip prompts | `--force` or `--non-interactive` |
| Create git tags | `--git-tag` |
| Push to remote | `--git-push` |

---

## Common Patterns

### Pattern 1: Feature Development

```bash
# 1. Create feature branch
git checkout -b feature/new-api

# 2. Create changeset
workspace changeset create \
  --bump minor \
  --env production

# 3. Make changes and commit multiple times
# ... develop ...
git add .
git commit -m "feat: add new endpoint"
workspace changeset update

# ... more development ...
git add .
git commit -m "test: add tests"
workspace changeset update

# 4. Preview versions before merge
workspace bump --dry-run

# 5. Merge to main
git checkout main
git merge feature/new-api

# 6. Release
workspace bump --execute --git-tag --git-push
```

### Pattern 2: Hotfix

```bash
# 1. Create hotfix branch
git checkout -b hotfix/security-fix

# 2. Create patch changeset
workspace changeset create \
  --bump patch \
  --env production \
  --non-interactive

# 3. Fix and commit
# ... fix issue ...
git commit -m "fix: security vulnerability"
workspace changeset update

# 4. Quick release
workspace bump --execute --git-tag --git-push --force
```

### Pattern 3: CI/CD Integration

```bash
# GitHub Actions example

# Step 1: Detect changes
CHANGES=$(workspace --format json --log-level silent changes --since origin/main)

# Step 2: Run tests only for affected packages
AFFECTED=$(echo $CHANGES | jq -r '.affectedPackages[].name')

# Step 3: Get version info
VERSION_INFO=$(workspace --format json --log-level silent bump --dry-run)

# Step 4: Release on main branch
if [ "$BRANCH" = "main" ]; then
  workspace bump --execute --git-tag --git-push --force
fi
```

### Pattern 4: Dependency Maintenance

```bash
# 1. Check for upgrades
workspace upgrade check

# 2. Apply safe upgrades with changeset
workspace upgrade apply \
  --minor-and-patch \
  --auto-changeset

# 3. Run tests
npm test

# 4. If tests fail, rollback
workspace upgrade backups restore backup_YYYYMMDD_HHMMSS

# 5. If tests pass, release
workspace bump --execute --git-tag
```

### Pattern 5: Health Monitoring

```bash
# Weekly audit report
workspace audit \
  --output weekly-audit-$(date +%Y-%m-%d).md \
  --verbosity detailed

# CI/CD health check (fail if score < 70)
HEALTH=$(workspace --format json audit | jq '.healthScore')
if [ "$HEALTH" -lt 70 ]; then
  echo "Health score too low: $HEALTH"
  exit 1
fi
```

### Pattern 6: Monorepo with Independent Versions

```bash
# Only bump packages with changesets
workspace bump --execute

# Bump specific package only
workspace bump \
  --packages "@org/core" \
  --execute

# Preview what will be bumped
workspace bump --dry-run --show-diff
```

### Pattern 7: Snapshot Versions for Testing

```bash
# Create snapshot for feature branch
workspace bump --snapshot --execute

# Result: @org/core@1.2.3-feature-new-api.abc123

# Publish to test registry
npm publish --tag next --registry https://test-registry.com
```

---

## Exit Codes

The `workspace` CLI uses standard exit codes:

| Code | Meaning | Description |
|------|---------|-------------|
| 0 | Success | Command completed successfully |
| 1 | General error | Command failed with error |
| 2 | Validation error | Invalid arguments or configuration |
| 3 | Not found | Resource not found (file, changeset, etc.) |
| 64 | Usage error | Incorrect command usage |
| 65 | Data error | Invalid input data |
| 66 | No input | Cannot open input file |
| 70 | Internal error | Internal software error |
| 74 | IO error | Input/output error |
| 77 | Permission error | Permission denied |
| 78 | Config error | Configuration file error |

### Examples

```bash
# Success
workspace bump --dry-run
echo $?  # 0

# Validation error
workspace init --strategy invalid
echo $?  # 2

# Not found
workspace changeset show nonexistent-branch
echo $?  # 3

# Usage error
workspace bump  # Missing --dry-run or --execute
echo $?  # 64
```

---

## Additional Resources

- **User Guide**: See [GUIDE.md](./GUIDE.md) for comprehensive tutorials and concepts
- **API Documentation**: See Rust docs for internal API reference
- **GitHub**: <https://github.com/websublime/workspace-tools>
- **Issues**: <https://github.com/websublime/workspace-tools/issues>

---

**Last Updated:** 2025-11-07  
**Version:** 0.1.0
