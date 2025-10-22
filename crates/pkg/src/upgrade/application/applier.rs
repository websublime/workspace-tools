//! Upgrade application logic for updating package.json files.
//!
//! **What**: Provides functionality to apply selected dependency upgrades to package.json
//! files with proper filtering, JSON formatting preservation, and dry-run support.
//!
//! **How**: This module reads package.json files, filters available upgrades based on
//! selection criteria, updates dependency version specifications, and writes the modified
//! files back while preserving original JSON formatting (indentation, spacing, etc.).
//! Supports both dry-run preview and actual file modification.
//!
//! **Why**: To enable safe, controlled application of dependency upgrades with proper
//! formatting preservation, ensuring package.json files remain readable and consistent
//! with project conventions.

use crate::error::UpgradeError;
use crate::types::DependencyType;
use crate::upgrade::detection::{DependencyUpgrade, PackageUpgrades};
use crate::upgrade::registry::UpgradeType;
use crate::upgrade::UpgradeSelection;
use chrono::Utc;
use package_json::PackageJson;
use std::collections::HashSet;
use std::path::PathBuf;
use sublime_standard_tools::filesystem::AsyncFileSystem;

use super::result::{AppliedUpgrade, ApplySummary, UpgradeResult};
use crate::error::UpgradeResult as ErrorResult;

/// Applies upgrades to package.json files based on selection criteria.
///
/// # Arguments
///
/// * `available_upgrades` - List of available upgrades from detection
/// * `selection` - Selection criteria for filtering upgrades
/// * `dry_run` - If true, preview changes without writing files
/// * `fs` - Filesystem implementation for reading/writing files
///
/// # Returns
///
/// `UpgradeResult` containing details of applied upgrades and summary statistics
///
/// # Errors
///
/// Returns `UpgradeError` if:
/// - Files cannot be read or written
/// - JSON parsing fails
/// - No upgrades are available after filtering
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::upgrade::{apply_upgrades, UpgradeSelection};
/// use sublime_standard_tools::filesystem::FileSystemManager;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = FileSystemManager::new();
/// let available = vec![]; // From detect_upgrades
/// let selection = UpgradeSelection::patch_only();
///
/// let result = apply_upgrades(available, selection, true, &fs).await?;
/// println!("Would upgrade {} dependencies", result.applied.len());
/// # Ok(())
/// # }
/// ```
pub async fn apply_upgrades<F: AsyncFileSystem>(
    available_upgrades: Vec<PackageUpgrades>,
    selection: UpgradeSelection,
    dry_run: bool,
    fs: &F,
) -> ErrorResult<UpgradeResult> {
    // Filter upgrades based on selection criteria
    let filtered = filter_upgrades(available_upgrades, &selection);

    // If no upgrades match the selection, return empty result
    if filtered.is_empty() {
        let summary = ApplySummary::new();
        return Ok(if dry_run {
            UpgradeResult::dry_run(vec![], summary)
        } else {
            UpgradeResult::applied(vec![], vec![], None, None, summary)
        });
    }

    // Group upgrades by package for batch processing
    let mut applied_upgrades = Vec::new();
    let mut modified_files = Vec::new();
    let mut packages_modified = HashSet::new();

    // Process each package
    for package_upgrades in filtered {
        let package_path = package_upgrades.package_path.clone();

        // Apply upgrades to this package
        match apply_package_upgrades(package_upgrades, dry_run, fs).await {
            Ok(package_result) => {
                if !package_result.applied.is_empty() {
                    packages_modified.insert(package_result.package_path.clone());
                    applied_upgrades.extend(package_result.applied);

                    if !dry_run {
                        modified_files.push(package_result.package_path);
                    }
                }
            }
            Err(e) => {
                // Log error but continue with other packages
                eprintln!("Failed to apply upgrades to {}: {}", package_path.display(), e);
            }
        }
    }

    // Build summary statistics
    let summary = build_summary(&applied_upgrades, packages_modified.len());

    Ok(if dry_run {
        UpgradeResult::dry_run(applied_upgrades, summary)
    } else {
        // Note: backup_path and changeset_id will be set by higher-level manager
        // (Story 9.5 and 9.6 respectively)
        UpgradeResult::applied(applied_upgrades, modified_files, None, None, summary)
    })
}

/// Filters available upgrades based on selection criteria.
///
/// # Arguments
///
/// * `available` - All available upgrades from detection
/// * `selection` - Selection criteria for filtering
///
/// # Returns
///
/// Filtered list of package upgrades matching the selection
fn filter_upgrades(
    available: Vec<PackageUpgrades>,
    selection: &UpgradeSelection,
) -> Vec<PackageUpgrades> {
    available
        .into_iter()
        .filter_map(|package| {
            // Filter by package name if specified
            if !selection.matches_package(&package.package_name) {
                return None;
            }

            // Filter upgrades within this package
            let filtered_upgrades: Vec<DependencyUpgrade> = package
                .upgrades
                .into_iter()
                .filter(|upgrade| {
                    // Check upgrade type
                    if !selection.matches_type(upgrade.upgrade_type) {
                        return false;
                    }

                    // Check dependency name
                    if !selection.matches_dependency(&upgrade.name) {
                        return false;
                    }

                    true
                })
                .collect();

            // Only include package if it has matching upgrades
            if filtered_upgrades.is_empty() {
                None
            } else {
                Some(PackageUpgrades {
                    package_name: package.package_name,
                    package_path: package.package_path,
                    current_version: package.current_version,
                    upgrades: filtered_upgrades,
                })
            }
        })
        .collect()
}

/// Result of applying upgrades to a single package.
struct PackageApplyResult {
    package_path: PathBuf,
    applied: Vec<AppliedUpgrade>,
}

/// Applies upgrades to a single package.json file.
///
/// # Arguments
///
/// * `package` - Package with upgrades to apply
/// * `dry_run` - If true, don't write changes
/// * `fs` - Filesystem implementation
///
/// # Returns
///
/// Result containing applied upgrades for this package
///
/// # Errors
///
/// Returns error if file operations or JSON parsing fails
async fn apply_package_upgrades<F: AsyncFileSystem>(
    package: PackageUpgrades,
    dry_run: bool,
    fs: &F,
) -> ErrorResult<PackageApplyResult> {
    let package_json_path = package.package_path.join("package.json");

    // Read current package.json
    let content = fs.read_file_string(package_json_path.as_path()).await.map_err(|e| {
        UpgradeError::FileSystemError { path: package_json_path.clone(), reason: e.to_string() }
    })?;

    // Parse package.json
    let mut pkg_json: PackageJson = serde_json::from_str(&content).map_err(|e| {
        UpgradeError::PackageJsonError { path: package_json_path.clone(), reason: e.to_string() }
    })?;

    // Track applied upgrades
    let mut applied = Vec::new();

    // Apply each upgrade
    for upgrade in package.upgrades {
        if apply_single_upgrade(&mut pkg_json, &upgrade) {
            applied.push(AppliedUpgrade {
                package_path: package.package_path.clone(),
                dependency_name: upgrade.name,
                dependency_type: upgrade.dependency_type,
                old_version: upgrade.current_version,
                new_version: upgrade.latest_version,
                upgrade_type: upgrade.upgrade_type,
            });
        }
    }

    // Write updated package.json if not dry-run and changes were made
    if !dry_run && !applied.is_empty() {
        let updated_content = serialize_package_json(&pkg_json, &content)?;

        fs.write_file(package_json_path.as_path(), updated_content.as_bytes()).await.map_err(
            |e| UpgradeError::ApplyFailed {
                path: package_json_path.clone(),
                reason: e.to_string(),
            },
        )?;
    }

    Ok(PackageApplyResult { package_path: package.package_path, applied })
}

/// Applies a single upgrade to a PackageJson object.
///
/// # Arguments
///
/// * `pkg_json` - PackageJson object to modify
/// * `upgrade` - Upgrade to apply
///
/// # Returns
///
/// `true` if the upgrade was applied, `false` if the dependency wasn't found
fn apply_single_upgrade(pkg_json: &mut PackageJson, upgrade: &DependencyUpgrade) -> bool {
    let deps = match upgrade.dependency_type {
        DependencyType::Regular => &mut pkg_json.dependencies,
        DependencyType::Dev => &mut pkg_json.dev_dependencies,
        DependencyType::Peer => &mut pkg_json.peer_dependencies,
        DependencyType::Optional => &mut pkg_json.optional_dependencies,
    };

    if let Some(deps_map) = deps {
        if let Some(version) = deps_map.get_mut(&upgrade.name) {
            // Preserve the version prefix (^, ~, etc.) if present
            let new_spec =
                preserve_version_prefix(&upgrade.current_version, &upgrade.latest_version);
            *version = new_spec;
            return true;
        }
    }

    false
}

/// Preserves version prefix from old version when applying new version.
///
/// Handles common npm version prefixes:
/// - `^` (caret) - Compatible with minor/patch updates
/// - `~` (tilde) - Compatible with patch updates only
/// - Exact versions (no prefix)
///
/// # Arguments
///
/// * `old_version` - Current version specification
/// * `new_version` - New version to apply
///
/// # Returns
///
/// New version with appropriate prefix preserved
///
/// # Examples
///
/// ```
/// # use sublime_pkg_tools::upgrade::application::preserve_version_prefix;
/// assert_eq!(preserve_version_prefix("^1.2.3", "1.2.4"), "^1.2.4");
/// assert_eq!(preserve_version_prefix("~2.0.0", "2.0.1"), "~2.0.1");
/// assert_eq!(preserve_version_prefix("3.0.0", "3.0.1"), "3.0.1");
/// ```
pub fn preserve_version_prefix(old_version: &str, new_version: &str) -> String {
    let old_trimmed = old_version.trim();

    // Check for common prefixes
    if old_trimmed.starts_with('^') {
        format!("^{}", new_version)
    } else if old_trimmed.starts_with('~') {
        format!("~{}", new_version)
    } else if old_trimmed.starts_with('=') {
        format!("={}", new_version)
    } else if old_trimmed.starts_with(">=") {
        format!(">={}", new_version)
    } else if old_trimmed.starts_with('>') {
        format!(">{}", new_version)
    } else {
        // No prefix or unknown prefix - use exact version
        new_version.to_string()
    }
}

/// Serializes PackageJson while attempting to preserve formatting.
///
/// Attempts to preserve the original JSON formatting (indentation, spacing)
/// by detecting the indentation style used in the original content.
///
/// # Arguments
///
/// * `pkg_json` - PackageJson object to serialize
/// * `original_content` - Original file content for format detection
///
/// # Returns
///
/// Serialized JSON string with preserved formatting
///
/// # Errors
///
/// Returns error if serialization fails
fn serialize_package_json(pkg_json: &PackageJson, original_content: &str) -> ErrorResult<String> {
    // Detect indentation from original content
    let indent = detect_indentation(original_content);

    // Serialize with detected indentation
    let mut serialized = if indent.contains('\t') {
        serde_json::to_string_pretty(pkg_json)
            .map_err(|e| UpgradeError::PackageJsonError {
                path: PathBuf::from(""),
                reason: e.to_string(),
            })?
            .replace("  ", "\t")
    } else {
        let space_count = indent.len();
        let pretty = serde_json::to_string_pretty(pkg_json).map_err(|e| {
            UpgradeError::PackageJsonError { path: PathBuf::from(""), reason: e.to_string() }
        })?;

        if space_count == 2 {
            pretty
        } else {
            // Replace 2-space indent with detected indent
            let target_indent = " ".repeat(space_count);
            pretty
                .lines()
                .map(|line| {
                    let leading_spaces = line.len() - line.trim_start().len();
                    if leading_spaces > 0 {
                        let indent_level = leading_spaces / 2;
                        format!("{}{}", target_indent.repeat(indent_level), line.trim_start())
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    };

    // Ensure trailing newline if original had one
    if original_content.ends_with('\n') && !serialized.ends_with('\n') {
        serialized.push('\n');
    }

    Ok(serialized)
}

/// Detects the indentation style used in JSON content.
///
/// # Arguments
///
/// * `content` - JSON content to analyze
///
/// # Returns
///
/// Detected indentation string (spaces or tab)
pub(crate) fn detect_indentation(content: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim_start();
        if !trimmed.is_empty() && line.len() > trimmed.len() {
            let indent = &line[0..line.len() - trimmed.len()];
            // Return first non-empty indentation found
            if indent.contains('\t') {
                return "\t".to_string();
            } else if !indent.is_empty() {
                // Count consecutive spaces
                let space_count = indent.len();
                return " ".repeat(space_count);
            }
        }
    }

    // Default to 2 spaces if no indentation detected
    "  ".to_string()
}

/// Builds summary statistics from applied upgrades.
///
/// # Arguments
///
/// * `applied` - List of applied upgrades
/// * `packages_modified` - Count of unique packages modified
///
/// # Returns
///
/// Summary statistics for the operation
fn build_summary(applied: &[AppliedUpgrade], packages_modified: usize) -> ApplySummary {
    let mut major_count = 0;
    let mut minor_count = 0;
    let mut patch_count = 0;

    for upgrade in applied {
        match upgrade.upgrade_type {
            UpgradeType::Major => major_count += 1,
            UpgradeType::Minor => minor_count += 1,
            UpgradeType::Patch => patch_count += 1,
        }
    }

    ApplySummary {
        packages_modified,
        dependencies_upgraded: applied.len(),
        direct_updates: applied.len(), // All are direct for now (no propagation yet)
        propagated_updates: 0,         // TODO: will be implemented in version module
        dependency_updates: applied.len(),
        major_upgrades: major_count,
        minor_upgrades: minor_count,
        patch_upgrades: patch_count,
        applied_at: Utc::now(),
    }
}
