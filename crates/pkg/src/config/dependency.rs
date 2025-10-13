use serde::{Deserialize, Serialize};

/// Dependency management configuration.
///
/// Controls dependency analysis, propagation, and circular dependency detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConfig {
    /// Whether to propagate version updates to dependents
    pub propagate_updates: bool,

    /// Whether to propagate dev dependency updates
    pub propagate_dev_dependencies: bool,

    /// Maximum depth for dependency propagation
    pub max_propagation_depth: u32,

    /// Whether to detect circular dependencies
    pub detect_circular: bool,

    /// Whether to fail on circular dependencies
    pub fail_on_circular: bool,

    /// Version bump type for dependency updates (patch/minor/major)
    pub dependency_update_bump: String,

    /// Whether to include peer dependencies in analysis
    pub include_peer_dependencies: bool,

    /// Whether to include optional dependencies in analysis
    pub include_optional_dependencies: bool,
}

impl Default for DependencyConfig {
    fn default() -> Self {
        Self {
            propagate_updates: true,
            propagate_dev_dependencies: false,
            max_propagation_depth: 10,
            detect_circular: true,
            fail_on_circular: true,
            dependency_update_bump: "patch".to_string(),
            include_peer_dependencies: false,
            include_optional_dependencies: false,
        }
    }
}
