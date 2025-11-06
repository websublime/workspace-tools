# E2E Test Plan - Workspace CLI

**Story**: WOR-TSK-130  
**Goal**: Comprehensive End-to-End test coverage for all CLI commands and workflows  
**Strategy**: Test Pyramid approach with reusable fixtures

---

## 📊 Test Coverage Strategy

```
┌─────────────────────────────────────────┐
│   TEST PYRAMID                          │
├─────────────────────────────────────────┤
│                                         │
│        E2E Workflows (15-20)            │  ← Complete user journeys
│       /                  \              │    ~10% of test suite
│      /   E2E Commands    \              │    ~5-10s execution
│     /    (80-120)        \              │
│    /                       \            │  ← Individual commands
│   /   Integration Tests    \            │    ~15% of test suite
│  /       (150-200)          \           │    ~2-3s execution
│ /                            \          │
│/    Unit Tests (500-700)      \         │  ← Fast, detailed tests
└─────────────────────────────────────────┘    ~75% of test suite
                                               ~0.1s execution
```

### Coverage Targets

- **Unit Tests**: 500-700 tests (already achieved ✅)
- **Integration Tests**: 150-200 tests (expand existing)
- **E2E Commands**: 80-120 tests (new)
- **E2E Workflows**: 15-20 tests (new)
- **Total Execution Time**: < 10s for full suite

---

## 🏗️ Implementation Phases

### Phase 1: Foundation (Priority: HIGH)

**Goal**: Create reusable test infrastructure

#### 1.1 Common Fixtures (`crates/cli/tests/common/fixtures.rs`)

```rust
/// Workspace fixture builder with fluent API
pub struct WorkspaceFixture {
    temp_dir: TempDir,
    root: PathBuf,
    git_initialized: bool,
    packages: Vec<PackageInfo>,
}

impl WorkspaceFixture {
    // === Factory Methods ===
    pub fn single_package() -> Self;
    pub fn monorepo_independent() -> Self;
    pub fn monorepo_unified() -> Self;
    
    // === Configuration ===
    pub fn with_config(self, config: ConfigBuilder) -> Self;
    pub fn with_custom_config(self, json: &str) -> Self;
    
    // === Git ===
    pub fn with_git(self) -> Self;
    pub fn with_commits(self, count: usize) -> Self;
    pub fn with_branch(self, name: &str) -> Self;
    
    // === Changesets ===
    pub fn add_changeset(self, changeset: ChangesetBuilder) -> Self;
    pub fn add_changesets(self, changesets: Vec<ChangesetBuilder>) -> Self;
    
    // === Dependencies ===
    pub fn with_internal_deps(self) -> Self;
    pub fn with_external_deps(self, deps: Vec<(&str, &str)>) -> Self;
    
    // === NPM ===
    pub fn with_npmrc(self, content: &str) -> Self;
    pub fn with_package_lock(self) -> Self;
    
    // === Assertions ===
    pub fn assert_config_exists(&self);
    pub fn assert_package_version(&self, expected: &str);
    pub fn assert_changeset_count(&self, expected: usize);
    pub fn assert_changelog_exists(&self);
}
```

#### 1.2 Assertion Helpers (`crates/cli/tests/common/assertions.rs`)

```rust
/// Custom assertions for CLI E2E tests
pub trait CliAssertions {
    fn assert_success(&self);
    fn assert_error_contains(&self, msg: &str);
    fn assert_json_output(&self) -> serde_json::Value;
}

/// File system assertions
pub fn assert_file_exists(path: &Path);
pub fn assert_file_contains(path: &Path, content: &str);
pub fn assert_json_file_valid<T: DeserializeOwned>(path: &Path) -> T;
pub fn assert_git_tag_exists(repo: &Path, tag: &str);
```

#### 1.3 Test Utilities (`crates/cli/tests/common/helpers.rs`)

```rust
/// Execute CLI command in test context
pub async fn execute_cli(
    workspace: &WorkspaceFixture,
    args: &[&str],
) -> TestResult;

/// Capture output for assertions
pub struct TestResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Mock dependencies
pub fn mock_npm_registry() -> MockRegistry;
pub fn mock_git_remote() -> MockRemote;
```

**Deliverables**:
- ✅ `common/fixtures.rs` with WorkspaceFixture
- ✅ `common/assertions.rs` with custom assertions
- ✅ `common/helpers.rs` with test utilities
- ✅ `common/mod.rs` with public exports

**Time Estimate**: 1-2 days

---

### Phase 2: Command E2E Tests (Priority: HIGH)

Each command gets comprehensive E2E coverage with real filesystem operations.

#### 2.1 Init Command (`e2e_init.rs`)

**Tests** (10-12 tests):
```rust
// Basic functionality
test_init_single_package_creates_config()
test_init_monorepo_creates_config()
test_init_with_json_format()
test_init_with_yaml_format()

// Error cases
test_init_fails_without_package_json()
test_init_with_existing_config_requires_force()

// Force flag
test_init_force_overwrites_existing()

// Git integration
test_init_creates_gitignore_entries()
test_init_with_git_initialized()

// Interactive mode
test_init_interactive_prompts()
test_init_non_interactive_uses_defaults()
```

#### 2.2 Changeset Commands (`e2e_changeset.rs`)

**Tests** (20-25 tests):
```rust
// Add command
test_changeset_add_creates_file()
test_changeset_add_with_minor_bump()
test_changeset_add_with_major_bump()
test_changeset_add_multiple_packages()
test_changeset_add_with_environments()
test_changeset_add_detects_git_branch()
test_changeset_add_fails_duplicate_branch()

// Update command
test_changeset_update_modifies_existing()
test_changeset_update_adds_commit()
test_changeset_update_changes_bump_type()
test_changeset_update_fails_not_found()

// List command
test_changeset_list_shows_all()
test_changeset_list_filters_by_package()
test_changeset_list_json_output()
test_changeset_list_empty_workspace()

// Show command
test_changeset_show_displays_details()
test_changeset_show_by_branch()
test_changeset_show_by_id()
test_changeset_show_not_found()

// Remove command
test_changeset_remove_deletes_file()
test_changeset_remove_with_confirmation()
test_changeset_remove_force_flag()

// Edit command
test_changeset_edit_opens_editor()
test_changeset_edit_validates_changes()
```

#### 2.3 Bump Commands (`e2e_bump.rs`)

**Tests** (15-20 tests):
```rust
// Preview
test_bump_preview_shows_changes()
test_bump_preview_independent_strategy()
test_bump_preview_unified_strategy()
test_bump_preview_no_changesets()
test_bump_preview_json_output()

// Execute
test_bump_execute_applies_versions()
test_bump_execute_updates_package_json()
test_bump_execute_creates_changelog()
test_bump_execute_archives_changesets()
test_bump_execute_with_git_tag()
test_bump_execute_with_git_commit()
test_bump_execute_cascading_bumps()
test_bump_execute_dry_run()
test_bump_execute_no_changelog()
test_bump_execute_unified_version()

// Error cases
test_bump_execute_fails_no_changesets()
test_bump_execute_fails_dirty_git()
test_bump_execute_rollback_on_error()
```

#### 2.4 Upgrade Commands (`e2e_upgrade.rs`)

**Tests** (15-18 tests):
```rust
// Check
test_upgrade_check_detects_outdated()
test_upgrade_check_respects_npmrc()
test_upgrade_check_filters_by_type()
test_upgrade_check_json_output()

// Apply
test_upgrade_apply_updates_package_json()
test_upgrade_apply_creates_backup()
test_upgrade_apply_updates_lock_file()
test_upgrade_apply_auto_changeset()
test_upgrade_apply_dry_run()
test_upgrade_apply_patch_only()

// Rollback
test_upgrade_rollback_restores_backup()
test_upgrade_rollback_lists_available()
test_upgrade_rollback_validates_id()

// Backups
test_upgrade_backups_list()
test_upgrade_backups_clean()
test_upgrade_backups_restore()
```

#### 2.5 Audit Commands (`e2e_audit.rs`)

**Tests** (8-10 tests):
```rust
// Breaking changes
test_audit_breaking_detects_issues()
test_audit_breaking_json_output()
test_audit_breaking_no_issues()

// Upgrades
test_audit_upgrades_shows_available()
test_audit_upgrades_filters()

// Report
test_audit_report_generates_markdown()
test_audit_report_generates_html()
test_audit_report_all_sections()
test_audit_report_custom_sections()
```

#### 2.6 Config Commands (`e2e_config.rs`)

**Tests** (5-7 tests):
```rust
test_config_show_displays_current()
test_config_show_json_output()
test_config_validate_valid_config()
test_config_validate_invalid_config()
test_config_validate_missing_file()
test_config_get_specific_key()
test_config_set_specific_key()
```

#### 2.7 Version Command (`e2e_version.rs`)

**Tests** (3-5 tests):
```rust
test_version_displays_info()
test_version_verbose_shows_details()
test_version_json_output()
test_version_shows_build_info()
```

**Total Command Tests**: ~80-120 tests  
**Time Estimate**: 3-5 days

---

### Phase 3: Workflow E2E Tests (Priority: MEDIUM)

Complete user journeys testing multiple commands together.

#### 3.1 Complete Release Workflow (`e2e_workflows.rs`)

```rust
#[tokio::test]
async fn test_complete_release_workflow_single_package() {
    // 1. Init workspace
    // 2. Create changeset
    // 3. Preview bump
    // 4. Execute bump
    // 5. Verify release artifacts
}

#[tokio::test]
async fn test_complete_release_workflow_monorepo() {
    // Same as above but with multiple packages
}

#[tokio::test]
async fn test_hotfix_workflow() {
    // 1. Create from release tag
    // 2. Add patch changeset
    // 3. Bump and release
}

#[tokio::test]
async fn test_upgrade_with_auto_changeset() {
    // 1. Check upgrades
    // 2. Apply with auto-changeset
    // 3. Preview bump
    // 4. Execute release
}

#[tokio::test]
async fn test_cascading_bumps_in_monorepo() {
    // Package B depends on Package A
    // Bump A should trigger bump in B
}

#[tokio::test]
async fn test_rollback_failed_upgrade() {
    // 1. Apply upgrade
    // 2. Simulate failure
    // 3. Rollback
    // 4. Verify restoration
}

#[tokio::test]
async fn test_ci_cd_simulation() {
    // Simulate complete CI/CD pipeline:
    // - PR: add changeset
    // - Merge: update changeset
    // - Main: bump and release
}

#[tokio::test]
async fn test_multiple_changesets_single_release() {
    // Multiple feature branches
    // All merged to main
    // Single release with all changes
}

#[tokio::test]
async fn test_audit_before_release() {
    // 1. Run audit
    // 2. Fix breaking changes
    // 3. Proceed with release
}

#[tokio::test]
async fn test_environment_specific_release() {
    // Changesets with environment filters
    // Bump respects environment constraints
}
```

**Total Workflow Tests**: ~15-20 tests  
**Time Estimate**: 2-3 days

---

### Phase 4: Documentation & Validation (Priority: LOW)

#### 4.1 Test Documentation

Create `crates/cli/tests/README.md`:
```markdown
# CLI E2E Tests

## Running Tests

# All tests
cargo test

# Only E2E tests
cargo test --test '*'

# Specific test file
cargo test --test e2e_init

# With output
cargo test --test e2e_init -- --nocapture

## Writing Tests

See E2E_TEST_PLAN.md for conventions and examples.
```

#### 4.2 CI Integration

Update `.github/workflows/test.yml`:
```yaml
- name: Run E2E Tests
  run: cargo test --tests
  timeout-minutes: 5
```

#### 4.3 Coverage Report

Generate coverage report:
```bash
cargo tarpaulin --out Html --output-dir coverage
```

**Time Estimate**: 1 day

---

## 📝 Test Conventions

### Naming

```rust
// Pattern: test_<command>_<action>_<scenario>
test_init_creates_config()
test_changeset_add_with_minor_bump()
test_bump_execute_applies_versions()
```

### Structure

```rust
#[tokio::test]
async fn test_example() {
    // ARRANGE: Setup test workspace
    let workspace = WorkspaceFixture::single_package()
        .with_git()
        .add_changeset(ChangesetBuilder::minor());
    
    // ACT: Execute command
    let result = execute_command(&workspace).await;
    
    // ASSERT: Verify outcome
    assert!(result.is_ok());
    workspace.assert_package_version("1.1.0");
}
```

### Cleanup

All tests must clean up resources:
```rust
// TempDir automatically cleans up on drop
let temp_dir = TempDir::new()?;
// No manual cleanup needed
```

---

## 🎯 Success Criteria

### Code Quality
- ✅ All tests pass
- ✅ No clippy warnings
- ✅ rustfmt compliant
- ✅ Well documented

### Coverage
- ✅ All commands have E2E tests
- ✅ Critical workflows tested
- ✅ Error paths covered
- ✅ Edge cases handled

### Performance
- ✅ Full suite < 10s
- ✅ Individual test < 1s
- ✅ Workflow tests < 2s
- ✅ No flaky tests

### Maintainability
- ✅ Reusable fixtures
- ✅ Clear test names
- ✅ Good documentation
- ✅ Easy to extend

---

## 📅 Timeline

| Phase | Duration | Status |
|-------|----------|--------|
| Phase 1: Foundation | 1-2 days | 🟡 In Progress |
| Phase 2: Commands | 3-5 days | ⚪ Pending |
| Phase 3: Workflows | 2-3 days | ⚪ Pending |
| Phase 4: Documentation | 1 day | ⚪ Pending |
| **Total** | **7-11 days** | |

---

## 🔄 Maintenance

### Adding New Tests

1. Use existing fixtures when possible
2. Follow naming conventions
3. Document complex scenarios
4. Keep tests focused and fast
5. Update this plan

### Handling Flaky Tests

1. Identify root cause
2. Add proper synchronization
3. Increase timeouts if needed
4. Use `#[flaky]` attribute temporarily
5. Fix or remove if unfixable

### Performance Monitoring

Track test execution time:
```bash
cargo test -- --report-time
```

Target: < 10s for full suite

---

## 📚 References

- [Test Pyramid](https://martinfowler.com/articles/practical-test-pyramid.html)
- [Testing Best Practices](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Integration Testing in Rust](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
