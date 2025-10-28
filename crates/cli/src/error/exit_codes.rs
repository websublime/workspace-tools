//! Exit code constants following the sysexits standard.
//!
//! This module defines exit codes used by the CLI to communicate the result
//! of command execution to the shell. The codes follow the BSD sysexits.h
//! convention for consistent behavior across Unix-like systems.
//!
//! # What
//!
//! Provides standard exit code constants that map to specific error conditions.
//! These codes help scripts and automation tools understand what went wrong
//! when a command fails.
//!
//! # How
//!
//! Each constant represents a specific exit condition as defined by the
//! sysexits.h standard. The CLI uses these codes via `CliError::exit_code()`
//! to return the appropriate status to the shell.
//!
//! # Why
//!
//! Using standard exit codes makes the CLI more predictable and allows
//! shell scripts and automation tools to handle errors appropriately based
//! on the type of failure.
//!
//! # Examples
//!
//! ```rust
//! use sublime_cli_tools::error::{CliError, ExitCode};
//!
//! let error = CliError::configuration("Invalid config");
//! assert_eq!(error.exit_code(), ExitCode::CONFIG);
//!
//! let error = CliError::network("Registry down");
//! assert_eq!(error.exit_code(), ExitCode::UNAVAILABLE);
//! ```
//!
//! # References
//!
//! - BSD sysexits.h: <https://man.freebsd.org/cgi/man.cgi?query=sysexits>

/// Exit code constants following the sysexits standard.
///
/// These constants are used throughout the CLI to provide consistent
/// exit codes that follow the BSD sysexits.h convention.
pub struct ExitCode;

impl ExitCode {
    /// Successful termination.
    ///
    /// The command completed successfully without errors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::ExitCode;
    ///
    /// assert_eq!(ExitCode::OK, 0);
    /// ```
    pub const OK: i32 = 0;

    /// Command line usage error.
    ///
    /// The command was used incorrectly, e.g., with invalid arguments,
    /// invalid options, or missing required parameters.
    ///
    /// # When to Use
    ///
    /// - Invalid command arguments
    /// - Missing required parameters
    /// - User cancelled an operation
    /// - Invalid user input
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::{CliError, ExitCode};
    ///
    /// let error = CliError::user("Invalid input");
    /// assert_eq!(error.exit_code(), ExitCode::USAGE);
    /// ```
    pub const USAGE: i32 = 64;

    /// Data format error.
    ///
    /// The input data was incorrect in some way. This should only be used
    /// for user data, not system files.
    ///
    /// # When to Use
    ///
    /// - Invalid version format
    /// - Invalid package.json
    /// - Invalid changeset data
    /// - Validation failures
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::{CliError, ExitCode};
    ///
    /// let error = CliError::validation("Invalid version");
    /// assert_eq!(error.exit_code(), ExitCode::DATAERR);
    /// ```
    pub const DATAERR: i32 = 65;

    /// Cannot open input file.
    ///
    /// An input file (not a system file) did not exist or was not readable.
    ///
    /// # When to Use
    ///
    /// - Required input file not found
    /// - Input file not readable
    pub const NOINPUT: i32 = 66;

    /// Addressee unknown.
    ///
    /// The user specified did not exist. This might be used for mail
    /// addresses or remote logins.
    pub const NOUSER: i32 = 67;

    /// Host name unknown.
    ///
    /// The host specified did not exist. This is used in mail addresses
    /// or network requests.
    pub const NOHOST: i32 = 68;

    /// Service unavailable.
    ///
    /// A service is unavailable. This can occur if a support program or
    /// file does not exist, cannot be opened, or a required network
    /// service is unavailable.
    ///
    /// # When to Use
    ///
    /// - npm registry unreachable
    /// - Network timeout
    /// - External service down
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::{CliError, ExitCode};
    ///
    /// let error = CliError::network("Registry unreachable");
    /// assert_eq!(error.exit_code(), ExitCode::UNAVAILABLE);
    /// ```
    pub const UNAVAILABLE: i32 = 69;

    /// Internal software error.
    ///
    /// An internal software error has been detected. This should be limited
    /// to non-operating system related errors as possible.
    ///
    /// # When to Use
    ///
    /// - Command execution failed
    /// - Git operation failed
    /// - Operation could not be completed
    /// - Internal processing error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::{CliError, ExitCode};
    ///
    /// let error = CliError::execution("Command failed");
    /// assert_eq!(error.exit_code(), ExitCode::SOFTWARE);
    ///
    /// let error = CliError::git("Repository not found");
    /// assert_eq!(error.exit_code(), ExitCode::SOFTWARE);
    /// ```
    pub const SOFTWARE: i32 = 70;

    /// System error (e.g., can't fork).
    ///
    /// An operating system error has been detected. This is intended to be
    /// used for such things as "cannot fork", "cannot create pipe", or the like.
    pub const OSERR: i32 = 71;

    /// Critical OS file missing.
    ///
    /// Some system file (e.g., /etc/passwd, /var/run/utmp, etc.) does not
    /// exist, cannot be opened, or has some sort of error (e.g., syntax error).
    pub const OSFILE: i32 = 72;

    /// Can't create (user) output file.
    ///
    /// A (user specified) output file cannot be created.
    pub const CANTCREAT: i32 = 73;

    /// Input/output error.
    ///
    /// An error occurred while doing I/O on some file.
    ///
    /// # When to Use
    ///
    /// - File read/write failed
    /// - Permission denied
    /// - Disk full
    /// - File not found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::{CliError, ExitCode};
    ///
    /// let error = CliError::io("Permission denied");
    /// assert_eq!(error.exit_code(), ExitCode::IOERR);
    /// ```
    pub const IOERR: i32 = 74;

    /// Temporary failure; user is invited to retry.
    ///
    /// Temporary failure, indicating something that is not really an error.
    /// In sendmail, this means that a mailer (e.g.) could not create a
    /// connection, and the request should be reattempted later.
    pub const TEMPFAIL: i32 = 75;

    /// Remote error in protocol.
    ///
    /// The remote system returned something that was "not possible" during
    /// a protocol exchange.
    pub const PROTOCOL: i32 = 76;

    /// Permission denied.
    ///
    /// You did not have sufficient permission to perform the operation.
    /// This is not intended for file system problems, which should use
    /// NOINPUT or CANTCREAT, but rather for higher level permissions.
    pub const NOPERM: i32 = 77;

    /// Configuration error.
    ///
    /// Something was found in an unconfigured or misconfigured state.
    ///
    /// # When to Use
    ///
    /// - Configuration file not found
    /// - Invalid configuration format
    /// - Configuration validation failed
    /// - Missing required configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::{CliError, ExitCode};
    ///
    /// let error = CliError::configuration("Config not found");
    /// assert_eq!(error.exit_code(), ExitCode::CONFIG);
    /// ```
    pub const CONFIG: i32 = 78;
}

impl ExitCode {
    /// Returns a human-readable description of the exit code.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::ExitCode;
    ///
    /// assert_eq!(ExitCode::description(ExitCode::OK), "Success");
    /// assert_eq!(ExitCode::description(ExitCode::CONFIG), "Configuration error");
    /// assert_eq!(ExitCode::description(999), "Unknown exit code");
    /// ```
    pub fn description(code: i32) -> &'static str {
        match code {
            Self::OK => "Success",
            Self::USAGE => "Command line usage error",
            Self::DATAERR => "Data format error",
            Self::NOINPUT => "Cannot open input",
            Self::NOUSER => "Addressee unknown",
            Self::NOHOST => "Host name unknown",
            Self::UNAVAILABLE => "Service unavailable",
            Self::SOFTWARE => "Internal software error",
            Self::OSERR => "System error",
            Self::OSFILE => "Critical OS file missing",
            Self::CANTCREAT => "Can't create output file",
            Self::IOERR => "Input/output error",
            Self::TEMPFAIL => "Temporary failure",
            Self::PROTOCOL => "Remote error in protocol",
            Self::NOPERM => "Permission denied",
            Self::CONFIG => "Configuration error",
            _ => "Unknown exit code",
        }
    }

    /// Checks if the exit code indicates success.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::ExitCode;
    ///
    /// assert!(ExitCode::is_success(ExitCode::OK));
    /// assert!(!ExitCode::is_success(ExitCode::CONFIG));
    /// ```
    pub fn is_success(code: i32) -> bool {
        code == Self::OK
    }

    /// Checks if the exit code indicates an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::error::ExitCode;
    ///
    /// assert!(!ExitCode::is_error(ExitCode::OK));
    /// assert!(ExitCode::is_error(ExitCode::CONFIG));
    /// assert!(ExitCode::is_error(ExitCode::SOFTWARE));
    /// ```
    pub fn is_error(code: i32) -> bool {
        code != Self::OK
    }
}
