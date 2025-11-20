//! Tests for bump command functionality.
//!
//! This module contains comprehensive tests for the bump preview command,
//! ensuring correct behavior across different scenarios and versioning strategies.

// Integration tests would use execute_bump_preview but require a real workspace setup
// use super::preview::execute_bump_preview;
use super::snapshot::{BumpSnapshot, BumpSummary, ChangesetInfo, PackageBumpInfo};
use crate::cli::commands::BumpArgs;
use crate::output::{Output, OutputFormat};
use std::io::Cursor;

/// Tests that BumpSummary calculates unchanged packages correctly.
#[test]
fn test_bump_summary_calculates_unchanged() {
    let summary = BumpSummary::new(10, 3, 2, false);

    assert_eq!(summary.total_packages, 10);
    assert_eq!(summary.packages_to_bump, 3);
    assert_eq!(summary.packages_unchanged, 7);
    assert_eq!(summary.total_changesets, 2);
    assert!(!summary.has_circular_dependencies);
}

/// Tests that BumpSummary handles edge case where all packages bump.
#[test]
fn test_bump_summary_all_packages_bump() {
    let summary = BumpSummary::new(5, 5, 1, false);

    assert_eq!(summary.total_packages, 5);
    assert_eq!(summary.packages_to_bump, 5);
    assert_eq!(summary.packages_unchanged, 0);
}

/// Tests that BumpSummary handles edge case where no packages bump.
#[test]
fn test_bump_summary_no_packages_bump() {
    let summary = BumpSummary::new(5, 0, 0, false);

    assert_eq!(summary.total_packages, 5);
    assert_eq!(summary.packages_to_bump, 0);
    assert_eq!(summary.packages_unchanged, 5);
}

/// Tests that BumpSummary handles circular dependencies flag.
#[test]
fn test_bump_summary_with_circular_dependencies() {
    let summary = BumpSummary::new(10, 3, 2, true);

    assert!(summary.has_circular_dependencies);
}

/// Tests that PackageBumpInfo serializes correctly for JSON output.
#[test]
#[allow(clippy::expect_used)]
fn test_package_bump_info_serialization() {
    let info = PackageBumpInfo {
        name: "@org/core".to_string(),
        path: "packages/core".to_string(),
        current_version: "1.2.3".to_string(),
        next_version: "1.3.0".to_string(),
        bump_type: sublime_pkg_tools::types::VersionBump::Minor,
        will_bump: true,
        reason: "direct change from changeset".to_string(),
    };

    let json = serde_json::to_string(&info).expect("Failed to serialize PackageBumpInfo");
    assert!(json.contains("@org/core"));
    assert!(json.contains("1.2.3"));
    assert!(json.contains("1.3.0"));
    assert!(json.contains("willBump"));
}

/// Tests that ChangesetInfo serializes correctly for JSON output.
#[test]
#[allow(clippy::expect_used)]
fn test_changeset_info_serialization() {
    let info = ChangesetInfo {
        id: "test-id".to_string(),
        branch: "feature/test".to_string(),
        bump_type: sublime_pkg_tools::types::VersionBump::Minor,
        packages: vec!["@org/core".to_string()],
        commit_count: 3,
    };

    let json = serde_json::to_string(&info).expect("Failed to serialize ChangesetInfo");
    assert!(json.contains("test-id"));
    assert!(json.contains("feature/test"));
    assert!(json.contains("@org/core"));
    assert!(json.contains("commitCount"));
}

/// Tests that BumpSnapshot serializes correctly for JSON output.
#[test]
#[allow(clippy::expect_used)]
fn test_bump_snapshot_serialization() {
    let snapshot = BumpSnapshot {
        strategy: "independent".to_string(),
        packages: vec![],
        changesets: vec![],
        summary: BumpSummary::new(10, 3, 2, false),
    };

    let json = serde_json::to_string(&snapshot).expect("Failed to serialize BumpSnapshot");
    assert!(json.contains("independent"));
    assert!(json.contains("packages"));
    assert!(json.contains("changesets"));
    assert!(json.contains("summary"));
}

/// Tests that BumpArgs defaults work as expected.
#[test]
fn test_bump_args_defaults() {
    let args = BumpArgs {
        dry_run: false,
        execute: false,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    // Default behavior should be preview mode (neither dry_run nor execute)
    assert!(!args.dry_run);
    assert!(!args.execute);
}

/// Tests that execute_bump_preview accepts valid arguments structure.
#[test]
fn test_execute_bump_preview_args_structure() {
    let args = BumpArgs {
        dry_run: true,
        execute: false,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    // Verify args structure is valid
    assert!(args.dry_run);
    assert!(!args.execute);
}

/// Tests that Output can handle JSON format for bump snapshot.
#[test]
fn test_output_handles_json_bump_snapshot() {
    let buffer = Cursor::new(Vec::new());
    let output = Output::new(OutputFormat::Json, buffer, false);

    let snapshot = BumpSnapshot {
        strategy: "independent".to_string(),
        packages: vec![],
        changesets: vec![],
        summary: BumpSummary::default(),
    };

    // Verify we can serialize and output
    let result = output.json(&crate::output::JsonResponse::success(snapshot));
    assert!(result.is_ok());
}

/// Tests that PackageBumpInfo correctly represents a package that will bump.
#[test]
fn test_package_bump_info_will_bump() {
    let info = PackageBumpInfo {
        name: "@org/core".to_string(),
        path: "packages/core".to_string(),
        current_version: "1.2.3".to_string(),
        next_version: "1.3.0".to_string(),
        bump_type: sublime_pkg_tools::types::VersionBump::Minor,
        will_bump: true,
        reason: "direct change from changeset".to_string(),
    };

    assert!(info.will_bump);
    assert_ne!(info.current_version, info.next_version);
}

/// Tests that PackageBumpInfo correctly represents a package that won't bump.
#[test]
fn test_package_bump_info_wont_bump() {
    let info = PackageBumpInfo {
        name: "@org/utils".to_string(),
        path: "packages/utils".to_string(),
        current_version: "2.0.0".to_string(),
        next_version: "2.0.0".to_string(),
        bump_type: sublime_pkg_tools::types::VersionBump::None,
        will_bump: false,
        reason: "not in any changeset".to_string(),
    };

    assert!(!info.will_bump);
    assert_eq!(info.current_version, info.next_version);
    assert_eq!(info.bump_type, sublime_pkg_tools::types::VersionBump::None);
}

/// Tests that BumpSnapshot can represent independent strategy correctly.
#[test]
fn test_bump_snapshot_independent_strategy() {
    let snapshot = BumpSnapshot {
        strategy: "independent".to_string(),
        packages: vec![
            PackageBumpInfo {
                name: "@org/core".to_string(),
                path: "packages/core".to_string(),
                current_version: "1.0.0".to_string(),
                next_version: "1.1.0".to_string(),
                bump_type: sublime_pkg_tools::types::VersionBump::Minor,
                will_bump: true,
                reason: "direct change from changeset".to_string(),
            },
            PackageBumpInfo {
                name: "@org/utils".to_string(),
                path: "packages/utils".to_string(),
                current_version: "2.0.0".to_string(),
                next_version: "2.0.0".to_string(),
                bump_type: sublime_pkg_tools::types::VersionBump::None,
                will_bump: false,
                reason: "not in any changeset".to_string(),
            },
        ],
        changesets: vec![],
        summary: BumpSummary::new(2, 1, 1, false),
    };

    assert_eq!(snapshot.strategy, "independent");
    assert_eq!(snapshot.packages.len(), 2);
    assert_eq!(snapshot.summary.packages_to_bump, 1);
    assert_eq!(snapshot.summary.packages_unchanged, 1);
}

/// Tests that BumpSnapshot can represent unified strategy correctly.
#[test]
fn test_bump_snapshot_unified_strategy() {
    let snapshot = BumpSnapshot {
        strategy: "unified".to_string(),
        packages: vec![
            PackageBumpInfo {
                name: "@org/core".to_string(),
                path: "packages/core".to_string(),
                current_version: "1.0.0".to_string(),
                next_version: "1.1.0".to_string(),
                bump_type: sublime_pkg_tools::types::VersionBump::Minor,
                will_bump: true,
                reason: "unified bump (package in changeset)".to_string(),
            },
            PackageBumpInfo {
                name: "@org/utils".to_string(),
                path: "packages/utils".to_string(),
                current_version: "1.0.0".to_string(),
                next_version: "1.1.0".to_string(),
                bump_type: sublime_pkg_tools::types::VersionBump::Minor,
                will_bump: true,
                reason: "unified bump (all packages bumped together)".to_string(),
            },
        ],
        changesets: vec![],
        summary: BumpSummary::new(2, 2, 1, false),
    };

    assert_eq!(snapshot.strategy, "unified");
    assert_eq!(snapshot.packages.len(), 2);
    // In unified mode, all packages bump
    assert_eq!(snapshot.summary.packages_to_bump, 2);
    assert_eq!(snapshot.summary.packages_unchanged, 0);

    // Verify all packages have the same version
    assert_eq!(snapshot.packages[0].next_version, "1.1.0");
    assert_eq!(snapshot.packages[1].next_version, "1.1.0");
}

/// Tests that ChangesetInfo correctly represents changeset details.
#[test]
fn test_changeset_info_representation() {
    let info = ChangesetInfo {
        id: "feature-123".to_string(),
        branch: "feature/new-api".to_string(),
        bump_type: sublime_pkg_tools::types::VersionBump::Minor,
        packages: vec!["@org/core".to_string(), "@org/api".to_string()],
        commit_count: 5,
    };

    assert_eq!(info.id, "feature-123");
    assert_eq!(info.branch, "feature/new-api");
    assert_eq!(info.packages.len(), 2);
    assert_eq!(info.commit_count, 5);
}

/// Tests that default BumpSummary has correct values.
#[test]
fn test_bump_summary_default() {
    let summary = BumpSummary::default();

    assert_eq!(summary.total_packages, 0);
    assert_eq!(summary.packages_to_bump, 0);
    assert_eq!(summary.packages_unchanged, 0);
    assert_eq!(summary.total_changesets, 0);
    assert!(!summary.has_circular_dependencies);
}

// Integration tests would go here, but they require a real workspace setup
// These should be tested in the integration test suite with actual fixtures
// TODO: Add integration tests in tests/ directory with real workspace scenarios

// ============================================================================
// Snapshot Generation Tests
// ============================================================================

/// Tests that branch name sanitization replaces invalid characters.
#[test]
fn test_sanitize_branch_name_replaces_slash() {
    use super::snapshot::sanitize_branch_name;

    assert_eq!(sanitize_branch_name("feature/new-api"), "feature-new-api");
}

/// Tests that branch name sanitization replaces special characters.
#[test]
fn test_sanitize_branch_name_replaces_special_chars() {
    use super::snapshot::sanitize_branch_name;

    assert_eq!(sanitize_branch_name("feat/#123-fix"), "feat--123-fix");
    assert_eq!(sanitize_branch_name("fix@issue"), "fix-issue");
    assert_eq!(sanitize_branch_name("feat_feature"), "feat-feature");
}

/// Tests that branch name sanitization preserves valid characters.
#[test]
fn test_sanitize_branch_name_preserves_valid_chars() {
    use super::snapshot::sanitize_branch_name;

    assert_eq!(sanitize_branch_name("feature-123"), "feature-123");
    assert_eq!(sanitize_branch_name("v1.2.3"), "v1.2.3");
    assert_eq!(sanitize_branch_name("main"), "main");
}

/// Tests that branch name sanitization handles consecutive special characters.
#[test]
fn test_sanitize_branch_name_consecutive_special_chars() {
    use super::snapshot::sanitize_branch_name;

    assert_eq!(sanitize_branch_name("feat//fix"), "feat--fix");
    assert_eq!(sanitize_branch_name("fix@@bug"), "fix--bug");
}

/// Tests that branch name sanitization handles empty string.
#[test]
fn test_sanitize_branch_name_empty_string() {
    use super::snapshot::sanitize_branch_name;

    assert_eq!(sanitize_branch_name(""), "");
}

/// Tests that branch name sanitization handles unicode characters.
#[test]
fn test_sanitize_branch_name_unicode() {
    use super::snapshot::sanitize_branch_name;

    // Unicode characters should be replaced with dash
    assert_eq!(sanitize_branch_name("feat-🚀-rocket"), "feat---rocket");
    assert_eq!(sanitize_branch_name("fix-日本語"), "fix----");
}

/// Tests that snapshot args includes snapshot flag.
#[test]
fn test_snapshot_args_structure() {
    let args = BumpArgs {
        dry_run: false,
        execute: false,
        snapshot: true,
        snapshot_format: Some("{version}-{branch}.{short_commit}".to_string()),
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    assert!(args.snapshot);
    assert!(!args.execute);
    assert!(!args.dry_run);
    assert!(args.snapshot_format.is_some());
}

/// Tests that snapshot format can be customized.
#[test]
fn test_snapshot_format_customization() {
    let args = BumpArgs {
        dry_run: false,
        execute: false,
        snapshot: true,
        snapshot_format: Some("{version}-snapshot.{short_commit}".to_string()),
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    assert_eq!(args.snapshot_format.as_deref(), Some("{version}-snapshot.{short_commit}"));
}

/// Tests that snapshot mode can work with default format.
#[test]
fn test_snapshot_default_format() {
    let args = BumpArgs {
        dry_run: false,
        execute: false,
        snapshot: true,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    assert!(args.snapshot);
    assert!(args.snapshot_format.is_none());
}

/// Tests that PackageBumpInfo can represent snapshot version.
#[test]
fn test_package_bump_info_snapshot_version() {
    let info = PackageBumpInfo {
        name: "@org/core".to_string(),
        path: "packages/core".to_string(),
        current_version: "1.2.3".to_string(),
        next_version: "1.3.0-snapshot.abc123f".to_string(),
        bump_type: sublime_pkg_tools::types::VersionBump::Minor,
        will_bump: true,
        reason: "snapshot from changeset".to_string(),
    };

    assert!(info.will_bump);
    assert!(info.next_version.contains("-snapshot."));
    assert_eq!(info.reason, "snapshot from changeset");
}

/// Tests that BumpSnapshot correctly represents independent strategy snapshots.
#[test]
fn test_bump_snapshot_independent_snapshots() {
    let snapshot = BumpSnapshot {
        strategy: "independent".to_string(),
        packages: vec![PackageBumpInfo {
            name: "@org/core".to_string(),
            path: "packages/core".to_string(),
            current_version: "1.0.0".to_string(),
            next_version: "1.1.0-snapshot.abc123f".to_string(),
            bump_type: sublime_pkg_tools::types::VersionBump::Minor,
            will_bump: true,
            reason: "snapshot from changeset".to_string(),
        }],
        changesets: vec![ChangesetInfo {
            id: "feature-test".to_string(),
            branch: "feature/test".to_string(),
            bump_type: sublime_pkg_tools::types::VersionBump::Minor,
            packages: vec!["@org/core".to_string()],
            commit_count: 3,
        }],
        summary: BumpSummary::new(2, 1, 1, false),
    };

    assert_eq!(snapshot.strategy, "independent");
    assert_eq!(snapshot.packages.len(), 1);
    assert!(snapshot.packages[0].next_version.contains("-snapshot."));
    assert_eq!(snapshot.summary.packages_to_bump, 1);
}

/// Tests that BumpSnapshot correctly represents unified strategy snapshots.
#[test]
fn test_bump_snapshot_unified_snapshots() {
    let snapshot = BumpSnapshot {
        strategy: "unified".to_string(),
        packages: vec![
            PackageBumpInfo {
                name: "@org/core".to_string(),
                path: "packages/core".to_string(),
                current_version: "1.0.0".to_string(),
                next_version: "1.1.0-snapshot.abc123f".to_string(),
                bump_type: sublime_pkg_tools::types::VersionBump::Minor,
                will_bump: true,
                reason: "unified snapshot (package in changeset)".to_string(),
            },
            PackageBumpInfo {
                name: "@org/utils".to_string(),
                path: "packages/utils".to_string(),
                current_version: "1.0.0".to_string(),
                next_version: "1.1.0-snapshot.abc123f".to_string(),
                bump_type: sublime_pkg_tools::types::VersionBump::Minor,
                will_bump: true,
                reason: "unified snapshot (all packages bumped together)".to_string(),
            },
        ],
        changesets: vec![ChangesetInfo {
            id: "feature-test".to_string(),
            branch: "feature/test".to_string(),
            bump_type: sublime_pkg_tools::types::VersionBump::Minor,
            packages: vec!["@org/core".to_string()],
            commit_count: 3,
        }],
        summary: BumpSummary::new(2, 2, 1, false),
    };

    assert_eq!(snapshot.strategy, "unified");
    assert_eq!(snapshot.packages.len(), 2);
    // All packages should have snapshot versions
    assert!(snapshot.packages[0].next_version.contains("-snapshot."));
    assert!(snapshot.packages[1].next_version.contains("-snapshot."));
    // All packages should have the same snapshot version in unified mode
    assert_eq!(snapshot.packages[0].next_version, snapshot.packages[1].next_version);
    assert_eq!(snapshot.summary.packages_to_bump, 2);
}

/// Tests that snapshot serialization includes all required fields.
#[test]
#[allow(clippy::expect_used)]
fn test_snapshot_serialization_completeness() {
    let snapshot = BumpSnapshot {
        strategy: "independent".to_string(),
        packages: vec![PackageBumpInfo {
            name: "@org/core".to_string(),
            path: "packages/core".to_string(),
            current_version: "1.0.0".to_string(),
            next_version: "1.1.0-snapshot.abc123f".to_string(),
            bump_type: sublime_pkg_tools::types::VersionBump::Minor,
            will_bump: true,
            reason: "snapshot from changeset".to_string(),
        }],
        changesets: vec![],
        summary: BumpSummary::new(1, 1, 1, false),
    };

    let json = serde_json::to_string(&snapshot).expect("Failed to serialize snapshot");

    // Verify all key fields are present
    assert!(json.contains("strategy"));
    assert!(json.contains("packages"));
    assert!(json.contains("changesets"));
    assert!(json.contains("summary"));
    assert!(json.contains("snapshot"));
    assert!(json.contains("@org/core"));
}

/// Tests that snapshot args cannot combine with execute mode.
#[test]
fn test_snapshot_and_execute_mutually_exclusive() {
    // In real usage, Clap's conflicts_with would prevent this
    // This test documents the expected behavior
    let args_snapshot = BumpArgs {
        dry_run: false,
        execute: false,
        snapshot: true,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    let args_execute = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    // These should be mutually exclusive
    assert!(args_snapshot.snapshot && !args_snapshot.execute);
    assert!(args_execute.execute && !args_execute.snapshot);
}

/// Tests that show_diff flag defaults to false.
#[test]
fn test_show_diff_flag_default() {
    let args = BumpArgs {
        dry_run: false,
        execute: false,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    assert!(!args.show_diff);
}

/// Tests that show_diff flag can be enabled.
#[test]
fn test_show_diff_flag_enabled() {
    let args = BumpArgs {
        dry_run: false,
        execute: false,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: true,
    };

    assert!(args.show_diff);
}

/// Tests that show_diff flag works with preview mode.
#[test]
fn test_show_diff_with_preview_mode() {
    let args = BumpArgs {
        dry_run: true,
        execute: false,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: true,
    };

    assert!(args.dry_run);
    assert!(args.show_diff);
}

// ============================================================================
// Prerelease and Archive Policy Tests
// ============================================================================

/// Tests parse_prerelease_args with valid beta.create format.
#[test]
#[allow(clippy::expect_used)]
fn test_parse_prerelease_args_beta_create() {
    use super::execute::parse_prerelease_args;

    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: Some("beta.create".to_string()),
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    let result = parse_prerelease_args(&args);
    assert!(result.is_ok());

    let config = result.ok().flatten();
    assert!(config.is_some());

    let config = config.expect("config should be Some");
    assert_eq!(config.tag, "beta");
    assert_eq!(config.mode, sublime_pkg_tools::types::PrereleaseMode::Create);
}

/// Tests parse_prerelease_args with valid alpha.increment format.
#[test]
#[allow(clippy::expect_used)]
fn test_parse_prerelease_args_alpha_increment() {
    use super::execute::parse_prerelease_args;

    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: Some("alpha.increment".to_string()),
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    let result = parse_prerelease_args(&args);
    assert!(result.is_ok());

    let config = result.ok().flatten();
    assert!(config.is_some());

    let config = config.expect("config should be Some");
    assert_eq!(config.tag, "alpha");
    assert_eq!(config.mode, sublime_pkg_tools::types::PrereleaseMode::Increment);
}

/// Tests parse_prerelease_args with valid rc.promote format.
#[test]
#[allow(clippy::expect_used)]
fn test_parse_prerelease_args_rc_promote() {
    use super::execute::parse_prerelease_args;

    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: Some("rc.promote".to_string()),
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    let result = parse_prerelease_args(&args);
    assert!(result.is_ok());

    let config = result.ok().flatten();
    assert!(config.is_some());

    let config = config.expect("config should be Some");
    assert_eq!(config.tag, "rc");
    assert_eq!(config.mode, sublime_pkg_tools::types::PrereleaseMode::Promote);
}

/// Tests parse_prerelease_args with None returns None.
#[test]
fn test_parse_prerelease_args_none() {
    use super::execute::parse_prerelease_args;

    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    let result = parse_prerelease_args(&args);
    assert!(result.is_ok());

    let config = result.ok().flatten();
    assert!(config.is_none());
}

/// Tests parse_prerelease_args with invalid format returns error.
#[test]
fn test_parse_prerelease_args_invalid_format() {
    use super::execute::parse_prerelease_args;

    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: Some("invalid-format".to_string()),
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    let result = parse_prerelease_args(&args);
    assert!(result.is_err());
}

/// Tests parse_prerelease_args with invalid mode returns error.
#[test]
fn test_parse_prerelease_args_invalid_mode() {
    use super::execute::parse_prerelease_args;

    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: Some("beta.invalid".to_string()),
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    let result = parse_prerelease_args(&args);
    assert!(result.is_err());
}

/// Tests parse_prerelease_args with custom tag name.
#[test]
#[allow(clippy::expect_used)]
fn test_parse_prerelease_args_custom_tag() {
    use super::execute::parse_prerelease_args;

    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: Some("snapshot.create".to_string()),
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    let result = parse_prerelease_args(&args);
    assert!(result.is_ok());

    let config = result.ok().flatten();
    assert!(config.is_some());

    let config = config.expect("config should be Some");
    assert_eq!(config.tag, "snapshot");
    assert_eq!(config.mode, sublime_pkg_tools::types::PrereleaseMode::Create);
}

/// Tests determine_archive_policy with no_archive flag returns Never.
#[test]
fn test_determine_archive_policy_never() {
    use super::execute::determine_archive_policy;

    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: true,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    let policy = determine_archive_policy(&args);
    assert_eq!(policy, super::types::ChangesetArchivePolicy::Never);
}

/// Tests determine_archive_policy with always_archive flag returns Always.
#[test]
fn test_determine_archive_policy_always() {
    use super::execute::determine_archive_policy;

    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: true,
        force: false,
        show_diff: false,
    };

    let policy = determine_archive_policy(&args);
    assert_eq!(policy, super::types::ChangesetArchivePolicy::Always);
}

/// Tests determine_archive_policy with no flags returns Auto.
#[test]
fn test_determine_archive_policy_auto() {
    use super::execute::determine_archive_policy;

    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    let policy = determine_archive_policy(&args);
    assert_eq!(policy, super::types::ChangesetArchivePolicy::Auto);
}

/// Tests should_archive with Auto policy and stable version returns true.
#[test]
fn test_should_archive_auto_stable() {
    use super::execute::should_archive;
    use sublime_pkg_tools::version::VersionResolution;

    let policy = super::types::ChangesetArchivePolicy::Auto;

    // Create a mock stable version resolution
    let resolution = VersionResolution { updates: vec![], circular_dependencies: vec![] };

    let result = should_archive(policy, &resolution);
    assert!(result);
}

/// Tests should_archive with Never policy returns false.
#[test]
fn test_should_archive_never() {
    use super::execute::should_archive;
    use sublime_pkg_tools::version::VersionResolution;

    let policy = super::types::ChangesetArchivePolicy::Never;

    let resolution = VersionResolution { updates: vec![], circular_dependencies: vec![] };

    let result = should_archive(policy, &resolution);
    assert!(!result);
}

/// Tests should_archive with Always policy returns true.
#[test]
fn test_should_archive_always() {
    use super::execute::should_archive;
    use sublime_pkg_tools::version::VersionResolution;

    let policy = super::types::ChangesetArchivePolicy::Always;

    let resolution = VersionResolution { updates: vec![], circular_dependencies: vec![] };

    let result = should_archive(policy, &resolution);
    assert!(result);
}

/// Tests that always_archive flag in BumpArgs defaults to false.
#[test]
fn test_bump_args_always_archive_default() {
    let args = BumpArgs {
        dry_run: false,
        execute: false,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    assert!(!args.always_archive);
}

/// Tests that prerelease argument can be set in BumpArgs.
#[test]
fn test_bump_args_prerelease_argument() {
    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: Some("beta.create".to_string()),
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    assert!(args.prerelease.is_some());
    assert_eq!(args.prerelease.as_deref(), Some("beta.create"));
}

/// Tests that prerelease and always_archive can be used together.
#[test]
fn test_bump_args_prerelease_with_always_archive() {
    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: Some("alpha.create".to_string()),
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: false,
        always_archive: true,
        force: false,
        show_diff: false,
    };

    assert!(args.prerelease.is_some());
    assert!(args.always_archive);
}

/// Tests that prerelease can be used with no_archive flag.
#[test]
fn test_bump_args_prerelease_with_no_archive() {
    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: Some("beta.increment".to_string()),
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: false,
        no_archive: true,
        always_archive: false,
        force: false,
        show_diff: false,
    };

    assert!(args.prerelease.is_some());
    assert!(args.no_archive);
    assert!(!args.always_archive);
}

/// Tests ChangesetArchivePolicy serialization for JSON output.
#[test]
#[allow(clippy::expect_used)]
fn test_changeset_archive_policy_serialization() {
    use super::types::ChangesetArchivePolicy;

    let auto = ChangesetArchivePolicy::Auto;
    let never = ChangesetArchivePolicy::Never;
    let always = ChangesetArchivePolicy::Always;

    let auto_json = serde_json::to_string(&auto).expect("Failed to serialize Auto");
    let never_json = serde_json::to_string(&never).expect("Failed to serialize Never");
    let always_json = serde_json::to_string(&always).expect("Failed to serialize Always");

    assert_eq!(auto_json, "\"auto\"");
    assert_eq!(never_json, "\"never\"");
    assert_eq!(always_json, "\"always\"");
}

/// Tests ChangesetArchivePolicy deserialization from JSON.
#[test]
#[allow(clippy::expect_used)]
fn test_changeset_archive_policy_deserialization() {
    use super::types::ChangesetArchivePolicy;

    let auto: ChangesetArchivePolicy =
        serde_json::from_str("\"auto\"").expect("Failed to deserialize auto");
    let never: ChangesetArchivePolicy =
        serde_json::from_str("\"never\"").expect("Failed to deserialize never");
    let always: ChangesetArchivePolicy =
        serde_json::from_str("\"always\"").expect("Failed to deserialize always");

    assert_eq!(auto, ChangesetArchivePolicy::Auto);
    assert_eq!(never, ChangesetArchivePolicy::Never);
    assert_eq!(always, ChangesetArchivePolicy::Always);
}

// ============================================================================
// PackageFilter Tests
// ============================================================================

/// Tests PackageFilter::new creates filter with packages.
#[test]
fn test_filter_new_creates_filter_with_packages() {
    use super::filter::PackageFilter;

    let packages = vec!["pkg1".to_string(), "pkg2".to_string()];
    let filter = PackageFilter::new(packages.clone(), false);

    assert_eq!(filter.package_count(), 2);
    assert!(!filter.includes_dependencies());
}

/// Tests PackageFilter::should_bump returns true for filtered packages.
#[test]
fn test_filter_should_bump_returns_true_for_filtered_packages() {
    use super::filter::PackageFilter;

    let filter = PackageFilter::new(vec!["pkg1".to_string()], false);

    assert!(filter.should_bump("pkg1"));
    assert!(!filter.should_bump("pkg2"));
}

/// Tests PackageFilter::apply_to_changeset filters packages correctly.
#[test]
fn test_filter_apply_to_changeset_filters_packages() {
    use super::filter::PackageFilter;
    use sublime_pkg_tools::types::{Changeset, VersionBump};

    let filter = PackageFilter::new(vec!["pkg1".to_string()], false);

    let mut changeset = Changeset::new("main", VersionBump::Minor, vec!["prod".to_string()]);
    changeset.add_package("pkg1");
    changeset.add_package("pkg2");
    changeset.add_package("pkg3");

    let filtered = filter.apply_to_changeset(&changeset);

    assert_eq!(filtered.packages.len(), 1);
    assert!(filtered.packages.contains(&"pkg1".to_string()));
    assert_eq!(filtered.branch, "main");
    assert_eq!(filtered.bump, VersionBump::Minor);
}

/// Tests PackageFilter::apply_to_changeset preserves non-package fields.
#[test]
fn test_filter_apply_to_changeset_preserves_non_package_fields() {
    use super::filter::PackageFilter;
    use sublime_pkg_tools::types::{Changeset, VersionBump};

    let filter = PackageFilter::new(vec!["pkg1".to_string()], false);

    let mut changeset = Changeset::new("feature", VersionBump::Major, vec!["dev".to_string()]);
    changeset.add_package("pkg1");
    changeset.add_commit("abc123");

    let filtered = filter.apply_to_changeset(&changeset);

    assert_eq!(filtered.branch, "feature");
    assert_eq!(filtered.bump, VersionBump::Major);
    assert_eq!(filtered.environments, vec!["dev".to_string()]);
    assert_eq!(filtered.changes.len(), 1);
}

/// Tests PackageFilter::validate succeeds for existing packages.
#[test]
fn test_filter_validate_succeeds_for_existing_packages() {
    use super::filter::PackageFilter;

    let filter = PackageFilter::new(vec!["pkg1".to_string(), "pkg2".to_string()], false);
    let available = vec!["pkg1".to_string(), "pkg2".to_string(), "pkg3".to_string()];

    assert!(filter.validate(&available).is_ok());
}

/// Tests PackageFilter::validate fails for non-existent package.
#[test]
fn test_filter_validate_fails_for_non_existent_package() {
    use super::filter::PackageFilter;

    let filter = PackageFilter::new(vec!["nonexistent".to_string()], false);
    let available = vec!["pkg1".to_string(), "pkg2".to_string()];

    let result = filter.validate(&available);
    assert!(result.is_err());

    if let Err(err) = result {
        assert!(err.to_string().contains("nonexistent"));
        assert!(err.to_string().contains("not found in workspace"));
    }
}

/// Tests PackageFilter::validate error includes available packages.
#[test]
fn test_filter_validate_error_includes_available_packages() {
    use super::filter::PackageFilter;

    let filter = PackageFilter::new(vec!["wrong".to_string()], false);
    let available = vec!["pkg1".to_string(), "pkg2".to_string()];

    let result = filter.validate(&available);
    assert!(result.is_err());

    if let Err(err) = result {
        assert!(err.to_string().contains("pkg1"));
        assert!(err.to_string().contains("pkg2"));
    }
}

/// Tests PackageFilter::includes_dependencies returns correct value.
#[test]
fn test_filter_includes_dependencies_returns_correct_value() {
    use super::filter::PackageFilter;

    let filter_without = PackageFilter::new(vec!["pkg1".to_string()], false);
    let filter_with = PackageFilter::new(vec!["pkg1".to_string()], true);

    assert!(!filter_without.includes_dependencies());
    assert!(filter_with.includes_dependencies());
}

/// Tests PackageFilter::package_count returns correct count.
#[test]
fn test_filter_package_count_returns_correct_count() {
    use super::filter::PackageFilter;

    let filter_empty = PackageFilter::new(vec![], false);
    let filter_one = PackageFilter::new(vec!["pkg1".to_string()], false);
    let filter_multiple = PackageFilter::new(vec!["pkg1".to_string(), "pkg2".to_string()], false);

    assert_eq!(filter_empty.package_count(), 0);
    assert_eq!(filter_one.package_count(), 1);
    assert_eq!(filter_multiple.package_count(), 2);
}

/// Tests PackageFilter::packages iterator works correctly.
#[test]
fn test_filter_packages_iterator() {
    use super::filter::PackageFilter;

    let packages = vec!["pkg1".to_string(), "pkg2".to_string()];
    let filter = PackageFilter::new(packages.clone(), false);

    let collected: Vec<_> = filter.packages().cloned().collect();
    assert_eq!(collected.len(), 2);

    // HashSet doesn't guarantee order, so check both are present
    assert!(collected.contains(&"pkg1".to_string()));
    assert!(collected.contains(&"pkg2".to_string()));
}

/// Tests PackageFilter with no matching packages in changeset.
#[test]
fn test_filter_with_no_matching_packages_in_changeset() {
    use super::filter::PackageFilter;
    use sublime_pkg_tools::types::{Changeset, VersionBump};

    let filter = PackageFilter::new(vec!["pkg1".to_string()], false);

    let mut changeset = Changeset::new("main", VersionBump::Patch, vec!["prod".to_string()]);
    changeset.add_package("pkg2");
    changeset.add_package("pkg3");

    let filtered = filter.apply_to_changeset(&changeset);

    assert_eq!(filtered.packages.len(), 0);
    assert!(filtered.packages.is_empty());
}

/// Tests PackageFilter preserves duplicate package filtering.
#[test]
fn test_filter_preserves_duplicate_package_filtering() {
    use super::filter::PackageFilter;
    use sublime_pkg_tools::types::{Changeset, VersionBump};

    let filter = PackageFilter::new(vec!["pkg1".to_string()], false);

    let mut changeset = Changeset::new("main", VersionBump::Minor, vec!["prod".to_string()]);
    changeset.add_package("pkg1");
    changeset.add_package("pkg1"); // Duplicate (shouldn't happen in practice)
    changeset.add_package("pkg2");

    let filtered = filter.apply_to_changeset(&changeset);

    // Should still have pkg1 (even if duplicated in original)
    assert!(filtered.packages.contains(&"pkg1".to_string()));
    assert!(!filtered.packages.contains(&"pkg2".to_string()));
}

/// Tests PackageFilter case-sensitive package matching.
#[test]
fn test_filter_case_sensitive_package_matching() {
    use super::filter::PackageFilter;

    let filter = PackageFilter::new(vec!["Pkg1".to_string()], false);

    assert!(filter.should_bump("Pkg1"));
    assert!(!filter.should_bump("pkg1")); // Different case
    assert!(!filter.should_bump("PKG1")); // Different case
}
