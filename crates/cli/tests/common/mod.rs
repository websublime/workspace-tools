//! Common test utilities for E2E CLI tests.
//!
//! **What**: Provides shared test infrastructure including fixtures,
//! assertions, and helpers for E2E testing.
//!
//! **How**: Exports reusable components that can be used across all
//! E2E test files.
//!
//! **Why**: Eliminates code duplication and ensures consistent test patterns.

pub mod assertions;
pub mod fixtures;
pub mod helpers;

// Re-export commonly used items
pub use helpers::*;
