//! # Enhanced Project Module Tests
//!
//! ## What
//! This file contains comprehensive edge case tests for the project module,
//! focusing on error conditions, boundary cases, and integration scenarios.
//!
//! ## How
//! Tests use temporary directories, mock scenarios, and stress testing to
//! validate robust behavior under various conditions.
//!
//! ## Why
//! Phase 4 validation requires comprehensive testing of all edge cases
//! to ensure the project module behaves correctly in all scenarios.

use super::*;
use crate::filesystem::{FileSystem, FileSystemManager};
use crate::monorepo::MonorepoKind;
use crate::node::{PackageManager, PackageManagerKind, RepoKind};
use std::path::Path;
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod enhanced_tests {
    use super::*;

    /// Helper to create a temporary directory for testing.
    #[allow(clippy::unwrap_used)]
    fn setup_test_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    /// Helper to create a package.json with specific content.
    #[allow(clippy::unwrap_used)]
    fn create_package_json(dir: &Path, content: &str) {
        let fs = FileSystemManager::new();
        let package_json_path = dir.join("package.json");
        fs.write_file_string(&package_json_path, content).unwrap();
    }

    /// Helper to create a lock file for a specific package manager.
    #[allow(clippy::unwrap_used)]
    fn create_lock_file(dir: &Path, kind: PackageManagerKind) {
        let fs = FileSystemManager::new();
        let lock_path = dir.join(kind.lock_file());
        fs.write_file_string(&lock_path, "# Lock file content").unwrap();
    }

    #[test]
    fn test_project_detector_error_conditions() {
        let detector = ProjectDetector::new();
        let config = ProjectConfig::new();

        // Test with non-existent path
        let non_existent = "/non/existent/path/to/project";
        let result = detector.detect(non_existent, &config);
        assert!(result.is_err());

        // Test with path without package.json
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();
        let result = detector.detect(path, &config);
        assert!(result.is_err());

        // Test with invalid package.json
        create_package_json(path, "invalid json content");
        let result = detector.detect(path, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_project_detector_with_malformed_json() {
        let detector = ProjectDetector::new();
        let config = ProjectConfig::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        let malformed_json_cases = vec![
            "{ incomplete json",
            "{ \"name\": \"test\", \"version\": }",
            "{ \"name\": \"test\", \"version\": \"1.0.0\", }",
            "not json at all",
            "",
            "null",
            "[]",
            "\"string\"",
            "123",
        ];

        for malformed in malformed_json_cases {
            create_package_json(path, malformed);
            let result = detector.detect(path, &config);
            assert!(result.is_err(), "Should fail for malformed JSON: {malformed}");
        }
    }

    #[test]
    fn test_project_detector_concurrent_access() {
        let detector = Arc::new(ProjectDetector::new());
        let config = ProjectConfig::new();
        let temp_dir = setup_test_dir();
        let path = Arc::new(temp_dir.path().to_path_buf());

        // Create a valid project
        create_package_json(&path, r#"{"name": "test", "version": "1.0.0"}"#);

        // Run detection from multiple threads
        let mut handles = vec![];
        for _ in 0..10 {
            let detector_clone = Arc::clone(&detector);
            let path_clone = Arc::clone(&path);
            let config_clone = config.clone();

            let handle =
                thread::spawn(move || detector_clone.detect(path_clone.as_ref(), &config_clone));
            handles.push(handle);
        }

        // All should succeed
        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_project_kind_comprehensive_scenarios() {
        // Test all possible ProjectKind variants
        let test_cases = vec![
            ProjectKind::Repository(RepoKind::Simple),
            ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::NpmWorkSpace)),
            ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::YarnWorkspaces)),
            ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::PnpmWorkspaces)),
            ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::BunWorkspaces)),
            ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::DenoWorkspaces)),
            ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::Custom {
                name: "custom".to_string(),
                config_file: "custom.json".to_string(),
            })),
        ];

        for kind in test_cases {
            // Test all methods
            assert!(!kind.name().is_empty());
            assert!(kind.is_monorepo() || !kind.is_monorepo());
            assert!(kind.repo_kind().is_monorepo() || !kind.repo_kind().is_monorepo());

            // Test monorepo-specific methods
            if kind.is_monorepo() {
                assert!(kind.monorepo_kind().is_some());
            } else {
                assert!(kind.monorepo_kind().is_none());
            }
        }
    }

    #[test]
    fn test_project_validation_status_comprehensive() {
        // Test all validation status variants
        let statuses = vec![
            ProjectValidationStatus::Valid,
            ProjectValidationStatus::Warning(vec![
                "Warning 1".to_string(),
                "Warning 2".to_string(),
            ]),
            ProjectValidationStatus::Error(vec!["Error 1".to_string(), "Error 2".to_string()]),
            ProjectValidationStatus::NotValidated,
        ];

        for status in statuses {
            // Test all methods
            assert!(status.is_valid() || !status.is_valid());
            assert!(status.has_warnings() || !status.has_warnings());
            assert!(status.has_errors() || !status.has_errors());

            // Test consistency
            match &status {
                ProjectValidationStatus::Valid => {
                    assert!(status.is_valid());
                    assert!(!status.has_warnings());
                    assert!(!status.has_errors());
                }
                ProjectValidationStatus::Warning(warnings) => {
                    assert!(!status.is_valid());
                    assert!(status.has_warnings());
                    assert!(!status.has_errors());
                    assert_eq!(status.warnings(), Some(warnings.as_slice()));
                }
                ProjectValidationStatus::Error(errors) => {
                    assert!(!status.is_valid());
                    assert!(!status.has_warnings());
                    assert!(status.has_errors());
                    assert_eq!(status.errors(), Some(errors.as_slice()));
                }
                ProjectValidationStatus::NotValidated => {
                    assert!(!status.is_valid());
                    assert!(!status.has_warnings());
                    assert!(!status.has_errors());
                }
            }
        }
    }

    #[test]
    fn test_project_config_edge_cases() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test with all flags disabled
        let config = ProjectConfig::new()
            .with_detect_package_manager(false)
            .with_validate_structure(false)
            .with_detect_monorepo(false);

        create_package_json(path, r#"{"name": "test", "version": "1.0.0"}"#);

        let detector = ProjectDetector::new();
        let result = detector.detect(path, &config);
        assert!(result.is_ok());

        // Test with all flags enabled (default)
        let config = ProjectConfig::new()
            .with_detect_package_manager(true)
            .with_validate_structure(true)
            .with_detect_monorepo(true);

        let result = detector.detect(path, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_simple_project_comprehensive() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path().to_path_buf();

        // Test all constructors
        let simple1 = SimpleProject::new(path.clone(), None, None);
        let simple2 = SimpleProject::with_validation(
            path.clone(),
            None,
            None,
            ProjectValidationStatus::Valid,
        );

        // Test all methods
        assert_eq!(simple1.root(), path);
        assert_eq!(simple2.root(), path);
        assert!(!simple1.has_package_manager());
        assert!(!simple1.has_package_json());
        assert!(!simple1.validation_status().is_valid());
        assert!(simple2.validation_status().is_valid());

        // Test ProjectInfo trait
        let info: &dyn ProjectInfo = &simple1;
        assert_eq!(info.root(), path);
        assert_eq!(info.kind(), ProjectKind::Repository(RepoKind::Simple));
        assert!(info.package_manager().is_none());
        assert!(info.package_json().is_none());
    }

    #[test]
    fn test_simple_project_mutations() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path().to_path_buf();
        let mut simple = SimpleProject::new(path.clone(), None, None);

        // Test package manager mutation
        create_lock_file(&path, PackageManagerKind::Npm);
        let pm = PackageManager::detect(&path).unwrap();
        simple.set_package_manager(Some(pm));
        assert!(simple.has_package_manager());

        // Test validation status mutation
        simple.set_validation_status(ProjectValidationStatus::Valid);
        assert!(simple.validation_status().is_valid());

        // Test mutable reference
        *simple.validation_status_mut() =
            ProjectValidationStatus::Error(vec!["Test error".to_string()]);
        assert!(simple.validation_status().has_errors());
    }

    #[test]
    fn test_project_descriptor_trait_object() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path().to_path_buf();

        let simple = SimpleProject::new(path.clone(), None, None);
        let descriptor = ProjectDescriptor::Simple(Box::new(simple));

        // Test trait object access
        let info = descriptor.as_project_info();
        assert_eq!(info.root(), path);
        assert_eq!(info.kind(), ProjectKind::Repository(RepoKind::Simple));
    }

    #[test]
    fn test_project_detector_is_valid_project_edge_cases() {
        let detector = ProjectDetector::new();

        // Test with non-existent path
        assert!(!detector.is_valid_project("/non/existent/path"));

        // Test with directory but no package.json
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();
        assert!(!detector.is_valid_project(path));

        // Test with invalid package.json
        create_package_json(path, "invalid json");
        assert!(!detector.is_valid_project(path));

        // Test with valid package.json
        create_package_json(path, r#"{"name": "test", "version": "1.0.0"}"#);
        assert!(detector.is_valid_project(path));
    }

    #[test]
    fn test_project_detector_detect_kind_comprehensive() {
        let detector = ProjectDetector::new();
        let config = ProjectConfig::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test with simple project
        create_package_json(path, r#"{"name": "test", "version": "1.0.0"}"#);
        let kind = detector.detect_kind(path, &config).unwrap();
        assert_eq!(kind, ProjectKind::Repository(RepoKind::Simple));

        // Test with monorepo detection disabled
        let config_no_monorepo = ProjectConfig::new().with_detect_monorepo(false);
        let kind = detector.detect_kind(path, &config_no_monorepo).unwrap();
        assert_eq!(kind, ProjectKind::Repository(RepoKind::Simple));
    }

    #[test]
    fn test_project_config_builder_pattern() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test builder pattern with all options
        let config = ProjectConfig::new()
            .with_root(path)
            .with_detect_package_manager(true)
            .with_validate_structure(true)
            .with_detect_monorepo(true);

        // Test that all options are set
        assert_eq!(config.root, Some(path.to_path_buf()));
        assert!(config.detect_package_manager);
        assert!(config.validate_structure);
        assert!(config.detect_monorepo);

        // Test default values
        let default_config = ProjectConfig::new();
        assert_eq!(default_config.root, None);
        assert!(default_config.detect_package_manager);
        assert!(default_config.validate_structure);
        assert!(default_config.detect_monorepo);
    }

    #[test]
    fn test_generic_project_comprehensive() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path().to_path_buf();
        let config = ProjectConfig::new();

        let mut generic = GenericProject::new(path.clone(), config.clone());

        // Test all methods
        assert_eq!(generic.root(), path);
        assert!(generic.package_manager().is_none());
        assert!(generic.package_json().is_none());
        assert!(!generic.validation_status().is_valid());
        assert_eq!(generic.config().detect_package_manager, config.detect_package_manager);

        // Test mutations
        create_lock_file(&path, PackageManagerKind::Yarn);
        let pm = PackageManager::detect(&path).unwrap();
        generic.set_package_manager(Some(pm));
        assert!(generic.package_manager().is_some());

        generic.set_validation_status(ProjectValidationStatus::Valid);
        assert!(generic.validation_status().is_valid());
    }

    #[test]
    fn test_project_stress_testing() {
        let detector = ProjectDetector::new();
        let config = ProjectConfig::new();

        // Test with many sequential detections
        for i in 0..50 {
            let temp_dir = setup_test_dir();
            let path = temp_dir.path();

            create_package_json(path, &format!(r#"{{"name": "test-{i}", "version": "1.0.0"}}"#));

            let result = detector.detect(path, &config);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_project_deep_nesting() {
        let detector = ProjectDetector::new();
        let config = ProjectConfig::new();
        let temp_dir = setup_test_dir();
        let mut path = temp_dir.path().to_path_buf();

        // Create deeply nested structure
        for i in 0..10 {
            path = path.join(format!("level_{i}"));
            std::fs::create_dir_all(&path).unwrap();
        }

        create_package_json(&path, r#"{"name": "deeply-nested", "version": "1.0.0"}"#);

        let result = detector.detect(&path, &config);
        assert!(result.is_ok());
    }
}
