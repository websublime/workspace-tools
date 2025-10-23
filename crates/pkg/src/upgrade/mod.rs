//! Dependency upgrade module for detecting and applying external package upgrades.
//!
//! **What**: Provides functionality to detect available upgrades for external npm packages,
//! apply those upgrades with optional filtering, and manage rollback on failures.
//!
//! **How**: This module integrates with npm registries (including private registries) to fetch
//! package metadata, compares current versions with available versions, and updates package.json
//! files. It supports dry-run mode, automatic changeset creation, and backup/rollback mechanisms.
//!
//! **Why**: To enable safe, automated dependency upgrades with fine-grained control over which
//! packages and versions to upgrade, supporting both security patches and feature updates while
//! maintaining project stability.
//!
//! # Features
//!
//! - **Upgrade Detection**: Detect available upgrades for external dependencies
//! - **Selective Upgrades**: Filter by patch/minor/major, specific packages, or dependencies
//! - **Registry Support**: Support for npm registry, private registries, and scoped packages
//! - **.npmrc Integration**: Read authentication and registry configuration from .npmrc
//! - **Dry-Run Mode**: Preview changes before applying them
//! - **Automatic Changeset**: Optionally create changesets for applied upgrades
//! - **Backup/Rollback**: Automatic backup and rollback on failure
//! - **Concurrency**: Parallel package metadata fetching for performance
//!
//! # Example
//!
//! ```rust,ignore
//! use sublime_pkg_tools::upgrade::{
//!     detect_upgrades, apply_upgrades, DetectionOptions, UpgradeSelection
//! };
//! use sublime_pkg_tools::config::PackageToolsConfig;
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let workspace_root = PathBuf::from(".");
//! let fs = FileSystemManager::new();
//! let config = PackageToolsConfig::default();
//!
//! // Detect available upgrades
//! let options = DetectionOptions::all();
//! let available = detect_upgrades(&workspace_root, options, &fs).await?;
//! println!("Found {} available upgrades", available.packages.len());
//!
//! // Apply patch upgrades only (dry run)
//! let selection = UpgradeSelection::patch_only();
//! let result = apply_upgrades(available.packages.clone(), selection, true, &fs).await?;
//! println!("Would upgrade {} dependencies", result.applied.len());
//!
//! // Apply for real
//! let result = apply_upgrades(available.packages, selection, false, &fs).await?;
//! println!("Upgraded {} dependencies", result.applied.len());
//! # Ok(())
//! # }
//! ```
//!
//! # Upgrade Selection
//!
//! Control which upgrades to apply using selection criteria:
//!
//! ```rust,ignore
//! use sublime_pkg_tools::upgrade::{
//!     detect_upgrades, apply_upgrades, DetectionOptions, UpgradeSelection
//! };
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let workspace_root = PathBuf::from(".");
//! # let fs = FileSystemManager::new();
//! # let available = detect_upgrades(&workspace_root, DetectionOptions::all(), &fs).await?;
//! // Only patch upgrades
//! let selection = UpgradeSelection::patch_only();
//! apply_upgrades(available.packages.clone(), selection, false, &fs).await?;
//!
//! // Patch and minor upgrades
//! let selection = UpgradeSelection::minor_and_patch();
//! apply_upgrades(available.packages.clone(), selection, false, &fs).await?;
//!
//! // Specific packages only
//! let selection = UpgradeSelection::packages(vec!["express".to_string(), "lodash".to_string()]);
//! apply_upgrades(available.packages.clone(), selection, false, &fs).await?;
//!
//! // Specific dependencies only
//! let selection = UpgradeSelection::dependencies(vec!["react".to_string()]);
//! apply_upgrades(available.packages, selection, false, &fs).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Private Registry Support
//!
//! Configure private registries and authentication:
//!
//! ```toml
//! [package_tools.upgrade.registry]
//! default_registry = "https://registry.npmjs.org"
//! timeout_secs = 30
//! retry_attempts = 3
//! read_npmrc = true
//!
//! [package_tools.upgrade.registry.scoped]
//! "@myorg" = "https://npm.myorg.com"
//! "@internal" = "https://registry.internal.corp"
//! ```
//!
//! # Automatic Changeset Creation
//!
//! Automatically create changesets for applied upgrades:
//!
//! ```rust,ignore
//! use sublime_pkg_tools::upgrade::{
//!     detect_upgrades, apply_upgrades, DetectionOptions, UpgradeSelection
//! };
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let workspace_root = PathBuf::from(".");
//! # let fs = FileSystemManager::new();
//! # let available = detect_upgrades(&workspace_root, DetectionOptions::all(), &fs).await?;
//! // TODO: will be implemented on story 9.6
//! // Apply upgrades and create changeset
//! let selection = UpgradeSelection::patch_only();
//! let result = apply_upgrades(available.packages, selection, false, &fs).await?;
//!
//! // Automatic changeset creation will be added in story 9.6
//! if let Some(changeset_id) = result.changeset_id {
//!     println!("Created changeset: {}", changeset_id);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Rollback on Failure
//!
//! Automatic rollback when upgrades fail:
//!
//! ```rust,ignore
//! use sublime_pkg_tools::upgrade::UpgradeManager;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let manager: UpgradeManager = todo!();
//! // TODO: will be implemented on story 9.5
//! // // Upgrades are automatically rolled back on failure
//! // match manager.apply_upgrades(selection, false).await {
//! //     Ok(result) => println!("Success: {} upgrades applied", result.applied.len()),
//! //     Err(e) => {
//! //         // Automatic rollback has occurred
//! //         println!("Upgrade failed and was rolled back: {}", e);
//! //     }
//! // }
//! //
//! // // Or manually rollback the last operation
//! // manager.rollback_last().await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Module Structure
//!
//! This module will contain:
//! - `manager`: The main `UpgradeManager` for orchestrating upgrade operations
//! - `registry`: Registry client for fetching package metadata
//! - `npmrc`: .npmrc parser for registry and authentication configuration
//! - `selection`: Upgrade selection and filtering logic
//! - `result`: Result types for upgrade operations
//! - `backup`: Backup and rollback mechanisms

#![allow(clippy::todo)]

// Registry module for NPM package metadata queries (Story 9.1 - IMPLEMENTED)
mod registry;

// Detection module for upgrade detection (Story 9.3 - IMPLEMENTED)
mod detection;

// Application module for applying upgrades (Story 9.4 - IMPLEMENTED)
mod application;

// Re-export registry public types
pub use registry::{
    npmrc::NpmrcConfig, PackageMetadata, RegistryClient, RepositoryInfo, UpgradeType,
};

// Re-export detection public types and functions
pub use detection::{
    detect_upgrades, DependencyUpgrade, DetectionOptions, PackageUpgrades, UpgradePreview,
    UpgradeSummary, VersionInfo,
};

// Re-export application public types and functions
pub use application::{
    apply_upgrades, AppliedUpgrade, ApplySummary, UpgradeResult, UpgradeSelection,
};

// Remaining modules will be implemented in subsequent stories (Epic 9)
// - backup: Backup and rollback (Story 9.5)
// - manager: Main UpgradeManager (Story 9.7)
