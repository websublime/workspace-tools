//! Core type definitions for package management.

pub mod dependency;
pub mod package;
pub mod version;

// Re-export main types for convenience
pub use dependency::Dependency;
pub use package::Package;
pub use version::{Version, VersionStability, VersionUpdateStrategy};
