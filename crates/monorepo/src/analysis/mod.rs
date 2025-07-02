//! Monorepo analysis module
//!
//! This module provides comprehensive analysis capabilities for monorepos,
//! including dependency graph analysis, change detection, version management,
//! and package classification.

mod analyzer;
mod diff;
pub mod types;

// Explicit re-exports from types module
pub use types::{
    AffectedPackagesAnalysis,
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
