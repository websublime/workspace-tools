//! Workflow implementations for monorepo operations
//!
//! This module provides complete workflows that orchestrate multiple components
//! to achieve complex monorepo operations like releases and development cycles.
//! Workflows integrate changesets, tasks, version management, and Git operations.

pub mod development;
pub mod integration;
mod progress;
pub mod release;
pub mod types;

#[cfg(test)]
mod tests;

// Import from types module instead of implementation files
pub use types::{DevelopmentWorkflow, ChangesetHookIntegration, ReleaseWorkflow};

// Explicit re-exports from types module
pub use types::{
    // Options
    ReleaseOptions,
    // Results
    ReleaseResult, DevelopmentResult, ChangeAnalysisResult,
    AffectedPackageInfo, VersionRecommendation, VersioningWorkflowResult,
    ImpactLevel, ConfidenceLevel, ChangeAnalysisWorkflowResult,
    // Status
    WorkflowStep, WorkflowProgress, WorkflowStatus,
    // Data
    PackageChangeFacts,
};
