# Workspace Node Tools CLI - Implementation Plan

**Status**: 📋 Ready for Implementation  
**Version**: 1.0  
**Based on**: PRD.md v1.0  
**Last Updated**: 2024-01-15

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Priority Analysis](#priority-analysis)
3. [Dependency Graph](#dependency-graph)
4. [Implementation Phases](#implementation-phases)
5. [Module Structure](#module-structure)
6. [Quality Standards](#quality-standards)
7. [Testing Strategy](#testing-strategy)
8. [Documentation Requirements](#documentation-requirements)
9. [Milestones & Timeline](#milestones--timeline)
10. [Risk Assessment](#risk-assessment)
11. [Development Workflow](#development-workflow)
12. [CI/CD Pipeline](#cicd-pipeline)
13. [Success Metrics](#success-metrics)

---

## Executive Summary

### Project Overview

`wnt` (Workspace Node Tools) is a comprehensive CLI tool for managing Node.js monorepos with changeset-based versioning. The implementation is divided into **5 major phases** across **10 core modules**, following strict quality standards (100% test coverage, 100% clippy compliance, 100% documentation).

### Key Success Criteria

- ✅ All modules pass clippy without warnings
- ✅ 100% test coverage (unit + integration + E2E)
- ✅ 100% API documentation with examples
- ✅ Zero `unwrap()`, `expect()`, `todo!()`, `panic!()`, `unimplemented!()`
- ✅ All errors implement `AsRef<str>`
- ✅ Internal visibility uses `pub(crate)` consistently
- ✅ Command execution < 100ms for most operations
- ✅ Git hooks execute < 500ms
- ✅ Cross-platform compatibility (macOS, Linux, Windows)

### Estimated Timeline

- **Phase 1**: 2-3 weeks (Foundation & Core Commands)
- **Phase 2**: 3-4 weeks (Changeset Management)
- **Phase 3**: 2-3 weeks (Version Management & Upgrades)
- **Phase 4**: 2-3 weeks (Audit & Advanced Features)
- **Phase 5**: 1-2 weeks (Distribution & Polish)
- **Total**: 10-15 weeks

---

## Priority Analysis

### Critical Path (Must Have - Phase 1 & 2)

1. **CLI Framework** - Argument parsing and command dispatch
2. **Error Handling** - User-friendly error messages
3. **Configuration Commands** - `init`, `config`
4. **Output Formatting** - JSON, Table, Progress indicators
5. **Changeset Commands** - Core workflow (`add`, `list`, `show`)

### High Priority (Phase 2 & 3)

6. **Version Commands** - `bump` with various modes
7. **Changes Analysis** - `changes` command
8. **Upgrade Commands** - Dependency management
9. **Git Integration** - Branch detection, commit tracking

### Medium Priority (Phase 3 & 4)

10. **Audit Commands** - Health checks and reporting
11. **Interactive Prompts** - Enhanced UX
12. **Color Output** - Terminal theming
13. **Shell Completions** - Bash, Zsh, Fish

### Low Priority (Phase 4 & 5)

14. **Installation Scripts** - Curl-based installer
15. **Self-Update** - Automatic version updates
16. **CI/CD Integration** - GitHub Actions examples

### Priority Rationale

```
CLI Framework + Error Handling (P0)
    ↓
Config Commands (init, config) (P1)
    ↓
Output Formatting (P1)
    ↓
Changeset Commands (add, list, show, update) (P1-P2)
    ↓
Version Commands (bump, preview) (P2)
    ↓
Changes + Upgrades (P2-P3)
    ↓
Audit + Interactive Features (P3)
    ↓
Distribution + Self-Update (P4)
```

**Why this order:**
- **CLI Framework first**: Foundation for all commands
- **Error handling early**: Critical for user experience
- **Config commands**: Users need to initialize projects
- **Output formatting**: Required by all commands
- **Changesets**: Core workflow that unlocks versioning
- **Version management**: Completes the release workflow
- **Upgrades & Audit**: Enhancement features
- **Distribution**: Final polish for public release

---

## Dependency Graph

### Module Dependencies

```mermaid
graph TD
    CLI[CLI Framework]
    Error[Error Handling]
    Output[Output Formatting]
    Config[Config Commands]
    
    Changeset[Changeset Commands]
    Version[Version Commands]
    Changes[Changes Commands]
    Upgrade[Upgrade Commands]
    Audit[Audit Commands]
    
    Utils[Utilities]
    Interactive[Interactive Prompts]
    Git[Git Integration]
    
    CLI --> Error
    CLI --> Output
    CLI --> Utils
    
    Error --> Config
    Error --> Changeset
    Error --> Version
    Error --> Changes
    Error --> Upgrade
    Error --> Audit
    
    Output --> Config
    Output --> Changeset
    Output --> Version
    Output --> Changes
    Output --> Upgrade
    Output --> Audit
    
    Utils --> Config
    Utils --> Changeset
    Utils --> Version
    
    Interactive --> Changeset
    Interactive --> Upgrade
    
    Git --> Changeset
    Git --> Changes
    
    Config --> Changeset
    Changeset --> Version
    Changes --> Version
    Changes --> Audit
    Version --> Audit
    Upgrade --> Audit
```

### External Dependencies

```
wnt (CLI)
    ↓
├─ sublime-package-tools (core logic)
├─ sublime-standard-tools (filesystem, config)
├─ sublime-git-tools (git operations)
├─ clap (CLI framework)
├─ dialoguer (interactive prompts)
├─ indicatif (progress bars)
├─ comfy-table (table rendering)
├─ crossterm (terminal control)
├─ console (styling)
└─ tokio, serde, anyhow, tracing (standard)
```

---

## Implementation Phases

## Phase 1: Foundation & Core Commands (Weeks 1-3)

### Objective
Establish CLI framework, error handling, and basic configuration commands.

### Deliverables

#### 1.1 Project Setup & Structure

**Tasks:**
- [ ] Initialize CLI crate structure
- [ ] Configure `Cargo.toml` with all dependencies
- [ ] Setup `main.rs` with basic CLI structure
- [ ] Create module structure following patterns
- [ ] Configure clippy rules and lint settings

**Files to create:**
```
crates/cli/
├── Cargo.toml
├── build.rs                     # Build-time shell completion generation
├── src/
│   ├── main.rs                  # Entry point with async runtime
│   ├── cli/
│   │   ├── mod.rs              # CLI definition and parsing
│   │   ├── commands.rs         # Command enum
│   │   ├── args.rs             # Global arguments
│   │   └── parser.rs           # Argument parsing logic
│   ├── error/
│   │   └── mod.rs              # CLI error types
│   ├── output/
│   │   └── mod.rs              # Output formatting (export only)
│   └── utils/
│       └── mod.rs              # Shared utilities
```

**Quality Gates:**
- ✅ Project compiles with all dependencies
- ✅ Clippy passes without warnings
- ✅ Basic CLI structure in place

#### 1.2 Error Handling Module

**Tasks:**
- [ ] Define `CliError` enum wrapping library errors
- [ ] Implement user-friendly error messages
- [ ] Create error context builders
- [ ] Implement `AsRef<str>` for all errors
- [ ] Add exit code mapping
- [ ] Create error formatting utilities

**Files:**
```
src/error/
├── mod.rs                       # CliError enum and conversions
├── display.rs                   # User-friendly error display
├── exit_codes.rs                # Exit code constants
└── tests.rs                     # Error tests
```

**Error Types:**
```rust
pub enum CliError {
    Configuration(String),
    Validation(String),
    Execution(String),
    Git(String),
    Package(String),
    Io(String),
    Network(String),
    User(String),                // User-caused errors (e.g., invalid input)
}

impl CliError {
    pub fn exit_code(&self) -> i32 { ... }
    pub fn user_message(&self) -> String { ... }
}
```

**Quality Gates:**
- ✅ All error variants have clear messages
- ✅ Exit codes follow sysexits standards
- ✅ Error context includes helpful suggestions
- ✅ 100% test coverage

#### 1.3 Output Formatting Module

**Tasks:**
- [ ] Create `OutputFormat` enum (Human, Json, Quiet)
- [ ] Implement table rendering with `comfy-table`
- [ ] Create JSON serialization utilities
- [ ] Implement progress bars with `indicatif`
- [ ] Create color/style helpers with `console`
- [ ] Add logging integration with `tracing`

**Files:**
```
src/output/
├── mod.rs                       # OutputFormat and main interface
├── table.rs                     # Table rendering utilities
├── json.rs                      # JSON output formatting
├── progress.rs                  # Progress indicators
├── style.rs                     # Color and styling
├── logger.rs                    # Logging setup
└── tests.rs                     # Output tests
```

**Key Components:**
```rust
pub enum OutputFormat {
    Human,
    Json,
    Quiet,
}

pub struct TableBuilder { ... }
pub struct ProgressBar { ... }
pub struct Styled { ... }

impl Output {
    pub fn success(&self, message: &str) { ... }
    pub fn error(&self, message: &str) { ... }
    pub fn warning(&self, message: &str) { ... }
    pub fn info(&self, message: &str) { ... }
    pub fn table(&self, data: TableData) { ... }
    pub fn json<T: Serialize>(&self, data: &T) { ... }
}
```

**Quality Gates:**
- ✅ All output modes work correctly
- ✅ Tables render properly in all terminal sizes
- ✅ JSON output is valid and complete
- ✅ Progress bars update smoothly
- ✅ Colors respect NO_COLOR environment variable

#### 1.4 CLI Framework

**Tasks:**
- [ ] Define main `Cli` struct with clap
- [ ] Create `Commands` enum for all subcommands
- [ ] Implement global arguments (verbose, format, color, etc.)
- [ ] Create command dispatcher
- [ ] Add version and help commands
- [ ] Implement shell completion generation

**Files:**
```
src/cli/
├── mod.rs                       # Cli struct and main parsing
├── commands.rs                  # Commands enum
├── args.rs                      # Global arguments
├── dispatch.rs                  # Command dispatcher
├── completions.rs               # Shell completion generation
└── tests.rs                     # CLI parsing tests
```

**CLI Structure:**
```rust
#[derive(Parser)]
#[command(name = "wnt")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Workspace Node Tools - Changeset-based version management")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    #[arg(global = true, short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    
    #[arg(global = true, long, value_enum, default_value = "human")]
    pub format: OutputFormat,
    
    #[arg(global = true, long)]
    pub no_color: bool,
    
    #[arg(global = true, long)]
    pub cwd: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    Init(InitArgs),
    Config(ConfigArgs),
    Changeset(ChangesetArgs),
    Bump(BumpArgs),
    Upgrade(UpgradeArgs),
    Audit(AuditArgs),
    Changes(ChangesArgs),
    Version,
    Help { command: Option<String> },
}
```

**Quality Gates:**
- ✅ All commands parse correctly
- ✅ Help text is comprehensive
- ✅ Shell completions generate for bash, zsh, fish
- ✅ Global arguments work across all commands

#### 1.5 Configuration Commands (`init`, `config`)

**Tasks:**
- [ ] Implement `wnt init` command
- [ ] Create interactive prompts for initialization
- [ ] Implement `wnt config show` command
- [ ] Implement `wnt config validate` command
- [ ] Add configuration templates
- [ ] Create validation feedback

**Files:**
```
src/commands/
├── mod.rs                       # Command exports
├── init.rs                      # Init command
└── config.rs                    # Config command
```

**Command Implementations:**
```rust
pub async fn execute_init(args: InitArgs, output: &Output) -> Result<()> {
    // 1. Check if already initialized
    // 2. Prompt for configuration options
    // 3. Detect workspace structure
    // 4. Create .changesets directory and .wnt-backups directory
    // 5. Generate repo.config.toml
    // 6. Initialize git integration
    // 7. Create example changeset
    // 8. Display success message
}

pub async fn execute_config(args: ConfigArgs, output: &Output) -> Result<()> {
    match args.subcommand {
        ConfigSubcommand::Show => { /* Display current config */ },
        ConfigSubcommand::Validate => { /* Validate and report */ },
    }
}
```

**Quality Gates:**
- ✅ Init command creates valid configuration
- ✅ Interactive prompts have sensible defaults
- ✅ Config validation provides actionable feedback
- ✅ E2E tests verify entire init flow

### Phase 1 Exit Criteria
- ✅ CLI framework compiles and runs
- ✅ Init and config commands work end-to-end
- ✅ Error handling provides helpful messages
- ✅ Output formatting works in all modes
- ✅ Clippy passes without warnings
- ✅ 100% test coverage on Phase 1 modules

---

## Phase 2: Changeset Management (Weeks 4-7)

### Objective
Implement comprehensive changeset commands for the core workflow.

### Deliverables

#### 2.1 Changeset Add Command

**Tasks:**
- [ ] Implement `wnt changeset add` command
- [ ] Create interactive prompt flow
- [ ] Support multiple packages selection
- [ ] Implement version bump selection
- [ ] Add commit message integration
- [ ] Create changeset file generation

**Files:**
```
src/commands/
├── changeset/
│   ├── mod.rs                   # Changeset subcommand router
│   ├── add.rs                   # Add command
│   ├── list.rs                  # List command
│   ├── show.rs                  # Show command
│   ├── update.rs                # Update command
│   ├── edit.rs                  # Edit command
│   ├── remove.rs                # Remove command
│   └── history.rs               # History command
```

**Interactive Flow:**
```rust
pub async fn execute_changeset_add(
    args: ChangesetAddArgs,
    output: &Output,
) -> Result<()> {
    // 1. Load workspace configuration
    // 2. Detect affected packages from git changes
    // 3. Prompt for packages (if not specified)
    // 4. Prompt for bump type (patch, minor, major)
    // 5. Prompt for summary
    // 6. Optional: detect related commits
    // 7. Generate changeset ID
    // 8. Create changeset file
    // 9. Display success and next steps
}
```

**Quality Gates:**
- ✅ Interactive prompts are intuitive
- ✅ Non-interactive mode works with flags
- ✅ Changeset files are valid YAML
- ✅ Git integration detects affected packages

#### 2.2 Changeset List Command

**Tasks:**
- [ ] Implement `wnt changeset list` command
- [ ] Support filtering by branch
- [ ] Support filtering by package
- [ ] Support sorting options
- [ ] Create table output format
- [ ] Create JSON output format

**Implementation:**
```rust
pub async fn execute_changeset_list(
    args: ChangesetListArgs,
    output: &Output,
) -> Result<()> {
    // 1. Load workspace configuration
    // 2. Scan .changesets directory
    // 3. Apply filters (branch, package, etc.)
    // 4. Sort by specified criteria
    // 5. Format as table or JSON
    // 6. Display results
}
```

**Quality Gates:**
- ✅ All filters work correctly
- ✅ Table output is readable
- ✅ JSON output is complete
- ✅ Performance < 100ms for 1000 changesets

#### 2.3 Changeset Show Command

**Tasks:**
- [ ] Implement `wnt changeset show` command
- [ ] Display full changeset details
- [ ] Show related commits
- [ ] Display affected packages
- [ ] Format for readability

**Quality Gates:**
- ✅ All changeset fields displayed
- ✅ Output is well-formatted
- ✅ Works with both IDs and file paths

#### 2.4 Changeset Update Command

**Tasks:**
- [ ] Implement `wnt changeset update` command
- [ ] Allow adding commits
- [ ] Allow modifying bump types
- [ ] Allow updating summary
- [ ] Track modification history

**Quality Gates:**
- ✅ Updates preserve changeset integrity
- ✅ Modification history is tracked
- ✅ Validation prevents invalid states

#### 2.5 Changeset Edit Command

**Tasks:**
- [ ] Implement `wnt changeset edit` command
- [ ] Open changeset in $EDITOR
- [ ] Validate after edit
- [ ] Handle concurrent modifications

**Quality Gates:**
- ✅ Editor detection works on all platforms
- ✅ Validation catches errors
- ✅ Graceful handling of invalid edits

#### 2.6 Changeset Remove Command

**Tasks:**
- [ ] Implement `wnt changeset remove` command
- [ ] Add confirmation prompts
- [ ] Support multiple removals
- [ ] Create archive before removal

**Quality Gates:**
- ✅ Confirmation prevents accidents
- ✅ Archive allows recovery
- ✅ Batch operations work correctly

#### 2.7 Changeset History Command

**Tasks:**
- [ ] Implement `wnt changeset history` command
- [ ] Show changeset timeline
- [ ] Display modifications
- [ ] Show related releases

**Quality Gates:**
- ✅ Timeline is accurate
- ✅ All modifications tracked
- ✅ Performance acceptable for large histories

### Phase 2 Exit Criteria
- ✅ All changeset commands work end-to-end
- ✅ Interactive prompts are polished
- ✅ Git integration is reliable
- ✅ Performance meets requirements
- ✅ Clippy passes without warnings
- ✅ 100% test coverage on changeset commands

---

## Phase 3: Version Management & Upgrades (Weeks 8-11)

### Objective
Implement version bumping, dependency upgrades, and release workflows.

### Deliverables

#### 3.1 Bump Command

**Tasks:**
- [ ] Implement `wnt bump` command
- [ ] Add preview mode (--dry-run)
- [ ] Implement execution mode (--execute)
- [ ] Add git integration (--git-commit, --git-tag, --git-push)
- [ ] Support snapshot versions
- [ ] Implement dependency version propagation
- [ ] Create version bump reports

**Files:**
```
src/commands/
├── bump/
│   ├── mod.rs                   # Bump command router
│   ├── execute.rs               # Main bump execution
│   ├── preview.rs               # Preview mode
│   ├── git_integration.rs       # Git operations
│   └── report.rs                # Bump report generation
```

**Implementation:**
```rust
pub async fn execute_bump(args: BumpArgs, output: &Output) -> Result<()> {
    // 1. Load configuration and changesets
    // 2. Calculate version bumps
    // 3. Resolve dependency versions
    // 4. Validate version consistency
    // 5. Preview mode: display changes
    // 6. Execute mode:
    //    a. Update package.json files
    //    b. Update changelog files
    //    c. Archive changesets
    //    d. Git commit (if --git-commit)
    //    e. Git tag (if --git-tag)
    //    f. Git push (if --git-push)
    // 7. Display success report
}
```

**Quality Gates:**
- ✅ Preview mode shows all changes accurately
- ✅ Execute mode updates all files correctly
- ✅ Git operations are atomic
- ✅ Rollback works on failures
- ✅ Performance < 1s for 100 packages

#### 3.2 Changes Command

**Tasks:**
- [ ] Implement `wnt changes` command
- [ ] Support working directory analysis
- [ ] Support commit range analysis
- [ ] Detect affected packages
- [ ] Show dependency impact

**Files:**
```
src/commands/
└── changes.rs                   # Changes analysis command
```

**Implementation:**
```rust
pub async fn execute_changes(
    args: ChangesArgs,
    output: &Output,
) -> Result<()> {
    // 1. Load workspace configuration
    // 2. Analyze changes (working dir or commit range)
    // 3. Detect affected packages
    // 4. Calculate dependency impact
    // 5. Format and display results
}
```

**Quality Gates:**
- ✅ Accurately detects affected packages
- ✅ Dependency impact is correct
- ✅ Works with large changesets
- ✅ Performance < 500ms

#### 3.3 Upgrade Commands

**Tasks:**
- [ ] Implement `wnt upgrade check` command
- [ ] Implement `wnt upgrade apply` command
- [ ] Implement `wnt upgrade rollback` command
- [ ] Add filtering by dependency type
- [ ] Support ignore patterns
- [ ] Create upgrade reports

**Files:**
```
src/commands/
├── upgrade/
│   ├── mod.rs                   # Upgrade subcommand router
│   ├── check.rs                 # Check for upgrades
│   ├── apply.rs                 # Apply upgrades
│   └── rollback.rs              # Rollback last upgrade
```

**Implementation:**
```rust
pub async fn execute_upgrade_check(
    args: UpgradeCheckArgs,
    output: &Output,
) -> Result<()> {
    // 1. Load configuration
    // 2. Detect available upgrades
    // 3. Filter by criteria
    // 4. Categorize by type
    // 5. Display upgrade report
}

pub async fn execute_upgrade_apply(
    args: UpgradeApplyArgs,
    output: &Output,
) -> Result<()> {
    // 1. Check for upgrades
    // 2. Create backup
    // 3. Apply upgrades
    // 4. Optionally create changeset
    // 5. Display results
}
```

**Quality Gates:**
- ✅ Upgrade detection is accurate
- ✅ Apply safely updates all packages
- ✅ Rollback restores previous state
- ✅ Backup system is reliable

### Phase 3 Exit Criteria
- ✅ Bump command completes full release workflow
- ✅ Changes command accurately detects impact
- ✅ Upgrade commands work reliably
- ✅ Git integration is solid
- ✅ Performance meets requirements
- ✅ Clippy passes without warnings
- ✅ 100% test coverage

---

## Phase 4: Audit & Advanced Features (Weeks 12-14)

### Objective
Implement audit system, health checks, and advanced interactive features.

### Deliverables

#### 4.1 Audit Command

**Tasks:**
- [ ] Implement `wnt audit` command
- [ ] Add comprehensive audit checks
- [ ] Add upgrade audit
- [ ] Add dependency audit
- [ ] Add version consistency audit
- [ ] Add breaking changes audit
- [ ] Calculate health scores
- [ ] Create detailed reports

**Files:**
```
src/commands/
├── audit/
│   ├── mod.rs                   # Audit command router
│   ├── comprehensive.rs         # Full audit
│   ├── upgrades.rs              # Upgrade audit
│   ├── dependencies.rs          # Dependency audit
│   ├── versions.rs              # Version consistency audit
│   ├── breaking.rs              # Breaking changes audit
│   └── report.rs                # Report generation
```

**Implementation:**
```rust
pub async fn execute_audit(args: AuditArgs, output: &Output) -> Result<()> {
    // 1. Load configuration
    // 2. Run selected audit checks
    // 3. Calculate health scores
    // 4. Generate report
    // 5. Display results with recommendations
}
```

**Quality Gates:**
- ✅ All audit checks are comprehensive
- ✅ Health scores are meaningful
- ✅ Reports provide actionable insights
- ✅ Performance < 2s for 100 packages

#### 4.2 Interactive Enhancements

**Tasks:**
- [ ] Enhance prompts with better UX
- [ ] Add multi-select for packages
- [ ] Add fuzzy search for packages
- [ ] Implement confirmation dialogs
- [ ] Add progress indicators for long operations

**Files:**
```
src/interactive/
├── mod.rs                       # Interactive utilities
├── prompts.rs                   # Custom prompts
├── select.rs                    # Multi-select helpers
└── confirm.rs                   # Confirmation dialogs
```

**Quality Gates:**
- ✅ Prompts are intuitive and fast
- ✅ Multi-select works smoothly
- ✅ Fuzzy search is responsive
- ✅ Works in all terminal types

#### 4.3 Advanced Output

**Tasks:**
- [ ] Implement diff visualization
- [ ] Add syntax highlighting for changesets
- [ ] Create summary reports
- [ ] Add export options (HTML, Markdown)

**Files:**
```
src/output/
├── diff.rs                      # Diff visualization
├── highlight.rs                 # Syntax highlighting
└── export.rs                    # Export formats
```

**Quality Gates:**
- ✅ Diffs are clear and readable
- ✅ Highlighting improves comprehension
- ✅ Exports are well-formatted

### Phase 4 Exit Criteria
- ✅ Audit system provides comprehensive insights
- ✅ Interactive features enhance UX
- ✅ Output is polished and professional
- ✅ Clippy passes without warnings
- ✅ 100% test coverage

---

## Phase 5: Distribution & Polish (Weeks 15-16)

### Objective
Finalize distribution, installation, and production readiness.

### Deliverables

#### 5.1 Installation Script

**Tasks:**
- [ ] Create curl-based installation script
- [ ] Support multiple platforms (macOS, Linux, Windows)
- [ ] Implement version detection
- [ ] Add checksum verification
- [ ] Create uninstall script

**Files:**
```
install.sh                       # Installation script
scripts/
├── install-dev.sh              # Development installation
└── uninstall.sh                # Uninstall script
```

**Implementation:**
```bash
#!/bin/sh
# Install script for wnt
# Usage: curl -fsSL https://wnt.dev/install.sh | sh

set -e

# Detect platform
# Download appropriate binary
# Verify checksum
# Install to /usr/local/bin or ~/.local/bin
# Setup shell completions
# Display success message
```

**Quality Gates:**
- ✅ Works on all supported platforms
- ✅ Handles errors gracefully
- ✅ Verifies integrity
- ✅ Provides clear feedback

#### 5.2 Build Configuration

**Tasks:**
- [ ] Optimize release builds
- [ ] Configure cross-compilation
- [ ] Setup binary stripping
- [ ] Create static binaries where possible

**Cargo.toml:**
```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

**Quality Gates:**
- ✅ Binary size < 10MB
- ✅ Startup time < 50ms
- ✅ Works on target platforms

#### 5.3 CI/CD Pipeline

**Tasks:**
- [ ] Create GitHub Actions workflows
- [ ] Setup automated testing
- [ ] Configure release automation
- [ ] Setup binary distribution
- [ ] Create Homebrew formula

**Files:**
```
.github/
└── workflows/
    ├── ci.yml                   # Continuous integration
    ├── release.yml              # Release automation
    └── install-test.yml         # Installation testing
```

**Quality Gates:**
- ✅ CI runs on all platforms
- ✅ Releases are automated
- ✅ Installation works end-to-end

#### 5.4 Documentation

**Tasks:**
- [ ] Create comprehensive README
- [ ] Write user guide
- [ ] Create command reference
- [ ] Add examples and tutorials
- [ ] Write migration guide

**Files:**
```
docs/
├── README.md
├── GUIDE.md                     # User guide
├── COMMANDS.md                  # Command reference
├── EXAMPLES.md                  # Examples
└── MIGRATION.md                 # Migration guide
```

**Quality Gates:**
- ✅ Documentation is complete
- ✅ Examples are tested
- ✅ Migration guide is clear

#### 5.5 Self-Update Mechanism

**Tasks:**
- [ ] Implement `wnt upgrade-self` command
- [ ] Check for new versions
- [ ] Download and verify new binary
- [ ] Replace current binary
- [ ] Handle permissions correctly

**Files:**
```
src/commands/
└── upgrade_self.rs              # Self-update command
```

**Quality Gates:**
- ✅ Works on all platforms
- ✅ Handles permissions correctly
- ✅ Provides rollback on failure

### Phase 5 Exit Criteria
- ✅ Installation works on all platforms
- ✅ CI/CD pipeline is complete
- ✅ Documentation is comprehensive
- ✅ Self-update works reliably
- ✅ Production ready for v0.1.0 release

---

## Module Structure

### File Organization

```
crates/cli/
├── Cargo.toml
├── build.rs
├── README.md
├── PRD.md
├── PLAN.md
├── src/
│   ├── main.rs                  # Entry point
│   ├── lib.rs                   # Library interface (for testing)
│   ├── cli/
│   │   ├── mod.rs              # CLI framework
│   │   ├── commands.rs         # Commands enum
│   │   ├── args.rs             # Global arguments
│   │   ├── dispatch.rs         # Command dispatcher
│   │   ├── completions.rs      # Shell completions
│   │   └── tests.rs
│   ├── commands/
│   │   ├── mod.rs              # Command exports
│   │   ├── init.rs
│   │   ├── config.rs
│   │   ├── changeset/
│   │   │   ├── mod.rs
│   │   │   ├── add.rs
│   │   │   ├── list.rs
│   │   │   ├── show.rs
│   │   │   ├── update.rs
│   │   │   ├── edit.rs
│   │   │   ├── remove.rs
│   │   │   └── history.rs
│   │   ├── bump/
│   │   │   ├── mod.rs
│   │   │   ├── execute.rs
│   │   │   ├── preview.rs
│   │   │   ├── git_integration.rs
│   │   │   └── report.rs
│   │   ├── upgrade/
│   │   │   ├── mod.rs
│   │   │   ├── check.rs
│   │   │   ├── apply.rs
│   │   │   └── rollback.rs
│   │   ├── audit/
│   │   │   ├── mod.rs
│   │   │   ├── comprehensive.rs
│   │   │   ├── upgrades.rs
│   │   │   ├── dependencies.rs
│   │   │   ├── versions.rs
│   │   │   ├── breaking.rs
│   │   │   └── report.rs
│   │   ├── changes.rs
│   │   └── upgrade_self.rs
│   ├── error/
│   │   ├── mod.rs
│   │   ├── display.rs
│   │   ├── exit_codes.rs
│   │   └── tests.rs
│   ├── output/
│   │   ├── mod.rs
│   │   ├── table.rs
│   │   ├── json.rs
│   │   ├── progress.rs
│   │   ├── style.rs
│   │   ├── logger.rs
│   │   ├── diff.rs
│   │   ├── highlight.rs
│   │   ├── export.rs
│   │   └── tests.rs
│   ├── interactive/
│   │   ├── mod.rs
│   │   ├── prompts.rs
│   │   ├── select.rs
│   │   └── confirm.rs
│   └── utils/
│       ├── mod.rs
│       ├── platform.rs
│       ├── editor.rs
│       └── terminal.rs
├── tests/
│   ├── integration/
│   │   ├── mod.rs
│   │   ├── init_test.rs
│   │   ├── changeset_test.rs
│   │   ├── bump_test.rs
│   │   ├── upgrade_test.rs
│   │   └── audit_test.rs
│   ├── e2e/
│   │   ├── mod.rs
│   │   ├── full_workflow_test.rs
│   │   └── git_integration_test.rs
│   └── fixtures/
│       ├── sample_monorepo/
│       └── test_configs/
├── install.sh
└── scripts/
    ├── install-dev.sh
    └── uninstall.sh
```

### Visibility Rules

```rust
// Public API - CLI commands interface
pub async fn execute_command(cmd: Commands, output: &Output) -> Result<()> { ... }

// Internal to crate - shared between command modules
pub(crate) struct CommandContext { ... }
pub(crate) fn validate_workspace() -> Result<()> { ... }

// Private to module
struct InternalState { ... }
fn helper_function() -> Result<()> { ... }
```

### main.rs Pattern

```rust
//! # Workspace Node Tools CLI
//!
//! ## What
//! Command-line interface for managing Node.js monorepos with changeset-based versioning.
//!
//! ## How
//! Built with clap for argument parsing, integrates sublime-package-tools for core logic,
//! and provides interactive prompts and formatted output.
//!
//! ## Why
//! To provide a fast, reliable, and user-friendly CLI for modern monorepo workflows.

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod cli;
mod commands;
mod error;
mod interactive;
mod output;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    // Parse CLI arguments
    let cli = cli::Cli::parse();

    // Setup output handler
    let output = output::Output::new(cli.format, cli.no_color);

    // Dispatch command
    match cli::dispatch(cli.command, &output).await {
        Ok(()) => Ok(()),
        Err(e) => {
            output.error(&e.to_string());
            std::process::exit(e.exit_code());
        }
    }
}
```

---

## Quality Standards

### Clippy Rules (Mandatory)

```rust
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]
```

### Code Quality Checklist

For every module/file:

- [ ] Module-level documentation (What, How, Why)
- [ ] All public functions documented with examples
- [ ] All command functions have comprehensive docs
- [ ] Error messages are user-friendly
- [ ] Help text is clear and comprehensive
- [ ] No `unwrap()` or `expect()` calls
- [ ] No `todo!()`, `unimplemented!()`, `panic!()`
- [ ] All `Result` types used correctly
- [ ] Errors provide helpful suggestions
- [ ] Internal types use `pub(crate)`
- [ ] Tests in separate files/modules
- [ ] Integration tests cover main workflows
- [ ] E2E tests verify complete user scenarios

### Error Handling Pattern

```rust
/// CLI-specific error type
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Validation failed: {0}")]
    Validation(String),
    
    #[error("Execution failed: {0}")]
    Execution(String),
    
    #[error("Git error: {0}")]
    Git(#[from] sublime_git_tools::Error),
    
    #[error("Package error: {0}")]
    Package(#[from] sublime_package_tools::Error),
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Configuration(_) => 78,  // EX_CONFIG
            CliError::Validation(_) => 65,     // EX_DATAERR
            CliError::Execution(_) => 70,      // EX_SOFTWARE
            CliError::Git(_) => 70,
            CliError::Package(_) => 70,
        }
    }
    
    pub fn user_message(&self) -> String {
        match self {
            CliError::Configuration(msg) => {
                format!("Configuration error: {}\n\nTry running 'wnt config validate' for more details.", msg)
            }
            CliError::Validation(msg) => {
                format!("Validation failed: {}\n\nPlease check your input and try again.", msg)
            }
            _ => self.to_string(),
        }
    }
}

impl AsRef<str> for CliError {
    fn as_ref(&self) -> &str {
        match self {
            CliError::Configuration(_) => "CliError::Configuration",
            CliError::Validation(_) => "CliError::Validation",
            CliError::Execution(_) => "CliError::Execution",
            CliError::Git(_) => "CliError::Git",
            CliError::Package(_) => "CliError::Package",
        }
    }
}
```

### Documentation Pattern

```rust
/// Execute the changeset add command
///
/// Creates a new changeset for tracking version bumps across packages.
/// This command can run interactively (prompting for input) or non-interactively
/// (using provided arguments).
///
/// # Arguments
///
/// * `args` - Command arguments parsed from CLI
/// * `output` - Output handler for displaying results
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if:
/// - Workspace is not initialized
/// - Git repository is not found
/// - No packages are affected
/// - User cancels operation
///
/// # Examples
///
/// ```no_run
/// use wnt::commands::changeset::add::execute;
/// use wnt::output::Output;
///
/// # async fn example() -> anyhow::Result<()> {
/// let args = ChangesetAddArgs {
///     packages: vec!["@example/package".to_string()],
///     bump: Some(BumpType::Minor),
///     message: Some("Add new feature".to_string()),
///     ..Default::default()
/// };
/// let output = Output::new(OutputFormat::Human, false);
///
/// execute(args, &output).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Interactive Mode
///
/// When run without arguments, the command will:
/// 1. Detect affected packages from git changes
/// 2. Prompt for package selection
/// 3. Prompt for bump type (patch, minor, major)
/// 4. Prompt for changeset summary
/// 5. Create and save the changeset file
pub async fn execute(args: ChangesetAddArgs, output: &Output) -> Result<()> {
    // Implementation
}
```

---

## Testing Strategy

### Test Organization

```
src/
├── commands/
│   └── changeset/
│       ├── add.rs
│       └── tests/
│           ├── mod.rs
│           ├── unit_tests.rs
│           └── integration_tests.rs
tests/
├── integration/                 # Integration tests
│   ├── mod.rs
│   ├── init_test.rs
│   └── changeset_test.rs
├── e2e/                        # End-to-end tests
│   ├── mod.rs
│   ├── full_workflow_test.rs
│   └── git_integration_test.rs
└── fixtures/                   # Test fixtures
    ├── sample_monorepo/
    └── test_configs/
```

### Test Categories

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bump_type() {
        let result = BumpType::from_str("patch");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), BumpType::Patch);
    }

    #[tokio::test]
    async fn test_validate_packages() {
        let packages = vec!["@example/pkg".to_string()];
        let result = validate_packages(&packages).await;
        assert!(result.is_ok());
    }
}
```

#### Integration Tests
```rust
#[tokio::test]
async fn test_init_command_creates_config() {
    let temp_dir = TempDir::new().unwrap();
    let args = InitArgs {
        path: temp_dir.path().to_path_buf(),
        force: false,
    };
    let output = Output::new(OutputFormat::Quiet, true);

    execute_init(args, &output).await.unwrap();

    assert!(temp_dir.path().join(".changesets").exists());
    assert!(temp_dir.path().join("repo.config.toml").exists());
}
```

#### E2E Tests
```rust
#[tokio::test]
async fn test_full_release_workflow() {
    // 1. Setup test repository
    let repo = setup_test_repo().await;
    
    // 2. Initialize wnt
    run_command(&["init"]).await.unwrap();
    
    // 3. Create changeset
    run_command(&["changeset", "add", "--packages", "pkg1", "--bump", "minor"]).await.unwrap();
    
    // 4. Bump versions
    let result = run_command(&["bump", "--execute"]).await.unwrap();
    
    // 5. Verify results
    assert!(result.contains("pkg1"));
    assert!(repo.path().join("packages/pkg1/package.json").exists());
    
    // Cleanup
    cleanup_test_repo(repo).await;
}
```

### Test Coverage Requirements

**100% coverage on:**
- All command execution paths
- All error handling paths
- All user input validation
- All output formatting
- All configuration parsing

**Tools:**
```bash
cargo tarpaulin --out Html --output-dir coverage/ --all-features --workspace
```

### Mock Implementations

```rust
pub(crate) struct MockOutput {
    messages: Arc<Mutex<Vec<String>>>,
}

impl MockOutput {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn get_messages(&self) -> Vec<String> {
        self.messages.lock().unwrap().clone()
    }
}

pub(crate) struct MockGit {
    commits: Vec<Commit>,
    branches: Vec<String>,
}

pub(crate) struct MockFileSystem {
    files: HashMap<PathBuf, String>,
}
```

---

## Documentation Requirements

### CLI Help Text (100%)

Every command must have:
- Brief description (one line)
- Detailed description
- Usage examples
- Argument descriptions
- Flag descriptions
- Exit code documentation

### API Documentation (100%)

All public functions must have:
- What: Brief description
- How: Detailed implementation notes
- Arguments: All parameters documented
- Returns: Return value documented
- Errors: All error cases documented
- Examples: At least one working example
- Related: Links to related functions/types

### User Documentation

Required documentation:
- README.md - Project overview and quick start
- GUIDE.md - Comprehensive user guide
- COMMANDS.md - Command reference
- EXAMPLES.md - Real-world examples
- MIGRATION.md - Migration guide from other tools

### Internal Documentation

```rust
//! # Module Name
//!
//! ## What
//! This module handles changeset creation and management.
//!
//! ## How
//! It provides an interactive prompt flow for creating changesets,
//! validates user input, and persists changesets to the filesystem.
//!
//! ## Why
//! Changesets are the core of the version management workflow,
//! requiring a dedicated module for maintainability.
```

---

## Milestones & Timeline

### Milestone 1: Foundation Complete (End of Week 3)

**Deliverables:**
- ✅ CLI framework functional
- ✅ Error handling complete
- ✅ Output formatting working
- ✅ Init and config commands working
- ✅ Basic tests passing

**Acceptance Criteria:**
- Can run `wnt init` successfully
- Can run `wnt config show`
- Error messages are helpful
- Output works in all modes

### Milestone 2: Changeset Management Complete (End of Week 7)

**Deliverables:**
- ✅ All changeset commands working
- ✅ Interactive prompts polished
- ✅ Git integration solid
- ✅ Comprehensive tests

**Acceptance Criteria:**
- Complete changeset workflow works
- Performance < 100ms for most operations
- E2E tests pass
- User feedback is positive

### Milestone 3: Version Management Complete (End of Week 11)

**Deliverables:**
- ✅ Bump command fully functional
- ✅ Changes command working
- ✅ Upgrade commands working
- ✅ Git operations atomic

**Acceptance Criteria:**
- Can complete full release workflow
- Git integration is reliable
- Rollback works correctly
- Performance meets requirements

### Milestone 4: Audit & Polish Complete (End of Week 14)

**Deliverables:**
- ✅ Audit system comprehensive
- ✅ Interactive features polished
- ✅ Output professional quality
- ✅ All tests passing

**Acceptance Criteria:**
- Audit provides actionable insights
- UX is intuitive and fast
- Documentation is complete
- Ready for beta release

### Milestone 5: Production Ready (End of Week 16)

**Deliverables:**
- ✅ Installation scripts working
- ✅ CI/CD pipeline complete
- ✅ Documentation comprehensive
- ✅ Self-update working
- ✅ v0.1.0 released

**Acceptance Criteria:**
- Works on all platforms
- Installation is smooth
- Documentation is clear
- Ready for public release

---

## Risk Assessment

### High Risk Items

#### 1. Cross-Platform Compatibility
**Risk**: CLI might not work consistently across macOS, Linux, and Windows

**Mitigation:**
- Test on all platforms early and often
- Use cross-platform libraries (crossterm, console)
- Setup CI to run tests on all platforms
- Document platform-specific issues

**Contingency:**
- Focus on macOS/Linux first
- Add Windows support in later phase

#### 2. Git Integration Reliability
**Risk**: Git operations might fail or leave repository in inconsistent state

**Mitigation:**
- Make all git operations atomic
- Implement comprehensive rollback
- Test extensively with different git states
- Use sublime-git-tools which is battle-tested

**Contingency:**
- Provide manual recovery instructions
- Add force flags for edge cases

#### 3. Performance in Large Monorepos
**Risk**: CLI might be slow with 100+ packages

**Mitigation:**
- Profile early and often
- Use async operations where possible
- Cache parsed data
- Implement parallel processing

**Contingency:**
- Add performance options (--fast, --parallel)
- Document performance expectations

#### 4. Terminal Compatibility
**Risk**: Output formatting might break in some terminals

**Mitigation:**
- Test in multiple terminal emulators
- Use battle-tested libraries (crossterm)
- Respect NO_COLOR environment variable
- Provide fallback modes

**Contingency:**
- Add --simple flag for basic output
- Document supported terminals

### Medium Risk Items

#### 1. Interactive Prompt UX
**Risk**: Prompts might be confusing or slow

**Mitigation:**
- User testing early in development
- Iterate based on feedback
- Provide non-interactive alternatives
- Add comprehensive help text

#### 2. Installation Complexity
**Risk**: Users might struggle to install

**Mitigation:**
- Test installation on clean systems
- Provide multiple installation methods
- Create detailed installation docs
- Add troubleshooting guide

#### 3. Documentation Maintenance
**Risk**: Documentation might become outdated

**Mitigation:**
- Generate docs from code where possible
- Include docs in PR reviews
- Setup doc testing
- Regular doc audits

### Low Risk Items

#### 1. Shell Completion Generation
**Risk**: Completions might not work in all shells

**Mitigation:**
- Use clap's built-in completion generation
- Test in common shells
- Provide manual installation instructions

#### 2. Self-Update Mechanism
**Risk**: Self-update might fail on some systems

**Mitigation:**
- Make self-update optional
- Provide alternative update methods
- Handle permissions carefully
- Test thoroughly

---

## Development Workflow

### Daily Workflow

```bash
# 1. Pull latest changes
git pull origin main

# 2. Create feature branch
git checkout -b feat/changeset-add-command

# 3. Implement feature
# - Write tests first (TDD)
# - Implement functionality
# - Document code

# 4. Run checks
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps

# 5. Commit changes
git add .
git commit -m "feat(changeset): implement add command"

# 6. Push and create PR
git push origin feat/changeset-add-command
gh pr create --title "feat(changeset): implement add command"
```

### PR Requirements

Before merging, PRs must:
- [ ] Pass all CI checks
- [ ] Have 100% test coverage
- [ ] Pass clippy without warnings
- [ ] Have comprehensive documentation
- [ ] Include examples if adding new commands
- [ ] Update CHANGELOG.md
- [ ] Be reviewed by at least one maintainer

### Conventional Commit Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Test additions or changes
- `chore`: Build process or tooling changes

**Scopes:**
- `cli`: CLI framework
- `changeset`: Changeset commands
- `bump`: Version bump commands
- `upgrade`: Upgrade commands
- `audit`: Audit commands
- `output`: Output formatting
- `error`: Error handling
- `docs`: Documentation

**Examples:**
```
feat(changeset): implement interactive add command

Add interactive prompt flow for creating changesets with
package selection, bump type, and summary input.

Closes #42

fix(output): handle wide unicode characters in tables

Tables were incorrectly calculating column widths when
displaying unicode characters. Fixed by using unicode-width
crate for accurate width calculation.

Fixes #73
```

---

## CI/CD Pipeline

### GitHub Actions Workflows

#### CI Workflow

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    name: Test
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable]
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
          profile: minimal
          override: true
          components: rustfmt, clippy
      
      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Cache target directory
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Check formatting
        run: cargo fmt --check
      
      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
      
      - name: Run tests
        run: cargo test --all-features --workspace
      
      - name: Check documentation
        run: cargo doc --no-deps --all-features

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      
      - name: Generate coverage
        run: cargo tarpaulin --out Xml --all-features --workspace
      
      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml
          fail_ci_if_error: true
      
      - name: Check 100% coverage
        run: |
          COVERAGE=$(cargo tarpaulin --out Json --all-features --workspace | jq '.files | map(.covered / .coverable) | add / length * 100')
          if (( $(echo "$COVERAGE < 100" | bc -l) )); then
            echo "Coverage is below 100%: $COVERAGE%"
            exit 1
          fi
```

#### Release Workflow

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
          override: true
      
      - name: Build release binary
        run: cargo build --release --target ${{ matrix.target }}
      
      - name: Strip binary (Linux/macOS)
        if: matrix.os != 'windows-latest'
        run: strip target/${{ matrix.target }}/release/wnt
      
      - name: Create archive
        shell: bash
        run: |
          VERSION=${GITHUB_REF#refs/tags/v}
          ARCHIVE="wnt-$VERSION-${{ matrix.target }}"
          
          if [ "${{ matrix.os }}" = "windows-latest" ]; then
            cp target/${{ matrix.target }}/release/wnt.exe .
            7z a "$ARCHIVE.zip" wnt.exe
          else
            cp target/${{ matrix.target }}/release/wnt .
            tar czf "$ARCHIVE.tar.gz" wnt
          fi
      
      - name: Generate checksum
        shell: bash
        run: |
          if [ "${{ matrix.os }}" = "windows-latest" ]; then
            shasum -a 256 *.zip > checksums.txt
          else
            shasum -a 256 *.tar.gz > checksums.txt
          fi
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: binaries
          path: |
            *.tar.gz
            *.zip
            checksums.txt

  release:
    name: Create Release
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Download artifacts
        uses: actions/download-artifact@v3
        with:
          name: binaries
      
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            *.tar.gz
            *.zip
            checksums.txt
          draft: false
          prerelease: false
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  publish:
    name: Publish to crates.io
    needs: release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      
      - name: Publish
        run: cargo publish --token ${{ secrets.CARGO_TOKEN }}
```

---

## Success Metrics

### Code Metrics

- ✅ **Test Coverage**: 100%
- ✅ **Clippy Compliance**: 100% (no warnings)
- ✅ **Documentation Coverage**: 100%
- ✅ **Binary Size**: < 10MB
- ✅ **Startup Time**: < 50ms

### Performance Metrics

- ✅ **Init command**: < 500ms
- ✅ **Config commands**: < 100ms
- ✅ **Changeset add**: < 200ms (interactive), < 100ms (non-interactive)
- ✅ **Changeset list**: < 100ms (100 changesets)
- ✅ **Bump preview**: < 1s (100 packages)
- ✅ **Bump execute**: < 2s (100 packages)
- ✅ **Upgrade check**: < 2s (100 packages)
- ✅ **Audit**: < 3s (100 packages)
- ✅ **Git hooks**: < 500ms

### Quality Metrics

- ✅ **Bug Reports**: < 5 per month (after v1.0)
- ✅ **CI Success Rate**: > 99%
- ✅ **Install Success Rate**: > 95%
- ✅ **User Satisfaction**: > 4.5/5 (GitHub stars ratio)

### Adoption Metrics

- ✅ **Downloads**: 1000+ in first month
- ✅ **GitHub Stars**: 100+ in first month
- ✅ **Active Projects**: 50+ in first quarter
- ✅ **Contributors**: 5+ in first quarter

---

## Appendix A: Exit Codes

Following sysexits.h standard:

```rust
pub mod exit_codes {
    pub const OK: i32 = 0;           // Successful termination
    pub const USAGE: i32 = 64;       // Command line usage error
    pub const DATAERR: i32 = 65;     // Data format error
    pub const NOINPUT: i32 = 66;     // Cannot open input
    pub const NOUSER: i32 = 67;      // Addressee unknown
    pub const NOHOST: i32 = 68;      // Host name unknown
    pub const UNAVAILABLE: i32 = 69; // Service unavailable
    pub const SOFTWARE: i32 = 70;    // Internal software error
    pub const OSERR: i32 = 71;       // System error
    pub const OSFILE: i32 = 72;      // Critical OS file missing
    pub const CANTCREAT: i32 = 73;   // Can't create output file
    pub const IOERR: i32 = 74;       // Input/output error
    pub const TEMPFAIL: i32 = 75;    // Temp failure; user is invited to retry
    pub const PROTOCOL: i32 = 76;    // Remote error in protocol
    pub const NOPERM: i32 = 77;      // Permission denied
    pub const CONFIG: i32 = 78;      // Configuration error
}
```

---

## Appendix B: Command Quick Reference

```bash
# Initialization
wnt init                         # Initialize workspace
wnt init --force                 # Force re-initialization

# Configuration
wnt config show                  # Display configuration
wnt config validate              # Validate configuration

# Changesets
wnt changeset add                # Add changeset (interactive)
wnt changeset add -p pkg1 -b minor  # Add changeset (non-interactive)
wnt changeset list               # List all changesets
wnt changeset list --branch main # List changesets for branch
wnt changeset show