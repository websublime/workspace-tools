//! Validation utilities for ensuring architectural constraints
//!
//! This module provides compile-time and runtime validation to ensure
//! that the codebase follows established architectural patterns.

pub mod ownership_validator;
pub mod ownership_assertions;

pub use ownership_validator::{NoSharedOwnership, MoveSemantics, NotCopy};