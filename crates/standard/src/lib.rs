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
//! ## Configuration Management
//!
//! ```rust,no_run
//! use sublime_standard_tools::config::{ConfigManager, ConfigScope, ConfigValue};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut config = ConfigManager::new();
//!
//! // Set configuration paths
//! config.set_path(ConfigScope::User, "~/.config/my-app.json");
//! config.set_path(ConfigScope::Project, "./my-app.json");
//!
//! // Load configurations
//! config.load_all()?;
//!
//! // Get and set values
//! if let Some(value) = config.get("timeout") {
//!     if let Some(timeout) = value.as_integer() {
//!         println!("Timeout: {} seconds", timeout);
//!     }
//! }
//!
//! config.set("timeout", ConfigValue::Integer(60));
//!
//! // Save configurations
//! config.save_all()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Validation
//!
//! ```rust
//! use sublime_standard_tools::validation::{ValidationResult, Validator, ValidationRule};
//!
//! struct MinLengthRule { min_length: usize }
//!
//! impl ValidationRule<String> for MinLengthRule {
//!     fn validate(&self, target: &String) -> ValidationResult {
//!         if target.len() < self.min_length {
//!             ValidationResult::Error(vec![
//!                 format!("String length must be at least {}", self.min_length)
//!             ])
//!         } else {
//!             ValidationResult::Valid
//!         }
//!     }
//! }
//!
//! # fn example() {
//! let mut validator = Validator::<String>::new();
//! validator.add_rule(MinLengthRule { min_length: 5 });
//!
//! let result = validator.validate(&"test".to_string());
//! assert!(result.has_errors());
//!
//! let result = validator.validate(&"valid text".to_string());
//! assert!(result.is_valid());
//! # }
//! ```
//!
//! # Error Handling
//!
//! The crate provides comprehensive error types that implement standard traits:
//!
//! ```rust
//! use sublime_standard_tools::error::{StandardError, CommandError};
//! use std::time::Duration;
//!
//! fn example() -> Result<(), StandardError> {
//!     // Create a command timeout error
//!     let error = CommandError::Timeout { duration: Duration::from_secs(5) };
//!
//!     // Convert to StandardError
//!     Err(error.into())
//! }
//! ```
//!
//! # Diagnostics Collection
//!
//! ```rust,no_run
//! use sublime_standard_tools::diagnostics::{DiagnosticCollector, DiagnosticLevel};
//!
//! # fn example() {
//! let collector = DiagnosticCollector::new();
//!
//! // Record diagnostics
//! collector.info("initialization", "System initialized successfully");
//! collector.warning("disk_space", "Low disk space detected");
//!
//! // Get critical and error diagnostics
//! let critical_errors = collector.entries_with_level_at_or_above(DiagnosticLevel::Error);
//! # }
//! ```
//!
//! # Caching
//!
//! ```rust,no_run
//! use sublime_standard_tools::cache::{Cache, CacheConfig, CacheStrategy};
//! use std::time::Duration;
//!
//! # fn example() {
//! // Create a cache with custom config
//! let config = CacheConfig {
//!     default_ttl: Duration::from_secs(60),
//!     capacity: 100,
//!     strategy: CacheStrategy::LRU,
//! };
//!
//! let cache = Cache::<String, String>::with_config(config);
//!
//! // Use the cache
//! cache.put("key1".to_string(), "value1".to_string());
//!
//! if let Some(value) = cache.get(&"key1".to_string()) {
//!     println!("Retrieved: {}", value);
//! }
//! # }
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

pub mod cache;
pub mod command;
pub mod config;
pub mod diagnostic;
pub mod error;
pub mod project;
pub mod validation;

// Re-export commonly used types
pub use command::{
    Command, CommandBuilder, CommandExecutor, CommandOutput, DefaultCommandExecutor,
    ResourceLimits, StreamConfig, StreamOutput,
};

pub use cache::{Cache, CacheConfig, CacheStrategy};
pub use config::{ConfigManager, ConfigScope, ConfigValue};
pub use diagnostic::{DiagnosticCollector, DiagnosticLevel};
pub use error::{CommandError, CommandResult, StandardError, StandardResult};
pub use validation::{
    CombinedRule, NumericRangeRule, ParseableRule, PathExistsRule, PatternRule, StringLengthRule,
    Validatable, ValidationContext, ValidationResult, ValidationRule, Validator,
};

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

    #[test]
    fn test_cache() {
        let cache = Cache::<String, String>::new();
        cache.put("key1".to_string(), "value1".to_string());

        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), None);
    }

    #[test]
    fn test_diagnostics() {
        let collector = DiagnosticCollector::new();
        collector.info("test", "info message");
        collector.warning("test", "warning message");

        let entries = collector.entries();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_validation() {
        let result1 = ValidationResult::Error(vec!["error".to_string()]);
        let result2 = ValidationResult::Warning(vec!["warning".to_string()]);

        let merged = result1.clone().merge(result2.clone());
        assert!(matches!(merged, ValidationResult::Error(_)));

        assert!(result1.has_errors());
        assert!(!result1.is_valid());
        assert!(result2.has_warnings());
        assert!(result2.is_valid());
    }
}
