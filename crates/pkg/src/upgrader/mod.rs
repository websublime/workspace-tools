//! Dependency upgrading functionality.

mod config;
mod registry;
mod status;
mod upgrader;

pub use config::{ExecutionMode, UpgradeConfig};
pub use registry::PackageRegistry;
pub use status::UpgradeStatus;
pub use upgrader::{AvailableUpgrade, DependencyUpgrader};

// Re-export from lib.rs for backwards compatibility
pub use crate::registry::package::NpmRegistry;
