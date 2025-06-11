//! Core type definitions module
//!
//! This module contains all core-related type definitions organized
//! in separate files for better maintainability and consistency.
//!
//! The module is organized as follows:
//! - `changeset`: Changeset-related types and status definitions
//! - `package`: Package information and version status types
//! - `versioning`: Version management types and result structures
//! - `impact_analysis`: Version impact analysis types
//! - `versioning_plan`: Versioning plan and step types
//! - `strategies`: Versioning strategy implementations

mod changeset;
mod package;
mod versioning;
mod impact_analysis;
mod versioning_plan;
mod strategies;

pub use changeset::*;
pub use package::*;
pub use versioning::*;
pub use impact_analysis::*;
pub use versioning_plan::*;
pub use strategies::*;