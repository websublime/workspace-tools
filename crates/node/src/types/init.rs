//! Init command type definitions.
//!
//! # What
//!
//! This module contains type definitions for the init command, including
//! parameter structures and response data types. The init command initializes
//! a new workspace configuration.
//!
//! # How
//!
//! Types are defined with `#[napi(object)]` attribute to be exposed as
//! JavaScript objects. The module provides:
//!
//! - `InitParams`: Input parameters for the init command
//! - `InitData`: Response data containing initialization result
//!
//! # Why
//!
//! The init command sets up the workspace configuration file (repo.config)
//! with the appropriate versioning strategy and settings.
//!
//! # Examples
//!
//! ```typescript
//! import { init, InitParams, InitData } from '@websublime/workspace-tools';
//!
//! const params: InitParams = {
//!   root: '.',
//!   strategy: 'independent',
//!   format: 'toml'
//! };
//! const result = await init(params);
//!
//! if (result.success) {
//!   const data: InitData = result.data;
//!   console.log(`Created config at: ${data.configPath}`);
//! }
//! ```

// TODO: will be implemented on story 3.3 - Init Types
// This module will contain:
// - InitParams: { root: string, strategy?: string, format?: string, force?: boolean }
// - InitData: { configPath: string, strategy: string, format: string, created: boolean }
