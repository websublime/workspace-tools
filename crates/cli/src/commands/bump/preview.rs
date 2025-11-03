//! Bump preview command implementation.
//!
//! This module implements the bump preview (dry-run) functionality which shows
//! what version bumps would be applied without modifying any files.
//!
//! # What
//!
//! Provides the `execute_bump_preview` function that:
//! - Loads all active changesets
//! - Calculates version bumps using `VersionResolver`
//! - Determines which packages will bump based on versioning strategy
//! - Displays preview in table or JSON format
//! - Shows changesets being processed and their details
//! - Does NOT modify any files (safe dry-run operation)
//!
//! # How
//!
//! The command flow:
//! 1. Loads workspace configuration to determine versioning strategy
//! 2. Creates ChangesetManager and loads all pending changesets
//! 3. If no changesets exist, reports "nothing to bump"
//! 4. Combines all changesets into a single merged changeset for version resolution
//! 5. Uses VersionResolver to calculate all version bumps
//! 6. Determines which packages will actually bump based on strategy:
//!    - Independent: Only packages in changeset.packages
//!    - Unified: All workspace packages
//! 7. Builds BumpSnapshot with all package information
//! 8. Outputs as formatted table (human mode) or JSON (automation mode)
//!
//! ## Strategy Handling
//!
//! ### Independent Strategy
//! - Packages ONLY bump if listed in changeset packages
//! - Other packages remain unchanged even if they're dependencies
//! - Preview shows "no changes" reason for non-changeset packages
//!
//! ### Unified Strategy
//! - ALL packages bump when ANY changeset exists
//! - All packages receive the same version
//! - Highest bump type from all changesets is used
//!
//! # Why
//!
//! Preview mode is essential for:
//! - Safe verification before applying version bumps
//! - Understanding impact of pending changesets
//! - CI/CD integration for automated release planning
//! - Reviewing what will change before commit
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::commands::bump::execute_bump_preview;
//! use sublime_cli_tools::cli::commands::BumpArgs;
//! use sublime_cli_tools::output::{Output, OutputFormat};
//! use std::io;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let args = BumpArgs {
//!     dry_run: true,
//!     execute: false,
//!     snapshot: false,
//!     snapshot_format: None,
//!     prerelease: None,
//!     packages: None,
//!     git_tag: false,
//!     git_push: false,
//!     git_commit: false,
//!     no_changelog: false,
//!     no_archive: false,
//!     force: false,
//! };
//!
//! let output = Output::new(OutputFormat::Human, io::stdout(), false);
//! execute_bump_preview(&args, &output, Path::new("."), None).await?;
//! # Ok(())
//! # }
//! ```

use crate::cli::commands::BumpArgs;
use crate::commands::bump::snapshot::{BumpSnapshot, BumpSummary, ChangesetInfo, PackageBumpInfo};
use crate::error::{CliError, Result};
use crate::output::styling::{StatusSymbol, print_item};
use crate::output::table::{ColumnAlignment, TableBuilder, TableTheme};
use crate::output::{JsonResponse, Output};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use sublime_pkg_tools::changeset::ChangesetManager;
use sublime_pkg_tools::config::ConfigLoader;
use sublime_pkg_tools::types::{Changeset, PackageInfo, Version, VersionBump};
use sublime_pkg_tools::version::VersionResolver;
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use tracing::{debug, info, warn};

/// Execute the bump preview command.
///
/// Shows what version bumps would be applied based on active changesets,
/// without modifying any files. This is the default mode for the bump command
/// and provides a safe way to verify what will change.
///
/// # Arguments
///
/// * `args` - Command arguments (though preview ignores most execution flags)
/// * `output` - Output handler for formatting and displaying results
/// * `root` - Workspace root directory
/// * `config_path` - Optional path to config file (from global `--config` option)
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if the operation fails.
///
/// # Errors
///
/// Returns an error if:
/// - The workspace is not initialized (no configuration found)
/// - Cannot load changesets or packages
/// - Version resolution fails
/// - File system operations fail
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::bump::execute_bump_preview;
/// use sublime_cli_tools::cli::commands::BumpArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = BumpArgs::default();
/// let output = Output::new(OutputFormat::Human, std::io::stdout(), false);
/// execute_bump_preview(&args, &output, Path::new("."), None).await?;
/// # Ok(())
/// # }
/// ```
pub async fn execute_bump_preview(
    _args: &BumpArgs,
    output: &Output,
    root: &Path,
    config_path: Option<&Path>,
) -> Result<()> {
    let workspace_root = root;
    debug!("Executing bump preview in workspace: {}", workspace_root.display());

    // Step 1: Load configuration
    let config = load_config(workspace_root, config_path).await?;
    info!("Configuration loaded successfully");

    // Step 2: Load all pending changesets
    let fs = FileSystemManager::new();
    let manager = ChangesetManager::new(workspace_root.to_path_buf(), fs.clone(), config.clone())
        .await
        .map_err(|e| CliError::execution(format!("Failed to create changeset manager: {e}")))?;

    let changesets = manager
        .list_pending()
        .await
        .map_err(|e| CliError::execution(format!("Failed to load changesets: {e}")))?;

    debug!("Loaded {} changeset(s)", changesets.len());

    // Step 3: Check if there are any changesets
    if changesets.is_empty() {
        if output.format().is_json() {
            let response: JsonResponse<BumpSnapshot> = JsonResponse::success(BumpSnapshot {
                strategy: config.version.strategy.to_string(),
                packages: vec![],
                changesets: vec![],
                summary: BumpSummary::default(),
            });
            output.json(&response)?;
        } else {
            output.info("No changesets found. Nothing to bump.")?;
        }
        return Ok(());
    }

    info!("Processing {} changeset(s)", changesets.len());

    // Step 4: Create VersionResolver (which will discover packages internally)
    let resolver = VersionResolver::new(workspace_root.to_path_buf(), config.clone())
        .await
        .map_err(|e| CliError::execution(format!("Failed to create version resolver: {e}")))?;

    // Step 5: Discover all workspace packages
    let all_packages = resolver
        .discover_packages()
        .await
        .map_err(|e| CliError::execution(format!("Failed to discover packages: {e}")))?;

    debug!("Discovered {} workspace package(s)", all_packages.len());

    // Step 6: Resolve versions based on strategy
    // For Independent: Only packages in changesets bump
    // For Unified: All packages bump with the highest bump type
    let snapshot = if config.version.strategy
        == sublime_pkg_tools::config::VersioningStrategy::Independent
    {
        build_independent_snapshot(&resolver, &changesets, &all_packages, workspace_root).await?
    } else {
        build_unified_snapshot(&resolver, &changesets, &all_packages, workspace_root).await?
    };

    debug!("Built bump snapshot with {} packages", snapshot.packages.len());

    // Step 7: Output results
    if output.format().is_json() {
        let response: JsonResponse<BumpSnapshot> = JsonResponse::success(snapshot);
        output.json(&response)?;
    } else {
        output_table(output, &snapshot)?;
    }

    Ok(())
}

/// Builds a bump snapshot for Independent versioning strategy.
///
/// In Independent mode, only packages explicitly listed in changesets receive
/// version bumps. Other packages remain unchanged.
async fn build_independent_snapshot(
    resolver: &VersionResolver,
    changesets: &[Changeset],
    all_packages: &[PackageInfo],
    workspace_root: &Path,
) -> Result<BumpSnapshot> {
    debug!("Building snapshot for Independent versioning strategy");

    // Collect all packages that are in changesets
    let mut changeset_packages: HashSet<String> = HashSet::new();
    for changeset in changesets {
        changeset_packages.extend(changeset.packages.iter().cloned());
    }

    // Merge all changesets to get the combined bump requirements
    let merged_changeset = merge_changesets(changesets)?;

    // Resolve versions using the merged changeset
    let resolution = resolver
        .resolve_versions(&merged_changeset)
        .await
        .map_err(|e| CliError::execution(format!("Failed to resolve versions: {e}")))?;

    debug!("Resolved {} package updates", resolution.updates.len());

    // Build package bump info
    let mut packages_info = Vec::new();
    let mut packages_to_bump = 0;

    // Create a map of package updates for quick lookup
    let update_map: HashMap<_, _> =
        resolution.updates.iter().map(|u| (u.name.clone(), u)).collect();

    for package in all_packages {
        let is_in_changeset = changeset_packages.contains(package.name());

        if let Some(update) = update_map.get(package.name()) {
            // Check if version actually changed
            let version_changed = update.current_version != update.next_version;

            // Determine bump reason and whether package will actually bump
            let (will_bump, reason) = if is_in_changeset {
                // Package is directly listed in a changeset
                if version_changed {
                    packages_to_bump += 1;
                    (true, "direct change from changeset".to_string())
                } else {
                    (false, "in changeset but version unchanged".to_string())
                }
            } else if version_changed {
                // Package not in changeset but version changed - must be dependency propagation
                // In Independent mode, we don't bump packages that aren't in changesets
                // even if their dependencies changed
                (false, "dependency updated (not bumping in independent mode)".to_string())
            } else {
                // Package not in changeset and no version change
                (false, "not in any changeset".to_string())
            };

            let bump_type = if will_bump {
                calculate_bump_type(&update.current_version, &update.next_version)
            } else {
                VersionBump::None
            };

            packages_info.push(PackageBumpInfo {
                name: package.name().to_string(),
                path: package
                    .path()
                    .strip_prefix(workspace_root)
                    .unwrap_or(package.path())
                    .display()
                    .to_string(),
                current_version: update.current_version.to_string(),
                next_version: if will_bump {
                    update.next_version.to_string()
                } else {
                    update.current_version.to_string()
                },
                bump_type,
                will_bump,
                reason,
            });
        }
    }

    // Build changeset info
    let changeset_infos: Vec<ChangesetInfo> = changesets
        .iter()
        .map(|cs| ChangesetInfo {
            id: cs.branch.clone(), // Use branch as ID since there's no separate id field
            branch: cs.branch.clone(),
            bump_type: cs.bump,
            packages: cs.packages.clone(),
            commit_count: cs.changes.len(),
        })
        .collect();

    let summary = BumpSummary::new(
        all_packages.len(),
        packages_to_bump,
        changesets.len(),
        !resolution.circular_dependencies.is_empty(),
    );

    if !resolution.circular_dependencies.is_empty() {
        warn!("Detected {} circular dependencies", resolution.circular_dependencies.len());
    }

    Ok(BumpSnapshot {
        strategy: "independent".to_string(),
        packages: packages_info,
        changesets: changeset_infos,
        summary,
    })
}

/// Builds a bump snapshot for Unified versioning strategy.
///
/// In Unified mode, ALL workspace packages receive the same version bump
/// when any changeset exists. The highest bump type from all changesets is used.
async fn build_unified_snapshot(
    resolver: &VersionResolver,
    changesets: &[Changeset],
    all_packages: &[PackageInfo],
    workspace_root: &Path,
) -> Result<BumpSnapshot> {
    debug!("Building snapshot for Unified versioning strategy");

    // Merge all changesets to get the combined bump requirements
    let merged_changeset = merge_changesets(changesets)?;

    // Resolve versions using the merged changeset
    let resolution = resolver
        .resolve_versions(&merged_changeset)
        .await
        .map_err(|e| CliError::execution(format!("Failed to resolve versions: {e}")))?;

    debug!("Resolved {} package updates", resolution.updates.len());

    // In unified mode, ALL packages bump (since changesets exist)
    let packages_to_bump = all_packages.len();

    // Build package bump info
    let mut packages_info = Vec::new();

    // Create a map of package updates for quick lookup
    let update_map: HashMap<_, _> =
        resolution.updates.iter().map(|u| (u.name.clone(), u)).collect();

    // Check which packages are directly in changesets for better reason messages
    let mut changeset_packages: HashSet<String> = HashSet::new();
    for changeset in changesets {
        changeset_packages.extend(changeset.packages.iter().cloned());
    }

    for package in all_packages {
        if let Some(update) = update_map.get(package.name()) {
            let bump_type = calculate_bump_type(&update.current_version, &update.next_version);
            let is_in_changeset = changeset_packages.contains(package.name());

            // Provide clear reason for unified bump
            let reason = if is_in_changeset {
                "unified bump (package in changeset)".to_string()
            } else {
                "unified bump (all packages bumped together)".to_string()
            };

            packages_info.push(PackageBumpInfo {
                name: package.name().to_string(),
                path: package
                    .path()
                    .strip_prefix(workspace_root)
                    .unwrap_or(package.path())
                    .display()
                    .to_string(),
                current_version: update.current_version.to_string(),
                next_version: update.next_version.to_string(),
                bump_type,
                will_bump: true,
                reason,
            });
        }
    }

    // Build changeset info
    let changeset_infos: Vec<ChangesetInfo> = changesets
        .iter()
        .map(|cs| ChangesetInfo {
            id: cs.branch.clone(), // Use branch as ID since there's no separate id field
            branch: cs.branch.clone(),
            bump_type: cs.bump,
            packages: cs.packages.clone(),
            commit_count: cs.changes.len(),
        })
        .collect();

    let summary = BumpSummary::new(
        all_packages.len(),
        packages_to_bump,
        changesets.len(),
        !resolution.circular_dependencies.is_empty(),
    );

    if !resolution.circular_dependencies.is_empty() {
        warn!("Detected {} circular dependencies", resolution.circular_dependencies.len());
    }

    Ok(BumpSnapshot {
        strategy: "unified".to_string(),
        packages: packages_info,
        changesets: changeset_infos,
        summary,
    })
}

/// Merges multiple changesets into a single changeset for version resolution.
///
/// Combines packages, commits, and environments from all changesets.
/// Uses the highest bump type across all changesets.
pub(crate) fn merge_changesets(changesets: &[Changeset]) -> Result<Changeset> {
    if changesets.is_empty() {
        return Err(CliError::execution("No changesets to merge"));
    }

    let mut all_packages: HashSet<String> = HashSet::new();
    let mut all_commits = Vec::new();
    let mut all_environments: HashSet<String> = HashSet::new();
    let mut highest_bump = VersionBump::None;

    for changeset in changesets {
        all_packages.extend(changeset.packages.iter().cloned());
        all_commits.extend(changeset.changes.iter().cloned());
        all_environments.extend(changeset.environments.iter().cloned());

        // Determine highest bump type (Major > Minor > Patch > None)
        let changeset_rank = bump_rank(changeset.bump);
        let highest_rank = bump_rank(highest_bump);
        if changeset_rank > highest_rank {
            highest_bump = changeset.bump;
        }
    }

    // Use first changeset as template and merge data
    let mut merged = changesets[0].clone();
    merged.packages = all_packages.into_iter().collect();
    merged.changes = all_commits;
    merged.environments = all_environments.into_iter().collect();
    merged.bump = highest_bump;

    Ok(merged)
}

/// Returns a numeric rank for bump type comparison.
///
/// Higher ranks represent more significant bumps.
fn bump_rank(bump: VersionBump) -> u8 {
    match bump {
        VersionBump::Major => 3,
        VersionBump::Minor => 2,
        VersionBump::Patch => 1,
        VersionBump::None => 0,
    }
}

/// Calculates the bump type by comparing two versions.
///
/// Determines whether the version change is a major, minor, or patch bump.
fn calculate_bump_type(current: &Version, next: &Version) -> VersionBump {
    if next.major() > current.major() {
        VersionBump::Major
    } else if next.minor() > current.minor() {
        VersionBump::Minor
    } else if next.patch() > current.patch() {
        VersionBump::Patch
    } else {
        VersionBump::None
    }
}

/// Loads workspace configuration.
///
/// Attempts to load configuration from the workspace, with auto-detection
/// of standard config file names.
pub(crate) async fn load_config(
    workspace_root: &Path,
    config_path: Option<&Path>,
) -> Result<sublime_pkg_tools::config::PackageToolsConfig> {
    debug!("Loading workspace configuration from: {}", workspace_root.display());

    let fs = FileSystemManager::new();

    // Try to find and load config file
    let mut found_config = None;
    if let Some(config) = config_path {
        // Use the explicitly provided config file
        let config_file =
            if config.is_absolute() { config.to_path_buf() } else { workspace_root.join(config) };

        if fs.exists(&config_file).await {
            found_config = Some(config_file);
        } else {
            return Err(CliError::configuration(format!(
                "Config file not found: {}",
                config_file.display()
            )));
        }
    } else {
        // Search for standard config file names
        let candidates = vec![
            workspace_root.join("repo.config.toml"),
            workspace_root.join("repo.config.json"),
            workspace_root.join("repo.config.yaml"),
            workspace_root.join("repo.config.yml"),
        ];

        for candidate in candidates {
            if fs.exists(&candidate).await {
                found_config = Some(candidate);
                break;
            }
        }
    }

    // Load configuration
    let config = if let Some(config_path) = found_config {
        match ConfigLoader::load_from_file(&config_path).await {
            Ok(config) => {
                info!("Configuration loaded from: {}", config_path.display());
                config
            }
            Err(e) => {
                return Err(CliError::configuration(format!(
                    "Failed to load configuration from {}: {e}",
                    config_path.display()
                )));
            }
        }
    } else {
        return Err(CliError::configuration(
            "Workspace not initialized. Run 'wnt init' first.".to_string(),
        ));
    };

    debug!("Configuration loaded successfully");
    Ok(config)
}

/// Outputs bump preview results as a formatted table.
///
/// Displays changesets being processed, packages with their version changes,
/// and a summary section with statistics.
fn output_table(output: &Output, snapshot: &BumpSnapshot) -> Result<()> {
    // Display strategy
    StatusSymbol::Info.print_line(&format!("Strategy: {}", snapshot.strategy));
    output.blank_line()?;

    // Display changesets section
    if !snapshot.changesets.is_empty() {
        StatusSymbol::Info
            .print_line(&format!("Active Changesets: {} changeset(s)", snapshot.changesets.len()));

        let mut changeset_table = TableBuilder::new()
            .theme(TableTheme::Minimal)
            .columns(&["Branch", "Bump", "Packages", "Commits"])
            .alignment(3, ColumnAlignment::Right)
            .build();

        for cs in &snapshot.changesets {
            changeset_table.add_row(&[
                &cs.branch,
                &cs.bump_type.to_string().to_lowercase(),
                &cs.packages.len().to_string(),
                &cs.commit_count.to_string(),
            ]);
        }

        output.table(&mut changeset_table)?;
        output.blank_line()?;
    }

    // Display packages section
    StatusSymbol::Info
        .print_line(&format!("Package Updates: {} package(s)", snapshot.packages.len()));

    let mut package_table = TableBuilder::new()
        .theme(TableTheme::Minimal)
        .columns(&["Package", "Current", "Next", "Bump", "Status"])
        .alignment(1, ColumnAlignment::Center)
        .alignment(2, ColumnAlignment::Center)
        .alignment(3, ColumnAlignment::Center)
        .build();

    for pkg in &snapshot.packages {
        let status = if pkg.will_bump { "✓ Will bump" } else { "○ No change" };
        let bump_display =
            if pkg.will_bump { pkg.bump_type.to_string().to_lowercase() } else { "-".to_string() };

        package_table.add_row(&[
            &pkg.name,
            &pkg.current_version,
            &pkg.next_version,
            &bump_display,
            status,
        ]);
    }

    output.table(&mut package_table)?;
    output.blank_line()?;

    // Display summary
    StatusSymbol::Info.print_line("Summary:");
    print_item("  Total packages", &snapshot.summary.total_packages.to_string(), false);
    print_item("  Will bump", &snapshot.summary.packages_to_bump.to_string(), false);
    print_item("  Unchanged", &snapshot.summary.packages_unchanged.to_string(), true);

    if snapshot.summary.has_circular_dependencies {
        output.blank_line()?;
        output
            .warning("Circular dependencies detected. Review dependency graph before bumping.")?;
    }

    Ok(())
}
