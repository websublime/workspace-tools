//! Standard tools for working with Node.js projects.
//!
//! What:
//! This crate provides foundational utilities for working with Node.js projects
//! from Rust applications. It includes robust command execution, project structure
//! handling, and comprehensive error management.
//!
//! Who:
//! This crate is designed for:
//! - Developers building Node.js tooling in Rust
//! - Build system implementors
//! - DevOps automation tools
//! - Monorepo management systems
//!
//! Why:
//! Building reliable Node.js tooling requires:
//! - Robust command execution with proper error handling
//! - Consistent project structure management
//! - Reliable package manager integration
//! - Cross-platform compatibility
//!
//! # Features
//!
//! ## Command Execution
//!
//! ```rust,no_run
//! use sublime_standard_tools::command::{CommandBuilder, DefaultCommandExecutor, CommandExecutor};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let executor = DefaultCommandExecutor::new();
//!
//! // Execute a command with timeout
//! let command = CommandBuilder::new("npm")
//!     .arg("install")
//!     .timeout(Duration::from_secs(60))
//!     .build();
//!
//! let output = executor.execute(command).await?;
//! println!("Installation completed: {}", output.success());
//! # Ok(())
//! # }
//! ```
//!
//! ## Streaming Output
//!
//! ```rust,no_run
//! use sublime_standard_tools::command::{
//!     CommandBuilder, DefaultCommandExecutor, CommandExecutor,
//!     StreamConfig, StreamOutput
//! };
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let executor = DefaultCommandExecutor::new();
//!
//! // Execute with streaming output
//! let command = CommandBuilder::new("npm")
//!     .arg("run")
//!     .arg("build")
//!     .build();
//!
//! let (mut stream, child) = executor
//!     .execute_stream(command, StreamConfig::default())
//!     .await?;
//!
//! while let Ok(Some(output)) = stream.next_timeout(Duration::from_secs(1)).await {
//!     match output {
//!         StreamOutput::Stdout(line) => println!("stdout: {}", line),
//!         StreamOutput::Stderr(line) => eprintln!("stderr: {}", line),
//!         StreamOutput::End => break,
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Error Handling
//!
//! The crate provides comprehensive error types that implement standard traits:
//!
//! ```rust
//! use sublime_standard_tools::error::{StandardError, CommandError};
//!
//! fn example() -> Result<(), StandardError> {
//!     let error = CommandError::new("Command failed to execute");
//!     Err(error.into())
//! }
//! ```
//!
//! # Configuration
//!
//! Resource limits and execution constraints can be configured:
//!
//! ```rust,no_run
//! use sublime_standard_tools::command::{CommandBuilder, ResourceLimits};
//! use std::time::Duration;
//!
//! let command = CommandBuilder::new("node")
//!     .arg("build.js")
//!     .timeout(Duration::from_secs(300))
//!     .resource_limits(
//!         ResourceLimits::new()
//!             .memory_limit(1024) // 1GB
//!             .cpu_limit(50)      // 50%
//!     )
//!     .build();
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

pub mod command;
pub mod error;

// Re-export commonly used types
pub use command::{
    Command, CommandBuilder, CommandExecutor, CommandOutput, DefaultCommandExecutor,
    ResourceLimits, StreamConfig, StreamOutput,
};

pub use error::{CommandError, CommandResult, StandardError, StandardResult};

/// Version of the crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the version of the crate
#[must_use]
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::time::Duration;

    #[tokio::test]
    async fn test_command_integration() -> Result<(), Box<dyn std::error::Error>> {
        let executor = DefaultCommandExecutor::new();
        let command =
            CommandBuilder::new("echo").arg("test").timeout(Duration::from_secs(1)).build();

        let output = executor.execute(command).await?;
        assert!(output.success());
        assert_eq!(output.stdout().trim(), "test");
        Ok(())
    }

    #[allow(clippy::expect_used)]
    #[test]
    fn test_error_conversion() {
        // Use a specific CommandError variant for clarity
        let cmd_error = CommandError::Timeout { duration: std::time::Duration::from_secs(1) };
        let std_error: StandardError = cmd_error.into();
        // Assert against the display string defined by StandardError::Command's #[error] attribute
        assert_eq!(std_error.to_string(), "Command execution error");

        // Optionally, check the source error type
        assert!(std_error.source().is_some());
        assert!(std_error.source().expect("Source should exist").is::<CommandError>());
    }

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
