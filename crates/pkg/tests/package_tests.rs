#[cfg(test)]
mod package_tests {
    use serde_json::json;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use ws_pkg::error::PkgError;
    use ws_pkg::registry::{DependencyUpdate, ResolutionResult};
    use ws_pkg::types::dependency::Dependency;
    use ws_pkg::types::package::{package_scope_name_version, Package, PackageInfo};

    // Helper function to create test dependencies
    fn create_test_dependency(name: &str, version: &str) -> Rc<RefCell<Dependency>> {
        Rc::new(RefCell::new(Dependency::new(name, version).unwrap()))
    }

    #[test]
    fn test_package_creation() {
        // Test basic package creation
        let pkg = Package::new("test-pkg", "1.0.0", None).unwrap();
        assert_eq!(pkg.name(), "test-pkg");
        assert_eq!(pkg.version_str(), "1.0.0");
        assert!(pkg.dependencies().is_empty());

        // Test with dependencies
        let deps = vec![
            create_test_dependency("dep1", "^1.0.0"),
            create_test_dependency("dep2", "^2.0.0"),
        ];
        let pkg = Package::new("test-pkg", "1.0.0", Some(deps)).unwrap();
        assert_eq!(pkg.dependencies().len(), 2);

        // Test invalid version
        let result = Package::new("test-pkg", "invalid", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_package_version_access() {
        let pkg = Package::new("test-pkg", "1.2.3", None).unwrap();

        // Test version string
        assert_eq!(pkg.version_str(), "1.2.3");

        // Test semver version
        let version = pkg.version();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_package_update() {
        // Test updating package version
        let pkg = Package::new("test-pkg", "1.0.0", None).unwrap();
        pkg.update_version("2.0.0").unwrap();
        assert_eq!(pkg.version_str(), "2.0.0");

        // Test updating dependency version
        let deps = vec![
            create_test_dependency("dep1", "^1.0.0"),
            create_test_dependency("dep2", "^2.0.0"),
        ];
        let pkg = Package::new("test-pkg", "1.0.0", Some(deps)).unwrap();
        pkg.update_dependency_version("dep1", "^1.1.0").unwrap();

        // Verify the update
        let dep1 = pkg.dependencies()[0].borrow();
        assert_eq!(dep1.version_str(), "^1.1.0");

        // Test updating nonexistent dependency
        let result = pkg.update_dependency_version("nonexistent", "^1.0.0");
        assert!(matches!(result,
            Err(PkgError::DependencyNotFound { name, package })
            if name == "nonexistent" && package == "test-pkg"
        ));
    }

    #[test]
    fn test_package_scope_name_version_parsing() {
        // Test normal scoped package
        let parsed = package_scope_name_version("@scope/pkg@1.0.0").unwrap();
        assert_eq!(parsed.name, "@scope/pkg");
        assert_eq!(parsed.version, "1.0.0");
        assert!(parsed.path.is_none());

        // Test with path
        let parsed = package_scope_name_version("@scope/pkg@1.0.0@/path/to/package").unwrap();
        assert_eq!(parsed.name, "@scope/pkg");
        assert_eq!(parsed.version, "1.0.0");
        assert_eq!(parsed.path, Some("/path/to/package".to_string()));

        // Test with colon version delimiter
        let parsed = package_scope_name_version("@scope/pkg:1.0.0").unwrap();
        assert_eq!(parsed.name, "@scope/pkg");
        assert_eq!(parsed.version, "1.0.0");

        // Test with no version specified
        let parsed = package_scope_name_version("@scope/pkg").unwrap();
        assert_eq!(parsed.name, "@scope/pkg");
        assert_eq!(parsed.version, "latest");

        // Test non-scoped package (should return None)
        let parsed = package_scope_name_version("regular-pkg@1.0.0");
        assert!(parsed.is_none());
    }

    #[test]
    fn test_package_info() {
        // Create package
        let pkg = Package::new("test-pkg", "1.0.0", None).unwrap();

        // Create package info
        let pkg_json = json!({
            "name": "test-pkg",
            "version": "1.0.0",
            "dependencies": {
                "dep1": "^1.0.0"
            },
            "devDependencies": {
                "dev-dep1": "^2.0.0"
            }
        });

        let pkg_info = PackageInfo::new(
            pkg,
            "path/to/package.json".to_string(),
            "path/to/package".to_string(),
            "relative/path".to_string(),
            pkg_json,
        );

        // Test version update
        pkg_info.update_version("2.0.0").unwrap();

        // Verify version updated in package and JSON without overlapping borrows
        {
            let pkg_version = pkg_info.package.borrow().version_str();
            assert_eq!(pkg_version, "2.0.0");
        } // Drop package borrow

        {
            let json_obj = pkg_info.pkg_json.borrow();
            assert_eq!(json_obj["version"].as_str().unwrap(), "2.0.0");
        } // Drop JSON borrow

        // Test dependency update with careful borrow management
        {
            pkg_info.update_dependency_version("dep1", "^1.1.0").unwrap();
        } // Any borrows from the update are dropped here

        // Now verify the update with a fresh borrow
        {
            let json_obj = pkg_info.pkg_json.borrow();
            assert_eq!(json_obj["dependencies"]["dep1"].as_str().unwrap(), "^1.1.0");
        }

        // Test dev dependency update, with careful borrow management
        {
            pkg_info.update_dependency_version("dev-dep1", "^2.1.0").unwrap();
        } // Any borrows from the update are dropped here

        // Verify dev dependency update with a fresh borrow
        {
            let json_obj = pkg_info.pkg_json.borrow();
            assert_eq!(json_obj["devDependencies"]["dev-dep1"].as_str().unwrap(), "^2.1.0");
        }
    }

    #[test]
    fn test_package_resolution_application() {
        // Create package with dependencies
        let deps = vec![create_test_dependency("dep1", "^1.0.0")];
        let pkg = Package::new("test-pkg", "1.0.0", Some(deps)).unwrap();

        // Create package info
        let pkg_json = json!({
            "name": "test-pkg",
            "version": "1.0.0",
            "dependencies": {
                "dep1": "^1.0.0"
            },
            "devDependencies": {
                "dev-dep1": "^2.0.0"
            }
        });

        let pkg_info = PackageInfo::new(
            pkg,
            "path/to/package.json".to_string(),
            "path/to/package".to_string(),
            "relative/path".to_string(),
            pkg_json,
        );

        // Create resolution result
        let mut resolved_versions = HashMap::new();
        resolved_versions.insert("dep1".to_string(), "1.1.0".to_string());
        resolved_versions.insert("dev-dep1".to_string(), "2.1.0".to_string());

        let updates_required = vec![
            DependencyUpdate {
                package_name: "test-pkg".to_string(),
                dependency_name: "dep1".to_string(),
                current_version: "^1.0.0".to_string(),
                new_version: "1.1.0".to_string(),
            },
            DependencyUpdate {
                package_name: "test-pkg".to_string(),
                dependency_name: "dev-dep1".to_string(),
                current_version: "^2.0.0".to_string(),
                new_version: "2.1.0".to_string(),
            },
        ];

        let resolution = ResolutionResult { resolved_versions, updates_required };

        // Apply resolution
        pkg_info.apply_dependency_resolution(&resolution).unwrap();

        // Verify dependencies updated in JSON
        {
            let json = pkg_info.pkg_json.borrow();
            assert_eq!(json["dependencies"]["dep1"].as_str().unwrap(), "1.1.0");
            assert_eq!(json["devDependencies"]["dev-dep1"].as_str().unwrap(), "2.1.0");
        }
    }
}
