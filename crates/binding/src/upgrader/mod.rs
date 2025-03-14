//! JavaScript bindings for dependency upgrading functionality.

pub mod config;
pub mod status;
pub mod upgrade;

// Re-export main types for convenience
pub use config::{
    create_default_upgrade_config, create_upgrade_config_from_strategy,
    create_upgrade_config_with_registries, ExecutionMode, UpgradeConfig,
};
pub use status::UpgradeStatus;
pub use upgrade::{AvailableUpgrade, DependencyUpgrader};
