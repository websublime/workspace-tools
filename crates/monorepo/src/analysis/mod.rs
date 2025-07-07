//! Monorepo analysis module
//!
//! This module provides streamlined analysis capabilities optimized for CLI/daemon usage
//! with sub-second performance targets. Features dependency graph analysis, change detection,
//! and impact analysis using direct base crate integration.
//!
//! # Main Types
//!
//! - [`MonorepoAnalyzer`] - Main analysis interface for dependency graphs and changes
//! - [`ChangeAnalysis`] - Comprehensive change analysis results
//!
//! # Performance
//!
//! - **Analysis Time**: < 1s for real-time CLI feedback
//! - **Memory Usage**: Efficient direct borrowing patterns
//! - **Hot Path Optimization**: Minimized filesystem I/O and allocations
//!
//! # Examples
//!
//! ```rust
//! use sublime_monorepo_tools::{MonorepoProject, MonorepoAnalyzer};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let project = MonorepoProject::new(".")?;
//! let analyzer = MonorepoAnalyzer::new(&project);
//!
//! // Detect changes since last commit
//! let changes = analyzer.detect_changes_since("HEAD~1", None)?;
//! println!("Found {} affected packages", changes.package_changes.len());
//!
//! // Analyze dependency relationships
//! for change in &changes.package_changes {
//!     println!("Package: {} ({:?})", change.package_name, change.change_type);
//! }
//! # Ok(())
//! # }
//! ```

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
