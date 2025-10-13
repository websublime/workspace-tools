# Configuration Guide - sublime_pkg_tools

This guide provides comprehensive documentation for configuring `sublime_pkg_tools` to suit your project's needs.

## Table of Contents

- [Overview](#overview)
- [Configuration Sources](#configuration-sources)
- [Configuration Structure](#configuration-structure)
- [Environment Variables](#environment-variables)
- [Configuration Examples](#configuration-examples)
- [Validation](#validation)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Overview

The `sublime_pkg_tools` configuration system is built on top of `sublime_standard_tools` and provides a flexible, hierarchical configuration management system that supports:

- Multiple configuration file formats (TOML, JSON, YAML)
- Environment variable overrides
- Project-specific and user-specific configuration
- Comprehensive validation
- Default values for all settings

## Configuration Sources

Configuration is loaded from multiple sources in the following priority order (highest to lowest):

1. **Environment Variables** (highest priority)
2. **Project Configuration Files** (`repo.config.{toml,json,yaml,yml}`)
3. **User Configuration Files** (`~/.config/sublime/config.{toml,json,yaml,yml}`)
4. **Default Values** (lowest priority)

### Configuration File Locations

#### Project Configuration
- `./repo.config.toml`
- `./repo.config.json`
- `./repo.config.yaml`
- `./repo.config.yml`

#### User Configuration
- `~/.config/sublime/config.toml` (Linux/macOS)
- `%APPDATA%\sublime\config.toml` (Windows)
- Similar for `.json`, `.yaml`, `.yml` extensions

## Configuration Structure

All package tools configuration is nested under the `[package_tools]` section:

```toml
[package_tools]

[package_tools.changeset]
# Changeset management settings

[package_tools.version]
# Version management settings

[package_tools.registry]
# NPM registry settings

[package_tools.release]
# Release management settings

[package_tools.dependency]
# Dependency analysis settings

[package_tools.conventional]
# Conventional commit settings

[package_tools.changelog]
# Changelog generation settings
```

### Changeset Configuration

Controls how changesets are created, stored, and managed.

```toml
[package_tools.changeset]
path = ".changesets"                    # Where changesets are stored
history_path = ".changesets/history"    # Where applied changesets are archived
available_environments = [              # Available deployment environments
    "dev",
    "test", 
    "qa",
    "staging",
    "prod"
]
default_environments = ["dev"]          # Default environments for new changesets
filename_format = "{branch}-{datetime}.json"  # Changeset filename pattern
max_pending_changesets = 100           # Maximum pending changesets (optional)
auto_archive_applied = true            # Auto-move applied changesets to history
```

### Version Configuration

Controls version resolution and snapshot generation.

```toml
[package_tools.version]
snapshot_format = "{version}-{commit}.snapshot"  # Snapshot version format
commit_hash_length = 7                           # Length of commit hash in snapshots
allow_snapshot_on_main = false                   # Allow snapshots on main branch
prerelease_format = "alpha.{number}"             # Pre-release format (optional)
build_metadata_format = "{timestamp}"            # Build metadata format (optional)
```

### Registry Configuration

Controls NPM registry interactions and authentication.

```toml
[package_tools.registry]
url = "https://registry.npmjs.org"     # Default registry URL
timeout = 30                           # Request timeout in seconds
retry_attempts = 3                     # Number of retry attempts
use_npmrc = true                       # Use .npmrc for authentication
default_access = "public"              # Default publish access level
skip_checks_in_dry_run = true          # Skip registry checks in dry-run mode

# Custom registry configurations
[package_tools.registry.registries.enterprise]
url = "https://npm.enterprise.com"
auth_type = "token"
auth_token = "${NPM_ENTERPRISE_TOKEN}"
timeout = 60
default_access = "restricted"
```

### Release Configuration

Controls release planning, execution, and tagging.

```toml
[package_tools.release]
strategy = "independent"               # "independent" or "unified" versioning
tag_format = "{package}@{version}"     # Git tag format
env_tag_format = "{package}@{version}-{environment}"  # Environment-specific tags
create_tags = true                     # Create Git tags during release
push_tags = true                       # Push tags to remote
create_changelog = true                # Generate changelog during release
changelog_file = "CHANGELOG.md"       # Changelog filename
commit_message = "chore(release): {package}@{version}"  # Release commit message
dry_run_by_default = false            # Enable dry-run mode by default
max_concurrent_releases = 5           # Maximum concurrent package releases
release_timeout = 300                 # Release timeout in seconds
```

### Dependency Configuration

Controls dependency analysis and update propagation.

```toml
[package_tools.dependency]
propagate_updates = true               # Propagate version updates to dependents
propagate_dev_dependencies = false     # Include dev dependencies in propagation
max_propagation_depth = 10             # Maximum propagation depth
detect_circular = true                 # Detect circular dependencies
fail_on_circular = true                # Fail on circular dependencies
dependency_update_bump = "patch"       # Bump type for dependency updates
include_peer_dependencies = false      # Include peer dependencies in analysis
include_optional_dependencies = false  # Include optional dependencies in analysis
```

### Conventional Commit Configuration

Controls conventional commit parsing and version bump calculation.

```toml
[package_tools.conventional]
parse_breaking_changes = true          # Parse breaking changes from commit body
require_conventional_commits = false   # Require conventional commit format
default_bump_type = "patch"           # Default bump for unknown commit types

# Breaking change detection patterns
breaking_change_patterns = [
    "BREAKING CHANGE:",
    "BREAKING-CHANGE:"
]

# Commit type configurations
[package_tools.conventional.types.feat]
bump = "minor"                         # Version bump type
changelog = true                       # Include in changelog
changelog_title = "Features"          # Changelog section title
breaking = false                       # Indicates breaking change

[package_tools.conventional.types.fix]
bump = "patch"
changelog = true
changelog_title = "Bug Fixes"
breaking = false

[package_tools.conventional.types.perf]
bump = "patch"
changelog = true
changelog_title = "Performance Improvements"
breaking = false

[package_tools.conventional.types.breaking]
bump = "major"
changelog = true
changelog_title = "Breaking Changes"
breaking = true

[package_tools.conventional.types.docs]
bump = "none"
changelog = false
breaking = false

[package_tools.conventional.types.style]
bump = "none"
changelog = false
breaking = false

[package_tools.conventional.types.refactor]
bump = "none"
changelog = false
breaking = false

[package_tools.conventional.types.test]
bump = "none"
changelog = false
breaking = false

[package_tools.conventional.types.build]
bump = "none"
changelog = false
breaking = false

[package_tools.conventional.types.ci]
bump = "none"
changelog = false
breaking = false

[package_tools.conventional.types.chore]
bump = "none"
changelog = false
breaking = false
```

### Changelog Configuration

Controls changelog generation and formatting.

```toml
[package_tools.changelog]
include_commit_hash = true             # Include commit hashes in entries
include_authors = true                 # Include author information
group_by_type = true                   # Group changes by commit type
include_date = true                    # Include release date
max_commits_per_release = 1000         # Maximum commits per release (optional)
template_file = "templates/changelog.hbs"  # Custom template file (optional)
link_commits = false                   # Link to commits in remote repository
commit_url_format = "https://github.com/owner/repo/commit/{hash}"  # Commit URL pattern

# Custom changelog sections
[package_tools.changelog.custom_sections]
migration = "Migration Guide"
deprecation = "Deprecations"
```

## Environment Variables

All configuration options can be overridden using environment variables with the `SUBLIME_PACKAGE_TOOLS_` prefix.

### Environment Variable Format

Environment variable names follow this pattern:
```
SUBLIME_PACKAGE_TOOLS_{SECTION}_{OPTION}
```

### Common Environment Variables

#### Changeset Configuration
```bash
SUBLIME_PACKAGE_TOOLS_CHANGESET_PATH=".changesets"
SUBLIME_PACKAGE_TOOLS_CHANGESET_HISTORY_PATH=".changesets/history"
SUBLIME_PACKAGE_TOOLS_CHANGESET_AVAILABLE_ENVIRONMENTS="dev,staging,prod"
SUBLIME_PACKAGE_TOOLS_CHANGESET_DEFAULT_ENVIRONMENTS="dev"
SUBLIME_PACKAGE_TOOLS_CHANGESET_FILENAME_FORMAT="{branch}-{datetime}.json"
SUBLIME_PACKAGE_TOOLS_CHANGESET_AUTO_ARCHIVE="true"
```

#### Version Configuration
```bash
SUBLIME_PACKAGE_TOOLS_VERSION_SNAPSHOT_FORMAT="{version}-{commit}.snapshot"
SUBLIME_PACKAGE_TOOLS_VERSION_COMMIT_HASH_LENGTH="7"
SUBLIME_PACKAGE_TOOLS_VERSION_ALLOW_SNAPSHOT_ON_MAIN="false"
```

#### Registry Configuration
```bash
SUBLIME_PACKAGE_TOOLS_REGISTRY_URL="https://registry.npmjs.org"
SUBLIME_PACKAGE_TOOLS_REGISTRY_TIMEOUT="30"
SUBLIME_PACKAGE_TOOLS_REGISTRY_RETRY_ATTEMPTS="3"
SUBLIME_PACKAGE_TOOLS_REGISTRY_USE_NPMRC="true"
SUBLIME_PACKAGE_TOOLS_REGISTRY_DEFAULT_ACCESS="public"
```

#### Release Configuration
```bash
SUBLIME_PACKAGE_TOOLS_RELEASE_STRATEGY="independent"
SUBLIME_PACKAGE_TOOLS_RELEASE_TAG_FORMAT="{package}@{version}"
SUBLIME_PACKAGE_TOOLS_RELEASE_CREATE_TAGS="true"
SUBLIME_PACKAGE_TOOLS_RELEASE_PUSH_TAGS="true"
SUBLIME_PACKAGE_TOOLS_RELEASE_DRY_RUN_BY_DEFAULT="false"
```

#### Dependency Configuration
```bash
SUBLIME_PACKAGE_TOOLS_DEPENDENCY_PROPAGATE_UPDATES="true"
SUBLIME_PACKAGE_TOOLS_DEPENDENCY_PROPAGATE_DEV="false"
SUBLIME_PACKAGE_TOOLS_DEPENDENCY_MAX_DEPTH="10"
SUBLIME_PACKAGE_TOOLS_DEPENDENCY_UPDATE_BUMP="patch"
```

#### Conventional Commit Configuration
```bash
SUBLIME_PACKAGE_TOOLS_CONVENTIONAL_PARSE_BREAKING="true"
SUBLIME_PACKAGE_TOOLS_CONVENTIONAL_REQUIRE="false"
SUBLIME_PACKAGE_TOOLS_CONVENTIONAL_DEFAULT_BUMP="patch"
```

#### Changelog Configuration
```bash
SUBLIME_PACKAGE_TOOLS_CHANGELOG_INCLUDE_COMMIT_HASH="true"
SUBLIME_PACKAGE_TOOLS_CHANGELOG_INCLUDE_AUTHORS="true"
SUBLIME_PACKAGE_TOOLS_CHANGELOG_GROUP_BY_TYPE="true"
SUBLIME_PACKAGE_TOOLS_CHANGELOG_INCLUDE_DATE="true"
```

## Configuration Examples

### Example 1: Monorepo with Independent Versioning

```toml
[package_tools.release]
strategy = "independent"
create_changelog = true

[package_tools.changeset]
available_environments = ["dev", "staging", "prod"]
default_environments = ["dev"]

[package_tools.dependency]
propagate_updates = true
max_propagation_depth = 5

[package_tools.conventional]
require_conventional_commits = true
```

### Example 2: Single Package with Unified Versioning

```toml
[package_tools.release]
strategy = "unified"
tag_format = "v{version}"
dry_run_by_default = false

[package_tools.changeset]
available_environments = ["prod"]
default_environments = ["prod"]

[package_tools.registry]
default_access = "public"
use_npmrc = true
```

### Example 3: Enterprise Setup with Custom Registry

```toml
[package_tools.registry]
url = "https://npm.enterprise.com"
default_access = "restricted"
timeout = 60

[package_tools.registry.registries.public]
url = "https://registry.npmjs.org"
auth_type = "token"
default_access = "public"

[package_tools.release]
create_tags = true
push_tags = false  # Manual tag pushing
```

### Example 4: Development Environment

```toml
[package_tools.release]
dry_run_by_default = true
push_tags = false
create_changelog = false

[package_tools.changeset]
default_environments = ["dev"]
auto_archive_applied = false

[package_tools.registry]
skip_checks_in_dry_run = true
```

## Validation

The configuration system includes comprehensive validation to ensure all settings are valid and compatible.

### Validation Rules

#### Changeset Validation
- `available_environments` must not be empty
- All `default_environments` must be in `available_environments`
- `filename_format` must be a valid format string

#### Version Validation
- `commit_hash_length` must be between 1 and 40
- Format strings must contain required placeholders

#### Registry Validation
- `timeout` must be greater than 0
- `retry_attempts` must be non-negative
- `url` must be a valid URL format

#### Release Validation
- `strategy` must be "independent" or "unified"
- `max_concurrent_releases` must be greater than 0
- `release_timeout` must be greater than 0

#### Dependency Validation
- `max_propagation_depth` must be greater than 0
- `dependency_update_bump` must be "none", "patch", "minor", or "major"

#### Conventional Commit Validation
- Commit type bump values must be "none", "patch", "minor", or "major"
- `default_bump_type` must be valid bump value

### Custom Validation

You can validate configuration programmatically:

```rust
use sublime_pkg_tools::config::{PackageToolsConfig, PackageToolsConfigManager};

let manager = PackageToolsConfigManager::new();
let config = PackageToolsConfig::default();

match manager.validate_config(&config) {
    Ok(()) => println!("Configuration is valid"),
    Err(e) => eprintln!("Configuration error: {}", e),
}
```

## Best Practices

### 1. Use Project-Specific Configuration

Store project-specific settings in `repo.config.toml` in your project root:

```toml
[package_tools.changeset]
available_environments = ["dev", "staging", "prod"]

[package_tools.conventional]
require_conventional_commits = true
```

### 2. Use Environment Variables for Secrets

Never store secrets in configuration files. Use environment variables:

```bash
export SUBLIME_PACKAGE_TOOLS_REGISTRY_URL="https://npm.enterprise.com"
export NPM_TOKEN="your-secret-token"
```

### 3. Different Configs for Different Environments

#### Development
```toml
[package_tools.release]
dry_run_by_default = true
push_tags = false
```

#### Production
```toml
[package_tools.release]
dry_run_by_default = false
push_tags = true
create_changelog = true
```

### 4. Validate Configuration in CI

Add configuration validation to your CI pipeline:

```yaml
# .github/workflows/validate.yml
- name: Validate Configuration
  run: |
    cargo run --example config_examples
```

### 5. Document Custom Settings

If you use custom settings, document them in your project:

```toml
# Custom enterprise registry setup
[package_tools.registry.registries.enterprise]
url = "https://npm.enterprise.com"
auth_type = "token"
```

## Troubleshooting

### Common Issues

#### 1. Configuration Not Loading

**Problem**: Configuration changes are not being applied.

**Solutions**:
- Check file locations and naming (must be `repo.config.*`)
- Verify TOML/JSON/YAML syntax
- Check environment variable names and values
- Use `PackageToolsConfigManager::get_env_overrides()` to debug

#### 2. Validation Errors

**Problem**: Configuration validation fails.

**Solutions**:
- Check that all required fields have valid values
- Ensure arrays are not empty where required
- Verify enum values (like `strategy = "independent"`)
- Use specific error messages to identify issues

#### 3. Environment Variables Not Working

**Problem**: Environment variables are not overriding configuration.

**Solutions**:
- Verify the `SUBLIME_PACKAGE_TOOLS_` prefix
- Check variable names match the mapping (see Environment Variables section)
- Ensure values are properly formatted (strings, booleans, numbers)
- Use `manager.get_env_overrides()` to verify detection

#### 4. Registry Authentication Issues

**Problem**: Cannot authenticate with custom registry.

**Solutions**:
- Check `.npmrc` file if `use_npmrc = true`
- Verify registry URL format
- Ensure authentication tokens are properly set
- Test registry connectivity separately

### Debug Configuration Loading

```rust
use sublime_pkg_tools::config::PackageToolsConfigManager;

let manager = PackageToolsConfigManager::new();

// Check environment overrides
let overrides = manager.get_env_overrides();
println!("Environment overrides: {:?}", overrides);

// Load and validate
match manager.load_config().await {
    Ok(config) => {
        println!("Configuration loaded successfully");
        println!("Release strategy: {}", config.release.strategy);
    }
    Err(e) => {
        eprintln!("Configuration error: {}", e);
    }
}
```

### Getting Help

If you encounter issues not covered in this guide:

1. Check the example configurations in `examples/config_examples.rs`
2. Review the test cases in `src/config/tests.rs`
3. Enable debug logging to see configuration loading details
4. Validate your configuration files with online TOML/JSON/YAML validators

## Migration Guide

### From Environment-Only Configuration

If you were previously using only environment variables:

1. Create a `repo.config.toml` file
2. Move common settings to the file
3. Keep secrets and environment-specific settings as environment variables

### From Manual Configuration

If you were manually managing configuration:

1. Use the `PackageToolsConfigManager` for all configuration loading
2. Remove hardcoded configuration values
3. Add validation calls to catch issues early

### Version Updates

When updating `sublime_pkg_tools`:

1. Check this documentation for new configuration options
2. Run configuration validation after updates
3. Review deprecation warnings in logs
4. Update examples and documentation as needed