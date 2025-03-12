//! Registry functionality for dependency management.

pub mod dependency;
// We'll implement these later as needed
// pub mod local;
// pub mod manager;
// pub mod package;

// Re-export main types for convenience
pub use dependency::{
    DependencyRegistry, DependencyUpdateInfo, ResolutionErrorType, ResolutionResult,
};
