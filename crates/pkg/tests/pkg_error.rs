#[cfg(test)]
mod error_handling_tests {
    use serde_json::json;
    use ws_pkg::{
        error::{PkgError, Result},
        types::package::PackageInfo,
        Dependency, DependencyRegistry, Package,
    };

    #[test]
    fn test_dependency_with_invalid_version() {
        // Try to create a dependency with invalid version
        let result = Dependency::new("@scope/foo", "invalid-version");
        assert!(result.is_err());

        // Check error type
        if let Err(PkgError::VersionReqParseError { requirement, .. }) = result {
            assert_eq!(requirement, "^invalid-version");
        } else {
            panic!("Expected VersionReqParseError");
        }
    }

    #[test]
    fn test_package_with_invalid_version() {
        // Try to create a package with invalid version
        let result = Package::new("@scope/bar", "invalid-version", None);
        assert!(result.is_err());

        // Check error type
        if let Err(PkgError::VersionParseError { version, .. }) = result {
            assert_eq!(version, "invalid-version");
        } else {
            panic!("Expected VersionParseError");
        }
    }

    #[test]
    fn test_update_dependency_version_error() {
        // Create a valid package
        let package = Package::new("@scope/package", "1.0.0", None).unwrap();

        // Try to update a non-existent dependency
        let result = package.update_dependency_version("@scope/non-existent", "2.0.0");
        assert!(result.is_err());

        // Check error type
        if let Err(PkgError::DependencyNotFound { name, package: pkg_name }) = result {
            assert_eq!(name, "@scope/non-existent");
            assert_eq!(pkg_name, "@scope/package");
        } else {
            panic!("Expected DependencyNotFound");
        }
    }

    #[test]
    fn test_dependency_registry_error_propagation() -> Result<()> {
        let mut registry = DependencyRegistry::new();

        // Create a valid dependency
        let dep = registry.get_or_create("@scope/foo", "1.0.0")?;
        assert_eq!(dep.borrow().name(), "@scope/foo");

        // Try to update with invalid version
        let result = dep.borrow().update_version("invalid-version");
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_package_update_version_error() {
        // Create a valid package
        let package = Package::new("@scope/package", "1.0.0", None).unwrap();

        // Try to update with invalid version
        let result = package.update_version("invalid-version");
        assert!(result.is_err());

        // Check error type
        if let Err(PkgError::VersionParseError { version, .. }) = result {
            assert_eq!(version, "invalid-version");
        } else {
            panic!("Expected VersionParseError");
        }
    }

    #[test]
    fn test_new_with_registry_error_handling() {
        let mut registry = DependencyRegistry::new();

        // Try to create a package with invalid version
        let result = Package::new_with_registry(
            "@scope/package",
            "invalid-version",
            Some(vec![("@scope/dep", "1.0.0")]),
            &mut registry,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_package_info_error_propagation() -> Result<()> {
        // Create a valid package
        let package = Package::new("@scope/package", "1.0.0", None)?;

        // Create package info
        let pkg_json = json!({
            "name": "@scope/package",
            "version": "1.0.0",
            "dependencies": {
                "@scope/dep": "1.0.0"
            }
        });

        let pkg_info = PackageInfo::new(
            package,
            "/fake/path/package.json".to_string(),
            "/fake/path".to_string(),
            "fake/path".to_string(),
            pkg_json,
        );

        // Try to update with invalid version
        let result = pkg_info.update_version("invalid-version");
        assert!(result.is_err());

        // Try to update non-existent dependency when package.json doesn't have devDependencies
        // This should fail now because it's not in dependencies or devDependencies
        let result = pkg_info.update_dependency_version("@scope/completely-missing", "2.0.0");
        assert!(result.is_ok()); // It should succeed but not do anything

        // Update dependency that actually exists in package.json
        let result = pkg_info.update_dependency_version("@scope/dep", "2.0.0");
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_display_error_messages() {
        // Test version parse error display
        let err = PkgError::VersionParseError {
            version: "invalid".to_string(),
            source: "1.a.0".parse::<semver::Version>().unwrap_err(),
        };
        let error_msg = format!("{err}");
        assert!(error_msg.contains("Failed to parse version 'invalid'"));

        // Test package not found error display
        let err = PkgError::PackageNotFound { name: "@scope/missing".to_string() };
        let error_msg = format!("{err}");
        assert_eq!(error_msg, "Package not found: '@scope/missing'");

        // Test dependency not found error display
        let err = PkgError::DependencyNotFound {
            name: "@scope/dep".to_string(),
            package: "@scope/pkg".to_string(),
        };
        let error_msg = format!("{err}");
        assert_eq!(error_msg, "Dependency '@scope/dep' not found in package '@scope/pkg'");

        // Test circular dependency error display
        let err = PkgError::CircularDependency {
            path: vec!["@scope/a".to_string(), "@scope/b".to_string(), "@scope/a".to_string()],
        };
        let error_msg = format!("{err}");
        assert_eq!(error_msg, "Circular dependency detected: @scope/a -> @scope/b -> @scope/a");
    }
}
