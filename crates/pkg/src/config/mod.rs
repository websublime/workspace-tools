//! Configuration module for sublime_pkg_tools.
//!
//! This module handles configuration loading, validation, and management
//! for package management tools. It extends the standard configuration
//! system from sublime_standard_tools with package-specific settings.
//!
//! # What
//!
//! Provides configuration structures and management for:
//! - Changeset management settings (paths, environments, formats)
//! - Version management settings (strategies, snapshot formats)
//! - Registry configuration (URLs, authentication, timeouts)
//! - Release management settings (strategies, tagging, dry-run)
//! - Dependency management settings (propagation, circular detection)
//! - Conventional commit settings (types, breaking change detection)
//! - Changelog generation settings (templates, formatting)
//!
//! # How
//!
//! Integrates with sublime_standard_tools configuration system using
//! the `Configurable` trait. Supports TOML, JSON, and YAML formats
//! with environment variable overrides and validation.
//!
//! # Why
//!
//! Centralized configuration ensures consistent behavior across all
//! package management operations and allows for flexible customization
//! per project or user preferences.
mod changelog;
mod changeset;
mod conventional;
mod dependency;
mod manager;
mod package;
mod registry;
mod release;
mod version;

#[cfg(test)]
mod tests;

pub use changelog::ChangelogConfig;
pub use changeset::ChangesetConfig;
pub use conventional::{ConventionalCommitType, ConventionalConfig};
pub use dependency::DependencyConfig;
pub use manager::{EnvMapping, PackageToolsConfigManager, ENV_PREFIX};
pub use package::PackageToolsConfig;
pub use registry::{CustomRegistryConfig, RegistryConfig};
pub use release::ReleaseConfig;
pub use version::VersionConfig;
