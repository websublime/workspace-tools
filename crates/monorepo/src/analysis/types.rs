//! Types for monorepo analysis results

use std::collections::HashMap;
use std::path::PathBuf;
use sublime_standard_tools::monorepo::MonorepoKind;
use serde_json::Value;

/// Result of monorepo analysis
#[derive(Debug, Clone)]
pub struct MonorepoAnalysisResult {
    /// Type of monorepo detected
    pub kind: MonorepoKind,
    
    /// Root path of the monorepo
    pub root_path: PathBuf,
    
    /// Package manager analysis
    pub package_manager: PackageManagerAnalysis,
    
    /// Package classification
    pub packages: PackageClassificationResult,
    
    /// Dependency graph analysis
    pub dependency_graph: DependencyGraphAnalysis,
    
    /// Registry analysis
    pub registries: RegistryAnalysisResult,
    
    /// Workspace configuration analysis
    pub workspace_config: WorkspaceConfigAnalysis,
}

/// Analysis of the package manager
#[derive(Debug, Clone)]
pub struct PackageManagerAnalysis {
    /// Type of package manager
    pub kind: sublime_standard_tools::monorepo::PackageManagerKind,
    
    /// Version of the package manager
    pub version: String,
    
    /// Path to lock file
    pub lock_file: PathBuf,
    
    /// Configuration files found
    pub config_files: Vec<PathBuf>,
    
    /// Workspace configuration (raw JSON)
    pub workspaces_config: Value,
}

/// Classification of packages in the monorepo
#[derive(Debug, Clone)]
pub struct PackageClassificationResult {
    /// Internal packages (part of the monorepo)
    pub internal_packages: Vec<PackageInformation>,
    
    /// External dependencies across all packages
    pub external_dependencies: Vec<String>,
    
    /// Development dependencies
    pub dev_dependencies: Vec<String>,
    
    /// Peer dependencies
    pub peer_dependencies: Vec<String>,
}

/// Detailed information about a package
#[derive(Debug, Clone)]
pub struct PackageInformation {
    /// Package name
    pub name: String,
    
    /// Package version
    pub version: String,
    
    /// Absolute path to package
    pub path: PathBuf,
    
    /// Path relative to monorepo root
    pub relative_path: PathBuf,
    
    /// Raw package.json content
    pub package_json: Value,
    
    /// Whether this is an internal package
    pub is_internal: bool,
    
    /// Direct dependencies
    pub dependencies: Vec<String>,
    
    /// Development dependencies
    pub dev_dependencies: Vec<String>,
    
    /// Workspace dependencies (internal)
    pub workspace_dependencies: Vec<String>,
    
    /// Packages that depend on this one
    pub dependents: Vec<String>,
}

/// Analysis of the dependency graph
#[derive(Debug, Clone)]
pub struct DependencyGraphAnalysis {
    /// Total number of nodes
    pub node_count: usize,
    
    /// Total number of edges
    pub edge_count: usize,
    
    /// Whether the graph has cycles
    pub has_cycles: bool,
    
    /// Detected circular dependencies
    pub cycles: Vec<Vec<String>>,
    
    /// Packages with version conflicts
    pub version_conflicts: HashMap<String, Vec<String>>,
    
    /// Packages that can be upgraded
    pub upgradable: HashMap<String, Vec<(String, String)>>,
    
    /// Maximum depth of the dependency tree
    pub max_depth: usize,
    
    /// Packages with the most dependencies
    pub most_dependencies: Vec<(String, usize)>,
    
    /// Packages with the most dependents
    pub most_dependents: Vec<(String, usize)>,
}

/// Analysis of configured registries
#[derive(Debug, Clone)]
pub struct RegistryAnalysisResult {
    /// Default registry URL
    pub default_registry: String,
    
    /// All configured registries
    pub registries: Vec<RegistryInfo>,
    
    /// Scoped registries
    pub scoped_registries: HashMap<String, String>,
    
    /// Authentication status for each registry
    pub auth_status: HashMap<String, bool>,
}

/// Information about a configured registry
#[derive(Debug, Clone)]
pub struct RegistryInfo {
    /// Registry URL
    pub url: String,
    
    /// Registry type
    pub registry_type: String,
    
    /// Whether authentication is configured
    pub has_auth: bool,
    
    /// Scopes associated with this registry
    pub scopes: Vec<String>,
}

/// Analysis of workspace configuration
#[derive(Debug, Clone)]
pub struct WorkspaceConfigAnalysis {
    /// Workspace patterns configured
    pub patterns: Vec<String>,
    
    /// Number of packages matching patterns
    pub matched_packages: usize,
    
    /// Packages not matching any pattern
    pub orphaned_packages: Vec<String>,
    
    /// Whether nohoist is configured (for Yarn)
    pub has_nohoist: bool,
    
    /// Nohoist patterns if configured
    pub nohoist_patterns: Vec<String>,
}

/// Result of available upgrades analysis
#[derive(Debug, Clone)]
pub struct UpgradeAnalysisResult {
    /// Total packages analyzed
    pub total_packages: usize,
    
    /// Packages with available upgrades
    pub upgradable_count: usize,
    
    /// Major upgrades available
    pub major_upgrades: Vec<UpgradeInfo>,
    
    /// Minor upgrades available
    pub minor_upgrades: Vec<UpgradeInfo>,
    
    /// Patch upgrades available
    pub patch_upgrades: Vec<UpgradeInfo>,
    
    /// Packages at latest version
    pub up_to_date: Vec<String>,
}

/// Information about an available upgrade
#[derive(Debug, Clone)]
pub struct UpgradeInfo {
    /// Package name
    pub package_name: String,
    
    /// Dependency name
    pub dependency_name: String,
    
    /// Current version
    pub current_version: String,
    
    /// Available version
    pub available_version: String,
    
    /// Type of upgrade
    pub upgrade_type: String,
}

/// Analysis of workspace patterns configuration
#[derive(Debug, Clone)]
pub struct WorkspacePatternAnalysis {
    /// Patterns defined in configuration
    pub config_patterns: Vec<String>,
    
    /// Auto-detected patterns from workspace structure
    pub auto_detected_patterns: Vec<String>,
    
    /// Effective patterns (those that actually match packages)
    pub effective_patterns: Vec<String>,
    
    /// Validation errors found in configuration
    pub validation_errors: Vec<String>,
    
    /// Statistics for each pattern
    pub pattern_statistics: Vec<PatternStatistics>,
    
    /// Packages that don't match any pattern
    pub orphaned_packages: Vec<String>,
}

/// Statistics for a workspace pattern
#[derive(Debug, Clone)]
pub struct PatternStatistics {
    /// The pattern string
    pub pattern: String,
    
    /// Number of packages this pattern matches
    pub matches: usize,
    
    /// Whether this pattern is effective (matches > 0)
    pub is_effective: bool,
    
    /// Specificity score for prioritization
    pub specificity: u32,
}