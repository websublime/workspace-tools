#!/usr/bin/env bash
#
# Development installation script for wnt (Workspace Node Tools)
#
# This script builds the CLI in release mode and installs it locally for testing.
# It's designed for development workflow, not production installations.
#
# Usage:
#   ./scripts/install-dev.sh [DESTINATION]
#
# Examples:
#   ./scripts/install-dev.sh                    # Install to ~/.local/bin
#   ./scripts/install-dev.sh /usr/local/bin    # Install to system-wide location
#   ./scripts/install-dev.sh ~/bin             # Install to custom location

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default installation directory
DEFAULT_INSTALL_DIR="$HOME/.local/bin"
INSTALL_DIR="${1:-$DEFAULT_INSTALL_DIR}"

# Project root (parent of scripts directory)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY_SOURCE="$PROJECT_ROOT/target/release/wnt"

echo -e "${BLUE}=== wnt CLI Development Installation ===${NC}\n"

# Check if binary exists, if not build it
if [ ! -f "$BINARY_SOURCE" ]; then
    echo -e "${YELLOW}Binary not found. Building release binary...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --package sublime_cli_tools --release
    echo ""
fi

# Verify binary was built
if [ ! -f "$BINARY_SOURCE" ]; then
    echo -e "${RED}Error: Failed to build binary${NC}"
    exit 1
fi

# Create installation directory if it doesn't exist
if [ ! -d "$INSTALL_DIR" ]; then
    echo -e "${YELLOW}Creating installation directory: $INSTALL_DIR${NC}"
    mkdir -p "$INSTALL_DIR"
fi

# Copy binary
echo -e "${BLUE}Installing wnt to: $INSTALL_DIR${NC}"
cp "$BINARY_SOURCE" "$INSTALL_DIR/wnt"
chmod +x "$INSTALL_DIR/wnt"

# Check if installation directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "\n${YELLOW}Warning: $INSTALL_DIR is not in your PATH${NC}"
    echo -e "${YELLOW}Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):${NC}"
    echo -e "${GREEN}export PATH=\"$INSTALL_DIR:\$PATH\"${NC}"
fi

# Display version
echo -e "\n${GREEN}✓ Installation complete!${NC}"
echo -e "\nInstalled version:"
"$INSTALL_DIR/wnt" version

echo -e "\n${BLUE}Usage:${NC}"
echo -e "  wnt --help              # Show help"
echo -e "  wnt init                # Initialize a workspace"
echo -e "  wnt changeset create    # Create a changeset"
echo -e "  wnt changeset edit      # Edit a changeset"
echo -e "  wnt changeset list      # List changesets"

echo -e "\n${GREEN}Happy testing!${NC}"
