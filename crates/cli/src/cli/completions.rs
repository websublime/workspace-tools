//! Shell completion generation.
//!
//! This module provides functionality to generate shell completion scripts
//! for various shells including bash, zsh, fish, and PowerShell.
//!
//! # What
//!
//! Provides:
//! - Shell completion generation for all supported shells
//! - `generate_completions` function for programmatic generation
//! - Support for bash, zsh, fish, and PowerShell
//! - Automatic command and argument completion
//!
//! # How
//!
//! Uses Clap's built-in completion generation to create shell-specific
//! completion scripts. These scripts enable tab-completion for commands,
//! subcommands, options, and arguments.
//!
//! # Why
//!
//! Shell completions significantly improve user experience by:
//! - Reducing typing and errors
//! - Discovering available commands and options
//! - Providing context-aware suggestions
//! - Following shell-specific conventions
//!
//! # Examples
//!
//! ```rust,ignore
//! use sublime_cli_tools::cli::{Cli, generate_completions};
//! use clap::CommandFactory;
//! use clap_complete::Shell;
//!
//! // Generate bash completions to stdout
//! let mut cli = Cli::command();
//! generate_completions(Shell::Bash, &mut cli, "wnt", &mut std::io::stdout());
//! ```
//!
//! Shell installation:
//!
//! ```bash
//! # Bash
//! wnt completions bash > /etc/bash_completion.d/wnt
//! # or
//! wnt completions bash > ~/.local/share/bash-completion/completions/wnt
//!
//! # Zsh
//! wnt completions zsh > /usr/local/share/zsh/site-functions/_wnt
//! # or
//! wnt completions zsh > ~/.zsh/completion/_wnt
//!
//! # Fish
//! wnt completions fish > ~/.config/fish/completions/wnt.fish
//!
//! # PowerShell
//! wnt completions powershell > $PROFILE
//! ```

use clap::Command;
use clap_complete::{Shell, generate};
use std::io::Write;

/// Generates shell completion script.
///
/// This function generates a completion script for the specified shell
/// and writes it to the provided writer. The generated script enables
/// tab-completion for all commands, subcommands, options, and arguments.
///
/// # Arguments
///
/// * `shell` - The target shell (bash, zsh, fish, or PowerShell)
/// * `cmd` - The Clap command to generate completions for
/// * `bin_name` - The name of the binary (typically "wnt")
/// * `buf` - The writer to output the completion script to
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::cli::{Cli, generate_completions};
/// use clap::CommandFactory;
/// use clap_complete::Shell;
/// use std::io::stdout;
///
/// let mut cmd = Cli::command();
/// generate_completions(Shell::Bash, &mut cmd, "wnt", &mut stdout());
/// ```
///
/// Generate to a file:
///
/// ```rust,ignore
/// use sublime_cli_tools::cli::{Cli, generate_completions};
/// use clap::CommandFactory;
/// use clap_complete::Shell;
/// use std::fs::File;
///
/// let mut cmd = Cli::command();
/// let mut file = File::create("completions.bash").unwrap();
/// generate_completions(Shell::Bash, &mut cmd, "wnt", &mut file);
/// ```
pub fn generate_completions<W: Write>(
    shell: Shell,
    cmd: &mut Command,
    bin_name: &str,
    buf: &mut W,
) {
    generate(shell, cmd, bin_name, buf);
}

/// Parses a shell name string into a Shell enum.
///
/// This function is useful for parsing shell names from command-line
/// arguments or configuration files.
///
/// # Arguments
///
/// * `shell_name` - The shell name as a string
///
/// # Returns
///
/// Returns `Some(Shell)` if the shell name is recognized, or `None` if not.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::cli::completions::parse_shell;
/// use clap_complete::Shell;
///
/// assert_eq!(parse_shell("bash"), Some(Shell::Bash));
/// assert_eq!(parse_shell("zsh"), Some(Shell::Zsh));
/// assert_eq!(parse_shell("fish"), Some(Shell::Fish));
/// assert_eq!(parse_shell("powershell"), Some(Shell::PowerShell));
/// assert_eq!(parse_shell("invalid"), None);
/// ```
#[must_use]
pub fn parse_shell(shell_name: &str) -> Option<Shell> {
    match shell_name.to_lowercase().as_str() {
        "bash" => Some(Shell::Bash),
        "zsh" => Some(Shell::Zsh),
        "fish" => Some(Shell::Fish),
        "powershell" | "pwsh" => Some(Shell::PowerShell),
        "elvish" => Some(Shell::Elvish),
        _ => None,
    }
}

/// Returns a list of supported shell names.
///
/// This function is useful for displaying available shells to users
/// or validating shell names.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::cli::completions::supported_shells;
///
/// let shells = supported_shells();
/// assert!(shells.contains(&"bash"));
/// assert!(shells.contains(&"zsh"));
/// assert!(shells.contains(&"fish"));
/// assert!(shells.contains(&"powershell"));
/// ```
#[must_use]
pub fn supported_shells() -> Vec<&'static str> {
    vec!["bash", "zsh", "fish", "powershell", "elvish"]
}

/// Generates installation instructions for the specified shell.
///
/// Returns a human-readable string with instructions on how to install
/// the generated completion script for the target shell.
///
/// # Arguments
///
/// * `shell` - The target shell
///
/// # Returns
///
/// A string containing installation instructions.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::cli::completions::installation_instructions;
/// use clap_complete::Shell;
///
/// let instructions = installation_instructions(Shell::Bash);
/// assert!(instructions.contains("bash"));
/// ```
#[must_use]
pub fn installation_instructions(shell: Shell) -> String {
    match shell {
        Shell::Bash => {
            "# Bash completion installation:
#
# Option 1: System-wide (requires sudo)
#   sudo cp completions.bash /etc/bash_completion.d/wnt
#
# Option 2: User-level
#   mkdir -p ~/.local/share/bash-completion/completions
#   cp completions.bash ~/.local/share/bash-completion/completions/wnt
#
# Option 3: Direct sourcing in ~/.bashrc
#   echo 'source /path/to/completions.bash' >> ~/.bashrc
#
# Then reload your shell:
#   source ~/.bashrc"
        }
        Shell::Zsh => {
            "# Zsh completion installation:
#
# Option 1: System-wide (requires sudo)
#   sudo cp completions.zsh /usr/local/share/zsh/site-functions/_wnt
#
# Option 2: User-level
#   mkdir -p ~/.zsh/completion
#   cp completions.zsh ~/.zsh/completion/_wnt
#   # Add to ~/.zshrc if not already present:
#   echo 'fpath=(~/.zsh/completion $fpath)' >> ~/.zshrc
#   echo 'autoload -Uz compinit && compinit' >> ~/.zshrc
#
# Then reload your shell:
#   source ~/.zshrc"
        }
        Shell::Fish => {
            "# Fish completion installation:
#
# Option 1: User-level (recommended)
#   mkdir -p ~/.config/fish/completions
#   cp completions.fish ~/.config/fish/completions/wnt.fish
#
# Option 2: System-wide (requires sudo)
#   sudo cp completions.fish /usr/share/fish/vendor_completions.d/wnt.fish
#
# Completions are loaded automatically by fish."
        }
        Shell::PowerShell => {
            "# PowerShell completion installation:
#
# Option 1: Add to your PowerShell profile
#   notepad $PROFILE
#   # Then paste the contents of completions.ps1
#
# Option 2: Source the file in your profile
#   echo '. /path/to/completions.ps1' >> $PROFILE
#
# Then reload your profile:
#   . $PROFILE"
        }
        Shell::Elvish => {
            "# Elvish completion installation:
#
# Add to your Elvish rc file (~/.elvish/rc.elv):
#   eval (wnt completions elvish)
#
# Or save to a file and source it:
#   wnt completions elvish > ~/.elvish/completions/wnt.elv
#   echo 'use ./completions/wnt' >> ~/.elvish/rc.elv"
        }
        _ => "# Unknown shell - no installation instructions available",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shell_valid() {
        assert_eq!(parse_shell("bash"), Some(Shell::Bash));
        assert_eq!(parse_shell("BASH"), Some(Shell::Bash));
        assert_eq!(parse_shell("zsh"), Some(Shell::Zsh));
        assert_eq!(parse_shell("fish"), Some(Shell::Fish));
        assert_eq!(parse_shell("powershell"), Some(Shell::PowerShell));
        assert_eq!(parse_shell("pwsh"), Some(Shell::PowerShell));
        assert_eq!(parse_shell("elvish"), Some(Shell::Elvish));
    }

    #[test]
    fn test_parse_shell_invalid() {
        assert_eq!(parse_shell("invalid"), None);
        assert_eq!(parse_shell(""), None);
        assert_eq!(parse_shell("cmd"), None);
    }

    #[test]
    fn test_supported_shells() {
        let shells = supported_shells();
        assert_eq!(shells.len(), 5);
        assert!(shells.contains(&"bash"));
        assert!(shells.contains(&"zsh"));
        assert!(shells.contains(&"fish"));
        assert!(shells.contains(&"powershell"));
        assert!(shells.contains(&"elvish"));
    }

    #[test]
    fn test_installation_instructions_not_empty() {
        assert!(!installation_instructions(Shell::Bash).is_empty());
        assert!(!installation_instructions(Shell::Zsh).is_empty());
        assert!(!installation_instructions(Shell::Fish).is_empty());
        assert!(!installation_instructions(Shell::PowerShell).is_empty());
        assert!(!installation_instructions(Shell::Elvish).is_empty());
    }

    #[test]
    fn test_installation_instructions_contains_shell_name() {
        assert!(installation_instructions(Shell::Bash).to_lowercase().contains("bash"));
        assert!(installation_instructions(Shell::Zsh).to_lowercase().contains("zsh"));
        assert!(installation_instructions(Shell::Fish).to_lowercase().contains("fish"));
    }
}
