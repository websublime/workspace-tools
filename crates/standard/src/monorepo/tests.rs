//! # Monorepo Module Tests
//!
//! ## What
//! This file contains unit tests for the monorepo module functionality,
//! ensuring all components work correctly independently and together.
//!
//! ## How
//! Tests are organized into sections covering MonorepoKind, MonorepoDescriptor,
//! PackageManagerKind, and PackageManager. Each test focuses on a specific
//! aspect of functionality with clear assertions.
//!
//! ## Why
//! Comprehensive testing ensures that the monorepo detection, analysis, and
//! package manager operations work correctly across different scenarios and
//! edge cases, providing confidence in the reliability of the module.

#[allow(clippy::unwrap_used)]
#[allow(clippy::get_unwrap)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests {
    use crate::error::{Error, MonorepoError};
    use crate::filesystem::{FileSystem, FileSystemManager};
    use crate::monorepo::{
        types::{PackageManager, PackageManagerKind},
        MonorepoDescriptor, MonorepoKind, WorkspacePackage,
    };
    use crate::monorepo::MonorepoDetector;
    use crate::project::{
        ConfigFormat, ConfigManager, ConfigScope, ConfigValue, ProjectConfig, ProjectManager,
        ProjectValidationStatus,
    };
    use crate::project::GenericProject;
    use std::collections::HashMap;
    use std::f64;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    // Helper function to create a temporary directory for tests
    fn setup_test_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temporary directory for test")
    }

    #[test]
    fn test_monorepo_kind_names() {
        assert_eq!(MonorepoKind::NpmWorkSpace.name(), "npm");
        assert_eq!(MonorepoKind::YarnWorkspaces.name(), "yarn");
        assert_eq!(MonorepoKind::PnpmWorkspaces.name(), "pnpm");
        assert_eq!(MonorepoKind::BunWorkspaces.name(), "bun");
        assert_eq!(MonorepoKind::DenoWorkspaces.name(), "deno");

        let custom = MonorepoKind::Custom {
            name: "turbo".to_string(),
            config_file: "turbo.json".to_string(),
        };
        assert_eq!(custom.name(), "turbo");
    }

    #[test]
    fn test_monorepo_kind_config_files() {
        assert_eq!(MonorepoKind::NpmWorkSpace.config_file(), "package.json");
        assert_eq!(MonorepoKind::YarnWorkspaces.config_file(), "package.json");
        assert_eq!(MonorepoKind::PnpmWorkspaces.config_file(), "pnpm-workspace.yaml");
        assert_eq!(MonorepoKind::BunWorkspaces.config_file(), "bunfig.toml");
        assert_eq!(MonorepoKind::DenoWorkspaces.config_file(), "deno.json");

        let custom =
            MonorepoKind::Custom { name: "nx".to_string(), config_file: "nx.json".to_string() };
        assert_eq!(custom.config_file(), "nx.json");
    }

    #[test]
    fn test_set_custom() {
        let npm = MonorepoKind::NpmWorkSpace;
        let custom = npm.set_custom("rush".to_string(), "rush.json".to_string());

        assert_eq!(custom.name(), "rush");
        assert_eq!(custom.config_file(), "rush.json");

        // Original should be unchanged
        assert_eq!(npm.name(), "npm");
    }

    #[test]
    fn test_monorepo_descriptor_creation() {
        let root = PathBuf::from("/fake/monorepo");
        let packages = vec![
            create_test_package("pkg-a", "1.0.0", "packages/a", &root, vec![], vec![]),
            create_test_package("pkg-b", "1.0.0", "packages/b", &root, vec!["pkg-a"], vec![]),
        ];

        let descriptor =
            MonorepoDescriptor::minimal(MonorepoKind::YarnWorkspaces, root.clone(), packages);

        assert_eq!(descriptor.kind().name(), "yarn");
        assert_eq!(descriptor.root(), root.as_path());
        assert_eq!(descriptor.packages().len(), 2);
    }

    #[test]
    fn test_get_package() {
        let root = PathBuf::from("/fake/monorepo");
        let packages = vec![
            create_test_package("pkg-a", "1.0.0", "packages/a", &root, vec![], vec![]),
            create_test_package("pkg-b", "1.0.0", "packages/b", &root, vec!["pkg-a"], vec![]),
        ];

        let descriptor = MonorepoDescriptor::minimal(MonorepoKind::YarnWorkspaces, root, packages);

        // Test existing package
        let pkg_a = descriptor.get_package("pkg-a");
        assert!(pkg_a.is_some());
        assert_eq!(pkg_a.unwrap().name, "pkg-a");

        // Test non-existent package
        let pkg_c = descriptor.get_package("pkg-c");
        assert!(pkg_c.is_none());
    }

    #[test]
    fn test_dependency_graph() {
        let root = PathBuf::from("/fake/monorepo");
        let packages = vec![
            create_test_package("pkg-a", "1.0.0", "packages/a", &root, vec![], vec![]),
            create_test_package("pkg-b", "1.0.0", "packages/b", &root, vec!["pkg-a"], vec![]),
            create_test_package(
                "pkg-c",
                "1.0.0",
                "packages/c",
                &root,
                vec!["pkg-a", "pkg-b"],
                vec![],
            ),
        ];

        let descriptor = MonorepoDescriptor::minimal(MonorepoKind::YarnWorkspaces, root, packages);

        let graph = descriptor.get_dependency_graph();

        // Check package A's dependents (B and C)
        let pkg_a_dependents = graph.get("pkg-a").unwrap();
        assert_eq!(pkg_a_dependents.len(), 2);
        assert!(pkg_a_dependents.iter().any(|pkg| pkg.name == "pkg-b"));
        assert!(pkg_a_dependents.iter().any(|pkg| pkg.name == "pkg-c"));

        // Check package B's dependents (C only)
        let pkg_b_dependents = graph.get("pkg-b").unwrap();
        assert_eq!(pkg_b_dependents.len(), 1);
        assert_eq!(pkg_b_dependents[0].name, "pkg-c");

        // Check package C's dependents (none)
        let pkg_c_dependents = graph.get("pkg-c").unwrap();
        assert_eq!(pkg_c_dependents.len(), 0);
    }

    #[test]
    fn test_find_dependencies_by_name() {
        let root = PathBuf::from("/fake/monorepo");
        let packages = vec![
            create_test_package("pkg-a", "1.0.0", "packages/a", &root, vec![], vec![]),
            create_test_package("pkg-b", "1.0.0", "packages/b", &root, vec!["pkg-a"], vec![]),
            create_test_package(
                "pkg-c",
                "1.0.0",
                "packages/c",
                &root,
                vec!["pkg-a"],
                vec!["pkg-b"],
            ),
        ];

        let descriptor = MonorepoDescriptor::minimal(MonorepoKind::YarnWorkspaces, root, packages);

        // Test dependencies of pkg-c (should include both pkg-a and pkg-b)
        let deps_c = descriptor.find_dependencies_by_name("pkg-c");
        assert_eq!(deps_c.len(), 2);
        assert!(deps_c.iter().any(|pkg| pkg.name == "pkg-a"));
        assert!(deps_c.iter().any(|pkg| pkg.name == "pkg-b"));

        // Test dependencies of pkg-b (should include only pkg-a)
        let deps_b = descriptor.find_dependencies_by_name("pkg-b");
        assert_eq!(deps_b.len(), 1);
        assert_eq!(deps_b[0].name, "pkg-a");

        // Test dependencies of pkg-a (should be empty)
        let deps_a = descriptor.find_dependencies_by_name("pkg-a");
        assert_eq!(deps_a.len(), 0);

        // Test non-existent package (should be empty)
        let deps_none = descriptor.find_dependencies_by_name("non-existent");
        assert_eq!(deps_none.len(), 0);
    }

    #[test]
    fn test_find_package_for_path() {
        let root = PathBuf::from("/fake/monorepo");
        let packages = vec![
            create_test_package("pkg-a", "1.0.0", "packages/a", &root, vec![], vec![]),
            create_test_package("pkg-b", "1.0.0", "packages/b", &root, vec![], vec![]),
        ];

        let descriptor =
            MonorepoDescriptor::minimal(MonorepoKind::YarnWorkspaces, root.clone(), packages);

        // Test absolute path in pkg-a
        let pkg_a_file = Path::new("/fake/monorepo/packages/a/src/index.js");
        let found_pkg = descriptor.find_package_for_path(pkg_a_file);
        assert!(found_pkg.is_some());
        assert_eq!(found_pkg.unwrap().name, "pkg-a");

        // Test relative path in pkg-b
        let pkg_b_file = Path::new("packages/b/src/component.js");
        let found_pkg = descriptor.find_package_for_path(pkg_b_file);
        assert!(found_pkg.is_some());
        assert_eq!(found_pkg.unwrap().name, "pkg-b");

        // Test path not in any package
        let outside_file = Path::new("/fake/monorepo/outside/file.js");
        let found_pkg = descriptor.find_package_for_path(outside_file);
        assert!(found_pkg.is_none());
    }

    #[test]
    fn test_package_manager_kind_lock_files() {
        assert_eq!(PackageManagerKind::Npm.lock_file(), "package-lock.json");
        assert_eq!(PackageManagerKind::Yarn.lock_file(), "yarn.lock");
        assert_eq!(PackageManagerKind::Pnpm.lock_file(), "pnpm-lock.yaml");
        assert_eq!(PackageManagerKind::Bun.lock_file(), "bun.lockb");
        assert_eq!(PackageManagerKind::Jsr.lock_file(), "jsr.json");
    }

    #[test]
    fn test_package_manager_kind_commands() {
        assert_eq!(PackageManagerKind::Npm.command(), "npm");
        assert_eq!(PackageManagerKind::Yarn.command(), "yarn");
        assert_eq!(PackageManagerKind::Pnpm.command(), "pnpm");
        assert_eq!(PackageManagerKind::Bun.command(), "bun");
        assert_eq!(PackageManagerKind::Jsr.command(), "jsr");
    }

    #[test]
    fn test_package_manager_creation() {
        let root = PathBuf::from("/project/root");
        let npm_manager = PackageManager::new(PackageManagerKind::Npm, &root);

        assert_eq!(npm_manager.kind(), PackageManagerKind::Npm);
        assert_eq!(npm_manager.root(), &root);
        assert_eq!(npm_manager.lock_file_path(), root.join("package-lock.json"));

        let yarn_manager = PackageManager::new(PackageManagerKind::Yarn, &root);
        assert_eq!(yarn_manager.kind(), PackageManagerKind::Yarn);
        assert_eq!(yarn_manager.lock_file_path(), root.join("yarn.lock"));
    }

    #[test]
    fn test_package_manager_detect() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();

        // Create npm lock file
        let npm_lock_path = temp_dir.path().join("package-lock.json");
        fs.write_file_string(&npm_lock_path, "{}").unwrap();

        // Detect should find npm
        let manager = PackageManager::detect(temp_dir.path()).unwrap();
        assert_eq!(manager.kind(), PackageManagerKind::Npm);
        assert_eq!(manager.root(), temp_dir.path());

        // Remove npm lock and add yarn lock
        fs.remove(&npm_lock_path).unwrap();
        fs.write_file_string(&temp_dir.path().join("yarn.lock"), "").unwrap();

        // Detect should find yarn
        let manager = PackageManager::detect(temp_dir.path()).unwrap();
        assert_eq!(manager.kind(), PackageManagerKind::Yarn);
    }

    #[test]
    fn test_package_manager_detect_failure() {
        let temp_dir = setup_test_dir();

        // No lock files, should fail
        let result = PackageManager::detect(temp_dir.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Monorepo(MonorepoError::ManagerNotFound)));
    }

    #[test]
    fn test_package_manager_lock_file_path() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();

        // Test npm with package-lock.json
        let npm_lock_path = temp_dir.path().join("package-lock.json");
        fs.write_file_string(&npm_lock_path, "{}").unwrap();

        let npm_manager = PackageManager::new(PackageManagerKind::Npm, temp_dir.path());
        assert_eq!(npm_manager.lock_file_path(), npm_lock_path);

        // Test npm with npm-shrinkwrap.json (alternative)
        fs.remove(&npm_lock_path).unwrap();
        let shrinkwrap_path = temp_dir.path().join("npm-shrinkwrap.json");
        fs.write_file_string(&shrinkwrap_path, "{}").unwrap();

        let npm_manager = PackageManager::new(PackageManagerKind::Npm, temp_dir.path());
        assert_eq!(npm_manager.lock_file_path(), shrinkwrap_path);
    }

    #[test]
    fn test_monorepo_error_display() {
        use crate::error::{FileSystemError, MonorepoError};
        use std::path::PathBuf;

        // Create separate FileSystemError instances for each test case
        let path = PathBuf::from("/fake/path");

        // Test each MonorepoError variant with a fresh FileSystemError
        let detection_error =
            MonorepoError::Detection { source: FileSystemError::NotFound { path: path.clone() } };
        assert!(detection_error.to_string().contains("Failed to detect monorepo type"));

        let parsing_error =
            MonorepoError::Parsing { source: FileSystemError::NotFound { path: path.clone() } };
        assert!(parsing_error.to_string().contains("Failed to parse monorepo descriptor"));

        let reading_error =
            MonorepoError::Reading { source: FileSystemError::NotFound { path: path.clone() } };
        assert!(reading_error.to_string().contains("Failed to read monorepo descriptor"));

        let writing_error = MonorepoError::Writing { source: FileSystemError::NotFound { path } };
        assert!(writing_error.to_string().contains("Failed to write monorepo descriptor"));

        let manager_not_found = MonorepoError::ManagerNotFound;
        assert_eq!(manager_not_found.to_string(), "Failed to find package manager");
    }

    #[test]
    fn test_monorepo_detector_new() {
        let detector = MonorepoDetector::new();
        // Simply test that we can create the detector
        assert!(detector.fs.exists(Path::new(".")));
    }

    #[test]
    fn test_monorepo_detector_with_filesystem() {
        let fs = FileSystemManager::new();
        let detector = MonorepoDetector::with_filesystem(fs);
        // Simply test that we can create the detector with custom filesystem
        assert!(detector.fs.exists(Path::new(".")));
    }

    #[test]
    fn test_is_monorepo_root() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();
        let detector = MonorepoDetector::new();

        // Create yarn lock file
        let yarn_lock_path = temp_dir.path().join("yarn.lock");
        fs.write_file_string(&yarn_lock_path, "").unwrap();

        // Should detect Yarn monorepo
        let result = detector.is_monorepo_root(temp_dir.path()).unwrap();
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), MonorepoKind::YarnWorkspaces));

        // Add package.json with workspaces field
        let package_json_path = temp_dir.path().join("package.json");
        fs.write_file_string(&package_json_path, r#"{"name":"test","workspaces":["packages/*"]}"#)
            .unwrap();

        // Remove yarn.lock and add pnpm-lock.yaml
        fs.remove(&yarn_lock_path).unwrap();
        fs.write_file_string(&temp_dir.path().join("pnpm-lock.yaml"), "").unwrap();

        // Should detect pnpm monorepo
        let result = detector.is_monorepo_root(temp_dir.path()).unwrap();
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), MonorepoKind::PnpmWorkspaces));

        // Remove all lock files - should no longer detect a monorepo
        fs.remove(&temp_dir.path().join("pnpm-lock.yaml")).unwrap();
        let result = detector.is_monorepo_root(temp_dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_find_monorepo_root() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();
        let detector = MonorepoDetector::new();

        // Create a nested directory structure
        let packages_dir = temp_dir.path().join("packages");
        let nested_dir = packages_dir.join("nested");
        fs.create_dir_all(&nested_dir).unwrap();

        // Add a yarn.lock file at the root
        fs.write_file_string(&temp_dir.path().join("yarn.lock"), "").unwrap();
        fs.write_file_string(
            &temp_dir.path().join("package.json"),
            r#"{"name":"test","workspaces":["packages/*"]}"#,
        )
        .unwrap();

        // Test from the root
        let result = detector.find_monorepo_root(temp_dir.path()).unwrap();
        assert!(result.is_some());
        let (root, kind) = result.unwrap();
        assert_eq!(root, temp_dir.path());
        assert!(matches!(kind, MonorepoKind::YarnWorkspaces));

        // Test from a nested directory
        let result = detector.find_monorepo_root(&nested_dir).unwrap();
        assert!(result.is_some());
        let (root, kind) = result.unwrap();
        assert_eq!(root, temp_dir.path());
        assert!(matches!(kind, MonorepoKind::YarnWorkspaces));
    }

    #[test]
    fn test_has_multiple_packages() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();
        let detector = MonorepoDetector::new();

        // Create a packages directory with two packages
        let packages_dir = temp_dir.path().join("packages");
        let pkg1_dir = packages_dir.join("pkg1");
        let pkg2_dir = packages_dir.join("pkg2");
        fs.create_dir_all(&pkg1_dir).unwrap();
        fs.create_dir_all(&pkg2_dir).unwrap();

        // Add package.json files
        fs.write_file_string(
            &pkg1_dir.join("package.json"),
            r#"{"name":"pkg1","version":"1.0.0"}"#,
        )
        .unwrap();
        fs.write_file_string(
            &pkg2_dir.join("package.json"),
            r#"{"name":"pkg2","version":"1.0.0"}"#,
        )
        .unwrap();

        // Should detect multiple packages
        assert!(detector.has_multiple_packages(temp_dir.path()));

        // Test with a non-monorepo directory
        let non_monorepo_dir = temp_dir.path().join("non-monorepo");
        fs.create_dir_all(&non_monorepo_dir).unwrap();
        fs.write_file_string(
            &non_monorepo_dir.join("package.json"),
            r#"{"name":"single","version":"1.0.0"}"#,
        )
        .unwrap();

        // Should not detect multiple packages
        assert!(!detector.has_multiple_packages(&non_monorepo_dir));
    }

    #[allow(clippy::panic)]
    #[test]
    fn test_detect_monorepo_error() {
        let temp_dir = setup_test_dir();
        let detector = MonorepoDetector::new();

        // Attempting to detect a monorepo in an empty directory should fail
        let result = detector.detect_monorepo(temp_dir.path());
        assert!(result.is_err());

        match result {
            Err(Error::Monorepo(MonorepoError::Detection { source: _ })) => {
                // This is the expected error
            }
            _ => panic!("Expected a MonorepoError::Detection error"),
        }
    }

    #[test]
    fn test_find_packages_from_patterns() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();
        let detector = MonorepoDetector::new();

        // Create package directories
        let packages_dir = temp_dir.path().join("packages");
        let apps_dir = temp_dir.path().join("apps");
        let pkg1 = packages_dir.join("pkg1");
        let app1 = apps_dir.join("app1");

        fs.create_dir_all(&pkg1).unwrap();
        fs.create_dir_all(&app1).unwrap();

        // Create package.json files
        fs.write_file_string(&pkg1.join("package.json"), r#"{"name":"pkg1","version":"1.0.0"}"#)
            .unwrap();
        fs.write_file_string(&app1.join("package.json"), r#"{"name":"app1","version":"1.0.0"}"#)
            .unwrap();

        // Test finding packages
        let patterns = vec!["packages/*".to_string(), "apps/*".to_string()];
        let packages = detector.find_packages_from_patterns(temp_dir.path(), &patterns).unwrap();

        // Should find both packages
        assert_eq!(packages.len(), 2);
        assert!(packages.iter().any(|p| p.name == "pkg1"));
        assert!(packages.iter().any(|p| p.name == "app1"));
    }

    #[test]
    fn test_find_packages_by_scanning() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();
        let detector = MonorepoDetector::new();

        // Create a root package.json
        fs.write_file_string(
            &temp_dir.path().join("package.json"),
            r#"{"name":"root","version":"1.0.0"}"#,
        )
        .unwrap();

        // Create nested package directories
        let deep_pkg_dir = temp_dir.path().join("deeply/nested/pkg");
        fs.create_dir_all(&deep_pkg_dir).unwrap();
        fs.write_file_string(
            &deep_pkg_dir.join("package.json"),
            r#"{"name":"nested-pkg","version":"1.0.0"}"#,
        )
        .unwrap();

        // Create a node_modules directory that should be ignored
        let node_modules_dir = temp_dir.path().join("node_modules/fake-pkg");
        fs.create_dir_all(&node_modules_dir).unwrap();
        fs.write_file_string(
            &node_modules_dir.join("package.json"),
            r#"{"name":"fake-pkg","version":"1.0.0"}"#,
        )
        .unwrap();

        // Test finding packages
        let packages = detector.find_packages_by_scanning(temp_dir.path()).unwrap();

        // Should find only the nested package, not the root or node_modules
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name, "nested-pkg");
    }

    #[test]
    fn test_read_package_json() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();
        let detector = MonorepoDetector::new();

        // Create a package with dependencies
        let pkg_dir = temp_dir.path().join("pkg");
        fs.create_dir_all(&pkg_dir).unwrap();

        let package_json_content = r#"{
            "name": "test-pkg",
            "version": "1.2.3",
            "dependencies": {
                "dep1": "^1.0.0",
                "dep2": "^2.0.0"
            },
            "devDependencies": {
                "dev-dep1": "^3.0.0"
            }
        }"#;

        fs.write_file_string(&pkg_dir.join("package.json"), package_json_content).unwrap();

        // Parse the package
        let package =
            detector.read_package_json(&pkg_dir.join("package.json"), temp_dir.path()).unwrap();

        // Verify package data
        assert_eq!(package.name, "test-pkg");
        assert_eq!(package.version, "1.2.3");
        assert_eq!(package.workspace_dependencies.len(), 2);
        assert!(package.workspace_dependencies.contains(&"dep1".to_string()));
        assert!(package.workspace_dependencies.contains(&"dep2".to_string()));
        assert_eq!(package.workspace_dev_dependencies.len(), 1);
        assert!(package.workspace_dev_dependencies.contains(&"dev-dep1".to_string()));
    }

    #[test]
    fn test_project_config() {
        // Test default implementation
        let default_config = ProjectConfig::default();
        assert!(default_config.root.is_none());
        assert!(default_config.detect_package_manager);
        assert!(default_config.validate_structure);
        assert!(default_config.detect_monorepo);

        // Test constructor
        let config = ProjectConfig::new();
        assert!(config.root.is_none());
        assert!(config.detect_package_manager);
        assert!(config.validate_structure);
        assert!(config.detect_monorepo);

        // Test builder pattern methods
        let custom_path = PathBuf::from("/test/project");
        let config = ProjectConfig::new()
            .with_root(&custom_path)
            .with_detect_package_manager(false)
            .with_validate_structure(false)
            .with_detect_monorepo(false);

        assert_eq!(config.root, Some(custom_path));
        assert!(!config.detect_package_manager);
        assert!(!config.validate_structure);
        assert!(!config.detect_monorepo);
    }

    #[test]
    fn test_project_validation_status() {
        // Test Valid variant
        let valid = ProjectValidationStatus::Valid;

        // Test Warning variant
        let warnings = vec!["Missing lock file".to_string(), "No node_modules".to_string()];
        let warning_status = ProjectValidationStatus::Warning(warnings.clone());

        // Test Error variant
        let errors = vec!["Invalid package.json".to_string()];
        let error_status = ProjectValidationStatus::Error(errors.clone());

        // Test NotValidated variant
        let not_validated = ProjectValidationStatus::NotValidated;

        // Test equality
        assert_eq!(valid, ProjectValidationStatus::Valid);
        assert_eq!(warning_status, ProjectValidationStatus::Warning(warnings));
        assert_eq!(error_status, ProjectValidationStatus::Error(errors));
        assert_eq!(not_validated, ProjectValidationStatus::NotValidated);

        // Test inequality
        assert_ne!(valid, warning_status);
        assert_ne!(valid, error_status);
        assert_ne!(valid, not_validated);
        assert_ne!(warning_status, error_status);
    }

    #[test]
    fn test_project_creation() {
        let temp_dir = setup_test_dir();
        let root = temp_dir.path().to_path_buf();
        let config = ProjectConfig::default();

        let project = GenericProject::new(&root, config);

        // Test project properties
        assert_eq!(project.root(), root.as_path());
        assert!(project.package_manager().is_none());
        assert!(matches!(project.validation_status(), &ProjectValidationStatus::NotValidated));
        assert!(project.package_json().is_none());
    }

    #[test]
    fn test_project_manager() {
        // Test constructor
        let manager = ProjectManager::new();

        // Test with_filesystem
        let fs = FileSystemManager::new();
        let custom_manager = ProjectManager::with_filesystem(fs.clone());

        // Test that both managers exist (basic existence check)
        // Create a temporary directory with package.json to test with
        let temp_dir = setup_test_dir();
        fs.write_file_string(&temp_dir.path().join("package.json"), r#"{"name": "test", "version": "1.0.0"}"#).unwrap();
        
        assert!(manager.is_valid_project(temp_dir.path()));
        assert!(custom_manager.is_valid_project(temp_dir.path()));
    }

    #[allow(clippy::panic)]
    #[test]
    fn test_detect_project() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();
        let project_manager = ProjectManager::new();

        // Create a valid package.json
        let package_json_content = r#"{
            "name": "test-project",
            "version": "2.1.0",
            "description": "A test project for validation",
            "license": "MIT",
            "dependencies": {
                "dep1": "^1.0.0"
            }
        }"#;

        let package_json_path = temp_dir.path().join("package.json");
        fs.write_file_string(&package_json_path, package_json_content).unwrap();

        // Create npm lock file
        let npm_lock_path = temp_dir.path().join("package-lock.json");
        fs.write_file_string(&npm_lock_path, "{}").unwrap();

        // Create node_modules directory for dependencies
        fs.create_dir_all(temp_dir.path().join("node_modules/dep1").as_path()).unwrap();

        // Test with default config
        let config = ProjectConfig::default();
        let result = project_manager.create_project(temp_dir.path(), &config);

        assert!(result.is_ok());
        let project_descriptor = result.unwrap();
        let project_info = project_descriptor.as_project_info();

        // Check project properties
        assert_eq!(project_info.root(), temp_dir.path());
        assert!(project_info.package_manager().is_some());
        assert_eq!(project_info.package_manager().unwrap().kind(), PackageManagerKind::Npm);

        // Package.json should be parsed
        let package_json = project_info.package_json();
        assert!(package_json.is_some());
        assert_eq!(package_json.unwrap().name, "test-project");

        // With proper setup, the project should be validated as Valid
        match project_info.validation_status() {
            ProjectValidationStatus::Valid => {}
            ProjectValidationStatus::Warning(warnings) => {
                panic!("Expected Valid status but got Warning with: {warnings:?}");
            }
            ProjectValidationStatus::Error(errors) => {
                panic!("Expected Valid status but got Error with: {errors:?}");
            }
            ProjectValidationStatus::NotValidated => {
                panic!("Expected Valid status but got NotValidated");
            }
        }

        // Test with disabled validation and package manager detection
        let config =
            ProjectConfig::new().with_validate_structure(false).with_detect_package_manager(false);

        let result = project_manager.create_project(temp_dir.path(), &config);
        assert!(result.is_ok());

        let project_descriptor = result.unwrap();
        let project_info = project_descriptor.as_project_info();
        assert_eq!(project_info.root(), temp_dir.path());
        assert!(project_info.package_manager().is_none()); // No package manager since detection was disabled
        assert!(matches!(project_info.validation_status(), &ProjectValidationStatus::NotValidated));
    }

    #[allow(clippy::panic)]
    #[test]
    fn test_validate_project() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();
        let project_manager = ProjectManager::new();

        // 1. Create a valid project
        let valid_dir = temp_dir.path().join("valid");
        fs.create_dir_all(&valid_dir).unwrap();

        let package_json_content = r#"{
            "name": "valid-project",
            "version": "2.1.0",
            "description": "A valid test project",
            "license": "MIT",
            "dependencies": {
                "dep1": "^1.0.0"
            }
        }"#;

        fs.write_file_string(&valid_dir.join("package.json"), package_json_content).unwrap();
        fs.write_file_string(&valid_dir.join("package-lock.json"), "{}").unwrap();
        // Create a proper node_modules structure
        fs.create_dir_all(&valid_dir.join("node_modules/dep1")).unwrap();

        let config = ProjectConfig::new()
            .with_validate_structure(true)
            .with_detect_package_manager(true);
        let result = project_manager.create_project(&valid_dir, &config);
        assert!(result.is_ok());
        
        let mut project_descriptor = result.unwrap();

        // Validate project
        let result = project_manager.validate_project(&mut project_descriptor);
        assert!(result.is_ok());

        // Check validation status - should be Valid now with proper node_modules
        match project_descriptor.as_project_info().validation_status() {
            ProjectValidationStatus::Valid => {}
            ProjectValidationStatus::Warning(warnings) => {
                panic!("Expected Valid status but got Warning with: {warnings:?}");
            }
            ProjectValidationStatus::Error(errors) => {
                panic!("Expected Valid status but got Error with: {errors:?}");
            }
            ProjectValidationStatus::NotValidated => {
                panic!("Expected Valid status but got NotValidated");
            }
        }

        // 2. Create a project with warnings (missing node_modules)
        let warning_dir = temp_dir.path().join("warning");
        fs.create_dir_all(&warning_dir).unwrap();

        fs.write_file_string(&warning_dir.join("package.json"), package_json_content).unwrap();
        fs.write_file_string(&warning_dir.join("package-lock.json"), "{}").unwrap();
        // Deliberately not creating node_modules

        let config = ProjectConfig::new()
            .with_validate_structure(true)
            .with_detect_package_manager(true);
        let result = project_manager.create_project(&warning_dir, &config);
        assert!(result.is_ok());
        
        let mut project_descriptor = result.unwrap();

        // Validate project
        let result = project_manager.validate_project(&mut project_descriptor);
        assert!(result.is_ok());

        match project_descriptor.as_project_info().validation_status() {
            ProjectValidationStatus::Warning(warnings) => {
                assert!(!warnings.is_empty());
                assert!(warnings.iter().any(|w| w.contains("node_modules")));
            }
            _ => panic!("Expected warning validation status"),
        }

        // 3. Create a project with errors (invalid package.json)
        let error_dir = temp_dir.path().join("error");
        fs.create_dir_all(&error_dir).unwrap();
        
        // Create invalid package.json that will cause parsing errors
        fs.write_file_string(&error_dir.join("package.json"), "{ invalid json").unwrap();

        let config = ProjectConfig::new()
            .with_validate_structure(true)
            .with_detect_package_manager(true);
        let result = project_manager.create_project(&error_dir, &config);
        
        // Should fail due to invalid JSON
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("package.json") || error_msg.contains("json"));
    }

    #[test]
    fn test_config_value_types() {
        // Test string value
        let string_val = ConfigValue::String("test".to_string());
        assert!(string_val.is_string());
        assert_eq!(string_val.as_string(), Some("test"));
        assert!(!string_val.is_integer());
        assert!(string_val.as_integer().is_none());

        // Test integer value
        let int_val = ConfigValue::Integer(42);
        assert!(int_val.is_integer());
        assert_eq!(int_val.as_integer(), Some(42));
        assert_eq!(int_val.as_float(), Some(42.0));

        // Test float value
        let float_val = ConfigValue::Float(f64::consts::PI);
        assert!(float_val.is_float());
        assert_eq!(float_val.as_float(), Some(f64::consts::PI));

        // Test boolean value
        let bool_val = ConfigValue::Boolean(true);
        assert!(bool_val.is_boolean());
        assert_eq!(bool_val.as_boolean(), Some(true));

        // Test array value
        let array_val = ConfigValue::Array(vec![ConfigValue::Integer(1), ConfigValue::Integer(2)]);
        assert!(array_val.is_array());
        let arr = array_val.as_array().unwrap();
        assert_eq!(arr.len(), 2);

        // Test map value
        let mut map = HashMap::new();
        map.insert("key".to_string(), ConfigValue::String("value".to_string()));
        let map_val = ConfigValue::Map(map);
        assert!(map_val.is_map());
        let map_ref = map_val.as_map().unwrap();
        assert_eq!(map_ref.len(), 1);

        // Test null value
        let null_val = ConfigValue::Null;
        assert!(null_val.is_null());
    }

    #[test]
    fn test_config_manager_basics() {
        let manager = ConfigManager::new();

        // Test set and get
        manager.set("test", ConfigValue::String("value".to_string()));
        let value = manager.get("test");
        assert!(value.is_some());
        assert_eq!(value.unwrap().as_string(), Some("value"));

        // Test remove
        let removed = manager.remove("test");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().as_string(), Some("value"));

        // Test after removal
        assert!(manager.get("test").is_none());
    }

    #[test]
    fn test_config_scopes() {
        let mut manager = ConfigManager::new();

        // Test setting paths for different scopes
        let global_path = PathBuf::from("/global/config.json");
        let user_path = PathBuf::from("/user/config.json");
        let project_path = PathBuf::from("/project/config.json");

        manager.set_path(ConfigScope::Global, &global_path);
        manager.set_path(ConfigScope::User, &user_path);
        manager.set_path(ConfigScope::Project, &project_path);

        // Test getting paths
        assert_eq!(manager.get_path(ConfigScope::Global), Some(&global_path));
        assert_eq!(manager.get_path(ConfigScope::User), Some(&user_path));
        assert_eq!(manager.get_path(ConfigScope::Project), Some(&project_path));
        assert_eq!(manager.get_path(ConfigScope::Runtime), None);
    }

    #[test]
    fn test_config_format_detection() {
        // Test format detection by extension
        assert_eq!(ConfigManager::detect_format(Path::new("config.json")), ConfigFormat::Json);
        assert_eq!(ConfigManager::detect_format(Path::new("config.toml")), ConfigFormat::Toml);
        assert_eq!(ConfigManager::detect_format(Path::new("config.yaml")), ConfigFormat::Yaml);
        assert_eq!(ConfigManager::detect_format(Path::new("config.yml")), ConfigFormat::Yaml);
        assert_eq!(
            ConfigManager::detect_format(Path::new("config.txt")),
            ConfigFormat::Json // Default for unknown extensions
        );
    }

    #[allow(clippy::panic)]
    #[test]
    fn test_config_parsing_serializing() {
        // JSON parsing
        let json_str = r#"{"name":"test","value":42,"enabled":true}"#;
        let json_result = ConfigManager::parse_config(json_str, ConfigFormat::Json);
        assert!(json_result.is_ok());

        let config_value = json_result.unwrap();
        if let ConfigValue::Map(map) = config_value {
            assert_eq!(map.get("name").unwrap().as_string(), Some("test"));
            assert_eq!(map.get("value").unwrap().as_integer(), Some(42));
            assert_eq!(map.get("enabled").unwrap().as_boolean(), Some(true));
        } else {
            panic!("Expected Map ConfigValue");
        }

        // Serialization
        let mut map = HashMap::new();
        map.insert("test".to_string(), ConfigValue::String("value".to_string()));
        let config = ConfigValue::Map(map);

        let json_str = ConfigManager::serialize_config(&config, ConfigFormat::Json);
        assert!(json_str.is_ok());
        assert!(json_str.unwrap().contains("test"));
    }

    // Helper function to create test packages
    fn create_test_package(
        name: &str,
        version: &str,
        location: &str,
        root: &Path,
        deps: Vec<&str>,
        dev_deps: Vec<&str>,
    ) -> WorkspacePackage {
        WorkspacePackage {
            name: name.to_string(),
            version: version.to_string(),
            location: PathBuf::from(location),
            absolute_path: root.join(location),
            workspace_dependencies: deps.into_iter().map(String::from).collect(),
            workspace_dev_dependencies: dev_deps.into_iter().map(String::from).collect(),
        }
    }
}
