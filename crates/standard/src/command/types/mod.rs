//! # Command Types Module
//!
//! ## What
//! This module provides a well-organized collection of command-related types,
//! breaking down the monolithic types file into focused, maintainable modules.
//!
//! ## How
//! Types are organized by responsibility:
//! - `command`: Core command types (Command, CommandBuilder, CommandOutput)
//! - `priority`: Priority and status enums (CommandPriority, CommandStatus)
//! - `queue`: Queue management types (CommandQueueResult, CommandQueueConfig)
//! - `stream`: Streaming types (StreamOutput, StreamConfig, CommandStream)
//! - `executor`: Executor types (DefaultCommandExecutor, SyncCommandExecutor)
//! - `queue_internal`: Internal queue implementation types
//!
//! ## Why
//! Modular organization improves maintainability, reduces cognitive load,
//! and enables better testing and documentation of individual components.

pub mod command;
pub mod executor;
pub mod internal;
pub mod priority;
pub mod queue;
pub mod stream;

// Re-export all public types for backward compatibility
pub use command::*;
pub use executor::*;
pub use internal::*;
pub use priority::*;
pub use queue::*;
pub use stream::*;
