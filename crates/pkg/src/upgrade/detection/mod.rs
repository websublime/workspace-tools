//! Upgrade detection module.
//!
//! **What**: Provides functionality to detect available upgrades for external npm packages
//! by scanning package.json files and querying package registries.
//!
//! **How**: This module scans the workspace for package.json files, extracts external
//! dependencies (filtering out workspace:, file:, link:, and portal: protocols), queries
//! npm registries concurrently for available versions, and classifies upgrades by type
//! (major, minor, patch).
//!
//! **Why**: To enable developers to discover available dependency upgrades with fine-grained
//! control over what to detect, supporting both security patches and feature updates while
//! providing clear classification of upgrade impact.

mod detector;

#[cfg(test)]
mod tests;

// Re-export public API
pub use detector::{
    detect_upgrades, DependencyUpgrade, DetectionOptions, PackageUpgrades, UpgradePreview,
    UpgradeSummary, VersionInfo,
};
