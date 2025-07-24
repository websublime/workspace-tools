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

// TODO: Re-enable when integrated with Package service
// pub use npm_client::{NpmRegistry, PackageRegistry, PackageRegistryClone};
// pub use registry_manager::{RegistryAuth, RegistryManager, RegistryType};
// pub use local_registry::LocalRegistry;