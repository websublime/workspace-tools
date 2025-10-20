//! Tests for the changes analysis module.
//!
//! These tests verify the functionality of the changes analysis system components,
//! including the analyzer, file mapping, and commit tracking.

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod analyzer_tests {
    use crate::changes::ChangesAnalyzer;
    use crate::config::PackageToolsConfig;
    use std::fs;
    use std::path::PathBuf;
    use sublime_git_tools::Repo;
    use sublime_standard_tools::filesystem::FileSystemManager;
    use tempfile::TempDir;

    /// Helper to create a test repository with a commit.
    async fn create_test_repo_with_commit() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();

        // Initialize git repository
        let repo = Repo::create(repo_path.to_str().unwrap()).unwrap();

        // Create package.json
        let package_json = r#"{
            "name": "test-package",
            "version": "1.0.0"
        }"#;
        fs::write(repo_path.join("package.json"), package_json).unwrap();

        // Stage and commit
        repo.add_all().unwrap();
        repo.commit("Initial commit").unwrap();

        (temp_dir, repo_path)
    }

    #[tokio::test]
    async fn test_analyzer_initialization_integration() {
        let (_temp, repo_path) = create_test_repo_with_commit().await;
        let git_repo = Repo::open(repo_path.to_str().unwrap()).unwrap();
        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        let analyzer = ChangesAnalyzer::new(repo_path, git_repo, fs, config).await;

        assert!(analyzer.is_ok());
    }

    #[tokio::test]
    async fn test_analyzer_with_monorepo_integration() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().to_path_buf();

        // Create monorepo structure
        let root_package = r#"{
            "name": "monorepo-root",
            "version": "1.0.0",
            "workspaces": ["packages/*"]
        }"#;
        fs::write(workspace_path.join("package.json"), root_package).unwrap();

        // Create pnpm-workspace.yaml for proper monorepo detection
        let pnpm_workspace = "packages:\n  - 'packages/*'\n";
        fs::write(workspace_path.join("pnpm-workspace.yaml"), pnpm_workspace).unwrap();

        // Create packages
        fs::create_dir_all(workspace_path.join("packages/pkg-a")).unwrap();
        let pkg_a = r#"{"name": "@test/pkg-a", "version": "1.0.0"}"#;
        fs::write(workspace_path.join("packages/pkg-a/package.json"), pkg_a).unwrap();

        // Initialize git
        let repo = Repo::create(workspace_path.to_str().unwrap()).unwrap();
        repo.add_all().unwrap();
        repo.commit("Initial monorepo").unwrap();

        let git_repo = Repo::open(workspace_path.to_str().unwrap()).unwrap();
        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        let analyzer =
            ChangesAnalyzer::new(workspace_path.clone(), git_repo, fs, config).await.unwrap();

        assert!(analyzer.is_monorepo());
        assert_eq!(analyzer.workspace_root(), workspace_path.as_path());
    }

    #[tokio::test]
    async fn test_analyzer_configuration_integration() {
        let (_temp, repo_path) = create_test_repo_with_commit().await;
        let git_repo = Repo::open(repo_path.to_str().unwrap()).unwrap();
        let fs = FileSystemManager::new();

        // Create custom configuration
        let mut config = PackageToolsConfig::default();
        config.changeset.path = ".custom-changesets".to_string();

        let analyzer = ChangesAnalyzer::new(repo_path, git_repo, fs, config).await.unwrap();

        assert_eq!(analyzer.config().changeset.path, ".custom-changesets");
    }
}

/// Tests for the PackageMapper functionality.
#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[allow(clippy::panic)]
mod mapping_tests {
    use crate::changes::mapping::PackageMapper;
    use crate::error::ChangesError;
    use std::path::PathBuf;
    use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
    use tempfile::TempDir;
    use tokio;

    /// Helper to create a test single-package workspace.
    async fn create_single_package_workspace() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let workspace_root = temp_dir.path().to_path_buf();

        let fs = FileSystemManager::new();

        // Create package.json
        let package_json = serde_json::json!({
            "name": "test-package",
            "version": "1.0.0"
        });

        fs.write_file_string(
            &workspace_root.join("package.json"),
            &serde_json::to_string_pretty(&package_json).expect("Failed to serialize"),
        )
        .await
        .expect("Failed to write package.json");

        // Create some test files
        fs.create_dir_all(&workspace_root.join("src")).await.expect("Failed to create src dir");
        fs.write_file_string(&workspace_root.join("src/index.ts"), "// test")
            .await
            .expect("Failed to write index.ts");

        (temp_dir, workspace_root)
    }

    /// Helper to create a test monorepo workspace.
    async fn create_monorepo_workspace() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let workspace_root = temp_dir.path().to_path_buf();

        let fs = FileSystemManager::new();

        // Create root package.json with workspaces
        let root_package_json = serde_json::json!({
            "name": "monorepo-root",
            "version": "1.0.0",
            "workspaces": ["packages/*"]
        });

        fs.write_file_string(
            &workspace_root.join("package.json"),
            &serde_json::to_string_pretty(&root_package_json).expect("Failed to serialize"),
        )
        .await
        .expect("Failed to write root package.json");

        // Create package-lock.json for npm workspaces detection
        fs.write_file_string(&workspace_root.join("package-lock.json"), "{}")
            .await
            .expect("Failed to write package-lock.json");

        // Create packages directory
        let packages_dir = workspace_root.join("packages");
        fs.create_dir_all(&packages_dir).await.expect("Failed to create packages dir");

        // Create package 1
        let pkg1_dir = packages_dir.join("core");
        fs.create_dir_all(&pkg1_dir).await.expect("Failed to create core package dir");
        let pkg1_json = serde_json::json!({
            "name": "@test/core",
            "version": "1.0.0"
        });
        fs.write_file_string(
            &pkg1_dir.join("package.json"),
            &serde_json::to_string_pretty(&pkg1_json).expect("Failed to serialize"),
        )
        .await
        .expect("Failed to write core package.json");
        fs.create_dir_all(&pkg1_dir.join("src")).await.expect("Failed to create core src dir");
        fs.write_file_string(&pkg1_dir.join("src/index.ts"), "// core")
            .await
            .expect("Failed to write core index.ts");

        // Create package 2
        let pkg2_dir = packages_dir.join("utils");
        fs.create_dir_all(&pkg2_dir).await.expect("Failed to create utils package dir");
        let pkg2_json = serde_json::json!({
            "name": "@test/utils",
            "version": "1.0.0"
        });
        fs.write_file_string(
            &pkg2_dir.join("package.json"),
            &serde_json::to_string_pretty(&pkg2_json).expect("Failed to serialize"),
        )
        .await
        .expect("Failed to write utils package.json");
        fs.create_dir_all(&pkg2_dir.join("src")).await.expect("Failed to create utils src dir");
        fs.write_file_string(&pkg2_dir.join("src/helper.ts"), "// utils")
            .await
            .expect("Failed to write utils helper.ts");

        (temp_dir, workspace_root)
    }

    #[tokio::test]
    async fn test_single_package_map_files() {
        let (_temp, workspace_root) = create_single_package_workspace().await;
        let fs = FileSystemManager::new();
        let mut mapper = PackageMapper::new(workspace_root.clone(), fs);

        let files = vec![PathBuf::from("src/index.ts"), PathBuf::from("package.json")];

        let result = mapper.map_files_to_packages(&files).await;
        assert!(result.is_ok());

        let package_files = result.expect("Expected Ok result");
        assert_eq!(package_files.len(), 1);
        assert!(package_files.contains_key("test-package"));
        assert_eq!(package_files["test-package"].len(), 2);
    }

    #[tokio::test]
    async fn test_single_package_find_package_for_file() {
        let (_temp, workspace_root) = create_single_package_workspace().await;
        let fs = FileSystemManager::new();
        let mut mapper = PackageMapper::new(workspace_root.clone(), fs);

        let file = PathBuf::from("src/index.ts");
        let result = mapper.find_package_for_file(&file).await;

        assert!(result.is_ok());
        let package_name = result.expect("Expected Ok result");
        assert_eq!(package_name, Some("test-package".to_string()));
    }

    #[tokio::test]
    async fn test_single_package_get_all_packages() {
        let (_temp, workspace_root) = create_single_package_workspace().await;
        let fs = FileSystemManager::new();
        let mut mapper = PackageMapper::new(workspace_root.clone(), fs);

        let result = mapper.get_all_packages().await;
        assert!(result.is_ok());

        let packages = result.expect("Expected Ok result");
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name(), "test-package");
    }

    #[tokio::test]
    async fn test_single_package_is_monorepo() {
        let (_temp, workspace_root) = create_single_package_workspace().await;
        let fs = FileSystemManager::new();
        let mut mapper = PackageMapper::new(workspace_root.clone(), fs);

        let result = mapper.is_monorepo().await;
        assert!(result.is_ok());
        assert!(!result.expect("Expected Ok result"));
    }

    #[tokio::test]
    async fn test_monorepo_map_files() {
        let (_temp, workspace_root) = create_monorepo_workspace().await;
        let fs = FileSystemManager::new();
        let mut mapper = PackageMapper::new(workspace_root.clone(), fs);

        let files = vec![
            PathBuf::from("packages/core/src/index.ts"),
            PathBuf::from("packages/utils/src/helper.ts"),
            PathBuf::from("README.md"), // Root file, should be omitted
        ];

        let result = mapper.map_files_to_packages(&files).await;
        assert!(result.is_ok());

        let package_files = result.expect("Expected Ok result");
        // Should have 2 packages (root file is omitted)
        assert_eq!(package_files.len(), 2);
        assert!(package_files.contains_key("@test/core"));
        assert!(package_files.contains_key("@test/utils"));
        assert_eq!(package_files["@test/core"].len(), 1);
        assert_eq!(package_files["@test/utils"].len(), 1);
    }

    #[tokio::test]
    async fn test_monorepo_find_package_for_file() {
        let (_temp, workspace_root) = create_monorepo_workspace().await;
        let fs = FileSystemManager::new();
        let mut mapper = PackageMapper::new(workspace_root.clone(), fs);

        // File in core package
        let file1 = PathBuf::from("packages/core/src/index.ts");
        let result1 = mapper.find_package_for_file(&file1).await;
        assert!(result1.is_ok());
        assert_eq!(result1.expect("Expected Ok"), Some("@test/core".to_string()));

        // File in utils package
        let file2 = PathBuf::from("packages/utils/src/helper.ts");
        let result2 = mapper.find_package_for_file(&file2).await;
        assert!(result2.is_ok());
        assert_eq!(result2.expect("Expected Ok"), Some("@test/utils".to_string()));

        // Root file
        let file3 = PathBuf::from("README.md");
        let result3 = mapper.find_package_for_file(&file3).await;
        assert!(result3.is_ok());
        assert_eq!(result3.expect("Expected Ok"), None);
    }

    #[tokio::test]
    async fn test_monorepo_get_all_packages() {
        let (_temp, workspace_root) = create_monorepo_workspace().await;
        let fs = FileSystemManager::new();
        let mut mapper = PackageMapper::new(workspace_root.clone(), fs);

        let result = mapper.get_all_packages().await;
        assert!(result.is_ok());

        let packages = result.expect("Expected Ok result");
        assert_eq!(packages.len(), 2);

        let package_names: Vec<_> = packages.iter().map(|p| p.name()).collect();
        assert!(package_names.contains(&"@test/core"));
        assert!(package_names.contains(&"@test/utils"));
    }

    #[tokio::test]
    async fn test_monorepo_is_monorepo() {
        let (_temp, workspace_root) = create_monorepo_workspace().await;
        let fs = FileSystemManager::new();
        let mut mapper = PackageMapper::new(workspace_root.clone(), fs);

        let result = mapper.is_monorepo().await;
        assert!(result.is_ok());
        assert!(result.expect("Expected Ok result"));
    }

    #[tokio::test]
    async fn test_cache_behavior() {
        let (_temp, workspace_root) = create_single_package_workspace().await;
        let fs = FileSystemManager::new();
        let mut mapper = PackageMapper::new(workspace_root.clone(), fs);

        let file = PathBuf::from("src/index.ts");

        // First call should cache
        let result1 = mapper.find_package_for_file(&file).await;
        assert!(result1.is_ok());

        // Check cache was populated
        assert!(mapper.file_cache.contains_key(&file));

        // Second call should use cache
        let result2 = mapper.find_package_for_file(&file).await;
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());

        // Clear cache
        mapper.clear_cache();
        assert!(mapper.file_cache.is_empty());
        assert!(mapper.cached_monorepo.is_none());
    }

    #[tokio::test]
    async fn test_normalize_path_relative() {
        let (_temp, workspace_root) = create_single_package_workspace().await;
        let fs = FileSystemManager::new();
        let mapper = PackageMapper::new(workspace_root.clone(), fs);

        let relative_path = PathBuf::from("src/index.ts");
        let result = mapper.normalize_path(&relative_path);

        assert!(result.is_ok());
        assert_eq!(result.expect("Expected Ok"), relative_path);
    }

    #[tokio::test]
    async fn test_normalize_path_absolute() {
        let (_temp, workspace_root) = create_single_package_workspace().await;
        let fs = FileSystemManager::new();
        let mapper = PackageMapper::new(workspace_root.clone(), fs);

        let absolute_path = workspace_root.join("src/index.ts");
        let result = mapper.normalize_path(&absolute_path);

        assert!(result.is_ok());
        assert_eq!(result.expect("Expected Ok"), PathBuf::from("src/index.ts"));
    }

    #[tokio::test]
    async fn test_normalize_path_outside_workspace() {
        let (_temp, workspace_root) = create_single_package_workspace().await;
        let fs = FileSystemManager::new();
        let mapper = PackageMapper::new(workspace_root.clone(), fs);

        // Create an absolute path that's definitely outside the workspace
        // Use a different temp directory to ensure it's outside
        let outside_temp = TempDir::new().expect("Failed to create outside temp dir");
        let outside_path = outside_temp.path().join("outside/file.ts");

        let result = mapper.normalize_path(&outside_path);

        assert!(result.is_err());
        if let Err(ChangesError::FileOutsideWorkspace { .. }) = result {
            // Expected error
        } else {
            panic!("Expected FileOutsideWorkspace error");
        }
    }

    #[tokio::test]
    async fn test_empty_file_list() {
        let (_temp, workspace_root) = create_single_package_workspace().await;
        let fs = FileSystemManager::new();
        let mut mapper = PackageMapper::new(workspace_root.clone(), fs);

        let files: Vec<PathBuf> = vec![];
        let result = mapper.map_files_to_packages(&files).await;

        assert!(result.is_ok());
        let package_files = result.expect("Expected Ok result");
        assert!(package_files.is_empty());
    }
}
