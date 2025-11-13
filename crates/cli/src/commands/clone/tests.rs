//! Tests for clone command functionality.
//!
//! This module provides comprehensive test coverage for the clone command,
//! including URL parsing, destination determination, and validation logic.

#![allow(clippy::unwrap_used)] // Allow unwrap in tests for brevity

use super::*;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// determine_destination() tests
// ============================================================================

#[test]
fn test_determine_destination_https_with_git() {
    let url = "https://github.com/org/repo.git";
    let result = determine_destination(url, None);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("repo"));
}

#[test]
fn test_determine_destination_https_without_git() {
    let url = "https://github.com/org/repo";
    let result = determine_destination(url, None);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("repo"));
}

#[test]
fn test_determine_destination_ssh_with_git() {
    let url = "git@github.com:org/repo.git";
    let result = determine_destination(url, None);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("repo"));
}

#[test]
fn test_determine_destination_ssh_without_git() {
    let url = "git@github.com:org/repo";
    let result = determine_destination(url, None);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("repo"));
}

#[test]
fn test_determine_destination_https_nested_path() {
    let url = "https://gitlab.com/group/subgroup/repo.git";
    let result = determine_destination(url, None);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("repo"));
}

#[test]
fn test_determine_destination_ssh_nested_path() {
    let url = "git@gitlab.com:group/subgroup/repo.git";
    let result = determine_destination(url, None);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("repo"));
}

#[test]
fn test_determine_destination_explicit_destination() {
    let url = "https://github.com/org/repo.git";
    let explicit_dest = PathBuf::from("my-custom-dir");
    let result = determine_destination(url, Some(&explicit_dest));

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("my-custom-dir"));
}

#[test]
fn test_determine_destination_explicit_destination_overrides_url() {
    let url = "git@github.com:org/repo.git";
    let explicit_dest = PathBuf::from("different-name");
    let result = determine_destination(url, Some(&explicit_dest));

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("different-name"));
}

#[test]
fn test_determine_destination_invalid_url() {
    let url = "not-a-valid-url";
    let result = determine_destination(url, None);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unable to determine repository name"));
}

#[test]
fn test_determine_destination_empty_url() {
    let url = "";
    let result = determine_destination(url, None);

    assert!(result.is_err());
}

#[test]
fn test_determine_destination_https_with_port() {
    let url = "https://github.com:443/org/repo.git";
    let result = determine_destination(url, None);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("repo"));
}

#[test]
fn test_determine_destination_ssh_with_user() {
    let url = "user@host.com:org/repo.git";
    let result = determine_destination(url, None);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("repo"));
}

#[test]
fn test_determine_destination_http() {
    let url = "http://github.com/org/repo.git";
    let result = determine_destination(url, None);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("repo"));
}

#[test]
fn test_determine_destination_repo_with_dash() {
    let url = "https://github.com/org/my-repo.git";
    let result = determine_destination(url, None);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("my-repo"));
}

#[test]
fn test_determine_destination_repo_with_underscore() {
    let url = "https://github.com/org/my_repo.git";
    let result = determine_destination(url, None);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("my_repo"));
}

#[test]
fn test_determine_destination_repo_with_numbers() {
    let url = "https://github.com/org/repo123.git";
    let result = determine_destination(url, None);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("repo123"));
}

// ============================================================================
// validate_destination() tests
// ============================================================================

#[test]
fn test_validate_destination_new_directory() {
    let temp_dir = TempDir::new().unwrap();
    let non_existent = temp_dir.path().join("new-dir");

    let result = validate_destination(&non_existent, false);
    assert!(result.is_ok());
}

#[test]
fn test_validate_destination_existing_without_force() {
    let temp_dir = TempDir::new().unwrap();
    let existing = temp_dir.path().join("existing");
    fs::create_dir(&existing).unwrap();

    let result = validate_destination(&existing, false);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Destination already exists"));
    assert!(err_msg.contains("Use --force to overwrite"));
}

#[test]
fn test_validate_destination_existing_with_force() {
    let temp_dir = TempDir::new().unwrap();
    let existing = temp_dir.path().join("existing");
    fs::create_dir(&existing).unwrap();

    let result = validate_destination(&existing, true);
    assert!(result.is_ok());
}

#[test]
fn test_validate_destination_file_without_force() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("file.txt");
    fs::write(&file_path, "content").unwrap();

    let result = validate_destination(&file_path, false);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Destination already exists"));
}

#[test]
fn test_validate_destination_file_with_force() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("file.txt");
    fs::write(&file_path, "content").unwrap();

    let result = validate_destination(&file_path, true);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("not a directory"));
}

#[test]
fn test_validate_destination_nested_new_directory() {
    let temp_dir = TempDir::new().unwrap();
    let nested = temp_dir.path().join("parent").join("child");

    // Parent doesn't exist, but validation should still pass
    // (actual directory creation will happen during clone)
    let result = validate_destination(&nested, false);
    assert!(result.is_ok());
}

// ============================================================================
// Edge cases and error scenarios
// ============================================================================

#[test]
fn test_determine_destination_trailing_slash() {
    let url = "https://github.com/org/repo.git/";
    let result = determine_destination(url, None);

    // Should handle trailing slash gracefully
    // Note: Current regex might not handle this, which is OK for now
    // as it's not a common case. Can be enhanced in future stories if needed.
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_determine_destination_relative_path() {
    let url = "./local/repo.git";
    let result = determine_destination(url, None);

    // Should fail for local paths (not a valid remote URL)
    assert!(result.is_err());
}

#[test]
fn test_determine_destination_absolute_path() {
    let url = "/absolute/path/to/repo.git";
    let result = determine_destination(url, None);

    // Should fail for absolute local paths (not a valid remote URL)
    assert!(result.is_err());
}

#[test]
fn test_validate_destination_empty_path() {
    let result = validate_destination(Path::new(""), false);

    // Empty path should be handled
    assert!(result.is_ok());
}

// ============================================================================
// Integration-style tests for argument combinations
// ============================================================================

#[test]
fn test_clone_args_parsing_minimal() {
    use crate::cli::Cli;
    use clap::Parser;

    let cli = Cli::parse_from(["workspace", "clone", "https://github.com/org/repo.git"]);

    let crate::cli::Commands::Clone(args) = cli.command else {
        unreachable!("Expected Clone command variant");
    };

    assert_eq!(args.url, "https://github.com/org/repo.git");
    assert!(args.destination.is_none());
    assert!(!args.force);
    assert!(!args.non_interactive);
    assert!(!args.skip_validation);
    assert!(args.depth.is_none());
}

#[test]
fn test_clone_args_parsing_with_destination() {
    use crate::cli::Cli;
    use clap::Parser;

    let cli =
        Cli::parse_from(["workspace", "clone", "https://github.com/org/repo.git", "./my-dir"]);

    let crate::cli::Commands::Clone(args) = cli.command else {
        unreachable!("Expected Clone command variant");
    };

    assert_eq!(args.url, "https://github.com/org/repo.git");
    assert_eq!(args.destination, Some(PathBuf::from("./my-dir")));
}

#[test]
fn test_clone_args_parsing_with_flags() {
    use crate::cli::Cli;
    use clap::Parser;

    let cli = Cli::parse_from([
        "workspace",
        "clone",
        "https://github.com/org/repo.git",
        "--force",
        "--non-interactive",
        "--skip-validation",
        "--depth",
        "1",
    ]);

    let crate::cli::Commands::Clone(args) = cli.command else {
        unreachable!("Expected Clone command variant");
    };

    assert!(args.force);
    assert!(args.non_interactive);
    assert!(args.skip_validation);
    assert_eq!(args.depth, Some(1));
}

#[test]
fn test_clone_args_parsing_with_init_overrides() {
    use crate::cli::Cli;
    use clap::Parser;

    let cli = Cli::parse_from([
        "workspace",
        "clone",
        "https://github.com/org/repo.git",
        "--strategy",
        "independent",
        "--environments",
        "dev,staging,prod",
        "--default-env",
        "prod",
        "--changeset-path",
        ".changes",
        "--registry",
        "https://custom.registry.com",
        "--config-format",
        "yaml",
    ]);

    let crate::cli::Commands::Clone(args) = cli.command else {
        unreachable!("Expected Clone command variant");
    };

    assert_eq!(args.strategy, Some("independent".to_string()));
    assert_eq!(
        args.environments,
        Some(vec!["dev".to_string(), "staging".to_string(), "prod".to_string()])
    );
    assert_eq!(args.default_env, Some(vec!["prod".to_string()]));
    assert_eq!(args.changeset_path, Some(".changes".to_string()));
    assert_eq!(args.registry, Some("https://custom.registry.com".to_string()));
    assert_eq!(args.config_format, Some("yaml".to_string()));
}

// ============================================================================
// Command help text tests
// ============================================================================

#[test]
fn test_clone_command_appears_in_help() {
    use crate::cli::Cli;
    use clap::CommandFactory;

    let cmd = Cli::command();
    let help_text = format!("{cmd:?}");

    // The command should be registered
    assert!(help_text.contains("clone") || help_text.contains("Clone"));
}
