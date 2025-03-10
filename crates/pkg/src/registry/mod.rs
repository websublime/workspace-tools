//! Registry functionality for dependency management.

pub mod dependency;
pub mod local;
pub mod manager;
pub mod package;

pub use dependency::{
    DependencyRegistry, DependencyResolutionError, DependencyUpdate, ResolutionResult,
};
pub use local::LocalRegistry;
pub use manager::{RegistryAuth, RegistryManager, RegistryType};
pub use package::{NpmRegistry, PackageRegistry};
