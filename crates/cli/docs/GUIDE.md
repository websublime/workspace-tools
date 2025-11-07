# Workspace Tools - User Guide

**Version:** 0.1.0
**Last Updated:** 2025-11-07

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Installation](#installation)
3. [Core Concepts](#core-concepts)
4. [Configuration](#configuration)
5. [Command Reference](#command-reference)
6. [Workflows](#workflows)
7. [CI/CD Integration](#cicd-integration)
8. [Best Practices](#best-practices)
9. [Troubleshooting](#troubleshooting)
10. [FAQ](#faq)
11. [Migration Guide](#migration-guide)

---

## Getting Started

### What is Workspace Tools?

Workspace Tools (`workspace`) is a CLI tool for managing Node.js projects with changeset-based versioning. It simplifies version management, dependency tracking, and project health monitoring in both single-package and monorepo environments.

### Key Features

- **Changeset System** - Track changes across feature branches with automated package detection
- **Version Management** - Bump versions with dependency propagation and multiple strategies
- **Upgrade Management** - Detect and apply dependency upgrades with safety checks
- **Audit System** - Comprehensive project health analysis with actionable insights
- **Git Integration** - Seamless workflow integration with hooks and CI/CD
- **CI/CD Ready** - JSON output modes for pipeline integration

### Quick Start (5 Minutes)

```bash
# 1. Install workspace
curl -fsSL https://install.workspace.dev | sh

# 2. Initialize your project
cd your-project
workspace init

# 3. Create a changeset for your feature
git checkout -b feature/new-api
workspace changeset create

# 4. Make your changes and commit
# ... edit files ...
git commit -m "feat: add new API endpoint"
workspace changeset update

# 5. Preview version bump
workspace bump --dry-run

# 6. Apply version changes
workspace bump --execute
```

---

## Installation

### Method 1: Curl Script (Recommended)

```bash
curl -fsSL https://install.workspace.dev | sh
```

The installation script will:
- Detect your OS and architecture
- Download the appropriate binary
- Verify checksums
- Install to `/usr/local/bin` or `~/.local/bin`
- Set up shell completions

#### Custom Installation Options

```bash
# Install specific version
curl -fsSL https://install.workspace.dev | sh -s -- --version v0.1.0

# Install to custom directory
curl -fsSL https://install.workspace.dev | sh -s -- --install-dir ~/.local/bin

# Verbose output
curl -fsSL https://install.workspace.dev | sh -s -- --verbose
```

### Method 2: Cargo

```bash
cargo install workspace
```

### Method 3: Pre-built Binaries

Download from [GitHub Releases](https://github.com/org/workspace-node-tools/releases):

- **macOS (Intel):** `workspace-macos-x86_64.tar.gz`
- **macOS (Apple Silicon):** `workspace-macos-aarch64.tar.gz`
- **Linux (x64):** `workspace-linux-x86_64.tar.gz`
- **Windows:** `workspace-windows-x86_64.zip`

#### Manual Installation

```bash
# Download and extract
tar -xzf workspace-*.tar.gz

# Move to PATH
sudo mv workspace /usr/local/bin/

# Verify installation
workspace --version
```

### Shell Completions

Completions are generated during installation. To manually install:

```bash
# Bash
workspace completions bash > ~/.local/share/bash-completion/completions/workspace

# Zsh
workspace completions zsh > ~/.zsh/completion/_workspace

# Fish
workspace completions fish > ~/.config/fish/completions/workspace.fish
```

### Verifying Installation

```bash
# Check version
workspace --version

# View detailed version info
workspace version --verbose

# Test with help
workspace --help
```

---

## Core Concepts

### Changesets

A **changeset** is a file that describes intended version bumps for one or more packages. Changesets are:

- Created per feature branch
- Track which packages are affected
- Specify the bump type (major, minor, patch)
- Include target environments
- Accumulate commits as work progresses

**Benefits:**
- Clear audit trail from change to release
- Team visibility into pending releases
- Safe versioning decisions before merge
- CI/CD integration for automated releases

### Versioning Strategies

#### Independent Strategy

Each package maintains its own version number independently.

```yaml
version:
  strategy: independent
```

**When to use:**
- Packages evolve at different rates
- Want to minimize version churn
- Clear package-level versioning

**Example:**
- `@myorg/core` bumps from 1.2.0 → 1.3.0
- `@myorg/utils` stays at 2.1.0 (no changes)
- `@myorg/cli` bumps from 0.5.0 → 0.5.1

#### Unified Strategy

All packages share the same version number.

```yaml
version:
  strategy: unified
```

**When to use:**
- Packages are tightly coupled
- Want coordinated releases
- Simplify version management

**Example:**
- All packages bump from 1.2.0 → 1.3.0 together
- Highest bump type wins (minor > patch)

### Environments

Environments define deployment targets for changesets.

**Common environments:**
- `dev` - Development environment
- `staging` - Staging/QA environment
- `production` - Production environment
- `qa` - Quality assurance environment

**Usage:**
```bash
# Create changeset for staging and production
workspace changeset create --env staging,production
```

### Package Detection

The CLI automatically detects affected packages by:

1. Analyzing Git changes in the working directory
2. Mapping file paths to package locations
3. Following workspace configuration
4. Detecting monorepo structure

**Supported project types:**
- Single package repositories
- npm workspaces
- Yarn workspaces
- pnpm workspaces
- Bun workspaces

---

## Configuration

Workspace Tools uses a layered configuration system that spans from low-level infrastructure (standard-tools) through package management (pkg-tools) to CLI-specific settings.

### Configuration Architecture

Configuration is organized in three layers:

1. **Standard Tools Layer** - Infrastructure configuration (filesystem, commands, monorepo detection)
2. **Package Tools Layer** - Package management configuration (changesets, versioning, upgrades, auditing)
3. **CLI Layer** - User interface configuration (output, logging, global settings)

### Configuration Sources & Priority

Configuration is loaded and merged from multiple sources (highest priority first):

1. **CLI Flags** - Command-line arguments (`--root`, `--config`, `--log-level`, etc.)
2. **Environment Variables** - `SUBLIME_*` prefixed variables
3. **Project Config File** - `repo.config.{toml,json,yaml}` in project root
4. **Global Config File** - `~/.config/sublime/config.toml` (optional)
5. **Default Values** - Built-in sensible defaults

---

### Initialization

```bash
# Interactive mode (recommended)
workspace init

# Non-interactive with defaults
workspace init --non-interactive

# Custom configuration
workspace init \
  --strategy independent \
  --config-format yaml \
  --environments "dev,staging,prod" \
  --default-env "prod" \
  --changeset-path .changesets
```

---

### Complete Configuration Reference

The following sections document all available configuration options across all layers.

#### Layer 1: Standard Tools Configuration

These settings control low-level infrastructure behavior.

##### Package Manager Configuration

```toml
[packageManagers]
# Detection order for package managers (first match wins)
detectionOrder = ["pnpm", "yarn", "npm", "bun", "jsr"]

# Custom lock file names (if non-standard)
[packageManagers.customLockFiles]
npm = "package-lock.json"
yarn = "yarn.lock"
pnpm = "pnpm-lock.yaml"

# Whether to detect package manager from environment variables
detectFromEnv = true

# Environment variable name for preferred package manager
envVarName = "PACKAGE_MANAGER"

# Custom binary paths for package managers (optional)
[packageManagers.binaryPaths]
npm = "/usr/local/bin/npm"

# Fallback package manager if none detected
fallback = "npm"
```

**Environment Variable Overrides:**
- `SUBLIME_PACKAGE_MANAGER_ORDER` - Comma-separated list (e.g., "npm,yarn,pnpm,bun,jsr")
- `SUBLIME_PACKAGE_MANAGER` - Preferred package manager name

##### Monorepo Configuration

```toml
[monorepo]
# Custom workspace directory patterns (glob patterns)
workspacePatterns = ["packages/*", "apps/*", "libs/*"]

# Additional directories to check for packages
packageDirectories = ["packages", "apps", "libs"]

# Patterns to exclude from package detection
excludePatterns = ["**/node_modules", "**/.git", "**/dist"]

# Maximum depth for recursive package search (1-20)
maxSearchDepth = 5

# Whether to follow symlinks during search
followSymlinks = false

# Custom patterns for workspace detection in package.json
customWorkspaceFields = ["workspaces", "workspace"]
```

**Environment Variable Overrides:**
- `SUBLIME_WORKSPACE_PATTERNS` - Comma-separated patterns (e.g., "packages/*,apps/*")
- `SUBLIME_PACKAGE_DIRECTORIES` - Comma-separated directory names
- `SUBLIME_EXCLUDE_PATTERNS` - Comma-separated exclude patterns
- `SUBLIME_MAX_SEARCH_DEPTH` - Maximum search depth (1-20)

##### Command Execution Configuration

```toml
[commands]
# Default timeout for command execution (in seconds, 1-3600)
defaultTimeout = "120s"

# Timeout overrides for specific commands
[commands.timeoutOverrides]
"npm install" = "300s"
"npm test" = "600s"

# Buffer size for command output streaming (bytes, 256-65536)
streamBufferSize = 8192

# Read timeout for streaming output (in seconds)
streamReadTimeout = "5s"

# Maximum concurrent commands in queue (1-100)
maxConcurrentCommands = 5

# Environment variables to set for all commands
[commands.envVars]
NODE_ENV = "production"

# Whether to inherit parent process environment
inheritEnv = true

# Queue collection window duration (milliseconds, 1-1000)
queueCollectionWindowMs = 100

# Queue collection sleep duration (microseconds, 10-10000)
queueCollectionSleepUs = 500

# Queue idle sleep duration (milliseconds, 1-1000)
queueIdleSleepMs = 10
```

**Environment Variable Overrides:**
- `SUBLIME_COMMAND_TIMEOUT` - Command execution timeout in seconds (1-3600)
- `SUBLIME_MAX_CONCURRENT` - Maximum concurrent commands (1-100)
- `SUBLIME_BUFFER_SIZE` - Command output buffer size in bytes (256-65536)
- `SUBLIME_COLLECTION_WINDOW_MS` - Queue collection window in milliseconds (1-1000)
- `SUBLIME_COLLECTION_SLEEP_US` - Queue collection sleep in microseconds (10-10000)
- `SUBLIME_IDLE_SLEEP_MS` - Queue idle sleep in milliseconds (1-1000)

##### Filesystem Configuration

```toml
[filesystem]
# Patterns to ignore during directory traversal
ignorePatterns = [
  "node_modules",
  ".git",
  "dist",
  "build",
  "coverage"
]

# Async I/O configuration
[filesystem.asyncIo]
# Buffer size for async I/O operations (bytes, 1024-1048576)
bufferSize = 8192

# Maximum concurrent filesystem operations (1-1000)
maxConcurrentOperations = 100

# Timeout for individual operations (seconds, 1-300)
operationTimeout = "30s"

# Retry configuration for filesystem operations
[filesystem.retry]
# Maximum number of retry attempts
maxAttempts = 3

# Initial delay between retries
initialDelay = "100ms"

# Maximum delay between retries
maxDelay = "5s"

# Backoff multiplier for exponential backoff
backoffMultiplier = 2.0

# Path conventions (override defaults)
[filesystem.pathConventions]
nodeModules = "node_modules"
packageJson = "package.json"
```

**Environment Variable Overrides:**
- `SUBLIME_IGNORE_PATTERNS` - Comma-separated filesystem ignore patterns
- `SUBLIME_ASYNC_BUFFER_SIZE` - Async I/O buffer size in bytes (1024-1048576)
- `SUBLIME_MAX_CONCURRENT_IO` - Maximum concurrent I/O operations (1-1000)
- `SUBLIME_IO_TIMEOUT` - I/O operation timeout in seconds (1-300)

##### Validation Configuration

```toml
[validation]
# Whether to require package.json at project root
requirePackageJson = true

# Required fields in package.json
requiredPackageFields = ["name", "version"]

# Whether to validate dependency versions
validateDependencies = true

# Custom validation rules (JSON values)
[validation.customRules]
# Example: custom rule data

# Whether to fail on validation warnings
strictMode = false
```

---

#### Layer 2: Package Tools Configuration

These settings control package management behavior.

##### Changeset Configuration

```toml
[changeset]
# Path to store active changesets
path = ".changesets"

# Path to store archived changesets
historyPath = ".changesets/history"

# List of valid environment names
environments = ["dev", "staging", "production"]

# Default environments for new changesets
defaultEnvironments = ["production"]
```

##### Version Configuration

```toml
[version]
# Versioning strategy: "independent" or "unified"
strategy = "independent"

# Default version bump type: "major", "minor", "patch", "none"
defaultBump = "patch"

# Format template for snapshot versions
# Variables: {version}, {branch}, {commit}, {short_commit}, {timestamp}
snapshotFormat = "{version}-{branch}.{short_commit}"

# Tag configuration
[version.tag]
# Tag format for releases
# Variables: {name}, {version}
format = "{name}@{version}"

# Optional prefix (e.g., "v" for "v1.0.0")
prefix = ""

# Include prerelease tags
includePrerelease = false
```

##### Dependency Configuration

```toml
[dependency]
# Version bump type for dependency updates: "major", "minor", "patch"
propagationBump = "patch"

# Whether to propagate regular dependencies
propagateDependencies = true

# Whether to propagate dev dependencies
propagateDevDependencies = false

# Whether to propagate peer dependencies
propagatePeerDependencies = false

# Maximum propagation depth
maxDepth = 10

# Whether to fail on circular dependencies
failOnCircular = true

# Skip workspace: protocol dependencies
skipWorkspaceProtocol = true

# Skip file: protocol dependencies
skipFileProtocol = true

# Skip link: protocol dependencies
skipLinkProtocol = true

# Skip portal: protocol dependencies
skipPortalProtocol = true
```

##### Upgrade Configuration

```toml
[upgrade]
# Automatically create changesets for upgrades
autoChangeset = false

# Version bump type for upgrade changesets: "major", "minor", "patch"
changesetBump = "patch"

# Registry configuration
[upgrade.registry]
# Default npm registry URL
defaultRegistry = "https://registry.npmjs.org"

# Scoped package registries
[upgrade.registry.scopedRegistries]
"@myorg" = "https://npm.myorg.com"

# Request timeout in seconds
timeoutSecs = 30

# Number of retry attempts
retryAttempts = 3

# Whether to read .npmrc configuration
readNpmrc = true

# Backup and rollback configuration
[upgrade.backup]
# Whether backups are enabled
enabled = true

# Path to store backups
path = ".workspace-backups"

# Number of backups to keep
keepCount = 5
```

##### Changelog Configuration

```toml
[changelog]
# Whether changelog generation is enabled
enabled = true

# Changelog format: "keepachangelog", "conventional", "custom"
format = "keepachangelog"

# Include links to commits
includeCommitLinks = true

# Repository URL for links (optional, auto-detected from git)
repositoryUrl = "https://github.com/org/repo"

# Conventional commits configuration
[changelog.conventional]
# Commit types to include
types = ["feat", "fix", "perf", "refactor", "docs", "style", "test", "build", "ci", "chore", "revert"]

# Section titles for each type
[changelog.conventional.sectionTitles]
feat = "Features"
fix = "Bug Fixes"
perf = "Performance Improvements"
refactor = "Refactoring"
docs = "Documentation"
style = "Style Changes"
test = "Tests"
build = "Build System"
ci = "CI/CD"
chore = "Chores"
revert = "Reverts"

# Template configuration (for custom format)
[changelog.template]
# Header template
header = "# Changelog\n\n"

# Version template
version = "## {version} ({date})\n\n"

# Section template
section = "### {title}\n\n"

# Entry template
entry = "- {message} ([{commit}]({commitUrl}))\n"

# Exclusion patterns
[changelog.exclude]
# Commit message patterns to exclude
patterns = ["^WIP:", "^test:", "^chore\\(release\\):"]

# Authors to exclude
authors = []

# Monorepo changelog mode: "perpackage", "root", "both"
monorepoMode = "perpackage"
```

##### Audit Configuration

```toml
[audit]
# Whether audits are enabled
enabled = true

# Minimum severity to report: "critical", "high", "medium", "low", "info"
minSeverity = "low"

# Configuration for audit sections
[audit.sections]
# Enable upgrades audit
upgrades = true

# Enable dependencies audit
dependencies = true

# Enable version consistency audit
versionConsistency = true

# Enable breaking changes audit
breakingChanges = true

# Weights for health score calculation
[audit.healthScoreWeights]
upgradesWeight = 0.25
dependenciesWeight = 0.30
versionConsistencyWeight = 0.25
breakingChangesWeight = 0.20
```

##### Git Configuration

```toml
[git]
# Base branch for comparisons
branchBase = "main"

# Auto-detect affected packages from Git changes
detectAffectedPackages = true

# Whether to require a clean working directory
requireCleanWorkingDirectory = false

# Whether to tag on release
tagOnRelease = true

# Whether to push tags automatically
pushTags = false
```

---

#### Layer 3: CLI Configuration

These settings control CLI-specific behavior (typically set via flags, not config file).

**Global Options** (apply to all commands):

| Option | Description | Default | Environment Variable |
|--------|-------------|---------|---------------------|
| `--root <PATH>` | Project root directory | Current directory | `WORKSPACE_ROOT` |
| `--config <PATH>` | Config file path | Auto-detect | `WORKSPACE_CONFIG` |
| `--log-level <LEVEL>` | Log level (stderr) | `info` | `WORKSPACE_LOG_LEVEL` |
| `--format <FORMAT>` | Output format (stdout) | `human` | `WORKSPACE_FORMAT` |
| `--no-color` | Disable colored output | `false` | `NO_COLOR` (any value) |

**Log Levels:**
- `silent` - No logs
- `error` - Only critical errors
- `warn` - Errors + warnings
- `info` - General progress (default)
- `debug` - Detailed operations
- `trace` - Very verbose debugging

**Output Formats:**
- `human` - Human-readable with colors and tables (default)
- `json` - Pretty-printed JSON
- `json-compact` - Compact JSON (single line)
- `quiet` - Minimal output

---

### Complete Configuration Examples

#### Full Configuration (TOML)

```toml
# ============================================================================
# Standard Tools Configuration (Layer 1)
# ============================================================================

[packageManagers]
detectionOrder = ["pnpm", "yarn", "npm"]
detectFromEnv = true
envVarName = "PACKAGE_MANAGER"
fallback = "npm"

[monorepo]
workspacePatterns = ["packages/*", "apps/*"]
packageDirectories = ["packages", "apps"]
excludePatterns = ["**/node_modules", "**/.git", "**/dist"]
maxSearchDepth = 5
followSymlinks = false

[commands]
defaultTimeout = "120s"
streamBufferSize = 8192
maxConcurrentCommands = 5
inheritEnv = true

[filesystem]
ignorePatterns = ["node_modules", ".git", "dist", "build"]

[filesystem.asyncIo]
bufferSize = 8192
maxConcurrentOperations = 100
operationTimeout = "30s"

[validation]
requirePackageJson = true
requiredPackageFields = ["name", "version"]
validateDependencies = true
strictMode = false

# ============================================================================
# Package Tools Configuration (Layer 2)
# ============================================================================

[changeset]
path = ".changesets"
historyPath = ".changesets/history"
environments = ["dev", "staging", "production"]
defaultEnvironments = ["production"]

[version]
strategy = "independent"
defaultBump = "patch"
snapshotFormat = "{version}-{branch}.{short_commit}"

[version.tag]
format = "{name}@{version}"
prefix = ""
includePrerelease = false

[dependency]
propagationBump = "patch"
propagateDependencies = true
propagateDevDependencies = false
propagatePeerDependencies = false
maxDepth = 10
failOnCircular = true
skipWorkspaceProtocol = true
skipFileProtocol = true
skipLinkProtocol = true
skipPortalProtocol = true

[upgrade]
autoChangeset = false
changesetBump = "patch"

[upgrade.registry]
defaultRegistry = "https://registry.npmjs.org"
timeoutSecs = 30
retryAttempts = 3
readNpmrc = true

[upgrade.backup]
enabled = true
path = ".workspace-backups"
keepCount = 5

[changelog]
enabled = true
format = "keepachangelog"
includeCommitLinks = true
monorepoMode = "perpackage"

[audit]
enabled = true
minSeverity = "low"

[audit.sections]
upgrades = true
dependencies = true
versionConsistency = true
breakingChanges = true

[audit.healthScoreWeights]
upgradesWeight = 0.25
dependenciesWeight = 0.30
versionConsistencyWeight = 0.25
breakingChangesWeight = 0.20

[git]
branchBase = "main"
detectAffectedPackages = true
requireCleanWorkingDirectory = false
tagOnRelease = true
pushTags = false
```

#### Full Configuration (YAML)

```yaml
# ============================================================================
# Standard Tools Configuration (Layer 1)
# ============================================================================

packageManagers:
  detectionOrder:
    - pnpm
    - yarn
    - npm
  detectFromEnv: true
  envVarName: PACKAGE_MANAGER
  fallback: npm

monorepo:
  workspacePatterns:
    - "packages/*"
    - "apps/*"
  packageDirectories:
    - packages
    - apps
  excludePatterns:
    - "**/node_modules"
    - "**/.git"
    - "**/dist"
  maxSearchDepth: 5
  followSymlinks: false

commands:
  defaultTimeout: 120s
  streamBufferSize: 8192
  maxConcurrentCommands: 5
  inheritEnv: true

filesystem:
  ignorePatterns:
    - node_modules
    - .git
    - dist
    - build
  asyncIo:
    bufferSize: 8192
    maxConcurrentOperations: 100
    operationTimeout: 30s

validation:
  requirePackageJson: true
  requiredPackageFields:
    - name
    - version
  validateDependencies: true
  strictMode: false

# ============================================================================
# Package Tools Configuration (Layer 2)
# ============================================================================

changeset:
  path: .changesets
  historyPath: .changesets/history
  environments:
    - dev
    - staging
    - production
  defaultEnvironments:
    - production

version:
  strategy: independent
  defaultBump: patch
  snapshotFormat: "{version}-{branch}.{short_commit}"
  tag:
    format: "{name}@{version}"
    prefix: ""
    includePrerelease: false

dependency:
  propagationBump: patch
  propagateDependencies: true
  propagateDevDependencies: false
  propagatePeerDependencies: false
  maxDepth: 10
  failOnCircular: true
  skipWorkspaceProtocol: true
  skipFileProtocol: true
  skipLinkProtocol: true
  skipPortalProtocol: true

upgrade:
  autoChangeset: false
  changesetBump: patch
  registry:
    defaultRegistry: "https://registry.npmjs.org"
    timeoutSecs: 30
    retryAttempts: 3
    readNpmrc: true
  backup:
    enabled: true
    path: .workspace-backups
    keepCount: 5

changelog:
  enabled: true
  format: keepachangelog
  includeCommitLinks: true
  monorepoMode: perpackage

audit:
  enabled: true
  minSeverity: low
  sections:
    upgrades: true
    dependencies: true
    versionConsistency: true
    breakingChanges: true
  healthScoreWeights:
    upgradesWeight: 0.25
    dependenciesWeight: 0.30
    versionConsistencyWeight: 0.25
    breakingChangesWeight: 0.20

git:
  branchBase: main
  detectAffectedPackages: true
  requireCleanWorkingDirectory: false
  tagOnRelease: true
  pushTags: false
```

---

### Configuration Management

```bash
# View current configuration
workspace config show

# View as JSON
workspace config show --format json

# View specific section
workspace config show --section changeset

# Validate configuration
workspace config validate

# Use custom config file
workspace --config custom.yaml bump --dry-run

# Override with environment variables
SUBLIME_MAX_CONCURRENT=10 workspace upgrade detect
```

---

### Directory Structure

After initialization, your project will have:

```
your-project/
├── .changesets/              # Active changesets (MUST be versioned)
│   ├── feature-new-api.yaml
│   └── fix-bug.yaml
├── .changesets/history/      # Archived changesets (MUST be versioned)
│   └── 2024-01-15-release.yaml
├── .workspace-backups/       # Upgrade backups (MUST NOT be versioned)
│   └── backup_20240115_103045/
├── repo.config.yaml          # Configuration file (MUST be versioned)
└── packages/                 # Your workspace packages
    ├── core/
    └── utils/
```

**Important Git Rules:**
- `.changesets/` **MUST be versioned** in Git
- `.changesets/history/` **MUST be versioned** in Git
- `.workspace-backups/` **MUST NOT be versioned** (add to `.gitignore`)

**Recommended `.gitignore` additions:**
```gitignore
# Workspace Tools
.workspace-backups/
```

---

## Command Reference

### Global Options

All global options apply to **ALL** commands and control behavior across the entire application.

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--root <PATH>` | `-r` | Project root directory | Current directory |
| `--log-level <LEVEL>` | `-l` | Log level (stderr) | `info` |
| `--format <FORMAT>` | `-f` | Output format (stdout) | `human` |
| `--no-color` | | Disable colored output | `false` |
| `--config <PATH>` | `-c` | Config file path | Auto-detect |

#### Log Levels

Controls verbosity of logs written to **stderr**:

- `silent` - No logs
- `error` - Only critical errors
- `warn` - Errors + warnings
- `info` - General progress (default)
- `debug` - Detailed operations
- `trace` - Very verbose debugging

#### Output Formats

Controls format of output written to **stdout**:

- `human` - Human-readable with colors and tables (default)
- `json` - Pretty-printed JSON
- `json-compact` - Compact JSON (single line)
- `quiet` - Minimal output

#### Stream Separation

**Critical principle:** Logs and output use separate streams:

- **stderr** - Logs only (controlled by `--log-level`)
- **stdout** - Command output only (controlled by `--format`)

This ensures JSON output is never mixed with logs:

```bash
# Clean JSON output, no logs
workspace --format json --log-level silent bump --dry-run

# JSON output with debug logs (separate streams)
workspace --format json --log-level debug bump --dry-run \
  > output.json 2> logs.txt
```

### Commands Overview

- `workspace init` - Initialize project configuration
- `workspace config` - Manage configuration
- `workspace changeset` - Manage changesets
- `workspace bump` - Bump package versions
- `workspace upgrade` - Manage dependency upgrades
- `workspace audit` - Run project health audit
- `workspace changes` - Analyze repository changes
- `workspace version` - Display version information

---

### workspace init

Initialize project configuration.

**Usage:**
```bash
workspace init [OPTIONS]
```

**Options:**
- `--changeset-path <PATH>` - Changeset directory (default: `.changesets`)
- `--environments <LIST>` - Comma-separated environments
- `--default-env <LIST>` - Default environments
- `--strategy <STRATEGY>` - Versioning strategy (`independent` | `unified`)
- `--registry <URL>` - NPM registry URL
- `--config-format <FORMAT>` - Config format (`json` | `toml` | `yaml`)
- `--force` - Overwrite existing config
- `--non-interactive` - No prompts, use defaults/flags

**Examples:**
```bash
# Interactive mode
workspace init

# Non-interactive with options
workspace init --non-interactive --strategy unified --config-format yaml

# Force re-initialization
workspace init --force
```

---

### workspace config

Manage configuration.

**Subcommands:**
- `workspace config show` - Display current configuration
- `workspace config validate` - Validate configuration file

**Examples:**
```bash
# Show configuration
workspace config show

# Show as JSON
workspace config show --format json

# Validate configuration
workspace config validate
```

---

### workspace changeset

Manage changesets for version control.

**Subcommands:**
- `create` - Create a new changeset
- `update` - Update existing changeset
- `list` - List all changesets
- `show` - Show changeset details
- `edit` - Edit changeset in editor
- `delete` - Delete a changeset
- `history` - Query archived changesets
- `check` - Check if changeset exists

#### workspace changeset create

Create a new changeset for the current branch.

**Usage:**
```bash
workspace changeset create [OPTIONS]
```

**Options:**
- `--bump <TYPE>` - Bump type (`major` | `minor` | `patch`)
- `--env <LIST>` - Comma-separated environments
- `--branch <NAME>` - Branch name (default: current branch)
- `--message <TEXT>` - Changeset message
- `--packages <LIST>` - Comma-separated packages
- `--non-interactive` - No prompts

**Examples:**
```bash
# Interactive mode
workspace changeset create

# Non-interactive
workspace changeset create --bump minor --env production,staging

# With message
workspace changeset create --message "Add new authentication system"
```

#### workspace changeset update

Update an existing changeset.

**Usage:**
```bash
workspace changeset update [ID] [OPTIONS]
```

**Arguments:**
- `[ID]` - Changeset ID or branch name (optional, default: current branch)

**Options:**
- `--commit <HASH>` - Add specific commit
- `--packages <LIST>` - Add packages
- `--bump <TYPE>` - Update bump type
- `--env <LIST>` - Add environments

**Examples:**
```bash
# Update current branch's changeset
workspace changeset update

# Update specific changeset
workspace changeset update feature/my-feature

# Add packages
workspace changeset update --packages "@myorg/core,@myorg/utils"
```

#### workspace changeset list

List all active changesets.

**Usage:**
```bash
workspace changeset list [OPTIONS]
```

**Options:**
- `--filter-package <NAME>` - Filter by package
- `--filter-bump <TYPE>` - Filter by bump type
- `--filter-env <ENV>` - Filter by environment
- `--sort <FIELD>` - Sort by (`date` | `bump` | `branch`)

**Examples:**
```bash
# List all changesets
workspace changeset list

# Filter by bump type
workspace changeset list --filter-bump major

# JSON output
workspace changeset list --format json
```

#### workspace changeset show

Show detailed information for a specific changeset.

**Usage:**
```bash
workspace changeset show <BRANCH>
```

**Examples:**
```bash
# Show changeset details
workspace changeset show feature/new-api

# JSON output
workspace changeset show feature/new-api --format json
```

#### workspace changeset delete

Delete a changeset.

**Usage:**
```bash
workspace changeset delete <BRANCH> [OPTIONS]
```

**Options:**
- `--force` - Skip confirmation

**Examples:**
```bash
# Delete with confirmation
workspace changeset delete old-feature

# Force delete
workspace changeset delete old-feature --force
```

#### workspace changeset history

Query archived changesets.

**Usage:**
```bash
workspace changeset history [OPTIONS]
```

**Options:**
- `--package <NAME>` - Filter by package
- `--since <DATE>` - Since date (ISO 8601)
- `--until <DATE>` - Until date (ISO 8601)
- `--env <ENV>` - Filter by environment
- `--bump <TYPE>` - Filter by bump type
- `--limit <N>` - Limit results

**Examples:**
```bash
# View all history
workspace changeset history

# Filter by package and date
workspace changeset history --package @myorg/core --since 2024-01-01
```

---

### workspace bump

Bump package versions based on changesets.

**Usage:**
```bash
workspace bump [OPTIONS]
```

**Options:**
- `--dry-run` - Preview changes without applying
- `--execute` - Apply version changes
- `--snapshot` - Generate snapshot versions
- `--snapshot-format <FORMAT>` - Snapshot format template
- `--prerelease <TAG>` - Pre-release tag (`alpha` | `beta` | `rc`)
- `--packages <LIST>` - Only bump specific packages
- `--git-tag` - Create Git tags
- `--git-push` - Push Git tags (requires `--git-tag`)
- `--git-commit` - Commit version changes
- `--no-changelog` - Skip changelog updates
- `--no-archive` - Keep changesets active
- `--force` - Skip confirmations
- `--show-diff` - Show detailed version diffs

**Examples:**
```bash
# Preview version bump
workspace bump --dry-run

# Apply version bump
workspace bump --execute

# Full CI/CD workflow
workspace bump --execute --git-commit --git-tag --git-push --force

# Generate snapshot versions
workspace bump --snapshot --execute

# Create pre-release
workspace bump --prerelease beta --execute
```

---

### workspace upgrade

Manage dependency upgrades.

**Subcommands:**
- `check` - Check for available upgrades
- `apply` - Apply dependency upgrades
- `backups` - Manage upgrade backups

#### workspace upgrade check

Check for available dependency upgrades.

**Usage:**
```bash
workspace upgrade check [OPTIONS]
```

**Options:**
- `--major` / `--no-major` - Include/exclude major upgrades
- `--minor` / `--no-minor` - Include/exclude minor upgrades
- `--patch` / `--no-patch` - Include/exclude patch upgrades
- `--dev` - Include dev dependencies
- `--peer` - Include peer dependencies
- `--packages <LIST>` - Only check specific packages
- `--registry <URL>` - Override registry URL

**Examples:**
```bash
# Check all upgrades
workspace upgrade check

# Only patch upgrades
workspace upgrade check --no-major --no-minor

# Specific packages
workspace upgrade check --packages "typescript,eslint"
```

#### workspace upgrade apply

Apply dependency upgrades.

**Usage:**
```bash
workspace upgrade apply [OPTIONS]
```

**Options:**
- `--dry-run` - Preview without applying
- `--patch-only` - Only apply patch upgrades
- `--minor-and-patch` - Only non-breaking upgrades
- `--packages <LIST>` - Only upgrade specific packages
- `--auto-changeset` - Automatically create changeset
- `--changeset-bump <TYPE>` - Changeset bump type
- `--no-backup` - Skip backup creation
- `--force` - Skip confirmations

**Examples:**
```bash
# Apply all safe upgrades
workspace upgrade apply --minor-and-patch

# Apply with auto-changeset
workspace upgrade apply --patch-only --auto-changeset

# Specific packages
workspace upgrade apply --packages "@types/node,typescript"
```

#### workspace upgrade backups

Manage upgrade backups.

**Subcommands:**
- `list` - List all backups
- `restore <ID>` - Restore a backup
- `clean` - Clean old backups

**Examples:**
```bash
# List backups
workspace upgrade backups list

# Restore backup
workspace upgrade backups restore backup_20240115_103045

# Clean old backups (keep last 5)
workspace upgrade backups clean --keep 5
```

---

### workspace audit

Run comprehensive project health audit.

**Usage:**
```bash
workspace audit [OPTIONS]
```

**Options:**
- `--sections <LIST>` - Sections to audit (default: `all`)
- `--output <PATH>` - Write to file
- `--min-severity <LEVEL>` - Minimum severity (`critical` | `high` | `medium` | `low` | `info`)
- `--verbosity <LEVEL>` - Detail level (`minimal` | `normal` | `detailed`)
- `--no-health-score` - Skip health score calculation
- `--export <FORMAT>` - Export format (`html` | `markdown`)
- `--export-file <PATH>` - Export file path

**Sections:**
- `all` - All audit sections (default)
- `upgrades` - Available upgrades
- `dependencies` - Dependency health
- `version-consistency` - Version consistency
- `breaking-changes` - Breaking changes

**Examples:**
```bash
# Full audit
workspace audit

# Specific sections
workspace audit --sections upgrades,dependencies

# Export as markdown
workspace audit --export markdown --export-file report.md

# Critical issues only
workspace audit --min-severity critical
```

---

### workspace changes

Analyze changes in the repository.

**Usage:**
```bash
workspace changes [OPTIONS]
```

**Options:**
- `--since <REF>` - Since commit/branch/tag
- `--until <REF>` - Until commit/branch/tag
- `--branch <NAME>` - Compare against branch
- `--staged` - Only staged changes
- `--unstaged` - Only unstaged changes
- `--packages <LIST>` - Filter by packages

**Examples:**
```bash
# Analyze working directory
workspace changes

# Changes since last tag
workspace changes --since $(git describe --tags --abbrev=0)

# Compare branches
workspace changes --branch main

# Only staged changes
workspace changes --staged
```

---

### workspace version

Display version information.

**Usage:**
```bash
workspace version [OPTIONS]
workspace --version
workspace -V
```

**Options:**
- `--verbose` - Show detailed version info

**Examples:**
```bash
# Simple version
workspace --version

# Detailed version info
workspace version --verbose

# JSON output
workspace version --format json
```

---

## Workflows

### Basic Workflow

Standard feature development workflow:

```bash
# 1. Create feature branch
git checkout -b feature/new-component

# 2. Create changeset
workspace changeset create
# Select: minor bump, production environment

# 3. Make changes and commit
# Edit files...
git add .
git commit -m "feat: add new component"

# 4. Update changeset (tracks commit)
workspace changeset update

# 5. Continue development...
# More commits...

# 6. Preview version bump
workspace bump --dry-run

# 7. Create PR and merge to main
# PR merged...

# 8. On main branch, bump versions
git checkout main
git pull
workspace bump --execute --git-commit --git-tag --git-push
```

### Advanced Workflow

Advanced workflow with multiple packages and environments:

```bash
# 1. Create feature branch
git checkout -b feature/api-refactor

# 2. Create changeset with specific packages
workspace changeset create \
  --bump major \
  --env staging,production \
  --packages "@myorg/core,@myorg/api" \
  --message "Refactor API for better performance"

# 3. Development cycle
while developing; do
  # Make changes
  git add .
  git commit -m "refactor: improve API performance"

  # Update changeset
  workspace changeset update
done

# 4. Check affected packages
workspace changes --staged

# 5. Run audit before release
workspace audit

# 6. Preview bump with diff
workspace bump --dry-run --show-diff

# 7. Generate snapshot for testing
workspace bump --snapshot --execute

# 8. Test snapshot versions...

# 9. Create PR and review

# 10. After merge, release
workspace bump --execute \
  --git-commit \
  --git-tag \
  --git-push \
  --force
```

### Hotfix Workflow

Emergency bug fix workflow:

```bash
# 1. Create hotfix branch from main
git checkout main
git checkout -b hotfix/security-fix

# 2. Create patch changeset
workspace changeset create --bump patch --env production

# 3. Fix the issue
# Edit files...
git commit -m "fix: patch security vulnerability"
workspace changeset update

# 4. Test fix

# 5. Preview and apply
workspace bump --dry-run
workspace bump --execute --git-tag

# 6. Merge back to main
git checkout main
git merge hotfix/security-fix
git push --tags
```

### Dependency Upgrade Workflow

Safe dependency upgrade process:

```bash
# 1. Check for upgrades
workspace upgrade check

# 2. Review available upgrades
workspace upgrade check --format json > upgrades.json

# 3. Apply patch upgrades safely
workspace upgrade apply --patch-only --auto-changeset

# 4. Test changes
npm test

# 5. Preview version bump
workspace bump --dry-run

# 6. Commit and release
workspace bump --execute --git-commit --git-tag
```

---

## CI/CD Integration

### GitHub Actions

#### Complete Release Workflow

`.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    branches: [main]

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'

      - name: Install workspace
        run: |
          curl -fsSL https://install.workspace.dev | sh
          echo "$HOME/.local/bin" >> $GITHUB_PATH

      - name: Install dependencies
        run: npm ci

      - name: Check for changesets
        id: check
        run: |
          COUNT=$(workspace changeset list --format json | jq '.total')
          echo "count=$COUNT" >> $GITHUB_OUTPUT

      - name: Preview version bump
        if: steps.check.outputs.count > 0
        run: workspace bump --dry-run

      - name: Run tests
        run: npm test

      - name: Bump versions and create tags
        if: steps.check.outputs.count > 0
        run: |
          workspace bump \
            --execute \
            --git-commit \
            --git-tag \
            --git-push \
            --force \
            --format json > bump-result.json
          cat bump-result.json

      - name: Publish packages
        if: steps.check.outputs.count > 0
        run: npm publish --workspaces
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}

      - name: Create GitHub Release
        if: steps.check.outputs.count > 0
        uses: softprops/action-gh-release@v1
        with:
          tag_name: ${{ fromJson(steps.bump.outputs.tags)[0] }}
          generate_release_notes: true
```

#### PR Changeset Check

`.github/workflows/pr-check.yml`:

```yaml
name: PR Changeset Check

on:
  pull_request:
    branches: [main]

jobs:
  check-changeset:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install workspace
        run: |
          curl -fsSL https://install.workspace.dev | sh
          echo "$HOME/.local/bin" >> $GITHUB_PATH

      - name: Check for changeset
        run: |
          if ! workspace changeset check --format json | jq -e '.exists'; then
            echo "::error::No changeset found for this PR"
            echo "Please run: workspace changeset create"
            exit 1
          fi
```

#### Audit Check

`.github/workflows/audit.yml`:

```yaml
name: Health Audit

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly
  workflow_dispatch:

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install workspace
        run: |
          curl -fsSL https://install.workspace.dev | sh
          echo "$HOME/.local/bin" >> $GITHUB_PATH

      - name: Run audit
        run: |
          workspace audit \
            --export markdown \
            --export-file audit-report.md

      - name: Upload audit report
        uses: actions/upload-artifact@v3
        with:
          name: audit-report
          path: audit-report.md

      - name: Check for critical issues
        run: |
          workspace audit \
            --format json \
            --min-severity critical | \
          jq -e '.summary.critical == 0'
```

### GitLab CI

`.gitlab-ci.yml`:

```yaml
stages:
  - check
  - test
  - release

variables:
  WORKSPACE_VERSION: "0.1.0"

before_script:
  - curl -fsSL https://install.workspace.dev | sh
  - export PATH="$HOME/.local/bin:$PATH"

check-changeset:
  stage: check
  script:
    - workspace changeset check

test:
  stage: test
  script:
    - npm ci
    - npm test

release:
  stage: release
  only:
    - main
  script:
    - workspace bump --dry-run
    - workspace bump --execute --git-commit --git-tag --git-push --force
    - npm publish --workspaces
```

### Jenkins Pipeline

`Jenkinsfile`:

```groovy
pipeline {
    agent any

    environment {
        PATH = "${env.HOME}/.local/bin:${env.PATH}"
    }

    stages {
        stage('Setup') {
            steps {
                sh 'curl -fsSL https://install.workspace.dev | sh'
            }
        }

        stage('Check Changesets') {
            steps {
                script {
                    def result = sh(
                        script: 'workspace changeset list --format json',
                        returnStdout: true
                    ).trim()
                    def json = readJSON text: result
                    env.HAS_CHANGESETS = json.total > 0
                }
            }
        }

        stage('Preview Bump') {
            when {
                expression { env.HAS_CHANGESETS == 'true' }
            }
            steps {
                sh 'workspace bump --dry-run'
            }
        }

        stage('Test') {
            steps {
                sh 'npm ci'
                sh 'npm test'
            }
        }

        stage('Release') {
            when {
                branch 'main'
                expression { env.HAS_CHANGESETS == 'true' }
            }
            steps {
                sh '''
                    workspace bump \
                        --execute \
                        --git-commit \
                        --git-tag \
                        --git-push \
                        --force
                '''
                sh 'npm publish --workspaces'
            }
        }
    }
}
```

---

## Best Practices

### Changeset Management

**DO:**
- ✅ Create changesets early in feature development
- ✅ Update changesets after each significant commit
- ✅ Use descriptive changeset messages
- ✅ Include all affected packages
- ✅ Choose appropriate bump types
- ✅ Target correct environments

**DON'T:**
- ❌ Create changesets on main/master branch
- ❌ Mix unrelated changes in one changeset
- ❌ Skip changeset creation for feature branches
- ❌ Manually edit changeset files (use CLI)
- ❌ Commit changesets without reviewing them

### Version Bumping

**Strategy Selection:**

Use **Independent** when:
- Packages evolve at different rates
- Want to minimize version churn
- Packages are loosely coupled
- Need package-specific versioning

Use **Unified** when:
- All packages are tightly coupled
- Want coordinated releases
- Simplify version management
- All packages should move together

**Bump Type Guidelines:**

- **Patch (0.0.X):** Bug fixes, documentation updates, minor tweaks
- **Minor (0.X.0):** New features, backward-compatible changes
- **Major (X.0.0):** Breaking changes, API modifications

### Git Integration

**Recommended Git Hooks:**

**post-commit hook** (`.git/hooks/post-commit`):
```bash
#!/bin/sh
# Auto-update changeset after commit

# Skip during rebase/merge
if [ -f .git/MERGE_HEAD ] || [ -d .git/rebase-merge ]; then
    exit 0
fi

# Update changeset if exists
workspace changeset update --log-level error 2>/dev/null || true
```

**pre-push hook** (`.git/hooks/pre-push`):
```bash
#!/bin/sh
# Validate changeset before push

current_branch=$(git rev-parse --abbrev-ref HEAD)

# Skip for main branches
if [ "$current_branch" = "main" ] || [ "$current_branch" = "master" ]; then
    exit 0
fi

# Check changeset exists
if ! workspace changeset check --log-level error 2>/dev/null; then
    echo "⚠️  No changeset found for: $current_branch"
    echo "Run: workspace changeset create"
    read -p "Push anyway? (y/N) " -n 1 -r
    echo
    [[ ! $REPLY =~ ^[Yy]$ ]] && exit 1
fi
```

### Performance Tips

**Fast Operations:**
- Use `--log-level silent` to disable logging
- Use `--format json` for machine parsing
- Avoid `--verbose` flags when not needed
- Cache configuration file location

**Slow Operations:**
- `workspace audit` - Run periodically, not on every commit
- `workspace upgrade check` - Run weekly or on-demand
- `workspace bump --dry-run` - Fast, safe to run frequently

### Team Collaboration

**Communication:**
- Document changeset workflow in team README
- Review changesets in PRs
- Use changeset messages to explain impact
- Share version bump previews before releases

**Code Review:**
- Review changeset files in PRs
- Verify correct packages are listed
- Confirm bump type is appropriate
- Check target environments

**Release Process:**
- Designate release manager
- Use CI/CD for consistent releases
- Create release notes from changesets
- Monitor release process

---

## Troubleshooting

### Common Issues

#### Issue: "Config file not found"

**Symptom:**
```
Error: Configuration file not found
```

**Solution:**
```bash
# Initialize configuration
workspace init

# Or specify config file
workspace --config path/to/config.toml <command>
```

---

#### Issue: "No changeset found for current branch"

**Symptom:**
```
Error: No changeset found for branch: feature/my-feature
```

**Solution:**
```bash
# Create a changeset first
workspace changeset create

# Or specify branch explicitly
workspace changeset update feature/my-feature
```

---

#### Issue: "Package not found in workspace"

**Symptom:**
```
Error: Package '@myorg/unknown' not found in workspace
```

**Solution:**
```bash
# List available packages
workspace changes

# Check workspace configuration
workspace config show
```

---

#### Issue: "Git repository not found"

**Symptom:**
```
Error: Not a git repository
```

**Solution:**
```bash
# Ensure you're in a git repository
git init

# Or specify correct root directory
workspace --root /path/to/repo <command>
```

---

#### Issue: "Permission denied" when creating tags

**Symptom:**
```
Error: Permission denied (publickey)
```

**Solution:**
```bash
# Configure SSH key
ssh-add ~/.ssh/id_rsa

# Or use HTTPS with token
git config credential.helper store

# Or use --no-git-push and push manually
workspace bump --execute --git-tag
git push --tags
```

---

#### Issue: Slow changeset operations

**Symptom:**
Operations take > 5 seconds

**Solution:**
```bash
# Disable logging
workspace --log-level silent changeset update

# Check for large file scans
git status --ignored

# Optimize Git operations
git gc
```

---

#### Issue: JSON output mixed with text

**Symptom:**
Invalid JSON when using `--format json`

**Solution:**
```bash
# Use silent log level for clean JSON
workspace --format json --log-level silent bump --dry-run

# Separate streams
workspace --format json bump --dry-run > output.json 2> logs.txt
```

---

### Debugging

**Enable Debug Logging:**
```bash
# Debug level
workspace --log-level debug <command>

# Trace level (very verbose)
workspace --log-level trace <command>
```

**Capture Output:**
```bash
# Save output and logs separately
workspace --format json <command> > output.json 2> debug.log

# Save everything together
workspace <command> &> combined.log
```

**Verify Configuration:**
```bash
# Validate config
workspace config validate

# Show effective config
workspace config show --format json
```

**Check Git State:**
```bash
# Verify Git status
git status

# Check branch
git branch

# View recent commits
git log --oneline -10
```

---

## FAQ

### General

**Q: What Node.js versions are supported?**
A: Node.js 16+ is recommended. The tool works with npm, yarn, pnpm, and bun workspaces.

**Q: Can I use this in a non-monorepo project?**
A: Yes! Workspace Tools supports both single-package and monorepo projects.

**Q: Does this replace semantic-release?**
A: Not directly. Workspace Tools focuses on changeset-based versioning with team collaboration features. It can complement or replace semantic-release depending on your workflow.

**Q: Is Windows supported?**
A: Yes, via WSL (Windows Subsystem for Linux). Native Windows support is planned.

### Changesets

**Q: Can I have multiple changesets per branch?**
A: No, one changeset per branch is recommended for clarity and consistency.

**Q: What happens to changesets after release?**
A: They're archived to `.changesets/history/` with release metadata for audit purposes.

**Q: Can I manually edit changeset files?**
A: While possible, use CLI commands (`update`, `edit`) to avoid format errors.

**Q: How do I delete a changeset?**
A: Use `workspace changeset delete <branch>`. Add `--force` to skip confirmation.

### Versioning

**Q: Which strategy should I use?**
A: Use **Independent** for loosely coupled packages, **Unified** for tightly coupled ones.

**Q: Can I change strategies later?**
A: Yes, but you'll need to manually synchronize versions if switching to Unified.

**Q: What are snapshot versions?**
A: Temporary versions for testing (e.g., `1.2.3-feature-branch.abc123`). Not published to registry.

**Q: How do pre-release versions work?**
A: Use `--prerelease alpha|beta|rc` to create versions like `1.2.0-alpha.0`.

### Upgrades

**Q: Are upgrades automatically applied?**
A: No, you must explicitly run `workspace upgrade apply`. Use `--dry-run` to preview.

**Q: Can I rollback upgrades?**
A: Yes, backups are created automatically. Use `workspace upgrade backups restore <id>`.

**Q: How often should I check for upgrades?**
A: Weekly checks are recommended. Use `--patch-only` for frequent safe updates.

### CI/CD

**Q: How do I prevent releases without changesets?**
A: Add a check in CI: `workspace changeset list --format json | jq -e '.total > 0'`

**Q: Can I run this in parallel?**
A: Most commands are safe to run in parallel except `bump --execute` which should be serialized.

**Q: How do I handle merge conflicts in changesets?**
A: Resolve conflicts in changeset YAML files manually or recreate the changeset.

### Audit

**Q: How is the health score calculated?**
A: Weighted average across audit sections (upgrades, dependencies, version consistency, breaking changes).

**Q: Can I customize audit rules?**
A: Currently no, but this is planned for a future release.

**Q: What severity levels exist?**
A: `critical`, `high`, `medium`, `low`, `info`

---

## Migration Guide

### From Changesets

If you're migrating from `@changesets/cli`:

**Similarities:**
- Both use changeset files
- Both support monorepos
- Both track version bumps

**Differences:**
- Workspace Tools adds environment targeting
- Different file format (YAML vs Markdown)
- Built-in audit and upgrade features

**Migration Steps:**

1. **Install Workspace Tools:**
```bash
curl -fsSL https://install.workspace.dev | sh
```

2. **Initialize Configuration:**
```bash
workspace init
```

3. **Convert Existing Changesets** (manual):

Old format (`.changeset/happy-pandas-fly.md`):
```markdown
---
"@myorg/core": minor
---

Add new feature
```

New format (`.changesets/feature-new-feature.yaml`):
```yaml
branch: feature/new-feature
bump: minor
packages:
  - "@myorg/core"
environments:
  - production
message: "Add new feature"
commits: []
```

4. **Update CI/CD:**

Replace:
```bash
npx changeset version
npx changeset publish
```

With:
```bash
workspace bump --execute --git-tag
npm publish --workspaces
```

5. **Update Scripts in package.json:**

```json
{
  "scripts": {
    "changeset": "workspace changeset create",
    "version": "workspace bump --execute",
    "publish": "workspace bump --execute --git-tag && npm publish --workspaces"
  }
}
```

---

### From Lerna

If you're migrating from Lerna:

**Migration Steps:**

1. **Install Workspace Tools:**
```bash
npm uninstall lerna
curl -fsSL https://install.workspace.dev | sh
```

2. **Initialize:**
```bash
workspace init --strategy unified  # If using fixed mode
# or
workspace init --strategy independent  # If using independent mode
```

3. **Replace Commands:**

| Lerna | Workspace Tools |
|-------|----------------|
| `lerna version` | `workspace bump --execute` |
| `lerna publish` | `workspace bump --execute --git-tag` + `npm publish` |
| `lerna changed` | `workspace changes` |
| `lerna diff` | `workspace changes` |

4. **Update CI/CD:**

Replace:
```bash
lerna version --conventional-commits --yes
lerna publish from-package --yes
```

With:
```bash
workspace bump --execute --git-tag --force
npm publish --workspaces
```

---

### From Manual Versioning

If you're currently versioning manually:

1. **Install and Initialize:**
```bash
curl -fsSL https://install.workspace.dev | sh
workspace init
```

2. **Establish Workflow:**

Before (manual):
```bash
# Edit package.json manually
npm version minor
git commit -am "chore: release v1.2.0"
git tag v1.2.0
npm publish
```

After (with Workspace Tools):
```bash
# Create feature branch
git checkout -b feature/new-feature

# Create changeset
workspace changeset create --bump minor

# Develop and commit
git commit -am "feat: add new feature"
workspace changeset update

# Preview
workspace bump --dry-run

# Release
workspace bump --execute --git-tag
npm publish
```

3. **Set Up Git Hooks** (optional but recommended):

```bash
# .git/hooks/post-commit
workspace changeset update --log-level error || true
```

4. **Document for Team:**
Create a `WORKFLOW.md` documenting the new process for your team.

---

## Additional Resources

### Links

- **Documentation:** https://workspace.dev/docs
- **GitHub:** https://github.com/org/workspace-node-tools
- **Issues:** https://github.com/org/workspace-node-tools/issues
- **Changelog:** https://github.com/org/workspace-node-tools/blob/main/CHANGELOG.md

### Community

- **Discord:** https://discord.gg/workspace-tools
- **Stack Overflow:** Tag `workspace-tools`

### Related Tools

- **@changesets/cli** - Original inspiration
- **Lerna** - Monorepo management
- **semantic-release** - Automated versioning
- **Rush** - Scalable monorepo manager

---

## Appendix

### Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Configuration error |
| 4 | Git error |
| 5 | File system error |
| 10 | Validation failed |
| 11 | Changeset error |
| 12 | Version resolution error |
| 13 | Upgrade error |
| 14 | Audit error |

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `NO_COLOR` | Disable colored output | `false` |
| `EDITOR` | Editor for `changeset edit` | `vim` / `nano` |
| `WORKSPACE_CONFIG` | Config file path | Auto-detect |
| `WORKSPACE_LOG_LEVEL` | Default log level | `info` |

### File Formats

**Changeset File (YAML):**
```yaml
id: "feature-new-api"
branch: "feature/new-api"
bump: "minor"
packages:
  - "@myorg/core"
  - "@myorg/api"
environments:
  - "staging"
  - "production"
message: "Add new REST API endpoints"
commits:
  - "abc123def456"
  - "def456ghi789"
createdAt: "2024-01-15T10:00:00Z"
updatedAt: "2024-01-15T14:30:00Z"
```

---

**Last Updated:** 2025-11-07
**Version:** 0.1.0
**License:** MIT
