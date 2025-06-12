//! Analysis type definitions module
//!
//! This module contains all analysis-related type definitions organized
//! in separate files for better maintainability and consistency.
//!
//! The module is organized as follows:
//! - `core`: Core analysis result types
//! - `package`: Package manager analysis types (submodule structure)
//! - `packages`: Package classification and information types
//! - `dependency`: Dependency graph analysis types (submodule structure)
//! - `registries`: Registry analysis types
//! - `workspace`: Workspace configuration analysis types
//! - `upgrades`: Package upgrade analysis types
//! - `diff`: Diff analysis and change detection types

mod analyzer;
mod core;
pub mod package;
mod packages;
pub mod dependency;
mod registries;
mod workspace;
mod upgrades;
pub mod diff;

// Explicit exports to avoid wildcard re-exports
pub use analyzer::MonorepoAnalyzer;

// Core types
pub use core::MonorepoAnalysisResult;

// Package manager types
pub use package::PackageManagerAnalysis;

// Package types
pub use packages::{PackageClassificationResult, PackageInformation};

// Dependency graph types
pub use dependency::DependencyGraphAnalysis;

// Registry types
pub use registries::{RegistryAnalysisResult, RegistryInfo};

// Workspace types
pub use workspace::{WorkspaceConfigAnalysis, WorkspacePatternAnalysis, PatternStatistics};

// Upgrade types
pub use upgrades::{UpgradeAnalysisResult, UpgradeInfo};

// Diff types
pub use diff::{
    ChangeAnalyzer, ChangeAnalysisResult, DiffAnalyzer, BranchComparisonResult,
    ChangeAnalysis, PackageChange, AffectedPackagesAnalysis, ChangeSignificanceResult,
    ComprehensiveChangeAnalysisResult
};