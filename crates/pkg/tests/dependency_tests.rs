#[cfg(test)]
mod dependency_tests {
    use ws_pkg::types::dependency::Dependency;

    #[test]
    fn test_dependency_creation() {
        // Test basic dependency creation
        let dep = Dependency::new("test-pkg", "^1.0.0").unwrap();
        assert_eq!(dep.name(), "test-pkg");
        assert_eq!(dep.version_str(), "^1.0.0");

        // Test with version without prefix
        let dep = Dependency::new("test-pkg", "1.0.0").unwrap();
        assert_eq!(dep.version_str(), "^1.0.0");

        // Test with tilde prefix
        let dep = Dependency::new("test-pkg", "~1.0.0").unwrap();
        assert_eq!(dep.version_str(), "~1.0.0");

        // Test invalid version
        let result = Dependency::new("test-pkg", "invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_dependency_version_access() {
        // Test version access methods
        let dep = Dependency::new("test-pkg", "^1.0.0").unwrap();

        // Check name access
        assert_eq!(dep.name(), "test-pkg");

        // Check version requirement access
        let version_req = dep.version();
        assert!(version_req.matches(&semver::Version::parse("1.0.0").unwrap()));
        assert!(version_req.matches(&semver::Version::parse("1.1.0").unwrap()));
        assert!(!version_req.matches(&semver::Version::parse("2.0.0").unwrap()));

        // Check version string
        assert_eq!(dep.version_str(), "^1.0.0");
    }

    #[test]
    fn test_dependency_update() {
        // Test updating version
        let dep = Dependency::new("test-pkg", "^1.0.0").unwrap();
        dep.update_version("^2.0.0").unwrap();
        assert_eq!(dep.version_str(), "^2.0.0");

        // Test updating to version without prefix
        let dep = Dependency::new("test-pkg", "^1.0.0").unwrap();
        dep.update_version("2.0.0").unwrap();
        assert_eq!(dep.version_str(), "^2.0.0");

        // Test updating with tilde prefix
        let dep = Dependency::new("test-pkg", "^1.0.0").unwrap();
        dep.update_version("~2.0.0").unwrap();
        assert_eq!(dep.version_str(), "~2.0.0");

        // Test invalid update
        let dep = Dependency::new("test-pkg", "^1.0.0").unwrap();
        let result = dep.update_version("invalid");
        assert!(result.is_err());
    }
}
