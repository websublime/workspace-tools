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
//! The function:
//! 1. Validates the input parameters (root path, command, filters, timeouts)
//! 2. Validates mutual exclusion between `filterPackage` and `affected`
//! 3. Calls the appropriate execution logic with timeout handling
//! 4. Returns a `JsonResponse<ExecuteData>` with per-package results
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
//! ```typescript
//! import { execute } from '@websublime/workspace-tools';
//!
//! // Run tests on all packages
//! const result = await execute({
//!   root: '.',
//!   cmd: 'npm test',
//!   parallel: true
//! });
//!
//! if (result.success) {
//!   console.log(`Command: ${result.data.command}`);
//!   console.log(`Summary: ${result.data.summary.successful}/${result.data.summary.total} succeeded`);
//!
//!   for (const pkg of result.data.results) {
//!     const icon = pkg.success ? '✓' : '✗';
//!     console.log(`${icon} ${pkg.packageName}: exit ${pkg.exitCode} (${pkg.durationMs}ms)`);
//!   }
//! }
//!
//! // Run tests on affected packages only
//! const affectedResult = await execute({
//!   root: '.',
//!   cmd: 'npm test',
//!   affected: true,
//!   branch: 'main',
//!   parallel: true,
//!   timeoutSecs: 300
//! });
//!
//! // Run build on specific packages
//! const buildResult = await execute({
//!   root: '.',
//!   cmd: 'npm run build',
//!   filterPackage: ['@scope/core', '@scope/utils'],
//!   parallel: true
//! });
//!
//! // Run lint with per-package timeout
//! const lintResult = await execute({
//!   root: '.',
//!   cmd: 'npm run lint',
//!   perPackageTimeoutSecs: 60
//! });
//!
//! // Handle timeouts
//! if (result.success) {
//!   const timedOut = result.data.results.filter(r => r.timedOut);
//!   if (timedOut.length > 0) {
//!     console.log(`${timedOut.length} packages timed out:`);
//!     for (const pkg of timedOut) {
//!       console.log(`  ${pkg.packageName}`);
//!     }
//!   }
//! }
//! ```

// TODO: will be implemented on story 6.3 - Execute Command
//
// Implementation outline:
//
// #[napi]
// pub async fn execute(params: ExecuteParams) -> JsonResponse<ExecuteData> {
//     // 1. Validate parameters
//     if let Err(e) = validate_root(&params.root) {
//         return JsonResponse::from_error_info(e);
//     }
//
//     // 2. Validate command is not empty
//     if let Err(e) = validate_message_not_empty(&params.cmd, "cmd") {
//         return JsonResponse::from_error_info(e);
//     }
//
//     // 3. Validate mutual exclusion between filterPackage and affected
//     let has_filter = params.filter_package.as_ref().map_or(false, |v| !v.is_empty());
//     let has_affected = params.affected.unwrap_or(false);
//     if let Err(e) = validate_mutual_exclusion(&[
//         ("filterPackage", has_filter),
//         ("affected", has_affected),
//     ]) {
//         return JsonResponse::from_error_info(e);
//     }
//
//     // 4. Validate timeouts if provided
//     if let Some(timeout) = params.timeout_secs {
//         if let Err(e) = validate_timeout(timeout, "timeoutSecs") {
//             return JsonResponse::from_error_info(e);
//         }
//     }
//     if let Some(per_pkg_timeout) = params.per_package_timeout_secs {
//         if let Err(e) = validate_timeout(per_pkg_timeout, "perPackageTimeoutSecs") {
//             return JsonResponse::from_error_info(e);
//         }
//     }
//
//     // 5. Create ExecuteArgs from params
//     let args = ExecuteArgs {
//         cmd: params.cmd.clone(),
//         filter_package: params.filter_package.clone(),
//         affected: params.affected.unwrap_or(false),
//         branch: params.branch.clone(),
//         parallel: params.parallel.unwrap_or(false),
//         timeout_secs: params.timeout_secs,
//         per_package_timeout_secs: params.per_package_timeout_secs,
//         ..Default::default()
//     };
//
//     // 6. Create Output with JSON format for capturing
//     let mut buffer = Vec::new();
//     let output = Output::new(OutputFormat::Json, &mut buffer, false);
//
//     // 7. Determine workspace root path
//     let root_path = Path::new(&params.root);
//     let config_path = None; // Will use default config discovery
//
//     // 8. Execute command across packages
//     //    - If affected: detect affected packages using git diff
//     //    - If filterPackage: filter to specified packages
//     //    - If parallel: run commands concurrently
//     //    - Apply timeout handling with tokio::time::timeout
//
//     // 9. Collect results per package
//     //    For each package execution:
//     //    - packageName: name of the package
//     //    - packagePath: path to the package
//     //    - success: whether exit code is 0
//     //    - exitCode: actual exit code
//     //    - stdout: captured stdout
//     //    - stderr: captured stderr
//     //    - durationMs: execution duration
//     //    - timedOut: whether execution timed out
//
//     // 10. Build summary
//     //     - total: total packages processed
//     //     - successful: packages with exit code 0
//     //     - failed: packages with non-zero exit code
//     //     - skipped: packages that were skipped
//     //     - timedOut: packages that timed out
//     //     - totalDurationMs: total execution time
//
//     // 11. Return JsonResponse::success(data) or JsonResponse::error(msg)
// }
//
// Note: The execute command is the most complex command in terms of runtime
// behavior. It uses tokio for async execution and timeout handling.
// The sublime_standard_tools::command module provides the CommandExecutor
// for running commands, and sublime_pkg_tools::changes module helps with
// affected package detection.
//
// Key considerations:
// - Timeout handling uses tokio::time::timeout
// - Parallel execution uses tokio::task::JoinSet
// - stdout/stderr are captured using the Output abstraction
// - Exit codes are preserved for each package execution
