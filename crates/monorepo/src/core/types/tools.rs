//! Monorepo tools type definitions

use crate::analysis::MonorepoAnalyzer;
use super::MonorepoProject;
use std::sync::Arc;

/// The main orchestrator for monorepo tools functionality
pub struct MonorepoTools {
    pub(crate) project: Arc<MonorepoProject>,
    pub(crate) analyzer: MonorepoAnalyzer,
}