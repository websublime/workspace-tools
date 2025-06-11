//! Workflow type definitions module
//!
//! This module contains all workflow-related type definitions organized
//! in separate files for better maintainability and consistency.
//!
//! The module is organized as follows:
//! - `options`: Configuration options for different workflow types
//! - `results`: Result types for workflow executions
//! - `status`: Status tracking and progress monitoring types

mod options;
mod results;
mod status;

pub use options::*;
pub use results::*;
pub use status::*;
