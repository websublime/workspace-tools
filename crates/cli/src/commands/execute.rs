//! Workspace execute command implementation.
//!
//! This module implements the `workspace execute` command which runs commands
//! across workspace packages with optional filtering and parallel execution.
//!
//! # What
//!
//! Provides the `execute_execute` function that:
//! - Parses command type (npm script or system command)
//! - Detects workspace packages
//! - Applies package filtering if specified
//! - Validates npm scripts exist before execution
//! - Executes commands with real-time streaming output
//! - Supports sequential (default) or parallel execution
//! - Reports execution summary with success/failure counts
//!
//! # How
//!
//! The command flow:
//! 1. Parses the `--cmd` parameter to determine command type
//! 2. Detects all workspace packages via monorepo detector
//! 3. Filters packages if `--filter-package` is specified
//! 4. For npm scripts, validates the script exists in each package.json
//! 5. Executes commands sequentially or in parallel based on `--parallel` flag
//! 6. Streams output in real-time to stdout
//! 7. Collects results and displays summary
//!
//! # Why
//!
//! Cross-package command execution is essential for:
//! - Running tests, linting, or builds across all packages
//! - CI/CD pipelines that need consistent command execution
//! - Development workflows with monorepo tooling
//! - Scripting and automation with filtered package execution
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::commands::execute::execute_execute;
//! use sublime_cli_tools::cli::commands::ExecuteArgs;
//! use sublime_cli_tools::output::{Output, OutputFormat};
//! use std::io;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let args = ExecuteArgs {
//!     cmd: "npm:lint".to_string(),
//!     filter_package: None,
//!     parallel: false,
//!     args: vec![],
//! };
//! let output = Output::new(OutputFormat::Human, io::stdout(), false);
//! execute_execute(&args, &output, Path::new(".")).await?;
//! # Ok(())
//! # }
//! ```

use crate::cli::commands::ExecuteArgs;
use crate::error::{CliError, Result};
use crate::output::{JsonResponse, Output};
use serde::Serialize;
use std::path::Path;
use std::time::{Duration, Instant};
use sublime_git_tools::Repo;
use sublime_pkg_tools::changes::ChangesAnalyzer;
use sublime_standard_tools::command::{Command, CommandBuilder, DefaultCommandExecutor, Executor};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait, WorkspacePackage};
use sublime_standard_tools::node::PackageManager;
use tokio::task::JoinSet;
use tracing::{debug, info, warn};

// ============================================================================
// Command Type
// ============================================================================

/// Type of command to execute.
///
/// Determines how the command string should be interpreted and executed.
#[derive(Debug, Clone)]
enum CommandType {
    /// npm script (e.g., npm:build -> npm run build).
    NpmScript {
        /// Script name from package.json scripts.
        script: String,
    },
    /// System command (e.g., node index.js).
    System {
        /// Program to execute.
        program: String,
        /// Arguments to pass to the program.
        args: Vec<String>,
    },
}

impl CommandType {
    /// Parse command string into CommandType.
    ///
    /// Commands prefixed with `npm:` are treated as npm scripts.
    /// All other commands are treated as system commands.
    ///
    /// # Arguments
    ///
    /// * `cmd` - The command string to parse
    ///
    /// # Returns
    ///
    /// The parsed command type.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let npm_cmd = CommandType::parse("npm:lint");
    /// // CommandType::NpmScript { script: "lint" }
    ///
    /// let sys_cmd = CommandType::parse("node index.js");
    /// // CommandType::System { program: "node", args: ["index.js"] }
    /// ```
    fn parse(cmd: &str) -> Self {
        if let Some(script) = cmd.strip_prefix("npm:") {
            CommandType::NpmScript { script: script.to_string() }
        } else {
            // Parse as system command - split on whitespace
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.is_empty() {
                CommandType::System { program: String::new(), args: Vec::new() }
            } else {
                CommandType::System {
                    program: parts[0].to_string(),
                    args: parts[1..].iter().map(|s| (*s).to_string()).collect(),
                }
            }
        }
    }

    /// Returns a display string for the command type.
    fn display(&self) -> String {
        match self {
            CommandType::NpmScript { script } => format!("npm run {script}"),
            CommandType::System { program, args } => {
                if args.is_empty() {
                    program.clone()
                } else {
                    let args_str = args.join(" ");
                    format!("{program} {args_str}")
                }
            }
        }
    }
}

// ============================================================================
// JSON Response Types
// ============================================================================

/// JSON response for execute command.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecuteJsonResponse {
    /// Command that was executed.
    command: String,
    /// Results for each package.
    results: Vec<PackageExecutionResultJson>,
    /// Execution summary.
    summary: ExecuteSummaryJson,
}

/// Result for a single package execution.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PackageExecutionResultJson {
    /// Package name.
    package: String,
    /// Whether execution succeeded.
    success: bool,
    /// Exit code from the command.
    exit_code: i32,
    /// Execution duration in milliseconds.
    duration_ms: u64,
    /// Error message if execution failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Execution summary for JSON output.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecuteSummaryJson {
    /// Total number of packages.
    total: usize,
    /// Number of successful executions.
    succeeded: usize,
    /// Number of failed executions.
    failed: usize,
    /// Total execution duration in milliseconds.
    total_duration_ms: u64,
}

/// Internal result type for package execution.
struct PackageResult {
    package_name: String,
    success: bool,
    exit_code: i32,
    duration: Duration,
    error: Option<String>,
    stdout: String,
    stderr: String,
}

// ============================================================================
// Command Implementation
// ============================================================================

/// Execute the workspace execute command.
///
/// Runs commands across workspace packages with optional filtering and
/// parallel execution support.
///
/// # Arguments
///
/// * `args` - Command arguments including cmd, filter, parallel flag
/// * `output` - Output handler for formatting results
/// * `root` - Workspace root directory path
/// * `_config_path` - Optional custom config file path (unused)
///
/// # Returns
///
/// Returns `Ok(())` if all commands succeed, or an error if:
/// - The workspace is not a valid monorepo
/// - No packages match the filter
/// - An npm script doesn't exist in a filtered package
///
/// # Errors
///
/// This function will return an error if:
/// - No packages are found in the workspace
/// - Filtered packages don't exist
/// - An npm script is not defined in a package.json
///
/// Command execution failures are recorded but don't cause the function to return
/// an error - instead, they're reported in the summary and the exit code reflects
/// the failure.
pub async fn execute_execute(args: &ExecuteArgs, output: &Output, root: &Path) -> Result<()> {
    info!("Executing workspace execute command");
    debug!("Command: {}", args.cmd);
    debug!("Filter: {:?}", args.filter_package);
    debug!("Affected: {}", args.affected);
    debug!("Parallel: {}", args.parallel);

    let fs = FileSystemManager::new();

    // Parse command type
    let cmd_type = CommandType::parse(&args.cmd);
    debug!("Parsed command type: {:?}", cmd_type);

    // Detect workspace packages based on mode (filter, affected, or all)
    let detector = MonorepoDetector::new();
    let packages = if args.affected {
        get_affected_packages(&detector, &fs, root, args, output).await?
    } else {
        get_target_packages(&detector, root, args.filter_package.as_ref()).await?
    };

    // Handle empty packages case
    if packages.is_empty() {
        if args.affected {
            // For --affected mode, no packages is a success (nothing to do)
            if output.format().is_json() {
                let response = ExecuteJsonResponse {
                    command: cmd_type.display(),
                    results: vec![],
                    summary: ExecuteSummaryJson {
                        total: 0,
                        succeeded: 0,
                        failed: 0,
                        total_duration_ms: 0,
                    },
                };
                output.json(&JsonResponse::success(response))?;
            } else {
                output.info("No affected packages found. Nothing to execute.")?;
            }
            return Ok(());
        }
        return Err(CliError::validation("No packages found in workspace"));
    }

    // For npm scripts, validate they exist in all target packages
    if let CommandType::NpmScript { ref script } = cmd_type {
        validate_npm_scripts(&packages, script, &fs).await?;
    }

    // Detect package manager for npm scripts
    let pm_command = detect_package_manager_command(root);

    // Execute commands
    let start_time = Instant::now();
    let results = if args.parallel {
        execute_parallel(&packages, &cmd_type, &pm_command, &args.args, output).await
    } else {
        execute_sequential(&packages, &cmd_type, &pm_command, &args.args, output).await
    };
    let total_duration = start_time.elapsed();

    // Generate summary
    let summary = generate_summary(&results, total_duration);

    // Format output
    if output.format().is_json() {
        output_json(output, &cmd_type, &results, &summary)?;
    } else {
        output_human_summary(output, &results, &summary)?;
    }

    // Return error if any command failed (for proper exit code)
    if summary.failed > 0 {
        return Err(CliError::execution(format!(
            "{} of {} commands failed",
            summary.failed, summary.total
        )));
    }

    Ok(())
}

/// Gets target packages, applying filter if specified.
async fn get_target_packages(
    detector: &MonorepoDetector,
    root: &Path,
    filter: Option<&Vec<String>>,
) -> Result<Vec<WorkspacePackage>> {
    // First try to detect as monorepo
    let packages = match detector.detect_packages(root).await {
        Ok(pkgs) if !pkgs.is_empty() => pkgs,
        Ok(_) | Err(_) => {
            // Fall back to root package for simple repos
            let fs = FileSystemManager::new();
            let package_json_path = root.join("package.json");

            if !fs.exists(&package_json_path).await {
                return Err(CliError::validation(
                    "No package.json found. Not a valid Node.js project.",
                ));
            }

            match fs.read_file_string(&package_json_path).await {
                Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json) => {
                        let name = json
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unnamed")
                            .to_string();
                        let version = json
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("0.0.0")
                            .to_string();

                        vec![WorkspacePackage {
                            name,
                            version,
                            location: ".".into(),
                            absolute_path: root.to_path_buf(),
                            workspace_dependencies: Vec::new(),
                            workspace_dev_dependencies: Vec::new(),
                        }]
                    }
                    Err(e) => {
                        return Err(CliError::validation(format!(
                            "Failed to parse package.json: {e}"
                        )));
                    }
                },
                Err(e) => {
                    return Err(CliError::validation(format!("Failed to read package.json: {e}")));
                }
            }
        }
    };

    // Apply filter if specified
    if let Some(filter_names) = filter {
        let filtered: Vec<WorkspacePackage> =
            packages.into_iter().filter(|p| filter_names.contains(&p.name)).collect();

        if filtered.is_empty() {
            return Err(CliError::validation(format!(
                "No packages match filter: {}",
                filter_names.join(", ")
            )));
        }

        // Check for packages in filter that don't exist
        let found_names: Vec<&str> = filtered.iter().map(|p| p.name.as_str()).collect();
        let not_found: Vec<&String> =
            filter_names.iter().filter(|name| !found_names.contains(&name.as_str())).collect();

        if !not_found.is_empty() {
            warn!(
                "Some filtered packages not found: {}",
                not_found.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
            );
        }

        Ok(filtered)
    } else {
        Ok(packages)
    }
}

/// Gets affected packages based on Git changes.
///
/// Uses the `ChangesAnalyzer` to detect which packages have been modified.
/// Supports three analysis modes:
/// - Working directory (default): Analyzes staged + unstaged changes
/// - Commit range: When `--since` is specified, analyzes changes between refs
/// - Branch comparison: When `--branch` is specified, compares against target branch
///
/// # Arguments
///
/// * `detector` - Monorepo detector for getting all packages
/// * `fs` - Filesystem manager for file operations
/// * `root` - Workspace root directory path
/// * `args` - Execute arguments containing affected detection options
/// * `output` - Output handler for displaying analysis mode info
///
/// # Returns
///
/// Returns a vector of `WorkspacePackage` that have been affected by changes.
///
/// # Errors
///
/// Returns an error if:
/// - Git repository cannot be opened
/// - Changes analysis fails
/// - Invalid Git references are provided
async fn get_affected_packages(
    detector: &MonorepoDetector,
    fs: &FileSystemManager,
    root: &Path,
    args: &ExecuteArgs,
    output: &Output,
) -> Result<Vec<WorkspacePackage>> {
    debug!("Detecting affected packages");
    debug!("Since: {:?}, Until: {:?}, Branch: {:?}", args.since, args.until, args.branch);

    // Open Git repository
    let repo = Repo::open(root.to_str().ok_or_else(|| {
        CliError::execution("Workspace root path contains invalid UTF-8".to_string())
    })?)
    .map_err(|e| {
        CliError::git(format!("Failed to open Git repository at {}: {e}", root.display()))
    })?;

    // Load configuration (use default if not found)
    let config = crate::commands::find_and_load_config(root, None).await?.unwrap_or_default();

    // Create changes analyzer
    let analyzer = ChangesAnalyzer::new(root.to_path_buf(), repo, fs.clone(), config)
        .await
        .map_err(|e| CliError::execution(format!("Failed to create changes analyzer: {e}")))?;

    // Determine analysis mode and perform analysis
    let report = if let Some(ref branch) = args.branch {
        // Branch comparison mode
        if !output.format().is_json() {
            output.info(&format!("Analyzing affected packages (comparing with {branch})..."))?;
        }
        info!("Analyzing changes comparing with branch: {branch}");

        let current_branch = analyzer
            .git_repo()
            .get_current_branch()
            .map_err(|e| CliError::git(format!("Failed to get current branch: {e}")))?;

        analyzer.analyze_commit_range(branch, &current_branch).await.map_err(|e| {
            CliError::execution(format!(
                "Failed to compare branches {branch}..{current_branch}: {e}"
            ))
        })?
    } else if args.since.is_some() || args.until.is_some() {
        // Commit range mode
        let from = args.since.as_deref().unwrap_or("HEAD~1");
        let to = args.until.as_deref().unwrap_or("HEAD");

        if !output.format().is_json() {
            output.info(&format!("Analyzing affected packages ({from}..{to})..."))?;
        }
        info!("Analyzing changes in commit range: {from}..{to}");

        analyzer.analyze_commit_range(from, to).await.map_err(|e| {
            CliError::execution(format!("Failed to analyze commit range {from}..{to}: {e}"))
        })?
    } else {
        // Working directory mode (default)
        if !output.format().is_json() {
            output.info("Analyzing affected packages (working directory)...")?;
        }
        info!("Analyzing working directory changes");

        analyzer
            .analyze_working_directory()
            .await
            .map_err(|e| CliError::execution(format!("Failed to analyze working directory: {e}")))?
    };

    // Extract affected package names from the report (only packages with actual changes)
    let affected_names: Vec<String> =
        report.packages_with_changes().iter().map(|p| p.package_name.clone()).collect();

    if affected_names.is_empty() {
        debug!("No affected packages found");
        return Ok(vec![]);
    }

    info!("Found {} affected packages: {:?}", affected_names.len(), affected_names);

    // Get all workspace packages and filter to only affected ones
    let all_packages = match detector.detect_packages(root).await {
        Ok(pkgs) if !pkgs.is_empty() => pkgs,
        Ok(_) | Err(_) => {
            // Fall back to root package for simple repos
            let package_json_path = root.join("package.json");

            if !fs.exists(&package_json_path).await {
                return Err(CliError::validation(
                    "No package.json found. Not a valid Node.js project.",
                ));
            }

            let content = fs
                .read_file_string(&package_json_path)
                .await
                .map_err(|e| CliError::validation(format!("Failed to read package.json: {e}")))?;

            let json: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| CliError::validation(format!("Failed to parse package.json: {e}")))?;

            let name = json.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed").to_string();
            let version =
                json.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();

            vec![WorkspacePackage {
                name,
                version,
                location: ".".into(),
                absolute_path: root.to_path_buf(),
                workspace_dependencies: Vec::new(),
                workspace_dev_dependencies: Vec::new(),
            }]
        }
    };

    // Filter to only affected packages
    let affected_packages: Vec<WorkspacePackage> =
        all_packages.into_iter().filter(|p| affected_names.contains(&p.name)).collect();

    if !output.format().is_json() && !affected_packages.is_empty() {
        let names: Vec<&str> = affected_packages.iter().map(|p| p.name.as_str()).collect();
        output.info(&format!("Affected packages: {}", names.join(", ")))?;
        output.blank_line()?;
    }

    Ok(affected_packages)
}

/// Validates that npm scripts exist in all target packages.
async fn validate_npm_scripts(
    packages: &[WorkspacePackage],
    script: &str,
    fs: &FileSystemManager,
) -> Result<()> {
    for package in packages {
        let package_json_path = package.absolute_path.join("package.json");

        let content = fs.read_file_string(&package_json_path).await.map_err(|e| {
            CliError::validation(format!(
                "Failed to read package.json for '{}': {}",
                package.name, e
            ))
        })?;

        let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            CliError::validation(format!(
                "Failed to parse package.json for '{}': {}",
                package.name, e
            ))
        })?;

        let has_script = json.get("scripts").and_then(|s| s.get(script)).is_some();

        if !has_script {
            return Err(CliError::validation(format!(
                "Script '{}' not found in package '{}'",
                script, package.name
            )));
        }
    }

    Ok(())
}

/// Detects package manager command to use.
fn detect_package_manager_command(root: &Path) -> String {
    match PackageManager::detect(root) {
        Ok(pm) => pm.kind().command().to_string(),
        Err(_) => "npm".to_string(), // Default to npm
    }
}

/// Executes commands sequentially across packages.
async fn execute_sequential(
    packages: &[WorkspacePackage],
    cmd_type: &CommandType,
    pm_command: &str,
    extra_args: &[String],
    output: &Output,
) -> Vec<PackageResult> {
    let mut results = Vec::with_capacity(packages.len());
    let executor = DefaultCommandExecutor::new();

    for package in packages {
        // Print package header
        if !output.format().is_json() {
            let pkg_name = &package.name;
            let header = match cmd_type {
                CommandType::NpmScript { script } => format!("{pkg_name} ({script})"),
                CommandType::System { program, .. } => format!("{pkg_name} ({program})"),
            };
            let _ = output.info(&header);
        }

        let result =
            execute_single_package(package, cmd_type, pm_command, extra_args, &executor, output)
                .await;

        results.push(result);

        // Add blank line between packages
        if !output.format().is_json() {
            let _ = output.blank_line();
        }
    }

    results
}

/// Executes commands in parallel across packages.
///
/// Uses tokio's `JoinSet` to spawn concurrent tasks for each package.
/// Results are collected and output after all tasks complete to avoid
/// interleaved output.
async fn execute_parallel(
    packages: &[WorkspacePackage],
    cmd_type: &CommandType,
    pm_command: &str,
    extra_args: &[String],
    output: &Output,
) -> Vec<PackageResult> {
    let mut set = JoinSet::new();

    // Spawn tasks for all packages
    for package in packages {
        let cmd_type = cmd_type.clone();
        let pm_command = pm_command.to_string();
        let extra_args = extra_args.to_vec();
        let package = package.clone();

        set.spawn(async move {
            let executor = DefaultCommandExecutor::new();
            execute_single_package_buffered(
                &package,
                &cmd_type,
                &pm_command,
                &extra_args,
                &executor,
            )
            .await
        });
    }

    // Collect results from all spawned tasks
    let mut results = Vec::with_capacity(packages.len());
    while let Some(res) = set.join_next().await {
        match res {
            Ok(result) => results.push(result),
            Err(e) => {
                // Handle join error (task panicked)
                results.push(PackageResult {
                    package_name: "unknown".to_string(),
                    success: false,
                    exit_code: -1,
                    duration: Duration::ZERO,
                    error: Some(format!("Task panicked: {e}")),
                    stdout: String::new(),
                    stderr: String::new(),
                });
            }
        }
    }

    // Output results after all complete (for parallel mode)
    if !output.format().is_json() {
        for result in &results {
            let pkg_name = &result.package_name;
            let header = match cmd_type {
                CommandType::NpmScript { script } => {
                    format!("{pkg_name} ({script})")
                }
                CommandType::System { program, .. } => {
                    format!("{pkg_name} ({program})")
                }
            };
            let _ = output.info(&header);

            // Output buffered stdout/stderr
            if !result.stdout.is_empty() {
                let _ = output.plain(&result.stdout);
            }
            if !result.stderr.is_empty() {
                let _ = output.plain(&result.stderr);
            }

            let _ = output.blank_line();
        }
    }

    results
}

/// Executes command for a single package with streaming output.
async fn execute_single_package(
    package: &WorkspacePackage,
    cmd_type: &CommandType,
    pm_command: &str,
    extra_args: &[String],
    executor: &DefaultCommandExecutor,
    output: &Output,
) -> PackageResult {
    let start = Instant::now();

    let command = build_command(cmd_type, pm_command, &package.absolute_path, extra_args);

    match executor.execute(command).await {
        Ok(cmd_output) => {
            let duration = start.elapsed();

            // Output stdout/stderr in real-time (non-JSON mode)
            if !output.format().is_json() {
                if !cmd_output.stdout().is_empty() {
                    let _ = output.plain(cmd_output.stdout().trim_end());
                }
                if !cmd_output.stderr().is_empty() {
                    let _ = output.plain(cmd_output.stderr().trim_end());
                }
            }

            let success = cmd_output.success();
            let exit_code = cmd_output.status();

            PackageResult {
                package_name: package.name.clone(),
                success,
                exit_code,
                duration,
                error: if success { None } else { Some(format!("Exit code: {exit_code}")) },
                stdout: cmd_output.stdout().to_string(),
                stderr: cmd_output.stderr().to_string(),
            }
        }
        Err(e) => {
            let duration = start.elapsed();
            let error_msg = format!("Execution failed: {e}");

            if !output.format().is_json() {
                let _ = output.error(&error_msg);
            }

            PackageResult {
                package_name: package.name.clone(),
                success: false,
                exit_code: -1,
                duration,
                error: Some(error_msg),
                stdout: String::new(),
                stderr: String::new(),
            }
        }
    }
}

/// Executes command for a single package with buffered output (for parallel execution).
async fn execute_single_package_buffered(
    package: &WorkspacePackage,
    cmd_type: &CommandType,
    pm_command: &str,
    extra_args: &[String],
    executor: &DefaultCommandExecutor,
) -> PackageResult {
    let start = Instant::now();

    let command = build_command(cmd_type, pm_command, &package.absolute_path, extra_args);

    match executor.execute(command).await {
        Ok(cmd_output) => {
            let duration = start.elapsed();
            let success = cmd_output.success();
            let exit_code = cmd_output.status();

            PackageResult {
                package_name: package.name.clone(),
                success,
                exit_code,
                duration,
                error: if success { None } else { Some(format!("Exit code: {exit_code}")) },
                stdout: cmd_output.stdout().to_string(),
                stderr: cmd_output.stderr().to_string(),
            }
        }
        Err(e) => {
            let duration = start.elapsed();

            PackageResult {
                package_name: package.name.clone(),
                success: false,
                exit_code: -1,
                duration,
                error: Some(format!("Execution failed: {e}")),
                stdout: String::new(),
                stderr: String::new(),
            }
        }
    }
}

/// Builds the command to execute.
fn build_command(
    cmd_type: &CommandType,
    pm_command: &str,
    working_dir: &Path,
    extra_args: &[String],
) -> Command {
    match cmd_type {
        CommandType::NpmScript { script } => {
            let mut builder =
                CommandBuilder::new(pm_command).arg("run").arg(script).current_dir(working_dir);

            for arg in extra_args {
                builder = builder.arg(arg);
            }

            builder.build()
        }
        CommandType::System { program, args } => {
            let mut builder = CommandBuilder::new(program).current_dir(working_dir);

            for arg in args {
                builder = builder.arg(arg);
            }
            for arg in extra_args {
                builder = builder.arg(arg);
            }

            builder.build()
        }
    }
}

/// Generates execution summary from results.
///
/// # Note on cast truncation
///
/// The `as_millis()` returns u128, but for any practical execution duration,
/// the value will never exceed u64::MAX (which is about 584 million years in ms).
#[allow(clippy::cast_possible_truncation)]
fn generate_summary(results: &[PackageResult], total_duration: Duration) -> ExecuteSummaryJson {
    let total = results.len();
    let succeeded = results.iter().filter(|r| r.success).count();
    let failed = total - succeeded;

    ExecuteSummaryJson {
        total,
        succeeded,
        failed,
        total_duration_ms: total_duration.as_millis() as u64,
    }
}

// ============================================================================
// Output Formatting
// ============================================================================

/// Outputs results in JSON format.
///
/// # Note on cast truncation
///
/// The `as_millis()` returns u128, but for any practical execution duration,
/// the value will never exceed u64::MAX (which is about 584 million years in ms).
#[allow(clippy::cast_possible_truncation)]
fn output_json(
    output: &Output,
    cmd_type: &CommandType,
    results: &[PackageResult],
    summary: &ExecuteSummaryJson,
) -> Result<()> {
    let response = ExecuteJsonResponse {
        command: cmd_type.display(),
        results: results
            .iter()
            .map(|r| PackageExecutionResultJson {
                package: r.package_name.clone(),
                success: r.success,
                exit_code: r.exit_code,
                duration_ms: r.duration.as_millis() as u64,
                error: r.error.clone(),
            })
            .collect(),
        summary: summary.clone(),
    };

    output.json(&JsonResponse::success(response))?;
    Ok(())
}

/// Outputs execution summary in human-readable format.
///
/// # Note on precision loss
///
/// The cast from u64 to f64 for duration display may lose precision for very
/// large values, but this is acceptable for human-readable duration display.
#[allow(clippy::cast_precision_loss)]
fn output_human_summary(
    output: &Output,
    results: &[PackageResult],
    summary: &ExecuteSummaryJson,
) -> Result<()> {
    let total = summary.total;
    let succeeded = summary.succeeded;
    let failed = summary.failed;
    let duration_secs = summary.total_duration_ms as f64 / 1000.0;

    output.plain("────────────────────────────────────────")?;
    output.info("Summary")?;
    output.plain(&format!("  Total: {total} | Succeeded: {succeeded} | Failed: {failed}"))?;
    output.plain(&format!("  Duration: {duration_secs:.1}s"))?;

    // Show failed packages
    let failed_results: Vec<_> = results.iter().filter(|r| !r.success).collect();
    if !failed_results.is_empty() {
        output.blank_line()?;
        output.warning("Failed packages:")?;
        for result in failed_results {
            let pkg_name = &result.package_name;
            let exit_code = result.exit_code;
            output.error(&format!("{pkg_name} (exit code: {exit_code})"))?;
        }
    }

    output.blank_line()?;
    Ok(())
}
