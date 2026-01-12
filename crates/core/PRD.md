# Product Requirements Document: workspace-core

## Document Information

| Field | Value |
|-------|-------|
| **Crate Name** | `workspace-core` |
| **Version** | `0.1.0` |
| **Status** | Draft |
| **Created** | 2026-01-12 |
| **Last Updated** | 2026-01-12 |

---

## 1. Executive Summary

### 1.1 Purpose

The `workspace-core` crate provides the foundational abstractions and detection mechanisms for working with JavaScript/TypeScript projects from Rust. It serves as the core building block for the workspace-node-tools ecosystem, enabling reliable detection of project types (single-package or monorepo) and package managers.

### 1.2 Scope

This crate focuses exclusively on:

- **Package Manager Detection**: Identifying which package manager (npm, yarn, pnpm, bun, deno) is used in a project
- **Project Type Detection**: Determining if a project is a simple single-package repository or a monorepo
- **Monorepo Analysis**: Detecting monorepo type, discovering workspace packages, and analyzing internal dependencies
- **Core Abstractions**: Providing reusable types and traits for the ecosystem

### 1.3 Out of Scope

The following concerns are explicitly **not** part of this crate:

- Command execution (delegated to a separate `workspace-executor` crate)
- Filesystem operations beyond detection (delegated to a separate `workspace-fs` crate)
- Git operations (delegated to a separate `workspace-git` crate)
- Version management and changesets (delegated to higher-level crates)
- CLI interfaces (delegated to the `workspace` CLI crate)

---

## 2. Problem Statement

### 2.1 Current Challenges

When building tools that interact with JavaScript/TypeScript projects from Rust, developers face several challenges:

1. **Package Manager Fragmentation**: The Node.js ecosystem has multiple package managers (npm, yarn, pnpm, bun, deno), each with different lock files, commands, and workspace configurations.

2. **Monorepo Complexity**: Monorepos can use different workspace implementations (npm/yarn workspaces, pnpm workspaces, bun workspaces, deno workspaces, or custom configurations), requiring format-specific detection logic.

3. **Inconsistent Detection**: Without a unified approach, tools often implement ad-hoc detection that fails in edge cases or doesn't handle all package managers.

4. **Type Safety**: Rust's type system can prevent many errors, but only if the abstractions are well-designed and comprehensive.

### 2.2 Solution

The `workspace-core` crate provides:

- A unified detection API that works across all major package managers
- Type-safe abstractions for project and repository concepts
- Configurable detection with sensible defaults
- Clear separation between simple projects and monorepos
- Workspace package discovery and dependency analysis

---

## 3. User Personas

### 3.1 Primary Users

| Persona | Description | Needs |
|---------|-------------|-------|
| **Rust Library Developer** | Building tools that interact with Node.js projects | Reliable detection, clear APIs, good documentation |
| **CLI Tool Author** | Creating command-line tools for Node.js workflows | Simple integration, configurable behavior, accurate detection |
| **Monorepo Tool Builder** | Building specialized monorepo management tools | Complete workspace analysis, dependency graphs, package discovery |

### 3.2 Use Cases

#### UC-1: Package Manager Detection

**Actor**: Any user  
**Goal**: Determine which package manager is used in a project  
**Precondition**: A valid Node.js project directory exists  
**Flow**:
1. User provides a path to a project directory
2. System checks for lock files in priority order
3. System returns the detected package manager kind and relevant metadata

#### UC-2: Project Type Detection

**Actor**: Any user  
**Goal**: Determine if a project is a simple repository or monorepo  
**Precondition**: A valid Node.js project directory exists  
**Flow**:
1. User provides a path to a project directory
2. System detects the package manager
3. System checks for workspace configuration
4. System returns the project type (simple or monorepo with specific kind)

#### UC-3: Monorepo Analysis

**Actor**: Monorepo Tool Builder  
**Goal**: Get complete information about a monorepo structure  
**Precondition**: A valid monorepo root directory exists  
**Flow**:
1. User provides a path to a monorepo root
2. System detects the monorepo kind
3. System discovers all workspace packages
4. System analyzes internal dependencies between packages
5. System returns a complete monorepo descriptor

#### UC-4: Find Project Root

**Actor**: Any user  
**Goal**: Find the root of a project from any subdirectory  
**Precondition**: Current directory is inside a Node.js project  
**Flow**:
1. User provides a starting path (or uses current directory)
2. System walks up the directory tree
3. System returns the project root and type

---

## 4. Functional Requirements

### 4.1 Package Manager Module (`node`)

#### FR-1.1: Package Manager Kind Enumeration

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1.1 | System SHALL support npm package manager detection | P0 |
| FR-1.1.2 | System SHALL support yarn package manager detection | P0 |
| FR-1.1.3 | System SHALL support pnpm package manager detection | P0 |
| FR-1.1.4 | System SHALL support bun package manager detection | P0 |
| FR-1.1.5 | System SHALL support deno package manager detection | P0 |

#### FR-1.2: Package Manager Detection

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.2.1 | System SHALL detect package manager by lock file presence | P0 |
| FR-1.2.2 | System SHALL support configurable detection order | P1 |
| FR-1.2.3 | System SHALL support environment variable override for package manager | P2 |
| FR-1.2.4 | System SHALL support custom lock file names | P2 |
| FR-1.2.5 | System SHALL provide fallback package manager configuration | P2 |

#### FR-1.3: Package Manager Metadata

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.3.1 | System SHALL provide command name for each package manager | P0 |
| FR-1.3.2 | System SHALL provide lock file name for each package manager | P0 |
| FR-1.3.3 | System SHALL indicate workspace support for each package manager | P0 |
| FR-1.3.4 | System SHALL provide workspace config file path when applicable | P1 |

### 4.2 Repository Module (`node`)

#### FR-2.1: Repository Kind

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1.1 | System SHALL distinguish between simple and monorepo repositories | P0 |
| FR-2.1.2 | System SHALL provide monorepo kind for monorepo repositories | P0 |
| FR-2.1.3 | System SHALL provide human-readable names for repository kinds | P1 |

### 4.3 Project Module (`project`)

#### FR-3.1: Project Detection

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1.1 | System SHALL detect projects from any valid project path | P0 |
| FR-3.1.2 | System SHALL find project root from any subdirectory | P0 |
| FR-3.1.3 | System SHALL validate project structure (package.json presence) | P1 |
| FR-3.1.4 | System SHALL support detection with custom configuration | P1 |

#### FR-3.2: Project Information

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.2.1 | System SHALL provide project root path | P0 |
| FR-3.2.2 | System SHALL provide detected package manager | P0 |
| FR-3.2.3 | System SHALL provide project kind (simple/monorepo) | P0 |
| FR-3.2.4 | System SHALL provide parsed package.json content | P1 |
| FR-3.2.5 | System SHALL provide validation status | P2 |

#### FR-3.3: Project Dependencies

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.3.1 | System SHALL parse production dependencies | P1 |
| FR-3.3.2 | System SHALL parse development dependencies | P1 |
| FR-3.3.3 | System SHALL parse optional dependencies | P2 |
| FR-3.3.4 | System SHALL parse peer dependencies | P2 |

### 4.4 Monorepo Module (`monorepo`)

#### FR-4.1: Monorepo Kind Detection

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.1.1 | System SHALL detect npm workspaces configuration | P0 |
| FR-4.1.2 | System SHALL detect yarn workspaces configuration | P0 |
| FR-4.1.3 | System SHALL detect pnpm workspaces configuration | P0 |
| FR-4.1.4 | System SHALL detect bun workspaces configuration | P0 |
| FR-4.1.5 | System SHALL detect deno workspaces configuration | P0 |
| FR-4.1.6 | System SHALL support custom monorepo configurations | P2 |

#### FR-4.2: Workspace Package Discovery

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.2.1 | System SHALL discover all packages matching workspace patterns | P0 |
| FR-4.2.2 | System SHALL respect exclusion patterns | P0 |
| FR-4.2.3 | System SHALL provide package name and version | P0 |
| FR-4.2.4 | System SHALL provide relative and absolute paths | P0 |
| FR-4.2.5 | System SHALL support configurable search depth | P1 |

#### FR-4.3: Workspace Dependency Analysis

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.3.1 | System SHALL identify internal workspace dependencies | P0 |
| FR-4.3.2 | System SHALL identify internal dev dependencies | P0 |
| FR-4.3.3 | System SHALL generate dependency graph between packages | P1 |
| FR-4.3.4 | System SHALL detect circular dependencies | P2 |

#### FR-4.4: Monorepo Descriptor

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.4.1 | System SHALL provide monorepo kind | P0 |
| FR-4.4.2 | System SHALL provide root path | P0 |
| FR-4.4.3 | System SHALL provide list of all packages | P0 |
| FR-4.4.4 | System SHALL provide package lookup by name | P0 |
| FR-4.4.5 | System SHALL find package containing a given path | P1 |

### 4.5 Configuration Module (`config`)

#### FR-5.1: Detection Configuration

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-5.1.1 | System SHALL provide default configuration | P0 |
| FR-5.1.2 | System SHALL support package manager detection order configuration | P1 |
| FR-5.1.3 | System SHALL support workspace pattern configuration | P1 |
| FR-5.1.4 | System SHALL support exclusion pattern configuration | P1 |
| FR-5.1.5 | System SHALL support search depth configuration | P1 |

### 4.6 Error Handling (`error`)

#### FR-6.1: Error Types

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-6.1.1 | System SHALL provide specific error types for each failure mode | P0 |
| FR-6.1.2 | System SHALL include context information in errors | P0 |
| FR-6.1.3 | System SHALL implement std::error::Error for all error types | P0 |
| FR-6.1.4 | System SHALL provide actionable error messages | P1 |

---

## 5. Non-Functional Requirements

### 5.1 Performance

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-1.1 | Package manager detection | < 10ms for typical project |
| NFR-1.2 | Project type detection | < 50ms for typical project |
| NFR-1.3 | Full monorepo analysis | < 500ms for 100 packages |
| NFR-1.4 | Memory usage | < 50MB for 100-package monorepo |

### 5.2 Reliability

| ID | Requirement |
|----|-------------|
| NFR-2.1 | System SHALL handle missing files gracefully |
| NFR-2.2 | System SHALL handle permission errors appropriately |
| NFR-2.3 | System SHALL handle malformed configuration files |
| NFR-2.4 | System SHALL handle symlinks according to configuration |

### 5.3 Compatibility

| ID | Requirement |
|----|-------------|
| NFR-3.1 | System SHALL work on Windows, macOS, and Linux |
| NFR-3.2 | System SHALL handle platform-specific path separators |
| NFR-3.3 | System SHALL support Rust stable (MSRV: 1.75+) |

### 5.4 Code Quality

| ID | Requirement |
|----|-------------|
| NFR-4.1 | All public APIs SHALL be documented with examples |
| NFR-4.2 | Code coverage SHALL exceed 80% |
| NFR-4.3 | All clippy warnings SHALL be addressed |
| NFR-4.4 | No unsafe code without explicit justification |

---

## 6. Architecture Overview

### 6.1 Module Structure

```
workspace-core/
├── src/
│   ├── lib.rs              # Crate root with re-exports
│   ├── node/               # Node.js abstractions
│   │   ├── mod.rs
│   │   ├── package_manager.rs
│   │   ├── repository.rs
│   │   ├── types.rs
│   │   └── tests.rs
│   ├── project/            # Project detection and management
│   │   ├── mod.rs
│   │   ├── detector.rs
│   │   ├── project.rs
│   │   ├── types.rs
│   │   └── tests.rs
│   ├── monorepo/           # Monorepo-specific functionality
│   │   ├── mod.rs
│   │   ├── detector.rs
│   │   ├── descriptor.rs
│   │   ├── types.rs
│   │   ├── workspace.rs
│   │   └── tests.rs
│   ├── config/             # Configuration types
│   │   ├── mod.rs
│   │   ├── detection.rs
│   │   └── tests.rs
│   └── error/              # Error types
│       ├── mod.rs
│       └── types.rs
└── tests/
    └── integration/        # E2E tests
```

### 6.2 Dependency Graph

```
┌─────────────────────────────────────────────────────────────┐
│                      workspace-core                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────┐    ┌──────────────┐    ┌─────────────────┐   │
│  │  error   │◄───│    node      │◄───│    project      │   │
│  └──────────┘    └──────────────┘    └─────────────────┘   │
│       ▲                ▲                     ▲              │
│       │                │                     │              │
│       │          ┌─────┴─────┐               │              │
│       └──────────│  monorepo │───────────────┘              │
│                  └───────────┘                              │
│                        ▲                                    │
│                        │                                    │
│                  ┌─────┴─────┐                              │
│                  │  config   │                              │
│                  └───────────┘                              │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 6.3 Key Design Principles

1. **Sync-First API**: Detection operations use synchronous I/O for simplicity. Async wrappers can be provided by consuming crates if needed.

2. **Zero Unsafe**: No unsafe code in this crate.

3. **Minimal Dependencies**: Only essential dependencies (serde, thiserror, walkdir).

4. **Trait-Based Abstractions**: Core behaviors defined as traits for testability and extensibility.

5. **Builder Pattern**: Complex types constructed via builders for ergonomic APIs.

---

## 7. API Design Principles

### 7.1 Naming Conventions

- **Types**: PascalCase (e.g., `PackageManagerKind`, `MonorepoDescriptor`)
- **Functions**: snake_case (e.g., `detect_package_manager`, `find_workspace_packages`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `DEFAULT_SEARCH_DEPTH`)
- **Modules**: snake_case (e.g., `package_manager`, `project`)

### 7.2 Error Handling

- All fallible operations return `Result<T, Error>`
- Specific error variants for each failure mode
- Context preserved through error chain
- `thiserror` for derive macros

### 7.3 Documentation Standards

- All public items documented
- Module-level docs with What/How/Why
- Examples for all public functions
- Cross-references between related items

---

## 8. Success Criteria

### 8.1 Acceptance Criteria

| Criterion | Measurement |
|-----------|-------------|
| All P0 requirements implemented | 100% coverage |
| All P1 requirements implemented | 100% coverage |
| Unit test coverage | > 80% |
| Integration tests passing | 100% |
| Documentation complete | All public APIs |
| Clippy clean | Zero warnings |

### 8.2 Quality Gates

1. **Code Review**: All code reviewed by at least one other developer
2. **CI/CD**: All tests pass on Windows, macOS, and Linux
3. **Documentation**: Generated docs reviewed for completeness
4. **Performance**: Benchmarks meet NFR targets

---

## 9. Future Considerations

### 9.1 Potential Extensions (Not in Scope)

- Turbo monorepo support
- Nx monorepo support
- Lerna monorepo support
- Rush monorepo support
- Custom package manager plugins
- Project scaffolding

### 9.2 Migration Path

The crate is designed to eventually replace the functionality in `temp/wnt-stable/crates/standard`, but with:

- Cleaner module boundaries
- Sync-first APIs (vs async-first in the old crate)
- Better separation of concerns
- Improved error handling

---

## 10. Glossary

| Term | Definition |
|------|------------|
| **Package Manager** | Tool for managing JavaScript dependencies (npm, yarn, pnpm, bun, deno) |
| **Lock File** | File that records exact dependency versions (package-lock.json, yarn.lock, etc.) |
| **Monorepo** | Repository containing multiple packages managed together |
| **Workspace** | Package manager feature for managing multiple packages in one repository |
| **Workspace Package** | Individual package within a monorepo workspace |
| **Project Root** | Directory containing the root package.json |

---

## 11. References

- [npm Workspaces Documentation](https://docs.npmjs.com/cli/v7/using-npm/workspaces)
- [Yarn Workspaces Documentation](https://yarnpkg.com/features/workspaces)
- [pnpm Workspaces Documentation](https://pnpm.io/workspaces)
- [Bun Workspaces Documentation](https://bun.sh/docs/install/workspaces)
- [Deno Workspaces Documentation](https://deno.land/manual/workspaces)
- [Original Crate Specification](../../temp/wnt-stable/crates/standard/SPEC.md)