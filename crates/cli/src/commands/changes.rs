//! Changes analysis command implementation.
//!
//! This module implements the `workspace changes` command which analyzes changes in
//! the workspace to detect affected packages and their change statistics.
//!
//! # What
//!
//! Provides the `execute_changes` function that:
//! - Analyzes working directory changes (unstaged/staged/both)
//! - Analyzes commit range changes (between refs)
//! - Analyzes branch comparison changes
//! - Maps changed files to affected packages
//! - Calculates change statistics (lines added/deleted, files changed)
//! - Displays results in table or JSON format
//!
//! # How
//!
//! The command flow:
//! 1. Loads workspace configuration and sets up analyzers
//! 2. Opens Git repository for the workspace
//! 3. Determines analysis mode based on command arguments:
//!    - Working directory: --staged, --unstaged, or both (default)
//!    - Commit range: --since and --until
//!    - Branch comparison: --branch
//! 4. Uses `ChangesAnalyzer` from pkg tools to perform analysis
//! 5. Filters results by --packages if specified
//! 6. Formats output as table (human) or JSON (automation)
//!
//! ## Analysis Modes
//!
//! ### Working Directory Mode (default)
//! - Analyzes uncommitted changes in the working tree
//! - Can filter to only staged (--staged) or unstaged (--unstaged)
//! - Default shows both staged and unstaged changes
//!
//! ### Commit Range Mode (--since / --until)
//! - Analyzes changes between two Git references
//! - --since: Starting reference (exclusive)
//! - --until: Ending reference (default: HEAD)
//! - Shows all commits and files changed in the range
//!
//! ### Branch Comparison Mode (--branch)
//! - Compares current branch against specified branch
//! - Shows changes that exist in current but not in target
//! - Useful for PR/MR review and planning
//!
//! # Why
//!
//! Change analysis is essential for:
//! - Understanding which packages are affected by changes
//! - Planning version bumps and releases
//! - CI/CD integration for affected package detection
//! - Code review and impact analysis
//! - Automated changeset creation
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::commands::changes::execute_changes;
//! use sublime_cli_tools::cli::commands::ChangesArgs;
//! use sublime_cli_tools::output::{Output, OutputFormat};
//! use std::io;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Analyze working directory changes
//! let args = ChangesArgs {
//!     since: None,
//!     until: None,
//!     branch: None,
//!     staged: false,
//!     unstaged: false,
//!     packages: None,
//! };
//!
//! let output = Output::new(OutputFormat::Human, io::stdout(), false);
//! execute_changes(&args, &output, Path::new("."), None).await?;
//! # Ok(())
//! # }
//! ```

use crate::cli::commands::ChangesArgs;
use crate::error::{CliError, Result};
use crate::output::table::{ColumnAlignment, TableBuilder, TableTheme};
use crate::output::{JsonResponse, Output};
use serde::Serialize;
use std::path::Path;
use sublime_git_tools::Repo;
use sublime_pkg_tools::changes::ChangesAnalyzer;
use sublime_pkg_tools::config::ConfigLoader;
use sublime_standard_tools::filesystem::FileSystemManager;
use tracing::{debug, info, warn};

/// Execute the changes analysis command.
///
/// Analyzes changes in the workspace and detects affected packages with detailed
/// statistics about files and lines changed.
///
/// # Arguments
///
/// * `args` - Command arguments specifying analysis mode and filters
/// * `output` - Output handler for formatting results
/// * `root` - Workspace root directory path
/// * `config_path` - Optional custom config file path
///
/// # Returns
///
/// Returns `Ok(())` if analysis succeeds, or an error if:
/// - Git repository cannot be opened
/// - Configuration cannot be loaded
/// - Analysis fails (e.g., invalid refs)
/// - Output formatting fails
///
/// # Errors
///
/// This function will return an error if:
/// - The path is not a Git repository
/// - Git references (--since, --until, --branch) are invalid
/// - Configuration file is invalid
/// - No packages are found in the workspace
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::commands::changes::execute_changes;
/// use sublime_cli_tools::cli::commands::ChangesArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::io;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = ChangesArgs {
///     since: Some("v1.0.0".to_string()),
///     until: Some("HEAD".to_string()),
///     branch: None,
///     staged: false,
///     unstaged: false,
///     packages: None,
/// };
///
/// let output = Output::new(OutputFormat::Human, io::stdout(), false);
/// execute_changes(&args, &output, Path::new("."), None).await?;
/// # Ok(())
/// # }
/// ```
pub async fn execute_changes(
    args: &ChangesArgs,
    output: &Output,
    root: &Path,
    config_path: Option<&Path>,
) -> Result<()> {
    info!("Executing changes analysis command");
    debug!("Workspace root: {}", root.display());
    debug!("Analysis mode: {:?}", determine_mode(args));

    // Load configuration
    let config = if let Some(path) = config_path {
        debug!("Loading config from: {}", path.display());
        ConfigLoader::load_from_file(path).await.map_err(|e| {
            CliError::configuration(format!("Failed to load config from {}: {e}", path.display()))
        })?
    } else {
        debug!("Loading config from workspace root");
        ConfigLoader::load_defaults()
            .await
            .map_err(|e| CliError::configuration(format!("Failed to load config: {e}")))?
    };

    // Open Git repository
    debug!("Opening Git repository at: {}", root.display());
    let repo = Repo::open(root.to_str().ok_or_else(|| {
        CliError::execution("Workspace root path contains invalid UTF-8".to_string())
    })?)
    .map_err(|e| {
        CliError::git(format!("Failed to open Git repository at {}: {e}", root.display()))
    })?;

    // Create filesystem manager
    let fs = FileSystemManager::new();

    // Create changes analyzer
    debug!("Creating changes analyzer");
    let analyzer = ChangesAnalyzer::new(root.to_path_buf(), repo, fs, config)
        .await
        .map_err(|e| CliError::execution(format!("Failed to create changes analyzer: {e}")))?;

    // Perform analysis based on mode
    let report = match determine_mode(args) {
        AnalysisMode::WorkingDirectory { staged, unstaged } => {
            info!("Analyzing working directory changes (staged: {staged}, unstaged: {unstaged})");

            // TODO: Future enhancement for pkg crate
            // The ChangesAnalyzer currently only supports analyze_working_directory()
            // which analyzes all changes (staged + unstaged). Filtering by staged/unstaged
            // would require either:
            // 1. Adding analyze_staged() and analyze_unstaged() methods to ChangesAnalyzer
            // 2. Filtering the results here based on git status flags
            //
            // For now, we analyze all working directory changes and warn if filtering was requested
            if staged && !unstaged {
                warn!(
                    "--staged flag is not yet fully implemented, showing all working directory changes"
                );
            } else if unstaged && !staged {
                warn!(
                    "--unstaged flag is not yet fully implemented, showing all working directory changes"
                );
            }

            // Analyze all working directory changes
            analyzer.analyze_working_directory().await.map_err(|e| {
                CliError::execution(format!("Failed to analyze working directory: {e}"))
            })?
        }
        AnalysisMode::CommitRange { from, to } => {
            info!("Analyzing commit range: {from}..{to}");
            analyzer.analyze_commit_range(&from, &to).await.map_err(|e| {
                CliError::execution(format!("Failed to analyze commit range {from}..{to}: {e}"))
            })?
        }
        AnalysisMode::BranchComparison { target } => {
            info!("Comparing current branch against: {target}");

            // Get current branch from Git repository
            let current_branch = analyzer
                .git_repo()
                .get_current_branch()
                .map_err(|e| CliError::git(format!("Failed to get current branch: {e}")))?;

            debug!("Current branch: {current_branch}");

            // Analyze changes between branches
            analyzer.analyze_commit_range(&target, &current_branch).await.map_err(|e| {
                CliError::execution(format!(
                    "Failed to compare branches {target}..{current_branch}: {e}"
                ))
            })?
        }
    };

    // Filter by packages if specified
    let filtered_report = if let Some(ref filter_packages) = args.packages {
        debug!("Filtering by packages: {:?}", filter_packages);
        filter_report_by_packages(report, filter_packages)
    } else {
        report
    };

    // Output results
    if output.format().is_json() {
        output_json(output, &filtered_report)?;
    } else {
        output_human(output, &filtered_report, args)?;
    }

    Ok(())
}

/// Determines the analysis mode from command arguments.
///
/// Returns the appropriate mode based on which flags and options are set:
/// - Branch comparison if --branch is specified
/// - Commit range if --since or --until is specified
/// - Working directory otherwise (with staged/unstaged flags)
///
/// # Arguments
///
/// * `args` - Command arguments
///
/// # Returns
///
/// The determined analysis mode
pub(crate) fn determine_mode(args: &ChangesArgs) -> AnalysisMode {
    if let Some(ref branch) = args.branch {
        AnalysisMode::BranchComparison { target: branch.clone() }
    } else if args.since.is_some() || args.until.is_some() {
        let from = args.since.as_deref().unwrap_or("HEAD~1");
        let to = args.until.as_deref().unwrap_or("HEAD");
        AnalysisMode::CommitRange { from: from.to_string(), to: to.to_string() }
    } else {
        AnalysisMode::WorkingDirectory { staged: args.staged, unstaged: args.unstaged }
    }
}

/// Analysis mode determined from command arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AnalysisMode {
    /// Working directory analysis with optional staged/unstaged filters
    WorkingDirectory {
        /// Only analyze staged changes
        staged: bool,
        /// Only analyze unstaged changes
        unstaged: bool,
    },
    /// Commit range analysis between two refs
    CommitRange {
        /// Starting reference (exclusive)
        from: String,
        /// Ending reference (inclusive)
        to: String,
    },
    /// Branch comparison against target branch
    BranchComparison {
        /// Target branch to compare against
        target: String,
    },
}

/// Filters a changes report to only include specified packages.
///
/// # Arguments
///
/// * `report` - The complete changes report
/// * `package_names` - Package names to include
///
/// # Returns
///
/// A new report containing only the filtered packages
pub(crate) fn filter_report_by_packages(
    mut report: sublime_pkg_tools::changes::ChangesReport,
    package_names: &[String],
) -> sublime_pkg_tools::changes::ChangesReport {
    report.packages.retain(|pkg| package_names.contains(&pkg.package_name));

    // Update summary
    report.summary.packages_with_changes = report.packages.len();
    report.summary.total_files_changed = report.packages.iter().map(|p| p.files.len()).sum();
    report.summary.total_commits = report.packages.iter().map(|p| p.commits.len()).sum();

    report
}

/// Outputs the changes report in JSON format.
///
/// Creates a structured JSON response with package changes and summary statistics.
///
/// # Arguments
///
/// * `output` - Output handler
/// * `report` - Changes report to output
///
/// # Returns
///
/// Returns `Ok(())` if output succeeds, or an error if JSON serialization fails
fn output_json(output: &Output, report: &sublime_pkg_tools::changes::ChangesReport) -> Result<()> {
    debug!("Formatting output as JSON");

    let response = JsonResponse::success(ChangesJsonResponse::from_report(report));
    output.json(&response)?;

    Ok(())
}

/// Outputs the changes report in human-readable format.
///
/// Displays a formatted table with package changes and statistics, followed by
/// a summary section.
///
/// # Arguments
///
/// * `output` - Output handler
/// * `report` - Changes report to output
/// * `args` - Command arguments (for context in output)
///
/// # Returns
///
/// Returns `Ok(())` if output succeeds, or an error if rendering fails
fn output_human(
    output: &Output,
    report: &sublime_pkg_tools::changes::ChangesReport,
    args: &ChangesArgs,
) -> Result<()> {
    debug!("Formatting output as human-readable");

    // Show analysis context
    display_analysis_context(output, args, report)?;

    output.blank_line()?;

    // Check if there are any changes
    if report.summary.packages_with_changes == 0 {
        output.info("No changes detected")?;
        return Ok(());
    }

    // Display affected packages table
    output.info(&format!("Affected Packages: {}", report.summary.packages_with_changes))?;
    output.blank_line()?;

    display_packages_table(output, report)?;

    // Display summary
    output.blank_line()?;
    display_summary(output, &report.summary)?;

    Ok(())
}

/// Displays the analysis context (mode and parameters).
///
/// Shows what type of analysis was performed and with what parameters.
fn display_analysis_context(
    output: &Output,
    args: &ChangesArgs,
    report: &sublime_pkg_tools::changes::ChangesReport,
) -> Result<()> {
    use sublime_pkg_tools::changes::AnalysisMode as PkgAnalysisMode;

    let context_msg = match &report.analysis_mode {
        PkgAnalysisMode::WorkingDirectory => {
            if args.staged {
                "Staged Changes"
            } else if args.unstaged {
                "Unstaged Changes"
            } else {
                "Working Directory Changes"
            }
        }
        PkgAnalysisMode::CommitRange => {
            if let Some(ref branch) = args.branch {
                return output.info(&format!("Branch Comparison: current vs {branch}"));
            }
            let from = args.since.as_deref().unwrap_or("HEAD~1");
            let to = args.until.as_deref().unwrap_or("HEAD");
            return output.info(&format!("Commit Range: {from}..{to}"));
        }
        PkgAnalysisMode::SingleCommit => "Single Commit Changes",
        PkgAnalysisMode::CommitList => "Changes from Commit List",
    };

    output.info(context_msg)?;
    Ok(())
}

/// Displays a table of affected packages with their change statistics.
///
/// # Arguments
///
/// * `output` - Output handler
/// * `report` - Changes report containing package data
///
/// # Returns
///
/// Returns `Ok(())` if table rendering succeeds
fn display_packages_table(
    output: &Output,
    report: &sublime_pkg_tools::changes::ChangesReport,
) -> Result<()> {
    let mut table = TableBuilder::new()
        .theme(TableTheme::Minimal)
        .columns(&["Package", "Files", "+Lines", "-Lines", "Changes"])
        .alignment(1, ColumnAlignment::Right)
        .alignment(2, ColumnAlignment::Right)
        .alignment(3, ColumnAlignment::Right)
        .alignment(4, ColumnAlignment::Left)
        .build();

    for package in &report.packages {
        // Get change type summary
        let change_summary = if package.files.is_empty() {
            "-".to_string()
        } else {
            format_change_types(&package.stats)
        };

        table.add_row(&[
            package.package_name.as_str(),
            &package
                .stats
                .files_added
                .saturating_add(package.stats.files_modified)
                .saturating_add(package.stats.files_deleted)
                .to_string(),
            &package.stats.lines_added.to_string(),
            &package.stats.lines_deleted.to_string(),
            &change_summary,
        ]);
    }

    output.table(&mut table)?;
    Ok(())
}

/// Formats change type statistics as a concise string.
///
/// Returns a string like "M:5 A:2 D:1" for modified, added, deleted files.
pub(crate) fn format_change_types(
    stats: &sublime_pkg_tools::changes::PackageChangeStats,
) -> String {
    let mut parts = Vec::new();

    if stats.files_modified > 0 {
        parts.push(format!("M:{}", stats.files_modified));
    }
    if stats.files_added > 0 {
        parts.push(format!("A:{}", stats.files_added));
    }
    if stats.files_deleted > 0 {
        parts.push(format!("D:{}", stats.files_deleted));
    }

    if parts.is_empty() { "-".to_string() } else { parts.join(" ") }
}

/// Displays a summary of all changes.
///
/// # Arguments
///
/// * `output` - Output handler
/// * `summary` - Changes summary data
///
/// # Returns
///
/// Returns `Ok(())` if output succeeds
fn display_summary(
    output: &Output,
    summary: &sublime_pkg_tools::changes::ChangesSummary,
) -> Result<()> {
    output.info("Summary:")?;
    output.plain(&format!("  Total packages: {}", summary.total_packages))?;
    output.plain(&format!("  Packages with changes: {}", summary.packages_with_changes))?;
    output.plain(&format!("  Files changed: {}", summary.total_files_changed))?;

    if summary.total_commits > 0 {
        output.plain(&format!("  Commits: {}", summary.total_commits))?;
    }

    Ok(())
}

// ============================================================================
// JSON Response Types
// ============================================================================

/// JSON response structure for changes analysis.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChangesJsonResponse {
    /// List of affected packages
    affected_packages: Vec<PackageChangesJson>,
    /// Summary statistics
    summary: ChangesSummaryJson,
}

impl ChangesJsonResponse {
    /// Creates a JSON response from a changes report.
    fn from_report(report: &sublime_pkg_tools::changes::ChangesReport) -> Self {
        Self {
            affected_packages: report
                .packages
                .iter()
                .map(PackageChangesJson::from_package)
                .collect(),
            summary: ChangesSummaryJson::from_summary(&report.summary, &report.packages),
        }
    }
}

/// JSON representation of package changes.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PackageChangesJson {
    /// Package name
    name: String,
    /// Package path relative to workspace root
    path: String,
    /// Number of files changed
    files_changed: usize,
    /// Number of lines added
    lines_added: usize,
    /// Number of lines deleted
    lines_deleted: usize,
    /// List of changed files
    changes: Vec<FileChangeJson>,
}

impl PackageChangesJson {
    /// Creates a JSON representation from a PackageChanges.
    fn from_package(package: &sublime_pkg_tools::changes::PackageChanges) -> Self {
        Self {
            name: package.package_name.clone(),
            path: package.package_location.to_str().unwrap_or("<invalid-utf8>").to_string(),
            files_changed: package.files.len(),
            lines_added: package.stats.lines_added,
            lines_deleted: package.stats.lines_deleted,
            changes: package.files.iter().map(FileChangeJson::from_file).collect(),
        }
    }
}

/// JSON representation of a file change.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileChangeJson {
    /// Change type (added, modified, deleted, renamed, copied)
    #[serde(rename = "type")]
    change_type: String,
    /// File path
    path: String,
    /// Number of lines added (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    lines_added: Option<usize>,
    /// Number of lines deleted (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    lines_deleted: Option<usize>,
}

impl FileChangeJson {
    /// Creates a JSON representation from a FileChange.
    fn from_file(file: &sublime_pkg_tools::changes::FileChange) -> Self {
        use sublime_pkg_tools::changes::FileChangeType;

        let change_type = match &file.change_type {
            FileChangeType::Added => "added",
            FileChangeType::Modified => "modified",
            FileChangeType::Deleted => "deleted",
            FileChangeType::Renamed => "renamed",
            FileChangeType::Copied => "copied",
            FileChangeType::Untracked => "untracked",
        };

        Self {
            change_type: change_type.to_string(),
            path: file.path.to_str().unwrap_or("<invalid-utf8>").to_string(),
            lines_added: file.lines_added,
            lines_deleted: file.lines_deleted,
        }
    }
}

/// JSON representation of changes summary.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChangesSummaryJson {
    /// Total number of files changed
    total_files: usize,
    /// Total number of packages
    total_packages: usize,
    /// Number of packages with changes
    packages_with_changes: usize,
    /// Total number of lines added
    lines_added: usize,
    /// Total number of lines deleted
    lines_deleted: usize,
}

impl ChangesSummaryJson {
    /// Creates a JSON summary from a ChangesSummary and packages data.
    fn from_summary(
        summary: &sublime_pkg_tools::changes::ChangesSummary,
        packages: &[sublime_pkg_tools::changes::PackageChanges],
    ) -> Self {
        // Calculate total lines from all packages
        let (total_lines_added, total_lines_deleted) =
            packages.iter().fold((0, 0), |(added, deleted), pkg| {
                (added + pkg.stats.lines_added, deleted + pkg.stats.lines_deleted)
            });

        Self {
            total_files: summary.total_files_changed,
            total_packages: summary.total_packages,
            packages_with_changes: summary.packages_with_changes,
            lines_added: total_lines_added,
            lines_deleted: total_lines_deleted,
        }
    }
}
