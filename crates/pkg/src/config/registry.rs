use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Registry configuration.
///
/// Controls NPM registry interactions including authentication,
/// publishing, and package information retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Default registry URL
    pub url: String,

    /// Request timeout in seconds
    pub timeout: u64,

    /// Number of retry attempts for failed requests
    pub retry_attempts: u32,

    /// Whether to use .npmrc for authentication
    pub use_npmrc: bool,

    /// Custom registry configurations
    pub registries: HashMap<String, CustomRegistryConfig>,

    /// Default publish access (public/restricted)
    pub default_access: String,

    /// Whether to skip registry checks during dry runs
    pub skip_checks_in_dry_run: bool,
}

/// Custom registry configuration.
///
/// Allows configuration of alternative registries with specific
/// authentication and access patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRegistryConfig {
    /// Registry URL
    pub url: String,

    /// Authentication type (token, basic, none)
    pub auth_type: String,

    /// Token or username for authentication
    pub auth_token: Option<String>,

    /// Password for basic authentication
    pub auth_password: Option<String>,

    /// Request timeout override
    pub timeout: Option<u64>,

    /// Registry-specific access level
    pub default_access: Option<String>,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            url: "https://registry.npmjs.org".to_string(),
            timeout: 30,
            retry_attempts: 3,
            use_npmrc: true,
            registries: HashMap::new(),
            default_access: "public".to_string(),
            skip_checks_in_dry_run: true,
        }
    }
}

impl Default for CustomRegistryConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            auth_type: "token".to_string(),
            auth_token: None,
            auth_password: None,
            timeout: None,
            default_access: None,
        }
    }
}
