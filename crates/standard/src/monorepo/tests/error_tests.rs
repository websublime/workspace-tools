//! # Error Tests
//!
//! ## What
//! This module tests error handling and display functionality for
//! monorepo-related errors.
//!
//! ## How
//! Tests verify proper error formatting, display messages, and
//! error type conversions.
//!
//! ## Why
//! Proper testing of error handling ensures users receive clear
//! and informative error messages when operations fail.

use crate::error::MonorepoError;

#[tokio::test]
async fn test_monorepo_error_display() {
    let manager_not_found_error = MonorepoError::ManagerNotFound;
    assert_eq!(manager_not_found_error.to_string(), "Failed to find package manager");

    // Test that the error implements the expected traits
    let error_string = format!("{manager_not_found_error}");
    assert!(!error_string.is_empty());

    // Test Debug implementation
    let debug_string = format!("{manager_not_found_error:?}");
    assert!(debug_string.contains("ManagerNotFound"));
}
