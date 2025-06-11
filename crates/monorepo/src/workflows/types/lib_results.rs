//! Workflow result types for lib.rs integration

use crate::analysis::{BranchComparisonResult, ChangeAnalysis};
use crate::core::{VersioningPlan, VersioningResult};

/// Result of a development workflow execution
pub struct DevelopmentResult {
    /// Tasks that were executed
    pub affected_tasks: Vec<String>,
}

/// Result of a change analysis workflow (Phase 2)
#[derive(Debug)]
pub enum ChangeAnalysisWorkflowResult {
    /// Result of branch comparison
    BranchComparison(BranchComparisonResult),
    /// Result of change analysis since a reference
    ChangeAnalysis(ChangeAnalysis),
}

/// Result of a versioning workflow execution (Phase 2)
#[derive(Debug)]
pub struct VersioningWorkflowResult {
    /// The versioning operation result
    pub versioning_result: VersioningResult,
    /// The plan that was executed
    pub plan_executed: Option<VersioningPlan>,
    /// Duration of the workflow execution
    pub duration: std::time::Duration,
}