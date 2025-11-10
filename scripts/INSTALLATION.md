# Installation Guide

Complete guide for installing `workspace` (Workspace Node Tools) CLI on all supported platforms.

## Table of Contents

- [Quick Start](#quick-start)
- [Installation Methods](#installation-methods)
- [Platform-Specific Instructions](#platform-specific-instructions)
- [Verification](#verification)
- [Shell Completions](#shell-completions)
- [Updating](#updating)
- [Uninstallation](#uninstallation)
- [Troubleshooting](#troubleshooting)

## Quick Start

### Recommended Installation (All Platforms)

```bash
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh
```

That's it! The script will:
- ✅ Detect your operating system and architecture
- ✅ Download the appropriate pre-built binary
- ✅ Verify the integrity with checksums
- ✅ Install to the best location for your system
- ✅ Set up shell completions

### Verify Installation

```bash
workspace --version
```

## Installation Methods

### Method 1: Curl Script (Recommended)

**Pros**: Fast, automatic, works everywhere  
**Cons**: Requires internet connection

```bash
# Install latest version
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh

# Install specific version
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --version v0.1.0

# Custom installation directory
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --install-dir ~/.local/bin

# Preview before installing
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | less
```

### Method 2: Homebrew (macOS/Linux)

**Pros**: Easy updates, integrated with system package manager  
**Cons**: macOS/Linux only, may lag behind releases

```bash
# Add tap (once)
brew tap websublime/workspace

# Install
brew install workspace

# Update
brew upgrade workspace

# Uninstall
brew uninstall workspace
```

### Method 3: Cargo (From Source)

**Pros**: Always latest code, customizable build  
**Cons**: Requires Rust toolchain, slower installation

```bash
# Install from crates.io
cargo install workspace-cli

# Install from git
cargo install --git https://github.com/websublime/workspace-tools workspace-cli

# Install specific version
cargo install workspace-cli --version 0.1.0
```

### Method 4: Pre-built Binaries

**Pros**: No dependencies, offline installation  
**Cons**: Manual download and setup

```bash
# 1. Download binary for your platform
# Visit: https://github.com/websublime/workspace-tools/releases

# 2. Extract archive
tar -xzf workspace-v0.1.0-x86_64-apple-darwin.tar.gz

# 3. Move to installation directory
sudo mv workspace /usr/local/bin/

# 4. Make executable
sudo chmod +x /usr/local/bin/workspace

# 5. Verify
workspace --version
```

### Method 5: Package Managers

#### APT (Debian/Ubuntu)

```bash
# Note: Package repository coming soon
# Add repository
# curl -fsSL https://packages.example.com/gpg.key | sudo apt-key add -
# echo "deb https://packages.example.com/apt stable main" | sudo tee /etc/apt/sources.list.d/workspace.list

# Install
sudo apt update
sudo apt install workspace

# Update
sudo apt upgrade workspace

# Uninstall
sudo apt remove workspace
```

#### YUM/DNF (RHEL/Fedora/CentOS)

```bash
# Note: Package repository coming soon
# Add repository
# sudo tee /etc/yum.repos.d/workspace.repo <<EOF
# [workspace]
# name=Workspace Node Tools
# baseurl=https://packages.example.com/rpm
# enabled=1
# gpgcheck=1
# gpgkey=https://packages.example.com/gpg.key
# EOF

# Install (RHEL/CentOS)
sudo yum install workspace

# Install (Fedora)
sudo dnf install workspace

# Update
sudo yum update workspace  # or sudo dnf upgrade workspace

# Uninstall
sudo yum remove workspace  # or sudo dnf remove workspace
```

#### Snap (Universal Linux)

```bash
# Install
sudo snap install workspace

# Update
sudo snap refresh workspace

# Uninstall
sudo snap remove workspace
```

## Platform-Specific Instructions

### macOS

#### Intel Macs (x86_64)

```bash
# Automatic installation
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh

# Or via Homebrew
brew install workspace
```

#### Apple Silicon Macs (M1/M2/M3 - aarch64)

```bash
# Automatic installation (detects ARM architecture)
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh

# Or via Homebrew (uses native ARM binary)
brew install workspace
```

#### Rosetta 2 Compatibility

```bash
# workspace runs natively on Apple Silicon
# No Rosetta required

# Verify architecture
file $(which workspace)
# Output: Mach-O 64-bit executable arm64
```

### Linux

#### Ubuntu/Debian

```bash
# Method 1: Install script (recommended)
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh

# Method 2: APT package
sudo apt install workspace

# Method 3: Download binary
wget https://github.com/websublime/workspace-tools/releases/latest/download/workspace-x86_64-unknown-linux-gnu.tar.gz
tar -xzf workspace-x86_64-unknown-linux-gnu.tar.gz
sudo mv workspace /usr/local/bin/
```

#### RHEL/CentOS/Fedora

```bash
# Method 1: Install script (recommended)
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh

# Method 2: YUM/DNF package
sudo yum install workspace

# Method 3: Download binary
curl -LO https://github.com/websublime/workspace-tools/releases/latest/download/workspace-x86_64-unknown-linux-gnu.tar.gz
tar -xzf workspace-x86_64-unknown-linux-gnu.tar.gz
sudo mv workspace /usr/local/bin/
```

#### Arch Linux

```bash
# Method 1: Install script
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh

# Method 2: AUR (coming soon)
yay -S workspace-bin

# Method 3: Manual
curl -LO https://github.com/websublime/workspace-tools/releases/latest/download/workspace-x86_64-unknown-linux-gnu.tar.gz
tar -xzf workspace-x86_64-unknown-linux-gnu.tar.gz
sudo mv workspace /usr/local/bin/
```

#### ARM Linux (Raspberry Pi, etc.)

```bash
# ARMv7 (32-bit)
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh
# Automatically downloads: workspace-armv7-unknown-linux-gnueabihf

# ARMv8/aarch64 (64-bit)
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh
# Automatically downloads: workspace-aarch64-unknown-linux-gnu
```

### Windows

#### Git Bash (Recommended)

```bash
# Install via Git Bash
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh

# Verify
workspace --version
```

#### WSL (Windows Subsystem for Linux)

```bash
# Inside WSL terminal
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh

# Verify
workspace --version
```

#### PowerShell (Native Windows)

```powershell
# Download installer
Invoke-WebRequest -Uri https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.ps1 -OutFile install.ps1

# Run installer
.\install.ps1

# Or download binary manually
Invoke-WebRequest -Uri https://github.com/websublime/workspace-tools/releases/latest/download/workspace-x86_64-pc-windows-msvc.zip -OutFile workspace.zip
Expand-Archive workspace.zip
Move-Item workspace\workspace.exe C:\Program Files\workspace\
```

#### Scoop

```powershell
# Add bucket
scoop bucket add workspace https://github.com/websublime/scoop-workspace

# Install
scoop install workspace

# Update
scoop update workspace

# Uninstall
scoop uninstall workspace
```

## Verification

### Basic Verification

```bash
# Check version
workspace --version

# Check help
workspace --help

# Verify installation path
which workspace

# Check file type
file $(which workspace)
```

### Detailed Verification

```bash
# Run version command (shows dependencies)
workspace version

# Expected output:
# workspace 0.1.0
# rust: 1.75.0
# sublime-package-tools: 0.1.0
# sublime-standard-tools: 0.1.0
# sublime-git-tools: 0.1.0
```

### Test Installation

```bash
# Create test directory
mkdir -p /tmp/workspace-test
cd /tmp/workspace-test

# Initialize a test workspace
workspace init --strategy independent

# Verify config was created
ls -la .workspace.toml

# Clean up
cd ..
rm -rf /tmp/workspace-test
```

## Shell Completions

Completions are automatically installed by the install script.

### Bash

```bash
# Location
~/.local/share/bash-completion/completions/workspace

# If completions don't work, add to ~/.bashrc:
if [ -f ~/.local/share/bash-completion/completions/workspace ]; then
    source ~/.local/share/bash-completion/completions/workspace
fi

# Then reload:
source ~/.bashrc
```

### Zsh

```bash
# Location
~/.local/share/zsh/site-functions/_wnt

# If completions don't work, add to ~/.zshrc:
fpath=(~/.local/share/zsh/site-functions $fpath)
autoload -Uz compinit && compinit

# Then reload:
source ~/.zshrc

# Or force rebuild completion cache:
rm -f ~/.zcompdump*
compinit
```

### Fish

```bash
# Location
~/.config/fish/completions/workspace.fish

# Completions are automatically loaded
# To reload:
fish_update_completions
```

### Manual Generation

```bash
# Generate completions manually
workspace completions bash > ~/.local/share/bash-completion/completions/workspace
workspace completions zsh > ~/.local/share/zsh/site-functions/_wnt
workspace completions fish > ~/.config/fish/completions/workspace.fish
```

## Updating

### Update via Install Script

```bash
# Install latest version (overwrites existing)
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh

# Install specific version
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --version v0.2.0
```

### Update via Package Manager

```bash
# Homebrew
brew upgrade workspace

# APT
sudo apt update && sudo apt upgrade workspace

# YUM/DNF
sudo yum update workspace  # or sudo dnf upgrade workspace

# Cargo
cargo install workspace-cli --force

# Snap
sudo snap refresh workspace

# Scoop
scoop update workspace
```

### Update via Self-Update (Coming Soon)

```bash
# Check for updates
workspace update check

# Update to latest
workspace update apply

# Update to specific version
workspace update apply --version v0.2.0
```

## Uninstallation

### Uninstall via Script

```bash
# Basic uninstall (keeps config)
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/uninstall.sh | sh

# Uninstall and remove config
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/uninstall.sh | sh -s -- --remove-config

# Non-interactive
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/uninstall.sh | sh -s -- --yes --remove-config
```

### Uninstall via Package Manager

```bash
# Homebrew
brew uninstall workspace

# APT
sudo apt remove workspace
sudo apt purge workspace  # Also removes config

# YUM/DNF
sudo yum remove workspace  # or sudo dnf remove workspace

# Cargo
cargo uninstall workspace-cli

# Snap
sudo snap remove workspace

# Scoop
scoop uninstall workspace
```

### Manual Uninstallation

```bash
# 1. Remove binary
sudo rm /usr/local/bin/workspace
# or
rm ~/.local/bin/workspace

# 2. Remove completions
rm ~/.local/share/bash-completion/completions/workspace
rm ~/.local/share/zsh/site-functions/_wnt
rm ~/.config/fish/completions/workspace.fish

# 3. Remove config (optional)
rm -rf ~/.config/workspace
rm ~/.workspace.toml
```

## Troubleshooting

### Installation Fails

#### "Platform not supported"

```bash
# Check your platform
uname -s  # Should be: Darwin, Linux, or MINGW*/MSYS*/CYGWIN*
uname -m  # Should be: x86_64, aarch64, arm64, or armv7l

# If unsupported, build from source:
cargo install --git https://github.com/websublime/workspace-tools workspace-cli
```

#### "Download failed"

```bash
# Check internet connection
curl -I https://github.com

# Check if URL is accessible
curl -I https://github.com/websublime/workspace-tools/releases

# Use verbose mode for more details
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --verbose

# For corporate networks, check proxy settings
export HTTP_PROXY=http://proxy.company.com:8080
export HTTPS_PROXY=http://proxy.company.com:8080
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh
```

#### "Checksum verification failed"

```bash
# Download may be corrupted, try again
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh

# Download manually and verify
wget https://github.com/websublime/workspace-tools/releases/download/v0.1.0/workspace-x86_64-apple-darwin.tar.gz
wget https://github.com/websublime/workspace-tools/releases/download/v0.1.0/checksums.txt

# Verify manually
grep "workspace-x86_64-apple-darwin.tar.gz" checksums.txt
sha256sum workspace-x86_64-apple-darwin.tar.gz  # Linux
shasum -a 256 workspace-x86_64-apple-darwin.tar.gz  # macOS
```

#### "Permission denied"

```bash
# Option 1: Use default user directory
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --install-dir ~/.local/bin

# Option 2: Use sudo for system-wide installation
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sudo sh

# Option 3: Change permissions of target directory
sudo chown -R $USER /usr/local/bin
```

### Binary Not Found After Installation

```bash
# Check if binary exists
ls -l ~/.local/bin/workspace
ls -l /usr/local/bin/workspace

# Check PATH
echo $PATH

# Add to PATH if missing (add to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/.local/bin:$PATH"
export PATH="/usr/local/bin:$PATH"

# Reload shell
source ~/.bashrc  # or source ~/.zshrc

# Or use full path
~/.local/bin/workspace --version
```

### Command Not Found

```bash
# Verify installation
which workspace

# If not found, check common locations
ls -l /usr/local/bin/workspace
ls -l ~/.local/bin/workspace
ls -l ~/bin/workspace

# Add to PATH
export PATH="$HOME/.local/bin:$PATH"

# Make permanent (add to shell config)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Completions Don't Work

```bash
# Bash
source ~/.local/share/bash-completion/completions/workspace

# Zsh
rm -f ~/.zcompdump*
autoload -Uz compinit && compinit

# Fish
fish_update_completions

# Regenerate completions
workspace completions bash > ~/.local/share/bash-completion/completions/workspace
workspace completions zsh > ~/.local/share/zsh/site-functions/_wnt
workspace completions fish > ~/.config/fish/completions/workspace.fish
```

### Old Version After Update

```bash
# Clear shell hash cache
hash -r

# Or restart shell
exec $SHELL

# Verify version
workspace --version

# Check if multiple installations exist
which -a workspace
```

## Advanced Installation Options

### Custom Installation Directory

```bash
# Install to custom directory
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --install-dir ~/my-tools/bin

# Add to PATH
export PATH="$HOME/my-tools/bin:$PATH"
```

### Skip Shell Completions

```bash
# Install without completions
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --no-shell-completions
```

### Offline Installation

```bash
# 1. Download on machine with internet
wget https://github.com/websublime/workspace-tools/releases/download/v0.1.0/workspace-x86_64-unknown-linux-gnu.tar.gz
wget https://github.com/websublime/workspace-tools/releases/download/v0.1.0/checksums.txt

# 2. Transfer files to offline machine

# 3. Verify checksum
grep "workspace-x86_64-unknown-linux-gnu.tar.gz" checksums.txt
sha256sum workspace-x86_64-unknown-linux-gnu.tar.gz

# 4. Extract and install
tar -xzf workspace-x86_64-unknown-linux-gnu.tar.gz
sudo mv workspace /usr/local/bin/
sudo chmod +x /usr/local/bin/workspace
```

### Corporate/Proxy Environment

```bash
# Set proxy
export HTTP_PROXY=http://proxy.company.com:8080
export HTTPS_PROXY=http://proxy.company.com:8080

# Install
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh

# Or use proxy with curl
curl -x http://proxy.company.com:8080 -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh
```

### Private Repository

```bash
# Set GitHub token
export WORKSPACE_GITHUB_TOKEN="ghp_your_token_here"

# Install
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh
```

## Next Steps

After installation:

1. **Initialize a workspace**
   ```bash
   cd /path/to/your/project
   workspace init
   ```

2. **Read the documentation**
   ```bash
   workspace --help
   workspace init --help
   workspace changeset --help
   ```

3. **Create your first changeset**
   ```bash
   workspace changeset add
   ```

4. **Explore commands**
   ```bash
   workspace config show
   workspace changes
   workspace audit
   ```

## Support

- **Documentation**: https://github.com/websublime/workspace-tools/docs
- **GitHub Issues**: https://github.com/websublime/workspace-tools/issues
- **Discussions**: https://github.com/websublime/workspace-tools/discussions
- **Discord**: https://discord.gg/workspace (coming soon)

## License

MIT or Apache-2.0
