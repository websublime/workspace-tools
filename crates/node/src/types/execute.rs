//! Execute command type definitions for Node.js bindings.
//!
//! # What
//!
//! This module defines all NAPI-compatible type structures for the execute command,
//! including input parameters and response data types. The execute command runs
//! arbitrary commands across workspace packages with filtering, parallelism,
//! and timeout support.
//!
//! # How
//!
//! Types are defined with the `#[napi(object)]` attribute to be automatically
//! exposed as JavaScript objects. The module provides:
//!
//! - **Input Parameters**: `ExecuteParams`
//! - **Response Data**: `ExecuteData`, `PackageExecutionResult`, `ExecuteSummary`
//! - **API Response**: `ExecuteApiResponse` for consistent success/error handling
//!
//! All types implement `Clone`, `Debug`, and `Serialize` for flexibility in
//! testing and serialization scenarios.
//!
//! # Why
//!
//! The execute command enables running scripts across multiple packages with
//! intelligent filtering (affected packages, specific packages) and execution
//! control (parallel, timeout). It's essential for CI/CD workflows and
//! development tasks. These types provide:
//!
//! - **Type safety**: Strong typing for JavaScript/TypeScript consumers
//! - **Documentation**: Self-documenting API through TypeScript definitions
//! - **Consistency**: Matches the CLI JSON output structure for compatibility
//! - **Validation**: Enables parameter validation before CLI execution
//! - **Timeout control**: Configurable timeouts at global and per-package level
//!
//! # Examples
//!
//! ## TypeScript Usage
//!
//! ```typescript
//! import { execute, ExecuteParams, ExecuteData } from '@websublime/workspace-tools';
//!
//! // Run tests on affected packages with timeout
//! const params: ExecuteParams = {
//!   root: '.',
//!   cmd: 'npm:test',
//!   affected: true,
//!   branch: 'main',
//!   parallel: true,
//!   timeoutSecs: 300,           // 5 minutes total timeout
//!   perPackageTimeoutSecs: 60   // 1 minute per package
//! };
//! const result = await execute(params);
//!
//! if (result.success) {
//!   const data: ExecuteData = result.data;
//!   console.log(`Command: ${data.command}`);
//!   console.log(`Packages: ${data.results.length}`);
//!   console.log(`Summary: ${data.summary.succeeded}/${data.summary.total} succeeded`);
//!
//!   for (const pkg of data.results) {
//!     const icon = pkg.success ? '✓' : '✗';
//!     console.log(`${icon} ${pkg.package}: exit code ${pkg.exitCode}`);
//!   }
//! } else {
//!   console.error(`Error [${result.error.code}]: ${result.error.message}`);
//! }
//!
//! // Run build on specific packages
//! const buildResult = await execute({
//!   root: '.',
//!   cmd: 'npm:build',
//!   filterPackage: ['@scope/core', '@scope/utils'],
//!   parallel: true
//! });
//!
//! // Run lint with per-package timeout
//! const lintResult = await execute({
//!   root: '.',
//!   cmd: 'npm:lint',
//!   perPackageTimeoutSecs: 60
//! });
//!
//! // Run system command across all packages
//! const systemResult = await execute({
//!   root: '.',
//!   cmd: 'ls -la',
//!   args: ['-h']  // Additional arguments
//! });
//! ```
//!
//! ## Rust Usage (Internal)
//!
//! ```rust,ignore
//! use sublime_node_tools::types::execute::{
//!     ExecuteParams, ExecuteData, PackageExecutionResult, ExecuteSummary
//! };
//!
//! // Creating params for validation
//! let params = ExecuteParams::new(".", "npm:test")
//!     .with_affected(true)
//!     .with_parallel(true)
//!     .with_timeout_secs(300);
//!
//! // Constructing response data
//! let result = PackageExecutionResult::new("@scope/pkg", true, 0, 1500);
//! let summary = ExecuteSummary::new(1, 1, 0, 1500);
//! let data = ExecuteData::new("npm:test", vec![result], summary);
//! ```

use napi_derive::napi;
use serde::Serialize;

use crate::error::ErrorInfo;

// ============================================================================
// Input Parameters
// ============================================================================

/// Input parameters for the execute command.
///
/// This structure defines the parameters for running commands across workspace
/// packages. It supports filtering by package names or affected packages,
/// parallel execution, and configurable timeouts.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `cmd`: The command to execute (required)
/// - `filter_package`: Filter by specific package names
/// - `affected`: Execute only on affected packages
/// - `since`: Git reference for affected detection start
/// - `until`: Git reference for affected detection end
/// - `branch`: Base branch for affected comparison
/// - `parallel`: Run commands in parallel
/// - `args`: Additional arguments to pass to the command
/// - `timeout_secs`: Global timeout in seconds
/// - `per_package_timeout_secs`: Per-package timeout in seconds
///
/// # Mutual Exclusion
///
/// `filter_package` and `affected` are mutually exclusive. Only one can be
/// specified at a time. Validation should ensure this constraint is enforced.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ExecuteParams {
///   root: string;
///   cmd: string;
///   filterPackage?: string[];
///   affected?: boolean;
///   since?: string;
///   until?: string;
///   branch?: string;
///   parallel?: boolean;
///   args?: string[];
///   timeoutSecs?: number;
///   perPackageTimeoutSecs?: number;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Run tests on affected packages
/// const params: ExecuteParams = {
///   root: '.',
///   cmd: 'npm:test',
///   affected: true,
///   branch: 'main',
///   parallel: true
/// };
///
/// // Run build on specific packages with timeout
/// const buildParams: ExecuteParams = {
///   root: '/path/to/workspace',
///   cmd: 'npm:build',
///   filterPackage: ['@scope/core', '@scope/utils'],
///   timeoutSecs: 600,
///   perPackageTimeoutSecs: 120
/// };
///
/// // Run system command with extra arguments
/// const systemParams: ExecuteParams = {
///   root: '.',
///   cmd: 'echo',
///   args: ['Hello', 'World']
/// };
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ExecuteParams {
    /// Workspace root directory path.
    ///
    /// This is the absolute or relative path to the root of the workspace.
    /// For monorepos, this should point to the root where the package manager
    /// configuration is located.
    pub root: String,

    /// Command to execute.
    ///
    /// Supports two formats:
    /// - `npm:<script>`: Runs an npm script (e.g., `npm:lint`, `npm:build`)
    /// - Plain command: Runs a system command (e.g., `ls -la`, `node index.js`)
    ///
    /// For npm scripts, the appropriate package manager (npm, yarn, pnpm, bun)
    /// is automatically detected and used.
    pub cmd: String,

    /// Filter packages to run command on.
    ///
    /// When provided, only executes the command in the specified packages.
    /// Package names should match exactly (e.g., `@scope/package`).
    ///
    /// Mutually exclusive with `affected`.
    #[napi(ts_type = "string[] | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_package: Option<Vec<String>>,

    /// Execute only on packages affected by changes.
    ///
    /// When `true`, automatically detects packages with changes and runs
    /// commands only on them. By default, analyzes working directory changes
    /// (staged + unstaged).
    ///
    /// Use `since`/`until` for commit range analysis, or `branch` for
    /// branch comparison.
    ///
    /// Mutually exclusive with `filter_package`.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected: Option<bool>,

    /// Since commit/branch/tag for affected detection.
    ///
    /// Used with `affected: true` to analyze changes since this Git reference.
    /// If not specified with `affected`, analyzes working directory changes.
    ///
    /// Example: `"HEAD~5"`, `"v1.0.0"`, `"main"`
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,

    /// Until commit/branch/tag for affected detection.
    ///
    /// Used with `affected: true` to analyze changes until this Git reference.
    /// Defaults to `HEAD` when `since` is specified.
    ///
    /// Example: `"HEAD"`, `"v2.0.0"`
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,

    /// Compare against branch for affected detection.
    ///
    /// Used with `affected: true` to compare current branch against the
    /// target branch. Detects packages changed between current branch and
    /// the specified branch.
    ///
    /// Example: `"main"`, `"develop"`
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// Run commands in parallel across packages.
    ///
    /// When `true`, all package commands run concurrently. By default,
    /// commands run sequentially.
    ///
    /// Note: Parallel execution may interleave output from different packages.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel: Option<bool>,

    /// Additional arguments passed to the command.
    ///
    /// These arguments are appended to the command after the base command
    /// and any npm script arguments.
    ///
    /// Example: `["--coverage", "--verbose"]`
    #[napi(ts_type = "string[] | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,

    /// Timeout for the entire execute operation in seconds.
    ///
    /// This is the global timeout for executing commands across all packages.
    /// A value of `0` is invalid; use `None` to indicate no timeout.
    ///
    /// If not provided, the default from the configuration is used (typically
    /// 300 seconds / 5 minutes).
    ///
    /// Note: This parameter overrides the configuration value when provided.
    #[napi(ts_type = "number | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u32>,

    /// Timeout per package execution in seconds.
    ///
    /// This is the timeout for executing the command in a single package.
    /// A value of `0` is invalid; use `None` to indicate no per-package timeout.
    ///
    /// If not provided, the default from the configuration is used (typically
    /// 60 seconds / 1 minute).
    ///
    /// Note: This parameter overrides the configuration value when provided.
    #[napi(ts_type = "number | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_package_timeout_secs: Option<u32>,
}

#[allow(dead_code)]
impl ExecuteParams {
    /// Creates a new `ExecuteParams` with the required root and command.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    /// * `cmd` - The command to execute
    ///
    /// # Returns
    ///
    /// A new `ExecuteParams` instance with default optional values.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::execute::ExecuteParams;
    ///
    /// let params = ExecuteParams::new(".", "npm:test");
    /// assert_eq!(params.root, ".");
    /// assert_eq!(params.cmd, "npm:test");
    /// ```
    #[must_use]
    pub fn new(root: impl Into<String>, cmd: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            cmd: cmd.into(),
            filter_package: None,
            affected: None,
            since: None,
            until: None,
            branch: None,
            parallel: None,
            args: None,
            timeout_secs: None,
            per_package_timeout_secs: None,
        }
    }

    /// Sets the filter packages.
    ///
    /// # Arguments
    ///
    /// * `packages` - Package names to filter
    ///
    /// # Returns
    ///
    /// The modified `ExecuteParams` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = ExecuteParams::new(".", "npm:test")
    ///     .with_filter_package(vec!["@scope/core".to_string()]);
    /// ```
    #[must_use]
    pub fn with_filter_package(mut self, packages: Vec<String>) -> Self {
        self.filter_package = Some(packages);
        self
    }

    /// Sets the affected flag.
    ///
    /// # Arguments
    ///
    /// * `affected` - Whether to run on affected packages only
    ///
    /// # Returns
    ///
    /// The modified `ExecuteParams` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = ExecuteParams::new(".", "npm:test")
    ///     .with_affected(true);
    /// ```
    #[must_use]
    pub fn with_affected(mut self, affected: bool) -> Self {
        self.affected = Some(affected);
        self
    }

    /// Sets the since reference for affected detection.
    ///
    /// # Arguments
    ///
    /// * `since` - Git reference to start from
    ///
    /// # Returns
    ///
    /// The modified `ExecuteParams` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = ExecuteParams::new(".", "npm:test")
    ///     .with_affected(true)
    ///     .with_since("HEAD~5");
    /// ```
    #[must_use]
    pub fn with_since(mut self, since: impl Into<String>) -> Self {
        self.since = Some(since.into());
        self
    }

    /// Sets the until reference for affected detection.
    ///
    /// # Arguments
    ///
    /// * `until` - Git reference to end at
    ///
    /// # Returns
    ///
    /// The modified `ExecuteParams` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = ExecuteParams::new(".", "npm:test")
    ///     .with_affected(true)
    ///     .with_since("v1.0.0")
    ///     .with_until("v2.0.0");
    /// ```
    #[must_use]
    pub fn with_until(mut self, until: impl Into<String>) -> Self {
        self.until = Some(until.into());
        self
    }

    /// Sets the branch for affected comparison.
    ///
    /// # Arguments
    ///
    /// * `branch` - Base branch to compare against
    ///
    /// # Returns
    ///
    /// The modified `ExecuteParams` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = ExecuteParams::new(".", "npm:test")
    ///     .with_affected(true)
    ///     .with_branch("main");
    /// ```
    #[must_use]
    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    /// Sets the parallel execution flag.
    ///
    /// # Arguments
    ///
    /// * `parallel` - Whether to run commands in parallel
    ///
    /// # Returns
    ///
    /// The modified `ExecuteParams` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = ExecuteParams::new(".", "npm:test")
    ///     .with_parallel(true);
    /// ```
    #[must_use]
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = Some(parallel);
        self
    }

    /// Sets additional arguments for the command.
    ///
    /// # Arguments
    ///
    /// * `args` - Additional arguments to pass
    ///
    /// # Returns
    ///
    /// The modified `ExecuteParams` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = ExecuteParams::new(".", "npm:test")
    ///     .with_args(vec!["--coverage".to_string()]);
    /// ```
    #[must_use]
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = Some(args);
        self
    }

    /// Sets the global timeout in seconds.
    ///
    /// # Arguments
    ///
    /// * `timeout_secs` - Timeout value in seconds
    ///
    /// # Returns
    ///
    /// The modified `ExecuteParams` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = ExecuteParams::new(".", "npm:test")
    ///     .with_timeout_secs(300); // 5 minutes
    /// ```
    #[must_use]
    pub fn with_timeout_secs(mut self, timeout_secs: u32) -> Self {
        self.timeout_secs = Some(timeout_secs);
        self
    }

    /// Sets the per-package timeout in seconds.
    ///
    /// # Arguments
    ///
    /// * `per_package_timeout_secs` - Per-package timeout value in seconds
    ///
    /// # Returns
    ///
    /// The modified `ExecuteParams` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = ExecuteParams::new(".", "npm:test")
    ///     .with_per_package_timeout_secs(60); // 1 minute per package
    /// ```
    #[must_use]
    pub fn with_per_package_timeout_secs(mut self, per_package_timeout_secs: u32) -> Self {
        self.per_package_timeout_secs = Some(per_package_timeout_secs);
        self
    }

    /// Checks if filter package is set.
    ///
    /// # Returns
    ///
    /// `true` if `filter_package` is `Some` with a non-empty vector.
    #[must_use]
    pub fn has_filter_package(&self) -> bool {
        self.filter_package.as_ref().is_some_and(|p| !p.is_empty())
    }

    /// Checks if affected mode is enabled.
    ///
    /// # Returns
    ///
    /// `true` if `affected` is `Some(true)`.
    #[must_use]
    pub fn is_affected(&self) -> bool {
        self.affected == Some(true)
    }

    /// Checks if parallel mode is enabled.
    ///
    /// # Returns
    ///
    /// `true` if `parallel` is `Some(true)`.
    #[must_use]
    pub fn is_parallel(&self) -> bool {
        self.parallel == Some(true)
    }
}

// ============================================================================
// Response Data Types
// ============================================================================

/// Result for a single package execution.
///
/// Contains the outcome of executing a command in one package, including
/// success status, exit code, duration, and any error message.
///
/// # Fields
///
/// - `package`: The package name
/// - `success`: Whether execution succeeded
/// - `exit_code`: Exit code from the command
/// - `duration_ms`: Execution duration in milliseconds
/// - `error`: Error message if execution failed
///
/// # TypeScript Definition
///
/// ```typescript
/// interface PackageExecutionResult {
///   package: string;
///   success: boolean;
///   exitCode: number;
///   durationMs: number;
///   error?: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Successful execution
/// const success: PackageExecutionResult = {
///   package: '@scope/core',
///   success: true,
///   exitCode: 0,
///   durationMs: 1500
/// };
///
/// // Failed execution
/// const failed: PackageExecutionResult = {
///   package: '@scope/utils',
///   success: false,
///   exitCode: 1,
///   durationMs: 500,
///   error: 'Test suite failed with 3 failing tests'
/// };
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct PackageExecutionResult {
    /// Package name.
    ///
    /// The name of the package where the command was executed.
    /// This matches the name in the package's `package.json`.
    pub package: String,

    /// Whether execution succeeded.
    ///
    /// - `true`: Command exited with code 0
    /// - `false`: Command exited with non-zero code or errored
    pub success: bool,

    /// Exit code from the command.
    ///
    /// The process exit code returned by the command.
    /// Typically `0` for success, non-zero for failure.
    /// May be `-1` if the process was terminated or couldn't be started.
    pub exit_code: i32,

    /// Execution duration in milliseconds.
    ///
    /// The time taken to execute the command in this package,
    /// measured from start to completion.
    ///
    /// Note: Uses `f64` for JavaScript compatibility. JavaScript numbers are
    /// internally f64, which can represent integers up to 2^53 without precision
    /// loss, more than sufficient for duration values.
    pub duration_ms: f64,

    /// Error message if execution failed.
    ///
    /// Contains error details when `success` is `false`.
    /// May include stderr output or error descriptions.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[allow(dead_code)]
impl PackageExecutionResult {
    /// Creates a new successful `PackageExecutionResult`.
    ///
    /// # Arguments
    ///
    /// * `package` - The package name
    /// * `success` - Whether execution succeeded
    /// * `exit_code` - The command exit code
    /// * `duration_ms` - Execution duration in milliseconds
    ///
    /// # Returns
    ///
    /// A new `PackageExecutionResult` instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::execute::PackageExecutionResult;
    ///
    /// let result = PackageExecutionResult::new("@scope/pkg", true, 0, 1500);
    /// assert!(result.success);
    /// ```
    #[must_use]
    pub fn new(
        package: impl Into<String>,
        success: bool,
        exit_code: i32,
        duration_ms: f64,
    ) -> Self {
        Self { package: package.into(), success, exit_code, duration_ms, error: None }
    }

    /// Creates a new successful execution result.
    ///
    /// # Arguments
    ///
    /// * `package` - The package name
    /// * `duration_ms` - Execution duration in milliseconds
    ///
    /// # Returns
    ///
    /// A new `PackageExecutionResult` with success=true and exit_code=0.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let result = PackageExecutionResult::success("@scope/pkg", 1500);
    /// assert!(result.success);
    /// assert_eq!(result.exit_code, 0);
    /// ```
    #[must_use]
    pub fn success(package: impl Into<String>, duration_ms: f64) -> Self {
        Self::new(package, true, 0, duration_ms)
    }

    /// Creates a new failed execution result.
    ///
    /// # Arguments
    ///
    /// * `package` - The package name
    /// * `exit_code` - The command exit code
    /// * `duration_ms` - Execution duration in milliseconds
    /// * `error` - Error message
    ///
    /// # Returns
    ///
    /// A new `PackageExecutionResult` with success=false.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let result = PackageExecutionResult::failure(
    ///     "@scope/pkg",
    ///     1,
    ///     500,
    ///     "Test failed"
    /// );
    /// assert!(!result.success);
    /// assert_eq!(result.exit_code, 1);
    /// ```
    #[must_use]
    pub fn failure(
        package: impl Into<String>,
        exit_code: i32,
        duration_ms: f64,
        error: impl Into<String>,
    ) -> Self {
        Self {
            package: package.into(),
            success: false,
            exit_code,
            duration_ms,
            error: Some(error.into()),
        }
    }

    /// Sets the error message.
    ///
    /// # Arguments
    ///
    /// * `error` - The error message
    ///
    /// # Returns
    ///
    /// The modified `PackageExecutionResult` instance for method chaining.
    #[must_use]
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }
}

/// Execution summary for all packages.
///
/// Provides aggregate statistics about command execution across all
/// targeted packages, including counts and total duration.
///
/// # Fields
///
/// - `total`: Total number of packages
/// - `succeeded`: Number of successful executions
/// - `failed`: Number of failed executions
/// - `total_duration_ms`: Total execution duration in milliseconds
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ExecuteSummary {
///   total: number;
///   succeeded: number;
///   failed: number;
///   totalDurationMs: number;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const summary: ExecuteSummary = {
///   total: 5,
///   succeeded: 4,
///   failed: 1,
///   totalDurationMs: 15000
/// };
///
/// // Check if all succeeded
/// if (summary.succeeded === summary.total) {
///   console.log('All packages passed!');
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ExecuteSummary {
    /// Total number of packages processed.
    ///
    /// This is the count of all packages that were targeted for execution,
    /// regardless of success or failure.
    pub total: u32,

    /// Number of successful executions.
    ///
    /// Count of packages where the command exited with code 0.
    pub succeeded: u32,

    /// Number of failed executions.
    ///
    /// Count of packages where the command exited with non-zero code
    /// or encountered an error.
    pub failed: u32,

    /// Total execution duration in milliseconds.
    ///
    /// For sequential execution, this is the sum of all package durations.
    /// For parallel execution, this is the wall-clock time from start to finish.
    ///
    /// Note: Uses `f64` for JavaScript compatibility. JavaScript numbers are
    /// internally f64, which can represent integers up to 2^53 without precision
    /// loss, more than sufficient for duration values.
    pub total_duration_ms: f64,
}

#[allow(dead_code)]
impl ExecuteSummary {
    /// Creates a new `ExecuteSummary`.
    ///
    /// # Arguments
    ///
    /// * `total` - Total number of packages
    /// * `succeeded` - Number of successful executions
    /// * `failed` - Number of failed executions
    /// * `total_duration_ms` - Total duration in milliseconds
    ///
    /// # Returns
    ///
    /// A new `ExecuteSummary` instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::execute::ExecuteSummary;
    ///
    /// let summary = ExecuteSummary::new(5, 4, 1, 15000);
    /// assert_eq!(summary.total, 5);
    /// assert_eq!(summary.succeeded, 4);
    /// assert_eq!(summary.failed, 1);
    /// ```
    #[must_use]
    pub fn new(total: u32, succeeded: u32, failed: u32, total_duration_ms: f64) -> Self {
        Self { total, succeeded, failed, total_duration_ms }
    }

    /// Creates an empty `ExecuteSummary`.
    ///
    /// # Returns
    ///
    /// A new `ExecuteSummary` with all values set to zero.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let summary = ExecuteSummary::empty();
    /// assert_eq!(summary.total, 0);
    /// ```
    #[must_use]
    pub fn empty() -> Self {
        Self::new(0, 0, 0, 0.0)
    }

    /// Creates a summary from a collection of package results.
    ///
    /// # Arguments
    ///
    /// * `results` - Slice of package execution results
    ///
    /// # Returns
    ///
    /// A new `ExecuteSummary` computed from the results.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let results = vec![
    ///     PackageExecutionResult::success("pkg1", 1000),
    ///     PackageExecutionResult::failure("pkg2", 1, 500, "error"),
    /// ];
    /// let summary = ExecuteSummary::from_results(&results);
    /// assert_eq!(summary.total, 2);
    /// assert_eq!(summary.succeeded, 1);
    /// assert_eq!(summary.failed, 1);
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_results(results: &[PackageExecutionResult]) -> Self {
        // Safe to truncate: In practice, workspaces don't have billions of packages.
        // Even the largest monorepos have at most thousands of packages.
        let total = results.len() as u32;
        let succeeded = results.iter().filter(|r| r.success).count() as u32;
        let failed = total - succeeded;
        let total_duration_ms: f64 = results.iter().map(|r| r.duration_ms).sum();

        Self::new(total, succeeded, failed, total_duration_ms)
    }

    /// Checks if all executions succeeded.
    ///
    /// # Returns
    ///
    /// `true` if `failed` is 0 and `total` > 0.
    #[must_use]
    pub fn all_succeeded(&self) -> bool {
        self.total > 0 && self.failed == 0
    }

    /// Checks if any execution failed.
    ///
    /// # Returns
    ///
    /// `true` if `failed` > 0.
    #[must_use]
    pub fn has_failures(&self) -> bool {
        self.failed > 0
    }
}

/// Execute command response data.
///
/// This is the main response structure returned by the execute command,
/// containing the executed command, results for each package, and
/// aggregate summary statistics.
///
/// # Fields
///
/// - `command`: The command that was executed
/// - `results`: Results for each package
/// - `summary`: Aggregate execution summary
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ExecuteData {
///   command: string;
///   results: PackageExecutionResult[];
///   summary: ExecuteSummary;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const data: ExecuteData = {
///   command: 'npm:test',
///   results: [
///     { package: '@scope/core', success: true, exitCode: 0, durationMs: 1500 },
///     { package: '@scope/utils', success: true, exitCode: 0, durationMs: 800 }
///   ],
///   summary: {
///     total: 2,
///     succeeded: 2,
///     failed: 0,
///     totalDurationMs: 2300
///   }
/// };
///
/// // Process results
/// for (const result of data.results) {
///   console.log(`${result.package}: ${result.success ? 'PASS' : 'FAIL'}`);
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ExecuteData {
    /// Command that was executed.
    ///
    /// The original command string that was passed to execute.
    /// For npm scripts, this is the `npm:<script>` format.
    pub command: String,

    /// Results for each package.
    ///
    /// Contains execution results for each package that was targeted.
    /// The order may vary for parallel execution.
    pub results: Vec<PackageExecutionResult>,

    /// Execution summary.
    ///
    /// Aggregate statistics about the execution across all packages.
    pub summary: ExecuteSummary,
}

#[allow(dead_code)]
impl ExecuteData {
    /// Creates a new `ExecuteData`.
    ///
    /// # Arguments
    ///
    /// * `command` - The command that was executed
    /// * `results` - Results for each package
    /// * `summary` - Execution summary
    ///
    /// # Returns
    ///
    /// A new `ExecuteData` instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::execute::{ExecuteData, ExecuteSummary};
    ///
    /// let results = vec![];
    /// let summary = ExecuteSummary::empty();
    /// let data = ExecuteData::new("npm:test", results, summary);
    /// assert_eq!(data.command, "npm:test");
    /// ```
    #[must_use]
    pub fn new(
        command: impl Into<String>,
        results: Vec<PackageExecutionResult>,
        summary: ExecuteSummary,
    ) -> Self {
        Self { command: command.into(), results, summary }
    }

    /// Creates a new `ExecuteData` with automatically computed summary.
    ///
    /// # Arguments
    ///
    /// * `command` - The command that was executed
    /// * `results` - Results for each package
    ///
    /// # Returns
    ///
    /// A new `ExecuteData` with summary computed from results.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let results = vec![
    ///     PackageExecutionResult::success("pkg1", 1000),
    /// ];
    /// let data = ExecuteData::from_results("npm:test", results);
    /// assert_eq!(data.summary.total, 1);
    /// ```
    #[must_use]
    pub fn from_results(command: impl Into<String>, results: Vec<PackageExecutionResult>) -> Self {
        let summary = ExecuteSummary::from_results(&results);
        Self::new(command, results, summary)
    }

    /// Creates an empty `ExecuteData`.
    ///
    /// # Arguments
    ///
    /// * `command` - The command that was executed
    ///
    /// # Returns
    ///
    /// A new `ExecuteData` with no results.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let data = ExecuteData::empty("npm:test");
    /// assert!(data.results.is_empty());
    /// assert_eq!(data.summary.total, 0);
    /// ```
    #[must_use]
    pub fn empty(command: impl Into<String>) -> Self {
        Self::new(command, vec![], ExecuteSummary::empty())
    }

    /// Returns the number of packages processed.
    ///
    /// # Returns
    ///
    /// The count of results.
    #[must_use]
    pub fn package_count(&self) -> usize {
        self.results.len()
    }

    /// Checks if all executions succeeded.
    ///
    /// # Returns
    ///
    /// `true` if all packages succeeded.
    #[must_use]
    pub fn all_succeeded(&self) -> bool {
        self.summary.all_succeeded()
    }

    /// Checks if any execution failed.
    ///
    /// # Returns
    ///
    /// `true` if any package failed.
    #[must_use]
    pub fn has_failures(&self) -> bool {
        self.summary.has_failures()
    }
}

// ============================================================================
// API Response Type
// ============================================================================

/// API response for the execute command.
///
/// This is a concrete (non-generic) response type specifically for the execute
/// command. It uses `#[napi(object)]` to enable automatic conversion to
/// JavaScript objects.
///
/// napi-rs cannot use generic types with `#[napi(object)]`, so each command
/// that returns structured data needs its own concrete response type.
///
/// # Fields
///
/// - `success`: Whether the operation succeeded
/// - `data`: The execute data (present when success is true)
/// - `error`: Error information (present when success is false)
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ExecuteApiResponse {
///   success: boolean;
///   data?: ExecuteData;
///   error?: ErrorInfo;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const result = await execute({ root: '.', cmd: 'npm:test' });
///
/// if (result.success) {
///   // result.data is ExecuteData
///   console.log(`${result.data.summary.succeeded}/${result.data.summary.total} succeeded`);
///   for (const pkg of result.data.results) {
///     const icon = pkg.success ? '✓' : '✗';
///     console.log(`${icon} ${pkg.package}`);
///   }
/// } else {
///   // result.error is ErrorInfo
///   console.error(`[${result.error.code}] ${result.error.message}`);
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ExecuteApiResponse {
    /// Whether the operation succeeded.
    ///
    /// - `true`: Command execution completed, `data` field will be present
    /// - `false`: Operation failed, `error` field will be present
    ///
    /// Note: This indicates whether the execute operation itself succeeded,
    /// not whether all package commands succeeded. Check `data.summary.failed`
    /// to determine if any package commands failed.
    pub success: bool,

    /// The execute data (only present when `success` is `true`).
    ///
    /// Contains execution results for each package and aggregate summary.
    #[napi(ts_type = "ExecuteData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ExecuteData>,

    /// Error information (only present when `success` is `false`).
    ///
    /// Contains structured error information with a Node.js-style error code,
    /// message, optional context, and error kind.
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl ExecuteApiResponse {
    /// Creates a successful execute response with data.
    ///
    /// # Arguments
    ///
    /// * `data` - The execute data to include
    ///
    /// # Returns
    ///
    /// A new `ExecuteApiResponse` with `success = true` and the provided data.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::execute::{ExecuteApiResponse, ExecuteData};
    ///
    /// let data = ExecuteData::empty("npm:test");
    /// let response = ExecuteApiResponse::success(data);
    /// assert!(response.success);
    /// assert!(response.data.is_some());
    /// ```
    #[must_use]
    pub fn success(data: ExecuteData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failed execute response with error information.
    ///
    /// # Arguments
    ///
    /// * `error` - The error information to include
    ///
    /// # Returns
    ///
    /// A new `ExecuteApiResponse` with `success = false` and the provided error.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::types::execute::ExecuteApiResponse;
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::validation("Invalid root path", Some("root"));
    /// let response = ExecuteApiResponse::failure(error);
    /// assert!(!response.success);
    /// assert!(response.error.is_some());
    /// ```
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns whether this response represents a success.
    ///
    /// # Returns
    ///
    /// `true` if the operation succeeded, `false` otherwise.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns whether this response represents a failure.
    ///
    /// # Returns
    ///
    /// `true` if the operation failed, `false` otherwise.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}
