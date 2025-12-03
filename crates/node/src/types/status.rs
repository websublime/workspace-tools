//! Status command type definitions.
//!
//! # What
//!
//! This module contains type definitions for the status command, including
//! parameter structures and response data types.
//!
//! # How
//!
//! Types are defined with `#[napi(object)]` attribute to be exposed as
//! JavaScript objects. The module provides:
//!
//! - `StatusParams`: Input parameters for the status command
//! - `StatusData`: Response data containing workspace status information
//!
//! # Why
//!
//! The status command is a fundamental operation that retrieves information
//! about the current workspace state, including package information, git
//! status, and configuration details.
//!
//! # Examples
//!
//! ```typescript
//! import { status, StatusParams, StatusData } from '@websublime/workspace-tools';
//!
//! const params: StatusParams = { root: '.' };
//! const result = await status(params);
//!
//! if (result.success) {
//!   const data: StatusData = result.data;
//!   console.log(`Repository: ${data.repository}`);
//!   console.log(`Package Manager: ${data.packageManager}`);
//!   console.log(`Packages: ${data.packages.length}`);
//! }
//! ```

// TODO: will be implemented on story 3.1 - Status Types
// This module will contain:
// - StatusParams: { root: string }
// - StatusData: { repository, packageManager, branch, packages, ... }
// - PackageStatusInfo: Extended package information with status details
