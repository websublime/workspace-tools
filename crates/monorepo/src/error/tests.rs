//! Comprehensive tests for the error module
//!
//! This module provides complete test coverage for all error handling functionality,
//! including error creation, conversion, display messages, and integration with base crates.

#[cfg(test)]
mod tests {
    use crate::error::{Error, Result};
    use std::io;
    use sublime_package_tools::errors::{
        DependencyResolutionError, PackageRegistryError, RegistryError, VersionError,
    };

    /// Helper function to create a test IO error
    fn create_test_io_error() -> io::Error {
        io::Error::new(io::ErrorKind::NotFound, "Test file not found")
    }

    /// Helper function to create a test JSON error
    fn create_test_json_error() -> serde_json::Error {
        serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err()
    }

    /// Helper function to create a test version error
    fn create_test_version_error() -> VersionError {
        VersionError::InvalidVersion("1.0.invalid".to_string())
    }

    /// Helper function to create a test dependency resolution error
    fn create_test_dependency_error() -> DependencyResolutionError {
        DependencyResolutionError::CircularDependency {
            path: vec!["package-a".to_string(), "package-b".to_string()],
        }
    }

    /// Helper function to create a test package registry error
    fn create_test_registry_error() -> PackageRegistryError {
        PackageRegistryError::NotFound {
            package_name: "test-package".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    /// Helper function to create a test registry management error
    fn create_test_registry_management_error() -> RegistryError {
        RegistryError::UrlNotFound("https://not-found-registry.com".to_string())
    }

    #[test]
    fn test_error_display_messages() {
        // Test all error variants display correctly
        let git_error = Error::git("Repository not found");
        assert!(git_error.to_string().contains("Git error: Repository not found"));

        let config_error = Error::config("Invalid configuration");
        assert_eq!(config_error.to_string(), "Configuration error: Invalid configuration");

        let analysis_error = Error::analysis("Analysis failed");
        assert_eq!(analysis_error.to_string(), "Analysis error: Analysis failed");

        let versioning_error = Error::versioning("Version bump failed");
        assert_eq!(versioning_error.to_string(), "Versioning error: Version bump failed");

        let task_error = Error::task("Task execution failed");
        assert_eq!(task_error.to_string(), "Task execution error: Task execution failed");

        let changeset_error = Error::changeset("Changeset creation failed");
        assert_eq!(changeset_error.to_string(), "Changeset error: Changeset creation failed");

        let hook_error = Error::hook("Pre-commit hook failed");
        assert_eq!(hook_error.to_string(), "Hook error: Pre-commit hook failed");

        let changelog_error = Error::changelog("Changelog generation failed");
        assert_eq!(changelog_error.to_string(), "Changelog error: Changelog generation failed");

        let plugin_error = Error::plugin("Plugin initialization failed");
        assert_eq!(plugin_error.to_string(), "Plugin error: Plugin initialization failed");

        let workflow_error = Error::workflow("Workflow execution failed");
        assert_eq!(workflow_error.to_string(), "Workflow error: Workflow execution failed");

        let project_init_error = Error::project_init("Project initialization failed");
        assert_eq!(
            project_init_error.to_string(),
            "Project initialization error: Project initialization failed"
        );

        let generic_error = Error::generic("Generic error occurred");
        assert_eq!(generic_error.to_string(), "Generic error occurred");
    }

    #[test]
    fn test_error_constructor_methods() {
        // Test all constructor methods work correctly
        let config_error = Error::config("config issue");
        assert!(matches!(config_error, Error::Config(_)));

        let analysis_error = Error::analysis("analysis issue");
        assert!(matches!(analysis_error, Error::Analysis(_)));

        let versioning_error = Error::versioning("versioning issue");
        assert!(matches!(versioning_error, Error::Versioning(_)));

        let task_error = Error::task("task issue");
        assert!(matches!(task_error, Error::Task(_)));

        let changeset_error = Error::changeset("changeset issue");
        assert!(matches!(changeset_error, Error::Changeset(_)));

        let package_error = Error::package("package issue");
        assert!(matches!(package_error, Error::Package(_)));

        let hook_error = Error::hook("hook issue");
        assert!(matches!(hook_error, Error::Hook(_)));

        let changelog_error = Error::changelog("changelog issue");
        assert!(matches!(changelog_error, Error::Changelog(_)));

        let plugin_error = Error::plugin("plugin issue");
        assert!(matches!(plugin_error, Error::Plugin(_)));

        let workflow_error = Error::workflow("workflow issue");
        assert!(matches!(workflow_error, Error::Workflow(_)));

        let project_init_error = Error::project_init("init issue");
        assert!(matches!(project_init_error, Error::ProjectInit(_)));

        let generic_error = Error::generic("generic issue");
        assert!(matches!(generic_error, Error::Generic(_)));
    }

    #[test]
    fn test_specialized_error_constructors() {
        // Test package_not_found
        let not_found_error = Error::package_not_found("my-package");
        assert_eq!(
            not_found_error.to_string(),
            "Package tools error: Package not found: my-package"
        );
        assert!(matches!(not_found_error, Error::Package(_)));

        // Test filesystem error
        let fs_error = Error::filesystem("Failed to read directory");
        assert!(matches!(fs_error, Error::Standard(_)));
        // The filesystem error wraps the message in a standard tools error format
        let error_string = fs_error.to_string();
        assert!(
            error_string.contains("Standard tools error")
                || error_string.contains("Failed to read directory")
        );

        // Test git error
        let git_error = Error::git("Failed to clone repository");
        assert!(matches!(git_error, Error::Generic(_)));
        assert!(git_error.to_string().contains("Git error: Failed to clone repository"));

        // Test dependency error
        let dep_error = Error::dependency("Circular dependency detected");
        assert!(matches!(dep_error, Error::Package(_)));
        assert!(dep_error.to_string().contains("Dependency error: Circular dependency detected"));

        // Test config validation error
        let validation_error = Error::config_validation("Missing required field");
        assert!(matches!(validation_error, Error::Config(_)));
        assert!(validation_error
            .to_string()
            .contains("Config validation error: Missing required field"));
    }

    #[test]
    fn test_error_conversions_from_std_types() {
        // Test conversion from IO error
        let io_error = create_test_io_error();
        let converted: Error = io_error.into();
        assert!(matches!(converted, Error::Io(_)));
        assert!(converted.to_string().contains("Test file not found"));

        // Test conversion from JSON error
        let json_error = create_test_json_error();
        let converted: Error = json_error.into();
        assert!(matches!(converted, Error::Json(_)));

        // Test conversion from String
        let string_error: Error = "String error message".to_string().into();
        assert!(matches!(string_error, Error::Generic(_)));
        assert_eq!(string_error.to_string(), "String error message");

        // Test conversion from &str
        let str_error: Error = "String slice error".into();
        assert!(matches!(str_error, Error::Generic(_)));
        assert_eq!(str_error.to_string(), "String slice error");
    }

    #[test]
    fn test_error_conversions_from_package_tools() {
        // Test conversion from VersionError
        let version_error = create_test_version_error();
        let converted: Error = version_error.into();
        assert!(matches!(converted, Error::Version(_)));
        assert!(converted.to_string().contains("1.0.invalid"));

        // Test conversion from DependencyResolutionError
        let dep_error = create_test_dependency_error();
        let converted: Error = dep_error.into();
        assert!(matches!(converted, Error::DependencyResolution(_)));
        assert!(converted.to_string().contains("package-a"));

        // Test conversion from PackageRegistryError
        let registry_error = create_test_registry_error();
        let converted: Error = registry_error.into();
        assert!(matches!(converted, Error::PackageRegistry(_)));
        assert!(converted.to_string().contains("test-package"));

        // Test conversion from RegistryError
        let reg_mgmt_error = create_test_registry_management_error();
        let converted: Error = reg_mgmt_error.into();
        assert!(matches!(converted, Error::Registry(_)));
        assert!(converted.to_string().contains("not-found-registry.com"));
    }

    #[allow(clippy::unnecessary_literal_unwrap)]
    #[test]
    fn test_error_result_type() {
        // Test successful result
        let success: Result<String> = Ok("success".to_string());
        assert!(success.is_ok());
        assert_eq!(success.unwrap(), "success");

        // Test error result
        let error: Result<String> = Err(Error::config("test error"));
        assert!(error.is_err());
        assert!(error.unwrap_err().to_string().contains("test error"));
    }

    #[allow(clippy::unnecessary_wraps)]
    #[test]
    fn test_error_chaining_and_propagation() -> Result<()> {
        // Test that errors can be properly chained through Result types
        fn inner_function() -> Result<String> {
            Err(Error::task("Inner task failed"))
        }

        fn outer_function() -> Result<String> {
            let _result = inner_function()?;
            Ok("success".to_string())
        }

        let result = outer_function();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, Error::Task(_)));
        assert!(error.to_string().contains("Inner task failed"));

        Ok(())
    }

    #[test]
    fn test_error_debug_output() {
        let error = Error::config("debug test");
        let debug_output = format!("{error:?}");
        assert!(debug_output.contains("Config"));
        assert!(debug_output.contains("debug test"));
    }

    #[allow(clippy::items_after_statements)]
    #[test]
    fn test_error_source_chain() {
        // Test that source errors are properly preserved
        let io_error = create_test_io_error();
        let converted_error = Error::from(io_error);

        // The source should be available through std::error::Error trait
        use std::error::Error as StdError;
        assert!(converted_error.source().is_some());
    }

    #[allow(clippy::panic)]
    #[test]
    fn test_error_equality_and_matching() {
        // Test pattern matching on error variants
        let config_error = Error::config("test");
        match config_error {
            Error::Config(msg) => assert_eq!(msg, "test"),
            _ => panic!("Expected Config error"),
        }

        let analysis_error = Error::analysis("analysis test");
        match analysis_error {
            Error::Analysis(msg) => assert_eq!(msg, "analysis test"),
            _ => panic!("Expected Analysis error"),
        }

        let generic_error = Error::generic("generic test");
        match generic_error {
            Error::Generic(msg) => assert_eq!(msg, "generic test"),
            _ => panic!("Expected Generic error"),
        }
    }

    #[test]
    fn test_error_size_and_memory_efficiency() {
        // Test that Error enum size is reasonable
        let error_size = std::mem::size_of::<Error>();

        // Error should not be excessively large (reasonable threshold)
        assert!(error_size <= 128, "Error size is {error_size} bytes, which may be too large");

        // Result<()> should also be reasonable
        let result_size = std::mem::size_of::<Result<()>>();
        assert!(result_size <= 136, "Result size is {result_size} bytes, which may be too large");
    }

    #[test]
    fn test_error_with_different_input_types() {
        // Test constructor methods accept various input types
        let owned_string = String::from("owned string error");
        let error1 = Error::config(owned_string);
        assert!(matches!(error1, Error::Config(_)));

        let string_slice = "string slice error";
        let error2 = Error::analysis(string_slice);
        assert!(matches!(error2, Error::Analysis(_)));

        let format_string = format!("formatted error {}", 42);
        let error3 = Error::task(format_string);
        assert!(matches!(error3, Error::Task(_)));
        assert!(error3.to_string().contains("42"));
    }

    #[test]
    fn test_integration_with_base_crates() {
        // Test that we can properly handle errors from different base crates

        // This simulates receiving errors from sublime_standard_tools
        let std_error = sublime_standard_tools::error::Error::operation("file not found");
        let converted: Error = std_error.into();
        assert!(matches!(converted, Error::Standard(_)));

        // Test filesystem error helper
        let fs_error = Error::filesystem("permission denied");
        assert!(matches!(fs_error, Error::Standard(_)));
        // Check if the error contains either the standard tools wrapper or the original message
        let error_string = fs_error.to_string();
        assert!(
            error_string.contains("Standard tools error")
                || error_string.contains("permission denied")
        );
    }

    #[test]
    fn test_error_context_preservation() {
        // Test that error context is preserved through conversions
        let original_message = "detailed error context with specific information";

        let config_error = Error::config(original_message);
        assert!(config_error.to_string().contains(original_message));

        let validation_error = Error::config_validation(original_message);
        assert!(validation_error.to_string().contains(original_message));
        assert!(validation_error.to_string().contains("Config validation error"));
    }

    #[test]
    fn test_must_use_attribute() {
        // Test that package_not_found has #[must_use] attribute
        // This is a compile-time check, but we can verify the error is created correctly
        let error = Error::package_not_found("unused-package");
        assert!(matches!(error, Error::Package(_)));
        assert!(error.to_string().contains("unused-package"));
    }

    #[test]
    fn test_complex_error_scenarios() {
        // Test complex real-world error scenarios

        // Scenario 1: Package discovery fails due to filesystem error
        let discovery_error = Error::filesystem("Cannot read package.json");
        let error_string = discovery_error.to_string();
        assert!(
            error_string.contains("Standard tools error")
                || error_string.contains("Cannot read package.json")
        );

        // Scenario 2: Version resolution fails with dependency cycle
        let cycle_error = Error::dependency("Circular dependency: A -> B -> A");
        assert!(cycle_error.to_string().contains("Circular dependency"));

        // Scenario 3: Configuration validation fails
        let config_error = Error::config_validation("Missing workspace patterns");
        assert!(config_error.to_string().contains("Config validation error"));
        assert!(config_error.to_string().contains("Missing workspace patterns"));

        // Scenario 4: Task execution fails with detailed context
        let task_error = Error::task("Build failed: TypeScript errors in src/index.ts:42");
        assert!(task_error.to_string().contains("TypeScript errors"));
        assert!(task_error.to_string().contains("src/index.ts:42"));
    }

    #[test]
    fn test_error_formatting_consistency() {
        // Test that all error types follow consistent formatting patterns
        let errors = vec![
            Error::config("test"),
            Error::analysis("test"),
            Error::versioning("test"),
            Error::task("test"),
            Error::changeset("test"),
            Error::hook("test"),
            Error::changelog("test"),
            Error::plugin("test"),
            Error::workflow("test"),
            Error::project_init("test"),
        ];

        for error in errors {
            let error_str = error.to_string();
            // All errors should end with the message "test"
            assert!(error_str.ends_with("test"));
            // All errors should contain "error" (case insensitive)
            assert!(error_str.to_lowercase().contains("error"));
        }
    }

    #[test]
    fn test_error_with_empty_and_special_strings() {
        // Test edge cases with empty and special strings
        let empty_error = Error::config("");
        assert_eq!(empty_error.to_string(), "Configuration error: ");

        let whitespace_error = Error::analysis("   ");
        assert_eq!(whitespace_error.to_string(), "Analysis error:    ");

        let newline_error = Error::task("line1\nline2");
        assert!(newline_error.to_string().contains("line1\nline2"));

        let unicode_error = Error::changeset("测试错误 🚀");
        assert!(unicode_error.to_string().contains("测试错误 🚀"));
    }
}
