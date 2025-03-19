use napi::{Error, Result as NapiResult, Status};
use std::fmt;
use ws_pkg::PkgError;
use ws_std::error::CommandError as WsCommandError;

/// JavaScript binding for ws_std::error::CommandError
#[napi]
#[derive(Debug)]
pub enum CommandError {
    /// Failed to run command
    Run,
    /// Failed to execute command
    Execution,
    /// Command failed with specific error
    Failure,
}

/// Convert a ws_std::error::CommandError to a NAPI Error
pub fn command_error_to_napi_error(error: WsCommandError) -> Error {
    let (status, message) = match error {
        WsCommandError::Run(io_err) => {
            (Status::GenericFailure, format!("Failed to run command: {}", io_err))
        }
        WsCommandError::Execution => {
            (Status::GenericFailure, "Failed to execute command".to_string())
        }
        WsCommandError::Failure { stdout, stderr } => (
            Status::GenericFailure,
            format!("Command failed with: stdout={}, stderr={}", stdout, stderr),
        ),
    };

    // Create error with the appropriate status and message
    let mut error = Error::new(status, message);
    error.status = status;

    error
}

/// Wraps a Result<T, CommandError> and converts it to a NAPI Result
pub fn handle_command_result<T>(
    result: std::result::Result<T, ws_std::error::CommandError>,
) -> NapiResult<T> {
    result.map_err(command_error_to_napi_error)
}

/// Custom JavaScript error codes for package errors
pub enum ErrorCode {
    VersionParseError,
    VersionReqParseError,
    PackageNotFound,
    DependencyNotFound,
    CircularDependency,
    NetworkError,
    RegistryError,
    AuthError,
    GenericError,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VersionParseError => write!(f, "VERSION_PARSE_ERROR"),
            Self::VersionReqParseError => write!(f, "VERSION_REQ_PARSE_ERROR"),
            Self::PackageNotFound => write!(f, "PACKAGE_NOT_FOUND"),
            Self::DependencyNotFound => write!(f, "DEPENDENCY_NOT_FOUND"),
            Self::CircularDependency => write!(f, "CIRCULAR_DEPENDENCY"),
            Self::NetworkError => write!(f, "NETWORK_ERROR"),
            Self::RegistryError => write!(f, "REGISTRY_ERROR"),
            Self::AuthError => write!(f, "AUTH_ERROR"),
            Self::GenericError => write!(f, "PKG_ERROR"),
        }
    }
}

/// Maps a ws_pkg::PkgError to a NAPI Error with appropriate status code and message
pub fn pkg_error_to_napi_error(error: PkgError) -> Error {
    let (status, message, code) = match error {
        // Version-related errors (usually user input problems)
        PkgError::VersionParseError { version, source } => (
            Status::InvalidArg,
            format!("Failed to parse version '{}': {}", version, source),
            ErrorCode::VersionParseError,
        ),

        PkgError::VersionReqParseError { requirement, source } => (
            Status::InvalidArg,
            format!("Failed to parse version requirement '{}': {}", requirement, source),
            ErrorCode::VersionReqParseError,
        ),

        // JSON parsing errors (usually file format problems)
        PkgError::JsonParseError { path, source } => {
            let msg = if let Some(path) = path {
                format!("Failed to parse JSON at '{}': {}", path.display(), source)
            } else {
                format!("Failed to parse JSON: {}", source)
            };
            (Status::GenericFailure, msg, ErrorCode::GenericError)
        }

        // IO errors (usually file system problems)
        PkgError::IoError { path, source } => {
            let msg = if let Some(path) = path {
                format!("IO error at '{}': {}", path.display(), source)
            } else {
                format!("IO error: {}", source)
            };
            (Status::GenericFailure, msg, ErrorCode::GenericError)
        }

        // Package/dependency not found errors
        PkgError::PackageNotFound { name } => (
            Status::GenericFailure,
            format!("Package not found: '{}'", name),
            ErrorCode::PackageNotFound,
        ),

        PkgError::DependencyNotFound { name, package } => (
            Status::GenericFailure,
            format!("Dependency '{}' not found in package '{}'", name, package),
            ErrorCode::DependencyNotFound,
        ),

        // Circular dependency errors (validation issues)
        PkgError::CircularDependency { path } => (
            Status::GenericFailure,
            format!("Circular dependency detected: {}", path.join(" -> ")),
            ErrorCode::CircularDependency,
        ),

        // Resolution errors
        PkgError::DependencyResolutionError => (
            Status::GenericFailure,
            "Error resolving dependencies".to_string(),
            ErrorCode::GenericError,
        ),

        // Network-related errors
        PkgError::NetworkError { url, source: _ } => (
            Status::GenericFailure,
            format!("Network error requesting '{}'", url), // Don't access source directly
            ErrorCode::NetworkError,
        ),

        PkgError::RegistryError { registry, message } => (
            Status::GenericFailure,
            format!("Registry error from '{}': {}", registry, message),
            ErrorCode::RegistryError,
        ),

        PkgError::AuthError { registry, message } => (
            Status::GenericFailure,
            format!("Authentication error for registry '{}': {}", registry, message),
            ErrorCode::AuthError,
        ),

        // Generic/other errors
        PkgError::Other { message } => (Status::GenericFailure, message, ErrorCode::GenericError),
    };

    // Create error with the appropriate status and message
    let mut error = Error::new(status, message);

    // Set the error code
    error.status = status;
    // Use the code string as a property that will be exposed to JavaScript
    // In real JS code, you would check error.code for these values
    error.reason = format!("{}: {}", code, error.reason);

    error
}

/// Wraps a Result<T, PkgError> and converts it to a NAPI Result
pub fn handle_pkg_result<T>(result: std::result::Result<T, PkgError>) -> NapiResult<T> {
    result.map_err(pkg_error_to_napi_error)
}

#[macro_export]
macro_rules! pkg_try {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return Err($crate::errors::pkg_error_to_napi_error(err)),
        }
    };
}

#[macro_export]
macro_rules! pkg_some {
    ($expr:expr, $msg:expr) => {
        match $expr {
            Some(val) => val,
            None => return Err(napi::Error::new(napi::Status::GenericFailure, $msg)),
        }
    };
}

#[macro_export]
macro_rules! cmd_try {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return Err(command_error_to_napi_error(err)),
        }
    };
}

#[cfg(test)]
mod errors_binding_tests {
    use super::*;
    use napi::{Result as NapiResult, Status};
    use std::path::PathBuf;

    #[test]
    fn test_version_parse_error() {
        let semver_err = "invalid".parse::<semver::Version>().unwrap_err();
        let pkg_error =
            PkgError::VersionParseError { version: "invalid".to_string(), source: semver_err };

        let napi_error = pkg_error_to_napi_error(pkg_error);
        assert_eq!(napi_error.status, Status::InvalidArg);
        assert!(napi_error.reason.contains("Failed to parse version"));
        assert!(napi_error.reason.contains("invalid"));
        assert!(napi_error.reason.contains("VERSION_PARSE_ERROR"));
    }

    #[test]
    fn test_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let pkg_error =
            PkgError::IoError { path: Some(PathBuf::from("/path/to/file.json")), source: io_err };

        let napi_error = pkg_error_to_napi_error(pkg_error);
        assert_eq!(napi_error.status, Status::GenericFailure);
        assert!(napi_error.reason.contains("IO error at '/path/to/file.json'"));
        assert!(napi_error.reason.contains("PKG_ERROR"));
    }

    #[test]
    fn test_package_not_found() {
        let pkg_error = PkgError::PackageNotFound { name: "test-pkg".to_string() };

        let napi_error = pkg_error_to_napi_error(pkg_error);
        assert_eq!(napi_error.status, Status::GenericFailure);
        assert!(napi_error.reason.contains("Package not found: 'test-pkg'"));
        assert!(napi_error.reason.contains("PACKAGE_NOT_FOUND"));
    }

    #[test]
    fn test_circular_dependency() {
        let pkg_error = PkgError::CircularDependency {
            path: vec!["pkg1".to_string(), "pkg2".to_string(), "pkg1".to_string()],
        };

        let napi_error = pkg_error_to_napi_error(pkg_error);
        assert_eq!(napi_error.status, Status::GenericFailure);
        assert!(napi_error.reason.contains("Circular dependency detected: pkg1 -> pkg2 -> pkg1"));
        assert!(napi_error.reason.contains("CIRCULAR_DEPENDENCY"));
    }

    #[test]
    fn test_network_error() {
        // Only run this test when the "reqwest" feature is enabled
        let blocking_client = reqwest::blocking::Client::new();

        // Force a connection error by trying to connect to an invalid address
        let resp = blocking_client.get("http://localhost:1").send();

        // This should fail and give us a reqwest::Error
        let req_error = resp.unwrap_err();

        let pkg_error =
            PkgError::NetworkError { url: "http://localhost:1".to_string(), source: req_error };

        let napi_error = pkg_error_to_napi_error(pkg_error);
        assert_eq!(napi_error.status, Status::GenericFailure);
        assert!(napi_error.reason.contains("Network error requesting 'http://localhost:1'"));
        assert!(napi_error.reason.contains("NETWORK_ERROR"));
    }

    #[test]
    fn test_handle_pkg_result() {
        // Test success case
        let success_result: std::result::Result<i32, PkgError> = Ok(42);
        let napi_success = handle_pkg_result(success_result);
        assert_eq!(napi_success.unwrap(), 42);

        // Test error case
        let error_result: std::result::Result<i32, PkgError> =
            Err(PkgError::PackageNotFound { name: "test-pkg".to_string() });
        let napi_error = handle_pkg_result(error_result);
        assert!(napi_error.is_err());
        let error = napi_error.unwrap_err();
        assert_eq!(error.status, Status::GenericFailure);
        assert!(error.reason.contains("Package not found"));
        assert!(error.reason.contains("PACKAGE_NOT_FOUND"));
    }

    #[test]
    fn test_pkg_try_macro() {
        // We'll need to test this in the context where it's actually used
        // A simplified example would be:
        fn test_fn(success: bool) -> NapiResult<i32> {
            let result: std::result::Result<i32, PkgError> = if success {
                Ok(42)
            } else {
                Err(PkgError::PackageNotFound { name: "test-pkg".to_string() })
            };

            Ok(pkg_try!(result))
        }

        // Test success case
        assert_eq!(test_fn(true).unwrap(), 42);

        // Test failure case
        let error = test_fn(false).unwrap_err();
        assert_eq!(error.status, Status::GenericFailure);
        assert!(error.reason.contains("Package not found"));
    }
}
