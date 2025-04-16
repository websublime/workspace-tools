//! Help message formatting utilities.
//!
//! Provides functions for consistent help message display.

use super::boxes::styled_box;
use super::lists::ListBuilder;
use super::messages;
use super::theme;

/// Format a command description for help text
pub fn format_command(name: &str, description: &str) -> String {
    format!("{} - {}", theme::primary_style(name), description)
}

/// Format a command usage example
pub fn usage_example(command: &str) -> String {
    messages::command_example(command)
}

/// Format an option description
pub fn format_option(flag: &str, description: &str) -> String {
    format!("{} - {}", theme::primary_style(flag), description)
}

/// Create a boxed help message
pub fn help_box(title: &str, content: &str) -> String {
    styled_box(Some(&format!(" {} ", title)), content, theme::primary_style)
}

/// Create a command help message
pub fn command_help(
    command: &str,
    description: &str,
    usage: &str,
    options: &[(&str, &str)],
) -> String {
    let mut content = String::new();

    // Add command description
    content.push_str(&format!("{}\n\n", description));

    // Add usage section
    content.push_str(&format!(
        "{}\n{}\n\n",
        theme::highlight_style("USAGE:"),
        usage_example(usage)
    ));

    // Add options section if there are any
    if !options.is_empty() {
        content.push_str(&format!("{}\n", theme::highlight_style("OPTIONS:")));

        // Format options as a list
        let options_list = ListBuilder::new()
            .no_markers()
            .indent(2)
            .items(options.iter().map(|(flag, desc)| format_option(flag, desc)))
            .build();

        content.push_str(&format!("{}\n", options_list));
    }

    help_box(command, &content)
}

/// Format a section of related commands
pub fn command_section(title: &str, commands: &[(&str, &str)]) -> String {
    let mut content = String::new();

    // Add section title
    content.push_str(&format!("{}\n", theme::highlight_style(title)));

    // Format commands as a list
    let commands_list = ListBuilder::new()
        .no_markers()
        .indent(2)
        .items(commands.iter().map(|(name, desc)| format_command(name, desc)))
        .build();

    content.push_str(&commands_list);

    content
}
