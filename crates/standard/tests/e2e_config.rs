//! # End-to-End Tests for Config Module
//!
//! ## What
//! Comprehensive e2e tests for configuration management including
//! `StandardConfig`, `ConfigManager`, `ConfigFormat`, and configuration parsing.
//!
//! ## How
//! Tests create realistic configuration files in TOML, JSON, and YAML formats
//! and verify parsing, validation, and merging operations.
//!
//! ## Why
//! E2E tests ensure the config module correctly parses and manages configuration
//! across different formats, validates settings, and properly merges configurations.

#![allow(clippy::print_stdout)]

use std::time::Duration;

use sublime_standard_tools::{
    command::DefaultCommandExecutor,
    config::{
        CommandConfig, ConfigFormat, Configurable, FilesystemConfig, MonorepoConfig,
        PackageManagerConfig, StandardConfig, ValidationConfig,
    },
    error::Result,
    filesystem::{AsyncFileSystem, FileSystemManager},
    monorepo::MonorepoDetector,
    node::PackageManagerKind,
};
use tempfile::TempDir;

// ============================================================================
// StandardConfig Creation Tests
// ============================================================================

#[tokio::test]
async fn test_standard_config_default() -> Result<()> {
    let config = StandardConfig::default();

    assert_eq!(config.version, "1.0");
    assert!(!config.package_managers.detection_order.is_empty());
    assert!(config.commands.default_timeout > Duration::ZERO);

    Ok(())
}

#[tokio::test]
async fn test_standard_config_validation() -> Result<()> {
    let config = StandardConfig::default();
    let result = config.validate();

    assert!(result.is_ok());

    Ok(())
}

// ============================================================================
// PackageManagerConfig Tests
// ============================================================================

#[tokio::test]
async fn test_package_manager_config_default() -> Result<()> {
    let config = PackageManagerConfig::default();

    // Default detection order should include common package managers
    assert!(!config.detection_order.is_empty());
    assert!(config.detect_from_env);

    Ok(())
}

#[tokio::test]
async fn test_package_manager_config_with_fallback() -> Result<()> {
    let config =
        PackageManagerConfig { fallback: Some(PackageManagerKind::Npm), ..Default::default() };

    assert_eq!(config.fallback, Some(PackageManagerKind::Npm));

    Ok(())
}

#[tokio::test]
async fn test_package_manager_config_custom_order() -> Result<()> {
    let config = PackageManagerConfig {
        detection_order: vec![
            PackageManagerKind::Pnpm,
            PackageManagerKind::Yarn,
            PackageManagerKind::Npm,
        ],
        ..Default::default()
    };

    assert_eq!(config.detection_order[0], PackageManagerKind::Pnpm);
    assert_eq!(config.detection_order.len(), 3);

    Ok(())
}

// ============================================================================
// MonorepoConfig Tests
// ============================================================================

#[tokio::test]
async fn test_monorepo_config_default() -> Result<()> {
    let config = MonorepoConfig::default();

    // Should have default workspace patterns
    assert!(!config.workspace_patterns.is_empty());
    assert!(!config.exclude_patterns.is_empty());
    assert!(config.max_search_depth > 0);

    Ok(())
}

#[tokio::test]
async fn test_monorepo_config_custom_patterns() -> Result<()> {
    let config = MonorepoConfig {
        workspace_patterns: vec![
            "packages/*".to_string(),
            "apps/*".to_string(),
            "tools/*".to_string(),
        ],
        ..Default::default()
    };

    assert_eq!(config.workspace_patterns.len(), 3);
    assert!(config.workspace_patterns.contains(&"packages/*".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_monorepo_config_exclude_patterns() -> Result<()> {
    let config = MonorepoConfig::default();

    // Default should exclude common directories
    assert!(config.exclude_patterns.contains(&"node_modules".to_string()));

    Ok(())
}

// ============================================================================
// CommandConfig Tests
// ============================================================================

#[tokio::test]
async fn test_command_config_default() -> Result<()> {
    let config = CommandConfig::default();

    assert!(config.max_concurrent_commands > 0);
    assert!(config.queue_collection_window_ms > 0);

    Ok(())
}

#[tokio::test]
async fn test_command_config_custom() -> Result<()> {
    let config = CommandConfig {
        max_concurrent_commands: 8,
        queue_collection_window_ms: 100,
        queue_collection_sleep_us: 500,
        ..Default::default()
    };

    assert_eq!(config.max_concurrent_commands, 8);
    assert_eq!(config.queue_collection_window_ms, 100);
    assert_eq!(config.queue_collection_sleep_us, 500);

    Ok(())
}

// ============================================================================
// FilesystemConfig Tests
// ============================================================================

#[tokio::test]
async fn test_filesystem_config_default() -> Result<()> {
    let config = FilesystemConfig::default();

    // Default ignore patterns should be set
    assert!(!config.ignore_patterns.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_filesystem_config_ignore_patterns() -> Result<()> {
    let config = FilesystemConfig::default();

    // Should have common ignore patterns
    assert!(config.ignore_patterns.contains(&"node_modules".to_string()));
    assert!(config.ignore_patterns.contains(&".git".to_string()));

    Ok(())
}

// ============================================================================
// ValidationConfig Tests
// ============================================================================

#[tokio::test]
async fn test_validation_config_default() -> Result<()> {
    let config = ValidationConfig::default();

    // Default validation settings
    assert!(config.require_package_json);

    Ok(())
}

// ============================================================================
// ConfigFormat Tests
// ============================================================================

#[tokio::test]
async fn test_config_format_from_extension() -> Result<()> {
    assert_eq!(ConfigFormat::from_extension("toml"), Some(ConfigFormat::Toml));
    assert_eq!(ConfigFormat::from_extension("json"), Some(ConfigFormat::Json));
    assert_eq!(ConfigFormat::from_extension("yaml"), Some(ConfigFormat::Yaml));
    assert_eq!(ConfigFormat::from_extension("yml"), Some(ConfigFormat::Yaml));
    assert_eq!(ConfigFormat::from_extension("unknown"), None);

    Ok(())
}

#[tokio::test]
async fn test_config_format_extension() -> Result<()> {
    assert_eq!(ConfigFormat::Toml.extension(), "toml");
    assert_eq!(ConfigFormat::Json.extension(), "json");
    assert_eq!(ConfigFormat::Yaml.extension(), "yaml");

    Ok(())
}

// ============================================================================
// TOML Configuration Parsing Tests
// ============================================================================

#[tokio::test]
async fn test_parse_toml_config() -> Result<()> {
    let toml_content = r#"
version = "1.0"

[package_managers]
detection_order = ["Pnpm", "Yarn", "Npm"]
detect_from_env = true

[monorepo]
max_search_depth = 8
workspace_patterns = ["packages/*", "apps/*"]

[commands]
max_concurrent_commands = 6

[filesystem]
ignore_patterns = ["node_modules", ".git", "dist"]

[validation]
strict_mode = true
require_package_json = true
"#;

    let config: StandardConfig = toml::from_str(toml_content).map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("TOML parse error: {e}"))
    })?;

    assert_eq!(config.version, "1.0");
    assert_eq!(config.package_managers.detection_order.len(), 3);
    assert_eq!(config.monorepo.max_search_depth, 8);
    assert_eq!(config.commands.max_concurrent_commands, 6);
    assert!(config.filesystem.ignore_patterns.contains(&"dist".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_parse_toml_partial_config() -> Result<()> {
    // Only specifying some fields - others should use defaults
    let toml_content = r#"
version = "1.0"

[monorepo]
max_search_depth = 10
"#;

    let config: StandardConfig = toml::from_str(toml_content).map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("TOML parse error: {e}"))
    })?;

    // Specified value
    assert_eq!(config.monorepo.max_search_depth, 10);

    // Default values for non-specified fields
    assert!(!config.package_managers.detection_order.is_empty());

    Ok(())
}

// ============================================================================
// JSON Configuration Parsing Tests
// ============================================================================

#[tokio::test]
async fn test_parse_json_config() -> Result<()> {
    let json_content = r#"{
        "version": "1.0",
        "package_managers": {
            "detection_order": ["Npm", "Yarn"],
            "detect_from_env": false
        },
        "monorepo": {
            "max_search_depth": 5,
            "workspace_patterns": ["packages/*"]
        },
        "commands": {
            "max_concurrent_commands": 4
        }
    }"#;

    let config: StandardConfig = serde_json::from_str(json_content).map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("JSON parse error: {e}"))
    })?;

    assert_eq!(config.version, "1.0");
    assert_eq!(config.package_managers.detection_order.len(), 2);
    assert!(!config.package_managers.detect_from_env);
    assert_eq!(config.monorepo.max_search_depth, 5);
    assert_eq!(config.commands.max_concurrent_commands, 4);

    Ok(())
}

// ============================================================================
// YAML Configuration Parsing Tests
// ============================================================================

#[tokio::test]
async fn test_parse_yaml_config() -> Result<()> {
    let yaml_content = r#"
version: "1.0"

package_managers:
  detection_order:
    - Pnpm
    - Yarn
    - Npm
  detect_from_env: true

monorepo:
  max_search_depth: 6
  workspace_patterns:
    - "packages/*"
    - "apps/*"
    - "libs/*"

commands:
  max_concurrent_commands: 8
"#;

    let config: StandardConfig = serde_yaml::from_str(yaml_content).map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("YAML parse error: {e}"))
    })?;

    assert_eq!(config.version, "1.0");
    assert_eq!(config.package_managers.detection_order.len(), 3);
    assert_eq!(config.monorepo.max_search_depth, 6);
    assert_eq!(config.monorepo.workspace_patterns.len(), 3);
    assert_eq!(config.commands.max_concurrent_commands, 8);

    Ok(())
}

// ============================================================================
// Configuration File Loading Tests
// ============================================================================

#[tokio::test]
async fn test_load_config_from_toml_file() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let config_content = r#"
version = "1.0"

[monorepo]
max_search_depth = 7
workspace_patterns = ["packages/*"]
"#;

    let config_path = temp_dir.path().join("config.toml");
    fs.write_file_string(&config_path, config_content).await?;

    // Read and parse
    let content = fs.read_file_string(&config_path).await?;
    let config: StandardConfig = toml::from_str(&content).map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("TOML parse error: {e}"))
    })?;

    assert_eq!(config.monorepo.max_search_depth, 7);

    Ok(())
}

#[tokio::test]
async fn test_load_config_from_json_file() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let config_content = r#"{
        "version": "1.0",
        "monorepo": {
            "max_search_depth": 9
        }
    }"#;

    let config_path = temp_dir.path().join("config.json");
    fs.write_file_string(&config_path, config_content).await?;

    // Read and parse
    let content = fs.read_file_string(&config_path).await?;
    let config: StandardConfig = serde_json::from_str(&content).map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("JSON parse error: {e}"))
    })?;

    assert_eq!(config.monorepo.max_search_depth, 9);

    Ok(())
}

// ============================================================================
// Configuration Serialization Tests
// ============================================================================

#[tokio::test]
async fn test_serialize_config_to_toml() -> Result<()> {
    let config = StandardConfig::default();

    let toml_str = toml::to_string_pretty(&config).map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("TOML serialize error: {e}"))
    })?;

    // Verify it's valid TOML by parsing it back
    let _: StandardConfig = toml::from_str(&toml_str).map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("TOML parse error: {e}"))
    })?;

    assert!(toml_str.contains("version"));

    Ok(())
}

#[tokio::test]
async fn test_serialize_config_to_json() -> Result<()> {
    let config = StandardConfig::default();

    let json_str = serde_json::to_string_pretty(&config).map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("JSON serialize error: {e}"))
    })?;

    // Verify it's valid JSON by parsing it back
    let _: StandardConfig = serde_json::from_str(&json_str).map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("JSON parse error: {e}"))
    })?;

    assert!(json_str.contains("version"));

    Ok(())
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_config_roundtrip_toml() -> Result<()> {
    let mut original = StandardConfig::default();
    original.monorepo.max_search_depth = 15;
    original.commands.max_concurrent_commands = 12;

    // Serialize to TOML
    let toml_str = toml::to_string_pretty(&original).map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("TOML serialize error: {e}"))
    })?;

    // Parse back
    let parsed: StandardConfig = toml::from_str(&toml_str).map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("TOML parse error: {e}"))
    })?;

    assert_eq!(parsed.monorepo.max_search_depth, 15);
    assert_eq!(parsed.commands.max_concurrent_commands, 12);

    Ok(())
}

#[tokio::test]
async fn test_config_roundtrip_json() -> Result<()> {
    let mut original = StandardConfig::default();
    original.monorepo.workspace_patterns = vec!["packages/*".to_string(), "apps/*".to_string()];

    // Serialize to JSON
    let json_str = serde_json::to_string_pretty(&original).map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("JSON serialize error: {e}"))
    })?;

    // Parse back
    let parsed: StandardConfig = serde_json::from_str(&json_str).map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("JSON parse error: {e}"))
    })?;

    assert_eq!(parsed.monorepo.workspace_patterns.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_config_with_components() -> Result<()> {
    let config = StandardConfig::default();

    // Use config with FileSystemManager
    let fs = FileSystemManager::with_standard_config(&config.filesystem);
    assert!(fs.config().operation_timeout > Duration::ZERO);

    // Use config with MonorepoDetector
    let _detector = MonorepoDetector::new_with_config(config.monorepo.clone());

    // Use config with CommandExecutor
    let _executor = DefaultCommandExecutor::new_with_config(config.commands.clone());

    Ok(())
}

#[tokio::test]
async fn test_realistic_project_config() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create a realistic project configuration file
    let config_content = r#"
# Project configuration for my-monorepo
version = "1.0"

[package_managers]
detection_order = ["Pnpm", "Yarn", "Npm"]
detect_from_env = true
fallback = "Npm"

[monorepo]
max_search_depth = 6
workspace_patterns = [
    "packages/*",
    "apps/*",
    "tools/*",
    "shared/*"
]
package_directories = [
    "packages",
    "apps",
    "tools",
    "shared"
]
exclude_patterns = [
    "node_modules",
    ".git",
    "dist",
    "build",
    ".next",
    "coverage"
]

[commands]
max_concurrent_commands = 8
queue_collection_window_ms = 50
queue_collection_sleep_us = 100

[filesystem]
ignore_patterns = [
    "node_modules",
    ".git",
    "dist",
    "build",
    ".DS_Store"
]

[validation]
strict_mode = false
require_package_json = true
validate_dependencies = true
"#;

    let config_path = temp_dir.path().join("sublime.config.toml");
    fs.write_file_string(&config_path, config_content).await?;

    // Load and validate the configuration
    let content = fs.read_file_string(&config_path).await?;
    let config: StandardConfig = toml::from_str(&content).map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("TOML parse error: {e}"))
    })?;

    // Verify configuration values
    assert_eq!(config.version, "1.0");
    assert_eq!(config.package_managers.detection_order[0], PackageManagerKind::Pnpm);
    assert_eq!(config.package_managers.fallback, Some(PackageManagerKind::Npm));
    assert_eq!(config.monorepo.max_search_depth, 6);
    assert_eq!(config.monorepo.workspace_patterns.len(), 4);
    assert_eq!(config.commands.max_concurrent_commands, 8);
    assert!(config.filesystem.ignore_patterns.contains(&"dist".to_string()));

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates a temporary directory for testing
fn create_temp_dir() -> Result<TempDir> {
    TempDir::new().map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("Failed to create temp dir: {e}"))
    })
}
