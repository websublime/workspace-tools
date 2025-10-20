# Test Infrastructure for sublime_pkg_tools

This directory contains comprehensive test infrastructure and integration tests for the `sublime_pkg_tools` crate, covering **Epics 1-5** (Foundation through Versioning Engine).

## Overview

The test infrastructure provides:

- **Mock Implementations**: In-memory mocks for filesystem, Git, and NPM registry
- **Test Fixtures**: Pre-built test data structures (monorepos, single packages)
- **Assertion Helpers**: Custom assertions for common test scenarios
- **Property-Based Testing**: Generators for proptest-based testing
- **Integration Tests**: Comprehensive end-to-end workflow tests

## Test Coverage

### Unit Tests (549 tests in src/)
- Configuration system (`config/`)
- Error handling (`error/`)
- Core types (`types/`)
- Version resolution (`version/`)

### Integration Tests (198 tests in tests/)
- **test_infrastructure.rs** (95 tests): Validates test utilities and fixtures
- **version_resolution_integration.rs** (103 tests): Complete version resolution workflows
  - Independent and unified strategies
  - Dependency propagation
  - Circular dependency detection
  - Single-package and monorepo scenarios
  - Dry-run and apply modes
  - Performance and stress tests

### Total: 747 tests with 100% pass rate

## Directory Structure

```
tests/
├── common/
│   ├── mod.rs              # Main test utilities module
│   ├── assertions.rs       # Custom assertion helpers
│   ├── fixtures.rs         # Test fixture builders
│   ├── generators.rs       # Proptest generators
│   └── mocks/
│       ├── mod.rs          # Mock implementations module
│       ├── filesystem.rs   # MockFileSystem
│       ├── git.rs          # MockGitRepository
│       └── registry.rs     # MockRegistry
├── fixtures/
│   ├── monorepo/           # Sample monorepo structure
│   └── single-package/     # Sample single package
├── test_infrastructure.rs        # Infrastructure validation tests (95 tests)
└── version_resolution_integration.rs  # Version resolution workflows (103 tests)
```

## Integration Test Scenarios

### Single Package Repository
- Basic version bumps (major, minor, patch)
- Version application and file modification
- Error handling for invalid scenarios

### Monorepo Workflows

#### Independent Strategy
- Packages versioned independently
- Dependency propagation between workspace packages
- Selective package updates
- Multi-depth dependency chains

#### Unified Strategy
- All packages maintain same version
- Unified version bumps across workspace
- Version synchronization

### Advanced Scenarios
- Circular dependency detection and prevention
- Workspace protocol preservation (`workspace:*`)
- Maximum depth propagation limits
- Dev dependency propagation control
- Dry-run vs actual application
- JSON formatting preservation

### Performance Tests
- Large monorepo handling (50+ packages)
- Deep dependency chains (10+ levels)
- Resolution speed benchmarks
- Apply operation performance

## Using the Test Infrastructure

### Creating Test Fixtures

```rust
use crate::common::fixtures::MonorepoFixtureBuilder;

let fixture = MonorepoFixtureBuilder::new("my-workspace")
    .add_package("packages/core", "core", "1.0.0")
    .add_package("packages/ui", "ui", "1.0.0")
    .build();

// Write to temporary directory
let temp = create_temp_dir().unwrap();
fixture.write_to_dir(temp.path()).unwrap();
```

### Mock Filesystem

The `MockFileSystem` provides an in-memory filesystem for testing without touching the real filesystem.

```rust
use crate::common::mocks::filesystem::MockFileSystem;

#[tokio::test]
async fn test_with_mock_fs() {
    let fs = MockFileSystem::new();
    
    // Write a file
    fs.write_file_string("/test.txt", "content").await.unwrap();
    
    // Read it back
    let content = fs.read_file_string("/test.txt").await.unwrap();
    assert_eq!(content, "content");
    
    // Check existence
    assert!(fs.exists("/test.txt").await);
}
```

### Mock Git Repository

The `MockGitRepository` simulates a Git repository with commits, branches, and tags.

```rust
use crate::common::MockGitRepository;

#[test]
fn test_with_mock_git() {
    let repo = MockGitRepository::new("/repo");
    
    // Add commits
    repo.add_commit("abc123", "feat: add feature", vec![]);
    repo.add_commit("def456", "fix: bug fix", vec![]);
    
    // Add branches and tags
    repo.add_branch("develop", "def456");
    repo.add_tag("v1.0.0", "def456");
    
    // Query commits
    let commits = repo.get_commits();
    assert_eq!(commits.len(), 2);
}
```

### Mock NPM Registry

The `MockRegistry` simulates an NPM registry for testing package upgrade operations.

```rust
use crate::common::MockRegistry;

#[test]
fn test_with_mock_registry() {
    let registry = MockRegistry::new();
    
    // Add packages with versions
    registry.add_package("react", vec!["18.0.0", "18.1.0", "18.2.0"]);
    
    // Get latest version
    let latest = registry.get_latest_version("react");
    assert_eq!(latest, Some("18.2.0".to_string()));
    
    // Deprecate packages
    registry.deprecate_package("old-pkg", "Use new-pkg instead");
}
```

### Test Fixtures

#### Package.json Builder

```rust
use crate::common::PackageJsonBuilder;

#[test]
fn test_package_json() {
    let json = PackageJsonBuilder::new("my-package")
        .version("1.0.0")
        .description("Test package")
        .add_dependency("lodash", "^4.17.21")
        .add_dev_dependency("jest", "^29.0.0")
        .build();
    
    assert!(json.contains("my-package"));
}
```

#### Monorepo Fixture Builder

```rust
use crate::common::MonorepoFixtureBuilder;

#[test]
fn test_monorepo() {
    let fixture = MonorepoFixtureBuilder::new("test-monorepo")
        .add_package("packages/pkg1", "pkg1", "1.0.0")
        .add_package("packages/pkg2", "pkg2", "2.0.0")
        .build();
    
    let files = fixture.generate_files();
    // Files map contains all package.json and source files
}
```

#### Writing Fixtures to Disk

```rust
use crate::common::{create_temp_dir, create_basic_monorepo_fixture};

#[test]
fn test_with_temp_dir() {
    let temp = create_temp_dir().unwrap();
    let fixture = create_basic_monorepo_fixture();
    
    fixture.write_to_dir(temp.path()).unwrap();
    
    // Now you have a real directory structure for testing
}
```

### Assertion Helpers

```rust
use crate::common::assertions::*;

#[test]
fn test_with_assertions() {
    // Version assertions
    assert_version_eq("1.2.3", "1.2.3");
    assert_version_gt("1.2.3", "1.2.2");
    assert_version_gte("1.2.3", "1.2.3");
    
    // String assertions
    assert_contains("hello world", "world");
    assert_not_contains("hello world", "goodbye");
    
    // JSON assertions
    let json = r#"{"name": "test", "version": "1.0.0"}"#;
    assert_json_field(json, "name", "test");
    
    // Collection assertions
    let vec = vec![1, 2, 3];
    assert_len(&vec, 3);
    assert_not_empty(&vec);
}
```

### Property-Based Testing

```rust
use crate::common::generators::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_version_parsing(version in semver_strategy()) {
        // Test that all generated versions can be parsed
        assert!(semver::Version::parse(&version).is_ok());
    }
    
    #[test]
    fn test_package_names(name in package_name_strategy()) {
        // Test package name validation
        assert!(!name.is_empty());
        assert!(name.len() >= 3);
    }
    
    #[test]
    fn test_conventional_commits(message in conventional_commit_strategy()) {
        // Test commit message parsing
        assert!(message.contains(':'));
    }
}
```

## Available Generators

- `semver_strategy()` - Semantic version strings (e.g., "1.2.3")
- `semver_with_prerelease_strategy()` - Versions with pre-release tags
- `semver_with_build_strategy()` - Versions with build metadata
- `package_name_strategy()` - Valid NPM package names
- `commit_type_strategy()` - Conventional commit types
- `conventional_commit_strategy()` - Complete conventional commit messages
- `file_path_strategy()` - Unix-style file paths
- `version_bump_strategy()` - Version bump types (major, minor, patch)
- `environment_strategy()` - Environment names
- `commit_hash_strategy()` - SHA-1 style commit hashes (40 chars)
- `short_commit_hash_strategy()` - Short commit hashes (7 chars)
- `author_name_strategy()` - Author names
- `author_email_strategy()` - Email addresses
- `branch_name_strategy()` - Git branch names
- `changeset_id_strategy()` - UUID-like changeset IDs
- `version_spec_strategy()` - Dependency version specs (^1.2.3, ~2.0.0, etc.)

## Static Fixtures

Pre-built fixtures are available in `tests/fixtures/`:

### Monorepo Fixture

Located in `tests/fixtures/monorepo/`:
- Root package with workspaces configuration
- Two packages: `@test/pkg-a` and `@test/pkg-b`
- Package B depends on Package A
- Both have dependencies and scripts

### Single Package Fixture

Located in `tests/fixtures/single-package/`:
- Standard Node.js package structure
- Dependencies, devDependencies, scripts
- Repository and engine information

Access fixtures using:

```rust
use crate::common::{fixtures_dir, fixture_path};

#[test]
fn test_with_static_fixture() {
    let monorepo = fixture_path("monorepo");
    assert!(monorepo.exists());
    
    let package_json = monorepo.join("package.json");
    assert!(package_json.exists());
}
```

## Best Practices

1. **Use Mocks for Isolation**: Prefer mock implementations over real filesystem/network operations
2. **Property-Based Tests for Edge Cases**: Use proptest generators to find edge cases
3. **Fixture Builders for Setup**: Use builders to create consistent test data
4. **Custom Assertions for Clarity**: Use assertion helpers for better error messages
5. **Temporary Directories**: Always use `create_temp_dir()` for filesystem tests

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test file
cargo test test_infrastructure

# Run with output
cargo test -- --nocapture

# Run property-based tests with more cases
cargo test -- --test-threads=1
```

## Adding New Test Utilities

When adding new test utilities:

1. Add mock implementations to `common/mocks/`
2. Add assertion helpers to `common/assertions.rs`
3. Add generators to `common/generators.rs`
4. Add fixture builders to `common/fixtures.rs`
5. Re-export from `common/mod.rs`
6. Add example usage to `test_infrastructure.rs`
7. Document in this README

## Coverage

The test infrastructure itself is tested in `test_infrastructure.rs` to ensure:
- All mocks work correctly
- Fixtures generate valid structures
- Assertions behave as expected
- Generators produce valid data

Run `cargo test test_infrastructure` to validate the infrastructure.