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

// Config types (Story 7.1 - Implemented)
pub(crate) mod config;

// Re-export config types for easier access
// Allow unused imports - these will be used by config commands (Stories 7.2-7.3)
#[allow(unused_imports)]
pub(crate) use config::{
    // Configuration Structures
    AuditConfigInfo,
    AuditSectionsConfigInfo,
    BackupConfigInfo,
    ChangelogConfigInfo,
    ChangesetConfigInfo,
    ConfigData,
    // API Responses
    ConfigShowApiResponse,
    // Response Data
    ConfigShowData,
    // Input Parameters
    ConfigShowParams,
    ConfigValidateApiResponse,
    ConfigValidateData,
    ConfigValidateParams,
    ConfigValidationIssue,
    DependencyConfigInfo,
    ExecuteConfigInfo,
    GitConfigInfo,
    HealthScoreWeightsInfo,
    RegistryConfigInfo,
    ScopedRegistryEntry,
    UpgradeConfigInfo,
    // Constants
    VALID_BUMP_TYPES,
    VALID_CHANGELOG_FORMATS,
    VALID_MONOREPO_MODES,
    VALID_SEVERITY_LEVELS,
    VALID_STRATEGIES as CONFIG_VALID_STRATEGIES,
    VersionConfigInfo,
};

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

// Upgrade types (Story 8.1 - Implemented)
pub(crate) mod upgrade;

// Re-export upgrade types for easier access
// Allow unused imports - these will be used by upgrade commands (Stories 8.2-8.4)
#[allow(unused_imports)]
pub(crate) use upgrade::{
    // Supporting Types
    AppliedUpgradeInfo,
    ApplySummaryInfo,
    // API Responses
    BackupCleanApiResponse,
    // Response Data
    BackupCleanData,
    // Input Parameters
    BackupCleanParams,
    BackupInfo,
    BackupListApiResponse,
    BackupListData,
    BackupListParams,
    BackupRestoreApiResponse,
    BackupRestoreData,
    BackupRestoreParams,
    // Constants
    DEFAULT_KEEP_COUNT,
    DependencyUpgradeInfo,
    FailedUpgradeInfo,
    PackageUpgradeInfo,
    SkippedUpgradeInfo,
    UpgradeApplyApiResponse,
    UpgradeApplyData,
    UpgradeApplyParams,
    UpgradeCheckApiResponse,
    UpgradeCheckData,
    UpgradeCheckParams,
    UpgradeSelectionInfo,
    UpgradeSummaryInfo,
    VALID_DEPENDENCY_TYPES as UPGRADE_VALID_DEPENDENCY_TYPES,
    VALID_UPGRADE_TYPES,
};

// TODO: will be implemented on story 9.1 (audit types)
pub(crate) mod audit;

// TODO: will be implemented on story 9.2 (changes types)
pub(crate) mod changes;

// TODO: will be implemented on story 9.3 (clone types)
pub(crate) mod clone;

// Execute types (Story 6.2 - Implemented, Story 6.3 - Execute command implemented)
pub(crate) mod execute;

// Re-export execute types for easier access
// Note: These types are primarily used via #[napi] macros, not internal Rust code
#[allow(unused_imports)]
pub(crate) use execute::{
    // API Response
    ExecuteApiResponse,
    // Response Data
    ExecuteData,
    // Input Parameters
    ExecuteParams,
    // Supporting Types
    ExecuteSummary,
    PackageExecutionResult,
};

// Common types used across multiple commands
pub(crate) mod common;
