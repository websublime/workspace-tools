# sublime_pkg_tools

A comprehensive Rust library for managing Node.js package versioning, changesets, changelogs, and dependency upgrades in both single-package projects and monorepos.

## Features

- **Changeset Management**: Track and manage changesets for coordinated package releases
- **Version Resolution**: Intelligent version calculation with dependency propagation
- **Dependency Upgrades**: Detect and apply external dependency upgrades automatically
- **Changelog Generation**: Generate changelogs in multiple formats (Keep a Changelog, Conventional Commits)
- **Changes Analysis**: Analyze working directory and commit ranges to identify affected packages
- **Audit & Health Checks**: Comprehensive dependency audits and health score calculation
- **Monorepo Support**: Full support for both independent and unified versioning strategies
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
use sublime_pkg_tools::config::{load_config, PackageToolsConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from default locations with environment overrides
    let config = load_config().await?;
    
    // Or use defaults
    let config = PackageToolsConfig::default();
    
    println!("Configuration loaded successfully!");
    Ok(())
}
```

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
  - `load_config.rs` - Different methods for loading configuration
  - `env_override.rs` - Environment variable override examples

Run examples:

```bash
# Load configuration example
cargo run --example load_config

# Environment variable override example
SUBLIME_PKG_VERSION_STRATEGY=unified cargo run --example env_override
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