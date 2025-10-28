//! Comprehensive tests for the error handling module.
//!
//! This module provides extensive test coverage for all error types,
//! conversions, exit codes, and display functionality.

#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use super::*;
use std::io;

#[test]
fn test_error_creation_helpers() {
    let error = CliError::configuration("test");
    assert_eq!(error.exit_code(), ExitCode::CONFIG);
    assert_eq!(error.kind(), "Configuration");
    assert_eq!(error.as_ref(), "CliError::Configuration");

    let error = CliError::validation("test");
    assert_eq!(error.exit_code(), ExitCode::DATAERR);
    assert_eq!(error.kind(), "Validation");
    assert_eq!(error.as_ref(), "CliError::Validation");

    let error = CliError::execution("test");
    assert_eq!(error.exit_code(), ExitCode::SOFTWARE);
    assert_eq!(error.kind(), "Execution");
    assert_eq!(error.as_ref(), "CliError::Execution");

    let error = CliError::git("test");
    assert_eq!(error.exit_code(), ExitCode::SOFTWARE);
    assert_eq!(error.kind(), "Git");
    assert_eq!(error.as_ref(), "CliError::Git");

    let error = CliError::package("test");
    assert_eq!(error.exit_code(), ExitCode::DATAERR);
    assert_eq!(error.kind(), "Package");
    assert_eq!(error.as_ref(), "CliError::Package");

    let error = CliError::io("test");
    assert_eq!(error.exit_code(), ExitCode::IOERR);
    assert_eq!(error.kind(), "Io");
    assert_eq!(error.as_ref(), "CliError::Io");

    let error = CliError::network("test");
    assert_eq!(error.exit_code(), ExitCode::UNAVAILABLE);
    assert_eq!(error.kind(), "Network");
    assert_eq!(error.as_ref(), "CliError::Network");

    let error = CliError::user("test");
    assert_eq!(error.exit_code(), ExitCode::USAGE);
    assert_eq!(error.kind(), "User");
    assert_eq!(error.as_ref(), "CliError::User");
}

#[test]
fn test_exit_code_mapping() {
    assert_eq!(CliError::configuration("test").exit_code(), 78);
    assert_eq!(CliError::validation("test").exit_code(), 65);
    assert_eq!(CliError::execution("test").exit_code(), 70);
    assert_eq!(CliError::git("test").exit_code(), 70);
    assert_eq!(CliError::package("test").exit_code(), 65);
    assert_eq!(CliError::io("test").exit_code(), 74);
    assert_eq!(CliError::network("test").exit_code(), 69);
    assert_eq!(CliError::user("test").exit_code(), 64);
}

#[test]
fn test_user_message_formatting() {
    let error = CliError::configuration("config not found");
    assert_eq!(error.user_message(), "Configuration error: config not found");

    let error = CliError::validation("invalid format");
    assert_eq!(error.user_message(), "Validation error: invalid format");

    let error = CliError::execution("command failed");
    assert_eq!(error.user_message(), "Execution error: command failed");

    let error = CliError::git("repo not found");
    assert_eq!(error.user_message(), "Git error: repo not found");

    let error = CliError::package("invalid package.json");
    assert_eq!(error.user_message(), "Package error: invalid package.json");

    let error = CliError::io("file not found");
    assert_eq!(error.user_message(), "I/O error: file not found");

    let error = CliError::network("registry down");
    assert_eq!(error.user_message(), "Network error: registry down");

    let error = CliError::user("cancelled");
    assert_eq!(error.user_message(), "Error: cancelled");
}

#[test]
fn test_display_implementation() {
    let error = CliError::configuration("test error");
    let display = format!("{error}");
    assert!(display.contains("Configuration error"));
    assert!(display.contains("test error"));

    let error = CliError::git("repository error");
    let display = format!("{error}");
    assert!(display.contains("Git error"));
    assert!(display.contains("repository error"));
}

#[test]
fn test_as_ref_implementation() {
    let error = CliError::configuration("test");
    assert_eq!(error.as_ref(), "CliError::Configuration");

    let error = CliError::validation("test");
    assert_eq!(error.as_ref(), "CliError::Validation");

    let error = CliError::execution("test");
    assert_eq!(error.as_ref(), "CliError::Execution");

    let error = CliError::git("test");
    assert_eq!(error.as_ref(), "CliError::Git");

    let error = CliError::package("test");
    assert_eq!(error.as_ref(), "CliError::Package");

    let error = CliError::io("test");
    assert_eq!(error.as_ref(), "CliError::Io");

    let error = CliError::network("test");
    assert_eq!(error.as_ref(), "CliError::Network");

    let error = CliError::user("test");
    assert_eq!(error.as_ref(), "CliError::User");
}

#[test]
fn test_from_io_error() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let cli_error: CliError = io_error.into();

    assert_eq!(cli_error.exit_code(), ExitCode::IOERR);
    assert!(cli_error.user_message().contains("I/O error"));
}

#[test]
fn test_from_serde_json_error() {
    let json = r#"{"invalid": json}"#;
    let result: std::result::Result<serde_json::Value, serde_json::Error> =
        serde_json::from_str(json);

    if let Err(json_error) = result {
        let cli_error: CliError = json_error.into();
        assert_eq!(cli_error.exit_code(), ExitCode::SOFTWARE);
        assert!(cli_error.user_message().contains("Execution error"));
        assert!(cli_error.user_message().contains("JSON error"));
    }
}

#[test]
fn test_from_toml_error() {
    let toml = "invalid = toml = content";
    let result: std::result::Result<toml::Value, toml::de::Error> = toml::from_str(toml);

    assert!(result.is_err());
    let toml_error = result.err().unwrap_or_else(|| panic!("Expected Err"));
    let cli_error: CliError = toml_error.into();
    assert_eq!(cli_error.exit_code(), ExitCode::CONFIG);
    assert!(cli_error.user_message().contains("Configuration error"));
    assert!(cli_error.user_message().contains("TOML parsing error"));
}

#[test]
fn test_from_yaml_error() {
    let yaml = "invalid: yaml: content: here:";
    let result: std::result::Result<serde_yaml::Value, serde_yaml::Error> =
        serde_yaml::from_str(yaml);

    if let Err(yaml_error) = result {
        let cli_error: CliError = yaml_error.into();
        assert_eq!(cli_error.exit_code(), ExitCode::CONFIG);
        assert!(cli_error.user_message().contains("Configuration error"));
        assert!(cli_error.user_message().contains("YAML parsing error"));
    }
}

#[test]
fn test_from_anyhow_error() {
    let anyhow_error = anyhow::anyhow!("test error");
    let cli_error: CliError = anyhow_error.into();

    assert_eq!(cli_error.exit_code(), ExitCode::SOFTWARE);
    assert!(cli_error.user_message().contains("Execution error"));
}

#[test]
fn test_error_trait_implementation() {
    let error = CliError::configuration("test error");
    let error_trait: &dyn std::error::Error = &error;

    assert!(format!("{error_trait}").contains("Configuration error"));
}

#[test]
fn test_debug_implementation() {
    let error = CliError::configuration("test error");
    let debug = format!("{error:?}");

    assert!(debug.contains("Configuration"));
    assert!(debug.contains("test error"));
}

#[test]
fn test_result_type_alias() {
    let result: Result<String> = Ok("success".to_string());
    assert!(result.is_ok());
    if let Ok(value) = result {
        assert_eq!(value, "success");
    }
}

#[test]
fn test_result_type_with_error() {
    let result: Result<String> = Err(CliError::configuration("test error"));
    assert!(result.is_err());

    if let Err(error) = result {
        assert_eq!(error.exit_code(), ExitCode::CONFIG);
    }
}

#[test]
fn test_kind_method() {
    assert_eq!(CliError::configuration("test").kind(), "Configuration");
    assert_eq!(CliError::validation("test").kind(), "Validation");
    assert_eq!(CliError::execution("test").kind(), "Execution");
    assert_eq!(CliError::git("test").kind(), "Git");
    assert_eq!(CliError::package("test").kind(), "Package");
    assert_eq!(CliError::io("test").kind(), "Io");
    assert_eq!(CliError::network("test").kind(), "Network");
    assert_eq!(CliError::user("test").kind(), "User");
}

#[test]
fn test_error_with_empty_message() {
    let error = CliError::configuration("");
    assert_eq!(error.user_message(), "Configuration error: ");
    assert_eq!(error.exit_code(), ExitCode::CONFIG);
}

#[test]
fn test_error_with_long_message() {
    let long_msg = "a".repeat(1000);
    let error = CliError::configuration(&long_msg);
    assert!(error.user_message().contains(&long_msg));
    assert_eq!(error.exit_code(), ExitCode::CONFIG);
}

#[test]
fn test_error_with_special_characters() {
    let special_msg = "Error with special chars: \n\t\"quotes\" and 'apostrophes'";
    let error = CliError::configuration(special_msg);
    assert!(error.user_message().contains(special_msg));
}

#[test]
fn test_error_with_unicode() {
    let unicode_msg = "Error with unicode: 你好 🎉 مرحبا";
    let error = CliError::configuration(unicode_msg);
    assert!(error.user_message().contains(unicode_msg));
}

#[test]
fn test_multiple_error_conversions() {
    // Test that we can convert multiple errors in sequence
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
    let cli_error1: CliError = io_error.into();

    let json_error = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
    let cli_error2: CliError = json_error.into();

    assert_eq!(cli_error1.exit_code(), ExitCode::IOERR);
    assert_eq!(cli_error2.exit_code(), ExitCode::SOFTWARE);
}

#[test]
fn test_error_chaining() {
    fn level3() -> Result<()> {
        Err(CliError::io("file not found"))
    }

    fn level2() -> Result<()> {
        level3()?;
        Ok(())
    }

    fn level1() -> Result<()> {
        level2()?;
        Ok(())
    }

    let result = level1();
    assert!(result.is_err(), "Expected error but got Ok");
    if let Err(error) = result {
        assert_eq!(error.exit_code(), ExitCode::IOERR);
        assert!(error.user_message().contains("file not found"));
    }
}

#[test]
fn test_error_into_string() {
    let error = CliError::configuration("test");
    let into_conversion: String = error.as_ref().to_string();
    assert_eq!(into_conversion, "CliError::Configuration");
}

#[test]
fn test_all_error_variants_have_unique_as_ref() {
    let errors = &[
        CliError::configuration("test"),
        CliError::validation("test"),
        CliError::execution("test"),
        CliError::git("test"),
        CliError::package("test"),
        CliError::io("test"),
        CliError::network("test"),
        CliError::user("test"),
    ];

    let refs: Vec<&str> = errors.iter().map(AsRef::as_ref).collect();

    // Check all are unique
    for (i, ref1) in refs.iter().enumerate() {
        for (j, ref2) in refs.iter().enumerate() {
            if i != j {
                assert_ne!(ref1, ref2, "Error refs should be unique");
            }
        }
    }
}

#[test]
fn test_all_error_variants_have_unique_kinds() {
    let errors = &[
        CliError::configuration("test"),
        CliError::validation("test"),
        CliError::execution("test"),
        CliError::git("test"),
        CliError::package("test"),
        CliError::io("test"),
        CliError::network("test"),
        CliError::user("test"),
    ];

    let kinds: Vec<&str> = errors.iter().map(CliError::kind).collect();

    // Check all are unique
    for (i, kind1) in kinds.iter().enumerate() {
        for (j, kind2) in kinds.iter().enumerate() {
            if i != j {
                assert_ne!(kind1, kind2, "Error kinds should be unique");
            }
        }
    }
}

#[test]
fn test_exit_codes_are_valid_sysexits() {
    let errors = vec![
        CliError::configuration("test"),
        CliError::validation("test"),
        CliError::execution("test"),
        CliError::git("test"),
        CliError::package("test"),
        CliError::io("test"),
        CliError::network("test"),
        CliError::user("test"),
    ];

    let valid_codes = [
        ExitCode::OK,
        ExitCode::USAGE,
        ExitCode::DATAERR,
        ExitCode::NOINPUT,
        ExitCode::NOUSER,
        ExitCode::NOHOST,
        ExitCode::UNAVAILABLE,
        ExitCode::SOFTWARE,
        ExitCode::OSERR,
        ExitCode::OSFILE,
        ExitCode::CANTCREAT,
        ExitCode::IOERR,
        ExitCode::TEMPFAIL,
        ExitCode::PROTOCOL,
        ExitCode::NOPERM,
        ExitCode::CONFIG,
    ];

    for error in errors {
        let code = error.exit_code();
        assert!(valid_codes.contains(&code), "Exit code {code} should be a valid sysexits code");
    }
}

#[test]
fn test_error_message_consistency() {
    // Ensure user_message and Display trait show the same content
    let error = CliError::configuration("test error");
    let user_msg = error.user_message();
    let display_msg = format!("{error}");

    assert_eq!(user_msg, display_msg);
}

#[test]
fn test_helper_methods_use_into() {
    // Test that helper methods accept various string types
    let string = String::from("test");
    let str_ref = "test";
    let owned = "test".to_string();

    let _e1 = CliError::configuration(string);
    let _e2 = CliError::validation(str_ref);
    let _e3 = CliError::execution(owned);
    let _e4 = CliError::git("test");
    let _e5 = CliError::package("test");
    let _e6 = CliError::io("test");
    let _e7 = CliError::network("test");
    let _e8 = CliError::user("test");
}

// Exit code tests
#[test]
fn test_exit_code_values() {
    assert_eq!(ExitCode::OK, 0);
    assert_eq!(ExitCode::USAGE, 64);
    assert_eq!(ExitCode::DATAERR, 65);
    assert_eq!(ExitCode::NOINPUT, 66);
    assert_eq!(ExitCode::NOUSER, 67);
    assert_eq!(ExitCode::NOHOST, 68);
    assert_eq!(ExitCode::UNAVAILABLE, 69);
    assert_eq!(ExitCode::SOFTWARE, 70);
    assert_eq!(ExitCode::OSERR, 71);
    assert_eq!(ExitCode::OSFILE, 72);
    assert_eq!(ExitCode::CANTCREAT, 73);
    assert_eq!(ExitCode::IOERR, 74);
    assert_eq!(ExitCode::TEMPFAIL, 75);
    assert_eq!(ExitCode::PROTOCOL, 76);
    assert_eq!(ExitCode::NOPERM, 77);
    assert_eq!(ExitCode::CONFIG, 78);
}

#[test]
fn test_exit_code_description() {
    assert_eq!(ExitCode::description(0), "Success");
    assert_eq!(ExitCode::description(64), "Command line usage error");
    assert_eq!(ExitCode::description(78), "Configuration error");
    assert_eq!(ExitCode::description(999), "Unknown exit code");
}

#[test]
fn test_exit_code_is_success() {
    assert!(ExitCode::is_success(0));
    assert!(!ExitCode::is_success(64));
    assert!(!ExitCode::is_success(78));
}

#[test]
fn test_exit_code_is_error() {
    assert!(!ExitCode::is_error(0));
    assert!(ExitCode::is_error(64));
    assert!(ExitCode::is_error(78));
}
