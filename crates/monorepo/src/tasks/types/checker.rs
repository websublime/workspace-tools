//! Task condition checker type definitions

use crate::core::MonorepoProject;
use std::sync::Arc;

/// Checker for evaluating task execution conditions
pub struct ConditionChecker {
    /// Reference to the monorepo project
    pub(crate) project: Arc<MonorepoProject>,
}