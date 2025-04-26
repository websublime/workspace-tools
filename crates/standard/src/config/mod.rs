//! Configuration management system.
//!
//! What:
//! This module provides a flexible configuration management system that
//! supports multiple configuration formats, scopes, and storage locations.
//!
//! Who:
//! Used by developers who need to:
//! - Manage application and component settings
//! - Support user and project configurations
//! - Load and save configuration files
//! - Access configuration from multiple contexts
//!
//! Why:
//! Effective configuration management is essential for:
//! - Customizable application behavior
//! - User preference handling
//! - Project-specific settings
//! - Runtime configuration changes

mod settings;

pub use settings::{ConfigFormat, ConfigManager, ConfigScope, ConfigValue};
