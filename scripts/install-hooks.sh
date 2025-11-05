#!/usr/bin/env sh
#
# Install wnt git hooks
#
# This script installs the wnt git hooks into your repository's .git/hooks directory.
# The hooks will automatically manage changesets during your git workflow.
#
# Usage:
#   ./scripts/install-hooks.sh [OPTIONS]
#
# Options:
#   --all                     Install all hooks (default)
#   --pre-commit              Install only pre-commit hook
#   --post-checkout           Install only post-checkout hook
#   --pre-push                Install only pre-push hook
#   --prepare-commit-msg      Install only prepare-commit-msg hook
#   --force                   Overwrite existing hooks
#   --uninstall               Remove all wnt hooks
#   --help                    Show this help message
#
# Examples:
#   ./scripts/install-hooks.sh                    # Install all hooks
#   ./scripts/install-hooks.sh --pre-commit       # Install only pre-commit
#   ./scripts/install-hooks.sh --force            # Force reinstall all
#   ./scripts/install-hooks.sh --uninstall        # Remove all hooks

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
INSTALL_ALL=true
INSTALL_PRE_COMMIT=false
INSTALL_POST_CHECKOUT=false
INSTALL_PRE_PUSH=false
INSTALL_PREPARE_COMMIT_MSG=false
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
        # Check if it's a wnt hook
        if grep -q "wnt (Workspace Node Tools)" "${dest}" 2>/dev/null; then
            printf "${YELLOW}⚠${NC} ${hook_name} already installed (use --force to overwrite)\n"
            return 0
        else
            printf "${YELLOW}⚠${NC} ${hook_name} exists but not a wnt hook (use --force to overwrite)\n"
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

    # Only remove if it's a wnt hook
    if grep -q "wnt (Workspace Node Tools)" "${dest}" 2>/dev/null; then
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
            --all)
                INSTALL_ALL=true
                shift
                ;;
            --pre-commit)
                INSTALL_ALL=false
                INSTALL_PRE_COMMIT=true
                shift
                ;;
            --post-checkout)
                INSTALL_ALL=false
                INSTALL_POST_CHECKOUT=true
                shift
                ;;
            --pre-push)
                INSTALL_ALL=false
                INSTALL_PRE_PUSH=true
                shift
                ;;
            --prepare-commit-msg)
                INSTALL_ALL=false
                INSTALL_PREPARE_COMMIT_MSG=true
                shift
                ;;
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

    printf "\n${BOLD}=== wnt Git Hooks Installation ===${NC}\n\n"

    if [ "${UNINSTALL}" = "true" ]; then
        printf "${BLUE}Uninstalling wnt hooks...${NC}\n\n"
        uninstall_hook "pre-commit"
        uninstall_hook "post-checkout"
        uninstall_hook "pre-push"
        uninstall_hook "prepare-commit-msg"
        printf "\n${GREEN}✓${NC} ${BOLD}Uninstallation complete${NC}\n\n"
        exit 0
    fi

    printf "${BLUE}Installing hooks to: ${CYAN}${HOOKS_DIR}${NC}\n\n"

    # Install hooks based on options
    if [ "${INSTALL_ALL}" = "true" ]; then
        install_hook "pre-commit"
        install_hook "post-checkout"
        install_hook "pre-push"
        install_hook "prepare-commit-msg"
    else
        if [ "${INSTALL_PRE_COMMIT}" = "true" ]; then
            install_hook "pre-commit"
        fi
        if [ "${INSTALL_POST_CHECKOUT}" = "true" ]; then
            install_hook "post-checkout"
        fi
        if [ "${INSTALL_PRE_PUSH}" = "true" ]; then
            install_hook "pre-push"
        fi
        if [ "${INSTALL_PREPARE_COMMIT_MSG}" = "true" ]; then
            install_hook "prepare-commit-msg"
        fi
    fi

    printf "\n${GREEN}✓${NC} ${BOLD}Installation complete!${NC}\n\n"

    # Show what each hook does
    printf "${BOLD}Installed hooks:${NC}\n"
    printf "  ${CYAN}pre-commit${NC}           Auto-updates changeset on commit\n"
    printf "  ${CYAN}post-checkout${NC}        Creates changeset for new branches\n"
    printf "  ${CYAN}pre-push${NC}             Validates changeset before push\n"
    printf "  ${CYAN}prepare-commit-msg${NC}   Enhances commit messages\n"

    printf "\n${BOLD}Configuration:${NC}\n"
    printf "  Add to ${CYAN}.wnt.toml${NC} to customize:\n"
    printf "    ${YELLOW}[git_hooks]${NC}\n"
    printf "    ${YELLOW}enabled = true${NC}\n"
    printf "    ${YELLOW}auto_update_on_commit = true${NC}\n"

    printf "\n${BOLD}To disable temporarily:${NC}\n"
    printf "  ${GREEN}WNT_SKIP_HOOKS=1 git commit${NC}\n"

    printf "\n${BOLD}To uninstall:${NC}\n"
    printf "  ${GREEN}./scripts/install-hooks.sh --uninstall${NC}\n"

    printf "\n"
}

main "$@"
