#!/usr/bin/env bash
#
# Test workspace CLI in a demo repository
#
# This script:
# 1. Builds the CLI binary
# 2. Creates a temporary demo repository
# 3. Initializes it with workspace
# 4. Creates sample changesets
# 5. Tests the edit command
#
# Usage:
#   ./scripts/test-in-demo.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY="$PROJECT_ROOT/target/release/workspace"

echo -e "${BLUE}=== workspace CLI Demo Test ===${NC}\n"

# Build the binary if it doesn't exist
if [ ! -f "$BINARY" ]; then
    echo -e "${YELLOW}Building release binary...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --package sublime_cli_tools --release
    echo ""
fi

# Create temporary demo directory
DEMO_DIR=$(mktemp -d -t workspace-demo-XXXXXX)
echo -e "${CYAN}Demo repository: $DEMO_DIR${NC}\n"

# Cleanup function
cleanup() {
    if [ -n "$DEMO_DIR" ] && [ -d "$DEMO_DIR" ]; then
        echo -e "\n${YELLOW}Cleaning up demo repository...${NC}"
        rm -rf "$DEMO_DIR"
        echo -e "${GREEN}✓ Cleanup complete${NC}"
    fi
}

# Register cleanup on exit
trap cleanup EXIT

# Navigate to demo directory
cd "$DEMO_DIR"

# Initialize git repository
echo -e "${BLUE}1. Initializing git repository...${NC}"
git init -q
git config user.email "test@example.com"
git config user.name "Test User"
echo -e "${GREEN}✓ Git repository initialized${NC}\n"

# Create a simple monorepo structure
echo -e "${BLUE}2. Creating monorepo structure...${NC}"
mkdir -p packages/package-a packages/package-b

# Create package.json files
cat > package.json << 'EOF'
{
  "name": "demo-monorepo",
  "version": "1.0.0",
  "private": true,
  "workspaces": [
    "packages/*"
  ]
}
EOF

cat > packages/package-a/package.json << 'EOF'
{
  "name": "@demo/package-a",
  "version": "1.0.0",
  "description": "Demo package A"
}
EOF

cat > packages/package-b/package.json << 'EOF'
{
  "name": "@demo/package-b",
  "version": "1.0.0",
  "description": "Demo package B"
}
EOF

git add .
git commit -q -m "initial commit"
echo -e "${GREEN}✓ Monorepo structure created${NC}\n"

# Initialize workspace
echo -e "${BLUE}3. Initializing workspace workspace...${NC}"
$BINARY init \
    --changeset-path .changesets \
    --environments dev,staging,prod \
    --default-env dev,staging \
    --strategy independent \
    --non-interactive

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ workspace initialized${NC}\n"
else
    echo -e "${RED}✗ workspace initialization failed${NC}\n"
    exit 1
fi

# Create a feature branch
echo -e "${BLUE}4. Creating feature branch...${NC}"
git checkout -q -b feature/test-edit
echo -e "${GREEN}✓ Branch created: feature/test-edit${NC}\n"

# Create a changeset
echo -e "${BLUE}5. Creating a changeset...${NC}"
$BINARY changeset create \
    --bump minor \
    --env dev,staging,prod \
    --packages @demo/package-a \
    --message "Add new feature to package A" \
    --non-interactive

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Changeset created${NC}\n"
else
    echo -e "${RED}✗ Changeset creation failed${NC}\n"
    exit 1
fi

# List changesets
echo -e "${BLUE}6. Listing changesets...${NC}"
$BINARY changeset list
echo ""

# Show changeset details
echo -e "${BLUE}7. Showing changeset details...${NC}"
$BINARY changeset show feature/test-edit
echo ""

# Test edit command (non-interactive - just verify it finds the file)
echo -e "${BLUE}8. Testing changeset edit command...${NC}"
echo -e "${YELLOW}Note: Edit command would normally open your editor.${NC}"
echo -e "${YELLOW}To test interactively, run:${NC}"
echo -e "${CYAN}  cd $DEMO_DIR${NC}"
echo -e "${CYAN}  $BINARY changeset edit${NC}"
echo ""

# Display changeset file location
CHANGESET_FILE=$(find .changesets -name "*.json" | head -n 1)
if [ -n "$CHANGESET_FILE" ]; then
    echo -e "${GREEN}✓ Changeset file found: $CHANGESET_FILE${NC}"
    echo -e "\n${BLUE}Changeset content:${NC}"
    cat "$CHANGESET_FILE" | jq . 2>/dev/null || cat "$CHANGESET_FILE"
    echo ""
fi

# Summary
echo -e "\n${GREEN}=== Test Summary ===${NC}"
echo -e "${GREEN}✓ Binary built successfully${NC}"
echo -e "${GREEN}✓ Demo repository created${NC}"
echo -e "${GREEN}✓ Workspace initialized${NC}"
echo -e "${GREEN}✓ Changeset created${NC}"
echo -e "${GREEN}✓ Changeset commands working${NC}"
echo ""

echo -e "${BLUE}=== Manual Testing Instructions ===${NC}"
echo -e "To test the edit command interactively:"
echo -e "  ${CYAN}1. Keep this terminal open (demo repo will be cleaned up on exit)${NC}"
echo -e "  ${CYAN}2. Open a new terminal${NC}"
echo -e "  ${CYAN}3. Run: cd $DEMO_DIR${NC}"
echo -e "  ${CYAN}4. Run: $BINARY changeset edit${NC}"
echo -e "  ${CYAN}5. Your editor should open the changeset file${NC}"
echo -e "  ${CYAN}6. Make changes and save${NC}"
echo -e "  ${CYAN}7. The command should validate your changes${NC}"
echo ""

echo -e "${YELLOW}Demo repository will remain available until you press Ctrl+C${NC}"
echo -e "${YELLOW}Press Ctrl+C to cleanup and exit...${NC}\n"

# Wait for user to interrupt
while true; do
    sleep 1
done
