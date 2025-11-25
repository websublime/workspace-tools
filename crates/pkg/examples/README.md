# Configuration Examples

This directory contains example configuration files and Rust programs demonstrating how to use the `sublime_pkg_tools` configuration system.

## Architecture Note

The `sublime_pkg_tools` crate is **agnostic to file discovery**. The crate provides:

- `PackageToolsConfig::default()` - Default configuration
- `PackageToolsConfig::from_str(content, format)` - Parse configuration from content

File discovery, reading, and environment variable overrides are the responsibility of the calling application (typically the CLI). This design enables flexibility for library users who may want to implement their own configuration loading strategies.

## TOML Configuration Examples

### Basic Configuration

[`basic-config.toml`](./basic-config.toml) - Complete configuration file with all default values and detailed comments explaining each option. Use this as a reference when creating your own configuration.

**Use case**: Understanding all available configuration options and their defaults.

```bash
# Copy to your project root
cp examples/basic-config.toml repo.config.toml
```

### Minimal Configuration

[`minimal-config.toml`](./minimal-config.toml) - Minimal configuration showing that you only need to specify settings that differ from defaults. Most projects can start with this.

**Use case**: Quick start with minimal configuration overhead.

```bash
# Start with minimal config
cp examples/minimal-config.toml repo.config.toml
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
cp examples/monorepo-config.toml repo.config.toml
```

## Rust Code Examples

### Parsing Configuration

[`load_config.rs`](./load_config.rs) - Demonstrates different methods for working with configuration:

1. Using `PackageToolsConfig::default()` for default configuration
2. Using `PackageToolsConfig::from_str()` to parse TOML content
3. Using `PackageToolsConfig::from_str()` to parse JSON content
4. Using `PackageToolsConfig::from_str()` to parse YAML content
5. Creating configuration programmatically

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

### Changeset Storage

[`changeset_storage.rs`](./changeset_storage.rs) - Demonstrates changeset storage operations.

**Run the example:**

```bash
cargo run --example changeset_storage
```

### Snapshot Versions

[`snapshot_version.rs`](./snapshot_version.rs) - Demonstrates snapshot version generation.

**Run the example:**

```bash
cargo run --example snapshot_version
```

## Quick Start

### For a Single Package Project

1. **No configuration needed** - defaults work out of the box:

```rust
use sublime_pkg_tools::config::PackageToolsConfig;

let config = PackageToolsConfig::default();
```

2. **Or parse from config content:**

```rust
use sublime_pkg_tools::config::{PackageToolsConfig, ConfigFormat};

let toml_content = r#"
[changeset]
path = ".changesets"

[version]
default_bump = "minor"
"#;

let config = PackageToolsConfig::from_str(toml_content, ConfigFormat::Toml)?;
```

### For a Monorepo Project

1. **Copy monorepo configuration:**

```bash
cp examples/monorepo-config.toml repo.config.toml
```

2. **Customize for your project:**

```toml
[changeset]
path = ".changesets"

[version]
# Choose versioning strategy
strategy = "independent"  # or "unified"

[changelog]
# Set your repository URL
repository_url = "https://github.com/your-org/your-repo"
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
    Ok(_) => println!("Configuration is valid"),
    Err(e) => eprintln!("Configuration error: {}", e),
}
```

## Common Patterns

### Pattern 1: Parse from String Content

```rust
use sublime_pkg_tools::config::{PackageToolsConfig, ConfigFormat};

// Parse TOML
let config = PackageToolsConfig::from_str(toml_content, ConfigFormat::Toml)?;

// Parse JSON
let config = PackageToolsConfig::from_str(json_content, ConfigFormat::Json)?;

// Parse YAML
let config = PackageToolsConfig::from_str(yaml_content, ConfigFormat::Yaml)?;
```

### Pattern 2: Override Specific Settings

```rust
use sublime_pkg_tools::config::PackageToolsConfig;

let mut config = PackageToolsConfig::default();
config.version.default_bump = "minor".to_string();
config.changeset.path = ".custom-changesets".to_string();
```

### Pattern 3: Partial Configuration with Defaults

Configuration uses `#[serde(default)]`, so you only need to specify fields you want to override:

```toml
# Only override what you need - everything else uses defaults
[version]
default_bump = "minor"

[changelog]
include_authors = true
```

### Pattern 4: CLI Integration Example

If you're building a CLI that uses this library, you would handle file discovery:

```rust
use sublime_pkg_tools::config::{PackageToolsConfig, ConfigFormat};
use std::fs;

// CLI discovers and reads the config file
let content = fs::read_to_string("repo.config.toml")?;

// Parse using the library
let config = PackageToolsConfig::from_str(&content, ConfigFormat::Toml)?;
```

## Configuration Formats

The library supports three configuration formats:

### TOML (Recommended)

```toml
[changeset]
path = ".changesets"

[version]
strategy = "independent"
default_bump = "patch"
```

### JSON

```json
{
    "changeset": {
        "path": ".changesets"
    },
    "version": {
        "strategy": "independent",
        "default_bump": "patch"
    }
}
```

### YAML

```yaml
changeset:
  path: ".changesets"

version:
  strategy: independent
  default_bump: patch
```

## Further Reading

- [SPEC.md](../SPEC.md) - Detailed specification for the pkg crate
- [CONCEPT.md](../CONCEPT.md) - High-level design and concepts
- [PLAN.md](../PLAN.md) - Implementation plan and module structure

## Troubleshooting

### Validation Errors

If validation fails:

1. Check error message for specific field
2. Verify enum values are correct (e.g., "independent" not "Independent")
3. Ensure paths are valid
4. Check that default environments are subset of available environments

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

## Support

For more help:

- Review [TOML documentation](https://toml.io/)
- See the API documentation for configuration structs
- Open an issue with your configuration file and error message
