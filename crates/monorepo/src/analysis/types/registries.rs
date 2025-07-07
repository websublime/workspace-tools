//! Registry analysis types

use std::collections::HashMap;

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
