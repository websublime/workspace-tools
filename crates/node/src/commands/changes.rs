//! Changes command implementation for Node.js bindings.
//!
//! # What
//!
//! This module implements the `changes` NAPI function that analyzes what files
//! and packages have changed since a given reference point. It supports multiple
//! analysis modes including working directory, commit range, and with versions.
//!
//! # How
//!
//! The function:
//! 1. Validates the input parameters (root path, since reference, mode)
//! 2. Calls `execute_changes` from `sublime_cli_tools`
//! 3. Captures the JSON output containing change analysis
//! 4. Returns a `JsonResponse<ChangesData>` with package changes and summary
//!
//! # Why
//!
//! Change analysis is essential for:
//! - Targeted builds: Only build packages that have changed
//! - Targeted tests: Only test affected packages
//! - Release planning: Understanding scope of changes
//! - CI/CD optimization: Reduce pipeline time by focusing on changed code
//!
//! # Examples
//!
//! ```typescript
//! import { changes } from '@websublime/workspace-tools';
//!
//! // Analyze changes since a branch (e.g., main)
//! const result = await changes({
//!   root: '.',
//!   since: 'main',
//!   mode: 'commitRange'
//! });
//!
//! if (result.success) {
//!   console.log(`Packages with changes: ${result.data.summary.packagesWithChanges}`);
//!   console.log(`Total files changed: ${result.data.summary.totalFilesChanged}`);
//!   console.log(`Total commits: ${result.data.summary.totalCommits}`);
//!
//!   for (const pkg of result.data.packages) {
//!     if (pkg.hasChanges) {
//!       console.log(`\n${pkg.name}:`);
//!       console.log(`  Files: ${pkg.stats.filesModified} modified, ${pkg.stats.filesAdded} added`);
//!       console.log(`  Lines: +${pkg.stats.linesAdded} -${pkg.stats.linesDeleted}`);
//!
//!       for (const file of pkg.files) {
//!         console.log(`    ${file.changeType}: ${file.path}`);
//!       }
//!     }
//!   }
//! }
//!
//! // Analyze working directory changes (uncommitted)
//! const wdResult = await changes({
//!   root: '.',
//!   mode: 'workingDirectory'
//! });
//!
//! // Analyze changes between two commits
//! const rangeResult = await changes({
//!   root: '.',
//!   since: 'v1.0.0',
//!   until: 'HEAD',
//!   mode: 'commitRange'
//! });
//!
//! // Get changes for specific packages only
//! const filteredResult = await changes({
//!   root: '.',
//!   since: 'main',
//!   packages: ['@scope/core', '@scope/utils']
//! });
//! ```

// TODO: will be implemented on story 9.2 - Changes Command
//
// Implementation outline:
//
// #[napi]
// pub async fn changes(params: ChangesParams) -> JsonResponse<ChangesData> {
//     // 1. Validate parameters
//     if let Err(e) = validate_root(&params.root) {
//         return JsonResponse::from_error_info(e);
//     }
//
//     // 2. Determine analysis mode
//     //    - workingDirectory: Analyze uncommitted changes
//     //    - commitRange: Analyze changes between commits/refs
//     //    - withVersions: Analyze with version bump suggestions
//
//     // 3. Create ChangesArgs from params
//     let args = ChangesArgs {
//         since: params.since,
//         until: params.until,
//         packages: params.packages,
//         include_stats: params.include_stats.unwrap_or(true),
//         ..Default::default()
//     };
//
//     // 4. Create Output with JSON format for capturing
//     let mut buffer = Vec::new();
//     let output = Output::new(OutputFormat::Json, &mut buffer, false);
//
//     // 5. Determine workspace root path
//     let root_path = Path::new(&params.root);
//     let config_path = None; // Will use default config discovery
//
//     // 6. Call execute_changes from CLI (commands/changes.rs)
//     match execute_changes(&args, &output, root_path, config_path).await {
//         Ok(()) => {
//             // 7. Parse JSON response from buffer
//             let json_str = String::from_utf8_lossy(&buffer);
//             match serde_json::from_str::<JsonResponse<ChangesData>>(&json_str) {
//                 Ok(response) => response,
//                 Err(e) => JsonResponse::error(format!("Failed to parse changes response: {}", e)),
//             }
//         }
//         Err(e) => JsonResponse::error(format!("Changes analysis failed: {}", e)),
//     }
// }
//
// Note: The changes command relies on git operations to detect file changes.
// It uses the sublime_git_tools crate for git operations and sublime_pkg_tools
// for package mapping. The ChangesAnalyzer and PackageMapper from pkg crate
// handle the heavy lifting.
