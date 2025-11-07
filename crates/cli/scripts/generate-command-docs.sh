#!/usr/bin/env bash
#
# Generates command documentation from CLI help text.
#
# This script extracts help text from the compiled CLI binary and formats
# it into markdown documentation. It ensures that the command reference
# stays in sync with the actual CLI implementation.
#
# Usage:
#   ./scripts/generate-command-docs.sh
#
# Requirements:
#   - CLI binary must be built first (cargo build --release)
#   - Standard Unix tools (grep, sed, awk)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
CLI_DIR="$PROJECT_ROOT/crates/cli"

# Output file
OUTPUT_FILE="$CLI_DIR/docs/COMMANDS_GENERATED.md"

echo "Generating command documentation from CLI help text..."
echo ""

# Find CLI binary
find_binary() {
    local binary_name="workspace"

    # Check platform-specific extension
    if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        binary_name="workspace.exe"
    fi

    # Check release build first
    if [[ -f "$PROJECT_ROOT/target/release/$binary_name" ]]; then
        echo "$PROJECT_ROOT/target/release/$binary_name"
        return 0
    fi

    # Check debug build
    if [[ -f "$PROJECT_ROOT/target/debug/$binary_name" ]]; then
        echo "$PROJECT_ROOT/target/debug/$binary_name"
        return 0
    fi

    return 1
}

echo "Finding CLI binary..."
if ! BINARY=$(find_binary); then
    echo -e "${RED}✗ CLI binary not found${NC}"
    echo ""
    echo "Build it first with:"
    echo "  cargo build --release"
    echo ""
    exit 1
fi

echo -e "${GREEN}✓ Found CLI binary:${NC} $BINARY"
echo ""

# Extract help text
extract_help() {
    local cmd=("$@")
    "$BINARY" "${cmd[@]}" --help 2>&1 || true
}

# Discover commands
discover_commands() {
    local help_text="$1"
    echo "$help_text" | \
        awk '/^Commands:$/,/^$/ {print}' | \
        grep -E '^\s+[a-z]' | \
        awk '{print $1}' | \
        grep -v '^help$' || true
}

# Discover subcommands
discover_subcommands() {
    local help_text="$1"
    echo "$help_text" | \
        awk '/^Commands:$/,/^$/ {print}' | \
        grep -E '^\s+[a-z]' | \
        awk '{print $1}' | \
        grep -v '^help$' || true
}

# Start generating markdown
echo "Extracting help text..."
echo ""

# Create markdown header
cat > "$OUTPUT_FILE" << 'EOF'
# Workspace Tools - Command Reference (Generated)

**Generated from CLI help text**

⚠️  **Note**: This file is auto-generated from CLI `--help` output.
For human-friendly documentation with examples, see [COMMANDS.md](./COMMANDS.md)

---

EOF

# Add generation timestamp
cat >> "$OUTPUT_FILE" << EOF
**Generated:** $(date -u '+%Y-%m-%d %H:%M:%S UTC')

---

## Table of Contents

- [Global Options](#global-options)
- [Commands](#commands)

---

EOF

# Extract and add global help
echo "  Extracting global options..."
GLOBAL_HELP=$(extract_help)

cat >> "$OUTPUT_FILE" << EOF
## Global Options

\`\`\`
$GLOBAL_HELP
\`\`\`

---

## Commands

EOF

# Discover all commands
COMMANDS=$(discover_commands "$GLOBAL_HELP")

if [[ -z "$COMMANDS" ]]; then
    echo -e "${YELLOW}⚠ No commands discovered${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Discovered commands:${NC}"
echo "$COMMANDS" | while read -r cmd; do
    echo "    - $cmd"
done
echo ""

# Extract help for each command
echo "Extracting command help..."
echo "$COMMANDS" | while read -r cmd; do
    echo "  Processing: $cmd"

    # Get command help
    CMD_HELP=$(extract_help "$cmd")

    # Add to markdown
    cat >> "$OUTPUT_FILE" << EOF
### \`$cmd\`

\`\`\`
$CMD_HELP
\`\`\`

EOF

    # Check for subcommands
    SUBCOMMANDS=$(discover_subcommands "$CMD_HELP")

    if [[ -n "$SUBCOMMANDS" ]]; then
        echo "$SUBCOMMANDS" | while read -r subcmd; do
            echo "    Processing: $cmd $subcmd"

            # Get subcommand help
            SUBCMD_HELP=$(extract_help "$cmd" "$subcmd")

            # Add to markdown
            cat >> "$OUTPUT_FILE" << EOF
#### \`$cmd $subcmd\`

\`\`\`
$SUBCMD_HELP
\`\`\`

EOF
        done
    fi

    # Add separator
    echo "---" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
done

# Add footer
cat >> "$OUTPUT_FILE" << 'EOF'

## Regeneration

To regenerate this documentation:

```bash
# Build the CLI first
cargo build --release

# Run the generation script
./crates/cli/scripts/generate-command-docs.sh
```

Or use the Rust version:

```bash
cargo run --bin generate_command_docs
```

## Comparison

Compare generated docs with manually maintained docs:

```bash
diff crates/cli/docs/COMMANDS.md crates/cli/docs/COMMANDS_GENERATED.md
```

The manually maintained `COMMANDS.md` includes:
- Detailed examples with output
- Common usage patterns
- Best practices
- JSON output examples
- Quick reference sections

This generated file provides:
- Raw CLI help text
- Exact command signatures
- Up-to-date option descriptions
- Verification that docs match implementation

---

**Last Generated:** $(date -u '+%Y-%m-%d %H:%M:%S UTC')
EOF

echo ""
echo -e "${GREEN}✓ Documentation generated:${NC} $OUTPUT_FILE"
echo ""
echo "Compare with existing documentation:"
echo "  diff $CLI_DIR/docs/COMMANDS.md $OUTPUT_FILE"
echo ""
echo "To update the main documentation, review the diff and manually"
echo "incorporate any changes to COMMANDS.md"
