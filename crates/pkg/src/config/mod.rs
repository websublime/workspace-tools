//! Configuration module for package tools.
//!
//! This module provides configuration management functionality that integrates
//! with the StandardConfig system from sublime-standard-tools. It defines
//! package-specific configuration options and extends the standard configuration
//! framework with package management capabilities.
//!
//! # Core Configuration Types
//!
//! - [`PackageToolsConfig`] - Main configuration structure
//! - [`VersionBumpConfig`] - Version bumping configuration  
//! - [`ResolutionConfig`] - Dependency resolution configuration
//! - [`CircularDependencyConfig`] - Circular dependency handling
//! - [`ContextAwareConfig`] - Context-aware features
//! - [`PerformanceConfig`] - Performance optimizations
//! - [`CacheConfig`] - Caching behavior
//!
//! # Integration with StandardConfig
//!
//! This configuration system is designed to extend and integrate with the
//! StandardConfig from sublime-standard-tools, providing a unified configuration
//! experience across all sublime tools.

pub mod package_config;
#[cfg(test)]
mod tests;

pub use package_config::*;