# Changelog

All notable changes to this project will be documented in this file.

## 0.0.1 - 2025-11-10

### Bug Fixes

#### WOR-TSK-100

- Correct doctest examples to pass args by reference

#### WOR-TSK-108

- Fix doctest imports and enhance dependency propagation output

#### WOR-TSK-110

- Fix sanitize_branch_name doctest for pub(crate) visibility

#### WOR-TSK-115

- Address story 7.1 review findings

#### WOR-TSK-120

- Correct doctest example type in confirm_with_details

#### WOR-TSK-121

- Add show_diff field to integration test BumpArgs
- Add show_diff field to all BumpArgs doctests

#### WOR-TSK-122

- Implement --output flag functionality for all audit commands

#### WOR-TSK-123

- Implement workspace package discovery in changeset create command

#### WOR-TSK-124

- Resolve clippy ptr_arg warning in changeset add
- Resolve doctest failure for detect_affected_packages
- Corrigir doctest falhado em logger.rs

#### WOR-TSK-127

- Correct bump command documentation to match implementation

#### WOR-TSK-130

- Resolve git commit path handling for absolute vs relative paths
- Implement changelog file writing and independent git repo handling
- Fix TOML config validation test
- Fix upgrade backup clean test fixtures
- Fix Windows CI clippy error in test helpers
- Fix Linux CI - changes branch comparison test

#### WOR-TSK-90

- Resolve all clippy errors and warnings across workspace

#### Cli

- Correct directory structure and git versioning strategy
- Fix monorepo detection and workspace pattern extraction

### Documentation

#### WOR-TSK-124

- Document automated release process
- Fix all rustdoc warnings and errors

#### WOR-TSK-126

- Create comprehensive user guide with complete configuration reference

#### WOR-TSK-127

- Create comprehensive command reference documentation
- Add documentation scripts usage guide

#### WOR-TSK-129

- Update README files with comprehensive project documentation

#### WOR-TSK-99

- Add comprehensive logging example
- Add story 3.4 completion summary

#### Cli

- Add comprehensive planning documentation
- Clarify version bump behavior and global options independence

### Features

#### WOR-TSK-100

- Implement changeset add command with interactive mode

#### WOR-TSK-102

- Implement changeset list command with filtering and sorting

#### WOR-TSK-103

- Implement changeset show command

#### WOR-TSK-104

- Implement changeset update command

#### WOR-TSK-105

- Implement changeset edit command

#### WOR-TSK-106

- Implement changeset remove command with archiving and confirmation

#### WOR-TSK-107

- Implement changeset history command with filtering

#### WOR-TSK-108

- Integrate bump preview command into CLI dispatcher

#### WOR-TSK-109

- Implement bump command execute mode

#### WOR-TSK-110

- Implement snapshot version command for testing unreleased changes

#### WOR-TSK-111

- Implement changes command for workspace analysis

#### WOR-TSK-112

- Implement upgrade check command

#### WOR-TSK-113

- Implement upgrade apply command

#### WOR-TSK-114

- Implement upgrade rollback command

#### WOR-TSK-115

- Implement comprehensive audit command

#### WOR-TSK-116

- Implement upgrade audit command

#### WOR-TSK-117

- Implement dependency audit command

#### WOR-TSK-118

- Implement version consistency audit command

#### WOR-TSK-119

- Implement breaking changes audit

#### WOR-TSK-120

- Implement enhanced interactive prompts

#### WOR-TSK-121

- Implement diff visualization for version changes and file modifications
- Integrate diff visualization into bump preview

#### WOR-TSK-122

- Implement export formats for audit reports

#### WOR-TSK-124

- Implement automatic package detection in changeset create

#### WOR-TSK-127

- Add command documentation generation scripts

#### WOR-TSK-130

- Implement changeset check command (story 4.3)

#### WOR-TSK-92

- Implement CLI framework with Clap

#### WOR-TSK-93

- Implement init command with workspace detection and configuration

#### WOR-TSK-94

- Implement config show command

#### WOR-TSK-95

- Implement config validate command with comprehensive validation

#### WOR-TSK-96

- Implement output format framework

#### WOR-TSK-97

- Implement table rendering with comfy-table

#### WOR-TSK-98

- Implement progress indicators for long-running operations

#### WOR-TSK-99

- Implement version command for story 1.4
- Add modern CLI branding with ASCII art and styled output
- Implement logging system with tracing
- Add PartialOrd and Ord to LogLevel enum
- Integrate logging initialization in main

### Miscellaneous Tasks

#### WOR-TSK-99

- Remove story completion summary document

### Refactor

#### WOR-TSK-106

- Eliminate code duplication in changeset commands

#### WOR-TSK-125

- Rename binary from wnt to workspace

#### WOR-TSK-91

- Move inline tests to tests.rs and fix doctests

### Styling

#### WOR-TSK-125

- Fix code formatting issues

#### WOR-TSK-130

- Apply cargo fmt formatting to E2E tests

### Testing

#### WOR-TSK-101

- Add comprehensive tests for changeset add non-interactive mode

#### WOR-TSK-109

- Add comprehensive integration tests for bump execute mode

#### WOR-TSK-117

- Add comprehensive tests for dependency audit

#### WOR-TSK-130

- Add E2E test infrastructure with fixtures, assertions and helpers
- Add e2e tests for init command (12 tests)
- Add comprehensive E2E tests for changeset commands
- Improve workspace fixtures for monorepo E2E tests
- Implement comprehensive E2E tests for bump commands (Phase 2.3)
- Implement E2E tests for upgrade commands (Phase 2.4)
- Implement comprehensive E2E tests for audit commands (Phase 2.5)
- Implement Phase 2.6 E2E tests for config commands
- Implement Phase 2.7 E2E tests for version command
- Implement E2E tests for changes command
- Implement 29 additional E2E tests and fix all clippy warnings
- Fix Phase 3 test infrastructure bugs (7 tests)

#### WOR-TSK-99

- Add comprehensive tests for logging system

#### Cli

- Add edge case tests for init command workspace handling

### Ci

#### WOR-TSK-131

- Repo configs and tools ([#1](https://github.com/websublime/workspace-tools/pull/1))

<!-- Made with ❤️ by WebSublime -->
