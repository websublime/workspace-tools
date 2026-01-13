//! # Tests Module
//!
//! Unit tests for all workspace-fs modules, organized by module.
//!
//! ## What
//!
//! This module contains the unit test suite for the workspace-fs crate. Tests are
//! organized into submodules that mirror the main crate structure, ensuring each
//! module's functionality is thoroughly validated.
//!
//! ## How
//!
//! Tests are structured following these conventions:
//! - Each source module has a corresponding test submodule (e.g., `mod error`, `mod config`)
//! - Tests use `#[tokio::test]` for async test functions
//! - The `MockFileSystem` is used for testing filesystem operations without disk I/O
//! - Tests follow the Arrange-Act-Assert pattern for clarity
//!
//! Test submodules:
//! - `error`: Tests for error type construction and display
//! - `config`: Tests for configuration builder and defaults
//! - `types`: Tests for `FileType`, `DirEntry`, and `Metadata`
//! - `path_ext`: Tests for path normalization and utilities
//! - `traits`: Tests for `FileSystem` trait behavior via mock
//! - `real`: Integration tests for `RealFileSystem` (uses tempdir)
//! - `mock`: Tests for `MockFileSystem` behavior
//!
//! ## Why
//!
//! A dedicated tests module provides:
//! - **Organization**: All unit tests in one place, separate from production code
//! - **Discoverability**: Easy to find and run tests for specific functionality
//! - **Maintainability**: Test code doesn't clutter implementation files
//! - **Coverage Tracking**: Clear mapping between modules and their tests
//!
//! ## Example
//!
//! ```rust,ignore
//! // Run all workspace-fs tests
//! cargo test -p workspace-fs
//!
//! // Run tests for a specific module
//! cargo test -p workspace-fs error
//!
//! // Run a specific test
//! cargo test -p workspace-fs test_read_error_display
//! ```

#[cfg(test)]
mod error {
    //! Tests for the error module.
    // TODO: will be implemented on epic workspace-node-tools-906 (Error Module)
}

#[cfg(test)]
mod config {
    //! Tests for the config module.
    // TODO: will be implemented on epic workspace-node-tools-g2t (Configuration Module)
}

#[cfg(test)]
mod types {
    //! Tests for the types module.
    // TODO: will be implemented on epic workspace-node-tools-3q8 (Types Module)
}

#[cfg(test)]
mod path_ext {
    //! Tests for the path_ext module.
    // TODO: will be implemented on epic workspace-node-tools-60y (PathExt Module)
}

#[cfg(test)]
mod traits {
    //! Tests for the traits module.
    // TODO: will be implemented on epic workspace-node-tools-hek (FileSystem Trait)
}

#[cfg(test)]
mod real {
    //! Tests for the real filesystem module.
    // TODO: will be implemented on epic workspace-node-tools-1gx (RealFileSystem Implementation)
}

#[cfg(test)]
mod mock {
    //! Tests for the mock filesystem module.
    // TODO: will be implemented on epic workspace-node-tools-0ea (MockFileSystem Implementation)
}
