#!/usr/bin/env sh
#
# Uninstall workspace git hooks
#
# This is a convenience script that calls install-hooks.sh with --uninstall.
#
# Usage:
#   ./scripts/uninstall-hooks.sh

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

exec "${SCRIPT_DIR}/install-hooks.sh" --uninstall
