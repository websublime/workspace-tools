//! NPM registry client for package metadata queries.
//!
//! **What**: Provides HTTP client functionality for querying NPM registries to fetch
//! package metadata, versions, and upgrade information.
//!
//! **How**: This module uses reqwest with retry middleware to communicate with NPM
//! registries, handling authentication, timeouts, scoped packages, and private registries.
//! It parses registry responses and provides type-safe access to package metadata.
//!
//! **Why**: To enable reliable package upgrade detection by fetching the latest versions
//! and metadata from NPM registries, with proper error handling and enterprise support.
//!
//! # Features
//!
//! - **Public NPM Registry**: Query packages from the public NPM registry
//! - **Private Registries**: Support for private registries with authentication
//! - **Scoped Packages**: Handle scoped packages with custom registry mappings
//! - **Retry Logic**: Automatic retry on transient failures with exponential backoff
//! - **Authentication**: Bearer token authentication for private packages
//! - **Timeout Handling**: Configurable timeouts with proper error reporting
//! - **Version Comparison**: Semantic versioning comparison and upgrade type detection
//!
//! # Example
//!
//! ```rust,no_run
//! use sublime_pkg_tools::upgrade::{RegistryClient, UpgradeType};
//! use sublime_pkg_tools::config::RegistryConfig;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let workspace_root = PathBuf::from(".");
//! let config = RegistryConfig::default();
//!
//! // Create client
//! let client = RegistryClient::new(&workspace_root, config).await?;
//!
//! // Query package metadata
//! let metadata = client.get_package_info("express").await?;
//! println!("Package: {}", metadata.name);
//! println!("Latest version: {}", metadata.latest);
//! println!("Available versions: {}", metadata.versions.len());
//!
//! // Check if deprecated
//! if metadata.is_deprecated() {
//!     println!("Warning: Package is deprecated!");
//! }
//!
//! // Get latest version directly
//! let latest = client.get_latest_version("lodash").await?;
//! println!("Latest lodash: {}", latest);
//!
//! // Compare versions
//! let upgrade = client.compare_versions("1.2.3", "2.0.0")?;
//! println!("Upgrade type: {}", upgrade);
//! # Ok(())
//! # }
//! ```
//!
//! # Private Registry
//!
//! ```rust,no_run
//! use sublime_pkg_tools::upgrade::RegistryClient;
//! use sublime_pkg_tools::config::RegistryConfig;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut config = RegistryConfig::default();
//!
//! // Configure scoped registry
//! config.scoped_registries.insert(
//!     "myorg".to_string(),
//!     "https://npm.myorg.com".to_string()
//! );
//!
//! // Add authentication token
//! config.auth_tokens.insert(
//!     "https://npm.myorg.com".to_string(),
//!     "npm_AbCdEf123456".to_string()
//! );
//!
//! let client = RegistryClient::new(&PathBuf::from("."), config).await?;
//!
//! // Query private package
//! let metadata = client.get_package_info("@myorg/private-package").await?;
//! println!("Private package: {}", metadata.name);
//! # Ok(())
//! # }
//! ```
//!
//! # Configuration
//!
//! The registry client is configured via `RegistryConfig`:
//!
//! ```toml
//! [package_tools.upgrade.registry]
//! default_registry = "https://registry.npmjs.org"
//! timeout_secs = 30
//! retry_attempts = 3
//! retry_delay_ms = 1000
//! read_npmrc = true
//!
//! [package_tools.upgrade.registry.scoped]
//! "@myorg" = "https://npm.myorg.com"
//! "@internal" = "https://registry.internal.corp"
//! ```
//!
//! # Error Handling
//!
//! All registry operations return `Result<T, UpgradeError>` with specific error variants:
//!
//! - `PackageNotFound`: Package doesn't exist in registry (404)
//! - `AuthenticationFailed`: Authentication required or failed (401/403)
//! - `RegistryTimeout`: Request timed out
//! - `RegistryError`: General registry error (HTTP error)
//! - `InvalidResponse`: Registry returned malformed data
//! - `NetworkError`: Network connectivity issues
//!
//! # Module Structure
//!
//! This module is private with public types re-exported through the parent `upgrade` module:
//!
//! - `client`: Main `RegistryClient` implementation (pub(crate))
//! - `types`: Data structures for registry responses and upgrade types (pub(crate))
//! - `tests`: Integration tests with mock HTTP server
//!
//! Public API is accessed via `sublime_pkg_tools::upgrade::{RegistryClient, PackageMetadata, ...}`

pub(crate) mod client;
pub mod npmrc;
pub(crate) mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use self::client::RegistryClient;
pub use self::types::{PackageMetadata, RepositoryInfo, UpgradeType};
