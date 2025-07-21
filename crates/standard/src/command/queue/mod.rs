//! # Command Queue Module
//!
//! ## What
//! This module organizes the command queue implementation into focused sub-modules
//! for better maintainability and code organization.
//!
//! ## How
//! The implementation is split into logical modules:
//! - `queue_result_impl`: Queue result implementations
//! - `command_queue_impl`: CommandQueue main implementation
//! - `queue_processor_impl`: QueueProcessor implementation
//!
//! ## Why
//! Breaking down the large queue implementation into focused modules
//! improves code organization and makes it easier to maintain and test.

pub mod processor;
pub mod queue;
pub mod result;

// No re-exports needed - implementations are for internal types
