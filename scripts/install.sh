#!/usr/bin/env sh
#
# Official installation script for Workspace Tools
#
# This script detects the operating system and architecture, downloads the
# appropriate binary from GitHub releases, verifies its integrity, and
# installs it to the specified location.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --version v0.1.0
#   curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --install-dir ~/.local/bin
#   curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --verbose
#   curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --no-color
#
# Options:
#   --version <VERSION>        Install specific version (default: latest)
#   --install-dir <DIR>        Custom installation directory
#   --no-shell-completions     Skip shell completions installation
#   --no-color                 Disable colored output
#   --verbose                  Enable verbose output
#   --help                     Show this help message
#
# Environment Variables:
#   WORKSPACE_VERSION                Version to install (overridden by --version)
#   WORKSPACE_INSTALL_DIR            Installation directory (overridden by --install-dir)
#   WORKSPACE_GITHUB_TOKEN           GitHub token for private repositories
#   NO_COLOR                   Disable colored output
#
# Exit Codes:
#   0   Success
#   1   General error
#   2   Invalid usage
#   3   Platform not supported
#   4   Download failed
#   5   Checksum verification failed
#   6   Installation failed

set -e

# Constants
readonly BINARY_NAME="workspace"
readonly GITHUB_REPO="workspace-tools"
readonly GITHUB_ORG="websublime"
readonly RELEASE_URL="https://github.com/${GITHUB_ORG}/${GITHUB_REPO}/releases"
readonly DEFAULT_INSTALL_DIR="/usr/local/bin"
readonly FALLBACK_INSTALL_DIR="$HOME/.local/bin"

# Exit codes
readonly EXIT_SUCCESS=0
readonly EXIT_ERROR=1
readonly EXIT_USAGE=2
readonly EXIT_UNSUPPORTED=3
readonly EXIT_DOWNLOAD=4
readonly EXIT_CHECKSUM=5
readonly EXIT_INSTALL=6

# Global state
VERBOSE=false
NO_COLOR=false
INSTALL_SHELL_COMPLETIONS=true
VERSION=""
INSTALL_DIR=""
TEMP_DIR=""

# Colors (will be disabled if NO_COLOR is set)
RED=""
GREEN=""
YELLOW=""
BLUE=""
CYAN=""
BOLD=""
NC=""

#######################################
# Initialize colors based on NO_COLOR setting
# Globals:
#   NO_COLOR, RED, GREEN, YELLOW, BLUE, CYAN, BOLD, NC
#######################################
init_colors() {
    if [ "${NO_COLOR}" = "false" ] && [ -t 1 ]; then
        RED='\033[0;31m'
        GREEN='\033[0;32m'
        YELLOW='\033[1;33m'
        BLUE='\033[0;34m'
        CYAN='\033[0;36m'
        BOLD='\033[1m'
        NC='\033[0m'
    fi
}

#######################################
# Print message to stderr
# Arguments:
#   Message to print
#######################################
log() {
    printf "%b\n" "$*" >&2
}

#######################################
# Print info message
# Arguments:
#   Message to print
#######################################
info() {
    log "${BLUE}==>${NC} ${BOLD}$*${NC}"
}

#######################################
# Print success message
# Arguments:
#   Message to print
#######################################
success() {
    log "${GREEN}✓${NC} $*"
}

#######################################
# Print warning message
# Arguments:
#   Message to print
#######################################
warn() {
    log "${YELLOW}⚠${NC} $*"
}

#######################################
# Print error message
# Arguments:
#   Message to print
#######################################
error() {
    log "${RED}✗${NC} ${BOLD}Error:${NC} $*"
}

#######################################
# Print verbose message if verbose mode is enabled
# Arguments:
#   Message to print
#######################################
verbose() {
    if [ "${VERBOSE}" = "true" ]; then
        log "${CYAN}[DEBUG]${NC} $*"
    fi
}

#######################################
# Cleanup temporary files on exit
# Globals:
#   TEMP_DIR
#######################################
cleanup() {
    if [ -n "${TEMP_DIR}" ] && [ -d "${TEMP_DIR}" ]; then
        verbose "Cleaning up temporary directory: ${TEMP_DIR}"
        rm -rf "${TEMP_DIR}"
    fi
}

#######################################
# Exit with error message and code
# Arguments:
#   Exit code
#   Error message
#######################################
die() {
    local exit_code="$1"
    shift
    error "$*"
    exit "${exit_code}"
}

#######################################
# Detect operating system
# Returns:
#   OS name (darwin, linux, windows)
#######################################
detect_os() {
    local os
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"

    case "${os}" in
        darwin)
            echo "darwin"
            ;;
        linux)
            echo "linux"
            ;;
        msys*|mingw*|cygwin*)
            echo "windows"
            ;;
        *)
            die "${EXIT_UNSUPPORTED}" "Unsupported operating system: ${os}"
            ;;
    esac
}

#######################################
# Detect system architecture
# Returns:
#   Architecture (x86_64, aarch64, arm)
#######################################
detect_arch() {
    local arch
    arch="$(uname -m)"

    case "${arch}" in
        x86_64|amd64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        armv7l|armv6l)
            echo "arm"
            ;;
        *)
            die "${EXIT_UNSUPPORTED}" "Unsupported architecture: ${arch}"
            ;;
    esac
}

#######################################
# Get target triple for Rust builds
# Arguments:
#   OS name
#   Architecture
# Returns:
#   Target triple (e.g., x86_64-apple-darwin)
#######################################
get_target_triple() {
    local os="$1"
    local arch="$2"

    case "${os}" in
        darwin)
            case "${arch}" in
                x86_64)
                    echo "x86_64-apple-darwin"
                    ;;
                aarch64)
                    echo "aarch64-apple-darwin"
                    ;;
                *)
                    die "${EXIT_UNSUPPORTED}" "Unsupported macOS architecture: ${arch}"
                    ;;
            esac
            ;;
        linux)
            case "${arch}" in
                x86_64)
                    echo "x86_64-unknown-linux-gnu"
                    ;;
                aarch64)
                    echo "aarch64-unknown-linux-gnu"
                    ;;
                arm)
                    echo "armv7-unknown-linux-gnueabihf"
                    ;;
                *)
                    die "${EXIT_UNSUPPORTED}" "Unsupported Linux architecture: ${arch}"
                    ;;
            esac
            ;;
        windows)
            case "${arch}" in
                x86_64)
                    echo "x86_64-pc-windows-msvc"
                    ;;
                *)
                    die "${EXIT_UNSUPPORTED}" "Unsupported Windows architecture: ${arch}"
                    ;;
            esac
            ;;
        *)
            die "${EXIT_UNSUPPORTED}" "Unsupported operating system: ${os}"
            ;;
    esac
}

#######################################
# Get latest version from GitHub releases
# Returns:
#   Latest version tag (e.g., v0.1.0)
#######################################
get_latest_version() {
    local url="${RELEASE_URL}/latest"
    verbose "Fetching latest version from: ${url}"

    if command -v curl >/dev/null 2>&1; then
        local response
        response=$(curl -fsSL "${url}" 2>&1) || die "${EXIT_DOWNLOAD}" "Failed to fetch latest version"

        # Extract version from redirect URL or HTML
        echo "${response}" | grep -oE 'v[0-9]+\.[0-9]+\.[0-9]+' | head -n1
    elif command -v wget >/dev/null 2>&1; then
        local response
        response=$(wget -qO- "${url}" 2>&1) || die "${EXIT_DOWNLOAD}" "Failed to fetch latest version"

        echo "${response}" | grep -oE 'v[0-9]+\.[0-9]+\.[0-9]+' | head -n1
    else
        die "${EXIT_ERROR}" "Neither curl nor wget found. Please install one of them."
    fi
}

#######################################
# Download file from URL
# Arguments:
#   URL
#   Destination path
#######################################
download_file() {
    local url="$1"
    local dest="$2"

    verbose "Downloading: ${url}"
    verbose "Destination: ${dest}"

    if command -v curl >/dev/null 2>&1; then
        local curl_opts="-fsSL"
        if [ "${VERBOSE}" = "true" ]; then
            curl_opts="-fL"
        fi

        if [ -n "${WORKSPACE_GITHUB_TOKEN}" ]; then
            curl ${curl_opts} -H "Authorization: token ${WORKSPACE_GITHUB_TOKEN}" -o "${dest}" "${url}" || \
                die "${EXIT_DOWNLOAD}" "Failed to download from ${url}"
        else
            curl ${curl_opts} -o "${dest}" "${url}" || \
                die "${EXIT_DOWNLOAD}" "Failed to download from ${url}"
        fi
    elif command -v wget >/dev/null 2>&1; then
        local wget_opts="-q"
        if [ "${VERBOSE}" = "true" ]; then
            wget_opts="-v"
        fi

        if [ -n "${WORKSPACE_GITHUB_TOKEN}" ]; then
            wget ${wget_opts} --header="Authorization: token ${WORKSPACE_GITHUB_TOKEN}" -O "${dest}" "${url}" || \
                die "${EXIT_DOWNLOAD}" "Failed to download from ${url}"
        else
            wget ${wget_opts} -O "${dest}" "${url}" || \
                die "${EXIT_DOWNLOAD}" "Failed to download from ${url}"
        fi
    else
        die "${EXIT_ERROR}" "Neither curl nor wget found. Please install one of them."
    fi
}

#######################################
# Verify SHA256 checksum
# Arguments:
#   File path
#   Expected checksum
# Returns:
#   0 if checksum matches, 1 otherwise
#######################################
verify_checksum() {
    local file="$1"
    local expected="$2"

    verbose "Verifying checksum for: ${file}"
    verbose "Expected: ${expected}"

    local actual
    if command -v sha256sum >/dev/null 2>&1; then
        actual=$(sha256sum "${file}" | awk '{print $1}')
    elif command -v shasum >/dev/null 2>&1; then
        actual=$(shasum -a 256 "${file}" | awk '{print $1}')
    else
        warn "Neither sha256sum nor shasum found. Skipping checksum verification."
        return 0
    fi

    verbose "Actual: ${actual}"

    if [ "${actual}" != "${expected}" ]; then
        die "${EXIT_CHECKSUM}" "Checksum verification failed\n  Expected: ${expected}\n  Got: ${actual}"
    fi

    success "Checksum verified"
}

#######################################
# Download and extract binary
# Arguments:
#   Version
#   Target triple
#   Destination directory
# Returns:
#   Path to extracted binary
#######################################
download_binary() {
    local version="$1"
    local target="$2"
    local dest_dir="$3"

    local archive_name="${BINARY_NAME}-${version}-${target}.tar.gz"
    local download_url="${RELEASE_URL}/download/${version}/${archive_name}"
    local checksum_url="${RELEASE_URL}/download/${version}/checksums.txt"

    info "Downloading ${BINARY_NAME} ${version} for ${target}"

    # Download archive
    local archive_path="${TEMP_DIR}/${archive_name}"
    download_file "${download_url}" "${archive_path}"

    # Download checksums
    local checksums_path="${TEMP_DIR}/checksums.txt"
    download_file "${checksum_url}" "${checksums_path}"

    # Extract expected checksum
    local expected_checksum
    expected_checksum=$(grep "${archive_name}" "${checksums_path}" | awk '{print $1}')

    if [ -z "${expected_checksum}" ]; then
        die "${EXIT_CHECKSUM}" "Checksum not found for ${archive_name}"
    fi

    # Verify checksum
    verify_checksum "${archive_path}" "${expected_checksum}"

    # Extract archive
    info "Extracting binary"
    tar -xzf "${archive_path}" -C "${TEMP_DIR}"

    local binary_path="${TEMP_DIR}/${BINARY_NAME}"
    if [ ! -f "${binary_path}" ]; then
        die "${EXIT_ERROR}" "Binary not found in archive"
    fi

    echo "${binary_path}"
}

#######################################
# Check if directory is writable
# Arguments:
#   Directory path
# Returns:
#   0 if writable, 1 otherwise
#######################################
is_writable() {
    local dir="$1"

    if [ ! -d "${dir}" ]; then
        # Check if we can create the directory
        local parent
        parent=$(dirname "${dir}")
        [ -w "${parent}" ]
    else
        [ -w "${dir}" ]
    fi
}

#######################################
# Determine installation directory
# Returns:
#   Installation directory path
#######################################
determine_install_dir() {
    # User specified directory
    if [ -n "${INSTALL_DIR}" ]; then
        echo "${INSTALL_DIR}"
        return
    fi

    # Environment variable
    if [ -n "${WORKSPACE_INSTALL_DIR}" ]; then
        echo "${WORKSPACE_INSTALL_DIR}"
        return
    fi

    # Try default directory
    if is_writable "${DEFAULT_INSTALL_DIR}"; then
        echo "${DEFAULT_INSTALL_DIR}"
        return
    fi

    # Fallback to user directory
    echo "${FALLBACK_INSTALL_DIR}"
}

#######################################
# Install binary to destination
# Arguments:
#   Source binary path
#   Destination directory
#######################################
install_binary() {
    local src="$1"
    local dest_dir="$2"
    local dest="${dest_dir}/${BINARY_NAME}"

    info "Installing to: ${dest_dir}"

    # Create destination directory if it doesn't exist
    if [ ! -d "${dest_dir}" ]; then
        verbose "Creating directory: ${dest_dir}"
        mkdir -p "${dest_dir}" || die "${EXIT_INSTALL}" "Failed to create directory: ${dest_dir}"
    fi

    # Install binary
    if is_writable "${dest_dir}"; then
        verbose "Installing without sudo"
        cp "${src}" "${dest}" || die "${EXIT_INSTALL}" "Failed to copy binary to ${dest}"
        chmod +x "${dest}" || die "${EXIT_INSTALL}" "Failed to make binary executable"
    else
        verbose "Installing with sudo"
        if command -v sudo >/dev/null 2>&1; then
            sudo cp "${src}" "${dest}" || die "${EXIT_INSTALL}" "Failed to copy binary to ${dest}"
            sudo chmod +x "${dest}" || die "${EXIT_INSTALL}" "Failed to make binary executable"
        else
            die "${EXIT_INSTALL}" "No write permission and sudo not available"
        fi
    fi

    success "Binary installed: ${dest}"
}

#######################################
# Install shell completions
# Arguments:
#   Binary path
#######################################
install_completions() {
    local binary="$1"

    if [ "${INSTALL_SHELL_COMPLETIONS}" = "false" ]; then
        verbose "Skipping shell completions installation"
        return
    fi

    info "Installing shell completions"

    # Bash completions
    if command -v bash >/dev/null 2>&1; then
        local bash_completion_dir="${HOME}/.local/share/bash-completion/completions"
        if [ ! -d "${bash_completion_dir}" ]; then
            mkdir -p "${bash_completion_dir}"
        fi

        if "${binary}" completions bash > "${bash_completion_dir}/${BINARY_NAME}" 2>/dev/null; then
            success "Bash completions installed"
        else
            verbose "Failed to install bash completions (not critical)"
        fi
    fi

    # Zsh completions
    if command -v zsh >/dev/null 2>&1; then
        local zsh_completion_dir="${HOME}/.local/share/zsh/site-functions"
        if [ ! -d "${zsh_completion_dir}" ]; then
            mkdir -p "${zsh_completion_dir}"
        fi

        if "${binary}" completions zsh > "${zsh_completion_dir}/_${BINARY_NAME}" 2>/dev/null; then
            success "Zsh completions installed"
        else
            verbose "Failed to install zsh completions (not critical)"
        fi
    fi

    # Fish completions
    if command -v fish >/dev/null 2>&1; then
        local fish_completion_dir="${HOME}/.config/fish/completions"
        if [ ! -d "${fish_completion_dir}" ]; then
            mkdir -p "${fish_completion_dir}"
        fi

        if "${binary}" completions fish > "${fish_completion_dir}/${BINARY_NAME}.fish" 2>/dev/null; then
            success "Fish completions installed"
        else
            verbose "Failed to install fish completions (not critical)"
        fi
    fi
}

#######################################
# Check if directory is in PATH
# Arguments:
#   Directory path
# Returns:
#   0 if in PATH, 1 otherwise
#######################################
is_in_path() {
    local dir="$1"

    case ":${PATH}:" in
        *:"${dir}":*)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

#######################################
# Display post-installation instructions
# Arguments:
#   Installation directory
#   Installed binary path
#######################################
show_post_install() {
    local install_dir="$1"
    local binary_path="${install_dir}/${BINARY_NAME}"

    log ""
    success "${BOLD}Installation complete!${NC}"
    log ""

    # Show version
    if [ -x "${binary_path}" ]; then
        local version_output
        version_output=$("${binary_path}" --version 2>/dev/null || echo "unknown")
        info "Installed: ${version_output}"
    fi

    log ""

    # PATH warning
    if ! is_in_path "${install_dir}"; then
        warn "Installation directory is not in PATH: ${install_dir}"
        log ""
        log "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        log "${GREEN}  export PATH=\"${install_dir}:\$PATH\"${NC}"
        log ""
        log "Then reload your shell:"
        log "${GREEN}  source ~/.bashrc${NC}  ${CYAN}# or source ~/.zshrc${NC}"
        log ""
    fi

    # Quick start
    log "${BOLD}Quick Start:${NC}"
    log "  ${GREEN}${BINARY_NAME} --help${NC}              Show help"
    log "  ${GREEN}${BINARY_NAME} init${NC}                Initialize a workspace"
    log "  ${GREEN}${BINARY_NAME} changeset add${NC}       Create a changeset"
    log ""
    log "For more information, visit: ${CYAN}https://github.com/websublime/workspace-tools${NC}"
    log ""
}

#######################################
# Display help message
#######################################
show_help() {
    cat << EOF
Workspace Node Tools (workspace) Installation Script

USAGE:
    curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh [OPTIONS]

OPTIONS:
    --version <VERSION>        Install specific version (default: latest)
    --install-dir <DIR>        Custom installation directory
    --no-shell-completions     Skip shell completions installation
    --no-color                 Disable colored output
    --verbose                  Enable verbose output
    --help                     Show this help message

ENVIRONMENT VARIABLES:
    WORKSPACE_VERSION                Version to install (overridden by --version)
    WORKSPACE_INSTALL_DIR            Installation directory (overridden by --install-dir)
    WORKSPACE_GITHUB_TOKEN           GitHub token for private repositories
    NO_COLOR                   Disable colored output

EXAMPLES:
    # Install latest version
    curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh

    # Install specific version
    curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --version v0.1.0

    # Custom installation directory
    curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --install-dir ~/.local/bin

    # Verbose output
    curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/install.sh | sh -s -- --verbose

EXIT CODES:
    0   Success
    1   General error
    2   Invalid usage
    3   Platform not supported
    4   Download failed
    5   Checksum verification failed
    6   Installation failed

For more information, visit: https://github.com/websublime/workspace-tools
EOF
}

#######################################
# Parse command line arguments
# Arguments:
#   All command line arguments
#######################################
parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --version)
                VERSION="$2"
                shift 2
                ;;
            --install-dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            --no-shell-completions)
                INSTALL_SHELL_COMPLETIONS=false
                shift
                ;;
            --no-color)
                NO_COLOR=true
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --help|-h)
                show_help
                exit "${EXIT_SUCCESS}"
                ;;
            *)
                die "${EXIT_USAGE}" "Unknown option: $1\nUse --help for usage information"
                ;;
        esac
    done
}

#######################################
# Main installation function
#######################################
main() {
    # Setup trap for cleanup
    trap cleanup EXIT INT TERM

    # Check for NO_COLOR environment variable
    if [ -n "${NO_COLOR}" ]; then
        NO_COLOR=true
    fi

    # Parse arguments
    parse_args "$@"

    # Initialize colors
    init_colors

    log ""
    log "${BOLD}=== Workspace Node Tools (workspace) Installation ===${NC}"
    log ""

    # Detect platform
    local os
    os=$(detect_os)
    verbose "Detected OS: ${os}"

    # Detect architecture
    local arch
    arch=$(detect_arch)
    verbose "Detected architecture: ${arch}"

    # Get target triple
    local target
    target=$(get_target_triple "${os}" "${arch}")
    info "Platform: ${target}"

    # Determine version
    if [ -z "${VERSION}" ]; then
        if [ -n "${WORKSPACE_VERSION}" ]; then
            VERSION="${WORKSPACE_VERSION}"
        else
            VERSION=$(get_latest_version)
        fi
    fi

    if [ -z "${VERSION}" ]; then
        die "${EXIT_ERROR}" "Could not determine version to install"
    fi

    info "Version: ${VERSION}"

    # Create temporary directory
    TEMP_DIR=$(mktemp -d)
    verbose "Temporary directory: ${TEMP_DIR}"

    # Download binary
    local binary_path
    binary_path=$(download_binary "${VERSION}" "${target}" "${TEMP_DIR}")

    # Determine installation directory
    local install_dir
    install_dir=$(determine_install_dir)

    # Install binary
    install_binary "${binary_path}" "${install_dir}"

    # Install shell completions
    install_completions "${install_dir}/${BINARY_NAME}"

    # Show post-installation instructions
    show_post_install "${install_dir}"
}

# Run main function
main "$@"
