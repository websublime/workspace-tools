//! Bump command type definitions.
//!
//! # What
//!
//! This module contains type definitions for bump commands (preview, apply,
//! snapshot), including parameter structures and response data types.
//!
//! # How
//!
//! Types are defined with `#[napi(object)]` attribute to be exposed as
//! JavaScript objects. The module provides:
//!
//! - `BumpPreviewParams`: Input parameters for previewing version bumps
//! - `BumpPreviewData`: Response data containing preview information
//! - `BumpApplyParams`: Input parameters for applying version bumps
//! - `BumpApplyData`: Response data containing applied changes
//! - `BumpSnapshotParams`: Input parameters for snapshot versioning
//! - `BumpSnapshotData`: Response data containing snapshot versions
//!
//! # Why
//!
//! The bump commands handle version management based on changesets. Preview
//! allows users to see what will change before applying, apply performs the
//! actual version bumps, and snapshot creates pre-release versions.
//!
//! # Examples
//!
//! ```typescript
//! import {
//!   bumpPreview,
//!   bumpApply,
//!   bumpSnapshot,
//!   BumpPreviewParams
//! } from '@websublime/workspace-tools';
//!
//! // Preview version bumps
//! const previewParams: BumpPreviewParams = {
//!   root: '.',
//!   showDiff: true
//! };
//! const previewResult = await bumpPreview(previewParams);
//! if (previewResult.success) {
//!   for (const pkg of previewResult.data.packages) {
//!     console.log(`${pkg.name}: ${pkg.currentVersion} -> ${pkg.nextVersion}`);
//!   }
//! }
//!
//! // Apply version bumps
//! const applyResult = await bumpApply({
//!   root: '.',
//!   execute: true,
//!   gitCommit: true,
//!   gitTag: true
//! });
//!
//! // Create snapshot versions
//! const snapshotResult = await bumpSnapshot({
//!   root: '.',
//!   format: '{version}-snapshot.{timestamp}'
//! });
//! ```

// TODO: will be implemented on story 5.1 - Bump Types
// This module will contain:
//
// Re-exports from sublime_pkg_tools:
// - pub use sublime_pkg_tools::version::{VersionResolution, PackageUpdate, ApplyResult, ApplySummary};
//
// NAPI-specific types:
// - BumpPreviewParams: { root, showDiff?, packages? }
// - BumpPreviewData: { packages: PackageVersionInfo[], dependencyUpdates, summary }
// - BumpApplyParams: { root, execute?, gitCommit?, gitTag?, gitPush?, packages? }
// - BumpApplyData: { applied: PackageVersionInfo[], summary, gitCommitSha?, gitTags? }
// - BumpSnapshotParams: { root, format?, packages? }
// - BumpSnapshotData: { packages: SnapshotVersionInfo[], format }
//
// Shared types:
// - PackageVersionInfo: { name, path, currentVersion, nextVersion, bump, dependencyUpdates }
// - SnapshotVersionInfo: { name, path, originalVersion, snapshotVersion }
