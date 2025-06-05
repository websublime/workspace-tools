//! Basic library integration tests
//!
//! Tests basic library functionality and public API accessibility.

use sublime_monorepo_tools::{
    VersionBumpType, PackageChangeType, ChangeSignificance,
    MonorepoConfig, ConfigManager, ChangeDetector, Error,
};

/// Test public API accessibility and basic functionality
#[test]
fn test_public_api_accessibility() {
    // Test that all main types are accessible and functional
    let bump = VersionBumpType::Major;
    let change = PackageChangeType::SourceCode;
    let significance = ChangeSignificance::High;
    
    // Validate enum functionality works
    assert_eq!(bump, VersionBumpType::Major);
    assert_eq!(change, PackageChangeType::SourceCode);
    assert_eq!(significance, ChangeSignificance::High);
    
    // Test enum ordering
    assert!(ChangeSignificance::Low < ChangeSignificance::Medium);
    assert!(ChangeSignificance::Medium < ChangeSignificance::High);
    
    // Test significance elevation
    assert_eq!(ChangeSignificance::Low.elevate(), ChangeSignificance::Medium);
    assert_eq!(ChangeSignificance::High.elevate(), ChangeSignificance::High);
}

/// Test configuration management basic functionality
#[test]
fn test_configuration_management() {
    let config_manager = ConfigManager::new();
    let config = config_manager.get().expect("Should get default config");
    
    // Test default configuration values
    assert_eq!(config.versioning.default_bump, VersionBumpType::Patch);
    assert!(config.versioning.propagate_changes);
    assert!(config.hooks.enabled);
    assert!(!config.environments.is_empty());
    
    // Test configuration presets
    let small_config = MonorepoConfig::small_project();
    let large_config = MonorepoConfig::large_project();
    
    assert!(small_config.tasks.max_concurrent < large_config.tasks.max_concurrent);
    assert!(!small_config.tasks.parallel);
    assert!(large_config.tasks.parallel);
}

/// Test change detection basic functionality  
#[test]
fn test_change_detection_basics() {
    let detector = ChangeDetector::new("/tmp");
    
    // Test that detector can be created
    let validation_errors = detector.engine().validate_rules();
    assert!(validation_errors.is_empty(), "Default rules should be valid");
    
    // Test change type variants
    let change_types = vec![
        PackageChangeType::SourceCode,
        PackageChangeType::Dependencies,
        PackageChangeType::Configuration,
        PackageChangeType::Documentation,
        PackageChangeType::Tests,
    ];
    
    for change_type in change_types {
        let display = format!("{change_type:?}");
        assert!(!display.is_empty(), "Change type should have display representation");
    }
}

/// Test error handling across the library
#[test]
fn test_error_handling() {
    // Test error construction
    let config_error = Error::config("Test configuration error");
    let analysis_error = Error::analysis("Test analysis error");
    
    // Test error display
    let config_msg = format!("{config_error}");
    let analysis_msg = format!("{analysis_error}");
    
    assert!(config_msg.contains("Configuration"));
    assert!(analysis_msg.contains("Analysis"));
    
    // Test error conversion
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "Test IO error");
    let converted_error: Error = io_error.into();
    let converted_msg = format!("{converted_error}");
    assert!(converted_msg.contains("IO") || converted_msg.contains("I/O"));
}

/// Test library version bump type functionality
#[test]
fn test_version_bump_types() {
    // Test all version bump variants
    let patch = VersionBumpType::Patch;
    let minor = VersionBumpType::Minor;
    let major = VersionBumpType::Major;
    
    // Test that each variant is distinct
    assert_ne!(patch, minor);
    assert_ne!(minor, major);
    assert_ne!(patch, major);
    
    // Test that they can be used in configuration
    let mut config = MonorepoConfig::default();
    config.versioning.default_bump = major;
    assert_eq!(config.versioning.default_bump, VersionBumpType::Major);
}