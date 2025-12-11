//! Type definitions module for Node.js bindings.
//!
//! # What
//!
//! This module contains all NAPI-compatible type definitions used for parameters
//! and responses in the Node.js bindings. Types are organized by command group
//! and follow a consistent naming pattern.
//!
//! # How
//!
//! The module is organized into submodules by functionality:
//!
//! - **`common`**: Shared types used across multiple commands
//! - **`status`**: Types for the status command
//! - **`init`**: Types for the init command
//! - **`config`**: Types for config commands (show, validate)
//! - **`changeset`**: Types for changeset commands (add, update, list, show, remove, history, check)
//! - **`bump`**: Types for bump commands (preview, apply, snapshot)
//! - **`upgrade`**: Types for upgrade commands (check, apply, backup)
//! - **`audit`**: Types for the audit command
//! - **`changes`**: Types for the changes command
//! - **`clone`**: Types for the clone command
//! - **`execute`**: Types for the execute command
//!
//! Each submodule typically contains:
//! - `*Params`: Input parameter structures
//! - `*Data`: Response data structures
//!
//! All types use `#[napi(object)]` to be exposed as JavaScript objects.
//!
//! # Why
//!
//! Organizing types by command group provides:
//! - Clear separation of concerns
//! - Easy discovery of related types
//! - Consistent patterns across all commands
//! - Type-safe interfaces for JavaScript/TypeScript consumers
//!
//! # Examples
//!
//! ```typescript
//! import type { StatusParams, StatusData, ChangesetAddParams } from '@websublime/workspace-tools';
//!
//! // Using status types
//! const params: StatusParams = { root: '.' };
//! const result = await status(params);
//!
//! // Using changeset types
//! const changesetParams: ChangesetAddParams = {
//!   root: '.',
//!   packages: ['@scope/pkg1'],
//!   bumpType: 'minor',
//!   message: 'Add new feature'
//! };
//! ```

// Status types (Story 3.1 - Implemented, Story 3.2 - Status command implemented)
pub(crate) mod status;

// Re-export status types for easier access
// Allow unused imports - these will be used by future commands (changeset, bump, etc.)
#[allow(unused_imports)]
pub(crate) use status::{
    BranchInfo, ChangesetInfo, PackageInfo, PackageManagerInfo, RepositoryInfo, StatusApiResponse,
    StatusData, StatusParams,
};

// Init types (Story 3.3 - Implemented)
pub(crate) mod init;

// Re-export init types for easier access
// Allow unused imports - these will be used by the init command (Story 3.4)
#[allow(unused_imports)]
pub(crate) use init::{
    InitApiResponse, InitData, InitParams, VALID_CONFIG_FORMATS, VALID_STRATEGIES,
};

// TODO: will be implemented on story 7.1 (config types)
pub(crate) mod config;

// Changeset types (Story 4.1 - Implemented)
pub(crate) mod changeset;

// Re-export changeset types for easier access
// Allow unused imports - these will be used by changeset commands (Stories 4.2-4.8)
#[allow(unused_imports)]
pub(crate) use changeset::{
    // Supporting Types
    ArchivedChangesetInfo,
    // API Responses
    ChangesetAddApiResponse,
    // Response Data
    ChangesetAddData,
    // Input Parameters
    ChangesetAddParams,
    ChangesetCheckApiResponse,
    ChangesetCheckData,
    ChangesetCheckParams,
    ChangesetDetailInfo,
    ChangesetHistoryApiResponse,
    ChangesetHistoryData,
    ChangesetHistoryParams,
    ChangesetListApiResponse,
    ChangesetListData,
    ChangesetListParams,
    ChangesetRemoveApiResponse,
    ChangesetRemoveData,
    ChangesetRemoveParams,
    ChangesetShowApiResponse,
    ChangesetShowData,
    ChangesetShowParams,
    ChangesetUpdateApiResponse,
    ChangesetUpdateData,
    ChangesetUpdateParams,
    ReleaseInfoData,
    ReleasedVersionEntry,
    UpdateSummaryInfo,
    // Constants
    VALID_SORT_OPTIONS,
};

// Bump types (Story 5.1 - Implemented)
pub(crate) mod bump;

// Re-export bump types for easier access
// Allow unused imports - these will be used by bump commands (Stories 5.2-5.4)
#[allow(unused_imports)]
pub(crate) use bump::{
    BumpApplyApiResponse,
    BumpApplyData,
    BumpApplyParams,
    // API Responses
    BumpPreviewApiResponse,
    // Response Data
    BumpPreviewData,
    // Input Parameters
    BumpPreviewParams,
    BumpSnapshotApiResponse,
    BumpSnapshotData,
    BumpSnapshotParams,
    BumpSummaryInfo,
    // Constants
    COMMON_PRERELEASE_TAGS,
    DEFAULT_SNAPSHOT_FORMAT,
    DependencyUpdateInfo,
    // Supporting Types
    PackageVersionInfo,
    SnapshotVersionInfo,
    VALID_DEPENDENCY_TYPES,
};

// TODO: will be implemented on story 8.1 (upgrade types)
pub(crate) mod upgrade;

// TODO: will be implemented on story 9.1 (audit types)
pub(crate) mod audit;

// TODO: will be implemented on story 9.2 (changes types)
pub(crate) mod changes;

// TODO: will be implemented on story 9.3 (clone types)
pub(crate) mod clone;

// TODO: will be implemented on story 6.2 (execute types)
pub(crate) mod execute;

// Common types used across multiple commands
pub(crate) mod common;
