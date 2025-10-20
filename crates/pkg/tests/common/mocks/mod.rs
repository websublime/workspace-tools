//! # Mock Implementations for Testing
//!
//! This module provides mock implementations of external dependencies used throughout
//! the `sublime_pkg_tools` crate. These mocks are designed to be used in tests to
//! isolate functionality and avoid external dependencies.
//!
//! ## What
//!
//! Provides mock implementations for:
//! - **MockFileSystem**: In-memory filesystem for testing file operations
//! - **MockGitRepository**: Mock Git repository for testing Git operations
//! - **MockRegistry**: Mock NPM registry for testing package upgrades
//!
//! ## How
//!
//! Each mock implementation maintains state in memory and implements the relevant
//! traits from the standard and git tools crates. This allows tests to run without
//! touching the real filesystem or making network calls.
//!
//! ## Why
//!
//! Mock implementations provide:
//! - Fast test execution without I/O overhead
//! - Predictable test behavior independent of external state
//! - Ability to test error conditions that are hard to reproduce
//! - Isolation between test cases

pub mod filesystem;
pub mod git;
pub mod registry;

// Re-export commonly used mock types
// Note: Specific items are exported from submodules as needed by individual tests
