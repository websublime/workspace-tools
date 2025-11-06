//! Tests for upgrade application module.
//!
//! **What**: Comprehensive tests for applying dependency upgrades, including
//! filtering, dry-run mode, format preservation, and error handling.
//!
//! **How**: Tests use mock filesystem to simulate package.json files and verify
//! that upgrades are correctly applied, filtered, and reported. Tests cover
//! various scenarios including selection filtering, version prefix preservation,
//! and JSON formatting preservation.
//!
//! **Why**: To ensure upgrade application works correctly across all supported
//! scenarios and edge cases, maintaining data integrity and user expectations.

#![allow(clippy::unwrap_used)]

use crate::types::DependencyType;
use crate::upgrade::application::applier::preserve_version_prefix;
use crate::upgrade::application::{UpgradeSelection, apply_upgrades};
use crate::upgrade::detection::{DependencyUpgrade, PackageUpgrades, VersionInfo};
use crate::upgrade::registry::UpgradeType;
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use sublime_standard_tools::error::{FileSystemError, Result as StandardResult};
use sublime_standard_tools::filesystem::AsyncFileSystem;

/// Mock filesystem for testing
struct MockFileSystem {
    files: HashMap<PathBuf, String>,
    read_errors: HashMap<PathBuf, String>,
    write_errors: HashMap<PathBuf, String>,
}

impl MockFileSystem {
    fn new() -> Self {
        Self { files: HashMap::new(), read_errors: HashMap::new(), write_errors: HashMap::new() }
    }

    fn add_file(&mut self, path: PathBuf, content: String) {
        self.files.insert(path, content);
    }

    fn add_read_error(&mut self, path: PathBuf, error: String) {
        self.read_errors.insert(path, error);
    }

    #[allow(dead_code)]
    fn add_write_error(&mut self, path: PathBuf, error: String) {
        self.write_errors.insert(path, error);
    }

    #[allow(dead_code)]
    fn get_file(&self, path: &PathBuf) -> Option<&String> {
        self.files.get(path)
    }
}

#[async_trait::async_trait]
impl AsyncFileSystem for MockFileSystem {
    async fn read_file(&self, path: &Path) -> StandardResult<Vec<u8>> {
        let path_buf = path.to_path_buf();
        if let Some(error) = self.read_errors.get(&path_buf) {
            return Err(
                FileSystemError::Io { path: path_buf.clone(), message: error.clone() }.into()
            );
        }

        self.files
            .get(&path_buf)
            .map(|s| s.as_bytes().to_vec())
            .ok_or_else(|| FileSystemError::NotFound { path: path_buf.clone() }.into())
    }

    async fn write_file(&self, path: &Path, _content: &[u8]) -> StandardResult<()> {
        let path_buf = path.to_path_buf();
        if let Some(error) = self.write_errors.get(&path_buf) {
            return Err(
                FileSystemError::Io { path: path_buf.clone(), message: error.clone() }.into()
            );
        }

        Ok(())
    }

    async fn read_file_string(&self, path: &Path) -> StandardResult<String> {
        let path_buf = path.to_path_buf();
        if let Some(error) = self.read_errors.get(&path_buf) {
            return Err(
                FileSystemError::Io { path: path_buf.clone(), message: error.clone() }.into()
            );
        }

        self.files
            .get(&path_buf)
            .cloned()
            .ok_or_else(|| FileSystemError::NotFound { path: path_buf.clone() }.into())
    }

    async fn write_file_string(&self, path: &Path, _contents: &str) -> StandardResult<()> {
        let path_buf = path.to_path_buf();
        if let Some(error) = self.write_errors.get(&path_buf) {
            return Err(
                FileSystemError::Io { path: path_buf.clone(), message: error.clone() }.into()
            );
        }

        Ok(())
    }

    async fn exists(&self, path: &Path) -> bool {
        self.files.contains_key(&path.to_path_buf())
    }

    async fn create_dir_all(&self, _path: &Path) -> StandardResult<()> {
        Ok(())
    }

    async fn read_dir(&self, _path: &Path) -> StandardResult<Vec<PathBuf>> {
        Ok(vec![])
    }

    async fn remove(&self, _path: &Path) -> StandardResult<()> {
        Ok(())
    }

    async fn walk_dir(&self, _path: &Path) -> StandardResult<Vec<PathBuf>> {
        Ok(vec![])
    }

    async fn metadata(&self, path: &Path) -> StandardResult<std::fs::Metadata> {
        Err(FileSystemError::NotFound { path: path.to_path_buf() }.into())
    }
}

fn create_test_package_json() -> String {
    r#"{
  "name": "test-package",
  "version": "1.0.0",
  "dependencies": {
    "lodash": "^4.17.20",
    "react": "^17.0.0"
  },
  "devDependencies": {
    "webpack": "^4.46.0"
  }
}
"#
    .to_string()
}

fn create_test_upgrade(
    name: &str,
    current: &str,
    latest: &str,
    upgrade_type: UpgradeType,
    dep_type: DependencyType,
) -> DependencyUpgrade {
    DependencyUpgrade {
        name: name.to_string(),
        current_version: current.to_string(),
        latest_version: latest.to_string(),
        upgrade_type,
        dependency_type: dep_type,
        registry_url: "https://registry.npmjs.org".to_string(),
        version_info: VersionInfo {
            available_versions: vec![latest.to_string()],
            latest_stable: latest.to_string(),
            latest_prerelease: None,
            deprecated: None,
            published_at: Some(Utc::now()),
        },
    }
}

fn create_package_upgrades(
    package_name: &str,
    package_path: PathBuf,
    upgrades: Vec<DependencyUpgrade>,
) -> PackageUpgrades {
    PackageUpgrades {
        package_name: package_name.to_string(),
        package_path,
        current_version: Some("1.0.0".to_string()),
        upgrades,
    }
}

#[tokio::test]
async fn test_apply_upgrades_dry_run() {
    let mut fs = MockFileSystem::new();
    let package_path = PathBuf::from("packages/test-package");
    let json_path = package_path.join("package.json");

    fs.add_file(json_path.clone(), create_test_package_json());

    let upgrades = vec![create_package_upgrades(
        "test-package",
        package_path.clone(),
        vec![create_test_upgrade(
            "lodash",
            "^4.17.20",
            "4.17.21",
            UpgradeType::Patch,
            DependencyType::Regular,
        )],
    )];

    let selection = UpgradeSelection::all();
    let result = apply_upgrades(upgrades, selection, true, &fs).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.dry_run);
    assert_eq!(result.applied.len(), 1);
    assert!(result.modified_files.is_empty()); // Dry-run doesn't modify files
    assert!(result.backup_path.is_none());
    assert_eq!(result.summary.dependencies_upgraded, 1);
    assert_eq!(result.summary.patch_upgrades, 1);
}

#[tokio::test]
async fn test_apply_upgrades_actual() {
    let mut fs = MockFileSystem::new();
    let package_path = PathBuf::from("packages/test-package");
    let json_path = package_path.join("package.json");

    fs.add_file(json_path.clone(), create_test_package_json());

    let upgrades = vec![create_package_upgrades(
        "test-package",
        package_path.clone(),
        vec![create_test_upgrade(
            "lodash",
            "^4.17.20",
            "4.17.21",
            UpgradeType::Patch,
            DependencyType::Regular,
        )],
    )];

    let selection = UpgradeSelection::all();
    let result = apply_upgrades(upgrades, selection, false, &fs).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(!result.dry_run);
    assert_eq!(result.applied.len(), 1);
    assert_eq!(result.modified_files.len(), 1);
    assert_eq!(result.summary.packages_modified, 1);
}

#[tokio::test]
async fn test_apply_upgrades_patch_only_filter() {
    let mut fs = MockFileSystem::new();
    let package_path = PathBuf::from("packages/test-package");
    let json_path = package_path.join("package.json");

    fs.add_file(json_path.clone(), create_test_package_json());

    let upgrades = vec![create_package_upgrades(
        "test-package",
        package_path.clone(),
        vec![
            create_test_upgrade(
                "lodash",
                "^4.17.20",
                "4.17.21",
                UpgradeType::Patch,
                DependencyType::Regular,
            ),
            create_test_upgrade(
                "react",
                "^17.0.0",
                "18.0.0",
                UpgradeType::Major,
                DependencyType::Regular,
            ),
            create_test_upgrade(
                "webpack",
                "^4.46.0",
                "4.47.0",
                UpgradeType::Minor,
                DependencyType::Dev,
            ),
        ],
    )];

    let selection = UpgradeSelection::patch_only();
    let result = apply_upgrades(upgrades, selection, true, &fs).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.applied.len(), 1); // Only patch upgrade
    assert_eq!(result.applied[0].dependency_name, "lodash");
    assert_eq!(result.summary.patch_upgrades, 1);
    assert_eq!(result.summary.major_upgrades, 0);
    assert_eq!(result.summary.minor_upgrades, 0);
}

#[tokio::test]
async fn test_apply_upgrades_minor_and_patch_filter() {
    let mut fs = MockFileSystem::new();
    let package_path = PathBuf::from("packages/test-package");
    let json_path = package_path.join("package.json");

    fs.add_file(json_path.clone(), create_test_package_json());

    let upgrades = vec![create_package_upgrades(
        "test-package",
        package_path.clone(),
        vec![
            create_test_upgrade(
                "lodash",
                "^4.17.20",
                "4.17.21",
                UpgradeType::Patch,
                DependencyType::Regular,
            ),
            create_test_upgrade(
                "react",
                "^17.0.0",
                "18.0.0",
                UpgradeType::Major,
                DependencyType::Regular,
            ),
            create_test_upgrade(
                "webpack",
                "^4.46.0",
                "4.47.0",
                UpgradeType::Minor,
                DependencyType::Dev,
            ),
        ],
    )];

    let selection = UpgradeSelection::minor_and_patch();
    let result = apply_upgrades(upgrades, selection, true, &fs).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.applied.len(), 2); // Patch + Minor
    assert_eq!(result.summary.patch_upgrades, 1);
    assert_eq!(result.summary.minor_upgrades, 1);
    assert_eq!(result.summary.major_upgrades, 0);
}

#[tokio::test]
async fn test_apply_upgrades_package_filter() {
    let mut fs = MockFileSystem::new();
    let package1_path = PathBuf::from("packages/package1");
    let package2_path = PathBuf::from("packages/package2");

    fs.add_file(package1_path.join("package.json"), create_test_package_json());
    fs.add_file(package2_path.join("package.json"), create_test_package_json());

    let upgrades = vec![
        create_package_upgrades(
            "package1",
            package1_path.clone(),
            vec![create_test_upgrade(
                "lodash",
                "^4.17.20",
                "4.17.21",
                UpgradeType::Patch,
                DependencyType::Regular,
            )],
        ),
        create_package_upgrades(
            "package2",
            package2_path.clone(),
            vec![create_test_upgrade(
                "react",
                "^17.0.0",
                "17.0.2",
                UpgradeType::Patch,
                DependencyType::Regular,
            )],
        ),
    ];

    let selection = UpgradeSelection::packages(vec!["package1".to_string()]);
    let result = apply_upgrades(upgrades, selection, true, &fs).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.applied.len(), 1);
    assert_eq!(result.applied[0].dependency_name, "lodash");
    assert_eq!(result.summary.packages_modified, 1);
}

#[tokio::test]
async fn test_apply_upgrades_dependency_filter() {
    let mut fs = MockFileSystem::new();
    let package_path = PathBuf::from("packages/test-package");
    let json_path = package_path.join("package.json");

    fs.add_file(json_path.clone(), create_test_package_json());

    let upgrades = vec![create_package_upgrades(
        "test-package",
        package_path.clone(),
        vec![
            create_test_upgrade(
                "lodash",
                "^4.17.20",
                "4.17.21",
                UpgradeType::Patch,
                DependencyType::Regular,
            ),
            create_test_upgrade(
                "react",
                "^17.0.0",
                "17.0.2",
                UpgradeType::Patch,
                DependencyType::Regular,
            ),
        ],
    )];

    let selection = UpgradeSelection::dependencies(vec!["lodash".to_string()]);
    let result = apply_upgrades(upgrades, selection, true, &fs).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.applied.len(), 1);
    assert_eq!(result.applied[0].dependency_name, "lodash");
}

#[tokio::test]
async fn test_apply_upgrades_no_matches() {
    let mut fs = MockFileSystem::new();
    let package_path = PathBuf::from("packages/test-package");
    let json_path = package_path.join("package.json");

    fs.add_file(json_path.clone(), create_test_package_json());

    let upgrades = vec![create_package_upgrades(
        "test-package",
        package_path.clone(),
        vec![create_test_upgrade(
            "lodash",
            "^4.17.20",
            "5.0.0",
            UpgradeType::Major,
            DependencyType::Regular,
        )],
    )];

    let selection = UpgradeSelection::patch_only(); // Only patch, but we have major
    let result = apply_upgrades(upgrades, selection, true, &fs).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.applied.len(), 0);
    assert!(!result.has_changes());
}

#[tokio::test]
async fn test_apply_upgrades_multiple_packages() {
    let mut fs = MockFileSystem::new();
    let package1_path = PathBuf::from("packages/package1");
    let package2_path = PathBuf::from("packages/package2");

    fs.add_file(package1_path.join("package.json"), create_test_package_json());
    fs.add_file(package2_path.join("package.json"), create_test_package_json());

    let upgrades = vec![
        create_package_upgrades(
            "package1",
            package1_path.clone(),
            vec![create_test_upgrade(
                "lodash",
                "^4.17.20",
                "4.17.21",
                UpgradeType::Patch,
                DependencyType::Regular,
            )],
        ),
        create_package_upgrades(
            "package2",
            package2_path.clone(),
            vec![create_test_upgrade(
                "react",
                "^17.0.0",
                "17.0.2",
                UpgradeType::Patch,
                DependencyType::Regular,
            )],
        ),
    ];

    let selection = UpgradeSelection::all();
    let result = apply_upgrades(upgrades, selection, true, &fs).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.applied.len(), 2);
    assert_eq!(result.summary.packages_modified, 2);
}

#[tokio::test]
async fn test_apply_upgrades_file_read_error() {
    let mut fs = MockFileSystem::new();
    let package_path = PathBuf::from("packages/test-package");
    let json_path = package_path.join("package.json");

    fs.add_read_error(json_path.clone(), "Permission denied".to_string());

    let upgrades = vec![create_package_upgrades(
        "test-package",
        package_path.clone(),
        vec![create_test_upgrade(
            "lodash",
            "^4.17.20",
            "4.17.21",
            UpgradeType::Patch,
            DependencyType::Regular,
        )],
    )];

    let selection = UpgradeSelection::all();
    let result = apply_upgrades(upgrades, selection, true, &fs).await;

    // Should succeed but skip the failing package
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.applied.len(), 0);
}

#[tokio::test]
async fn test_preserve_version_prefix_caret() {
    assert_eq!(preserve_version_prefix("^1.2.3", "1.2.4"), "^1.2.4");
    assert_eq!(preserve_version_prefix("^2.0.0", "3.0.0"), "^3.0.0");
}

#[tokio::test]
async fn test_preserve_version_prefix_tilde() {
    assert_eq!(preserve_version_prefix("~1.2.3", "1.2.4"), "~1.2.4");
    assert_eq!(preserve_version_prefix("~2.0.0", "2.1.0"), "~2.1.0");
}

#[tokio::test]
async fn test_preserve_version_prefix_exact() {
    assert_eq!(preserve_version_prefix("1.2.3", "1.2.4"), "1.2.4");
    assert_eq!(preserve_version_prefix("2.0.0", "3.0.0"), "3.0.0");
}

#[tokio::test]
async fn test_preserve_version_prefix_gte() {
    assert_eq!(preserve_version_prefix(">=1.2.3", "1.2.4"), ">=1.2.4");
    assert_eq!(preserve_version_prefix(">2.0.0", "2.1.0"), ">2.1.0");
}

#[tokio::test]
async fn test_upgrade_result_methods() {
    let mut fs = MockFileSystem::new();
    let package_path = PathBuf::from("packages/test-package");
    let json_path = package_path.join("package.json");

    fs.add_file(json_path.clone(), create_test_package_json());

    let upgrades = vec![create_package_upgrades(
        "test-package",
        package_path.clone(),
        vec![create_test_upgrade(
            "lodash",
            "^4.17.20",
            "4.17.21",
            UpgradeType::Patch,
            DependencyType::Regular,
        )],
    )];

    let selection = UpgradeSelection::all();
    let result = apply_upgrades(upgrades, selection, true, &fs).await.unwrap();

    assert!(result.has_changes());
    assert_eq!(result.packages_modified(), 1);
    assert_eq!(result.dependencies_upgraded(), 1);
}

#[tokio::test]
async fn test_applied_upgrade_methods() {
    let mut fs = MockFileSystem::new();
    let package_path = PathBuf::from("packages/test-package");
    let json_path = package_path.join("package.json");

    fs.add_file(json_path.clone(), create_test_package_json());

    let upgrades = vec![create_package_upgrades(
        "test-package",
        package_path.clone(),
        vec![
            create_test_upgrade(
                "lodash",
                "^4.17.20",
                "4.17.21",
                UpgradeType::Patch,
                DependencyType::Regular,
            ),
            create_test_upgrade(
                "react",
                "^17.0.0",
                "17.1.0",
                UpgradeType::Minor,
                DependencyType::Regular,
            ),
            create_test_upgrade(
                "webpack",
                "^4.46.0",
                "5.0.0",
                UpgradeType::Major,
                DependencyType::Dev,
            ),
        ],
    )];

    let selection = UpgradeSelection::all();
    let result = apply_upgrades(upgrades, selection, true, &fs).await.unwrap();

    assert_eq!(result.applied.len(), 3);

    // Check patch upgrade
    let patch_upgrade = &result.applied[0];
    assert!(patch_upgrade.is_patch());
    assert!(!patch_upgrade.is_minor());
    assert!(!patch_upgrade.is_major());
    assert!(!patch_upgrade.is_breaking());
    assert_eq!(patch_upgrade.version_change(), "^4.17.20 → 4.17.21");

    // Check minor upgrade
    let minor_upgrade = &result.applied[1];
    assert!(!minor_upgrade.is_patch());
    assert!(minor_upgrade.is_minor());
    assert!(!minor_upgrade.is_major());
    assert!(!minor_upgrade.is_breaking());

    // Check major upgrade
    let major_upgrade = &result.applied[2];
    assert!(!major_upgrade.is_patch());
    assert!(!major_upgrade.is_minor());
    assert!(major_upgrade.is_major());
    assert!(major_upgrade.is_breaking());
}

#[tokio::test]
async fn test_summary_statistics() {
    let mut fs = MockFileSystem::new();
    let package_path = PathBuf::from("packages/test-package");
    let json_path = package_path.join("package.json");

    fs.add_file(json_path.clone(), create_test_package_json());

    let upgrades = vec![create_package_upgrades(
        "test-package",
        package_path.clone(),
        vec![
            create_test_upgrade(
                "lodash",
                "^4.17.20",
                "4.17.21",
                UpgradeType::Patch,
                DependencyType::Regular,
            ),
            create_test_upgrade(
                "react",
                "^17.0.0",
                "17.1.0",
                UpgradeType::Minor,
                DependencyType::Regular,
            ),
            create_test_upgrade(
                "webpack",
                "^4.46.0",
                "5.0.0",
                UpgradeType::Major,
                DependencyType::Dev,
            ),
        ],
    )];

    let selection = UpgradeSelection::all();
    let result = apply_upgrades(upgrades, selection, true, &fs).await.unwrap();

    assert_eq!(result.summary.packages_modified, 1);
    assert_eq!(result.summary.dependencies_upgraded, 3);
    assert_eq!(result.summary.patch_upgrades, 1);
    assert_eq!(result.summary.minor_upgrades, 1);
    assert_eq!(result.summary.major_upgrades, 1);
    assert_eq!(result.summary.total_upgrades(), 3);
    assert!(result.summary.has_major_upgrades());
    assert!(result.summary.has_changes());
}

// ============================================================================
// Selection Tests
// ============================================================================

mod selection_tests {
    use super::*;

    #[test]
    fn test_selection_all() {
        let selection = UpgradeSelection::all();
        assert!(selection.all);
        assert!(!selection.patch_only);
        assert!(!selection.minor_and_patch);
        assert!(selection.packages.is_none());
        assert!(selection.dependencies.is_none());
    }

    #[test]
    fn test_selection_patch_only() {
        let selection = UpgradeSelection::patch_only();
        assert!(!selection.all);
        assert!(selection.patch_only);
        assert!(!selection.minor_and_patch);
    }

    #[test]
    fn test_selection_minor_and_patch() {
        let selection = UpgradeSelection::minor_and_patch();
        assert!(!selection.all);
        assert!(!selection.patch_only);
        assert!(selection.minor_and_patch);
    }

    #[test]
    fn test_selection_packages() {
        let packages = vec!["pkg1".to_string(), "pkg2".to_string()];
        let selection = UpgradeSelection::packages(packages.clone());
        assert_eq!(selection.packages, Some(packages));
    }

    #[test]
    fn test_selection_dependencies() {
        let deps = vec!["react".to_string(), "lodash".to_string()];
        let selection = UpgradeSelection::dependencies(deps.clone());
        assert_eq!(selection.dependencies, Some(deps));
    }

    #[test]
    fn test_matches_type_all() {
        let selection = UpgradeSelection::all();
        assert!(selection.matches_type(UpgradeType::Patch));
        assert!(selection.matches_type(UpgradeType::Minor));
        assert!(selection.matches_type(UpgradeType::Major));
    }

    #[test]
    fn test_matches_type_patch_only() {
        let selection = UpgradeSelection::patch_only();
        assert!(selection.matches_type(UpgradeType::Patch));
        assert!(!selection.matches_type(UpgradeType::Minor));
        assert!(!selection.matches_type(UpgradeType::Major));
    }

    #[test]
    fn test_matches_type_minor_and_patch() {
        let selection = UpgradeSelection::minor_and_patch();
        assert!(selection.matches_type(UpgradeType::Patch));
        assert!(selection.matches_type(UpgradeType::Minor));
        assert!(!selection.matches_type(UpgradeType::Major));
    }

    #[test]
    fn test_matches_type_max_upgrade_type() {
        let selection =
            UpgradeSelection { max_upgrade_type: Some(UpgradeType::Minor), ..Default::default() };
        assert!(selection.matches_type(UpgradeType::Patch));
        assert!(selection.matches_type(UpgradeType::Minor));
        assert!(!selection.matches_type(UpgradeType::Major));
    }

    #[test]
    fn test_matches_type_no_filter() {
        let selection = UpgradeSelection::default();
        assert!(selection.matches_type(UpgradeType::Patch));
        assert!(selection.matches_type(UpgradeType::Minor));
        assert!(selection.matches_type(UpgradeType::Major));
    }

    #[test]
    fn test_matches_package() {
        let selection = UpgradeSelection::packages(vec!["pkg1".to_string()]);
        assert!(selection.matches_package("pkg1"));
        assert!(!selection.matches_package("pkg2"));
    }

    #[test]
    fn test_matches_package_no_filter() {
        let selection = UpgradeSelection::default();
        assert!(selection.matches_package("any-package"));
    }

    #[test]
    fn test_matches_dependency() {
        let selection = UpgradeSelection::dependencies(vec!["react".to_string()]);
        assert!(selection.matches_dependency("react"));
        assert!(!selection.matches_dependency("lodash"));
    }

    #[test]
    fn test_matches_dependency_no_filter() {
        let selection = UpgradeSelection::default();
        assert!(selection.matches_dependency("any-dependency"));
    }

    #[test]
    fn test_has_filters() {
        let selection = UpgradeSelection::default();
        assert!(!selection.has_filters());

        let selection = UpgradeSelection::all();
        assert!(selection.has_filters());

        let selection = UpgradeSelection::patch_only();
        assert!(selection.has_filters());

        let selection = UpgradeSelection::packages(vec!["pkg".to_string()]);
        assert!(selection.has_filters());

        let selection =
            UpgradeSelection { max_upgrade_type: Some(UpgradeType::Patch), ..Default::default() };
        assert!(selection.has_filters());
    }
}

// ============================================================================
// Result Tests
// ============================================================================

mod result_tests {
    use super::*;
    use crate::upgrade::application::{AppliedUpgrade, ApplySummary, UpgradeResult};

    #[test]
    fn test_upgrade_result_dry_run() {
        let summary = ApplySummary::new();
        let result = UpgradeResult::dry_run(vec![], summary);

        assert!(result.dry_run);
        assert!(result.modified_files.is_empty());
        assert!(result.backup_path.is_none());
        assert!(result.changeset_id.is_none());
        assert!(!result.has_changes());
    }

    #[test]
    fn test_upgrade_result_applied() {
        let summary = ApplySummary::new();
        let files = vec![PathBuf::from("package.json")];
        let backup = Some(PathBuf::from(".backups"));
        let result = UpgradeResult::applied(vec![], files.clone(), backup.clone(), None, summary);

        assert!(!result.dry_run);
        assert_eq!(result.modified_files, files);
        assert_eq!(result.backup_path, backup);
        assert!(!result.has_changes());
    }

    #[test]
    fn test_applied_upgrade_is_patch() {
        let upgrade = AppliedUpgrade {
            package_path: PathBuf::from("."),
            dependency_name: "lodash".to_string(),
            dependency_type: DependencyType::Regular,
            old_version: "4.17.20".to_string(),
            new_version: "4.17.21".to_string(),
            upgrade_type: UpgradeType::Patch,
        };

        assert!(upgrade.is_patch());
        assert!(!upgrade.is_minor());
        assert!(!upgrade.is_major());
        assert!(!upgrade.is_breaking());
    }

    #[test]
    fn test_applied_upgrade_is_minor() {
        let upgrade = AppliedUpgrade {
            package_path: PathBuf::from("."),
            dependency_name: "react".to_string(),
            dependency_type: DependencyType::Regular,
            old_version: "17.0.0".to_string(),
            new_version: "17.1.0".to_string(),
            upgrade_type: UpgradeType::Minor,
        };

        assert!(!upgrade.is_patch());
        assert!(upgrade.is_minor());
        assert!(!upgrade.is_major());
        assert!(!upgrade.is_breaking());
    }

    #[test]
    fn test_applied_upgrade_is_major() {
        let upgrade = AppliedUpgrade {
            package_path: PathBuf::from("."),
            dependency_name: "webpack".to_string(),
            dependency_type: DependencyType::Dev,
            old_version: "4.46.0".to_string(),
            new_version: "5.0.0".to_string(),
            upgrade_type: UpgradeType::Major,
        };

        assert!(!upgrade.is_patch());
        assert!(!upgrade.is_minor());
        assert!(upgrade.is_major());
        assert!(upgrade.is_breaking());
    }

    #[test]
    fn test_applied_upgrade_version_change() {
        let upgrade = AppliedUpgrade {
            package_path: PathBuf::from("."),
            dependency_name: "lodash".to_string(),
            dependency_type: DependencyType::Regular,
            old_version: "4.17.20".to_string(),
            new_version: "4.17.21".to_string(),
            upgrade_type: UpgradeType::Patch,
        };

        assert_eq!(upgrade.version_change(), "4.17.20 → 4.17.21");
    }

    #[test]
    fn test_apply_summary_new() {
        let summary = ApplySummary::new();
        assert_eq!(summary.packages_modified, 0);
        assert_eq!(summary.dependencies_upgraded, 0);
        assert_eq!(summary.total_upgrades(), 0);
        assert!(!summary.has_major_upgrades());
        assert!(!summary.has_changes());
    }

    #[test]
    fn test_apply_summary_total_upgrades() {
        let mut summary = ApplySummary::new();
        summary.major_upgrades = 1;
        summary.minor_upgrades = 2;
        summary.patch_upgrades = 3;

        assert_eq!(summary.total_upgrades(), 6);
    }

    #[test]
    fn test_apply_summary_has_major_upgrades() {
        let mut summary = ApplySummary::new();
        assert!(!summary.has_major_upgrades());

        summary.major_upgrades = 1;
        assert!(summary.has_major_upgrades());
    }

    #[test]
    fn test_apply_summary_has_changes() {
        let mut summary = ApplySummary::new();
        assert!(!summary.has_changes());

        summary.dependencies_upgraded = 1;
        assert!(summary.has_changes());
    }

    #[test]
    fn test_apply_summary_default() {
        let summary = ApplySummary::default();
        assert_eq!(summary.packages_modified, 0);
        assert!(!summary.has_changes());
    }
}

// ============================================================================
// Applier Tests
// ============================================================================

mod applier_tests {
    use crate::upgrade::application::applier::{detect_indentation, preserve_version_prefix};

    #[test]
    fn test_preserve_version_prefix_caret() {
        assert_eq!(preserve_version_prefix("^1.2.3", "1.2.4"), "^1.2.4");
        assert_eq!(preserve_version_prefix("^2.0.0", "2.1.0"), "^2.1.0");
    }

    #[test]
    fn test_preserve_version_prefix_tilde() {
        assert_eq!(preserve_version_prefix("~1.2.3", "1.2.4"), "~1.2.4");
        assert_eq!(preserve_version_prefix("~2.0.0", "2.0.1"), "~2.0.1");
    }

    #[test]
    fn test_preserve_version_prefix_exact() {
        assert_eq!(preserve_version_prefix("1.2.3", "1.2.4"), "1.2.4");
        assert_eq!(preserve_version_prefix("2.0.0", "2.1.0"), "2.1.0");
    }

    #[test]
    fn test_preserve_version_prefix_other() {
        assert_eq!(preserve_version_prefix(">=1.2.3", "1.2.4"), ">=1.2.4");
        assert_eq!(preserve_version_prefix(">2.0.0", "2.1.0"), ">2.1.0");
        assert_eq!(preserve_version_prefix("=3.0.0", "3.0.1"), "=3.0.1");
    }

    #[test]
    fn test_detect_indentation_spaces() {
        let content = r#"{
  "name": "test",
  "version": "1.0.0"
}"#;
        assert_eq!(detect_indentation(content), "  ");
    }

    #[test]
    fn test_detect_indentation_tabs() {
        let content = "{\n\t\"name\": \"test\",\n\t\"version\": \"1.0.0\"\n}";
        assert_eq!(detect_indentation(content), "\t");
    }

    #[test]
    fn test_detect_indentation_four_spaces() {
        let content = r#"{
    "name": "test",
    "version": "1.0.0"
}"#;
        assert_eq!(detect_indentation(content), "    ");
    }

    #[test]
    fn test_detect_indentation_default() {
        let content = r#"{"name":"test","version":"1.0.0"}"#;
        assert_eq!(detect_indentation(content), "  ");
    }
}
