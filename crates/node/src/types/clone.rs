//! Clone command type definitions.
//!
//! # What
//!
//! This module contains type definitions for the clone command, including
//! parameter structures and response data types. The clone command clones
//! a repository and optionally initializes it as a workspace.
//!
//! # How
//!
//! Types are defined with `#[napi(object)]` attribute to be exposed as
//! JavaScript objects. The module provides:
//!
//! - `CloneParams`: Input parameters for the clone command
//! - `CloneData`: Response data containing clone operation results
//!
//! # Why
//!
//! The clone command provides a convenient way to clone repositories with
//! workspace configuration, handling SSH authentication and initialization
//! in a single operation.
//!
//! # Examples
//!
//! ```typescript
//! import { clone, CloneParams, CloneData } from '@websublime/workspace-tools';
//!
//! const params: CloneParams = {
//!   root: '.',
//!   url: 'git@github.com:org/repo.git',
//!   destination: 'my-repo',
//!   initialize: true
//! };
//! const result = await clone(params);
//!
//! if (result.success) {
//!   const data: CloneData = result.data;
//!   console.log(`Cloned to: ${data.path}`);
//!   console.log(`Branch: ${data.branch}`);
//!   if (data.initialized) {
//!     console.log(`Workspace initialized with config at: ${data.configPath}`);
//!   }
//! }
//! ```

// TODO: will be implemented on story 9.3 - Clone Types
// This module will contain:
//
// NAPI-specific types:
// - CloneParams: {
//     root: string,           // Base directory for clone
//     url: string,            // Repository URL (HTTPS or SSH)
//     destination?: string,   // Target directory name
//     branch?: string,        // Branch to checkout
//     depth?: number,         // Shallow clone depth
//     initialize?: boolean,   // Initialize workspace config after clone
//     strategy?: string,      // Versioning strategy if initializing
//     sshKeyPaths?: string[]  // Custom SSH key paths for authentication
//   }
// - CloneData: {
//     path: string,           // Absolute path to cloned repository
//     branch: string,         // Current branch after clone
//     url: string,            // Normalized repository URL
//     initialized: boolean,   // Whether workspace was initialized
//     configPath?: string     // Path to config file if initialized
//   }
