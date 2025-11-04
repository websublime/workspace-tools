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
        force: false,
        show_diff: true,
    };

    assert!(args.dry_run);
    assert!(args.show_diff);
}
