# Workspace Node Tools - Node.js Bindings Development Story Map

**Version**: 1.0  
**Based on**: PLAN_NODE.md v1.0 & NAPI_RESEARCH.md v1.0  
**Last Updated**: 2025-01-20  
**Status**: 📋 Ready for Development

> **📝 Note:** This story map covers the implementation of Node.js bindings via napi-rs for the workspace-tools CLI. It exposes 23 `execute_*` functions as native JavaScript/TypeScript functions.

---

## Table of Contents

1. [Story Map Overview](#story-map-overview)
2. [Effort Metrics Definition](#effort-metrics-definition)
3. [Epic 1: NAPI Foundation](#epic-1-napi-foundation)
4. [Epic 2: Error & Validation System](#epic-2-error--validation-system)
5. [Epic 3: POC Commands (Status & Init)](#epic-3-poc-commands-status--init)
6. [Epic 4: Changeset Commands](#epic-4-changeset-commands)
7. [Epic 5: Bump Commands](#epic-5-bump-commands)
8. [Epic 6: Execute Command with Timeout](#epic-6-execute-command-with-timeout)
9. [Epic 7: Config Commands](#epic-7-config-commands)
10. [Epic 8: Upgrade Commands](#epic-8-upgrade-commands)
11. [Epic 9: Remaining Commands](#epic-9-remaining-commands)
12. [Epic 10: Testing & Quality](#epic-10-testing--quality)
13. [Epic 11: Documentation & Release](#epic-11-documentation--release)
14. [Summary](#summary)
15. [How to Use This Story Map](#how-to-use-this-story-map)

---

## Story Map Overview

### Epic Breakdown

```
Phase 1: Foundation (Week 1)
├── Epic 1: NAPI Foundation
└── Epic 2: Error & Validation System

Phase 2: POC (Week 2)
└── Epic 3: POC Commands (Status & Init)

Phase 3: Core Commands (Weeks 3-4)
├── Epic 4: Changeset Commands
└── Epic 5: Bump Commands

Phase 4: Execute & Config (Week 5)
├── Epic 6: Execute Command with Timeout
└── Epic 7: Config Commands

Phase 5: Remaining Commands (Weeks 5-6)
├── Epic 8: Upgrade Commands
└── Epic 9: Remaining Commands (Audit, Changes, Clone)

Phase 6: Polish & Release (Weeks 7-8)
├── Epic 10: Testing & Quality
└── Epic 11: Documentation & Release
```

### Total Story Count

- **Epics**: 11
- **User Stories**: 45
- **Functions**: 23 NAPI bindings

### Architectural Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Return Type | JS Objects via `#[napi(object)]` | Type-safe, no `JSON.parse()`, superior DX |
| Error Handling | `ApiResponse<T>` + `ErrorInfo` | Consistent, programmatic, Node.js style |
| Validation | NAPI layer (fail fast) | Clear errors before CLI invocation |
| Timeout (Execute) | Config default + parameter override | Maximum flexibility |
| CLI-only params | Fixed values (`non_interactive=true`) | API is programmatic, no prompts |

---

## Effort Metrics Definition

### Effort Levels

| Level | Time Estimate | Complexity | Examples |
|-------|--------------|------------|----------|
| **Minimal** | 1-2 hours | Trivial | Simple struct, basic type, straightforward test |
| **Low** | 3-6 hours | Simple | Single NAPI function, basic validation, simple parsing |
| **Medium** | 1-2 days | Moderate | Complex command, comprehensive testing, multi-type response |
| **High** | 3-5 days | Complex | Multi-step workflow, timeout handling, extensive edge cases |
| **Massive** | 1-2 weeks | Very Complex | Complete command suite, E2E testing, cross-platform validation |

### Estimation Guidelines

**Minimal (1-2h)**:
- Creating simple NAPI structs with `#[napi(object)]`
- Adding basic type exports
- Writing straightforward unit tests
- Simple documentation updates

**Low (3-6h)**:
- Implementing single NAPI function with clear flow
- Creating validation for simple parameters
- Writing unit tests for single function
- Adding function-level documentation

**Medium (1-2d)**:
- Implementing commands with complex response types
- Creating comprehensive validation logic
- Writing Node.js integration tests
- Full TypeScript type verification

**High (3-5d)**:
- Implementing commands with timeout/async complexity
- CLI output parsing and conversion
- Full test coverage with edge cases
- Performance optimization

**Massive (1-2w)**:
- Complete command subsystem (e.g., all changeset commands)
- Cross-platform testing
- E2E testing scenarios
- Comprehensive documentation and examples

---

## Epic 1: NAPI Foundation

**Phase**: 1  
**Total Effort**: High  
**Dependencies**: None  
**Goal**: Establish NAPI crate structure with build configuration

### Story 1.1: Create Crate Structure
**Effort**: Low  
**Priority**: Critical

**As a** developer  
**I want** the basic NAPI crate structure created  
**So that** I can start implementing Node.js bindings

**Description**:
Set up the foundational crate structure with proper dependencies, clippy rules, and module scaffolding.

**Tasks**:
1. Create `crates/node/` directory structure
   - Create `Cargo.toml` with all dependencies
   - Create `build.rs` for napi-build
   - Create `src/lib.rs` entry point
   - Create module directories
   - **Effort**: Low

2. Configure `Cargo.toml`
   - Add napi and napi-derive dependencies (v3.0.0)
   - Add workspace crate dependencies (cli, git, pkg, standard)
   - Add serde and serde_json for parsing
   - Add tokio with full features including time
   - Configure crate-type as cdylib
   - Add clippy lints configuration
   - **Effort**: Low

3. Create `build.rs`
   - Call `napi_build::setup()`
   - Add proper documentation
   - **Effort**: Minimal

4. Create module structure
   - Create `src/error.rs` placeholder
   - Create `src/validation.rs` placeholder
   - Create `src/response.rs` placeholder
   - Create `src/types/mod.rs` and submodules
   - Create `src/commands/mod.rs` and submodules
   - **Effort**: Minimal

5. Update workspace `Cargo.toml`
   - Add `crates/node` to members
   - **Effort**: Minimal

**Acceptance Criteria**:
- [ ] `Cargo.toml` contains all required dependencies
- [ ] Project compiles without errors
- [ ] `cargo fmt` runs successfully
- [ ] `cargo clippy` passes without warnings
- [ ] Module structure follows PLAN_NODE.md
- [ ] `build.rs` executes correctly

**Definition of Done**:
- [ ] Crate compiles
- [ ] Clippy 100%
- [ ] Basic structure documented
- [ ] PR approved and merged

**Dependencies**: None

**Blocked By**: None

**Blocks**: All other stories

---

### Story 1.2: Configure Package.json Build Scripts
**Effort**: Low  
**Priority**: Critical

**As a** developer  
**I want** the npm package build scripts configured  
**So that** I can build and test bindings locally

**Description**:
Verify and update the package.json scripts to point to the new crate location.

**Tasks**:
1. Verify `packages/workspace-tools/package.json` scripts
   - Ensure `build-binding` points to correct manifest path
   - Ensure output paths are correct
   - **Effort**: Minimal

2. Test build locally
   - Run `pnpm build-binding`
   - Verify `.node` file is generated
   - Verify `binding.d.ts` is generated
   - Verify `binding.js` is generated
   - **Effort**: Low

3. Update `.npmrc` if needed
   - Ensure proper registry configuration
   - **Effort**: Minimal

**Acceptance Criteria**:
- [ ] `pnpm build-binding` succeeds
- [ ] Native `.node` binary is generated
- [ ] TypeScript definitions are generated
- [ ] JavaScript wrapper is generated

**Definition of Done**:
- [ ] Build works locally
- [ ] All generated files in correct locations
- [ ] Scripts documented

**Dependencies**: Story 1.1

**Blocked By**: Story 1.1

**Blocks**: Story 3.1

---

### Story 1.3: Implement lib.rs Entry Point
**Effort**: Low  
**Priority**: Critical

**As a** developer  
**I want** the main lib.rs properly configured  
**So that** NAPI macros work and exports are available

**Description**:
Implement the main entry point with proper clippy rules, module imports, and NAPI macro setup.

**Tasks**:
1. Configure clippy rules
   - Add `#![warn(missing_docs)]`
   - Add `#![warn(rustdoc::missing_crate_level_docs)]`
   - Add `#![deny(unused_must_use)]`
   - Add `#![deny(clippy::unwrap_used)]`
   - Add `#![deny(clippy::expect_used)]`
   - Add `#![deny(clippy::todo)]`
   - Add `#![deny(clippy::unimplemented)]`
   - Add `#![deny(clippy::panic)]`
   - **Effort**: Minimal

2. Add NAPI macro
   - Add `#[macro_use] extern crate napi_derive;`
   - **Effort**: Minimal

3. Define module structure
   - Add `mod error;`
   - Add `mod validation;`
   - Add `mod response;`
   - Add `pub(crate) mod types;`
   - Add `pub(crate) mod commands;`
   - **Effort**: Minimal

4. Add crate-level documentation
   - Document What, How, Why
   - Add usage examples
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] lib.rs compiles with all clippy rules
- [ ] NAPI macro available
- [ ] All modules declared
- [ ] Crate documentation present

**Definition of Done**:
- [ ] Code compiles
- [ ] Clippy 100%
- [ ] Documentation complete

**Dependencies**: Story 1.1

**Blocked By**: Story 1.1

**Blocks**: Epic 2

---

## Epic 2: Error & Validation System

**Phase**: 1  
**Total Effort**: High  
**Dependencies**: Epic 1  
**Goal**: Implement error handling, validation, and response wrapper

### Story 2.1: Implement ErrorInfo Structure
**Effort**: Medium  
**Priority**: Critical

**As a** developer  
**I want** a structured error type for NAPI  
**So that** JavaScript consumers receive consistent error information

**Description**:
Implement the `ErrorInfo` struct with Node.js-style error codes and conversion from CLI errors.

**Tasks**:
1. Create `src/error.rs`
   - Add module documentation (What, How, Why)
   - **Effort**: Minimal

2. Define ErrorInfo struct
   - Add `code: String` (e.g., "EVALIDATION", "EGIT")
   - Add `message: String`
   - Add `context: Option<String>`
   - Add `kind: String`
   - Use `#[napi(object)]` derive
   - Document each field
   - **Effort**: Low

3. Define error code constants
   - `ECONFIG` for Configuration errors
   - `EVALIDATION` for Validation errors
   - `EEXEC` for Execution errors
   - `EGIT` for Git errors
   - `EPKG` for Package errors
   - `ENOENT` for File not found
   - `EIO` for I/O errors
   - `ENETWORK` for Network errors
   - `EUSER` for User cancelled
   - `ETIMEOUT` for Timeout errors
   - **Effort**: Low

4. Implement `cli_error_to_info` conversion
   - Match on `CliError` variants
   - Map to appropriate error codes
   - Extract message and context
   - **Effort**: Medium

5. Write unit tests
   - Test each error code mapping
   - Test message formatting
   - Test context extraction
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] ErrorInfo has `#[napi(object)]` derive
- [ ] All 10 error codes defined
- [ ] CliError converts correctly to ErrorInfo
- [ ] TypeScript types generate correctly
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Code compiles
- [ ] Clippy 100%
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 1.3

**Blocked By**: Story 1.3

**Blocks**: Story 2.3

---

### Story 2.2: Implement Validation System
**Effort**: Medium  
**Priority**: Critical

**As a** developer  
**I want** parameter validators  
**So that** invalid inputs are caught before CLI invocation

**Description**:
Implement validation utilities for common parameter patterns.

**Tasks**:
1. Create `src/validation.rs`
   - Add module documentation
   - **Effort**: Minimal

2. Define ValidationError struct
   - Add `field: String`
   - Add `message: String`
   - Add `value: Option<String>`
   - **Effort**: Low

3. Implement `From<ValidationError> for ErrorInfo`
   - Convert to EVALIDATION code
   - Format message with field name
   - **Effort**: Low

4. Implement validators module
   - `path_exists(path: &str) -> Result<(), ValidationError>`
   - `not_empty(field: &str, value: &str) -> Result<(), ValidationError>`
   - `bump_type(value: &str) -> Result<(), ValidationError>`
   - `timeout(field: &str, value: u64, min: u64, max: u64) -> Result<(), ValidationError>`
   - **Effort**: Medium

5. Write unit tests
   - Test path_exists with valid/invalid paths
   - Test not_empty with empty/non-empty strings
   - Test bump_type with valid/invalid values
   - Test timeout with boundary values
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] ValidationError defined
- [ ] All validators implemented
- [ ] Validators return clear error messages
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Code compiles
- [ ] Clippy 100%
- [ ] Tests pass
- [ ] Documentation complete

**Dependencies**: Story 2.1

**Blocked By**: Story 2.1

**Blocks**: Story 3.1

---

### Story 2.3: Implement ApiResponse Wrapper
**Effort**: Medium  
**Priority**: Critical

**As a** developer  
**I want** a consistent response wrapper  
**So that** all NAPI functions return the same structure

**Description**:
Implement the `ApiResponse<T>` wrapper that encapsulates success/failure states.

**Tasks**:
1. Create `src/response.rs`
   - Add module documentation
   - **Effort**: Minimal

2. Define ApiResponse struct
   - Add `success: bool`
   - Add `data: Option<T>`
   - Add `error: Option<ErrorInfo>`
   - Use `#[napi(object)]` derive
   - Document each field
   - **Effort**: Low

3. Implement helper methods
   - `fn success(data: T) -> Self`
   - `fn failure(error: ErrorInfo) -> Self`
   - `fn failure_from_io(error: std::io::Error) -> Self`
   - `fn failure_from_cli(error: CliError) -> Self`
   - **Effort**: Medium

4. Verify TypeScript generation
   - Build and check `binding.d.ts`
   - Ensure generic types generate correctly
   - **Effort**: Low

5. Write unit tests
   - Test success construction
   - Test failure construction
   - Test helper methods
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] ApiResponse has `#[napi(object)]` derive
- [ ] Helper methods implemented
- [ ] TypeScript types generate correctly
- [ ] 100% test coverage

**Definition of Done**:
- [ ] Code compiles
- [ ] Clippy 100%
- [ ] Tests pass
- [ ] TypeScript verified

**Dependencies**: Story 2.1

**Blocked By**: Story 2.1

**Blocks**: Story 3.1

---

## Epic 3: POC Commands (Status & Init)

**Phase**: 2  
**Total Effort**: High  
**Dependencies**: Epic 2  
**Goal**: Implement status and init as proof-of-concept

### Story 3.1: Implement Status Types
**Effort**: Medium  
**Priority**: Critical

**As a** developer  
**I want** the status command types defined  
**So that** I can implement the status function

**Description**:
Define all NAPI types for the status command parameters and response.

**Tasks**:
1. Create `src/types/status.rs`
   - Add module documentation
   - **Effort**: Minimal

2. Define StatusParams
   - Add `root: Option<String>`
   - Add `config_path: Option<String>`
   - Use `#[napi(object)]`
   - **Effort**: Low

3. Define RepositoryInfo
   - Add `kind: String`
   - Add `monorepo_type: Option<String>`
   - Use `#[napi(object)]`
   - **Effort**: Low

4. Define PackageManagerInfo
   - Add `name: String`
   - Add `lock_file: String`
   - Use `#[napi(object)]`
   - **Effort**: Low

5. Define BranchInfo
   - Add `name: String`
   - Use `#[napi(object)]`
   - **Effort**: Minimal

6. Define ChangesetInfo
   - Add `id: String`
   - Use `#[napi(object)]`
   - **Effort**: Minimal

7. Define PackageInfo
   - Add `name: String`
   - Add `version: String`
   - Add `path: String`
   - Use `#[napi(object)]`
   - **Effort**: Low

8. Define StatusData
   - Add `repository: RepositoryInfo`
   - Add `package_manager: PackageManagerInfo`
   - Add `branch: Option<BranchInfo>`
   - Add `changesets: Vec<ChangesetInfo>`
   - Add `packages: Vec<PackageInfo>`
   - Use `#[napi(object)]`
   - **Effort**: Low

9. Update types/mod.rs
   - Export status types
   - **Effort**: Minimal

**Acceptance Criteria**:
- [ ] All types have `#[napi(object)]` derive
- [ ] All types documented
- [ ] Types match CLI JSON output structure
- [ ] TypeScript types generate correctly

**Definition of Done**:
- [ ] Code compiles
- [ ] Clippy 100%
- [ ] TypeScript verified

**Dependencies**: Story 2.3

**Blocked By**: Story 2.3

**Blocks**: Story 3.2

---

### Story 3.2: Implement Status Command
**Effort**: High  
**Priority**: Critical

**As a** developer  
**I want** the status function implemented  
**So that** Node.js can retrieve workspace information

**Description**:
Implement the `status` NAPI function that calls CLI and returns structured data.

**Tasks**:
1. Create `src/commands/status.rs`
   - Add module documentation (What, How, Why)
   - **Effort**: Minimal

2. Implement parameter validation
   - Validate root path exists (if provided)
   - Handle current directory fallback
   - **Effort**: Low

3. Create CLI output capture mechanism
   - Create in-memory buffer
   - Configure Output with JSON format
   - **Effort**: Medium

4. Call execute_status from CLI
   - Create StatusArgs
   - Call execute_status with output and root
   - Handle Result
   - **Effort**: Medium

5. Implement JSON response parsing
   - Parse JSON bytes to serde_json::Value
   - Map to StatusData struct
   - Handle parsing errors
   - **Effort**: Medium

6. Export function with #[napi]
   - Add proper JSDoc comments
   - Add example in documentation
   - **Effort**: Low

7. Update commands/mod.rs
   - Export status function
   - **Effort**: Minimal

8. Update lib.rs
   - Re-export status
   - **Effort**: Minimal

**Acceptance Criteria**:
- [ ] Function callable from Node.js
- [ ] Returns ApiResponse<StatusData>
- [ ] Handles validation errors with EVALIDATION
- [ ] Handles CLI errors with appropriate codes
- [ ] TypeScript types correct
- [ ] Documentation with example

**Definition of Done**:
- [ ] Code compiles
- [ ] Clippy 100%
- [ ] Function works end-to-end
- [ ] Documentation complete

**Dependencies**: Story 3.1

**Blocked By**: Story 3.1

**Blocks**: Story 3.4

---

### Story 3.3: Implement Init Types
**Effort**: Medium  
**Priority**: High

**As a** developer  
**I want** the init command types defined  
**So that** I can implement the init function

**Description**:
Define all NAPI types for the init command parameters and response.

**Tasks**:
1. Create `src/types/init.rs`
   - Add module documentation
   - **Effort**: Minimal

2. Define InitParams
   - Add `root: Option<String>`
   - Add `changeset_path: Option<String>`
   - Add `environments: Option<Vec<String>>`
   - Add `default_env: Option<String>`
   - Add `strategy: Option<String>`
   - Add `registry: Option<String>`
   - Add `config_format: Option<String>`
   - Add `force: Option<bool>`
   - Use `#[napi(object)]`
   - Document each field
   - **Effort**: Medium

3. Define InitData response
   - Add `config_path: String`
   - Add `changeset_path: String`
   - Add `packages_count: u32`
   - Use `#[napi(object)]`
   - **Effort**: Low

4. Update types/mod.rs
   - Export init types
   - **Effort**: Minimal

**Acceptance Criteria**:
- [ ] All types have `#[napi(object)]` derive
- [ ] All types documented
- [ ] Optional fields correctly typed
- [ ] TypeScript types generate correctly

**Definition of Done**:
- [ ] Code compiles
- [ ] Clippy 100%
- [ ] TypeScript verified

**Dependencies**: Story 2.3

**Blocked By**: Story 2.3

**Blocks**: Story 3.4

---

### Story 3.4: Implement Init Command
**Effort**: High  
**Priority**: High

**As a** developer  
**I want** the init function implemented  
**So that** Node.js can initialize workspaces programmatically

**Description**:
Implement the `init` NAPI function following the pattern established in status.

**Tasks**:
1. Create `src/commands/init.rs`
   - Add module documentation
   - **Effort**: Minimal

2. Implement parameter validation
   - Validate root path
   - Validate strategy if provided
   - Validate config_format if provided
   - **Effort**: Medium

3. Create CLI args with fixed values
   - Set `non_interactive = true` (API is programmatic)
   - Map other parameters
   - **Effort**: Low

4. Call execute_init from CLI
   - Handle Result
   - Parse response
   - **Effort**: Medium

5. Export function with #[napi]
   - Add documentation with example
   - **Effort**: Low

6. Update exports
   - Update commands/mod.rs
   - Update lib.rs
   - **Effort**: Minimal

**Acceptance Criteria**:
- [ ] Function callable from Node.js
- [ ] Returns ApiResponse<InitData>
- [ ] non_interactive always true
- [ ] Handles errors correctly
- [ ] Documentation complete

**Definition of Done**:
- [ ] Code compiles
- [ ] Clippy 100%
- [ ] Function works end-to-end

**Dependencies**: Story 3.2, Story 3.3

**Blocked By**: Story 3.2, Story 3.3

**Blocks**: Story 3.5

---

### Story 3.5: POC Node.js Integration Tests
**Effort**: High  
**Priority**: Critical

**As a** developer  
**I want** integration tests for status and init  
**So that** I can verify the bindings work correctly

**Description**:
Create comprehensive Node.js tests for the POC commands.

**Tasks**:
1. Create test setup
   - Create `__test__/status.test.ts`
   - Create `__test__/init.test.ts`
   - Setup test fixtures
   - **Effort**: Medium

2. Write status success tests
   - Test with valid workspace
   - Verify response structure
   - Verify TypeScript types match
   - **Effort**: Medium

3. Write status error tests
   - Test with invalid root path
   - Test error code is EVALIDATION or ENOENT
   - Verify error message is helpful
   - **Effort**: Medium

4. Write init success tests
   - Test in temp directory
   - Verify config created
   - Verify changesets directory created
   - **Effort**: Medium

5. Write init error tests
   - Test with invalid path
   - Test with existing config (no force)
   - **Effort**: Medium

6. Verify TypeScript compilation
   - Ensure no type errors
   - Test IntelliSense works
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] All tests pass
- [ ] TypeScript compiles without errors
- [ ] Error handling verified
- [ ] Response types verified
- [ ] Tests documented

**Definition of Done**:
- [ ] Tests pass in CI
- [ ] Coverage reported
- [ ] Documentation complete

**Dependencies**: Story 3.4

**Blocked By**: Story 3.4

**Blocks**: Epic 4

---

## Epic 4: Changeset Commands

**Phase**: 3  
**Total Effort**: Massive  
**Dependencies**: Epic 3  
**Goal**: Implement all 7 changeset commands

### Story 4.1: Implement Changeset Types
**Effort**: High  
**Priority**: High

**As a** developer  
**I want** all changeset types defined  
**So that** I can implement changeset commands

**Description**:
Define NAPI types for all changeset command parameters and responses.

**Tasks**:
1. Create `src/types/changeset.rs`
   - Add module documentation
   - **Effort**: Minimal

2. Define ChangesetAddParams
   - Add all fields from CLI args:
     - `root?: string`
     - `configPath?: string`
     - `bump?: 'major' | 'minor' | 'patch'`
     - `environments?: string[]`
     - `branch?: string`
     - `message?: string`
     - `packages?: string[]`
     - `force?: boolean` (overwrites existing changeset)
   - Map `non_interactive` as internal (always true)
   - **Effort**: Medium

3. Define ChangesetUpdateParams
   - Add id, commit, packages, bump, env
   - **Effort**: Low

4. Define ChangesetListParams
   - Add filter options
   - **Effort**: Low

5. Define ChangesetShowParams
   - Add branch field
   - **Effort**: Minimal

6. Define ChangesetRemoveParams
   - Add branch, force fields
   - **Effort**: Minimal

7. Define ChangesetHistoryParams
   - Add all filter and pagination fields
   - **Effort**: Low

8. Define ChangesetCheckParams
   - Add branch field
   - **Effort**: Minimal

9. Define response types
   - ChangesetData
   - ChangesetListData
   - ChangesetHistoryData
   - ChangesetCheckData
   - **Effort**: Medium

10. Update types/mod.rs
    - Export all changeset types
    - **Effort**: Minimal

**Acceptance Criteria**:
- [ ] All 7 param types defined
- [ ] All response types defined
- [ ] TypeScript types generate correctly
- [ ] Documentation complete
- [ ] force parameter included in ChangesetAddParams
- [ ] force parameter included in ChangesetRemoveParams

**Definition of Done**:
- [ ] Code compiles
- [ ] Clippy 100%
- [ ] TypeScript verified

**Dependencies**: Story 3.5

**Blocked By**: Story 3.5

**Blocks**: Story 4.2

---

### Story 4.2: Implement changesetAdd
**Effort**: High  
**Priority**: High

**As a** developer  
**I want** the changesetAdd function implemented  
**So that** Node.js can create changesets programmatically

**Description**:
Implement the `changesetAdd` NAPI function.

**Tasks**:
1. Create `src/commands/changeset.rs`
   - Add module documentation
   - **Effort**: Minimal

2. Implement changesetAdd
   - Validate parameters
   - Create ChangesetCreateArgs with non_interactive=true
   - Support force parameter to overwrite existing changeset
   - Call execute_add
   - Parse response
   - Return ApiResponse<ChangesetData>
   - **Effort**: High

3. Add documentation and examples
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Function callable from Node.js
- [ ] Creates changeset successfully
- [ ] non_interactive always true
- [ ] force parameter overwrites existing changeset
- [ ] Error handling correct

**Definition of Done**:
- [ ] Code compiles
- [ ] Clippy 100%
- [ ] Function works

**Dependencies**: Story 4.1

**Blocked By**: Story 4.1

**Blocks**: Story 4.3

---

### Story 4.3: Implement changesetUpdate
**Effort**: Medium  
**Priority**: High

**Description**:
Implement the `changesetUpdate` NAPI function.

**Tasks**:
1. Implement changesetUpdate in changeset.rs
   - Validate id parameter required
   - Call execute_update
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Function works end-to-end
- [ ] Validation for required id

**Dependencies**: Story 4.2

---

### Story 4.4: Implement changesetList
**Effort**: Medium  
**Priority**: High

**Description**:
Implement the `changesetList` NAPI function.

**Tasks**:
1. Implement changesetList
   - All filter parameters optional
   - Return ChangesetListData with array
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Function returns list correctly
- [ ] Filters work

**Dependencies**: Story 4.2

---

### Story 4.5: Implement changesetShow
**Effort**: Low  
**Priority**: Medium

**Description**:
Implement the `changesetShow` NAPI function.

**Tasks**:
1. Implement changesetShow
   - Validate branch required
   - Return single changeset data
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Function returns changeset details

**Dependencies**: Story 4.2

---

### Story 4.6: Implement changesetRemove
**Effort**: Low  
**Priority**: Medium

**Description**:
Implement the `changesetRemove` NAPI function.

**Tasks**:
1. Implement changesetRemove
   - Validate branch required
   - Return void on success
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Function removes changeset

**Dependencies**: Story 4.2

---

### Story 4.7: Implement changesetHistory
**Effort**: Medium  
**Priority**: Medium

**Description**:
Implement the `changesetHistory` NAPI function.

**Tasks**:
1. Implement changesetHistory
   - Support all filter options
   - Return paginated history
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Function returns history
- [ ] Filters and limits work

**Dependencies**: Story 4.2

---

### Story 4.8: Implement changesetCheck
**Effort**: Low  
**Priority**: Medium

**Description**:
Implement the `changesetCheck` NAPI function.

**Tasks**:
1. Implement changesetCheck
   - Check if changeset exists
   - Return check result
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Function returns check status

**Dependencies**: Story 4.2

---

### Story 4.9: Changeset Integration Tests
**Effort**: High  
**Priority**: High

**Description**:
Create comprehensive Node.js tests for all changeset commands.

**Tasks**:
1. Create `__test__/changeset.test.ts`
2. Test each command (add, update, list, show, remove, history, check)
3. Test error cases
4. Test with various parameters
   - **Effort**: High

**Acceptance Criteria**:
- [ ] All 7 functions tested
- [ ] Error handling verified
- [ ] TypeScript types verified

**Dependencies**: Story 4.8

---

## Epic 5: Bump Commands

**Phase**: 3  
**Total Effort**: High  
**Dependencies**: Epic 4  
**Goal**: Implement all 3 bump commands with full prerelease and snapshot support

### Story 5.1: Implement Bump Types
**Effort**: Medium  
**Priority**: High

**Description**:
Define NAPI types for bump command parameters and responses. Each command has its own params struct to support different use cases.

**Tasks**:
1. Create `src/types/bump.rs`
2. Define `BumpPreviewParams`:
   - `root?: string`
   - `configPath?: string`
   - `packages?: string[]`
   - `showDiff?: boolean`
3. Define `BumpApplyParams`:
   - `root?: string`
   - `configPath?: string`
   - `packages?: string[]`
   - `gitCommit?: boolean`
   - `gitTag?: boolean`
   - `gitPush?: boolean`
   - `prerelease?: string` (alpha, beta, rc, or custom)
   - `noChangelog?: boolean`
   - `noArchive?: boolean`
   - `alwaysArchive?: boolean`
   - `force?: boolean`
4. Define `BumpSnapshotParams`:
   - `root?: string`
   - `configPath?: string`
   - `packages?: string[]`
   - `format?: string` (template with variables)
5. Define `BumpPreviewData`, `BumpApplyData`, `BumpSnapshotData`
6. Define `PackageVersionInfo`, `SnapshotVersionInfo`, `DependencyUpdate`
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] All types defined with #[napi(object)]
- [ ] TypeScript types generated correctly
- [ ] Prerelease parameter included in BumpApplyParams
- [ ] Snapshot format parameter included in BumpSnapshotParams
- [ ] alwaysArchive parameter included in BumpApplyParams

**Dependencies**: Story 4.9

---

### Story 5.2: Implement bumpPreview
**Effort**: High  
**Priority**: High

**Description**:
Implement the `bumpPreview` NAPI function.

**Tasks**:
1. Create `src/commands/bump.rs`
2. Implement bumpPreview
   - Set dry_run = true internally
   - Support showDiff parameter
   - Parse preview results including package versions and dependency updates
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Returns preview of changes
- [ ] No actual changes made
- [ ] showDiff option works correctly
- [ ] Returns strategy, packages, summary, and changesets

**Dependencies**: Story 5.1

---

### Story 5.3: Implement bumpApply
**Effort**: High  
**Priority**: High

**Description**:
Implement the `bumpApply` NAPI function with full prerelease support.

**Tasks**:
1. Implement bumpApply
   - Set execute = true internally
   - Support all git options (gitCommit, gitTag, gitPush)
   - Support prerelease parameter for alpha/beta/rc versions
   - Support changelog/archive control (noChangelog, noArchive, alwaysArchive)
   - Validate prerelease tag format
   - **Effort**: High

**Prerelease Workflow**:
- `prerelease: "beta"` → `1.2.3` → `1.3.0-beta.0`
- `prerelease: "rc"` → `1.3.0-beta.1` → `1.3.0-rc.0`
- Without prerelease → `1.3.0-rc.0` → `1.3.0` (promoted to stable)

**Acceptance Criteria**:
- [ ] Applies version bumps
- [ ] Git operations work if enabled
- [ ] Prerelease versions created correctly (SemVer compliant)
- [ ] alwaysArchive forces changeset archiving for prereleases
- [ ] noChangelog skips changelog generation
- [ ] Returns packagesUpdated, changesetsArchived, filesModified, tagsCreated, commitSha

**Dependencies**: Story 5.2

---

### Story 5.4: Implement bumpSnapshot
**Effort**: Medium  
**Priority**: Medium

**Description**:
Implement the `bumpSnapshot` NAPI function for generating temporary test versions.

**Tasks**:
1. Implement bumpSnapshot
   - Set snapshot = true internally
   - Support format template option
   - Validate format contains valid variables
   - Does NOT consume/archive changesets
   - **Effort**: Medium

**Snapshot Format Variables**:
- `{version}` - Base version (e.g., 1.2.3)
- `{branch}` - Git branch name (sanitized)
- `{commit}` - Full commit hash
- `{short_commit}` - Short commit hash (7 chars)
- `{timestamp}` - Unix timestamp

**Default Format**: `{version}-snapshot.{short_commit}`

**Acceptance Criteria**:
- [ ] Creates snapshot versions (NOT SemVer compliant)
- [ ] Format template with variables works
- [ ] Changesets are NOT archived
- [ ] Returns packages with originalVersion and snapshotVersion

**Dependencies**: Story 5.2

---

### Story 5.5: Implement Bump Validators
**Effort**: Low  
**Priority**: High

**Description**:
Implement validators for prerelease tags and snapshot formats.

**Tasks**:
1. Add `prerelease_tag` validator
   - Must contain only `[0-9A-Za-z-]`
   - Common values: alpha, beta, rc
2. Add `snapshot_format` validator
   - Must contain at least one valid variable
   - Valid: {version}, {branch}, {short_commit}, {commit}, {timestamp}

**Acceptance Criteria**:
- [ ] prerelease_tag validator rejects invalid characters
- [ ] snapshot_format validator requires valid variables
- [ ] Clear error messages for validation failures

**Dependencies**: Story 2.2

---

### Story 5.6: Bump Integration Tests
**Effort**: High  
**Priority**: High

**Description**:
Create Node.js tests for bump commands including prerelease and snapshot workflows.

**Tasks**:
1. Create `__test__/bump.test.ts`
2. Test bumpPreview:
   - Returns preview with packages, versions, and bump types
   - showDiff option works
   - No changes made to files
3. Test bumpApply:
   - Applies version bumps correctly
   - Git commit created when gitCommit=true
   - Git tags created when gitTag=true
   - Git push works when gitPush=true
4. Test bumpApply with prerelease:
   - `prerelease: "alpha"` creates alpha version
   - `prerelease: "beta"` creates beta version
   - `prerelease: "rc"` creates release candidate
   - Custom prerelease tags work
   - noArchive keeps changesets active for prereleases
   - alwaysArchive forces archiving for prereleases
5. Test bumpSnapshot:
   - Default format generates correct snapshot version
   - Custom format template works
   - All variables work: {version}, {branch}, {short_commit}, {commit}, {timestamp}
   - Changesets NOT archived after snapshot
6. Test error cases:
   - Invalid root path returns ENOENT
   - Invalid prerelease tag returns EVALIDATION
   - Invalid snapshot format returns EVALIDATION
   - No changesets returns appropriate message
   - **Effort**: High

**Test Scenarios**:

```typescript
// Prerelease workflow test
describe('prerelease workflow', () => {
  it('creates beta prerelease', async () => {
    const result = await bumpApply({
      root: testWorkspace,
      prerelease: 'beta',
      gitCommit: true,
      gitTag: true,
    });
    expect(result.success).toBe(true);
    // Version should be like 1.3.0-beta.0
  });
});

// Snapshot workflow test
describe('snapshot workflow', () => {
  it('creates snapshot with custom format', async () => {
    const result = await bumpSnapshot({
      root: testWorkspace,
      format: '{version}-{branch}.{short_commit}',
    });
    expect(result.success).toBe(true);
    // Version should be like 1.2.3-feature-x.abc123f
  });
});
```

**Acceptance Criteria**:
- [ ] All 3 functions tested (preview, apply, snapshot)
- [ ] Prerelease workflow tested with alpha, beta, rc
- [ ] Snapshot workflow tested with custom format
- [ ] Git integration tested (commit, tag, push)
- [ ] Error cases tested with proper error codes
- [ ] Workflow tested end-to-end

**Dependencies**: Story 5.5

---

## Epic 6: Execute Command with Timeout

**Phase**: 4  
**Total Effort**: High  
**Dependencies**: Epic 5  
**Goal**: Implement execute command with timeout support

### Story 6.1: Add ExecuteConfig to pkg crate
**Effort**: Medium  
**Priority**: High

**Description**:
Add ExecuteConfig to the pkg crate for timeout defaults.

**Tasks**:
1. Create `crates/pkg/src/config/execute.rs`
2. Define ExecuteConfig struct
   - timeout_secs (default 300)
   - per_package_timeout_secs (default 60)
   - max_parallel (default 8)
3. Add to PackageToolsConfig
4. Update config loading
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] ExecuteConfig defined
- [ ] Loaded from repo.config [execute] section
- [ ] Defaults applied correctly

**Dependencies**: Story 5.5

---

### Story 6.2: Implement Execute Types
**Effort**: Medium  
**Priority**: High

**Description**:
Define NAPI types for execute command.

**Tasks**:
1. Create `src/types/execute.rs`
2. Define ExecuteParams
   - Include timeout_secs and per_package_timeout_secs
3. Define PackageExecutionResult
4. Define ExecuteSummary
5. Define ExecuteData
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] All types with #[napi(object)]
- [ ] Timeout parameters included

**Dependencies**: Story 6.1

---

### Story 6.3: Implement Execute Command
**Effort**: High  
**Priority**: High

**Description**:
Implement the `execute` NAPI function with timeout.

**Tasks**:
1. Create `src/commands/execute.rs`
2. Implement parameter validation
   - Validate cmd not empty
   - Validate mutual exclusion (filterPackage vs affected)
   - Validate timeout ranges
3. Implement timeout resolution
   - Load from config
   - Apply parameter overrides
4. Implement execute with tokio::time::timeout
5. Return ETIMEOUT error code on timeout
   - **Effort**: High

**Acceptance Criteria**:
- [ ] Function executes commands
- [ ] Timeout works correctly
- [ ] ETIMEOUT error returned on timeout
- [ ] Mutual exclusion validated

**Dependencies**: Story 6.2

---

### Story 6.4: Execute Integration Tests
**Effort**: High  
**Priority**: High

**Description**:
Create Node.js tests for execute command.

**Tasks**:
1. Create `__test__/execute.test.ts`
2. Test basic execution
3. Test parallel execution
4. Test timeout behavior
5. Test affected packages
6. Test filter packages
7. Test mutual exclusion error
   - **Effort**: High

**Acceptance Criteria**:
- [ ] All scenarios tested
- [ ] Timeout verified
- [ ] Error handling verified

**Dependencies**: Story 6.3

---

## Epic 7: Config Commands

**Phase**: 4  
**Total Effort**: Medium  
**Dependencies**: Epic 6  
**Goal**: Implement config show and validate commands

### Story 7.1: Implement Config Types
**Effort**: Low  
**Priority**: Medium

**Description**:
Define NAPI types for config commands.

**Tasks**:
1. Create `src/types/config.rs`
2. Define ConfigShowParams
3. Define ConfigValidateParams
4. Define ConfigData
5. Define ConfigValidationData
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Types defined
- [ ] TypeScript correct

**Dependencies**: Story 6.4

---

### Story 7.2: Implement configShow
**Effort**: Medium  
**Priority**: Medium

**Description**:
Implement the `configShow` NAPI function.

**Tasks**:
1. Create `src/commands/config.rs`
2. Implement configShow
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Returns current config

**Dependencies**: Story 7.1

---

### Story 7.3: Implement configValidate
**Effort**: Medium  
**Priority**: Medium

**Description**:
Implement the `configValidate` NAPI function.

**Tasks**:
1. Implement configValidate
   - Return validation status
   - Return any errors found
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Validates config
- [ ] Returns issues if any

**Dependencies**: Story 7.2

---

### Story 7.4: Config Integration Tests
**Effort**: Medium  
**Priority**: Medium

**Description**:
Create Node.js tests for config commands.

**Tasks**:
1. Create `__test__/config.test.ts`
2. Test show and validate
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Both functions tested

**Dependencies**: Story 7.3

---

## Epic 8: Upgrade Commands

**Phase**: 5  
**Total Effort**: High  
**Dependencies**: Epic 7  
**Goal**: Implement all 5 upgrade commands

### Story 8.1: Implement Upgrade Types
**Effort**: Medium  
**Priority**: Medium

**Description**:
Define NAPI types for upgrade commands.

**Tasks**:
1. Create `src/types/upgrade.rs`
2. Define all params and response types for:
   - upgradeCheck
   - upgradeApply
   - backupList
   - backupRestore
   - backupClean
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] All types defined

**Dependencies**: Story 7.4

---

### Story 8.2: Implement upgradeCheck
**Effort**: Medium  
**Priority**: Medium

**Tasks**:
1. Create `src/commands/upgrade.rs`
2. Implement upgradeCheck
   - **Effort**: Medium

**Dependencies**: Story 8.1

---

### Story 8.3: Implement upgradeApply
**Effort**: Medium  
**Priority**: Medium

**Tasks**:
1. Implement upgradeApply
   - **Effort**: Medium

**Dependencies**: Story 8.2

---

### Story 8.4: Implement Backup Commands
**Effort**: Medium  
**Priority**: Low

**Description**:
Implement backupList, backupRestore, backupClean.

**Tasks**:
1. Implement backupList
2. Implement backupRestore
3. Implement backupClean
   - **Effort**: Medium

**Dependencies**: Story 8.3

---

### Story 8.5: Upgrade Integration Tests
**Effort**: Medium  
**Priority**: Medium

**Tasks**:
1. Create `__test__/upgrade.test.ts`
2. Test all 5 functions
   - **Effort**: Medium

**Dependencies**: Story 8.4

---

## Epic 9: Remaining Commands

**Phase**: 5  
**Total Effort**: Medium  
**Dependencies**: Epic 8  
**Goal**: Implement audit, changes, and clone commands

### Story 9.1: Implement Audit
**Effort**: Medium  
**Priority**: Medium

**Tasks**:
1. Create `src/types/audit.rs`
2. Create `src/commands/audit.rs`
3. Implement audit function
   - **Effort**: Medium

**Dependencies**: Story 8.5

---

### Story 9.2: Implement Changes
**Effort**: Medium  
**Priority**: Medium

**Tasks**:
1. Create `src/types/changes.rs`
2. Create `src/commands/changes.rs`
3. Implement changes function
   - **Effort**: Medium

**Dependencies**: Story 9.1

---

### Story 9.3: Implement Clone
**Effort**: Medium  
**Priority**: Low

**Tasks**:
1. Create `src/types/clone.rs`
2. Create `src/commands/clone.rs`
3. Implement clone function
   - non_interactive = true
   - **Effort**: Medium

**Dependencies**: Story 9.2

---

### Story 9.4: Remaining Commands Tests
**Effort**: Medium  
**Priority**: Medium

**Tasks**:
1. Create tests for audit, changes, clone
   - **Effort**: Medium

**Dependencies**: Story 9.3

---

## Epic 10: Testing & Quality

**Phase**: 6  
**Total Effort**: High  
**Dependencies**: Epic 9  
**Goal**: Achieve 100% Clippy, high test coverage

### Story 10.1: Rust Test Coverage
**Effort**: High  
**Priority**: Critical

**Tasks**:
1. Add unit tests for all error mappings
2. Add unit tests for all validators
3. Add unit tests for response helpers
4. Run cargo tarpaulin for coverage report
5. Achieve 100% coverage
   - **Effort**: High

**Acceptance Criteria**:
- [ ] 100% Rust test coverage

**Dependencies**: Story 9.4

---

### Story 10.2: Node.js Test Coverage
**Effort**: High  
**Priority**: Critical

**Tasks**:
1. Review all test files
2. Add missing test cases
3. Test edge cases and error scenarios
4. Achieve >90% coverage
   - **Effort**: High

**Acceptance Criteria**:
- [ ] >90% Node.js test coverage

**Dependencies**: Story 10.1

---

### Story 10.3: Cross-Platform Testing
**Effort**: Medium  
**Priority**: High

**Tasks**:
1. Test on macOS (x64 and ARM64)
2. Test on Linux (x64)
3. Test on Windows (x64)
4. Fix any platform-specific issues
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] All platforms work

**Dependencies**: Story 10.2

---

### Story 10.4: Performance Benchmarks
**Effort**: Medium  
**Priority**: Medium

**Tasks**:
1. Create benchmark suite
2. Measure NAPI call overhead
3. Verify <50ms overhead vs CLI
4. Document results
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] Performance targets met

**Dependencies**: Story 10.3

---

## Epic 11: Documentation & Release

**Phase**: 6  
**Total Effort**: Medium  
**Dependencies**: Epic 10  
**Goal**: Complete documentation and release

### Story 11.1: Update index.ts Exports
**Effort**: Low  
**Priority**: High

**Tasks**:
1. Update `packages/workspace-tools/src/index.ts`
2. Export all 23 new functions
3. Export all types
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] All exports available

**Dependencies**: Story 10.4

---

### Story 11.2: Update README
**Effort**: Medium  
**Priority**: High

**Tasks**:
1. Update `packages/workspace-tools/README.md`
2. Add usage examples for key functions
3. Document error handling patterns
4. Document TypeScript types
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] README comprehensive

**Dependencies**: Story 11.1

---

### Story 11.3: Update CI/CD
**Effort**: Medium  
**Priority**: High

**Tasks**:
1. Update GitHub Actions for multi-platform builds
2. Add test job for Node.js tests
3. Configure npm publish workflow
   - **Effort**: Medium

**Acceptance Criteria**:
- [ ] CI builds all platforms
- [ ] Tests run in CI
- [ ] Publish works

**Dependencies**: Story 11.2

---

### Story 11.4: Release Preparation
**Effort**: Low  
**Priority**: Critical

**Tasks**:
1. Bump version in Cargo.toml
2. Bump version in package.json
3. Update CHANGELOG
4. Create git tag
5. Publish to npm
   - **Effort**: Low

**Acceptance Criteria**:
- [ ] Package published to npm
- [ ] Version is correct

**Dependencies**: Story 11.3

---

## Summary

### Total Story Count

| Epic | Stories | Priority |
|------|---------|----------|
| Epic 1: NAPI Foundation | 3 | Critical |
| Epic 2: Error & Validation | 3 | Critical |
| Epic 3: POC Commands | 5 | Critical |
| Epic 4: Changeset Commands | 9 | High |
| Epic 5: Bump Commands | 6 | High |
| Epic 6: Execute Command | 4 | High |
| Epic 7: Config Commands | 4 | Medium |
| Epic 8: Upgrade Commands | 5 | Medium |
| Epic 9: Remaining Commands | 4 | Medium |
| Epic 10: Testing & Quality | 4 | Critical |
| Epic 11: Documentation & Release | 4 | High |
| **Total** | **51** | |

### Effort Distribution

| Effort Level | Count | Percentage |
|--------------|-------|------------|
| Minimal | 15 | 30% |
| Low | 10 | 20% |
| Medium | 15 | 30% |
| High | 8 | 16% |
| Massive | 2 | 4% |

### Critical Path

```
Story 1.1 (Crate Structure)
    ↓
Story 1.3 (lib.rs)
    ↓
Story 2.1 (ErrorInfo)
    ↓
Story 2.2 (Validation) → Story 2.3 (ApiResponse)
    ↓
Story 3.1 (Status Types)
    ↓
Story 3.2 (Status Command)
    ↓
Story 3.5 (POC Tests)
    ↓
[All remaining epics]
```

### Parallel Work Opportunities

After Epic 3 (POC) is complete:
- Epic 4 (Changeset) and Epic 5 (Bump) can start in parallel
- Epic 6 (Execute) depends on Epic 5 completion
- Epic 7-9 can be parallelized after Epic 6

### Estimated Timeline

| Phase | Weeks | Epics |
|-------|-------|-------|
| Phase 1: Foundation | 1 | Epics 1-2 |
| Phase 2: POC | 1 | Epic 3 |
| Phase 3: Core Commands | 2 | Epics 4-5 |
| Phase 4: Execute & Config | 1 | Epics 6-7 |
| Phase 5: Remaining | 1-2 | Epics 8-9 |
| Phase 6: Polish | 1-2 | Epics 10-11 |
| **Total** | **7-9 weeks** | |

### Quality Gates

| Gate | Requirement |
|------|-------------|
| POC Complete | status() and init() work from Node.js |
| Core Complete | All 23 functions implemented |
| Quality Complete | Clippy 100%, Tests >90% coverage |
| Release Ready | All platforms build, docs complete |

---

## How to Use This Story Map

### For Planning

1. Review epic dependencies before sprint planning
2. Prioritize critical path stories
3. Identify parallel work opportunities
4. Account for effort estimates in capacity planning

### For Development

1. Start each story by reading its description and tasks
2. Follow acceptance criteria as a checklist
3. Reference PLAN_NODE.md and NAPI_RESEARCH.md for technical details
4. Update story status as work progresses

### For Review

1. Verify all acceptance criteria are met
2. Ensure Definition of Done is satisfied
3. Check that blocked stories can proceed
4. Update documentation as needed

### For Tracking Progress

| Status | Symbol |
|--------|--------|
| Not Started | ⬜ |
| In Progress | 🔄 |
| Blocked | 🚫 |
| Complete | ✅ |

---

**Document maintained by**: Development Team  
**Last updated**: 2025-01-20  
**Next review**: After Phase 1 completion