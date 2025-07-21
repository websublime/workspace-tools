//! # Project Types Module
//!
//! ## What
//! This module provides a well-organized collection of project-related types,
//! breaking down the monolithic types file into focused, maintainable modules.
//!
//! ## How
//! Types are organized by responsibility:
//! - `project`: Core project types and traits
//! - `validation`: Validation status and related types
//! - `descriptor`: Project descriptor enum
//!
//! ## Why
//! Modular organization improves maintainability, reduces cognitive load,
//! and enables better testing and documentation of individual components.

pub mod descriptor;
pub mod project;
pub mod validation;

// Re-export all public types for backward compatibility
pub use descriptor::*;
pub use project::*;
pub use validation::*;
