# Workspace Tools

[![Pull Request](https://github.com/websublime/workspace-tools/workflows/Pull%20Request/badge.svg)](https://github.com/websublime/workspace-tools/actions)
[![Crates.io](https://img.shields.io/crates/v/sublime_cli_tools.svg)](https://crates.io/crates/sublime_cli_tools)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](./crates/cli/README.md)

**Modern, changeset-based version management for JavaScript/TypeScript projects**

Workspace Tools provides a comprehensive CLI (`workspace`) and Rust libraries for managing JavaScript/TypeScript single-package repositories and monorepos with changeset-based versioning, automated dependency management, and project health auditing.

---

## 🎯 Why Workspace Tools?

Modern JavaScript/TypeScript development requires sophisticated tooling for version management, especially in monorepos. Workspace Tools solves this with:

- **🔄 Changeset-Based Workflow** - Track changes across feature branches with automated package detection
- **📦 Smart Version Management** - Support for both independent and unified versioning strategies
- **⚡ Dependency Intelligence** - Automatic upgrade detection with safety checks and rollback capability
- **🏥 Project Health Auditing** - Comprehensive analysis with actionable insights and health scores
- **🔧 Git Integration** - Seamless workflow integration with hooks and automation
- **🤖 CI/CD Ready** - JSON output modes and silent operation for pipeline integration
- **⚙️ Cross-Platform** - Works consistently on Windows, Linux, and macOS

---

## 🚀 Quick Start

### Installation

Choose your preferred installation method:

**Automated Installer** (recommended - works on all platforms):
```bash
# Unix/Linux/macOS
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/websublime/workspace-tools/releases/latest/download/sublime_cli_tools-installer.sh | sh

# Windows PowerShell
powershell -ExecutionPolicy ByPass -c "irm https://github.com/websublime/workspace-tools/releases/latest/download/sublime_cli_tools-installer.ps1 | iex"
```

**Using Cargo** (alternative for Rust developers):
```bash
cargo install sublime_cli_tools
```

**Manual Download** (pre-built binaries):
```bash
# macOS (Apple Silicon)
curl -L https://github.com/websublime/workspace-tools/releases/latest/download/sublime_cli_tools-aarch64-apple-darwin.tar.xz | tar xJ
sudo mv workspace /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/websublime/workspace-tools/releases/latest/download/sublime_cli_tools-x86_64-apple-darwin.tar.xz | tar xJ
sudo mv workspace /usr/local/bin/

# Linux (x86_64 - GNU)
curl -L https://github.com/websublime/workspace-tools/releases/latest/download/sublime_cli_tools-x86_64-unknown-linux-gnu.tar.xz | tar xJ
sudo mv workspace /usr/local/bin/

# Linux (x86_64 - MUSL - static)
curl -L https://github.com/websublime/workspace-tools/releases/latest/download/sublime_cli_tools-x86_64-unknown-linux-musl.tar.xz | tar xJ
sudo mv workspace /usr/local/bin/

# Windows
# Download from: https://github.com/websublime/workspace-tools/releases/latest
# Extract sublime_cli_tools-x86_64-pc-windows-msvc.zip and add to PATH
```

**From Source**:
```bash
git clone https://github.com/websublime/workspace-tools.git
cd workspace-tools
cargo install --path crates/cli
```

Verify installation:
```bash
workspace --version
```

---

### 5-Minute Workflow

Get started with a complete changeset-based workflow:

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

# 4. Preview version changes
workspace bump --dry-run

# 5. Apply version bumps and release
workspace bump --execute --git-tag --git-push
```

That's it! You now have a complete audit trail from development to release.

---

## ✨ Key Features

### Changeset Management

Track changes across feature branches with full commit history:

```bash
# Create changeset for current branch
workspace changeset create --bump minor --env production

# Automatically detect affected packages from git changes
workspace changeset update

# List all active changesets
workspace changeset list

# View detailed changeset information
workspace changeset show feature/new-api

# Query archived changesets
workspace changeset history --package @myorg/core
```

### Version Management

Intelligent version bumping with multiple strategies:

```bash
# Preview version changes (safe, no modifications)
workspace bump --dry-run

# Apply version changes
workspace bump --execute

# Full release workflow with git operations
workspace bump --execute --git-commit --git-tag --git-push

# Generate snapshot versions for testing
workspace bump --snapshot --execute

# Create pre-release versions
workspace bump --prerelease beta --execute
```

**Versioning Strategies:**
- **Independent**: Each package maintains its own version (only affected packages bump)
- **Unified**: All packages share the same version (coordinated releases)

### Dependency Upgrades

Safe dependency management with automatic detection:

```bash
# Check for available upgrades
workspace upgrade check

# Apply safe upgrades (patch and minor only)
workspace upgrade apply --minor-and-patch

# Apply upgrades with automatic changeset creation
workspace upgrade apply --patch-only --auto-changeset

# Rollback if needed
workspace upgrade backups list
workspace upgrade backups restore backup_20240107_103045
```

### Project Health Auditing

Comprehensive health analysis with actionable recommendations:

```bash
# Full project audit
workspace audit

# Specific audit sections
workspace audit --sections upgrades,dependencies

# Generate markdown report
workspace audit --output audit-report.md

# JSON output for CI/CD
workspace --format json audit
```

**Audit Sections:**
- **Upgrades**: Available dependency upgrades with breaking change detection
- **Dependencies**: Circular dependencies, missing packages, deprecated packages
- **Version Consistency**: Version alignment across monorepo
- **Breaking Changes**: Detect breaking changes from commits and dependencies

### CI/CD Integration

Designed for automation with JSON output and silent operation:

```bash
# Get version information for CI/CD (clean JSON, no logs)
workspace --format json --log-level silent bump --dry-run

# Detect affected packages from changes
workspace --format json changes --since main

# Health check in CI/CD
workspace --format json --log-level silent audit --sections upgrades

# Automated release
workspace bump --execute --git-tag --git-push --force
```

### Git Hooks Integration

Automate changeset management throughout your development workflow with Git hooks:

```bash
# Install hooks (one-time setup)
./scripts/install-hooks.sh

# Hooks are now active!
```

**Available Hooks:**
- **pre-commit**: Validates changeset exists, prompts to create if missing
- **post-commit**: Automatically adds commit SHA to changeset
- **post-checkout**: Creates changeset when starting new feature branches
- **pre-push**: Adds all branch commits to changeset before pushing

**Workflow with Hooks:**
```bash
# Create feature branch
git checkout -b feature/new-thing
# → Hook prompts to create changeset

# Make commits (as many as you want)
git commit -m "feat: add feature"
# → Hook validates changeset exists

# Push (commits are added here)
git push origin feature/new-thing
# → Hook adds all branch commits to changeset
# → Hook creates commit "chore: update changeset"
# → Push includes all commits + changeset update
```

**Skip Hooks Temporarily:**
```bash
WORKSPACE_SKIP_HOOKS=1 git commit -m "wip"
```

For complete documentation, see [Git Hooks Documentation](./scripts/git-hooks/README.md).

---

## 📚 Documentation

### Documentation

- **[CLI Documentation](./crates/cli/README.md)** - Complete CLI documentation including installation, configuration, commands, and workflows

### Core Concepts

**Changesets**: Track intended version bumps across feature branches with full commit history and affected packages.

**Versioning Strategies**:
- **Independent**: Each package evolves at its own pace (only packages in changesets bump)
- **Unified**: All packages share the same version (coordinated releases across workspace)

**Environments**: Define deployment targets (dev, staging, production) for changesets.

**Package Detection**: Automatically detects affected packages from Git changes in monorepos.

### Library Documentation

The project provides reusable Rust libraries:

- **[sublime_package_tools](./crates/pkg/)** - Package management, changesets, versioning, upgrades, auditing
- **[sublime_standard_tools](./crates/standard/)** - Filesystem operations, command execution, configuration
- **[sublime_git_tools](./crates/git/)** - Git operations and integrations
- **[sublime_cli_tools](./crates/cli/)** - CLI interface and user interaction

---

## 🏗️ Architecture

### Project Structure

```
workspace-tools/
├── crates/
│   ├── cli/              # CLI application (workspace binary)
│   │   ├── docs/         # User documentation
│   │   ├── src/          # CLI implementation
│   │   └── Cargo.toml
│   ├── pkg/              # Package tools library
│   │   ├── src/          # Core logic (changesets, versions, upgrades, audits)
│   │   ├── SPEC.md       # API specification
│   │   └── Cargo.toml
│   ├── standard/         # Standard tools library
│   │   ├── src/          # Infrastructure (filesystem, commands, config)
│   │   ├── SPEC.md       # API specification
│   │   └── Cargo.toml
│   └── git/              # Git tools library
│       ├── src/          # Git operations
│       ├── SPEC.md       # API specification
│       └── Cargo.toml
├── scripts/              # Build and release automation
├── Cargo.toml            # Workspace configuration
└── README.md             # This file
```

### Design Principles

1. **Layered Architecture**: Clean separation between CLI, business logic, and infrastructure
2. **Error Handling**: Comprehensive error types with user-friendly messages
3. **Testability**: 100% test coverage target with unit, integration, and E2E tests
4. **Documentation**: All public APIs documented with examples
5. **Quality**: 100% Clippy compliance with strict linting rules
6. **Safety**: No `unwrap()`, `expect()`, `todo!()`, or `panic!()` in production code

---

## 🎓 Example Workflows

### Feature Development

```bash
# Create feature branch
git checkout -b feature/authentication

# Create changeset
workspace changeset create \
  --bump minor \
  --env production,staging \
  --message "Add JWT authentication"

# Develop and track changes
git commit -m "feat: add JWT tokens"
workspace changeset update

git commit -m "test: add auth tests"
workspace changeset update

# Preview versions before merge
workspace bump --dry-run

# After merge to main, release
git checkout main
git merge feature/authentication
workspace bump --execute --git-tag --git-push
```

### Hotfix Workflow

```bash
# Create hotfix branch
git checkout -b hotfix/security-patch

# Create patch changeset
workspace changeset create --bump patch --env production

# Fix and commit
git commit -m "fix: patch security vulnerability"
workspace changeset update

# Quick release
workspace bump --execute --git-tag --git-push --force
```

### Dependency Maintenance

```bash
# Check for upgrades
workspace upgrade check

# Apply safe upgrades with changeset
workspace upgrade apply --minor-and-patch --auto-changeset

# Run tests
npm test

# If tests pass, release
workspace bump --execute --git-tag
```

### CI/CD Pipeline

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    branches: [main]

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install workspace
        run: cargo install sublime_cli_tools
      
      - name: Check for changesets
        id: check
        run: |
          COUNT=$(workspace --format json changeset list | jq '.total')
          echo "count=$COUNT" >> $GITHUB_OUTPUT
      
      - name: Release
        if: steps.check.outputs.count > 0
        run: |
          workspace bump --execute --git-tag --git-push --force
```

---

## 🛠️ Development

### Prerequisites

- Rust 1.90.0 or higher
- Node.js 18+ (for testing with JavaScript/TypeScript projects)
- Git

### Building

```bash
# Clone repository
git clone https://github.com/websublime/workspace-tools.git
cd workspace-tools

# Build all crates
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run clippy (must pass 100%)
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### Testing

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p sublime_package_tools
cargo test -p sublime_cli_tools

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test --test '*'
```

### Quality Standards

This project maintains strict quality standards:

- ✅ **100% Clippy compliance** - Pedantic mode enabled
- ✅ **No unsafe operations** - No `unwrap()`, `expect()`, `todo!()`, `unimplemented!()`, or `panic!()`
- ✅ **Comprehensive documentation** - All public APIs documented with examples
- ✅ **Test coverage** - Target 100% coverage with unit, integration, and E2E tests
- ✅ **Consistent patterns** - Same patterns across all crates
- ✅ **Internal visibility** - Uses `pub(crate)` for internal modules

---

## 🚀 Release Process

This project uses **fully automated releases** powered by **Release Please**:

### How It Works

1. **Use conventional commits** (`feat:`, `fix:`, `docs:`, etc.)
2. **Merge to main** via pull request
3. **Release Please creates Release PR** automatically with:
   - Version updates
   - Changelog generation
   - Release notes
4. **Merge Release PR** → Automated publication:
   - Crates published to crates.io
   - Binaries built for all platforms
   - GitHub Release created

**Zero manual release commands!** 🎉

### Conventional Commits

```bash
# Feature (minor version bump)
git commit -m "feat(cli): add interactive mode"

# Bug fix (patch version bump)
git commit -m "fix(pkg): resolve version conflict"

# Breaking change (major version bump)
git commit -m "feat!: redesign API"

# Documentation (no version bump)
git commit -m "docs: update README"
```

For detailed information, see [RELEASE.md](./RELEASE.md).

---

## 🌟 Supported Platforms

Every release includes optimized binaries for:

- **Linux x86_64** (GNU and MUSL - static)
- **macOS x86_64** (Intel) and ARM64 (Apple Silicon)
- **Windows x86_64**

All binaries are:
- Fully optimized with LTO
- Stripped of debug symbols
- Automatically tested in CI
- SHA256 checksums included

---

## 📖 Configuration

### Layered Configuration System

Configuration spans three layers:

1. **Standard Tools Layer** - Infrastructure (filesystem, commands, monorepo detection)
2. **Package Tools Layer** - Package management (changesets, versioning, upgrades)
3. **CLI Layer** - User interface (output, logging, global settings)

### Configuration Sources (Priority Order)

1. CLI flags (`--root`, `--config`, `--log-level`, etc.)
2. Environment variables (`SUBLIME_*`, `WORKSPACE_*`)
3. Project config file (`repo.config.{toml,json,yaml}`)
4. Global config file (`~/.config/sublime/config.toml`)
5. Built-in defaults

### Quick Configuration

```bash
# Initialize with interactive prompts
workspace init

# Initialize with options
workspace init \
  --strategy independent \
  --config-format yaml \
  --environments "dev,staging,prod"

# View current configuration
workspace config show

# Validate configuration
workspace config validate
```

For complete configuration documentation, see the [CLI Documentation](./crates/cli/README.md).

---

## 🤝 Contributing

We welcome contributions! Please follow these guidelines:

1. Check the [Development Roadmap](./crates/cli/STORY_MAP.md) for planned work
2. Follow the [Implementation Plan](./crates/cli/PLAN.md) patterns
3. Ensure 100% Clippy compliance
4. Document all public APIs with examples
5. Write comprehensive tests
6. Use conventional commits

See [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed contribution guidelines.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE-MIT](./LICENSE-MIT) file for details.

---

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) for performance and reliability
- Inspired by [Changesets](https://github.com/changesets/changesets) for the changeset workflow
- Powered by excellent Rust crates: [clap](https://github.com/clap-rs/clap), [tokio](https://tokio.rs/), [serde](https://serde.rs/)

---

## 📞 Support & Community

- **Issues**: [GitHub Issues](https://github.com/websublime/workspace-tools/issues)
- **Discussions**: [GitHub Discussions](https://github.com/websublime/workspace-tools/discussions)
- **Documentation**: [CLI Documentation](./crates/cli/README.md)

---

<div align="center">

**[Documentation](./crates/cli/README.md)** •
**[Examples](#-example-workflows)** •
**[Contributing](./CONTRIBUTING.md)**

Made with ❤️ by WebSublime

</div>
