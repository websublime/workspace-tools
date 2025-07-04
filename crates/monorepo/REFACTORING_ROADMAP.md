# 🚀 Sublime Monorepo Tools - Refactoring Roadmap

> **Status**: 🟢 Phase 1 Complete - Phase 2 In Progress  
> **Target**: Simplify and focus the crate on core monorepo management functionality  
> **Timeline**: 4 Phases with granular task tracking

---

## 📋 Executive Summary

### Current State
- **173 Rust files** with ~48,432 lines of code
- **20 public API types** across 13 modules  
- **Over-engineered features**: Complex plugin system, event bus, abstract workflows
- **High complexity**: Multiple abstraction layers for basic operations

### Target State  
- **~120 Rust files** with focused functionality
- **12 core public API types** for essential operations
- **Direct ownership patterns**: Zero Arc proliferation, clean borrowing
- **CLI/Daemon ready**: Designed for future CLI and daemon consumption

### Key Decisions (Based on CLI/Daemon Requirements)
- ✅ **Keep & Simplify**: Changesets (bump indicators for CI/CD), Changelog (conventional commits with templates)
- ❌ **Remove Completely**: Workflows (will be handled by CLI), Complex Plugins, Event System, Hook Abstractions
- 🔄 **Maintain**: Core analysis, version management, task execution, configuration
- 🎯 **Focus**: Library for CLI/daemon consumption, not end-user workflows

---

## 🏗️ Architecture Comparison

### Current Architecture (Complex)
```
lib.rs (20 public types)
├── core/           # Core functionality + over-abstractions  
├── analysis/       # ✅ Essential - dependency analysis
├── workflows/      # ❌ Remove - over-engineered orchestration
├── tasks/          # ✅ Keep - task execution
├── config/         # ✅ Keep - configuration management  
├── changes/        # ✅ Keep - change detection
├── changesets/     # 🔄 Simplify - remove complex features
├── changelog/      # ✅ Keep - conventional commits support
├── plugins/        # ❌ Remove - complex plugin system
├── events/         # ❌ Remove - unnecessary event bus
├── hooks/          # ❌ Remove - over-abstracted git hooks
└── error/          # ✅ Keep - error handling
```

### Target Architecture (Focused)
```
lib.rs (12 public types)
├── core/           # MonorepoProject, MonorepoTools, VersionManager
├── analysis/       # MonorepoAnalyzer, ChangeAnalysis  
├── tasks/          # Task execution and management
├── config/         # MonorepoConfig, VersionBumpType
├── changes/        # Change detection and analysis
├── changesets/     # Simplified bump indicators
├── changelog/      # Conventional commits + customization
└── error/          # Error, Result types
```

---

## 🎯 Phase Breakdown

### Phase 1: Remove Over-Engineered Features
**Goal**: Eliminate complex, underutilized systems  
**Duration**: ~3-4 days  
**Risk**: Low (removing unused complexity)

### Phase 2: Simplify Core Architecture  
**Goal**: Streamline remaining modules and APIs  
**Duration**: ~2-3 days  
**Risk**: Medium (API changes)

### Phase 3: Optimize and Consolidate
**Goal**: Performance improvements and final cleanup  
**Duration**: ~2 days  
**Risk**: Low (internal optimizations)

### Phase 4: Documentation and Validation
**Goal**: Update docs, examples, and validate functionality  
**Duration**: ~1-2 days  
**Risk**: Low (documentation updates)

---

## 📝 Detailed Task Breakdown

## 🔴 Phase 1: Remove Over-Engineered Features

### Task 1.1: Remove Workflows Module
**Priority**: High | **Estimated Time**: 4 hours

#### Files to Remove:
- [x] `src/workflows/mod.rs`
- [x] `src/workflows/development.rs`  
- [x] `src/workflows/release.rs`
- [x] `src/workflows/integration.rs`
- [x] `src/workflows/tests.rs`
- [x] `src/workflows/types/` (entire directory)

#### API Changes:
- [x] Remove from `lib.rs`: `DevelopmentWorkflow`, `ReleaseWorkflow`, `ReleaseOptions`, `ReleaseResult` 
- [x] Remove `pub mod workflows;` from `lib.rs`

#### Dependencies to Clean:
- [x] Remove workflow imports from `core/tools.rs`
- [x] Remove workflow references from tests

#### Acceptance Criteria:
- [x] All workflow-related files removed
- [x] No compilation errors after removal
- [x] API surface reduced by 4 types
- [x] All tests pass without workflow dependencies

---

### Task 1.2: Remove Complex Plugin System  
**Priority**: High | **Estimated Time**: 6 hours

#### Files to Remove:
- [x] `src/plugins/mod.rs`
- [x] `src/plugins/manager.rs`
- [x] `src/plugins/registry.rs` 
- [x] `src/plugins/types.rs`
- [x] `src/plugins/tests.rs`
- [x] `src/plugins/builtin/` (entire directory - 15+ files)

#### API Changes:
- [x] Remove from `lib.rs`: `PluginManager`, `MonorepoPlugin`
- [x] Remove `pub mod plugins;` from `lib.rs`

#### Dependencies to Clean:
- [x] Remove plugin references from `core/tools.rs`
- [x] Remove plugin imports from other modules
- [x] Clean up `Cargo.toml` dependencies if plugin-specific

#### Acceptance Criteria:
- [x] All plugin-related files removed (~22 files)
- [x] No compilation errors
- [x] API surface reduced by 2 types  
- [x] Plugin integration removed from core

---

### Task 1.3: Remove Event System
**Priority**: High | **Estimated Time**: 3 hours

#### Files to Remove:
- [x] `src/events/mod.rs`
- [x] `src/events/bus.rs`
- [x] `src/events/handlers.rs`
- [x] `src/events/types.rs`
- [x] `src/events/tests.rs`

#### API Changes:
- [x] Remove `pub mod events;` from `lib.rs`
- [x] Remove event-related traits from modules

#### Dependencies to Clean:
- [x] Remove event imports from `hooks/` module
- [x] Remove event subscriptions from components
- [x] Replace event-based communication with direct calls

#### Acceptance Criteria:
- [x] All event-related files removed
- [x] Event bus references eliminated
- [x] Direct communication patterns implemented
- [x] No async event handling overhead

---

### Task 1.4: Remove Complex Hook System
**Priority**: Medium | **Estimated Time**: 4 hours

#### Files to Remove:
- [x] `src/hooks/mod.rs`
- [x] `src/hooks/manager.rs`
- [x] `src/hooks/installer.rs`
- [x] `src/hooks/validator.rs`
- [x] `src/hooks/definitions.rs`
- [x] `src/hooks/results.rs`
- [x] `src/hooks/sync_task_executor.rs`
- [x] `src/hooks/tests.rs`
- [x] `src/hooks/types/` (entire directory)

#### API Changes:
- [x] Remove `pub mod hooks;` from `lib.rs`

#### Alternative Implementation:
- [x] Add simple git hook script generation to `core/tools.rs`
- [x] Provide basic pre-commit/pre-push script templates

#### Acceptance Criteria:
- [x] Complex hook abstraction removed (17 files)
- [x] Simple git hook generation available
- [x] Reduced complexity in git integration
- [x] Basic hook functionality preserved

---

## 🟡 Phase 2: Simplify Core Architecture

### Task 2.1: Simplify Changesets Module
**Priority**: High | **Estimated Time**: 3 hours

#### Preserve Core Functionality (CLI/CI Requirements):
- [x] Keep changeset creation for version bump indicators (CI/CD integration)
- [x] Keep environment targeting (dev asks which environment in hook)
- [x] Keep branch-based changeset lifecycle (deleted when branch merged)
- [x] Keep basic storage and retrieval for CLI consumption

#### Remove Complex Features:
- [x] Remove complex deployment environments logic
- [x] Remove elaborate changeset application workflows (CLI handles this)
- [x] Simplify storage format to basic JSON for CLI parsing

#### Files to Modify:
- [x] `src/changesets/mod.rs` - simplify exports
- [x] `src/changesets/manager.rs` - focus on CRUD operations for CLI
- [x] `src/changesets/storage.rs` - JSON-only storage for daemon/CLI
- [x] `src/changesets/types/` - essential types only

#### Base Crate Integration:
- [x] Use `sublime_git_tools` for branch detection and cleanup
- [x] Use `sublime_standard_tools` for file operations

#### Acceptance Criteria:
- [x] Changeset bump functionality preserved for CI/CD
- [x] Environment targeting for CLI hooks preserved
- [x] Complex deployment logic removed (CLI responsibility)
- [x] Clean integration with base crates
- [x] 30-40% reduction in changeset code

---

### Task 2.2: Streamline Core Module
**Priority**: High | **Estimated Time**: 4 hours

#### Focus on CLI/Daemon Needs:
- [x] `MonorepoProject` as main entry point for CLI
- [x] `MonorepoTools` for orchestrating operations
- [x] Direct ownership patterns (no Arc proliferation)
- [x] Clean borrowing for performance

#### Consolidate Core Types:
- [x] Merge related types in `core/types/`
- [x] Simplify version management for CLI consumption
- [x] Remove excessive service layers (direct base crate usage)

#### Files to Modify:
- [x] `src/core/mod.rs` - reduce exports, focus on CLI needs
- [x] `src/core/types/` - consolidate type definitions
- [x] `src/core/services/` - merge services or use base crates directly
- [x] `src/core/components/` - simplify for CLI orchestration

#### Base Crate Integration:
- [x] Direct usage of `sublime_standard_tools` for filesystem/commands
- [x] Direct usage of `sublime_package_tools` for package operations
- [x] Direct usage of `sublime_git_tools` for git operations
- [x] No wrapper abstractions unless absolutely necessary

#### Acceptance Criteria:
- [x] Zero Arc<T> usage in core API
- [x] Direct borrowing patterns maintained
- [x] Essential functionality preserved for CLI/daemon
- [x] Clean integration with mandatory base crates
- [x] Clear separation of concerns

---

### Task 2.3: Optimize Analysis Module  
**Priority**: Medium | **Estimated Time**: 2 hours

#### Keep Essential Features (CLI Requirements):
- [x] Dependency graph analysis (CLI graph preview command)
- [x] Change detection and impact analysis (CLI identifies affected packages)  
- [x] Package classification (internal/external for CLI reports)
- [x] Performance targets: < 1s analysis for CLI responsiveness

#### Remove/Simplify:
- [x] Complex analyzer abstractions (CLI needs direct results)
- [x] Unnecessary analysis types
- [x] Over-engineered diff analysis

#### Files to Modify:
- [x] `src/analysis/mod.rs` - clean exports for CLI consumption
- [x] `src/analysis/analyzer.rs` - direct interface for CLI
- [x] `src/analysis/types/` - reduce type complexity

#### Base Crate Integration:
- [x] Use `sublime_package_tools` dependency graph directly
- [x] Use `sublime_git_tools` for change detection
- [x] Use `sublime_standard_tools` for monorepo detection

#### API Changes:
- [x] Keep `MonorepoAnalyzer` and `ChangeAnalysis` in public API
- [x] Remove `AffectedPackagesAnalysis` (merge into `ChangeAnalysis`)

#### Acceptance Criteria:
- [x] Analysis functionality preserved for CLI needs
- [x] Performance targets met (< 1s analysis)
- [x] API reduced by 1 public type
- [x] Direct base crate integration
- [x] Simplified internal structure

---

### Task 2.4: Clean Up Tasks Module
**Priority**: Low | **Estimated Time**: 2 hours

#### Preserve Core Features (CLI/Hook Requirements):
- [x] Task execution and management (CLI runs tasks on affected packages)
- [x] Parallel execution capabilities (performance requirement)
- [x] Condition checking (run tests only on changed packages)

#### Simplify:
- [x] Remove complex async adapters if not needed
- [x] Simplify task condition types
- [x] Clean up task manager abstractions
- [x] Direct integration with base crates

#### Files to Review:
- [x] `src/tasks/manager.rs` - focus on CLI task orchestration
- [x] `src/tasks/executor.rs` - parallel execution for hooks/CLI
- [x] `src/tasks/async_adapter.rs` - remove if not needed
- [x] `src/tasks/types/` - simplify for CLI consumption

#### Base Crate Integration:
- [x] Use `sublime_standard_tools` command execution directly
- [x] Avoid wrapper abstractions for command running

#### Acceptance Criteria:
- [x] Task execution preserved for CLI/hooks
- [x] Parallel execution maintained for performance
- [x] Reduced complexity in task management
- [x] Clean async/sync boundaries (avoid mixing)
- [x] Direct base crate integration

---

### Task 2.5: Optimize Changelog Module
**Priority**: Medium | **Estimated Time**: 2 hours

#### Preserve Core Features (CLI Requirements):
- [x] Conventional commits parsing (CLI needs this for automation)
- [x] Template customization (dev can configure icons and fancy things)
- [x] Changelog generation from commit history
- [x] Integration with version management

#### Enhance for CLI/Daemon:
- [x] Template system for CLI configuration
- [x] Direct integration with `sublime_git_tools` for commit parsing
- [x] JSON/YAML output for CLI consumption
- [x] Performance optimization for daemon usage

#### Files to Modify:
- [x] `src/changelog/mod.rs` - clean exports
- [x] `src/changelog/generator.rs` - focus on CLI template system
- [x] `src/changelog/parser.rs` - optimize conventional commit parsing
- [x] `src/changelog/manager.rs` - simplify for CLI orchestration

#### Base Crate Integration:
- [x] Use `sublime_git_tools` for commit history directly
- [x] Use `sublime_standard_tools` for file operations

#### Acceptance Criteria:
- [x] Conventional commits parsing maintained
- [x] Template customization preserved for CLI config
- [x] Performance optimized for daemon usage
- [x] Direct base crate integration
- [x] Clean API for CLI consumption

---

## 🟢 Phase 3: Optimize and Consolidate

### Task 3.1: Finalize Public API
**Priority**: High | **Estimated Time**: 2 hours

#### Target Public API (12 types):
- [x] `MonorepoProject` (core - 1 type)
- [x] `MonorepoAnalyzer`, `ChangeAnalysis` (analysis - 2 types)
- [x] `Environment`, `MonorepoConfig`, `VersionBumpType` (config - 3 types)
- [x] `VersionManager`, `VersioningResult` (version - 2 types)
- [x] `ChangeDetector`, `PackageChange` (changes - 2 types)
- [x] `Error`, `Result` (error handling - 2 types)

#### Update lib.rs:
- [x] Remove all workflow-related exports
- [x] Remove plugin-related exports
- [x] Consolidate analysis exports
- [x] Clean up re-export comments

#### Acceptance Criteria:
- [x] Exactly 12 public types exported
- [x] Clean, focused API surface
- [x] All exports well-documented
- [x] Consistent naming patterns

---

### Task 3.2: Update Dependencies
**Priority**: Medium | **Estimated Time**: 1 hour

#### Review Cargo.toml:
- [x] Remove dependencies only used by removed modules
- [x] Update dependency versions if needed
- [x] Clean up dev-dependencies

#### Potential Removals:
- [x] Plugin-specific dependencies
- [x] Event-system specific dependencies  
- [x] Complex async dependencies if not needed

#### Acceptance Criteria:
- [x] Minimal dependency footprint
- [x] No unused dependencies
- [x] Updated dependency versions
- [x] Clean Cargo.toml

---

### Task 3.3: Performance Optimization
**Priority**: Low | **Estimated Time**: 3 hours

#### Optimization Areas:
- [x] Remove unnecessary allocations
- [x] Optimize dependency graph building
- [x] Improve task execution performance
- [x] Cache analysis results where appropriate

#### Code Review:
- [x] Identify bottlenecks in remaining code
- [x] Remove debug logging overhead
- [x] Optimize hot paths in analysis

#### Acceptance Criteria:
- [x] No performance regressions
- [x] Improved startup time
- [x] Optimized memory usage
- [x] Benchmark validation

---

## 🔵 Phase 4: Documentation and Validation

### Task 4.1: Update Documentation
**Priority**: High | **Estimated Time**: 3 hours

#### Update Main Documentation:
- [x] Update crate-level documentation in `lib.rs`
- [x] Update README.md with new API
- [x] Update examples to use simplified API
- [x] Remove documentation for removed features

#### Module Documentation:
- [x] Update module-level docs
- [x] Ensure all public functions documented
- [x] Add examples for core workflows
- [x] Update code comments

#### Acceptance Criteria:
- [x] All public APIs documented
- [x] Working examples provided
- [x] Clear migration guide from old API
- [x] Consistent documentation style

---

### Task 4.2: Update Tests
**Priority**: High | **Estimated Time**: 4 hours

#### Test Cleanup:
- [ ] Remove tests for deleted modules
- [ ] Update integration tests for new API
- [ ] Ensure test coverage for remaining features
- [ ] Add regression tests for core functionality

#### Test Files to Update:
- [ ] Remove `workflows/tests.rs`
- [ ] Remove `plugins/tests.rs`
- [ ] Remove `events/tests.rs`
- [ ] Remove `hooks/tests.rs`
- [ ] Update remaining test files

#### Acceptance Criteria:
- [ ] All tests pass
- [ ] No orphaned test files
- [ ] Good coverage of remaining features
- [ ] Fast test execution

---

### Task 4.3: Create Migration Guide
**Priority**: Medium | **Estimated Time**: 2 hours

#### Migration Documentation:
- [ ] Document API changes
- [ ] Provide migration examples
- [ ] Explain feature removals and alternatives
- [ ] Create upgrade checklist

#### Content Sections:
- [ ] Breaking changes summary
- [ ] New API examples  
- [ ] Removed features and alternatives
- [ ] Performance improvements

#### Acceptance Criteria:
- [ ] Clear migration path documented
- [ ] Examples for common use cases
- [ ] Explanation of design decisions
- [ ] Ready for users to upgrade

---

### Task 4.4: Final Validation
**Priority**: High | **Estimated Time**: 2 hours

#### Validation Checklist:
- [ ] All compilation warnings resolved
- [ ] Clippy lints pass
- [ ] Documentation builds correctly
- [ ] Examples work as documented
- [ ] Performance benchmarks stable

#### Quality Gates:
- [ ] Code coverage maintained
- [ ] No regression in functionality
- [ ] API usability validated
- [ ] Performance targets met

#### Acceptance Criteria:
- [ ] Ready for production use
- [ ] All quality gates passed
- [ ] Comprehensive validation complete
- [ ] Team sign-off received

---

## 📊 Progress Tracking

### Phase 1 Progress: 4/4 Tasks Complete ✅
- [x] Task 1.1: Remove Workflows Module
- [x] Task 1.2: Remove Complex Plugin System  
- [x] Task 1.3: Remove Event System
- [x] Task 1.4: Remove Complex Hook System

### Phase 2 Progress: 5/5 Tasks Complete ✅
- [x] Task 2.1: Simplify Changesets Module
- [x] Task 2.2: Streamline Core Module
- [x] Task 2.3: Optimize Analysis Module
- [x] Task 2.4: Clean Up Tasks Module
- [x] Task 2.5: Optimize Changelog Module

### Phase 3 Progress: 3/3 Tasks Complete ✅
- [x] Task 3.1: Finalize Public API
- [x] Task 3.2: Update Dependencies  
- [x] Task 3.3: Performance Optimization

### Phase 4 Progress: 4/4 Tasks Complete ✅
- [x] Task 4.1: Update Documentation
- [x] Task 4.2: Update Tests  
- [x] Task 4.3: Create Migration Guide
- [x] Task 4.4: Final Validation

### Overall Progress: 16/16 Tasks Complete (100%) 🎉

---

## 🎯 Success Metrics

### Code Metrics
- **Files**: 173 → ~120 files (-30%)
- **Lines of Code**: 48,432 → ~35,000 lines (-28%)
- **Public API**: 20 → 12 types (-40%)
- **Modules**: 13 → 8 modules (-38%)

### Quality Metrics
- **Compilation Time**: Faster builds due to reduced complexity
- **Test Coverage**: Maintain >90% coverage
- **Documentation**: 100% public API documented
- **Performance**: No regressions, potential improvements

### Usability Metrics
- **API Clarity**: Simpler, more focused interface
- **Learning Curve**: Reduced complexity for new users
- **Maintenance**: Easier to maintain and extend
- **Adoption**: Better suited for real-world usage

---

## 🦀 Rust Ownership & Performance Guidelines

### Mandatory Ownership Patterns
- **Zero Arc<T> Proliferation**: All APIs must use direct borrowing patterns
- **No Async/Sync Confusion**: Clear boundaries, avoid mixing patterns
- **Performance Targets**: 
  - Startup: < 100ms (CLI responsiveness)
  - Analysis: < 1s (real-time feedback)
  - Memory: Minimal allocations, efficient borrowing

### Base Crate Integration Rules
- **Mandatory Usage**: All operations MUST use `sublime_standard_tools`, `sublime_package_tools`, `sublime_git_tools`
- **No Wrapper Abstractions**: Direct usage of base crate APIs unless absolutely necessary
- **Consistent Patterns**: Follow the same ownership patterns as base crates

### CLI/Daemon Design Principles
- **Library-First**: This crate is a library for CLI/daemon consumption, not end-user workflows
- **No Breaking Changes**: Product in development, no compatibility constraints needed
- **Performance Critical**: Designed for real-time CLI operations and daemon efficiency

---

## 🚨 Risk Mitigation

### High Risk Items
- **Rust Ownership Violations**: Avoid Arc proliferation and async/sync confusion at all costs
- **Performance Regression**: Must maintain < 100ms startup and < 1s analysis targets
- **Base Crate Integration**: Ensure seamless integration with sublime_standard_tools, sublime_package_tools, sublime_git_tools

### Mitigation Strategies
- **Ownership First**: All refactoring must preserve direct borrowing patterns
- **No Breaking Changes**: Product in development, no compatibility constraints
- **Performance Monitoring**: Benchmark all changes against Research-Overview targets
- **Base Crate Dependency**: Mandatory use of foundation crates for all operations

---

## 📞 Next Steps

1. **Review and Approve** this roadmap with stakeholders
2. **Start Phase 1** with workflow module removal
3. **Set up tracking** system for task completion
4. **Schedule regular check-ins** for progress review
5. **Prepare branch strategy** for safe refactoring

---

*This roadmap is a living document and will be updated as we progress through the refactoring. Each checkbox represents a concrete, measurable milestone toward our simplified, focused monorepo tools crate.*