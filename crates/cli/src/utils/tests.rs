//! Tests for utility modules.
//!
//! This module contains tests for all utility functionality in the CLI.
//!
//! # What
//!
//! Tests cover:
//! - Editor detection and launching
//! - Cross-platform compatibility
//! - Error handling
//!
//! # How
//!
//! Tests use:
//! - Platform-specific conditional compilation
//! - Mock commands for testing availability checks
//! - Error case validation
//!
//! # Why
//!
//! Comprehensive testing ensures:
//! - Utilities work correctly across platforms
//! - Error handling is robust
//! - Edge cases are handled properly

#[cfg(test)]
mod editor_tests {
    use crate::utils::editor::{detect_default_editor, detect_editor, is_command_available};

    // Note: Tests that set environment variables have been removed due to
    // concurrent test execution issues. Editor detection with environment
    // variables is tested through integration tests and manual verification.

    #[test]
    fn test_detect_default_editor() {
        // This test verifies that the default editor detection
        // returns a result (either success or appropriate error)
        let result = detect_default_editor();

        // On Unix, should find nano, vim, or vi, or return error
        // On Windows, should find notepad or return error
        // We can't assert specific behavior since it depends on the system
        match result {
            Ok(editor) => {
                assert!(!editor.is_empty(), "Editor should not be empty");
            }
            Err(e) => {
                // Error message should be user-friendly
                assert!(e.to_string().contains("editor") || e.to_string().contains("EDITOR"));
            }
        }
    }

    #[test]
    fn test_detect_editor_returns_result() {
        // Test that detect_editor returns a valid result
        // (either finds an editor or returns a clear error)
        let result = detect_editor();

        match result {
            Ok(editor) => {
                assert!(!editor.is_empty(), "Editor should not be empty");
            }
            Err(e) => {
                // Error should mention editor or environment variables
                let msg = e.to_string();
                assert!(
                    msg.contains("editor") || msg.contains("EDITOR") || msg.contains("VISUAL"),
                    "Error message should be descriptive: {msg}",
                );
            }
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_is_command_available_unix() {
        // Test with a command that should always exist on Unix
        assert!(is_command_available("ls"), "ls command should be available on Unix systems");

        // Test with a command that should not exist
        assert!(
            !is_command_available("this-command-definitely-does-not-exist-12345"),
            "Non-existent command should not be available"
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_is_command_available_windows() {
        // Test with a command that should always exist on Windows
        assert!(is_command_available("cmd.exe"), "cmd.exe should be available on Windows systems");

        // Test with a command that should not exist
        assert!(
            !is_command_available("this-command-definitely-does-not-exist-12345"),
            "Non-existent command should not be available"
        );
    }

    #[test]
    fn test_detect_default_editor_cross_platform() {
        // Test that detect_default_editor returns a result on all platforms
        let result = detect_default_editor();

        #[cfg(unix)]
        {
            // On Unix, should find at least one editor or return an error
            match result {
                Ok(editor) => {
                    assert!(
                        editor == "nano" || editor == "vim" || editor == "vi",
                        "Unexpected editor: {editor}",
                    );
                }
                Err(_) => {
                    // It's acceptable to not have any editors in test environment
                }
            }
        }

        #[cfg(windows)]
        {
            // On Windows, should find notepad or return an error
            match result {
                Ok(editor) => {
                    assert_eq!(editor, "notepad.exe");
                }
                Err(_) => {
                    // It's acceptable to not have notepad in test environment
                }
            }
        }
    }

    #[test]
    fn test_is_command_available_nonexistent() {
        // Verify that a clearly non-existent command returns false
        assert!(
            !is_command_available("absolutely-nonexistent-command-xyz-12345"),
            "Non-existent command should return false"
        );
    }

    // Note: Tests for open_in_editor are not included as they would require
    // actual file creation and editor launching, which is not suitable for
    // unit tests. These are covered by integration tests.
}
