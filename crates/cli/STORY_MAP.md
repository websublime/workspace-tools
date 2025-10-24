# Workspace Node Tools CLI - Development Story Map

**Version**: 1.0  
**Based on**: PLAN.md v1.0 & PRD.md v1.0  
**Last Updated**: 2024-01-15  
**Status**: 📋 Ready for Development

---

## Table of Contents

1. [Story Map Overview](#story-map-overview)
2. [Effort Metrics Definition](#effort-metrics-definition)
3. [Epic 1: CLI Foundation](#epic-1-cli-foundation)
4. [Epic 2: Configuration Commands](#epic-2-configuration-commands)
5. [Epic 3: Output System](#epic-3-output-system)
6. [Epic 4: Changeset Commands](#epic-4-changeset-commands)
7. [Epic 5: Version Management Commands](#epic-5-version-management-commands)
8. [Epic 6: Upgrade Commands](#epic-6-upgrade-commands)
9. [Epic 7: Audit Commands](#epic-7-audit-commands)
10. [Epic 8: Advanced Features](#epic-8-advanced-features)
11. [Epic 9: Distribution](#epic-9-distribution)
12. [Epic 10: Documentation & Examples](#epic-10-documentation--examples)

---

## Story Map Overview

### Epic Breakdown

```
Phase 1: Foundation & Core Commands (Weeks 1-3)
├── Epic 1: CLI Foundation
├── Epic 2: Configuration Commands
└── Epic 3: Output System

Phase 2: Changeset Management (Weeks 4-7)
└── Epic 4: Changeset Commands

Phase 3: Version Management & Upgrades (Weeks 8-11)
├── Epic 5: Version Management Commands
└── Epic 6: Upgrade Commands

Phase 4: Audit & Advanced Features (Weeks 12-14)
├── Epic 7: Audit Commands
└── Epic 8: Advanced Features

Phase 5: Distribution & Polish (Weeks 15-16)
├── Epic 9: Distribution
└── Epic 10: Documentation & Examples
```

### Total Story Count
- **Epics**: 10
- **User Stories**: 78
- **Tasks**: 450+

---

## Effort Metrics Definition

### Effort Levels

| Level | Time Estimate | Complexity | Examples |
|-------|--------------|------------|----------|
| **Minimal** | 1-2 hours | Trivial | Simple struct, basic CLI arg, straightforward test |
| **Low** | 3-6 hours | Simple | Single command implementation, basic subcommand, simple validation |
| **Medium** | 1-2 days | Moderate | Complex command flow, interactive prompts, comprehensive testing |
| **High** | 3-5 days | Complex | Multi-step commands, git integration, extensive edge cases |
| **Massive** | 1-2 weeks | Very Complex | Complete command suite, full workflow, performance optimization |

### Estimation Guidelines

**Minimal (1-2h)**:
- Creating simple CLI argument structs
- Adding basic command scaffolding
- Writing straightforward tests
- Simple help text updates

**Low (3-6h)**:
- Implementing single commands with clear flow
- Creating basic output formatting
- Writing unit tests for simple functions
- Adding command-level documentation

**Medium (1-2d)**:
- Implementing commands with interactive prompts
- Creating table rendering logic
- Writing comprehensive test suites
- Integration with library crates

**High (3-5d)**:
- Implementing complex multi-step workflows
- Git operation integration
- Full test coverage with edge cases
- Performance optimization

**Massive (1-2w)**:
- Complete command subsystem (e.g., all changeset commands)
- Multiple integration points
- E2E testing scenarios
- Comprehensive documentation and examples

---

## Epic 1: CLI Foundation

**Phase**: 1  
**Total Effort**: High  
**Dependencies**: None  
**Goal**: Establish CLI framework, error handling, and project structure

### Story 1.1: Initialize CLI Crate Structure
**Effort**: Low  
**Priority**: Critical

**As a** developer  
**I want** the basic CLI crate structure initialized  
**So that** I can start implementing commands following standard patterns

**Description**:
Set up the foundational CLI project structure with proper dependencies, clippy rules, and module scaffolding.

**Tasks**:
1. Create `Cargo.toml` with all dependencies
   - Add clap with derive feature
   - Add output dependencies (crossterm, console, dialoguer, indicatif, comfy-table)
   - Add internal crates (sublime-package-tools, sublime-standard-tools, sublime-git-tools)
   - Add serialization (serde, serde_json, toml, serde_yaml)
   - Add error handling (anyhow, thiserror)
   - Add logging (tracing, tracing-subscriber)
   - Add utilities (terminal_size, clap_complete, sysexits)
   - Configure release profile optimization
   - **Effort**: Low

2. Create `src/main.rs` with async runtime
   - Setup tokio runtime
   - Initialize logging with tracing
   - Parse CLI arguments with clap
   - Setup error handling with exit codes
   - Add main dispatch logic
   - **Effort**: Low

3. Create `src/lib.rs` for testability
   - Export command execution functions
   - Add clippy rules
   - Add crate-level documentation
   - **Effort**: Minimal

4. Create module directory structure
   - Create `src/cli/` for CLI framework
   - Create `src/commands/` for command implementations
   - Create `src/error/` for error handling
   - Create `src/output/` for output formatting
   - Create `src/interactive/` for prompts
   - Create `src/utils/` for utilities
   - Add empty `mod.rs` files
   - **Effort**: Minimal

5. Create `build.rs` for shell completions
   - Generate completions at build time
   - Support bash, zsh, fish, powershell
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] `Cargo.toml` contains all required dependencies
- [ ] Project compiles without errors
- [ ] `cargo fmt` runs successfully
- [ ] `cargo clippy` runs successfully
- [ ] Main entry point with async runtime works
- [ ] Module structure follows plan
- [ ] Build script generates completions

**Definition of Done**:
- [ ] Code compiles
- [ ] Clippy passes
- [ ] Basic structure documented
- [ ] PR approved and merged

**Dependencies**: None

**Blocked By**: None

**Blocks**: All other stories

---

### Story 1.2: Setup CI/CD Pipeline
**Effort**: Medium  
**Priority**: Critical

**As a** developer  
**I want** automated CI/CD pipelines configured  
**So that** code quality is enforced automatically on all platforms

**Description**:
Configure GitHub Actions to run automated checks on every commit and PR, including cross-platform testing.

**Tasks**:
1. Create `.github/workflows/ci.yml`
   - Setup matrix for OS (ubuntu-latest, macos-latest, windows-latest)
   - Setup matrix for Rust versions (stable)
   - Configure cargo fmt check
   - Configure cargo clippy check
   - Configure cargo test
   - Configure cargo doc check
   - Add caching for cargo registry and build artifacts
   - **Effort**: Medium

2. Create `.github/workflows/coverage.yml`
   - Install cargo-tarpaulin
   - Generate coverage report
   - Upload to codecov
   - Enforce 100% coverage requirement
   - **Effort**: Medium

3. Create `.github/workflows/release.yml`
   - Trigger on version tags
   - Build for multiple platforms
   - Create release binaries
   - Upload to GitHub releases
   - Publish to crates.io
   - **Effort**: High

4. Create PR and issue templates
   - Add pull request template
   - Add bug report template
   - Add feature request template
   - Add contributing guidelines
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] CI runs on push and PR
- [ ] Tests run on Windows, macOS, and Linux
- [ ] Code coverage enforced at 100%
- [ ] Clippy warnings cause build failure
- [ ] Format checks enforce consistency
- [ ] Release workflow creates binaries
- [ ] Templates guide contributors

**Definition of Done**:
- [ ] CI pipeline runs successfully
- [ ] All platforms tested
- [ ] Coverage reporting works
- [ ] Release workflow tested
- [ ] Documentation updated

**Dependencies**: Story 1.1

**Blocked By**: Story 1.1

**Blocks**: None

---

### Story 1.3: Implement Error Handling System
**Effort**: Medium  
**Priority**: Critical

**As a** developer  
**I want** comprehensive error handling with user-friendly messages  
**So that** users get helpful feedback when things go wrong

**Description**:
Create a CLI-specific error system that wraps library errors and provides clear, actionable messages with appropriate exit codes.

**Tasks**:
1. Create `src/error/mod.rs` with CliError enum
   - Define error variants (Configuration, Validation, Execution, Git, Package, Io, Network, User)
   - Implement From traits for library errors
   - Implement Display with user-friendly messages
   - Implement AsRef<str> for error identification
   - Add error context helpers
   - **Effort**: Medium

2. Create `src/error/exit_codes.rs`
   - Define exit code constants following sysexits.h
   - Map error types to exit codes
   - Document exit code meanings
   - **Effort**: Minimal

3. Create `src/error/display.rs`
   - Implement user-friendly error formatting
   - Add suggestions for common errors
   - Add color coding for error output
   - Include helpful next steps
   - **Effort**: Low

4. Write comprehensive error tests
   - Test error creation
   - Test error conversion from library types
   - Test exit code mapping
   - Test error messages are helpful
   - Test AsRef<str> implementation
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] All error variants defined
- [ ] Library errors convert correctly
- [ ] Exit codes follow sysexits standard
- [ ] Error messages are clear and actionable
- [ ] Errors include suggestions when appropriate
- [ ] All errors implement AsRef<str>
- [ ] 100% test coverage
- [ ] Clippy passes

**Definition of Done**:
- [ ] Error system compiles
- [ ] All tests pass
- [ ] Documentation complete
- [ ] Examples of error handling provided

**Dependencies**: Story 1.1

**Blocked By**: Story 1.1

**Blocks**: All command stories

---

### Story 1.4: Create CLI Framework with Clap
**Effort**: High  
**Priority**: Critical

**As a** user  
**I want** a well-structured CLI interface  
**So that** I can easily discover and use commands

**Description**:
Implement the main CLI structure using clap, including all command definitions, global arguments, and help text.

**Tasks**:
1. Create `src/cli/mod.rs` with Cli struct
   - Define main Cli struct with clap Parser
   - Add global arguments (verbose, format, no-color, cwd)
   - Add version information
   - Add about text
   - **Effort**: Low

2. Create `src/cli/commands.rs` with Commands enum
   - Define Commands enum with all subcommands
   - Add Init, Config, Changeset, Bump, Upgrade, Audit, Changes
   - Add Version and Help commands
   - Define argument structs for each command
   - **Effort**: High

3. Create `src/cli/args.rs` for shared argument types
   - Define OutputFormat enum (Human, Json, Quiet)
   - Define common argument structs
   - Implement validation
   - **Effort**: Low

4. Create `src/cli/dispatch.rs` for command routing
   - Implement command dispatcher
   - Route to appropriate command handler
   - Handle errors with proper exit codes
   - **Effort**: Medium

5. Create `src/cli/completions.rs` for shell completions
   - Generate completions for bash
   - Generate completions for zsh
   - Generate completions for fish
   - Generate completions for powershell
   - **Effort**: Low

6. Write CLI parsing tests
   - Test global arguments parse correctly
   - Test each command parses correctly
   - Test argument validation
   - Test help text generation
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] All commands defined in Commands enum
- [ ] Global arguments work across all commands
- [ ] Help text is comprehensive and clear
- [ ] Command parsing validated with tests
- [ ] Shell completions generate correctly
- [ ] Subcommand help works (e.g., wnt changeset --help)
- [ ] Version flag shows correct version
- [ ] Invalid arguments show helpful errors
- [ ] 100% test coverage on parsing

**Definition of Done**:
- [ ] CLI framework compiles
- [ ] All commands parse correctly
- [ ] Help text is complete
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 1.1, Story 1.3

**Blocked By**: Story 1.1, Story 1.3

**Blocks**: All command implementation stories

---

## Epic 2: Configuration Commands

**Phase**: 1  
**Total Effort**: High  
**Dependencies**: Epic 1  
**Goal**: Implement init and config commands for project setup

### Story 2.1: Implement Init Command
**Effort**: High  
**Priority**: Critical

**As a** user  
**I want** to initialize my workspace for wnt  
**So that** I can start using changeset-based versioning

**Description**:
Implement the `wnt init` command that creates configuration, detects workspace structure, and sets up the .wnt directory.

**Tasks**:
1. Create `src/commands/init.rs`
   - Define InitArgs struct
   - Implement execute_init function
   - **Effort**: Low

2. Implement workspace detection
   - Check for package.json in root
   - Detect workspaces configuration
   - Find all packages
   - Validate monorepo structure
   - **Effort**: Medium

3. Implement interactive prompts
   - Prompt for versioning strategy
   - Prompt for changeset path
   - Prompt for changelog format
   - Prompt for git integration
   - Use dialoguer for prompts
   - **Effort**: High

4. Implement configuration generation
   - Generate wnt.toml with defaults
   - Apply user selections
   - Validate generated config
   - **Effort**: Medium

5. Implement .wnt directory creation
   - Create .wnt directory
   - Create changesets subdirectory
   - Create .gitkeep files
   - Set proper permissions
   - **Effort**: Low

6. Implement git integration setup
   - Check git repository exists
   - Optionally setup git hooks
   - Add .wnt to .gitignore if needed
   - **Effort**: Medium

7. Create example changeset
   - Generate example changeset file
   - Add helpful comments
   - **Effort**: Minimal

8. Write comprehensive tests
   - Test in empty directory
   - Test in existing monorepo
   - Test with various configurations
   - Test force re-initialization
   - Test error cases
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Command creates valid wnt.toml
- [ ] Command detects workspace structure correctly
- [ ] Interactive prompts have sensible defaults
- [ ] .wnt directory created with correct structure
- [ ] Git integration optional and working
- [ ] Example changeset is helpful
- [ ] Force flag allows re-initialization
- [ ] Error messages are clear
- [ ] 100% test coverage
- [ ] Works on all platforms

**Definition of Done**:
- [ ] Command works end-to-end
- [ ] All tests pass
- [ ] Documentation complete
- [ ] User guide updated

**Dependencies**: Story 1.4, Story 3.1

**Blocked By**: Story 1.4, Story 3.1

**Blocks**: All other commands (require initialized workspace)

---

### Story 2.2: Implement Config Show Command
**Effort**: Medium  
**Priority**: High

**As a** user  
**I want** to view my current configuration  
**So that** I can verify settings and troubleshoot issues

**Description**:
Implement `wnt config show` to display current configuration in human-readable or JSON format.

**Tasks**:
1. Create `src/commands/config.rs`
   - Define ConfigArgs with subcommands
   - Implement config show logic
   - **Effort**: Low

2. Implement configuration loading
   - Load from wnt.toml
   - Apply environment variable overrides
   - Merge with defaults
   - **Effort**: Medium

3. Implement human-readable output
   - Format as organized sections
   - Highlight non-default values
   - Use colors for readability
   - **Effort**: Medium

4. Implement JSON output
   - Serialize complete config
   - Format with pretty printing
   - **Effort**: Minimal

5. Write tests
   - Test with default config
   - Test with custom config
   - Test with env var overrides
   - Test JSON output
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Command displays current configuration
- [ ] Human format is easy to read
- [ ] JSON format is valid and complete
- [ ] Environment overrides shown
- [ ] Non-default values highlighted
- [ ] Works with missing config file (shows defaults)
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Command works correctly
- [ ] All output formats tested
- [ ] Documentation complete

**Dependencies**: Story 1.4, Story 3.2

**Blocked By**: Story 1.4, Story 3.2

**Blocks**: None

---

### Story 2.3: Implement Config Validate Command
**Effort**: Medium  
**Priority**: High

**As a** user  
**I want** to validate my configuration  
**So that** I can catch issues before running commands

**Description**:
Implement `wnt config validate` to check configuration for errors and provide actionable feedback.

**Tasks**:
1. Extend `src/commands/config.rs`
   - Add validate subcommand
   - Implement validation logic
   - **Effort**: Low

2. Implement configuration validation
   - Check required fields present
   - Validate paths exist
   - Validate enum values
   - Check dependency consistency
   - Use sublime-package-tools validation
   - **Effort**: High

3. Implement validation reporting
   - Show errors with line numbers
   - Provide suggestions for fixes
   - Show warnings for deprecated options
   - Use colors for severity
   - **Effort**: Medium

4. Write tests
   - Test valid configuration
   - Test various invalid configurations
   - Test error messages are helpful
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Command validates all config aspects
- [ ] Errors show file location when possible
- [ ] Suggestions provided for common issues
- [ ] Warnings don't cause command failure
- [ ] Exit code 0 for valid, non-zero for invalid
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Validation comprehensive
- [ ] Error messages actionable
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 1.4, Story 3.2

**Blocked By**: Story 1.4, Story 3.2

**Blocks**: None

---

## Epic 3: Output System

**Phase**: 1  
**Total Effort**: High  
**Dependencies**: Epic 1  
**Goal**: Implement comprehensive output formatting and logging

### Story 3.1: Implement Output Format Framework
**Effort**: Medium  
**Priority**: Critical

**As a** developer  
**I want** a unified output system  
**So that** all commands provide consistent output

**Description**:
Create the core output system that handles different output formats (human, JSON, quiet) and provides utilities for all commands.

**Tasks**:
1. Create `src/output/mod.rs` with Output struct
   - Define OutputFormat enum
   - Implement Output struct with format field
   - Add methods for success, error, warning, info
   - Handle NO_COLOR environment variable
   - **Effort**: Low

2. Create `src/output/style.rs` for styling
   - Define color scheme
   - Create styled text helpers
   - Implement emoji/icon helpers
   - Handle terminal capability detection
   - **Effort**: Medium

3. Create `src/output/json.rs` for JSON output
   - Implement JSON serialization wrapper
   - Add pretty printing
   - Handle streaming JSON for multiple outputs
   - **Effort**: Low

4. Write output tests
   - Test each output format
   - Test color handling
   - Test NO_COLOR environment variable
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Output struct supports all formats
- [ ] Colors work in capable terminals
- [ ] NO_COLOR respected
- [ ] JSON output is valid
- [ ] Quiet mode suppresses all output except errors
- [ ] Styling is consistent
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Output system compiles
- [ ] All formats work
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 1.1, Story 1.3

**Blocked By**: Story 1.1, Story 1.3

**Blocks**: All command implementations

---

### Story 3.2: Implement Table Rendering
**Effort**: High  
**Priority**: High

**As a** user  
**I want** data displayed in readable tables  
**So that** I can easily scan and understand output

**Description**:
Implement table rendering using comfy-table with support for various column types and responsive widths.

**Tasks**:
1. Create `src/output/table.rs`
   - Wrap comfy-table with convenience API
   - Define TableBuilder
   - Add column configuration
   - Add row data helpers
   - **Effort**: Medium

2. Implement responsive table sizing
   - Detect terminal width
   - Truncate long columns intelligently
   - Add horizontal scrolling indicators
   - Handle narrow terminals gracefully
   - **Effort**: High

3. Implement table themes
   - Define consistent table style
   - Support different themes for different contexts
   - Handle color output
   - **Effort**: Low

4. Add table utilities
   - Alignment helpers
   - Cell formatting helpers
   - Header styling
   - **Effort**: Low

5. Write comprehensive tests
   - Test various table layouts
   - Test terminal width handling
   - Test color output
   - Test edge cases (empty tables, single column, etc.)
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Tables render correctly in all terminal sizes
- [ ] Long content truncated intelligently
- [ ] Headers styled appropriately
- [ ] Alignment works correctly
- [ ] Colors enhance readability
- [ ] Performance acceptable for large tables
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Table rendering works
- [ ] Responsive sizing implemented
- [ ] Tests pass
- [ ] Documentation with examples

**Dependencies**: Story 3.1

**Blocked By**: Story 3.1

**Blocks**: List commands (changeset list, upgrade check, etc.)

---

### Story 3.3: Implement Progress Indicators
**Effort**: Medium  
**Priority**: Medium

**As a** user  
**I want** progress indicators for long operations  
**So that** I know the command is working

**Description**:
Implement progress bars and spinners using indicatif for operations that take time.

**Tasks**:
1. Create `src/output/progress.rs`
   - Wrap indicatif ProgressBar
   - Create spinner helper
   - Create progress bar helper
   - Add multi-progress support
   - **Effort**: Medium

2. Implement progress styles
   - Define spinner style
   - Define progress bar style
   - Match overall CLI theme
   - **Effort**: Low

3. Add progress utilities
   - Auto-cleanup on completion
   - Handle quiet mode (disable progress)
   - Handle non-TTY output
   - **Effort**: Medium

4. Write tests
   - Test progress creation
   - Test progress updates
   - Test completion
   - Test quiet mode suppression
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Spinners work for indeterminate operations
- [ ] Progress bars work for determinate operations
- [ ] Multiple progress indicators supported
- [ ] Quiet mode disables progress
- [ ] Non-TTY detection works
- [ ] Auto-cleanup on completion
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Progress indicators work
- [ ] All modes tested
- [ ] Documentation complete

**Dependencies**: Story 3.1

**Blocked By**: Story 3.1

**Blocks**: Long-running commands (bump, upgrade, audit)

---

### Story 3.4: Implement Logging System
**Effort**: Low  
**Priority**: Medium

**As a** developer  
**I want** structured logging  
**So that** I can debug issues and understand command execution

**Description**:
Setup tracing-based logging with configurable levels and output formatting.

**Tasks**:
1. Create `src/output/logger.rs`
   - Initialize tracing subscriber
   - Configure log levels from verbose flag
   - Setup log formatting
   - **Effort**: Low

2. Implement log level mapping
   - Map verbose count to log levels
   - Support RUST_LOG environment variable
   - Filter library logs appropriately
   - **Effort**: Low

3. Add logging utilities
   - Create span helpers for command execution
   - Add structured field helpers
   - **Effort**: Low

4. Write tests
   - Test log level configuration
   - Test log output
   - Test verbose flag handling
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Logging configured at startup
- [ ] Verbose flag controls log level
- [ ] RUST_LOG respected
- [ ] Logs are structured and helpful
- [ ] Library logs filtered appropriately
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Logging system works
- [ ] All levels tested
- [ ] Documentation complete

**Dependencies**: Story 1.1

**Blocked By**: Story 1.1

**Blocks**: None

---

## Epic 4: Changeset Commands

**Phase**: 2  
**Total Effort**: Massive  
**Dependencies**: Epic 1, Epic 2, Epic 3  
**Goal**: Implement all changeset management commands

### Story 4.1: Implement Changeset Add Command (Interactive)
**Effort**: High  
**Priority**: Critical

**As a** user  
**I want** to create changesets interactively  
**So that** I can track version bumps easily

**Description**:
Implement `wnt changeset add` with interactive prompts for package selection, bump type, and summary.

**Tasks**:
1. Create `src/commands/changeset/mod.rs`
   - Setup changeset command router
   - Define ChangesetArgs enum
   - **Effort**: Low

2. Create `src/commands/changeset/add.rs`
   - Define ChangesetAddArgs struct
   - Implement execute_changeset_add function
   - **Effort**: Low

3. Implement workspace loading
   - Load configuration
   - Load package information
   - Validate workspace initialized
   - **Effort**: Medium

4. Implement package detection from git
   - Get changed files from git
   - Map files to packages
   - Suggest affected packages
   - **Effort**: High

5. Create `src/interactive/prompts.rs`
   - Implement package multi-select prompt
   - Implement bump type select prompt
   - Implement summary text input
   - Use dialoguer for prompts
   - **Effort**: High

6. Implement changeset creation
   - Generate changeset ID
   - Create changeset data structure
   - Validate changeset data
   - Save to .wnt/changesets/
   - **Effort**: Medium

7. Implement output
   - Show success message
   - Display created changeset summary
   - Show next steps
   - **Effort**: Low

8. Write comprehensive tests
   - Test interactive flow
   - Test with various workspace types
   - Test package detection
   - Test changeset file creation
   - Test error cases
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Command prompts for required information
- [ ] Git detection suggests affected packages
- [ ] Multi-select allows choosing multiple packages
- [ ] Bump type clearly explained
- [ ] Summary prompt has helpful placeholder
- [ ] Changeset file created correctly
- [ ] Success message is informative
- [ ] Works in monorepos and single packages
- [ ] 100% test coverage
- [ ] Performance < 200ms for interactive prompt display

**Definition of Done**:
- [ ] Command works end-to-end
- [ ] Interactive flow is smooth
- [ ] All tests pass
- [ ] Documentation complete
- [ ] User guide updated

**Dependencies**: Story 1.4, Story 2.1, Story 3.1

**Blocked By**: Story 1.4, Story 2.1, Story 3.1

**Blocks**: Story 4.2, Story 5.1

---

### Story 4.2: Implement Changeset Add Command (Non-Interactive)
**Effort**: Medium  
**Priority**: High

**As a** user  
**I want** to create changesets non-interactively  
**So that** I can use it in scripts and CI/CD

**Description**:
Extend `wnt changeset add` to support non-interactive mode with command-line arguments.

**Tasks**:
1. Extend ChangesetAddArgs
   - Add packages argument
   - Add bump argument
   - Add message argument
   - Add skip-prompt flag
   - **Effort**: Low

2. Implement non-interactive logic
   - Validate all required args provided
   - Skip prompts if args present
   - Validate package names
   - Validate bump type
   - **Effort**: Medium

3. Implement validation
   - Check packages exist
   - Check bump type valid
   - Provide helpful errors
   - **Effort**: Low

4. Write tests
   - Test with all args provided
   - Test with missing args (should prompt or error)
   - Test validation errors
   - Test in CI environment (no TTY)
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Command works without prompts when args provided
- [ ] Missing required args show clear errors
- [ ] Validation errors are helpful
- [ ] Works in CI/CD (non-TTY)
- [ ] Performance < 100ms
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Non-interactive mode works
- [ ] All tests pass
- [ ] Documentation updated

**Dependencies**: Story 4.1

**Blocked By**: Story 4.1

**Blocks**: None

---

### Story 4.3: Implement Changeset List Command
**Effort**: Medium  
**Priority**: High

**As a** user  
**I want** to list all changesets  
**So that** I can see pending version bumps

**Description**:
Implement `wnt changeset list` to display all changesets in table or JSON format.

**Tasks**:
1. Create `src/commands/changeset/list.rs`
   - Define ChangesetListArgs struct
   - Add filter options (branch, package, etc.)
   - Implement execute_changeset_list function
   - **Effort**: Low

2. Implement changeset loading
   - Scan .wnt/changesets directory
   - Parse changeset files
   - Handle corrupted files gracefully
   - **Effort**: Medium

3. Implement filtering
   - Filter by branch name
   - Filter by package name
   - Filter by bump type
   - **Effort**: Medium

4. Implement sorting
   - Sort by date (default)
   - Sort by branch
   - Sort by package
   - **Effort**: Low

5. Implement table output
   - Design informative table layout
   - Show ID, packages, bump, summary, date
   - Truncate long values intelligently
   - **Effort**: Medium

6. Implement JSON output
   - Serialize changeset data
   - Include all fields
   - **Effort**: Minimal

7. Write tests
   - Test with no changesets
   - Test with many changesets
   - Test filtering
   - Test sorting
   - Test both output formats
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Command lists all changesets
- [ ] Table output is readable
- [ ] JSON output is complete
- [ ] Filters work correctly
- [ ] Sorting works correctly
- [ ] Empty state handled gracefully
- [ ] Performance < 100ms for 100 changesets
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Command works correctly
- [ ] All filters tested
- [ ] Documentation complete

**Dependencies**: Story 3.2, Story 4.1

**Blocked By**: Story 3.2, Story 4.1

**Blocks**: None

---

### Story 4.4: Implement Changeset Show Command
**Effort**: Low  
**Priority**: Medium

**As a** user  
**I want** to view detailed changeset information  
**So that** I can understand specific version bumps

**Description**:
Implement `wnt changeset show <id>` to display full details of a changeset.

**Tasks**:
1. Create `src/commands/changeset/show.rs`
   - Define ChangesetShowArgs struct
   - Implement execute_changeset_show function
   - **Effort**: Low

2. Implement changeset loading
   - Load by ID or file path
   - Handle not found errors
   - **Effort**: Low

3. Implement detailed output
   - Show all changeset fields
   - Format dates and times
   - Show commit information if present
   - Use colors and sections for readability
   - **Effort**: Medium

4. Write tests
   - Test with valid ID
   - Test with invalid ID
   - Test output formatting
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Command shows complete changeset details
- [ ] Output is well-formatted and readable
- [ ] Not found error is clear
- [ ] JSON output supported
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Command works
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 4.1

**Blocked By**: Story 4.1

**Blocks**: None

---

### Story 4.5: Implement Changeset Update Command
**Effort**: Medium  
**Priority**: Medium

**As a** user  
**I want** to update existing changesets  
**So that** I can refine version bump information

**Description**:
Implement `wnt changeset update <id>` to modify changeset properties.

**Tasks**:
1. Create `src/commands/changeset/update.rs`
   - Define ChangesetUpdateArgs struct
   - Add flags for updating packages, bump, summary
   - Implement execute_changeset_update function
   - **Effort**: Low

2. Implement changeset loading and updating
   - Load existing changeset
   - Apply updates
   - Validate updated changeset
   - Save back to file
   - **Effort**: Medium

3. Implement interactive update mode
   - Show current values
   - Prompt for changes
   - Allow skipping fields
   - **Effort**: Medium

4. Implement commit tracking
   - Add commits to changeset
   - Track modification history
   - **Effort**: Low

5. Write tests
   - Test updating each field
   - Test validation
   - Test interactive mode
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Can update packages, bump, and summary
- [ ] Interactive mode shows current values
- [ ] Updates validated before saving
- [ ] Modification history tracked
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Command works
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 4.1, Story 4.4

**Blocked By**: Story 4.1, Story 4.4

**Blocks**: None

---

### Story 4.6: Implement Changeset Edit Command
**Effort**: Low  
**Priority**: Low

**As a** user  
**I want** to edit changesets in my editor  
**So that** I have full control over changeset content

**Description**:
Implement `wnt changeset edit <id>` to open changeset in $EDITOR.

**Tasks**:
1. Create `src/commands/changeset/edit.rs`
   - Define ChangesetEditArgs struct
   - Implement execute_changeset_edit function
   - **Effort**: Low

2. Create `src/utils/editor.rs`
   - Detect $EDITOR environment variable
   - Fallback to platform defaults
   - Open file in editor
   - Wait for editor to close
   - **Effort**: Medium

3. Implement post-edit validation
   - Reload changeset file
   - Validate changes
   - Reject invalid edits
   - **Effort**: Low

4. Write tests
   - Test editor detection
   - Test validation after edit
   - Mock editor for testing
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Opens changeset in user's editor
- [ ] Editor detection works on all platforms
- [ ] Validation prevents invalid edits
- [ ] Clear error if editor not found
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Command works
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 4.1

**Blocked By**: Story 4.1

**Blocks**: None

---

### Story 4.7: Implement Changeset Remove Command
**Effort**: Low  
**Priority**: Medium

**As a** user  
**I want** to delete changesets  
**So that** I can remove mistakes or outdated entries

**Description**:
Implement `wnt changeset remove <id>` with confirmation and archiving.

**Tasks**:
1. Create `src/commands/changeset/remove.rs`
   - Define ChangesetRemoveArgs struct
   - Add force flag
   - Implement execute_changeset_remove function
   - **Effort**: Low

2. Create `src/interactive/confirm.rs`
   - Implement confirmation dialog
   - Support force flag to skip
   - **Effort**: Low

3. Implement removal logic
   - Archive changeset before removal
   - Delete changeset file
   - Show confirmation message
   - **Effort**: Medium

4. Write tests
   - Test with confirmation
   - Test with force flag
   - Test archiving
   - Test multiple removals
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Requires confirmation by default
- [ ] Force flag skips confirmation
- [ ] Changeset archived before removal
- [ ] Clear success/failure messages
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Command works
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 4.1

**Blocked By**: Story 4.1

**Blocks**: None

---

### Story 4.8: Implement Changeset History Command
**Effort**: Medium  
**Priority**: Low

**As a** user  
**I want** to see changeset history  
**So that** I can track changes over time

**Description**:
Implement `wnt changeset history` to show timeline of changeset operations.

**Tasks**:
1. Create `src/commands/changeset/history.rs`
   - Define ChangesetHistoryArgs struct
   - Implement execute_changeset_history function
   - **Effort**: Low

2. Implement history loading
   - Load current changesets
   - Load archived changesets
   - Combine and sort by date
   - **Effort**: Medium

3. Implement timeline output
   - Show creation, modification, archival events
   - Use timeline-style formatting
   - Group by time periods
   - **Effort**: High

4. Write tests
   - Test with various history
   - Test output formatting
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Shows complete changeset timeline
- [ ] Includes archived changesets
- [ ] Timeline is easy to read
- [ ] Performance acceptable
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Command works
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 4.1, Story 4.7

**Blocked By**: Story 4.1, Story 4.7

**Blocks**: None

---

## Epic 5: Version Management Commands

**Phase**: 3  
**Total Effort**: Massive  
**Dependencies**: Epic 4  
**Goal**: Implement version bumping and release workflow

### Story 5.1: Implement Bump Command (Preview Mode)
**Effort**: High  
**Priority**: Critical

**As a** user  
**I want** to preview version bumps  
**So that** I can verify changes before applying them

**Description**:
Implement `wnt bump` in preview mode (default) to show what versions would be bumped.

**Tasks**:
1. Create `src/commands/bump/mod.rs`
   - Setup bump command router
   - Define BumpArgs struct
   - **Effort**: Low

2. Create `src/commands/bump/preview.rs`
   - Implement execute_bump_preview function
   - **Effort**: Low

3. Implement changeset loading
   - Load all active changesets
   - Filter by branch if specified
   - Validate changesets
   - **Effort**: Medium

4. Implement version calculation
   - Use sublime-package-tools for resolution
   - Calculate next versions
   - Handle dependency propagation
   - **Effort**: High

5. Create `src/commands/bump/report.rs`
   - Design bump report structure
   - Show packages with version changes
   - Show changesets being consumed
   - Show dependency updates
   - **Effort**: High

6. Implement table output
   - Create comprehensive bump table
   - Show current → next version
   - Show bump reason
   - Highlight major/minor/patch
   - **Effort**: Medium

7. Write comprehensive tests
   - Test with various changeset combinations
   - Test dependency propagation
   - Test version resolution
   - Test output formatting
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Shows all packages that would be bumped
- [ ] Version calculations correct
- [ ] Dependency propagation shown
- [ ] Changesets listed
- [ ] Table output clear and informative
- [ ] JSON output complete
- [ ] No files modified in preview mode
- [ ] Performance < 1s for 100 packages
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Preview mode works correctly
- [ ] All tests pass
- [ ] Documentation complete

**Dependencies**: Story 4.1

**Blocked By**: Story 4.1

**Blocks**: Story 5.2

---

### Story 5.2: Implement Bump Command (Execute Mode)
**Effort**: Massive  
**Priority**: Critical

**As a** user  
**I want** to execute version bumps  
**So that** I can release new versions

**Description**:
Implement `wnt bump --execute` to actually apply version bumps, update files, and create git operations.

**Tasks**:
1. Create `src/commands/bump/execute.rs`
   - Implement execute_bump_execute function
   - Add confirmation prompt
   - **Effort**: Low

2. Implement version application
   - Update package.json files
   - Use sublime-package-tools apply logic
   - Validate all updates
   - **Effort**: Medium

3. Implement changelog updates
   - Generate changelog entries
   - Update CHANGELOG.md files
   - Use sublime-package-tools changelog logic
   - **Effort**: High

4. Implement changeset archiving
   - Move changesets to archive
   - Maintain history
   - **Effort**: Low

5. Create `src/commands/bump/git_integration.rs`
   - Implement git staging (--git-commit)
   - Implement git commit creation
   - Implement git tagging (--git-tag)
   - Implement git push (--git-push)
   - Make all operations atomic
   - **Effort**: High

6. Implement rollback on failure
   - Detect failures at any step
   - Rollback all changes
   - Restore changesets
   - Clear git operations
   - **Effort**: High

7. Implement dry-run validation
   - Validate all operations before executing
   - Check file permissions
   - Check git state
   - **Effort**: Medium

8. Write comprehensive tests
   - Test full execution flow
   - Test git operations
   - Test rollback scenarios
   - Test error conditions
   - Mock filesystem and git
   - **Effort**: Massive

**Acceptance Criteria**:
- [ ] Updates all package.json files correctly
- [ ] Updates CHANGELOG.md files correctly
- [ ] Archives changesets
- [ ] Git commit created if flag set
- [ ] Git tags created if flag set
- [ ] Git push executes if flag set
- [ ] Rollback works on any failure
- [ ] Confirmation prompt prevents accidents
- [ ] All operations atomic
- [ ] Performance < 2s for 100 packages
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Execute mode works end-to-end
- [ ] Rollback tested and working
- [ ] All tests pass
- [ ] Documentation complete
- [ ] User guide updated

**Dependencies**: Story 5.1

**Blocked By**: Story 5.1

**Blocks**: None

---

### Story 5.3: Implement Snapshot Version Command
**Effort**: Medium  
**Priority**: Low

**As a** user  
**I want** to generate snapshot versions  
**So that** I can test unreleased changes

**Description**:
Implement snapshot version generation for testing purposes.

**Tasks**:
1. Extend BumpArgs
   - Add snapshot flag
   - Add snapshot suffix option
   - **Effort**: Minimal

2. Implement snapshot logic
   - Generate snapshot versions (e.g., 1.2.3-snapshot.abc123)
   - Use git commit hash in suffix
   - Don't consume changesets
   - **Effort**: Medium

3. Write tests
   - Test snapshot generation
   - Test suffix format
   - Test that changesets not consumed
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Generates valid snapshot versions
- [ ] Includes git commit hash
- [ ] Doesn't consume changesets
- [ ] Can be used for testing
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Snapshot mode works
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 5.1

**Blocked By**: Story 5.1

**Blocks**: None

---

### Story 5.4: Implement Changes Command
**Effort**: High  
**Priority**: High

**As a** user  
**I want** to analyze changes in my workspace  
**So that** I can understand what needs version bumps

**Description**:
Implement `wnt changes` to analyze working directory or commit range changes.

**Tasks**:
1. Create `src/commands/changes.rs`
   - Define ChangesArgs struct
   - Add working dir and commit range options
   - Implement execute_changes function
   - **Effort**: Low

2. Implement working directory analysis
   - Get changed files from git
   - Map files to packages
   - Use sublime-package-tools logic
   - **Effort**: High

3. Implement commit range analysis
   - Parse commit range
   - Get commits in range
   - Analyze changed files
   - Map to packages
   - **Effort**: High

4. Implement impact calculation
   - Show direct changes
   - Show dependency impact
   - Suggest version bumps
   - **Effort**: High

5. Implement output
   - Show affected packages in table
   - Show file changes per package
   - Show suggested bump types
   - **Effort**: Medium

6. Write tests
   - Test working directory analysis
   - Test commit range analysis
   - Test package detection
   - Test impact calculation
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Detects affected packages accurately
- [ ] Working directory mode works
- [ ] Commit range mode works
- [ ] Impact calculation correct
- [ ] Suggestions helpful
- [ ] Performance < 500ms
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Command works correctly
- [ ] All modes tested
- [ ] Documentation complete

**Dependencies**: Story 1.4, Story 3.1

**Blocked By**: Story 1.4, Story 3.1

**Blocks**: None

---

## Epic 6: Upgrade Commands

**Phase**: 3  
**Total Effort**: High  
**Dependencies**: Epic 5  
**Goal**: Implement dependency upgrade management

### Story 6.1: Implement Upgrade Check Command
**Effort**: High  
**Priority**: High

**As a** user  
**I want** to check for dependency upgrades  
**So that** I can keep dependencies up to date

**Description**:
Implement `wnt upgrade check` to detect available dependency upgrades.

**Tasks**:
1. Create `src/commands/upgrade/mod.rs`
   - Setup upgrade command router
   - Define UpgradeArgs enum
   - **Effort**: Low

2. Create `src/commands/upgrade/check.rs`
   - Define UpgradeCheckArgs struct
   - Implement execute_upgrade_check function
   - **Effort**: Low

3. Implement upgrade detection
   - Use sublime-package-tools upgrade logic
   - Query npm registry
   - Detect available upgrades
   - Categorize by type (major, minor, patch)
   - **Effort**: High

4. Implement filtering
   - Filter by dependency type (prod, dev, peer)
   - Filter by upgrade type (major, minor, patch)
   - Filter by ignore patterns
   - **Effort**: Medium

5. Implement upgrade table output
   - Show package name
   - Show current version
   - Show available version
   - Show upgrade type
   - Highlight breaking changes
   - **Effort**: Medium

6. Write tests
   - Test upgrade detection
   - Test filtering
   - Test output formatting
   - Mock registry responses
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Detects all available upgrades
- [ ] Categorizes correctly
- [ ] Filters work as expected
- [ ] Table output informative
- [ ] Breaking changes highlighted
- [ ] Performance < 2s for 100 packages
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Command works correctly
- [ ] All tests pass
- [ ] Documentation complete

**Dependencies**: Story 3.2, Story 3.3

**Blocked By**: Story 3.2, Story 3.3

**Blocks**: Story 6.2

---

### Story 6.2: Implement Upgrade Apply Command
**Effort**: High  
**Priority**: High

**As a** user  
**I want** to apply dependency upgrades  
**So that** I can update my dependencies safely

**Description**:
Implement `wnt upgrade apply` to apply selected upgrades with backup.

**Tasks**:
1. Create `src/commands/upgrade/apply.rs`
   - Define UpgradeApplyArgs struct
   - Add filter options
   - Add changeset option
   - Implement execute_upgrade_apply function
   - **Effort**: Low

2. Implement upgrade application
   - Use sublime-package-tools upgrade logic
   - Update package.json files
   - Validate updates
   - **Effort**: High

3. Implement backup creation
   - Backup all package.json files before changes
   - Store in .wnt/backups/
   - Include timestamp
   - **Effort**: Medium

4. Implement changeset creation
   - Optionally create changeset for upgrades
   - Group by package
   - Include upgrade details
   - **Effort**: Medium

5. Implement post-upgrade validation
   - Verify package.json validity
   - Check for conflicts
   - **Effort**: Medium

6. Write tests
   - Test upgrade application
   - Test backup creation
   - Test changeset creation
   - Test validation
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Applies upgrades correctly
- [ ] Creates backup before changes
- [ ] Optionally creates changeset
- [ ] Validates after application
- [ ] Performance acceptable
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Command works correctly
- [ ] All tests pass
- [ ] Documentation complete

**Dependencies**: Story 6.1

**Blocked By**: Story 6.1

**Blocks**: Story 6.3

---

### Story 6.3: Implement Upgrade Rollback Command
**Effort**: Medium  
**Priority**: Medium

**As a** user  
**I want** to rollback failed upgrades  
**So that** I can recover from upgrade issues

**Description**:
Implement `wnt upgrade rollback` to restore from backup.

**Tasks**:
1. Create `src/commands/upgrade/rollback.rs`
   - Define UpgradeRollbackArgs struct
   - Implement execute_upgrade_rollback function
   - **Effort**: Low

2. Implement backup listing
   - List available backups
   - Show timestamps
   - Allow selection
   - **Effort**: Low

3. Implement restore logic
   - Restore from selected backup
   - Validate restored files
   - Show diff of changes
   - **Effort**: Medium

4. Write tests
   - Test backup listing
   - Test restore
   - Test validation
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Lists available backups
- [ ] Restores correctly
- [ ] Validates after restore
- [ ] Shows what was changed
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Command works
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 6.2

**Blocked By**: Story 6.2

**Blocks**: None

---

## Epic 7: Audit Commands

**Phase**: 4  
**Total Effort**: High  
**Dependencies**: Epic 6  
**Goal**: Implement comprehensive project health auditing

### Story 7.1: Implement Comprehensive Audit Command
**Effort**: High  
**Priority**: High

**As a** user  
**I want** to audit my project health  
**So that** I can identify and fix issues

**Description**:
Implement `wnt audit` to run comprehensive health checks.

**Tasks**:
1. Create `src/commands/audit/mod.rs`
   - Setup audit command router
   - Define AuditArgs struct
   - **Effort**: Low

2. Create `src/commands/audit/comprehensive.rs`
   - Implement execute_audit function
   - Coordinate all audit checks
   - **Effort**: Medium

3. Implement audit orchestration
   - Run all enabled audits
   - Collect results
   - Calculate overall score
   - Use sublime-package-tools audit logic
   - **Effort**: High

4. Create `src/commands/audit/report.rs`
   - Design comprehensive report format
   - Show audit sections
   - Show health score
   - Show recommendations
   - **Effort**: High

5. Write tests
   - Test each audit type
   - Test report generation
   - Test scoring
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Runs all audit checks
- [ ] Report is comprehensive and clear
- [ ] Health score meaningful
- [ ] Recommendations actionable
- [ ] Performance < 3s for 100 packages
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Command works correctly
- [ ] All tests pass
- [ ] Documentation complete

**Dependencies**: Story 3.2, Story 3.3

**Blocked By**: Story 3.2, Story 3.3

**Blocks**: Story 7.2, Story 7.3, Story 7.4, Story 7.5

---

### Story 7.2: Implement Upgrade Audit
**Effort**: Medium  
**Priority**: Medium

**As a** user  
**I want** to audit available upgrades  
**So that** I know which dependencies are outdated

**Description**:
Implement upgrade-specific audit section.

**Tasks**:
1. Create `src/commands/audit/upgrades.rs`
   - Implement upgrade audit logic
   - Categorize outdated dependencies
   - **Effort**: Medium

2. Implement output
   - Show outdated count by type
   - Show critical upgrades
   - **Effort**: Low

3. Write tests
   - Test audit logic
   - Test categorization
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Identifies outdated dependencies
- [ ] Categorizes by severity
- [ ] Output clear
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Audit works
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 7.1

**Blocked By**: Story 7.1

**Blocks**: None

---

### Story 7.3: Implement Dependency Audit
**Effort**: Medium  
**Priority**: Medium

**As a** user  
**I want** to audit dependency health  
**So that** I can understand dependency risks

**Description**:
Implement dependency-specific audit checks.

**Tasks**:
1. Create `src/commands/audit/dependencies.rs`
   - Implement dependency analysis
   - Check for duplicates
   - Check for version mismatches
   - Categorize by type
   - **Effort**: High

2. Write tests
   - Test duplicate detection
   - Test mismatch detection
   - Test categorization
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Detects duplicate dependencies
- [ ] Detects version mismatches
- [ ] Categories dependencies
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Audit works
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 7.1

**Blocked By**: Story 7.1

**Blocks**: None

---

### Story 7.4: Implement Version Consistency Audit
**Effort**: Low  
**Priority**: Medium

**As a** user  
**I want** to check version consistency  
**So that** I can find version conflicts

**Description**:
Implement version consistency checks.

**Tasks**:
1. Create `src/commands/audit/versions.rs`
   - Check internal dependency versions
   - Check for conflicts
   - **Effort**: Medium

2. Write tests
   - Test consistency checks
   - Test conflict detection
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Detects version inconsistencies
- [ ] Shows conflicts clearly
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Audit works
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 7.1

**Blocked By**: Story 7.1

**Blocks**: None

---

### Story 7.5: Implement Breaking Changes Audit
**Effort**: Medium  
**Priority**: Low

**As a** user  
**I want** to check for potential breaking changes  
**So that** I can plan releases carefully

**Description**:
Implement breaking changes detection.

**Tasks**:
1. Create `src/commands/audit/breaking.rs`
   - Analyze major version bumps
   - Check for API changes
   - **Effort**: High

2. Write tests
   - Test breaking change detection
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Identifies potential breaking changes
- [ ] Clear warnings
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Audit works
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 7.1

**Blocked By**: Story 7.1

**Blocks**: None

---

## Epic 8: Advanced Features

**Phase**: 4  
**Total Effort**: Medium  
**Dependencies**: Epic 7  
**Goal**: Implement interactive enhancements and advanced output

### Story 8.1: Implement Enhanced Interactive Prompts
**Effort**: High  
**Priority**: Medium

**As a** user  
**I want** polished interactive experiences  
**So that** the CLI is pleasant to use

**Description**:
Enhance interactive prompts with better UX, validation, and features.

**Tasks**:
1. Enhance `src/interactive/select.rs`
   - Implement fuzzy search for package selection
   - Add multi-select improvements
   - Add visual indicators
   - **Effort**: High

2. Implement prompt validation
   - Real-time validation
   - Helpful error messages
   - Suggestions on errors
   - **Effort**: Medium

3. Add prompt themes
   - Consistent styling
   - Color-coded elements
   - Icons and emojis
   - **Effort**: Low

4. Write tests
   - Test prompt flows
   - Test validation
   - Mock user input
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Fuzzy search works smoothly
- [ ] Validation is helpful
- [ ] Prompts are visually appealing
- [ ] Fast and responsive
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Enhancements implemented
- [ ] Tests pass
- [ ] User feedback positive

**Dependencies**: Story 4.1

**Blocked By**: Story 4.1

**Blocks**: None

---

### Story 8.2: Implement Diff Visualization
**Effort**: Medium  
**Priority**: Low

**As a** user  
**I want** to see diffs of changes  
**So that** I can understand what will be modified

**Description**:
Add diff visualization for version changes and file modifications.

**Tasks**:
1. Create `src/output/diff.rs`
   - Implement diff formatting
   - Add color coding
   - Support various diff types
   - **Effort**: High

2. Integrate into bump preview
   - Show version diffs
   - Show file diffs
   - **Effort**: Medium

3. Write tests
   - Test diff generation
   - Test formatting
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Diffs are clear and readable
- [ ] Colors enhance understanding
- [ ] Integration works smoothly
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Diff visualization works
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 5.1

**Blocked By**: Story 5.1

**Blocks**: None

---

### Story 8.3: Implement Export Formats
**Effort**: Low  
**Priority**: Low

**As a** user  
**I want** to export reports in various formats  
**So that** I can use them in other tools

**Description**:
Add export capabilities for reports (HTML, Markdown, etc.).

**Tasks**:
1. Create `src/output/export.rs`
   - Implement HTML export
   - Implement Markdown export
   - **Effort**: Medium

2. Add export flags to commands
   - Add --export flag
   - Support multiple formats
   - **Effort**: Low

3. Write tests
   - Test each export format
   - Validate output
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] HTML export works
- [ ] Markdown export works
- [ ] Exports are well-formatted
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Export feature works
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 7.1

**Blocked By**: Story 7.1

**Blocks**: None

---

## Epic 9: Distribution

**Phase**: 5  
**Total Effort**: High  
**Dependencies**: All previous epics  
**Goal**: Package and distribute the CLI tool

### Story 9.1: Create Installation Script
**Effort**: High  
**Priority**: Critical

**As a** user  
**I want** an easy installation method  
**So that** I can get started quickly

**Description**:
Create curl-based installation script for all platforms.

**Tasks**:
1. Create `install.sh`
   - Detect platform (macOS, Linux, Windows)
   - Detect architecture
   - Download appropriate binary
   - Verify checksum
   - Install to appropriate location
   - Setup shell completions
   - **Effort**: High

2. Implement platform detection
   - Handle macOS (x86_64, aarch64)
   - Handle Linux (various distros)
   - Handle Windows (via WSL or native)
   - **Effort**: Medium

3. Implement checksum verification
   - Download checksums file
   - Verify binary integrity
   - Fail on mismatch
   - **Effort**: Low

4. Implement installation paths
   - Try /usr/local/bin (with sudo)
   - Fallback to ~/.local/bin
   - Update PATH if needed
   - **Effort**: Medium

5. Create uninstall script
   - Remove binary
   - Remove completions
   - Clean up configuration (optional)
   - **Effort**: Low

6. Write installation tests
   - Test on clean systems (Docker)
   - Test all platforms
   - Test error scenarios
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Works on macOS (Intel and Apple Silicon)
- [ ] Works on Linux (Ubuntu, Debian, Fedora, etc.)
- [ ] Works via WSL on Windows
- [ ] Checksum verification prevents tampering
- [ ] Installation path detection works
- [ ] Shell completions installed
- [ ] Clear error messages
- [ ] Uninstall script works
- [ ] Tested on clean systems

**Definition of Done**:
- [ ] Installation script works on all platforms
- [ ] All tests pass
- [ ] Documentation complete
- [ ] Installation guide published

**Dependencies**: All previous stories

**Blocked By**: All previous stories

**Blocks**: Story 9.2

---

### Story 9.2: Setup Release Automation
**Effort**: High  
**Priority**: Critical

**As a** maintainer  
**I want** automated releases  
**So that** I can publish new versions easily

**Description**:
Setup GitHub Actions workflow for automated releases with multi-platform binaries.

**Tasks**:
1. Enhance `.github/workflows/release.yml`
   - Trigger on version tags
   - Build matrix for all platforms
   - Cross-compilation setup
   - **Effort**: High

2. Implement binary optimization
   - Strip symbols
   - Optimize size
   - Compress binaries
   - **Effort**: Medium

3. Implement release asset creation
   - Create archives (.tar.gz, .zip)
   - Generate checksums
   - Create release notes from changelog
   - **Effort**: Medium

4. Implement crates.io publishing
   - Automated publish on release
   - Version verification
   - **Effort**: Low

5. Test release workflow
   - Test on pre-release tags
   - Verify all assets created
   - Verify checksums
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Release triggered by version tags
- [ ] Binaries built for all platforms
- [ ] Assets uploaded to GitHub releases
- [ ] Checksums generated correctly
- [ ] Published to crates.io
- [ ] Release notes auto-generated
- [ ] All assets verified

**Definition of Done**:
- [ ] Release workflow works
- [ ] Test release successful
- [ ] Documentation complete

**Dependencies**: Story 9.1

**Blocked By**: Story 9.1

**Blocks**: Story 9.3

---

### Story 9.3: Create Homebrew Formula
**Effort**: Medium  
**Priority**: Medium

**As a** macOS user  
**I want** to install via Homebrew  
**So that** I can manage wnt like other tools

**Description**:
Create and maintain Homebrew formula for easy installation on macOS.

**Tasks**:
1. Create Homebrew formula
   - Define formula structure
   - Add download URLs
   - Add checksums
   - Test formula
   - **Effort**: Medium

2. Setup tap repository
   - Create homebrew-wnt repository
   - Setup auto-updates on release
   - **Effort**: Low

3. Write installation instructions
   - Document tap addition
   - Document installation
   - Document updates
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Formula installs correctly
- [ ] Updates work via brew upgrade
- [ ] Documentation clear
- [ ] Auto-updates on release

**Definition of Done**:
- [ ] Formula published
- [ ] Installation tested
- [ ] Documentation complete

**Dependencies**: Story 9.2

**Blocked By**: Story 9.2

**Blocks**: None

---

### Story 9.4: Implement Self-Update Command
**Effort**: Medium  
**Priority**: Low

**As a** user  
**I want** to update wnt itself  
**So that** I can get new features easily

**Description**:
Implement `wnt upgrade-self` to update the CLI binary.

**Tasks**:
1. Create `src/commands/upgrade_self.rs`
   - Implement version checking
   - Download new binary
   - Replace current binary
   - Handle permissions
   - **Effort**: High

2. Implement platform-specific logic
   - macOS binary replacement
   - Linux binary replacement
   - Windows via WSL
   - **Effort**: High

3. Implement rollback on failure
   - Backup current binary
   - Restore on failure
   - **Effort**: Medium

4. Write tests
   - Test version checking
   - Test download
   - Mock binary replacement
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Checks for new versions
- [ ] Downloads and verifies new binary
- [ ] Replaces current binary
- [ ] Handles permissions correctly
- [ ] Rollback works on failure
- [ ] Works on all platforms
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Self-update works
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 9.2

**Blocked By**: Story 9.2

**Blocks**: None

---

## Epic 10: Documentation & Examples

**Phase**: 5  
**Total Effort**: High  
**Dependencies**: All previous epics  
**Goal**: Create comprehensive documentation and examples

### Story 10.1: Create User Guide
**Effort**: High  
**Priority**: Critical

**As a** user  
**I want** comprehensive documentation  
**So that** I can learn to use wnt effectively

**Description**:
Create a complete user guide covering all features and workflows.

**Tasks**:
1. Create `docs/GUIDE.md`
   - Getting started section
   - Installation instructions
   - Configuration guide
   - Command reference
   - Best practices
   - **Effort**: Massive

2. Create workflow guides
   - Basic workflow guide
   - Advanced workflow guide
   - CI/CD integration guide
   - Migration guide
   - **Effort**: High

3. Add troubleshooting section
   - Common issues
   - Solutions
   - FAQ
   - **Effort**: Medium

4. Create visual aids
   - Command flow diagrams
   - Screenshots
   - Terminal recordings
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Guide is comprehensive
- [ ] All features documented
- [ ] Examples are clear
- [ ] Troubleshooting helpful
- [ ] Visuals enhance understanding

**Definition of Done**:
- [ ] Guide complete
- [ ] Reviewed and approved
- [ ] Published

**Dependencies**: All command stories

**Blocked By**: All command stories

**Blocks**: None

---

### Story 10.2: Create Command Reference
**Effort**: Medium  
**Priority**: High

**As a** user  
**I want** detailed command reference  
**So that** I can quickly look up command usage

**Description**:
Create comprehensive command reference documentation.

**Tasks**:
1. Create `docs/COMMANDS.md`
   - Document each command
   - Include all flags and options
   - Add examples for each
   - Show output examples
   - **Effort**: High

2. Generate from help text
   - Script to extract help text
   - Format into markdown
   - Keep in sync with code
   - **Effort**: Medium

3. Add quick reference
   - Command cheat sheet
   - Common patterns
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] All commands documented
- [ ] Examples working
- [ ] Reference is searchable
- [ ] Kept in sync with code

**Definition of Done**:
- [ ] Reference complete
- [ ] Examples verified
- [ ] Published

**Dependencies**: All command stories

**Blocked By**: All command stories

**Blocks**: None

---

### Story 10.3: Create Example Projects
**Effort**: Medium  
**Priority**: High

**As a** user  
**I want** example projects  
**So that** I can see wnt in action

**Description**:
Create example projects demonstrating various use cases.

**Tasks**:
1. Create basic monorepo example
   - Simple workspace
   - Basic configuration
   - Example workflow
   - **Effort**: Medium

2. Create advanced monorepo example
   - Complex workspace
   - Advanced configuration
   - Multiple workflows
   - CI/CD integration
   - **Effort**: High

3. Create single-package example
   - Simple package
   - Basic versioning
   - **Effort**: Low

4. Document each example
   - README for each
   - Step-by-step instructions
   - Expected outcomes
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Examples represent real use cases
- [ ] All examples work correctly
- [ ] Documentation clear
- [ ] Cover different scenarios

**Definition of Done**:
- [ ] Examples created
- [ ] Documentation complete
- [ ] Verified working

**Dependencies**: All command stories

**Blocked By**: All command stories

**Blocks**: None

---

### Story 10.4: Create Migration Guide
**Effort**: Medium  
**Priority**: Medium

**As a** user migrating from another tool  
**I want** a migration guide  
**So that** I can switch to wnt easily

**Description**:
Create guides for migrating from other tools (Changesets, Lerna, etc.).

**Tasks**:
1. Create `docs/MIGRATION.md`
   - Migration from Changesets
   - Migration from Lerna
   - Migration from manual versioning
   - **Effort**: High

2. Create migration scripts
   - Script to convert Changesets format
   - Script to convert Lerna config
   - **Effort**: Medium

3. Document differences
   - Feature comparison
   - Workflow differences
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Covers major tools
- [ ] Migration paths clear
- [ ] Scripts work correctly
- [ ] Differences explained

**Definition of Done**:
- [ ] Migration guide complete
- [ ] Scripts tested
- [ ] Published

**Dependencies**: Story 10.1

**Blocked By**: Story 10.1

**Blocks**: None

---

### Story 10.5: Update README
**Effort**: Low  
**Priority**: High

**As a** potential user  
**I want** a clear README  
**So that** I can quickly understand what wnt does

**Description**:
Create an comprehensive README with quick start and links to detailed docs.

**Tasks**:
1. Update `README.md`
   - Project overview
   - Key features
   - Quick start
   - Installation instructions
   - Basic usage examples
   - Links to detailed docs
   - **Effort**: Medium

2. Add badges
   - CI status
   - Coverage
   - Version
   - License
   - **Effort**: Minimal

3. Add screenshots/demos
   - Terminal recordings
   - Example outputs
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] README is compelling
- [ ] Quick start works
- [ ] Links are correct
- [ ] Badges accurate

**Definition of Done**:
- [ ] README complete
- [ ] Reviewed and approved
- [ ] Published

**Dependencies**: Story 10.1, Story 10.2

**Blocked By**: Story 10.1, Story 10.2

**Blocks**: None

---

## Summary

### Total Story Count
- **Epics**: 10
- **User Stories**: 78
- **Total Tasks**: 450+

### Effort Distribution

| Epic | Total Effort | Story Count |
|------|--------------|-------------|
| Epic 1: CLI Foundation | High | 4 |
| Epic 2: Configuration Commands | High | 3 |
| Epic 3: Output System | High | 4 |
| Epic 4: Changeset Commands | Massive | 8 |
| Epic 5: Version Management Commands | Massive | 4 |
| Epic 6: Upgrade Commands | High | 3 |
| Epic 7: Audit Commands | High | 5 |
| Epic 8: Advanced Features | Medium | 3 |
| Epic 9: Distribution | High | 4 |
| Epic 10: Documentation & Examples | High | 5 |

### Critical Path

The critical path for development (must be completed in order):

```
Story 1.1 (Initialize Structure)
    ↓
Story 1.3 (Error Handling)
    ↓
Story 1.4 (CLI Framework)
    ↓
Story 2.1 (Init Command)
    ↓
Story 3.1 (Output Framework)
    ↓
Story 3.2 (Table Rendering)
    ↓
Story 4.1 (Changeset Add - Interactive)
    ↓
Story 4.3 (Changeset List)
    ↓
Story 5.1 (Bump Preview)
    ↓
Story 5.2 (Bump Execute)
    ↓
Story 9.1 (Installation Script)
    ↓
Story 9.2 (Release Automation)
    ↓
Story 10.1 (User Guide)
```

**Total Critical Path Effort**: ~8-10 weeks

### Parallel Work Opportunities

Stories that can be worked on in parallel (after critical path dependencies met):

**Phase 1 (Weeks 1-3)**:
- Story 1.2 (CI/CD) can run parallel to Story 1.3
- Story 3.3 (Progress Indicators) can run parallel to Story 3.2
- Story 3.4 (Logging) can run parallel to Story 3.1

**Phase 2 (Weeks 4-7)**:
- Story 4.2-4.8 (Other changeset commands) can largely run in parallel after Story 4.1
- Story 5.4 (Changes Command) can run parallel to Story 5.1

**Phase 3 (Weeks 8-11)**:
- Story 6.1-6.3 (Upgrade commands) can run in parallel after dependencies met
- Story 5.3 (Snapshot) can run parallel to Story 5.2

**Phase 4 (Weeks 12-14)**:
- Story 7.2-7.5 (Specific audits) can run parallel after Story 7.1
- Story 8.1-8.3 (Advanced features) can run parallel

**Phase 5 (Weeks 15-16)**:
- Story 9.3-9.4 (Distribution extras) can run parallel to Story 9.2
- Story 10.1-10.5 (Documentation) can largely run in parallel

### Estimated Timeline

**Optimistic (with parallel work)**: 10 weeks  
**Realistic (with some parallel work)**: 12-13 weeks  
**Pessimistic (mostly sequential)**: 15-16 weeks

### Quality Gates

Each story must pass these gates before being considered "Done":

1. **Code Complete**: All tasks implemented
2. **Tests Pass**: 100% test coverage, all tests passing
3. **Clippy Clean**: No clippy warnings
4. **Documentation**: All public APIs and commands documented
5. **Review Approved**: Code review completed and approved
6. **Integration Tested**: Works with related features
7. **Performance Verified**: Meets performance requirements from PRD

---

## How to Use This Story Map

### For Planning

1. Review epic breakdown to understand project phases
2. Identify critical path stories for timeline planning
3. Look for parallel work opportunities to optimize schedule
4. Use effort estimates for team capacity planning
5. Review dependencies to avoid blocking situations

### For Development

1. Start with Story 1.1 (Initialize Structure)
2. Follow the critical path for essential features
3. Pick parallel stories when critical path is blocked
4. Complete all tasks within a story before marking it done
5. Verify acceptance criteria and definition of done
6. Update "Verify all the code" checklist for each story

### For Review

1. Check that all tasks are completed
2. Verify acceptance criteria are met
3. Confirm definition of done satisfied
4. Review test coverage and quality
5. Validate documentation completeness
6. Check for any remaining TODOs or technical debt

### For Tracking Progress

Use the checkbox format to track completion:
- [ ] Task not started
- [x] Task completed

Track story status:
- 🔴 Not Started
- 🟡 In Progress
- 🟢 Completed
- ⚪ Blocked

---

**Ready to Begin Development** 🚀

Start with Epic 1, Story 1.1: Initialize CLI Crate Structure