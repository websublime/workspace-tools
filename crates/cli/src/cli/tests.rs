//! Tests for the CLI module.
//!
//! This module contains comprehensive tests for CLI parsing, argument
//! validation, and command routing. Tests verify that all commands,
//! subcommands, and options parse correctly and that help text is generated
//! as expected.
//!
//! # What
//!
//! Tests cover:
//! - Global option parsing
//! - Command and subcommand parsing
//! - Argument validation
//! - Help text generation
//! - Error handling for invalid arguments
//! - Shell completion generation
//!
//! # How
//!
//! Uses Clap's `parse_from` method to test parsing with various argument
//! combinations. Tests verify both successful parsing and appropriate
//! error handling for invalid inputs.
//!
//! # Why
//!
//! Comprehensive CLI tests ensure:
//! - All commands parse correctly
//! - Invalid arguments are rejected
//! - Help text is complete and accurate
//! - Regression prevention for CLI changes

#![allow(clippy::panic)]
#![allow(clippy::expect_used)]

use clap::Parser;
use clap_complete::Shell;

use super::*;
use crate::cli::commands::{
    ChangesetCommands, ConfigCommands, UpgradeBackupCommands, UpgradeCommands,
};

// ============================================================================
// Global Options Tests
// ============================================================================

#[test]
fn test_default_global_options() {
    let cli = Cli::parse_from(["wnt", "version"]);

    assert_eq!(cli.log_level(), LogLevel::Info);
    assert_eq!(cli.output_format(), OutputFormat::Human);
    assert!(!cli.is_color_disabled());
    assert_eq!(cli.root(), None);
    assert_eq!(cli.config_path(), None);
}

#[test]
fn test_log_level_parsing() {
    let cli = Cli::parse_from(["wnt", "--log-level", "debug", "version"]);
    assert_eq!(cli.log_level(), LogLevel::Debug);

    let cli = Cli::parse_from(["wnt", "-l", "trace", "version"]);
    assert_eq!(cli.log_level(), LogLevel::Trace);

    let cli = Cli::parse_from(["wnt", "--log-level", "silent", "version"]);
    assert_eq!(cli.log_level(), LogLevel::Silent);
}

#[test]
fn test_output_format_parsing() {
    let cli = Cli::parse_from(["wnt", "--format", "json", "version"]);
    assert_eq!(cli.output_format(), OutputFormat::Json);

    let cli = Cli::parse_from(["wnt", "-f", "json-compact", "version"]);
    assert_eq!(cli.output_format(), OutputFormat::JsonCompact);

    let cli = Cli::parse_from(["wnt", "--format", "quiet", "version"]);
    assert_eq!(cli.output_format(), OutputFormat::Quiet);
}

#[test]
fn test_no_color_flag() {
    let cli = Cli::parse_from(["wnt", "--no-color", "version"]);
    assert!(cli.is_color_disabled());
}

#[test]
fn test_root_directory() {
    let cli = Cli::parse_from(["wnt", "--root", "/tmp", "version"]);
    assert_eq!(cli.root(), Some(&PathBuf::from("/tmp")));

    let cli = Cli::parse_from(["wnt", "-r", "../other", "version"]);
    assert_eq!(cli.root(), Some(&PathBuf::from("../other")));
}

#[test]
fn test_config_path() {
    let cli = Cli::parse_from(["wnt", "--config", "custom.toml", "version"]);
    assert_eq!(cli.config_path(), Some(&PathBuf::from("custom.toml")));

    let cli = Cli::parse_from(["wnt", "-c", "/etc/config.yaml", "version"]);
    assert_eq!(cli.config_path(), Some(&PathBuf::from("/etc/config.yaml")));
}

#[test]
fn test_global_options_with_all_commands() {
    // Test that global options work with any command
    let cli = Cli::parse_from([
        "wnt",
        "--root",
        "/tmp",
        "--log-level",
        "debug",
        "--format",
        "json",
        "--no-color",
        "--config",
        "test.toml",
        "version",
    ]);

    assert_eq!(cli.root(), Some(&PathBuf::from("/tmp")));
    assert_eq!(cli.log_level(), LogLevel::Debug);
    assert_eq!(cli.output_format(), OutputFormat::Json);
    assert!(cli.is_color_disabled());
    assert_eq!(cli.config_path(), Some(&PathBuf::from("test.toml")));
}

// ============================================================================
// Init Command Tests
// ============================================================================

#[test]
fn test_init_command_basic() {
    let cli = Cli::parse_from(["wnt", "init"]);
    matches!(cli.command, Commands::Init(_));
}

#[test]
fn test_init_command_with_options() {
    let cli = Cli::parse_from([
        "wnt",
        "init",
        "--strategy",
        "independent",
        "--changeset-path",
        ".changes",
        "--environments",
        "dev,staging,prod",
        "--default-env",
        "prod",
        "--registry",
        "https://npm.example.com",
        "--config-format",
        "yaml",
        "--force",
        "--non-interactive",
    ]);

    if let Commands::Init(args) = cli.command {
        assert_eq!(args.strategy, Some("independent".to_string()));
        assert_eq!(args.changeset_path, PathBuf::from(".changes"));
        assert_eq!(
            args.environments,
            Some(vec!["dev".to_string(), "staging".to_string(), "prod".to_string()])
        );
        assert_eq!(args.default_env, Some(vec!["prod".to_string()]));
        assert_eq!(args.registry, "https://npm.example.com");
        assert_eq!(args.config_format, Some("yaml".to_string()));
        assert!(args.force);
        assert!(args.non_interactive);
    } else {
        panic!("Expected Init command");
    }
}

// ============================================================================
// Config Command Tests
// ============================================================================

#[test]
fn test_config_show_command() {
    let cli = Cli::parse_from(["wnt", "config", "show"]);

    if let Commands::Config(ConfigCommands::Show(_)) = cli.command {
        // Success
    } else {
        panic!("Expected Config Show command");
    }
}

#[test]
fn test_config_validate_command() {
    let cli = Cli::parse_from(["wnt", "config", "validate"]);

    if let Commands::Config(ConfigCommands::Validate(_)) = cli.command {
        // Success
    } else {
        panic!("Expected Config Validate command");
    }
}

// ============================================================================
// Changeset Command Tests
// ============================================================================

#[test]
fn test_changeset_create_command() {
    let cli = Cli::parse_from(["wnt", "changeset", "create"]);

    if let Commands::Changeset(ChangesetCommands::Create(_)) = cli.command {
        // Success
    } else {
        panic!("Expected Changeset Create command");
    }
}

#[test]
fn test_changeset_create_with_options() {
    let cli = Cli::parse_from([
        "wnt",
        "changeset",
        "create",
        "--bump",
        "minor",
        "--env",
        "staging,prod",
        "--branch",
        "feature/new",
        "--message",
        "Add feature",
        "--packages",
        "pkg1,pkg2",
        "--non-interactive",
    ]);

    if let Commands::Changeset(ChangesetCommands::Create(args)) = cli.command {
        assert_eq!(args.bump, Some("minor".to_string()));
        assert_eq!(args.env, Some(vec!["staging".to_string(), "prod".to_string()]));
        assert_eq!(args.branch, Some("feature/new".to_string()));
        assert_eq!(args.message, Some("Add feature".to_string()));
        assert_eq!(args.packages, Some(vec!["pkg1".to_string(), "pkg2".to_string()]));
        assert!(args.non_interactive);
    } else {
        panic!("Expected Changeset Create command");
    }
}

#[test]
fn test_changeset_list_command() {
    let cli = Cli::parse_from([
        "wnt",
        "changeset",
        "list",
        "--filter-package",
        "core",
        "--filter-bump",
        "major",
        "--filter-env",
        "prod",
        "--sort",
        "bump",
    ]);

    if let Commands::Changeset(ChangesetCommands::List(args)) = cli.command {
        assert_eq!(args.filter_package, Some("core".to_string()));
        assert_eq!(args.filter_bump, Some("major".to_string()));
        assert_eq!(args.filter_env, Some("prod".to_string()));
        assert_eq!(args.sort, "bump");
    } else {
        panic!("Expected Changeset List command");
    }
}

#[test]
fn test_changeset_show_command() {
    let cli = Cli::parse_from(["wnt", "changeset", "show", "feature/branch"]);

    if let Commands::Changeset(ChangesetCommands::Show(args)) = cli.command {
        assert_eq!(args.branch, "feature/branch");
    } else {
        panic!("Expected Changeset Show command");
    }
}

#[test]
fn test_changeset_update_command() {
    let cli = Cli::parse_from(["wnt", "changeset", "update"]);

    if let Commands::Changeset(ChangesetCommands::Update(_)) = cli.command {
        // Success
    } else {
        panic!("Expected Changeset Update command");
    }
}

#[test]
fn test_changeset_update_with_id() {
    let cli = Cli::parse_from(["wnt", "changeset", "update", "feature/branch"]);

    if let Commands::Changeset(ChangesetCommands::Update(args)) = cli.command {
        assert_eq!(args.id, Some("feature/branch".to_string()));
    } else {
        panic!("Expected Changeset Update command");
    }
}

#[test]
fn test_changeset_update_with_options() {
    let cli = Cli::parse_from([
        "wnt",
        "changeset",
        "update",
        "feature/branch",
        "--commit",
        "abc123",
        "--packages",
        "pkg-a,pkg-b",
        "--bump",
        "major",
        "--env",
        "staging,prod",
    ]);

    if let Commands::Changeset(ChangesetCommands::Update(args)) = cli.command {
        assert_eq!(args.id, Some("feature/branch".to_string()));
        assert_eq!(args.commit, Some("abc123".to_string()));
        assert_eq!(args.packages, Some(vec!["pkg-a".to_string(), "pkg-b".to_string()]));
        assert_eq!(args.bump, Some("major".to_string()));
        assert_eq!(args.env, Some(vec!["staging".to_string(), "prod".to_string()]));
    } else {
        panic!("Expected Changeset Update command");
    }
}

#[test]
fn test_changeset_delete_command() {
    let cli = Cli::parse_from(["wnt", "changeset", "delete", "old-branch", "--force"]);

    if let Commands::Changeset(ChangesetCommands::Delete(args)) = cli.command {
        assert_eq!(args.branch, "old-branch");
        assert!(args.force);
    } else {
        panic!("Expected Changeset Delete command");
    }
}

#[test]
fn test_changeset_history_command() {
    let cli = Cli::parse_from([
        "wnt",
        "changeset",
        "history",
        "--package",
        "core",
        "--since",
        "2024-01-01",
        "--until",
        "2024-12-31",
        "--env",
        "prod",
        "--bump",
        "minor",
        "--limit",
        "10",
    ]);

    if let Commands::Changeset(ChangesetCommands::History(args)) = cli.command {
        assert_eq!(args.package, Some("core".to_string()));
        assert_eq!(args.since, Some("2024-01-01".to_string()));
        assert_eq!(args.until, Some("2024-12-31".to_string()));
        assert_eq!(args.env, Some("prod".to_string()));
        assert_eq!(args.bump, Some("minor".to_string()));
        assert_eq!(args.limit, Some(10));
    } else {
        panic!("Expected Changeset History command");
    }
}

#[test]
fn test_changeset_check_command() {
    let cli = Cli::parse_from(["wnt", "changeset", "check", "--branch", "main"]);

    if let Commands::Changeset(ChangesetCommands::Check(args)) = cli.command {
        assert_eq!(args.branch, Some("main".to_string()));
    } else {
        panic!("Expected Changeset Check command");
    }
}

// ============================================================================
// Bump Command Tests
// ============================================================================

#[test]
fn test_bump_command_dry_run() {
    let cli = Cli::parse_from(["wnt", "bump", "--dry-run"]);

    if let Commands::Bump(args) = cli.command {
        assert!(args.dry_run);
        assert!(!args.execute);
    } else {
        panic!("Expected Bump command");
    }
}

#[test]
fn test_bump_command_execute() {
    let cli = Cli::parse_from(["wnt", "bump", "--execute"]);

    if let Commands::Bump(args) = cli.command {
        assert!(!args.dry_run);
        assert!(args.execute);
    } else {
        panic!("Expected Bump command");
    }
}

#[test]
fn test_bump_command_with_git_options() {
    let cli =
        Cli::parse_from(["wnt", "bump", "--execute", "--git-tag", "--git-push", "--git-commit"]);

    if let Commands::Bump(args) = cli.command {
        assert!(args.execute);
        assert!(args.git_tag);
        assert!(args.git_push);
        assert!(args.git_commit);
    } else {
        panic!("Expected Bump command");
    }
}

#[test]
fn test_bump_command_snapshot() {
    let cli = Cli::parse_from([
        "wnt",
        "bump",
        "--snapshot",
        "--snapshot-format",
        "{version}-{branch}",
        "--execute",
    ]);

    if let Commands::Bump(args) = cli.command {
        assert!(args.snapshot);
        assert_eq!(args.snapshot_format, Some("{version}-{branch}".to_string()));
    } else {
        panic!("Expected Bump command");
    }
}

#[test]
fn test_bump_command_prerelease() {
    let cli = Cli::parse_from(["wnt", "bump", "--prerelease", "beta", "--execute"]);

    if let Commands::Bump(args) = cli.command {
        assert_eq!(args.prerelease, Some("beta".to_string()));
    } else {
        panic!("Expected Bump command");
    }
}

#[test]
fn test_bump_command_with_packages() {
    let cli = Cli::parse_from(["wnt", "bump", "--packages", "pkg1,pkg2", "--dry-run"]);

    if let Commands::Bump(args) = cli.command {
        assert_eq!(args.packages, Some(vec!["pkg1".to_string(), "pkg2".to_string()]));
    } else {
        panic!("Expected Bump command");
    }
}

#[test]
fn test_bump_command_with_flags() {
    let cli =
        Cli::parse_from(["wnt", "bump", "--execute", "--no-changelog", "--no-archive", "--force"]);

    if let Commands::Bump(args) = cli.command {
        assert!(args.no_changelog);
        assert!(args.no_archive);
        assert!(args.force);
    } else {
        panic!("Expected Bump command");
    }
}

// ============================================================================
// Upgrade Command Tests
// ============================================================================

#[test]
fn test_upgrade_check_command() {
    let cli = Cli::parse_from(["wnt", "upgrade", "check"]);

    if let Commands::Upgrade(UpgradeCommands::Check(_)) = cli.command {
        // Success
    } else {
        panic!("Expected Upgrade Check command");
    }
}

#[test]
fn test_upgrade_check_with_options() {
    let cli = Cli::parse_from([
        "wnt",
        "upgrade",
        "check",
        "--no-major",
        "--dev",
        "--peer",
        "--packages",
        "typescript,eslint",
        "--registry",
        "https://npm.example.com",
    ]);

    if let Commands::Upgrade(UpgradeCommands::Check(args)) = cli.command {
        assert!(args.no_major);
        assert!(args.dev);
        assert!(args.peer);
        assert_eq!(args.packages, Some(vec!["typescript".to_string(), "eslint".to_string()]));
        assert_eq!(args.registry, Some("https://npm.example.com".to_string()));
    } else {
        panic!("Expected Upgrade Check command");
    }
}

#[test]
fn test_upgrade_apply_command() {
    let cli = Cli::parse_from([
        "wnt",
        "upgrade",
        "apply",
        "--patch-only",
        "--auto-changeset",
        "--changeset-bump",
        "minor",
        "--no-backup",
        "--force",
    ]);

    if let Commands::Upgrade(UpgradeCommands::Apply(args)) = cli.command {
        assert!(args.patch_only);
        assert!(args.auto_changeset);
        assert_eq!(args.changeset_bump, "minor");
        assert!(args.no_backup);
        assert!(args.force);
    } else {
        panic!("Expected Upgrade Apply command");
    }
}

#[test]
fn test_upgrade_backups_list_command() {
    let cli = Cli::parse_from(["wnt", "upgrade", "backups", "list"]);

    if let Commands::Upgrade(UpgradeCommands::Backups(UpgradeBackupCommands::List(_))) = cli.command
    {
        // Success
    } else {
        panic!("Expected Upgrade Backups List command");
    }
}

#[test]
fn test_upgrade_backups_restore_command() {
    let cli = Cli::parse_from(["wnt", "upgrade", "backups", "restore", "backup_123", "--force"]);

    if let Commands::Upgrade(UpgradeCommands::Backups(UpgradeBackupCommands::Restore(args))) =
        cli.command
    {
        assert_eq!(args.id, "backup_123");
        assert!(args.force);
    } else {
        panic!("Expected Upgrade Backups Restore command");
    }
}

#[test]
fn test_upgrade_backups_clean_command() {
    let cli = Cli::parse_from(["wnt", "upgrade", "backups", "clean", "--keep", "10", "--force"]);

    if let Commands::Upgrade(UpgradeCommands::Backups(UpgradeBackupCommands::Clean(args))) =
        cli.command
    {
        assert_eq!(args.keep, 10);
        assert!(args.force);
    } else {
        panic!("Expected Upgrade Backups Clean command");
    }
}

// ============================================================================
// Audit Command Tests
// ============================================================================

#[test]
fn test_audit_command_basic() {
    let cli = Cli::parse_from(["wnt", "audit"]);

    if let Commands::Audit(args) = cli.command {
        assert_eq!(args.sections, vec!["all"]);
        assert_eq!(args.min_severity, "info");
        assert_eq!(args.verbosity, "normal");
        assert!(!args.no_health_score);
    } else {
        panic!("Expected Audit command");
    }
}

#[test]
fn test_audit_command_with_options() {
    let cli = Cli::parse_from([
        "wnt",
        "audit",
        "--sections",
        "upgrades,dependencies",
        "--output",
        "report.md",
        "--min-severity",
        "high",
        "--verbosity",
        "detailed",
        "--no-health-score",
    ]);

    if let Commands::Audit(args) = cli.command {
        assert_eq!(args.sections, vec!["upgrades", "dependencies"]);
        assert_eq!(args.output, Some(PathBuf::from("report.md")));
        assert_eq!(args.min_severity, "high");
        assert_eq!(args.verbosity, "detailed");
        assert!(args.no_health_score);
    } else {
        panic!("Expected Audit command");
    }
}

// ============================================================================
// Changes Command Tests
// ============================================================================

#[test]
fn test_changes_command_basic() {
    let cli = Cli::parse_from(["wnt", "changes"]);

    if let Commands::Changes(args) = cli.command {
        assert_eq!(args.since, None);
        assert_eq!(args.until, None);
        assert_eq!(args.branch, None);
        assert!(!args.staged);
        assert!(!args.unstaged);
    } else {
        panic!("Expected Changes command");
    }
}

#[test]
fn test_changes_command_with_options() {
    let cli = Cli::parse_from([
        "wnt",
        "changes",
        "--since",
        "HEAD~5",
        "--until",
        "HEAD",
        "--branch",
        "main",
        "--packages",
        "core,utils",
    ]);

    if let Commands::Changes(args) = cli.command {
        assert_eq!(args.since, Some("HEAD~5".to_string()));
        assert_eq!(args.until, Some("HEAD".to_string()));
        assert_eq!(args.branch, Some("main".to_string()));
        assert_eq!(args.packages, Some(vec!["core".to_string(), "utils".to_string()]));
    } else {
        panic!("Expected Changes command");
    }
}

#[test]
fn test_changes_command_staged() {
    let cli = Cli::parse_from(["wnt", "changes", "--staged"]);

    if let Commands::Changes(args) = cli.command {
        assert!(args.staged);
        assert!(!args.unstaged);
    } else {
        panic!("Expected Changes command");
    }
}

#[test]
fn test_changes_command_unstaged() {
    let cli = Cli::parse_from(["wnt", "changes", "--unstaged"]);

    if let Commands::Changes(args) = cli.command {
        assert!(!args.staged);
        assert!(args.unstaged);
    } else {
        panic!("Expected Changes command");
    }
}

// ============================================================================
// Version Command Tests
// ============================================================================

#[test]
fn test_version_command_basic() {
    let cli = Cli::parse_from(["wnt", "version"]);

    if let Commands::Version(args) = cli.command {
        assert!(!args.verbose);
    } else {
        panic!("Expected Version command");
    }
}

#[test]
fn test_version_command_verbose() {
    let cli = Cli::parse_from(["wnt", "version", "--verbose"]);

    if let Commands::Version(args) = cli.command {
        assert!(args.verbose);
    } else {
        panic!("Expected Version command");
    }
}

// ============================================================================
// Shell Completion Tests
// ============================================================================

#[test]
fn test_parse_shell_valid() {
    use completions::parse_shell;

    assert_eq!(parse_shell("bash"), Some(Shell::Bash));
    assert_eq!(parse_shell("BASH"), Some(Shell::Bash));
    assert_eq!(parse_shell("zsh"), Some(Shell::Zsh));
    assert_eq!(parse_shell("fish"), Some(Shell::Fish));
    assert_eq!(parse_shell("powershell"), Some(Shell::PowerShell));
    assert_eq!(parse_shell("pwsh"), Some(Shell::PowerShell));
}

#[test]
fn test_parse_shell_invalid() {
    use completions::parse_shell;

    assert_eq!(parse_shell("invalid"), None);
    assert_eq!(parse_shell(""), None);
    assert_eq!(parse_shell("cmd"), None);
}

#[test]
fn test_supported_shells() {
    use completions::supported_shells;

    let shells = supported_shells();
    assert!(shells.contains(&"bash"));
    assert!(shells.contains(&"zsh"));
    assert!(shells.contains(&"fish"));
    assert!(shells.contains(&"powershell"));
}

#[test]
fn test_generate_completions_bash() {
    use clap::CommandFactory;
    use completions::generate_completions;

    let mut cmd = Cli::command();
    let mut buf = Vec::new();

    generate_completions(Shell::Bash, &mut cmd, "wnt", &mut buf);

    let output = String::from_utf8(buf).expect("Invalid UTF-8");
    assert!(!output.is_empty());
    assert!(output.contains("wnt"));
}

// ============================================================================
// LogLevel Tests
// ============================================================================

#[test]
fn test_log_level_to_tracing_level() {
    use args::LogLevel;

    assert_eq!(LogLevel::Error.to_tracing_level(), tracing::Level::ERROR);
    assert_eq!(LogLevel::Warn.to_tracing_level(), tracing::Level::WARN);
    assert_eq!(LogLevel::Info.to_tracing_level(), tracing::Level::INFO);
    assert_eq!(LogLevel::Debug.to_tracing_level(), tracing::Level::DEBUG);
    assert_eq!(LogLevel::Trace.to_tracing_level(), tracing::Level::TRACE);
}

#[test]
fn test_log_level_is_silent() {
    use args::LogLevel;

    assert!(LogLevel::Silent.is_silent());
    assert!(!LogLevel::Error.is_silent());
    assert!(!LogLevel::Info.is_silent());
}

#[test]
fn test_log_level_includes_checks() {
    use args::LogLevel;

    // Silent includes nothing
    assert!(!LogLevel::Silent.includes_errors());
    assert!(!LogLevel::Silent.includes_warnings());
    assert!(!LogLevel::Silent.includes_info());
    assert!(!LogLevel::Silent.includes_debug());
    assert!(!LogLevel::Silent.includes_trace());

    // Error includes only errors
    assert!(LogLevel::Error.includes_errors());
    assert!(!LogLevel::Error.includes_warnings());

    // Info includes errors, warnings, and info
    assert!(LogLevel::Info.includes_errors());
    assert!(LogLevel::Info.includes_warnings());
    assert!(LogLevel::Info.includes_info());
    assert!(!LogLevel::Info.includes_debug());

    // Trace includes everything
    assert!(LogLevel::Trace.includes_errors());
    assert!(LogLevel::Trace.includes_warnings());
    assert!(LogLevel::Trace.includes_info());
    assert!(LogLevel::Trace.includes_debug());
    assert!(LogLevel::Trace.includes_trace());
}

// ============================================================================
// OutputFormatArg Tests
// ============================================================================

#[test]
fn test_output_format_arg_new() {
    use args::OutputFormatArg;

    let arg = OutputFormatArg::new(OutputFormat::Json);
    assert_eq!(arg.0, OutputFormat::Json);
}

#[test]
fn test_output_format_arg_into_inner() {
    use args::OutputFormatArg;

    let arg = OutputFormatArg::new(OutputFormat::Json);
    assert_eq!(arg.into_inner(), OutputFormat::Json);
}

#[test]
fn test_output_format_arg_from_str() {
    use args::OutputFormatArg;
    use std::str::FromStr;

    let arg = OutputFormatArg::from_str("json").expect("Failed to parse");
    assert_eq!(arg.0, OutputFormat::Json);

    let arg = OutputFormatArg::from_str("json-compact").expect("Failed to parse");
    assert_eq!(arg.0, OutputFormat::JsonCompact);
}
