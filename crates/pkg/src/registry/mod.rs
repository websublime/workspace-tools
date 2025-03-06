//! Registry functionality for dependency management.

pub mod dependency;
pub mod package;

pub use dependency::{
    DependencyRegistry, DependencyResolutionError, DependencyUpdate, ResolutionResult,
};
pub use package::{NpmRegistry, PackageRegistry};
