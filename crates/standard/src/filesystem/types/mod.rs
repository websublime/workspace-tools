//! # Filesystem Types Module
//!
//! ## What
//! This module provides a well-organized collection of filesystem-related types,
//! breaking down the monolithic types file into focused, maintainable modules.
//!
//! ## How
//! Types are organized by responsibility:
//! - `traits`: Core async filesystem trait
//! - `config`: Configuration-related types
//! - `path_types`: Node.js path type enums
//! - `path_utils`: Path utilities and extension traits
//!
//! ## Why
//! Modular organization improves maintainability, reduces cognitive load,
//! and enables better testing and documentation of individual components.

pub mod traits;
pub mod config;
pub mod path_types;
pub mod path_utils;

// Re-export all public types for backward compatibility
pub use traits::*;
pub use config::*;
pub use path_types::*;
pub use path_utils::*;