//! Package classification and information types

use serde_json::Value;
use std::path::PathBuf;

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
