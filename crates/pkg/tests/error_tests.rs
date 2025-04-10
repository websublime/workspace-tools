#[cfg(test)]
mod error_tests {
    use std::error::Error;
    use std::io;
    use sublime_package_tools::{DependencyResolutionError, PackageError, VersionError};

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
        let semver_error = semver::Error::InvalidVersion("invalid".to_string());

        let parse_error = VersionError::Parse {
            error: semver_error.clone(),
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
}
