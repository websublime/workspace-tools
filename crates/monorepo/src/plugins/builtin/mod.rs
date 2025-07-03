//! Built-in plugin implementations
//!
//! This module provides default plugin implementations that are compiled into the application.
//! These plugins demonstrate the plugin system and provide essential functionality.
//!
//! ## Architecture
//!
//! The builtin plugins are organized into separate modules for maintainability:
//! - [`analyzer`] - Code analysis and dependency tracking
//! - [`generator`] - Code generation and templating  
//! - [`validator`] - Validation and quality assurance
//! - [`configurator`] - Configuration generation and project analysis
//!
//! Common utilities shared across plugins are provided in the [`common`] module.

pub mod analyzer;
pub mod common;
pub mod configurator; 
pub mod generator;
pub mod validator;

// Re-export plugin structs for backwards compatibility
pub use analyzer::AnalyzerPlugin;
pub use configurator::ConfiguratorPlugin;
pub use generator::GeneratorPlugin;
pub use validator::ValidatorPlugin;