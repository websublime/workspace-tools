//! External service clients for package management
//!
//! This module provides clients for external package registries and services,
//! handling network communication and external resource access.

pub mod npm_client;
pub mod registry_manager;
pub mod package_registry_client;
pub mod local_registry;

#[cfg(test)]
mod tests;

// Phase 4.2 integration pending - External API re-exports for full integration
#[allow(unused_imports)]
pub use npm_client::{NpmRegistry, PackageRegistry, PackageRegistryClone};
#[allow(unused_imports)]
pub use registry_manager::{RegistryAuth, RegistryManager, RegistryType};
#[allow(unused_imports)]
pub use local_registry::LocalRegistry;