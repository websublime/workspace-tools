//! CLI error type implementation.
//!
//! This module contains the main `CliError` enum and all its implementations,
//! including helper methods, conversions, and trait implementations.

use crate::error::exit_codes::ExitCode;

/// Result type alias for CLI operations.
///
/// This is the standard result type used throughout the CLI, wrapping
/// the `CliError` enum for error cases.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::error::Result;
///
/// fn example() -> Result<String> {
///     Ok("success".to_string())
/// }
/// ```
pub type Result<T> = std::result::Result<T, CliError>;

/// Main error type for CLI operations.
///
/// This enum represents all possible error conditions that can occur
/// during CLI execution. Each variant provides context-specific information
/// and maps to an appropriate exit code following the sysexits standard.
///
/// # Error Categories
///
/// - `Configuration`: Config file issues, invalid settings
/// - `Validation`: Invalid arguments, invalid state
/// - `Execution`: Command failures, operation failures
/// - `Git`: Repository errors, git operation failures
/// - `Package`: Package.json issues, dependency problems
/// - `Io`: File system errors, permission issues
/// - `Network`: Registry unreachable, download failures
/// - `User`: Invalid input, cancelled operations
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::error::CliError;
///
/// let error = CliError::configuration("Invalid config");
/// assert_eq!(error.exit_code(), 78);
/// assert_eq!(error.as_ref(), "CliError::Configuration");
/// ```
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// Configuration-related errors (invalid, not found, parsing failed).
    ///
    /// Exit code: 78 (EX_CONFIG)
    ///
    /// # Examples
    ///
    /// - Configuration file not found
    /// - Invalid configuration format
    /// - Configuration validation failed
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Validation errors (invalid arguments, invalid state).
    ///
    /// Exit code: 65 (EX_DATAERR)
    ///
    /// # Examples
    ///
    /// - Invalid command arguments
    /// - Invalid version format
    /// - Invalid changeset data
    #[error("Validation error: {0}")]
    Validation(String),

    /// Execution errors (command failed, operation failed).
    ///
    /// Exit code: 70 (EX_SOFTWARE)
    ///
    /// # Examples
    ///
    /// - Command execution failed
    /// - Operation could not be completed
    /// - Internal processing error
    #[error("Execution error: {0}")]
    Execution(String),

    /// Git-related errors (repository not found, git operation failed).
    ///
    /// Exit code: 70 (EX_SOFTWARE)
    ///
    /// # Examples
    ///
    /// - Repository not found
    /// - Git command failed
    /// - Branch operation failed
    #[error("Git error: {0}")]
    Git(String),

    /// Package-related errors (package not found, package.json invalid).
    ///
    /// Exit code: 65 (EX_DATAERR)
    ///
    /// # Examples
    ///
    /// - Package.json not found
    /// - Invalid package.json format
    /// - Dependency resolution failed
    #[error("Package error: {0}")]
    Package(String),

    /// I/O errors (file not found, permission denied).
    ///
    /// Exit code: 74 (EX_IOERR)
    ///
    /// # Examples
    ///
    /// - File not found
    /// - Permission denied
    /// - Disk full
    #[error("I/O error: {0}")]
    Io(String),

    /// Network errors (registry unreachable, download failed).
    ///
    /// Exit code: 69 (EX_UNAVAILABLE)
    ///
    /// # Examples
    ///
    /// - npm registry unreachable
    /// - Download timeout
    /// - Network connection lost
    #[error("Network error: {0}")]
    Network(String),

    /// User-caused errors (invalid input, cancelled operation).
    ///
    /// Exit code: 64 (EX_USAGE)
    ///
    /// # Examples
    ///
    /// - Invalid user input
    /// - Operation cancelled by user
    /// - Interactive prompt declined
    #[error("User error: {0}")]
    User(String),
}

impl CliError {
    /// Creates a new Configuration error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::CliError;
    ///
    /// let error = CliError::configuration("File not found");
    /// assert_eq!(error.exit_code(), 78);
    /// ```
    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }

    /// Creates a new Validation error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::CliError;
    ///
    /// let error = CliError::validation("Invalid version format");
    /// assert_eq!(error.exit_code(), 65);
    /// ```
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Creates a new Execution error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::CliError;
    ///
    /// let error = CliError::execution("Command failed");
    /// assert_eq!(error.exit_code(), 70);
    /// ```
    pub fn execution(msg: impl Into<String>) -> Self {
        Self::Execution(msg.into())
    }

    /// Creates a new Git error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::CliError;
    ///
    /// let error = CliError::git("Repository not found");
    /// assert_eq!(error.exit_code(), 70);
    /// ```
    pub fn git(msg: impl Into<String>) -> Self {
        Self::Git(msg.into())
    }

    /// Creates a new Package error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::CliError;
    ///
    /// let error = CliError::package("package.json not found");
    /// assert_eq!(error.exit_code(), 65);
    /// ```
    pub fn package(msg: impl Into<String>) -> Self {
        Self::Package(msg.into())
    }

    /// Creates a new I/O error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::CliError;
    ///
    /// let error = CliError::io("Permission denied");
    /// assert_eq!(error.exit_code(), 74);
    /// ```
    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io(msg.into())
    }

    /// Creates a new Network error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::CliError;
    ///
    /// let error = CliError::network("Registry unreachable");
    /// assert_eq!(error.exit_code(), 69);
    /// ```
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    /// Creates a new User error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::CliError;
    ///
    /// let error = CliError::user("Operation cancelled");
    /// assert_eq!(error.exit_code(), 64);
    /// ```
    pub fn user(msg: impl Into<String>) -> Self {
        Self::User(msg.into())
    }

    /// Returns the exit code for this error following sysexits convention.
    ///
    /// # Exit Codes
    ///
    /// - `Configuration`: 78 (EX_CONFIG)
    /// - `Validation`: 65 (EX_DATAERR)
    /// - `Package`: 65 (EX_DATAERR)
    /// - `Execution`: 70 (EX_SOFTWARE)
    /// - `Git`: 70 (EX_SOFTWARE)
    /// - `Io`: 74 (EX_IOERR)
    /// - `Network`: 69 (EX_UNAVAILABLE)
    /// - `User`: 64 (EX_USAGE)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::CliError;
    ///
    /// let error = CliError::configuration("Invalid config");
    /// assert_eq!(error.exit_code(), 78);
    ///
    /// let error = CliError::network("Registry down");
    /// assert_eq!(error.exit_code(), 69);
    /// ```
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Configuration(_) => ExitCode::CONFIG,
            Self::Validation(_) | Self::Package(_) => ExitCode::DATAERR,
            Self::Execution(_) | Self::Git(_) => ExitCode::SOFTWARE,
            Self::Io(_) => ExitCode::IOERR,
            Self::Network(_) => ExitCode::UNAVAILABLE,
            Self::User(_) => ExitCode::USAGE,
        }
    }

    /// Returns a user-friendly error message.
    ///
    /// This message is displayed to the user and provides clear
    /// information about what went wrong and how to fix it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::CliError;
    ///
    /// let error = CliError::configuration("File not found");
    /// let message = error.user_message();
    /// assert!(message.contains("Configuration error"));
    /// ```
    pub fn user_message(&self) -> String {
        match self {
            Self::Configuration(msg) => format!("Configuration error: {msg}"),
            Self::Validation(msg) => format!("Validation error: {msg}"),
            Self::Execution(msg) => format!("Execution error: {msg}"),
            Self::Git(msg) => format!("Git error: {msg}"),
            Self::Package(msg) => format!("Package error: {msg}"),
            Self::Io(msg) => format!("I/O error: {msg}"),
            Self::Network(msg) => format!("Network error: {msg}"),
            Self::User(msg) => format!("Error: {msg}"),
        }
    }

    /// Returns the error category as a string identifier.
    ///
    /// This is useful for programmatic error handling and logging.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::CliError;
    ///
    /// let error = CliError::configuration("Test");
    /// assert_eq!(error.kind(), "Configuration");
    ///
    /// let error = CliError::network("Test");
    /// assert_eq!(error.kind(), "Network");
    /// ```
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Configuration(_) => "Configuration",
            Self::Validation(_) => "Validation",
            Self::Execution(_) => "Execution",
            Self::Git(_) => "Git",
            Self::Package(_) => "Package",
            Self::Io(_) => "Io",
            Self::Network(_) => "Network",
            Self::User(_) => "User",
        }
    }
}

impl AsRef<str> for CliError {
    /// Returns a string identifier for the error type.
    ///
    /// This is useful for error matching and categorization in logs
    /// or error reporting systems.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::CliError;
    ///
    /// let error = CliError::configuration("test");
    /// assert_eq!(error.as_ref(), "CliError::Configuration");
    ///
    /// let error = CliError::git("test");
    /// assert_eq!(error.as_ref(), "CliError::Git");
    /// ```
    fn as_ref(&self) -> &str {
        match self {
            Self::Configuration(_) => "CliError::Configuration",
            Self::Validation(_) => "CliError::Validation",
            Self::Execution(_) => "CliError::Execution",
            Self::Git(_) => "CliError::Git",
            Self::Package(_) => "CliError::Package",
            Self::Io(_) => "CliError::Io",
            Self::Network(_) => "CliError::Network",
            Self::User(_) => "CliError::User",
        }
    }
}

// Error conversions from internal crates

impl From<sublime_git_tools::RepoError> for CliError {
    /// Converts a git repository error to a CLI error.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_cli_tools::error::CliError;
    /// use sublime_git_tools::RepoError;
    ///
    /// // Example conversion from RepoError to CliError
    /// let git_error = RepoError::OpenRepoFailure(git2::Error::from_str("test"));
    /// let cli_error: CliError = git_error.into();
    /// assert_eq!(cli_error.exit_code(), 70);
    /// ```
    fn from(error: sublime_git_tools::RepoError) -> Self {
        Self::Git(format!("{error}"))
    }
}

impl From<sublime_pkg_tools::error::Error> for CliError {
    /// Converts a package tools error to a CLI error.
    ///
    /// Maps package tool errors to appropriate CLI error categories.
    fn from(error: sublime_pkg_tools::error::Error) -> Self {
        match error {
            sublime_pkg_tools::error::Error::Config(_) => Self::Configuration(format!("{error}")),
            sublime_pkg_tools::error::Error::Version(_) => Self::Validation(format!("{error}")),
            sublime_pkg_tools::error::Error::Changeset(_)
            | sublime_pkg_tools::error::Error::Changes(_)
            | sublime_pkg_tools::error::Error::Changelog(_)
            | sublime_pkg_tools::error::Error::Upgrade(_)
            | sublime_pkg_tools::error::Error::Audit(_)
            | sublime_pkg_tools::error::Error::Json(_) => Self::Execution(format!("{error}")),
            sublime_pkg_tools::error::Error::FileSystem(msg) => Self::Io(msg),
            sublime_pkg_tools::error::Error::Git(msg) => Self::Git(msg),
            sublime_pkg_tools::error::Error::IO(io_err) => Self::Io(format!("{io_err}")),
        }
    }
}

impl From<sublime_standard_tools::error::Error> for CliError {
    /// Converts a standard tools error to a CLI error.
    ///
    /// Maps standard tool errors to appropriate CLI error categories.
    fn from(error: sublime_standard_tools::error::Error) -> Self {
        match error {
            sublime_standard_tools::error::Error::Config(_) => {
                Self::Configuration(format!("{error}"))
            }
            sublime_standard_tools::error::Error::FileSystem(_) => Self::Io(format!("{error}")),
            sublime_standard_tools::error::Error::Command(_) => Self::Execution(format!("{error}")),
            sublime_standard_tools::error::Error::Monorepo(_)
            | sublime_standard_tools::error::Error::Workspace(_) => {
                Self::Package(format!("{error}"))
            }
            sublime_standard_tools::error::Error::Operation(msg) => Self::Execution(msg),
        }
    }
}

impl From<std::io::Error> for CliError {
    /// Converts a standard I/O error to a CLI error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::CliError;
    /// use std::io;
    ///
    /// let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    /// let cli_error: CliError = io_error.into();
    /// assert_eq!(cli_error.exit_code(), 74);
    /// ```
    fn from(error: std::io::Error) -> Self {
        Self::Io(format!("{error}"))
    }
}

impl From<serde_json::Error> for CliError {
    /// Converts a JSON serialization error to a CLI error.
    fn from(error: serde_json::Error) -> Self {
        Self::Execution(format!("JSON error: {error}"))
    }
}

impl From<toml::de::Error> for CliError {
    /// Converts a TOML parsing error to a CLI error.
    fn from(error: toml::de::Error) -> Self {
        Self::Configuration(format!("TOML parsing error: {error}"))
    }
}

impl From<serde_yaml::Error> for CliError {
    /// Converts a YAML parsing error to a CLI error.
    fn from(error: serde_yaml::Error) -> Self {
        Self::Configuration(format!("YAML parsing error: {error}"))
    }
}

impl From<anyhow::Error> for CliError {
    /// Converts an anyhow error to a CLI error.
    fn from(error: anyhow::Error) -> Self {
        Self::Execution(format!("{error}"))
    }
}
