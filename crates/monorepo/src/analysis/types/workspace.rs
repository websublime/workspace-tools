//! Workspace configuration analysis types

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