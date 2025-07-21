//! # Monorepo Tests Module
//!
//! ## What
//! This module organizes comprehensive tests for the monorepo functionality
//! into focused, maintainable test modules.
//!
//! ## How
//! Tests are organized by functionality:
//! - `test_utils`: Common test utilities and helper functions
//! - `monorepo_kind_tests`: Tests for MonorepoKind enum
//! - `monorepo_descriptor_tests`: Tests for MonorepoDescriptor functionality
//! - `package_manager_tests`: Tests for PackageManager operations
//! - `error_tests`: Tests for error handling and display
//!
//! ## Why
//! Modular test organization improves maintainability, reduces cognitive load,
//! and enables better testing isolation and documentation.

#[allow(clippy::unwrap_used)]
#[allow(clippy::get_unwrap)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod monorepo_kind_tests;

#[cfg(test)]
mod monorepo_descriptor_tests;

#[cfg(test)]
mod package_manager_tests;

#[cfg(test)]
mod error_tests;
