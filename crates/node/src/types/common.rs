//! Common type definitions and re-exports for Node.js bindings.
//!
//! # What
//!
//! This module provides NAPI-compatible wrappers and re-exports for types that
//! are used across multiple commands in the Node.js bindings. Instead of duplicating
//! types that already exist in `sublime_pkg_tools` and `sublime_standard_tools`,
//! this module re-exports them and provides any necessary NAPI wrappers.
//!
//! # How
//!
//! The module follows these principles:
//! - **Re-export existing types**: Types from `pkg` and `standard` crates are re-exported
//! - **NAPI wrappers**: When needed, thin wrapper types are created with `#[napi(object)]`
//! - **Conversion traits**: `From`/`Into` implementations convert between internal and NAPI types
//!
//! # Why
//!
//! Avoiding type duplication ensures consistency with the CLI layer and reduces
//! maintenance burden. The NAPI layer should be a thin binding over existing
//! functionality, not a parallel implementation.
//!
//! # Examples
//!
//! ```rust,ignore
//! use sublime_node_tools::types::common::{VersionBump, PackageInfo};
//!
//! // These are re-exported from sublime_pkg_tools
//! let bump = VersionBump::Minor;
//! ```

// Allow unused imports for placeholder re-exports that will be used in future stories
#[allow(unused_imports)]
// Re-export types from sublime_pkg_tools that are commonly used
pub(crate) use sublime_pkg_tools::types::VersionBump;

// Re-export types from sublime_standard_tools that are commonly used
#[allow(unused_imports)]
pub(crate) use sublime_standard_tools::monorepo::MonorepoKind;
#[allow(unused_imports)]
pub(crate) use sublime_standard_tools::node::{PackageManagerKind, RepoKind};

// Re-export the JSON response from CLI for consistency
#[allow(unused_imports)]
pub(crate) use sublime_cli_tools::output::JsonResponse;
