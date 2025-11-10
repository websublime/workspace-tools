# Scripts Directory

Utility scripts for building, installing, testing, and managing the `workspace` CLI.

## Table of Contents

- [Installation Scripts](#installation-scripts)
  - [install.sh - Production Installation](#-installsh---production-installation)
  - [install-dev.sh - Development Installation](#-install-devsh---development-installation)
  - [uninstall.sh - Uninstallation](#-uninstallsh---uninstallation)
- [Testing Scripts](#testing-scripts)
  - [test-in-demo.sh - Interactive Testing](#-test-in-demosh---interactive-testing)
- [Quick Start](#quick-start)
- [Development Workflow](#development-workflow)
- [Troubleshooting](#troubleshooting)

## Installation Scripts

### 🚀 `install.sh` - Production Installation

Official installation script for `workspace` CLI. Downloads pre-built binaries from GitHub releases, verifies checksums, and installs system-wide or locally.

**Usage:**
```bash
# Install latest version
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh

# Install specific version
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --version v0.1.0

# Custom installation directory
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --install-dir ~/.local/bin

# Skip shell completions
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --no-shell-completions

# Verbose output
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --verbose

# Show help
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --help
```

**Options:**
- `--version <VERSION>` - Install specific version (default: latest)
- `--install-dir <DIR>` - Custom installation directory
- `--no-shell-completions` - Skip shell completions installation
- `--no-color` - Disable colored output
- `--verbose` - Enable verbose output
- `--help` - Show help message

**Environment Variables:**
- `WORKSPACE_VERSION` - Version to install (overridden by `--version`)
- `WORKSPACE_INSTALL_DIR` - Installation directory (overridden by `--install-dir`)
- `WORKSPACE_GITHUB_TOKEN` - GitHub token for private repositories
- `NO_COLOR` - Disable colored output

**What it does:**
1. Detects operating system (macOS, Linux, Windows via Git Bash)
2. Detects architecture (x86_64, aarch64, arm)
3. Downloads appropriate pre-built binary from GitHub releases
4. Downloads and verifies SHA256 checksum
5. Extracts binary from archive
6. Installs to `/usr/local/bin` (with sudo if needed) or `~/.local/bin`
7. Installs shell completions (bash, zsh, fish)
8. Shows post-installation instructions

**Supported Platforms:**
- macOS (Intel: x86_64, Apple Silicon: aarch64)
- Linux (x86_64, aarch64, armv7)
- Windows via Git Bash/WSL (x86_64)

**Exit Codes:**
- `0` - Success
- `1` - General error
- `2` - Invalid usage
- `3` - Platform not supported
- `4` - Download failed
- `5` - Checksum verification failed
- `6` - Installation failed

### 📦 `install-dev.sh` - Development Installation

Install the `workspace` CLI binary locally for development and testing. Builds from source and installs to specified location.

**Usage:**
```bash
# Install to default location (~/.local/bin)
./scripts/install-dev.sh

# Install to custom location
./scripts/install-dev.sh ~/bin

# Install system-wide (requires sudo)
sudo ./scripts/install-dev.sh /usr/local/bin
```

**What it does:**
1. Checks for existing release binary
2. Builds release binary if not present (`cargo build --release`)
3. Copies binary to specified location
4. Makes it executable
5. Shows installation status and usage examples
6. Warns if install location is not in PATH

**When to use:**
- During active development
- Testing local changes
- When you have the source code
- When pre-built binaries are not available

### 🗑️ `uninstall.sh` - Uninstallation

Removes `workspace` CLI binary, shell completions, and optionally configuration files.

**Usage:**
```bash
# Basic uninstall
./scripts/uninstall.sh

# Uninstall with configuration removal
./scripts/uninstall.sh --remove-config

# Uninstall from custom directory
./scripts/uninstall.sh --install-dir ~/.local/bin

# Non-interactive uninstall
./scripts/uninstall.sh --yes --remove-config

# Show help
./scripts/uninstall.sh --help
```

**Options:**
- `--remove-config` - Remove configuration files
- `--install-dir <DIR>` - Custom installation directory to remove from
- `--no-color` - Disable colored output
- `--verbose` - Enable verbose output
- `--yes` - Skip confirmation prompts
- `--help` - Show help message

**Environment Variables:**
- `WORKSPACE_INSTALL_DIR` - Installation directory (overridden by `--install-dir`)
- `NO_COLOR` - Disable colored output

**What it does:**
1. Searches for `workspace` binary in common locations
2. Displays found installation
3. Asks for confirmation (unless `--yes` is used)
4. Removes binary (with sudo if needed)
5. Removes shell completions (bash, zsh, fish)
6. Optionally removes configuration files (if `--remove-config`)
7. Shows completion message

**Configuration Files Searched:**
- Project-level: `.workspace.toml`, `.wntrc`, `workspace.config.json` (in current directory and parents)
- User-level: `~/.config/workspace/config.toml`, `~/.workspace.toml`

**Exit Codes:**
- `0` - Success
- `1` - General error
- `2` - Invalid usage
- `3` - Binary not found

## Testing Scripts

### 🧪 `test-in-demo.sh` - Interactive Testing

Create a temporary demo repository and test the CLI interactively.

**Usage:**
```bash
./scripts/test-in-demo.sh
```

**What it does:**
1. Builds the release binary
2. Creates a temporary monorepo structure
3. Initializes git repository
4. Runs `workspace init` to setup workspace
5. Creates a feature branch
6. Creates a sample changeset
7. Lists and shows changesets
8. Provides instructions for testing edit command
9. Keeps demo repo available for manual testing
10. Cleans up on exit (Ctrl+C)

**Perfect for:**
- Testing the edit command interactively
- Verifying end-to-end workflows
- Quick manual testing without affecting real projects
- Demonstrating CLI features

## Quick Start

### For End Users (Production)

```bash
# Install latest version
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh

# Verify installation
workspace version

# Initialize your workspace
cd /path/to/your/project
workspace init
```

### For Developers (Development)

```bash
# 1. Clone repository
git clone https://github.com/websublime/workspace-tools.git
cd workspace-tools

# 2. Build and install locally
./scripts/install-dev.sh

# 3. Verify installation
workspace version

# 4. Test in demo repository
./scripts/test-in-demo.sh
```

### Testing in Demo Repository

```bash
# Run the demo script
./scripts/test-in-demo.sh

# In another terminal (while demo is running)
cd /tmp/workspace-demo-XXXXXX  # Use the path shown in output
workspace changeset edit
```

## Development Workflow

### Making Changes and Testing

```bash
# 1. Make changes to CLI code
vim crates/cli/src/commands/changeset/edit.rs

# 2. Build release
cargo build --package sublime_cli_tools --release

# 3. Install locally
./scripts/install-dev.sh

# 4. Test in demo
./scripts/test-in-demo.sh

# 5. Or test in real project
cd ~/my-real-project
workspace changeset edit
```

### Rapid Iteration

```bash
# Watch and rebuild on changes (requires cargo-watch)
cargo watch -x 'build --package sublime_cli_tools --release' -s './scripts/install-dev.sh'

# In another terminal, test your changes
workspace --help
```

### Manual Build and Copy

If you prefer to build and copy manually:

```bash
# Build release binary
cargo build --package sublime_cli_tools --release

# Binary location
ls -lh target/release/workspace

# Copy to your desired location
cp target/release/workspace ~/bin/workspace
# or
cp target/release/workspace ~/.local/bin/workspace
# or
sudo cp target/release/workspace /usr/local/bin/workspace

# Make executable
chmod +x ~/bin/workspace
```

## Testing the Edit Command

### Option 1: Using test-in-demo.sh (Recommended)

```bash
# Start the demo
./scripts/test-in-demo.sh

# In another terminal (follow instructions from script output)
cd /tmp/workspace-demo-XXXXXX
export EDITOR=vim  # or your preferred editor
workspace changeset edit

# The changeset file will open in your editor
# Make changes, save, and exit
# The command validates your changes automatically
```

### Option 2: In Your Own Test Repository

```bash
# Install CLI
./scripts/install-dev.sh

# Navigate to your test repo
cd /path/to/test-repo

# Initialize workspace
workspace init

# Create a changeset
git checkout -b feature/test
workspace changeset create --bump minor --env dev

# Edit the changeset
workspace changeset edit

# Or specify branch
workspace changeset edit feature/test
```

## Environment Variables

The scripts respect these environment variables:

### Installation Scripts
- `WORKSPACE_VERSION` - Version to install (install.sh)
- `WORKSPACE_INSTALL_DIR` - Installation directory (install.sh, uninstall.sh)
- `WORKSPACE_GITHUB_TOKEN` - GitHub token for private repositories (install.sh)
- `NO_COLOR` - Disable colored output (all scripts)

### CLI Usage
- `EDITOR` - Your preferred text editor (used by `workspace changeset edit`)
- `VISUAL` - Alternative editor specification (higher priority than EDITOR)

## Binary Information

### Release Binary
The release binary is optimized for size and performance:
- **Size**: ~7-10MB (varies by platform)
- **Strip**: Debug symbols removed
- **LTO**: Link Time Optimization enabled
- **Optimization**: Level 3 (or z for size)

### Build Configuration
From `Cargo.toml`:
```toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Enable Link Time Optimization
codegen-units = 1    # Better optimization
strip = true         # Strip symbols
panic = "abort"      # Smaller binary
```

## Troubleshooting

### Installation Issues

#### "Platform not supported" error
```bash
# Check your platform
uname -s  # OS
uname -m  # Architecture

# Supported platforms:
# - macOS: x86_64, aarch64
# - Linux: x86_64, aarch64, armv7
# - Windows: x86_64 (via Git Bash/WSL)
```

#### "Download failed" error
```bash
# Check internet connection
curl -I https://github.com

# Use verbose mode to see details
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --verbose

# For private repositories, provide GitHub token
export WORKSPACE_GITHUB_TOKEN="your_token_here"
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh
```

#### "Checksum verification failed" error
```bash
# Download may be corrupted, try again
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh

# If problem persists, report issue with verbose output
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --verbose
```

### Development Issues

#### "Binary not found" error

Run the install script, it will build automatically:
```bash
./scripts/install-dev.sh
```

#### "Command not found: workspace"

Add the installation directory to your PATH:
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.local/bin:$PATH"

# Reload shell
source ~/.bashrc  # or source ~/.zshrc
```

Or verify where workspace is installed:
```bash
which workspace
ls -l ~/.local/bin/workspace
ls -l /usr/local/bin/workspace
```

#### Editor doesn't open

Set your EDITOR environment variable:
```bash
export EDITOR=nano  # or vim, code, emacs, etc.
workspace changeset edit
```

#### Demo script fails

Ensure you have required tools:
```bash
# Check git
git --version

# Check jq (optional, for JSON formatting)
jq --version

# Check cargo
cargo --version

# Check Rust
rustc --version
```

### Uninstallation Issues

#### "Binary not found" error

The binary may already be uninstalled or installed in a custom location:
```bash
# Search for workspace in common locations
which workspace
find ~ -name workspace -type f 2>/dev/null

# Uninstall from custom directory
./scripts/uninstall.sh --install-dir /path/to/custom/dir
```

#### Permission denied errors

Use sudo for system-wide installations:
```bash
sudo ./scripts/uninstall.sh --install-dir /usr/local/bin
```

## Shell Completions

Shell completions are automatically installed by `install.sh` to:

### Bash
```bash
# Location
~/.local/share/bash-completion/completions/workspace

# Reload completions
source ~/.bashrc
```

### Zsh
```bash
# Location
~/.local/share/zsh/site-functions/_wnt

# Reload completions
source ~/.zshrc
# or
rm -f ~/.zcompdump; compinit
```

### Fish
```bash
# Location
~/.config/fish/completions/workspace.fish

# Completions are automatically loaded
```

### Manual Generation

If you need to regenerate completions:
```bash
# Bash
workspace completions bash > ~/.local/share/bash-completion/completions/workspace

# Zsh
workspace completions zsh > ~/.local/share/zsh/site-functions/_wnt

# Fish
workspace completions fish > ~/.config/fish/completions/workspace.fish
```

## Notes

### Security Considerations
- The install script verifies SHA256 checksums to prevent tampering
- Always use HTTPS when downloading scripts
- Review scripts before piping to shell: `curl -fsSL <url> | less`
- For production, pin specific versions: `--version v0.1.0`

### Best Practices
- Use `install.sh` for production installations (stable, tested binaries)
- Use `install-dev.sh` for development (latest code, frequent changes)
- Use `test-in-demo.sh` for testing features without affecting real projects
- Always backup configuration before using `uninstall.sh --remove-config`

### Platform-Specific Notes
- **macOS**: Uses Apple's `shasum` for checksum verification
- **Linux**: Uses GNU `sha256sum` for checksum verification
- **Windows**: Use Git Bash or WSL for best compatibility
- **ARM**: Supported on both macOS (Apple Silicon) and Linux

### Performance
- Installation script is optimized for minimal output (unless `--verbose`)
- Binary downloads are typically 5-10MB compressed
- Installation completes in seconds on modern hardware
- Shell completions improve CLI usability

### Cleanup
- Temporary files are automatically cleaned up on script exit
- The demo script cleans up on exit (Ctrl+C)
- Uninstall script only removes workspace-related files, never user data (unless `--remove-config`)

## Contributing

When modifying installation scripts:

1. **Test on all platforms**: macOS (Intel & Apple Silicon), Linux (multiple distros), Windows (Git Bash/WSL)
2. **Verify exit codes**: Ensure proper exit codes are used
3. **Update documentation**: Keep this README in sync with script changes
4. **Test error scenarios**: Network failures, permission issues, missing dependencies
5. **Follow shell best practices**: Use `set -e`, quote variables, handle errors gracefully
6. **Maintain POSIX compatibility**: Use `sh` not `bash` for maximum compatibility (except install-dev.sh)

## Additional Resources

- [Main README](../README.md) - Project overview
- [PRD](../crates/cli/PRD.md) - Product requirements
- [PLAN](../crates/cli/PLAN.md) - Implementation plan
- [STORY_MAP](../crates/cli/STORY_MAP.md) - Development roadmap
- [Website](https://github.com/websublime/workspace-tools) - Official documentation
- [GitHub](https://github.com/websublime/workspace-tools) - Source code and releases
