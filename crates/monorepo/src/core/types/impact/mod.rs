//! Impact analysis types

pub mod analysis;
pub use analysis::{
    BreakingChangeAnalysis, DependencyChainImpact, PackageImpactAnalysis, VersionImpactAnalysis,
};
