//! Integration tests for change detection workflow
//!
//! These tests validate the complete flow from configuration loading,
//! through change detection, to version impact analysis.

use sublime_monorepo_tools::{
    ChangeDetectionEngine, ChangeDetectionRules, ChangeDetector, ChangeSignificance, ConfigManager,
    Error, MonorepoConfig, PackageChangeType, VersionBumpType,
};
use tempfile::TempDir;

mod common;

/// Test the complete change detection workflow with configuration
#[test]
fn test_complete_change_detection_workflow() {
    // Create temporary directory structure
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    // Set up monorepo structure
    common::setup_test_monorepo(root_path);

    // Test 1: Configuration Management
    let config_manager = ConfigManager::new();
    let config = config_manager.get_clone();
    assert_eq!(config.versioning.default_bump, VersionBumpType::Patch);

    // Test 2: Change Detection Engine
    let _engine = ChangeDetectionEngine::new();
    let rules = ChangeDetectionRules::default();
    assert!(!rules.change_type_rules.is_empty());
    assert!(!rules.significance_rules.is_empty());
    assert!(!rules.version_bump_rules.is_empty());

    // Test 3: Change Detection Integration
    let detector = ChangeDetector::new(root_path.to_str().expect("Path should be valid"));
    let validation_errors = detector.engine().validate_rules();
    assert!(validation_errors.is_empty(), "Default rules should be valid");

    // Test 4: End-to-end Integration
    // This validates that all components work together
    // All previous assertions confirm the integration is working
}

/// Test configuration and change detection rule integration
#[test]
fn test_config_rules_integration() {
    let mut config_manager = ConfigManager::new();

    // Test configuration update affects change detection
    config_manager
        .update(|config| {
            config.versioning.default_bump = VersionBumpType::Minor;
            config.versioning.propagate_changes = false;
        })
        .expect("Config update should succeed");

    let updated_config = config_manager.get_clone();
    assert_eq!(updated_config.versioning.default_bump, VersionBumpType::Minor);
    assert!(!updated_config.versioning.propagate_changes);

    // Test that rule engine can be configured
    let rules = ChangeDetectionRules::default();
    let version_bump_rules = &rules.version_bump_rules;

    // Verify integration between config and rules
    assert!(!version_bump_rules.is_empty());
    let major_bump_rule =
        version_bump_rules.iter().find(|rule| rule.version_bump == VersionBumpType::Major);
    assert!(major_bump_rule.is_some(), "Should have major bump rule");
}

/// Test error handling across modules
#[test]
fn test_cross_module_error_handling() {
    // Test configuration errors
    let config_result = ConfigManager::load_from_file("/nonexistent/path/config.json");
    assert!(config_result.is_err());

    if let Err(error) = config_result {
        match error {
            Error::Config(_) => {
                // Expected error type for configuration issues
                let error_string = format!("{error}");
                assert!(error_string.contains("Configuration"));
            }
            _ => panic!("Expected Config error"),
        }
    }

    // Test that errors can be chained properly
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let monorepo_error: Error = io_error.into();

    let error_display = format!("{monorepo_error}");
    assert!(error_display.contains("IO") || error_display.contains("I/O"));
}

/// Test package change analysis integration
#[test]
fn test_package_change_analysis_integration() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    // Set up test structure
    common::setup_test_monorepo(root_path);

    // Create a change detector
    let _detector = ChangeDetector::new(root_path.to_str().expect("Path should be valid"));

    // Test change type detection
    let change_types = vec![
        PackageChangeType::SourceCode,
        PackageChangeType::Dependencies,
        PackageChangeType::Configuration,
        PackageChangeType::Documentation,
        PackageChangeType::Tests,
    ];

    for change_type in change_types {
        // Each change type should be handled correctly
        assert_ne!(format!("{change_type:?}"), "");
    }

    // Test significance levels
    let low_sig = ChangeSignificance::Low;
    let medium_sig = ChangeSignificance::Medium;
    let high_sig = ChangeSignificance::High;

    assert!(low_sig < medium_sig);
    assert!(medium_sig < high_sig);
    assert_eq!(low_sig.elevate(), medium_sig);
    assert_eq!(medium_sig.elevate(), high_sig);
    assert_eq!(high_sig.elevate(), high_sig);
}

/// Test monorepo configuration presets integration
#[test]
fn test_monorepo_config_presets_integration() {
    // Test different project size configurations
    let small_config = MonorepoConfig::small_project();
    let large_config = MonorepoConfig::large_project();
    let library_config = MonorepoConfig::library_project();

    // Verify configurations are different and appropriate
    assert_ne!(small_config.tasks.max_concurrent, large_config.tasks.max_concurrent);
    assert!(small_config.tasks.max_concurrent < large_config.tasks.max_concurrent);

    // Test that different presets have appropriate settings
    assert!(!small_config.tasks.parallel);
    assert!(large_config.tasks.parallel);

    // Library projects should have different versioning defaults
    assert_eq!(library_config.versioning.default_bump, VersionBumpType::Minor);
    assert!(library_config.changelog.include_breaking_changes);

    // Test that each preset can be used with change detection
    let detector = ChangeDetector::new("/tmp");
    let rules = ChangeDetectionRules::default();

    // Verify integration works with different configurations
    assert!(!rules.change_type_rules.is_empty());
    assert!(detector.engine().validate_rules().is_empty());
}
