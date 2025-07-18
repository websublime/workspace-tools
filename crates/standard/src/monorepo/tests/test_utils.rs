//! # Test Utilities
//!
//! ## What
//! This module provides common test utilities, helper functions, and shared
//! imports for all monorepo module tests.
//!
//! ## How
//! Contains helper functions for creating test data, temporary directories,
//! and common test assertions used across different test modules.
//!
//! ## Why
//! Centralizing test utilities reduces duplication and ensures consistent
//! test setup across all test modules.

use crate::monorepo::WorkspacePackage;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Helper function to create a temporary directory for tests
pub(crate) fn setup_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory for test")
}

/// Helper function to create test packages
pub(crate) fn create_test_package(
    name: &str,
    version: &str,
    location: &str,
    root: &Path,
    deps: Vec<&str>,
    dev_deps: Vec<&str>,
) -> WorkspacePackage {
    WorkspacePackage {
        name: name.to_string(),
        version: version.to_string(),
        location: PathBuf::from(location),
        absolute_path: root.join(location),
        workspace_dependencies: deps.into_iter().map(String::from).collect(),
        workspace_dev_dependencies: dev_deps.into_iter().map(String::from).collect(),
    }
}