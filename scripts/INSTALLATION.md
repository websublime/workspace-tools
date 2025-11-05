# Installation Guide

Complete guide for installing `wnt` (Workspace Node Tools) CLI on all supported platforms.

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
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh
```

That's it! The script will:
- ✅ Detect your operating system and architecture
- ✅ Download the appropriate pre-built binary
- ✅ Verify the integrity with checksums
- ✅ Install to the best location for your system
- ✅ Set up shell completions

### Verify Installation

```bash
wnt --version
```

## Installation Methods

### Method 1: Curl Script (Recommended)

**Pros**: Fast, automatic, works everywhere  
**Cons**: Requires internet connection

```bash
# Install latest version
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh

# Install specific version
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh -s -- --version v0.1.0

# Custom installation directory
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh -s -- --install-dir ~/.local/bin

# Preview before installing
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | less
```

### Method 2: Homebrew (macOS/Linux)

**Pros**: Easy updates, integrated with system package manager  
**Cons**: macOS/Linux only, may lag behind releases

```bash
# Add tap (once)
brew tap websublime/wnt

# Install
brew install wnt

# Update
brew upgrade wnt

# Uninstall
brew uninstall wnt
```

### Method 3: Cargo (From Source)

**Pros**: Always latest code, customizable build  
**Cons**: Requires Rust toolchain, slower installation

```bash
# Install from crates.io
cargo install wnt-cli

# Install from git
cargo install --git https://github.com/websublime/workspace-node-tools wnt-cli

# Install specific version
cargo install wnt-cli --version 0.1.0
```

### Method 4: Pre-built Binaries

**Pros**: No dependencies, offline installation  
**Cons**: Manual download and setup

```bash
# 1. Download binary for your platform
# Visit: https://github.com/websublime/workspace-node-tools/releases

# 2. Extract archive
tar -xzf wnt-v0.1.0-x86_64-apple-darwin.tar.gz

# 3. Move to installation directory
sudo mv wnt /usr/local/bin/

# 4. Make executable
sudo chmod +x /usr/local/bin/wnt

# 5. Verify
wnt --version
```

### Method 5: Package Managers

#### APT (Debian/Ubuntu)

```bash
# Note: Package repository coming soon
# Add repository
# curl -fsSL https://packages.example.com/gpg.key | sudo apt-key add -
# echo "deb https://packages.example.com/apt stable main" | sudo tee /etc/apt/sources.list.d/wnt.list

# Install
sudo apt update
sudo apt install wnt

# Update
sudo apt upgrade wnt

# Uninstall
sudo apt remove wnt
```

#### YUM/DNF (RHEL/Fedora/CentOS)

```bash
# Note: Package repository coming soon
# Add repository
# sudo tee /etc/yum.repos.d/wnt.repo <<EOF
# [wnt]
# name=Workspace Node Tools
# baseurl=https://packages.example.com/rpm
# enabled=1
# gpgcheck=1
# gpgkey=https://packages.example.com/gpg.key
# EOF

# Install (RHEL/CentOS)
sudo yum install wnt

# Install (Fedora)
sudo dnf install wnt

# Update
sudo yum update wnt  # or sudo dnf upgrade wnt

# Uninstall
sudo yum remove wnt  # or sudo dnf remove wnt
```

#### Snap (Universal Linux)

```bash
# Install
sudo snap install wnt

# Update
sudo snap refresh wnt

# Uninstall
sudo snap remove wnt
```

## Platform-Specific Instructions

### macOS

#### Intel Macs (x86_64)

```bash
# Automatic installation
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh

# Or via Homebrew
brew install wnt
```

#### Apple Silicon Macs (M1/M2/M3 - aarch64)

```bash
# Automatic installation (detects ARM architecture)
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh

# Or via Homebrew (uses native ARM binary)
brew install wnt
```

#### Rosetta 2 Compatibility

```bash
# wnt runs natively on Apple Silicon
# No Rosetta required

# Verify architecture
file $(which wnt)
# Output: Mach-O 64-bit executable arm64
```

### Linux

#### Ubuntu/Debian

```bash
# Method 1: Install script (recommended)
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh

# Method 2: APT package
sudo apt install wnt

# Method 3: Download binary
wget https://github.com/websublime/workspace-node-tools/releases/latest/download/wnt-x86_64-unknown-linux-gnu.tar.gz
tar -xzf wnt-x86_64-unknown-linux-gnu.tar.gz
sudo mv wnt /usr/local/bin/
```

#### RHEL/CentOS/Fedora

```bash
# Method 1: Install script (recommended)
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh

# Method 2: YUM/DNF package
sudo yum install wnt

# Method 3: Download binary
curl -LO https://github.com/websublime/workspace-node-tools/releases/latest/download/wnt-x86_64-unknown-linux-gnu.tar.gz
tar -xzf wnt-x86_64-unknown-linux-gnu.tar.gz
sudo mv wnt /usr/local/bin/
```

#### Arch Linux

```bash
# Method 1: Install script
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh

# Method 2: AUR (coming soon)
yay -S wnt-bin

# Method 3: Manual
curl -LO https://github.com/websublime/workspace-node-tools/releases/latest/download/wnt-x86_64-unknown-linux-gnu.tar.gz
tar -xzf wnt-x86_64-unknown-linux-gnu.tar.gz
sudo mv wnt /usr/local/bin/
```

#### ARM Linux (Raspberry Pi, etc.)

```bash
# ARMv7 (32-bit)
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh
# Automatically downloads: wnt-armv7-unknown-linux-gnueabihf

# ARMv8/aarch64 (64-bit)
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh
# Automatically downloads: wnt-aarch64-unknown-linux-gnu
```

### Windows

#### Git Bash (Recommended)

```bash
# Install via Git Bash
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh

# Verify
wnt --version
```

#### WSL (Windows Subsystem for Linux)

```bash
# Inside WSL terminal
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh

# Verify
wnt --version
```

#### PowerShell (Native Windows)

```powershell
# Download installer
Invoke-WebRequest -Uri https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.ps1 -OutFile install.ps1

# Run installer
.\install.ps1

# Or download binary manually
Invoke-WebRequest -Uri https://github.com/websublime/workspace-node-tools/releases/latest/download/wnt-x86_64-pc-windows-msvc.zip -OutFile wnt.zip
Expand-Archive wnt.zip
Move-Item wnt\wnt.exe C:\Program Files\wnt\
```

#### Scoop

```powershell
# Add bucket
scoop bucket add wnt https://github.com/websublime/scoop-wnt

# Install
scoop install wnt

# Update
scoop update wnt

# Uninstall
scoop uninstall wnt
```

## Verification

### Basic Verification

```bash
# Check version
wnt --version

# Check help
wnt --help

# Verify installation path
which wnt

# Check file type
file $(which wnt)
```

### Detailed Verification

```bash
# Run version command (shows dependencies)
wnt version

# Expected output:
# wnt 0.1.0
# rust: 1.75.0
# sublime-package-tools: 0.1.0
# sublime-standard-tools: 0.1.0
# sublime-git-tools: 0.1.0
```

### Test Installation

```bash
# Create test directory
mkdir -p /tmp/wnt-test
cd /tmp/wnt-test

# Initialize a test workspace
wnt init --strategy independent

# Verify config was created
ls -la .wnt.toml

# Clean up
cd ..
rm -rf /tmp/wnt-test
```

## Shell Completions

Completions are automatically installed by the install script.

### Bash

```bash
# Location
~/.local/share/bash-completion/completions/wnt

# If completions don't work, add to ~/.bashrc:
if [ -f ~/.local/share/bash-completion/completions/wnt ]; then
    source ~/.local/share/bash-completion/completions/wnt
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
~/.config/fish/completions/wnt.fish

# Completions are automatically loaded
# To reload:
fish_update_completions
```

### Manual Generation

```bash
# Generate completions manually
wnt completions bash > ~/.local/share/bash-completion/completions/wnt
wnt completions zsh > ~/.local/share/zsh/site-functions/_wnt
wnt completions fish > ~/.config/fish/completions/wnt.fish
```

## Updating

### Update via Install Script

```bash
# Install latest version (overwrites existing)
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh

# Install specific version
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh -s -- --version v0.2.0
```

### Update via Package Manager

```bash
# Homebrew
brew upgrade wnt

# APT
sudo apt update && sudo apt upgrade wnt

# YUM/DNF
sudo yum update wnt  # or sudo dnf upgrade wnt

# Cargo
cargo install wnt-cli --force

# Snap
sudo snap refresh wnt

# Scoop
scoop update wnt
```

### Update via Self-Update (Coming Soon)

```bash
# Check for updates
wnt update check

# Update to latest
wnt update apply

# Update to specific version
wnt update apply --version v0.2.0
```

## Uninstallation

### Uninstall via Script

```bash
# Basic uninstall (keeps config)
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/uninstall.sh | sh

# Uninstall and remove config
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/uninstall.sh | sh -s -- --remove-config

# Non-interactive
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/uninstall.sh | sh -s -- --yes --remove-config
```

### Uninstall via Package Manager

```bash
# Homebrew
brew uninstall wnt

# APT
sudo apt remove wnt
sudo apt purge wnt  # Also removes config

# YUM/DNF
sudo yum remove wnt  # or sudo dnf remove wnt

# Cargo
cargo uninstall wnt-cli

# Snap
sudo snap remove wnt

# Scoop
scoop uninstall wnt
```

### Manual Uninstallation

```bash
# 1. Remove binary
sudo rm /usr/local/bin/wnt
# or
rm ~/.local/bin/wnt

# 2. Remove completions
rm ~/.local/share/bash-completion/completions/wnt
rm ~/.local/share/zsh/site-functions/_wnt
rm ~/.config/fish/completions/wnt.fish

# 3. Remove config (optional)
rm -rf ~/.config/wnt
rm ~/.wnt.toml
```

## Troubleshooting

### Installation Fails

#### "Platform not supported"

```bash
# Check your platform
uname -s  # Should be: Darwin, Linux, or MINGW*/MSYS*/CYGWIN*
uname -m  # Should be: x86_64, aarch64, arm64, or armv7l

# If unsupported, build from source:
cargo install --git https://github.com/websublime/workspace-node-tools wnt-cli
```

#### "Download failed"

```bash
# Check internet connection
curl -I https://github.com

# Check if URL is accessible
curl -I https://github.com/websublime/workspace-node-tools/releases

# Use verbose mode for more details
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh -s -- --verbose

# For corporate networks, check proxy settings
export HTTP_PROXY=http://proxy.company.com:8080
export HTTPS_PROXY=http://proxy.company.com:8080
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh
```

#### "Checksum verification failed"

```bash
# Download may be corrupted, try again
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh

# Download manually and verify
wget https://github.com/websublime/workspace-node-tools/releases/download/v0.1.0/wnt-x86_64-apple-darwin.tar.gz
wget https://github.com/websublime/workspace-node-tools/releases/download/v0.1.0/checksums.txt

# Verify manually
grep "wnt-x86_64-apple-darwin.tar.gz" checksums.txt
sha256sum wnt-x86_64-apple-darwin.tar.gz  # Linux
shasum -a 256 wnt-x86_64-apple-darwin.tar.gz  # macOS
```

#### "Permission denied"

```bash
# Option 1: Use default user directory
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh -s -- --install-dir ~/.local/bin

# Option 2: Use sudo for system-wide installation
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sudo sh

# Option 3: Change permissions of target directory
sudo chown -R $USER /usr/local/bin
```

### Binary Not Found After Installation

```bash
# Check if binary exists
ls -l ~/.local/bin/wnt
ls -l /usr/local/bin/wnt

# Check PATH
echo $PATH

# Add to PATH if missing (add to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/.local/bin:$PATH"
export PATH="/usr/local/bin:$PATH"

# Reload shell
source ~/.bashrc  # or source ~/.zshrc

# Or use full path
~/.local/bin/wnt --version
```

### Command Not Found

```bash
# Verify installation
which wnt

# If not found, check common locations
ls -l /usr/local/bin/wnt
ls -l ~/.local/bin/wnt
ls -l ~/bin/wnt

# Add to PATH
export PATH="$HOME/.local/bin:$PATH"

# Make permanent (add to shell config)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Completions Don't Work

```bash
# Bash
source ~/.local/share/bash-completion/completions/wnt

# Zsh
rm -f ~/.zcompdump*
autoload -Uz compinit && compinit

# Fish
fish_update_completions

# Regenerate completions
wnt completions bash > ~/.local/share/bash-completion/completions/wnt
wnt completions zsh > ~/.local/share/zsh/site-functions/_wnt
wnt completions fish > ~/.config/fish/completions/wnt.fish
```

### Old Version After Update

```bash
# Clear shell hash cache
hash -r

# Or restart shell
exec $SHELL

# Verify version
wnt --version

# Check if multiple installations exist
which -a wnt
```

## Advanced Installation Options

### Custom Installation Directory

```bash
# Install to custom directory
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh -s -- --install-dir ~/my-tools/bin

# Add to PATH
export PATH="$HOME/my-tools/bin:$PATH"
```

### Skip Shell Completions

```bash
# Install without completions
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh -s -- --no-shell-completions
```

### Offline Installation

```bash
# 1. Download on machine with internet
wget https://github.com/websublime/workspace-node-tools/releases/download/v0.1.0/wnt-x86_64-unknown-linux-gnu.tar.gz
wget https://github.com/websublime/workspace-node-tools/releases/download/v0.1.0/checksums.txt

# 2. Transfer files to offline machine

# 3. Verify checksum
grep "wnt-x86_64-unknown-linux-gnu.tar.gz" checksums.txt
sha256sum wnt-x86_64-unknown-linux-gnu.tar.gz

# 4. Extract and install
tar -xzf wnt-x86_64-unknown-linux-gnu.tar.gz
sudo mv wnt /usr/local/bin/
sudo chmod +x /usr/local/bin/wnt
```

### Corporate/Proxy Environment

```bash
# Set proxy
export HTTP_PROXY=http://proxy.company.com:8080
export HTTPS_PROXY=http://proxy.company.com:8080

# Install
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh

# Or use proxy with curl
curl -x http://proxy.company.com:8080 -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh
```

### Private Repository

```bash
# Set GitHub token
export WNT_GITHUB_TOKEN="ghp_your_token_here"

# Install
curl -fsSL https://raw.githubusercontent.com/websublime/workspace-node-tools/main/scripts/install.sh | sh
```

## Next Steps

After installation:

1. **Initialize a workspace**
   ```bash
   cd /path/to/your/project
   wnt init
   ```

2. **Read the documentation**
   ```bash
   wnt --help
   wnt init --help
   wnt changeset --help
   ```

3. **Create your first changeset**
   ```bash
   wnt changeset add
   ```

4. **Explore commands**
   ```bash
   wnt config show
   wnt changes
   wnt audit
   ```

## Support

- **Documentation**: https://github.com/websublime/workspace-node-tools/docs
- **GitHub Issues**: https://github.com/websublime/workspace-node-tools/issues
- **Discussions**: https://github.com/websublime/workspace-node-tools/discussions
- **Discord**: https://discord.gg/wnt (coming soon)

## License

MIT or Apache-2.0
