//! # Command Executor Types
//!
//! ## What
//! This module defines command executor types for different execution strategies,
//! including default, synchronous, and shared executors.
//!
//! ## How
//! The executors provide different interfaces for command execution,
//! from simple async execution to sophisticated synchronous wrappers.
//!
//! ## Why
//! Different execution strategies are needed for different use cases,
//! from simple async execution to complex synchronous wrapping.

use std::sync::Arc;

/// Default implementation of the `CommandExecutor` trait.
///
/// Provides a standard implementation for executing commands directly without
/// any custom behavior.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::types::{DefaultCommandExecutor, CommandExecutor};
/// use sublime_standard_tools::command::types::Command;
///
/// let executor = DefaultCommandExecutor::default();
/// // Use the executor to run commands
/// ```
#[derive(Debug, Default)]
pub struct DefaultCommandExecutor;

/// Synchronous command executor that wraps async execution
///
/// Uses a dedicated runtime to provide synchronous command execution
/// without spreading async infection throughout the codebase. This eliminates
/// the need for async contexts when synchronous execution is preferred.
///
/// # Examples
///
/// ```rust
/// use sublime_standard_tools::command::{SyncCommandExecutor, CommandBuilder};
/// use sublime_standard_tools::error::Result;
///
/// # fn example() -> Result<()> {
/// let executor = SyncCommandExecutor::new()?;
/// let command = CommandBuilder::new("echo").arg("hello").build();
/// let output = executor.execute_sync(command)?;
/// println!("Output: {}", output.stdout());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct SyncCommandExecutor {
    /// Dedicated runtime for async operations
    pub(crate) runtime: tokio::runtime::Runtime,
    /// Underlying async executor
    pub(crate) executor: DefaultCommandExecutor,
}

/// Shared synchronous executor instance
///
/// Provides a globally shared synchronous command executor to avoid
/// creating multiple runtimes. This is the preferred way to perform
/// synchronous command execution to reduce resource usage.
///
/// # Examples
///
/// ```rust
/// use sublime_standard_tools::command::{SharedSyncExecutor, CommandBuilder};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let executor = SharedSyncExecutor::try_instance()?;
/// let command = CommandBuilder::new("git").arg("status").build();
/// let output = executor.execute(command)?;
/// println!("Git status: {}", output.stdout());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct SharedSyncExecutor {
    /// Shared synchronous executor instance
    pub(crate) executor: Arc<SyncCommandExecutor>,
}

/// Global state for shared synchronous executor
///
/// Represents the possible states of the global shared synchronous executor:
/// uninitialized, successfully created, or failed to create with error details.
#[derive(Debug)]
pub(crate) enum GlobalExecutorState {
    /// Executor has not been initialized yet
    Uninitialized,
    /// Executor was successfully created
    Success(&'static SharedSyncExecutor),
    /// Executor creation failed with this error
    Error(crate::error::Error),
}