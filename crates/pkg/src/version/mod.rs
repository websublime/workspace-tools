//! Version utilities and types
//!
//! Provides types and functions for working with semantic versions,
//! version comparisons, and version update strategies.

pub mod cascade_bumper;
pub mod change_set;
pub mod version;
pub mod versioning_strategy;

#[cfg(test)]
mod tests;

// Phase 4.2 exports - Core cascade bumping functionality
pub use cascade_bumper::{CascadeBumper, CascadeBumpAnalysis, CascadeContextInfo};
pub use change_set::{BumpExecutionMode, ChangeSet};
pub use versioning_strategy::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy};
