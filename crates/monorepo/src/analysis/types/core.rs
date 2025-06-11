//! Core analysis result types

use std::path::PathBuf;
use sublime_standard_tools::monorepo::MonorepoKind;

/// Result of monorepo analysis
#[derive(Debug, Clone)]
pub struct MonorepoAnalysisResult {
    /// Type of monorepo detected
    pub kind: MonorepoKind,
    
    /// Root path of the monorepo
    pub root_path: PathBuf,
    
    /// Package manager analysis
    pub package_manager: super::PackageManagerAnalysis,
    
    /// Package classification
    pub packages: super::PackageClassificationResult,
    
    /// Dependency graph analysis
    pub dependency_graph: super::DependencyGraphAnalysis,
    
    /// Registry analysis
    pub registries: super::RegistryAnalysisResult,
    
    /// Workspace configuration analysis
    pub workspace_config: super::WorkspaceConfigAnalysis,
}