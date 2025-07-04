//! Workflow implementations for monorepo operations
//!
//! This module provides complete workflows that orchestrate multiple components
//! to achieve complex monorepo operations like releases and development cycles.
//! Workflows integrate changesets, tasks, version management, and Git operations.

pub mod development;
pub mod integration;
mod progress;
pub mod release;
#[cfg(test)]
mod tests;
pub mod types;

// Import from types module instead of implementation files
pub use types::{ChangesetHookIntegration, DevelopmentWorkflow, ReleaseWorkflow};

// Explicit re-exports from types module
pub use types::{
    AffectedPackageInfo,
    ChangeAnalysisResult,
    ChangeAnalysisWorkflowResult,
    ConfidenceLevel,
    DevelopmentResult,
    ImpactLevel,
    // Data
    PackageChangeFacts,
    // Options
    ReleaseOptions,
    // Results
    ReleaseResult,
    VersionRecommendation,
    VersioningWorkflowResult,
    WorkflowProgress,
    WorkflowStatus,
    // Status
    WorkflowStep,
};
