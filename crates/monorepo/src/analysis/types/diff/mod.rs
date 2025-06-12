//! Diff analysis type definitions
//!
//! This module contains all type definitions related to diff analysis and change detection.

use crate::changes::{ChangeSignificance, PackageChangeType};
use crate::core::MonorepoProject;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use sublime_git_tools::{GitChangedFile, GitFileStatus};
use chrono;

/// Trait for analyzing specific types of file changes
pub trait ChangeAnalyzer: Send + Sync {
    /// Check if this analyzer can handle the given file
    fn can_analyze(&self, file_path: &str) -> bool;

    /// Analyze a file change and return analysis result
    fn analyze_change(&self, change: &GitChangedFile) -> ChangeAnalysisResult;
}

/// Result of analyzing a single file change
#[derive(Debug, Clone)]
pub struct ChangeAnalysisResult {
    /// Type of change detected
    pub change_type: PackageChangeType,
    /// Significance of the change
    pub significance: ChangeSignificance,
    /// Additional context about the change
    pub context: Vec<String>,
}

/// Analyzer for detecting and analyzing differences between branches and commits
pub struct DiffAnalyzer {
    /// Reference to the monorepo project
    pub(crate) project: Arc<MonorepoProject>,
    /// Collection of change analyzers for different file types
    pub(crate) analyzers: Vec<Box<dyn ChangeAnalyzer>>,
}

/// Result of comparing two branches
#[derive(Debug, Clone)]
pub struct BranchComparisonResult {
    /// Base branch name
    pub base_branch: String,
    /// Target branch name
    pub target_branch: String,
    /// Files that changed between branches
    pub changed_files: Vec<GitChangedFile>,
    /// Names of affected packages
    pub affected_packages: Vec<String>,
    /// Common ancestor commit
    pub merge_base: String,
    /// Potential merge conflicts
    pub conflicts: Vec<String>,
}

/// Analysis of changes between commits or branches
#[derive(Debug, Clone, Default)]
pub struct ChangeAnalysis {
    /// Starting reference
    pub from_ref: String,
    /// Ending reference
    pub to_ref: String,
    /// All changed files
    pub changed_files: Vec<GitChangedFile>,
    /// Changes grouped by package
    pub package_changes: Vec<PackageChange>,
    /// Analysis of affected packages including dependents
    pub affected_packages: AffectedPackagesAnalysis,
    /// Significance analysis for each change
    pub significance_analysis: Vec<ChangeSignificanceResult>,
}

/// Information about changes to a specific package
#[derive(Debug, Clone)]
pub struct PackageChange {
    /// Name of the changed package
    pub package_name: String,
    /// Files that changed in this package
    pub changed_files: Vec<GitChangedFile>,
    /// Type of change detected
    pub change_type: PackageChangeType,
    /// Significance level of the change
    pub significance: ChangeSignificance,
    /// Suggested version bump
    pub suggested_version_bump: crate::config::VersionBumpType,
}

/// Analysis of how changes affect packages in the monorepo
#[derive(Debug, Clone, Default)]
pub struct AffectedPackagesAnalysis {
    /// Packages directly changed
    pub directly_affected: Vec<String>,
    /// Packages affected through dependencies
    pub dependents_affected: Vec<String>,
    /// Graph showing change propagation
    pub change_propagation_graph: HashMap<String, Vec<String>>,
    /// Impact scores for each package
    pub impact_scores: HashMap<String, f32>,
    /// Total number of affected packages
    pub total_affected_count: usize,
}

/// Analysis of the significance of changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeSignificanceResult {
    /// Package name
    pub package_name: String,
    /// Original significance before analysis
    pub original_significance: ChangeSignificance,
    /// Final significance after analysis
    pub final_significance: ChangeSignificance,
    /// Reasons for significance determination
    pub reasons: Vec<String>,
    /// Suggested version bump
    pub suggested_version_bump: crate::config::VersionBumpType,
}

/// Comprehensive result of change analysis
#[derive(Debug, Clone)]
pub struct ComprehensiveChangeAnalysisResult {
    /// The commit or reference that was analyzed against
    pub since_ref: String,
    /// The target commit or reference (None for current state)
    pub until_ref: Option<String>,
    /// Detected changes
    pub changes: ChangeAnalysis,
    /// Analysis of affected packages
    pub affected_packages: AffectedPackagesAnalysis,
    /// Significance analysis
    pub significance: ChangeSignificanceResult,
    /// Timestamp when the analysis was performed
    pub analysis_timestamp: chrono::DateTime<chrono::Utc>,
}