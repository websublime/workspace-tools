//! Upgrade application module for applying dependency upgrades.
//!
//! **What**: Provides functionality to apply selected dependency upgrades to package.json
//! files with filtering, formatting preservation, and dry-run support.
//!
//! **How**: This module implements the logic to read package.json files, apply version
//! updates based on selection criteria, preserve JSON formatting, and write changes back
//! to disk. It supports both dry-run preview and actual file modification, with proper
//! error handling and detailed result reporting.
//!
//! **Why**: To enable safe, controlled application of dependency upgrades with fine-grained
//! filtering, ensuring package.json files remain properly formatted and changes are
//! transparent and reversible.
//!
//! # Module Structure
//!
//! - `applier`: Core logic for applying upgrades to package.json files
//! - `selection`: Selection criteria for filtering which upgrades to apply
//! - `result`: Result types containing applied upgrade details and statistics
//!
//! # Examples
//!
//! ```rust,ignore
//! use sublime_pkg_tools::upgrade::{
//!     apply_upgrades, UpgradeSelection, detect_upgrades, DetectionOptions
//! };
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let fs = FileSystemManager::new();
//! let workspace_root = PathBuf::from(".");
//!
//! // Detect available upgrades
//! let options = DetectionOptions::all();
//! let available = detect_upgrades(&workspace_root, options, &fs).await?;
//!
//! // Preview patch upgrades only (dry-run)
//! let selection = UpgradeSelection::patch_only();
//! let preview = apply_upgrades(available.packages, selection.clone(), true, &fs).await?;
//! println!("Would upgrade {} dependencies", preview.applied.len());
//!
//! // Apply the upgrades for real
//! let result = apply_upgrades(available.packages, selection, false, &fs).await?;
//! println!("Upgraded {} dependencies in {} packages",
//!     result.dependencies_upgraded(),
//!     result.packages_modified());
//! # Ok(())
//! # }
//! ```

pub(crate) mod applier;
mod changeset;
pub(crate) mod result;
mod selection;

#[cfg(test)]
mod tests;

// Re-export public API
pub use applier::apply_upgrades;
pub use changeset::apply_with_changeset;
pub use result::{AppliedUpgrade, ApplySummary, UpgradeResult};
pub use selection::UpgradeSelection;
