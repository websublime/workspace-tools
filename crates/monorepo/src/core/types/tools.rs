//! Monorepo tools type definitions

use super::MonorepoProject;
use crate::analysis::MonorepoAnalyzer;

/// The main orchestrator for monorepo tools functionality
///
/// Uses direct borrowing from MonorepoProject components instead of trait objects.
/// This follows Rust ownership principles and eliminates Arc proliferation.
pub struct MonorepoTools<'a> {
    pub(crate) project: &'a MonorepoProject,
    pub(crate) analyzer: MonorepoAnalyzer<'a>,
}
