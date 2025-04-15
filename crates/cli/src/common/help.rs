//! Custom help formatter for the CLI

use colored::Colorize;
use std::io::{self, Write};

/// Formats and prints a nicely formatted help message
pub fn print_custom_help(
    title: &str,
    description: &str,
    usage: &str,
    options: &[(&str, &str)],
    examples: &[(&str, &str)],
) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // Title and description
    writeln!(handle, "{}", title.bright_green().bold())?;
    writeln!(handle, "{}", description)?;
    writeln!(handle)?;

    // Usage
    writeln!(handle, "{}", "USAGE:".bright_yellow().bold())?;
    writeln!(handle, "  {}", usage)?;
    writeln!(handle)?;

    // Options
    if !options.is_empty() {
        writeln!(handle, "{}", "OPTIONS:".bright_yellow().bold())?;

        // Find the maximum length of option names for padding
        let max_len = options.iter().map(|(name, _)| name.len()).max().unwrap_or(0);

        for (name, desc) in options {
            let padding = " ".repeat(max_len - name.len() + 2);
            writeln!(handle, "  {}{}{}", name.bright_blue(), padding, desc)?;
        }
        writeln!(handle)?;
    }

    // Examples
    if !examples.is_empty() {
        writeln!(handle, "{}", "EXAMPLES:".bright_yellow().bold())?;

        for (cmd, desc) in examples {
            writeln!(handle, "  # {}", desc)?;
            writeln!(handle, "  {}", cmd.bright_green())?;
            writeln!(handle)?;
        }
    }

    Ok(())
}
