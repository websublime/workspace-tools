//! Analyzer types for monorepo analysis
//!
//! This module contains type definitions for the monorepo analyzer.

use crate::core::MonorepoProject;
use std::sync::Arc;

/// Analyzer for comprehensive monorepo analysis
pub struct MonorepoAnalyzer {
    /// Reference to the monorepo project
    pub(crate) project: Arc<MonorepoProject>,
}