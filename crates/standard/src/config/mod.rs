//! Configuration module for flexible and extensible configuration management.
//!
//! This module provides a generic configuration framework that can be used throughout
//! the crate. It supports multiple configuration sources (files, environment variables,
//! defaults), various formats (TOML, JSON, YAML), and hierarchical configuration with
//! proper merging semantics.
//!
//! # Architecture
//!
//! The configuration system is built around these core concepts:
//!
//! - **Configurable**: Trait for types that can be configured
//! - **ConfigProvider**: Trait for configuration sources
//! - **ConfigManager**: Generic manager for any Configurable type
//! - **ConfigSource**: Different sources of configuration data
//! - **StandardConfig**: The standard configuration for this crate
//!
//! # Example
//!
//! ```no_run
//! use sublime_standard_tools::config::{ConfigManager, StandardConfig};
//!
//! async fn load_config() -> Result<StandardConfig, Box<dyn std::error::Error>> {
//!     let manager = ConfigManager::<StandardConfig>::builder()
//!         .with_defaults()
//!         .with_file("~/.config/sublime/config.toml")
//!         .with_file(".sublime.toml")
//!         .with_env_prefix("SUBLIME")
//!         .build()?;
//!
//!     let config = manager.load().await?;
//!     Ok(config)
//! }
//! ```

#![warn(missing_docs)]

pub mod format;
pub mod manager;
pub mod source;
pub mod standard;
pub mod traits;
pub mod value;

// Re-export main types for convenience
pub use crate::error::{ConfigError, ConfigResult};
pub use format::ConfigFormat;
pub use manager::{ConfigBuilder, ConfigManager};
pub use source::{ConfigSource, ConfigSourcePriority};
pub use standard::StandardConfig;
pub use traits::{ConfigProvider, Configurable};
pub use value::ConfigValue;

// Re-export commonly used types from submodules
pub use standard::{
    CommandConfig, FilesystemConfig, MonorepoConfig, PackageManagerConfig, ValidationConfig,
};
