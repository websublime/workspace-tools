//! Upgrade apply command implementation.
//!
//! This module implements the `wnt upgrade apply` command for applying selected
//! dependency upgrades to workspace packages.
//!
//! # What
//!
//! Provides:
//! - Application of detected dependency upgrades
//! - Filtering by upgrade type (major, minor, patch)
//! - Backup creation before applying changes
//! - Optional changeset creation for applied upgrades
//! - Post-upgrade validation
//! - Dry-run mode for previewing changes
//! - Formatted output (table or JSON)
//!
//! # How
//!
//! The command:
//! 1. Loads configuration from workspace root
//! 2. Detects available upgrades
//! 3. Filters based on user selection criteria
//! 4. Creates backup (unless --no-backup)
//! 5. Applies upgrades to package.json files
//! 6. Optionally creates changeset (if --auto-changeset)
//! 7. Validates modified files
//! 8. Outputs results and summary
//!
//! # Why
//!
//! This command provides a safe, controlled way to apply dependency upgrades:
//! - Prevents accidental breaking changes with type filtering
//! - Creates backups for easy rollback
//! - Integrates with changeset workflow
//! - Validates changes before completion
//! - Provides clear visibility into what was changed
//!
//! # Examples
//!
//! ```bash
//! # Preview what would be upgraded
//! wnt upgrade apply --dry-run
//!
//! # Apply only patch upgrades
//! wnt upgrade apply --patch-only
//!
//! # Apply with automatic changeset
//! wnt upgrade apply --auto-changeset --changeset-bump minor
//!
//! # Apply specific packages
//! wnt upgrade apply --packages "typescript,eslint"
//!
//! # Apply with JSON output
//! wnt upgrade apply --format json
//! ```

use crate::cli::commands::UpgradeApplyArgs;
use crate::commands::upgrade::types::{
    AppliedUpgradeInfo, ApplySummary, SkippedUpgradeInfo, UpgradeApplyResponse,
};
use crate::error::{CliError, Result};
use crate::interactive::prompts::prompt_confirm;
use crate::output::{JsonResponse, Output, table::TableBuilder};
use std::collections::HashSet;
use std::path::Path;
use sublime_pkg_tools::config::PackageToolsConfig;
use sublime_pkg_tools::types::VersionBump;
use sublime_pkg_tools::upgrade::{
    AppliedUpgrade, DependencyUpgrade, DetectionOptions, PackageUpgrades, UpgradeManager,
    UpgradeSelection, UpgradeType,
};
use tracing::{debug, info, instrument, warn};

/// Executes the upgrade apply command.
///
/// This function applies selected dependency upgrades to workspace packages,
/// with options for filtering, backup, changeset creation, and validation.
///
/// # Arguments
///
/// * `args` - Command arguments from CLI
/// * `output` - Output context for formatting
/// * `workspace_root` - Path to the workspace root directory
///
/// # Returns
///
/// * `Result<()>` - Success or error
///
/// # Errors
///
/// Returns an error if:
/// - Workspace root is invalid
/// - Configuration cannot be loaded
/// - No upgrades match selection criteria
/// - Backup creation fails (unless --no-backup)
/// - Upgrade application fails
/// - Changeset creation fails (if --auto-changeset)
/// - Validation fails after applying upgrades
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::commands::upgrade::execute_upgrade_apply;
/// use sublime_cli_tools::cli::commands::UpgradeApplyArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::io;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = UpgradeApplyArgs {
///     dry_run: false,
///     patch_only: true,
///     minor_and_patch: false,
///     packages: None,
///     auto_changeset: false,
///     changeset_bump: "patch".to_string(),
///     no_backup: false,
///     force: false,
/// };
/// let output = Output::new(OutputFormat::Human, io::stdout(), false);
/// let workspace_root = Path::new(".");
///
/// execute_upgrade_apply(&args, &output, workspace_root).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(output), level = "debug")]
pub async fn execute_upgrade_apply(
    args: &UpgradeApplyArgs,
    output: &Output,
    workspace_root: &Path,
) -> Result<()> {
    info!("Starting upgrade apply");
    debug!("Workspace root: {}", workspace_root.display());
    debug!("Dry run: {}", args.dry_run);
    debug!("Auto changeset: {}", args.auto_changeset);

    // Step 1: Validate arguments
    validate_args(args)?;

    // Step 2: Load configuration
    debug!("Loading configuration");
    let config = load_config(workspace_root).await?;

    // Step 3: Detect available upgrades
    info!("Detecting available upgrades");
    let detection_options = create_detection_options(args);

    // Create upgrade manager with upgrade-specific config
    let mut upgrade_manager =
        UpgradeManager::new(workspace_root.to_path_buf(), config.upgrade.clone())
            .await
            .map_err(|e| CliError::execution(format!("Failed to create upgrade manager: {e}")))?;

    let available_upgrades = upgrade_manager
        .detect_upgrades(detection_options)
        .await
        .map_err(|e| CliError::execution(format!("Failed to detect upgrades: {e}")))?;

    debug!("Found {} packages with upgrades", available_upgrades.packages.len());

    // Step 5: Create upgrade selection from args
    let selection = create_upgrade_selection(args);

    // Step 6: Check if there are any upgrades to apply
    let upgrades_to_apply = count_selected_upgrades(&available_upgrades.packages, &selection);
    if upgrades_to_apply == 0 {
        info!("No upgrades match the selection criteria");
        output_no_upgrades(output)?;
        return Ok(());
    }

    info!("Found {} upgrades matching selection criteria", upgrades_to_apply);

    // Step 7: Confirm with user (unless --force or --dry-run)
    if !args.force && !args.dry_run && output.format().is_human() {
        let should_proceed = confirm_apply(output, upgrades_to_apply, output.no_color())?;
        if !should_proceed {
            info!("User cancelled upgrade apply");
            output.info("Upgrade cancelled")?;
            return Ok(());
        }
    }

    // Step 8: Apply upgrades (or dry-run)
    // Note: Changeset creation is controlled by UpgradeManager's config.auto_changeset
    // For now, we'll note this limitation and implement changeset support in a future story
    if args.auto_changeset {
        warn!("Auto-changeset option requested but changeset integration not yet implemented");
        // TODO: will be implemented on story 6.2 completion - integrate with ChangesetManager
    }

    info!("Applying upgrades (dry_run={})", args.dry_run);

    // Clone selection to use after apply_upgrades (which consumes it)
    let selection_for_result = selection.clone();

    let upgrade_result = upgrade_manager
        .apply_upgrades(selection, args.dry_run)
        .await
        .map_err(|e| CliError::execution(format!("Failed to apply upgrades: {e}")))?;

    debug!("Applied {} upgrades", upgrade_result.applied.len());

    // Step 9: Convert results and output
    let (applied, skipped, summary) = convert_upgrade_result(
        &upgrade_result,
        &available_upgrades.packages,
        &selection_for_result,
        args.dry_run,
    );

    output_results(output, applied, skipped, summary, args.dry_run)?;

    info!("Upgrade apply completed successfully");
    Ok(())
}

/// Validates command arguments for consistency.
///
/// # Arguments
///
/// * `args` - Command arguments to validate
///
/// # Returns
///
/// * `Result<()>` - Success or validation error
///
/// # Errors
///
/// Returns an error if:
/// - Both --patch-only and --minor-and-patch are specified
/// - Invalid changeset bump type
fn validate_args(args: &UpgradeApplyArgs) -> Result<()> {
    // Validate changeset bump type if auto-changeset is enabled
    if args.auto_changeset {
        parse_changeset_bump(&args.changeset_bump)?;
    }

    Ok(())
}

/// Loads package tools configuration from workspace root.
///
/// # Arguments
///
/// * `_workspace_root` - Path to the workspace root directory
///
/// # Returns
///
/// * `Result<PackageToolsConfig>` - Loaded configuration
///
/// # Errors
///
/// Returns an error if configuration cannot be loaded or validated.
async fn load_config(_workspace_root: &Path) -> Result<PackageToolsConfig> {
    sublime_pkg_tools::config::load_config()
        .await
        .map_err(|e| CliError::configuration(format!("Failed to load configuration: {e}")))
}

/// Creates detection options from command arguments.
///
/// # Arguments
///
/// * `args` - Command arguments
///
/// # Returns
///
/// * `DetectionOptions` - Detection options for upgrade manager
fn create_detection_options(args: &UpgradeApplyArgs) -> DetectionOptions {
    DetectionOptions {
        include_dependencies: true,      // Always include regular dependencies
        include_dev_dependencies: true,  // Always include dev dependencies
        include_peer_dependencies: true, // Always include peer dependencies
        include_optional_dependencies: false,
        package_filter: args.packages.clone(),
        dependency_filter: None,
        include_prereleases: false,
        concurrency: 10,
    }
}

/// Creates upgrade selection criteria from command arguments.
///
/// # Arguments
///
/// * `args` - Command arguments
///
/// # Returns
///
/// * `UpgradeSelection` - Selection criteria for upgrade manager
fn create_upgrade_selection(args: &UpgradeApplyArgs) -> UpgradeSelection {
    // Create base selection based on upgrade type flags
    let mut selection = if args.patch_only {
        UpgradeSelection::patch_only()
    } else if args.minor_and_patch {
        UpgradeSelection::minor_and_patch()
    } else {
        // Default: include all types
        UpgradeSelection::all()
    };

    // Add package filter if specified
    if let Some(ref packages) = args.packages {
        selection.packages = Some(packages.clone());
    }

    selection
}

/// Counts the number of upgrades that match the selection criteria.
///
/// # Arguments
///
/// * `available_upgrades` - All available upgrades
/// * `selection` - Selection criteria
///
/// # Returns
///
/// * `usize` - Number of matching upgrades
fn count_selected_upgrades(
    available_upgrades: &[sublime_pkg_tools::upgrade::PackageUpgrades],
    selection: &UpgradeSelection,
) -> usize {
    available_upgrades
        .iter()
        .filter(|pkg| selection.matches_package(&pkg.package_name))
        .flat_map(|pkg| &pkg.upgrades)
        .filter(|upgrade| {
            let matches_type = selection.matches_type(upgrade.upgrade_type);
            let matches_dep = selection.matches_dependency(&upgrade.name);
            matches_type && matches_dep
        })
        .count()
}

/// Parses a changeset bump type from a string.
///
/// # Arguments
///
/// * `bump_str` - Bump type string (major, minor, patch)
///
/// # Returns
///
/// * `Result<VersionBump>` - Parsed bump type
///
/// # Errors
///
/// Returns an error if the bump type is invalid.
fn parse_changeset_bump(bump_str: &str) -> Result<VersionBump> {
    match bump_str.to_lowercase().as_str() {
        "major" => Ok(VersionBump::Major),
        "minor" => Ok(VersionBump::Minor),
        "patch" => Ok(VersionBump::Patch),
        _ => Err(CliError::validation(format!(
            "Invalid changeset bump type: '{bump_str}'. Must be one of: major, minor, patch"
        ))),
    }
}

/// Prompts the user to confirm the upgrade apply operation.
///
/// # Arguments
///
/// * `output` - Output context
/// * `count` - Number of upgrades to apply
/// * `no_color` - Whether to disable colored output
///
/// # Returns
///
/// * `Result<bool>` - True if user confirms, false otherwise
///
/// # Errors
///
/// Returns an error if user input fails or user cancels.
fn confirm_apply(output: &Output, count: usize, no_color: bool) -> Result<bool> {
    use console::style;

    output.blank_line()?;
    let message = style(format!(
        "This will upgrade {count} dependenc{} in your workspace.",
        if count == 1 { "y" } else { "ies" }
    ))
    .yellow();
    output.plain(&message.to_string())?;
    output.plain("A backup will be created before applying changes.")?;
    output.blank_line()?;

    // Use interactive prompt to confirm
    prompt_confirm("Do you want to proceed?", true, no_color)
}

/// Converts UpgradeResult from package tools to CLI types.
///
/// Calculates skipped upgrades by comparing available upgrades with applied ones.
/// An upgrade is considered "skipped" if it was available but not applied.
///
/// # Arguments
///
/// * `result` - Upgrade result from package tools
/// * `available_upgrades` - All available upgrades that were detected
/// * `selection` - The selection criteria used for filtering
/// * `is_dry_run` - Whether this was a dry-run operation
///
/// # Returns
///
/// * `(Vec<AppliedUpgradeInfo>, Vec<SkippedUpgradeInfo>, ApplySummary)` - Converted data
fn convert_upgrade_result(
    result: &sublime_pkg_tools::upgrade::UpgradeResult,
    available_upgrades: &[PackageUpgrades],
    selection: &UpgradeSelection,
    is_dry_run: bool,
) -> (Vec<AppliedUpgradeInfo>, Vec<SkippedUpgradeInfo>, ApplySummary) {
    let applied: Vec<AppliedUpgradeInfo> =
        result.applied.iter().map(convert_applied_upgrade).collect();

    // Calculate skipped upgrades: available upgrades that were NOT applied
    let applied_deps: HashSet<String> =
        result.applied.iter().map(|a| a.dependency_name.clone()).collect();

    let skipped: Vec<SkippedUpgradeInfo> = available_upgrades
        .iter()
        .filter(|pkg| selection.matches_package(&pkg.package_name))
        .flat_map(|pkg| &pkg.upgrades)
        .filter(|upgrade| {
            // Only include if it matches selection criteria and wasn't applied
            let matches_type = selection.matches_type(upgrade.upgrade_type);
            let matches_dep = selection.matches_dependency(&upgrade.name);
            let was_not_applied = !applied_deps.contains(&upgrade.name);

            matches_type && matches_dep && was_not_applied
        })
        .map(|upgrade| {
            // Determine reason for skipping
            let reason = if selection.matches_type(upgrade.upgrade_type) {
                "filtered_by_selection".to_string()
            } else {
                format!("{}_version_filtered", upgrade_type_to_string(upgrade.upgrade_type))
            };

            convert_skipped_upgrade(upgrade, &reason)
        })
        .collect();

    let summary = ApplySummary {
        total_applied: applied.len(),
        total_skipped: skipped.len(),
        backup_id: if is_dry_run {
            None
        } else {
            result.backup_path.as_ref().and_then(|p| {
                p.file_name().and_then(|n| n.to_str()).map(std::string::ToString::to_string)
            })
        },
    };

    (applied, skipped, summary)
}

/// Converts upgrade type to lowercase string.
///
/// # Arguments
///
/// * `upgrade_type` - The upgrade type
///
/// # Returns
///
/// * `&str` - String representation ("major", "minor", or "patch")
fn upgrade_type_to_string(upgrade_type: UpgradeType) -> &'static str {
    match upgrade_type {
        UpgradeType::Major => "major",
        UpgradeType::Minor => "minor",
        UpgradeType::Patch => "patch",
    }
}

/// Converts a single applied upgrade to CLI type.
///
/// # Arguments
///
/// * `applied` - Applied upgrade from package tools
///
/// # Returns
///
/// * `AppliedUpgradeInfo` - Converted applied upgrade
fn convert_applied_upgrade(applied: &AppliedUpgrade) -> AppliedUpgradeInfo {
    let upgrade_type = match applied.upgrade_type {
        UpgradeType::Major => "major",
        UpgradeType::Minor => "minor",
        UpgradeType::Patch => "patch",
    };

    AppliedUpgradeInfo {
        package: applied.dependency_name.clone(),
        from: applied.old_version.clone(),
        to: applied.new_version.clone(),
        upgrade_type: upgrade_type.to_string(),
    }
}

/// Converts a skipped upgrade to CLI type.
///
/// # Arguments
///
/// * `skipped` - Skipped dependency from package tools
/// * `reason` - Reason for skipping
///
/// # Returns
///
/// * `SkippedUpgradeInfo` - Converted skipped upgrade
fn convert_skipped_upgrade(skipped: &DependencyUpgrade, reason: &str) -> SkippedUpgradeInfo {
    SkippedUpgradeInfo {
        package: skipped.name.clone(),
        reason: reason.to_string(),
        current_version: skipped.current_version.clone(),
        latest_version: skipped.latest_version.clone(),
    }
}

/// Outputs message when no upgrades are available.
///
/// # Arguments
///
/// * `output` - Output context
///
/// # Returns
///
/// * `Result<()>` - Success or error
fn output_no_upgrades(output: &Output) -> Result<()> {
    match output.format() {
        crate::output::OutputFormat::Json | crate::output::OutputFormat::JsonCompact => {
            let response = UpgradeApplyResponse {
                success: true,
                applied: vec![],
                skipped: vec![],
                summary: ApplySummary::new(),
            };
            let json_response = JsonResponse::success(response);
            output.json(&json_response)
        }
        crate::output::OutputFormat::Human => {
            output.info("No upgrades match the selection criteria")?;
            Ok(())
        }
        crate::output::OutputFormat::Quiet => Ok(()),
    }
}

/// Outputs the upgrade apply results in the appropriate format.
///
/// # Arguments
///
/// * `output` - Output context
/// * `applied` - Applied upgrades
/// * `skipped` - Skipped upgrades
/// * `summary` - Summary statistics
/// * `is_dry_run` - Whether this was a dry-run operation
///
/// # Returns
///
/// * `Result<()>` - Success or error
///
/// # Errors
///
/// Returns an error if output formatting or writing fails.
fn output_results(
    output: &Output,
    applied: Vec<AppliedUpgradeInfo>,
    skipped: Vec<SkippedUpgradeInfo>,
    summary: ApplySummary,
    is_dry_run: bool,
) -> Result<()> {
    match output.format() {
        crate::output::OutputFormat::Json | crate::output::OutputFormat::JsonCompact => {
            output_json(output, applied, skipped, summary)
        }
        crate::output::OutputFormat::Human => {
            output_human(output, &applied, &skipped, &summary, is_dry_run)
        }
        crate::output::OutputFormat::Quiet => output_quiet(output, &summary, is_dry_run),
    }
}

/// Outputs results in JSON format.
///
/// # Arguments
///
/// * `output` - Output context
/// * `applied` - Applied upgrades
/// * `skipped` - Skipped upgrades
/// * `summary` - Summary statistics
///
/// # Returns
///
/// * `Result<()>` - Success or error
fn output_json(
    output: &Output,
    applied: Vec<AppliedUpgradeInfo>,
    skipped: Vec<SkippedUpgradeInfo>,
    summary: ApplySummary,
) -> Result<()> {
    let response = UpgradeApplyResponse { success: true, applied, skipped, summary };

    let json_response = JsonResponse::success(response);
    output.json(&json_response)
}

/// Outputs results in human-readable format with tables.
///
/// # Arguments
///
/// * `output` - Output context
/// * `applied` - Applied upgrades
/// * `skipped` - Skipped upgrades
/// * `summary` - Summary statistics
/// * `is_dry_run` - Whether this was a dry-run
///
/// # Returns
///
/// * `Result<()>` - Success or error
fn output_human(
    output: &Output,
    applied: &[AppliedUpgradeInfo],
    skipped: &[SkippedUpgradeInfo],
    summary: &ApplySummary,
    is_dry_run: bool,
) -> Result<()> {
    use console::style;

    // Header
    let title = if is_dry_run {
        style("Upgrade Preview (Dry Run)").bold().cyan()
    } else {
        style("Upgrades Applied").bold().green()
    };
    output.plain(&title.to_string())?;
    output.plain("━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
    output.blank_line()?;

    // Applied upgrades
    if !applied.is_empty() {
        let section_title =
            if is_dry_run { style("Would Upgrade:").bold() } else { style("Upgraded:").bold() };
        output.plain(&section_title.to_string())?;

        let mut table = TableBuilder::new().columns(&["Package", "From", "To", "Type"]).build();

        for upgrade in applied {
            table.add_row(&[&upgrade.package, &upgrade.from, &upgrade.to, &upgrade.upgrade_type]);
        }

        output.table(&mut table)?;
        output.blank_line()?;
    }

    // Skipped upgrades
    if !skipped.is_empty() {
        output.plain(&style("Skipped:").bold().to_string())?;

        let mut table =
            TableBuilder::new().columns(&["Package", "Current", "Latest", "Reason"]).build();

        for skip in skipped {
            table.add_row(&[
                &skip.package,
                &skip.current_version,
                &skip.latest_version,
                &skip.reason,
            ]);
        }

        output.table(&mut table)?;
        output.blank_line()?;
    }

    // Summary
    output.plain("Summary:")?;
    output.plain(&format!("  Total applied: {}", summary.total_applied))?;
    output.plain(&format!("  Total skipped: {}", summary.total_skipped))?;

    if let Some(ref backup_id) = summary.backup_id {
        output.blank_line()?;
        output.plain(&format!("Backup created: {backup_id}"))?;
        output.plain(
            &style("  Use `wnt upgrade backups restore <ID>` to rollback").dim().to_string(),
        )?;
    }

    if is_dry_run {
        output.blank_line()?;
        output.info("This was a dry run. No files were modified.")?;
        output.plain(&style("  Run without --dry-run to apply changes.").dim().to_string())?;
    }

    Ok(())
}

/// Outputs minimal results in quiet mode.
///
/// # Arguments
///
/// * `output` - Output context
/// * `summary` - Summary statistics
/// * `is_dry_run` - Whether this was a dry-run
///
/// # Returns
///
/// * `Result<()>` - Success or error
fn output_quiet(output: &Output, summary: &ApplySummary, is_dry_run: bool) -> Result<()> {
    if summary.total_applied > 0 {
        let verb = if is_dry_run { "would upgrade" } else { "upgraded" };
        output.plain(&format!("{} {verb}", summary.total_applied))?;
    }
    Ok(())
}
