//! # Project Module Tests
//!
//! ## What
//! This file contains comprehensive tests for the project module,
//! covering project detection, validation, and management functionality.
//!
//! ## How
//! Tests use temporary directories and mock filesystem operations to
//! verify behavior across different project types and configurations.
//!
//! ## Why
//! Comprehensive testing ensures the project module works correctly
//! across different project structures and edge cases, maintaining
//! reliability and preventing regressions.

use super::*;
use crate::filesystem::{FileSystem, FileSystemManager};
use crate::node::{PackageManager, PackageManagerKind, RepoKind};
use std::path::Path;
use tempfile::TempDir;

#[allow(clippy::unwrap_used)]
/// Creates a temporary directory for testing.
fn setup_test_dir() -> TempDir {
    TempDir::new().unwrap()
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::uninlined_format_args)]
/// Creates a basic package.json file in the given directory.
fn create_package_json(dir: &Path, name: &str, version: &str) {
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
    fs.write_file_string(&package_json_path, &content).unwrap();
}

#[allow(clippy::unwrap_used)]
/// Creates a package manager lock file in the given directory.
fn create_lock_file(dir: &Path, kind: PackageManagerKind) {
    let fs = FileSystemManager::new();
    let lock_path = dir.join(kind.lock_file());
    fs.write_file_string(&lock_path, "# Lock file content").unwrap();
}

#[cfg(test)]
mod project_kind_tests {
    use super::*;
    use crate::monorepo::MonorepoKind;

    #[test]
    fn test_project_kind_simple() {
        let kind = ProjectKind::Repository(RepoKind::Simple);
        assert_eq!(kind.name(), "simple");
        assert!(!kind.is_monorepo());
    }

    #[test]
    fn test_project_kind_monorepo() {
        let kind = ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::YarnWorkspaces));
        assert_eq!(kind.name(), "yarn monorepo");
        assert!(kind.is_monorepo());
    }

    #[test]
    fn test_project_kind_equality() {
        let kind1 = ProjectKind::Repository(RepoKind::Simple);
        let kind2 = ProjectKind::Repository(RepoKind::Simple);
        let kind3 = ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::NpmWorkSpace));

        assert_eq!(kind1, kind2);
        assert_ne!(kind1, kind3);
    }
}

#[cfg(test)]
mod project_validation_status_tests {
    use super::*;

    #[test]
    fn test_validation_status_valid() {
        let status = ProjectValidationStatus::Valid;
        assert!(status.is_valid());
        assert!(!status.has_warnings());
        assert!(!status.has_errors());
        assert!(status.warnings().is_none());
        assert!(status.errors().is_none());
    }

    #[test]
    fn test_validation_status_warning() {
        let warnings = vec!["Missing LICENSE".to_string()];
        let status = ProjectValidationStatus::Warning(warnings.clone());

        assert!(!status.is_valid());
        assert!(status.has_warnings());
        assert!(!status.has_errors());
        assert_eq!(status.warnings(), Some(warnings.as_slice()));
        assert!(status.errors().is_none());
    }

    #[test]
    fn test_validation_status_error() {
        let errors = vec!["Invalid package.json".to_string()];
        let status = ProjectValidationStatus::Error(errors.clone());

        assert!(!status.is_valid());
        assert!(!status.has_warnings());
        assert!(status.has_errors());
        assert!(status.warnings().is_none());
        assert_eq!(status.errors(), Some(errors.as_slice()));
    }

    #[test]
    fn test_validation_status_not_validated() {
        let status = ProjectValidationStatus::NotValidated;
        assert!(!status.is_valid());
        assert!(!status.has_warnings());
        assert!(!status.has_errors());
        assert!(status.warnings().is_none());
        assert!(status.errors().is_none());
    }
}

#[cfg(test)]
mod project_config_tests {
    use super::*;

    #[test]
    fn test_project_config_default() {
        let config = ProjectConfig::new();
        assert!(config.detect_package_manager);
        assert!(config.validate_structure);
        assert!(config.detect_monorepo);
        assert!(config.root.is_none());
    }

    #[test]
    fn test_project_config_builder() {
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
}

#[cfg(test)]
mod simple_project_tests {
    use super::*;

    #[test]
    fn test_simple_project_creation() {
        let root = "/test/project".into();
        let project = SimpleProject::new(root, None, None);

        assert_eq!(project.root(), Path::new("/test/project"));
        assert_eq!(project.kind(), ProjectKind::Repository(RepoKind::Simple));
        assert!(!project.has_package_json());
        assert!(!project.has_package_manager());
        assert_eq!(project.validation_status(), &ProjectValidationStatus::NotValidated);
    }

    #[test]
    fn test_simple_project_with_validation() {
        let root = "/test/project".into();
        let status = ProjectValidationStatus::Valid;
        let project = SimpleProject::with_validation(root, None, None, status);

        assert_eq!(project.root(), Path::new("/test/project"));
        assert!(project.validation_status().is_valid());
    }

    #[test]
    fn test_simple_project_setters() {
        let root = "/test/project".into();
        let mut project = SimpleProject::new(root, None, None);

        let package_manager = PackageManager::new(PackageManagerKind::Npm, "/test/project");
        project.set_package_manager(Some(package_manager));
        assert!(project.has_package_manager());

        project.set_validation_status(ProjectValidationStatus::Valid);
        assert!(project.validation_status().is_valid());
    }
}

#[allow(clippy::match_wildcard_for_single_variants)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[allow(clippy::assertions_on_constants)]
#[cfg(test)]
mod project_detector_tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let _detector = ProjectDetector::new();
        // Just ensure it can be created without panicking
        assert!(true);
    }

    #[test]
    fn test_is_valid_project_with_valid_project() {
        let temp_dir = setup_test_dir();
        let detector = ProjectDetector::new();

        // Create a valid project
        create_package_json(temp_dir.path(), "test-project", "1.0.0");

        assert!(detector.is_valid_project(temp_dir.path()));
    }

    #[test]
    fn test_is_valid_project_without_package_json() {
        let temp_dir = setup_test_dir();
        let detector = ProjectDetector::new();

        // Directory exists but no package.json
        assert!(!detector.is_valid_project(temp_dir.path()));
    }

    #[test]
    fn test_is_valid_project_with_invalid_package_json() {
        let temp_dir = setup_test_dir();
        let detector = ProjectDetector::new();
        let fs = FileSystemManager::new();

        // Create invalid package.json
        let package_json_path = temp_dir.path().join("package.json");
        fs.write_file_string(&package_json_path, "invalid json content").unwrap();

        assert!(!detector.is_valid_project(temp_dir.path()));
    }

    #[test]
    fn test_detect_kind_simple_project() {
        let temp_dir = setup_test_dir();
        let detector = ProjectDetector::new();
        let config = ProjectConfig::new().with_detect_monorepo(false);

        create_package_json(temp_dir.path(), "test-project", "1.0.0");

        let kind = detector.detect_kind(temp_dir.path(), &config).unwrap();
        assert_eq!(kind, ProjectKind::Repository(RepoKind::Simple));
    }

    #[test]
    fn test_detect_simple_project() {
        let temp_dir = setup_test_dir();
        let detector = ProjectDetector::new();
        let config = ProjectConfig::new().with_detect_monorepo(false);

        create_package_json(temp_dir.path(), "test-project", "1.0.0");
        create_lock_file(temp_dir.path(), PackageManagerKind::Npm);

        let result = detector.detect(temp_dir.path(), &config);
        assert!(result.is_ok());

        let project = result.unwrap();
        match project {
            ProjectDescriptor::Simple(simple) => {
                assert_eq!(simple.root(), temp_dir.path());
                assert_eq!(simple.kind(), ProjectKind::Repository(RepoKind::Simple));
                assert!(simple.has_package_json());
                assert!(simple.has_package_manager());
            }
            _ => panic!("Expected simple project"),
        }
    }
}

#[allow(clippy::assertions_on_constants)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod project_manager_tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let _manager = ProjectManager::new();
        // Just ensure it can be created without panicking
        assert!(true);
    }

    #[test]
    fn test_is_valid_project() {
        let temp_dir = setup_test_dir();
        let manager = ProjectManager::new();

        create_package_json(temp_dir.path(), "test-project", "1.0.0");
        assert!(manager.is_valid_project(temp_dir.path()));
    }

    #[test]
    fn test_create_project() {
        let temp_dir = setup_test_dir();
        let manager = ProjectManager::new();
        let config = ProjectConfig::new().with_detect_monorepo(false);

        create_package_json(temp_dir.path(), "test-project", "1.0.0");
        create_lock_file(temp_dir.path(), PackageManagerKind::Npm);

        let result = manager.create_project(temp_dir.path(), &config);
        assert!(result.is_ok());

        let project = result.unwrap();
        let info = project.as_project_info();
        assert_eq!(info.root(), temp_dir.path());
        assert_eq!(info.kind(), ProjectKind::Repository(RepoKind::Simple));
    }

    #[test]
    fn test_find_project_root() {
        let temp_dir = setup_test_dir();
        let manager = ProjectManager::new();

        // Create nested directory structure
        let nested_dir = temp_dir.path().join("src").join("components");
        std::fs::create_dir_all(&nested_dir).unwrap();

        // Create package.json at root
        create_package_json(temp_dir.path(), "test-project", "1.0.0");

        // Should find root from nested path
        let root = manager.find_project_root(&nested_dir);
        assert_eq!(root, Some(temp_dir.path().to_path_buf()));
    }

    #[test]
    fn test_find_project_root_not_found() {
        let temp_dir = setup_test_dir();
        let manager = ProjectManager::new();

        // No package.json anywhere
        let root = manager.find_project_root(temp_dir.path());
        assert!(root.is_none());
    }
}

#[allow(clippy::assertions_on_constants)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod project_validator_tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let _validator = ProjectValidator::new();
        // Just ensure it can be created without panicking
        assert!(true);
    }

    #[test]
    fn test_validate_project_descriptor_valid() {
        let temp_dir = setup_test_dir();
        let validator = ProjectValidator::new();

        create_package_json(temp_dir.path(), "test-project", "1.0.0");
        create_lock_file(temp_dir.path(), PackageManagerKind::Npm);

        let package_manager = PackageManager::new(PackageManagerKind::Npm, temp_dir.path());
        let fs = FileSystemManager::new();
        let content = fs.read_file_string(&temp_dir.path().join("package.json")).unwrap();
        let package_json = serde_json::from_str(&content).unwrap();

        let simple_project = SimpleProject::new(
            temp_dir.path().to_path_buf(),
            Some(package_manager),
            Some(package_json),
        );

        let mut project = ProjectDescriptor::Simple(Box::new(simple_project));

        let result = validator.validate_project(&mut project);
        assert!(result.is_ok());

        // The project should have some validation status (not NotValidated)
        match project.as_project_info().validation_status() {
            ProjectValidationStatus::NotValidated => panic!("Project should be validated"),
            _ => {} // Valid, Warning, or Error are all acceptable
        }
    }

    #[test]
    fn test_validate_project_descriptor_missing_package_json() {
        let temp_dir = setup_test_dir();
        let validator = ProjectValidator::new();

        // No package.json created
        let simple_project = SimpleProject::new(temp_dir.path().to_path_buf(), None, None);
        let mut project = ProjectDescriptor::Simple(Box::new(simple_project));

        let result = validator.validate_project(&mut project);
        assert!(result.is_ok());

        // Should have errors
        assert!(project.as_project_info().validation_status().has_errors());
    }

    #[test]
    fn test_validate_project_descriptor_with_warnings() {
        let temp_dir = setup_test_dir();
        let validator = ProjectValidator::new();

        // Create package.json with potential warning conditions
        let fs = FileSystemManager::new();
        let package_json_path = temp_dir.path().join("package.json");
        let content = r#"{
  "name": "test-project",
  "version": "1.0.0"
}"#;
        fs.write_file_string(&package_json_path, content).unwrap();

        let package_json = serde_json::from_str(content).unwrap();
        let simple_project =
            SimpleProject::new(temp_dir.path().to_path_buf(), None, Some(package_json));

        let mut project = ProjectDescriptor::Simple(Box::new(simple_project));

        let result = validator.validate_project(&mut project);
        assert!(result.is_ok());

        // Should have warnings (missing description, license, etc.)
        assert!(project.as_project_info().validation_status().has_warnings());
    }
}
