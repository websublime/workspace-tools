//! Business logic services for package management
//!
//! This module provides services that encapsulate business logic operations,
//! maintaining clean separation between data structures and business operations.

pub mod package_service;
pub mod workspace_resolver;

pub use package_service::*;
pub use workspace_resolver::{WorkspaceAwareDependencyResolver, WorkspaceResolutionResult};