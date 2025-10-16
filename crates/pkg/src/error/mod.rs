//! Error types and error handling utilities for package tools operations.
//!
//! **What**: Provides a comprehensive error hierarchy for all package tools operations,
//! including detailed error contexts, error recovery strategies, and result type aliases.
//!
//! **How**: This module defines domain-specific error types for each major operation area
//! (changesets, versioning, dependencies, upgrades, changelog, audit), with rich context
//! information and support for error chaining and recovery.
//!
//! **Why**: To provide clear, actionable error messages that help users understand what
//! went wrong and how to fix it, while enabling robust error handling and recovery in
//! automated workflows.
//!
//! # Features
//!
//! - **Hierarchical Errors**: Structured error types organized by operation domain
//! - **Rich Context**: Detailed error information including paths, operations, and reasons
//! - **Error Chaining**: Support for nested errors to preserve error context
//! - **Recovery Strategies**: Pluggable error recovery mechanisms
//! - **Display Formatting**: Human-readable error messages
//! - **Debug Information**: Detailed debug output for troubleshooting
//! - **Error Conversion**: Automatic conversion from standard library and dependency errors
//!
//! # Error Categories
//!
//! ## ChangesetError
//! Errors related to changeset operations (create, load, update, archive).
//!
//! ## VersionError
//! Errors related to version resolution, propagation, and application.
//!
//! ## DependencyError
//! Errors related to dependency graph construction and circular dependency detection.
//!
//! ## UpgradeError
//! Errors related to dependency upgrade detection and application.
//!
//! ## ChangelogError
//! Errors related to changelog generation and parsing.
//!
//! ## AuditError
//! Errors related to audits and health checks.
//!
//! ## ConfigError
//! Errors related to configuration loading and validation.
//!
//! # Example
//!
//! ```rust,ignore
//! use sublime_pkg_tools::error::{ChangesetError, ChangesetResult};
//!
//! fn load_changeset(branch: &str) -> ChangesetResult<String> {
//!     if branch.is_empty() {
//!         return Err(ChangesetError::InvalidBranch {
//!             branch: branch.to_string(),
//!             reason: "Branch name cannot be empty".to_string(),
//!         });
//!     }
//!
//!     // TODO: will be implemented on story 6.3
//!     Ok("changeset content".to_string())
//! }
//! ```
//!
//! # Error Context
//!
//! Add context to errors for better debugging:
//!
//! ```rust,ignore
//! use sublime_pkg_tools::error::{ErrorContext, ChangesetResult};
//!
//! # fn load_changeset(branch: &str) -> ChangesetResult<String> { Ok("test".to_string()) }
//! #
//! fn process_changeset(branch: &str) -> ChangesetResult<()> {
//!     let content = load_changeset(branch)
//!         .with_context(|| format!("Failed to load changeset for branch '{}'", branch))?;
//!
//!     // TODO: will be implemented on story 6.3
//!     Ok(())
//! }
//! ```
//!
//! # Error Recovery
//!
//! Use recovery strategies for resilient operations:
//!
//! ```rust,ignore
//! use sublime_pkg_tools::error::{ErrorRecoveryManager, RecoveryStrategy};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // TODO: will be implemented on story 3.2
//! // let mut recovery = ErrorRecoveryManager::new();
//! // recovery.add_strategy(RecoveryStrategy::Retry { max_attempts: 3, delay: std::time::Duration::from_secs(1) });
//! //
//! // let result = recovery.recover(|| async {
//! //     // operation that might fail
//! //     Ok(())
//! // }).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Module Structure
//!
//! This module will contain:
//! - Error type definitions for each operation domain
//! - Result type aliases for convenience
//! - Error context utilities
//! - Error recovery mechanisms
//! - Error conversion implementations

#![allow(clippy::todo)]

// Module will be implemented in subsequent stories (Epic 3)
