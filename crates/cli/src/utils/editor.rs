//! Editor utility module.
//!
//! This module provides functionality to open files in the user's preferred
//! text editor, with platform-specific fallbacks.
//!
//! # What
//!
//! Provides the `open_in_editor` function that:
//! - Detects the user's preferred editor from environment variables
//! - Falls back to platform-specific defaults
//! - Spawns the editor process and waits for completion
//! - Returns success/failure status
//!
//! # How
//!
//! Editor detection follows this priority order:
//! 1. $VISUAL environment variable (preferred for visual editors)
//! 2. $EDITOR environment variable (standard Unix convention)
//! 3. Platform-specific defaults:
//!    - Windows: notepad.exe
//!    - Unix/Linux/macOS: nano, vim, vi (in order of availability)
//!
//! The editor is launched as a child process with the file path as an argument,
//! and the function waits synchronously for the editor to exit.
//!
//! # Why
//!
//! Opening files in the user's editor is essential for:
//! - Allowing manual editing of changeset files
//! - Providing a familiar editing experience
//! - Respecting user preferences and conventions
//! - Supporting both CLI and GUI editors
//! - Cross-platform compatibility
//!
//! This is essential for commands like `changeset edit` where users need
//! to modify structured data files directly.
//!
//! # Examples
//!
//! ```rust,ignore
//! use sublime_cli_tools::utils::editor::open_in_editor;
//! use std::path::Path;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let file_path = Path::new(".changesets/feature-branch.json");
//! open_in_editor(file_path)?;
//! # Ok(())
//! # }
//! ```

use crate::error::{CliError, Result};
use std::path::Path;
use std::process::Command;
use tracing::{debug, info};

/// Opens a file in the user's preferred text editor.
///
/// Detects the editor from environment variables or uses platform defaults,
/// then spawns the editor process and waits for it to complete.
///
/// # Arguments
///
/// * `file_path` - Path to the file to open in the editor
///
/// # Returns
///
/// Returns `Ok(())` if the editor was launched successfully and exited cleanly.
///
/// # Errors
///
/// Returns an error if:
/// - No suitable editor can be found
/// - The editor process fails to spawn
/// - The editor exits with a non-zero status code
/// - File path conversion to string fails (non-UTF-8 paths)
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::utils::editor::open_in_editor;
/// use std::path::Path;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Open a changeset file for editing
/// let file_path = Path::new(".changesets/my-feature.json");
/// open_in_editor(file_path)?;
/// println!("Editor closed successfully");
/// # Ok(())
/// # }
/// ```
pub fn open_in_editor(file_path: &Path) -> Result<()> {
    let editor = detect_editor()?;

    info!("Opening file in editor: {} (file: {})", editor, file_path.display());

    // Convert path to string for command argument
    let file_str = file_path
        .to_str()
        .ok_or_else(|| CliError::io("File path contains invalid UTF-8 characters"))?;

    // Spawn the editor process
    let mut child = Command::new(&editor)
        .arg(file_str)
        .spawn()
        .map_err(|e| CliError::execution(format!("Failed to launch editor '{editor}': {e}")))?;

    debug!("Editor process spawned, waiting for completion");

    // Wait for the editor to exit
    let status = child
        .wait()
        .map_err(|e| CliError::execution(format!("Failed to wait for editor to exit: {e}")))?;

    if !status.success() {
        return Err(CliError::execution(format!(
            "Editor '{}' exited with status code: {}",
            editor,
            status.code().map_or(-1, |code| code)
        )));
    }

    info!("Editor closed successfully");
    Ok(())
}

/// Detects the user's preferred text editor.
///
/// Checks environment variables and platform defaults to find an available editor.
///
/// # Detection Order
///
/// 1. `VISUAL` environment variable
/// 2. `EDITOR` environment variable
/// 3. Platform-specific defaults (first available):
///    - Unix/Linux/macOS: nano, vim, vi
///    - Windows: notepad.exe
///
/// # Returns
///
/// Returns the editor command name/path as a string.
///
/// # Errors
///
/// Returns an error if no editor can be detected or all defaults are unavailable.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::utils::editor::detect_editor;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let editor = detect_editor()?;
/// println!("Detected editor: {}", editor);
/// # Ok(())
/// # }
/// ```
pub fn detect_editor() -> Result<String> {
    // Try VISUAL first (preferred for visual/GUI editors)
    if let Ok(visual) = std::env::var("VISUAL")
        && !visual.trim().is_empty()
    {
        debug!("Using VISUAL environment variable: {}", visual);
        return Ok(visual);
    }

    // Try EDITOR (standard Unix convention)
    if let Ok(editor) = std::env::var("EDITOR")
        && !editor.trim().is_empty()
    {
        debug!("Using EDITOR environment variable: {}", editor);
        return Ok(editor);
    }

    // Platform-specific defaults
    debug!("No editor environment variables set, trying platform defaults");
    detect_default_editor()
}

/// Detects a platform-specific default editor.
///
/// Searches for commonly available editors on the current platform.
///
/// # Unix/Linux/macOS
///
/// Tries in order: nano, vim, vi
///
/// # Windows
///
/// Uses: notepad.exe
///
/// # Returns
///
/// Returns the first available default editor.
///
/// # Errors
///
/// Returns an error if no default editor can be found on the system.
pub(crate) fn detect_default_editor() -> Result<String> {
    #[cfg(unix)]
    {
        // Try common Unix editors in order of user-friendliness
        let unix_editors = ["nano", "vim", "vi"];

        for editor in &unix_editors {
            if is_command_available(editor) {
                debug!("Using default Unix editor: {}", editor);
                return Ok((*editor).to_string());
            }
        }

        Err(CliError::user(
            "No text editor found. Please set the EDITOR or VISUAL environment variable, \
             or install nano, vim, or vi.",
        ))
    }

    #[cfg(windows)]
    {
        // On Windows, notepad.exe is always available
        let notepad = "notepad.exe";
        if is_command_available(notepad) {
            debug!("Using default Windows editor: {}", notepad);
            return Ok(notepad.to_string());
        }

        Err(CliError::user(
            "No text editor found. Please set the EDITOR or VISUAL environment variable.",
        ))
    }

    #[cfg(not(any(unix, windows)))]
    {
        Err(CliError::user(
            "No text editor could be detected on this platform. \
             Please set the EDITOR or VISUAL environment variable.",
        ))
    }
}

/// Checks if a command is available in the system PATH.
///
/// # Arguments
///
/// * `command` - The command name to check
///
/// # Returns
///
/// Returns `true` if the command exists and is executable, `false` otherwise.
pub(crate) fn is_command_available(command: &str) -> bool {
    #[cfg(unix)]
    {
        Command::new("which")
            .arg(command)
            .output()
            .is_ok_and(|output| output.status.success())
    }

    #[cfg(windows)]
    {
        Command::new("where")
            .arg(command)
            .output()
            .is_ok_and(|output| output.status.success())
    }

    #[cfg(not(any(unix, windows)))]
    {
        // For other platforms, assume the command exists if it's provided
        // This is a fallback that allows the system to try launching it
        true
    }
}
