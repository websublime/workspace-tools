//! Bump command implementations for Node.js bindings.
//!
//! # What
//!
//! This module implements the bump NAPI functions that handle version management
//! based on pending changesets. It supports previewing changes, applying versions,
//! and generating snapshot versions for pre-releases.
//!
//! # How
//!
//! The module provides the following functions:
//!
//! - `bumpPreview`: Shows what versions would be bumped without making changes
//! - `bumpApply`: Applies version bumps, updates dependencies, and optionally commits/tags
//! - `bumpSnapshot`: Generates snapshot versions for pre-release testing
//!
//! Each function:
//! 1. Validates the input parameters
//! 2. Calls the appropriate `execute_*` function from `sublime_cli_tools`
//! 3. Captures the JSON output
//! 4. Returns a `JsonResponse<T>` with the result
//!
//! # Why
//!
//! Version bumping is the culmination of the changeset workflow. These commands
//! provide fine-grained control over how versions are updated, including dry-run
//! capabilities and git integration for automated releases.
//!
//! # Examples
//!
//! ```typescript
//! import {
//!   bumpPreview,
//!   bumpApply,
//!   bumpSnapshot
//! } from '@websublime/workspace-tools';
//!
//! // Preview version bumps (dry run)
//! const previewResult = await bumpPreview({
//!   root: '.',
//!   showDiff: true
//! });
//! if (previewResult.success) {
//!   for (const pkg of previewResult.data.packages) {
//!     console.log(`${pkg.name}: ${pkg.currentVersion} -> ${pkg.nextVersion} (${pkg.bump})`);
//!   }
//!   console.log(`\nDependency updates: ${previewResult.data.summary.dependenciesUpdated}`);
//! }
//!
//! // Apply version bumps with git commit and tags
//! const applyResult = await bumpApply({
//!   root: '.',
//!   execute: true,
//!   gitCommit: true,
//!   gitTag: true,
//!   gitPush: false  // Don't push automatically
//! });
//! if (applyResult.success) {
//!   console.log(`Applied ${applyResult.data.applied.length} version bumps`);
//!   console.log(`Git commit: ${applyResult.data.gitCommitSha}`);
//!   console.log(`Git tags: ${applyResult.data.gitTags.join(', ')}`);
//! }
//!
//! // Generate snapshot versions for pre-release
//! const snapshotResult = await bumpSnapshot({
//!   root: '.',
//!   format: '{version}-snapshot.{timestamp}'
//! });
//! if (snapshotResult.success) {
//!   for (const pkg of snapshotResult.data.packages) {
//!     console.log(`${pkg.name}: ${pkg.snapshotVersion}`);
//!   }
//! }
//! ```

// TODO: will be implemented on story 5.2-5.4 - Bump Commands
//
// Implementation outline for bumpPreview:
//
// #[napi]
// pub async fn bump_preview(params: BumpPreviewParams) -> JsonResponse<BumpPreviewData> {
//     // 1. Validate parameters
//     if let Err(e) = validate_root(&params.root) {
//         return JsonResponse::from_error_info(e);
//     }
//
//     // 2. Create BumpArgs from params (with dry_run = true implicitly)
//     // 3. Create Output with JSON format for capturing
//     // 4. Call execute_bump_preview from CLI (commands/bump/preview.rs)
//     // 5. Parse JSON response containing packages, dependency updates, summary
//     // 6. Return JsonResponse::success(data) or JsonResponse::error(msg)
// }
//
// Implementation outline for bumpApply:
//
// #[napi]
// pub async fn bump_apply(params: BumpApplyParams) -> JsonResponse<BumpApplyData> {
//     // 1. Validate parameters
//     if let Err(e) = validate_root(&params.root) {
//         return JsonResponse::from_error_info(e);
//     }
//
//     // 2. Create BumpArgs from params
//     //    - execute: whether to actually write changes
//     //    - git_commit: whether to create a git commit
//     //    - git_tag: whether to create git tags
//     //    - git_push: whether to push to remote
//     // 3. Create Output with JSON format for capturing
//     // 4. Call execute_bump_apply from CLI (commands/bump/execute.rs)
//     // 5. Parse JSON response containing applied packages, git info, summary
//     // 6. Return JsonResponse::success(data) or JsonResponse::error(msg)
// }
//
// Implementation outline for bumpSnapshot:
//
// #[napi]
// pub async fn bump_snapshot(params: BumpSnapshotParams) -> JsonResponse<BumpSnapshotData> {
//     // 1. Validate parameters
//     if let Err(e) = validate_root(&params.root) {
//         return JsonResponse::from_error_info(e);
//     }
//
//     // 2. Create BumpArgs from params with snapshot mode
//     //    - format: snapshot version format template
//     // 3. Create Output with JSON format for capturing
//     // 4. Call execute_bump_snapshot from CLI (commands/bump/snapshot.rs)
//     // 5. Parse JSON response containing packages with snapshot versions
//     // 6. Return JsonResponse::success(data) or JsonResponse::error(msg)
// }
