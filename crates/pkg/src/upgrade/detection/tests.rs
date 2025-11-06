//! Tests for upgrade detection functionality.
//!
//! **What**: Comprehensive test suite for upgrade detection, including unit tests for
//! individual functions and integration tests for the complete detection workflow.
//!
//! **How**: Uses mock registries, temporary filesystems, and test fixtures to verify
//! detection logic, filtering, concurrency, and error handling.
//!
//! **Why**: To ensure upgrade detection works correctly across various scenarios including
//! different dependency types, version specs, filtering options, and edge cases.

#![allow(clippy::unwrap_used)]
#![allow(clippy::field_reassign_with_default)]

use super::detector::{
    extract_dependencies, extract_version_from_spec, find_latest_prerelease, find_latest_version,
    find_package_json_files, is_internal_dependency, read_package_json,
};
use super::*;
use crate::error::UpgradeError;
use crate::types::DependencyType;
use crate::upgrade::registry::UpgradeType;
use chrono::Utc;
use package_json::PackageJson;
use std::collections::HashMap;
use std::path::Path;
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use tempfile::TempDir;

/// Helper to create a test package.json file
async fn create_test_package_json(
    fs: &FileSystemManager,
    path: &Path,
    name: &str,
    version: &str,
    deps: Option<HashMap<String, String>>,
    dev_deps: Option<HashMap<String, String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut package_json = PackageJson::default();
    package_json.name = name.to_string();
    package_json.version = version.to_string();
    package_json.dependencies = deps;
    package_json.dev_dependencies = dev_deps;

    let content = serde_json::to_string_pretty(&package_json)?;
    fs.write_file_string(path, &content).await?;
    Ok(())
}

#[tokio::test]
async fn test_detection_options_defaults() {
    let options = DetectionOptions::default();
    assert!(!options.include_dependencies);
    assert!(!options.include_dev_dependencies);
    assert!(!options.include_peer_dependencies);
    assert!(!options.include_optional_dependencies);
    assert_eq!(options.concurrency, 10);
    assert!(!options.include_prereleases);
    assert!(options.package_filter.is_none());
    assert!(options.dependency_filter.is_none());
}

#[tokio::test]
async fn test_detection_options_all() {
    let options = DetectionOptions::all();
    assert!(options.include_dependencies);
    assert!(options.include_dev_dependencies);
    assert!(options.include_peer_dependencies);
    assert!(options.include_optional_dependencies);
    assert_eq!(options.concurrency, 10);
}

#[tokio::test]
async fn test_detection_options_production_only() {
    let options = DetectionOptions::production_only();
    assert!(options.include_dependencies);
    assert!(!options.include_dev_dependencies);
    assert!(!options.include_peer_dependencies);
    assert!(!options.include_optional_dependencies);
}

#[tokio::test]
async fn test_detection_options_dev_only() {
    let options = DetectionOptions::dev_only();
    assert!(!options.include_dependencies);
    assert!(options.include_dev_dependencies);
    assert!(!options.include_peer_dependencies);
    assert!(!options.include_optional_dependencies);
}

#[test]
fn test_is_internal_dependency() {
    // Workspace protocol
    assert!(is_internal_dependency("workspace:*"));
    assert!(is_internal_dependency("workspace:^1.0.0"));
    assert!(is_internal_dependency("workspace:~"));

    // File protocol
    assert!(is_internal_dependency("file:../local-package"));
    assert!(is_internal_dependency("file:./packages/core"));

    // Link protocol
    assert!(is_internal_dependency("link:../local-package"));
    assert!(is_internal_dependency("link:./packages/utils"));

    // Portal protocol
    assert!(is_internal_dependency("portal:../local-package"));
    assert!(is_internal_dependency("portal:./packages/shared"));

    // External dependencies
    assert!(!is_internal_dependency("^1.2.3"));
    assert!(!is_internal_dependency("~2.0.0"));
    assert!(!is_internal_dependency("1.0.0"));
    assert!(!is_internal_dependency(">=1.0.0"));
    assert!(!is_internal_dependency("latest"));
}

#[test]
fn test_extract_version_from_spec() {
    // Caret range
    assert_eq!(extract_version_from_spec("^1.2.3").unwrap(), "1.2.3");
    assert_eq!(extract_version_from_spec("^0.0.1").unwrap(), "0.0.1");

    // Tilde range
    assert_eq!(extract_version_from_spec("~2.0.0").unwrap(), "2.0.0");
    assert_eq!(extract_version_from_spec("~1.5.9").unwrap(), "1.5.9");

    // Comparison operators
    assert_eq!(extract_version_from_spec(">=1.0.0").unwrap(), "1.0.0");
    assert_eq!(extract_version_from_spec(">1.0.0").unwrap(), "1.0.0");
    assert_eq!(extract_version_from_spec("<=2.0.0").unwrap(), "2.0.0");
    assert_eq!(extract_version_from_spec("<2.0.0").unwrap(), "2.0.0");
    assert_eq!(extract_version_from_spec("=1.5.0").unwrap(), "1.5.0");

    // Exact version
    assert_eq!(extract_version_from_spec("1.5.0").unwrap(), "1.5.0");
    assert_eq!(extract_version_from_spec("2.0.0-beta.1").unwrap(), "2.0.0-beta.1");

    // With whitespace
    assert_eq!(extract_version_from_spec("  ^1.2.3  ").unwrap(), "1.2.3");
}

#[test]
fn test_find_latest_version() {
    let versions = vec![
        "1.0.0".to_string(),
        "1.2.0".to_string(),
        "1.1.0".to_string(),
        "2.0.0".to_string(),
        "1.5.0".to_string(),
    ];
    assert_eq!(find_latest_version(&versions).unwrap(), "2.0.0");

    // With prereleases
    let versions_with_pre = vec![
        "1.0.0".to_string(),
        "2.0.0-alpha.1".to_string(),
        "2.0.0".to_string(),
        "2.1.0-beta.1".to_string(),
    ];
    assert_eq!(find_latest_version(&versions_with_pre).unwrap(), "2.1.0-beta.1");

    // Single version
    let single = vec!["1.0.0".to_string()];
    assert_eq!(find_latest_version(&single).unwrap(), "1.0.0");
}

#[test]
fn test_find_latest_prerelease() {
    let versions = vec![
        "1.0.0".to_string(),
        "2.0.0-alpha.1".to_string(),
        "2.0.0-beta.1".to_string(),
        "2.0.0".to_string(),
        "2.1.0-rc.1".to_string(),
    ];
    assert_eq!(find_latest_prerelease(&versions), Some("2.1.0-rc.1".to_string()));

    // No prereleases
    let no_prerelease = vec!["1.0.0".to_string(), "2.0.0".to_string()];
    assert_eq!(find_latest_prerelease(&no_prerelease), None);

    // Only prereleases
    let only_pre = vec!["1.0.0-alpha.1".to_string(), "1.0.0-beta.1".to_string()];
    assert_eq!(find_latest_prerelease(&only_pre), Some("1.0.0-beta.1".to_string()));
}

#[test]
fn test_detection_options_package_filtering() {
    let mut options = DetectionOptions::default();
    options.package_filter = Some(vec!["my-package".to_string(), "other-package".to_string()]);

    assert!(options.matches_package_filter("my-package"));
    assert!(options.matches_package_filter("other-package"));
    assert!(!options.matches_package_filter("unknown-package"));

    // No filter means match all
    let options_no_filter = DetectionOptions::default();
    assert!(options_no_filter.matches_package_filter("any-package"));
}

#[test]
fn test_detection_options_dependency_filtering() {
    let mut options = DetectionOptions::default();
    options.dependency_filter =
        Some(vec!["express".to_string(), "lodash".to_string(), "react".to_string()]);

    assert!(options.matches_dependency_filter("express"));
    assert!(options.matches_dependency_filter("lodash"));
    assert!(options.matches_dependency_filter("react"));
    assert!(!options.matches_dependency_filter("vue"));

    // No filter means match all
    let options_no_filter = DetectionOptions::default();
    assert!(options_no_filter.matches_dependency_filter("any-dep"));
}

#[tokio::test]
async fn test_extract_dependencies_regular() {
    let mut package_json = PackageJson::default();
    let mut deps = HashMap::new();
    deps.insert("express".to_string(), "^4.17.1".to_string());
    deps.insert("lodash".to_string(), "^4.17.21".to_string());
    deps.insert("internal".to_string(), "workspace:*".to_string());
    package_json.dependencies = Some(deps);

    let mut options = DetectionOptions::default();
    options.include_dependencies = true;

    let result = extract_dependencies(&package_json, &options);

    assert_eq!(result.len(), 2); // workspace:* should be filtered out
    assert!(result.iter().any(|d| d.name == "express"));
    assert!(result.iter().any(|d| d.name == "lodash"));
    assert!(!result.iter().any(|d| d.name == "internal"));
}

#[tokio::test]
async fn test_extract_dependencies_dev() {
    let mut package_json = PackageJson::default();
    let mut dev_deps = HashMap::new();
    dev_deps.insert("jest".to_string(), "^27.0.0".to_string());
    dev_deps.insert("eslint".to_string(), "^8.0.0".to_string());
    package_json.dev_dependencies = Some(dev_deps);

    let mut options = DetectionOptions::default();
    options.include_dev_dependencies = true;

    let result = extract_dependencies(&package_json, &options);

    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|d| d.dependency_type == DependencyType::Dev));
}

#[tokio::test]
async fn test_extract_dependencies_with_filters() {
    let mut package_json = PackageJson::default();
    let mut deps = HashMap::new();
    deps.insert("express".to_string(), "^4.17.1".to_string());
    deps.insert("lodash".to_string(), "^4.17.21".to_string());
    deps.insert("react".to_string(), "^18.0.0".to_string());
    package_json.dependencies = Some(deps);

    let mut options = DetectionOptions::default();
    options.include_dependencies = true;
    options.dependency_filter = Some(vec!["express".to_string(), "react".to_string()]);

    let result = extract_dependencies(&package_json, &options);

    assert_eq!(result.len(), 2);
    assert!(result.iter().any(|d| d.name == "express"));
    assert!(result.iter().any(|d| d.name == "react"));
    assert!(!result.iter().any(|d| d.name == "lodash"));
}

#[tokio::test]
async fn test_extract_dependencies_mixed_types() {
    let mut package_json = PackageJson::default();

    let mut deps = HashMap::new();
    deps.insert("express".to_string(), "^4.17.1".to_string());
    package_json.dependencies = Some(deps);

    let mut dev_deps = HashMap::new();
    dev_deps.insert("jest".to_string(), "^27.0.0".to_string());
    package_json.dev_dependencies = Some(dev_deps);

    let mut peer_deps = HashMap::new();
    peer_deps.insert("react".to_string(), "^18.0.0".to_string());
    package_json.peer_dependencies = Some(peer_deps);

    let options = DetectionOptions::all();
    let result = extract_dependencies(&package_json, &options);

    assert_eq!(result.len(), 3);
    assert!(
        result.iter().any(|d| d.name == "express" && d.dependency_type == DependencyType::Regular)
    );
    assert!(result.iter().any(|d| d.name == "jest" && d.dependency_type == DependencyType::Dev));
    assert!(result.iter().any(|d| d.name == "react" && d.dependency_type == DependencyType::Peer));
}

#[tokio::test]
async fn test_extract_dependencies_filters_protocols() {
    let mut package_json = PackageJson::default();
    let mut deps = HashMap::new();
    deps.insert("workspace-dep".to_string(), "workspace:*".to_string());
    deps.insert("file-dep".to_string(), "file:../local".to_string());
    deps.insert("link-dep".to_string(), "link:../linked".to_string());
    deps.insert("portal-dep".to_string(), "portal:../portal".to_string());
    deps.insert("external-dep".to_string(), "^1.0.0".to_string());
    package_json.dependencies = Some(deps);

    let mut options = DetectionOptions::default();
    options.include_dependencies = true;

    let result = extract_dependencies(&package_json, &options);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "external-dep");
}

#[tokio::test]
async fn test_read_package_json_valid() {
    let temp_dir = TempDir::new().unwrap();
    let fs = FileSystemManager::new();
    let package_json_path = temp_dir.path().join("package.json");

    create_test_package_json(&fs, &package_json_path, "test-package", "1.0.0", None, None)
        .await
        .unwrap();

    let result = read_package_json(&package_json_path, &fs).await;
    assert!(result.is_ok());

    let package_json = result.unwrap();
    assert_eq!(package_json.name, "test-package".to_string());
    assert_eq!(package_json.version, "1.0.0".to_string());
}

#[tokio::test]
async fn test_read_package_json_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let fs = FileSystemManager::new();
    let package_json_path = temp_dir.path().join("nonexistent.json");

    let result = read_package_json(&package_json_path, &fs).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UpgradeError::FileSystemError { .. }));
}

#[tokio::test]
async fn test_read_package_json_invalid() {
    let temp_dir = TempDir::new().unwrap();
    let fs = FileSystemManager::new();
    let package_json_path = temp_dir.path().join("package.json");

    // Write invalid JSON
    fs.write_file_string(&package_json_path, "{ invalid json }").await.unwrap();

    let result = read_package_json(&package_json_path, &fs).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UpgradeError::PackageJsonError { .. }));
}

#[tokio::test]
async fn test_find_package_json_files_single_package() {
    let temp_dir = TempDir::new().unwrap();
    let fs = FileSystemManager::new();
    let package_json_path = temp_dir.path().join("package.json");

    create_test_package_json(&fs, &package_json_path, "test-package", "1.0.0", None, None)
        .await
        .unwrap();

    let result = find_package_json_files(temp_dir.path(), &fs).await;
    assert!(result.is_ok());

    let files = result.unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0], package_json_path);
}

#[tokio::test]
async fn test_find_package_json_files_no_package() {
    let temp_dir = TempDir::new().unwrap();
    let fs = FileSystemManager::new();

    let result = find_package_json_files(temp_dir.path(), &fs).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UpgradeError::NoPackagesFound { .. }));
}

#[test]
fn test_upgrade_preview_serialization() {
    let preview = UpgradePreview {
        detected_at: Utc::now(),
        packages: vec![],
        summary: UpgradeSummary {
            packages_scanned: 1,
            total_dependencies: 5,
            upgrades_available: 3,
            major_upgrades: 1,
            minor_upgrades: 1,
            patch_upgrades: 1,
            deprecated_dependencies: 0,
        },
    };

    let json = serde_json::to_string(&preview);
    assert!(json.is_ok());

    let deserialized: Result<UpgradePreview, _> = serde_json::from_str(&json.unwrap());
    assert!(deserialized.is_ok());
}

#[test]
fn test_dependency_upgrade_serialization() {
    let upgrade = DependencyUpgrade {
        name: "express".to_string(),
        current_version: "^4.17.1".to_string(),
        latest_version: "4.18.2".to_string(),
        upgrade_type: UpgradeType::Minor,
        dependency_type: DependencyType::Regular,
        registry_url: "https://registry.npmjs.org".to_string(),
        version_info: VersionInfo {
            available_versions: vec!["4.17.1".to_string(), "4.18.2".to_string()],
            latest_stable: "4.18.2".to_string(),
            latest_prerelease: None,
            deprecated: None,
            published_at: None,
        },
    };

    let json = serde_json::to_string(&upgrade);
    assert!(json.is_ok());

    let deserialized: Result<DependencyUpgrade, _> = serde_json::from_str(&json.unwrap());
    assert!(deserialized.is_ok());
}

#[test]
fn test_extract_version_from_spec_edge_cases() {
    // Empty after trimming
    let result = extract_version_from_spec("^");
    assert!(result.is_err());

    let result = extract_version_from_spec("~");
    assert!(result.is_err());

    let result = extract_version_from_spec("");
    assert!(result.is_err());

    // Valid edge cases
    assert_eq!(extract_version_from_spec("0.0.1").unwrap(), "0.0.1");
    assert_eq!(extract_version_from_spec("1.0.0-rc.1").unwrap(), "1.0.0-rc.1");
}
