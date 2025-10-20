//! Integration tests for the changes analysis module.
//!
//! These tests verify the integration between different components of the changes
//! analysis system, including the analyzer, file mapping, and commit tracking.

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod integration_tests {
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
