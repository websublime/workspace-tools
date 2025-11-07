# Workspace Node Tools CLI

[![Pull Request](https://github.com/websublime/workspace-node-tools/workflows/Pull%20Request/badge.svg)](https://github.com/websublime/workspace-node-tools/actions)
[![Crates.io](https://img.shields.io/crates/v/sublime_cli_tools.svg)](https://crates.io/crates/sublime_cli_tools)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A comprehensive command-line interface for managing Node.js workspaces and monorepos with changeset-based version management.

---

## Overview

`workspace` (Workspace Node Tools CLI) provides:

- **Configuration Management**: Initialize and validate workspace configurations
- **Changeset Workflow**: Create, update, list, show, edit, and remove changesets
- **Version Management**: Intelligent version bumping with preview mode and multiple strategies
- **Dependency Upgrades**: Detect, apply, and rollback dependency updates
- **Audit System**: Comprehensive health checks with actionable insights
- **Change Analysis**: Detect affected packages from Git changes
- **CI/CD Integration**: JSON output modes and silent operation for automation

---

## Installation

### Quick Install (Recommended)

**Using Cargo:**
```bash
cargo install sublime_cli_tools
```

**From GitHub Releases** (pre-built binaries):

Download pre-built binaries from [GitHub Releases](https://github.com/websublime/workspace-node-tools/releases):

**macOS (Apple Silicon)**:
```bash
curl -L https://github.com/websublime/workspace-node-tools/releases/latest/download/workspace-v0.1.0-aarch64-apple-darwin.tar.gz | tar xz
sudo mv workspace /usr/local/bin/
```

**macOS (Intel)**:
```bash
curl -L https://github.com/websublime/workspace-node-tools/releases/latest/download/workspace-v0.1.0-x86_64-apple-darwin.tar.gz | tar xz
sudo mv workspace /usr/local/bin/
```

**Linux (x86_64 GNU)**:
```bash
curl -L https://github.com/websublime/workspace-node-tools/releases/latest/download/workspace-v0.1.0-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv workspace /usr/local/bin/
```

**Linux (x86_64 MUSL - static binary)**:
```bash
curl -L https://github.com/websublime/workspace-node-tools/releases/latest/download/workspace-v0.1.0-x86_64-unknown-linux-musl.tar.gz | tar xz
sudo mv workspace /usr/local/bin/
```

**Windows**:
Download `workspace-v0.1.0-x86_64-pc-windows-msvc.zip` from releases and extract.

### From Source

```bash
git clone https://github.com/websublime/workspace-node-tools.git
cd workspace-node-tools
cargo install --path crates/cli
```

### Verify Installation

```bash
workspace --version
workspace --help
```

---

## Quick Start

Get started with a complete workflow in 5 minutes:

```bash
# 1. Initialize your project
cd your-project
workspace init

# 2. Create a feature branch and changeset
git checkout -b feature/new-api
workspace changeset create

# 3. Make changes and track them
# ... edit files ...
git commit -m "feat: add new API endpoint"
workspace changeset update

# 4. Preview version changes (safe, no modifications)
workspace bump --dry-run

# 5. Apply version bumps and release
workspace bump --execute --git-tag --git-push
```

---

## Documentation

### User Documentation

- **[User Guide](./docs/GUIDE.md)** - Comprehensive guide covering:
  - Installation methods
  - Configuration (layered system with environment variables)
  - Core concepts (changesets, versioning strategies, environments)
  - Workflows (feature development, hotfixes, dependency upgrades)
  - CI/CD integration examples
  - Best practices

- **[Command Reference](./docs/COMMANDS.md)** - Complete command documentation:
  - All commands with synopsis and descriptions
  - Global options (--root, --log-level, --format, --no-color, --config)
  - Command-specific options
  - Output examples (human and JSON formats)
  - Common patterns and quick reference

### Key Commands

```bash
# Configuration
workspace init                          # Initialize project
workspace config show                   # View configuration
workspace config validate               # Validate configuration

# Changesets
workspace changeset create              # Create changeset
workspace changeset update              # Update with commits
workspace changeset list                # List all changesets
workspace changeset show <branch>       # Show details

# Version Management
workspace bump --dry-run                # Preview (default, safe)
workspace bump --execute                # Apply versions
workspace bump --execute --git-tag      # Release with tags

# Dependency Upgrades
workspace upgrade check                 # Check for upgrades
workspace upgrade apply --patch-only    # Apply safe upgrades
workspace upgrade backups list          # List backups

# Project Health
workspace audit                         # Full health audit
workspace audit --sections upgrades     # Specific audit
workspace changes                       # Analyze changes
```

---

## Development Status

🚧 **This crate is currently under active development** 🚧

**Current Status**: Foundation & Core Commands

**Completed**:
- ✅ Project structure initialized
- ✅ Dependencies configured
- ✅ Error handling framework
- ✅ Output formatting framework (human, JSON, compact)
- ✅ Logging system (independent from output format)
- ✅ Global options context (--root, --log-level, --format, --no-color, --config)
- ✅ Clippy rules enforced (100% compliance)
- ✅ Build system with shell completions
- ✅ Documentation structure

**Next Steps** (see [STORY_MAP.md](./STORY_MAP.md)):
- Story 1.2: CI/CD Pipeline
- Story 1.3: Error Handling System
- Story 1.4: CLI Framework with Clap
- Story 2.x: Configuration Commands
- Story 4.x: Changeset Commands
- Story 5.x: Version Management Commands

For detailed development roadmap, see [STORY_MAP.md](./STORY_MAP.md) and [PLAN.md](./PLAN.md).

---

## Features

### Global Options

All commands support these global options:

- `--root <PATH>` - Project root directory (default: current directory)
- `--log-level <LEVEL>` - Log level: silent, error, warn, info (default), debug, trace
- `--format <FORMAT>` - Output format: human (default), json, json-compact, quiet
- `--no-color` - Disable colored output
- `--config <PATH>` - Path to config file (default: auto-detect)

**Key Principle**: Logging (stderr) and output (stdout) are completely independent!

```bash
# Clean JSON output with no logs (perfect for automation)
workspace --format json --log-level silent bump --dry-run

# JSON output with debug logs (logs to stderr, JSON to stdout)
workspace --format json --log-level debug bump --dry-run > output.json 2> debug.log
```

### Output Formats

- **human**: Human-readable with colors and tables (default)
- **json**: Pretty-printed JSON for scripting
- **json-compact**: Compact JSON (single line) for CI/CD
- **quiet**: Minimal output

### Versioning Strategies

**Independent Strategy**: Each package maintains its own version
- Only packages listed in changesets get version bumps
- Ideal for packages that evolve independently

**Unified Strategy**: All packages share the same version
- When any package needs a bump, all packages get bumped
- Ideal for tightly-coupled packages

### CI/CD Integration

Designed for automation with:
- JSON output modes
- Silent logging (no logs in output)
- Non-interactive modes
- Exit codes for scripting
- Atomic operations

Example GitHub Actions workflow:

```yaml
- name: Check for changesets
  run: |
    COUNT=$(workspace --format json --log-level silent changeset list | jq '.total')
    if [ "$COUNT" -gt 0 ]; then
      workspace bump --execute --git-tag --git-push --force
    fi
```

---

## Building

```bash
# Build the CLI
cargo build

# Build in release mode (optimized)
cargo build --release

# Run clippy (must pass 100%)
cargo clippy -- -D warnings

# Format code
cargo fmt

# Run tests
cargo test
```

---

## Project Structure

```
crates/cli/
├── Cargo.toml           # Dependencies and configuration
├── build.rs             # Build-time shell completion generation
├── src/
│   ├── main.rs         # Binary entry point with async runtime
│   ├── lib.rs          # Library exports for testability
│   ├── cli/            # CLI framework (Clap integration)
│   ├── commands/       # Command implementations
│   ├── error/          # Error types and handling
│   ├── output/         # Output formatting and logging
│   └── ...             # Additional modules
├── docs/               # User documentation
│   ├── GUIDE.md        # Comprehensive user guide
│   └── COMMANDS.md     # Command reference
├── STORY_MAP.md        # Development roadmap
├── PLAN.md             # Implementation plan
└── PRD.md              # Product requirements
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

## Quality Standards

This crate follows strict quality standards:

- ✅ **100% Clippy compliance** - Pedantic mode enabled, no warnings allowed
- ✅ **No unsafe operations** - No `unwrap()`, `expect()`, `todo!()`, `unimplemented!()`, or `panic!()` in production code
- ✅ **Comprehensive documentation** - All public APIs documented with examples
- ✅ **Module-level documentation** - Explaining What, How, and Why
- ✅ **Test coverage target** - 100% coverage with unit, integration, and E2E tests
- ✅ **Consistent patterns** - Same patterns across all modules
- ✅ **Internal visibility** - Uses `pub(crate)` for internal modules

Clippy rules enforced:

```rust
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]
```

---

## Dependencies

### Core CLI Dependencies

- **clap** - CLI argument parsing with derive macros
- **tokio** - Async runtime for concurrent operations

### Terminal & UI

- **crossterm** - Cross-platform terminal control
- **console** - Terminal styling and colors
- **dialoguer** - Interactive prompts for user input
- **indicatif** - Progress bars for long operations
- **comfy-table** - Beautiful table rendering

### Internal Crates

- **sublime_package_tools** - Package and version management logic
- **sublime_standard_tools** - Node.js utilities and filesystem operations
- **sublime_git_tools** - Git operations and integrations

---

## Release Process

This project uses **fully automated releases** powered by **Release Please**:

### How It Works

1. **Use conventional commits** (`feat:`, `fix:`, etc.)
   ```bash
   git commit -m "feat(cli): add interactive mode"
   git commit -m "fix(pkg): resolve version conflict"
   ```

2. **Merge to main** via PR

3. **Release Please creates Release PR** automatically
   - Updates versions
   - Updates CHANGELOG
   - Prepares release notes

4. **Review and merge the Release PR**
   - Crates published to crates.io
   - Binaries built for all platforms
   - GitHub Release created

**Zero manual commands!** 🎉

### Supported Platforms

Every release includes optimized binaries for:
- Linux x86_64 (GNU and MUSL - static)
- macOS x86_64 (Intel) and ARM64 (Apple Silicon)
- Windows x86_64

All binaries are fully optimized (LTO, strip symbols) and automatically tested.

For detailed information, see [../../RELEASE.md](../../RELEASE.md).

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
- **[User Guide](./docs/GUIDE.md)** - Comprehensive user documentation
- **[Command Reference](./docs/COMMANDS.md)** - Complete command documentation
- **[Story Map](./STORY_MAP.md)** - Development roadmap
- **[Implementation Plan](./PLAN.md)** - Detailed technical plan
- **[Product Requirements](./PRD.md)** - Feature requirements

---

<div align="center">

**[Documentation](./docs/GUIDE.md)** •
**[Commands](./docs/COMMANDS.md)** •
**[Contributing](../../CONTRIBUTING.md)** •
**[Issues](https://github.com/websublime/workspace-node-tools/issues)**

Part of [Workspace Node Tools](../../README.md)

</div>
