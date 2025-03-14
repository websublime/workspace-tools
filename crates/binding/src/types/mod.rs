//! Type definitions for the ws_binding crate.

pub mod dependency;
pub mod diff;
pub mod package;
pub mod version;

// Re-export main types for convenience
pub use dependency::Dependency;
pub use diff::{ChangeType, DependencyChange, PackageDiff};
pub use package::Package;
pub use version::Version;
