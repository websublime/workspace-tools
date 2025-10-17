# sublime_pkg_tools - Development Story Map

**Version**: 1.0  
**Based on**: PLAN.md v1.0  
**Last Updated**: 2024-01-15  
**Status**: 📋 Ready for Development

---

## Table of Contents

1. [Story Map Overview](#story-map-overview)
2. [Effort Metrics Definition](#effort-metrics-definition)
3. [Epic 1: Project Foundation](#epic-1-project-foundation)
4. [Epic 2: Configuration System](#epic-2-configuration-system)
5. [Epic 3: Error Handling](#epic-3-error-handling)
6. [Epic 4: Core Types](#epic-4-core-types)
7. [Epic 5: Versioning Engine](#epic-5-versioning-engine)
8. [Epic 6: Changeset Management](#epic-6-changeset-management)
9. [Epic 7: Changes Analysis](#epic-7-changes-analysis)
10. [Epic 8: Changelog Generation](#epic-8-changelog-generation)
11. [Epic 9: Dependency Upgrades](#epic-9-dependency-upgrades)
12. [Epic 10: Audit & Health Checks](#epic-10-audit--health-checks)
13. [Epic 11: Integration & Documentation](#epic-11-integration--documentation)

---

## Story Map Overview

### Epic Breakdown

```
Phase 1: Foundation (Weeks 1-3)
├── Epic 1: Project Foundation
├── Epic 2: Configuration System
├── Epic 3: Error Handling
└── Epic 4: Core Types

Phase 2: Core Functionality (Weeks 4-7)
├── Epic 5: Versioning Engine
├── Epic 6: Changeset Management
└── Epic 7: Changes Analysis

Phase 3: Advanced Features (Weeks 8-10)
├── Epic 8: Changelog Generation
└── Epic 9: Dependency Upgrades

Phase 4: Integration & Polish (Weeks 11-12)
├── Epic 10: Audit & Health Checks
└── Epic 11: Integration & Documentation
```

### Total Story Count
- **Epics**: 11
- **User Stories**: 67
- **Tasks**: 342+

---

## Effort Metrics Definition

### Effort Levels

| Level | Time Estimate | Complexity | Examples |
|-------|--------------|------------|----------|
| **Minimal** | 1-2 hours | Trivial | Simple struct, basic export, straightforward test |
| **Low** | 3-6 hours | Simple | Single function implementation, basic error type, simple validation |
| **Medium** | 1-2 days | Moderate | Complex algorithm, multiple integration points, comprehensive testing |
| **High** | 3-5 days | Complex | Core module implementation, advanced logic, extensive edge cases |
| **Massive** | 1-2 weeks | Very Complex | Complete subsystem, multiple dependencies, full integration |

### Estimation Guidelines

**Minimal (1-2h)**:
- Creating simple data structures
- Adding basic exports
- Writing straightforward tests
- Simple documentation updates

**Low (3-6h)**:
- Implementing single functions with clear logic
- Creating basic error types
- Writing unit tests for simple functions
- Adding module-level documentation

**Medium (1-2d)**:
- Implementing algorithms with moderate complexity
- Creating trait implementations
- Writing comprehensive test suites
- Integration with external crates

**High (3-5d)**:
- Implementing core business logic modules
- Complex algorithm implementations (e.g., circular dependency detection)
- Full test coverage with edge cases
- API design and documentation

**Massive (1-2w)**:
- Complete subsystem implementation
- Multiple module integration
- Performance optimization
- Comprehensive documentation and examples

---

## Epic 1: Project Foundation

**Phase**: 1  
**Total Effort**: Medium  
**Dependencies**: None  
**Goal**: Establish the basic project structure and development environment

### Story 1.1: Initialize Crate Structure
**Effort**: Low  
**Priority**: Critical

**As a** developer  
**I want** the basic crate structure initialized  
**So that** I can start implementing modules following the standard patterns

**Description**:
Set up the foundational project structure following `sublime_standard_tools` patterns. This includes directory structure, Cargo.toml configuration, and basic file scaffolding.

**Tasks**:
1. Create `Cargo.toml` with all dependencies
   - Add internal crates (standard, git)
   - Add external dependencies (tokio, serde, etc.)
   - Configure features and metadata
   - **Effort**: Minimal

2. Create `src/lib.rs` with crate documentation
   - Add clippy rules
   - Add crate-level documentation
   - Implement `version()` function
   - Include CONCEPT.md in docs
   - **Effort**: Low

3. Create module directory structure
   - Create all module directories
   - Add empty `mod.rs` files
   - Setup proper module exports
   - **Effort**: Minimal

4. Configure development tools
   - Setup `.cargo/config.toml`
   - Add `rustfmt.toml`
   - Configure clippy settings
   - **Effort**: Minimal

**Acceptance Criteria**:
- [ ] `Cargo.toml` contains all required dependencies
- [ ] Project compiles without errors
- [ ] `cargo fmt` runs successfully
- [ ] `cargo clippy` runs successfully (may have warnings at this stage)
- [ ] `lib.rs` has crate-level documentation
- [ ] `version()` function returns correct version
- [ ] All module directories created
- [ ] Module structure follows `sublime_standard_tools` patterns

**Definition of Done**:
- [ ] Code compiles
- [ ] Clippy passes (allowing some warnings for empty modules)
- [ ] Basic documentation in place
- [ ] PR approved and merged

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 1.2: Setup CI/CD Pipeline
**Effort**: Medium  
**Priority**: Critical

**As a** developer  
**I want** automated CI/CD pipelines configured  
**So that** code quality is enforced automatically

**Description**:
Configure GitHub Actions (or equivalent) to run automated checks on every commit and PR. This ensures consistent code quality and prevents regressions.

**Tasks**:
1. Create CI workflow for tests
   - Setup matrix for OS (Ubuntu, macOS, Windows)
   - Setup matrix for Rust versions (stable, nightly)
   - Configure test execution
   - **Effort**: Low

2. Create CI workflow for code quality
   - Add `cargo fmt` check
   - Add `cargo clippy` check
   - Add `cargo doc` check
   - **Effort**: Minimal

3. Setup code coverage pipeline
   - Install and configure tarpaulin
   - Upload coverage to codecov
   - Add coverage badge
   - Add 100% coverage requirement check
   - **Effort**: Medium

4. Create PR templates and guidelines
   - Add PR template
   - Add contributing guidelines
   - Document commit message format
   - **Effort**: Minimal

**Acceptance Criteria**:
- [ ] CI runs on push and PR
- [ ] Tests run on all platforms (Ubuntu, macOS, Windows)
- [ ] Tests run on stable and nightly Rust
- [ ] `cargo fmt --check` enforced
- [ ] `cargo clippy -- -D warnings` enforced
- [ ] Coverage report generated and uploaded
- [ ] PR template available
- [ ] All checks must pass before merge

**Definition of Done**:
- [ ] CI pipeline executes successfully
- [ ] All quality checks pass
- [ ] Coverage reporting works
- [ ] Documentation updated

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 1.3: Setup Testing Infrastructure
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** testing infrastructure and helpers in place  
**So that** I can write comprehensive tests efficiently

**Description**:
Create reusable test utilities, mock implementations, and test fixtures that will be used across all modules.

**Tasks**:
1. Create test helpers module
   - Create `tests/common/mod.rs`
   - Add filesystem mock utilities
   - Add git mock utilities
   - Add assertion helpers
   - **Effort**: Medium

2. Create test fixtures
   - Create sample monorepo structure
   - Create sample single-package structure
   - Add sample package.json files
   - Add sample config files
   - **Effort**: Low

3. Setup mock implementations
   - Create `MockFileSystem` struct
   - Create `MockGitRepository` struct
   - Create `MockRegistry` struct
   - Implement required traits
   - **Effort**: High

4. Add property-based testing setup
   - Add proptest dependency
   - Create version property generators
   - Create commit message generators
   - Add example property tests
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Test helpers module accessible from all tests
- [ ] Mock implementations available
- [ ] Test fixtures in `tests/fixtures/`
- [ ] Proptest generators working
- [ ] Example tests using helpers pass
- [ ] Documentation for test utilities complete

**Definition of Done**:
- [ ] Test infrastructure compiles
- [ ] Example tests pass
- [ ] Documentation complete
- [ ] Ready for use in module tests

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

## Epic 2: Configuration System

**Phase**: 1  
**Total Effort**: High  
**Dependencies**: Epic 1  
**Goal**: Implement complete configuration system with validation and defaults

### Story 2.1: Define Configuration Structure
**Effort**: Medium  
**Priority**: Critical

**As a** developer  
**I want** all configuration structures defined  
**So that** modules can access their configuration consistently

**Description**:
Define all configuration structs, enums, and types that will be used across the crate. This includes the main `PackageToolsConfig` and all sub-configurations.

**Tasks**:
1. Create `src/config/types.rs`
   - Define `PackageToolsConfig` struct
   - Add all configuration fields
   - Implement `Default` trait
   - Add serde derives
   - **Effort**: Low

2. Create `src/config/changeset.rs`
   - Define `ChangesetConfig` struct
   - Add path, history_path, environments
   - Implement `Default` with sensible values
   - Add documentation
   - **Effort**: Minimal

3. Create `src/config/version.rs`
   - Define `VersionConfig` struct
   - Define `VersioningStrategy` enum
   - Define `DependencyConfig` struct
   - Add all propagation settings
   - Implement defaults
   - **Effort**: Low

4. Create `src/config/git.rs`
   - Define `GitConfig` struct
   - Add merge commit templates
   - Add breaking warning templates
   - Implement defaults
   - **Effort**: Minimal

5. Create `src/config/changelog.rs`
   - Define `ChangelogConfig` struct
   - Define `ConventionalConfig` struct
   - Define format enums
   - Implement defaults
   - **Effort**: Low

6. Create `src/config/upgrade.rs`
   - Define `UpgradeConfig` struct
   - Define `RegistryConfig` struct
   - Define `BackupConfig` struct
   - Implement defaults
   - **Effort**: Low

7. Create `src/config/audit.rs`
   - Define `AuditConfig` struct
   - Define all audit section configs
   - Implement defaults
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] All config structs defined
- [ ] All configs implement `Default`
- [ ] All configs implement `Serialize` and `Deserialize`
- [ ] All configs have field documentation
- [ ] Default values match CONCEPT.md specifications
- [ ] Structs use `pub(crate)` for internal fields appropriately
- [ ] Clippy passes without warnings
- [ ] All configs accessible via `PackageToolsConfig`

**Definition of Done**:
- [ ] All config files compile
- [ ] Defaults instantiate correctly
- [ ] Serialization/deserialization works
- [ ] Documentation complete
- [ ] Tests written and passing

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 2.2: Implement Configuration Loading
**Effort**: High  
**Priority**: Critical

**As a** user of the library  
**I want** to load configuration from files  
**So that** I can customize behavior without code changes

**Description**:
Implement configuration loading from TOML/YAML/JSON files using the `sublime_standard_tools` ConfigManager. Support environment variable overrides and validation.

**Tasks**:
1. Implement `Configurable` trait
   - Implement for `PackageToolsConfig`
   - Add validation logic
   - Add merge logic
   - **Effort**: Medium

2. Create configuration loader
   - Integrate with `ConfigManager`
   - Support multiple file formats
   - Add file path resolution
   - **Effort**: Medium

3. Implement environment variable overrides
   - Parse env vars with `SUBLIME_PKG_` prefix
   - Map to config fields
   - Apply overrides correctly
   - **Effort**: Medium

4. Add configuration validation
   - Create `src/config/validation.rs`
   - Validate paths exist
   - Validate enum values
   - Validate dependencies between settings
   - Return detailed errors
   - **Effort**: High

5. Write comprehensive tests
   - Test file loading (TOML, YAML, JSON)
   - Test defaults
   - Test env var overrides
   - Test validation logic
   - Test error cases
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Can load config from TOML file
- [ ] Can load config from YAML file
- [ ] Can load config from JSON file
- [ ] Environment variables override file config
- [ ] Invalid config returns detailed error
- [ ] Default config passes validation
- [ ] Validation errors are clear and actionable
- [ ] 100% test coverage on config loading
- [ ] Clippy passes
- [ ] Documentation includes examples

**Definition of Done**:
- [ ] All loading scenarios work
- [ ] Validation comprehensive
- [ ] Tests pass with 100% coverage
- [ ] Documentation complete
- [ ] Integration with standard tools verified

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 2.3: Configuration Documentation and Examples
**Effort**: Low  
**Priority**: High

**As a** user  
**I want** clear documentation on configuration options  
**So that** I can configure the library correctly

**Description**:
Create comprehensive documentation for all configuration options, including examples, default values, and best practices.

**Tasks**:
1. Document configuration structure
   - Add module-level docs
   - Document each config field
   - Add examples for common scenarios
   - **Effort**: Medium

2. Create configuration examples
   - Add example TOML configs
   - Add example with env vars
   - Add monorepo example
   - Add single-package example
   - **Effort**: Low

3. Add configuration guide
   - Create `docs/guides/configuration.md`
   - Explain each section
   - Show migration examples
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Every config option documented
- [ ] Examples compile and work
- [ ] Configuration guide complete
- [ ] Examples in `examples/` directory
- [ ] README mentions configuration

**Definition of Done**:
- [ ] Documentation complete
- [ ] Examples working
- [ ] Guide published

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

## Epic 3: Error Handling

**Phase**: 1  
**Total Effort**: Medium  
**Dependencies**: Epic 1  
**Goal**: Implement comprehensive error handling system

### Story 3.1: Define Error Types
**Effort**: Medium  
**Priority**: Critical

**As a** developer  
**I want** all error types defined upfront  
**So that** I can use them consistently across modules

**Description**:
Define all domain-specific error types using thiserror. Each module gets its own error type, and all errors implement `AsRef<str>` as required.

**Tasks**:
1. Create main error enum in `src/error/mod.rs`
   - Define `Error` enum with variants
   - Implement `From` for all domain errors
   - Implement `AsRef<str>`
   - Add display implementations
   - **Effort**: Low

2. Create `src/error/config.rs`
   - Define `ConfigError` enum
   - Add variants for validation, parsing, file errors
   - Implement `AsRef<str>`
   - Add context fields
   - **Effort**: Minimal

3. Create `src/error/version.rs`
   - Define `VersionError` enum
   - Add variants for parsing, resolution, propagation
   - Implement `AsRef<str>`
   - **Effort**: Minimal

4. Create `src/error/changeset.rs`
   - Define `ChangesetError` enum
   - Add variants for storage, validation, git
   - Implement `AsRef<str>`
   - **Effort**: Minimal

5. Create `src/error/changes.rs`
   - Define `ChangesError` enum
   - Add variants for git, mapping, analysis
   - Implement `AsRef<str>`
   - **Effort**: Minimal

6. Create `src/error/changelog.rs`
   - Define `ChangelogError` enum
   - Add variants for parsing, generation, formatting
   - Implement `AsRef<str>`
   - **Effort**: Minimal

7. Create `src/error/upgrade.rs`
   - Define `UpgradeError` enum
   - Add variants for registry, backup, rollback
   - Implement `AsRef<str>`
   - **Effort**: Minimal

8. Create `src/error/audit.rs`
   - Define `AuditError` enum
   - Add variants for analysis, reporting
   - Implement `AsRef<str>`
   - **Effort**: Minimal

**Acceptance Criteria**:
- [ ] All error types defined
- [ ] All errors use `thiserror::Error`
- [ ] All errors implement `AsRef<str>`
- [ ] Error messages are clear and actionable
- [ ] Error variants cover all failure scenarios
- [ ] Type aliases defined (e.g., `ConfigResult<T>`)
- [ ] Clippy passes
- [ ] Documentation complete

**Definition of Done**:
- [ ] All error files compile
- [ ] Error conversions work
- [ ] Tests verify `AsRef<str>` implementation
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 3.2: Error Context and Recovery
**Effort**: Low  
**Priority**: Medium

**As a** developer  
**I want** rich error context and recovery strategies  
**So that** I can provide helpful error messages to users

**Description**:
Add error context helpers and optional recovery strategies following patterns from `sublime_standard_tools`.

**Tasks**:
1. Create error context trait
   - Define `ErrorContext` trait
   - Add context attachment methods
   - **Effort**: Low

2. Implement error recovery patterns
   - Add recovery strategy enum
   - Implement for common errors
   - **Effort**: Medium

3. Add error tests
   - Test error creation
   - Test error conversion
   - Test `AsRef<str>` implementation
   - Test error messages
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Error context can be attached
- [ ] Recovery strategies available
- [ ] Tests cover all error types
- [ ] Error messages tested
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Context system works
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

## Epic 4: Core Types

**Phase**: 1  
**Total Effort**: Medium  
**Dependencies**: Epic 1, Epic 3  
**Goal**: Define all core data structures used across modules

### Story 4.1: Version Types
**Effort**: Medium  
**Priority**: Critical

**As a** developer  
**I want** version types defined  
**So that** I can handle semantic versioning correctly

**Description**:
Define Version struct with parsing, comparison, and bumping capabilities using the `semver` crate.

**Tasks**:
1. Create `src/types/version.rs`
   - Define `Version` struct
   - Wrap `semver::Version`
   - Add `parse()` method
   - Add `bump()` method for each bump type
   - **Effort**: Low

2. Define `VersionBump` enum
   - Add Major, Minor, Patch, None variants
   - Implement Display
   - Add serialization
   - **Effort**: Minimal

3. Define `VersioningStrategy` enum
   - Add Independent, Unified variants
   - Add documentation
   - **Effort**: Minimal

4. Implement version operations
   - Implement comparison (PartialOrd, Ord)
   - Add increment methods
   - Add snapshot version generation
   - **Effort**: Low

5. Write comprehensive tests
   - Test parsing valid versions
   - Test parsing invalid versions
   - Test bumping (all types)
   - Test comparisons
   - Property-based tests for parsing
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] `Version` parses semver strings correctly
- [ ] Bumping works for all types
- [ ] Comparisons work correctly
- [ ] Invalid versions return errors (not panic)
- [ ] Serialization/deserialization works
- [ ] 100% test coverage
- [ ] Property tests pass
- [ ] Clippy passes

**Definition of Done**:
- [ ] Version types complete
- [ ] Tests pass with 100% coverage
- [ ] Documentation with examples
- [ ] No unwrap/expect used

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 4.2: Package Types
**Effort**: Medium  
**Priority**: Critical

**As a** developer  
**I want** package information structures  
**So that** I can work with package metadata consistently

**Description**:
Define `PackageInfo` struct that aggregates package.json data and workspace information.

**Tasks**:
1. Create `src/types/package.rs`
   - Define `PackageInfo` struct
   - Add package_json field (from package-json crate)
   - Add workspace field
   - Add path field
   - **Effort**: Low

2. Implement package methods
   - Add `name()` accessor
   - Add `version()` accessor
   - Add `all_dependencies()` method
   - Add `is_internal()` check
   - **Effort**: Low

3. Add dependency helpers
   - Filter workspace protocols
   - Filter local protocols
   - Get internal vs external deps
   - **Effort**: Medium

4. Write tests
   - Test with real package.json
   - Test dependency filtering
   - Test internal detection
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] `PackageInfo` contains all needed data
- [ ] Accessors work correctly
- [ ] Dependency filtering accurate
- [ ] Works with package-json crate
- [ ] 100% test coverage
- [ ] Clippy passes

**Definition of Done**:
- [ ] PackageInfo complete
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 4.3: Changeset Types
**Effort**: Low  
**Priority**: Critical

**As a** developer  
**I want** changeset data structures defined  
**So that** I can store and manipulate changesets

**Description**:
Define the `Changeset` struct and related types for storing release information.

**Tasks**:
1. Create `src/types/changeset.rs`
   - Define `Changeset` struct
   - Add all fields (branch, bump, environments, packages, changes)
   - Add timestamps
   - Implement serialization
   - **Effort**: Low

2. Define `ArchivedChangeset` struct
   - Add changeset field
   - Add `ReleaseInfo` struct
   - Add applied_at, applied_by, git_commit, versions
   - **Effort**: Minimal

3. Add changeset methods
   - Validation helpers
   - Update helpers
   - **Effort**: Low

4. Write tests
   - Test serialization/deserialization
   - Test JSON format matches spec
   - Test validation
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Changeset matches CONCEPT.md specification
- [ ] Serializes to clean JSON
- [ ] All fields accessible
- [ ] Validation works
- [ ] Tests pass 100%
- [ ] Clippy passes

**Definition of Done**:
- [ ] Changeset types complete
- [ ] JSON format verified
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 4.4: Dependency Types
**Effort**: Minimal  
**Priority**: High

**As a** developer  
**I want** dependency-related types defined  
**So that** I can categorize and work with dependencies

**Description**:
Define enums and structs for dependency classification and management.

**Tasks**:
1. Create `src/types/dependency.rs`
   - Define `DependencyType` enum (Regular, Dev, Peer, Optional)
   - Define protocol enums (Workspace, File, Link, Portal)
   - Add helper functions
   - **Effort**: Minimal

2. Write tests
   - Test enum variants
   - Test serialization
   - **Effort**: Minimal

**Acceptance Criteria**:
- [ ] All dependency types defined
- [ ] Serialization works
- [ ] Tests pass
- [ ] Documentation complete

**Definition of Done**:
- [ ] Types complete
- [ ] Tests pass
- [ ] Ready for use

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

## Epic 5: Versioning Engine

**Phase**: 2  
**Total Effort**: Massive  
**Dependencies**: Epic 2, Epic 3, Epic 4  
**Goal**: Implement complete version resolution and propagation system

### Story 5.1: Version Resolver Foundation
**Effort**: High  
**Priority**: Critical

**As a** developer  
**I want** the VersionResolver structure implemented  
**So that** I can resolve versions for packages

**Description**:
Implement the main `VersionResolver` struct with project detection and initialization logic.

**Tasks**:
1. Create `src/version/resolver.rs`
   - Define `VersionResolver` struct
   - Add workspace_root, strategy, fs fields
   - Implement `new()` constructor
   - Add monorepo/single-package detection
   - **Effort**: Medium

2. Implement project detection
   - Use `MonorepoDetector` from standard tools
   - Detect if monorepo or single package
   - Load package information
   - **Effort**: Medium

3. Add package discovery
   - Find all packages in workspace
   - Load package.json for each
   - Create `PackageInfo` instances
   - **Effort**: Medium

4. Write initialization tests
   - Test with monorepo fixture
   - Test with single-package fixture
   - Test error cases
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] `VersionResolver::new()` works
- [ ] Detects monorepo correctly
- [ ] Detects single-package correctly
- [ ] Loads all packages
- [ ] Returns errors for invalid projects
- [ ] Tests pass 100%
- [ ] Clippy passes
- [ ] No unwrap/expect

**Definition of Done**:
- [ ] Resolver initializes correctly
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 5.2: Dependency Graph Construction
**Effort**: High  
**Priority**: Critical

**As a** developer  
**I want** to build a dependency graph  
**So that** I can detect relationships between packages

**Description**:
Implement `DependencyGraph` that represents internal package dependencies.

**Tasks**:
1. Create `src/version/graph.rs`
   - Define `DependencyGraph` struct
   - Use petgraph or custom implementation
   - Add node_map for package lookup
   - **Effort**: Medium

2. Implement graph construction
   - Parse dependencies from package.json
   - Filter internal vs external
   - Add edges for dependencies
   - **Effort**: High

3. Add graph queries
   - Get dependents of a package
   - Get dependencies of a package
   - Check if package exists
   - **Effort**: Low

4. Write tests
   - Test graph construction
   - Test with various dependency structures
   - Test queries
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Graph builds from packages
- [ ] Internal dependencies identified
- [ ] External dependencies filtered out
- [ ] Queries work correctly
- [ ] Tests pass 100%
- [ ] Handles workspace:* protocols
- [ ] Clippy passes

**Definition of Done**:
- [ ] Graph construction works
- [ ] Tests comprehensive
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 5.3: Circular Dependency Detection
**Effort**: High  
**Priority**: Critical

**As a** developer  
**I want** to detect circular dependencies  
**So that** I can prevent infinite propagation loops

**Description**:
Implement Tarjan's algorithm or similar to detect cycles in the dependency graph.

**Tasks**:
1. Implement cycle detection algorithm
   - Choose algorithm (Tarjan's or DFS-based)
   - Implement in `graph.rs`
   - Return all cycles found
   - **Effort**: High

2. Create `CircularDependency` type
   - Store cycle path
   - Add helpful error messages
   - **Effort**: Low

3. Add detection to graph
   - Add `detect_cycles()` method
   - Return `Vec<CircularDependency>`
   - **Effort**: Low

4. Write comprehensive tests
   - Test with no cycles
   - Test with single cycle
   - Test with multiple cycles
   - Test with nested cycles
   - Property-based tests
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Detects all circular dependencies
- [ ] Returns clear cycle paths
- [ ] No false positives
- [ ] No false negatives
- [ ] Performance acceptable (< 1s for 100 packages)
- [ ] Tests cover all cases
- [ ] 100% test coverage
- [ ] Clippy passes

**Definition of Done**:
- [ ] Algorithm correct and tested
- [ ] Performance verified
- [ ] Documentation with examples
- [ ] Property tests pass

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 5.4: Version Resolution Logic
**Effort**: High  
**Priority**: Critical

**As a** developer  
**I want** to resolve versions for changed packages  
**So that** I can determine what version each package should become

**Description**:
Implement the core version resolution logic that calculates next versions based on changeset bump type.

**Tasks**:
1. Create `src/version/resolution.rs`
   - Define `VersionResolution` struct
   - Define `PackageUpdate` struct
   - Add resolution logic
   - **Effort**: Medium

2. Implement direct resolution
   - For packages in changeset, apply bump
   - Calculate next version
   - Create `PackageUpdate` entries
   - **Effort**: Medium

3. Add resolution validation
   - Verify all packages exist
   - Check versions are valid
   - Validate bump types
   - **Effort**: Low

4. Write resolution tests
   - Test with different bump types
   - Test with various current versions
   - Test error cases
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Resolves versions correctly
- [ ] Handles Major, Minor, Patch bumps
- [ ] Works with unified strategy
- [ ] Works with independent strategy
- [ ] Validates inputs
- [ ] Returns clear errors
- [ ] Tests pass 100%
- [ ] Clippy passes

**Definition of Done**:
- [ ] Resolution logic complete
- [ ] All strategies work
- [ ] Tests comprehensive
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 5.5: Dependency Propagation
**Effort**: Massive  
**Priority**: Critical

**As a** developer  
**I want** dependency updates to propagate through the graph  
**So that** dependent packages are updated automatically

**Description**:
Implement the dependency propagation algorithm that updates all packages that depend on changed packages.

**Tasks**:
1. Create `src/version/propagation.rs`
   - Define propagation algorithm
   - Use BFS or topological sort
   - Track propagation depth
   - **Effort**: High

2. Implement propagation logic
   - For each updated package, find dependents
   - Apply propagation bump to dependents
   - Update dependency specs in package.json
   - Recurse until no more updates
   - **Effort**: High

3. Add propagation configuration
   - Respect max_depth setting
   - Respect propagation_bump setting
   - Filter by dependency types
   - Skip workspace/local protocols
   - **Effort**: Medium

4. Handle circular dependencies
   - Detect during propagation
   - Skip or report (based on config)
   - Ensure termination
   - **Effort**: Medium

5. Write extensive tests
   - Test simple propagation (A->B, A changes)
   - Test chain propagation (A->B->C, A changes)
   - Test diamond dependency (A->B, A->C, B->D, C->D)
   - Test circular dependencies
   - Test max_depth limits
   - Test different propagation bumps
   - **Effort**: Massive

**Acceptance Criteria**:
- [ ] Propagation reaches all dependents
- [ ] Respects configuration settings
- [ ] Terminates with circular deps
- [ ] Updates dependency specs correctly
- [ ] Skips workspace:* and file: protocols
- [ ] Performance acceptable
- [ ] Tests cover all scenarios
- [ ] 100% test coverage
- [ ] Clippy passes
- [ ] No infinite loops

**Definition of Done**:
- [ ] Propagation algorithm complete
- [ ] All edge cases handled
- [ ] Tests comprehensive
- [ ] Performance verified
- [ ] Documentation with diagrams

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 5.6: Snapshot Version Generation
**Effort**: Low  
**Priority**: Medium

**As a** developer  
**I want** to generate snapshot versions  
**So that** I can deploy branch builds

**Description**:
Implement snapshot version generation with configurable format.

**Tasks**:
1. Create `src/version/snapshot.rs`
   - Parse snapshot format template
   - Replace variables ({version}, {branch}, {commit})
   - Generate snapshot version string
   - **Effort**: Low

2. Add snapshot validation
   - Ensure valid semver format
   - Check branch name safety
   - **Effort**: Low

3. Write tests
   - Test with different formats
   - Test variable replacement
   - Test validation
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Generates valid snapshot versions
- [ ] Format configurable
- [ ] All variables replaced
- [ ] Validation works
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Snapshot generation works
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 5.7: Apply Versions with Dry-Run
**Effort**: High  
**Priority**: Critical

**As a** developer  
**I want** to apply versions to package.json files  
**So that** I can update package versions

**Description**:
Implement version application logic that writes updated versions to package.json files, with dry-run support.

**Tasks**:
1. Implement package.json reading
   - Use `FileSystemManager` from standard tools
   - Parse with package-json crate
   - Handle errors gracefully
   - **Effort**: Low

2. Implement package.json writing
   - Update version field
   - Update dependency specs
   - Preserve formatting
   - Use atomic writes
   - **Effort**: Medium

3. Implement dry-run mode
   - Skip writes when dry_run=true
   - Return what would be written
   - **Effort**: Low

4. Add rollback support
   - Backup files before writing
   - Restore on error
   - Clean up on success
   - **Effort**: Medium

5. Write tests
   - Test dry-run (no files changed)
   - Test actual writing
   - Test rollback on error
   - Test atomic writes
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Writes versions correctly
- [ ] Dry-run doesn't modify files
- [ ] Rollback works on failure
- [ ] Preserves JSON formatting
- [ ] Uses atomic writes
- [ ] Tests pass 100%
- [ ] Works cross-platform
- [ ] Clippy passes

**Definition of Done**:
- [ ] Apply versions works
- [ ] Dry-run verified
- [ ] Rollback tested
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 5.8: Version Resolution Integration Tests
**Effort**: High  
**Priority**: High

**As a** developer  
**I want** end-to-end integration tests for versioning  
**So that** I can verify the complete workflow

**Description**:
Write comprehensive integration tests that verify the entire version resolution and application workflow.

**Tasks**:
1. Create integration test fixtures
   - Setup test monorepo
   - Create various dependency structures
   - **Effort**: Medium

2. Write workflow tests
   - Test complete resolution workflow
   - Test propagation in real project
   - Test dry-run then apply
   - **Effort**: High

3. Write edge case tests
   - Test with circular deps
   - Test with max depth
   - Test with different strategies
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Full workflow tested
- [ ] Edge cases covered
- [ ] Tests run in CI
- [ ] 100% of resolution logic covered

**Definition of Done**:
- [ ] Integration tests pass
- [ ] Coverage verified
- [ ] CI integration complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

## Epic 6: Changeset Management

**Phase**: 2  
**Total Effort**: High  
**Dependencies**: Epic 2, Epic 3, Epic 4  
**Goal**: Implement complete changeset CRUD and storage

### Story 6.1: Changeset Storage Trait
**Effort**: Low  
**Priority**: Critical

**As a** developer  
**I want** a storage abstraction  
**So that** I can swap storage implementations

**Description**:
Define the `ChangesetStorage` trait that abstracts changeset persistence.

**Tasks**:
1. Create `src/changeset/storage.rs`
   - Define `ChangesetStorage` trait
   - Add async methods (save, load, exists, delete, list, archive)
   - Add documentation
   - **Effort**: Low

2. Define storage types
   - Define result types
   - Add storage errors
   - **Effort**: Minimal

**Acceptance Criteria**:
- [ ] Trait defined with all methods
- [ ] Methods are async
- [ ] Error types appropriate
- [ ] Documentation complete
- [ ] Ready for implementation

**Definition of Done**:
- [ ] Trait compiles
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 6.2: File-Based Storage Implementation
**Effort**: High  
**Priority**: Critical

**As a** developer  
**I want** file-based changeset storage  
**So that** changesets persist to disk

**Description**:
Implement `FileBasedChangesetStorage` that stores changesets as JSON files.

**Tasks**:
1. Implement FileBasedChangesetStorage struct
   - Add root_path, changeset_dir, history_dir fields
   - Use `FileSystemManager` from standard tools
   - **Effort**: Low

2. Implement save method
   - Serialize changeset to JSON
   - Write to file atomically
   - Create directories if needed
   - **Effort**: Medium

3. Implement load method
   - Read file
   - Deserialize JSON
   - Handle missing files
   - **Effort**: Low

4. Implement exists method
   - Check file existence
   - **Effort**: Minimal

5. Implement delete method
   - Delete changeset file
   - Handle errors
   - **Effort**: Minimal

6. Implement list_pending method
   - List all changeset files
   - Parse and return
   - **Effort**: Low

7. Implement archive method
   - Move changeset to history
   - Add release info
   - **Effort**: Medium

8. Implement load_archived method
   - Load from history directory
   - Parse archived format
   - **Effort**: Low

9. Write comprehensive tests
   - Test all methods
   - Test with mock filesystem
   - Test error cases
   - Test concurrent access
   - **Effort**: High

**Acceptance Criteria**:
- [ ] All trait methods implemented
- [ ] Uses atomic file operations
- [ ] Handles concurrent access
- [ ] Error messages clear
- [ ] Tests pass 100%
- [ ] Works cross-platform
- [ ] Clippy passes

**Definition of Done**:
- [ ] Implementation complete
- [ ] Tests comprehensive
- [ ] Documentation complete
- [ ] Concurrent access tested

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 6.3: Changeset Manager
**Effort**: High  
**Priority**: Critical

**As a** developer  
**I want** a high-level changeset manager  
**So that** I can create and manage changesets easily

**Description**:
Implement `ChangesetManager` that provides a high-level API for changeset operations.

**Tasks**:
1. Create `src/changeset/manager.rs`
   - Define `ChangesetManager` struct
   - Add storage and git_repo fields
   - Implement `new()` constructor
   - **Effort**: Low

2. Implement create method
   - Validate branch name
   - Create new changeset
   - Save to storage
   - **Effort**: Medium

3. Implement load method
   - Load from storage
   - Return error if not found
   - **Effort**: Low

4. Implement update method
   - Load existing changeset
   - Apply updates
   - Validate changes
   - Save back to storage
   - **Effort**: Medium

5. Implement delete method
   - Delete from storage
   - **Effort**: Minimal

6. Implement list_pending method
   - Get all pending changesets
   - **Effort**: Low

7. Write tests
   - Test create
   - Test load
   - Test update
   - Test delete
   - Test list
   - Test error cases
   - **Effort**: High

**Acceptance Criteria**:
- [ ] All CRUD operations work
- [ ] Validation prevents invalid data
- [ ] Errors clear and actionable
- [ ] Tests pass 100%
- [ ] Clippy passes
- [ ] Documentation complete

**Definition of Done**:
- [ ] Manager complete
- [ ] Tests pass
- [ ] Documentation with examples

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 6.4: Git Integration for Commits
**Effort**: High  
**Priority**: High

**As a** developer  
**I want** to detect affected packages from git commits  
**So that** I can automatically populate changesets

**Description**:
Implement git integration that detects which packages are affected by commits.

**Tasks**:
1. Create `src/changeset/git_integration.rs`
   - Use `sublime_git_tools` crate
   - Implement package detection from diffs
   - **Effort**: Medium

2. Implement add_commits_from_git
   - Parse commit range
   - Get commits from git
   - Detect affected packages for each commit
   - Update changeset
   - **Effort**: High

3. Add package detection logic
   - Map changed files to packages
   - Use monorepo detection
   - Handle root files
   - **Effort**: Medium

4. Write tests
   - Test with mock git repo
   - Test package detection
   - Test commit addition
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Detects affected packages correctly
- [ ] Works in monorepo
- [ ] Works in single-package
- [ ] Handles various commit ranges
- [ ] Tests pass 100%
- [ ] Uses git tools correctly

**Definition of Done**:
- [ ] Git integration works
- [ ] Tests comprehensive
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 6.5: Changeset History and Archiving
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** to archive and query changesets  
**So that** I can track release history

**Description**:
Implement changeset archiving and history query functionality.

**Tasks**:
1. Create `src/changeset/history.rs`
   - Define `ChangesetHistory` struct
   - Implement query methods
   - **Effort**: Medium

2. Implement archive method
   - Create `ArchivedChangeset`
   - Add release info
   - Save to history
   - **Effort**: Low

3. Implement query methods
   - Query by date range
   - Query by package
   - Query by environment
   - Query by bump type
   - **Effort**: Medium

4. Write tests
   - Test archiving
   - Test queries
   - Test with multiple archives
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Archiving works correctly
- [ ] Queries return correct results
- [ ] History is queryable
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] History and archiving complete
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

## Epic 7: Changes Analysis

**Phase**: 2  
**Total Effort**: High  
**Dependencies**: Epic 2, Epic 3, Epic 4, Epic 6  
**Goal**: Implement git-based changes analysis system

### Story 7.1: Changes Analyzer Foundation
**Effort**: Medium  
**Priority**: Critical

**As a** developer  
**I want** a changes analyzer structure  
**So that** I can analyze git changes

**Description**:
Implement the main `ChangesAnalyzer` struct with git and monorepo integration.

**Tasks**:
1. Create `src/changes/analyzer.rs`
   - Define `ChangesAnalyzer` struct
   - Add workspace_root, git_repo, monorepo_detector, fs
   - Implement constructor
   - **Effort**: Low

2. Setup git integration
   - Initialize `GitRepository` from git tools
   - Validate git repo
   - **Effort**: Low

3. Setup monorepo integration
   - Initialize `MonorepoDetector`
   - Detect project type
   - **Effort**: Low

4. Write initialization tests
   - Test with valid repo
   - Test with invalid repo
   - Test error cases
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Analyzer initializes correctly
- [ ] Git integration works
- [ ] Monorepo detection works
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Foundation complete
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 7.2: File-to-Package Mapping
**Effort**: High  
**Priority**: Critical

**As a** developer  
**I want** to map changed files to packages  
**So that** I know which packages were affected

**Description**:
Implement logic to map file paths to their owning packages in a monorepo.

**Tasks**:
1. Create `src/changes/mapping.rs`
   - Implement mapping algorithm
   - Handle monorepo packages
   - Handle single-package
   - **Effort**: High

2. Implement package ownership detection
   - Check if file is under package path
   - Handle root files
   - Handle shared files
   - **Effort**: Medium

3. Add caching for performance
   - Cache package paths
   - Cache mapping results
   - **Effort**: Low

4. Write tests
   - Test with various file paths
   - Test in monorepo
   - Test in single-package
   - Test edge cases
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Maps files correctly
- [ ] Works in monorepo
- [ ] Works in single-package
- [ ] Handles edge cases
- [ ] Performance acceptable
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Mapping logic complete
- [ ] Tests comprehensive
- [ ] Performance verified

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 7.3: Working Directory Analysis
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** to analyze uncommitted changes  
**So that** I can see what's changed before committing

**Description**:
Implement analysis of working directory changes (staged and unstaged).

**Tasks**:
1. Implement working directory analysis
   - Use git status from git tools
   - Get all changed files
   - Map to packages
   - **Effort**: Medium

2. Create ChangesReport
   - Aggregate by package
   - Calculate statistics
   - **Effort**: Low

3. Write tests
   - Test with staged changes
   - Test with unstaged changes
   - Test with both
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Detects working directory changes
- [ ] Maps to packages correctly
- [ ] Report accurate
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Working directory analysis works
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 7.4: Commit Range Analysis
**Effort**: High  
**Priority**: Critical

**As a** developer  
**I want** to analyze commit ranges  
**So that** I can see changes between branches/tags

**Description**:
Implement analysis of changes between two git references (commits, branches, tags).

**Tasks**:
1. Implement commit range analysis
   - Parse git references
   - Get commits in range using git tools
   - Get file changes for each commit
   - **Effort**: High

2. Implement commit-to-package association
   - For each commit, find affected packages
   - Associate commits with packages
   - Handle multi-package commits
   - **Effort**: Medium

3. Create detailed report
   - Group by package
   - Include commit details
   - Calculate statistics
   - **Effort**: Low

4. Write tests
   - Test various commit ranges
   - Test branch comparison
   - Test tag comparison
   - Test multi-package commits
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Analyzes commit ranges correctly
- [ ] Associates commits with packages
- [ ] Handles multi-package commits
- [ ] Report detailed and accurate
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Commit range analysis works
- [ ] Tests comprehensive
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 7.5: Version Preview Calculation
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** to see current and next versions  
**So that** I can preview version changes

**Description**:
Add version preview calculation based on changeset bump type.

**Tasks**:
1. Implement version calculation
   - Get current version from package.json
   - Apply changeset bump
   - Calculate next version
   - **Effort**: Low

2. Add to analysis report
   - Include current_version
   - Include next_version
   - Include bump_type
   - **Effort**: Low

3. Integrate with version resolver
   - Use version bumping logic
   - Ensure consistency
   - **Effort**: Low

4. Write tests
   - Test with different bumps
   - Test with various current versions
   - Test version calculation accuracy
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Calculates versions correctly
- [ ] Shows in report
- [ ] Consistent with version resolver
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Version preview works
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 7.6: Changes Statistics
**Effort**: Low  
**Priority**: Medium

**As a** developer  
**I want** statistics on changes  
**So that** I can understand the scope of changes

**Description**:
Add comprehensive statistics calculation for changes.

**Tasks**:
1. Create `src/changes/stats.rs`
   - Implement statistics calculation
   - Count files by change type
   - Count lines changed
   - Count commits
   - **Effort**: Low

2. Add to report structures
   - Add stats to `PackageChanges`
   - Add stats to `ChangesReport`
   - **Effort**: Minimal

3. Write tests
   - Test stat calculation
   - Test with various changes
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Statistics accurate
- [ ] All metrics calculated
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Statistics complete
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

## Epic 8: Changelog Generation

**Phase**: 3  
**Total Effort**: Massive  
**Dependencies**: Epic 2, Epic 3, Epic 4, Epic 6, Epic 7  
**Goal**: Implement complete changelog generation system

### Story 8.1: Conventional Commit Parser
**Effort**: High  
**Priority**: Critical

**As a** developer  
**I want** to parse conventional commits  
**So that** I can group changes by type

**Description**:
Implement a parser for conventional commit messages following the specification.

**Tasks**:
1. Create `src/changelog/conventional.rs`
   - Define `ConventionalCommit` struct
   - Implement parser with regex
   - Extract type, scope, description
   - **Effort**: Medium

2. Implement breaking change detection
   - Check for `!` after type/scope
   - Check for `BREAKING CHANGE:` in footer
   - **Effort**: Low

3. Implement footer parsing
   - Parse key-value footers
   - Extract references (#123)
   - **Effort**: Medium

4. Add section type mapping
   - Map commit type to section (feat → Features)
   - Handle unknown types
   - **Effort**: Low

5. Write comprehensive tests
   - Test valid formats
   - Test invalid formats
   - Test breaking changes
   - Test footers
   - Test edge cases
   - Property-based tests
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Parses all valid conventional commits
- [ ] Rejects invalid formats gracefully
- [ ] Detects breaking changes correctly
- [ ] Extracts references (#123, closes #456)
- [ ] Maps to correct sections
- [ ] Tests pass 100%
- [ ] Property tests pass
- [ ] Follows specification exactly

**Definition of Done**:
- [ ] Parser complete
- [ ] All tests pass
- [ ] Documentation with examples
- [ ] Specification compliance verified

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 8.2: Changelog Generator Foundation
**Effort**: Medium  
**Priority**: Critical

**As a** developer  
**I want** a changelog generator structure  
**So that** I can generate changelogs

**Description**:
Implement the main `ChangelogGenerator` struct with git and config integration.

**Tasks**:
1. Create `src/changelog/generator.rs`
   - Define `ChangelogGenerator` struct
   - Add workspace_root, git_repo, fs, config
   - Implement constructor
   - **Effort**: Low

2. Setup git integration
   - Initialize git repository
   - Add tag detection methods
   - **Effort**: Low

3. Setup configuration
   - Load changelog config
   - Validate settings
   - **Effort**: Low

4. Write initialization tests
   - Test construction
   - Test with various configs
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Generator initializes
- [ ] Git integration works
- [ ] Config loaded correctly
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Foundation complete
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 8.3: Version Detection from Git Tags
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** to detect previous versions from tags  
**So that** I can generate changelogs automatically

**Description**:
Implement logic to detect and parse version tags from git.

**Tasks**:
1. Implement tag detection
   - List all tags using git tools
   - Filter version tags
   - **Effort**: Low

2. Implement tag parsing
   - Parse monorepo tags (@pkg/name@1.0.0)
   - Parse root tags (v1.0.0)
   - Handle custom tag formats
   - **Effort**: Medium

3. Implement previous version detection
   - Find latest tag before current version
   - Handle no previous version
   - **Effort**: Low

4. Write tests
   - Test tag parsing
   - Test version detection
   - Test with various tag formats
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Detects tags correctly
- [ ] Parses monorepo tags
- [ ] Finds previous version
- [ ] Handles missing tags
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Tag detection works
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 8.4: Changelog Data Collection
**Effort**: High  
**Priority**: Critical

**As a** developer  
**I want** to collect changelog data from commits  
**So that** I can build a changelog

**Description**:
Implement logic to collect all data needed for a changelog from git commits.

**Tasks**:
1. Implement commit collection
   - Get commits between versions
   - Filter by package (monorepo)
   - **Effort**: Medium

2. Implement commit parsing
   - Parse each commit message
   - Try conventional commits first
   - Fallback to plain message
   - **Effort**: Low

3. Implement grouping by section
   - Group by commit type
   - Separate breaking changes
   - Sort within groups
   - **Effort**: Medium

4. Add metadata collection
   - Collect commit hashes
   - Collect authors
   - Collect dates
   - Extract references
   - **Effort**: Low

5. Write tests
   - Test commit collection
   - Test parsing
   - Test grouping
   - Test with various commits
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Collects all commits
- [ ] Parses correctly
- [ ] Groups by section
- [ ] Metadata complete
- [ ] Works in monorepo
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Data collection complete
- [ ] Tests comprehensive
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 8.5: Keep a Changelog Formatter
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** to format changelogs in Keep a Changelog format  
**So that** I follow community standards

**Description**:
Implement formatter for the Keep a Changelog format.

**Tasks**:
1. Create `src/changelog/formatter/keep_a_changelog.rs`
   - Implement formatter
   - Add version header
   - Add sections
   - Format entries
   - **Effort**: Medium

2. Add linking
   - Add commit links
   - Add issue links
   - **Effort**: Low

3. Write tests
   - Test formatting
   - Test with various data
   - Test output format
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Generates valid Keep a Changelog format
- [ ] Includes all sections
- [ ] Links work
- [ ] Tests pass 100%
- [ ] Follows specification

**Definition of Done**:
- [ ] Formatter complete
- [ ] Tests pass
- [ ] Specification compliance verified

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 8.6: Conventional Commits Formatter
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** to format changelogs with conventional grouping  
**So that** changes are organized by type

**Description**:
Implement formatter that groups changes by conventional commit type.

**Tasks**:
1. Create `src/changelog/formatter/conventional.rs`
   - Implement formatter
   - Group by type (feat, fix, etc.)
   - Add breaking changes section first
   - **Effort**: Medium

2. Add section titles
   - Use configured section titles
   - Handle custom types
   - **Effort**: Low

3. Write tests
   - Test grouping
   - Test section order
   - Test with various commits
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Groups by commit type
- [ ] Breaking changes first
- [ ] Titles configurable
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Formatter complete
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 8.7: Custom Template Formatter
**Effort**: Low  
**Priority**: Medium

**As a** developer  
**I want** custom changelog templates  
**So that** I can match my organization's style

**Description**:
Implement formatter that uses custom templates from configuration.

**Tasks**:
1. Create `src/changelog/formatter/custom.rs`
   - Implement template renderer
   - Replace variables
   - **Effort**: Low

2. Add template variables
   - {version}, {date}, {title}, {description}, etc.
   - **Effort**: Minimal

3. Write tests
   - Test rendering
   - Test with various templates
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Renders templates correctly
- [ ] All variables replaced
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Template formatter complete
- [ ] Tests pass
- [ ] Documentation with examples

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 8.8: Changelog File Management
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** to update CHANGELOG.md files  
**So that** changelogs persist

**Description**:
Implement logic to create, update, and parse CHANGELOG.md files.

**Tasks**:
1. Implement changelog creation
   - Create new CHANGELOG.md
   - Add header
   - Add first version
   - **Effort**: Low

2. Implement changelog updating
   - Parse existing CHANGELOG.md
   - Prepend new version
   - Preserve existing content
   - **Effort**: Medium

3. Implement existing changelog parser
   - Create `src/changelog/parser.rs`
   - Parse version sections
   - Extract dates
   - **Effort**: Medium

4. Add dry-run support
   - Return content without writing
   - **Effort**: Minimal

5. Write tests
   - Test creation
   - Test updating
   - Test parsing
   - Test dry-run
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Creates new changelogs
- [ ] Updates existing changelogs
- [ ] Preserves existing content
- [ ] Parser works correctly
- [ ] Dry-run doesn't write
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] File management complete
- [ ] Tests comprehensive
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 8.9: Merge Commit Message Generation
**Effort**: Low  
**Priority**: Medium

**As a** developer  
**I want** to generate merge commit messages  
**So that** releases have consistent commit messages

**Description**:
Implement merge commit message generation using configured templates.

**Tasks**:
1. Implement message generation
   - Load template from config
   - Replace variables
   - Add breaking changes warning if needed
   - **Effort**: Low

2. Add variable replacement
   - {version}, {package_name}, {changelog_summary}, etc.
   - **Effort**: Low

3. Write tests
   - Test generation
   - Test with various templates
   - Test breaking changes
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Generates messages correctly
- [ ] All variables replaced
- [ ] Breaking changes included
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Message generation works
- [ ] Tests pass
- [ ] Documentation with examples

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 8.10: Generate from Changeset
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** to generate changelogs from changesets  
**So that** I can automate changelog creation

**Description**:
Implement high-level method to generate changelogs from changeset and version resolution.

**Tasks**:
1. Implement changeset integration
   - Accept changeset and version resolution
   - Generate changelog for each package
   - **Effort**: Medium

2. Handle monorepo and single-package
   - Generate per-package in monorepo
   - Generate root in single-package
   - Respect configuration
   - **Effort**: Low

3. Write integration tests
   - Test with real changeset
   - Test with version resolution
   - Test both project types
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Generates from changeset
- [ ] Works for monorepo
- [ ] Works for single-package
- [ ] Integration tests pass
- [ ] 100% coverage

**Definition of Done**:
- [ ] Integration complete
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

## Epic 9: Dependency Upgrades

**Phase**: 3  
**Total Effort**: High  
**Dependencies**: Epic 2, Epic 3, Epic 4, Epic 6  
**Goal**: Implement dependency upgrade detection and application

### Story 9.1: Registry Client Foundation
**Effort**: Medium  
**Priority**: Critical

**As a** developer  
**I want** a registry client  
**So that** I can query package versions

**Description**:
Implement HTTP client for querying npm registry and private registries.

**Tasks**:
1. Create `src/upgrade/registry/client.rs`
   - Define `RegistryClient` struct
   - Setup HTTP client (reqwest)
   - Add retry logic
   - **Effort**: Medium

2. Implement package metadata query
   - GET package info from registry
   - Parse response
   - Extract versions and metadata
   - **Effort**: Medium

3. Add error handling
   - Handle network errors
   - Handle 404s
   - Handle timeouts
   - **Effort**: Low

4. Write tests with mock server
   - Setup mock HTTP server (mockito)
   - Test successful queries
   - Test error cases
   - Test retries
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Client queries registry successfully
- [ ] Handles private registries
- [ ] Retry logic works
- [ ] Error handling comprehensive
- [ ] Tests pass 100%
- [ ] Mock server used for tests

**Definition of Done**:
- [ ] Registry client works
- [ ] Tests comprehensive
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 9.2: .npmrc Parsing and Configuration
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** to read .npmrc files  
**So that** I respect existing registry configuration

**Description**:
Implement .npmrc file parsing to extract registry URLs and authentication tokens.

**Tasks**:
1. Create `src/upgrade/registry/npmrc.rs`
   - Define `NpmrcConfig` struct
   - Implement parser
   - **Effort**: Medium

2. Implement parsing logic
   - Parse registry URLs
   - Parse scoped registries
   - Parse auth tokens
   - Handle comments
   - **Effort**: Medium

3. Implement resolution
   - Resolve registry for package
   - Resolve auth token for registry
   - **Effort**: Low

4. Write tests
   - Test various .npmrc formats
   - Test scoped packages
   - Test auth tokens
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Parses .npmrc correctly
- [ ] Extracts registries and auth
- [ ] Handles scoped packages
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Parser complete
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 9.3: Upgrade Detection
**Effort**: High  
**Priority**: Critical

**As a** developer  
**I want** to detect available upgrades  
**So that** I know what can be updated

**Description**:
Implement logic to detect upgrades for all external dependencies.

**Tasks**:
1. Create `src/upgrade/detection.rs`
   - Implement upgrade detection
   - Query registry for each dependency
   - Compare versions
   - **Effort**: High

2. Implement version comparison
   - Determine upgrade type (major, minor, patch)
   - Use semver crate
   - **Effort**: Low

3. Add concurrent queries
   - Query multiple packages in parallel
   - Add configurable concurrency limit
   - **Effort**: Medium

4. Add filtering
   - Filter internal dependencies
   - Filter workspace protocols
   - Filter by dependency type
   - **Effort**: Low

5. Write tests
   - Test detection
   - Test with various dependencies
   - Test filtering
   - Test concurrency
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Detects all external upgrades
- [ ] Classifies upgrade types correctly
- [ ] Concurrent queries work
- [ ] Filtering accurate
- [ ] Performance acceptable
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Detection complete
- [ ] Tests comprehensive
- [ ] Performance verified

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 9.4: Upgrade Application
**Effort**: High  
**Priority**: Critical

**As a** developer  
**I want** to apply upgrades  
**So that** dependencies are updated

**Description**:
Implement logic to apply selected upgrades to package.json files.

**Tasks**:
1. Create `src/upgrade/apply.rs`
   - Implement apply logic
   - Update package.json files
   - Preserve formatting
   - **Effort**: High

2. Add dry-run support
   - Return changes without writing
   - **Effort**: Low

3. Implement selection filtering
   - Apply only selected upgrades
   - Filter by upgrade type
   - Filter by package
   - **Effort**: Medium

4. Write tests
   - Test application
   - Test dry-run
   - Test filtering
   - Test file preservation
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Applies upgrades correctly
- [ ] Dry-run works
- [ ] Preserves JSON formatting
- [ ] Selection filtering works
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Application complete
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 9.5: Backup and Rollback
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** automatic backup and rollback  
**So that** I can recover from failed upgrades

**Description**:
Implement backup system that creates snapshots before applying upgrades.

**Tasks**:
1. Create `src/upgrade/backup.rs`
   - Define backup structure
   - Implement backup creation
   - Store in .sublime/backups/
   - **Effort**: Medium

2. Implement rollback
   - Restore files from backup
   - Handle partial failures
   - **Effort**: Medium

3. Add backup management
   - List backups
   - Clean old backups
   - Respect max_backups config
   - **Effort**: Low

4. Write tests
   - Test backup creation
   - Test rollback
   - Test cleanup
   - Test with failures
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Creates backups before apply
- [ ] Rollback restores correctly
- [ ] Backup management works
- [ ] Tests pass 100%
- [ ] No data loss

**Definition of Done**:
- [ ] Backup system complete
- [ ] Rollback verified
- [ ] Tests comprehensive

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 9.6: Automatic Changeset Creation
**Effort**: Low  
**Priority**: Medium

**As a** developer  
**I want** changesets created automatically after upgrades  
**So that** upgrades are tracked

**Description**:
Implement automatic changeset creation when upgrades are applied.

**Tasks**:
1. Integrate with changeset manager
   - Create changeset after apply
   - Add affected packages
   - Set bump type to patch
   - **Effort**: Low

2. Make it configurable
   - Respect auto_changeset config
   - Allow disabling
   - **Effort**: Minimal

3. Write tests
   - Test changeset creation
   - Test with config disabled
   - Test integration
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Creates changeset automatically
- [ ] Changeset correct (patch bump)
- [ ] Configurable
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Auto-creation works
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 9.7: Upgrade Manager Integration
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** a high-level upgrade manager  
**So that** I can use upgrades easily

**Description**:
Implement `UpgradeManager` that ties all upgrade functionality together.

**Tasks**:
1. Create `src/upgrade/manager.rs`
   - Define `UpgradeManager` struct
   - Implement all public methods
   - **Effort**: Medium

2. Write integration tests
   - Test complete upgrade workflow
   - Test with real fixtures
   - Test error recovery
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Manager provides clean API
- [ ] All features work together
- [ ] Integration tests pass
- [ ] Documentation complete

**Definition of Done**:
- [ ] Manager complete
- [ ] Tests pass
- [ ] Ready for use

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

## Epic 10: Audit & Health Checks

**Phase**: 4  
**Total Effort**: High  
**Dependencies**: All previous epics  
**Goal**: Implement comprehensive repository health checks

### Story 10.1: Audit Manager Foundation
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** an audit manager  
**So that** I can run health checks

**Description**:
Implement the main `AuditManager` struct that coordinates all audit sections.

**Tasks**:
1. Create `src/audit/manager.rs`
   - Define `AuditManager` struct
   - Add all subsystem references
   - Implement constructor
   - **Effort**: Low

2. Setup integrations
   - Initialize upgrade manager
   - Initialize changes analyzer
   - Setup other dependencies
   - **Effort**: Low

3. Write initialization tests
   - Test construction
   - Test with various configs
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Manager initializes
- [ ] All integrations work
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Foundation complete
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 10.2: Upgrade Audit Section
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** to audit available upgrades  
**So that** I know what needs updating

**Description**:
Implement upgrade audit section using the upgrade manager.

**Tasks**:
1. Create `src/audit/sections/upgrades.rs`
   - Use upgrade manager
   - Collect upgrade information
   - Create audit section
   - **Effort**: Medium

2. Add issue detection
   - Deprecated packages → Critical
   - Major upgrades → Warning
   - Minor/Patch → Info
   - **Effort**: Low

3. Write tests
   - Test with various upgrade scenarios
   - Test issue detection
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Detects upgrades correctly
- [ ] Issues have correct severity
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Upgrade audit complete
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 10.3: Dependency Audit Section
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** to audit dependencies  
**So that** I know about circular deps and conflicts

**Description**:
Implement dependency audit section with graph analysis.

**Tasks**:
1. Create `src/audit/sections/dependencies.rs`
   - Use dependency graph
   - Detect circular dependencies
   - Detect version conflicts
   - **Effort**: Medium

2. Add issue detection
   - Circular deps → Critical
   - Version conflicts → Warning
   - **Effort**: Low

3. Write tests
   - Test circular detection
   - Test conflict detection
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Detects circular dependencies
- [ ] Detects version conflicts
- [ ] Issues correct
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Dependency audit complete
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 10.4: Dependency Categorization
**Effort**: Medium  
**Priority**: Medium

**As a** developer  
**I want** dependencies categorized  
**So that** I understand my dependency structure

**Description**:
Implement dependency categorization into internal, external, workspace, local.

**Tasks**:
1. Create `src/audit/sections/categorization.rs`
   - Categorize all dependencies
   - List internal packages
   - List external packages
   - List workspace links
   - List local links
   - **Effort**: Medium

2. Add statistics
   - Count each category
   - Calculate percentages
   - **Effort**: Low

3. Write tests
   - Test categorization
   - Test with various projects
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Categorizes correctly
- [ ] All categories covered
- [ ] Statistics accurate
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Categorization complete
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 10.5: Breaking Changes Audit
**Effort**: Low  
**Priority**: Medium

**As a** developer  
**I want** to audit breaking changes  
**So that** I'm aware of them

**Description**:
Implement breaking changes audit using conventional commits and changesets.

**Tasks**:
1. Create `src/audit/sections/breaking_changes.rs`
   - Use changes analyzer
   - Detect breaking changes from commits
   - Create audit section
   - **Effort**: Low

2. Write tests
   - Test detection
   - Test with various scenarios
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Detects breaking changes
- [ ] Lists affected packages
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Breaking changes audit complete
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 10.6: Version Consistency Audit
**Effort**: Low  
**Priority**: Medium

**As a** developer  
**I want** to audit version consistency  
**So that** I know about inconsistent internal dependencies

**Description**:
Implement version consistency audit for internal dependencies.

**Tasks**:
1. Create `src/audit/sections/version_consistency.rs`
   - Check internal dependency versions
   - Find inconsistencies
   - **Effort**: Low

2. Add recommendations
   - Suggest consistent versions
   - **Effort**: Minimal

3. Write tests
   - Test consistency checking
   - Test recommendations
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Detects inconsistencies
- [ ] Recommendations clear
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Consistency audit complete
- [ ] Tests pass
- [ ] Documentation complete

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 10.7: Health Score Calculation
**Effort**: Medium  
**Priority**: Medium

**As a** developer  
**I want** a health score  
**So that** I have a single metric for repository health

**Description**:
Implement health score calculation based on all audit findings.

**Tasks**:
1. Create `src/audit/health_score.rs`
   - Define scoring algorithm
   - Weight different issues
   - Calculate 0-100 score
   - **Effort**: Medium

2. Add configurable weights
   - Allow customization
   - Provide sensible defaults
   - **Effort**: Low

3. Write tests
   - Test calculation
   - Test with various scenarios
   - Verify fairness
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Score calculated correctly
- [ ] Range is 0-100
- [ ] Weights configurable
- [ ] Tests pass 100%
- [ ] Algorithm documented

**Definition of Done**:
- [ ] Health score works
- [ ] Tests pass
- [ ] Algorithm explained in docs

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 10.8: Report Formatting
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** formatted audit reports  
**So that** I can read and share them

**Description**:
Implement report formatters for Markdown and JSON.

**Tasks**:
1. Create `src/audit/formatter.rs`
   - Implement Markdown formatter
   - Implement JSON formatter
   - **Effort**: Medium

2. Add formatting options
   - Color support
   - Verbosity levels
   - **Effort**: Low

3. Write tests
   - Test Markdown output
   - Test JSON output
   - Verify formats
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Markdown format readable
- [ ] JSON format valid
- [ ] Tests pass 100%
- [ ] Examples in documentation

**Definition of Done**:
- [ ] Formatters complete
- [ ] Tests pass
- [ ] Documentation with examples

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 10.9: Audit Integration Tests
**Effort**: High  
**Priority**: High

**As a** developer  
**I want** complete audit integration tests  
**So that** I verify the entire audit system

**Description**:
Write comprehensive integration tests for the complete audit workflow.

**Tasks**:
1. Create integration test fixtures
   - Setup test projects
   - Add various issues
   - **Effort**: Medium

2. Write workflow tests
   - Test complete audit
   - Test with real data
   - Test all sections
   - **Effort**: High

3. Write performance tests
   - Verify acceptable performance
   - Test with large monorepos
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Full audit tested
- [ ] All sections work
- [ ] Performance acceptable
- [ ] Tests pass 100%

**Definition of Done**:
- [ ] Integration tests complete
- [ ] Performance verified
- [ ] Ready for production

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

## Epic 11: Integration & Documentation

**Phase**: 4  
**Total Effort**: High  
**Dependencies**: All previous epics  
**Goal**: Complete integration testing and documentation

### Story 11.1: End-to-End Workflow Tests
**Effort**: High  
**Priority**: Critical

**As a** developer  
**I want** E2E workflow tests  
**So that** I verify complete release workflows

**Description**:
Write comprehensive end-to-end tests that verify complete release workflows from changeset creation to archiving.

**Tasks**:
1. Create E2E test scenarios
   - Complete release workflow
   - Monorepo release
   - Single-package release
   - Upgrade workflow
   - **Effort**: High

2. Write workflow tests
   - Test each scenario
   - Verify all integrations
   - **Effort**: Massive

3. Add CI integration
   - Run E2E tests in CI
   - Add to test suite
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] All workflows tested
- [ ] Tests pass consistently
- [ ] CI integration works
- [ ] Coverage meets requirements

**Definition of Done**:
- [ ] E2E tests complete
- [ ] All passing in CI
- [ ] Documentation updated

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 11.2: API Documentation
**Effort**: High  
**Priority**: Critical

**As a** user  
**I want** complete API documentation  
**So that** I can use the library effectively

**Description**:
Complete all API documentation with examples for every public item.

**Tasks**:
1. Review and complete module docs
   - Ensure What/How/Why for each module
   - Add examples
   - **Effort**: High

2. Complete function documentation
   - Document all parameters
   - Document return values
   - Document errors
   - Add examples
   - **Effort**: Massive

3. Generate and review docs
   - Run `cargo doc`
   - Review generated docs
   - Fix issues
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] 100% documentation coverage
- [ ] All examples compile
- [ ] All examples work
- [ ] `cargo doc` passes without warnings

**Definition of Done**:
- [ ] Documentation complete
- [ ] Examples verified
- [ ] Published to docs.rs

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 11.3: Usage Examples
**Effort**: Medium  
**Priority**: High

**As a** user  
**I want** runnable examples  
**So that** I can learn by doing

**Description**:
Create runnable examples in the `examples/` directory covering common use cases.

**Tasks**:
1. Create basic examples
   - Basic changeset usage
   - Version resolution
   - Changelog generation
   - **Effort**: Medium

2. Create advanced examples
   - Dependency upgrades
   - Audit reports
   - Complete workflows
   - **Effort**: Medium

3. Test all examples
   - Verify they compile
   - Verify they run
   - Add to CI
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] 6+ examples created
- [ ] All examples compile
- [ ] All examples run successfully
- [ ] Examples documented
- [ ] Examples in CI

**Definition of Done**:
- [ ] Examples complete
- [ ] All working
- [ ] CI integration done

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 11.4: User Guides
**Effort**: High  
**Priority**: High

**As a** user  
**I want** comprehensive guides  
**So that** I can learn the library

**Description**:
Create user guides covering common scenarios and best practices.

**Tasks**:
1. Create getting started guide
   - Installation
   - Basic usage
   - First changeset
   - **Effort**: Medium

2. Create monorepo guide
   - Monorepo setup
   - Managing multiple packages
   - Best practices
   - **Effort**: High

3. Create CI/CD integration guide
   - GitHub Actions example
   - GitLab CI example
   - Best practices
   - **Effort**: Medium

4. Create troubleshooting guide
   - Common issues
   - Solutions
   - FAQ
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] All guides complete
- [ ] Examples verified
- [ ] Clear and helpful
- [ ] Published

**Definition of Done**:
- [ ] Guides written
- [ ] Examples tested
- [ ] Published to docs/

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 11.5: README and Crate Metadata
**Effort**: Low  
**Priority**: High

**As a** user  
**I want** a great README  
**So that** I understand the project quickly

**Description**:
Create comprehensive README and update crate metadata.

**Tasks**:
1. Write README.md
   - Overview
   - Features
   - Quick start
   - Links
   - **Effort**: Medium

2. Update Cargo.toml metadata
   - Description
   - Keywords
   - Categories
   - Links
   - **Effort**: Minimal

3. Create CHANGELOG.md
   - Initial version entry
   - **Effort**: Minimal

**Acceptance Criteria**:
- [ ] README comprehensive
- [ ] Metadata complete
- [ ] CHANGELOG started
- [ ] Links work

**Definition of Done**:
- [ ] README complete
- [ ] Metadata updated
- [ ] Ready for publication

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 11.6: Performance Benchmarks
**Effort**: Medium  
**Priority**: Medium

**As a** developer  
**I want** performance benchmarks  
**So that** I can track performance

**Description**:
Create performance benchmarks for critical operations.

**Tasks**:
1. Setup criterion.rs
   - Add dependency
   - Configure benchmarks
   - **Effort**: Low

2. Create benchmarks
   - Version resolution
   - Dependency propagation
   - Changelog generation
   - **Effort**: Medium

3. Add to CI
   - Run benchmarks
   - Track performance
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Benchmarks created
- [ ] Baseline established
- [ ] CI integration works
- [ ] Performance acceptable

**Definition of Done**:
- [ ] Benchmarks complete
- [ ] CI tracking enabled
- [ ] Performance documented

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

### Story 11.7: Release Preparation
**Effort**: Low  
**Priority**: High

**As a** maintainer  
**I want** the crate ready for release  
**So that** I can publish v1.0.0

**Description**:
Final checks and preparation for v1.0.0 release.

**Tasks**:
1. Final quality checks
   - Run all tests
   - Check clippy
   - Check documentation
   - Check examples
   - **Effort**: Low

2. Version bump to 1.0.0
   - Update Cargo.toml
   - Update CHANGELOG.md
   - Tag release
   - **Effort**: Minimal

3. Pre-publish checks
   - Run `cargo publish --dry-run`
   - Verify package contents
   - **Effort**: Minimal

**Acceptance Criteria**:
- [ ] All tests pass
- [ ] 100% clippy compliance
- [ ] 100% documentation
- [ ] 100% test coverage
- [ ] Examples work
- [ ] Ready for publication

**Definition of Done**:
- [ ] All checks pass
- [ ] Version tagged
- [ ] Ready to publish

- [ ] Verify all the code if needs to be updated with the new implementation, looking for TODOS that are waiting for this implementation
---

## Summary

### Total Story Count
- **Epics**: 11
- **User Stories**: 67
- **Tasks**: 342+

### Effort Distribution

| Epic | Effort | Story Count |
|------|--------|-------------|
| Epic 1: Project Foundation | Medium | 3 |
| Epic 2: Configuration System | High | 3 |
| Epic 3: Error Handling | Medium | 2 |
| Epic 4: Core Types | Medium | 4 |
| Epic 5: Versioning Engine | Massive | 8 |
| Epic 6: Changeset Management | High | 5 |
| Epic 7: Changes Analysis | High | 6 |
| Epic 8: Changelog Generation | Massive | 10 |
| Epic 9: Dependency Upgrades | High | 7 |
| Epic 10: Audit & Health Checks | High | 9 |
| Epic 11: Integration & Documentation | High | 10 |

### Critical Path
1. Epic 1 → Epic 2 → Epic 3 → Epic 4 (Foundation - 3 weeks)
2. Epic 5 → Epic 6 → Epic 7 (Core - 4 weeks)
3. Epic 8 → Epic 9 (Advanced - 3 weeks)
4. Epic 10 → Epic 11 (Polish - 2 weeks)

**Total Timeline**: 12 weeks

---

## How to Use This Story Map

### For Planning
1. Review each epic in sequence
2. Break down stories into sprint-sized chunks
3. Assign stories to developers
4. Track progress using acceptance criteria

### For Development
1. Pick a story from the current epic
2. Review tasks and effort estimate
3. Implement following quality standards
4. Verify all acceptance criteria
5. Complete definition of done

### For Review
1. Check all acceptance criteria met
2. Verify tests pass with 100% coverage
3. Verify clippy passes
4. Verify documentation complete
5. Approve and merge

---

**STORY_MAP.md STATUS**: ✅ **COMPLETE** - Ready for sprint planning