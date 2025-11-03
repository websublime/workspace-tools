//! Tests for upgrade commands.
//!
//! This module contains comprehensive tests for the upgrade command implementations.
//!
//! # What
//!
//! Tests cover:
//! - Type structures and their methods
//! - Summary calculations
//! - JSON serialization/deserialization
//! - Integration tests for full command execution (Story 6.2)
//!
//! # How
//!
//! Tests use:
//! - Unit tests for type methods and helpers
//! - Mock data structures for testing conversions
//! - Integration tests will be added when we implement mock UpgradeManager
//!
//! # Why
//!
//! Comprehensive testing ensures:
//! - Correct behavior across all scenarios
//! - No regressions when refactoring
//! - Clear documentation through test examples
//! - Confidence in upgrade detection accuracy

#![allow(clippy::expect_used)]

use crate::cli::commands::UpgradeCheckArgs;
use crate::commands::upgrade::check::create_detection_options;
use crate::commands::upgrade::types::*;

// TODO: will be implemented on story 6.2
// Integration tests will be added when we implement mock UpgradeManager
// to test the full command execution flow including detection and output.

// ============================================================================
// Detection Options Tests
// ============================================================================

#[test]
fn test_create_detection_options_defaults() {
    let args = UpgradeCheckArgs {
        major: true,
        no_major: false,
        minor: true,
        no_minor: false,
        patch: true,
        no_patch: false,
        dev: true,
        peer: false,
        packages: None,
        registry: None,
    };

    let options = create_detection_options(&args).expect("Should create detection options");
    assert!(options.include_dependencies);
    assert!(options.include_dev_dependencies);
    assert!(!options.include_peer_dependencies);
}

#[test]
fn test_create_detection_options_no_major() {
    let args = UpgradeCheckArgs {
        major: true,
        no_major: true,
        minor: true,
        no_minor: false,
        patch: true,
        no_patch: false,
        dev: true,
        peer: false,
        packages: None,
        registry: None,
    };

    let options = create_detection_options(&args).expect("Should create detection options");
    assert!(options.include_dependencies);
    assert!(options.include_dev_dependencies);
}

#[test]
fn test_create_detection_options_all_disabled() {
    let args = UpgradeCheckArgs {
        major: true,
        no_major: true,
        minor: true,
        no_minor: true,
        patch: true,
        no_patch: true,
        dev: true,
        peer: false,
        packages: None,
        registry: None,
    };

    let result = create_detection_options(&args);
    assert!(result.is_err(), "Should fail when all upgrade types are disabled");
}

// ============================================================================
// Type Tests
// ============================================================================

#[test]
fn test_upgrade_summary_default() {
    let summary = UpgradeSummary::default();

    assert_eq!(summary.total_packages, 0);
    assert_eq!(summary.packages_with_upgrades, 0);
    assert_eq!(summary.total_upgrades, 0);
    assert_eq!(summary.major_upgrades, 0);
    assert_eq!(summary.minor_upgrades, 0);
    assert_eq!(summary.patch_upgrades, 0);
    assert!(!summary.has_upgrades());
    assert_eq!(summary.non_breaking_count(), 0);
}

#[test]
fn test_upgrade_summary_calculations() {
    let summary = UpgradeSummary {
        total_packages: 10,
        packages_with_upgrades: 5,
        total_upgrades: 15,
        major_upgrades: 3,
        minor_upgrades: 8,
        patch_upgrades: 4,
    };

    assert!(summary.has_upgrades());
    assert_eq!(summary.non_breaking_count(), 12, "Non-breaking should be minor + patch");
}

#[test]
fn test_package_upgrade_info_methods() {
    let empty_package = PackageUpgradeInfo {
        name: "@org/empty".to_string(),
        path: "packages/empty".to_string(),
        upgrades: vec![],
    };

    assert_eq!(empty_package.upgrade_count(), 0);
    assert!(!empty_package.has_upgrades());

    let package_with_upgrades = PackageUpgradeInfo {
        name: "@org/core".to_string(),
        path: "packages/core".to_string(),
        upgrades: vec![
            DependencyUpgradeInfo {
                package: "typescript".to_string(),
                current_version: "5.0.0".to_string(),
                latest_version: "5.3.3".to_string(),
                upgrade_type: "minor".to_string(),
                breaking: false,
            },
            DependencyUpgradeInfo {
                package: "eslint".to_string(),
                current_version: "8.0.0".to_string(),
                latest_version: "9.0.0".to_string(),
                upgrade_type: "major".to_string(),
                breaking: true,
            },
        ],
    };

    assert_eq!(package_with_upgrades.upgrade_count(), 2);
    assert!(package_with_upgrades.has_upgrades());
}

#[test]
fn test_dependency_upgrade_info_type_detection() {
    let major_upgrade = DependencyUpgradeInfo {
        package: "react".to_string(),
        current_version: "17.0.0".to_string(),
        latest_version: "18.2.0".to_string(),
        upgrade_type: "major".to_string(),
        breaking: true,
    };

    assert!(major_upgrade.is_major());
    assert!(!major_upgrade.is_minor());
    assert!(!major_upgrade.is_patch());
    assert!(major_upgrade.breaking);

    let minor_upgrade = DependencyUpgradeInfo {
        package: "lodash".to_string(),
        current_version: "4.17.0".to_string(),
        latest_version: "4.18.0".to_string(),
        upgrade_type: "minor".to_string(),
        breaking: false,
    };

    assert!(!minor_upgrade.is_major());
    assert!(minor_upgrade.is_minor());
    assert!(!minor_upgrade.is_patch());
    assert!(!minor_upgrade.breaking);

    let patch_upgrade = DependencyUpgradeInfo {
        package: "axios".to_string(),
        current_version: "1.6.0".to_string(),
        latest_version: "1.6.2".to_string(),
        upgrade_type: "patch".to_string(),
        breaking: false,
    };

    assert!(!patch_upgrade.is_major());
    assert!(!patch_upgrade.is_minor());
    assert!(patch_upgrade.is_patch());
    assert!(!patch_upgrade.breaking);
}

#[test]
fn test_upgrade_check_response_serialization() {
    let response = UpgradeCheckResponse {
        success: true,
        packages: vec![PackageUpgradeInfo {
            name: "@org/test".to_string(),
            path: "packages/test".to_string(),
            upgrades: vec![DependencyUpgradeInfo {
                package: "typescript".to_string(),
                current_version: "5.0.0".to_string(),
                latest_version: "5.3.3".to_string(),
                upgrade_type: "minor".to_string(),
                breaking: false,
            }],
        }],
        summary: UpgradeSummary {
            total_packages: 1,
            packages_with_upgrades: 1,
            total_upgrades: 1,
            major_upgrades: 0,
            minor_upgrades: 1,
            patch_upgrades: 0,
        },
    };

    // Test JSON serialization
    let json = serde_json::to_string_pretty(&response).expect("Should serialize to JSON");

    assert!(json.contains("\"success\": true"));
    assert!(json.contains("\"@org/test\""));
    assert!(json.contains("\"typescript\""));
    assert!(json.contains("\"currentVersion\""));
    assert!(json.contains("\"latestVersion\""));
    assert!(json.contains("\"totalPackages\""));
    assert!(json.contains("\"totalUpgrades\""));

    // Test deserialization
    let deserialized: UpgradeCheckResponse =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    assert!(deserialized.success);
    assert_eq!(deserialized.packages.len(), 1);
    assert_eq!(deserialized.summary.total_upgrades, 1);
}

// ============================================================================
// Apply Type Tests
// ============================================================================

#[test]
fn test_apply_summary_default() {
    let summary = ApplySummary::default();

    assert_eq!(summary.total_applied, 0);
    assert_eq!(summary.total_skipped, 0);
    assert!(summary.backup_id.is_none());
    assert_eq!(summary.total_processed(), 0);
}

#[test]
fn test_apply_summary_calculations() {
    let summary = ApplySummary {
        total_applied: 10,
        total_skipped: 3,
        backup_id: Some("backup_20240115_103045".to_string()),
    };

    assert_eq!(summary.total_applied, 10);
    assert_eq!(summary.total_skipped, 3);
    assert_eq!(summary.total_processed(), 13);
    assert!(summary.backup_id.is_some());
}

#[test]
fn test_applied_upgrade_info() {
    let applied = AppliedUpgradeInfo {
        package: "typescript".to_string(),
        from: "5.0.0".to_string(),
        to: "5.3.3".to_string(),
        upgrade_type: "minor".to_string(),
    };

    assert_eq!(applied.package, "typescript");
    assert_eq!(applied.from, "5.0.0");
    assert_eq!(applied.to, "5.3.3");
    assert_eq!(applied.upgrade_type, "minor");
}

#[test]
fn test_skipped_upgrade_info() {
    let skipped = SkippedUpgradeInfo {
        package: "eslint".to_string(),
        reason: "major_version".to_string(),
        current_version: "8.0.0".to_string(),
        latest_version: "9.0.0".to_string(),
    };

    assert_eq!(skipped.package, "eslint");
    assert_eq!(skipped.reason, "major_version");
    assert_eq!(skipped.current_version, "8.0.0");
    assert_eq!(skipped.latest_version, "9.0.0");
}

#[test]
fn test_upgrade_apply_response_serialization() {
    let response = UpgradeApplyResponse {
        success: true,
        applied: vec![AppliedUpgradeInfo {
            package: "typescript".to_string(),
            from: "5.0.0".to_string(),
            to: "5.3.3".to_string(),
            upgrade_type: "minor".to_string(),
        }],
        skipped: vec![SkippedUpgradeInfo {
            package: "eslint".to_string(),
            reason: "major_version".to_string(),
            current_version: "8.0.0".to_string(),
            latest_version: "9.0.0".to_string(),
        }],
        summary: ApplySummary {
            total_applied: 1,
            total_skipped: 1,
            backup_id: Some("backup_20240115_103045".to_string()),
        },
    };

    // Test JSON serialization
    let json = serde_json::to_string_pretty(&response).expect("Should serialize to JSON");

    assert!(json.contains("\"success\": true"));
    assert!(json.contains("\"typescript\""));
    assert!(json.contains("\"from\": \"5.0.0\""));
    assert!(json.contains("\"to\": \"5.3.3\""));
    assert!(json.contains("\"type\": \"minor\""));
    assert!(json.contains("\"eslint\""));
    assert!(json.contains("\"reason\": \"major_version\""));
    assert!(json.contains("\"currentVersion\": \"8.0.0\""));
    assert!(json.contains("\"latestVersion\": \"9.0.0\""));
    assert!(json.contains("\"totalApplied\": 1"));
    assert!(json.contains("\"totalSkipped\": 1"));
    assert!(json.contains("\"backupId\": \"backup_20240115_103045\""));

    // Test deserialization
    let deserialized: UpgradeApplyResponse =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    assert!(deserialized.success);
    assert_eq!(deserialized.applied.len(), 1);
    assert_eq!(deserialized.skipped.len(), 1);
    assert_eq!(deserialized.summary.total_applied, 1);
    assert_eq!(deserialized.summary.total_skipped, 1);
    assert_eq!(deserialized.summary.backup_id, Some("backup_20240115_103045".to_string()));
}

#[test]
fn test_upgrade_apply_response_empty() {
    let response = UpgradeApplyResponse {
        success: true,
        applied: vec![],
        skipped: vec![],
        summary: ApplySummary::default(),
    };

    let json = serde_json::to_string_pretty(&response).expect("Should serialize to JSON");

    assert!(json.contains("\"success\": true"));
    assert!(json.contains("\"applied\": []"));
    assert!(json.contains("\"skipped\": []"));
    assert!(json.contains("\"totalApplied\": 0"));
    assert!(json.contains("\"totalSkipped\": 0"));

    // backupId should be omitted when None due to skip_serializing_if
    assert!(!json.contains("backupId"));
}

#[test]
fn test_upgrade_apply_response_no_backup() {
    let response = UpgradeApplyResponse {
        success: true,
        applied: vec![AppliedUpgradeInfo {
            package: "lodash".to_string(),
            from: "4.17.0".to_string(),
            to: "4.17.21".to_string(),
            upgrade_type: "patch".to_string(),
        }],
        skipped: vec![],
        summary: ApplySummary { total_applied: 1, total_skipped: 0, backup_id: None },
    };

    let json = serde_json::to_string_pretty(&response).expect("Should serialize to JSON");

    assert!(json.contains("\"success\": true"));
    assert!(json.contains("\"totalApplied\": 1"));

    // backupId should be omitted when None
    assert!(!json.contains("backupId"));
}

// TODO: Mock-based integration tests will be added when we implement
// a test harness for UpgradeManager to test the full command execution
// flow including detection, application, backup, and changeset creation.
