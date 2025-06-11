//! Package upgrade analysis types

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