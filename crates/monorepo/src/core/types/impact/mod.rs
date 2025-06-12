//! Impact analysis types

pub mod analysis;
pub use analysis::{VersionImpactAnalysis, PackageImpactAnalysis, BreakingChangeAnalysis, DependencyChainImpact};