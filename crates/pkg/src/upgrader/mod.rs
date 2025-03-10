//! Dependency upgrading functionality.

mod config;
mod registry;
mod status;
mod upgrader;

pub use crate::registry::{
    LocalRegistry, NpmRegistry, RegistryAuth, RegistryManager, RegistryType,
};
pub use config::{ExecutionMode, UpgradeConfig};
pub use registry::PackageRegistry;
pub use status::UpgradeStatus;
pub use upgrader::{AvailableUpgrade, DependencyUpgrader};
