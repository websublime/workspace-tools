//! Monorepo analysis module
//!
//! This module provides comprehensive analysis capabilities for monorepos,
//! including dependency graph analysis, change detection, version management,
//! and package classification.

mod analyzer;
mod diff;
pub mod types;

#[cfg(test)]
mod tests;

// Essential re-exports for CLI consumption
pub use types::{
    BranchComparisonResult,
    ChangeAnalysis,
    ChangeAnalysisResult,
    // Diff
    ChangeAnalyzer,
    ChangeSignificanceResult,
    ComprehensiveChangeAnalysisResult,
    // Dependency
    DependencyGraphAnalysis,
    DiffAnalyzer,
    // Core
    MonorepoAnalysisResult,
    // Analyzer
    MonorepoAnalyzer,
    // Package
    PackageClassificationResult,
    PackageInformation,
    // Package manager
    PackageManagerAnalysis,
    PatternStatistics,
    // Registry
    RegistryAnalysisResult,
    RegistryInfo,
    // Upgrade
    UpgradeAnalysisResult,
    UpgradeInfo,
    // Workspace
    WorkspaceConfigAnalysis,
    WorkspacePatternAnalysis,
};
