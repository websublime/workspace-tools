//! Business logic services for package management
//!
//! This module provides services that encapsulate business logic operations,
//! maintaining clean separation between data structures and business operations.

pub mod concurrent_processor;
pub mod package_command_service;
pub mod package_service;
pub mod performance_optimizer;
pub mod workspace_resolver;

#[cfg(test)]
mod tests;

pub use concurrent_processor::*;
// Phase 4.2 integration pending - PackageCommandService API for full integration
#[allow(unused_imports)]
pub use package_command_service::*;
pub use package_service::*;
pub use performance_optimizer::*;