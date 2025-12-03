//! Upgrade command implementations for Node.js bindings.
//!
//! # What
//!
//! This module implements the upgrade NAPI functions that handle dependency
//! upgrades from npm registries. It provides controlled upgrade workflows with
//! backup and restore capabilities for safety.
//!
//! # How
//!
//! The module provides the following functions:
//!
//! - `upgradeCheck`: Checks for available dependency upgrades
//! - `upgradeApply`: Applies selected upgrades to package.json files
//! - `backupCreate`: Creates a backup of current package.json files
//! - `backupRestore`: Restores package.json files from a backup
//! - `backupList`: Lists available backups
//!
//! Each function:
//! 1. Validates the input parameters
//! 2. Calls the appropriate `execute_*` function from `sublime_cli_tools`
//! 3. Captures the JSON output
//! 4. Returns a `JsonResponse<T>` with the result
//!
//! # Why
//!
//! Dependency management is critical for maintaining healthy projects. These
//! commands provide safe, controlled upgrade workflows that integrate with
//! the changeset system for tracking upgrade-related changes.
//!
//! # Examples
//!
//! ```typescript
//! import {
//!   upgradeCheck,
//!   upgradeApply,
//!   backupCreate,
//!   backupRestore,
//!   backupList
//! } from '@websublime/workspace-tools';
//!
//! // Check for available upgrades
//! const checkResult = await upgradeCheck({
//!   root: '.',
//!   includeMajor: false,
//!   includeMinor: true,
//!   includePatch: true
//! });
//! if (checkResult.success) {
//!   console.log(`Found ${checkResult.data.summary.totalUpgrades} upgrades:`);
//!   console.log(`  Major: ${checkResult.data.summary.majorUpgrades}`);
//!   console.log(`  Minor: ${checkResult.data.summary.minorUpgrades}`);
//!   console.log(`  Patch: ${checkResult.data.summary.patchUpgrades}`);
//!
//!   for (const pkg of checkResult.data.packages) {
//!     for (const dep of pkg.dependencies) {
//!       console.log(`${pkg.packageName}: ${dep.name} ${dep.currentVersion} -> ${dep.latestVersion}`);
//!     }
//!   }
//! }
//!
//! // Create a backup before applying upgrades
//! const backupResult = await backupCreate({ root: '.' });
//! const backupId = backupResult.data.backupId;
//!
//! // Apply upgrades (minor and patch only)
//! const applyResult = await upgradeApply({
//!   root: '.',
//!   selection: { minor: true, patch: true },
//!   createChangeset: true
//! });
//! if (applyResult.success) {
//!   console.log(`Applied ${applyResult.data.summary.totalApplied} upgrades`);
//!   if (applyResult.data.changesetId) {
//!     console.log(`Created changeset: ${applyResult.data.changesetId}`);
//!   }
//! }
//!
//! // If something went wrong, restore from backup
//! if (needsRollback) {
//!   const restoreResult = await backupRestore({
//!     root: '.',
//!     backupId
//!   });
//! }
//!
//! // List available backups
//! const listResult = await backupList({ root: '.' });
//! for (const backup of listResult.data.backups) {
//!   console.log(`${backup.id}: ${backup.createdAt} (${backup.packages.length} packages)`);
//! }
//! ```

// TODO: will be implemented on story 8.2-8.4 - Upgrade Commands
//
// Implementation outline for upgradeCheck:
//
// #[napi]
// pub async fn upgrade_check(params: UpgradeCheckParams) -> JsonResponse<UpgradeCheckData> {
//     // 1. Validate parameters
//     if let Err(e) = validate_root(&params.root) {
//         return JsonResponse::from_error_info(e);
//     }
//
//     // 2. Create detection options from params
//     //    - include_major, include_minor, include_patch
//     //    - packages filter (optional)
//     // 3. Create Output with JSON format for capturing
//     // 4. Call detect_upgrades from sublime_pkg_tools
//     // 5. Parse and format response with summary
//     // 6. Return JsonResponse::success(data) or JsonResponse::error(msg)
// }
//
// Implementation outline for upgradeApply:
//
// #[napi]
// pub async fn upgrade_apply(params: UpgradeApplyParams) -> JsonResponse<UpgradeApplyData> {
//     // 1. Validate parameters
//     if let Err(e) = validate_root(&params.root) {
//         return JsonResponse::from_error_info(e);
//     }
//
//     // 2. Optionally create backup if params.create_backup is true
//     // 3. Create UpgradeSelection from params.selection
//     // 4. Create Output with JSON format for capturing
//     // 5. Call apply_upgrades or apply_with_changeset from sublime_pkg_tools
//     // 6. Parse response with applied, skipped, failed counts
//     // 7. Return JsonResponse::success(data) or JsonResponse::error(msg)
// }
//
// Implementation outline for backupCreate:
//
// #[napi]
// pub async fn backup_create(params: BackupCreateParams) -> JsonResponse<BackupCreateData> {
//     // 1. Validate parameters
//     if let Err(e) = validate_root(&params.root) {
//         return JsonResponse::from_error_info(e);
//     }
//
//     // 2. Create BackupManager instance
//     // 3. Call create_backup
//     // 4. Return backup metadata (id, created_at, packages)
// }
//
// Implementation outline for backupRestore:
//
// #[napi]
// pub async fn backup_restore(params: BackupRestoreParams) -> JsonResponse<BackupRestoreData> {
//     // 1. Validate parameters
//     if let Err(e) = validate_root(&params.root) {
//         return JsonResponse::from_error_info(e);
//     }
//     // Also validate backup_id is not empty
//
//     // 2. Create BackupManager instance
//     // 3. Call restore_backup with the backup_id
//     // 4. Return restore results
// }
//
// Implementation outline for backupList:
//
// #[napi]
// pub async fn backup_list(params: BackupListParams) -> JsonResponse<BackupListData> {
//     // 1. Validate parameters
//     if let Err(e) = validate_root(&params.root) {
//         return JsonResponse::from_error_info(e);
//     }
//
//     // 2. Create BackupManager instance
//     // 3. Call list_backups
//     // 4. Return list of backup metadata
// }
