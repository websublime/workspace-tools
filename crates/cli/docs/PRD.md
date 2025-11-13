# Product Requirements Document (PRD)
# Workspace Node Tools CLI

**Version:** 0.1.0 (Draft)  
**Created:** 2024  
**Status:** Draft for Iteration  
**Target Audience:** Node.js/TypeScript developers working with single packages or monorepos

> **📝 Note:** This is a historical design document. The installation methods described in Section 10 have been superseded by **cargo-dist** automated installers. See the main [README.md](../../README.md) for current installation instructions.

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Product Vision](#product-vision)
3. [Goals and Objectives](#goals-and-objectives)
4. [Target Users](#target-users)
5. [User Scenarios](#user-scenarios)
6. [Feature Requirements](#feature-requirements)
7. [Command Reference](#command-reference)
8. [Technical Architecture](#technical-architecture)
9. [UI/UX Design](#uiux-design)
10. [Installation and Distribution](#installation-and-distribution)
11. [Success Metrics](#success-metrics)
12. [Future Considerations](#future-considerations)

---

## 1. Executive Summary

The Workspace Node Tools CLI (`workspace`) is a command-line interface for managing Node.js projects (single packages and monorepos) with a focus on version management, changesets, dependency upgrades, and project health auditing. It provides a unified interface to the `sublime_package_tools` crate and integrates seamlessly with Git workflows, CI/CD pipelines, and developer workflows.

### Key Features
- **Configuration Management**: Initialize and manage project configuration
- **Changeset System**: Track changes across branches with automated package detection
- **Version Management**: Bump versions with dependency propagation
- **Upgrade Management**: Detect and apply dependency upgrades with safety checks
- **Audit System**: Comprehensive project health analysis with actionable insights
- **Git Integration**: Automated workflows for hooks and CI/CD pipelines
- **CI/CD Ready**: JSON output modes for pipeline integration

---

## 2. Product Vision

**Vision Statement:**  
Empower Node.js developers with a robust, modern CLI tool that simplifies version management, dependency tracking, and project health monitoring in both single-package and monorepo environments.

### Core Principles
1. **Developer-First**: Intuitive commands that match developer mental models
2. **Automation-Friendly**: Designed for Git hooks, GitHub Actions, and CI/CD
3. **Safety by Default**: Dry-run modes, validation, and backup mechanisms
4. **Visibility**: Clear, actionable output with appropriate detail levels
5. **Cross-Platform**: Works consistently on Windows, Linux, and macOS

---

## 3. Goals and Objectives

### Primary Goals
1. Simplify version management in monorepos with complex dependency graphs
2. Provide clear audit trails through changeset tracking
3. Enable safe dependency upgrades with impact analysis
4. Deliver comprehensive project health insights
5. Integrate seamlessly into existing Git and CI/CD workflows

### Success Criteria
- Reduce time spent on manual version bumping by 80%
- Provide changeset-to-release audit trails for 100% of changes
- Detect 100% of available dependency upgrades
- Generate actionable audit reports with health scores
- Support all major package managers (npm, yarn, pnpm, bun)

---

## 4. Target Users

### Primary Users
1. **Individual Developers**
   - Working on single packages or small monorepos
   - Need efficient version management
   - Want to track changes across feature branches

2. **Team Developers**
   - Working in collaborative monorepo environments
   - Need coordinated version releases
   - Require audit trails for compliance

3. **DevOps Engineers**
   - Integrating into CI/CD pipelines
   - Automating release processes
   - Monitoring project health

### User Personas

#### Persona 1: "Sarah - Frontend Developer"
- Works on a monorepo with 15 packages
- Creates feature branches daily
- Needs quick changeset creation without context switching
- Uses GitHub Actions for releases

#### Persona 2: "Mike - Platform Engineer"
- Manages releases for a 50+ package monorepo
- Needs to understand dependency impact before releases
- Requires detailed audit reports for security reviews
- Integrates with custom CI/CD pipelines

#### Persona 3: "Alex - Open Source Maintainer"
- Maintains multiple independent packages
- Needs clear changelogs for users
- Wants automated dependency upgrade detection
- Values minimal configuration

---

## 5. User Scenarios

### Scenario A: Project Initialization
**Context:** Developer starts a new project or adds workspace to existing project

**Flow:**
1. Developer runs `workspace init` in project root
2. CLI detects project type (single/monorepo) and package manager
3. Interactive prompts collect configuration:
   - Changeset directory (default: `.changesets/`)
   - Available environments (dev, staging, prod, etc.)
   - Default environments
   - Versioning strategy (independent/unified)
   - NPM registry URL
   - Configuration format (JSON/TOML/YAML)
4. CLI generates `repo.config.[ext]` file with collected values
5. CLI validates configuration and confirms success

**Alternative Flow:** Developer provides all values via CLI flags, skipping prompts

**Expected Outcome:** 
- Valid configuration file created
- Developer can immediately start using other commands

---

### Scenario B: Feature Branch Changeset Creation
**Context:** Developer creates a feature branch and needs to track intended changes

**Flow:**
1. Developer creates new branch: `git checkout -b feature/new-component`
2. Developer runs: `workspace changeset create` (manually or via git hook)
3. CLI detects current branch and prompts:
   - "Which environments should this target?" (multi-select from config)
   - "What bump type will this be?" (patch/minor/major)
4. CLI creates changeset file in configured directory
5. Developer continues working on feature

**Alternative Flow:** 
- Developer manually runs `workspace changeset create --bump minor --env production,staging`
- Git hook skipped, changeset created immediately

**Expected Outcome:**
- Changeset file created with branch name, bump type, environments
- Ready to track commits as they occur

---

### Scenario C: Commit Tracking
**Context:** Developer commits changes on feature branch

**Flow:**
1. Developer runs: `git commit -m "feat: add new component"`
2. Developer runs: `workspace changeset update` (manually or via git hook)
3. CLI:
   - Loads changeset for current branch
   - Detects which packages were modified in commit
   - Adds commit hash to changeset
   - Adds affected packages to changeset
   - Updates changeset file
4. No user interaction required

**Expected Outcome:**
- Changeset automatically updated with commit hash and affected packages
- No interruption to developer workflow

---

### Scenario D: Project Audit
**Context:** Developer wants comprehensive project health analysis

**Flow:**
1. Developer runs: `workspace audit`
2. CLI prompts:
   - "Report format?" (markdown/json/json-compact)
   - "Audit sections?" (all/upgrades/dependencies/version-consistency/breaking-changes)
3. CLI executes selected audits:
   - Analyzes dependency tree
   - Checks for available upgrades
   - Detects version inconsistencies
   - Identifies breaking changes
   - Calculates health score
4. CLI generates formatted report
5. Displays summary and saves full report

**Alternative Flow:**
- `workspace audit --format markdown --output audit-report.md --sections upgrades,dependencies`
- Non-interactive mode for automation

**Expected Outcome:**
- Comprehensive audit report with actionable insights
- Health score with breakdown by category
- Clear recommendations for improvements

---

### Scenario E: Dependency Upgrades
**Context:** Developer wants to update dependencies safely

**Flow:**
1. Developer runs: `workspace upgrade check`
2. CLI:
   - Scans all package.json files
   - Queries npm registry for latest versions
   - Categorizes upgrades (major/minor/patch)
   - Displays upgrade table with current → latest versions
3. CLI prompts: "Apply these upgrades?" (yes/no/select)
4. If yes:
   - Creates backup of package.json files
   - Applies selected upgrades
   - Optionally creates changeset
   - Reports success/failures
5. Developer reviews changes and tests

**Alternative Flow:**
- `workspace upgrade apply --dry-run` - Preview only
- `workspace upgrade apply --patch-only --auto-changeset` - Safe automated upgrades
- `workspace upgrade apply --packages @types/node,eslint` - Specific packages only

**Expected Outcome:**
- Dependencies safely upgraded with backup
- Optional changeset created for tracking
- Clear report of what changed

---

### Scenario F: Release Process (CI/CD)
**Context:** Feature branch merged to main, CI/CD pipeline executes release

**Flow:**
1. GitHub Action triggers on merge to main
2. Action runs: `workspace bump --execute --git-tag --git-push`
3. CLI:
   - Loads all active changesets
   - Resolves version bumps for affected packages
   - Propagates version changes through dependency graph
   - Updates package.json files with new versions
   - Generates/updates CHANGELOG.md files
   - Creates git tags for releases (if --git-tag)
   - Pushes tags to remote (if --git-push)
   - Archives processed changesets
4. Action publishes packages to registry

**Expected Outcome:**
- All packages versioned correctly
- Changelogs updated with all changes from changesets
- Git tags created for each release
- Changesets archived with release info
- Clean state for next development cycle

---

### Scenario G: CI/CD Info Action
**Context:** Pipeline needs package change information for decision making

**Flow:**
1. GitHub Action runs as first step: `workspace bump --dry-run --format json`
2. CLI:
   - Analyzes changesets without making changes
   - Determines which packages will be bumped
   - Calculates snapshot versions for feature branches
   - Outputs JSON with all information
3. Action parses JSON output
4. Subsequent actions use information for:
   - Conditional deployment
   - Asset versioning
   - Test matrix generation
   - Notification formatting

**Expected Outcome:**
- JSON output with complete change information
- No side effects (dry-run mode)
- Downstream actions have all needed data

---

### Scenario H: Changeset Management
**Context:** Developer needs to view or modify changesets

**Flow:**
1. Developer runs: `workspace changeset list`
2. CLI displays all pending changesets with:
   - Branch name
   - Bump type
   - Affected packages
   - Number of commits
   - Target environments
3. Developer can:
   - View details: `workspace changeset show feature/my-branch`
   - Update: `workspace changeset update --bump major`
   - Delete: `workspace changeset delete feature/old-branch`
   - Query history: `workspace changeset history --package @myorg/core`

**Expected Outcome:**
- Full visibility into changeset state
- Easy modification capabilities
- Historical tracking

---

### Scenario I: Repository Cloning with Setup

**Context:** Developer needs to clone a repository and start working immediately

**Flow:**
1. Developer runs: `workspace clone https://github.com/company/api.git ~/dev/api`
2. CLI displays progress bar during clone operation
3. CLI detects if workspace configuration exists:
   - **Path 3a - Config exists**: 
     - Validates configuration file
     - Checks directory structure (.changesets/, .changesets/history/, .workspace-backups/)
     - Verifies .gitignore entries
     - Reports validation results
   - **Path 3b - No config**: 
     - Automatically starts init command
     - Prompts for configuration (or uses flags if --non-interactive)
     - Creates configuration file
     - Creates directory structure
     - Updates .gitignore
4. CLI displays success message with next steps

**Alternative Flow 1:** Developer omits destination
```bash
workspace clone https://github.com/company/api.git
# Clones to ./api/ (using repository name)
```

**Alternative Flow 2:** Developer provides all init flags for non-interactive setup
```bash
workspace clone https://github.com/user/lib.git ./my-lib \
  --non-interactive \
  --strategy independent \
  --environments "dev,staging,prod" \
  --default-env "prod"
# Clones and initializes without prompts
```

**Alternative Flow 3:** Destination already exists
```bash
workspace clone https://github.com/company/api.git ~/dev/api
# Error: Destination already exists
# Use --force to remove and re-clone
```

**Alternative Flow 4:** Validation fails on existing config
```bash
workspace clone https://github.com/team/broken.git ~/dev/broken
# Clone succeeds
# Validation detects:
#   - Missing .changesets/history/ directory
#   - Invalid strategy value in config
# Reports errors with suggested fixes
```

**Expected Outcome:**
- Repository cloned successfully to destination
- Workspace configuration verified or created
- Developer can immediately use workspace commands
- Clear feedback on what was done and next steps

---

## 6. Feature Requirements

### 6.1 Configuration Management

#### F-001: Project Initialization
**Priority:** P0 (Must Have)  
**Description:** Initialize new or existing project with workspace configuration

**Requirements:**
- Detect project type (single package/monorepo)
- Detect package manager (npm/yarn/pnpm/bun)
- Interactive prompts for all configuration values
- Support for non-interactive mode with CLI flags
- Validate configuration before writing
- Support multiple configuration formats (JSON/TOML/YAML)
- Create default directory structure for changesets

**Acceptance Criteria:**
- ✓ Successfully initializes both single-package and monorepo projects
- ✓ Generated config passes validation
- ✓ Non-interactive mode works with all flags provided
- ✓ Clear error messages for invalid inputs

#### F-002: Configuration Validation
**Priority:** P0 (Must Have)  
**Description:** Validate existing configuration files

**Requirements:**
- Validate all required fields present
- Validate field types and formats
- Check referenced directories exist or can be created
- Validate environment names (no duplicates)
- Validate registry URLs
- Provide clear, actionable error messages

**Acceptance Criteria:**
- ✓ Catches all common configuration errors
- ✓ Provides specific line/field information for errors
- ✓ Suggests corrections where possible

#### F-003: Configuration Display
**Priority:** P2 (Nice to Have)  
**Description:** Display current configuration

**Requirements:**
- Show all configuration values
- Indicate which values are defaults vs. explicitly set
- Support JSON output for programmatic access
- Show configuration file location

---

### 6.2 Changeset System

#### F-010: Changeset Creation
**Priority:** P0 (Must Have)  
**Description:** Create changeset for current branch

**Requirements:**
- Auto-detect current branch name
- Interactive prompts for bump type and environments
- Support non-interactive mode with flags
- Validate branch doesn't already have changeset
- Create changeset with unique ID
- Store in configured changeset directory
- Support custom changeset messages

**Acceptance Criteria:**
- ✓ Creates valid changeset file
- ✓ Prevents duplicate changesets
- ✓ Works in both interactive and non-interactive modes
- ✓ Changeset includes all required metadata

#### F-011: Changeset Update
**Priority:** P0 (Must Have)  
**Description:** Update changeset with new commits and affected packages

**Requirements:**
- Accept optional changeset ID/branch parameter
- When no ID provided, detect current branch automatically
- Search for changeset matching the branch name
- If no matching changeset found, log error message indicating no changeset exists for the branch
- Analyze git diff to determine affected packages
- Add commit hash to changeset
- Add newly affected packages to changeset
- Handle monorepo package detection correctly
- Skip if no changeset exists (graceful degradation)
- Support manual package specification

**Acceptance Criteria:**
- ✓ Works without ID parameter (uses current branch)
- ✓ Works with explicit ID/branch parameter
- ✓ Clear error message when no changeset found for branch
- ✓ Correctly detects affected packages in monorepo
- ✓ Handles edge cases (deleted packages, renamed files)
- ✓ Updates changeset atomically
- ✓ Provides clear feedback on what was added

#### F-012: Changeset Listing
**Priority:** P1 (Should Have)  
**Description:** List all pending changesets

**Requirements:**
- Display all changesets in configured directory
- Show key information: branch, bump, packages, commit count
- Support filtering by package, bump type, environment
- Support sorting options
- Provide summary statistics

**Acceptance Criteria:**
- ✓ Lists all valid changesets
- ✓ Filters work correctly
- ✓ Output is readable and informative

#### F-013: Changeset Details
**Priority:** P1 (Should Have)  
**Description:** Show detailed information for specific changeset

**Requirements:**
- Display all changeset metadata
- Show all affected packages
- List all commits with messages
- Show target environments
- Display creation and last update timestamps

**Acceptance Criteria:**
- ✓ Shows complete changeset information
- ✓ Formatting is clear and readable
- ✓ Handles missing optional fields gracefully

#### F-014: Changeset History
**Priority:** P1 (Should Have)  
**Description:** Query archived changesets

**Requirements:**
- Query by package name
- Query by date range
- Query by environment
- Query by bump type
- Display release information for archived changesets
- Support pagination for large result sets

**Acceptance Criteria:**
- ✓ All query filters work correctly
- ✓ Returns archived changesets with release info
- ✓ Handles large histories efficiently

#### F-015: Changeset Modification
**Priority:** P1 (Should Have)  
**Description:** Modify existing changeset

**Requirements:**
- Update bump type
- Add/remove environments
- Add/remove packages manually
- Update changeset message
- Validate changes before saving

**Acceptance Criteria:**
- ✓ All modification operations work
- ✓ Maintains changeset integrity
- ✓ Validates before committing changes

#### F-016: Changeset Deletion
**Priority:** P1 (Should Have)  
**Description:** Delete changeset

**Requirements:**
- Delete specified changeset
- Require confirmation (unless --force flag)
- Optionally archive instead of delete
- Prevent accidental deletion of wrong changeset

**Acceptance Criteria:**
- ✓ Safely deletes changeset
- ✓ Confirmation works as expected
- ✓ Clear feedback on what was deleted

---

### 6.3 Version Management

#### F-020: Version Bump
**Priority:** P0 (Must Have)  
**Description:** Bump package versions based on changesets

**Requirements:**
- Load all active changesets
- Resolve versions for all affected packages
- Propagate version changes through dependency graph
- Handle both unified and independent versioning strategies
- Support dry-run mode (no changes)
- Update package.json files
- Optionally commit version changes (--git-commit)
- Optionally create git tags (--git-tag)
- Optionally push tags to remote (--git-push)
- Generate/updates changelogs
- Archive processed changesets
- Support snapshot versions for feature branches
- Support pre-release versions (alpha, beta, rc)

**Version Bump Behavior by Project Type:**

1. **Single Repository (Single Package)**
   - Only one package exists in the project
   - Version bump applies to that single package
   - Changesets specify which commits are included in the version bump
   - Result: One version, one tag (e.g., `v1.2.0` or `my-package@1.2.0`)

2. **Monorepo with Independent Strategy**
   - Each package maintains its own independent version
   - **Only packages listed in changesets receive version bumps**
   - Packages not in any active changeset remain at their current version
   - Dependency propagation: If package A depends on workspace package B, and B gets bumped, A's dependency reference is updated but A's version only bumps if A is also in a changeset OR if configured to auto-propagate
   - Result: Multiple versions, one tag per bumped package (e.g., `@org/pkg-a@1.2.0`, `@org/pkg-b@2.0.0`)

3. **Monorepo with Unified Strategy**
   - All workspace packages share the same version number
   - When ANY package listed in changesets requires a bump, ALL workspace packages receive the same version bump
   - The highest bump type from all changesets is applied (major > minor > patch)
   - All packages move to the new unified version, regardless of whether they had code changes
   - Result: One unified version applied to all packages, one tag per package or one monorepo tag (configurable)

**Key Principle:** 
The changeset's `packages` field (Vec<String>) determines which packages are affected. Only packages explicitly listed in active changesets (or all packages in unified strategy) will have their versions bumped.

**Acceptance Criteria:**
- ✓ Single repo: bumps only package version when changeset exists
- ✓ Monorepo independent: bumps only packages listed in changesets
- ✓ Monorepo unified: bumps all packages when any changeset exists
- ✓ Correctly identifies affected packages from changesets
- ✓ Dependency propagation works correctly
- ✓ Dry-run mode produces accurate preview showing which packages will bump
- ✓ Git operations work when flags provided
- ✓ Works without git operations when flags omitted
- ✓ Git tags created with correct format (when --git-tag)
- ✓ Tags pushed successfully (when --git-push)
- ✓ Changesets archived with release info

#### F-021: Version Preview
**Priority:** P0 (Must Have)  
**Description:** Preview version changes without applying

**Requirements:**
- Show current version → next version for all packages
- Clearly indicate which packages will be bumped vs. which will remain unchanged
- Display dependency propagation chain
- Calculate and show dependency graph changes
- Support JSON output for CI/CD
- Show which changesets will be processed
- Highlight circular dependencies if any
- Show versioning strategy being used (independent/unified)
- For independent strategy: show only affected packages
- For unified strategy: show all workspace packages receiving new version

**Acceptance Criteria:**
- ✓ Accurately predicts all version changes
- ✓ Clearly shows which packages are affected by changesets
- ✓ Shows which packages remain unchanged
- ✓ JSON output is machine-readable
- ✓ Shows complete dependency impact
- ✓ Displays strategy being used

#### F-022: Snapshot Version Generation
**Priority:** P1 (Should Have)  
**Description:** Generate snapshot versions for feature branches

**Requirements:**
- Support configurable snapshot format templates
- Include branch name, commit hash, timestamp
- Handle branch name sanitization
- Generate unique snapshot identifiers
- Support custom snapshot variables
- Respect versioning strategy (independent vs unified)
- Only generate snapshots for packages listed in changesets (independent)
- Generate snapshots for all packages (unified) when any changeset exists

**Acceptance Criteria:**
- ✓ Generates valid semver-compatible versions
- ✓ Respects independent vs unified strategy
- ✓ Only affects packages that would be bumped in a normal release
- ✓ Snapshots are unique and sortable
- ✓ Works with all package managers

---

### 6.4 Dependency Upgrades

#### F-030: Upgrade Detection
**Priority:** P0 (Must Have)  
**Description:** Detect available dependency upgrades

**Requirements:**
- Scan all package.json files in project
- Query npm registry for latest versions
- Support custom registries and scoped registries
- Respect package manager lock files
- Categorize upgrades (major/minor/patch)
- Support filtering by upgrade type
- Show current → latest version for each dependency
- Calculate total upgrade count and statistics
- Support .npmrc configuration reading

**Acceptance Criteria:**
- ✓ Detects all available upgrades
- ✓ Correctly categorizes upgrade types
- ✓ Works with custom registries
- ✓ Respects package manager conventions

#### F-031: Upgrade Application
**Priority:** P0 (Must Have)  
**Description:** Apply dependency upgrades

**Requirements:**
- Support selective upgrade application
- Create backups before applying
- Update package.json files
- Support automatic changeset creation
- Handle upgrade failures gracefully
- Support rollback via backup
- Report successes and failures
- Support dry-run mode

**Acceptance Criteria:**
- ✓ Successfully applies upgrades
- ✓ Backups work correctly
- ✓ Rollback restores exact previous state
- ✓ Clear reporting of what changed

#### F-032: Upgrade Filtering
**Priority:** P1 (Should Have)  
**Description:** Filter upgrades by various criteria

**Requirements:**
- Filter by upgrade type (major/minor/patch)
- Filter by dependency type (regular/dev/peer)
- Filter by specific package names
- Filter by package scope
- Combine multiple filters

**Acceptance Criteria:**
- ✓ All filters work correctly
- ✓ Multiple filters combine logically
- ✓ Clear indication of active filters

#### F-033: Upgrade Backup Management
**Priority:** P1 (Should Have)  
**Description:** Manage upgrade backups

**Requirements:**
- List all backups with metadata
- Restore specific backup
- Delete old backups
- Configure backup retention policy
- Show backup size and packages affected

**Acceptance Criteria:**
- ✓ Backups can be listed and restored
- ✓ Cleanup respects retention policy
- ✓ Restore works correctly

---

### 6.5 Audit System

#### F-040: Comprehensive Audit
**Priority:** P0 (Must Have)  
**Description:** Execute comprehensive project audit

**Requirements:**
- Execute all audit sections
- Calculate overall health score
- Generate detailed reports
- Support multiple output formats (markdown/json)
- Include actionable recommendations
- Support selective section execution
- Show issue counts by severity
- Support verbosity levels

**Acceptance Criteria:**
- ✓ Executes all audit sections successfully
- ✓ Generates accurate health score
- ✓ Reports are comprehensive and actionable
- ✓ All output formats work correctly

#### F-041: Upgrade Audit
**Priority:** P0 (Must Have)  
**Description:** Audit available upgrades

**Requirements:**
- Detect all available upgrades
- Categorize by type (major/minor/patch)
- Calculate upgrade health score
- Identify critical/security upgrades
- Generate upgrade recommendations
- Show outdated dependency statistics

**Acceptance Criteria:**
- ✓ Detects all upgrades correctly
- ✓ Recommendations are relevant
- ✓ Security upgrades highlighted

#### F-042: Dependency Audit
**Priority:** P0 (Must Have)  
**Description:** Audit dependency health

**Requirements:**
- Detect circular dependencies
- Identify missing dependencies
- Find deprecated packages
- Categorize dependencies (internal/external/workspace/local)
- Calculate dependency health score
- Detect version inconsistencies within monorepo
- Check for phantom dependencies
- Analyze dependency tree depth

**Acceptance Criteria:**
- ✓ Detects all dependency issues
- ✓ Categorization is accurate
- ✓ Clear explanation of issues

#### F-043: Version Consistency Audit
**Priority:** P1 (Should Have)  
**Description:** Audit version consistency across monorepo

**Requirements:**
- Detect same dependency with different versions
- Identify version range conflicts
- Check workspace protocol usage
- Suggest version alignment
- Calculate consistency score

**Acceptance Criteria:**
- ✓ Finds all version inconsistencies
- ✓ Suggestions are practical
- ✓ Works in both monorepo and single-package contexts

#### F-044: Breaking Changes Audit
**Priority:** P1 (Should Have)  
**Description:** Detect potential breaking changes

**Requirements:**
- Parse conventional commits for breaking changes
- Analyze major version bumps in dependencies
- Parse changelogs for breaking change sections
- Report packages with breaking changes
- Calculate breaking change impact

**Acceptance Criteria:**
- ✓ Detects breaking changes from multiple sources
- ✓ Impact analysis is accurate
- ✓ Clear reporting of affected areas

#### F-045: Health Score Calculation
**Priority:** P1 (Should Have)  
**Description:** Calculate project health score

**Requirements:**
- Weighted scoring across all audit sections
- Configurable weights per section
- Diminishing returns for high issue counts
- Score breakdown by category
- Historical score tracking (future)
- Score range: 0-100

**Acceptance Criteria:**
- ✓ Score is calculated consistently
- ✓ Weights are configurable
- ✓ Score breakdown is clear

#### F-046: Report Formatting
**Priority:** P0 (Must Have)  
**Description:** Format audit reports in multiple formats

**Requirements:**
- Markdown format for human reading
- JSON format for programmatic access
- JSON compact format for CI/CD
- Support output to file or stdout
- Include all audit data in reports
- Support verbosity levels (minimal/normal/detailed)

**Acceptance Criteria:**
- ✓ All formats generate valid output
- ✓ Reports include all relevant information
- ✓ Verbosity levels work correctly

---

### 6.6 Changes Analysis

#### F-050: Working Directory Analysis
**Priority:** P1 (Should Have)  
**Description:** Analyze uncommitted changes

**Requirements:**
- Detect modified files in working directory
- Map files to packages in monorepo
- Show packages with changes
- Display file-level change statistics
- Support staged vs. unstaged differentiation

**Acceptance Criteria:**
- ✓ Accurately detects all changes
- ✓ Package mapping works in monorepos
- ✓ Statistics are accurate

#### F-051: Commit Range Analysis
**Priority:** P1 (Should Have)  
**Description:** Analyze changes between commits/branches

**Requirements:**
- Support commit hash ranges
- Support branch comparisons
- Map changes to packages
- Show commit history for affected packages
- Display change statistics

**Acceptance Criteria:**
- ✓ Correctly analyzes commit ranges
- ✓ Branch comparison works
- ✓ Package mapping is accurate

#### F-052: Affected Packages Detection
**Priority:** P0 (Must Have)  
**Description:** Detect which packages are affected by changes

**Requirements:**
- Analyze file paths to determine package ownership
- Handle monorepo workspace configurations
- Support custom package detection rules
- Exclude non-source files (docs, tests) optionally
- Handle edge cases (deleted packages, new packages)

**Acceptance Criteria:**
- ✓ Correctly identifies affected packages
- ✓ Works with all monorepo types
- ✓ Handles edge cases gracefully

---

### 6.7 Git Integration

#### F-060: Branch Detection
**Priority:** P0 (Must Have)  
**Description:** Detect and work with current Git branch

**Requirements:**
- Detect current branch name
- Handle detached HEAD state
- Support custom branch naming conventions
- Sanitize branch names for use in versions

**Acceptance Criteria:**
- ✓ Always correctly identifies branch
- ✓ Handles special cases
- ✓ Sanitization produces valid identifiers

#### F-061: Commit Information
**Priority:** P1 (Should Have)  
**Description:** Extract commit information for changesets

**Requirements:**
- Get commit hash
- Get commit message
- Get commit author
- Get commit timestamp
- Support commit ranges

**Acceptance Criteria:**
- ✓ Retrieves all commit info correctly
- ✓ Handles merge commits
- ✓ Works with all git versions

#### F-095: Repository Clone Command
**Priority:** P1 (Should Have)  
**Description:** Clone repository and automatically setup workspace

**Requirements:**
- Clone repository from URL using git (HTTPS and SSH)
- Support optional destination path (defaults to repository name)
- Display progress bar during clone operation
- Detect existing workspace configuration
- Validate configuration if exists:
  - Check config file is valid and parseable
  - Verify .changesets/ directory exists
  - Verify .changesets/history/ directory exists
  - Verify .workspace-backups/ directory exists
  - Check .gitignore has required entries
- Run init automatically if no configuration:
  - Support interactive mode (prompts)
  - Support non-interactive mode (flags)
  - Create all required directories
  - Generate configuration file
  - Update .gitignore
- Handle clone failures gracefully (network, auth, disk space)
- Support force flag to overwrite existing destination
- Support shallow clone with depth parameter
- Provide clear feedback on validation results
- Suggest fixes for validation errors
- Support JSON output for automation

**Acceptance Criteria:**
- ✓ Successfully clones from HTTPS URLs
- ✓ Successfully clones from SSH URLs
- ✓ Destination defaults to repository name if not provided
- ✓ Progress bar shows clone progress
- ✓ Detects existing configuration correctly
- ✓ Validation runs all required checks
- ✓ Validation errors show specific issues
- ✓ Validation errors include suggested fixes
- ✓ Init executes automatically when no config found
- ✓ Interactive prompts work in init flow
- ✓ Non-interactive mode works with all flags
- ✓ Force flag removes existing destination
- ✓ Depth flag creates shallow clone
- ✓ Clear error messages for all failure scenarios
- ✓ JSON output includes all relevant information
- ✓ Works on Windows, Linux, and macOS
- ✓ Handles network failures gracefully
- ✓ Handles authentication errors clearly
- ✓ Timeout handled by libgit2 (errors propagated)

---

### 6.8 Output and Logging

#### F-070: Logging Levels
**Priority:** P0 (Must Have)  
**Description:** Configurable logging output for all commands

**Important:** Logging and output format are **completely independent**. You can have:
- JSON output with no logs (`--format json --log-level silent`)
- JSON output with debug logs (`--format json --log-level debug`)
- Text output with no logs (`--format text --log-level silent`)
- Any combination you need

**Requirements:**
- Support log levels: silent, error, warn, info, debug, trace
- Global flag for log level: `--log-level`
- Default to 'info' level
- Respect NO_COLOR environment variable
- **Write logs to stderr, output to stdout** (completely separate streams)
- **Every subcommand must log its operations according to the configured level**
- Log messages should be contextual and informative
- Progress updates during long operations
- Clear indication of what the command is doing
- Logging works independently of output format

**Logging by Level:**

1. **silent**: No logs at all
   ```bash
   workspace --log-level silent bump --execute
   # No progress output, only final result
   ```

2. **error**: Only critical errors
   ```bash
   workspace --log-level error bump --execute
   # ERROR: Failed to update package.json: Permission denied
   ```

3. **warn**: Errors + warnings
   ```bash
   workspace --log-level warn upgrade check
   # WARN: Package 'eslint' has major version update available
   # WARN: Breaking changes detected
   ```

4. **info** (default): General progress
   ```bash
   workspace --log-level info bump --execute
   # INFO: Loading configuration...
   # INFO: Loading changesets...
   # INFO: Found 2 active changesets
   # INFO: Resolving versions...
   # INFO: Updating package.json files...
   # INFO: Creating git tags...
   # INFO: Done!
   ```

5. **debug**: Detailed operations
   ```bash
   workspace --log-level debug bump --execute
   # DEBUG: Reading config from repo.config.yaml
   # DEBUG: Strategy: independent
   # DEBUG: Loading changeset: feature/new-api
   # DEBUG: Changeset packages: @org/core
   # DEBUG: Calculating version for @org/core: 1.2.3 -> 1.3.0
   # DEBUG: Writing to packages/core/package.json
   # DEBUG: Creating tag @org/core@1.3.0
   ```

6. **trace**: Very verbose debugging
   ```bash
   workspace --log-level trace upgrade check
   # TRACE: Entering upgrade check command
   # TRACE: Loading workspace packages from packages/*/package.json
   # TRACE: Found package: @org/core at packages/core
   # TRACE: Reading dependencies from @org/core
   # TRACE: Querying registry for typescript current: 5.0.0
   # TRACE: Registry response: latest 5.3.3
   # TRACE: Comparing versions: 5.0.0 < 5.3.3 = true
   ```

**Examples in Different Commands:**

```bash
# Init with info logging (default)
workspace init
# INFO: Detecting project type...
# INFO: Found package.json at root
# INFO: Detected: single package project
# INFO: Creating configuration...

# Changeset update with debug
workspace --log-level debug changeset update
# DEBUG: Detecting current branch: feature/new-api
# DEBUG: Loading changeset: .changesets/feature-new-api.json
# DEBUG: Analyzing git diff since last commit
# DEBUG: Detected changes in: packages/core/src/index.ts
# DEBUG: Mapped to package: @org/core
# DEBUG: Adding package to changeset
# DEBUG: Saving changeset

# Audit with trace
workspace --log-level trace audit
# TRACE: Loading configuration
# TRACE: Initializing audit manager
# TRACE: Running upgrade audit...
# TRACE: Querying registry for 45 dependencies...
# (very detailed logs)
```

**Acceptance Criteria:**
- ✓ All log levels work correctly in ALL commands
- ✓ Each subcommand logs appropriate operations at each level
- ✓ Logs go to stderr, final output to stdout (separate streams)
- ✓ JSON output works with any log level (including silent)
- ✓ JSON output is never mixed with logs
- ✓ Logging and format flags are completely independent
- ✓ Appropriate default level (info)
- ✓ Output separation works correctly
- ✓ NO_COLOR environment variable respected
- ✓ Log messages are clear and contextual

#### F-071: JSON Output Mode
**Priority:** P0 (Must Have)  
**Description:** Machine-readable JSON output

**Requirements:**
- Global flag: `--format json` (and `--format json-compact` for audit)
- Consistent JSON structure across commands
- Include success/error status in all responses
- Include all relevant data
- Valid JSON always (no mixed output)
- All commands must support JSON output:
  - `workspace init`
  - `workspace config show`
  - `workspace config validate`
  - `workspace changeset create`
  - `workspace changeset update`
  - `workspace changeset list`
  - `workspace changeset show`
  - `workspace changeset delete`
  - `workspace changeset history`
  - `workspace changeset check`
  - `workspace bump` (all modes: dry-run, execute, snapshot)
  - `workspace upgrade check`
  - `workspace upgrade apply`
  - `workspace upgrade backups list`
  - `workspace audit` (all sections)
  - `workspace changes`
  - `workspace version`

**Acceptance Criteria:**
- ✓ All commands listed above support JSON output
- ✓ JSON is always valid and parseable
- ✓ Structure is consistent across commands (success, data, errors)
- ✓ No logs or debug output mixed with JSON when --format json is used
- ✓ Examples provided in documentation for each command

#### F-072: Progress Indication
**Priority:** P1 (Should Have)  
**Description:** Show progress for long-running operations

**Requirements:**
- Spinner for indeterminate operations
- Progress bar for determinate operations
- Clear status messages
- Suppress when not TTY
- Suppress in JSON mode

**Acceptance Criteria:**
- ✓ Progress indication is helpful
- ✓ Doesn't interfere with output
- ✓ Properly suppressed when needed

#### F-073: Color Output
**Priority:** P1 (Should Have)  
**Description:** Colorized output for readability

**Requirements:**
- Color for success/error/warning/info
- Syntax highlighting for code snippets
- Respect NO_COLOR environment variable
- Disable when not TTY
- `--no-color` flag to force disable

**Acceptance Criteria:**
- ✓ Colors improve readability
- ✓ Works on all platforms
- ✓ Can be disabled

---

### 6.9 Error Handling

#### F-080: Error Messages
**Priority:** P0 (Must Have)  
**Description:** Clear, actionable error messages

**Requirements:**
- Explain what went wrong
- Explain why it happened
- Suggest how to fix it
- Include error codes for programmatic handling
- Show context (file, line, command)
- Support verbose error output

**Acceptance Criteria:**
- ✓ Errors are understandable
- ✓ Suggestions are helpful
- ✓ Error codes are consistent

#### F-081: Validation
**Priority:** P0 (Must Have)  
**Description:** Validate inputs before execution

**Requirements:**
- Validate all CLI arguments
- Validate configuration files
- Validate project state
- Fail fast with clear messages
- Provide validation summaries

**Acceptance Criteria:**
- ✓ Invalid inputs caught early
- ✓ Validation messages are clear
- ✓ No silent failures

#### F-082: Graceful Degradation
**Priority:** P1 (Should Have)  
**Description:** Handle missing features gracefully

**Requirements:**
- Work without git when appropriate
- Work with partial configuration
- Provide helpful warnings
- Suggest fixes for common issues

**Acceptance Criteria:**
- ✓ Degrades gracefully
- ✓ Warnings are helpful
- ✓ Core features still work

---

## 7. Command Reference

### 7.1 Command Structure

```
workspace [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGS]
```

### 7.2 Global Options

**Important:** All global options apply to ALL subcommands. Every subcommand MUST respect these settings.

**Key Principle:** Global options are **completely independent** from each other:
- **Logging** (`--log-level`) controls what goes to **stderr**
- **Format** (`--format`) controls what goes to **stdout**
- They work together but don't affect each other

| Flag | Short | Description | Default | Output Stream |
|------|-------|-------------|---------|---------------|
| `--root <PATH>` | `-r` | Project root directory | Current directory | N/A |
| `--log-level <LEVEL>` | `-l` | Log level (silent\|error\|warn\|info\|debug\|trace) | info | **stderr** |
| `--format <FORMAT>` | `-f` | Output format (text\|json\|json-compact) | text | **stdout** |
| `--no-color` | | Disable colored output | false | both |
| `--config <PATH>` | `-c` | Path to config file | Auto-detect | N/A |
| `--help` | `-h` | Show help | | stdout |
| `--version` | `-V` | Show version | | stdout |

---

### Global Options Detailed Behavior

#### 1. `--root <PATH>` - Working Directory

Changes working directory before executing command.

**Behavior:**
- All file operations are relative to this path
- Config file lookup starts from this directory
- Git operations work in this directory

**Examples:**
```bash
# Run from different directory
workspace --root /path/to/project bump --dry-run

# Multiple projects
workspace --root ~/projects/app1 audit
workspace --root ~/projects/app2 audit

# Relative paths work too
workspace --root ../other-project changeset list
```

---

#### 2. `--log-level <LEVEL>` - Logging (stderr)

Controls verbosity of operation logs written to **stderr**.

**Levels:**
- `silent`: No logs at all
- `error`: Only critical errors
- `warn`: Errors + warnings
- `info`: General progress (default)
- `debug`: Detailed operations
- `trace`: Very verbose debugging

**Key Points:**
- ✅ Logs go to **stderr** only
- ✅ Works with **any** output format
- ✅ Can be completely disabled with `silent`
- ✅ Does NOT affect stdout

**Examples:**
```bash
# JSON output with NO logs (clean JSON only)
workspace --format json --log-level silent bump --dry-run > result.json

# JSON output WITH debug logs (logs to stderr, JSON to stdout)
workspace --format json --log-level debug bump --dry-run > result.json 2> logs.txt

# Text output with NO logs (clean output only)
workspace --log-level silent changeset list

# Debug logging with text output
workspace --log-level debug upgrade check
```

---

#### 3. `--format <FORMAT>` - Output Format (stdout)

Controls output format written to **stdout**.

**Formats:**
- `text`: Human-readable with colors and tables (default)
- `json`: Pretty-printed JSON
- `json-compact`: Compact JSON (mainly for audit)

**Key Points:**
- ✅ Output goes to **stdout** only
- ✅ Works with **any** log level
- ✅ JSON is never mixed with logs
- ✅ Does NOT affect stderr

**Examples:**
```bash
# JSON with info logging (logs to stderr, JSON to stdout)
workspace --format json bump --dry-run

# JSON with silent logging (ONLY JSON, no logs at all)
workspace --format json --log-level silent bump --dry-run

# Text with debug logging
workspace --format text --log-level debug changeset list

# Separate streams to different files
workspace --format json --log-level debug bump --execute > output.json 2> debug.log
```

---

#### 4. `--no-color` - Disable Colors

Disables ANSI color codes in both stderr and stdout.

**Behavior:**
- Removes colors from logs (stderr)
- Removes colors from text output (stdout)
- Respects NO_COLOR environment variable
- Has no effect on JSON output (already no colors)

**Examples:**
```bash
# No colors in output and logs
workspace --no-color changeset list

# Useful for CI/CD
workspace --no-color --log-level info audit

# Environment variable
NO_COLOR=1 workspace bump --dry-run

# File redirection (colors would appear as escape codes)
workspace --no-color audit > report.txt
```

---

#### 5. `--config <PATH>` - Config File Override

Override default config file location.

**Behavior:**
- Uses specified config instead of auto-detected one
- Path can be relative or absolute
- Useful for testing different configurations

**Examples:**
```bash
# Use specific config
workspace --config ./test-config.yaml init

# Test different strategies
workspace --config ./independent-config.yaml bump --dry-run
workspace --config ./unified-config.yaml bump --dry-run

# Absolute path
workspace --config /etc/myproject/config.json audit
```

---

### Combining Global Options

**Independence Examples:**

```bash
# 1. JSON output, NO logs (clean JSON only)
workspace --format json --log-level silent bump --dry-run
# stdout: {"success": true, ...}
# stderr: (nothing)

# 2. JSON output, DEBUG logs (logs separate from JSON)
workspace --format json --log-level debug bump --dry-run
# stdout: {"success": true, ...}
# stderr: DEBUG: Loading config...
#         DEBUG: Found 2 changesets...

# 3. Text output, NO logs (clean text only)
workspace --format text --log-level silent audit
# stdout: Audit Results...
# stderr: (nothing)

# 4. Text output, INFO logs (default behavior)
workspace --format text --log-level info bump --execute
# stdout: Version Bump Preview...
# stderr: INFO: Loading configuration...
#         INFO: Found 2 changesets...

# 5. All options combined
workspace --root ~/project \
    --config ./custom.yaml \
    --format json \
    --log-level debug \
    --no-color \
    bump --dry-run > output.json 2> debug.log

# 6. Silent JSON for automation (most common CI/CD use case)
workspace --format json --log-level silent bump --execute

# 7. Debug everything for troubleshooting
workspace --log-level trace --format text upgrade check

# 8. Different directory, no colors, with logs
workspace --root /other/project --no-color --log-level info audit
```

---

### Stream Separation Guarantee

**Always true regardless of options:**
- **stderr**: Logs only (controlled by `--log-level`)
- **stdout**: Final output only (controlled by `--format`)
- **Never mixed**: JSON is always valid, logs never appear in stdout

**Practical Examples:**

```bash
# Capture output and logs separately
workspace --format json bump --execute > result.json 2> process.log

# Discard logs, keep only output
workspace --format json bump --execute 2>/dev/null > result.json

# Discard output, keep only logs (unusual but possible)
workspace --log-level debug bump --execute >/dev/null 2> debug.log

# Everything to same file (not recommended)
workspace --format json bump --execute &> combined.log

# Silent mode - no logs, only output (best for scripting)
workspace --log-level silent --format json bump --execute
```

### 7.3 Commands

---

#### `workspace init`

Initialize project configuration.

**Usage:**
```bash
workspace init [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--changeset-path <PATH>` | Changeset directory | `.changesets/` |
| `--environments <LIST>` | Comma-separated environments | Prompt |
| `--default-env <LIST>` | Default environments | Prompt |
| `--strategy <STRATEGY>` | Versioning strategy (independent\|unified) | Prompt |
| `--registry <URL>` | NPM registry URL | `https://registry.npmjs.org` |
| `--format <FORMAT>` | Config format (json\|toml\|yaml) | Prompt |
| `--force` | Overwrite existing config | false |
| `--non-interactive` | No prompts, use defaults/flags | false |

**Note:** Supports global `--format json` flag for machine-readable output.

**Examples:**
```bash
# Interactive mode
workspace init

# Non-interactive with all options
workspace init --non-interactive --strategy unified --format yaml --environments "dev,staging,prod" --default-env "prod"

# Minimal non-interactive
workspace init --non-interactive --format json

# JSON output for automation
workspace init --non-interactive --format json > init-result.json
```

**Output (text format):**
```
✓ Configuration initialized successfully

  Config file: repo.config.yaml
  Strategy: independent
  Changesets: .changesets/
  Environments: dev, staging, production
  Default: production
```

**Output (--format json):**
```json
{
  "success": true,
  "configFile": "repo.config.yaml",
  "configFormat": "yaml",
  "strategy": "independent",
  "changesetPath": ".changesets/",
  "environments": ["dev", "staging", "production"],
  "defaultEnvironments": ["production"],
  "registry": "https://registry.npmjs.org"
}
```

---

#### `workspace config`

Manage configuration.

**Subcommands:**
- `workspace config show` - Display current configuration
- `workspace config validate` - Validate configuration file
- `workspace config get <KEY>` - Get specific config value
- `workspace config set <KEY> <VALUE>` - Set config value (future)

**Usage:**
```bash
workspace config show [OPTIONS]
workspace config validate [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--format <FORMAT>` | Output format (text\|json) |

**Note:** Supports global `--format json` flag for machine-readable output.

**Examples:**
```bash
# Show configuration
workspace config show

# Show as JSON
workspace config show --format json

# Validate configuration
workspace config validate

# Validate with JSON output
workspace config validate --format json
```

**Output (config show, text format):**
```
Configuration
━━━━━━━━━━━━━

Strategy: independent
Changeset Path: .changesets/
Environments: dev, staging, production
Default Environments: production
Registry: https://registry.npmjs.org
```

**Output (config show, --format json):**
```json
{
  "success": true,
  "config": {
    "changeset": {
      "path": ".changesets/",
      "environments": ["dev", "staging", "production"],
      "defaultEnvironments": ["production"]
    },
    "version": {
      "strategy": "independent",
      "defaultBump": "patch",
      "snapshotFormat": "{version}-{branch}.{short_commit}"
    },
    "registry": "https://registry.npmjs.org"
  }
}
```

**Output (config validate, text format):**
```
✓ Configuration is valid

All checks passed:
  ✓ Config file exists
  ✓ All required fields present
  ✓ Environments valid
  ✓ Changeset directory exists
```

**Output (config validate, --format json):**
```json
{
  "success": true,
  "valid": true,
  "checks": [
    { "name": "Config file exists", "passed": true },
    { "name": "All required fields present", "passed": true },
    { "name": "Environments valid", "passed": true },
    { "name": "Changeset directory exists", "passed": true }
  ]
}
```

---

#### `workspace changeset`

Manage changesets.

**Subcommands:**
- `workspace changeset create` - Create new changeset
- `workspace changeset update` - Update current changeset
- `workspace changeset list` - List all changesets
- `workspace changeset show <BRANCH>` - Show changeset details
- `workspace changeset delete <BRANCH>` - Delete changeset
- `workspace changeset history` - Query archived changesets
- `workspace changeset check` - Check if changeset exists (for hooks)

**Usage:**
```bash
workspace changeset create [OPTIONS]
workspace changeset update [ID] [OPTIONS]
workspace changeset list [OPTIONS]
workspace changeset show <BRANCH> [OPTIONS]
workspace changeset delete <BRANCH> [OPTIONS]
workspace changeset history [OPTIONS]
workspace changeset check [OPTIONS]
```

**Options (create):**
| Flag | Description | Default |
|------|-------------|---------|
| `--bump <TYPE>` | Bump type (major\|minor\|patch) | Prompt |
| `--env <LIST>` | Comma-separated environments | Prompt |
| `--branch <NAME>` | Branch name | Current branch |
| `--message <TEXT>` | Changeset message | Empty |
| `--packages <LIST>` | Comma-separated packages | Auto-detect |
| `--non-interactive` | No prompts | false |

**Arguments (update):**
| Argument | Description |
|----------|-------------|
| `<ID>` | Changeset ID or branch name (optional, default: current branch) |

**Options (update):**
| Flag | Description |
|------|-------------|
| `--commit <HASH>` | Add specific commit |
| `--packages <LIST>` | Add specific packages |

**Options (list):**
| Flag | Description |
|------|-------------|
| `--filter-package <NAME>` | Filter by package |
| `--filter-bump <TYPE>` | Filter by bump type |
| `--filter-env <ENV>` | Filter by environment |
| `--sort <FIELD>` | Sort by field (date\|bump\|branch) |

**Options (history):**
| Flag | Description |
|------|-------------|
| `--package <NAME>` | Filter by package |
| `--since <DATE>` | Since date (ISO 8601) |
| `--until <DATE>` | Until date (ISO 8601) |
| `--env <ENV>` | Filter by environment |
| `--bump <TYPE>` | Filter by bump type |
| `--limit <N>` | Limit results |

**Note:** All changeset commands support global `--format json` flag for machine-readable output.

**Examples:**
```bash
# Create changeset interactively
workspace changeset create

# Create with all options
workspace changeset create --bump minor --env "staging,prod" --message "Add new feature"

# Create with JSON output
workspace changeset create --bump minor --format json

# Update current branch's changeset (auto-detects branch)
workspace changeset update

# Update specific changeset by ID or branch name
workspace changeset update feature/my-feature

# List all changesets
workspace changeset list

# List with filtering
workspace changeset list --filter-bump major --sort date

# List as JSON
workspace changeset list --format json

# Show specific changeset
workspace changeset show feature/new-component

# Show as JSON
workspace changeset show feature/new-component --format json

# Delete changeset with confirmation
workspace changeset delete old-feature

# Force delete without confirmation
workspace changeset delete old-feature --force

# Query history
workspace changeset history --package @myorg/core --since 2024-01-01

# Query history as JSON
workspace changeset history --format json

# Check if changeset exists (for Git hooks)
workspace changeset check

# Check with JSON output
workspace changeset check --format json
```

**Output (list, text format):**
```
Active Changesets
━━━━━━━━━━━━━━━━━

feature/new-api (minor)
  Packages: @myorg/core
  Environments: production
  Commits: 5
  Created: 2024-01-15

hotfix/security (patch)
  Packages: @myorg/utils, @myorg/cli
  Environments: production, staging
  Commits: 2
  Created: 2024-01-14

Total: 2 changesets
```

**Output (list, --format json):**
```json
{
  "success": true,
  "changesets": [
    {
      "id": "feature-new-api",
      "branch": "feature/new-api",
      "bump": "minor",
      "packages": ["@myorg/core"],
      "environments": ["production"],
      "commits": ["abc123", "def456", "ghi789", "jkl012", "mno345"],
      "createdAt": "2024-01-15T10:00:00Z",
      "updatedAt": "2024-01-15T14:30:00Z"
    },
    {
      "id": "hotfix-security",
      "branch": "hotfix/security",
      "bump": "patch",
      "packages": ["@myorg/utils", "@myorg/cli"],
      "environments": ["production", "staging"],
      "commits": ["pqr678", "stu901"],
      "createdAt": "2024-01-14T09:00:00Z",
      "updatedAt": "2024-01-14T09:15:00Z"
    }
  ],
  "total": 2
}
```

**Output (show, --format json):**
```json
{
  "success": true,
  "changeset": {
    "id": "feature-new-api",
    "branch": "feature/new-api",
    "bump": "minor",
    "packages": ["@myorg/core"],
    "environments": ["production"],
    "commits": ["abc123", "def456"],
    "createdAt": "2024-01-15T10:00:00Z",
    "updatedAt": "2024-01-15T14:30:00Z"
  }
}
```

**Output (check, --format json):**
```json
{
  "success": true,
  "exists": true,
  "changeset": {
    "id": "feature-new-api",
    "branch": "feature/new-api"
  }
}
```

---

#### `workspace bump`

Bump package versions based on changesets.

**Behavior:**
- **Single Repository**: Bumps the single package version based on active changesets
- **Monorepo (Independent)**: Bumps only packages listed in active changesets
- **Monorepo (Unified)**: Bumps all workspace packages to the same version when any changeset exists

**Usage:**
```bash
workspace bump [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--dry-run` | Preview changes without applying | false |
| `--execute` | Apply version changes | false (requires explicit) |
| `--snapshot` | Generate snapshot versions | false |
| `--snapshot-format <FORMAT>` | Snapshot format template | From config |
| `--prerelease <TAG>` | Pre-release tag (alpha\|beta\|rc) | None |
| `--packages <LIST>` | Only bump specific packages (overrides changeset packages) | All affected |
| `--git-tag` | Create git tags for releases | false |
| `--git-push` | Push git tags to remote (requires --git-tag) | false |
| `--git-commit` | Commit version changes | false |
| `--no-changelog` | Don't update changelogs | false |
| `--no-archive` | Don't archive changesets | false |
| `--force` | Skip confirmations | false |

**Examples:**
```bash
# Preview version changes
workspace bump --dry-run

# Preview with JSON output (for CI/CD)
workspace bump --dry-run --format json

# Execute version bump (only update files)
workspace bump --execute

# Execute with git operations
workspace bump --execute --git-tag --git-push

# Execute with git commit and tags
workspace bump --execute --git-commit --git-tag

# Generate snapshot versions for feature branch
workspace bump --snapshot --execute

# Create pre-release versions
workspace bump --prerelease beta --execute

# Bump specific packages only
workspace bump --packages "@myorg/core,@myorg/utils" --execute --dry-run

# Full CI/CD workflow
workspace bump --execute --git-commit --git-tag --git-push --force
```

**Output (dry-run, Independent Strategy):**
```
Version Bump Preview
━━━━━━━━━━━━━━━━━━━━
Strategy: Independent

Packages to bump (from changesets):
  @myorg/core: 1.2.3 → 1.3.0 (minor, direct change)
  @myorg/utils: 2.0.1 → 2.1.0 (minor, dependency propagation)

Packages unchanged:
  @myorg/cli: 0.5.0 (no changes)
  @myorg/docs: 1.0.0 (no changes)

Changesets to process:
  ✓ feature/new-api (minor, 5 commits, packages: @myorg/core)
  ✓ feature/fix-bug (patch, 2 commits, packages: @myorg/utils)

Git tags to create:
  @myorg/core@1.3.0
  @myorg/utils@2.1.0

Run with --execute to apply changes.
```

**Output (dry-run, Unified Strategy):**
```
Version Bump Preview
━━━━━━━━━━━━━━━━━━━━
Strategy: Unified

All packages will be bumped to: 1.3.0
  @myorg/core: 1.2.3 → 1.3.0 (minor bump applied)
  @myorg/utils: 2.0.1 → 1.3.0 (unified version)
  @myorg/cli: 0.5.0 → 1.3.0 (unified version)
  @myorg/docs: 1.0.0 → 1.3.0 (unified version)

Changesets to process:
  ✓ feature/new-api (minor, 5 commits, packages: @myorg/core)
  ✓ feature/fix-bug (patch, 2 commits, packages: @myorg/utils)

Highest bump type: minor (determines unified version)

Git tags to create:
  @myorg/core@1.3.0
  @myorg/utils@1.3.0
  @myorg/cli@1.3.0
  @myorg/docs@1.3.0

Run with --execute to apply changes.
```

**JSON Output (--format json --dry-run):**
```json
{
  "success": true,
  "strategy": "independent",
  "packages": [
    {
      "name": "@myorg/core",
      "path": "packages/core",
      "currentVersion": "1.2.3",
      "nextVersion": "1.3.0",
      "bump": "minor",
      "reason": "direct",
      "willBump": true
    },
    {
      "name": "@myorg/utils",
      "path": "packages/utils",
      "currentVersion": "2.0.1",
      "nextVersion": "2.1.0",
      "bump": "minor",
      "reason": "dependency_propagation",
      "willBump": true
    },
    {
      "name": "@myorg/cli",
      "path": "packages/cli",
      "currentVersion": "0.5.0",
      "nextVersion": "0.5.0",
      "bump": "none",
      "reason": "no_changes",
      "willBump": false
    }
  ],
  "changesets": [
    {
      "id": "cs_abc123",
      "branch": "feature/new-api",
      "bump": "minor",
      "commits": 5
    }
  ],
  "tags": ["@myorg/core@1.3.0"],
  "summary": {
    "totalPackages": 3,
    "totalChangesets": 2,
    "totalTags": 3
  }
}
```

---

#### `workspace upgrade`

Manage dependency upgrades.

**Subcommands:**
- `workspace upgrade check` - Detect available upgrades
- `workspace upgrade apply` - Apply upgrades
- `workspace upgrade backups` - Manage backups

**Usage:**
```bash
workspace upgrade check [OPTIONS]
workspace upgrade apply [OPTIONS]
workspace upgrade backups list [OPTIONS]
workspace upgrade backups restore <ID> [OPTIONS]
workspace upgrade backups clean [OPTIONS]
```

**Options (check):**
| Flag | Description | Default |
|------|-------------|---------|
| `--major` | Include major upgrades | true |
| `--minor` | Include minor upgrades | true |
| `--patch` | Include patch upgrades | true |
| `--dev` | Include dev dependencies | true |
| `--peer` | Include peer dependencies | false |
| `--packages <LIST>` | Only check specific packages | All |
| `--registry <URL>` | Override registry URL | From config |

**Options (apply):**
| Flag | Description | Default |
|------|-------------|---------|
| `--dry-run` | Preview without applying | false |
| `--patch-only` | Only apply patch upgrades | false |
| `--minor-and-patch` | Only minor and patch | false |
| `--packages <LIST>` | Only upgrade specific packages | All detected |
| `--auto-changeset` | Automatically create changeset | false |
| `--changeset-bump <TYPE>` | Changeset bump type | patch |
| `--no-backup` | Skip backup creation | false |
| `--force` | Skip confirmations | false |

**Note:** All upgrade commands support global `--format json` flag for machine-readable output.

**Examples:**
```bash
# Check for all upgrades
workspace upgrade check

# Check with JSON output
workspace upgrade check --format json

# Check patch upgrades only
workspace upgrade check --no-major --no-minor

# Check specific packages
workspace upgrade check --packages "typescript,eslint"

# Apply all patch upgrades with auto-changeset
workspace upgrade apply --patch-only --auto-changeset

# Apply specific upgrades
workspace upgrade apply --packages "@types/node,typescript"

# Dry-run to see what would be upgraded
workspace upgrade apply --dry-run

# Apply with JSON output
workspace upgrade apply --format json

# List backups
workspace upgrade backups list

# Restore backup
workspace upgrade backups restore backup_20240115_103045

# Clean old backups (keep last 5)
workspace upgrade backups clean --keep 5
```

**Output (check):**
```
Dependency Upgrades Available
━━━━━━━━━━━━━━━━━━━━━━━━━━━━

@myorg/core:
  ┌──────────────────┬─────────┬─────────┬────────┐
  │ Package          │ Current │ Latest  │ Type   │
  ├──────────────────┼─────────┼─────────┼────────┤
  │ typescript       │ 5.0.0   │ 5.3.3   │ minor  │
  │ eslint           │ 8.0.0   │ 9.0.0   │ major  │
  │ vitest           │ 1.0.0   │ 1.2.1   │ minor  │
  └──────────────────┴─────────┴─────────┴────────┘

Summary:
  Total upgrades: 15
  Major: 3
  Minor: 8
  Patch: 4
```

**Output (check, --format json):**
```json
{
  "success": true,
  "packages": [
    {
      "name": "@myorg/core",
      "path": "packages/core",
      "upgrades": [
        {
          "package": "typescript",
          "currentVersion": "5.0.0",
          "latestVersion": "5.3.3",
          "type": "minor",
          "breaking": false
        },
        {
          "package": "eslint",
          "currentVersion": "8.0.0",
          "latestVersion": "9.0.0",
          "type": "major",
          "breaking": true
        }
      ]
    }
  ],
  "summary": {
    "totalUpgrades": 15,
    "major": 3,
    "minor": 8,
    "patch": 4
  }
}
```

**Output (apply, --format json):**
```json
{
  "success": true,
  "applied": [
    {
      "package": "typescript",
      "from": "5.0.0",
      "to": "5.3.3",
      "type": "minor"
    }
  ],
  "skipped": [
    {
      "package": "eslint",
      "reason": "major_version",
      "currentVersion": "8.0.0",
      "latestVersion": "9.0.0"
    }
  ],
  "summary": {
    "totalApplied": 12,
    "totalSkipped": 3,
    "backupId": "backup_20240115_103045"
  }
}
```

---

#### `workspace audit`

Run project health audit.

**Usage:**
```bash
workspace audit [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--sections <LIST>` | Sections to audit (all\|upgrades\|dependencies\|version-consistency\|breaking-changes) | all |
| `--format <FORMAT>` | Output format (text\|markdown\|json\|json-compact) | text |
| `--output <PATH>` | Write to file | stdout |
| `--min-severity <LEVEL>` | Minimum severity (critical\|warning\|info) | info |
| `--verbosity <LEVEL>` | Detail level (minimal\|normal\|detailed) | normal |
| `--no-health-score` | Skip health score calculation | false |

**Examples:**
```bash
# Full audit
workspace audit

# Specific sections
workspace audit --sections upgrades,dependencies

# Generate markdown report
workspace audit --format markdown --output audit-report.md

# JSON for CI/CD
workspace audit --format json

# JSON compact for CI/CD
workspace audit --format json-compact

# Only critical and warning severity issues
workspace audit --min-severity warning

# Detailed output
workspace audit --verbosity detailed
```

**Output (text format):**
```
Project Health Audit
━━━━━━━━━━━━━━━━━━━━

Health Score: 78/100 (Good)
  Upgrades:           85/100
  Dependencies:       72/100
  Version Consistency: 80/100
  Breaking Changes:   75/100

Summary:
  Total Issues: 12
  Critical: 0
  High: 2
  Medium: 5
  Low: 5

━━━━━━━━━━━━━━━━━━━━
Upgrade Audit
━━━━━━━━━━━━━━━━━━━━

[HIGH] 3 major upgrades available
  Security or feature updates recommended
  
  Affected packages:
    - eslint: 8.0.0 → 9.0.0
    - vite: 4.0.0 → 5.0.0

[MEDIUM] 8 minor upgrades available
  Consider updating for new features
  
━━━━━━━━━━━━━━━━━━━━
Dependency Audit
━━━━━━━━━━━━━━━━━━━━

[HIGH] Circular dependency detected
  @myorg/core → @myorg/utils → @myorg/core
  
  Recommendation: Refactor to remove cycle

[MEDIUM] 2 deprecated packages found
  - request: 2.88.0 (deprecated, use axios instead)
  - mkdirp: 0.5.0 (deprecated, use fs.mkdir recursive)

[INFO] Dependency categorization:
  Internal: 12 packages
  External: 145 packages
  Workspace: 8 links
  Local: 2 links

━━━━━━━━━━━━━━━━━━━━
Recommendations
━━━━━━━━━━━━━━━━━━━━

1. Address circular dependency between core and utils
2. Replace deprecated packages (request, mkdirp)
3. Consider upgrading major versions for security
4. Run 'workspace upgrade check' for detailed upgrade info
```

---

#### `workspace changes`

Analyze changes in repository.

**Usage:**
```bash
workspace changes [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--since <REF>` | Since commit/branch/tag | None (working dir) |
| `--until <REF>` | Until commit/branch/tag | HEAD |
| `--branch <NAME>` | Compare against branch | None |
| `--staged` | Only staged changes | false |
| `--unstaged` | Only unstaged changes | false |
| `--packages <LIST>` | Filter by packages | All |

**Note:** Supports global `--format json` flag for machine-readable output.

**Examples:**
```bash
# Analyze working directory changes
workspace changes

# Changes since last tag
workspace changes --since $(git describe --tags --abbrev=0)

# Changes between commits
workspace changes --since abc123 --until def456

# Changes in current branch vs main
workspace changes --branch main

# Only staged changes
workspace changes --staged

# JSON output for CI/CD
workspace changes --format json
```

**Output (text format):**
```
Changes Analysis
━━━━━━━━━━━━━━━━

Affected Packages: 3

@myorg/core:
  Files changed: 5
  Lines added: 145
  Lines deleted: 32
  
  Changes:
    M src/index.ts
    M src/utils.ts
    A src/new-feature.ts
    D src/old-code.ts

@myorg/utils:
  Files changed: 2
  Lines added: 45
  Lines deleted: 12

Summary:
  Total files: 7
  Total packages: 3
  Lines added: 190
  Lines deleted: 44
```

**Output (--format json):**
```json
{
  "success": true,
  "affectedPackages": [
    {
      "name": "@myorg/core",
      "path": "packages/core",
      "filesChanged": 5,
      "linesAdded": 145,
      "linesDeleted": 32,
      "changes": [
        { "type": "modified", "path": "src/index.ts" },
        { "type": "modified", "path": "src/utils.ts" },
        { "type": "added", "path": "src/new-feature.ts" },
        { "type": "deleted", "path": "src/old-code.ts" }
      ]
    },
    {
      "name": "@myorg/utils",
      "path": "packages/utils",
      "filesChanged": 2,
      "linesAdded": 45,
      "linesDeleted": 12,
      "changes": [
        { "type": "modified", "path": "src/helper.ts" },
        { "type": "modified", "path": "src/validator.ts" }
      ]
    }
  ],
  "summary": {
    "totalFiles": 7,
    "totalPackages": 3,
    "linesAdded": 190,
    "linesDeleted": 44
  }
}
```

---

#### `workspace clone`

Clone a repository and setup workspace automatically.

**Usage:**
```bash
workspace clone <URL> [DESTINATION] [OPTIONS]
```

**Arguments:**
| Argument | Description | Required |
|----------|-------------|----------|
| `<URL>` | Repository URL (HTTPS or SSH) | Yes |
| `[DESTINATION]` | Local path for cloned repository | No (defaults to repo name) |

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--changeset-path <PATH>` | Changeset directory | `.changesets` |
| `--environments <LIST>` | Comma-separated environments | Prompt |
| `--default-env <LIST>` | Default environments | Prompt |
| `--strategy <STRATEGY>` | Versioning strategy (independent\|unified) | Prompt |
| `--registry <URL>` | NPM registry URL | `https://registry.npmjs.org` |
| `--config-format <FORMAT>` | Config format (json\|toml\|yaml) | Prompt |
| `--non-interactive` | No prompts, use defaults/flags | false |
| `--skip-validation` | Skip validation if config exists | false |
| `--force` | Remove existing destination | false |
| `--depth <N>` | Create shallow clone with depth N | Full clone |

**Note:** Options related to init (changeset-path, environments, strategy, etc.) are only used if the cloned repository does not have workspace configuration.

**Examples:**
```bash
# Clone with default destination (./api/)
workspace clone https://github.com/company/api.git

# Clone to specific destination
workspace clone https://github.com/company/api.git ~/dev/api

# Clone private repo (SSH)
workspace clone git@github.com:company/private-api.git

# Clone with non-interactive setup
workspace clone https://github.com/user/lib.git ./my-lib \
  --non-interactive \
  --strategy independent \
  --environments "dev,staging,prod"

# Clone and skip validation (faster)
workspace clone https://github.com/company/api.git --skip-validation

# Force clone (remove existing destination)
workspace clone https://github.com/company/api.git --force

# Shallow clone (faster, less history)
workspace clone https://github.com/company/huge-repo.git --depth 1

# Clone with JSON output (for scripts)
workspace --format json clone https://github.com/company/api.git ./api
```

**Behavior:**

1. **Clone Phase:**
   - Validates URL format
   - Checks destination doesn't exist (unless --force)
   - Clones repository with progress bar
   - Handles authentication (SSH keys, credentials)
   - Reports clone errors (network, auth, disk space)

2. **Detection Phase:**
   - Searches for configuration file (repo.config.yaml/json/toml)
   - Checks for workspace directories

3. **Path A - Configuration Exists:**
   - Validates configuration file can be parsed
   - Checks all required directories exist
   - Validates .gitignore has required entries
   - Reports validation results:
     - **Success**: Shows "Workspace is ready!"
     - **Failure**: Lists errors with suggested fixes
   - Skipped if --skip-validation flag used

4. **Path B - No Configuration:**
   - Automatically runs init command
   - Uses interactive prompts or provided flags
   - Creates configuration and directory structure
   - Updates .gitignore
   - Reports init success

**Output (text format, config exists and valid):**
```
⠋ Cloning repository from https://github.com/company/api.git...
Receiving objects: 100% (1523/1523), 2.4 MiB | 1.2 MiB/s, done.
Resolving deltas: 100% (891/891), done.
✓ Repository cloned successfully to ~/dev/api

⠋ Detecting workspace configuration...
✓ Configuration found: repo.config.yaml

⠋ Validating workspace...
✓ Configuration file is valid
✓ Strategy: independent
✓ Changesets directory exists (.changesets/)
✓ History directory exists (.changesets/history/)
✓ Backup directory exists (.workspace-backups/)
✓ Git ignore configured correctly

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Workspace cloned and validated successfully!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Next steps:
  cd ~/dev/api
  workspace changeset list
  workspace bump --dry-run
```

**Output (text format, no config, init runs):**
```
⠋ Cloning repository from https://github.com/user/lib.git...
Receiving objects: 100% (342/342), 456 KiB | 228 KiB/s, done.
✓ Repository cloned successfully to ./my-lib

⠋ Detecting workspace configuration...
ℹ No workspace configuration found

Initializing workspace...

Detected: single package project

? Which environments do you want available? › 
  ◯ development
  ◉ staging
  ◉ production
  
[... init prompts continue ...]

✓ Workspace initialized successfully!

Next steps:
  cd ./my-lib
  workspace changeset create
```

**Output (JSON format):**
```json
{
  "success": true,
  "url": "https://github.com/company/api.git",
  "destination": "/Users/dev/api",
  "config_exists": true,
  "init_executed": false,
  "validation": {
    "is_valid": true,
    "strategy": "independent",
    "checks": [
      {
        "name": "Configuration file",
        "passed": true,
        "error": null,
        "suggestion": null
      },
      {
        "name": "Changesets directory",
        "passed": true,
        "error": null,
        "suggestion": null
      },
      {
        "name": "History directory",
        "passed": true,
        "error": null,
        "suggestion": null
      },
      {
        "name": "Backups directory",
        "passed": true,
        "error": null,
        "suggestion": null
      },
      {
        "name": ".gitignore configuration",
        "passed": true,
        "error": null,
        "suggestion": null
      }
    ]
  }
}
```

**Exit Codes:**
- `0` - Success (cloned and configured or validated)
- `1` - Clone failed (network, auth, or URL invalid)
- `2` - Destination exists (use --force to overwrite)
- `3` - Validation failed (config exists but invalid)
- `4` - Init failed (no config, init unsuccessful)

**Error Examples:**

**Network failure:**
```
Error: Failed to clone repository
  Could not connect to https://github.com/company/api.git
  
  Possible causes:
  - No internet connection
  - Repository does not exist
  - Server is down
  
  Suggestions:
  - Check your internet connection
  - Verify the repository URL is correct
  - Try again later
```

**Authentication failure:**
```
Error: Failed to clone repository
  Authentication required for git@github.com:company/private-api.git
  
  Possible causes:
  - SSH key not configured
  - SSH key not added to ssh-agent
  - Wrong permissions on SSH key
  
  Suggestions:
  - Generate SSH key: ssh-keygen -t ed25519 -C "your_email@example.com"
  - Add to ssh-agent: ssh-add ~/.ssh/id_ed25519
  - Add public key to GitHub: https://github.com/settings/keys
```

**Destination exists:**
```
Error: Destination already exists
  Path ~/dev/api already exists
  
  Suggestions:
  - Use a different destination path
  - Remove existing directory: rm -rf ~/dev/api
  - Use --force flag to overwrite: workspace clone <url> ~/dev/api --force
```

**Validation failure:**
```
✓ Repository cloned to ~/dev/broken
✓ Configuration found: repo.config.json
✗ Validation failed

Errors found:
  ✗ Missing directory: .changesets/history/
     Suggestion: mkdir .changesets/history
  
  ✗ Invalid strategy in config: "wrongvalue"
     Expected: "independent" or "unified"
     Suggestion: Edit repo.config.json and fix strategy value
  
  ✗ Missing .gitignore entry: .workspace-backups/
     Suggestion: Add '.workspace-backups/' to .gitignore

To fix:
  1. Run: workspace config validate
     (for detailed validation report)
  
  2. Run: workspace init --force
     (to reinitialize with correct configuration)
```

---

#### `workspace version`

Display version information.

**Usage:**
```bash
workspace version [OPTIONS]
workspace --version
workspace -V
```

**Options:**
| Flag | Description |
|------|-------------|
| `--verbose` | Show detailed version info |

**Examples:**
```bash
# Simple version
workspace version
workspace --version

# Detailed version info
workspace version --verbose

# JSON output
workspace version --format json
```

**Output (text format):**
```
workspace 0.1.0
```

**Output (--verbose, text format):**
```
workspace 0.1.0

  Rust version: 1.75.0
  sublime-package-tools: 0.1.0
  sublime-standard-tools: 0.1.0
  sublime-git-tools: 0.1.0

Build:
  Profile: release
  Target: x86_64-apple-darwin
  Features: default
```

**Output (--format json):**
```json
{
  "success": true,
  "version": "0.1.0",
  "rustVersion": "1.75.0",
  "dependencies": {
    "sublime-package-tools": "0.1.0",
    "sublime-standard-tools": "0.1.0",
    "sublime-git-tools": "0.1.0"
  },
  "build": {
    "profile": "release",
    "target": "x86_64-apple-darwin",
    "features": ["default"]
  }
}
```

**Output:**
```
workspace 0.1.0

# Verbose:
workspace 0.1.0
  sublime_package_tools: 0.1.0
  sublime_standard_tools: 0.1.0
  sublime_git_tools: 0.1.0
  
Platform: macOS (aarch64)
Rustc: 1.75.0
```

---

#### `workspace help`

Show help information.

**Usage:**
```bash
workspace help [COMMAND]
workspace <COMMAND> --help
workspace --help
```

**Examples:**
```bash
# General help
workspace help
workspace --help

# Command-specific help
workspace help changeset
workspace changeset create --help
```

---

### 7.4 Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Configuration error |
| 4 | Git error |
| 5 | File system error |
| 10 | Validation failed |
| 11 | Changeset error |
| 12 | Version resolution error |
| 13 | Upgrade error |
| 14 | Audit error |

---

## 8. Technical Architecture

### 8.1 Technology Stack

**Core:**
- Rust (latest stable)
- Tokio async runtime (if needed for async operations)

**CLI Framework:**
- clap v4.5 with derive macros for argument parsing

**UI/Terminal:**
- crossterm v0.29 for cross-platform terminal control
- console v0.16 for terminal abstraction and styling
- indicatif v0.18 for progress bars and spinners
- comfy-table v7.2 for beautiful table rendering

**Interactive Components (Choose One):**
- dialoguer v0.12 (mature, feature-complete) **OR**
- cliclack v0.3 (modern, minimal aesthetic - inspired by Clack NPM package)

**Styling (Optional):**
- owo-colors v4.2 for zero-allocation colors (if performance critical)

**Serialization:**
- serde v1.0 with derive features
- serde_json v1.0 for JSON
- toml v0.8 for TOML
- serde_yaml v0.9 for YAML

**Error Handling:**
- anyhow v1.0 for CLI binary error handling
- thiserror v1.0 for library-exported typed errors

**Logging:**
- tracing v0.1 for structured logging
- tracing-subscriber v0.3 for log formatting

**Utilities:**
- terminal_size v0.4 for terminal dimensions
- clap_complete v4.5 for shell completion generation
- sysexits v0.10 for standard exit codes

**Our Crates:**
- sublime_package_tools (pkg crate)
- sublime_standard_tools (standard crate)
- sublime_git_tools (git crate)

**Complete Cargo.toml Dependencies:**
```toml
[dependencies]
# Core CLI
clap = { version = "4.5", features = ["derive"] }
crossterm = "0.29"
console = "0.16"

# Interactive (Choose One)
dialoguer = "0.12"  # Mature, feature-complete
# cliclack = "0.3"  # Modern, minimal (alternative)

# Progress & Tables
indicatif = "0.18"
comfy-table = "7.2"

# Styling (optional)
# owo-colors = "4.2"  # Zero-alloc colors if needed

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
serde_yaml = "0.9"

# Error Handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Utilities
terminal_size = "0.4"
clap_complete = "4.5"
sysexits = "0.10"

# Our crates
sublime-package-tools = { path = "../pkg" }
sublime-standard-tools = { path = "../standard" }
sublime-git-tools = { path = "../git" }
```

### 8.2 Module Structure

```
crates/cli/
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs                  # Library exports
│   ├── cli/
│   │   ├── mod.rs              # CLI argument parsing
│   │   ├── commands/           # Command implementations
│   │   │   ├── mod.rs
│   │   │   ├── init.rs
│   │   │   ├── config.rs
│   │   │   ├── changeset.rs
│   │   │   ├── bump.rs
│   │   │   ├── upgrade.rs
│   │   │   ├── audit.rs
│   │   │   ├── changes.rs
│   │   │   └── version.rs
│   │   └── args.rs             # Argument structures
│   ├── ui/
│   │   ├── mod.rs              # UI module
│   │   ├── components/         # Reusable UI components
│   │   │   ├── mod.rs
│   │   │   ├── spinner.rs
│   │   │   ├── progress.rs
│   │   │   ├── table.rs
│   │   │   ├── list.rs
│   │   │   ├── prompt.rs
│   │   │   ├── confirm.rs
│   │   │   └── select.rs
│   │   ├── theme.rs            # Color theme and styles
│   │   ├── formatter.rs        # Output formatting
│   │   └── display.rs          # Display helpers
│   ├── output/
│   │   ├── mod.rs              # Output module
│   │   ├── json.rs             # JSON output
│   │   ├── markdown.rs         # Markdown output
│   │   └── text.rs             # Text output
│   ├── error.rs                # CLI error types
│   └── utils.rs                # Utility functions
├── Cargo.toml
└── README.md
```

### 8.3 Error Handling Strategy

1. **Error Types:**
   - Wrap underlying crate errors with context
   - Provide CLI-specific error variants
   - Include suggestions for resolution

2. **Error Display:**
   - User-friendly messages by default
   - Verbose mode for debugging
   - JSON mode for programmatic handling

3. **Error Recovery:**
   - Graceful degradation where possible
   - Clear rollback instructions
   - Preserve state on failures

### 8.4 Performance Considerations

1. **Startup Time:**
   - Target < 100ms cold start
   - Lazy initialization where possible
   - Minimize dependency tree

2. **Command Execution:**
   - Parallel operations where safe
   - Streaming output for long operations
   - Cancellable operations (Ctrl+C)

3. **Memory Usage:**
   - Stream large files
   - Limit in-memory data structures
   - Release resources promptly

### 8.5 Testing Strategy

1. **Unit Tests:**
   - Test each command handler
   - Test UI components
   - Test formatters

2. **Integration Tests:**
   - Test with real project structures
   - Test Git integration
   - Test configuration loading

3. **End-to-End Tests:**
   - Test complete workflows
   - Test error scenarios
   - Test on all platforms

4. **Snapshot Tests:**
   - Test output formatting
   - Test JSON schemas
   - Test help text

---

## 9. UI/UX Design

### 9.1 Design Principles

1. **Minimal and Modern:**
   - Clean, uncluttered output
   - Consistent visual hierarchy
   - Thoughtful use of whitespace

2. **Progressive Disclosure:**
   - Show essential information by default
   - More details available with flags
   - Helpful hints without overwhelming

3. **Feedback and Confirmation:**
   - Always confirm destructive actions
   - Show progress for long operations
   - Clear success/failure indicators

4. **Consistency:**
   - Consistent command structure
   - Consistent flag names
   - Consistent output format

### 9.2 Visual Design

**Header:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
workspace-tools v0.1.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Color Scheme:**
- Success: Green
- Error: Red
- Warning: Yellow
- Info: Blue
- Highlight: Cyan
- Dimmed: Gray

**Icons/Symbols:**
- ✓ Success
- ✗ Error
- ⚠ Warning
- ℹ Info
- → Arrow/Flow
- ┌─ Table borders
- • Bullet points

**Progress Indicators:**
- Spinner for indeterminate: `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`
- Progress bar: `[████████░░░░░░░░░░░░] 40%`

### 9.3 Interactive Components

**Prompts:**
```
? What bump type will this be? (Use arrow keys)
  ❯ patch - Backward compatible bug fixes
    minor - New backward compatible features
    major - Breaking changes
```

**Multi-Select:**
```
? Which environments should this target? (Space to select, Enter to confirm)
  ◯ development
  ◉ staging
  ◉ production
  ◯ qa
```

**Confirmation:**
```
? Apply these upgrades? This will modify 3 package.json files. (y/N)
```

**Input:**
```
? Enter changeset message (optional):
  › Add new authentication system
```

### 9.4 Table Rendering

**Simple Table:**
```
┌──────────────────┬─────────┬─────────┬────────┐
│ Package          │ Current │ Latest  │ Type   │
├──────────────────┼─────────┼─────────┼────────┤
│ typescript       │ 5.0.0   │ 5.3.3   │ minor  │
│ eslint           │ 8.0.0   │ 9.0.0   │ major  │
│ vitest           │ 1.0.0   │ 1.2.1   │ minor  │
└──────────────────┴─────────┴─────────┴────────┘
```

**List with Details:**
```
Affected Packages:

@myorg/core
  Version: 1.2.3 → 1.3.0 (minor)
  Reason: Direct change from changeset
  Files: 5 changed

@myorg/utils
  Version: 2.0.1 → 2.1.0 (minor)
  Reason: Propagated from @myorg/core
  Files: 2 changed
```

### 9.5 Help Text Format

```
workspace-changeset-create
Create a new changeset for the current branch

USAGE:
    workspace changeset create [OPTIONS]

OPTIONS:
    --bump <TYPE>
            Bump type for this changeset
            
            [possible values: major, minor, patch]
            
    --env <LIST>
            Comma-separated list of environments
            
            [example: production,staging]
            
    --message <TEXT>
            Optional description of changes
            
    --non-interactive
            Skip interactive prompts
            
    -h, --help
            Print help information

EXAMPLES:
    # Interactive mode (prompts for all options)
    workspace changeset create
    
    # Non-interactive with options
    workspace changeset create --bump minor --env production,staging
    
    # With message
    workspace changeset create --bump patch --message "Fix critical bug"
```

---

## 10. Installation and Distribution

### 10.1 Installation Methods

#### Method 1: Curl Script (Recommended)
```bash
curl -fsSL https://raw.githubusercontent.com/org/repo/main/install.sh | sh
```

The script will:
1. Detect OS and architecture
2. Download appropriate binary
3. Verify checksum
4. Install to `/usr/local/bin` (or user-specified location)
5. Verify installation

**Script features:**
- Support for macOS (Intel/ARM), Linux (x64/ARM), Windows
- Checksum verification
- Version selection
- Custom install location
- Unattended mode

#### Method 2: Homebrew (macOS/Linux)
```bash
brew install org/tap/workspace
```

#### Method 3: Cargo
```bash
cargo install workspace
```

#### Method 4: Pre-built Binaries
Download from GitHub Releases for your platform:
- `workspace-macos-x86_64.tar.gz`
- `workspace-macos-aarch64.tar.gz`
- `workspace-linux-x86_64.tar.gz`
- `workspace-linux-aarch64.tar.gz`
- `workspace-windows-x86_64.zip`

#### Method 5: Package Managers
- **Arch Linux:** AUR package
- **Debian/Ubuntu:** .deb package
- **Windows:** Scoop, Chocolatey

### 10.2 Installation Script Specification

**Location:** `scripts/install.sh`

**Features:**
- OS detection (macOS, Linux, Windows via Git Bash)
- Architecture detection (x86_64, aarch64, arm)
- Version selection (latest, specific version)
- Custom install directory
- Checksum verification (SHA256)
- Proper error handling and rollback
- Colored output
- Verbose mode

**Usage:**
```bash
# Install latest version
curl -fsSL https://install.workspace.dev | sh

# Install specific version
curl -fsSL https://install.workspace.dev | sh -s -- --version v0.1.0

# Custom install location
curl -fsSL https://install.workspace.dev | sh -s -- --install-dir ~/.local/bin

# Verbose output
curl -fsSL https://install.workspace.dev | sh -s -- --verbose
```

### 10.3 Build Configuration

**Cargo.toml features:**
```toml
[profile.release]
opt-level = "z"          # Optimize for size
lto = true               # Enable Link Time Optimization
codegen-units = 1        # Better optimization
strip = true             # Strip symbols
panic = "abort"          # Smaller binary

[dependencies]
# Minimize dependency tree
# Use features to include only what's needed
```

**Binary sizes target:**
- macOS: < 10 MB
- Linux: < 10 MB
- Windows: < 12 MB

### 10.4 CI/CD Pipeline

**GitHub Actions Workflow:**

1. **Build Matrix:**
   - OS: macOS (Intel/ARM), Linux (x64), Windows
   - Rust: stable, beta

2. **Steps:**
   - Checkout code
   - Setup Rust toolchain
   - Cache dependencies
   - Run tests
   - Run Clippy
   - Build release binary
   - Strip and compress
   - Generate checksums
   - Upload artifacts

3. **Release Process:**
   - Tag commit
   - Build for all platforms
   - Create GitHub Release
   - Upload binaries and checksums
   - Update Homebrew tap
   - Publish to crates.io

### 10.5 Git Hook Examples

**Note:** The CLI does not provide git hook functionality itself. It's designed to be fast enough to be called from git hooks. Below are example hook scripts.

**Example: post-commit hook**

`.git/hooks/post-commit`:
```bash
#!/bin/sh
# Automatically update changeset after each commit

# Only run if we're not in a rebase/merge
if [ -f .git/MERGE_HEAD ] || [ -d .git/rebase-merge ] || [ -d .git/rebase-apply ]; then
    exit 0
fi

# Check if workspace is available
if ! command -v workspace &> /dev/null; then
    echo "workspace not found, skipping changeset update"
    exit 0
fi

# Update changeset (will skip if no changeset exists)
workspace changeset update --log-level error 2>/dev/null || true

exit 0
```

**Example: pre-push hook**

`.git/hooks/pre-push`:
```bash
#!/bin/sh
# Validate changesets before pushing

# Check if workspace is available
if ! command -v workspace &> /dev/null; then
    exit 0
fi

# Get current branch
current_branch=$(git rev-parse --abbrev-ref HEAD)

# Skip for main/master branches
if [ "$current_branch" = "main" ] || [ "$current_branch" = "master" ]; then
    exit 0
fi

# Check if changeset exists
if ! workspace changeset check --log-level error 2>/dev/null; then
    echo "⚠️  No changeset found for branch: $current_branch"
    echo "Run: workspace changeset create"
    read -p "Push anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

exit 0
```

**Example: GitHub Actions workflow**

`.github/workflows/release.yml`:
```yaml
name: Release
on:
  push:
    branches: [main]

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
          
      - name: Install workspace
        run: |
          curl -fsSL https://install.workspace.dev | sh
          echo "$HOME/.local/bin" >> $GITHUB_PATH
          
      - name: Bump versions and create tags
        run: workspace bump --execute --git-commit --git-tag --git-push --format json > bump-result.json
        
      - name: Publish packages
        run: npm publish --workspaces
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

**Installation of hooks:**

Users can create these hooks manually or use tools like `husky` to manage them:

```json
{
  "husky": {
    "hooks": {
      "post-commit": "workspace changeset update --log-level error || true",
      "pre-push": "workspace changeset check || true"
    }
  }
}
```

### 10.6 Update Mechanism

**Future Enhancement:**
```bash
# Check for updates
workspace update check

# Install latest version
workspace update install

# Update to specific version
workspace update install --version 0.2.0
```

### 10.7 Performance Requirements for Git Hooks

Since the CLI may be called from git hooks, performance is critical:

**Performance Targets:**
- `workspace changeset check`: < 100ms
- `workspace changeset update`: < 300ms
- `workspace changeset create`: < 500ms (with prompts disabled)

**Optimization Strategies:**
- Lazy loading of configuration
- Minimal dependency tree for core operations
- Efficient file I/O operations
- No network calls for basic operations
- Fast exit for "nothing to do" scenarios

**Graceful Degradation:**
- If run outside git repo, provide clear error
- If no config file, provide helpful message
- If no changeset exists, exit cleanly
- Never block git operations

---

## 11. Success Metrics

### 11.1 Adoption Metrics

- **Downloads:**
  - Target: 1,000 downloads in first month
  - Target: 10,000 downloads in first year

- **GitHub Stars:**
  - Target: 100 stars in first 3 months
  - Target: 500 stars in first year

- **Active Users:**
  - Track via telemetry (opt-in)
  - Target: 100 weekly active projects

### 11.2 Performance Metrics

- **Startup Time:**
  - Target: < 100ms cold start
  - Measure: `time workspace --version`

- **Command Execution:**
  - `workspace changeset create`: < 500ms
  - `workspace changeset update`: < 200ms
  - `workspace bump --dry-run`: < 2s for 50 packages
  - `workspace audit`: < 5s for 50 packages

### 11.3 Quality Metrics

- **Test Coverage:**
  - Target: > 80% code coverage
  - All commands have integration tests
  - All platforms tested

- **Bug Reports:**
  - Target: < 5 critical bugs in first 3 months
  - 90% of bugs resolved within 1 week

- **Documentation:**
  - 100% of commands documented
  - Examples for all common use cases
  - Troubleshooting guide

### 11.4 User Satisfaction

- **Issue Response Time:**
  - Target: First response < 24 hours
  - Target: Resolution < 1 week for P1 bugs

- **Feature Requests:**
  - Track and prioritize
  - Implement top 3 requests per quarter

---

## 12. Future Considerations

### 12.1 Phase 2 Features (v0.2.0)

1. **Plugin System:**
   - Custom commands via plugins
   - Custom formatters
   - Custom audit rules

2. **Configuration Profiles:**
   - Multiple profiles (dev, CI, prod)
   - Profile switching
   - Profile inheritance

3. **Advanced Git Integration:**
   - Conventional commits enforcement
   - Automatic changelog from commits
   - PR description generation

4. **Workspace Enhancements:**
   - Workspace dependency graph visualization
   - Circular dependency resolution suggestions
   - Workspace-wide scripts

5. **Telemetry (Opt-in):**
   - Anonymous usage statistics
   - Performance metrics
   - Error reporting

### 12.2 Phase 3 Features (v0.3.0)

1. **Web Dashboard:**
   - Visualize project health over time
   - Dependency graph visualization
   - Release history

2. **CI