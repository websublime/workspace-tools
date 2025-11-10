#!/usr/bin/env sh
#
# Uninstall script for Workspace Tools
#
# This script removes the workspace binary, shell completions, and optionally
# configuration files from the system.
#
# Usage:
#   ./uninstall.sh [OPTIONS]
#   curl -fsSL https://raw.githubusercontent.com/websublime/workspace-tools/main/scripts/uninstall.sh | sh
#
# Options:
#   --remove-config            Remove configuration files
#   --install-dir <DIR>        Custom installation directory to remove from
#   --no-color                 Disable colored output
#   --verbose                  Enable verbose output
#   --yes                      Skip confirmation prompts
#   --help                     Show this help message
#
# Environment Variables:
#   WORKSPACE_INSTALL_DIR            Installation directory (overridden by --install-dir)
#   NO_COLOR                   Disable colored output
#
# Exit Codes:
#   0   Success
#   1   General error
#   2   Invalid usage
#   3   Not found

set -e

# Constants
readonly BINARY_NAME="workspace"
readonly DEFAULT_INSTALL_DIRS="/usr/local/bin $HOME/.local/bin"

# Exit codes
readonly EXIT_SUCCESS=0
readonly EXIT_ERROR=1
readonly EXIT_USAGE=2
readonly EXIT_NOT_FOUND=3

# Global state
VERBOSE=false
NO_COLOR=false
REMOVE_CONFIG=false
SKIP_CONFIRMATION=false
INSTALL_DIR=""
FOUND_BINARY=""

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
# Ask for user confirmation
# Arguments:
#   Prompt message
# Returns:
#   0 if confirmed, 1 otherwise
#######################################
confirm() {
    local prompt="$1"

    if [ "${SKIP_CONFIRMATION}" = "true" ]; then
        return 0
    fi

    printf "%b" "${YELLOW}?${NC} ${prompt} ${CYAN}[y/N]${NC} " >&2
    read -r response

    case "${response}" in
        [yY][eE][sS]|[yY])
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

#######################################
# Find workspace binary in system
# Returns:
#   Path to binary if found, empty otherwise
#######################################
find_binary() {
    # User specified directory
    if [ -n "${INSTALL_DIR}" ]; then
        local binary="${INSTALL_DIR}/${BINARY_NAME}"
        if [ -f "${binary}" ]; then
            echo "${binary}"
            return 0
        fi
        return 1
    fi

    # Environment variable
    if [ -n "${WORKSPACE_INSTALL_DIR}" ]; then
        local binary="${WORKSPACE_INSTALL_DIR}/${BINARY_NAME}"
        if [ -f "${binary}" ]; then
            echo "${binary}"
            return 0
        fi
    fi

    # Check if it's in PATH
    if command -v "${BINARY_NAME}" >/dev/null 2>&1; then
        command -v "${BINARY_NAME}"
        return 0
    fi

    # Search in default directories
    for dir in ${DEFAULT_INSTALL_DIRS}; do
        local binary="${dir}/${BINARY_NAME}"
        if [ -f "${binary}" ]; then
            echo "${binary}"
            return 0
        fi
    done

    return 1
}

#######################################
# Remove binary file
# Arguments:
#   Binary path
#######################################
remove_binary() {
    local binary="$1"
    local dir
    dir=$(dirname "${binary}")

    info "Removing binary: ${binary}"

    if [ ! -f "${binary}" ]; then
        warn "Binary not found: ${binary}"
        return 0
    fi

    if [ -w "${dir}" ]; then
        verbose "Removing without sudo"
        rm -f "${binary}" || die "${EXIT_ERROR}" "Failed to remove binary: ${binary}"
    else
        verbose "Removing with sudo"
        if command -v sudo >/dev/null 2>&1; then
            sudo rm -f "${binary}" || die "${EXIT_ERROR}" "Failed to remove binary: ${binary}"
        else
            die "${EXIT_ERROR}" "No write permission and sudo not available"
        fi
    fi

    success "Binary removed"
}

#######################################
# Remove shell completions
#######################################
remove_completions() {
    info "Removing shell completions"

    local removed=0

    # Bash completions
    local bash_completion="${HOME}/.local/share/bash-completion/completions/${BINARY_NAME}"
    if [ -f "${bash_completion}" ]; then
        verbose "Removing bash completion: ${bash_completion}"
        rm -f "${bash_completion}" && removed=$((removed + 1))
    fi

    # Zsh completions
    local zsh_completion="${HOME}/.local/share/zsh/site-functions/_${BINARY_NAME}"
    if [ -f "${zsh_completion}" ]; then
        verbose "Removing zsh completion: ${zsh_completion}"
        rm -f "${zsh_completion}" && removed=$((removed + 1))
    fi

    # Fish completions
    local fish_completion="${HOME}/.config/fish/completions/${BINARY_NAME}.fish"
    if [ -f "${fish_completion}" ]; then
        verbose "Removing fish completion: ${fish_completion}"
        rm -f "${fish_completion}" && removed=$((removed + 1))
    fi

    if [ ${removed} -gt 0 ]; then
        success "Removed ${removed} shell completion file(s)"
    else
        verbose "No shell completions found"
    fi
}

#######################################
# Find configuration files
# Returns:
#   List of configuration files found
#######################################
find_config_files() {
    local files=""

    # Project-level config (current directory and parents)
    local dir="${PWD}"
    while [ "${dir}" != "/" ]; do
        if [ -f "${dir}/.workspace.toml" ]; then
            files="${files} ${dir}/.workspace.toml"
        fi
        if [ -f "${dir}/.wntrc" ]; then
            files="${files} ${dir}/.wntrc"
        fi
        if [ -f "${dir}/workspace.config.json" ]; then
            files="${files} ${dir}/workspace.config.json"
        fi
        dir=$(dirname "${dir}")
    done

    # User-level config
    if [ -f "${HOME}/.config/workspace/config.toml" ]; then
        files="${files} ${HOME}/.config/workspace/config.toml"
    fi

    if [ -f "${HOME}/.workspace.toml" ]; then
        files="${files} ${HOME}/.workspace.toml"
    fi

    echo "${files}"
}

#######################################
# Remove configuration files
#######################################
remove_config() {
    if [ "${REMOVE_CONFIG}" = "false" ]; then
        verbose "Skipping configuration removal"
        return 0
    fi

    local config_files
    config_files=$(find_config_files)

    if [ -z "${config_files}" ]; then
        verbose "No configuration files found"
        return 0
    fi

    log ""
    info "Found configuration files:"
    for file in ${config_files}; do
        log "  ${file}"
    done
    log ""

    if ! confirm "Remove these configuration files?"; then
        info "Keeping configuration files"
        return 0
    fi

    info "Removing configuration files"

    for file in ${config_files}; do
        if [ -f "${file}" ]; then
            verbose "Removing: ${file}"
            rm -f "${file}" || warn "Failed to remove: ${file}"
        fi
    done

    # Remove empty directories
    if [ -d "${HOME}/.config/workspace" ]; then
        if [ -z "$(ls -A "${HOME}/.config/workspace")" ]; then
            verbose "Removing empty directory: ${HOME}/.config/workspace"
            rmdir "${HOME}/.config/workspace" 2>/dev/null || true
        fi
    fi

    success "Configuration files removed"
}

#######################################
# Display completion message
#######################################
show_completion() {
    log ""
    success "${BOLD}Uninstallation complete!${NC}"
    log ""
    log "Thank you for using ${BINARY_NAME}!"
    log ""

    if [ "${REMOVE_CONFIG}" = "false" ]; then
        log "${CYAN}Note:${NC} Configuration files were not removed."
        log "To remove them, run:"
        log "${GREEN}  $(basename "$0") --remove-config${NC}"
        log ""
    fi

    log "To reinstall, visit: ${CYAN}https://github.com/websublime/workspace-tools${NC}"
    log ""
}

#######################################
# Display help message
#######################################
show_help() {
    cat << EOF
Workspace Node Tools (workspace) Uninstall Script

USAGE:
    uninstall.sh [OPTIONS]

OPTIONS:
    --remove-config            Remove configuration files
    --install-dir <DIR>        Custom installation directory to remove from
    --no-color                 Disable colored output
    --verbose                  Enable verbose output
    --yes                      Skip confirmation prompts
    --help                     Show this help message

ENVIRONMENT VARIABLES:
    WORKSPACE_INSTALL_DIR            Installation directory (overridden by --install-dir)
    NO_COLOR                   Disable colored output

EXAMPLES:
    # Basic uninstall
    ./uninstall.sh

    # Uninstall with configuration removal
    ./uninstall.sh --remove-config

    # Uninstall from custom directory
    ./uninstall.sh --install-dir ~/.local/bin

    # Non-interactive uninstall
    ./uninstall.sh --yes --remove-config

EXIT CODES:
    0   Success
    1   General error
    2   Invalid usage
    3   Binary not found

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
            --remove-config)
                REMOVE_CONFIG=true
                shift
                ;;
            --install-dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            --no-color)
                NO_COLOR=true
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --yes|-y)
                SKIP_CONFIRMATION=true
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
# Main uninstall function
#######################################
main() {
    # Check for NO_COLOR environment variable
    if [ -n "${NO_COLOR}" ]; then
        NO_COLOR=true
    fi

    # Parse arguments
    parse_args "$@"

    # Initialize colors
    init_colors

    log ""
    log "${BOLD}=== Workspace Node Tools (workspace) Uninstall ===${NC}"
    log ""

    # Find binary
    FOUND_BINARY=$(find_binary)

    if [ -z "${FOUND_BINARY}" ]; then
        die "${EXIT_NOT_FOUND}" "${BINARY_NAME} is not installed or could not be found"
    fi

    info "Found: ${FOUND_BINARY}"

    # Show version
    if [ -x "${FOUND_BINARY}" ]; then
        local version
        version=$("${FOUND_BINARY}" --version 2>/dev/null || echo "unknown")
        verbose "Version: ${version}"
    fi

    log ""

    # Confirm uninstall
    if ! confirm "Are you sure you want to uninstall ${BINARY_NAME}?"; then
        info "Uninstall cancelled"
        exit "${EXIT_SUCCESS}"
    fi

    log ""

    # Remove binary
    remove_binary "${FOUND_BINARY}"

    # Remove completions
    remove_completions

    # Remove configuration (if requested)
    remove_config

    # Show completion message
    show_completion
}

# Run main function
main "$@"
