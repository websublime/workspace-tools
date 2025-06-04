//! Tests for error handling

use sublime_monorepo_tools::error::Error;
use sublime_git_tools::RepoError;
use sublime_standard_tools::error::Error as StandardError;

#[test]
fn test_error_from_git() {
    let git_error = RepoError::CanonicalPathFailure(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Path not found",
    ));
    
    let error: Error = git_error.into();
    assert!(matches!(error, Error::Git(_)));
    assert!(error.to_string().contains("Git operation failed"));
}

#[test]
fn test_error_from_standard() {
    let std_error = StandardError::operation("Test operation failed");
    
    let error: Error = std_error.into();
    assert!(matches!(error, Error::Standard(_)));
    assert!(error.to_string().contains("Standard tools error"));
}

#[test]
fn test_error_constructors() {
    let config_error = Error::config("Invalid configuration");
    assert!(matches!(config_error, Error::Config(_)));
    assert!(config_error.to_string().contains("Invalid configuration"));
    
    let analysis_error = Error::analysis("Analysis failed");
    assert!(matches!(analysis_error, Error::Analysis(_)));
    assert!(analysis_error.to_string().contains("Analysis failed"));
    
    let versioning_error = Error::versioning("Version conflict");
    assert!(matches!(versioning_error, Error::Versioning(_)));
    assert!(versioning_error.to_string().contains("Version conflict"));
    
    let task_error = Error::task("Task execution failed");
    assert!(matches!(task_error, Error::Task(_)));
    assert!(task_error.to_string().contains("Task execution failed"));
    
    let changeset_error = Error::changeset("Changeset not found");
    assert!(matches!(changeset_error, Error::Changeset(_)));
    assert!(changeset_error.to_string().contains("Changeset not found"));
    
    let hook_error = Error::hook("Hook failed");
    assert!(matches!(hook_error, Error::Hook(_)));
    assert!(hook_error.to_string().contains("Hook failed"));
    
    let changelog_error = Error::changelog("Changelog generation failed");
    assert!(matches!(changelog_error, Error::Changelog(_)));
    assert!(changelog_error.to_string().contains("Changelog generation failed"));
    
    let plugin_error = Error::plugin("Plugin error");
    assert!(matches!(plugin_error, Error::Plugin(_)));
    assert!(plugin_error.to_string().contains("Plugin error"));
    
    let workflow_error = Error::workflow("Workflow failed");
    assert!(matches!(workflow_error, Error::Workflow(_)));
    assert!(workflow_error.to_string().contains("Workflow failed"));
    
    let project_init_error = Error::project_init("Project initialization failed");
    assert!(matches!(project_init_error, Error::ProjectInit(_)));
    assert!(project_init_error.to_string().contains("Project initialization failed"));
    
    let generic_error = Error::generic("Generic error");
    assert!(matches!(generic_error, Error::Generic(_)));
    assert_eq!(generic_error.to_string(), "Generic error");
}

#[test]
fn test_io_error_conversion() {
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let error: Error = io_error.into();
    
    assert!(matches!(error, Error::Io(_)));
    assert!(error.to_string().contains("IO error"));
    assert!(error.to_string().contains("Access denied"));
}

#[test]
fn test_json_error_conversion() {
    let json_str = r#"{"invalid": json,}"#;
    let json_error = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
    let error: Error = json_error.into();
    
    assert!(matches!(error, Error::Json(_)));
    assert!(error.to_string().contains("JSON error"));
}

#[test]
fn test_error_chain() {
    use std::error::Error as StdError;
    
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let error: Error = io_error.into();
    
    // Test that we can access the source error
    assert!(error.source().is_some());
    
    let config_error = Error::config("Configuration loading failed");
    // Config errors don't have a source by default
    assert!(config_error.source().is_none());
}