//! Execute command implementation for Node.js bindings.
//!
//! # What
//!
//! This module implements the `execute` NAPI function that runs arbitrary
//! commands across workspace packages with filtering, parallelism, and timeout
//! support. It's essential for CI/CD workflows and development automation.
//!
//! # How
//!
//! The implementation follows this flow:
//!
//! 1. **Parameter validation**: Validates root path, command, mutual exclusion,
//!    and timeout ranges
//! 2. **Timeout resolution**: Loads defaults from config with parameter overrides
//! 3. **Output capture**: Uses a `SharedBuffer` wrapper around `Arc<Mutex<Vec<u8>>>`
//!    to capture the CLI's JSON output
//! 4. **CLI execution**: Calls `execute_execute` from `sublime_cli_tools` with
//!    timeout protection via `tokio::time::timeout`
//! 5. **Response parsing**: Parses the JSON output using serde into intermediate types
//! 6. **Type conversion**: Converts CLI response types to NAPI-compatible types
//! 7. **Result wrapping**: Returns an `ExecuteApiResponse` for consistent error handling
//!
//! ## SharedBuffer Pattern
//!
//! The `Output` struct from the CLI crate takes ownership of the writer and doesn't
//! expose an `into_inner()` method. To work around this limitation, we use a
//! `SharedBuffer` that wraps `Arc<Mutex<Vec<u8>>>`:
//!
//! ```text
//! SharedBuffer(Arc<Mutex<Vec<u8>>>)
//!        │
//!        ├── Clone → Output::new(..., SharedBuffer, ...)
//!        │                    │
//!        │                    ▼
//!        │              write() calls
//!        │                    │
//!        │                    ▼
//!        │              Arc<Mutex<Vec<u8>>> (shared)
//!        │
//!        └── After execution: extract bytes from Arc
//! ```
//!
//! ## Timeout Handling
//!
//! The execute command supports two levels of timeout:
//!
//! - **Global timeout** (`timeout_secs`): Maximum time for the entire operation
//! - **Per-package timeout** (`per_package_timeout_secs`): Maximum time per package
//!   (currently handled at CLI level)
//!
//! When a timeout occurs, the function returns an `ETIMEOUT` error code.
//!
//! # Why
//!
//! The execute command enables efficient workspace-wide operations:
//! - Run tests only on affected packages to speed up CI
//! - Build all packages in parallel
//! - Execute arbitrary scripts with timeout protection
//! - Filter execution to specific packages
//!
//! # Examples
//!
//! ## TypeScript Usage
//!
//! ```typescript
//! import { execute } from '@websublime/workspace-tools';
//!
//! // Run tests on all packages
//! const result = await execute({
//!   root: '.',
//!   cmd: 'npm:test',
//!   parallel: true
//! });
//!
//! if (result.success) {
//!   console.log(`Command: ${result.data.command}`);
//!   console.log(`Summary: ${result.data.summary.succeeded}/${result.data.summary.total} succeeded`);
//!
//!   for (const pkg of result.data.results) {
//!     const icon = pkg.success ? '✓' : '✗';
//!     console.log(`${icon} ${pkg.package}: exit ${pkg.exitCode} (${pkg.durationMs}ms)`);
//!   }
//! }
//!
//! // Run tests on affected packages only
//! const affectedResult = await execute({
//!   root: '.',
//!   cmd: 'npm:test',
//!   affected: true,
//!   branch: 'main',
//!   parallel: true,
//!   timeoutSecs: 300
//! });
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
//! // Handle timeouts
//! if (!result.success && result.error.code === 'ETIMEOUT') {
//!   console.log('Execution timed out');
//! }
//! ```
//!
//! ## Error Handling
//!
//! ```typescript
//! const result = await execute({
//!   root: '.',
//!   cmd: 'npm:test',
//!   filterPackage: ['@scope/pkg'],
//!   affected: true // Error: mutual exclusion
//! });
//!
//! if (!result.success) {
//!   switch (result.error.code) {
//!     case 'EVALIDATION':
//!       console.error('Validation error:', result.error.message);
//!       break;
//!     case 'ETIMEOUT':
//!       console.error('Operation timed out');
//!       break;
//!     case 'ENOENT':
//!       console.error('Path not found:', result.error.message);
//!       break;
//!     default:
//!       console.error('Unexpected error:', result.error.message);
//!   }
//! }
//! ```

use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use napi_derive::napi;
use serde::Deserialize;

use crate::error::ErrorInfo;
use crate::types::execute::{
    ExecuteApiResponse, ExecuteData, ExecuteParams, ExecuteSummary, PackageExecutionResult,
};
use crate::validation::validators;

use sublime_cli_tools::cli::commands::ExecuteArgs;
use sublime_cli_tools::commands::execute::execute_execute;
use sublime_cli_tools::output::{Output, OutputFormat};
use sublime_pkg_tools::config::ExecuteConfig;

// ============================================================================
// Constants
// ============================================================================

/// Maximum allowed timeout in seconds (24 hours).
///
/// Prevents unreasonably long timeouts that could cause resource exhaustion.
const MAX_TIMEOUT_SECS: u64 = 86400;

/// Maximum allowed per-package timeout in seconds (1 hour).
///
/// Prevents unreasonably long per-package timeouts.
const MAX_PER_PACKAGE_TIMEOUT_SECS: u64 = 3600;

// ============================================================================
// SharedBuffer - Output Capture Mechanism
// ============================================================================

/// A shared buffer wrapper for capturing CLI output.
///
/// The `Output` struct from `sublime_cli_tools` takes ownership of the writer
/// and doesn't expose an `into_inner()` method. This wrapper uses `Arc<Mutex<Vec<u8>>>`
/// to allow sharing the buffer between the caller and the Output, enabling
/// extraction of the written bytes after command execution.
///
/// # Thread Safety
///
/// The buffer is protected by a `Mutex` to ensure safe concurrent access,
/// although in typical usage the writes happen sequentially.
///
/// # Examples
///
/// ```rust,ignore
/// let buffer = SharedBuffer::new();
/// let output = Output::new(OutputFormat::Json, buffer.clone(), true);
///
/// // ... execute command that writes to output ...
///
/// let bytes = buffer.take_bytes();
/// ```
#[derive(Debug, Clone)]
pub(crate) struct SharedBuffer {
    /// The inner buffer wrapped in `Arc<Mutex>` for shared ownership.
    inner: Arc<Mutex<Vec<u8>>>,
}

impl SharedBuffer {
    /// Creates a new empty shared buffer.
    ///
    /// # Returns
    ///
    /// A new `SharedBuffer` instance ready for use with `Output::new()`.
    pub(crate) fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(Vec::new())) }
    }

    /// Extracts the bytes written to the buffer.
    ///
    /// This method clones the current buffer contents. The original buffer
    /// remains intact for potential further writes.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing all bytes written to the buffer.
    ///
    /// # Panics
    ///
    /// This method will return an empty Vec if the mutex is poisoned,
    /// rather than panicking, to maintain robustness.
    pub(crate) fn take_bytes(&self) -> Vec<u8> {
        match self.inner.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => {
                // If the mutex is poisoned, still try to get the data
                // This provides graceful degradation
                poisoned.into_inner().clone()
            }
        }
    }
}

impl Write for SharedBuffer {
    /// Writes data to the shared buffer.
    ///
    /// # Arguments
    ///
    /// * `buf` - The byte slice to write
    ///
    /// # Returns
    ///
    /// The number of bytes written, or an I/O error if the mutex is poisoned.
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.inner.lock() {
            Ok(mut guard) => guard.write(buf),
            Err(_) => Err(std::io::Error::other("Mutex poisoned")),
        }
    }

    /// Flushes the buffer.
    ///
    /// Since `Vec<u8>` is an in-memory buffer, this is a no-op.
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// Required for `Output::new()` which expects `Write + Send`
// SAFETY: SharedBuffer uses Arc<Mutex<_>> which is Send + Sync
unsafe impl Send for SharedBuffer {}

// ============================================================================
// CLI Response Types (for parsing JSON output)
// ============================================================================

/// CLI JSON response wrapper for execute command.
///
/// This type mirrors the `JsonResponse<T>` structure from the CLI crate,
/// used for deserializing the captured JSON output.
#[derive(Debug, Deserialize)]
pub(crate) struct CliJsonResponse<T> {
    /// Whether the operation succeeded.
    pub(crate) success: bool,
    /// The response data (present when success is true).
    pub(crate) data: Option<T>,
    /// Error message (present when success is false).
    pub(crate) error: Option<String>,
}

/// CLI execute response data structure.
///
/// Mirrors the `ExecuteJsonResponse` structure from the CLI's execute command.
/// Field names use camelCase to match the JSON output format.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliExecuteData {
    /// Command that was executed.
    pub(crate) command: String,
    /// Results for each package.
    pub(crate) results: Vec<CliPackageExecutionResult>,
    /// Execution summary.
    pub(crate) summary: CliExecuteSummary,
}

/// CLI package execution result.
///
/// Mirrors the `PackageExecutionResultJson` structure from the CLI's execute command.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliPackageExecutionResult {
    /// Package name.
    pub(crate) package: String,
    /// Whether execution succeeded.
    pub(crate) success: bool,
    /// Exit code from the command.
    pub(crate) exit_code: i32,
    /// Execution duration in milliseconds.
    pub(crate) duration_ms: u64,
    /// Error message if execution failed.
    pub(crate) error: Option<String>,
}

/// CLI execution summary.
///
/// Mirrors the `ExecuteSummaryJson` structure from the CLI's execute command.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliExecuteSummary {
    /// Total number of packages.
    pub(crate) total: usize,
    /// Number of successful executions.
    pub(crate) succeeded: usize,
    /// Number of failed executions.
    pub(crate) failed: usize,
    /// Total execution duration in milliseconds.
    pub(crate) total_duration_ms: u64,
}

// ============================================================================
// Conversion Functions
// ============================================================================

/// Converts CLI execute data to NAPI-compatible `ExecuteData`.
///
/// This function performs a conversion from the CLI's internal types to the
/// NAPI types exposed to JavaScript.
///
/// # Arguments
///
/// * `cli_data` - The parsed CLI response data
///
/// # Returns
///
/// An `ExecuteData` instance suitable for returning to JavaScript.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
// Justification for cast_possible_truncation: It's practically impossible to have
// more than 4 billion packages in a workspace. The u32 limit is sufficient.
// Justification for cast_precision_loss: Duration values in milliseconds will never
// exceed 2^53 (the safe integer limit for f64), so no precision is lost in practice.
// JavaScript numbers are f64 internally, making this conversion necessary for NAPI.
pub(crate) fn convert_to_napi_execute(cli_data: CliExecuteData) -> ExecuteData {
    let results: Vec<PackageExecutionResult> = cli_data
        .results
        .into_iter()
        .map(|r| PackageExecutionResult {
            package: r.package,
            success: r.success,
            exit_code: r.exit_code,
            // Convert u64 to f64 for JavaScript compatibility
            // f64 can represent integers up to 2^53 without precision loss,
            // which is more than sufficient for millisecond durations
            duration_ms: r.duration_ms as f64,
            error: r.error,
        })
        .collect();

    let summary = ExecuteSummary {
        total: cli_data.summary.total as u32,
        succeeded: cli_data.summary.succeeded as u32,
        failed: cli_data.summary.failed as u32,
        // Convert u64 to f64 for JavaScript compatibility
        total_duration_ms: cli_data.summary.total_duration_ms as f64,
    };

    ExecuteData { command: cli_data.command, results, summary }
}

/// Parses the JSON response from the CLI and converts it to NAPI types.
///
/// # Arguments
///
/// * `json_bytes` - The raw JSON bytes captured from CLI output
///
/// # Returns
///
/// * `Ok(ExecuteData)` - Successfully parsed and converted execute data
/// * `Err(ErrorInfo)` - Parsing failed or CLI returned an error
///
/// # Errors
///
/// Returns an error if:
/// - The JSON is malformed or cannot be parsed
/// - The CLI returned `success: false` with an error message
/// - The CLI returned `success: true` but `data` is missing
pub(crate) fn parse_execute_response(json_bytes: &[u8]) -> Result<ExecuteData, ErrorInfo> {
    // Convert bytes to string first for better error messages
    let json_str = std::str::from_utf8(json_bytes)
        .map_err(|e| ErrorInfo::execution(format!("Invalid UTF-8 in CLI response: {e}")))?;

    // Handle empty response
    if json_str.trim().is_empty() {
        return Err(ErrorInfo::execution("CLI returned empty response"));
    }

    // Parse the JSON response
    let response: CliJsonResponse<CliExecuteData> =
        serde_json::from_str(json_str).map_err(|e| {
            ErrorInfo::execution(format!(
                "Failed to parse CLI JSON response: {e} (length={})",
                json_str.len()
            ))
        })?;

    // Check for CLI-level errors
    if !response.success {
        let error_message = response.error.unwrap_or_else(|| "Unknown CLI error".to_string());
        return Err(ErrorInfo::execution(error_message));
    }

    // Extract and convert data
    let cli_data =
        response.data.ok_or_else(|| ErrorInfo::execution("CLI returned success but no data"))?;

    Ok(convert_to_napi_execute(cli_data))
}

// ============================================================================
// Parameter Validation
// ============================================================================

/// Validates execute command parameters.
///
/// Ensures the root path is valid, command is not empty, mutual exclusion
/// is respected, and timeout values are within allowed ranges.
///
/// # Arguments
///
/// * `params` - The execute parameters to validate
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated root path
/// * `Err(ErrorInfo)` - Validation failed
pub(crate) fn validate_params(params: &ExecuteParams) -> Result<PathBuf, ErrorInfo> {
    // Validate root path exists and is a directory
    validators::root(&params.root)?;

    // Validate command is not empty
    validators::not_empty("cmd", &params.cmd)?;

    // Validate mutual exclusion: filterPackage vs affected
    let has_filter = params.filter_package.as_ref().is_some_and(|v| !v.is_empty());
    let has_affected = params.affected.unwrap_or(false);
    validators::mutual_exclusion(&[("filterPackage", has_filter), ("affected", has_affected)])?;

    // Validate timeout ranges if provided
    // Note: 0 is allowed as it means "use default from config"
    if let Some(timeout) = params.timeout_secs
        && timeout > 0
    {
        validators::timeout("timeoutSecs", u64::from(timeout), 1, MAX_TIMEOUT_SECS)?;
    }

    if let Some(per_pkg_timeout) = params.per_package_timeout_secs
        && per_pkg_timeout > 0
    {
        validators::timeout(
            "perPackageTimeoutSecs",
            u64::from(per_pkg_timeout),
            1,
            MAX_PER_PACKAGE_TIMEOUT_SECS,
        )?;
    }

    Ok(PathBuf::from(&params.root))
}

// ============================================================================
// Timeout Resolution
// ============================================================================

/// Resolves the effective timeout values from parameters and config defaults.
///
/// Priority:
/// 1. Parameter value (if provided and > 0)
/// 2. Config default
/// 3. Fallback default
///
/// # Arguments
///
/// * `params` - The execute parameters with optional timeout overrides
///
/// # Returns
///
/// A tuple of (global_timeout_secs, per_package_timeout_secs).
/// A value of 0 means no timeout.
pub(crate) fn resolve_timeouts(params: &ExecuteParams) -> (u64, u64) {
    // Load default config
    let default_config = ExecuteConfig::default();

    // Resolve global timeout
    let global_timeout = match params.timeout_secs {
        Some(0) => 0, // Explicit 0 means no timeout
        Some(t) => u64::from(t),
        None => default_config.timeout_secs,
    };

    // Resolve per-package timeout
    let per_package_timeout = match params.per_package_timeout_secs {
        Some(0) => 0, // Explicit 0 means no timeout
        Some(t) => u64::from(t),
        None => default_config.per_package_timeout_secs,
    };

    (global_timeout, per_package_timeout)
}

/// Converts `ExecuteParams` to CLI `ExecuteArgs`.
///
/// Maps the NAPI parameters to CLI arguments for command execution.
///
/// # Arguments
///
/// * `params` - The NAPI execute parameters
///
/// # Returns
///
/// An `ExecuteArgs` struct ready for CLI execution.
pub(crate) fn convert_params_to_args(params: &ExecuteParams) -> ExecuteArgs {
    ExecuteArgs {
        cmd: params.cmd.clone(),
        filter_package: params.filter_package.clone(),
        affected: params.affected.unwrap_or(false),
        since: params.since.clone(),
        until: params.until.clone(),
        branch: params.branch.clone(),
        parallel: params.parallel.unwrap_or(false),
        args: params.args.clone().unwrap_or_default(),
    }
}

// ============================================================================
// NAPI Function
// ============================================================================

/// Execute commands across workspace packages.
///
/// Runs the specified command on workspace packages with optional filtering,
/// parallel execution, and timeout protection.
///
/// This function is the main entry point for Node.js applications to execute
/// commands across workspace packages. It handles all the complexity of CLI
/// invocation, timeout management, and response parsing internally.
///
/// @param params - Execute parameters containing:
///   - `root`: Workspace root directory path (required)
///   - `cmd`: Command to execute (required, e.g., `npm:test` or `ls -la`)
///   - `filterPackage`: Optional filter to specific packages
///   - `affected`: Execute only on affected packages
///   - `since`: Since commit/branch/tag for affected detection
///   - `until`: Until commit/branch/tag for affected detection
///   - `branch`: Compare against branch for affected detection
///   - `parallel`: Run commands in parallel
///   - `args`: Additional arguments to pass to command
///   - `timeoutSecs`: Global timeout override (0 = no timeout)
///   - `perPackageTimeoutSecs`: Per-package timeout override (0 = no timeout)
///
/// @returns `Promise<ExecuteApiResponse>` containing:
///   - On success: `{ success: true, data: ExecuteData }`
///   - On failure: `{ success: false, error: ErrorInfo }`
///
/// @example Basic usage
/// ```typescript
/// const result = await execute({
///   root: '/path/to/project',
///   cmd: 'npm:test'
/// });
/// if (result.success) {
///   console.log(`${result.data.summary.succeeded}/${result.data.summary.total} succeeded`);
/// } else {
///   console.error(`Error: ${result.error.code} - ${result.error.message}`);
/// }
/// ```
///
/// @example With timeout and parallel execution
/// ```typescript
/// const result = await execute({
///   root: '/path/to/project',
///   cmd: 'npm:build',
///   parallel: true,
///   timeoutSecs: 600,
///   perPackageTimeoutSecs: 120
/// });
/// ```
///
/// @example Error handling
/// ```typescript
/// const result = await execute({
///   root: '/nonexistent',
///   cmd: 'npm:test'
/// });
/// if (!result.success) {
///   if (result.error.code === 'ENOENT') {
///     console.error('Path not found');
///   } else if (result.error.code === 'ETIMEOUT') {
///     console.error('Operation timed out');
///   } else if (result.error.code === 'EVALIDATION') {
///     console.error('Invalid parameters');
///   }
/// }
/// ```
#[napi]
pub async fn execute(params: ExecuteParams) -> ExecuteApiResponse {
    // 1. Validate parameters (synchronous validation before spawning)
    let root_path = match validate_params(&params) {
        Ok(path) => path,
        Err(error) => return ExecuteApiResponse::failure(error),
    };

    // 2. Resolve timeouts
    let (global_timeout, _per_package_timeout) = resolve_timeouts(&params);

    // 3. Convert params to CLI args
    let args = convert_params_to_args(&params);

    // 4. Execute CLI command in a blocking task with timeout
    // The CLI's execute_execute uses types that are not Send/Sync (git2::Repository),
    // so we must run it on a blocking thread via spawn_blocking.
    let execute_future = tokio::task::spawn_blocking(move || {
        // Create a new tokio runtime for the blocking context
        // This is necessary because execute_execute is async but we're in a blocking context
        let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(rt) => rt,
            Err(e) => {
                return Err(ErrorInfo::execution(format!("Failed to create runtime: {e}")));
            }
        };

        rt.block_on(async {
            // Create shared buffer for output capture
            let buffer = SharedBuffer::new();

            // Create Output with JSON format
            let output = Output::new(OutputFormat::Json, buffer.clone(), true);

            // Execute the CLI command
            if let Err(cli_error) = execute_execute(&args, &output, &root_path).await {
                return Err(ErrorInfo::from(cli_error));
            }

            // Extract and parse JSON
            let json_bytes = buffer.take_bytes();
            parse_execute_response(&json_bytes)
        })
    });

    // 5. Apply global timeout if configured
    let result = if global_timeout > 0 {
        match tokio::time::timeout(Duration::from_secs(global_timeout), execute_future).await {
            Ok(spawn_result) => spawn_result,
            Err(_timeout_elapsed) => {
                return ExecuteApiResponse::failure(ErrorInfo::timeout(format!(
                    "Execution timed out after {global_timeout} seconds"
                )));
            }
        }
    } else {
        execute_future.await
    };

    // 6. Handle spawn_blocking result
    match result {
        Ok(Ok(data)) => ExecuteApiResponse::success(data),
        Ok(Err(error)) => ExecuteApiResponse::failure(error),
        Err(join_error) => ExecuteApiResponse::failure(ErrorInfo::execution(format!(
            "Task execution failed: {join_error}"
        ))),
    }
}
