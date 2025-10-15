//! Configuration module for package tools settings and management.
//!
//! **What**: Provides comprehensive configuration management for all package tools functionality,
//! including changesets, versioning, dependencies, upgrades, changelog, and audit settings.
//!
//! **How**: This module integrates with `sublime_standard_tools` configuration system to load
//! settings from TOML files, environment variables, and programmatic sources. It provides
//! validation, merging, and type-safe access to all configuration options.
//!
//! **Why**: To enable flexible, environment-specific configuration that supports both simple
//! single-package projects and complex monorepo setups, with sensible defaults and clear
//! validation rules.
//!
//! # Features
//!
//! - **Hierarchical Configuration**: Load from multiple sources with priority ordering
//! - **Environment Overrides**: Override settings via environment variables
//! - **Validation**: Validate configuration before use
//! - **Merging**: Merge configurations from different sources
//! - **Type Safety**: Strongly-typed configuration structures
//! - **Documentation**: Comprehensive inline documentation for all settings
//! - **Sensible Defaults**: Work out of the box with minimal configuration
//!
//! # Example
//!
//! ```rust,ignore
//! use sublime_pkg_tools::config::PackageToolsConfig;
//! use sublime_standard_tools::config::ConfigManager;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load from file with defaults
//! let config = ConfigManager::<PackageToolsConfig>::builder()
//!     .with_defaults(PackageToolsConfig::default())
//!     .with_file_optional("package-tools.toml")
//!     .with_env_prefix("PKG_TOOLS")
//!     .build()
//!     .await?
//!     .load()
//!     .await?;
//!
//! // Access configuration
//! println!("Changeset path: {}", config.changeset.path);
//! println!("Version strategy: {:?}", config.version.strategy);
//! # Ok(())
//! # }
//! ```
//!
//! # Configuration File Format
//!
//! Configuration is typically stored in a TOML file:
//!
//! ```toml
//! [package_tools.changeset]
//! path = ".changesets"
//! history_path = ".changesets/history"
//! available_environments = ["development", "staging", "production"]
//! default_environments = ["production"]
//!
//! [package_tools.version]
//! strategy = "independent"
//! default_bump = "patch"
//! snapshot_format = "{version}-{branch}.{timestamp}"
//!
//! [package_tools.dependency]
//! propagation_bump = "patch"
//! propagate_dependencies = true
//! propagate_dev_dependencies = false
//! propagate_peer_dependencies = true
//! max_depth = 10
//! fail_on_circular = true
//!
//! [package_tools.upgrade]
//! auto_changeset = true
//! changeset_bump = "patch"
//!
//! [package_tools.upgrade.registry]
//! default_registry = "https://registry.npmjs.org"
//! timeout_secs = 30
//! retry_attempts = 3
//!
//! [package_tools.changelog]
//! enabled = true
//! format = "keep-a-changelog"
//! include_commit_links = true
//! repository_url = "https://github.com/org/repo"
//!
//! [package_tools.audit]
//! enabled = true
//! min_severity = "warning"
//! ```
//!
//! # Environment Variables
//!
//! Settings can be overridden using environment variables with the configured prefix:
//!
//! ```bash
//! export PKG_TOOLS_CHANGESET_PATH=".custom-changesets"
//! export PKG_TOOLS_VERSION_STRATEGY="unified"
//! export PKG_TOOLS_DEPENDENCY_PROPAGATION_BUMP="minor"
//! ```
//!
//! # Configuration Validation
//!
//! All configuration structures implement validation:
//!
//! ```rust,ignore
//! use sublime_pkg_tools::config::PackageToolsConfig;
//! use sublime_standard_tools::config::Configurable;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = PackageToolsConfig::default();
//!
//! // Validate configuration
//! config.validate()?;
//!
//! println!("Configuration is valid");
//! # Ok(())
//! # }
//! ```
//!
//! # Module Structure
//!
//! This module will contain:
//! - `package_tools`: Main `PackageToolsConfig` structure
//! - `changeset`: Changeset-specific configuration
//! - `version`: Versioning strategy and options
//! - `dependency`: Dependency propagation settings
//! - `upgrade`: Upgrade detection and application settings
//! - `changelog`: Changelog generation configuration
//! - `audit`: Audit and health check settings
//! - `git`: Git integration settings

#![allow(clippy::todo)]

// Module will be implemented in subsequent stories (Epic 2)
