#[cfg(test)]
mod dependency_core_tests {
    use sublime_package_tools::{Dependency, VersionError};

    #[test]
    fn test_dependency_creation() {
        // Test valid dependency creation
        let dep = Dependency::new("test-package", "^1.0.0");
        assert!(dep.is_ok());

        let dep = dep.unwrap();
        assert_eq!(dep.name(), "test-package");
        assert_eq!(dep.version().to_string(), "^1.0.0");

        // Test invalid version
        let dep = Dependency::new("test-package", "invalid-version");
        assert!(dep.is_err());
        assert!(matches!(dep.unwrap_err(), VersionError::Parse { .. }));

        // Test workspace marker (should be rejected)
        let dep = Dependency::new("test-package", "workspace:*");
        assert!(dep.is_err());
        assert!(matches!(dep.unwrap_err(), VersionError::InvalidVersion(_)));
    }

    #[test]
    fn test_version_requirements() {
        // Test various version requirements
        let test_cases = [
            ("^1.0.0", "1.0.0", true),
            ("^1.0.0", "1.1.0", true),
            ("^1.0.0", "2.0.0", false),
            ("~1.0.0", "1.0.1", true),
            ("~1.0.0", "1.1.0", false),
            (">=1.0.0", "1.0.0", true),
            (">=1.0.0", "0.9.0", false),
        ];

        for (req, version, should_match) in test_cases {
            let dep = Dependency::new("test", req).unwrap();
            let matches = dep.matches(version).unwrap();
            assert_eq!(
                matches,
                should_match,
                "{} {} match {}",
                req,
                if should_match { "should" } else { "should not" },
                version
            );
        }
    }

    #[test]
    fn test_update_version() {
        let dep = Dependency::new("test", "^1.0.0").unwrap();

        // Update to a new version requirement
        let result = dep.update_version("^2.0.0");
        assert!(result.is_ok());
        assert_eq!(dep.version().to_string(), "^2.0.0");

        // Try invalid version
        let result = dep.update_version("invalid");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VersionError::Parse { .. }));

        // Version should remain unchanged after failed update
        assert_eq!(dep.version().to_string(), "^2.0.0");
    }

    #[test]
    fn test_fixed_version() {
        let dep = Dependency::new("test", "^1.2.3").unwrap();
        let fixed = dep.fixed_version().unwrap();
        assert_eq!(fixed.to_string(), "1.2.3");

        // Test with a tilde version
        let dep = Dependency::new("test", "~2.3.4").unwrap();
        let fixed = dep.fixed_version().unwrap();
        assert_eq!(fixed.to_string(), "2.3.4");
    }

    #[test]
    fn test_rc_sharing() {
        // Test that Rc<RefCell> sharing works as expected
        let dep1 = Dependency::new("shared", "^1.0.0").unwrap();
        let dep2 = dep1.clone();

        // Update through one reference
        dep1.update_version("^2.0.0").unwrap();

        // Both should reflect the change
        assert_eq!(dep1.version().to_string(), "^2.0.0");
        assert_eq!(dep2.version().to_string(), "^2.0.0");

        // Update through the other reference
        dep2.update_version("^3.0.0").unwrap();

        // Both should reflect the change again
        assert_eq!(dep1.version().to_string(), "^3.0.0");
        assert_eq!(dep2.version().to_string(), "^3.0.0");
    }

    #[test]
    fn test_compare_versions() {
        let dep = Dependency::new("test", "^1.0.0").unwrap();

        // Compare with a higher version
        let result = dep.compare_versions("2.0.0").unwrap();
        assert_eq!(result, std::cmp::Ordering::Less);

        // Compare with the same version
        let result = dep.compare_versions("1.0.0").unwrap();
        assert_eq!(result, std::cmp::Ordering::Equal);

        // Compare with a lower version
        let result = dep.compare_versions("0.9.0").unwrap();
        assert_eq!(result, std::cmp::Ordering::Greater);

        // Compare with an invalid version
        let result = dep.compare_versions("invalid");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VersionError::Parse { .. }));
    }
}
