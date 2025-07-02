//! Workflow type definitions module
//!
//! This module contains all workflow-related type definitions organized
//! in separate files for better maintainability and consistency.
//!
//! The module is organized as follows:
//! - `options`: Configuration options for different workflow types
//! - `results`: Result types for workflow executions
//! - `status`: Status tracking and progress monitoring types
//! - `data`: Simple data structures used within workflows

mod data;
mod development;
mod integration;
mod options;
mod release;
mod results;
mod status;

// Explicit exports to avoid wildcard re-exports

// Options types
pub use options::ReleaseOptions;

// Results types
pub use results::{
    AffectedPackageInfo, ChangeAnalysisResult, ChangeAnalysisWorkflowResult, ConfidenceLevel,
    DevelopmentResult, ImpactLevel, ReleaseResult, VersionRecommendation, VersioningWorkflowResult,
};

// Status types
pub use status::{SubStep, WorkflowProgress, WorkflowStatus, WorkflowStep};

// Data types
pub use data::PackageChangeFacts;

// Implementation structs
pub use development::DevelopmentWorkflow;
pub use integration::ChangesetHookIntegration;
pub use release::ReleaseWorkflow;
