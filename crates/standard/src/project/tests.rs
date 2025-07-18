//! # Project Module Tests
//!
//! ## What
//! This file contains comprehensive tests for the project module,
//! covering project detection, validation, and management functionality
//! across different project types, configurations, and edge cases.
//!
//! ## How
//! Tests use temporary directories and mock filesystem operations to
//! verify behavior across different project types and configurations.
//! Enhanced tests cover concurrent access, error conditions, boundary
//! cases, and stress scenarios.
//!
//! ## Why
//! Comprehensive testing ensures the project module works correctly
//! across different project structures and edge cases, maintaining
//! reliability and preventing regressions. This unified test suite
//! combines basic functionality tests with enhanced edge case coverage.

use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

use crate::filesystem::{AsyncFileSystem, FileSystemManager};
use crate::monorepo::MonorepoKind;
use crate::node::{PackageManager, PackageManagerKind, RepoKind};

use super::*;

#[allow(clippy::unwrap_used)]
#[allow(clippy::uninlined_format_args)]
#[allow(clippy::match_wildcard_for_single_variants)]
#[allow(clippy::panic)]
#[allow(clippy::assertions_on_constants)]
#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // HELPER FUNCTIONS
    // =============================================================================

    /// Creates a temporary directory for testing
    fn setup_test_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    /// Creates a basic package.json file in the given directory
    async fn create_package_json(dir: &Path, name: &str, version: &str) {
        let fs = FileSystemManager::new();
        let package_json_path = dir.join("package.json");
        let content = format!(
            r#"{{
  "name": "{}",
  "version": "{}",
  "description": "Test package",
  "main": "index.js",
  "scripts": {{
    "start": "node index.js"
  }}
}}"#,
            name, version
        );
        fs.write_file_string(&package_json_path, &content).await.unwrap();
    }

    /// Creates a package.json with specific content
    #[allow(clippy::unwrap_used)]
    async fn create_package_json_with_content(dir: &Path, content: &str) {
        let fs = FileSystemManager::new();
        let package_json_path = dir.join("package.json");
        fs.write_file_string(&package_json_path, content).await.unwrap();
    }

    /// Creates a package manager lock file in the given directory
    async fn create_lock_file(dir: &Path, kind: PackageManagerKind) {
        let fs = FileSystemManager::new();
        let lock_path = dir.join(kind.lock_file());
        fs.write_file_string(&lock_path, "# Lock file content").await.unwrap();
    }

    // =============================================================================
    // PROJECT KIND TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_project_kind_simple() {
        let kind = ProjectKind::Repository(RepoKind::Simple);
        assert_eq!(kind.name(), "simple");
        assert!(!kind.is_monorepo());
    }

    #[tokio::test]
    async fn test_project_kind_monorepo() {
        let kind = ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::YarnWorkspaces));
        assert_eq!(kind.name(), "yarn monorepo");
        assert!(kind.is_monorepo());
    }

    #[tokio::test]
    async fn test_project_kind_equality() {
        let kind1 = ProjectKind::Repository(RepoKind::Simple);
        let kind2 = ProjectKind::Repository(RepoKind::Simple);
        let kind3 = ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::NpmWorkSpace));

        assert_eq!(kind1, kind2);
        assert_ne!(kind1, kind3);
    }

    #[tokio::test]
    async fn test_project_kind_comprehensive_scenarios() {
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

    // =============================================================================
    // PROJECT VALIDATION STATUS TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_validation_status_valid() {
        let status = ProjectValidationStatus::Valid;
        assert!(status.is_valid());
        assert!(!status.has_warnings());
        assert!(!status.has_errors());
        assert!(status.warnings().is_none());
        assert!(status.errors().is_none());
    }

    #[tokio::test]
    async fn test_validation_status_warning() {
        let warnings = vec!["Missing LICENSE".to_string()];
        let status = ProjectValidationStatus::Warning(warnings.clone());

        assert!(!status.is_valid());
        assert!(status.has_warnings());
        assert!(!status.has_errors());
        assert_eq!(status.warnings(), Some(warnings.as_slice()));
        assert!(status.errors().is_none());
    }

    #[tokio::test]
    async fn test_validation_status_error() {
        let errors = vec!["Invalid package.json".to_string()];
        let status = ProjectValidationStatus::Error(errors.clone());

        assert!(!status.is_valid());
        assert!(!status.has_warnings());
        assert!(status.has_errors());
        assert!(status.warnings().is_none());
        assert_eq!(status.errors(), Some(errors.as_slice()));
    }

    #[tokio::test]
    async fn test_validation_status_not_validated() {
        let status = ProjectValidationStatus::NotValidated;
        assert!(!status.is_valid());
        assert!(!status.has_warnings());
        assert!(!status.has_errors());
        assert!(status.warnings().is_none());
        assert!(status.errors().is_none());
    }

    #[tokio::test]
    async fn test_project_validation_status_comprehensive() {
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

    // =============================================================================
    // PROJECT CONFIG TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_project_config_default() {
        let config = ProjectConfig::new();
        assert!(config.detect_package_manager);
        assert!(config.validate_structure);
        assert!(config.detect_monorepo);
        assert!(config.root.is_none());
    }

    #[tokio::test]
    async fn test_project_config_builder() {
        let config = ProjectConfig::new()
            .with_root("/test/path")
            .with_detect_package_manager(false)
            .with_validate_structure(false)
            .with_detect_monorepo(false);

        assert!(!config.detect_package_manager);
        assert!(!config.validate_structure);
        assert!(!config.detect_monorepo);
        assert_eq!(config.root, Some("/test/path".into()));
    }

    #[tokio::test]
    async fn test_project_config_edge_cases() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test with all flags disabled
        let config = ProjectConfig::new()
            .with_detect_package_manager(false)
            .with_validate_structure(false)
            .with_detect_monorepo(false);

        create_package_json(path, "test", "1.0.0").await;

        let detector = ProjectDetector::new();
        let result = detector.detect(path, &config).await;
        assert!(result.is_ok());

        // Test with all flags enabled (default)
        let config = ProjectConfig::new()
            .with_detect_package_manager(true)
            .with_validate_structure(true)
            .with_detect_monorepo(true);

        let result = detector.detect(path, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_project_config_builder_pattern() {
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

    // =============================================================================
    // SIMPLE PROJECT TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_simple_project_creation() {
        let root = "/test/project".into();
        let project = Project::new(root, ProjectKind::Repository(RepoKind::Simple));

        assert_eq!(project.root(), Path::new("/test/project"));
        assert_eq!(project.kind(), ProjectKind::Repository(RepoKind::Simple));
        assert!(project.package_json().is_none());
        assert!(project.package_manager().is_none());
        assert_eq!(project.validation_status(), &ProjectValidationStatus::NotValidated);
    }

    #[tokio::test]
    async fn test_simple_project_with_validation() {
        let root = "/test/project".into();
        let status = ProjectValidationStatus::Valid;
        let mut project = Project::new(root, ProjectKind::Repository(RepoKind::Simple));
        project.set_validation_status(status);

        assert_eq!(project.root(), Path::new("/test/project"));
        assert!(project.validation_status().is_valid());
    }

    #[tokio::test]
    async fn test_simple_project_setters() {
        let root = "/test/project".into();
        let mut project = Project::new(root, ProjectKind::Repository(RepoKind::Simple));

        let package_manager = PackageManager::new(PackageManagerKind::Npm, "/test/project");
        project.package_manager = Some(package_manager);
        assert!(project.package_manager().is_some());

        project.set_validation_status(ProjectValidationStatus::Valid);
        assert!(project.validation_status().is_valid());
    }

    #[tokio::test]
    async fn test_simple_project_comprehensive() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path().to_path_buf();

        // Test all constructors
        let simple1 = Project::new(path.clone(), ProjectKind::Repository(RepoKind::Simple));
        let mut simple2 = Project::new(path.clone(), ProjectKind::Repository(RepoKind::Simple));
        simple2.set_validation_status(ProjectValidationStatus::Valid);

        // Test all methods
        assert_eq!(simple1.root(), path);
        assert_eq!(simple2.root(), path);
        assert!(simple1.package_manager().is_none());
        assert!(simple1.package_json().is_none());
        assert!(!simple1.validation_status().is_valid());
        assert!(simple2.validation_status().is_valid());

        // Test ProjectInfo trait
        let info: &dyn ProjectInfo = &simple1;
        assert_eq!(info.root(), path);
        assert_eq!(info.kind(), ProjectKind::Repository(RepoKind::Simple));
        assert!(info.package_manager().is_none());
        assert!(info.package_json().is_none());
    }

    #[tokio::test]
    async fn test_simple_project_mutations() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path().to_path_buf();
        let mut simple = Project::new(path.clone(), ProjectKind::Repository(RepoKind::Simple));

        // Test package manager mutation
        create_lock_file(&path, PackageManagerKind::Npm).await;
        let pm = PackageManager::detect(&path).unwrap();
        simple.package_manager = Some(pm);
        assert!(simple.package_manager().is_some());

        // Test validation status mutation
        simple.set_validation_status(ProjectValidationStatus::Valid);
        assert!(simple.validation_status().is_valid());

        // Test direct status mutation
        simple.set_validation_status(ProjectValidationStatus::Error(vec!["Test error".to_string()]));
        assert!(simple.validation_status().has_errors());
    }

    #[tokio::test]
    async fn test_project_descriptor_trait_object() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path().to_path_buf();

        let project = Project::new(path.clone(), ProjectKind::Repository(RepoKind::Simple));
        let descriptor = ProjectDescriptor::NodeJs(project);

        // Test trait object access
        let info = descriptor.as_project_info();
        assert_eq!(info.root(), path);
        assert_eq!(info.kind(), ProjectKind::Repository(RepoKind::Simple));
    }

    // =============================================================================
    // PROJECT DETECTOR TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_detector_creation() {
        let _detector = ProjectDetector::new();
        // Just ensure it can be created without panicking
        assert!(true);
    }

    #[tokio::test]
    async fn test_is_valid_project_with_valid_project() {
        let temp_dir = setup_test_dir();
        let detector = ProjectDetector::new();

        // Create a valid project
        create_package_json(temp_dir.path(), "test-project", "1.0.0").await;

        assert!(detector.is_valid_project(temp_dir.path()).await);
    }

    #[tokio::test]
    async fn test_is_valid_project_without_package_json() {
        let temp_dir = setup_test_dir();
        let detector = ProjectDetector::new();

        // Directory exists but no package.json
        assert!(!detector.is_valid_project(temp_dir.path()).await);
    }

    #[tokio::test]
    async fn test_is_valid_project_with_invalid_package_json() {
        let temp_dir = setup_test_dir();
        let detector = ProjectDetector::new();
        let fs = FileSystemManager::new();

        // Create invalid package.json
        let package_json_path = temp_dir.path().join("package.json");
        fs.write_file_string(&package_json_path, "invalid json content").await.unwrap();

        assert!(!detector.is_valid_project(temp_dir.path()).await);
    }

    #[tokio::test]
    async fn test_detect_kind_simple_project() {
        let temp_dir = setup_test_dir();
        let detector = ProjectDetector::new();
        let config = ProjectConfig::new().with_detect_monorepo(false);

        create_package_json(temp_dir.path(), "test-project", "1.0.0").await;

        let kind = detector.detect_kind(temp_dir.path(), &config).await.unwrap();
        assert_eq!(kind, ProjectKind::Repository(RepoKind::Simple));
    }

    #[tokio::test]
    async fn test_detect_simple_project() {
        let temp_dir = setup_test_dir();
        let detector = ProjectDetector::new();
        let config = ProjectConfig::new().with_detect_monorepo(false);

        create_package_json(temp_dir.path(), "test-project", "1.0.0").await;
        create_lock_file(temp_dir.path(), PackageManagerKind::Npm).await;

        let result = detector.detect(temp_dir.path(), &config).await;
        assert!(result.is_ok());

        let project = result.unwrap();
        match project {
            ProjectDescriptor::NodeJs(project) => {
                assert_eq!(project.root(), temp_dir.path());
                assert_eq!(project.kind(), ProjectKind::Repository(RepoKind::Simple));
                assert!(project.package_json().is_some());
                assert!(project.package_manager().is_some());
            }
        }
    }

    #[tokio::test]
    async fn test_project_detector_error_conditions() {
        let detector = ProjectDetector::new();
        let config = ProjectConfig::new();

        // Test with non-existent path
        let non_existent = "/non/existent/path/to/project";
        let result = detector.detect(non_existent, &config).await;
        assert!(result.is_err());

        // Test with path without package.json
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();
        let result = detector.detect(path, &config).await;
        assert!(result.is_err());

        // Test with invalid package.json
        create_package_json_with_content(path, "invalid json content").await;
        let result = detector.detect(path, &config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_project_detector_with_malformed_json() {
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
            create_package_json_with_content(path, malformed).await;
            let result = detector.detect(path, &config).await;
            assert!(result.is_err(), "Should fail for malformed JSON: {malformed}");
        }
    }

    #[tokio::test]
    async fn test_project_detector_is_valid_project_edge_cases() {
        let detector = ProjectDetector::new();

        // Test with non-existent path
        assert!(!detector.is_valid_project("/non/existent/path").await);

        // Test with directory but no package.json
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();
        assert!(!detector.is_valid_project(path).await);

        // Test with invalid package.json
        create_package_json_with_content(path, "invalid json").await;
        assert!(!detector.is_valid_project(path).await);

        // Test with valid package.json
        create_package_json_with_content(path, r#"{"name": "test", "version": "1.0.0"}"#).await;
        assert!(detector.is_valid_project(path).await);
    }

    #[tokio::test]
    async fn test_project_detector_detect_kind_comprehensive() {
        let detector = ProjectDetector::new();
        let config = ProjectConfig::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test with simple project
        create_package_json_with_content(path, r#"{"name": "test", "version": "1.0.0"}"#).await;
        let kind = detector.detect_kind(path, &config).await.unwrap();
        assert_eq!(kind, ProjectKind::Repository(RepoKind::Simple));

        // Test with monorepo detection disabled
        let config_no_monorepo = ProjectConfig::new().with_detect_monorepo(false);
        let kind = detector.detect_kind(path, &config_no_monorepo).await.unwrap();
        assert_eq!(kind, ProjectKind::Repository(RepoKind::Simple));
    }

    #[tokio::test]
    async fn test_project_detector_concurrent_access() {
        let detector = Arc::new(ProjectDetector::new());
        let config = ProjectConfig::new();
        let temp_dir = setup_test_dir();
        let path = Arc::new(temp_dir.path().to_path_buf());

        // Create a valid project
        create_package_json_with_content(&path, r#"{"name": "test", "version": "1.0.0"}"#).await;

        // Run detection from multiple async tasks
        let mut handles = vec![];
        for _ in 0..10 {
            let handle = tokio::spawn({
                let detector_clone = Arc::clone(&detector);
                let path_clone = Arc::clone(&path);
                let config_clone = config.clone();
                async move { detector_clone.detect(path_clone.as_ref(), &config_clone).await }
            });
            handles.push(handle);
        }

        // All should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    // =============================================================================
    // PROJECT MANAGER TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_manager_creation() {
        let _manager = ProjectManager::new();
        // Just ensure it can be created without panicking
        assert!(true);
    }

    #[tokio::test]
    async fn test_is_valid_project() {
        let temp_dir = setup_test_dir();
        let manager = ProjectManager::new();

        create_package_json(temp_dir.path(), "test-project", "1.0.0").await;
        assert!(manager.is_valid_project(temp_dir.path()).await);
    }

    #[tokio::test]
    async fn test_create_project() {
        let temp_dir = setup_test_dir();
        let manager = ProjectManager::new();
        let config = ProjectConfig::new().with_detect_monorepo(false);

        create_package_json(temp_dir.path(), "test-project", "1.0.0").await;
        create_lock_file(temp_dir.path(), PackageManagerKind::Npm).await;

        let result = manager.create_project(temp_dir.path(), &config).await;
        assert!(result.is_ok());

        let project = result.unwrap();
        let info = project.as_project_info();
        assert_eq!(info.root(), temp_dir.path());
        assert_eq!(info.kind(), ProjectKind::Repository(RepoKind::Simple));
    }

    #[tokio::test]
    async fn test_find_project_root() {
        let temp_dir = setup_test_dir();
        let manager = ProjectManager::new();

        // Create nested directory structure
        let nested_dir = temp_dir.path().join("src").join("components");
        std::fs::create_dir_all(&nested_dir).unwrap();

        // Create package.json at root
        create_package_json(temp_dir.path(), "test-project", "1.0.0").await;

        // Should find root from nested path
        let root = manager.find_project_root(&nested_dir).await;
        assert_eq!(root, Some(temp_dir.path().to_path_buf()));
    }

    #[tokio::test]
    async fn test_find_project_root_not_found() {
        let temp_dir = setup_test_dir();
        let manager = ProjectManager::new();

        // No package.json anywhere
        let root = manager.find_project_root(temp_dir.path()).await;
        assert!(root.is_none());
    }

    // =============================================================================
    // PROJECT VALIDATOR TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_validator_creation() {
        let _validator = ProjectValidator::new();
        // Just ensure it can be created without panicking
        assert!(true);
    }

    #[tokio::test]
    async fn test_validate_project_descriptor_valid() {
        let temp_dir = setup_test_dir();
        let validator = ProjectValidator::new();

        create_package_json(temp_dir.path(), "test-project", "1.0.0").await;
        create_lock_file(temp_dir.path(), PackageManagerKind::Npm).await;

        let package_manager = PackageManager::new(PackageManagerKind::Npm, temp_dir.path());
        let fs = FileSystemManager::new();
        let content = fs.read_file_string(&temp_dir.path().join("package.json")).await.unwrap();
        let package_json = serde_json::from_str(&content).unwrap();

        let mut unified_project = Project::new(
            temp_dir.path().to_path_buf(),
            ProjectKind::Repository(RepoKind::Simple),
        );
        unified_project.package_manager = Some(package_manager);
        unified_project.package_json = Some(package_json);

        let mut project = ProjectDescriptor::NodeJs(unified_project);

        let result = validator.validate_project(&mut project).await;
        assert!(result.is_ok());

        // The project should have some validation status (not NotValidated)
        match project.as_project_info().validation_status() {
            ProjectValidationStatus::NotValidated => panic!("Project should be validated"),
            _ => {} // Valid, Warning, or Error are all acceptable
        }
    }

    #[tokio::test]
    async fn test_validate_project_descriptor_missing_package_json() {
        let temp_dir = setup_test_dir();
        let validator = ProjectValidator::new();

        // No package.json created
        let unified_project = Project::new(temp_dir.path().to_path_buf(), ProjectKind::Repository(RepoKind::Simple));
        let mut project = ProjectDescriptor::NodeJs(unified_project);

        let result = validator.validate_project(&mut project).await;
        assert!(result.is_ok());

        // Should have errors
        assert!(project.as_project_info().validation_status().has_errors());
    }

    #[tokio::test]
    async fn test_validate_project_descriptor_with_warnings() {
        let temp_dir = setup_test_dir();
        let validator = ProjectValidator::new();

        // Create package.json with potential warning conditions
        let fs = FileSystemManager::new();
        let package_json_path = temp_dir.path().join("package.json");
        let content = r#"{
  "name": "test-project",
  "version": "1.0.0"
}"#;
        fs.write_file_string(&package_json_path, content).await.unwrap();

        let package_json = serde_json::from_str(content).unwrap();
        let mut unified_project = Project::new(temp_dir.path().to_path_buf(), ProjectKind::Repository(RepoKind::Simple));
        unified_project.package_json = Some(package_json);

        let mut project = ProjectDescriptor::NodeJs(unified_project);

        let result = validator.validate_project(&mut project).await;
        assert!(result.is_ok());

        // Should have warnings (missing description, license, etc.)
        assert!(project.as_project_info().validation_status().has_warnings());
    }

    // =============================================================================
    // GENERIC PROJECT TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_generic_project_comprehensive() {
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
        create_lock_file(&path, PackageManagerKind::Yarn).await;
        let pm = PackageManager::detect(&path).unwrap();
        generic.set_package_manager(Some(pm));
        assert!(generic.package_manager().is_some());

        generic.set_validation_status(ProjectValidationStatus::Valid);
        assert!(generic.validation_status().is_valid());
    }

    // =============================================================================
    // STRESS TESTING AND EDGE CASES
    // =============================================================================

    #[tokio::test]
    async fn test_project_stress_testing() {
        let detector = ProjectDetector::new();
        let config = ProjectConfig::new();

        // Test with many sequential detections
        for i in 0..50 {
            let temp_dir = setup_test_dir();
            let path = temp_dir.path();

            create_package_json_with_content(path, &format!(r#"{{"name": "test-{i}", "version": "1.0.0"}}"#)).await;

            let result = detector.detect(path, &config).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_project_deep_nesting() {
        let detector = ProjectDetector::new();
        let config = ProjectConfig::new();
        let temp_dir = setup_test_dir();
        let mut path = temp_dir.path().to_path_buf();

        // Create deeply nested structure
        for i in 0..10 {
            path = path.join(format!("level_{i}"));
            std::fs::create_dir_all(&path).unwrap();
        }

        create_package_json_with_content(&path, r#"{"name": "deeply-nested", "version": "1.0.0"}"#).await;

        let result = detector.detect(&path, &config).await;
        assert!(result.is_ok());
    }
}