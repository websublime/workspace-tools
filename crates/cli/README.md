# Workspace Node Tools CLI

A comprehensive command-line interface for managing Node.js workspaces and monorepos with changeset-based version management.

## Overview

`wnt` (Workspace Node Tools) is a CLI tool that provides:

- **Configuration Management**: Initialize and validate workspace configurations
- **Changeset Workflow**: Add, list, show, update, edit, and remove changesets
- **Version Management**: Bump versions with preview mode
- **Dependency Upgrades**: Check for and apply dependency updates
- **Audit System**: Comprehensive health checks and issue detection
- **Change Analysis**: Detect affected packages from file changes

## Installation

### From Source

```bash
cargo install --path .
```

### Using Cargo

```bash
cargo install sublime_cli
```

## Development Status

🚧 **This crate is currently under development** 🚧

**Current Status**: Story 1.1 - CLI Foundation Complete

- ✅ Project structure initialized
- ✅ Dependencies configured
- ✅ Error handling framework
- ✅ Output formatting framework
- ✅ Clippy rules enforced
- ✅ Build system with shell completions

**Next Steps**:
- Story 1.2: CI/CD Pipeline
- Story 1.3: Error Handling System
- Story 1.4: CLI Framework with Clap

## Building

```bash
# Build the CLI
cargo build

# Build in release mode
cargo build --release

# Run clippy
cargo clippy -- -D warnings

# Format code
cargo fmt
```

## Project Structure

```
crates/cli/
├── Cargo.toml           # Dependencies and configuration
├── build.rs             # Build-time shell completion generation
├── src/
│   ├── main.rs         # Binary entry point with async runtime
│   ├── lib.rs          # Library exports for testability
│   ├── cli/            # CLI framework (to be implemented)
│   ├── commands/       # Command implementations (to be implemented)
│   ├── error/          # Error types and handling
│   ├── output/         # Output formatting and logging
│   └── ...             # Additional modules (to be added)
```

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
- `User`: User-caused errors (invalid input, cancelled)

Each error maps to appropriate exit codes following sysexits conventions.

### Output Formats

The CLI supports multiple output formats:

- `human`: Human-readable with colors and tables (default)
- `json`: Pretty-printed JSON
- `json-compact`: Compact JSON (single line)
- `quiet`: Minimal output

### Logging vs Output

- **Logging** (`--log-level`): Controls stderr output for debugging
- **Output** (`--format`): Controls stdout for command results
- These are completely independent and can be combined freely

## Quality Standards

This crate follows strict quality standards:

- ✅ 100% clippy compliance (pedantic mode)
- ✅ No `unwrap()`, `expect()`, `todo!()`, `unimplemented!()`, or `panic!()`
- ✅ Comprehensive documentation
- ✅ All public APIs documented with examples
- ✅ Module-level documentation explaining What, How, and Why

## Dependencies

### Core
- `clap`: CLI argument parsing
- `tokio`: Async runtime

### Terminal & UI
- `crossterm`: Cross-platform terminal control
- `console`: Terminal styling
- `dialoguer`: Interactive prompts
- `indicatif`: Progress bars
- `comfy-table`: Table rendering

### Internal Crates
- `sublime_pkg_tools`: Package and version management
- `sublime_standard_tools`: Node.js utilities
- `sublime_git_tools`: Git operations

## License

MIT

## Contributing

This is an active development project. Please follow the established patterns:

1. Check STORY_MAP.md for planned work
2. Follow the implementation guidelines in PLAN.md
3. Ensure 100% clippy compliance
4. Document all public APIs
5. Write comprehensive tests

## Links

- [Story Map](./STORY_MAP.md) - Development roadmap
- [Implementation Plan](./PLAN.md) - Detailed technical plan
- [Product Requirements](./PRD.md) - Feature requirements