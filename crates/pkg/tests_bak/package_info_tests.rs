#[cfg(test)]
mod package_info_tests {
    use serde_json::json;
    use std::cell::RefCell;
    use std::fs;
    use std::path::Path;
    use std::rc::Rc;
    use tempfile::TempDir;

    use sublime_package_tools::{
        Dependency, DependencyResolutionError, Package, PackageError, PackageInfo,
        ResolutionResult, VersionError,
    };

    // Helper to create a test PackageInfo
    fn create_test_package_info() -> (PackageInfo, TempDir) {
        // Create a temporary directory for test files
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        // Create package.json content
        let pkg_json = json!({
            "name": "test-package",
            "version": "1.0.0",
            "dependencies": {
                "dep1": "^1.0.0",
                "dep2": "^2.0.0"
            },
            "devDependencies": {
                "dev-dep1": "^1.0.0"
            }
        });

        // Write package.json to temp directory
        let pkg_json_path = temp_path.join("package.json");
        fs::write(&pkg_json_path, pkg_json.to_string()).expect("Failed to write package.json");

        // Create dependencies
        let dep1 = Rc::new(RefCell::new(Dependency::new("dep1", "^1.0.0").unwrap()));
        let dep2 = Rc::new(RefCell::new(Dependency::new("dep2", "^2.0.0").unwrap()));

        // Create package
        let package =
            Package::new("test-package", "1.0.0", Some(vec![Rc::clone(&dep1), Rc::clone(&dep2)]))
                .unwrap();

        // Create PackageInfo
        let pkg_info = PackageInfo::new(
            package,
            pkg_json_path.to_string_lossy().to_string(),
            temp_path.to_string_lossy().to_string(),
            "test-package".to_string(),
            pkg_json,
        );

        (pkg_info, temp_dir)
    }

    #[test]
    fn test_package_info_creation() {
        let (pkg_info, _temp_dir) = create_test_package_info();

        // Verify package info
        let package = pkg_info.package.borrow();
        assert_eq!(package.name(), "test-package");
        assert_eq!(package.version_str(), "1.0.0");
        assert_eq!(package.dependencies().len(), 2);

        // Verify JSON content
        let json = pkg_info.pkg_json.borrow();
        assert_eq!(json["name"], "test-package");
        assert_eq!(json["version"], "1.0.0");
        assert!(json["dependencies"].is_object());
        assert!(json["devDependencies"].is_object());
    }

    #[test]
    fn test_update_package_version() {
        let (pkg_info, _temp_dir) = create_test_package_info();

        // Update version
        let result = pkg_info.update_version("2.0.0");
        assert!(result.is_ok());

        // Verify package version is updated
        assert_eq!(pkg_info.package.borrow().version_str(), "2.0.0");

        // Verify JSON is updated
        assert_eq!(pkg_info.pkg_json.borrow()["version"], "2.0.0");

        // Test invalid version
        let result = pkg_info.update_version("invalid");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VersionError::Parse { .. }));

        // Version should remain unchanged after error
        assert_eq!(pkg_info.package.borrow().version_str(), "2.0.0");
        assert_eq!(pkg_info.pkg_json.borrow()["version"], "2.0.0");
    }

    #[test]
    fn test_update_dependency_version() {
        let (pkg_info, _temp_dir) = create_test_package_info();

        // Update regular dependency
        let result = pkg_info.update_dependency_version("dep1", "^1.5.0");
        assert!(result.is_ok());

        // Verify JSON updated for the regular dependency
        {
            let json = pkg_info.pkg_json.borrow();
            assert_eq!(json["dependencies"]["dep1"], "^1.5.0");
        }

        // Update dev dependency
        let result = pkg_info.update_dependency_version("dev-dep1", "^1.5.0");
        assert!(result.is_ok());

        // Verify JSON updated for the dev dependency
        {
            let json = pkg_info.pkg_json.borrow();
            assert_eq!(json["devDependencies"]["dev-dep1"], "^1.5.0");
        }

        // Test non-existent dependency
        let result = pkg_info.update_dependency_version("non-existent", "^1.0.0");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DependencyResolutionError::DependencyNotFound { .. }
        ));
    }

    #[test]
    fn test_apply_dependency_resolution() {
        let (pkg_info, _temp_dir) = create_test_package_info();

        // Create resolution result
        let mut resolved_versions = std::collections::HashMap::new();
        resolved_versions.insert("dep1".to_string(), "^1.5.0".to_string());
        resolved_versions.insert("dep2".to_string(), "^2.3.0".to_string());
        resolved_versions.insert("dev-dep1".to_string(), "^1.2.0".to_string());

        let resolution = ResolutionResult { resolved_versions, updates_required: Vec::new() };

        // Apply resolution
        let result = pkg_info.apply_dependency_resolution(&resolution);
        assert!(result.is_ok());

        // Verify JSON updated
        let json = pkg_info.pkg_json.borrow();
        assert_eq!(json["dependencies"]["dep1"], "^1.5.0");
        assert_eq!(json["dependencies"]["dep2"], "^2.3.0");
        assert_eq!(json["devDependencies"]["dev-dep1"], "^1.2.0");
    }

    #[test]
    fn test_write_package_json() {
        let (pkg_info, _temp_dir) = create_test_package_info();

        // Make some changes
        pkg_info.update_version("2.0.0").unwrap();
        pkg_info.update_dependency_version("dep1", "^1.5.0").unwrap();

        // Write changes to disk
        let result = pkg_info.write_package_json();
        assert!(result.is_ok());

        // Verify file was written with changes
        let file_path = Path::new(&pkg_info.package_json_path);
        let content = fs::read_to_string(file_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["version"], "2.0.0");
        assert_eq!(parsed["dependencies"]["dep1"], "^1.5.0");
        assert_eq!(parsed["dependencies"]["dep2"], "^2.0.0");
    }

    #[test]
    fn test_error_handling() {
        let (mut pkg_info, _temp_dir) = create_test_package_info();

        // Test with invalid path
        let invalid_path = "/path/that/does/not/exist/package.json";
        pkg_info.package_json_path = invalid_path.to_string();

        // Should fail to write
        let result = pkg_info.write_package_json();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PackageError::PackageJsonIoFailure { .. }));
    }
}
