# sublime_pkg_tools

A comprehensive Rust library for managing Node.js package versioning, changesets, changelogs, and dependency upgrades in both single-package projects and monorepos.

## Features

- **Changeset Management**: Track and manage changesets for coordinated package releases
- **Version Resolution**: Intelligent version calculation with dependency propagation and prerelease support (alpha, beta, RC)
- **Dependency Upgrades**: Detect and apply external dependency upgrades automatically
- **Changelog Generation**: Generate changelogs in multiple formats (Keep a Changelog, Conventional Commits)
- **Changes Analysis**: Analyze working directory and commit ranges to identify affected packages
- **Audit & Health Checks**: Comprehensive dependency audits and health score calculation
- **Monorepo Support**: Full support for both independent and unified versioning strategies
- **Prerelease Workflow**: Full SemVer 2.0.0 prerelease support with create, increment, and promote modes
- **Flexible Configuration**: TOML-based configuration with environment variable overrides

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sublime_pkg_tools = "0.1.0"
```

## Quick Start

### Basic Usage

```rust
use sublime_pkg_tools::config::{ConfigFormat, PackageToolsConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse configuration from string content
    let toml_content = r#"
        [package_tools.changeset]
        path = ".changesets"
        
        [package_tools.version]
        strategy = "independent"
    "#;
    
    let config = PackageToolsConfig::from_str(toml_content, ConfigFormat::Toml)?;
    
    // Or use defaults
    let config = PackageToolsConfig::default();
    
    println!("Configuration loaded successfully!");
    Ok(())
}
```

> **Note:** This crate is configuration-agnostic. It does not search for or load
> configuration files from the filesystem. The CLI layer (`sublime_cli_tools`) is
> responsible for discovering and reading configuration files, then passing the
> parsed content to this crate via `PackageToolsConfig::from_str()`.

### Configuration

The library uses TOML-based configuration. Create a `package-tools.toml` file in your project root:

```toml
[package_tools.changeset]
path = ".changesets"
available_environments = ["development", "staging", "production"]

[package_tools.version]
strategy = "independent"  # or "unified" for monorepos
default_bump = "patch"

[package_tools.changelog]
enabled = true
format = "keep-a-changelog"
repository_url = "https://github.com/org/repo"
```

For a complete configuration reference, see:
- [Configuration Guide](docs/guides/configuration.md) - Comprehensive guide with all options
- [Examples](examples/) - Ready-to-use configuration examples

### Environment Variables

Override any configuration setting using environment variables:

```bash
export SUBLIME_PKG_VERSION_STRATEGY="unified"
export SUBLIME_PKG_CHANGESET_PATH=".custom-changesets"
export SUBLIME_PKG_AUDIT_MIN_SEVERITY="info"
```

## Configuration Options

The library provides comprehensive configuration for all aspects of package management:

- **Changeset**: Storage paths, history location, deployment environments
- **Version**: Versioning strategy, default bump types, snapshot formats
- **Dependency**: Propagation rules, circular dependency handling
- **Upgrade**: Registry configuration, backup settings, automatic changesets
- **Changelog**: Format selection, commit parsing, link generation
- **Git**: Commit message templates, breaking change warnings
- **Audit**: Health checks, dependency analysis, version consistency

See the [Configuration Guide](docs/guides/configuration.md) for detailed documentation.

## Examples

Check out the [`examples/`](examples/) directory for:

- **TOML Examples**:
  - `basic-config.toml` - Complete reference with all options
  - `minimal-config.toml` - Quick start with minimal settings
  - `monorepo-config.toml` - Advanced monorepo configuration

- **Code Examples**:
  - `load_config.rs` - Parsing configuration from string content

Run examples:

```bash
# Load configuration example
cargo run --example load_config
```

## Documentation

- [Configuration Guide](docs/guides/configuration.md) - Complete configuration reference
- [Concept Document](CONCEPT.md) - High-level design and architecture
- [Implementation Plan](PLAN.md) - Detailed implementation roadmap
- [Story Map](STORY_MAP.md) - Development story breakdown
- API Documentation - Run `cargo doc --open`

## Use Cases

### Single Package Project

Minimal configuration for a single npm package:

```toml
[package_tools.version]
strategy = "independent"

[package_tools.changelog]
repository_url = "https://github.com/org/package"
```

### Monorepo with Unified Versioning

All packages share the same version:

```toml
[package_tools.version]
strategy = "unified"

[package_tools.changelog]
monorepo_mode = "both"
```

### Private Registry

Using private npm registry with authentication:

```toml
[package_tools.upgrade.registry]
default_registry = "https://npm.pkg.github.com"
read_npmrc = true

[package_tools.upgrade.registry.scoped]
"@myorg" = "https://npm.pkg.github.com"
```

### Prerelease Workflow

Working with prerelease versions (alpha, beta, RC):

```rust
use sublime_pkg_tools::types::{Version, VersionBump};
use sublime_pkg_tools::types::prerelease::{PrereleaseConfig, PrereleaseMode};

// Start with stable version 1.2.3
let version = Version::parse("1.2.3")?;

// Phase 1: Create beta prerelease for new features
let beta_config = PrereleaseConfig::create("beta".to_string());
let beta_0 = version.bump_with_prerelease(VersionBump::Minor, Some(&beta_config))?;
// Result: 1.3.0-beta.0

// Phase 2: Increment beta for fixes
let increment_config = PrereleaseConfig::increment("beta".to_string());
let beta_1 = beta_0.bump_with_prerelease(VersionBump::None, Some(&increment_config))?;
// Result: 1.3.0-beta.1

// Phase 3: Create release candidate
let rc_config = PrereleaseConfig::create("rc".to_string());
let rc_0 = beta_1.bump_with_prerelease(VersionBump::None, Some(&rc_config))?;
// Result: 1.3.0-rc.0

// Phase 4: Promote to stable release
let promote_config = PrereleaseConfig::promote("rc".to_string());
let stable = rc_0.bump_with_prerelease(VersionBump::None, Some(&promote_config))?;
// Result: 1.3.0 (stable)
```

**Prerelease Modes:**
- **Create**: Generate new prerelease from stable or change prerelease tag
- **Increment**: Increment the prerelease number for the same tag
- **Promote**: Remove prerelease tag to create stable version

**SemVer Compliance:**
- Follows SemVer 2.0.0 specification
- Prerelease format: `MAJOR.MINOR.PATCH-PRERELEASE` (e.g., `1.3.0-beta.0`)
- Valid prerelease tags: ASCII alphanumerics and hyphens only (`[0-9A-Za-z-]`)
- Common conventions: `alpha`, `beta`, `rc` (release candidate)

## Architecture

The library is organized into logical modules:

- `config` - Configuration management and loading
- `changeset` - Changeset storage and management
- `version` - Version resolution and dependency propagation
- `upgrade` - External dependency upgrade detection and application
- `changelog` - Changelog generation with multiple formats
- `changes` - Working directory and commit range analysis
- `audit` - Dependency audits and health checks
- `types` - Core data types and structures
- `error` - Error types and handling

## Requirements

- Rust 1.70 or later
- Tokio async runtime
- Access to Node.js project with package.json files

## Dependencies

This crate builds on:

- `sublime_standard_tools` - File system, configuration, and Node.js abstractions
- `sublime_git_tools` - Git repository operations
- Standard async ecosystem (tokio, futures)
- Serialization (serde, serde_json, toml)
- HTTP client (reqwest) for registry operations

## Development Status

This library is under active development. Current status:

- ✅ Configuration System (Epic 2) - Complete
- 🚧 Error Handling (Epic 3) - In Progress
- 📋 Core Types (Epic 4) - Planned
- 📋 Versioning Engine (Epic 5) - Planned
- 📋 Additional features - See [Story Map](STORY_MAP.md)

## Contributing

We follow strict code quality standards:

- 100% clippy compliance with strict lints
- 100% test coverage
- Comprehensive documentation
- No assumptions - validate all inputs
- Robust error handling

See our [development rules](../../CLAUDE.md) for detailed guidelines.

## License

[License information to be added]

## Support

For issues, questions, or contributions:

1. Check the [Configuration Guide](docs/guides/configuration.md)
2. Review [examples](examples/)
3. Read the API documentation
4. Open an issue on GitHub

## Acknowledgments

Part of the Sublime Tools ecosystem for Node.js project management in Rust.