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

### Quick Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh
```

This will download the appropriate binary for your platform and install it to `~/.local/bin/wnt`.

### From GitHub Releases

Download pre-built binaries from [GitHub Releases](https://github.com/websublime/workspace-node-tools/releases):

**Linux (x86_64 GNU)**:
```bash
wget https://github.com/websublime/workspace-node-tools/releases/latest/download/wnt-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
tar xzf wnt-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
sudo mv wnt /usr/local/bin/
```

**Linux (x86_64 MUSL - static binary)**:
```bash
wget https://github.com/websublime/workspace-node-tools/releases/latest/download/wnt-v0.1.0-x86_64-unknown-linux-musl.tar.gz
tar xzf wnt-v0.1.0-x86_64-unknown-linux-musl.tar.gz
sudo mv wnt /usr/local/bin/
```

**macOS (Intel)**:
```bash
wget https://github.com/websublime/workspace-node-tools/releases/latest/download/wnt-v0.1.0-x86_64-apple-darwin.tar.gz
tar xzf wnt-v0.1.0-x86_64-apple-darwin.tar.gz
sudo mv wnt /usr/local/bin/
```

**macOS (Apple Silicon)**:
```bash
wget https://github.com/websublime/workspace-node-tools/releases/latest/download/wnt-v0.1.0-aarch64-apple-darwin.tar.gz
tar xzf wnt-v0.1.0-aarch64-apple-darwin.tar.gz
sudo mv wnt /usr/local/bin/
```

**Windows**:
Download `wnt-v0.1.0-x86_64-pc-windows-msvc.zip` from releases and extract.

### Using Cargo

```bash
cargo install sublime_cli_tools
```

### From Source

```bash
git clone https://github.com/websublime/workspace-node-tools.git
cd workspace-node-tools
cargo install --path crates/cli
```

### Verify Installation

After installation, verify it works:

```bash
wnt --version
wnt --help
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

## Release Process

This project uses a **fully automated release process** powered by **Release Please**.

### How It Works

Releases are 100% automatic using conventional commits:

1. **Use conventional commit messages** (`feat:`, `fix:`, etc.)
   ```bash
   git commit -m "feat: add new command"
   git commit -m "fix: resolve bug"
   ```

2. **Merge to main** via PR

3. **Release Please creates a Release PR** automatically
   - Updates versions
   - Updates CHANGELOG
   - Prepares release notes

4. **Review and merge the Release PR**
   - Crates published to crates.io
   - Binaries built for all platforms
   - GitHub Release created

**Zero manual commands!** 🎉

### For Developers

Use conventional commit format:

```bash
# Feature (minor bump)
git commit -m "feat(cli): add interactive mode"

# Bug fix (patch bump)
git commit -m "fix(pkg): resolve version conflict"

# Breaking change (major bump)
git commit -m "feat!: redesign API"
# or
git commit -m "feat: change API" -m "BREAKING CHANGE: old API removed"

# No version bump
git commit -m "docs: update README"
git commit -m "chore: update dependencies"
```

Merge PR → Release Please handles everything!

### Supported Platforms

Every release includes optimized binaries for:
- Linux x86_64 (GNU and MUSL - static)
- macOS x86_64 (Intel) and ARM64 (Apple Silicon)
- Windows x86_64

All binaries are:
- Fully optimized (LTO, strip symbols)
- Automatically tested in CI
- SHA256 checksums included
- Available on GitHub Releases and crates.io

For detailed information, see [RELEASE.md](../../RELEASE.md).

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