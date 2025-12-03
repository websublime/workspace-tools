//! Changeset command type definitions.
//!
//! # What
//!
//! This module contains type definitions for changeset commands (add, update,
//! list, show, remove, history, check), including parameter structures and
//! response data types.
//!
//! # How
//!
//! Types are defined with `#[napi(object)]` attribute to be exposed as
//! JavaScript objects. The module provides:
//!
//! - `ChangesetAddParams`: Input parameters for creating a changeset
//! - `ChangesetAddData`: Response data containing created changeset info
//! - `ChangesetUpdateParams`: Input parameters for updating a changeset
//! - `ChangesetListParams`: Input parameters for listing changesets
//! - `ChangesetListData`: Response data containing list of changesets
//! - `ChangesetShowParams`: Input parameters for showing a changeset
//! - `ChangesetShowData`: Response data containing changeset details
//! - `ChangesetRemoveParams`: Input parameters for removing a changeset
//! - `ChangesetHistoryParams`: Input parameters for querying history
//! - `ChangesetHistoryData`: Response data containing archived changesets
//! - `ChangesetCheckParams`: Input parameters for checking changeset status
//! - `ChangesetCheckData`: Response data containing check results
//!
//! # Why
//!
//! Changesets are the core workflow for tracking changes before version bumps.
//! These types enable programmatic access to the changeset workflow.
//!
//! # Examples
//!
//! ```typescript
//! import {
//!   changesetAdd,
//!   changesetList,
//!   changesetShow,
//!   ChangesetAddParams
//! } from '@websublime/workspace-tools';
//!
//! // Add a new changeset
//! const addParams: ChangesetAddParams = {
//!   root: '.',
//!   packages: ['@scope/pkg1', '@scope/pkg2'],
//!   bumpType: 'minor',
//!   message: 'Add new feature'
//! };
//! const addResult = await changesetAdd(addParams);
//!
//! // List pending changesets
//! const listResult = await changesetList({ root: '.' });
//! if (listResult.success) {
//!   for (const cs of listResult.data.changesets) {
//!     console.log(`${cs.branch}: ${cs.packages.join(', ')}`);
//!   }
//! }
//!
//! // Show a specific changeset
//! const showResult = await changesetShow({ root: '.', branch: 'feature/xyz' });
//! ```

// TODO: will be implemented on story 4.1 - Changeset Types
// This module will contain:
//
// Re-exports from sublime_pkg_tools:
// - pub use sublime_pkg_tools::types::{Changeset, ArchivedChangeset, ReleaseInfo, UpdateSummary};
//
// NAPI-specific types:
// - ChangesetAddParams: { root, packages, bumpType, message, environments? }
// - ChangesetAddData: { id, branch, packages, bump, created }
// - ChangesetUpdateParams: { root, branch, packages?, bumpType?, message?, addCommits? }
// - ChangesetUpdateData: { updated, summary }
// - ChangesetListParams: { root, verbose? }
// - ChangesetListData: { changesets: ChangesetInfo[], count }
// - ChangesetShowParams: { root, branch }
// - ChangesetShowData: { changeset: ChangesetInfo }
// - ChangesetRemoveParams: { root, branch, force? }
// - ChangesetRemoveData: { removed, branch }
// - ChangesetHistoryParams: { root, package?, startDate?, endDate?, limit? }
// - ChangesetHistoryData: { archived: ArchivedChangesetInfo[], count }
// - ChangesetCheckParams: { root, branch? }
// - ChangesetCheckData: { hasChangeset, branch?, packages? }
