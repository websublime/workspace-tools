//! UI components for consistent CLI output styling.
//!
//! This module provides a collection of UI widgets and utilities
//! for creating visually appealing and consistent command-line interfaces.

mod boxes;
mod help;
mod inputs;
mod lists;
mod messages;
mod progress;
mod symbols;
mod tables;
mod theme;

// Re-export everything for easier imports
pub use boxes::{error_box, help_box, info_box, plain_box, styled_box, success_box, warning_box};
pub use help::{
    command_help, command_section, format_command, format_option, help_box as help, usage_example,
};
pub use inputs::{
    confirm, multi_select, password, progress_bar as progress_bar_input, select,
    spinner as spinner_input, text, text_with_default, InputPrompt,
};
pub use lists::{bullet_list, numbered_list, separated_list, ListBuilder, ListMarkerType};
pub use messages::{
    command_example, error, file_path, highlight, info, key_value, muted, primary, secondary,
    section_header, success, warning,
};
pub use progress::{
    custom_progress_bar, download_progress_bar, multi_progress, progress_bar, progress_collection,
    progress_iterator, progress_vec, spinner, StagedProgress,
};
pub use symbols::{Symbol, SymbolType};
pub use tables::{create_table, key_value_table};
pub use theme::{
    current_palette, dim_style, error_style, highlight_style, info_style, muted_style,
    primary_style, secondary_style, success_style, use_default_theme, use_theme, warning_style,
    ThemePalette, ThemeStyle,
};

/// Initialize the UI system with the default theme
pub fn init() {
    theme::use_default_theme();
}

/// Initialize the UI system with a custom theme
pub fn init_with_theme(theme_name: &str) -> Result<(), String> {
    theme::use_theme(theme_name)
}

/// Convenience function to determine if color output is supported
pub fn supports_color() -> bool {
    console::colors_enabled()
}

/// Convenience function to determine if unicode symbols are supported
pub fn supports_unicode() -> bool {
    // Basic heuristic - we could make this more sophisticated
    std::env::var("NO_UNICODE").is_err()
}
