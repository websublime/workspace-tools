//! Config command type definitions.
//!
//! # What
//!
//! This module contains type definitions for config commands (show, validate),
//! including parameter structures and response data types.
//!
//! # How
//!
//! Types are defined with `#[napi(object)]` attribute to be exposed as
//! JavaScript objects. The module provides:
//!
//! - `ConfigShowParams`: Input parameters for the config show command
//! - `ConfigShowData`: Response data containing configuration details
//! - `ConfigValidateParams`: Input parameters for the config validate command
//! - `ConfigValidateData`: Response data containing validation results
//!
//! # Why
//!
//! The config commands allow users to inspect and validate the workspace
//! configuration (repo.config) programmatically.
//!
//! # Examples
//!
//! ```typescript
//! import { configShow, configValidate } from '@websublime/workspace-tools';
//!
//! // Show configuration
//! const showResult = await configShow({ root: '.' });
//! if (showResult.success) {
//!   console.log(`Strategy: ${showResult.data.strategy}`);
//!   console.log(`Changeset path: ${showResult.data.changesetPath}`);
//! }
//!
//! // Validate configuration
//! const validateResult = await configValidate({ root: '.' });
//! if (validateResult.success) {
//!   console.log(`Valid: ${validateResult.data.valid}`);
//!   if (validateResult.data.warnings.length > 0) {
//!     console.log('Warnings:', validateResult.data.warnings);
//!   }
//! }
//! ```

// TODO: will be implemented on story 7.1 - Config Types
// This module will contain:
// - ConfigShowParams: { root: string }
// - ConfigShowData: { configPath, strategy, changesetPath, ... }
// - ConfigValidateParams: { root: string }
// - ConfigValidateData: { valid: boolean, errors: string[], warnings: string[] }
