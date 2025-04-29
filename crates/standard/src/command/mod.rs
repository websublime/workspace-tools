//! Command execution management for the sublime_standard_tools crate.
//!
//! What:
//! This module provides a robust system for executing external commands with features
//! such as timeout handling, resource limits, and streaming output. It handles all
//! aspects of command execution, from configuration to output processing.
//!
//! Who:
//! This module is used by developers who need to:
//! - Execute external commands with proper error handling
//! - Stream command output in real-time
//! - Manage command timeouts and resource limits
//! - Process command output in a structured way
//!
//! Why:
//! Reliable command execution is essential for interacting with external tools and
//! processes. This module provides a safe, efficient, and feature-rich way to
//! handle these interactions.

mod executor;
mod queue;
mod stream;
mod types;

pub use executor::{CommandExecutor, DefaultCommandExecutor};
pub use queue::{
    CommandPriority, CommandQueue, CommandQueueConfig, CommandQueueResult, CommandStatus,
};
pub use stream::{CommandStream, StreamConfig, StreamOutput};
pub use types::{Command, CommandBuilder, CommandOutput, ResourceLimits};
