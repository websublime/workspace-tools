//! Unit tests for error module

#[cfg(test)]
mod tests {
    use crate::error::*;
    use std::io;
    use std::error::Error as StdError;

    #[test]
    fn test_result_type_alias() {
        // Test the Result type alias exists and can be used
        #[allow(clippy::unnecessary_wraps)]
        fn success_function() -> Result<String> {
            Ok("success".to_string())
        }
        
        fn error_function() -> Result<String> {
            Err(Error::config("Test error"))
        }
        
        assert!(success_function().is_ok());
        assert!(error_function().is_err());
        
        let success_result = success_function().unwrap();
        assert_eq!(success_result, "success");
        
        let error_result = error_function().unwrap_err();
        // Test that error can be displayed
        let error_string = format!("{error_result}");
        assert!(!error_string.is_empty());
    }

    #[test]
    fn test_error_from_io() {
        // Test conversion from IO errors
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let monorepo_error: Error = io_error.into();
        
        // Test that error can be displayed
        let error_string = format!("{monorepo_error}");
        assert!(error_string.contains("IO") || error_string.contains("I/O"));
    }

    #[test]
    fn test_error_display() {
        // Test that errors can be displayed
        let error = Error::config("Test configuration error");
        
        let display_string = format!("{error}");
        let debug_string = format!("{error:?}");
        
        assert!(!display_string.is_empty());
        assert!(!debug_string.is_empty());
        assert!(display_string.contains("Configuration"));
    }

    #[test]
    fn test_error_constructors() {
        // Test the available error constructors
        let config_error = Error::config("Invalid configuration format");
        let analysis_error = Error::analysis("Analysis failed");
        let versioning_error = Error::versioning("Version conflict");
        let task_error = Error::task("Task execution failed");
        
        // Test that all constructors work
        assert!(format!("{config_error}").contains("Configuration"));
        assert!(format!("{analysis_error}").contains("Analysis"));
        assert!(format!("{versioning_error}").contains("Versioning"));
        assert!(format!("{task_error}").contains("Task"));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_str = r#"{"invalid": json}"#;
        let json_error = serde_json::from_str::<serde_json::Value>(json_str)
            .expect_err("Should fail to parse invalid JSON");
        let monorepo_error: Error = json_error.into();
        
        let error_string = format!("{monorepo_error}");
        assert!(error_string.contains("JSON"));
    }

    #[test]
    fn test_error_send_sync() {
        // Test that error types implement Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
        
        // Test that error can be passed across thread boundaries
        let error = Error::config("Test error");
        let error_string = format!("{error}");
        assert!(!error_string.is_empty());
    }

    #[test]
    fn test_error_source_chain() {
        // Test that errors can be chained
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let monorepo_error: Error = io_error.into();
        
        // Test that source error can be accessed
        assert!(StdError::source(&monorepo_error).is_some());
    }

    #[test]
    fn test_error_chain() {
        // Test error chaining
        let root_cause = io::Error::new(io::ErrorKind::NotFound, "Original error");
        let monorepo_error: Error = root_cause.into();
        
        // Test that the error can be chained
        let chained_error = Error::config(format!("Failed to load config: {monorepo_error}"));
        
        let error_string = format!("{chained_error}");
        assert!(error_string.contains("Configuration"));
        assert!(error_string.contains("Failed to load config"));
    }
}