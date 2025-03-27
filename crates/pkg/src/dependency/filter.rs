#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DependencyFilter {
    /// Include only production dependencies
    ProductionOnly,
    /// Include production and development dependencies
    WithDevelopment,
    /// Include production, development, and optional dependencies
    AllDependencies,
}

impl Default for DependencyFilter {
    fn default() -> Self {
        Self::WithDevelopment
    }
}
