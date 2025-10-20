//! # Test Infrastructure Integration Tests
//!
//! This test file demonstrates the usage of all test utilities provided by the
//! test infrastructure. It serves as both documentation and validation that the
//! test infrastructure is working correctly.

// Re-export the common test utilities
mod common;

use common::{
    assertions::*,
    create_temp_dir, fixture_path,
    fixtures::{
        create_basic_monorepo_fixture, create_single_package_fixture, MonorepoFixtureBuilder,
        PackageJsonBuilder,
    },
    fixtures_dir,
    generators::*,
    mocks::{filesystem::MockFileSystem, git::MockGitRepository, registry::MockRegistry},
};
use proptest::prelude::*;

/// Tests that the fixtures directory exists and is accessible
#[test]
fn test_fixtures_directory_exists() {
    let dir = fixtures_dir();
    assert_path_exists(&dir);
    assert_is_dir(&dir);
}

/// Tests that fixture paths can be constructed correctly
#[test]
fn test_fixture_paths() {
    let monorepo_path = fixture_path("monorepo");
    assert_path_exists(&monorepo_path);
    assert_is_dir(&monorepo_path);

    let single_path = fixture_path("single-package");
    assert_path_exists(&single_path);
    assert_is_dir(&single_path);
}

/// Tests temporary directory creation
#[test]
fn test_temp_dir_creation() {
    let temp = create_temp_dir().unwrap();
    assert_path_exists(temp.path());
    assert_is_dir(temp.path());
}

/// Tests MockFileSystem basic operations
#[tokio::test]
async fn test_mock_filesystem_operations() {
    let fs = MockFileSystem::new();

    // Test write and read
    let path = std::path::Path::new("/test/file.txt");
    fs.write_file_string(path, "test content").await.unwrap();

    let content = fs.read_file_string(path).await.unwrap();
    assert_eq!(content, "test content");

    // Test exists
    assert!(fs.exists(path).await);
    assert!(fs.is_file(path));

    // Test directory operations
    let dir = std::path::Path::new("/test");
    assert!(fs.is_dir(dir));
}

/// Tests MockFileSystem with package.json files
#[tokio::test]
async fn test_mock_filesystem_with_package_json() {
    let fs = MockFileSystem::new();

    let package_json = PackageJsonBuilder::new("test-package")
        .version("1.2.3")
        .description("Test package for infrastructure")
        .add_dependency("react", "^18.0.0")
        .add_dev_dependency("jest", "^29.0.0")
        .build();

    let path = std::path::Path::new("/package.json");
    fs.write_file_string(path, &package_json).await.unwrap();

    let content = fs.read_file_string(path).await.unwrap();
    assert_contains(&content, r#""name": "test-package""#);
    assert_contains(&content, r#""version": "1.2.3""#);
    assert_json_field(&content, "name", "test-package");
    assert_json_field(&content, "version", "1.2.3");
}

/// Tests MockGitRepository commit operations
#[test]
fn test_mock_git_repository() {
    let repo = MockGitRepository::new("/repo");

    // Add commits
    repo.add_commit(
        "abc123",
        "feat: add new feature",
        vec![std::path::PathBuf::from("/src/feature.js")],
    );
    repo.add_commit("def456", "fix: fix bug", vec![std::path::PathBuf::from("/src/bug.js")]);

    // Test commit retrieval
    let commits = repo.get_commits();
    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].hash, "abc123");
    assert_eq!(commits[1].hash, "def456");

    // Test branches
    repo.add_branch("develop", "def456");
    let branches = repo.get_branches();
    assert!(branches.contains_key("develop"));

    // Test tags
    repo.add_tag("v1.0.0", "def456");
    let tags = repo.get_tags();
    assert_eq!(tags.get("v1.0.0"), Some(&"def456".to_string()));
}

/// Tests MockGitRepository with commit builder
#[test]
fn test_mock_git_commit_builder() {
    use common::mocks::git::MockCommit;

    let commit = MockCommit::builder("abc123")
        .message("feat(api): add new endpoint")
        .author("John Doe")
        .author_email("john@example.com")
        .add_file("/src/api/endpoint.js")
        .add_file("/src/api/types.ts")
        .lines_added(50)
        .lines_deleted(10)
        .build();

    assert_eq!(commit.hash, "abc123");
    assert_eq!(commit.message, "feat(api): add new endpoint");
    assert_eq!(commit.author, "John Doe");
    assert_eq!(commit.files_changed.len(), 2);
    assert_eq!(commit.lines_added, 50);
    assert_eq!(commit.lines_deleted, 10);
}

/// Tests MockRegistry package operations
#[test]
fn test_mock_registry() {
    let registry = MockRegistry::new();

    // Add packages
    registry.add_package("react", vec!["18.0.0", "18.1.0", "18.2.0"]);
    registry.add_package("vue", vec!["3.0.0", "3.1.0", "3.2.0"]);

    // Test package retrieval
    let react = registry.get_package("react");
    assert!(react.is_some());

    let react_metadata = react.unwrap();
    assert_eq!(react_metadata.name, "react");
    assert_eq!(react_metadata.versions.len(), 3);

    // Test latest version
    let latest = registry.get_latest_version("react");
    assert_eq!(latest, Some("18.2.0".to_string()));

    // Test package count
    assert_eq!(registry.package_count(), 2);
}

/// Tests MockRegistry deprecation functionality
#[test]
fn test_mock_registry_deprecation() {
    let registry = MockRegistry::new();
    registry.add_package("old-package", vec!["1.0.0", "2.0.0"]);

    // Deprecate entire package
    registry.deprecate_package("old-package", "Use new-package instead");

    let metadata = registry.get_package("old-package").unwrap();
    assert!(metadata.deprecated.is_some());
    assert_contains(&metadata.deprecated.unwrap(), "new-package");

    // Deprecate specific version
    registry.deprecate_version("old-package", "1.0.0", "Use 2.0.0");

    // Refetch metadata to get updated version deprecation
    let metadata = registry.get_package("old-package").unwrap();
    let version_metadata = metadata.versions.get("1.0.0").unwrap();
    assert!(version_metadata.deprecated.is_some());
}

/// Tests PackageJsonBuilder with various configurations
#[test]
fn test_package_json_builder() {
    let json = PackageJsonBuilder::new("my-package")
        .version("2.3.4")
        .description("A test package")
        .add_dependency("lodash", "^4.17.21")
        .add_dependency("axios", "^1.4.0")
        .add_dev_dependency("jest", "^29.0.0")
        .add_script("test", "jest")
        .add_script("build", "tsc")
        .build();

    assert_json_field(&json, "name", "my-package");
    assert_json_field(&json, "version", "2.3.4");
    assert_contains(&json, "lodash");
    assert_contains(&json, "jest");
    assert_contains(&json, r#""scripts""#);
}

/// Tests MonorepoFixtureBuilder
#[test]
fn test_monorepo_fixture_builder() {
    let fixture = MonorepoFixtureBuilder::new("test-monorepo")
        .workspace_patterns(vec!["packages/*".to_string()])
        .add_package("packages/pkg1", "pkg1", "1.0.0")
        .add_package("packages/pkg2", "pkg2", "2.0.0")
        .build();

    assert_eq!(fixture.root_name, "test-monorepo");
    assert_eq!(fixture.packages.len(), 2);

    let files = fixture.generate_files();
    assert!(files.contains_key(&std::path::PathBuf::from("package.json")));
    assert!(files.contains_key(&std::path::PathBuf::from("packages/pkg1/package.json")));
    assert!(files.contains_key(&std::path::PathBuf::from("packages/pkg2/package.json")));
}

/// Tests basic monorepo fixture creation
#[test]
fn test_create_basic_monorepo_fixture() {
    let fixture = create_basic_monorepo_fixture();
    assert_eq!(fixture.packages.len(), 2);

    let files = fixture.generate_files();
    assert_not_empty(&files.keys().collect::<Vec<_>>());
}

/// Tests single package fixture creation
#[test]
fn test_create_single_package_fixture() {
    let files = create_single_package_fixture("test-pkg", "1.0.0");
    assert_len(&files.keys().collect::<Vec<_>>(), 2);
    assert!(files.contains_key(&std::path::PathBuf::from("package.json")));
    assert!(files.contains_key(&std::path::PathBuf::from("index.js")));
}

/// Tests version assertion helpers
#[test]
fn test_version_assertions() {
    assert_version_eq("1.2.3", "1.2.3");
    assert_version_gt("1.2.3", "1.2.2");
    assert_version_gte("1.2.3", "1.2.3");
    assert_version_gte("1.2.3", "1.2.2");
}

/// Tests string assertion helpers
#[test]
fn test_string_assertions() {
    let text = "Hello, world!";
    assert_contains(text, "world");
    assert_not_contains(text, "goodbye");
}

/// Tests collection assertion helpers
#[test]
fn test_collection_assertions() {
    let vec = vec![1, 2, 3];
    assert_len(&vec, 3);
    assert_not_empty(&vec);

    let empty_vec: Vec<i32> = vec![];
    assert_empty(&empty_vec);
}

// Property-based tests demonstrating generators

proptest! {
    /// Tests that semver generator produces valid versions
    #[test]
    fn prop_test_semver_generation(version in semver_strategy()) {
        // All generated versions should be parseable
        let result = semver::Version::parse(&version);
        prop_assert!(result.is_ok(), "Generated invalid version: {}", version);
    }

    /// Tests that package name generator produces valid names
    #[test]
    fn prop_test_package_names(name in package_name_strategy()) {
        prop_assert!(!name.is_empty());
        prop_assert!(name.len() >= 3);
        // Package names should be lowercase
        prop_assert_eq!(name.to_lowercase(), name);
    }

    /// Tests that conventional commit generator produces valid commits
    #[test]
    fn prop_test_conventional_commits(message in conventional_commit_strategy()) {
        // All conventional commits should have a colon
        prop_assert!(message.contains(':'));
        // Should start with a valid type
        let types = ["feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert"];
        let starts_with_type = types.iter().any(|t| message.starts_with(t));
        prop_assert!(starts_with_type, "Invalid commit type in: {}", message);
    }

    /// Tests that commit hash generator produces valid hashes
    #[test]
    fn prop_test_commit_hashes(hash in commit_hash_strategy()) {
        prop_assert_eq!(hash.len(), 40);
        prop_assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Tests that version specs are properly formatted
    #[test]
    fn prop_test_version_specs(spec in version_spec_strategy()) {
        // Should start with a version prefix
        let prefixes = ["^", "~", ">=", "<=", "="];
        let has_prefix = prefixes.iter().any(|p| spec.starts_with(p));
        prop_assert!(has_prefix, "Invalid version spec: {}", spec);
    }

    /// Tests that author emails are valid
    #[test]
    fn prop_test_author_emails(email in author_email_strategy()) {
        prop_assert!(email.contains('@'));
        prop_assert!(email.contains('.'));
        let parts: Vec<&str> = email.split('@').collect();
        prop_assert_eq!(parts.len(), 2);
    }
}

/// Integration test combining multiple mock implementations
#[tokio::test]
async fn test_integration_mocks() {
    // Setup filesystem
    let fs = MockFileSystem::new();
    let package_json = PackageJsonBuilder::new("integration-test")
        .version("1.0.0")
        .add_dependency("react", "^18.0.0")
        .build();
    fs.write_file_string(std::path::Path::new("/package.json"), &package_json).await.unwrap();

    // Setup git repository
    let repo = MockGitRepository::new("/repo");
    repo.add_commit("abc123", "feat: initial commit", vec![]);
    repo.add_commit("def456", "fix: bug fix", vec![]);

    // Setup registry
    let registry = MockRegistry::new();
    registry.add_package("react", vec!["18.0.0", "18.2.0"]);

    // Verify everything works together
    assert!(fs.exists(std::path::Path::new("/package.json")).await);
    assert_eq!(repo.commit_count(), 2);
    assert_eq!(registry.package_count(), 1);

    let latest_react = registry.get_latest_version("react");
    assert_eq!(latest_react, Some("18.2.0".to_string()));
}

/// Test that writes a fixture to a temporary directory
#[test]
fn test_fixture_write_to_temp() {
    let temp = create_temp_dir().unwrap();
    let fixture = create_basic_monorepo_fixture();

    fixture.write_to_dir(temp.path()).unwrap();

    // Verify files were created
    let root_package = temp.path().join("package.json");
    assert_path_exists(&root_package);
    assert_is_file(&root_package);

    let pkg1 = temp.path().join("packages/pkg-a/package.json");
    assert_path_exists(&pkg1);
    assert_is_file(&pkg1);
}

/// Tests MockFileSystem with pre-populated files
#[tokio::test]
async fn test_mock_filesystem_with_files() {
    use std::collections::HashMap;
    use std::path::PathBuf;

    let mut files = HashMap::new();
    files.insert(PathBuf::from("/test1.txt"), "content1".to_string());
    files.insert(PathBuf::from("/test2.txt"), "content2".to_string());
    files.insert(PathBuf::from("/dir/test3.txt"), "content3".to_string());

    let fs = MockFileSystem::with_files(files);

    assert_eq!(fs.list_files().len(), 3);
    let content1 = fs.read_file_string(std::path::Path::new("/test1.txt")).await.unwrap();
    assert_eq!(content1, "content1");
}
