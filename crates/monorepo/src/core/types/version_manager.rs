//! Version manager type definitions

use super::{MonorepoProject, VersioningStrategy};
use std::sync::Arc;

/// Manager for package versioning with dependency propagation
pub struct VersionManager {
    /// Reference to the monorepo project
    pub(crate) project: Arc<MonorepoProject>,
    /// Versioning strategy to use
    pub(crate) strategy: Box<dyn VersioningStrategy>,
}