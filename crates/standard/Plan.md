# Standard Crate Architectural Reorganization Plan

## Overview

This document outlines the comprehensive architectural reorganization plan for the `sublime_standard_tools` crate to address fundamental issues in type organization, separation of concerns, and module responsibility boundaries.

## Problem Statement

### Critical Architectural Issues Identified

1. **Misplaced Core Types**: `PackageManagerKind` and `PackageManager` are in the `monorepo` module but represent generic Node.js concepts used by all project types
2. **Missing Repository Abstraction**: No unified concept for repository types (simple vs monorepo)
3. **Conceptual Dependencies**: Project module depends on monorepo module for generic functionality
4. **Unclear Separation of Concerns**: Package management mixed with monorepo-specific logic

### Current Type Misalignment

| Type | Current Location | Correct Location | Justification |
|------|-----------------|------------------|---------------|
| `PackageManagerKind` | `monorepo/` ❌ | `node/` ✅ | Generic Node.js concept |
| `PackageManager` | `monorepo/` ❌ | `node/` ✅ | Used by all project types |
| `MonorepoKind` | `monorepo/` ✅ | `monorepo/` ✅ | Monorepo-specific |
| `RepoKind` | Missing ❌ | `node/` ✅ | Fundamental abstraction needed |

## Goals

1. **Clean Architecture**: Clear separation of concerns with proper module boundaries
2. **Type Safety**: Hierarchical type system that reflects real-world relationships
3. **Reusability**: Generic concepts available to all modules that need them
4. **Breaking Changes**: Complete architectural cleanup without compatibility constraints

## Architectural Vision

### New Module Structure
```
src/
├── node/           # Generic Node.js concepts
│   ├── types.rs    # RepoKind, core traits
│   ├── package_manager.rs  # PackageManagerKind, PackageManager
│   └── repository.rs       # Repository abstractions
├── project/        # Project detection and management
├── monorepo/       # Monorepo-specific functionality
├── filesystem/     # File system abstractions
├── command/        # Command execution
└── error/          # Error types
```

### New Type Hierarchy
```rust
// Core repository concept
pub enum RepoKind {
    Simple,
    Monorepo(MonorepoKind),
}

// Project types use repository concept
pub enum ProjectKind {
    Repository(RepoKind),
}

// Clean separation: node concepts independent of monorepo
use crate::node::{PackageManager, PackageManagerKind, RepoKind};
use crate::monorepo::MonorepoKind;
```

## Implementation Roadmap

```
Phase 1: Foundation   ████████████████████████████████████████████ [100%] - ✅ COMPLETED
Phase 2: Migration    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [0%] - 1 Session
Phase 3: Integration  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [0%] - 1 Session
Phase 4: Validation   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [0%] - 1 Session
Phase 5: Documentation ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ [0%] - 1 Session
```

---

## Phase 1: Foundation - Create Node.js Module

### Objectives
- Establish new module structure for generic Node.js concepts
- Define core repository abstractions
- Create foundation for type migration

### Tasks

#### 1.1 Module Structure Creation
- [x] Create `src/node/` directory
- [x] Create `src/node/mod.rs` with module declarations
- [x] Create `src/node/types.rs` for core repository types
- [x] Create `src/node/package_manager.rs` for package management
- [x] Create `src/node/repository.rs` for repository abstractions
- [x] Create `src/node/tests.rs` for comprehensive testing

#### 1.2 Core Type Definitions
- [x] Define `RepoKind` enum:
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub enum RepoKind {
      /// Simple Node.js repository (single package.json)
      Simple,
      /// Monorepo with specific monorepo type
      Monorepo(MonorepoKind),
  }
  ```
- [x] Define `RepositoryInfo` trait for unified repository interface
- [x] Implement methods for `RepoKind` (name, is_monorepo, etc.)
- [x] Update `src/lib.rs` to export new node module

#### 1.3 Package Manager Foundation
- [x] Design `PackageManagerKind` enum structure
- [x] Design `PackageManager` struct interface
- [x] Plan detection and utility methods
- [x] Design integration points with repository types

### Completion Criteria
- [x] `cargo build` succeeds
- [x] New module structure documented
- [x] Core types defined with proper visibility
- [x] Foundation ready for migration

---

## Phase 2: Migration - Move Types from Monorepo

### Objectives
- **BREAKING CHANGE**: Move package manager types to node module
- Implement RepoKind functionality
- Remove old types without aliases

### Tasks

#### 2.1 Package Manager Migration
- [ ] **BREAKING**: Move `PackageManagerKind` from `monorepo/types.rs` to `node/package_manager.rs`
- [ ] **BREAKING**: Move `PackageManager` struct to `node/package_manager.rs`
- [ ] **BREAKING**: Move implementation methods from `monorepo/manager.rs`
- [ ] **BREAKING**: Delete package manager code from monorepo module
- [ ] Update imports in monorepo module to use `crate::node::`

#### 2.2 Repository Kind Implementation
- [ ] Implement `RepoKind` methods (name, is_monorepo, etc.)
- [ ] **BREAKING**: Refactor `ProjectKind` to use `RepoKind`
- [ ] **BREAKING**: Update `ProjectDescriptor` for new hierarchy
- [ ] Remove old type definitions completely

#### 2.3 Import Updates
- [ ] Update monorepo module imports
- [ ] Update project module imports
- [ ] Remove cross-module dependencies
- [ ] Verify clean module boundaries

### Completion Criteria
- [ ] `cargo build` succeeds
- [ ] `cargo clippy -- -D warnings` produces no warnings
- [ ] No package manager types remain in monorepo module
- [ ] Clean import structure established

---

## Phase 3: Integration - Update All Dependents

### Objectives
- **BREAKING CHANGE**: Update all modules to use new type structure
- Implement unified repository interface
- Ensure all detection logic works with new types

### Tasks

#### 3.1 Project Module Updates
- [ ] **BREAKING**: Refactor `ProjectKind` to use `RepoKind`
- [ ] **BREAKING**: Update `ProjectInfo` trait for new structure
- [ ] **BREAKING**: Modify `ProjectDetector` for repository-first approach
- [ ] **BREAKING**: Update `SimpleProject` to use `node::PackageManager`
- [ ] **BREAKING**: Rewrite `ProjectDescriptor` enum variants

#### 3.2 Monorepo Module Updates
- [ ] **BREAKING**: Remove all package manager type definitions
- [ ] Update `MonorepoDescriptor` to work with new `PackageManager`
- [ ] Ensure `MonorepoKind` integrates properly with `RepoKind`
- [ ] Update detection logic to use node module types
- [ ] Verify workspace functionality still works

#### 3.3 Cross-Module Integration
- [ ] Ensure proper trait implementations across modules
- [ ] Verify repository detection works end-to-end
- [ ] Test package manager detection for all project types
- [ ] Validate dependency graph analysis still functions

### Completion Criteria
- [ ] `cargo build` succeeds
- [ ] All modules use clean import structure
- [ ] Repository detection works for simple and monorepo projects
- [ ] No conceptual dependency violations

---

## Phase 4: Validation - Testing and Quality Assurance

### Objectives
- **BREAKING CHANGE**: Rewrite all tests for new architecture
- Ensure quality standards are met
- Validate all functionality works end-to-end

### Tasks

#### 4.1 Comprehensive Test Rewrite
- [ ] **BREAKING**: Rewrite `node/tests.rs` for new types
- [ ] **BREAKING**: Update `project/tests.rs` for new architecture
- [ ] **BREAKING**: Update `monorepo/tests.rs` for new imports
- [ ] Add integration tests for cross-module functionality
- [ ] Test edge cases and error conditions

#### 4.2 Quality Verification
- [ ] Execute `cargo build` - must succeed with 0 errors
- [ ] Execute `cargo clippy -- -D warnings` - must produce 0 warnings
- [ ] Execute `cargo test -- --nocapture` - must pass all tests
- [ ] Verify documentation builds correctly
- [ ] Check test coverage for new code

#### 4.3 Real-world Validation
- [ ] Test with actual monorepo structures
- [ ] Test with simple Node.js projects
- [ ] Verify package manager detection across platforms
- [ ] Test error handling and edge cases

### Completion Criteria
- [ ] All tests pass without errors
- [ ] Code quality standards met (clippy, rustfmt)
- [ ] Real-world usage scenarios validated
- [ ] Performance not degraded

---

## Phase 5: Documentation - Complete Architecture Documentation

### Objectives
- **BREAKING CHANGE**: Rewrite all documentation for new architecture
- Document new module structure and responsibilities
- Create examples for new API

### Tasks

#### 5.1 Specification Updates
- [ ] **BREAKING**: Completely rewrite `SPEC.md` for new architecture
- [ ] Document new module structure and responsibilities
- [ ] Update API documentation for all public types
- [ ] Create comprehensive examples for new usage patterns

#### 5.2 Code Documentation
- [ ] Add module-level documentation for `node/` module
- [ ] Document all public types and traits
- [ ] Add usage examples to type documentation
- [ ] Ensure all clippy documentation requirements met

#### 5.3 Breaking Changes Documentation
- [ ] Document architectural changes and rationale
- [ ] List all breaking changes clearly
- [ ] Provide examples of new usage patterns
- [ ] No migration guide needed (development product)

#### 5.4 Final Cleanup
- [ ] **BREAKING**: Remove all deprecated code
- [ ] **BREAKING**: Update `lib.rs` exports for clean API
- [ ] Verify documentation consistency
- [ ] Validate final API surface

### Completion Criteria
- [ ] SPEC.md reflects new architecture accurately
- [ ] All public APIs documented with examples
- [ ] Module responsibilities clearly defined
- [ ] Clean, breaking-change-complete API

---

## Implementation Notes

### Breaking Changes Policy
- **Zero Compatibility**: Product in development, no backward compatibility required
- **Clean Slate**: Remove old types completely, no aliases or deprecation
- **API Surface**: Completely new public API designed for clarity
- **Documentation**: Rewrite all documentation from scratch

### Quality Standards
- **Mandatory Compilation**: `cargo build` must succeed at each phase
- **Zero Warnings**: `cargo clippy -- -D warnings` must produce no warnings
- **Test Coverage**: `cargo test -- --nocapture` must pass all tests
- **Documentation**: All public APIs must be documented with examples

### Module Responsibilities

#### `node/` Module (NEW)
- Generic Node.js repository concepts
- Package manager abstractions
- Repository type definitions
- Cross-platform Node.js utilities

#### `project/` Module
- Project detection and management
- Unified API for all project types
- Configuration management
- Validation logic

#### `monorepo/` Module
- Monorepo-specific functionality
- Workspace analysis
- Dependency graph operations
- Monorepo type definitions

### Success Criteria

- [ ] Clean module architecture with clear boundaries
- [ ] No conceptual dependencies between modules
- [ ] Unified repository abstraction working for all project types
- [ ] Package manager functionality available to all modules
- [ ] Zero warnings from clippy
- [ ] All tests passing
- [ ] Complete documentation coverage
- [ ] Real-world usage scenarios validated

## Checkpoint Strategy

Each phase represents a complete session checkpoint:

1. **Session 1**: Complete Phase 1 (Foundation)
2. **Session 2**: Complete Phase 2 (Migration)
3. **Session 3**: Complete Phase 3 (Integration)
4. **Session 4**: Complete Phase 4 (Validation)
5. **Session 5**: Complete Phase 5 (Documentation)

Each phase must be 100% complete before proceeding to the next phase, with all quality criteria met.