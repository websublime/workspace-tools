# Configuration Examples

This directory contains example configuration files and Rust programs demonstrating how to use the `sublime_pkg_tools` configuration system.

## TOML Configuration Examples

### Basic Configuration

[`basic-config.toml`](./basic-config.toml) - Complete configuration file with all default values and detailed comments explaining each option. Use this as a reference when creating your own configuration.

**Use case**: Understanding all available configuration options and their defaults.

```bash
# Copy to your project root
cp examples/basic-config.toml package-tools.toml
# Or use with custom path
cp examples/basic-config.toml .sublime/package-tools.toml
```

### Minimal Configuration

[`minimal-config.toml`](./minimal-config.toml) - Minimal configuration showing that you only need to specify settings that differ from defaults. Most projects can start with this.

**Use case**: Quick start with minimal configuration overhead.

```bash
# Start with minimal config
cp examples/minimal-config.toml package-tools.toml
# Then add only what you need to customize
```

### Monorepo Configuration

[`monorepo-config.toml`](./monorepo-config.toml) - Advanced configuration for monorepo projects with multiple packages, showing features like:
- Unified versioning strategy
- Multiple deployment environments
- Scoped registry configuration
- Per-package and root changelogs
- Comprehensive audit settings

**Use case**: Complex monorepo with multiple packages and deployment pipelines.

```bash
# For monorepo projects
cp examples/monorepo-config.toml package-tools.toml
```

## Rust Code Examples

### Loading Configuration

[`load_config.rs`](./load_config.rs) - Demonstrates four different methods for loading configuration:

1. Using the convenience `load_config()` function
2. Using `ConfigManager` with builder pattern
3. Loading from specific file paths
4. Creating configuration programmatically

**Run the example:**

```bash
cargo run --example load_config
```

### Audit Report Formatting

[`audit_report_formatting.rs`](./audit_report_formatting.rs) - Demonstrates how to format audit reports in multiple formats (Markdown, JSON) with different verbosity levels and formatting options.

**Features demonstrated:**
- Creating audit reports with sample data
- Formatting as Markdown (default, minimal, detailed)
- Exporting as JSON (pretty and compact)
- Querying report data (issues, health score, status)
- Using custom formatting options
- Filtering issues by severity

**Run the example:**

```bash
cargo run --example audit_report_formatting
```

**Output examples:**

```bash
# Default markdown format
cargo run --example audit_report_formatting | grep -A 50 "Default Markdown"

# JSON export
cargo run --example audit_report_formatting | grep -A 20 "JSON Format"

# Report queries
cargo run --example audit_report_formatting | grep -A 10 "Report Query"
```

### Environment Variable Overrides

[`env_override.rs`](./env_override.rs) - Shows how to override configuration settings using environment variables.

**Run without overrides:**

```bash
cargo run --example env_override
```

**Run with environment variable overrides:**

```bash
SUBLIME_PKG_VERSION_STRATEGY=unified \
SUBLIME_PKG_CHANGESET_PATH=".custom-changesets" \
SUBLIME_PKG_AUDIT_MIN_SEVERITY=info \
cargo run --example env_override
```

**Common environment variables:**

```bash
# Changeset settings
export SUBLIME_PKG_CHANGESET_PATH=".changesets"
export SUBLIME_PKG_CHANGESET_HISTORY_PATH=".changesets/history"

# Version settings
export SUBLIME_PKG_VERSION_STRATEGY="unified"
export SUBLIME_PKG_VERSION_DEFAULT_BUMP="minor"

# Registry settings
export SUBLIME_PKG_UPGRADE_REGISTRY_DEFAULT_REGISTRY="https://registry.npmjs.org"
export SUBLIME_PKG_UPGRADE_REGISTRY_TIMEOUT_SECS="60"

# Changelog settings
export SUBLIME_PKG_CHANGELOG_FORMAT="conventional"
SUBLIME_PKG_CHANGELOG_REPOSITORY_URL="https://github.com/org/repo"

# Audit settings
export SUBLIME_PKG_AUDIT_MIN_SEVERITY="warning"
export SUBLIME_PKG_AUDIT_SECTIONS_UPGRADES="true"
```

## Quick Start

### For a Single Package Project

1. **No configuration needed** - defaults work out of the box:

```rust
use sublime_pkg_tools::config::PackageToolsConfig;

let config = PackageToolsConfig::default();
```

2. **Or create minimal config file:**

```bash
cp examples/minimal-config.toml package-tools.toml
# Edit to set your repository URL
```

### For a Monorepo Project

1. **Copy monorepo configuration:**

```bash
cp examples/monorepo-config.toml package-tools.toml
```

2. **Customize for your project:**

```toml
[package_tools.version]
# Choose versioning strategy
strategy = "independent"  # or "unified"

[package_tools.changelog]
# Set your repository URL
repository_url = "https://github.com/your-org/your-repo"
```

### For CI/CD Pipelines

Use environment variables to override settings per environment:

```yaml
# GitHub Actions example
env:
  SUBLIME_PKG_VERSION_STRATEGY: "unified"
  SUBLIME_PKG_AUDIT_MIN_SEVERITY: "warning"
  SUBLIME_PKG_CHANGELOG_REPOSITORY_URL: ${{ github.repository_url }}
```

## Configuration Sections

All examples demonstrate these configuration sections:

- **Changeset** - Where changesets are stored and what environments are available
- **Version** - Versioning strategy (independent vs unified)
- **Dependency** - How version changes propagate through dependency graphs
- **Upgrade** - Settings for detecting and applying external dependency upgrades
- **Changelog** - Changelog generation format and behavior
- **Git** - Git commit message templates
- **Audit** - Dependency audits and health checks
- **Report Formatting** - How to generate and format audit reports

## Testing Your Configuration

All examples include validation. You can test your configuration:

```rust
use sublime_pkg_tools::config::PackageToolsConfig;
use sublime_standard_tools::config::Configurable;

let config = PackageToolsConfig::default();

// Validate returns Result<(), ConfigError>
match config.validate() {
    Ok(_) => println!("✓ Configuration is valid"),
    Err(e) => eprintln!("✗ Configuration error: {}", e),
}
```

## Common Patterns

### Pattern 1: Load with Fallback to Defaults

```rust
use sublime_pkg_tools::config::{load_config, ConfigLoader};

// Try to load config, fall back to defaults
let config = load_config()
    .await
    .unwrap_or_else(|_| async { ConfigLoader::load_defaults().await.unwrap() });
```

### Pattern 2: Override Specific Settings

```rust
use sublime_pkg_tools::config::PackageToolsConfig;

let mut config = PackageToolsConfig::default();
config.version.default_bump = "minor".to_string();
config.changeset.path = ".custom-changesets".to_string();
```

### Pattern 3: Environment-Specific Configuration

```rust
use sublime_pkg_tools::config::load_config_from_file;

// Determine config file based on environment
let config_file = if std::env::var("ENV").unwrap_or_default() == "production" {
    "config/production.toml"
} else {
    "config/development.toml"
};

let config = load_config_from_file(config_file).await?;
```

## Further Reading

- [Configuration Guide](../docs/guides/configuration.md) - Comprehensive configuration documentation
- [API Documentation](../docs/api/) - Detailed API reference for all configuration types
- [CONCEPT.md](../CONCEPT.md) - High-level design and concepts
- [PLAN.md](../PLAN.md) - Implementation plan and module structure

## Troubleshooting

### Configuration File Not Found

If you get a "configuration file not found" error:

1. Check that the file exists in the expected location
2. Use an absolute path or ensure your working directory is correct
3. Use `load_config()` instead which looks in default locations
4. Use `ConfigLoader::load_from_files()` for multiple optional files

```rust
use sublime_pkg_tools::config::{load_config, ConfigLoader};

// Option 1: Load from default locations (won't fail if files missing)
let config = load_config().await?;

// Option 2: Load from multiple files (skips missing files)
let config = ConfigLoader::load_from_files(vec![
    "package-tools.toml",
    ".sublime/package-tools.toml",
]).await?;
```

### Environment Variables Not Working

Ensure you:

1. Use the correct prefix (default: `SUBLIME_PKG_`)
2. Use uppercase with underscores
3. Nested fields use the full path separated by underscores

```bash
# Correct
export SUBLIME_PKG_VERSION_STRATEGY="unified"

# Incorrect
export PKG_TOOLS_VERSION_STRATEGY="unified"  # Wrong prefix
export sublime_pkg_version_strategy="unified"  # Not uppercase
export SUBLIMEPKG_VERSION_STRATEGY="unified"  # Missing underscore
```

### Report Formatting

For audit reports, you can customize the output format:

```rust
use sublime_pkg_tools::audit::{FormatOptions, Verbosity, AuditReportExt};

// Minimal output (summary only)
let options = FormatOptions::default()
    .with_verbosity(Verbosity::Minimal);
let markdown = report.to_markdown_with_options(&options);

// Detailed output with metadata
let options = FormatOptions::default()
    .with_verbosity(Verbosity::Detailed)
    .with_metadata(true);
let markdown = report.to_markdown_with_options(&options);

// JSON export
let json = report.to_json()?;
std::fs::write("audit-report.json", json)?;
```

### Validation Errors

If validation fails:

1. Check error message for specific field
2. Verify enum values are correct (e.g., "independent" not "Independent")
3. Ensure paths are valid
4. Check that default environments are subset of available environments

## Support

For more help:

- Check the [Configuration Guide](../docs/guides/configuration.md)
- Review [TOML documentation](https://toml.io/)
- See the API documentation for configuration structs
- Open an issue with your configuration file and error message