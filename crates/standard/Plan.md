# Project Module Refactoring Plan

## Overview

This document outlines the refactoring plan to extract generic project functionality from the monorepo module and create a unified API for working with all types of Node.js projects (simple repositories and monorepos).

## Problem Statement

Currently, the `monorepo` module contains generic project management functionality that should be available for all Node.js projects, not just monorepos. This creates:
- Code duplication when working with simple repositories
- Inconsistent APIs between project types
- Tight coupling of generic functionality with monorepo-specific code

## Goals

1. **Unified API**: Single entry point for detecting and managing any Node.js project
2. **Code Reuse**: Share common functionality between simple and monorepo projects
3. **Extensibility**: Easy to add new project types in the future
4. **Clarity**: Clear separation of responsibilities

## Roadmap

```
Phase 1: Foundation (Complete) ████████████████████████████████████████████████████████████
Phase 2: Migration (Complete)  ████████████████████████████████████████████████████████████
Phase 3: Integration          ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Phase 4: Testing              ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Phase 5: Documentation        ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## Phase 1: Foundation - Create Project Module Structure

### Tasks

#### 1.1 Create Module Structure
- [x] Create `src/project/` directory
- [x] Create `src/project/mod.rs` with module declarations
- [x] Create `src/project/types.rs` for core types
- [x] Create `src/project/detector.rs` for project detection
- [x] Create `src/project/manager.rs` for project management
- [x] Create `src/project/validator.rs` for validation logic
- [x] Create `src/project/simple.rs` for simple project implementation
- [x] Create `src/project/tests.rs` for tests

#### 1.2 Define Core Types
- [x] Create `ProjectKind` enum:
  ```rust
  pub enum ProjectKind {
      Simple,
      Monorepo(MonorepoKind),
  }
  ```
- [x] Create `ProjectInfo` trait:
  ```rust
  pub trait ProjectInfo: Send + Sync {
      fn root(&self) -> &Path;
      fn package_manager(&self) -> Option<&PackageManager>;
      fn package_json(&self) -> Option<&PackageJson>;
      fn validation_status(&self) -> &ProjectValidationStatus;
      fn kind(&self) -> ProjectKind;
  }
  ```
- [x] Create `ProjectDescriptor` enum:
  ```rust
  pub enum ProjectDescriptor {
      Simple(SimpleProject),
      Monorepo(MonorepoDescriptor),
  }
  ```

## Phase 2: Migration - Move Generic Code from Monorepo

### Tasks

#### 2.1 Move Core Types
- [x] Move `ProjectConfig` from `monorepo/types.rs` to `project/types.rs`
- [x] Move `ProjectValidationStatus` from `monorepo/types.rs` to `project/types.rs`
- [x] Move `Project` struct from `monorepo/types.rs` to `project/types.rs` (rename to `GenericProject`)
- [x] Update imports in monorepo module

#### 2.2 Move Management Code
- [x] Move `ProjectManager` from `monorepo/project.rs` to `project/manager.rs`
- [x] Extract validation logic from `monorepo/project.rs` to `project/validator.rs`
- [x] Update `ProjectManager` to work with `ProjectInfo` trait
- [x] Remove duplicate code in `monorepo/project.rs` (file completely removed)
- [x] Update `MonorepoDescriptor` to implement `ProjectInfo` trait properly

#### 2.3 Update Package Manager Detection
- [x] Keep `PackageManager` in monorepo module but make it generic for both project types
- [x] Ensure package manager detection works for both simple and monorepo projects
- [x] Update `MonorepoDetector` to detect package manager and load root package.json
- [x] Remove all TODO placeholders from `MonorepoDescriptor` implementation

## Phase 3: Integration - Implement New Functionality

### Tasks

#### 3.1 Implement SimpleProject
- [ ] Create `SimpleProject` struct in `project/simple.rs`:
  ```rust
  pub struct SimpleProject {
      root: PathBuf,
      package_manager: Option<PackageManager>,
      package_json: Option<PackageJson>,
      validation_status: ProjectValidationStatus,
  }
  ```
- [ ] Implement `ProjectInfo` trait for `SimpleProject`
- [ ] Implement construction and detection methods

#### 3.2 Implement ProjectDetector
- [ ] Create unified detector in `project/detector.rs`:
  ```rust
  pub struct ProjectDetector<F: FileSystem> {
      fs: F,
      monorepo_detector: MonorepoDetector<F>,
  }
  ```
- [ ] Implement `detect` method that returns `ProjectDescriptor`
- [ ] Add logic to differentiate between simple and monorepo projects
- [ ] Handle edge cases (no package.json, etc.)

#### 3.3 Update MonorepoDescriptor
- [ ] Make `MonorepoDescriptor` implement `ProjectInfo` trait
- [ ] Ensure all monorepo functionality continues to work
- [ ] Update monorepo detection to use new project abstractions

## Phase 4: Testing and Validation

### Tasks

#### 4.1 Unit Tests
- [ ] Write tests for `SimpleProject` in `project/tests.rs`
- [ ] Write tests for `ProjectDetector`
- [ ] Write tests for trait implementations
- [ ] Test edge cases (empty directories, missing files, etc.)

#### 4.2 Integration Tests
- [ ] Test detection of simple Node.js projects
- [ ] Test detection of various monorepo types
- [ ] Test package manager detection in both contexts
- [ ] Test validation for both project types

#### 4.3 Regression Testing
- [ ] Ensure all existing monorepo tests still pass
- [ ] Verify no breaking changes in public API (where possible)
- [ ] Test with real-world project examples

## Phase 5: Documentation and Finalization

### Tasks

#### 5.1 Update Documentation
- [ ] Update SPEC.md with new project module specification
- [ ] Add examples for simple project usage
- [ ] Document migration guide for breaking changes
- [ ] Update module-level documentation

#### 5.2 Update Public API
- [ ] Export new types from `lib.rs`
- [ ] Deprecate old APIs if necessary
- [ ] Ensure backward compatibility where possible

#### 5.3 Code Examples
- [ ] Create example: Detecting project type
- [ ] Create example: Working with simple projects
- [ ] Create example: Unified API usage
- [ ] Update existing examples

## Implementation Notes

### Breaking Changes
- `Project`, `ProjectManager`, etc. will move from `monorepo` module to `project` module
- Some type names may change (e.g., `Project` → `GenericProject`)

### Migration Strategy
1. Create new module without removing old code
2. Gradually migrate functionality
3. Update tests incrementally
4. Deprecate old APIs before removal

### Performance Considerations
- Minimize filesystem operations during detection
- Cache detection results where appropriate
- Lazy-load package.json content

## Success Criteria

- [ ] All Node.js projects (simple and monorepo) can be detected and managed through unified API
- [ ] No code duplication between simple and monorepo implementations
- [ ] All existing tests pass
- [ ] New functionality is well-tested
- [ ] Documentation is complete and accurate
- [ ] Performance is not degraded

## Session Checkpoints

Each phase can be completed in a separate session:

1. **Session 1**: Complete Phase 1 (Foundation)
2. **Session 2**: Complete Phase 2 (Migration)
3. **Session 3**: Complete Phase 3 (Integration)
4. **Session 4**: Complete Phase 4 (Testing)
5. **Session 5**: Complete Phase 5 (Documentation)

Mark each checkbox as tasks are completed to track progress across sessions.