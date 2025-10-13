use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

/// Package information from registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Package name
    pub name: String,
    /// Latest version
    pub version: String,
    /// Package description
    pub description: Option<String>,
    /// Available versions
    pub versions: Vec<String>,
    /// Dist tags (latest, beta, etc.)
    pub dist_tags: HashMap<String, String>,
    /// Package maintainers
    pub maintainers: Vec<String>,
    /// Last modified timestamp
    pub modified: String,
}

/// Options for package publishing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishOptions {
    /// Package directory path
    pub package_path: PathBuf,
    /// Access level (public/restricted)
    pub access: String,
    /// Dist tag for this publish
    pub tag: String,
    /// Whether this is a dry run
    pub dry_run: bool,
    /// Custom registry URL override
    pub registry: Option<String>,
}

impl Default for PublishOptions {
    fn default() -> Self {
        Self {
            package_path: PathBuf::from("."),
            access: "public".to_string(),
            tag: "latest".to_string(),
            dry_run: false,
            registry: None,
        }
    }
}
