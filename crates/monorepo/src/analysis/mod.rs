//! Monorepo analysis module
//! 
//! This module provides comprehensive analysis capabilities for monorepos,
//! including dependency graph analysis, change detection, and package classification.

mod types;
mod analyzer;
mod change_detector;
mod change_rules;
mod change_engine;

pub use types::*;
pub use analyzer::MonorepoAnalyzer;
pub use change_detector::{
    ChangeDetector,
    PackageChange,
    PackageChangeType,
    ChangeSignificance,
};
pub use change_rules::*;
pub use change_engine::ChangeDetectionEngine;