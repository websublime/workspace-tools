//! # Package Manager Implementation
//!
//! ## What
//! This module previously contained package manager implementations.
//! These have been moved to the node module as package managers are
//! generic Node.js concepts, not monorepo-specific.
//!
//! ## How
//! All package manager functionality is now available through:
//! - `crate::node::PackageManager`
//! - `crate::node::PackageManagerKind`
//!
//! ## Why
//! This architectural change ensures proper separation of concerns where
//! generic Node.js concepts are in the node module and monorepo-specific
//! functionality remains in this module.

// This file is intentionally empty as all implementations have been moved to node module
