//! # Command Execution Module
//!
//! ## What
//! This module provides a comprehensive framework for executing external commands,
//! managing command output, and handling command queues with priority ordering.
//! It offers abstractions for synchronous and streaming command execution with
//! proper error handling, timeouts, and resource management.
//!
//! ## How
//! The module is organized into several components:
//! - `Command` and `CommandBuilder`: For creating and configuring commands
//! - `CommandExecutor`: For executing commands and capturing their output
//! - `CommandOutput`: For structured access to command results
//! - `CommandStream`: For real-time streaming of command output
//! - `CommandQueue`: For prioritized, concurrent command execution
//!
//! All operations follow a consistent error handling pattern and provide
//! timeouts and resource cleanup to ensure reliability.
//!
//! ## Why
//! Running external processes is error-prone and requires careful management
//! of resources, concurrency, and error handling. This module encapsulates these
//! concerns behind a clean API that promotes safe practices and provides
//! consistent behavior across different execution patterns.

mod builder;
mod executor;
mod output;
mod queue;
mod stream;
mod types;

#[cfg(test)]
mod tests;

pub use types::{
    Command, CommandBuilder, CommandOutput, CommandPriority, CommandQueue, CommandQueueConfig,
    CommandQueueResult, CommandStatus, CommandStream, DefaultCommandExecutor, SharedSyncExecutor,
    StreamConfig, StreamOutput, SyncCommandExecutor,
};

pub use executor::Executor;
