//! Apply upgrades with automatic changeset creation.
//!
//! **What**: Provides a high-level function for applying dependency upgrades with
//! automatic changeset creation, integrating the upgrade application with the
//! changeset workflow.
//!
//! **How**: This module wraps the core upgrade application logic and adds automatic
//! changeset creation based on configuration. It determines which packages were
//! affected and creates or updates a changeset accordingly.
//!
//! **Why**: To provide a seamless integration between dependency upgrades and the
//! changeset workflow, ensuring all upgrades are properly tracked for versioning
//! and release management.

use super::creator::create_changeset_for_upgrades;
use crate::changeset::ChangesetManager;
use crate::config::UpgradeConfig;
use crate::error::UpgradeResult;
use crate::upgrade::UpgradeSelection;
use crate::upgrade::detection::PackageUpgrades;
use std::collections::HashSet;
use std::path::Path;
use sublime_standard_tools::filesystem::AsyncFileSystem;

use crate::upgrade::application::applier::apply_upgrades;
use crate::upgrade::application::result::UpgradeResult as UpgradeResultType;

/// Applies upgrades with automatic changeset creation.
///
/// This function applies dependency upgrades and optionally creates or updates a changeset
/// based on the configuration. It's the recommended entry point for upgrade operations
/// that should be tracked through the changeset workflow.
///
/// # Workflow
///
/// 1. Apply upgrades to package.json files
/// 2. If `auto_changeset` is enabled and not dry-run:
///    - Get current git branch
///    - Create or update changeset with affected packages
///    - Set version bump type from config
/// 3. Return result with changeset ID
///
/// # Arguments
///
/// * `available_upgrades` - List of available upgrades from detection
/// * `selection` - Selection criteria for filtering upgrades
/// * `dry_run` - If true, preview changes without writing files or creating changesets
/// * `workspace_root` - Root directory of the workspace
/// * `config` - Upgrade configuration including changeset settings
/// * `changeset_manager` - Manager instance for changeset operations (optional)
/// * `fs` - Filesystem implementation for reading/writing files
///
/// # Returns
///
/// `UpgradeResult` containing details of applied upgrades, summary, and changeset ID
///
/// # Errors
///
/// Returns `UpgradeError` if:
/// - Upgrade application fails
/// - Changeset creation fails (only if auto_changeset is enabled)
/// - Configuration is invalid
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::upgrade::{
///     apply_with_changeset, detect_upgrades, DetectionOptions, UpgradeSelection
/// };
/// use sublime_pkg_tools::changeset::ChangesetManager;
/// use sublime_pkg_tools::config::{PackageToolsConfig, UpgradeConfig};
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace_root = PathBuf::from(".");
/// let fs = FileSystemManager::new();
/// let config = PackageToolsConfig::default();
///
/// // Detect upgrades
/// let options = DetectionOptions::all();
/// let available = detect_upgrades(&workspace_root, options, &fs).await?;
///
/// // Apply with automatic changeset creation
/// let manager = ChangesetManager::new(&workspace_root).await?;
/// let selection = UpgradeSelection::patch_only();
/// let result = apply_with_changeset(
///     available.packages,
///     selection,
///     false, // Not dry run
///     &workspace_root,
///     &config.upgrade,
///     Some(&manager),
///     &fs
/// ).await?;
///
/// if let Some(changeset_id) = result.changeset_id {
///     println!("Upgrades tracked in changeset: {}", changeset_id);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn apply_with_changeset<F, S>(
    available_upgrades: Vec<PackageUpgrades>,
    selection: UpgradeSelection,
    dry_run: bool,
    workspace_root: &Path,
    config: &UpgradeConfig,
    changeset_manager: Option<&ChangesetManager<S>>,
    fs: &F,
) -> UpgradeResult<UpgradeResultType>
where
    F: AsyncFileSystem,
    S: crate::changeset::ChangesetStorage + Send + Sync,
{
    // Apply upgrades using the core applier
    let mut result = apply_upgrades(available_upgrades, selection, dry_run, fs).await?;

    // If not dry run and auto_changeset is enabled, create changeset
    if !dry_run
        && config.auto_changeset
        && let Some(manager) = changeset_manager
    {
        // Extract affected package names from the result
        let affected_packages = extract_affected_packages(&result);

        if !affected_packages.is_empty() {
            // Create or update changeset
            let changeset_id = create_changeset_for_upgrades(
                manager,
                affected_packages,
                workspace_root,
                &config.changeset_bump,
            )
            .await?;

            // Update result with changeset ID
            result.changeset_id = changeset_id;
        }
    }

    Ok(result)
}

/// Extracts unique package names from upgrade results.
///
/// # Arguments
///
/// * `result` - Upgrade result containing applied upgrades
///
/// # Returns
///
/// Set of unique package names that were modified
pub fn extract_affected_packages(result: &UpgradeResultType) -> HashSet<String> {
    let mut packages = HashSet::new();

    for upgrade in &result.applied {
        // Extract package name from package path
        if let Some(package_name) = extract_package_name(&upgrade.package_path) {
            packages.insert(package_name);
        }
    }

    packages
}

/// Extracts package name from a package path.
///
/// This function attempts to read the package.json file to get the package name.
/// If that fails, it uses the directory name as a fallback.
///
/// # Arguments
///
/// * `package_path` - Path to the package directory
///
/// # Returns
///
/// Package name if it can be determined, None otherwise
fn extract_package_name(package_path: &Path) -> Option<String> {
    // Try to read package.json to get the actual package name
    let package_json_path = package_path.join("package.json");

    if let Ok(content) = std::fs::read_to_string(&package_json_path)
        && let Ok(pkg_json) = serde_json::from_str::<serde_json::Value>(&content)
        && let Some(name) = pkg_json.get("name").and_then(|v| v.as_str())
    {
        return Some(name.to_string());
    }

    // Fallback to directory name
    package_path.file_name().and_then(|n| n.to_str()).map(String::from)
}
