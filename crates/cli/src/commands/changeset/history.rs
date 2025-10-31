//! Changeset history command implementation.
//!
//! This module implements the `changeset history` command for querying archived changesets.
//!
//! # What
//!
//! Provides the `execute_history` function that:
//! - Queries archived changesets from the history directory
//! - Supports filtering by package, date range, environment, and bump type
//! - Supports limiting the number of results
//! - Outputs results in table or JSON format
//! - Shows both changeset and release information
//!
//! # How
//!
//! The command flow:
//! 1. Loads workspace configuration and validates initialization
//! 2. Creates ChangesetManager to access changeset storage
//! 3. Creates ChangesetHistory instance for querying archived changesets
//! 4. Applies filters based on command arguments (package, date, env, bump)
//! 5. Limits results if specified
//! 6. Formats output as either a human-readable table or JSON
//! 7. Displays summary statistics (total count)
//!
//! Uses:
//! - `ChangesetHistory` from pkg tools for archive queries
//! - Table rendering for human-readable output
//! - JSON serialization for machine-readable output
//!
//! # Why
//!
//! Querying changeset history is essential for:
//! - Reviewing past releases and version changes
//! - Understanding what was released in specific packages
//! - Auditing release history for compliance
//! - Analyzing release patterns and trends
//! - CI/CD integration for release history tracking
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::commands::changeset::execute_history;
//! use sublime_cli_tools::cli::commands::ChangesetHistoryArgs;
//! use sublime_cli_tools::output::{Output, OutputFormat};
//! use std::io;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let args = ChangesetHistoryArgs {
//!     package: Some("my-package".to_string()),
//!     since: None,
//!     until: None,
//!     env: None,
//!     bump: None,
//!     limit: Some(10),
//! };
//!
//! let output = Output::new(OutputFormat::Human, io::stdout(), false);
//! execute_history(&args, &output, None, None).await?;
//! # Ok(())
//! # }
//! ```

use super::common::load_config;
use super::types::ArchivedChangesetInfo;
use crate::cli::commands::ChangesetHistoryArgs;
use crate::commands::changeset::list::parse_bump_type;
use crate::error::{CliError, Result};
use crate::output::styling::{Section, print_item};
use crate::output::table::{ColumnAlignment, TableBuilder, TableTheme};
use crate::output::{JsonResponse, Output};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::Path;
use sublime_pkg_tools::changeset::ChangesetHistory;
use sublime_pkg_tools::types::ArchivedChangeset;
use sublime_standard_tools::filesystem::FileSystemManager;
use tracing::{debug, info};

/// Response data for changeset history command (JSON output).
///
/// Contains the list of archived changesets and summary information.
#[derive(Debug, Serialize)]
struct ChangesetHistoryResponse {
    /// Whether the operation succeeded.
    success: bool,
    /// List of archived changesets with their details.
    changesets: Vec<ArchivedChangesetInfo>,
    /// Total number of changesets returned.
    total: usize,
}

/// Execute the changeset history command.
///
/// Queries archived changesets with optional filtering by package, date range,
/// environment, bump type, and result limit. Outputs results in either
/// human-readable table format or JSON format.
///
/// # Arguments
///
/// * `args` - Command arguments including filters and limit options
/// * `output` - Output handler for formatting and displaying results
/// * `root` - Optional workspace root directory (defaults to current directory)
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
/// - The history directory cannot be read
/// - Invalid filter values are provided (date format, bump type)
/// - File system operations fail
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::changeset::execute_history;
/// use sublime_cli_tools::cli::commands::ChangesetHistoryArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::io;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = ChangesetHistoryArgs {
///     package: Some("my-package".to_string()),
///     since: Some("2025-01-01".to_string()),
///     until: None,
///     env: Some("production".to_string()),
///     bump: Some("minor".to_string()),
///     limit: Some(20),
/// };
///
/// let output = Output::new(OutputFormat::Human, io::stdout(), false);
/// execute_history(&args, &output, None, None).await?;
/// # Ok(())
/// # }
/// ```
pub async fn execute_history(
    args: &ChangesetHistoryArgs,
    output: &Output,
    root: Option<&Path>,
    config_path: Option<&Path>,
) -> Result<()> {
    let workspace_root = root.unwrap_or_else(|| Path::new("."));
    debug!("Querying changeset history in workspace: {}", workspace_root.display());

    // Load configuration
    let config = load_config(workspace_root, config_path).await?;

    info!("Configuration loaded successfully");

    // Create changeset history for querying archived changesets
    let storage = sublime_pkg_tools::changeset::FileBasedChangesetStorage::new(
        workspace_root.to_path_buf(),
        config.changeset.path.clone(),
        config.changeset.history_path.clone(),
        FileSystemManager::new(),
    );
    let history = ChangesetHistory::new(Box::new(storage));

    // Query archived changesets based on filters
    let mut changesets = query_changesets(&history, args).await?;

    debug!("Loaded {} archived changeset(s)", changesets.len());

    // Apply limit if specified
    if let Some(limit) = args.limit
        && changesets.len() > limit
    {
        changesets.truncate(limit);
        debug!("Limited results to {} changeset(s)", limit);
    }

    let total = changesets.len();
    info!("Displaying {} archived changeset(s) after filtering", total);

    // Output results
    if output.format().is_json() {
        output_json(output, changesets)?;
    } else {
        output_table(output, &changesets)?;
    }

    Ok(())
}

/// Query changesets from history based on filter arguments.
///
/// Applies filters in the most efficient order to minimize processing.
/// Prioritizes specific filters (package, date range) over broader ones.
///
/// # Arguments
///
/// * `history` - ChangesetHistory instance for querying
/// * `args` - Command arguments with filter criteria
///
/// # Errors
///
/// Returns an error if:
/// - Date parsing fails for since/until arguments
/// - Bump type parsing fails
/// - History query operations fail
async fn query_changesets(
    history: &ChangesetHistory,
    args: &ChangesetHistoryArgs,
) -> Result<Vec<ArchivedChangeset>> {
    let mut changesets: Vec<ArchivedChangeset>;

    // Track which filter was used as primary query to avoid re-applying it
    let mut used_package_primary = false;
    let mut used_date_primary = false;
    let mut used_env_primary = false;
    let mut used_bump_primary = false;

    // Start with the most specific filter first
    if let Some(package) = &args.package {
        debug!("Querying by package: {}", package);
        changesets = history
            .query_by_package(package)
            .await
            .map_err(|e| CliError::Execution(format!("Failed to query changesets by package: {e}")))?;
        used_package_primary = true;
    } else if args.since.is_some() || args.until.is_some() {
        // Query by date range
        let since = if let Some(since_str) = &args.since {
            parse_date(since_str)?
        } else {
            // Use earliest possible date
            DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z")
                .map_err(|e| CliError::Validation(format!("Failed to parse default since date: {e}")))?
                .with_timezone(&Utc)
        };

        let until = if let Some(until_str) = &args.until {
            parse_date(until_str)?
        } else {
            // Use current time
            Utc::now()
        };

        debug!("Querying by date range: {} to {}", since, until);
        changesets = history
            .query_by_date(since, until)
            .await
            .map_err(|e| CliError::Execution(format!("Failed to query changesets by date: {e}")))?;
        used_date_primary = true;
    } else if let Some(env) = &args.env {
        debug!("Querying by environment: {}", env);
        changesets = history
            .query_by_environment(env)
            .await
            .map_err(|e| {
                CliError::Execution(format!("Failed to query changesets by environment: {e}"))
            })?;
        used_env_primary = true;
    } else if let Some(bump_str) = &args.bump {
        let bump = parse_bump_type(bump_str)?;
        debug!("Querying by bump type: {:?}", bump);
        changesets = history
            .query_by_bump(bump)
            .await
            .map_err(|e| {
                CliError::Execution(format!("Failed to query changesets by bump type: {e}"))
            })?;
        used_bump_primary = true;
    } else {
        // No specific filter, get all changesets
        debug!("Getting all archived changesets");
        changesets = history
            .list_all()
            .await
            .map_err(|e| CliError::Execution(format!("Failed to list archived changesets: {e}")))?;
    }

    // Apply additional filters that weren't used as primary query
    if !used_package_primary && let Some(package) = &args.package {
        debug!("Applying package filter: {}", package);
        changesets.retain(|cs| cs.changeset.has_package(package));
    }

    if !used_env_primary && let Some(env) = &args.env {
        debug!("Applying environment filter: {}", env);
        changesets.retain(|cs| cs.changeset.environments.contains(env));
    }

    if !used_bump_primary && let Some(bump_str) = &args.bump {
        let bump = parse_bump_type(bump_str)?;
        debug!("Applying bump type filter: {:?}", bump);
        changesets.retain(|cs| cs.changeset.bump == bump);
    }

    // Apply date range filter if it wasn't the primary query
    if !used_date_primary && let Some(since_str) = &args.since {
        let since = parse_date(since_str)?;
        debug!("Applying since date filter: {}", since);
        changesets.retain(|cs| cs.release_info.applied_at >= since);
    }

    if !used_date_primary && let Some(until_str) = &args.until {
        let until = parse_date(until_str)?;
        debug!("Applying until date filter: {}", until);
        changesets.retain(|cs| cs.release_info.applied_at <= until);
    }

    // Sort by release date (most recent first)
    changesets.sort_by(|a, b| b.release_info.applied_at.cmp(&a.release_info.applied_at));

    Ok(changesets)
}

/// Parse a date string into a DateTime<Utc>.
///
/// Supports multiple date formats:
/// - ISO 8601 date: "2025-01-01"
/// - RFC 3339 datetime: "2025-01-01T00:00:00Z"
///
/// # Arguments
///
/// * `date_str` - Date string to parse
///
/// # Errors
///
/// Returns an error if the date string cannot be parsed.
fn parse_date(date_str: &str) -> Result<DateTime<Utc>> {
    // Try parsing as date only first
    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Ok(date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| {
                CliError::Validation(format!("Invalid date components in: {date_str}"))
            })?
            .and_utc());
    }

    // Try parsing as full RFC3339 datetime
    DateTime::parse_from_rfc3339(date_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| {
            CliError::Validation(format!(
                "Invalid date format '{date_str}'. Expected ISO 8601 date (YYYY-MM-DD) or RFC3339 datetime: {e}"
            ))
        })
}

/// Output changesets in JSON format.
fn output_json(output: &Output, changesets: Vec<ArchivedChangeset>) -> Result<()> {
    let items: Vec<ArchivedChangesetInfo> =
        changesets.into_iter().map(std::convert::Into::into).collect();

    let total = items.len();
    let response = JsonResponse::success(ChangesetHistoryResponse {
        success: true,
        changesets: items,
        total,
    });

    output
        .json(&response)
        .map_err(|e| CliError::Execution(format!("Failed to output JSON: {e}")))?;

    Ok(())
}

/// Output changesets in human-readable table format.
fn output_table(output: &Output, changesets: &[ArchivedChangeset]) -> Result<()> {
    let total = changesets.len();

    if total == 0 {
        output
            .info("No archived changesets found")
            .map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;
        output
            .blank_line()
            .map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;
        return Ok(());
    }

    // Create table
    let mut table = TableBuilder::new()
        .theme(TableTheme::Default)
        .columns(&["Branch", "Bump", "Packages", "Environments", "Applied At", "Applied By"])
        .alignment(0, ColumnAlignment::Left)
        .alignment(1, ColumnAlignment::Left)
        .alignment(2, ColumnAlignment::Left)
        .alignment(3, ColumnAlignment::Left)
        .alignment(4, ColumnAlignment::Left)
        .alignment(5, ColumnAlignment::Left)
        .max_width(120)
        .build();

    // Add rows
    for archived in changesets {
        let packages_display = if archived.changeset.packages.is_empty() {
            String::from("-")
        } else if archived.changeset.packages.len() == 1 {
            archived.changeset.packages[0].clone()
        } else {
            format!("{} packages", archived.changeset.packages.len())
        };

        let environments_display = if archived.changeset.environments.is_empty() {
            String::from("-")
        } else {
            archived.changeset.environments.join(", ")
        };

        let bump_display = archived.changeset.bump.to_string().to_lowercase();

        let applied_display = archived.release_info.applied_at.format("%Y-%m-%d %H:%M").to_string();

        table.add_row(&[
            &archived.changeset.branch,
            &bump_display,
            &packages_display,
            &environments_display,
            &applied_display,
            &archived.release_info.applied_by,
        ]);
    }

    // Display table
    output
        .table(&mut table)
        .map_err(|e| CliError::Execution(format!("Failed to render table: {e}")))?;

    output
        .blank_line()
        .map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;

    // Display summary
    let section = Section::new("Summary");
    section.print();
    print_item("Total Archived Changesets", &total.to_string(), true);

    Ok(())
}
