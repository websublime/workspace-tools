//! Monorepo analysis module
//! 
//! This module provides comprehensive analysis capabilities for monorepos,
//! including dependency graph analysis, change detection, version management,
//! and package classification.

pub mod types;
mod analyzer;
mod diff;

#[cfg(test)]
mod tests;

// Explicit re-exports from types module
pub use types::{
    // Analyzer
    MonorepoAnalyzer,
    // Core
    MonorepoAnalysisResult,
    // Package manager
    PackageManagerAnalysis,
    // Package
    PackageClassificationResult, PackageInformation,
    // Dependency
    DependencyGraphAnalysis,
    // Registry
    RegistryAnalysisResult, RegistryInfo,
    // Workspace
    WorkspaceConfigAnalysis, WorkspacePatternAnalysis, PatternStatistics,
    // Upgrade
    UpgradeAnalysisResult, UpgradeInfo,
    // Diff
    ChangeAnalyzer, ChangeAnalysisResult, DiffAnalyzer, BranchComparisonResult,
    ChangeAnalysis, AffectedPackagesAnalysis, ChangeSignificanceResult,
    ComprehensiveChangeAnalysisResult,
};
