# Workspace Tools CLI

[![Pull Request](https://github.com/websublime/workspace-tools/workflows/Pull%20Request/badge.svg)](https://github.com/websublime/workspace-tools/actions)
[![Crates.io](https://img.shields.io/crates/v/sublime_cli_tools.svg)](https://crates.io/crates/sublime_cli_tools)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A comprehensive command-line interface for managing Node.js workspaces and monorepos with changeset-based version management.

---

## Overview

`workspace` (Workspace Tools CLI) provides:

- **Configuration Management**: Initialize and validate workspace configurations
- **Changeset Workflow**: Create, update, list, show, edit, and remove changesets
- **Version Management**: Intelligent version bumping with preview mode and multiple strategies
- **Dependency Upgrades**: Detect, apply, and rollback dependency updates
- **Audit System**: Comprehensive health checks with actionable insights
- **Change Analysis**: Detect affected packages from Git changes
- **CI/CD Integration**: JSON output modes and silent operation for automation

---

## Installation & Quick Start

For installation instructions and quick start guide, see the [main README](../../README.md#-quick-start).

---

## Documentation

This README provides complete CLI documentation based on the actual source code.

### Available Commands

```bash
workspace init                        # Initialize project configuration
workspace config <subcommand>         # Manage configuration
workspace changeset <subcommand>      # Manage changesets
workspace bump [options]              # Bump package versions
workspace upgrade <subcommand>        # Manage dependency upgrades
workspace audit [options]             # Run project health audit
workspace changes [options]           # Analyze repository changes
workspace version [options]           # Display version information
```

---

## Commands Reference

### `init` - Initialize Project Configuration

Creates a new configuration file for changeset-based version management.

**Usage:**
```bash
workspace init [OPTIONS]
```

**Options:**
- `--changeset-path <PATH>` - Changeset directory path (default: `.changesets`)
- `--environments <LIST>` - Comma-separated list of environments (e.g., `dev,staging,prod`)
- `--default-env <LIST>` - Comma-separated list of default environments
- `--strategy <STRATEGY>` - Versioning strategy (`independent` or `unified`)
- `--registry <URL>` - NPM registry URL (default: `https://registry.npmjs.org`)
- `--config-format <FORMAT>` - Configuration file format (`json`, `toml`, or `yaml`)
- `--force` - Overwrite existing configuration
- `--non-interactive` - Use default values without prompting

**Examples:**
```bash
# Interactive initialization
workspace init

# Non-interactive with specific options
workspace init --non-interactive --strategy independent --config-format yaml

# Force overwrite existing config
workspace init --force --environments "dev,staging,prod"
```

---

### `config` - Manage Configuration

View and validate project configuration.

#### `config show` - Display Current Configuration

Shows all configuration values from the detected config file.

**Usage:**
```bash
workspace config show
```

**Examples:**
```bash
# Show configuration (human-readable)
workspace config show

# Show configuration as JSON
workspace --format json config show
```

#### `config validate` - Validate Configuration File

Checks that the configuration file is valid and all required fields are present.

**Usage:**
```bash
workspace config validate
```

**Examples:**
```bash
# Validate configuration
workspace config validate

# Validate with detailed logging
workspace --log-level debug config validate
```

---

### `changeset` - Manage Changesets

Create, update, list, and manage changesets for version control.

#### `changeset create` - Create New Changeset

Creates a changeset for the current branch.

**Usage:**
```bash
workspace changeset create [OPTIONS]
```

**Options:**
- `--bump <TYPE>` - Bump type (`major`, `minor`, or `patch`)
- `--env <LIST>` - Comma-separated list of environments
- `--branch <NAME>` - Branch name (defaults to current Git branch)
- `--message <TEXT>` - Optional description of changes
- `--packages <LIST>` - Comma-separated list of packages (auto-detected if not provided)
- `--non-interactive` - Use provided flags without prompting

**Examples:**
```bash
# Interactive creation
workspace changeset create

# Non-interactive with specific options
workspace changeset create --bump minor --env "staging,prod" --non-interactive

# Create with message
workspace changeset create --bump patch --message "Fix critical bug"
```

#### `changeset update` - Update Existing Changeset

Adds commits or packages to an existing changeset.

**Usage:**
```bash
workspace changeset update [ID] [OPTIONS]
```

**Arguments:**
- `<ID>` - Changeset ID or branch name (defaults to current branch)

**Options:**
- `--commit <HASH>` - Add specific commit hash
- `--packages <LIST>` - Comma-separated list of packages to add
- `--bump <TYPE>` - Update bump type (`major`, `minor`, or `patch`)
- `--env <LIST>` - Comma-separated list of environments to add

**Examples:**
```bash
# Update changeset for current branch
workspace changeset update

# Update specific changeset
workspace changeset update feature/new-api --bump major

# Add specific commit
workspace changeset update --commit abc123def
```

#### `changeset list` - List All Changesets

Shows all active changesets with optional filtering and sorting.

**Usage:**
```bash
workspace changeset list [OPTIONS]
```

**Options:**
- `--filter-package <NAME>` - Filter by package name
- `--filter-bump <TYPE>` - Filter by bump type (`major`, `minor`, or `patch`)
- `--filter-env <ENV>` - Filter by environment
- `--sort <FIELD>` - Sort by field (`date`, `bump`, or `branch`; default: `date`)

**Examples:**
```bash
# List all changesets
workspace changeset list

# Filter by package
workspace changeset list --filter-package "@myorg/core"

# Filter and sort
workspace changeset list --filter-bump major --sort bump
```

#### `changeset show` - Show Changeset Details

Displays detailed information about a specific changeset.

**Usage:**
```bash
workspace changeset show <BRANCH>
```

**Arguments:**
- `<BRANCH>` - Branch name or changeset ID

**Examples:**
```bash
# Show changeset for branch
workspace changeset show feature/new-api

# Show as JSON
workspace --format json changeset show feature/new-api
```

#### `changeset edit` - Edit Changeset

Opens the changeset file in your editor ($EDITOR).

**Usage:**
```bash
workspace changeset edit [BRANCH]
```

**Arguments:**
- `<BRANCH>` - Branch name (defaults to current Git branch)

**Examples:**
```bash
# Edit changeset for current branch
workspace changeset edit

# Edit changeset for specific branch
workspace changeset edit feature/new-api
```

#### `changeset delete` - Delete Changeset

Removes a changeset from active changesets.

**Usage:**
```bash
workspace changeset delete <BRANCH> [OPTIONS]
```

**Arguments:**
- `<BRANCH>` - Branch name

**Options:**
- `--force` - Skip confirmation prompt

**Examples:**
```bash
# Delete with confirmation
workspace changeset delete feature/old-branch

# Delete without confirmation
workspace changeset delete feature/old-branch --force
```

#### `changeset history` - Query Changeset History

Searches archived changesets with filtering options.

**Usage:**
```bash
workspace changeset history [OPTIONS]
```

**Options:**
- `--package <NAME>` - Filter by package name
- `--since <DATE>` - Since date (ISO 8601, e.g., `2024-01-01`)
- `--until <DATE>` - Until date (ISO 8601)
- `--env <ENV>` - Filter by environment
- `--bump <TYPE>` - Filter by bump type
- `--limit <N>` - Limit number of results

**Examples:**
```bash
# Show all history
workspace changeset history

# Filter by package and date range
workspace changeset history --package "@myorg/core" --since "2024-01-01"

# Limit results
workspace changeset history --limit 10
```

#### `changeset check` - Check if Changeset Exists

Checks if a changeset exists for the current or specified branch. Useful for Git hooks.

**Usage:**
```bash
workspace changeset check [OPTIONS]
```

**Options:**
- `--branch <NAME>` - Branch name (defaults to current Git branch)

**Examples:**
```bash
# Check current branch
workspace changeset check

# Check specific branch
workspace changeset check --branch feature/new-api
```

---

### `bump` - Bump Package Versions

Calculates and applies version bumps according to active changesets and the configured versioning strategy.

**Usage:**
```bash
workspace bump [OPTIONS]
```

**Options:**
- `--dry-run` - Preview changes without applying (safe, default behavior)
- `--execute` - Apply version changes (required for actual changes)
- `--snapshot` - Generate snapshot versions
- `--snapshot-format <FORMAT>` - Snapshot format template (variables: `{version}`, `{branch}`, `{short_commit}`, `{commit}`)
- `--prerelease <TAG>` - Pre-release tag (`alpha`, `beta`, or `rc`)
- `--packages <LIST>` - Comma-separated list of packages to bump (overrides changeset packages)
- `--git-tag` - Create Git tags for releases (format: `package@version`)
- `--git-push` - Push Git tags to remote (requires `--git-tag`)
- `--git-commit` - Commit version changes
- `--no-changelog` - Skip changelog generation/updates
- `--no-archive` - Keep changesets active after bump
- `--force` - Skip confirmations
- `--show-diff` - Show detailed version diffs (preview mode only)

**Examples:**
```bash
# Preview version bumps (safe, no changes)
workspace bump --dry-run

# Apply version bumps
workspace bump --execute

# Full release workflow
workspace bump --execute --git-commit --git-tag --git-push

# Snapshot version for testing
workspace bump --snapshot --execute

# Pre-release version
workspace bump --prerelease beta --execute

# Show detailed diffs in preview
workspace bump --dry-run --show-diff
```

---

### `upgrade` - Manage Dependency Upgrades

Check for available upgrades and apply them to workspace packages.

#### `upgrade check` - Check for Available Upgrades

Detects outdated dependencies in workspace packages.

**Usage:**
```bash
workspace upgrade check [OPTIONS]
```

**Options:**
- `--major` / `--no-major` - Include/exclude major version upgrades (default: include)
- `--minor` / `--no-minor` - Include/exclude minor version upgrades (default: include)
- `--patch` / `--no-patch` - Include/exclude patch version upgrades (default: include)
- `--dev` - Include dev dependencies (default: true)
- `--peer` - Include peer dependencies (default: false)
- `--packages <LIST>` - Comma-separated list of packages to check
- `--registry <URL>` - Override registry URL

**Examples:**
```bash
# Check all upgrades
workspace upgrade check

# Check only non-breaking upgrades
workspace upgrade check --no-major

# Check specific packages
workspace upgrade check --packages "@myorg/core,@myorg/utils"

# Include peer dependencies
workspace upgrade check --peer
```

#### `upgrade apply` - Apply Dependency Upgrades

Updates dependencies to newer versions.

**Usage:**
```bash
workspace upgrade apply [OPTIONS]
```

**Options:**
- `--dry-run` - Preview without applying
- `--patch-only` - Only apply patch upgrades
- `--minor-and-patch` - Only apply minor and patch upgrades (non-breaking)
- `--packages <LIST>` - Comma-separated list of packages to upgrade
- `--auto-changeset` - Automatically create changeset for upgrades
- `--changeset-bump <TYPE>` - Changeset bump type (`major`, `minor`, or `patch`; default: `patch`)
- `--no-backup` - Skip backup creation
- `--force` - Skip confirmations

**Examples:**
```bash
# Preview upgrades
workspace upgrade apply --dry-run

# Apply safe upgrades with automatic changeset
workspace upgrade apply --minor-and-patch --auto-changeset

# Apply patch upgrades only
workspace upgrade apply --patch-only --force

# Apply upgrades for specific packages
workspace upgrade apply --packages "@myorg/core"
```

#### `upgrade backups` - Manage Upgrade Backups

List, restore, or clean upgrade backups.

**`upgrade backups list`** - List all backups:
```bash
workspace upgrade backups list
```

**`upgrade backups restore`** - Restore a backup:
```bash
workspace upgrade backups restore <ID> [OPTIONS]
```
Options:
- `--force` - Skip confirmation prompt

Example:
```bash
workspace upgrade backups restore backup_20240115_103045
```

**`upgrade backups clean`** - Clean old backups:
```bash
workspace upgrade backups clean [OPTIONS]
```
Options:
- `--keep <N>` - Number of recent backups to keep (default: 5)
- `--force` - Skip confirmation prompt

Example:
```bash
workspace upgrade backups clean --keep 3 --force
```

---

### `audit` - Run Project Health Audit

Analyzes project health including upgrades, dependencies, version consistency, and breaking changes.

**Usage:**
```bash
workspace audit [OPTIONS]
```

**Options:**
- `--sections <LIST>` - Comma-separated list of sections to audit (default: `all`)
  - Options: `all`, `upgrades`, `dependencies`, `version-consistency`, `breaking-changes`
- `--output <PATH>` - Write output to file
- `--min-severity <LEVEL>` - Minimum severity level (default: `info`)
  - Options: `critical`, `high`, `medium`, `low`, `info`
- `--verbosity <LEVEL>` - Detail level (default: `normal`)
  - Options: `minimal`, `normal`, `detailed`
- `--no-health-score` - Skip health score calculation
- `--export <FORMAT>` - Export format (`html` or `markdown`; requires `--export-file`)
- `--export-file <PATH>` - File path for exported report (requires `--export`)

**Examples:**
```bash
# Full audit
workspace audit

# Specific sections
workspace audit --sections upgrades,dependencies

# High severity issues only
workspace audit --min-severity high

# Export to markdown
workspace audit --export markdown --export-file audit-report.md

# JSON output for CI/CD
workspace --format json audit
```

---

### `changes` - Analyze Repository Changes

Detects which packages are affected by changes in the working directory or between commits.

**Usage:**
```bash
workspace changes [OPTIONS]
```

**Options:**
- `--since <REF>` - Since commit/branch/tag (analyzes changes since this Git reference)
- `--until <REF>` - Until commit/branch/tag (default: `HEAD`)
- `--branch <NAME>` - Compare against branch
- `--staged` - Only staged changes (cannot be used with `--unstaged`)
- `--unstaged` - Only unstaged changes (cannot be used with `--staged`)
- `--packages <LIST>` - Comma-separated list of packages to filter

**Examples:**
```bash
# Analyze working directory changes
workspace changes

# Changes since specific commit
workspace changes --since HEAD~5

# Changes between commits
workspace changes --since v1.0.0 --until v1.1.0

# Only staged changes
workspace changes --staged

# Compare against branch
workspace changes --branch main

# Filter specific packages
workspace changes --packages "@myorg/core"
```

---

### `version` - Display Version Information

Shows the CLI version and optionally detailed build information.

**Usage:**
```bash
workspace version [OPTIONS]
```

**Options:**
- `--verbose` - Show detailed version information (includes Rust version, dependencies, and build info)

**Examples:**
```bash
# Show version
workspace version

# Show detailed information
workspace version --verbose
```

---

## Global Options

All commands support these global options that control behavior across the entire application:

**Options:**
- `-r, --root <PATH>` - Project root directory (default: current directory)
  - Changes working directory before executing the command
  - All file operations will be relative to this path
  
- `-l, --log-level <LEVEL>` - Logging level (default: `info`)
  - Controls verbosity of operation logs written to **stderr**
  - Does NOT affect command output (stdout)
  - Levels: `silent`, `error`, `warn`, `info`, `debug`, `trace`
  
- `-f, --format <FORMAT>` - Output format (default: `human`)
  - Controls format of command output written to **stdout**
  - Does NOT affect logging (stderr)
  - Formats: `human`, `json`, `json-compact`, `quiet`
  
- `--no-color` - Disable colored output
  - Removes ANSI color codes from both logs (stderr) and output (stdout)
  - Also respects the `NO_COLOR` environment variable
  - Useful for CI/CD environments and file redirection
  
- `-c, --config <PATH>` - Path to config file
  - Override default config file location
  - Path can be relative or absolute
  - Default: Auto-detect (`.changesets.{toml,json,yaml,yml}`)

**Stream Separation:**

The CLI maintains strict separation between:
- **stderr**: Logs only (controlled by `--log-level`)
- **stdout**: Command output only (controlled by `--format`)

This ensures JSON output is never contaminated with logs, enabling reliable piping and parsing in scripts.

**Examples:**
```bash
# Clean JSON output with no logs (perfect for automation)
workspace --format json --log-level silent bump --dry-run

# JSON output with debug logs (logs to stderr, JSON to stdout)
workspace --format json --log-level debug bump --dry-run > output.json 2> debug.log

# Change working directory
workspace --root /path/to/project changeset list

# Use custom config file
workspace --config custom-config.toml config show
```

---

## Configuration

The CLI uses a configuration file (`repo.config.toml`, `repo.config.json`, or `repo.config.yaml`) to control all aspects of workspace management. This file is typically created by running `workspace init`.

### Configuration File Location

The CLI automatically detects configuration files in this order:
1. Path specified via `--config` flag
2. `repo.config.toml` in project root
3. `repo.config.yaml` or `repo.config.yml` in project root
4. `repo.config.json` in project root

### Configuration Structure

The configuration is organized into logical sections under the `[package_tools]` namespace:

```toml
[package_tools.changeset]
# Changeset storage and management

[package_tools.version]
# Versioning strategy

[package_tools.dependency]
# Dependency propagation

[package_tools.upgrade]
# Dependency upgrade detection

[package_tools.changelog]
# Changelog generation

[package_tools.git]
# Git integration

[package_tools.audit]
# Health checks and audits

[package_tools.workspace]
# Monorepo workspace patterns (optional)
```

---

### Complete Example Configuration

```toml
[package_tools.changeset]
path = ".changesets"
history_path = ".changesets/history"
available_environments = ["development", "staging", "production"]
default_environments = ["production"]

[package_tools.version]
strategy = "independent"  # or "unified"
default_bump = "patch"
snapshot_format = "{version}-{branch}.{timestamp}"

[package_tools.dependency]
propagation_bump = "patch"
propagate_dependencies = true
propagate_dev_dependencies = false
propagate_peer_dependencies = true
max_depth = 10
fail_on_circular = true
skip_workspace_protocol = true
skip_file_protocol = true
skip_link_protocol = true
skip_portal_protocol = true

[package_tools.upgrade]
auto_changeset = true
changeset_bump = "patch"

[package_tools.upgrade.registry]
default_registry = "https://registry.npmjs.org"
scoped_registries = {}
auth_tokens = {}
timeout_secs = 30
retry_attempts = 3
retry_delay_ms = 1000
read_npmrc = true

[package_tools.upgrade.backup]
enabled = true
backup_dir = ".workspace-backups"
keep_after_success = false
max_backups = 5

[package_tools.changelog]
enabled = true
format = "keep-a-changelog"  # or "conventional" or "custom"
filename = "CHANGELOG.md"
include_commit_links = true
include_issue_links = true
include_authors = false
repository_url = "https://github.com/your-org/your-repo"
monorepo_mode = "per-package"  # or "root" or "both"
version_tag_format = "{name}@{version}"
root_tag_format = "v{version}"

[package_tools.changelog.conventional]
enabled = true
breaking_section = "Breaking Changes"

[package_tools.changelog.conventional.types]
feat = "Features"
fix = "Bug Fixes"
perf = "Performance Improvements"
refactor = "Code Refactoring"
docs = "Documentation"
build = "Build System"
ci = "Continuous Integration"
test = "Tests"
chore = "Chores"

[package_tools.changelog.exclude]
patterns = []
authors = []

[package_tools.changelog.template]
header = "# Changelog\n\nAll notable changes to this project will be documented in this file.\n\n"
version_header = "## [{version}] - {date}"
section_header = "### {section}"
entry_format = "- {description} ({hash})"

[package_tools.git]
merge_commit_template = "chore(release): {version}\n\nRelease version {version}\n\n{changelog_summary}"
monorepo_merge_commit_template = "chore(release): {package_name}@{version}\n\nRelease {package_name} version {version}\n\n{changelog_summary}"
include_breaking_warning = true
breaking_warning_template = "\n⚠️  BREAKING CHANGES: {breaking_changes_count}\n"

[package_tools.audit]
enabled = true
min_severity = "warning"  # "critical", "warning", or "info"

[package_tools.audit.sections]
upgrades = true
dependencies = true
breaking_changes = true
categorization = true
version_consistency = true

[package_tools.audit.upgrades]
include_patch = true
include_minor = true
include_major = true
deprecated_as_critical = true

[package_tools.audit.dependencies]
check_circular = true
check_missing = false
check_unused = false
check_version_conflicts = true

[package_tools.audit.breaking_changes]
check_conventional_commits = true
check_changelog = true

[package_tools.audit.version_consistency]
fail_on_inconsistency = false
warn_on_inconsistency = true

[package_tools.audit.health_score_weights]
critical_weight = 15.0
warning_weight = 5.0
info_weight = 1.0
security_multiplier = 1.5
breaking_changes_multiplier = 1.3
dependencies_multiplier = 1.2
version_consistency_multiplier = 1.0
upgrades_multiplier = 0.8
other_multiplier = 1.0

# Optional: Only for monorepo projects with workspace patterns
[package_tools.workspace]
patterns = ["packages/*", "apps/*"]
```

---

### Configuration Reference

#### `[package_tools.changeset]` - Changeset Management

Controls where changesets are stored and what environments are available.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `path` | String | `".changesets"` | Directory for active changesets |
| `history_path` | String | `".changesets/history"` | Directory for archived changesets |
| `available_environments` | Array | `["production"]` | Valid environment names for deployment targeting |
| `default_environments` | Array | `["production"]` | Environments used when none are specified |

**Example:**
```toml
[package_tools.changeset]
path = ".changesets"
history_path = ".changesets/history"
available_environments = ["dev", "staging", "prod"]
default_environments = ["prod"]
```

---

#### `[package_tools.version]` - Versioning Strategy

Defines how package versions are calculated and applied.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `strategy` | String | `"independent"` | Versioning strategy: `"independent"` (each package has own version) or `"unified"` (all packages share same version) |
| `default_bump` | String | `"patch"` | Default version bump when not specified in changeset: `"major"`, `"minor"`, `"patch"`, or `"none"` |
| `snapshot_format` | String | `"{version}-{branch}.{timestamp}"` | Format template for snapshot versions. Placeholders: `{version}`, `{branch}`, `{timestamp}`, `{short_hash}` |

**Example:**
```toml
[package_tools.version]
strategy = "unified"
default_bump = "minor"
snapshot_format = "{version}-snapshot.{short_hash}"
```

---

#### `[package_tools.dependency]` - Dependency Propagation

Controls how version changes propagate through the dependency graph.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `propagation_bump` | String | `"patch"` | Version bump type for propagated updates: `"major"`, `"minor"`, `"patch"`, or `"none"` |
| `propagate_dependencies` | Boolean | `true` | Propagate version updates to regular dependencies |
| `propagate_dev_dependencies` | Boolean | `false` | Propagate version updates to devDependencies |
| `propagate_peer_dependencies` | Boolean | `true` | Propagate version updates to peerDependencies |
| `max_depth` | Integer | `10` | Maximum depth for dependency propagation (0 = unlimited) |
| `fail_on_circular` | Boolean | `true` | Fail when circular dependencies are detected |
| `skip_workspace_protocol` | Boolean | `true` | Skip dependencies using workspace protocol (`workspace:*`) |
| `skip_file_protocol` | Boolean | `true` | Skip dependencies using file protocol (`file:../path`) |
| `skip_link_protocol` | Boolean | `true` | Skip dependencies using link protocol (`link:../path`) |
| `skip_portal_protocol` | Boolean | `true` | Skip dependencies using portal protocol (`portal:../path`) |

**Example:**
```toml
[package_tools.dependency]
propagation_bump = "minor"
propagate_dependencies = true
propagate_dev_dependencies = true
max_depth = 5
fail_on_circular = false
```

---

#### `[package_tools.upgrade]` - Dependency Upgrades

Settings for detecting and applying external dependency upgrades.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `auto_changeset` | Boolean | `true` | Automatically create changeset for upgrades |
| `changeset_bump` | String | `"patch"` | Version bump type for automatic changeset: `"major"`, `"minor"`, `"patch"`, or `"none"` |

**Example:**
```toml
[package_tools.upgrade]
auto_changeset = true
changeset_bump = "patch"
```

##### `[package_tools.upgrade.registry]` - Registry Configuration

NPM registry communication settings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `default_registry` | String | `"https://registry.npmjs.org"` | Default NPM registry URL |
| `scoped_registries` | Map | `{}` | Scoped registry mappings (scope → URL, without `@` prefix) |
| `auth_tokens` | Map | `{}` | Authentication tokens (registry URL → token) |
| `timeout_secs` | Integer | `30` | HTTP request timeout in seconds |
| `retry_attempts` | Integer | `3` | Number of retry attempts for failed requests |
| `retry_delay_ms` | Integer | `1000` | Delay between retry attempts in milliseconds |
| `read_npmrc` | Boolean | `true` | Read configuration from `.npmrc` files (workspace root + user home directory). Workspace `.npmrc` takes precedence over user `~/.npmrc` |

**`.npmrc` File Support:**

When `read_npmrc` is enabled, the system automatically reads NPM registry configuration from:
1. **User home directory**: `~/.npmrc` (lower precedence)
2. **Workspace root**: `.npmrc` (higher precedence, overrides user settings)

The `.npmrc` file format supports:
- **Registry URLs**: `registry=https://registry.npmjs.org`
- **Scoped registries**: `@myorg:registry=https://npm.myorg.com`
- **Authentication tokens**: `//npm.myorg.com/:_authToken=npm_AbCdEf123456`
- **Environment variables**: `//npm.myorg.com/:_authToken=${NPM_TOKEN}`
- **Comments**: Lines starting with `#` or inline comments

**Example `.npmrc`:**
```ini
# Default registry
registry=https://registry.npmjs.org

# Scoped registry for @myorg packages
@myorg:registry=https://npm.myorg.com

# Authentication token using environment variable
//npm.myorg.com/:_authToken=${NPM_TOKEN}
```

**Example `repo.config.toml`:**
```toml
[package_tools.upgrade.registry]
default_registry = "https://registry.npmjs.org"
timeout_secs = 60
retry_attempts = 5
read_npmrc = true  # Reads from ~/.npmrc and workspace .npmrc

[package_tools.upgrade.registry.scoped_registries]
myorg = "https://npm.myorg.com"  # Note: scope without @ prefix

[package_tools.upgrade.registry.auth_tokens]
"npm.myorg.com" = "npm_token_here"  # Can also use "https://npm.myorg.com"
```

**Precedence Order (highest to lowest):**
1. `auth_tokens` and `scoped_registries` in `repo.config.toml`
2. Workspace `.npmrc` (project root)
3. User `~/.npmrc` (home directory)

##### `[package_tools.upgrade.backup]` - Backup Configuration

Backup and rollback settings for upgrade operations.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | Boolean | `true` | Enable automatic backups before upgrades |
| `backup_dir` | String | `".workspace-backups"` | Directory where backups are stored |
| `keep_after_success` | Boolean | `false` | Keep backups after successful operations |
| `max_backups` | Integer | `5` | Maximum number of backups to retain |

**Example:**
```toml
[package_tools.upgrade.backup]
enabled = true
backup_dir = ".backups"
keep_after_success = true
max_backups = 10
```

---

#### `[package_tools.changelog]` - Changelog Generation

Controls how changelogs are generated and formatted.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | Boolean | `true` | Enable changelog generation |
| `format` | String | `"keep-a-changelog"` | Changelog format: `"keep-a-changelog"`, `"conventional"`, or `"custom"` |
| `filename` | String | `"CHANGELOG.md"` | Changelog filename |
| `include_commit_links` | Boolean | `true` | Include links to commits |
| `include_issue_links` | Boolean | `true` | Include links to issues (e.g., #123) |
| `include_authors` | Boolean | `false` | Include author attribution |
| `repository_url` | String | `None` | Repository URL for generating links (auto-detected from git remote if not set) |
| `monorepo_mode` | String | `"per-package"` | Changelog location: `"per-package"`, `"root"`, or `"both"` |
| `version_tag_format` | String | `"{name}@{version}"` | Format for version tags in monorepo. Placeholders: `{name}`, `{version}` |
| `root_tag_format` | String | `"v{version}"` | Format for root version tags. Placeholder: `{version}` |

**Example:**
```toml
[package_tools.changelog]
enabled = true
format = "conventional"
include_authors = true
repository_url = "https://github.com/myorg/myrepo"
monorepo_mode = "both"
```

##### `[package_tools.changelog.conventional]` - Conventional Commits

Conventional commits parsing configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | Boolean | `true` | Enable conventional commits parsing |
| `breaking_section` | String | `"Breaking Changes"` | Title for breaking changes section |
| `types` | Map | See below | Map of commit types to display titles |

**Default Types:**
```toml
[package_tools.changelog.conventional.types]
feat = "Features"
fix = "Bug Fixes"
perf = "Performance Improvements"
refactor = "Code Refactoring"
docs = "Documentation"
build = "Build System"
ci = "Continuous Integration"
test = "Tests"
chore = "Chores"
```

##### `[package_tools.changelog.exclude]` - Exclusion Rules

Defines which commits to exclude from changelogs.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `patterns` | Array | `[]` | Regex patterns for commit messages to exclude |
| `authors` | Array | `[]` | Authors whose commits should be excluded |

**Example:**
```toml
[package_tools.changelog.exclude]
patterns = ["^chore\\(release\\):", "^Merge branch"]
authors = ["dependabot[bot]"]
```

##### `[package_tools.changelog.template]` - Custom Templates

Templates for changelog generation.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `header` | String | See default | Template for changelog header |
| `version_header` | String | `"## [{version}] - {date}"` | Template for version headers. Placeholders: `{version}`, `{date}` |
| `section_header` | String | `"### {section}"` | Template for section headers. Placeholder: `{section}` |
| `entry_format` | String | `"- {description} ({hash})"` | Template for individual entries. Placeholders: `{description}`, `{hash}` |

---

#### `[package_tools.git]` - Git Integration

Git commit message templates and breaking change warnings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `merge_commit_template` | String | See default | Template for single-package release commits |
| `monorepo_merge_commit_template` | String | See default | Template for monorepo release commits |
| `include_breaking_warning` | Boolean | `true` | Include breaking change warnings in merge commits |
| `breaking_warning_template` | String | `"\n⚠️  BREAKING CHANGES: {breaking_changes_count}\n"` | Template for breaking change warnings |

**Available Placeholders:**
- `{version}` - New version being released
- `{previous_version}` - Previous version
- `{package_name}` - Package name
- `{bump_type}` - Version bump type (Major, Minor, Patch, None)
- `{date}` - Release date (YYYY-MM-DD)
- `{breaking_changes_count}` - Number of breaking changes
- `{features_count}` - Number of new features
- `{fixes_count}` - Number of bug fixes
- `{changelog_summary}` - Brief summary from changelog
- `{author}` - Current git user

**Default Templates:**
```toml
[package_tools.git]
merge_commit_template = """chore(release): {version}

Release version {version}

{changelog_summary}"""

monorepo_merge_commit_template = """chore(release): {package_name}@{version}

Release {package_name} version {version}

{changelog_summary}"""
```

---

#### `[package_tools.audit]` - Audit Configuration

Settings for project health checks and audits.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | Boolean | `true` | Enable audit system |
| `min_severity` | String | `"warning"` | Minimum severity level for reporting: `"critical"`, `"warning"`, or `"info"` |

##### `[package_tools.audit.sections]` - Audit Sections

Controls which audit sections to execute.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `upgrades` | Boolean | `true` | Run upgrade availability audits |
| `dependencies` | Boolean | `true` | Run dependency health audits |
| `breaking_changes` | Boolean | `true` | Check for breaking changes |
| `categorization` | Boolean | `true` | Categorize dependencies |
| `version_consistency` | Boolean | `true` | Check version consistency |

##### `[package_tools.audit.upgrades]` - Upgrade Audit

Controls which upgrade types to include in audits.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `include_patch` | Boolean | `true` | Include patch version upgrades |
| `include_minor` | Boolean | `true` | Include minor version upgrades |
| `include_major` | Boolean | `true` | Include major version upgrades |
| `deprecated_as_critical` | Boolean | `true` | Treat deprecated packages as critical issues |

##### `[package_tools.audit.dependencies]` - Dependency Audit

Controls which dependency checks to perform.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `check_circular` | Boolean | `true` | Detect circular dependencies |
| `check_missing` | Boolean | `false` | Check for missing dependencies |
| `check_unused` | Boolean | `false` | Check for unused dependencies |
| `check_version_conflicts` | Boolean | `true` | Check for version conflicts |

##### `[package_tools.audit.breaking_changes]` - Breaking Changes Audit

Controls how breaking changes are detected.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `check_conventional_commits` | Boolean | `true` | Check for breaking changes in conventional commits |
| `check_changelog` | Boolean | `true` | Check for breaking changes in changelogs |

##### `[package_tools.audit.version_consistency]` - Version Consistency

Controls how version inconsistencies are handled.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `fail_on_inconsistency` | Boolean | `false` | Fail when version inconsistencies are detected |
| `warn_on_inconsistency` | Boolean | `true` | Warn when version inconsistencies are detected |

##### `[package_tools.audit.health_score_weights]` - Health Score Weights

Controls how issues affect the overall health score calculation.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `critical_weight` | Float | `15.0` | Points deducted per critical issue |
| `warning_weight` | Float | `5.0` | Points deducted per warning issue |
| `info_weight` | Float | `1.0` | Points deducted per info issue |
| `security_multiplier` | Float | `1.5` | Multiplier for security issues |
| `breaking_changes_multiplier` | Float | `1.3` | Multiplier for breaking changes issues |
| `dependencies_multiplier` | Float | `1.2` | Multiplier for dependency issues |
| `version_consistency_multiplier` | Float | `1.0` | Multiplier for version consistency issues |
| `upgrades_multiplier` | Float | `0.8` | Multiplier for upgrade issues |
| `other_multiplier` | Float | `1.0` | Multiplier for other issues |

**Example:**
```toml
[package_tools.audit.health_score_weights]
critical_weight = 20.0
warning_weight = 10.0
security_multiplier = 2.0
breaking_changes_multiplier = 1.5
```

---

#### `[package_tools.workspace]` - Workspace Configuration (Optional)

Project-specific workspace patterns for monorepo projects. This section is optional and only used for monorepo projects.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `patterns` | Array | `[]` | Workspace patterns from package.json (e.g., `["packages/*", "apps/*"]`) |

**Example:**
```toml
[package_tools.workspace]
patterns = ["packages/*", "apps/*", "libs/*"]
```

**Note:** This represents the actual workspace patterns declared in your project's `package.json`. For single-package projects, this section is omitted entirely.

---

### Configuration Management

#### View Configuration

```bash
# Show current configuration
workspace config show

# Show as JSON
workspace --format json config show
```

#### Validate Configuration

```bash
# Validate configuration file
workspace config validate

# Validate with detailed logging
workspace --log-level debug config validate
```

#### Initialize Configuration

```bash
# Interactive initialization
workspace init

# Non-interactive with defaults
workspace init --non-interactive

# Specify options
workspace init --strategy unified --config-format yaml
```

---

## Architecture

### Error Handling

The CLI uses a unified `CliError` enum that wraps all error types:

- `Configuration`: Config file errors
- `Validation`: Argument/state validation errors
- `Execution`: Command execution failures
- `Git`: Git operation errors
- `Package`: Package/package.json errors
- `Io`: File system errors
- `Network`: Network/registry errors
- `User`: User-caused errors (invalid input, cancelled operations)

Each error maps to appropriate exit codes following sysexits conventions and includes user-friendly messages with actionable suggestions.

### Output System

The CLI separates logging from output:

**Logging** (`--log-level`): Controls stderr output for debugging
- Levels: silent, error, warn, info, debug, trace
- Always goes to stderr
- Independent from output format

**Output** (`--format`): Controls stdout for command results
- Formats: human, json, json-compact, quiet
- Always goes to stdout
- Independent from logging

This separation ensures JSON output is never mixed with logs, perfect for CI/CD pipelines.

---

## Contributing

This is an active development project. Please follow the established patterns:

1. Check [STORY_MAP.md](./STORY_MAP.md) for planned work
2. Follow the implementation guidelines in [PLAN.md](./PLAN.md)
3. Read the [PRD.md](./PRD.md) for feature requirements
4. Ensure 100% Clippy compliance
5. Document all public APIs with examples
6. Write comprehensive tests
7. Use conventional commits for automatic releases

See [../../CONTRIBUTING.md](../../CONTRIBUTING.md) for detailed contribution guidelines.

---

## License

This project is licensed under the MIT License - see the [../../LICENSE-MIT](../../LICENSE-MIT) file for details.

---

## Links

- **[Root Project README](../../README.md)** - Project overview and features

### Planning Documents (Historical)

- **[Product Requirements](./docs/PRD.md)** - Feature requirements and initial design
- **[Implementation Plan](./docs/PLAN.md)** - Detailed technical plan
- **[Story Map](./docs/STORY_MAP.md)** - Development roadmap
- **[CLI Draft](./docs/CLI.md)** - Initial CLI design concepts

---

<div align="center">


**[Contributing](../../CONTRIBUTING.md)** •
**[Issues](https://github.com/websublime/workspace-tools/issues)**

Part of [Workspace Tools](../../README.md)

</div>
