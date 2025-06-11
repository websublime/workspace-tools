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

pub use types::*;
pub use analyzer::MonorepoAnalyzer;
pub use diff::{
    DiffAnalyzer,
    BranchComparisonResult,
    ChangeAnalysis,
    AffectedPackagesAnalysis,
    ChangeSignificanceResult,
    ChangeAnalyzer,
    ChangeAnalysisResult,
    PackageChange as DiffPackageChange,
};
