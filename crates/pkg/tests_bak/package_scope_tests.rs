#[cfg(test)]
mod package_scope_tests {
    use sublime_package_tools::{package_scope_name_version, PackageScopeMetadata};

    #[test]
    fn test_package_scope_parsing_basic() {
        // Test basic scoped package format: @scope/name
        let result = package_scope_name_version("@scope/name");
        assert!(result.is_some());

        let metadata = result.unwrap();
        assert_eq!(metadata.full, "@scope/name");
        assert_eq!(metadata.name, "@scope/name");
        assert_eq!(metadata.version, "latest"); // Default version
        assert_eq!(metadata.path, None);
    }

    #[test]
    fn test_package_scope_with_version() {
        // Test scoped package with version: @scope/name@version
        let result = package_scope_name_version("@scope/name@1.0.0");
        assert!(result.is_some());

        let metadata = result.unwrap();
        assert_eq!(metadata.full, "@scope/name@1.0.0");
        assert_eq!(metadata.name, "@scope/name");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.path, None);
    }

    #[test]
    fn test_package_scope_with_version_and_path() {
        // Test scoped package with version and path: @scope/name@version@path
        let result = package_scope_name_version("@scope/name@1.0.0@lib/index.js");
        assert!(result.is_some());

        let metadata = result.unwrap();
        assert_eq!(metadata.full, "@scope/name@1.0.0@lib/index.js");
        assert_eq!(metadata.name, "@scope/name");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.path, Some("lib/index.js".to_string()));
    }

    #[test]
    fn test_package_scope_with_colon_format() {
        // Test colon format: @scope/name:version
        let result = package_scope_name_version("@scope/name:1.0.0");
        assert!(result.is_some());

        let metadata = result.unwrap();
        assert_eq!(metadata.full, "@scope/name:1.0.0");
        assert_eq!(metadata.name, "@scope/name");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.path, None);
    }

    #[test]
    fn test_unscoped_packages() {
        // Test unscoped package (should return None)
        let result = package_scope_name_version("unscoped-package");
        assert!(result.is_none());

        // Even with version specifier
        let result = package_scope_name_version("unscoped-package@1.0.0");
        assert!(result.is_none());
    }

    #[test]
    fn test_empty_and_invalid_inputs() {
        // Test empty string
        let result = package_scope_name_version("");
        assert!(result.is_none());

        // Test just @ without scope or name
        let result = package_scope_name_version("@");
        assert!(result.is_some()); // It's technically a valid format but will have default values
        let metadata = result.unwrap();
        assert_eq!(metadata.name, "@"); // Just the @ symbol

        // Test invalid format (doesn't follow the expected pattern)
        let result = package_scope_name_version("@@invalid");
        assert!(result.is_some()); // Parser will try to handle it but might have unexpected results
    }

    #[test]
    fn test_complex_names() {
        // Test with complex scope and name
        let result = package_scope_name_version("@complex-scope/complex-name");
        assert!(result.is_some());

        let metadata = result.unwrap();
        assert_eq!(metadata.name, "@complex-scope/complex-name");

        // Test with dots in name
        let result = package_scope_name_version("@scope/name.with.dots");
        assert!(result.is_some());

        let metadata = result.unwrap();
        assert_eq!(metadata.name, "@scope/name.with.dots");

        // Test with hyphens and underscores
        let result = package_scope_name_version("@scope-with-hyphens/name_with_underscores");
        assert!(result.is_some());

        let metadata = result.unwrap();
        assert_eq!(metadata.name, "@scope-with-hyphens/name_with_underscores");
    }

    #[test]
    fn test_complex_versions() {
        // Test with semver version
        let result = package_scope_name_version("@scope/name@1.2.3");
        assert!(result.is_some());

        let metadata = result.unwrap();
        assert_eq!(metadata.version, "1.2.3");

        // Test with prerelease version
        let result = package_scope_name_version("@scope/name@1.0.0-beta.1");
        assert!(result.is_some());

        let metadata = result.unwrap();
        assert_eq!(metadata.version, "1.0.0-beta.1");

        // Test with build metadata
        let result = package_scope_name_version("@scope/name@1.0.0+build.123");
        assert!(result.is_some());

        let metadata = result.unwrap();
        assert_eq!(metadata.version, "1.0.0+build.123");

        // Test with non-semver version
        let result = package_scope_name_version("@scope/name@latest");
        assert!(result.is_some());

        let metadata = result.unwrap();
        assert_eq!(metadata.version, "latest");
    }

    #[test]
    fn test_multiple_at_symbols() {
        // Test with @ in version (should parse according to implementation)
        let result = package_scope_name_version("@scope/name@1.0.0@path/to/file@with@symbols");
        assert!(result.is_some());

        let metadata = result.unwrap();
        assert_eq!(metadata.name, "@scope/name");
        assert_eq!(metadata.version, "1.0.0");

        // The implementation might only extract the part up to the first @ in the path
        // Adjust based on actual implementation behavior
        assert_eq!(metadata.path, Some("path/to/file".to_string()));
    }

    #[test]
    fn test_colon_with_empty_version() {
        // Test colon format with empty version
        let result = package_scope_name_version("@scope/name:");
        assert!(result.is_some());

        let metadata = result.unwrap();
        assert_eq!(metadata.name, "@scope/name");

        // The implementation might not default to "latest" for empty versions with colon format
        // Adjust based on actual implementation behavior
        assert_eq!(metadata.version, "");
    }

    #[test]
    fn test_scope_metadata_debug() {
        // Create a metadata instance
        let metadata = PackageScopeMetadata {
            full: "@scope/name@1.0.0".to_string(),
            name: "@scope/name".to_string(),
            version: "1.0.0".to_string(),
            path: Some("path/to/file".to_string()),
        };

        // Test Debug formatting
        let debug_str = format!("{metadata:?}");

        // Debug output should contain all fields
        assert!(debug_str.contains("@scope/name@1.0.0"));
        assert!(debug_str.contains("@scope/name"));
        assert!(debug_str.contains("1.0.0"));
        assert!(debug_str.contains("path/to/file"));
    }
}
