#!/usr/bin/env sh
#
# Install workspace git hooks
#
# This script installs the workspace git hooks into your repository's .git/hooks directory.
# The hook will automatically sync changesets before pushing.
#
# Usage:
#   ./scripts/install-hooks.sh [OPTIONS]
#
# Options:
#   --force          Overwrite existing hooks
#   --uninstall      Remove workspace hooks
#   --help           Show this help message
#
# Examples:
#   ./scripts/install-hooks.sh           # Install pre-push hook
#   ./scripts/install-hooks.sh --force   # Force reinstall
#   ./scripts/install-hooks.sh --uninstall # Remove hook

set -e

# Colors
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    CYAN='\033[0;36m'
    BOLD='\033[1m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    CYAN=''
    BOLD=''
    NC=''
fi

# Default options
FORCE=false
UNINSTALL=false

# Script directory
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOKS_SOURCE_DIR="${SCRIPT_DIR}/git-hooks"

# Git hooks directory
GIT_DIR=$(git rev-parse --git-dir 2>/dev/null || echo "")
if [ -z "${GIT_DIR}" ]; then
    printf "${RED}✗${NC} Not a git repository\n" >&2
    exit 1
fi

HOOKS_DIR="${GIT_DIR}/hooks"

#######################################
# Show help message
#######################################
show_help() {
    sed -n '2,/^$/p' "$0" | sed 's/^# \?//'
}

#######################################
# Install a single hook
# Arguments:
#   Hook name
#######################################
install_hook() {
    local hook_name="$1"
    local source="${HOOKS_SOURCE_DIR}/${hook_name}"
    local dest="${HOOKS_DIR}/${hook_name}"

    if [ ! -f "${source}" ]; then
        printf "${RED}✗${NC} Hook source not found: ${hook_name}\n"
        return 1
    fi

    # Check if hook already exists
    if [ -f "${dest}" ] && [ "${FORCE}" = "false" ]; then
        # Check if it's a workspace hook
        if grep -q "workspace (Workspace Tools)" "${dest}" 2>/dev/null; then
            printf "${YELLOW}⚠${NC} ${hook_name} already installed (use --force to overwrite)\n"
            return 0
        else
            printf "${YELLOW}⚠${NC} ${hook_name} exists but not a workspace hook (use --force to overwrite)\n"
            return 0
        fi
    fi

    # Copy hook
    cp "${source}" "${dest}"
    chmod +x "${dest}"

    printf "${GREEN}✓${NC} Installed: ${CYAN}${hook_name}${NC}\n"
}

#######################################
# Uninstall a single hook
# Arguments:
#   Hook name
#######################################
uninstall_hook() {
    local hook_name="$1"
    local dest="${HOOKS_DIR}/${hook_name}"

    if [ ! -f "${dest}" ]; then
        return 0
    fi

    # Only remove if it's a workspace hook
    if grep -q "workspace (Workspace Tools)" "${dest}" 2>/dev/null; then
        rm -f "${dest}"
        printf "${GREEN}✓${NC} Removed: ${CYAN}${hook_name}${NC}\n"
    fi
}

#######################################
# Parse command line arguments
#######################################
parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --force)
                FORCE=true
                shift
                ;;
            --uninstall)
                UNINSTALL=true
                shift
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                printf "${RED}✗${NC} Unknown option: $1\n" >&2
                printf "Use --help for usage information\n" >&2
                exit 2
                ;;
        esac
    done
}

#######################################
# Main installation function
#######################################
main() {
    parse_args "$@"

    # Create hooks directory if it doesn't exist
    mkdir -p "${HOOKS_DIR}"

    printf "\n${BOLD}=== workspace Git Hooks Installation ===${NC}\n\n"

    if [ "${UNINSTALL}" = "true" ]; then
        printf "${BLUE}Uninstalling workspace hooks...${NC}\n\n"
        uninstall_hook "pre-push"
        printf "\n${GREEN}✓${NC} ${BOLD}Uninstallation complete${NC}\n\n"
        exit 0
    fi

    printf "${BLUE}Installing hooks to: ${CYAN}${HOOKS_DIR}${NC}\n\n"

    # Install pre-push hook
    install_hook "pre-push"

    printf "\n${GREEN}✓${NC} ${BOLD}Installation complete!${NC}\n\n"

    # Show what the hook does
    printf "${BOLD}Installed hook:${NC}\n"
    printf "  ${CYAN}pre-push${NC}  Syncs all commits to changeset before pushing\n"

    printf "\n${BOLD}Workflow:${NC}\n"
    printf "  ${GREEN}1.${NC} Create branch & changeset:  ${CYAN}git checkout -b feature/name && workspace changeset create${NC}\n"
    printf "  ${GREEN}2.${NC} Make commits:                ${CYAN}git commit -m \"feat: ...\"${NC} ${BLUE}(as many as you want)${NC}\n"
    printf "  ${GREEN}3.${NC} Push to remote:              ${CYAN}git push${NC}\n"
    printf "     ${BLUE}→${NC} Hook syncs all commits to changeset\n"
    printf "     ${BLUE}→${NC} Creates sync commit if needed\n"
    printf "     ${BLUE}→${NC} Push proceeds automatically\n"

    printf "\n${BOLD}Key points:${NC}\n"
    printf "  ${YELLOW}•${NC} Sync commits (${CYAN}chore: sync changeset${NC}) are created automatically\n"
    printf "  ${YELLOW}•${NC} These are maintenance commits and don't need to be in the changeset\n"
    printf "  ${YELLOW}•${NC} All your feature commits are tracked automatically\n"

    printf "\n${BOLD}Configuration:${NC}\n"
    printf "  Add to ${CYAN}.workspace.toml${NC} to customize:\n"
    printf "    ${YELLOW}[git_hooks]${NC}\n"
    printf "    ${YELLOW}enabled = true${NC}\n"
    printf "    ${YELLOW}sync_on_push = true${NC}\n"

    printf "\n${BOLD}To disable temporarily:${NC}\n"
    printf "  ${GREEN}WORKSPACE_SKIP_HOOKS=1 git push${NC}\n"

    printf "\n${BOLD}To uninstall:${NC}\n"
    printf "  ${GREEN}./scripts/install-hooks.sh --uninstall${NC}\n"

    printf "\n"
}

main "$@"
