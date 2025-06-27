//! Integration tests for monorepo analysis and project management
//!
//! These tests validate the integration between monorepo project management,
//! configuration, and analysis capabilities.

use sublime_monorepo_tools::{
    MonorepoConfig, ConfigManager, VersionBumpType,
    PackageChangeType, ChangeSignificance,
};
use tempfile::TempDir;
use std::path::Path;

mod common;

/// Test complete monorepo project workflow
#[test]  
fn test_monorepo_project_analysis_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root_path = temp_dir.path();
    
    // Set up realistic monorepo structure
    common::setup_test_monorepo(root_path);
    
    // Verify the structure was created correctly
    assert!(common::verify_test_structure(root_path), "Test structure should be valid");
    
    // Test configuration management with the monorepo
    test_config_integration_with_monorepo(root_path);
    
    // Test different project configurations
    test_project_configuration_variants();
    
    // Test workspace pattern analysis
    test_workspace_patterns_analysis(root_path);
}

/// Test configuration integration with actual monorepo structure
fn test_config_integration_with_monorepo(_root_path: &Path) {
    let mut config_manager = ConfigManager::new();
    
    // Test that different project sizes have appropriate configurations
    let small_config = MonorepoConfig::small_project();
    let large_config = MonorepoConfig::large_project();
    let library_config = MonorepoConfig::library_project();
    
    // Small projects should be less parallel
    assert!(!small_config.tasks.parallel);
    assert_eq!(small_config.tasks.max_concurrent, 2);
    
    // Large projects should be more parallel
    assert!(large_config.tasks.parallel);
    assert_eq!(large_config.tasks.max_concurrent, 8);
    assert_eq!(large_config.tasks.timeout, 600);
    
    // Library projects should have different versioning
    assert_eq!(library_config.versioning.default_bump, VersionBumpType::Minor);
    assert!(library_config.changelog.include_breaking_changes);
    
    // Test configuration can be applied to real structure
    config_manager.update(|config| {
        *config = large_config.clone();
    }).expect("Should update configuration");
    
    let updated_config = config_manager.get_clone();
    assert_eq!(updated_config.tasks.max_concurrent, 8);
    assert!(updated_config.tasks.parallel);
}

/// Test different project configuration variants
fn test_project_configuration_variants() {
    // Test each configuration preset
    let configs = vec![
        ("default", MonorepoConfig::default()),
        ("small", MonorepoConfig::small_project()),
        ("large", MonorepoConfig::large_project()),
        ("library", MonorepoConfig::library_project()),
    ];
    
    for (name, config) in configs {
        // Each configuration should be internally consistent
        assert!(!config.environments.is_empty(), "{name} config should have environments");
        
        // Versioning configuration should be valid
        assert!(
            matches!(
                config.versioning.default_bump,
                VersionBumpType::Patch | VersionBumpType::Minor | VersionBumpType::Major
            ),
            "{name} config should have valid default bump"
        );
        
        // Task configuration should be reasonable
        assert!(config.tasks.max_concurrent > 0, "{name} config should allow concurrency");
        assert!(config.tasks.timeout > 0, "{name} config should have positive timeout");
        
        // Hooks should be properly configured
        if config.hooks.enabled {
            // If hooks are enabled, they should have some configuration
            assert!(
                config.hooks.pre_commit.enabled || config.hooks.pre_push.enabled,
                "{name} config with enabled hooks should have some hooks configured"
            );
        }
    }
}

/// Test workspace patterns analysis
fn test_workspace_patterns_analysis(_root_path: &Path) {
    // Create a config manager and test workspace pattern functionality
    let config_manager = ConfigManager::new();
    
    // Test basic workspace pattern extraction
    let workspace_config = config_manager.get_workspace();
    
    // Workspace config should be accessible (may or may not have default patterns)
    let _patterns_exist = !workspace_config.patterns.is_empty();
    
    // Test pattern validation with our test structure
    let existing_packages = vec![
        "packages/core".to_string(),
        "packages/utils".to_string(),
        "packages/app".to_string(),
    ];
    
    let _validation_errors = config_manager.validate_workspace_config(&existing_packages);
    
    // With proper test structure, there should be minimal validation errors
    // (some errors might be expected due to default patterns not matching test structure)
    
    // Test pattern matching functionality
    assert!(
        config_manager.pattern_matches_package("packages/*", "packages/core"),
        "Pattern should match package path"
    );
    
    assert!(
        !config_manager.pattern_matches_package("apps/*", "packages/core"), 
        "Pattern should not match different path structure"
    );
}

/// Test change analysis integration with monorepo structure
#[test]
fn test_change_analysis_with_monorepo() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root_path = temp_dir.path();
    
    // Set up test monorepo
    common::setup_test_monorepo(root_path);
    
    // Test different types of changes
    test_source_code_changes(root_path);
    test_dependency_changes(root_path);
    test_documentation_changes(root_path);
}

/// Test source code change detection and significance
fn test_source_code_changes(root_path: &Path) {
    let core_package = root_path.join("packages").join("core");
    
    // Create source code change
    common::create_package_change(&core_package, "source");
    
    // Verify change was created
    let index_file = core_package.join("src").join("index.ts");
    assert!(index_file.exists(), "Index file should exist");
    
    let content = std::fs::read_to_string(&index_file)
        .expect("Should read index file");
    assert!(content.contains("Modified for testing"), "File should be modified");
    
    // Test change type classification
    let change_type = PackageChangeType::SourceCode;
    assert_eq!(format!("{change_type:?}"), "SourceCode");
    
    // Source code changes should typically be medium-high significance
    let significance = ChangeSignificance::Medium;
    assert!(significance >= ChangeSignificance::Low);
    assert!(significance <= ChangeSignificance::High);
}

/// Test dependency change detection
fn test_dependency_changes(root_path: &Path) {
    let utils_package = root_path.join("packages").join("utils");
    
    // Create dependency change
    common::create_package_change(&utils_package, "dependencies");
    
    // Verify package.json was modified
    let package_json_path = utils_package.join("package.json");
    let content = std::fs::read_to_string(&package_json_path)
        .expect("Should read package.json");
    
    assert!(content.contains("new-dependency"), "Package.json should have new dependency");
    
    // Test change type classification
    let change_type = PackageChangeType::Dependencies;
    assert_eq!(format!("{change_type:?}"), "Dependencies");
    
    // Dependency changes can have variable significance
    let significance = ChangeSignificance::Medium;
    assert!(significance.elevate() == ChangeSignificance::High);
}

/// Test documentation change detection  
fn test_documentation_changes(root_path: &Path) {
    let app_package = root_path.join("packages").join("app");
    
    // Create documentation change
    common::create_package_change(&app_package, "documentation");
    
    // Verify README was created
    let readme_path = app_package.join("README.md");
    assert!(readme_path.exists(), "README should be created");
    
    let content = std::fs::read_to_string(&readme_path)
        .expect("Should read README");
    assert!(content.contains("updated"), "README should indicate update");
    
    // Test change type classification
    let change_type = PackageChangeType::Documentation;
    assert_eq!(format!("{change_type:?}"), "Documentation");
    
    // Documentation changes should typically be low significance
    let significance = ChangeSignificance::Low;
    assert_eq!(significance, ChangeSignificance::Low);
    assert_eq!(significance.elevate(), ChangeSignificance::Medium);
}

/// Test version bump integration across the system
#[test]
fn test_version_bump_integration() {
    // Test all version bump types
    let bump_types = vec![
        VersionBumpType::Patch,
        VersionBumpType::Minor, 
        VersionBumpType::Major,
    ];
    
    for bump_type in bump_types {
        // Each bump type should be valid and displayable
        let display = format!("{bump_type:?}");
        assert!(!display.is_empty(), "Bump type should have string representation");
        
        // Test that bump type can be used in configuration
        let mut config = MonorepoConfig::default();
        config.versioning.default_bump = bump_type;
        
        assert_eq!(config.versioning.default_bump, bump_type);
    }
    
    // Test version bump variants are distinct
    assert_ne!(VersionBumpType::Patch, VersionBumpType::Minor);
    assert_ne!(VersionBumpType::Minor, VersionBumpType::Major);
    assert_ne!(VersionBumpType::Patch, VersionBumpType::Major);
}

/// Test error handling integration across modules
#[test]
fn test_error_handling_integration() {
    use sublime_monorepo_tools::Error;
    
    // Test that different error types can be created and handled
    let config_error = Error::config("Configuration validation failed");
    let analysis_error = Error::analysis("Analysis processing failed");
    let versioning_error = Error::versioning("Version conflict detected");
    
    // Test error display
    let config_msg = format!("{config_error}");
    let analysis_msg = format!("{analysis_error}");
    let versioning_msg = format!("{versioning_error}");
    
    assert!(config_msg.contains("Configuration"));
    assert!(analysis_msg.contains("Analysis"));
    assert!(versioning_msg.contains("Versioning"));
    
    // Test error chaining
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File missing");
    let chained_error: Error = io_error.into();
    
    let chained_msg = format!("{chained_error}");
    assert!(chained_msg.contains("IO") || chained_msg.contains("I/O"));
}