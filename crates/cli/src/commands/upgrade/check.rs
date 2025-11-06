//! Upgrade check command implementation.
//!
//! This module implements the `workspace upgrade check` command for detecting available
//! dependency upgrades in workspace packages.
//!
//! # What
//!
//! Provides:
//! - Detection of available dependency upgrades
//! - Filtering by upgrade type (major, minor, patch)
//! - Filtering by dependency type (prod, dev, peer)
//! - Registry querying for latest versions
//! - Formatted output (table or JSON)
//!
//! # How
//!
//! The command:
//! 1. Loads configuration from workspace root
//! 2. Creates detection options from command arguments
//! 3. Uses sublime-package-tools UpgradeManager to detect upgrades
//! 4. Filters results based on user preferences
//! 5. Formats output as table (human) or JSON
//! 6. Displays summary statistics
//!
//! # Why
//!
//! This command provides visibility into available updates before applying them,
//! allowing users to:
//! - Identify security and bug fix patches
//! - Plan major version migrations
//! - Keep dependencies current
//! - Understand upgrade impact across workspace
//!
//! # Examples
//!
//! ```bash
//! # Check all upgrades
//! workspace upgrade check
//!
//! # Check only patch upgrades
//! workspace upgrade check --no-major --no-minor
//!
//! # Check with JSON output
//! workspace upgrade check --format json
//!
//! # Check specific packages
//! workspace upgrade check --packages "typescript,eslint"
//! ```

use crate::cli::commands::UpgradeCheckArgs;
use crate::commands::upgrade::types::{
    DependencyUpgradeInfo, PackageUpgradeInfo, UpgradeCheckResponse, UpgradeSummary,
};
use crate::error::{CliError, Result};
use crate::output::{JsonResponse, Output, table::TableBuilder};
use std::path::Path;
use sublime_pkg_tools::config::PackageToolsConfig;
use sublime_pkg_tools::upgrade::{
    DependencyUpgrade, DetectionOptions, PackageUpgrades, UpgradeManager, UpgradeType,
};
use tracing::{debug, info, instrument};

/// Executes the upgrade check command.
///
/// This function detects available dependency upgrades for workspace packages,
/// filters them according to user preferences, and outputs the results.
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
/// - Registry queries fail
/// - Upgrade detection fails
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::commands::upgrade::execute_upgrade_check;
/// use sublime_cli_tools::cli::commands::UpgradeCheckArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::io;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = UpgradeCheckArgs {
///     major: true,
///     no_major: false,
///     minor: true,
///     no_minor: false,
///     patch: true,
///     no_patch: false,
///     dev: true,
///     peer: false,
///     packages: None,
///     registry: None,
/// };
/// let output = Output::new(OutputFormat::Human, io::stdout(), false);
/// let workspace_root = Path::new(".");
///
/// execute_upgrade_check(&args, &output, workspace_root).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(output), level = "debug")]
pub async fn execute_upgrade_check(
    args: &UpgradeCheckArgs,
    output: &Output,
    workspace_root: &Path,
) -> Result<()> {
    info!("Starting upgrade check");
    debug!("Workspace root: {}", workspace_root.display());

    // Step 1: Load configuration
    debug!("Loading configuration");
    let config = load_config(workspace_root).await?;

    // Step 2: Create detection options from arguments
    debug!("Creating detection options");
    let detection_options = create_detection_options(args)?;
    debug!(
        "Detection options: include_deps={}, include_dev={}, include_peer={}",
        detection_options.include_dependencies,
        detection_options.include_dev_dependencies,
        detection_options.include_peer_dependencies
    );

    // Step 3: Detect upgrades
    info!("Detecting available upgrades");
    let upgrade_manager =
        UpgradeManager::new(workspace_root.to_path_buf(), config.upgrade)
            .await
            .map_err(|e| CliError::execution(format!("Failed to create upgrade manager: {e}")))?;

    let upgrade_preview = upgrade_manager
        .detect_upgrades(detection_options)
        .await
        .map_err(|e| CliError::execution(format!("Failed to detect upgrades: {e}")))?;

    debug!("Found upgrades in {} packages", upgrade_preview.packages.len());

    // Step 4: Filter results by upgrade type based on CLI flags
    let include_major = args.major && !args.no_major;
    let include_minor = args.minor && !args.no_minor;
    let include_patch = args.patch && !args.no_patch;

    let filtered_upgrades = filter_by_upgrade_type(
        &upgrade_preview.packages,
        include_major,
        include_minor,
        include_patch,
    );

    // Step 5: Convert to our CLI types and calculate summary
    let (packages, summary) = convert_and_summarize(&filtered_upgrades);

    // Step 6: Output results
    output_results(output, packages, summary)?;

    info!("Upgrade check completed successfully");
    Ok(())
}

/// Loads package tools configuration from workspace root.
///
/// # Arguments
///
/// * `_workspace_root` - Path to the workspace root directory (unused as config loader looks in current dir)
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
/// Handles the logic for --no-* flags overriding default true values.
/// Note: The CLI supports filtering by upgrade type (major/minor/patch), but
/// the package tools API doesn't directly support this. We'll filter the results
/// after detection based on these preferences.
///
/// # Arguments
///
/// * `args` - Command arguments
///
/// # Returns
///
/// * `Result<DetectionOptions>` - Detection options for upgrade manager
///
/// # Errors
///
/// Returns an error if conflicting options are specified.
pub(crate) fn create_detection_options(args: &UpgradeCheckArgs) -> Result<DetectionOptions> {
    // Validate that at least one type is enabled
    let include_major = args.major && !args.no_major;
    let include_minor = args.minor && !args.no_minor;
    let include_patch = args.patch && !args.no_patch;

    if !include_major && !include_minor && !include_patch {
        return Err(CliError::validation(
            "At least one upgrade type (major, minor, or patch) must be enabled".to_string(),
        ));
    }

    // Create detection options for the package tools API
    let options = DetectionOptions {
        include_dependencies: true, // Always include regular dependencies
        include_dev_dependencies: args.dev,
        include_peer_dependencies: args.peer,
        include_optional_dependencies: false, // Not exposed in CLI yet
        package_filter: args.packages.clone(),
        dependency_filter: None,    // Not exposed in CLI yet
        include_prereleases: false, // Not exposed in CLI yet
        concurrency: 10,            // Default concurrency
    };

    Ok(options)
}

/// Filters upgrade results by upgrade type (major, minor, patch).
///
/// This filters out upgrades based on the CLI flags. The package tools API
/// returns all upgrades, so we filter them here based on user preferences.
///
/// # Arguments
///
/// * `upgrades` - All detected upgrades
/// * `include_major` - Whether to include major upgrades
/// * `include_minor` - Whether to include minor upgrades
/// * `include_patch` - Whether to include patch upgrades
///
/// # Returns
///
/// * `Vec<PackageUpgrades>` - Filtered upgrades
fn filter_by_upgrade_type(
    upgrades: &[PackageUpgrades],
    include_major: bool,
    include_minor: bool,
    include_patch: bool,
) -> Vec<PackageUpgrades> {
    upgrades
        .iter()
        .map(|pkg| {
            let filtered_upgrades: Vec<DependencyUpgrade> = pkg
                .upgrades
                .iter()
                .filter(|upgrade| match upgrade.upgrade_type {
                    UpgradeType::Major => include_major,
                    UpgradeType::Minor => include_minor,
                    UpgradeType::Patch => include_patch,
                })
                .cloned()
                .collect();

            PackageUpgrades {
                package_name: pkg.package_name.clone(),
                package_path: pkg.package_path.clone(),
                current_version: pkg.current_version.clone(),
                upgrades: filtered_upgrades,
            }
        })
        .filter(|pkg| !pkg.upgrades.is_empty())
        .collect()
}

/// Converts sublime-package-tools types to CLI types and calculates summary.
///
/// # Arguments
///
/// * `upgrades` - Detected upgrades from package tools
///
/// # Returns
///
/// * `(Vec<PackageUpgradeInfo>, UpgradeSummary)` - Converted data and summary
fn convert_and_summarize(
    upgrades: &[PackageUpgrades],
) -> (Vec<PackageUpgradeInfo>, UpgradeSummary) {
    let mut summary = UpgradeSummary::new();
    summary.total_packages = upgrades.len();

    let packages: Vec<PackageUpgradeInfo> = upgrades
        .iter()
        .filter_map(|pkg| {
            let package_upgrades: Vec<DependencyUpgradeInfo> = pkg
                .upgrades
                .iter()
                .map(|dep| convert_dependency_upgrade(dep, &mut summary))
                .collect();

            if package_upgrades.is_empty() {
                None
            } else {
                summary.packages_with_upgrades += 1;
                Some(PackageUpgradeInfo {
                    name: pkg.package_name.clone(),
                    path: pkg.package_path.to_str().unwrap_or("unknown").to_string(),
                    upgrades: package_upgrades,
                })
            }
        })
        .collect();

    (packages, summary)
}

/// Converts a single dependency upgrade and updates summary statistics.
///
/// # Arguments
///
/// * `dep` - Dependency upgrade from package tools
/// * `summary` - Summary to update with counts
///
/// # Returns
///
/// * `DependencyUpgradeInfo` - Converted dependency upgrade
fn convert_dependency_upgrade(
    dep: &DependencyUpgrade,
    summary: &mut UpgradeSummary,
) -> DependencyUpgradeInfo {
    summary.total_upgrades += 1;

    let (upgrade_type, breaking) = match dep.upgrade_type {
        UpgradeType::Major => {
            summary.major_upgrades += 1;
            ("major", true)
        }
        UpgradeType::Minor => {
            summary.minor_upgrades += 1;
            ("minor", false)
        }
        UpgradeType::Patch => {
            summary.patch_upgrades += 1;
            ("patch", false)
        }
    };

    DependencyUpgradeInfo {
        package: dep.name.clone(),
        current_version: dep.current_version.clone(),
        latest_version: dep.latest_version.clone(),
        upgrade_type: upgrade_type.to_string(),
        breaking,
    }
}

/// Outputs the upgrade check results in the appropriate format.
///
/// # Arguments
///
/// * `output` - Output context
/// * `packages` - Package upgrade information
/// * `summary` - Summary statistics
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
    packages: Vec<PackageUpgradeInfo>,
    summary: UpgradeSummary,
) -> Result<()> {
    match output.format() {
        crate::output::OutputFormat::Json | crate::output::OutputFormat::JsonCompact => {
            output_json(output, packages, summary)
        }
        crate::output::OutputFormat::Human => output_human(output, &packages, &summary),
        crate::output::OutputFormat::Quiet => output_quiet(output, &summary),
    }
}

/// Outputs results in JSON format.
///
/// # Arguments
///
/// * `output` - Output context
/// * `packages` - Package upgrade information
/// * `summary` - Summary statistics
///
/// # Returns
///
/// * `Result<()>` - Success or error
fn output_json(
    output: &Output,
    packages: Vec<PackageUpgradeInfo>,
    summary: UpgradeSummary,
) -> Result<()> {
    let response = UpgradeCheckResponse { success: true, packages, summary };

    let json_response = JsonResponse::success(response);
    output.json(&json_response)
}

/// Outputs results in human-readable format with tables.
///
/// # Arguments
///
/// * `output` - Output context
/// * `packages` - Package upgrade information
/// * `summary` - Summary statistics
///
/// # Returns
///
/// * `Result<()>` - Success or error
fn output_human(
    output: &Output,
    packages: &[PackageUpgradeInfo],
    summary: &UpgradeSummary,
) -> Result<()> {
    use console::style;

    // Header
    let title = style("Dependency Upgrades Available").bold().cyan();
    output.plain(&title.to_string())?;
    output.plain("━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
    output.blank_line()?;

    if !summary.has_upgrades() {
        output.success("All dependencies are up to date!")?;
        return Ok(());
    }

    // Output each package's upgrades in a table
    for (idx, package) in packages.iter().enumerate() {
        if idx > 0 {
            output.blank_line()?;
        }

        let package_header = style(&package.name).bold().green();
        output.plain(&format!("{package_header}:"))?;

        let mut table =
            TableBuilder::new().columns(&["Package", "Current", "Latest", "Type"]).build();

        for upgrade in &package.upgrades {
            table.add_row(&[
                &upgrade.package,
                &upgrade.current_version,
                &upgrade.latest_version,
                &upgrade.upgrade_type,
            ]);
        }

        output.table(&mut table)?;
    }

    // Summary
    output.blank_line()?;
    output.plain("Summary:")?;
    output.plain(&format!("  Total packages: {}", summary.total_packages))?;
    output.plain(&format!("  Packages with upgrades: {}", summary.packages_with_upgrades))?;
    output.plain(&format!("  Total upgrades: {}", summary.total_upgrades))?;
    output.plain(&format!("  Major: {}", summary.major_upgrades))?;
    output.plain(&format!("  Minor: {}", summary.minor_upgrades))?;
    output.plain(&format!("  Patch: {}", summary.patch_upgrades))?;

    Ok(())
}

/// Outputs minimal results in quiet mode.
///
/// # Arguments
///
/// * `output` - Output context
/// * `summary` - Summary statistics
///
/// # Returns
///
/// * `Result<()>` - Success or error
fn output_quiet(output: &Output, summary: &UpgradeSummary) -> Result<()> {
    if summary.has_upgrades() {
        output.plain(&format!("{} upgrades available", summary.total_upgrades))?;
    }
    Ok(())
}
