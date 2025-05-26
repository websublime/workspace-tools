#[cfg(test)]
mod error_tests {
    use std::error::Error;
    use std::io;
    use sublime_package_tools::{
        DependencyResolutionError, PackageError, PackageRegistryError, RegistryError, VersionError,
    };

    #[test]
    fn test_dependency_resolution_error() {
        // Test creating different error types
        let parse_error =
            DependencyResolutionError::VersionParseError("Invalid version".to_string());
        let _incompatible = DependencyResolutionError::IncompatibleVersions {
            name: "react".to_string(),
            versions: vec!["16.0.0".to_string(), "17.0.0".to_string()],
            requirements: vec!["^16.0.0".to_string(), "^17.0.0".to_string()],
        };
        let _no_valid = DependencyResolutionError::NoValidVersion {
            name: "lodash".to_string(),
            requirements: vec!["^3.0.0".to_string(), "^4.0.0".to_string()],
        };
        let _not_found = DependencyResolutionError::DependencyNotFound {
            name: "missing".to_string(),
            package: "my-app".to_string(),
        };
        let _circular = DependencyResolutionError::CircularDependency {
            path: vec!["a".to_string(), "b".to_string(), "c".to_string(), "a".to_string()],
        };

        // Test error trait implementation
        assert_eq!(parse_error.to_string(), "Failed to parse version: Invalid version");

        // Test source
        assert!(parse_error.source().is_none());
    }

    #[test]
    fn test_package_error() {
        // Test creating package errors
        let io_error_file = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let io_error_json = io::Error::new(io::ErrorKind::NotFound, "File not found");

        let parse_failure = PackageError::PackageJsonParseFailure {
            path: "package.json".to_string(),
            error: serde_json::Error::io(io_error_file),
        };

        let io_failure = PackageError::PackageJsonIoFailure {
            path: "package.json".to_string(),
            error: io_error_json,
        };

        let between_failure = PackageError::PackageBetweenFailure("Failed to diff".to_string());

        let not_found = PackageError::PackageNotFound("missing-pkg".to_string());

        // Test error trait implementation
        assert!(parse_failure.to_string().contains("Failed to parse package json"));
        assert!(io_failure.to_string().contains("Failed to read/write package json"));
        assert!(between_failure.to_string().contains("Failed to diff package between"));
        assert!(not_found.to_string().contains("Failed to found package"));

        // Test source
        assert!(parse_failure.source().is_some());
        assert!(io_failure.source().is_some());
        assert!(between_failure.source().is_none());
        assert!(not_found.source().is_none());
    }

    #[test]
    fn test_version_error() {
        // Test creating version errors
        // Fix: Create semver error by parsing an invalid version
        let semver_error = semver::Version::parse("invalid").unwrap_err();

        let parse_error = VersionError::Parse {
            error: semver_error,
            message: "Failed to parse version".to_string(),
        };

        let invalid_error = VersionError::InvalidVersion("1.x".to_string());

        // Test error trait implementation
        assert!(parse_error.to_string().contains("Failed to parse version"));
        assert!(invalid_error.to_string().contains("Invalid version"));

        // Test source
        assert!(parse_error.source().is_some());
        assert!(invalid_error.source().is_none());
    }

    #[test]
    fn test_registry_error() {
        use std::io;

        // Test creating registry errors
        let url_not_supported = RegistryError::UrlNotSupported("https://example.com".to_string());
        let url_not_found = RegistryError::UrlNotFound("https://example.com".to_string());

        // Test with io::Error
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let npmrc_failure =
            RegistryError::NpmRcFailure { path: "/path/to/.npmrc".to_string(), error: io_error };

        // Test error messages
        assert!(url_not_supported.to_string().contains("Failed to support url"));
        assert!(url_not_found.to_string().contains("Registry not found"));
        assert!(npmrc_failure.to_string().contains("Failed to read npmrc file"));

        // Test from io::Error
        //let from_io = RegistryError::from(io::Error::from("File not found"));
        //assert!(matches!(from_io, RegistryError::NpmRcFailure { .. }));

        // Test AsRef<str> implementation
        assert_eq!(url_not_supported.as_ref(), "UrlNotSupported");
        assert_eq!(url_not_found.as_ref(), "UrlNotFound");
        assert_eq!(npmrc_failure.as_ref(), "NpmRcFailure");
    }

    #[test]
    fn test_package_registry_error() {
        // Test NotFound error
        let not_found = PackageRegistryError::NotFound {
            package_name: "react".to_string(),
            version: "17.0.0".to_string(),
        };

        // Test LockFailure
        let lock_failure = PackageRegistryError::LockFailure;

        // Test error messages
        assert!(not_found.to_string().contains("Failed to found package"));
        assert!(lock_failure.to_string().contains("Failed to acquire lock"));

        // Test AsRef<str> implementation
        assert_eq!(not_found.as_ref(), "NotFound");
        assert_eq!(lock_failure.as_ref(), "LockFailure");

        // We can't easily create a PoisonError in a test without causing actual panics
        // So we'll just verify that the LockFailure variant exists and has the expected string representation
        assert_eq!(
            PackageRegistryError::LockFailure.to_string(),
            "Failed to acquire lock on packages"
        );
    }

    #[test]
    fn test_clone_errors() {
        // Test cloning version errors
        let parse_ver_err = VersionError::Parse {
            // Create a proper semver::Error by trying to parse an invalid version
            error: semver::Version::parse("invalid").unwrap_err(),
            message: "Failed to parse".to_string(),
        };
        let invalid_ver_err = VersionError::InvalidVersion("Bad version".to_string());

        let cloned_parse = parse_ver_err.clone();
        let cloned_invalid = invalid_ver_err.clone();

        // Check strings match after cloning
        assert_eq!(parse_ver_err.to_string(), cloned_parse.to_string());
        assert_eq!(invalid_ver_err.to_string(), cloned_invalid.to_string());

        // Test cloning package errors
        let between_err = PackageError::PackageBetweenFailure("Cannot diff".to_string());
        let not_found_err = PackageError::PackageNotFound("missing-pkg".to_string());

        let cloned_between = between_err.clone();
        let cloned_not_found = not_found_err.clone();

        // Check strings match after cloning
        assert_eq!(between_err.to_string(), cloned_between.to_string());
        assert_eq!(not_found_err.to_string(), cloned_not_found.to_string());
    }

    #[test]
    fn test_as_ref_implementations() {
        // Test AsRef<str> for VersionError
        let ve1 = VersionError::Parse {
            error: semver::Version::parse("invalid").unwrap_err(),
            message: "Failed to parse".to_string(),
        };
        let ve2 = VersionError::InvalidVersion("Bad version".to_string());

        assert_eq!(ve1.as_ref(), "VersionErrorParse");
        assert_eq!(ve2.as_ref(), "VersionErrorInvalidVersion");

        // Test AsRef<str> for DependencyResolutionError
        let dre1 = DependencyResolutionError::VersionParseError("Invalid version".to_string());
        let dre2 = DependencyResolutionError::IncompatibleVersions {
            name: "pkg".to_string(),
            versions: vec!["1.0.0".to_string()],
            requirements: vec!["^1.0.0".to_string()],
        };
        let dre3 = DependencyResolutionError::NoValidVersion {
            name: "pkg".to_string(),
            requirements: vec!["^1.0.0".to_string()],
        };
        let dre4 = DependencyResolutionError::DependencyNotFound {
            name: "dep".to_string(),
            package: "pkg".to_string(),
        };
        let dre5 = DependencyResolutionError::CircularDependency {
            path: vec!["a".to_string(), "b".to_string()],
        };

        assert_eq!(dre1.as_ref(), "VersionParseError");
        assert_eq!(dre2.as_ref(), "IncompatibleVersions");
        assert_eq!(dre3.as_ref(), "NoValidVersion");
        assert_eq!(dre4.as_ref(), "DependencyNotFound");
        assert_eq!(dre5.as_ref(), "CircularDependency");

        // Test AsRef<str> for PackageError
        let pe1 = PackageError::PackageJsonParseFailure {
            path: "package.json".to_string(),
            error: serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            )),
        };
        let pe2 = PackageError::PackageJsonIoFailure {
            path: "package.json".to_string(),
            error: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
        };
        let pe3 = PackageError::PackageBetweenFailure("Cannot diff".to_string());
        let pe4 = PackageError::PackageNotFound("missing-pkg".to_string());

        assert_eq!(pe1.as_ref(), "PackageJsonParseFailure");
        assert_eq!(pe2.as_ref(), "PackageJsonIoFailure");
        assert_eq!(pe3.as_ref(), "PackageBetweenFailure");
        assert_eq!(pe4.as_ref(), "PackageNotFound");
    }
}
