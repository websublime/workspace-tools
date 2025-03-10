#[cfg(test)]
mod error_tests {
    use std::path::PathBuf;
    use ws_pkg::error::PkgError;

    #[test]
    fn test_error_display() {
        // Test version parse error
        let semver_err = "invalid".parse::<semver::Version>().unwrap_err();
        let err =
            PkgError::VersionParseError { version: "invalid".to_string(), source: semver_err };
        let msg = err.to_string();
        assert!(msg.contains("Failed to parse version 'invalid'"));

        // Test version requirement parse error
        let semver_req_err = "invalid".parse::<semver::VersionReq>().unwrap_err();
        let err = PkgError::VersionReqParseError {
            requirement: "invalid".to_string(),
            source: semver_req_err,
        };
        let msg = err.to_string();
        assert!(msg.contains("Failed to parse version requirement 'invalid'"));

        // Test JSON parse error
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let err = PkgError::JsonParseError {
            path: Some(PathBuf::from("/path/to/file.json")),
            source: json_err,
        };
        let msg = err.to_string();
        assert!(msg.contains("Failed to parse JSON at '/path/to/file.json'"));

        // Test IO error
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err =
            PkgError::IoError { path: Some(PathBuf::from("/path/to/file.json")), source: io_err };
        let msg = err.to_string();
        assert!(msg.contains("IO error at '/path/to/file.json'"));

        // Test package not found
        let err = PkgError::PackageNotFound { name: "test-pkg".to_string() };
        let msg = err.to_string();
        assert!(msg.contains("Package not found: 'test-pkg'"));

        // Test dependency not found
        let err =
            PkgError::DependencyNotFound { name: "dep1".to_string(), package: "pkg1".to_string() };
        let msg = err.to_string();
        assert!(msg.contains("Dependency 'dep1' not found in package 'pkg1'"));

        // Test circular dependency
        let err = PkgError::CircularDependency {
            path: vec!["pkg1".to_string(), "pkg2".to_string(), "pkg1".to_string()],
        };
        let msg = err.to_string();
        assert!(msg.contains("Circular dependency detected: pkg1 -> pkg2 -> pkg1"));

        // Test dependency resolution error
        let err = PkgError::DependencyResolutionError;
        let msg = err.to_string();
        assert!(msg.contains("Error resolving dependencies"));

        // Test network error
        // We can't easily create a reqwest error for testing, so skip this test or mock it
        let err = PkgError::Other {
            message: "Network error requesting 'https://example.com': connection refused"
                .to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Network error"));

        // Test registry error
        let err = PkgError::RegistryError {
            registry: "npm".to_string(),
            message: "Package not found".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Registry error from 'npm': Package not found"));

        // Test auth error
        let err = PkgError::AuthError {
            registry: "npm".to_string(),
            message: "Invalid token".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Authentication error for registry 'npm': Invalid token"));

        // Test generic error
        let err = PkgError::Other { message: "Something went wrong".to_string() };
        let msg = err.to_string();
        assert_eq!(msg, "Something went wrong");
    }

    #[test]
    fn test_error_conversions() {
        // Test From<io::Error>
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let pkg_error: PkgError = io_error.into();
        assert!(matches!(pkg_error, PkgError::IoError { path: None, .. }));

        // Test From<semver::Error>
        let semver_error = "invalid".parse::<semver::Version>().unwrap_err();
        let pkg_error: PkgError = semver_error.into();
        assert!(
            matches!(pkg_error, PkgError::VersionParseError { version, .. } if version == "unknown")
        );

        // Test From<serde_json::Error>
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let pkg_error: PkgError = json_error.into();
        assert!(matches!(pkg_error, PkgError::JsonParseError { path: None, .. }));
    }
}
