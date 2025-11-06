# Testing Guide for Workspace Node Tools CLI

This guide covers how to build, install, and test the `workspace` CLI tool.

## Table of Contents

- [Quick Start](#quick-start)
- [Building the CLI](#building-the-cli)
- [Installation Options](#installation-options)
- [Testing in a Demo Repository](#testing-in-a-demo-repository)
- [Testing the Edit Command](#testing-the-edit-command)
- [Manual Testing](#manual-testing)
- [Development Workflow](#development-workflow)

## Quick Start

```bash
# 1. Build and install locally
./scripts/install-local.sh

# 2. Verify installation
workspace version

# 3. Test in a demo repository
./scripts/test-in-demo.sh
```

## Building the CLI

### Development Build

Fast build for development and testing:

```bash
cargo build --package sublime_cli_tools
```

Binary location: `target/debug/workspace`

### Release Build

Optimized build for production use:

```bash
cargo build --package sublime_cli_tools --release
```

Binary location: `target/release/workspace` (~7MB)

### Build Features

The release build includes:
- LTO (Link-Time Optimization)
- Symbol stripping
- Optimization level 3
- Panic = abort for smaller binary

## Installation Options

### Option 1: Using install-local.sh (Recommended)

```bash
# Install to ~/.local/bin (default)
./scripts/install-local.sh

# Install to custom location
./scripts/install-local.sh ~/bin

# Install system-wide
sudo ./scripts/install-local.sh /usr/local/bin
```

### Option 2: Manual Installation

```bash
# Build release binary
cargo build --package sublime_cli_tools --release

# Copy to desired location
cp target/release/workspace ~/.local/bin/workspace
chmod +x ~/.local/bin/workspace

# Add to PATH if needed
export PATH="$HOME/.local/bin:$PATH"
```

### Option 3: Run Without Installing

```bash
# Build and run directly
cargo run --package sublime_cli_tools --release -- --help

# Example commands
cargo run --package sublime_cli_tools --release -- init
cargo run --package sublime_cli_tools --release -- changeset create
```

## Testing in a Demo Repository

### Using the Demo Script

The `test-in-demo.sh` script creates a complete test environment:

```bash
./scripts/test-in-demo.sh
```

This script:
1. ✅ Builds the CLI binary
2. ✅ Creates a temporary monorepo
3. ✅ Initializes git repository
4. ✅ Sets up workspace with `workspace init`
5. ✅ Creates a feature branch
6. ✅ Creates sample changesets
7. ✅ Demonstrates all commands
8. ✅ Keeps environment alive for manual testing
9. ✅ Cleans up on exit

### Manual Testing in Demo

While the demo script is running, open a new terminal:

```bash
# Navigate to demo directory (path shown in script output)
cd /tmp/workspace-demo-XXXXXX

# Test commands
workspace changeset list
workspace changeset show feature/test-edit
workspace changeset edit

# Test with different options
workspace changeset edit --format json
EDITOR=vim workspace changeset edit
```

## Testing the Edit Command

### Prerequisites

Set your preferred editor:

```bash
export EDITOR=vim      # or nano, code, emacs, etc.
export VISUAL=code     # optional, higher priority than EDITOR
```

### Test Scenario 1: Basic Edit

```bash
# Start demo environment
./scripts/test-in-demo.sh

# In another terminal
cd /tmp/workspace-demo-XXXXXX
workspace changeset edit

# Your editor opens the changeset file
# Make changes and save
# Command validates and applies changes
```

### Test Scenario 2: Edit Specific Branch

```bash
# Create multiple changesets
git checkout -b feature/feature-a
workspace changeset create --bump minor --non-interactive

git checkout -b feature/feature-b
workspace changeset create --bump patch --non-interactive

# Edit specific changeset
workspace changeset edit feature/feature-a
```

### Test Scenario 3: Validation Testing

Test that validation catches errors:

```bash
workspace changeset edit

# In editor, try invalid changes:
# 1. Change branch name → Should reject
# 2. Remove all packages → Should reject
# 3. Remove all environments → Should reject
# 4. Invalid JSON syntax → Should reject
# 5. Valid changes → Should accept
```

### Test Scenario 4: JSON Output

```bash
workspace changeset edit --format json
workspace changeset edit feature/branch --format json-compact
```

### Test Scenario 5: Different Editors

```bash
# Test with different editors
EDITOR=nano workspace changeset edit
EDITOR=vim workspace changeset edit
EDITOR=code workspace changeset edit
VISUAL=emacs workspace changeset edit
```

## Manual Testing

### Create a Real Test Repository

```bash
# Create test directory
mkdir -p ~/test-repos/workspace-test
cd ~/test-repos/workspace-test

# Initialize git
git init
git config user.email "test@example.com"
git config user.name "Test User"

# Create monorepo structure
mkdir -p packages/core packages/utils

# Create package.json files
cat > package.json << 'EOF'
{
  "name": "test-monorepo",
  "private": true,
  "workspaces": ["packages/*"]
}
EOF

cat > packages/core/package.json << 'EOF'
{
  "name": "@test/core",
  "version": "1.0.0"
}
EOF

cat > packages/utils/package.json << 'EOF'
{
  "name": "@test/utils",
  "version": "1.0.0"
}
EOF

# Initial commit
git add .
git commit -m "initial commit"

# Initialize workspace
workspace init

# Create feature branch and changeset
git checkout -b feature/test-edit
workspace changeset create --bump minor

# Test edit
workspace changeset edit
```

### Full Workflow Test

```bash
# 1. Initialize
workspace init --strategy independent

# 2. Create feature branch
git checkout -b feature/add-logging

# 3. Create changeset
workspace changeset create \
  --bump minor \
  --env dev,staging,prod \
  --packages @test/core \
  --message "Add logging support"

# 4. List changesets
workspace changeset list

# 5. Show details
workspace changeset show feature/add-logging

# 6. Edit changeset
workspace changeset edit

# 7. Update changeset (add more packages)
workspace changeset update \
  --packages @test/utils \
  --bump major

# 8. List again to verify
workspace changeset list

# 9. Edit again
workspace changeset edit
```

## Development Workflow

### Rapid Testing Cycle

```bash
# 1. Make changes to code
vim crates/cli/src/commands/changeset/edit.rs

# 2. Run tests
cargo test --package sublime_cli_tools

# 3. Check clippy
cargo clippy --package sublime_cli_tools -- -D warnings

# 4. Build release
cargo build --package sublime_cli_tools --release

# 5. Test in demo
./scripts/test-in-demo.sh
```

### Testing Specific Features

```bash
# Test only editor module
cargo test --package sublime_cli_tools editor

# Test only changeset commands
cargo test --package sublime_cli_tools changeset

# Test specific function
cargo test --package sublime_cli_tools test_detect_editor
```

### Debug Mode

```bash
# Run with debug logging
RUST_LOG=debug workspace changeset edit

# Run with trace logging
RUST_LOG=trace workspace changeset edit

# Log to file
RUST_LOG=debug workspace changeset edit 2> debug.log
```

## Troubleshooting

### Editor Not Opening

**Problem:** Editor doesn't launch when running `workspace changeset edit`

**Solutions:**
```bash
# Set EDITOR environment variable
export EDITOR=nano

# Or set VISUAL
export VISUAL=vim

# Or specify inline
EDITOR=code workspace changeset edit

# Check available editors
which nano vim vi code
```

### Validation Errors

**Problem:** Changes are rejected after editing

**Cause:** Validation checks failed

**Solutions:**
- Don't change the `branch` field
- Keep at least one package in `packages` array
- Keep at least one environment in `environments` array
- Ensure valid JSON syntax
- Check error message for specific issue

### Binary Not Found

**Problem:** `workspace: command not found`

**Solutions:**
```bash
# Check if installed
which workspace

# Check PATH
echo $PATH

# Add to PATH
export PATH="$HOME/.local/bin:$PATH"

# Or reinstall
./scripts/install-local.sh
```

### Demo Script Fails

**Problem:** Test script exits with error

**Solutions:**
```bash
# Ensure git is configured
git config --global user.email "you@example.com"
git config --global user.name "Your Name"

# Check cargo is available
cargo --version

# Run in verbose mode
bash -x ./scripts/test-in-demo.sh
```

## Environment Variables

The CLI respects these environment variables:

| Variable | Purpose | Example |
|----------|---------|---------|
| `EDITOR` | Default text editor | `export EDITOR=vim` |
| `VISUAL` | Visual editor (higher priority) | `export VISUAL=code` |
| `RUST_LOG` | Logging level | `export RUST_LOG=debug` |

## Performance Testing

### Build Times

```bash
# Measure release build time
time cargo build --package sublime_cli_tools --release

# Measure debug build time
time cargo build --package sublime_cli_tools
```

### Binary Size

```bash
# Check release binary size
ls -lh target/release/workspace

# Check debug binary size
ls -lh target/debug/workspace

# Strip additional symbols (already done in release)
strip target/release/workspace
```

### Runtime Performance

```bash
# Time command execution
time workspace changeset list
time workspace changeset edit

# Profile with perf (Linux)
perf record workspace changeset edit
perf report
```

## Integration Testing

### Test with Real Projects

```bash
# Test in a real monorepo
cd ~/path/to/real-project

# Install CLI
~/.local/bin/workspace --version

# Test workflow
~/.local/bin/workspace init
~/.local/bin/workspace changeset create
~/.local/bin/workspace changeset edit
```

### CI/CD Testing

The CLI can be tested in CI/CD pipelines:

```yaml
# Example GitHub Actions
- name: Build CLI
  run: cargo build --package sublime_cli_tools --release

- name: Install CLI
  run: |
    mkdir -p ~/.local/bin
    cp target/release/workspace ~/.local/bin/
    echo "$HOME/.local/bin" >> $GITHUB_PATH

- name: Test CLI
  run: |
    workspace version
    workspace --help
```

## Next Steps

After testing:

1. ✅ Verify all commands work as expected
2. ✅ Check error messages are clear and helpful
3. ✅ Test edge cases and error conditions
4. ✅ Validate cross-platform compatibility
5. ✅ Document any issues or improvements
6. ✅ Create issues for bugs or enhancements
7. ✅ Update documentation based on findings

## Resources

- [CLI README](crates/cli/README.md)
- [Story Map](crates/cli/STORY_MAP.md)
- [Implementation Plan](crates/cli/PLAN.md)
- [Product Requirements](crates/cli/PRD.md)
- [Scripts Documentation](scripts/README.md)

## Support

For issues or questions:
1. Check this testing guide
2. Review the troubleshooting section
3. Check existing issues in the repository
4. Create a new issue with detailed information