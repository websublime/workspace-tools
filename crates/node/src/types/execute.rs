//! Execute command type definitions.
//!
//! # What
//!
//! This module contains type definitions for the execute command, including
//! parameter structures and response data types. The execute command runs
//! arbitrary commands across workspace packages with filtering, parallelism,
//! and timeout support.
//!
//! # How
//!
//! Types are defined with `#[napi(object)]` attribute to be exposed as
//! JavaScript objects. The module provides:
//!
//! - `ExecuteParams`: Input parameters for the execute command
//! - `ExecuteData`: Response data containing execution results
//!
//! # Why
//!
//! The execute command enables running scripts across multiple packages with
//! intelligent filtering (affected packages, specific packages) and execution
//! control (parallel, timeout). It's essential for CI/CD workflows and
//! development tasks.
//!
//! # Examples
//!
//! ```typescript
//! import { execute, ExecuteParams, ExecuteData } from '@websublime/workspace-tools';
//!
//! // Run tests on affected packages
//! const params: ExecuteParams = {
//!   root: '.',
//!   cmd: 'npm test',
//!   affected: true,
//!   branch: 'main',
//!   parallel: true,
//!   timeoutSecs: 300
//! };
//! const result = await execute(params);
//!
//! if (result.success) {
//!   const data: ExecuteData = result.data;
//!   console.log(`Command: ${data.command}`);
//!   console.log(`Packages: ${data.results.length}`);
//!   console.log(`Summary: ${data.summary.successful}/${data.summary.total} succeeded`);
//!
//!   for (const pkg of data.results) {
//!     const icon = pkg.success ? '✓' : '✗';
//!     console.log(`${icon} ${pkg.packageName}: ${pkg.exitCode}`);
//!   }
//! }
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
//! ```

// TODO: will be implemented on story 6.2 - Execute Types
// This module will contain:
//
// NAPI-specific types:
// - ExecuteParams: {
//     root: string,                  // Workspace root path
//     cmd: string,                   // Command to execute
//     filterPackage?: string[],      // Filter by package names (mutually exclusive with affected)
//     affected?: boolean,            // Run on affected packages only (mutually exclusive with filterPackage)
//     branch?: string,               // Base branch for affected detection (default: main)
//     parallel?: boolean,            // Run commands in parallel
//     timeoutSecs?: number,          // Global timeout for entire operation
//     perPackageTimeoutSecs?: number // Timeout per package execution
//   }
// - ExecuteData: {
//     command: string,               // The command that was executed
//     results: PackageExecutionResult[],  // Results per package
//     summary: ExecutionSummary      // Overall execution summary
//   }
//
// Shared types:
// - PackageExecutionResult: {
//     packageName: string,           // Name of the package
//     packagePath: string,           // Path to the package
//     success: boolean,              // Whether execution succeeded
//     exitCode: number,              // Exit code of the command
//     stdout: string,                // Standard output
//     stderr: string,                // Standard error
//     durationMs: number,            // Execution duration in milliseconds
//     timedOut: boolean              // Whether execution timed out
//   }
// - ExecutionSummary: {
//     total: number,                 // Total packages processed
//     successful: number,            // Packages with successful execution
//     failed: number,                // Packages with failed execution
//     skipped: number,               // Packages that were skipped
//     timedOut: number,              // Packages that timed out
//     totalDurationMs: number        // Total duration in milliseconds
//   }
//
// Note: filterPackage and affected are mutually exclusive parameters.
// Validation should ensure only one is provided at a time.
