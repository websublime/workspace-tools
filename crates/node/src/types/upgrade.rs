//! Upgrade command type definitions.
//!
//! # What
//!
//! This module contains type definitions for upgrade commands (check, apply,
//! backup), including parameter structures and response data types.
//!
//! # How
//!
//! Types are defined with `#[napi(object)]` attribute to be exposed as
//! JavaScript objects. The module provides:
//!
//! - `UpgradeCheckParams`: Input parameters for checking available upgrades
//! - `UpgradeCheckData`: Response data containing available upgrades
//! - `UpgradeApplyParams`: Input parameters for applying upgrades
//! - `UpgradeApplyData`: Response data containing applied upgrades
//! - `BackupCreateParams`: Input parameters for creating a backup
//! - `BackupCreateData`: Response data containing backup information
//! - `BackupRestoreParams`: Input parameters for restoring from backup
//! - `BackupRestoreData`: Response data containing restore results
//! - `BackupListParams`: Input parameters for listing backups
//! - `BackupListData`: Response data containing backup list
//!
//! # Why
//!
//! The upgrade commands handle dependency updates from npm registries.
//! They provide controlled upgrade workflows with backup and restore
//! capabilities for safety.
//!
//! # Examples
//!
//! ```typescript
//! import {
//!   upgradeCheck,
//!   upgradeApply,
//!   backupCreate,
//!   backupRestore,
//!   UpgradeCheckParams
//! } from '@websublime/workspace-tools';
//!
//! // Check for available upgrades
//! const checkParams: UpgradeCheckParams = {
//!   root: '.',
//!   includeMajor: false,
//!   includeMinor: true,
//!   includePatch: true
//! };
//! const checkResult = await upgradeCheck(checkParams);
//! if (checkResult.success) {
//!   for (const pkg of checkResult.data.packages) {
//!     for (const dep of pkg.dependencies) {
//!       console.log(`${dep.name}: ${dep.currentVersion} -> ${dep.latestVersion}`);
//!     }
//!   }
//! }
//!
//! // Apply upgrades with backup
//! const applyResult = await upgradeApply({
//!   root: '.',
//!   createBackup: true,
//!   selection: { minor: true, patch: true }
//! });
//!
//! // Restore from backup if needed
//! if (applyResult.success && applyResult.data.backupId) {
//!   const restoreResult = await backupRestore({
//!     root: '.',
//!     backupId: applyResult.data.backupId
//!   });
//! }
//! ```

// TODO: will be implemented on story 8.1 - Upgrade Types
// This module will contain:
//
// Re-exports from sublime_pkg_tools:
// - pub use sublime_pkg_tools::upgrade::{
//     PackageUpgrades, DependencyUpgrade, UpgradeType, UpgradePreview,
//     UpgradeSummary, UpgradeResult, AppliedUpgrade, ApplySummary,
//     BackupMetadata, DetectionOptions, UpgradeSelection
// };
//
// NAPI-specific types:
// - UpgradeCheckParams: { root, includeMajor?, includeMinor?, includePatch?, packages? }
// - UpgradeCheckData: { packages: PackageUpgradeInfo[], summary: UpgradeSummaryInfo }
// - UpgradeApplyParams: { root, createBackup?, selection?, createChangeset? }
// - UpgradeApplyData: { applied, skipped, failed, backupId?, changesetId? }
// - BackupCreateParams: { root }
// - BackupCreateData: { backupId, createdAt, packages }
// - BackupRestoreParams: { root, backupId }
// - BackupRestoreData: { restored, packages }
// - BackupListParams: { root }
// - BackupListData: { backups: BackupInfo[] }
//
// Shared types:
// - PackageUpgradeInfo: { packageName, packagePath, dependencies: DependencyUpgradeInfo[] }
// - DependencyUpgradeInfo: { name, currentVersion, latestVersion, upgradeType, dependencyType }
// - UpgradeSummaryInfo: { totalUpgrades, majorUpgrades, minorUpgrades, patchUpgrades }
// - BackupInfo: { id, createdAt, packages, sizeBytes }
