//! Package management utilities for workspaces.
//!
//! This module provides tools for managing package dependencies, resolving version conflicts,
//! detecting dependency cycles, and upgrading dependencies.
//!
//! ## Registry Integration
//!
//! The library includes robust integration with package registries:
//!
//! - Multiple registry support through the `RegistryManager`
//! - Authentication for private registries
//! - Scoped package support (associating scopes with specific registries)
//! - Caching of registry responses to improve performance
//! - Support for loading configuration from `.npmrc` files
//!
//! ### Example: Working with Package Registries
//!
//! ```rust,no_run
//! use ws_pkg::{RegistryManager, RegistryType, RegistryAuth};
//!
//! // Create a new registry manager
//! let mut registry_manager = RegistryManager::new();
//!
//! // Add a custom registry for a specific scope
//! registry_manager.add_registry("https://custom-registry.com", RegistryType::Custom("my-client".to_string())).unwrap();
//! registry_manager.associate_scope("@my-scope", "https://custom-registry.com").unwrap();
//!
//! // Set authentication for the registry
//! let auth = RegistryAuth {
//!     token: "my-auth-token".to_string(),
//!     token_type: "Bearer".to_string(),
//!     always: false,
//! };
//! registry_manager.set_auth("https://custom-registry.com", auth).unwrap();
//!
//! // Get latest version of a package
//! let latest_version = registry_manager.get_latest_version("@my-scope/package").unwrap();
//! println!("Latest version: {:?}", latest_version);
//! ```

pub mod bump;
pub mod error;
pub mod graph;
pub mod registry;
pub mod types;
pub mod upgrader;

// Re-export commonly used items for convenience
pub use error::{PkgError, Result};
pub use graph::{DependencyGraph, Node};
pub use registry::{
    DependencyRegistry, LocalRegistry, NpmRegistry, RegistryAuth, RegistryManager, RegistryType,
    ResolutionResult,
};
pub use types::{
    dependency::Dependency,
    diff::{ChangeType, DependencyChange, PackageDiff},
    package::Package,
    version::{Version, VersionRelationship, VersionStability, VersionUpdateStrategy},
};
pub use upgrader::{
    AvailableUpgrade, DependencyUpgrader, ExecutionMode, PackageRegistry, UpgradeConfig,
    UpgradeStatus,
};
