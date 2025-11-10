# Changelog

All notable changes to this project will be documented in this file.

## 0.0.1 - 2025-11-10

### Bug Fixes

#### WOR-TSK-109

- Fix unified strategy to bump all workspace packages
- Correct unified strategy test assertion to expect version 3.0.0

#### WOR-TSK-11

- Complete documentation and test alignment for pkg tools foundation

#### WOR-TSK-123

- Correct backup directory default from .pkg-backups to .wnt-backups

#### WOR-TSK-19

- Resolve doctest type inference failures in ChangeEntry methods

#### WOR-TSK-34

- Fix MockFileSystem path operations for Windows compatibility

#### WOR-TSK-35

- Resolve doctest compilation errors on CI

#### WOR-TSK-38

- Correct documentation example in audit error module

#### WOR-TSK-40

- Make manager mutable in recovery doctest example

#### WOR-TSK-42

- Fix doc examples to use correct PackageJson API

#### WOR-TSK-48

- Correct doctest imports to use public API path

#### WOR-TSK-52

- Handle Windows extended-length path prefix in test

#### WOR-TSK-56

- Correct changeset git integration test failures and bugs

#### WOR-TSK-59

- Make normalize_path test cross-platform compatible

#### WOR-TSK-75

- Make NpmrcConfig public and fix doctest compilation errors

#### WOR-TSK-77

- Change application module visibility from pub(crate) to private
- Remove doctest from internal preserve_version_prefix function

#### WOR-TSK-78

- Add milliseconds to backup ID to ensure uniqueness
- Normalize path separators for cross-platform compatibility
- Use normalized paths as HashMap keys for Windows compatibility
- Improve path normalization and directory existence checks for Windows
- Use relative paths in tests for cross-platform Windows compatibility
- Fix backup module visibility and doctest imports

#### WOR-TSK-83

- Correct doctest imports to use public audit module path

#### WOR-TSK-90

- Resolve all clippy errors and warnings across workspace

#### Pkg

- Update internal references to use direct type names
- Consolidate all tests into single tests.rs file per CLAUDE.md standards
- Resolve module export and import issues
- Resolve test compilation and monorepo scaling issues

### Documentation

#### WOR-TSK-10

- Enhance standard crate documentation with comprehensive examples and API specification

#### WOR-TSK-12

- Add comprehensive AsRef<str> implementation documentation

#### WOR-TSK-124

- Fix all rustdoc warnings and errors

#### WOR-TSK-14

- Add comprehensive VersionResolver usage example
- Add comprehensive documentation for Story 1.4 implementation

#### WOR-TSK-16

- Complete review analysis of package.json operations implementation
- Fix failing doctests in package module

#### WOR-TSK-18

- Add practical examples demonstrating dependency graph usage
- Add comprehensive documentation for dependency graph system

#### WOR-TSK-21

- Add changeset builder architecture documentation

#### WOR-TSK-34

- Add implementation summary for Story 1.3

#### WOR-TSK-37

- Add comprehensive configuration documentation and examples

#### WOR-TSK-48

- Add TODO verification item to all Definition of Done sections

#### WOR-TSK-52

- Add implementation audit report
- Add comprehensive integration test analysis for Epics 1-5

#### WOR-TSK-54

- Add comprehensive example for FileBasedChangesetStorage

#### WOR-TSK-75

- Mark Story 9.2 acceptance criteria as complete

#### Pkg

- Update consolidation plan with Phase 1 completion status
- Clarify graph module responsibilities and separation
- Update documentation to use direct type names
- Mark consolidation plan as complete
- Add comprehensive architectural analysis and refactoring plan
- Add comprehensive SRP refactoring analysis and design documentation
- Update Plan.md with Phase 2.1 AsyncFileSystem integration completion
- Update Plan.md - FASE 3 100% COMPLETADA com Task 3.3 Hash Tree structured queryable model
- Complete Phase 4.1 documentation and roadmap update
- Complete Phase 4.2 enterprise cascade version bumping
- Prepare v2.0 rewrite plan and mark v1.x as obsolete
- Add comprehensive API refactoring documentation
- Add comprehensive CONCEPT.md for changeset-based package management
- Update PLAN.md with phased implementation roadmap
- Update STORY_MAP.md with complete development story map
- Remove deprecated documentation files
- Add comprehensive type relationships documentation
- Add Phase 1 & 2 implementation documentation
- Create comprehensive API specification

### Features

#### WOR-TSK-10

- Add complete planning and story map for sublime-pkg-tools

#### WOR-TSK-11

- Implement foundation module structure for pkg crate

#### WOR-TSK-119

- Implement breaking changes audit

#### WOR-TSK-12

- Refactor error handling system into modular structure
- Implement AsRef<str> for all error types

#### WOR-TSK-13

- Implement comprehensive configuration system integration

#### WOR-TSK-14

- Implement basic version types for package management
- Add VersionResolver with MonorepoDetector integration

#### WOR-TSK-15

- Implement complete version management system with advanced parsing and ranges

#### WOR-TSK-16

- Integrate StandardConfig with PackageJsonEditor for enhanced validation

#### WOR-TSK-17

- Implement conventional commit parsing with Git integration

#### WOR-TSK-18

- Implement complete dependency graph with cycle detection
- Implement dependency graph builder and analyzer

#### WOR-TSK-19

- Add main function to changeset examples

#### WOR-TSK-20

- Implement changeset storage with file-based backend

#### WOR-TSK-21

- Add parse_commit_message method to ConventionalCommitService
- Implement ChangesetBuilder following Git-first architecture
- Export ChangesetBuilder from changeset module
- Add error types for changeset builder operations

#### WOR-TSK-33

- Initialize sublime_pkg_tools crate structure

#### WOR-TSK-34

- Implement comprehensive testing infrastructure

#### WOR-TSK-35

- Implement configuration structures for package tools

#### WOR-TSK-36

- Implement configuration loading and enhanced validation

#### WOR-TSK-38

- Implement comprehensive error type system for pkg tools

#### WOR-TSK-40

- Implement error context and recovery system

#### WOR-TSK-41

- Implement version types with comprehensive testing

#### WOR-TSK-42

- Implement PackageInfo and DependencyType

#### WOR-TSK-43

- Implement changeset types with comprehensive validation and serialization

#### WOR-TSK-44

- Implement dependency types and protocol detection

#### WOR-TSK-45

- Implement VersionResolver foundation with project detection

#### WOR-TSK-46

- Implement dependency graph construction for version resolution

#### WOR-TSK-48

- Implement Story 5.4 - version resolution logic

#### WOR-TSK-49

- Implement dependency propagation for version changes

#### WOR-TSK-50

- Implement snapshot version generation

#### WOR-TSK-51

- Implement apply versions with dry-run support

#### WOR-TSK-52

- Implement version resolution integration tests

#### WOR-TSK-53

- Implement ChangesetStorage trait with comprehensive documentation

#### WOR-TSK-54

- Implement FileBasedChangesetStorage

#### WOR-TSK-55

- Implement ChangesetManager with CRUD operations

#### WOR-TSK-57

- Implement changeset history and archiving

#### WOR-TSK-58

- Implement ChangesAnalyzer foundation with git and monorepo integration

#### WOR-TSK-59

- Implement file-to-package mapping for changes analysis

#### WOR-TSK-60

- Implement working directory analysis

#### WOR-TSK-61

- Implement commit range analysis for changes module

#### WOR-TSK-62

- Implement version preview calculation for changes analysis

#### WOR-TSK-64

- Implement conventional commit parser

#### WOR-TSK-65

- Implement changelog generator foundation

#### WOR-TSK-66

- Implement version detection from git tags

#### WOR-TSK-67

- Implement changelog data collection for story 8.4

#### WOR-TSK-68

- Implement Keep a Changelog formatter

#### WOR-TSK-69

- Implement conventional commits formatter

#### WOR-TSK-70

- Implement custom template formatter for changelogs

#### WOR-TSK-71

- Implement changelog file management (Story 8.8)

#### WOR-TSK-72

- Implement merge commit message generation

#### WOR-TSK-73

- Implement generate_from_changeset for automated changelog generation

#### WOR-TSK-74

- Implement registry client foundation for NPM package queries

#### WOR-TSK-75

- Implement .npmrc parser and configuration
- Add package_name parameter to compare_versions for better error reporting

#### WOR-TSK-76

- Implement upgrade detection for external dependencies

#### WOR-TSK-77

- Implement upgrade application with filtering and dry-run support

#### WOR-TSK-78

- Implement backup and rollback functionality for upgrade operations

#### WOR-TSK-79

- Add automatic changeset creation module

#### WOR-TSK-80

- Implement UpgradeManager integration for unified dependency upgrade workflow

#### WOR-TSK-81

- Implement AuditManager foundation with comprehensive subsystem integration

#### WOR-TSK-82

- Implement upgrade audit section with issue detection

#### WOR-TSK-83

- Implement dependency audit section with circular dependency and version conflict detection

#### WOR-TSK-84

- Implement dependency categorization audit section

#### WOR-TSK-85

- Implement breaking changes audit section

#### WOR-TSK-86

- Implement version consistency audit section

#### WOR-TSK-87

- Implement health score calculation with configurable weights

#### WOR-TSK-88

- Implement audit report formatting

#### Git,changelog

- Implement dynamic repository hosting configuration

#### Monorepo

- Implement comprehensive Git hooks management system (Phase 3)

#### Pkg

- Complete Phase 1 foundation and critical bug fixes
- Implement SRP-compliant dependency services architecture
- Add VersionError::IO variant for filesystem operations
- Implement async filesystem integration in ContextDetector
- Implement async filesystem integration in PackageService
- Implement async filesystem integration in VersionManager
- Complete Task 2.2 - Project/Monorepo Detection Integration
- Complete Task 2.3 - Command Integration with enterprise PackageCommandService
- Complete Task 3.1 - All Dependency Protocols Support with context-aware parsing
- Complete Task 3.2 - Context-Aware Internal/External Classification
- Complete Task 3.3 - Hash Tree como Objeto Estruturado (Não Só Visualização)
- Implement Phase 4.1 context-aware performance optimizations
- Add comprehensive clippy allows for Phase 4.1 completion
- Implement enterprise cascade version bumping with multiple strategies
- Complete Phase 4.3 network resilience implementation
- Complete Task 2.1.5 - PackageManager integration and final tests
- Complete Task 2.2.1 - DependencyAnalyzer base structure
- Add common traits for shared behavior patterns
- Add type aliases for common string types
- Add prelude module for convenient imports
- Add WorkspaceConfig for project-specific workspace patterns
- Integrate WorkspaceConfig into PackageToolsConfig

#### Safety

- Eliminate all unwrap usage and achieve production readiness compliance

#### Workspace

- Enhance package versioning and tag management

### Miscellaneous Tasks

#### Pkg

- Remove completed consolidation plan

### Refactor

#### WOR-TSK-125

- Rename binary from wnt to workspace

#### WOR-TSK-16

- Enhance package discovery using MonorepoDetector from standard crate

#### WOR-TSK-18

- Fix clippy collapsible_match warning in package module

#### WOR-TSK-21

- Integrate MonorepoDetector into PackageChangeDetector

#### WOR-TSK-52

- Eliminate all 64 clippy warnings in pkg crate

#### WOR-TSK-79

- Update application module exports
- Expose changeset module in upgrade exports

#### Pkg

- Remove Rc<RefCell<>> wrappers for thread safety
- Transform Registry into enterprise-grade facade pattern
- Migrate to Rust-idiomatic planning approach
- Reorganize tests to dedicated tests.rs files per CLAUDE.md standards
- Remove redundant convenience functions from package module
- Update changeset detector to use direct PackageJson API
- Update public exports to remove convenience functions
- Reset lib.rs for new architecture implementation
- Remove PackageUpdate duplication from types module

### Styling

#### WOR-TSK-124

- Apply rustfmt with CI configuration

### Testing

#### WOR-TSK-14

- Add comprehensive test suite for version types

#### WOR-TSK-18

- Add comprehensive test suite for dependency graph functionality

#### WOR-TSK-21

- Add comprehensive integration tests for ChangesetBuilder

#### WOR-TSK-47

- Add comprehensive circular dependency detection tests

#### WOR-TSK-54

- Add comprehensive unit tests for FileBasedChangesetStorage
- Add integration tests with real Git workflows

#### WOR-TSK-63

- Add comprehensive statistics tests for changes module

#### WOR-TSK-78

- Add comprehensive path normalization and directory existence tests

#### WOR-TSK-89

- Add comprehensive audit integration tests

#### Pkg

- Update test imports to use direct type names
- Add comprehensive test coverage for versioning strategies
- Update package module tests to use direct APIs
- Add comprehensive tests for WorkspaceConfig

### Ci

#### WOR-TSK-131

- Repo configs and tools ([#1](https://github.com/websublime/workspace-tools/pull/1))

<!-- Made with ❤️ by WebSublime -->
