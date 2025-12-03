//! Changes command type definitions.
//!
//! # What
//!
//! This module contains type definitions for the changes command, including
//! parameter structures and response data types. The changes command analyzes
//! what files and packages have changed since a given reference point.
//!
//! # How
//!
//! Types are defined with `#[napi(object)]` attribute to be exposed as
//! JavaScript objects. The module provides:
//!
//! - `ChangesParams`: Input parameters for the changes command
//! - `ChangesData`: Response data containing change analysis results
//!
//! # Why
//!
//! The changes command helps identify which packages have been modified,
//! enabling targeted builds, tests, and releases. It supports multiple
//! analysis modes including working directory, commit range, and with versions.
//!
//! # Examples
//!
//! ```typescript
//! import { changes, ChangesParams, ChangesData } from '@websublime/workspace-tools';
//!
//! // Analyze changes since a branch
//! const params: ChangesParams = {
//!   root: '.',
//!   since: 'main',
//!   mode: 'commitRange'
//! };
//! const result = await changes(params);
//!
//! if (result.success) {
//!   const data: ChangesData = result.data;
//!   console.log(`Packages with changes: ${data.summary.packagesWithChanges}`);
//!   console.log(`Total files changed: ${data.summary.totalFilesChanged}`);
//!
//!   for (const pkg of data.packages) {
//!     if (pkg.hasChanges) {
//!       console.log(`${pkg.name}: ${pkg.stats.filesModified} files modified`);
//!     }
//!   }
//! }
//!
//! // Analyze working directory changes
//! const wdResult = await changes({
//!   root: '.',
//!   mode: 'workingDirectory'
//! });
//! ```

// TODO: will be implemented on story 9.2 - Changes Types
// This module will contain:
//
// Re-exports from sublime_pkg_tools:
// - pub use sublime_pkg_tools::changes::{
//     ChangesReport, PackageChanges, FileChange, FileChangeType,
//     CommitInfo, ChangesSummary, PackageChangeStats, AnalysisMode
// };
//
// NAPI-specific types:
// - ChangesParams: { root, since?, until?, mode?, packages?, includeStats? }
// - ChangesData: { packages: PackageChangesInfo[], summary: ChangesSummaryInfo, mode }
//
// Shared types:
// - PackageChangesInfo: {
//     name, path, files: FileChangeInfo[], commits: CommitInfo[],
//     hasChanges, currentVersion?, nextVersion?, stats: PackageChangeStatsInfo
//   }
// - FileChangeInfo: { path, changeType, linesAdded?, linesDeleted? }
// - ChangesSummaryInfo: {
//     totalPackages, packagesWithChanges, totalFilesChanged, totalCommits
//   }
// - PackageChangeStatsInfo: {
//     filesAdded, filesModified, filesDeleted, linesAdded, linesDeleted
//   }
