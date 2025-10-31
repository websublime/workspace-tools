# Scripts Directory

Utility scripts for building, installing, and testing the `wnt` CLI.

## Scripts

### 📦 `install-local.sh`

Install the `wnt` CLI binary locally for testing.

**Usage:**
```bash
# Install to default location (~/.local/bin)
./scripts/install-local.sh

# Install to custom location
./scripts/install-local.sh ~/bin

# Install system-wide (requires sudo)
sudo ./scripts/install-local.sh /usr/local/bin
```

**What it does:**
1. Builds the release binary if not present
2. Copies binary to specified location
3. Makes it executable
4. Shows installation status and usage examples
5. Warns if install location is not in PATH

### 🧪 `test-in-demo.sh`

Create a temporary demo repository and test the CLI interactively.

**Usage:**
```bash
./scripts/test-in-demo.sh
```

**What it does:**
1. Builds the release binary
2. Creates a temporary monorepo structure
3. Initializes git repository
4. Runs `wnt init` to setup workspace
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

### Build and Install Locally

```bash
# 1. Build release binary
cargo build --package sublime_cli_tools --release

# 2. Install to ~/.local/bin
./scripts/install-local.sh

# 3. Verify installation
wnt version
```

### Test in Demo Repository

```bash
# Run the demo script
./scripts/test-in-demo.sh

# In another terminal (while demo is running)
cd /tmp/wnt-demo-XXXXXX  # Use the path shown in output
wnt changeset edit
```

## Manual Build and Copy

If you prefer to build and copy manually:

```bash
# Build release binary
cargo build --package sublime_cli_tools --release

# Binary location
ls -lh target/release/wnt

# Copy to your desired location
cp target/release/wnt ~/bin/wnt
# or
cp target/release/wnt ~/.local/bin/wnt
# or
sudo cp target/release/wnt /usr/local/bin/wnt

# Make executable
chmod +x ~/bin/wnt
```

## Testing the Edit Command

### Option 1: Using test-in-demo.sh (Recommended)

```bash
# Start the demo
./scripts/test-in-demo.sh

# In another terminal (follow instructions from script output)
cd /tmp/wnt-demo-XXXXXX
export EDITOR=vim  # or your preferred editor
wnt changeset edit

# The changeset file will open in your editor
# Make changes, save, and exit
# The command validates your changes automatically
```

### Option 2: In Your Own Test Repository

```bash
# Install CLI
./scripts/install-local.sh

# Navigate to your test repo
cd /path/to/test-repo

# Initialize workspace
wnt init

# Create a changeset
git checkout -b feature/test
wnt changeset create --bump minor --env dev

# Edit the changeset
wnt changeset edit

# Or specify branch
wnt changeset edit feature/test
```

## Environment Variables

The scripts respect these environment variables:

- `EDITOR` - Your preferred text editor (used by `wnt changeset edit`)
- `VISUAL` - Alternative editor specification (higher priority than EDITOR)

## Binary Size

The release binary is optimized for size and performance:
- **Size**: ~7MB
- **Strip**: Symbols removed
- **LTO**: Enabled
- **Optimization**: Level 3

## Troubleshooting

### "Binary not found" error

Run the install script, it will build automatically:
```bash
./scripts/install-local.sh
```

### "Command not found: wnt"

Add the installation directory to your PATH:
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.local/bin:$PATH"

# Reload shell
source ~/.bashrc  # or source ~/.zshrc
```

### Editor doesn't open

Set your EDITOR environment variable:
```bash
export EDITOR=nano  # or vim, code, etc.
wnt changeset edit
```

### Demo script fails

Ensure you have required tools:
```bash
# Check git
git --version

# Check jq (optional, for JSON formatting)
jq --version

# Check cargo
cargo --version
```

## Development Workflow

```bash
# 1. Make changes to CLI code
vim crates/cli/src/commands/changeset/edit.rs

# 2. Build release
cargo build --package sublime_cli_tools --release

# 3. Test in demo
./scripts/test-in-demo.sh

# 4. Or install and test in real project
./scripts/install-local.sh
cd ~/my-real-project
wnt changeset edit
```

## Notes

- The demo script creates temporary directories that are cleaned up on exit
- The install script doesn't overwrite existing configurations
- Both scripts work on macOS and Linux
- Windows users should use WSL or adapt scripts for PowerShell