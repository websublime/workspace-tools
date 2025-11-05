# Configuration Guide

## Table of Contents

- [Overview](#overview)
- [Getting Started](#getting-started)
- [Configuration File Location](#configuration-file-location)
- [Configuration Sections](#configuration-sections)
  - [Changeset Configuration](#changeset-configuration)
  - [Version Configuration](#version-configuration)
  - [Dependency Configuration](#dependency-configuration)
  - [Upgrade Configuration](#upgrade-configuration)
  - [Changelog Configuration](#changelog-configuration)
  - [Git Configuration](#git-configuration)
  - [Audit Configuration](#audit-configuration)
- [Environment Variables](#environment-variables)
- [Loading Configuration](#loading-configuration)
- [Configuration Validation](#configuration-validation)
- [Common Scenarios](#common-scenarios)
- [Migration Guide](#migration-guide)
- [Troubleshooting](#troubleshooting)

## Overview

The `sublime_pkg_tools` library uses a comprehensive configuration system that integrates with the `sublime_standard_tools` configuration framework. Configuration can be loaded from:

1. **TOML files** - Primary configuration method
2. **Environment variables** - Override specific settings
3. **Programmatic defaults** - Fallback when no configuration is provided

All configuration is optional - the library ships with sensible defaults that work for most projects.

## Getting Started

### Minimal Configuration

For most projects, you can start with no configuration at all:

```rust
use sublime_pkg_tools::config::PackageToolsConfig;

let config = PackageToolsConfig::default();
```

This provides all default settings suitable for a single-package project with standard npm registry.

### Basic TOML File

Create a `package-tools.toml` file in your project root:

```toml
[package_tools.changeset]
path = ".changesets"

[package_tools.version]
strategy = "independent"

[package_tools.changelog]
repository_url = "https://github.com/org/repo"
```

### Loading Configuration

```rust
use sublime_pkg_tools::config::{PackageToolsConfig, load_config};
use sublime_standard_tools::config::ConfigManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Option 1: Load from default locations with env override
    let config = load_config().await?;
    
    // Option 2: Load defaults only
    let config = ConfigLoader::load_defaults().await?;
    
    // Option 3: Load from specific file
    let config = load_config_from_file("custom-config.toml").await?;
    
    Ok(())
}
```

## Configuration File Location

The library looks for configuration files in the following locations (in order of precedence):

1. Path specified programmatically
2. `package-tools.toml` in current directory
3. `.config/package-tools.toml` in current directory
4. `package-tools.toml` in workspace root (for monorepos)

You can also specify a custom path:

```rust
let config = load_config_from_file("/path/to/config.toml").await?;
```

## Configuration Sections

### Changeset Configuration

Controls where changesets are stored and what environments are available.

```toml
[package_tools.changeset]
# Directory for active changesets
path = ".changesets"

# Directory for archived changesets (release history)
history_path = ".changesets/history"

# Available environments for deployment targeting
available_environments = ["development", "staging", "production"]

# Default environments when none specified
default_environments = ["production"]
```

**Fields:**

- `path` (String): Directory where active changesets are stored
  - Default: `".changesets"`
  - Use relative paths from workspace root

- `history_path` (String): Directory for archived changesets
  - Default: `".changesets/history"`
  - Contains release history with metadata

- `available_environments` (Array<String>): Valid environment names
  - Default: `["production"]`
  - Used for deployment targeting
  - Common values: `development`, `staging`, `production`, `qa`, `preview`

- `default_environments` (Array<String>): Environments to use when unspecified
  - Default: `["production"]`
  - Must be subset of `available_environments`

**Example: Multi-environment Setup**

```toml
[package_tools.changeset]
path = ".changesets"
history_path = ".changesets/history"
available_environments = ["dev", "staging", "prod"]
default_environments = ["staging", "prod"]
```

### Version Configuration

Defines versioning strategy and version resolution behavior.

```toml
[package_tools.version]
# Versioning strategy: "independent" or "unified"
strategy = "independent"

# Default bump type: "major", "minor", "patch", "none"
default_bump = "patch"

# Snapshot version format
snapshot_format = "{version}-{branch}.{timestamp}"
```

**Fields:**

- `strategy` (VersioningStrategy): How packages are versioned
  - `"independent"`: Each package has its own version (default)
  - `"unified"`: All packages share the same version
  - Choose `independent` for loosely coupled packages
  - Choose `unified` for tightly coupled monorepo packages

- `default_bump` (String): Default version increment
  - Values: `"major"`, `"minor"`, `"patch"`, `"none"`
  - Default: `"patch"`
  - Used when changeset doesn't specify a bump type

- `snapshot_format` (String): Template for snapshot versions
  - Default: `"{version}-{branch}.{timestamp}"`
  - Placeholders:
    - `{version}`: Base version number
    - `{branch}`: Git branch name (sanitized)
    - `{timestamp}`: Unix timestamp
    - `{short_hash}`: Short git commit hash
  - Example output: `1.2.3-feature.1234567890`

**Example: Unified Versioning**

```toml
[package_tools.version]
strategy = "unified"
default_bump = "minor"
snapshot_format = "{version}-snapshot.{timestamp}"
```

### Dependency Configuration

Controls how version changes propagate through dependency graphs.

```toml
[package_tools.dependency]
# Bump type for propagated changes
propagation_bump = "patch"

# Which dependency types to propagate
propagate_dependencies = true
propagate_dev_dependencies = false
propagate_peer_dependencies = true

# Safety limits
max_depth = 10
fail_on_circular = true

# Protocol skipping
skip_workspace_protocol = true
skip_file_protocol = true
skip_link_protocol = true
skip_portal_protocol = true
```

**Fields:**

- `propagation_bump` (String): Bump type when propagating changes
  - Values: `"major"`, `"minor"`, `"patch"`, `"none"`
  - Default: `"patch"`
  - Used when a dependency is updated due to propagation

- `propagate_dependencies` (Boolean): Propagate to regular dependencies
  - Default: `true`
  - Controls updates to `dependencies` field

- `propagate_dev_dependencies` (Boolean): Propagate to devDependencies
  - Default: `false`
  - Usually disabled as dev dependencies don't affect consumers

- `propagate_peer_dependencies` (Boolean): Propagate to peerDependencies
  - Default: `true`
  - Important for maintaining peer dependency compatibility

- `max_depth` (Integer): Maximum propagation depth
  - Default: `10`
  - Prevents infinite loops in complex graphs
  - Increase for deep dependency trees

- `fail_on_circular` (Boolean): Fail if circular dependencies detected
  - Default: `true`
  - Set to `false` to only warn about circular dependencies

- `skip_workspace_protocol` (Boolean): Skip `workspace:*` version specs
  - Default: `true`
  - Workspace protocol handled by package manager

- `skip_file_protocol` (Boolean): Skip `file:` version specs
  - Default: `true`

- `skip_link_protocol` (Boolean): Skip `link:` version specs
  - Default: `true`

- `skip_portal_protocol` (Boolean): Skip `portal:` version specs
  - Default: `true`

**Example: Aggressive Propagation**

```toml
[package_tools.dependency]
propagation_bump = "minor"
propagate_dependencies = true
propagate_dev_dependencies = true
propagate_peer_dependencies = true
max_depth = 20
fail_on_circular = true
```

### Upgrade Configuration

Settings for detecting and applying external dependency upgrades.

```toml
[package_tools.upgrade]
auto_changeset = true
changeset_bump = "patch"

[package_tools.upgrade.registry]
default_registry = "https://registry.npmjs.org"
timeout_secs = 30
retry_attempts = 3
retry_delay_ms = 1000
read_npmrc = true

[package_tools.upgrade.registry.scoped]
"@myorg" = "https://npm.pkg.github.com"

[package_tools.upgrade.backup]
enabled = true
backup_dir = ".wnt-backups"
keep_after_success = false
max_backups = 5
```

**Fields:**

- `auto_changeset` (Boolean): Auto-create changeset for upgrades
  - Default: `true`
  - Creates changeset automatically when upgrades are applied

- `changeset_bump` (String): Bump type for auto-created changeset
  - Default: `"patch"`
  - Values: `"major"`, `"minor"`, `"patch"`

**Registry Configuration:**

- `default_registry` (String): Default npm registry URL
  - Default: `"https://registry.npmjs.org"`

- `timeout_secs` (Integer): Request timeout in seconds
  - Default: `30`

- `retry_attempts` (Integer): Number of retry attempts
  - Default: `3`

- `retry_delay_ms` (Integer): Delay between retries in milliseconds
  - Default: `1000`

- `read_npmrc` (Boolean): Read and respect `.npmrc` configuration
  - Default: `true`
  - Reads authentication tokens and registry overrides

- `scoped` (Map<String, String>): Scoped registry mappings
  - Maps scope to registry URL
  - Example: `"@myorg" = "https://npm.pkg.github.com"`

**Backup Configuration:**

- `enabled` (Boolean): Create backups before upgrades
  - Default: `true`

- `backup_dir` (String): Backup storage directory
  - Default: `".wnt-backups"`

- `keep_after_success` (Boolean): Keep backups after successful upgrade
  - Default: `false`

- `max_backups` (Integer): Maximum backups to retain
  - Default: `5`

**Example: Private Registry Setup**

```toml
[package_tools.upgrade.registry]
default_registry = "https://registry.npmjs.org"
timeout_secs = 60
read_npmrc = true

[package_tools.upgrade.registry.scoped]
"@myorg" = "https://npm.pkg.github.com"
"@internal" = "https://registry.internal.company.com"
```

### Changelog Configuration

Controls changelog generation format and behavior.

```toml
[package_tools.changelog]
enabled = true
format = "keep-a-changelog"
filename = "CHANGELOG.md"
include_commit_links = true
include_issue_links = true
include_authors = false
repository_url = "https://github.com/org/repo"
monorepo_mode = "per-package"
version_tag_format = "{package}@{version}"
root_tag_format = "v{version}"

[package_tools.changelog.conventional]
enabled = true

[package_tools.changelog.conventional.sections]
feat = "Features"
fix = "Bug Fixes"
breaking = "BREAKING CHANGES"

[package_tools.changelog.exclude]
patterns = ["**/node_modules/**"]
authors = []
```

**Fields:**

- `enabled` (Boolean): Enable changelog generation
  - Default: `true`

- `format` (ChangelogFormat): Changelog format
  - Values: `"keep-a-changelog"`, `"conventional"`, `"custom"`
  - Default: `"keep-a-changelog"`

- `filename` (String): Changelog filename
  - Default: `"CHANGELOG.md"`

- `include_commit_links` (Boolean): Include commit links
  - Default: `true`
  - Requires `repository_url`

- `include_issue_links` (Boolean): Include issue/PR links
  - Default: `true`
  - Requires `repository_url`

- `include_authors` (Boolean): Include author names
  - Default: `false`

- `repository_url` (String): Repository URL for links
  - Optional
  - Example: `"https://github.com/org/repo"`

- `monorepo_mode` (MonorepoMode): Monorepo changelog strategy
  - Values: `"per-package"`, `"root"`, `"both"`
  - Default: `"per-package"`

- `version_tag_format` (String): Git tag format for versions
  - Default: `"{package}@{version}"`
  - Placeholders: `{package}`, `{version}`

- `root_tag_format` (String): Git tag format for root versions
  - Default: `"v{version}"`

**Conventional Commits:**

- `enabled` (Boolean): Parse conventional commits
  - Default: `true`

- `sections` (Map<String, String>): Commit type to section mapping
  - Maps commit type (e.g., `feat`) to section name (e.g., `Features`)

- `breaking` (String): Section name for breaking changes
  - Default: `"BREAKING CHANGES"`

**Exclude Configuration:**

- `patterns` (Array<String>): File patterns to exclude
  - Default: `["**/node_modules/**", "**/dist/**", "**/.git/**"]`

- `authors` (Array<String>): Author emails to exclude
  - Default: `[]`
  - Useful for excluding bot commits

**Example: Conventional Commits Format**

```toml
[package_tools.changelog]
enabled = true
format = "conventional"
repository_url = "https://github.com/org/repo"
include_commit_links = true
include_authors = true

[package_tools.changelog.conventional]
enabled = true

[package_tools.changelog.conventional.sections]
feat = "✨ Features"
fix = "🐛 Bug Fixes"
perf = "⚡ Performance"
docs = "📚 Documentation"
breaking = "💥 BREAKING CHANGES"
```

### Git Configuration

Templates for git commit messages and warnings.

```toml
[package_tools.git]
merge_commit_template = """
Release {package} v{version}

{changelog}
"""

monorepo_merge_commit_template = """
Release multiple packages

{summary}
"""

include_breaking_warning = true

breaking_warning_template = """
⚠️ BREAKING CHANGES: This release contains {count} breaking change(s).
"""
```

**Fields:**

- `merge_commit_template` (String): Template for single package releases
  - Placeholders: `{version}`, `{package}`, `{bump}`, `{environments}`, `{changelog}`

- `monorepo_merge_commit_template` (String): Template for multi-package releases
  - Placeholders: `{packages}`, `{summary}`

- `include_breaking_warning` (Boolean): Include breaking change warnings
  - Default: `true`

- `breaking_warning_template` (String): Warning message template
  - Placeholder: `{count}`

**Example: Conventional Commit Format**

```toml
[package_tools.git]
merge_commit_template = """
chore(release): publish {package} v{version}

{changelog}

Environments: {environments}
"""

include_breaking_warning = true
```

### Audit Configuration

Settings for dependency audits and health checks.

```toml
[package_tools.audit]
enabled = true
min_severity = "warning"

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
check_missing = true
check_unused = true
check_version_conflicts = true

[package_tools.audit.breaking_changes]
check_conventional_commits = true
check_changelog = true

[package_tools.audit.version_consistency]
fail_on_inconsistency = false
warn_on_inconsistency = true
```

**Fields:**

- `enabled` (Boolean): Enable audit functionality
  - Default: `true`

- `min_severity` (String): Minimum severity to report
  - Values: `"critical"`, `"warning"`, `"info"`
  - Default: `"warning"`

**Sections Configuration:**

- `upgrades` (Boolean): Audit available upgrades
- `dependencies` (Boolean): Audit dependency issues
- `breaking_changes` (Boolean): Audit breaking changes
- `categorization` (Boolean): Categorize dependencies
- `version_consistency` (Boolean): Check version consistency

**Upgrades Audit:**

- `include_patch` (Boolean): Include patch upgrades
- `include_minor` (Boolean): Include minor upgrades
- `include_major` (Boolean): Include major upgrades
- `deprecated_as_critical` (Boolean): Treat deprecated packages as critical

**Dependencies Audit:**

- `check_circular` (Boolean): Check for circular dependencies
- `check_missing` (Boolean): Check for missing dependencies
- `check_unused` (Boolean): Check for unused dependencies
- `check_version_conflicts` (Boolean): Check for version conflicts

**Breaking Changes Audit:**

- `check_conventional_commits` (Boolean): Check commits for breaking changes
- `check_changelog` (Boolean): Check changelog for breaking changes

**Version Consistency Audit:**

- `fail_on_inconsistency` (Boolean): Fail on version inconsistencies
- `warn_on_inconsistency` (Boolean): Warn about version inconsistencies

## Environment Variables

Configuration values can be overridden using environment variables with a configured prefix (default: `PKG_TOOLS`).

### Format

Environment variables use the format: `PREFIX_SECTION_FIELD`

- All uppercase
- Sections separated by underscores
- Nested fields use double underscores

### Examples

```bash
# Changeset configuration
export SUBLIME_PKG_CHANGESET_PATH=".custom-changesets"
export SUBLIME_PKG_CHANGESET_HISTORY_PATH=".custom-history"

# Version configuration
export SUBLIME_PKG_VERSION_STRATEGY="unified"
export SUBLIME_PKG_VERSION_DEFAULT_BUMP="minor"

# Dependency configuration
export SUBLIME_PKG_DEPENDENCY_PROPAGATION_BUMP="minor"
export SUBLIME_PKG_DEPENDENCY_MAX_DEPTH="20"

# Upgrade configuration
export SUBLIME_PKG_UPGRADE_AUTO_CHANGESET="true"
export SUBLIME_PKG_UPGRADE_REGISTRY_DEFAULT_REGISTRY="https://custom-registry.com"
export SUBLIME_PKG_UPGRADE_REGISTRY_TIMEOUT_SECS="60"

# Changelog configuration
export SUBLIME_PKG_CHANGELOG_ENABLED="true"
export SUBLIME_PKG_CHANGELOG_FORMAT="conventional"
export SUBLIME_PKG_CHANGELOG_REPOSITORY_URL="https://github.com/org/repo"

# Audit configuration
export SUBLIME_PKG_AUDIT_ENABLED="true"
export SUBLIME_PKG_AUDIT_MIN_SEVERITY="info"
```

### CI/CD Integration

Environment variables are particularly useful in CI/CD environments:

```yaml
# GitHub Actions example
env:
  SUBLIME_PKG_CHANGESET_PATH: ".changesets"
  SUBLIME_PKG_VERSION_STRATEGY: "unified"
  SUBLIME_PKG_UPGRADE_AUTO_CHANGESET: "true"
```

## Loading Configuration

### Using ConfigManager

The recommended way to load configuration:

```rust
use sublime_pkg_tools::config::{PackageToolsConfig, load_config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load with defaults, optional files, and env overrides
    let config = load_config().await?;
    
    println!("Changeset path: {}", config.changeset.path);
    println!("Version strategy: {:?}", config.version.strategy);
    
    Ok(())
}
```

### Manual Configuration

For more control:

```rust
use sublime_pkg_tools::config::{ConfigLoader, PackageToolsConfig};
use sublime_standard_tools::config::Configurable;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigLoader::load_from_file("package-tools.toml").await?;
    
    // Validate configuration
    config.validate()?;
    
    Ok(())
}
```

### Programmatic Configuration

Build configuration programmatically:

```rust
use sublime_pkg_tools::config::{
    PackageToolsConfig, VersionConfig, VersioningStrategy
};

let mut config = PackageToolsConfig::default();
config.version.strategy = VersioningStrategy::Unified;
config.version.default_bump = "minor".to_string();
config.changeset.path = ".custom-changesets".to_string();

// Validate before use
config.validate()?;
```

## Configuration Validation

All configuration is validated before use.

### Validation Rules

1. **Path formats**: Paths must be valid relative or absolute paths
2. **Enum values**: Strategy and format values must be valid enum variants
3. **Environment lists**: Default environments must be subset of available
4. **Numeric ranges**: Timeouts, retries, etc. must be positive
5. **URL formats**: Repository and registry URLs must be valid

### Validation Example

```rust
use sublime_pkg_tools::config::PackageToolsConfig;
use sublime_standard_tools::config::Configurable;

let config = PackageToolsConfig::default();

match config.validate() {
    Ok(_) => println!("Configuration is valid"),
    Err(e) => eprintln!("Configuration error: {}", e),
}
```

### Common Validation Errors

**Invalid Strategy**
```toml
[package_tools.version]
strategy = "invalid"  # Error: Must be "independent" or "unified"
```

**Invalid Default Environments**
```toml
[package_tools.changeset]
available_environments = ["production"]
default_environments = ["staging"]  # Error: Not in available_environments
```

**Invalid Bump Type**
```toml
[package_tools.version]
default_bump = "huge"  # Error: Must be "major", "minor", "patch", or "none"
```

## Common Scenarios

### Scenario 1: Single Package Project

Minimal configuration for a single npm package:

```toml
[package_tools.version]
strategy = "independent"

[package_tools.changelog]
repository_url = "https://github.com/org/package"
```

### Scenario 2: Monorepo with Unified Versioning

All packages share the same version:

```toml
[package_tools.version]
strategy = "unified"
default_bump = "minor"

[package_tools.changelog]
monorepo_mode = "both"
repository_url = "https://github.com/org/monorepo"
```

### Scenario 3: Monorepo with Independent Versioning

Each package has its own version:

```toml
[package_tools.version]
strategy = "independent"

[package_tools.dependency]
propagation_bump = "patch"
propagate_dependencies = true

[package_tools.changelog]
monorepo_mode = "per-package"
```

### Scenario 4: Private Registry

Using private npm registry:

```toml
[package_tools.upgrade.registry]
default_registry = "https://npm.pkg.github.com"
read_npmrc = true

[package_tools.upgrade.registry.scoped]
"@myorg" = "https://npm.pkg.github.com"
```

### Scenario 5: CI/CD Release Pipeline

Configuration optimized for automated releases:

```toml
[package_tools.changeset]
path = ".changesets"
available_environments = ["staging", "production"]

[package_tools.version]
strategy = "unified"

[package_tools.upgrade]
auto_changeset = true

[package_tools.audit]
enabled = true
min_severity = "warning"

[package_tools.audit.version_consistency]
fail_on_inconsistency = true
```

### Scenario 6: Development Workflow

Configuration for development with multiple environments:

```toml
[package_tools.changeset]
available_environments = ["dev", "qa", "staging", "prod"]
default_environments = ["dev"]

[package_tools.version]
snapshot_format = "{version}-dev.{timestamp}"

[package_tools.changelog]
include_authors = true
```

## Migration Guide

### From changesets

If migrating from `@changesets/cli`:

**Before (changesets):**
```json
{
  "changelog": "@changesets/cli/changelog",
  "commit": false,
  "linked": [],
  "access": "restricted",
  "baseBranch": "main"
}
```

**After (sublime_pkg_tools):**
```toml
[package_tools.version]
strategy = "independent"

[package_tools.changelog]
enabled = true
format = "keep-a-changelog"
```

### From Lerna

If migrating from Lerna:

**Before (lerna.json):**
```json
{
  "version": "independent",
  "npmClient": "npm",
  "command": {
    "publish": {
      "conventionalCommits": true
    }
  }
}
```

**After (sublime_pkg_tools):**
```toml
[package_tools.version]
strategy = "independent"

[package_tools.changelog]
format = "conventional"

[package_tools.changelog.conventional]
enabled = true
```

## Troubleshooting

### Configuration Not Loading

**Problem**: Configuration file not being read

**Solution**: Check file location and permissions

```bash
# Verify file exists
ls -la package-tools.toml

# Check from correct directory
pwd

# Try absolute path
export PKG_TOOLS_CONFIG_PATH=/absolute/path/to/config.toml
```

### Environment Variables Not Working

**Problem**: Environment variable overrides not applying

**Solution**: Verify variable names and format

```bash
# Use correct prefix and format
export SUBLIME_PKG_VERSION_STRATEGY="unified"  # Correct
export PKG_TOOLS_VERSION_STRATEGY="unified"    # Wrong - incorrect prefix

# Print all SUBLIME_PKG variables
env | grep SUBLIME_PKG
```

### Validation Errors

**Problem**: Configuration validation fails

**Solution**: Check error message for specific issue

```rust
use sublime_pkg_tools::config::PackageToolsConfig;

let config = PackageToolsConfig::default();
if let Err(e) = config.validate() {
    eprintln!("Validation error: {}", e);
    // Check specific field causing error
}
```

### Registry Authentication

**Problem**: Cannot access private registry

**Solution**: Ensure `.npmrc` is configured correctly

```bash
# Check .npmrc exists and has token
cat .npmrc

# Verify read_npmrc is enabled
grep "read_npmrc" package-tools.toml

# Test registry access
curl -H "Authorization: Bearer $TOKEN" https://npm.pkg.github.com/@myorg/package
```

### Circular Dependencies

**Problem**: Circular dependency detected

**Solution**: Review dependency graph or disable strict checking

```toml
[package_tools.dependency]
# Only warn instead of failing
fail_on_circular = false
```

Or investigate the circular dependency:

```rust
use sublime_pkg_tools::version::DependencyGraph;

// Build graph and detect cycles
let graph = DependencyGraph::from_packages(&packages)?;
let cycles = graph.detect_cycles();

for cycle in cycles {
    println!("Circular dependency: {:?}", cycle);
}
```

## Best Practices

### 1. Use Version Control

Always commit your configuration file:

```bash
git add package-tools.toml
git commit -m "Add package tools configuration"
```

### 2. Document Custom Settings

Add comments explaining non-standard configuration:

```toml
[package_tools.version]
# Using unified versioning because all packages are tightly coupled
strategy = "unified"

[package_tools.dependency]
# Higher depth needed for deep dependency tree
max_depth = 20
```

### 3. Validate in CI

Add configuration validation to CI pipeline:

```yaml
# .github/workflows/ci.yml
- name: Validate configuration
  run: cargo test config_validation
```

### 4. Use Environment-Specific Overrides

Keep base configuration in file, override per environment:

```bash
# Production
export SUBLIME_PKG_AUDIT_MIN_SEVERITY="warning"

# Development
export SUBLIME_PKG_AUDIT_MIN_SEVERITY="info"
```

### 5. Start Simple

Begin with minimal configuration and add settings as needed:

```toml
# Start with just repository URL
[package_tools.changelog]
repository_url = "https://github.com/org/repo"

# Add more settings as project evolves
```

## Further Reading

- [API Documentation](../api/config.md) - Detailed API reference
- [Examples](../../examples/) - Complete configuration examples
- [Standard Tools Configuration](https://github.com/sublime-tools/standard-tools) - Underlying config system
- [Story 2.1](../STORY_2.1_SUMMARY.md) - Configuration structure design
- [Story 2.2](../STORY_2.2_SUMMARY.md) - Configuration loading implementation

## Support

For issues or questions:

1. Check [troubleshooting section](#troubleshooting)
2. Review [examples](../../examples/)
3. See API documentation for configuration types
4. Open an issue on GitHub with your configuration file