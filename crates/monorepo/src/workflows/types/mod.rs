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

mod options;
mod results;
mod status;
mod data;

// Explicit exports to avoid wildcard re-exports

// Options types
pub use options::ReleaseOptions;

// Results types
pub use results::{
    ReleaseResult, DevelopmentResult, ChangeAnalysisResult,
    AffectedPackageInfo, VersionRecommendation, VersioningWorkflowResult,
    ImpactLevel, ConfidenceLevel, ChangeAnalysisWorkflowResult
};

// Status types
pub use status::{WorkflowStep, WorkflowProgress, WorkflowStatus};

// Data types
pub use data::PackageChangeFacts;
