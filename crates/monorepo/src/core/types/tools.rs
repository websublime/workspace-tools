//! Monorepo tools type definitions

use crate::analysis::MonorepoAnalyzer;
use super::MonorepoProject;

/// The main orchestrator for monorepo tools functionality
/// 
/// Uses direct borrowing from MonorepoProject components instead of trait objects.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct MonorepoTools<'a> {
    pub(crate) project: &'a MonorepoProject,
    pub(crate) analyzer: MonorepoAnalyzer<'a>,
}