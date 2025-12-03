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

// TODO: will be implemented on story 3.1 (status types)
pub(crate) mod status;

// TODO: will be implemented on story 3.3 (init types)
pub(crate) mod init;

// TODO: will be implemented on story 7.1 (config types)
pub(crate) mod config;

// TODO: will be implemented on story 4.1 (changeset types)
pub(crate) mod changeset;

// TODO: will be implemented on story 5.1 (bump types)
pub(crate) mod bump;

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
