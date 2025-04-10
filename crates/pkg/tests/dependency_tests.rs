#[cfg(test)]
mod dependency_tests {
    use std::cmp::Ordering;

    use sublime_package_tools::Dependency;

    #[test]
    fn test_dependency_creation() {
        let dep = Dependency::new("react", "^17.0.0");
        assert!(dep.is_ok());

        let invalid_dep = Dependency::new("react", "not-a-version");
        assert!(invalid_dep.is_err());
    }

    #[test]
    fn test_dependency_getters() {
        let dep = Dependency::new("react", "^17.0.0").unwrap();
        assert_eq!(dep.name(), "react");
        assert_eq!(dep.version().to_string(), "^17.0.0");
    }

    #[test]
    fn test_dependency_to_string() {
        let dep = Dependency::new("react", "^17.0.0").unwrap();
        assert_eq!(dep.to_string(), "react@^17.0.0");
    }

    #[test]
    fn test_fixed_version() {
        // Without operators
        let dep = Dependency::new("react", "17.0.0").unwrap();
        assert_eq!(dep.fixed_version().unwrap().to_string(), "17.0.0");

        // With caret
        let dep_caret = Dependency::new("react", "^17.0.0").unwrap();
        assert_eq!(dep_caret.fixed_version().unwrap().to_string(), "17.0.0");

        // With tilde
        let dep_tilde = Dependency::new("react", "~17.0.0").unwrap();
        assert_eq!(dep_tilde.fixed_version().unwrap().to_string(), "17.0.0");
    }

    #[test]
    fn test_compare_versions() {
        let dep = Dependency::new("react", "^17.0.0").unwrap();

        // Equal
        assert_eq!(dep.compare_versions("17.0.0").unwrap(), Ordering::Equal);

        // Later version
        assert_eq!(dep.compare_versions("17.0.1").unwrap(), Ordering::Less);

        // Earlier version
        assert_eq!(dep.compare_versions("16.0.0").unwrap(), Ordering::Greater);

        // Invalid version
        assert!(dep.compare_versions("not-a-version").is_err());
    }

    #[test]
    fn test_update_version() {
        let dep = Dependency::new("react", "^17.0.0").unwrap();
        assert!(dep.update_version("^18.0.0").is_ok());
        assert_eq!(dep.version().to_string(), "^18.0.0");

        assert!(dep.update_version("not-a-version").is_err());
    }

    #[test]
    fn test_version_matching() {
        // Caret ranges
        let dep_caret = Dependency::new("react", "^17.0.0").unwrap();
        assert!(dep_caret.matches("17.0.0").unwrap());
        assert!(dep_caret.matches("17.0.1").unwrap());
        assert!(dep_caret.matches("17.1.0").unwrap());
        assert!(!dep_caret.matches("18.0.0").unwrap());

        // Tilde ranges
        let dep_tilde = Dependency::new("react", "~17.0.0").unwrap();
        assert!(dep_tilde.matches("17.0.0").unwrap());
        assert!(dep_tilde.matches("17.0.9").unwrap());
        assert!(!dep_tilde.matches("17.1.0").unwrap());

        // Range with greater than
        let dep_gt = Dependency::new("react", ">17.0.0").unwrap();
        assert!(!dep_gt.matches("17.0.0").unwrap());
        assert!(dep_gt.matches("17.0.1").unwrap());

        // Range with greater than or equal
        let dep_gte = Dependency::new("react", ">=17.0.0").unwrap();
        assert!(dep_gte.matches("17.0.0").unwrap());
        assert!(dep_gte.matches("17.0.1").unwrap());

        // Invalid version to match
        assert!(dep_caret.matches("not-a-version").is_err());
    }
}
