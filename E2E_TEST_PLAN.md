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

---

## 🚨 COMPREHENSIVE GAP ANALYSIS - MISSING TESTS

**Date**: 2025-11-07  
**Analysis**: Deep review identified **74 missing test scenarios**  
**Current Coverage**: 71% (147/207 features tested)  
**Target Coverage**: 100% (221 tests total)

See [E2E_COVERAGE_GAP_ANALYSIS.md](./E2E_COVERAGE_GAP_ANALYSIS.md) for detailed analysis.

---

### Phase 2.1A: Init Command - Missing Tests (7 tests)

**Priority**: 🟡 HIGH  
**Current**: 12 tests, **Missing**: 7 tests

```rust
// Environment configuration
test_init_with_default_environments()
  // Test: --environments dev,staging,prod --default-env prod
  // Verify: Only prod is marked as default

test_init_invalid_strategy_fails()
  // Test: --strategy invalidstrategy
  // Verify: Fails with clear error message

test_init_invalid_format_fails()
  // Test: --config-format xml
  // Verify: Fails with supported formats list

test_init_invalid_registry_url_fails()
  // Test: --registry not-a-url
  // Verify: Fails with URL validation error

// Git integration
test_init_creates_gitignore_entries()
  // Test: Run init, check .gitignore contains .changesets
  // Verify: .gitignore updated or created

test_init_with_git_not_initialized()
  // Test: Init in directory without git
  // Verify: Works correctly, no git operations

// Interactive mode
test_init_interactive_prompts()
  // Test: Run without --non-interactive
  // Verify: Prompts for strategy, environments, etc.
```

---

### Phase 2.2A: Config Command - Missing Tests (2 tests)

**Priority**: 🟢 MEDIUM  
**Current**: 18 tests, **Missing**: 2 tests

```rust
// Format support
test_config_show_toml_format()
test_config_show_yaml_format()
  // Test: Create config in TOML/YAML, run config show
  // Verify: Shows correct values

test_config_validate_toml_format()
test_config_validate_yaml_format()
  // Test: Create config in TOML/YAML, run config validate
  // Verify: Validates correctly
```

**⚠️ NOTE**: Plan mentions `config get` and `config set` but these commands **DO NOT EXIST** in CLI. Remove from plan or add to CLI as feature request.

---

### Phase 2.3A: Changeset Command - Missing Tests (13 tests)

**Priority**: 🔴 CRITICAL (edit/check), 🟡 HIGH (filters)  
**Current**: 25 tests, **Missing**: 13 tests

```rust
// === Create command ===
test_changeset_create_with_custom_message()
  // Test: --message "feat: add new feature"
  // Verify: Message stored in changeset

test_changeset_create_with_custom_branch()
  // Test: --branch custom-branch
  // Verify: Uses custom branch instead of git detection

test_changeset_create_auto_detect_packages_vs_manual()
  // Test: Compare auto-detect vs --packages flag
  // Verify: Both work, manual overrides auto-detect

test_changeset_create_interactive_prompts()
  // Test: Run without --non-interactive
  // Verify: Prompts for bump, environments, message

// === Update command ===
test_changeset_update_adds_environment()
  // Test: --env staging,prod
  // Verify: Adds environments to existing changeset

test_changeset_update_adds_packages()
  // Test: --packages package-a,package-b
  // Verify: Adds packages to existing changeset

test_changeset_update_multiple_operations()
  // Test: --commit abc123 --packages pkg-a --bump major
  // Verify: All updates applied

// === List command ===
test_changeset_list_filter_by_bump_type()
  // Test: --filter-bump major
  // Verify: Only shows major changesets

test_changeset_list_filter_by_environment()
  // Test: --filter-env prod
  // Verify: Only shows changesets with prod env

test_changeset_list_sort_by_bump()
test_changeset_list_sort_by_branch()
test_changeset_list_sort_by_date()
  // Test: --sort <field>
  // Verify: Sorted correctly

// === 🔴 CRITICAL: Edit command - COMPLETELY UNTESTED ===
test_changeset_edit_opens_editor()
  // Test: Mock $EDITOR, verify opens changeset file
  // Verify: Editor opened with correct file

test_changeset_edit_validates_changes_after_edit()
  // Test: Edit changeset, make invalid change
  // Verify: Validation catches errors

test_changeset_edit_current_branch()
test_changeset_edit_specific_branch()
  // Test: Edit with/without branch argument
  // Verify: Edits correct changeset

// === History command ===
test_changeset_history_filter_by_package()
  // Test: --package package-a
  // Verify: Only shows history for package-a

test_changeset_history_date_range()
  // Test: --since 2024-01-01 --until 2024-12-31
  // Verify: Shows changesets in date range

test_changeset_history_filter_by_environment()
  // Test: --env prod
  // Verify: Shows only prod changesets

test_changeset_history_filter_by_bump()
  // Test: --bump major
  // Verify: Shows only major bumps

test_changeset_history_with_limit()
  // Test: --limit 10
  // Verify: Shows max 10 results

// === 🔴 CRITICAL: Check command - COMPLETELY UNTESTED ===
test_changeset_check_exists_for_current_branch()
  // Test: Create changeset, run check
  // Verify: Exit code 0

test_changeset_check_exists_for_specific_branch()
  // Test: --branch feature-branch
  // Verify: Exit code 0 if exists

test_changeset_check_not_exists()
  // Test: Run check with no changeset
  // Verify: Exit code non-zero

test_changeset_check_exit_code_for_git_hooks()
  // Test: Simulate git hook usage
  // Verify: Correct exit codes for automation
```

---

### Phase 2.4A: Bump Command - Missing Tests (6 tests)

**Priority**: 🔴 CRITICAL (snapshot/prerelease)  
**Current**: 25 tests, **Missing**: 6 tests

```rust
// === 🔴 CRITICAL: Snapshot versions - COMPLETELY UNTESTED ===
test_bump_snapshot_generates_snapshot_version()
  // Test: --execute --snapshot
  // Verify: Version becomes 1.2.3-<branch>.<commit>

test_bump_snapshot_with_custom_format()
  // Test: --snapshot-format "{version}-{branch}.{short_commit}"
  // Verify: Custom format applied

test_bump_snapshot_format_variables()
  // Test: All variables: {version}, {branch}, {commit}, {short_commit}
  // Verify: All substituted correctly

// === 🔴 CRITICAL: Pre-release - COMPLETELY UNTESTED ===
test_bump_prerelease_alpha()
test_bump_prerelease_beta()
test_bump_prerelease_rc()
  // Test: --prerelease <tag>
  // Verify: Version becomes 1.2.3-alpha.1, etc.

// === Git operations ===
test_bump_git_tag_and_push_to_remote()
  // Test: --git-tag --git-push
  // Verify: Tags created and pushed

// === Advanced flags ===
test_bump_no_archive_keeps_changesets()
  // Test: --execute --no-archive
  // Verify: Changesets NOT archived

test_bump_force_skips_confirmations()
  // Test: --execute --force
  // Verify: No prompts, auto-confirms
```

---

### Phase 2.5A: Upgrade Command - Missing Tests (10 tests)

**Priority**: 🟡 HIGH  
**Current**: 16 tests, **Missing**: 10 tests

```rust
// === Check command ===
test_upgrade_check_no_major()
test_upgrade_check_no_minor()
test_upgrade_check_no_patch()
  // Test: --no-major, --no-minor, --no-patch
  // Verify: Excludes specified bump types

test_upgrade_check_with_peer_dependencies()
  // Test: --peer
  // Verify: Includes peer dependencies

test_upgrade_check_without_dev_dependencies()
  // Test: --dev=false or without --dev
  // Verify: Excludes dev dependencies

test_upgrade_check_specific_packages()
  // Test: --packages package-a,package-b
  // Verify: Only checks specified packages

test_upgrade_check_custom_registry()
  // Test: --registry https://custom-registry.com
  // Verify: Uses custom registry

// === Apply command ===
test_upgrade_apply_minor_and_patch_only()
  // Test: --minor-and-patch
  // Verify: Only applies non-breaking upgrades

test_upgrade_apply_no_backup()
  // Test: --no-backup
  // Verify: No backup created

test_upgrade_apply_force()
  // Test: --force
  // Verify: No confirmation prompts

test_upgrade_apply_custom_changeset_bump()
  // Test: --auto-changeset --changeset-bump major
  // Verify: Changeset created with major bump

// === Backups commands ===
test_upgrade_backups_clean_with_custom_keep()
  // Test: --keep 10
  // Verify: Keeps 10 most recent backups

test_upgrade_backups_clean_force()
  // Test: --force
  // Verify: No confirmation prompt

test_upgrade_backups_restore_force()
  // Test: --force
  // Verify: No confirmation prompt
```

---

### Phase 2.6A: Audit Command - Missing Tests (2 tests)

**Priority**: 🟢 MEDIUM  
**Current**: 25 tests, **Missing**: 2 tests

```rust
test_audit_breaking_changes_section()
  // Test: --sections breaking-changes
  // Verify: Shows only breaking changes analysis

test_audit_output_to_file_explicit()
  // Test: --output audit-report.txt
  // Verify: Report written to file (distinct from --export)
```

---

### Phase 2.7A: Changes Command - Missing Tests (2 tests)

**Priority**: 🔴 CRITICAL  
**Current**: 14 tests, **Missing**: 2 tests

```rust
test_changes_filter_by_packages()
  // Test: --packages package-a,package-b
  // Verify: Only shows changes for specified packages

test_changes_filter_by_packages_with_dependencies()
  // Test: Change pkg-a, pkg-b depends on pkg-a
  // Test: --packages package-b
  // Verify: Shows pkg-b even though change in pkg-a
```

---

### Phase 3A: Workflow Tests - CRITICAL MISSING (20 tests)

**Priority**: 🔴 CRITICAL  
**Current**: 0 tests, **Missing**: 20 tests  
**Status**: ⚪ **NOT IMPLEMENTED**

```rust
// === Release workflows ===
#[tokio::test]
async fn test_complete_release_workflow_single_package() {
    // 1. init workspace
    // 2. changeset create --bump minor
    // 3. bump --dry-run (preview)
    // 4. bump --execute
    // 5. Verify: version bumped, changelog updated, changeset archived
}

#[tokio::test]
async fn test_complete_release_workflow_monorepo() {
    // Same as above but with 3+ packages
    // Verify: All changesets processed correctly
}

#[tokio::test]
async fn test_hotfix_workflow() {
    // 1. Checkout from release tag
    // 2. Create patch changeset
    // 3. Bump and release
    // 4. Verify: Patch version applied
}

#[tokio::test]
async fn test_environment_specific_release() {
    // 1. Create changesets with environment filters
    // 2. Bump respects environment constraints
    // 3. Verify: Only correct packages bumped per environment
}

// === Upgrade workflows ===
#[tokio::test]
async fn test_upgrade_with_auto_changeset_complete_flow() {
    // 1. upgrade check
    // 2. upgrade apply --auto-changeset
    // 3. bump preview
    // 4. bump execute
    // 5. Verify: Complete upgrade → release cycle
}

#[tokio::test]
async fn test_rollback_failed_upgrade_complete_flow() {
    // 1. upgrade apply
    // 2. Simulate failure
    // 3. upgrade backups restore
    // 4. Verify: Restored to previous state
}

// === Cascading and dependencies ===
#[tokio::test]
async fn test_cascading_bumps_in_monorepo_complete() {
    // Package B depends on A
    // 1. Change A, create changeset
    // 2. Bump
    // 3. Verify: B also bumped due to dependency
}

#[tokio::test]
async fn test_dependency_impact_across_packages() {
    // A ← B ← C (dependency chain)
    // 1. Change A
    // 2. Verify: changes command shows B and C affected
}

// === CI/CD simulation ===
#[tokio::test]
async fn test_ci_cd_pull_request_workflow() {
    // Simulate PR workflow:
    // 1. Create feature branch
    // 2. changeset create
    // 3. changeset check (for Git hook)
    // 4. Verify: Changeset validation passes
}

#[tokio::test]
async fn test_ci_cd_merge_and_release_workflow() {
    // Simulate merge to main:
    // 1. Merge feature branch
    // 2. changeset update (add merge commit)
    // 3. bump execute --git-tag --git-commit
    // 4. Verify: Release artifacts created
}

#[tokio::test]
async fn test_ci_cd_with_changeset_validation() {
    // CI pipeline validation:
    // 1. Run changeset check
    // 2. If fails, block merge
    // 3. Verify: Exit codes correct for CI
}

// === Multi-changeset workflows ===
#[tokio::test]
async fn test_multiple_changesets_single_release() {
    // Real-world: Multiple features merged
    // 1. Create 3 changesets (different branches)
    // 2. Merge all to main
    // 3. Single bump execute
    // 4. Verify: All changes in one release
}

#[tokio::test]
async fn test_multiple_teams_multiple_changesets() {
    // Multiple teams working simultaneously:
    // 1. Team A: major change
    // 2. Team B: minor change
    // 3. Team C: patch change
    // 4. Merge all, bump once
    // 5. Verify: Highest bump wins (major)
}

// === Audit integration ===
#[tokio::test]
async fn test_audit_before_release_workflow() {
    // Best practice workflow:
    // 1. audit --sections all
    // 2. Fix critical issues
    // 3. bump execute
    // 4. Verify: Clean audit before release
}

#[tokio::test]
async fn test_audit_blocks_release_on_critical_issues() {
    // Automated gate:
    // 1. Create breaking change
    // 2. audit --min-severity critical
    // 3. Verify: Critical issues found
    // 4. Attempt bump should warn/fail
}

// === Error recovery ===
#[tokio::test]
async fn test_partial_failure_rollback_workflow() {
    // Test transaction-like behavior:
    // 1. Start bump execute
    // 2. Fail during changelog generation
    // 3. Verify: All changes rolled back
}

#[tokio::test]
async fn test_network_failure_recovery_workflow() {
    // Test resilience:
    // 1. upgrade check (mock network failure)
    // 2. Retry
    // 3. Verify: Recovers gracefully
}

#[tokio::test]
async fn test_conflicting_changes_resolution_workflow() {
    // Conflict handling:
    // 1. Two changesets modify same package
    // 2. Different bump types
    // 3. Verify: Conflict detected and resolved
}

// === Snapshot and pre-release ===
#[tokio::test]
async fn test_snapshot_deployment_workflow() {
    // Feature branch deployment:
    // 1. changeset create
    // 2. bump --snapshot
    // 3. Deploy to staging
    // 4. Verify: Snapshot version created
}

#[tokio::test]
async fn test_prerelease_to_stable_promotion_workflow() {
    // Alpha → Beta → RC → Stable:
    // 1. bump --prerelease alpha
    // 2. bump --prerelease beta
    // 3. bump --prerelease rc
    // 4. bump --execute (stable)
    // 5. Verify: Version progression correct
}

// === Backup and restore ===
#[tokio::test]
async fn test_upgrade_backup_restore_complete_workflow() {
    // Full backup lifecycle:
    // 1. upgrade apply (creates backup)
    // 2. Make more changes
    // 3. upgrade backups list
    // 4. upgrade backups restore
    // 5. Verify: Restored correctly
}
```

---

### Phase 4A: Cross-Cutting Concerns (14 tests)

**Priority**: ⚪ LOW  
**Status**: Future enhancement

```rust
// === Global flags ===
test_all_commands_with_custom_root()
test_all_commands_with_log_level_debug()
test_all_commands_with_no_color()
test_all_commands_with_custom_config_path()

// === Exit codes ===
test_exit_codes_consistency()
  // 0 = success, 1 = error, 2 = usage error

// === Error messages ===
test_error_messages_clarity()
  // All errors have actionable messages

// === Performance ===
test_large_monorepo_performance()
  // 100+ packages, should complete < 5s

test_many_changesets_performance()
  // 50+ changesets, should handle efficiently

test_deep_dependency_trees_performance()
  // 10+ levels deep, should resolve correctly

// === Concurrency ===
test_concurrent_changeset_creation()
  // Multiple users creating changesets

test_concurrent_bump_operations()
  // Race condition handling
```

---

## 📊 Updated Coverage Summary

### By Priority

| Priority | Tests Missing | Time Estimate |
|----------|---------------|---------------|
| 🔴 **CRITICAL** | 34 | 1 week |
| 🟡 **HIGH** | 24 | 3-4 days |
| 🟢 **MEDIUM** | 10 | 2-3 days |
| ⚪ **LOW** | 14 | 1-2 weeks |
| **TOTAL** | **82** | **3-4 weeks** |

### By Command

| Command | Implemented | Missing | Total | Coverage |
|---------|-------------|---------|-------|----------|
| Init | 12 | 7 | 19 | 63% |
| Config | 18 | 2 | 20 | 90% |
| Changeset | 25 | 13 | 38 | 66% |
| Bump | 25 | 6 | 31 | 81% |
| Upgrade | 16 | 10 | 26 | 62% |
| Audit | 25 | 2 | 27 | 93% |
| Changes | 14 | 2 | 16 | 88% |
| Version | 12 | 1 | 13 | 92% |
| **Workflows** | **0** | **20** | **20** | **0%** |
| **Cross-Cutting** | **0** | **14** | **14** | **0%** |
| **TOTAL** | **147** | **77** | **224** | **66%** |

---

## 🎯 Updated Timeline

| Phase | Duration | Priority | Status |
|-------|----------|----------|--------|
| Phase 1: Foundation | 1-2 days | ✅ DONE | 🟢 Complete |
| Phase 2: Commands (Original) | 3-5 days | ✅ DONE | 🟢 Complete |
| **Phase 2A: Command Gaps** | **3-4 days** | **🔴 CRITICAL** | **⚪ Pending** |
| **Phase 3A: Workflow Tests** | **1 week** | **🔴 CRITICAL** | **⚪ Pending** |
| Phase 4: Documentation | 1 day | 🟢 MEDIUM | ⚪ Pending |
| **Phase 4A: Cross-Cutting** | **1-2 weeks** | **⚪ LOW** | **⚪ Pending** |
| **TOTAL REMAINING** | **2.5-4 weeks** | | |

---

## 🚀 Recommended Execution Order

### Sprint 1: Critical Workflows (1 week)
1. **Phase 3A: Workflow Tests** (20 tests) - **HIGHEST PRIORITY**
   - Complete release workflows
   - CI/CD simulation
   - Multi-changeset scenarios
   
### Sprint 2: Critical Commands (1 week)
2. **Phase 2A: Critical Command Gaps** (34 tests)
   - `changeset edit` (4 tests)
   - `changeset check` (4 tests)
   - `bump --snapshot/--prerelease` (6 tests)
   - `changes --packages` (2 tests)
   - Other high-priority flags

### Sprint 3: Polish (1 week)
3. **Phase 2A: Medium Priority Gaps** (10 tests)
4. **Phase 4: Documentation** (1 day)

### Sprint 4: Future Enhancements (Optional)
5. **Phase 4A: Cross-Cutting Concerns** (14 tests)
   - Performance tests
   - Concurrency tests
   - Global flags consistency

---

## ✅ Action Items

- [x] Complete Phase 1 (Foundation)
- [x] Complete Phase 2 (Command Tests - Original Set)
- [x] Comprehensive gap analysis completed
- [ ] **NEXT**: Implement Phase 3A Workflow Tests (20 tests)
- [ ] **THEN**: Implement Phase 2A Critical Gaps (34 tests)
- [ ] Update documentation
- [ ] Add CI performance monitoring
- [ ] Establish coverage metrics tracking

---

**Last Updated**: 2025-11-07  
**Next Review**: After Phase 3A completion
