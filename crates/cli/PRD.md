# Product Requirements Document (PRD)
# Workspace Node Tools CLI

**Version:** 0.1.0 (Draft)  
**Created:** 2024  
**Status:** Draft for Iteration  
**Target Audience:** Node.js/TypeScript developers working with single packages or monorepos

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

The Workspace Node Tools CLI (`wnt`) is a command-line interface for managing Node.js projects (single packages and monorepos) with a focus on version management, changesets, dependency upgrades, and project health auditing. It provides a unified interface to the `sublime_package_tools` crate and integrates seamlessly with Git workflows, CI/CD pipelines, and developer workflows.

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
**Context:** Developer starts a new project or adds wnt to existing project

**Flow:**
1. Developer runs `wnt init` in project root
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
2. Developer runs: `wnt changeset create` (manually or via git hook)
3. CLI detects current branch and prompts:
   - "Which environments should this target?" (multi-select from config)
   - "What bump type will this be?" (patch/minor/major)
4. CLI creates changeset file in configured directory
5. Developer continues working on feature

**Alternative Flow:** 
- Developer manually runs `wnt changeset create --bump minor --env production,staging`
- Git hook skipped, changeset created immediately

**Expected Outcome:**
- Changeset file created with branch name, bump type, environments
- Ready to track commits as they occur

---

### Scenario C: Commit Tracking
**Context:** Developer commits changes on feature branch

**Flow:**
1. Developer runs: `git commit -m "feat: add new component"`
2. Developer runs: `wnt changeset update` (manually or via git hook)
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
1. Developer runs: `wnt audit`
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
- `wnt audit --format markdown --output audit-report.md --sections upgrades,dependencies`
- Non-interactive mode for automation

**Expected Outcome:**
- Comprehensive audit report with actionable insights
- Health score with breakdown by category
- Clear recommendations for improvements

---

### Scenario E: Dependency Upgrades
**Context:** Developer wants to update dependencies safely

**Flow:**
1. Developer runs: `wnt upgrade check`
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
- `wnt upgrade apply --dry-run` - Preview only
- `wnt upgrade apply --patch-only --auto-changeset` - Safe automated upgrades
- `wnt upgrade apply --packages @types/node,eslint` - Specific packages only

**Expected Outcome:**
- Dependencies safely upgraded with backup
- Optional changeset created for tracking
- Clear report of what changed

---

### Scenario F: Release Process (CI/CD)
**Context:** Feature branch merged to main, CI/CD pipeline executes release

**Flow:**
1. GitHub Action triggers on merge to main
2. Action runs: `wnt bump --execute --git-tag --git-push`
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
1. GitHub Action runs as first step: `wnt bump --dry-run --format json`
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
1. Developer runs: `wnt changeset list`
2. CLI displays all pending changesets with:
   - Branch name
   - Bump type
   - Affected packages
   - Number of commits
   - Target environments
3. Developer can:
   - View details: `wnt changeset show feature/my-branch`
   - Update: `wnt changeset update --bump major`
   - Delete: `wnt changeset delete feature/old-branch`
   - Query history: `wnt changeset history --package @myorg/core`

**Expected Outcome:**
- Full visibility into changeset state
- Easy modification capabilities
- Historical tracking

---

## 6. Feature Requirements

### 6.1 Configuration Management

#### F-001: Project Initialization
**Priority:** P0 (Must Have)  
**Description:** Initialize new or existing project with wnt configuration

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
- Detect current branch and load associated changeset
- Analyze git diff to determine affected packages
- Add commit hash to changeset
- Add newly affected packages to changeset
- Handle monorepo package detection correctly
- Skip if no changeset exists (graceful degradation)
- Support manual package specification

**Acceptance Criteria:**
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

**Acceptance Criteria:**
- ✓ Correctly bumps all affected packages
- ✓ Dependency propagation works correctly
- ✓ Dry-run mode produces accurate preview
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
- Display dependency propagation chain
- Calculate and show dependency graph changes
- Support JSON output for CI/CD
- Show which changesets will be processed
- Highlight circular dependencies if any

**Acceptance Criteria:**
- ✓ Accurately predicts all version changes
- ✓ JSON output is machine-readable
- ✓ Shows complete dependency impact

#### F-022: Snapshot Version Generation
**Priority:** P1 (Should Have)  
**Description:** Generate snapshot versions for feature branches

**Requirements:**
- Support configurable snapshot format templates
- Include branch name, commit hash, timestamp
- Handle branch name sanitization
- Generate unique snapshot identifiers
- Support custom snapshot variables

**Acceptance Criteria:**
- ✓ Generates valid semver-compatible versions
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

---

### 6.8 Output and Logging

#### F-070: Logging Levels
**Priority:** P0 (Must Have)  
**Description:** Configurable logging output

**Requirements:**
- Support log levels: silent, error, warn, info, debug, trace
- Global flag for log level: `--log-level`
- Default to 'info' level
- Respect NO_COLOR environment variable
- Write logs to stderr, output to stdout

**Acceptance Criteria:**
- ✓ All log levels work correctly
- ✓ Appropriate default level
- ✓ Output separation works

#### F-071: JSON Output Mode
**Priority:** P0 (Must Have)  
**Description:** Machine-readable JSON output

**Requirements:**
- Global flag: `--format json`
- Consistent JSON structure across commands
- Include success/error status
- Include all relevant data
- Valid JSON always (no mixed output)

**Acceptance Criteria:**
- ✓ All commands support JSON output
- ✓ JSON is always valid
- ✓ Structure is documented

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
wnt [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGS]
```

### 7.2 Global Options

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--root <PATH>` | `-r` | Project root directory | Current directory |
| `--log-level <LEVEL>` | `-l` | Log level (silent\|error\|warn\|info\|debug\|trace) | info |
| `--format <FORMAT>` | `-f` | Output format (text\|json\|json-compact) | text |
| `--no-color` | | Disable colored output | false |
| `--config <PATH>` | `-c` | Path to config file | Auto-detect |
| `--help` | `-h` | Show help | |
| `--version` | `-V` | Show version | |

### 7.3 Commands

---

#### `wnt init`

Initialize project configuration.

**Usage:**
```bash
wnt init [OPTIONS]
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

**Examples:**
```bash
# Interactive mode
wnt init

# Non-interactive with all options
wnt init --non-interactive --strategy unified --format yaml --environments "dev,staging,prod" --default-env "prod"

# Minimal non-interactive
wnt init --non-interactive --format json
```

**Output:**
- Success: Configuration file path and summary
- Error: Validation errors with suggestions

---

#### `wnt config`

Manage configuration.

**Subcommands:**
- `wnt config show` - Display current configuration
- `wnt config validate` - Validate configuration file
- `wnt config get <KEY>` - Get specific config value
- `wnt config set <KEY> <VALUE>` - Set config value (future)

**Usage:**
```bash
wnt config show [OPTIONS]
wnt config validate [OPTIONS]
```

**Options:**
| Flag | Description |
|------|-------------|
| `--format <FORMAT>` | Output format (text\|json) |

**Examples:**
```bash
wnt config show
wnt config show --format json
wnt config validate
```

---

#### `wnt changeset`

Manage changesets.

**Subcommands:**
- `wnt changeset create` - Create new changeset
- `wnt changeset update` - Update current changeset
- `wnt changeset list` - List all changesets
- `wnt changeset show <BRANCH>` - Show changeset details
- `wnt changeset delete <BRANCH>` - Delete changeset
- `wnt changeset history` - Query archived changesets
- `wnt changeset check` - Check if changeset exists (for hooks)

**Usage:**
```bash
wnt changeset create [OPTIONS]
wnt changeset update [OPTIONS]
wnt changeset list [OPTIONS]
wnt changeset show <BRANCH> [OPTIONS]
wnt changeset delete <BRANCH> [OPTIONS]
wnt changeset history [OPTIONS]
wnt changeset check [OPTIONS]
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

**Options (update):**
| Flag | Description |
|------|-------------|
| `--branch <NAME>` | Branch name (default: current) |
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

**Examples:**
```bash
# Create changeset interactively
wnt changeset create

# Create with all options
wnt changeset create --bump minor --env "staging,prod" --message "Add new feature"

# Update current changeset
wnt changeset update

# List all changesets
wnt changeset list

# List with filtering
wnt changeset list --filter-bump major --sort date

# Show specific changeset
wnt changeset show feature/new-component

# Delete changeset with confirmation
wnt changeset delete old-feature

# Force delete without confirmation
wnt changeset delete old-feature --force

# Query history
wnt changeset history --package @myorg/core --since 2024-01-01

# Check if changeset exists (for Git hooks)
wnt changeset check
```

---

#### `wnt bump`

Bump package versions based on changesets.

**Usage:**
```bash
wnt bump [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--dry-run` | Preview changes without applying | false |
| `--execute` | Apply version changes | false (requires explicit) |
| `--snapshot` | Generate snapshot versions | false |
| `--snapshot-format <FORMAT>` | Snapshot format template | From config |
| `--prerelease <TAG>` | Pre-release tag (alpha\|beta\|rc) | None |
| `--packages <LIST>` | Only bump specific packages | All affected |
| `--git-tag` | Create git tags for releases | false |
| `--git-push` | Push git tags to remote (requires --git-tag) | false |
| `--git-commit` | Commit version changes | false |
| `--no-changelog` | Don't update changelogs | false |
| `--no-archive` | Don't archive changesets | false |
| `--force` | Skip confirmations | false |

**Examples:**
```bash
# Preview version changes
wnt bump --dry-run

# Preview with JSON output (for CI/CD)
wnt bump --dry-run --format json

# Execute version bump (only update files)
wnt bump --execute

# Execute with git operations
wnt bump --execute --git-tag --git-push

# Execute with git commit and tags
wnt bump --execute --git-commit --git-tag

# Generate snapshot versions for feature branch
wnt bump --snapshot --execute

# Create pre-release versions
wnt bump --prerelease beta --execute

# Bump specific packages only
wnt bump --packages "@myorg/core,@myorg/utils" --execute --dry-run

# Full CI/CD workflow
wnt bump --execute --git-commit --git-tag --git-push --force
```

**Output (dry-run):**
```
Version Bump Preview
━━━━━━━━━━━━━━━━━━━━

Packages to bump:
  @myorg/core: 1.2.3 → 1.3.0 (minor)
  @myorg/utils: 2.0.1 → 2.1.0 (minor, propagated)
  @myorg/cli: 0.5.0 → 0.5.1 (patch, propagated)

Changesets to process:
  ✓ feature/new-api (minor, 5 commits)
  ✓ feature/fix-bug (patch, 2 commits)

Git tags to create:
  @myorg/core@1.3.0
  @myorg/utils@2.1.0
  @myorg/cli@0.5.1

Run with --execute to apply changes.
```

**JSON Output (--format json --dry-run):**
```json
{
  "success": true,
  "packages": [
    {
      "name": "@myorg/core",
      "path": "packages/core",
      "currentVersion": "1.2.3",
      "nextVersion": "1.3.0",
      "bump": "minor",
      "reason": "direct"
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

#### `wnt upgrade`

Manage dependency upgrades.

**Subcommands:**
- `wnt upgrade check` - Detect available upgrades
- `wnt upgrade apply` - Apply upgrades
- `wnt upgrade backups` - Manage backups

**Usage:**
```bash
wnt upgrade check [OPTIONS]
wnt upgrade apply [OPTIONS]
wnt upgrade backups list [OPTIONS]
wnt upgrade backups restore <ID> [OPTIONS]
wnt upgrade backups clean [OPTIONS]
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

**Examples:**
```bash
# Check for all upgrades
wnt upgrade check

# Check patch upgrades only
wnt upgrade check --no-major --no-minor

# Check specific packages
wnt upgrade check --packages "typescript,eslint"

# Apply all patch upgrades with auto-changeset
wnt upgrade apply --patch-only --auto-changeset

# Apply specific upgrades
wnt upgrade apply --packages "@types/node,typescript"

# Dry-run to see what would be upgraded
wnt upgrade apply --dry-run

# List backups
wnt upgrade backups list

# Restore backup
wnt upgrade backups restore backup_20240115_103045

# Clean old backups (keep last 5)
wnt upgrade backups clean --keep 5
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

---

#### `wnt audit`

Run project health audit.

**Usage:**
```bash
wnt audit [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--sections <LIST>` | Sections to audit (all\|upgrades\|dependencies\|version-consistency\|breaking-changes) | all |
| `--format <FORMAT>` | Output format (text\|markdown\|json\|json-compact) | text |
| `--output <PATH>` | Write to file | stdout |
| `--min-severity <LEVEL>` | Minimum severity (critical\|high\|medium\|low\|info) | info |
| `--verbosity <LEVEL>` | Detail level (minimal\|normal\|detailed) | normal |
| `--no-health-score` | Skip health score calculation | false |

**Examples:**
```bash
# Full audit
wnt audit

# Specific sections
wnt audit --sections upgrades,dependencies

# Generate markdown report
wnt audit --format markdown --output audit-report.md

# JSON for CI/CD
wnt audit --format json-compact

# Only critical and high severity issues
wnt audit --min-severity high

# Detailed output
wnt audit --verbosity detailed
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
4. Run 'wnt upgrade check' for detailed upgrade info
```

---

#### `wnt changes`

Analyze changes in repository.

**Usage:**
```bash
wnt changes [OPTIONS]
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

**Examples:**
```bash
# Analyze working directory changes
wnt changes

# Changes since last tag
wnt changes --since $(git describe --tags --abbrev=0)

# Changes between commits
wnt changes --since abc123 --until def456

# Changes in current branch vs main
wnt changes --branch main

# Only staged changes
wnt changes --staged
```

**Output:**
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

---

#### `wnt version`

Display version information.

**Usage:**
```bash
wnt version [OPTIONS]
wnt --version
wnt -V
```

**Options:**
| Flag | Description |
|------|-------------|
| `--verbose` | Show detailed version info |

**Examples:**
```bash
# Simple version
wnt version
wnt --version

# Detailed version info
wnt version --verbose
```

**Output:**
```
wnt 0.1.0

# Verbose:
wnt 0.1.0
  sublime_package_tools: 0.1.0
  sublime_standard_tools: 0.1.0
  sublime_git_tools: 0.1.0
  
Platform: macOS (aarch64)
Rustc: 1.75.0
```

---

#### `wnt help`

Show help information.

**Usage:**
```bash
wnt help [COMMAND]
wnt <COMMAND> --help
wnt --help
```

**Examples:**
```bash
# General help
wnt help
wnt --help

# Command-specific help
wnt help changeset
wnt changeset create --help
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
workspace-node-tools v0.1.0
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
wnt-changeset-create
Create a new changeset for the current branch

USAGE:
    wnt changeset create [OPTIONS]

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
    wnt changeset create
    
    # Non-interactive with options
    wnt changeset create --bump minor --env production,staging
    
    # With message
    wnt changeset create --bump patch --message "Fix critical bug"
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
brew install org/tap/wnt
```

#### Method 3: Cargo
```bash
cargo install wnt
```

#### Method 4: Pre-built Binaries
Download from GitHub Releases for your platform:
- `wnt-macos-x86_64.tar.gz`
- `wnt-macos-aarch64.tar.gz`
- `wnt-linux-x86_64.tar.gz`
- `wnt-linux-aarch64.tar.gz`
- `wnt-windows-x86_64.zip`

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
curl -fsSL https://install.wnt.dev | sh

# Install specific version
curl -fsSL https://install.wnt.dev | sh -s -- --version v0.1.0

# Custom install location
curl -fsSL https://install.wnt.dev | sh -s -- --install-dir ~/.local/bin

# Verbose output
curl -fsSL https://install.wnt.dev | sh -s -- --verbose
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

# Check if wnt is available
if ! command -v wnt &> /dev/null; then
    echo "wnt not found, skipping changeset update"
    exit 0
fi

# Update changeset (will skip if no changeset exists)
wnt changeset update --log-level error 2>/dev/null || true

exit 0
```

**Example: pre-push hook**

`.git/hooks/pre-push`:
```bash
#!/bin/sh
# Validate changesets before pushing

# Check if wnt is available
if ! command -v wnt &> /dev/null; then
    exit 0
fi

# Get current branch
current_branch=$(git rev-parse --abbrev-ref HEAD)

# Skip for main/master branches
if [ "$current_branch" = "main" ] || [ "$current_branch" = "master" ]; then
    exit 0
fi

# Check if changeset exists
if ! wnt changeset check --log-level error 2>/dev/null; then
    echo "⚠️  No changeset found for branch: $current_branch"
    echo "Run: wnt changeset create"
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
          
      - name: Install wnt
        run: |
          curl -fsSL https://install.wnt.dev | sh
          echo "$HOME/.local/bin" >> $GITHUB_PATH
          
      - name: Bump versions and create tags
        run: wnt bump --execute --git-commit --git-tag --git-push --format json > bump-result.json
        
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
      "post-commit": "wnt changeset update --log-level error || true",
      "pre-push": "wnt changeset check || true"
    }
  }
}
```

### 10.6 Update Mechanism

**Future Enhancement:**
```bash
# Check for updates
wnt update check

# Install latest version
wnt update install

# Update to specific version
wnt update install --version 0.2.0
```

### 10.7 Performance Requirements for Git Hooks

Since the CLI may be called from git hooks, performance is critical:

**Performance Targets:**
- `wnt changeset check`: < 100ms
- `wnt changeset update`: < 300ms
- `wnt changeset create`: < 500ms (with prompts disabled)

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
  - Measure: `time wnt --version`

- **Command Execution:**
  - `wnt changeset create`: < 500ms
  - `wnt changeset update`: < 200ms
  - `wnt bump --dry-run`: < 2s for 50 packages
  - `wnt audit`: < 5s for 50 packages

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