use serde::{Deserialize, Serialize};

/// Registry authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryAuth {
    /// Authentication type
    pub auth_type: AuthType,
    /// Authentication token or username
    pub token: Option<String>,
    /// Password for basic authentication
    pub password: Option<String>,
}

/// Authentication type for registry access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    /// No authentication
    None,
    /// Token-based authentication
    Token,
    /// Basic username/password authentication
    Basic,
    /// Authentication from .npmrc file
    Npmrc,
}

impl Default for RegistryAuth {
    fn default() -> Self {
        Self { auth_type: AuthType::None, token: None, password: None }
    }
}
