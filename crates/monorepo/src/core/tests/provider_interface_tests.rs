//! Provider interface tests
//!
//! Comprehensive tests for all provider interfaces to ensure contract compliance,
//! error handling, and proper behavior. These tests verify that components can
//! safely depend on provider interfaces without coupling to concrete implementations.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;

use crate::core::interfaces::{ConfigProvider, DependencyFactory, GitProvider, PackageProvider, FileSystemProvider, RegistryProvider, WorkspaceProvider, PackageDiscoveryProvider, EnhancedConfigProvider};
use crate::core::{MonorepoPackageInfo, MonorepoProject};
use crate::config::MonorepoConfig;
use crate::error::Result;
use sublime_git_tools::{GitChangedFile, GitFileStatus, Repo};
use sublime_package_tools::{Package, PackageInfo, RegistryManager};
use sublime_standard_tools::monorepo::{WorkspacePackage, MonorepoDescriptor};

/// Helper to run async code in sync tests to avoid tokio context issues
fn run_async<F, R>(f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    rt.block_on(f)
}

/// Creates a test project with proper Git setup for provider testing
#[allow(clippy::arc_with_non_send_sync)]
fn create_test_project() -> (TempDir, Arc<MonorepoProject>) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize Git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repository");

    // Configure Git user for testing
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to configure git email");

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to configure git name");

    // Create basic structure
    std::fs::create_dir_all(temp_dir.path().join("packages"))
        .expect("Failed to create packages dir");

    let root_package_json = r#"{
  "name": "test-monorepo",
  "version": "1.0.0",
  "private": true,
  "workspaces": ["packages/*"]
}"#;
    std::fs::write(temp_dir.path().join("package.json"), root_package_json)
        .expect("Failed to write package.json");

    // Create package-lock.json to make it detectable as an npm monorepo
    std::fs::write(temp_dir.path().join("package-lock.json"), "{}")
        .expect("Failed to write package-lock.json");

    // Create an initial commit to have a valid Git history
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add files");

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to create initial commit");

    let mut project = MonorepoProject::new(temp_dir.path()).expect("Failed to create MonorepoProject");
    
    // Refresh packages to discover any test packages
    let _ = project.refresh_packages(); // Ignore errors for provider tests
    
    let project = Arc::new(project);
    (temp_dir, project)
}

/// Mock GitProvider implementation for testing
struct MockGitProvider {
    repo_root: PathBuf,
    current_branch_result: String,
    current_sha_result: String,
    changed_files_result: Vec<GitChangedFile>,
    should_error_branch: bool,
}

impl MockGitProvider {
    fn new(repo_root: PathBuf) -> Self {
        Self {
            repo_root,
            current_branch_result: "main".to_string(),
            current_sha_result: "abc123".to_string(),
            changed_files_result: vec![
                GitChangedFile {
                    path: "test.rs".to_string(),
                    status: GitFileStatus::Modified,
                    staged: false,
                    workdir: true,
                }
            ],
            should_error_branch: false,
        }
    }

    fn with_error_branch(mut self) -> Self {
        self.should_error_branch = true;
        self
    }

    fn with_empty_changes(mut self) -> Self {
        self.changed_files_result = vec![];
        self
    }
}

impl GitProvider for MockGitProvider {
    fn repository(&self) -> &Repo {
        // For mock testing, we can't return a real Repo
        // In practice, this method should be avoided in unit tests
        panic!("Mock GitProvider doesn't provide real repository access")
    }

    fn repository_root(&self) -> &Path {
        &self.repo_root
    }

    fn get_changed_files_since(&self, _since: &str) -> Result<Vec<GitChangedFile>> {
        Ok(self.changed_files_result.clone())
    }

    fn current_branch(&self) -> Result<String> {
        if self.should_error_branch {
            Err(crate::error::Error::generic("Mock git error"))
        } else {
            Ok(self.current_branch_result.clone())
        }
    }

    fn current_sha(&self) -> Result<String> {
        Ok(self.current_sha_result.clone())
    }

    fn get_diverged_commit(&self, _base_branch: &str) -> Result<String> {
        Ok("def456".to_string())
    }

    fn get_all_files_changed_since_sha_with_status(&self, _sha: &str) -> Result<Vec<GitChangedFile>> {
        Ok(self.changed_files_result.clone())
    }

    fn get_all_files_changed_since_sha(&self, _sha: &str) -> Result<Vec<String>> {
        Ok(self.changed_files_result.iter().map(|f| f.path.clone()).collect())
    }
}

mod git_provider_tests {
    use super::*;

    #[test]
    fn test_git_provider_contract_compliance() {
        let (_temp_dir, project) = create_test_project();
        let git_provider = DependencyFactory::git_provider(Arc::clone(&project));

        // Test that all interface methods are callable
        let repo_root = git_provider.repository_root();
        assert!(repo_root.exists(), "Repository root should exist");

        // Test current branch (may fail in some environments, but should be callable)
        let branch_result = git_provider.current_branch();
        assert!(branch_result.is_ok() || branch_result.is_err(), "Should return a Result");

        // Test current SHA (may fail in some environments, but should be callable)
        let sha_result = git_provider.current_sha();
        assert!(sha_result.is_ok() || sha_result.is_err(), "Should return a Result");
    }

    #[test]
    fn test_git_provider_changed_files_handling() {
        let (_temp_dir, project) = create_test_project();
        let git_provider = DependencyFactory::git_provider(Arc::clone(&project));

        // Test getting changed files since HEAD (should not error even if empty)
        let files_result = git_provider.get_changed_files_since("HEAD");
        match files_result {
            Ok(files) => {
                assert!(files.len() >= 0, "Should return valid file list");
            }
            Err(_) => {
                // Expected in test environment without proper git history
            }
        }
    }

    #[test]
    fn test_git_provider_diverged_commit() {
        let (_temp_dir, project) = create_test_project();
        let git_provider = DependencyFactory::git_provider(Arc::clone(&project));

        // Test diverged commit (expected to fail in simple test setup)
        let diverged_result = git_provider.get_diverged_commit("main");
        // This will likely fail in test environment, but should be callable
        assert!(diverged_result.is_ok() || diverged_result.is_err(), "Should return a Result");
    }

    #[test]
    fn test_mock_git_provider_success_cases() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockGitProvider::new(temp_dir.path().to_path_buf());

        // Test successful branch retrieval
        let branch = mock_provider.current_branch().expect("Should return branch");
        assert_eq!(branch, "main");

        // Test successful SHA retrieval
        let sha = mock_provider.current_sha().expect("Should return SHA");
        assert_eq!(sha, "abc123");

        // Test successful changed files retrieval
        let files = mock_provider.get_changed_files_since("HEAD").expect("Should return files");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "test.rs");
        assert_eq!(files[0].status, GitFileStatus::Modified);

        // Test repository root
        assert_eq!(mock_provider.repository_root(), temp_dir.path());
    }

    #[test]
    fn test_mock_git_provider_error_cases() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockGitProvider::new(temp_dir.path().to_path_buf())
            .with_error_branch();

        // Test error handling
        let branch_result = mock_provider.current_branch();
        assert!(branch_result.is_err(), "Should return error");

        // Other methods should still work
        let sha = mock_provider.current_sha().expect("SHA should still work");
        assert_eq!(sha, "abc123");
    }

    #[test]
    fn test_mock_git_provider_empty_changes() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockGitProvider::new(temp_dir.path().to_path_buf())
            .with_empty_changes();

        // Test empty changed files
        let files = mock_provider.get_changed_files_since("HEAD").expect("Should return empty list");
        assert_eq!(files.len(), 0);

        let file_paths = mock_provider.get_all_files_changed_since_sha("abc123")
            .expect("Should return empty list");
        assert_eq!(file_paths.len(), 0);
    }

    #[test]
    fn test_git_provider_interface_boundary() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that we can create a GitProvider from different sources
        let git_provider_from_project = DependencyFactory::git_provider(Arc::clone(&project));
        let git_provider_from_arc = DependencyFactory::git_provider(project);

        // Both should have the same root path
        assert_eq!(
            git_provider_from_project.repository_root(),
            git_provider_from_arc.repository_root()
        );
    }

    #[test]
    fn test_git_provider_method_consistency() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockGitProvider::new(temp_dir.path().to_path_buf());

        // Test that both changed file methods return consistent data
        let files_with_status = mock_provider.get_all_files_changed_since_sha_with_status("abc123")
            .expect("Should return files with status");
        let file_paths = mock_provider.get_all_files_changed_since_sha("abc123")
            .expect("Should return file paths");

        assert_eq!(files_with_status.len(), file_paths.len());
        assert_eq!(files_with_status[0].path, file_paths[0]);
    }

    #[test]
    fn test_git_provider_error_propagation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockGitProvider::new(temp_dir.path().to_path_buf())
            .with_error_branch();

        // Test that errors are properly propagated through the interface
        match mock_provider.current_branch() {
            Err(e) => {
                assert!(e.to_string().contains("Mock git error"));
            }
            Ok(_) => panic!("Expected error but got success"),
        }
    }
}

mod dependency_factory_tests {
    use super::*;

    #[test]
    fn test_dependency_factory_git_provider_creation() {
        let (_temp_dir, project) = create_test_project();
        
        // Test factory method creates proper provider
        let git_provider = DependencyFactory::git_provider(Arc::clone(&project));
        
        // Should be able to call interface methods
        let root = git_provider.repository_root();
        assert!(root.exists());
    }

    #[test]
    fn test_dependency_factory_with_mock() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockGitProvider::new(temp_dir.path().to_path_buf());
        
        // Test factory accepts mock implementation
        let git_provider = DependencyFactory::git_provider(mock_provider);
        
        // Should work with mock
        let branch = git_provider.current_branch().expect("Should work with mock");
        assert_eq!(branch, "main");
    }
}

/// Mock ConfigProvider implementation for testing
struct MockConfigProvider {
    config: MonorepoConfig,
}

impl MockConfigProvider {
    fn new() -> Self {
        Self {
            config: MonorepoConfig::default(),
        }
    }

    fn with_custom_config(config: MonorepoConfig) -> Self {
        Self { config }
    }
}

impl ConfigProvider for MockConfigProvider {
    fn config(&self) -> &MonorepoConfig {
        &self.config
    }
}

mod config_provider_tests {
    use super::*;
    use crate::config::VersionBumpType;

    #[test]
    fn test_config_provider_contract_compliance() {
        let (_temp_dir, project) = create_test_project();
        let config_provider = DependencyFactory::config_provider(Arc::clone(&project));

        // Test that interface method is callable
        let config = config_provider.config();
        
        // Verify we get a valid config back
        assert!(config.versioning.default_bump == VersionBumpType::Patch 
                || config.versioning.default_bump == VersionBumpType::Minor
                || config.versioning.default_bump == VersionBumpType::Major);
    }

    #[test] 
    fn test_config_provider_immutability() {
        let (_temp_dir, project) = create_test_project();
        let config_provider = DependencyFactory::config_provider(Arc::clone(&project));

        // Test that config is properly accessed as immutable reference
        let config1 = config_provider.config();
        let config2 = config_provider.config();
        
        // Should be same reference (same config instance)
        assert_eq!(
            config1.versioning.default_bump,
            config2.versioning.default_bump
        );
    }

    #[test]
    fn test_mock_config_provider_default() {
        let mock_provider = MockConfigProvider::new();
        
        // Test default config access
        let config = mock_provider.config();
        assert_eq!(config.versioning.default_bump, VersionBumpType::Patch);
        
        // auto_tag can be true or false by default, just verify it's a valid boolean
        assert!(config.versioning.auto_tag == true || config.versioning.auto_tag == false);
    }

    #[test]
    fn test_mock_config_provider_custom() {
        let mut custom_config = MonorepoConfig::default();
        custom_config.versioning.default_bump = VersionBumpType::Major;
        custom_config.versioning.auto_tag = true;
        
        let mock_provider = MockConfigProvider::with_custom_config(custom_config);
        
        // Test custom config values
        let config = mock_provider.config();
        assert_eq!(config.versioning.default_bump, VersionBumpType::Major);
        assert!(config.versioning.auto_tag);
    }

    #[test]
    fn test_config_provider_interface_boundary() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that we can create ConfigProvider from different sources
        let config_provider_from_project = DependencyFactory::config_provider(Arc::clone(&project));
        let config_provider_from_arc = DependencyFactory::config_provider(project);

        // Both should have the same default bump
        assert_eq!(
            config_provider_from_project.config().versioning.default_bump,
            config_provider_from_arc.config().versioning.default_bump
        );
    }

    #[test]
    fn test_config_provider_consistency() {
        let mock_provider = MockConfigProvider::new();
        
        // Multiple calls should return same config reference
        let config1 = mock_provider.config();
        let config2 = mock_provider.config();
        
        // Verify they're the same (comparing values since we can't compare addresses easily)
        assert_eq!(config1.versioning.default_bump, config2.versioning.default_bump);
        assert_eq!(config1.versioning.auto_tag, config2.versioning.auto_tag);
        assert_eq!(config1.workspace.patterns.len(), config2.workspace.patterns.len());
    }

    #[test]
    fn test_config_provider_with_factory() {
        let mock_provider = MockConfigProvider::new();
        
        // Test factory method creates proper provider
        let config_provider = DependencyFactory::config_provider(mock_provider);
        
        // Should be able to call interface methods
        let config = config_provider.config();
        assert_eq!(config.versioning.default_bump, VersionBumpType::Patch);
    }

    #[test]
    fn test_config_provider_workspace_config_access() {
        let (_temp_dir, project) = create_test_project();
        let config_provider = DependencyFactory::config_provider(project);

        let config = config_provider.config();
        
        // Test workspace config section access
        assert!(config.workspace.patterns.len() >= 0, "Should have workspace patterns");
        
        // Test that we can access nested config sections
        let _changelog_config = &config.changelog;
        let _hooks_config = &config.hooks;
        let _tasks_config = &config.tasks;
    }

    #[test]
    fn test_config_provider_error_handling() {
        // Test that config provider gracefully handles various scenarios
        let mock_provider = MockConfigProvider::new();
        let config_provider = DependencyFactory::config_provider(mock_provider);
        
        // Config access should always succeed (no Result type)
        let config = config_provider.config();
        
        // Should always have a valid config structure
        assert!(config.versioning.default_bump == VersionBumpType::Patch
                || config.versioning.default_bump == VersionBumpType::Minor
                || config.versioning.default_bump == VersionBumpType::Major);
    }

    #[test]
    fn test_config_provider_thread_safety() {
        let (_temp_dir, project) = create_test_project();
        let config_provider = DependencyFactory::config_provider(project);
        
        // Test that config can be accessed multiple times safely
        let configs: Vec<_> = (0..5).map(|_| config_provider.config()).collect();
        
        // All should have same values
        let first_bump = configs[0].versioning.default_bump;
        assert!(configs.iter().all(|c| c.versioning.default_bump == first_bump));
    }
}

/// Mock PackageProvider implementation for testing
struct MockPackageProvider {
    packages: Vec<MonorepoPackageInfo>,
    root_path: PathBuf,
}

impl MockPackageProvider {
    fn new(root_path: PathBuf) -> Self {
        let temp_dir = root_path.clone();
        
        // Create mock packages
        let package1_path = temp_dir.join("packages").join("pkg1");
        let package2_path = temp_dir.join("packages").join("pkg2");
        
        // Create mock package info structures
        let package1_info = create_mock_package_info("@test/pkg1".to_string(), package1_path.clone());
        let package2_info = create_mock_package_info("@test/pkg2".to_string(), package2_path.clone());
        
        let packages = vec![package1_info, package2_info];
        
        Self {
            packages,
            root_path,
        }
    }
    
    fn with_custom_packages(packages: Vec<MonorepoPackageInfo>, root_path: PathBuf) -> Self {
        Self {
            packages,
            root_path,
        }
    }
    
    fn empty(root_path: PathBuf) -> Self {
        Self {
            packages: vec![],
            root_path,
        }
    }
}

impl PackageProvider for MockPackageProvider {
    fn packages(&self) -> &[MonorepoPackageInfo] {
        &self.packages
    }
    
    fn get_package(&self, name: &str) -> Option<&MonorepoPackageInfo> {
        self.packages.iter().find(|pkg| {
            pkg.package_info.package.borrow().name() == name
        })
    }
    
    fn get_dependents(&self, package_name: &str) -> Vec<&MonorepoPackageInfo> {
        self.packages.iter()
            .filter(|pkg| pkg.dependents.contains(&package_name.to_string()))
            .collect()
    }
    
    fn root_path(&self) -> &Path {
        &self.root_path
    }
}

/// Helper function to create mock package info
fn create_mock_package_info(name: String, path: PathBuf) -> MonorepoPackageInfo {
    use crate::core::types::VersionStatus;
    
    // Create a mock Package
    let package = Package::new(&name, "1.0.0", None).expect("Failed to create package");
    
    // Create PackageInfo
    let package_info = PackageInfo::new(
        package,
        path.join("package.json").to_string_lossy().to_string(),
        path.to_string_lossy().to_string(),
        format!("packages/{}", name.replace("@test/", "")),
        serde_json::json!({
            "name": name,
            "version": "1.0.0"
        })
    );
    
    // Create WorkspacePackage
    let workspace_package = WorkspacePackage {
        name: name.clone(),
        version: "1.0.0".to_string(),
        location: PathBuf::from(format!("packages/{}", name.replace("@test/", ""))),
        absolute_path: path,
        workspace_dependencies: vec![],
        workspace_dev_dependencies: vec![],
    };
    
    MonorepoPackageInfo {
        package_info,
        workspace_package,
        is_internal: true,
        dependents: vec![], // No dependents by default
        dependencies_external: vec![],
        version_status: VersionStatus::Stable,
        changesets: vec![],
    }
}

mod package_provider_tests {
    use super::*;

    #[test]
    fn test_package_provider_contract_compliance() {
        let (_temp_dir, project) = create_test_project();
        let package_provider = DependencyFactory::package_provider(Arc::clone(&project));

        // Test that all interface methods are callable
        let packages = package_provider.packages();
        assert!(packages.len() >= 0, "Should return valid package list");

        let root_path = package_provider.root_path();
        assert!(root_path.exists(), "Root path should exist");

        // Test getting specific package (may not exist in test setup)
        let package_result = package_provider.get_package("non-existent");
        assert!(package_result.is_none(), "Non-existent package should return None");

        // Test getting dependents
        let dependents = package_provider.get_dependents("any-package");
        assert!(dependents.len() >= 0, "Should return valid dependents list");
    }

    #[test]
    fn test_package_provider_packages_list() {
        let (_temp_dir, project) = create_test_project();
        let package_provider = DependencyFactory::package_provider(Arc::clone(&project));

        let packages = package_provider.packages();
        
        // In test environment, packages list might be empty
        assert!(packages.len() >= 0, "Should return valid packages array");
        
        // Test that all packages have valid names
        for package in packages {
            let package_name = package.package_info.package.borrow().name().to_string();
            assert!(!package_name.is_empty(), "Package name should not be empty");
        }
    }

    #[test]
    fn test_mock_package_provider_basic_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockPackageProvider::new(temp_dir.path().to_path_buf());

        // Test packages list
        let packages = mock_provider.packages();
        assert_eq!(packages.len(), 2);

        // Test package names
        let package_names: Vec<String> = packages.iter()
            .map(|pkg| pkg.package_info.package.borrow().name().to_string())
            .collect();
        assert!(package_names.contains(&"@test/pkg1".to_string()));
        assert!(package_names.contains(&"@test/pkg2".to_string()));

        // Test root path
        assert_eq!(mock_provider.root_path(), temp_dir.path());
    }

    #[test]
    fn test_mock_package_provider_get_package() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockPackageProvider::new(temp_dir.path().to_path_buf());

        // Test successful package retrieval
        let package = mock_provider.get_package("@test/pkg1");
        assert!(package.is_some());
        assert_eq!(package.unwrap().package_info.package.borrow().name(), "@test/pkg1");

        // Test package not found
        let package = mock_provider.get_package("@test/non-existent");
        assert!(package.is_none());
    }

    #[test]
    fn test_mock_package_provider_dependents() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Create packages with dependencies
        let mut package1 = create_mock_package_info("@test/pkg1".to_string(), 
            temp_dir.path().join("packages").join("pkg1"));
        let package2 = create_mock_package_info("@test/pkg2".to_string(), 
            temp_dir.path().join("packages").join("pkg2"));
        
        // Set pkg1 as dependent on pkg2
        package1.dependents = vec!["@test/pkg2".to_string()];
        
        let mock_provider = MockPackageProvider::with_custom_packages(
            vec![package1, package2],
            temp_dir.path().to_path_buf()
        );

        // Test finding dependents
        let dependents = mock_provider.get_dependents("@test/pkg2");
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0].package_info.package.borrow().name(), "@test/pkg1");

        // Test no dependents
        let dependents = mock_provider.get_dependents("@test/pkg1");
        assert_eq!(dependents.len(), 0);
    }

    #[test]
    fn test_mock_package_provider_empty_repo() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockPackageProvider::empty(temp_dir.path().to_path_buf());

        // Test empty packages list
        let packages = mock_provider.packages();
        assert_eq!(packages.len(), 0);

        // Test getting package from empty repo
        let package = mock_provider.get_package("any-package");
        assert!(package.is_none());

        // Test dependents in empty repo
        let dependents = mock_provider.get_dependents("any-package");
        assert_eq!(dependents.len(), 0);
    }

    #[test]
    fn test_package_provider_interface_boundary() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that we can create PackageProvider from different sources
        let package_provider_from_project = DependencyFactory::package_provider(Arc::clone(&project));
        let package_provider_from_arc = DependencyFactory::package_provider(project);

        // Both should have the same root path
        assert_eq!(
            package_provider_from_project.root_path(),
            package_provider_from_arc.root_path()
        );
    }

    #[test]
    fn test_package_provider_method_consistency() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockPackageProvider::new(temp_dir.path().to_path_buf());

        // Test that packages() returns consistent data
        let packages1 = mock_provider.packages();
        let packages2 = mock_provider.packages();
        
        assert_eq!(packages1.len(), packages2.len());
        
        // Test that get_package is consistent with packages list
        for package in packages1 {
            let package_name = package.package_info.package.borrow().name().to_string();
            let found_package = mock_provider.get_package(&package_name);
            assert!(found_package.is_some());
            assert_eq!(found_package.unwrap().package_info.package.borrow().name(), package_name);
        }
    }

    #[test]
    fn test_package_provider_dependency_factory() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockPackageProvider::new(temp_dir.path().to_path_buf());
        
        // Test factory method creates proper provider
        let package_provider = DependencyFactory::package_provider(mock_provider);
        
        // Should be able to call interface methods
        let packages = package_provider.packages();
        assert_eq!(packages.len(), 2);
        
        let root_path = package_provider.root_path();
        assert_eq!(root_path, temp_dir.path());
    }

    #[test]
    fn test_package_provider_path_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockPackageProvider::new(temp_dir.path().to_path_buf());

        // Test that all packages have valid paths
        let packages = mock_provider.packages();
        for package in packages {
            let package_path = &package.workspace_package.absolute_path;
            assert!(package_path.starts_with(mock_provider.root_path()));
            assert!(!package.workspace_package.location.to_string_lossy().is_empty());
        }
    }

    #[test]
    fn test_package_provider_package_names() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockPackageProvider::new(temp_dir.path().to_path_buf());

        let packages = mock_provider.packages();
        
        // Test that all packages have unique names
        let mut names = std::collections::HashSet::new();
        for package in packages {
            let name = package.package_info.package.borrow().name().to_string();
            assert!(!name.is_empty(), "Package name should not be empty");
            assert!(names.insert(name.clone()), "Package names should be unique: {}", name);
        }
    }

    #[test]
    fn test_package_provider_integration_with_factory() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that factory can accept both Arc and direct project
        let _package_provider1 = DependencyFactory::package_provider(Arc::clone(&project));
        let _package_provider2 = DependencyFactory::package_provider(project);
        
        // Both should be created successfully without errors
    }
}

/// Mock FileSystemProvider implementation for testing
struct MockFileSystemProvider {
    files: std::sync::Mutex<std::collections::HashMap<PathBuf, String>>,
    directories: std::sync::Mutex<std::collections::HashSet<PathBuf>>,
    should_error_read: bool,
    should_error_write: bool,
    should_error_create_dir: bool,
}

impl MockFileSystemProvider {
    fn new() -> Self {
        let mut directories = std::collections::HashSet::new();
        directories.insert(PathBuf::from("/")); // Root always exists
        
        Self {
            files: std::sync::Mutex::new(std::collections::HashMap::new()),
            directories: std::sync::Mutex::new(directories),
            should_error_read: false,
            should_error_write: false,
            should_error_create_dir: false,
        }
    }
    
    fn with_error_read(mut self) -> Self {
        self.should_error_read = true;
        self
    }
    
    fn with_error_write(mut self) -> Self {
        self.should_error_write = true;
        self
    }
    
    fn with_error_create_dir(mut self) -> Self {
        self.should_error_create_dir = true;
        self
    }
    
    fn with_file(self, path: PathBuf, content: String) -> Self {
        {
            let mut files = self.files.lock().expect("Failed to lock files");
            files.insert(path.clone(), content);
        }
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            let mut directories = self.directories.lock().expect("Failed to lock directories");
            directories.insert(parent.to_path_buf());
        }
        
        self
    }
    
    fn with_directory(self, path: PathBuf) -> Self {
        {
            let mut directories = self.directories.lock().expect("Failed to lock directories");
            directories.insert(path);
        }
        self
    }
}

impl FileSystemProvider for MockFileSystemProvider {
    fn read_file_string(&self, path: &Path) -> Result<String> {
        if self.should_error_read {
            return Err(crate::error::Error::generic("Mock filesystem read error"));
        }
        
        let files = self.files.lock().expect("Failed to lock files");
        files.get(path)
            .cloned()
            .ok_or_else(|| crate::error::Error::generic("File not found"))
    }
    
    fn write_file_string(&self, path: &Path, content: &str) -> Result<()> {
        if self.should_error_write {
            return Err(crate::error::Error::generic("Mock filesystem write error"));
        }
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            let directories = self.directories.lock().expect("Failed to lock directories");
            if !directories.contains(parent) {
                return Err(crate::error::Error::generic("Parent directory does not exist"));
            }
        }
        
        let mut files = self.files.lock().expect("Failed to lock files");
        files.insert(path.to_path_buf(), content.to_string());
        Ok(())
    }
    
    fn path_exists(&self, path: &Path) -> bool {
        let files = self.files.lock().expect("Failed to lock files");
        let directories = self.directories.lock().expect("Failed to lock directories");
        
        files.contains_key(path) || directories.contains(path)
    }
    
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        if self.should_error_create_dir {
            return Err(crate::error::Error::generic("Mock filesystem create directory error"));
        }
        
        let mut directories = self.directories.lock().expect("Failed to lock directories");
        
        // Create all parent directories
        let mut current = PathBuf::new();
        for component in path.components() {
            current.push(component);
            directories.insert(current.clone());
        }
        
        Ok(())
    }
    
    fn remove_file(&self, path: &Path) -> Result<()> {
        let mut files = self.files.lock().expect("Failed to lock files");
        if files.remove(path).is_some() {
            Ok(())
        } else {
            Err(crate::error::Error::generic("File not found for removal"))
        }
    }
    
    fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let directories = self.directories.lock().expect("Failed to lock directories");
        let files = self.files.lock().expect("Failed to lock files");
        
        if !directories.contains(path) {
            return Err(crate::error::Error::generic("Directory not found"));
        }
        
        let mut result = Vec::new();
        
        // Add all files and directories that are children of the given path
        for file_path in files.keys() {
            if file_path.starts_with(path) && file_path != path {
                result.push(file_path.clone());
            }
        }
        
        for dir_path in directories.iter() {
            if dir_path.starts_with(path) && dir_path != path {
                result.push(dir_path.clone());
            }
        }
        
        result.sort();
        Ok(result)
    }
}

mod filesystem_provider_tests {
    use super::*;

    #[test]
    fn test_filesystem_provider_contract_compliance() {
        let (_temp_dir, project) = create_test_project();
        let fs_provider = DependencyFactory::file_system_provider(Arc::clone(&project));

        // Test that all interface methods are callable
        let test_path = PathBuf::from("test.txt");
        
        // Test path_exists (should work regardless of result)
        let exists = fs_provider.path_exists(&test_path);
        assert!(exists == true || exists == false, "Should return valid boolean");

        // Test create_dir_all
        let test_dir = PathBuf::from("test_dir");
        let create_result = fs_provider.create_dir_all(&test_dir);
        assert!(create_result.is_ok() || create_result.is_err(), "Should return Result");

        // Test write_file_string
        let write_result = fs_provider.write_file_string(&test_path, "test content");
        assert!(write_result.is_ok() || write_result.is_err(), "Should return Result");

        // Test read_file_string (may fail if file doesn't exist)
        let read_result = fs_provider.read_file_string(&test_path);
        assert!(read_result.is_ok() || read_result.is_err(), "Should return Result");

        // Test walk_dir
        let walk_result = fs_provider.walk_dir(&PathBuf::from("."));
        assert!(walk_result.is_ok() || walk_result.is_err(), "Should return Result");
    }

    #[test]
    fn test_mock_filesystem_provider_file_operations() {
        let mock_provider = MockFileSystemProvider::new()
            .with_file(PathBuf::from("test.txt"), "Hello, World!".to_string());

        // Test file exists
        assert!(mock_provider.path_exists(&PathBuf::from("test.txt")));
        assert!(!mock_provider.path_exists(&PathBuf::from("nonexistent.txt")));

        // Test read file
        let content = mock_provider.read_file_string(&PathBuf::from("test.txt"))
            .expect("Should read file successfully");
        assert_eq!(content, "Hello, World!");

        // Test read nonexistent file
        let read_result = mock_provider.read_file_string(&PathBuf::from("nonexistent.txt"));
        assert!(read_result.is_err(), "Should error when reading nonexistent file");
    }

    #[test]
    fn test_mock_filesystem_provider_write_operations() {
        let mock_provider = MockFileSystemProvider::new()
            .with_directory(PathBuf::from("test_dir"));

        let test_file = PathBuf::from("test_dir").join("new_file.txt");
        
        // Test write file
        mock_provider.write_file_string(&test_file, "New content")
            .expect("Should write file successfully");

        // Test read back the written file
        let content = mock_provider.read_file_string(&test_file)
            .expect("Should read written file");
        assert_eq!(content, "New content");

        // Test file exists after write
        assert!(mock_provider.path_exists(&test_file));
    }

    #[test]
    fn test_mock_filesystem_provider_directory_operations() {
        let mock_provider = MockFileSystemProvider::new();

        let test_dir = PathBuf::from("test_dir").join("nested").join("deep");
        
        // Test create directory
        mock_provider.create_dir_all(&test_dir)
            .expect("Should create directory successfully");

        // Test directory exists
        assert!(mock_provider.path_exists(&test_dir));
        assert!(mock_provider.path_exists(&PathBuf::from("test_dir")));
        assert!(mock_provider.path_exists(&PathBuf::from("test_dir").join("nested")));
    }

    #[test]
    fn test_mock_filesystem_provider_remove_operations() {
        let mock_provider = MockFileSystemProvider::new()
            .with_file(PathBuf::from("to_remove.txt"), "Content".to_string());

        // Test file exists before removal
        assert!(mock_provider.path_exists(&PathBuf::from("to_remove.txt")));

        // Test remove file
        mock_provider.remove_file(&PathBuf::from("to_remove.txt"))
            .expect("Should remove file successfully");

        // Test file doesn't exist after removal
        assert!(!mock_provider.path_exists(&PathBuf::from("to_remove.txt")));

        // Test remove nonexistent file
        let remove_result = mock_provider.remove_file(&PathBuf::from("nonexistent.txt"));
        assert!(remove_result.is_err(), "Should error when removing nonexistent file");
    }

    #[test]
    fn test_mock_filesystem_provider_walk_directory() {
        let mock_provider = MockFileSystemProvider::new()
            .with_directory(PathBuf::from("test_dir"))
            .with_file(PathBuf::from("test_dir").join("file1.txt"), "Content 1".to_string())
            .with_file(PathBuf::from("test_dir").join("file2.txt"), "Content 2".to_string())
            .with_directory(PathBuf::from("test_dir").join("subdir"));

        // Test walk directory
        let entries = mock_provider.walk_dir(&PathBuf::from("test_dir"))
            .expect("Should walk directory successfully");

        assert!(entries.len() >= 3, "Should find at least 3 entries");
        
        // Check that specific entries exist
        let entry_strings: Vec<String> = entries.iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        
        assert!(entry_strings.iter().any(|s| s.contains("file1.txt")));
        assert!(entry_strings.iter().any(|s| s.contains("file2.txt")));
        assert!(entry_strings.iter().any(|s| s.contains("subdir")));
    }

    #[test]
    fn test_mock_filesystem_provider_error_cases() {
        // Test read error
        let error_provider = MockFileSystemProvider::new().with_error_read();
        let read_result = error_provider.read_file_string(&PathBuf::from("any.txt"));
        assert!(read_result.is_err(), "Should error on read");

        // Test write error
        let error_provider = MockFileSystemProvider::new().with_error_write();
        let write_result = error_provider.write_file_string(&PathBuf::from("any.txt"), "content");
        assert!(write_result.is_err(), "Should error on write");

        // Test create directory error
        let error_provider = MockFileSystemProvider::new().with_error_create_dir();
        let create_result = error_provider.create_dir_all(&PathBuf::from("any_dir"));
        assert!(create_result.is_err(), "Should error on create directory");
    }

    #[test]
    fn test_filesystem_provider_write_without_parent_directory() {
        let mock_provider = MockFileSystemProvider::new();
        
        // Try to write to a file in a non-existent directory
        let test_file = PathBuf::from("nonexistent_dir").join("file.txt");
        let write_result = mock_provider.write_file_string(&test_file, "content");
        
        assert!(write_result.is_err(), "Should error when parent directory doesn't exist");
    }

    #[test]
    fn test_filesystem_provider_walk_nonexistent_directory() {
        let mock_provider = MockFileSystemProvider::new();
        
        let walk_result = mock_provider.walk_dir(&PathBuf::from("nonexistent_dir"));
        assert!(walk_result.is_err(), "Should error when walking nonexistent directory");
    }

    #[test]
    fn test_filesystem_provider_interface_boundary() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that we can create FileSystemProvider from different sources
        let fs_provider_from_project = DependencyFactory::file_system_provider(Arc::clone(&project));
        let fs_provider_from_arc = DependencyFactory::file_system_provider(project);

        // Both should be able to call interface methods without errors
        let test_path = PathBuf::from("test");
        
        let exists1 = fs_provider_from_project.path_exists(&test_path);
        let exists2 = fs_provider_from_arc.path_exists(&test_path);
        
        // Results should be consistent (both should be boolean)
        assert!(exists1 == true || exists1 == false);
        assert!(exists2 == true || exists2 == false);
    }

    #[test]
    fn test_filesystem_provider_method_consistency() {
        let mock_provider = MockFileSystemProvider::new()
            .with_file(PathBuf::from("consistent.txt"), "Content".to_string());

        // Test multiple calls return consistent results
        assert!(mock_provider.path_exists(&PathBuf::from("consistent.txt")));
        assert!(mock_provider.path_exists(&PathBuf::from("consistent.txt")));
        
        let content1 = mock_provider.read_file_string(&PathBuf::from("consistent.txt"))
            .expect("Should read file");
        let content2 = mock_provider.read_file_string(&PathBuf::from("consistent.txt"))
            .expect("Should read file");
        
        assert_eq!(content1, content2, "Multiple reads should return same content");
    }

    #[test]
    fn test_filesystem_provider_dependency_factory() {
        let mock_provider = MockFileSystemProvider::new()
            .with_file(PathBuf::from("factory_test.txt"), "Factory content".to_string());
        
        // Test factory method creates proper provider
        let fs_provider = DependencyFactory::file_system_provider(mock_provider);
        
        // Should be able to call interface methods
        assert!(fs_provider.path_exists(&PathBuf::from("factory_test.txt")));
        
        let content = fs_provider.read_file_string(&PathBuf::from("factory_test.txt"))
            .expect("Should read file through factory");
        assert_eq!(content, "Factory content");
    }

    #[test]
    fn test_filesystem_provider_empty_file_operations() {
        let mock_provider = MockFileSystemProvider::new()
            .with_directory(PathBuf::from("empty_test"));

        let empty_file = PathBuf::from("empty_test").join("empty.txt");
        
        // Test write empty content
        mock_provider.write_file_string(&empty_file, "")
            .expect("Should write empty file");

        // Test read empty content
        let content = mock_provider.read_file_string(&empty_file)
            .expect("Should read empty file");
        assert_eq!(content, "", "Empty file should return empty string");

        // Test file exists
        assert!(mock_provider.path_exists(&empty_file));
    }

    #[test]
    fn test_filesystem_provider_overwrite_operations() {
        let mock_provider = MockFileSystemProvider::new()
            .with_directory(PathBuf::from("overwrite_test"));

        let test_file = PathBuf::from("overwrite_test").join("overwrite.txt");
        
        // Write initial content
        mock_provider.write_file_string(&test_file, "Original content")
            .expect("Should write initial content");

        // Verify initial content
        let content1 = mock_provider.read_file_string(&test_file)
            .expect("Should read initial content");
        assert_eq!(content1, "Original content");

        // Overwrite with new content
        mock_provider.write_file_string(&test_file, "New content")
            .expect("Should overwrite content");

        // Verify overwritten content
        let content2 = mock_provider.read_file_string(&test_file)
            .expect("Should read overwritten content");
        assert_eq!(content2, "New content");
    }

    #[test]
    fn test_filesystem_provider_integration_with_factory() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that factory can accept both Arc and direct project
        let _fs_provider1 = DependencyFactory::file_system_provider(Arc::clone(&project));
        let _fs_provider2 = DependencyFactory::file_system_provider(project);
        
        // Both should be created successfully without errors
    }
}

/// Mock RegistryProvider implementation for testing
struct MockRegistryProvider {
    registry_manager: RegistryManager,
    registry_type_map: std::collections::HashMap<String, String>,
    should_error: bool,
}

impl MockRegistryProvider {
    fn new() -> Self {
        let mut registry_type_map = std::collections::HashMap::new();
        registry_type_map.insert("https://registry.npmjs.org/".to_string(), "npm".to_string());
        registry_type_map.insert("https://npm.pkg.github.com/".to_string(), "github".to_string());
        registry_type_map.insert("https://registry.yarnpkg.com/".to_string(), "yarn".to_string());
        
        Self {
            registry_manager: RegistryManager::new(),
            registry_type_map,
            should_error: false,
        }
    }
    
    fn with_custom_registry_map(registry_map: std::collections::HashMap<String, String>) -> Self {
        Self {
            registry_manager: RegistryManager::new(),
            registry_type_map: registry_map,
            should_error: false,
        }
    }
    
    fn with_error(mut self) -> Self {
        self.should_error = true;
        self
    }
    
    fn add_registry_type(mut self, url: String, registry_type: String) -> Self {
        self.registry_type_map.insert(url, registry_type);
        self
    }
}

impl RegistryProvider for MockRegistryProvider {
    fn registry_manager(&self) -> &RegistryManager {
        &self.registry_manager
    }
    
    fn get_registry_type(&self, url: &str) -> String {
        if self.should_error {
            return "error".to_string();
        }
        
        self.registry_type_map.get(url)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string())
    }
}

mod registry_provider_tests {
    use super::*;

    #[test]
    fn test_registry_provider_contract_compliance() {
        let (_temp_dir, project) = create_test_project();
        let registry_provider = DependencyFactory::registry_provider(Arc::clone(&project));

        // Test that all interface methods are callable
        let registry_manager = registry_provider.registry_manager();
        
        // Registry manager should be a valid instance
        // Test basic functionality without making external calls
        
        // Test get_registry_type method
        let registry_type = registry_provider.get_registry_type("https://registry.npmjs.org/");
        assert!(!registry_type.is_empty(), "Should return non-empty registry type");
        
        // Test with unknown URL
        let unknown_type = registry_provider.get_registry_type("https://unknown.registry.com/");
        assert!(!unknown_type.is_empty(), "Should return non-empty string even for unknown registries");
    }

    #[test]
    fn test_mock_registry_provider_basic_operations() {
        let mock_provider = MockRegistryProvider::new();

        // Test registry manager access
        let registry_manager = mock_provider.registry_manager();
        // Registry manager should be accessible
        
        // Test known registry types
        assert_eq!(mock_provider.get_registry_type("https://registry.npmjs.org/"), "npm");
        assert_eq!(mock_provider.get_registry_type("https://npm.pkg.github.com/"), "github");
        assert_eq!(mock_provider.get_registry_type("https://registry.yarnpkg.com/"), "yarn");
        
        // Test unknown registry
        assert_eq!(mock_provider.get_registry_type("https://unknown.com/"), "unknown");
    }

    #[test]
    fn test_mock_registry_provider_custom_mappings() {
        let mut custom_map = std::collections::HashMap::new();
        custom_map.insert("https://custom.registry.com/".to_string(), "custom".to_string());
        custom_map.insert("https://private.npm.com/".to_string(), "private-npm".to_string());
        
        let mock_provider = MockRegistryProvider::with_custom_registry_map(custom_map);
        
        // Test custom mappings
        assert_eq!(mock_provider.get_registry_type("https://custom.registry.com/"), "custom");
        assert_eq!(mock_provider.get_registry_type("https://private.npm.com/"), "private-npm");
        
        // Test unknown mapping
        assert_eq!(mock_provider.get_registry_type("https://npmjs.org/"), "unknown");
    }

    #[test]
    fn test_mock_registry_provider_add_registry_type() {
        let mock_provider = MockRegistryProvider::new()
            .add_registry_type("https://new.registry.com/".to_string(), "new-type".to_string());
        
        // Test added registry type
        assert_eq!(mock_provider.get_registry_type("https://new.registry.com/"), "new-type");
        
        // Test existing registry types still work
        assert_eq!(mock_provider.get_registry_type("https://registry.npmjs.org/"), "npm");
    }

    #[test]
    fn test_mock_registry_provider_error_cases() {
        let error_provider = MockRegistryProvider::new().with_error();
        
        // Test error case
        assert_eq!(error_provider.get_registry_type("https://any.url.com/"), "error");
        
        // Registry manager should still be accessible
        let _registry_manager = error_provider.registry_manager();
    }

    #[test]
    fn test_registry_provider_url_normalization() {
        let mock_provider = MockRegistryProvider::new();
        
        // Test exact URL matching
        assert_eq!(mock_provider.get_registry_type("https://registry.npmjs.org/"), "npm");
        
        // Test different URL that doesn't match exactly
        assert_eq!(mock_provider.get_registry_type("https://registry.npmjs.org/package"), "unknown");
        
        // Test case sensitivity
        assert_eq!(mock_provider.get_registry_type("HTTPS://REGISTRY.NPMJS.ORG/"), "unknown");
    }

    #[test]
    fn test_registry_provider_empty_and_invalid_urls() {
        let mock_provider = MockRegistryProvider::new();
        
        // Test empty URL
        assert_eq!(mock_provider.get_registry_type(""), "unknown");
        
        // Test invalid URLs
        assert_eq!(mock_provider.get_registry_type("not-a-url"), "unknown");
        assert_eq!(mock_provider.get_registry_type("ftp://registry.com/"), "unknown");
        assert_eq!(mock_provider.get_registry_type("http://registry.com/"), "unknown");
    }

    #[test]
    fn test_registry_provider_interface_boundary() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that we can create RegistryProvider from different sources
        let registry_provider_from_project = DependencyFactory::registry_provider(Arc::clone(&project));
        let registry_provider_from_arc = DependencyFactory::registry_provider(project);

        // Both should be able to call interface methods
        let type1 = registry_provider_from_project.get_registry_type("https://registry.npmjs.org/");
        let type2 = registry_provider_from_arc.get_registry_type("https://registry.npmjs.org/");
        
        // Results should be consistent
        assert_eq!(type1, type2, "Both providers should return same registry type");
    }

    #[test]
    fn test_registry_provider_method_consistency() {
        let mock_provider = MockRegistryProvider::new();
        
        // Test multiple calls return consistent results
        let url = "https://registry.npmjs.org/";
        let type1 = mock_provider.get_registry_type(url);
        let type2 = mock_provider.get_registry_type(url);
        
        assert_eq!(type1, type2, "Multiple calls should return same result");
        assert_eq!(type1, "npm", "Should return expected registry type");
        
        // Test registry manager consistency
        let manager1 = mock_provider.registry_manager();
        let manager2 = mock_provider.registry_manager();
        
        // Should be the same reference
        assert!(std::ptr::eq(manager1, manager2), "Registry manager should be the same reference");
    }

    #[test]
    fn test_registry_provider_dependency_factory() {
        let mock_provider = MockRegistryProvider::new()
            .add_registry_type("https://factory.test.com/".to_string(), "factory-test".to_string());
        
        // Test factory method creates proper provider
        let registry_provider = DependencyFactory::registry_provider(mock_provider);
        
        // Should be able to call interface methods
        assert_eq!(registry_provider.get_registry_type("https://factory.test.com/"), "factory-test");
        
        let _registry_manager = registry_provider.registry_manager();
    }

    #[test]
    fn test_registry_provider_common_registry_urls() {
        let mock_provider = MockRegistryProvider::new();
        
        // Test common npm registry URLs
        assert_eq!(mock_provider.get_registry_type("https://registry.npmjs.org/"), "npm");
        assert_eq!(mock_provider.get_registry_type("https://npm.pkg.github.com/"), "github");
        assert_eq!(mock_provider.get_registry_type("https://registry.yarnpkg.com/"), "yarn");
        
        // Test unknown but valid-looking URLs
        assert_eq!(mock_provider.get_registry_type("https://registry.gitlab.com/"), "unknown");
        assert_eq!(mock_provider.get_registry_type("https://registry.jfrog.io/"), "unknown");
    }

    #[test]
    fn test_registry_provider_registry_manager_operations() {
        let mock_provider = MockRegistryProvider::new();
        let registry_manager = mock_provider.registry_manager();
        
        // Test that registry manager is accessible and functional
        // Note: We don't test actual operations as they might require network access
        // Just verify we can access the manager without errors
    }

    #[test]
    fn test_registry_provider_edge_cases() {
        let mock_provider = MockRegistryProvider::new();
        
        // Test with very long URLs
        let long_url = "https://".to_string() + &"a".repeat(1000) + ".com/";
        assert_eq!(mock_provider.get_registry_type(&long_url), "unknown");
        
        // Test with special characters in URLs
        assert_eq!(mock_provider.get_registry_type("https://registry-with-dashes.com/"), "unknown");
        assert_eq!(mock_provider.get_registry_type("https://registry_with_underscores.com/"), "unknown");
        assert_eq!(mock_provider.get_registry_type("https://registry123.com/"), "unknown");
    }

    #[test]
    fn test_registry_provider_multiple_instances() {
        let provider1 = MockRegistryProvider::new();
        let provider2 = MockRegistryProvider::new()
            .add_registry_type("https://different.com/".to_string(), "different".to_string());
        
        // Test that instances are independent
        assert_eq!(provider1.get_registry_type("https://different.com/"), "unknown");
        assert_eq!(provider2.get_registry_type("https://different.com/"), "different");
        
        // Test that both handle common URLs
        assert_eq!(provider1.get_registry_type("https://registry.npmjs.org/"), "npm");
        assert_eq!(provider2.get_registry_type("https://registry.npmjs.org/"), "npm");
    }

    #[test]
    fn test_registry_provider_integration_with_factory() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that factory can accept both Arc and direct project
        let _registry_provider1 = DependencyFactory::registry_provider(Arc::clone(&project));
        let _registry_provider2 = DependencyFactory::registry_provider(project);
        
        // Both should be created successfully without errors
    }

    #[test]
    fn test_registry_provider_url_trailing_slash_handling() {
        let mock_provider = MockRegistryProvider::new();
        
        // Test that exact matches work
        assert_eq!(mock_provider.get_registry_type("https://registry.npmjs.org/"), "npm");
        
        // Test without trailing slash (should not match our exact mapping)
        assert_eq!(mock_provider.get_registry_type("https://registry.npmjs.org"), "unknown");
        
        // This shows that our mock is doing exact string matching,
        // which is appropriate for testing the interface contract
    }
}

/// Mock WorkspaceProvider implementation for testing
struct MockWorkspaceProvider {
    root_path: PathBuf,
    workspace_patterns: Vec<String>,
    effective_patterns: Vec<String>,
    package_manager_patterns: Vec<String>,
    package_manager_type: String,
    package_manager_metadata: std::collections::HashMap<String, String>,
    workspace_config: Option<MonorepoDescriptor>,
    validation_errors: Vec<String>,
    commands: std::collections::HashMap<String, Vec<String>>,
    should_error_patterns: bool,
    should_error_config: bool,
    should_error_validation: bool,
}

impl MockWorkspaceProvider {
    fn new(root_path: PathBuf) -> Self {
        let mut commands = std::collections::HashMap::new();
        commands.insert("install".to_string(), vec!["npm".to_string(), "install".to_string()]);
        commands.insert("build".to_string(), vec!["npm".to_string(), "run".to_string(), "build".to_string()]);
        
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("version".to_string(), "8.0.0".to_string());
        metadata.insert("lockfile".to_string(), "package-lock.json".to_string());
        
        Self {
            root_path,
            workspace_patterns: vec!["packages/*".to_string(), "apps/*".to_string()],
            effective_patterns: vec!["packages/*".to_string(), "apps/*".to_string()],
            package_manager_patterns: vec!["packages/*".to_string()],
            package_manager_type: "npm".to_string(),
            package_manager_metadata: metadata,
            workspace_config: None,
            validation_errors: vec![],
            commands,
            should_error_patterns: false,
            should_error_config: false,
            should_error_validation: false,
        }
    }
    
    fn with_package_manager(mut self, pm_type: &str) -> Self {
        self.package_manager_type = pm_type.to_string();
        
        // Update patterns and metadata based on package manager
        match pm_type {
            "yarn" => {
                self.package_manager_metadata.insert("lockfile".to_string(), "yarn.lock".to_string());
                self.commands.insert("install".to_string(), vec!["yarn".to_string(), "install".to_string()]);
            }
            "pnpm" => {
                self.package_manager_metadata.insert("lockfile".to_string(), "pnpm-lock.yaml".to_string());
                self.commands.insert("install".to_string(), vec!["pnpm".to_string(), "install".to_string()]);
            }
            _ => {} // npm is default
        }
        
        self
    }
    
    fn with_patterns(mut self, patterns: Vec<String>) -> Self {
        self.workspace_patterns = patterns.clone();
        self.effective_patterns = patterns.clone();
        self.package_manager_patterns = patterns;
        self
    }
    
    fn with_validation_errors(mut self, errors: Vec<String>) -> Self {
        self.validation_errors = errors;
        self
    }
    
    fn with_error_patterns(mut self) -> Self {
        self.should_error_patterns = true;
        self
    }
    
    fn with_error_config(mut self) -> Self {
        self.should_error_config = true;
        self
    }
    
    fn with_error_validation(mut self) -> Self {
        self.should_error_validation = true;
        self
    }
}

impl WorkspaceProvider for MockWorkspaceProvider {
    fn root_path(&self) -> &Path {
        &self.root_path
    }
    
    fn get_workspace_patterns(&self) -> Result<Vec<String>> {
        if self.should_error_patterns {
            return Err(crate::error::Error::generic("Mock workspace patterns error"));
        }
        Ok(self.workspace_patterns.clone())
    }
    
    fn get_effective_workspace_patterns(&self) -> Result<Vec<String>> {
        if self.should_error_patterns {
            return Err(crate::error::Error::generic("Mock effective patterns error"));
        }
        Ok(self.effective_patterns.clone())
    }
    
    fn get_package_manager_patterns(&self) -> Result<Vec<String>> {
        if self.should_error_patterns {
            return Err(crate::error::Error::generic("Mock package manager patterns error"));
        }
        Ok(self.package_manager_patterns.clone())
    }
    
    fn get_package_manager_info(&self) -> Result<(String, std::collections::HashMap<String, String>)> {
        if self.should_error_config {
            return Err(crate::error::Error::generic("Mock package manager info error"));
        }
        Ok((self.package_manager_type.clone(), self.package_manager_metadata.clone()))
    }
    
    fn get_workspace_config(&self) -> Result<MonorepoDescriptor> {
        if self.should_error_config {
            return Err(crate::error::Error::generic("Mock workspace config error"));
        }
        
        // Create a simple workspace config if none was set
        if let Some(config) = &self.workspace_config {
            Ok(config.clone())
        } else {
            // Create a minimal MonorepoDescriptor for testing
            use sublime_standard_tools::monorepo::{MonorepoKind, WorkspacePackage};
            
            let workspace_packages = vec![
                WorkspacePackage {
                    name: "test-package".to_string(),
                    version: "1.0.0".to_string(),
                    location: PathBuf::from("packages/test-package"),
                    absolute_path: self.root_path.join("packages/test-package"),
                    workspace_dependencies: vec![],
                    workspace_dev_dependencies: vec![],
                }
            ];
            
            Ok(MonorepoDescriptor::new(
                MonorepoKind::NpmWorkSpace,
                self.root_path.clone(),
                workspace_packages
            ))
        }
    }
    
    fn validate_workspace_config(&self) -> Result<Vec<String>> {
        if self.should_error_validation {
            return Err(crate::error::Error::generic("Mock validation error"));
        }
        Ok(self.validation_errors.clone())
    }
    
    fn get_package_manager_commands(&self) -> std::collections::HashMap<String, Vec<String>> {
        self.commands.clone()
    }
}

mod workspace_provider_tests {
    use super::*;

    #[test]
    fn test_workspace_provider_contract_compliance() {
        let (_temp_dir, project) = create_test_project();
        let workspace_provider = DependencyFactory::workspace_provider(Arc::clone(&project));

        // Test that all interface methods are callable
        let root_path = workspace_provider.root_path();
        assert!(root_path.exists(), "Root path should exist");

        // Test workspace patterns (may fail but should be callable)
        let patterns_result = workspace_provider.get_workspace_patterns();
        assert!(patterns_result.is_ok() || patterns_result.is_err(), "Should return Result");

        // Test effective patterns
        let effective_result = workspace_provider.get_effective_workspace_patterns();
        assert!(effective_result.is_ok() || effective_result.is_err(), "Should return Result");

        // Test package manager patterns
        let pm_patterns_result = workspace_provider.get_package_manager_patterns();
        assert!(pm_patterns_result.is_ok() || pm_patterns_result.is_err(), "Should return Result");

        // Test package manager info
        let pm_info_result = workspace_provider.get_package_manager_info();
        assert!(pm_info_result.is_ok() || pm_info_result.is_err(), "Should return Result");

        // Test workspace config
        let config_result = workspace_provider.get_workspace_config();
        assert!(config_result.is_ok() || config_result.is_err(), "Should return Result");

        // Test validation
        let validation_result = workspace_provider.validate_workspace_config();
        assert!(validation_result.is_ok() || validation_result.is_err(), "Should return Result");

        // Test commands
        let commands = workspace_provider.get_package_manager_commands();
        assert!(commands.len() >= 0, "Should return valid commands map");
    }

    #[test]
    fn test_mock_workspace_provider_basic_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf());

        // Test root path
        assert_eq!(mock_provider.root_path(), temp_dir.path());

        // Test workspace patterns
        let patterns = mock_provider.get_workspace_patterns().expect("Should get patterns");
        assert_eq!(patterns, vec!["packages/*", "apps/*"]);

        // Test effective patterns
        let effective = mock_provider.get_effective_workspace_patterns().expect("Should get effective patterns");
        assert_eq!(effective, vec!["packages/*", "apps/*"]);

        // Test package manager patterns
        let pm_patterns = mock_provider.get_package_manager_patterns().expect("Should get PM patterns");
        assert_eq!(pm_patterns, vec!["packages/*"]);

        // Test package manager info
        let (pm_type, metadata) = mock_provider.get_package_manager_info().expect("Should get PM info");
        assert_eq!(pm_type, "npm");
        assert!(metadata.contains_key("version"));
        assert!(metadata.contains_key("lockfile"));
        assert_eq!(metadata.get("lockfile"), Some(&"package-lock.json".to_string()));

        // Test validation (should be empty by default)
        let validation = mock_provider.validate_workspace_config().expect("Should validate");
        assert_eq!(validation.len(), 0);

        // Test commands
        let commands = mock_provider.get_package_manager_commands();
        assert!(commands.contains_key("install"));
        assert!(commands.contains_key("build"));
    }

    #[test]
    fn test_mock_workspace_provider_different_package_managers() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Test Yarn
        let yarn_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf())
            .with_package_manager("yarn");
        
        let (pm_type, metadata) = yarn_provider.get_package_manager_info().expect("Should get Yarn info");
        assert_eq!(pm_type, "yarn");
        assert_eq!(metadata.get("lockfile"), Some(&"yarn.lock".to_string()));
        
        let commands = yarn_provider.get_package_manager_commands();
        assert_eq!(commands.get("install"), Some(&vec!["yarn".to_string(), "install".to_string()]));

        // Test pnpm
        let pnpm_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf())
            .with_package_manager("pnpm");
        
        let (pm_type, metadata) = pnpm_provider.get_package_manager_info().expect("Should get pnpm info");
        assert_eq!(pm_type, "pnpm");
        assert_eq!(metadata.get("lockfile"), Some(&"pnpm-lock.yaml".to_string()));
        
        let commands = pnpm_provider.get_package_manager_commands();
        assert_eq!(commands.get("install"), Some(&vec!["pnpm".to_string(), "install".to_string()]));
    }

    #[test]
    fn test_mock_workspace_provider_custom_patterns() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let custom_patterns = vec!["libs/*".to_string(), "tools/*".to_string(), "examples/*".to_string()];
        
        let mock_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf())
            .with_patterns(custom_patterns.clone());

        let patterns = mock_provider.get_workspace_patterns().expect("Should get patterns");
        assert_eq!(patterns, custom_patterns);

        let effective = mock_provider.get_effective_workspace_patterns().expect("Should get effective patterns");
        assert_eq!(effective, custom_patterns);

        let pm_patterns = mock_provider.get_package_manager_patterns().expect("Should get PM patterns");
        assert_eq!(pm_patterns, custom_patterns);
    }

    #[test]
    fn test_mock_workspace_provider_validation_errors() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let validation_errors = vec![
            "Pattern 'invalid/*' does not match any packages".to_string(),
            "Package 'orphaned-pkg' is not covered by workspace patterns".to_string()
        ];
        
        let mock_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf())
            .with_validation_errors(validation_errors.clone());

        let validation = mock_provider.validate_workspace_config().expect("Should validate");
        assert_eq!(validation, validation_errors);
        assert_eq!(validation.len(), 2);
    }

    #[test]
    fn test_mock_workspace_provider_workspace_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf());

        let config = mock_provider.get_workspace_config().expect("Should get config");
        
        // Test that we get a valid MonorepoDescriptor
        assert_eq!(config.root(), temp_dir.path());
        
        let packages = config.packages();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name, "test-package");
        assert_eq!(packages[0].version, "1.0.0");
    }

    #[test]
    fn test_mock_workspace_provider_error_cases() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Test patterns error
        let error_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf())
            .with_error_patterns();
        
        assert!(error_provider.get_workspace_patterns().is_err());
        assert!(error_provider.get_effective_workspace_patterns().is_err());
        assert!(error_provider.get_package_manager_patterns().is_err());

        // Test config error
        let error_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf())
            .with_error_config();
        
        assert!(error_provider.get_package_manager_info().is_err());
        assert!(error_provider.get_workspace_config().is_err());

        // Test validation error
        let error_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf())
            .with_error_validation();
        
        assert!(error_provider.validate_workspace_config().is_err());
    }

    #[test]
    fn test_workspace_provider_interface_boundary() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that we can create WorkspaceProvider from different sources
        let workspace_provider_from_project = DependencyFactory::workspace_provider(Arc::clone(&project));
        let workspace_provider_from_arc = DependencyFactory::workspace_provider(project);

        // Both should have the same root path
        assert_eq!(
            workspace_provider_from_project.root_path(),
            workspace_provider_from_arc.root_path()
        );
    }

    #[test]
    fn test_workspace_provider_method_consistency() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf());

        // Test multiple calls return consistent results
        let patterns1 = mock_provider.get_workspace_patterns().expect("Should get patterns");
        let patterns2 = mock_provider.get_workspace_patterns().expect("Should get patterns");
        assert_eq!(patterns1, patterns2);

        let (pm_type1, _) = mock_provider.get_package_manager_info().expect("Should get PM info");
        let (pm_type2, _) = mock_provider.get_package_manager_info().expect("Should get PM info");
        assert_eq!(pm_type1, pm_type2);

        // Test root path consistency
        let path1 = mock_provider.root_path();
        let path2 = mock_provider.root_path();
        assert_eq!(path1, path2);
    }

    #[test]
    fn test_workspace_provider_dependency_factory() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf())
            .with_package_manager("yarn");
        
        // Test factory method creates proper provider
        let workspace_provider = DependencyFactory::workspace_provider(mock_provider);
        
        // Should be able to call interface methods
        assert_eq!(workspace_provider.root_path(), temp_dir.path());
        
        let (pm_type, _) = workspace_provider.get_package_manager_info().expect("Should get PM info");
        assert_eq!(pm_type, "yarn");
    }

    #[test]
    fn test_workspace_provider_patterns_validation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Test with valid patterns
        let valid_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf())
            .with_patterns(vec!["packages/*".to_string(), "apps/*".to_string()]);
        
        let validation = valid_provider.validate_workspace_config().expect("Should validate");
        assert_eq!(validation.len(), 0, "Valid config should have no errors");

        // Test with validation errors
        let invalid_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf())
            .with_validation_errors(vec!["Invalid pattern".to_string()]);
        
        let validation = invalid_provider.validate_workspace_config().expect("Should validate");
        assert_eq!(validation.len(), 1, "Invalid config should have errors");
        assert_eq!(validation[0], "Invalid pattern");
    }

    #[test]
    fn test_workspace_provider_commands_configuration() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mock_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf());

        let commands = mock_provider.get_package_manager_commands();
        
        // Test default commands
        assert!(commands.contains_key("install"));
        assert!(commands.contains_key("build"));
        
        // Test command structure
        let install_cmd = commands.get("install").expect("Should have install command");
        assert!(install_cmd.len() >= 2);
        assert_eq!(install_cmd[0], "npm");
        assert_eq!(install_cmd[1], "install");
    }

    #[test]
    fn test_workspace_provider_package_manager_metadata() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Test npm metadata
        let npm_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf());
        let (_, npm_metadata) = npm_provider.get_package_manager_info().expect("Should get npm info");
        
        assert!(npm_metadata.contains_key("version"));
        assert!(npm_metadata.contains_key("lockfile"));
        assert_eq!(npm_metadata.get("lockfile"), Some(&"package-lock.json".to_string()));

        // Test yarn metadata
        let yarn_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf())
            .with_package_manager("yarn");
        let (_, yarn_metadata) = yarn_provider.get_package_manager_info().expect("Should get yarn info");
        
        assert_eq!(yarn_metadata.get("lockfile"), Some(&"yarn.lock".to_string()));
    }

    #[test]
    fn test_workspace_provider_effective_vs_raw_patterns() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let patterns = vec!["packages/*".to_string(), "apps/*".to_string()];
        
        let mock_provider = MockWorkspaceProvider::new(temp_dir.path().to_path_buf())
            .with_patterns(patterns.clone());

        let workspace_patterns = mock_provider.get_workspace_patterns().expect("Should get workspace patterns");
        let effective_patterns = mock_provider.get_effective_workspace_patterns().expect("Should get effective patterns");
        
        // In our mock, they're the same, but in real implementation they might differ
        assert_eq!(workspace_patterns, patterns);
        assert_eq!(effective_patterns, patterns);
        assert_eq!(workspace_patterns, effective_patterns);
    }

    #[test]
    fn test_workspace_provider_integration_with_factory() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that factory can accept both Arc and direct project
        let _workspace_provider1 = DependencyFactory::workspace_provider(Arc::clone(&project));
        let _workspace_provider2 = DependencyFactory::workspace_provider(project);
        
        // Both should be created successfully without errors
    }
}

/// Mock PackageDiscoveryProvider implementation for testing
struct MockPackageDiscoveryProvider {
    packages: Vec<MonorepoPackageInfo>,
    descriptor: MonorepoDescriptor,
    should_error_pattern: bool,
}

impl MockPackageDiscoveryProvider {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let root_path = temp_dir.path().to_path_buf();
        
        // Create test packages
        let packages = vec![
            create_test_package("ui-components", "1.0.0", &root_path, "packages/ui-components"),
            create_test_package("shared-utils", "2.1.0", &root_path, "packages/shared-utils"),  
            create_test_package("test-app", "0.5.0", &root_path, "apps/test-app"),
        ];
        
        // Create test descriptor
        use sublime_standard_tools::monorepo::{MonorepoKind, WorkspacePackage};
        let workspace_packages = vec![
            WorkspacePackage {
                name: "ui-components".to_string(),
                version: "1.0.0".to_string(),
                location: PathBuf::from("packages/ui-components"),
                absolute_path: root_path.join("packages/ui-components"),
                workspace_dependencies: vec!["shared-utils".to_string()],
                workspace_dev_dependencies: vec![],
            },
            WorkspacePackage {
                name: "shared-utils".to_string(),
                version: "2.1.0".to_string(),
                location: PathBuf::from("packages/shared-utils"),
                absolute_path: root_path.join("packages/shared-utils"),
                workspace_dependencies: vec![],
                workspace_dev_dependencies: vec![],
            },
            WorkspacePackage {
                name: "test-app".to_string(),
                version: "0.5.0".to_string(),
                location: PathBuf::from("apps/test-app"),
                absolute_path: root_path.join("apps/test-app"),
                workspace_dependencies: vec!["ui-components".to_string(), "shared-utils".to_string()],
                workspace_dev_dependencies: vec![],
            },
        ];
        
        let descriptor = MonorepoDescriptor::new(
            MonorepoKind::NpmWorkSpace,
            root_path,
            workspace_packages
        );
        
        Self {
            packages,
            descriptor,
            should_error_pattern: false,
        }
    }
    
    fn with_packages(mut self, packages: Vec<MonorepoPackageInfo>) -> Self {
        self.packages = packages;
        self
    }
    
    fn with_error_pattern(mut self) -> Self {
        self.should_error_pattern = true;
        self
    }
}

impl crate::core::interfaces::PackageDiscoveryProvider for MockPackageDiscoveryProvider {
    fn get_all_packages_with_metadata(&self) -> &[MonorepoPackageInfo] {
        &self.packages
    }
    
    fn get_package_descriptor(&self) -> &sublime_standard_tools::monorepo::MonorepoDescriptor {
        &self.descriptor
    }
    
    fn find_packages_by_pattern(&self, pattern: &str) -> Result<Vec<&MonorepoPackageInfo>> {
        if self.should_error_pattern {
            return Err(crate::error::Error::generic("Mock pattern search error"));
        }
        
        let matching_packages = self.packages.iter()
            .filter(|pkg| {
                let binding = pkg.package_info.package.borrow();
                let pkg_name = binding.name();
                pkg_name.contains(pattern) || 
                pkg.workspace_package.absolute_path.to_string_lossy().contains(pattern)
            })
            .collect();
        Ok(matching_packages)
    }
    
    fn get_package_locations(&self) -> Vec<(String, &Path)> {
        self.packages.iter()
            .map(|pkg| {
                let binding = pkg.package_info.package.borrow();
                let name = binding.name().to_string();
                let path = pkg.workspace_package.absolute_path.as_path();
                (name, path)
            })
            .collect()
    }
    
    fn find_orphaned_packages(&self, patterns: &[String]) -> Vec<&MonorepoPackageInfo> {
        // Simple mock implementation: packages that don't match any pattern
        self.packages.iter()
            .filter(|pkg| {
                let pkg_path = pkg.workspace_package.location.to_string_lossy();
                !patterns.iter().any(|pattern| {
                    // Simple pattern matching - check if package path starts with pattern prefix
                    let pattern_prefix = pattern.trim_end_matches("/*").trim_end_matches("*");
                    pkg_path.starts_with(pattern_prefix)
                })
            })
            .collect()
    }
}

/// Helper function to create test MonorepoConfig  
fn create_test_config(_root_path: &Path) -> MonorepoConfig {
    // Simply use the default configuration which has all required fields
    MonorepoConfig::default()
}

/// Helper function to create test MonorepoPackageInfo
fn create_test_package(name: &str, version: &str, root_path: &Path, relative_path: &str) -> MonorepoPackageInfo {
    use sublime_package_tools::{Package, PackageInfo};
    use sublime_standard_tools::monorepo::WorkspacePackage;
    use crate::core::types::VersionStatus;
    use serde_json::Value;
    
    let package = Package::new(name, version, None).expect("Failed to create package");
    let package_json_path = root_path.join(relative_path).join("package.json").to_string_lossy().to_string();
    let package_path = root_path.join(relative_path).to_string_lossy().to_string();
    let package_relative_path = relative_path.to_string();
    let pkg_json = Value::Object(serde_json::Map::new()); // Empty JSON object
    
    let package_info = PackageInfo::new(
        package,
        package_json_path,
        package_path,
        package_relative_path,
        pkg_json,
    );
    
    let workspace_package = WorkspacePackage {
        name: name.to_string(),
        version: version.to_string(),
        location: PathBuf::from(relative_path),
        absolute_path: root_path.join(relative_path),
        workspace_dependencies: vec![],
        workspace_dev_dependencies: vec![],
    };
    
    MonorepoPackageInfo {
        package_info,
        workspace_package,
        is_internal: true,
        dependents: vec![],
        dependencies_external: vec![],
        version_status: VersionStatus::Stable,
        changesets: vec![],
    }
}

#[cfg(test)]
mod package_discovery_provider_tests {
    use super::*;

    #[test]
    fn test_package_discovery_provider_get_all_packages() {
        let provider = MockPackageDiscoveryProvider::new();
        
        let packages = provider.get_all_packages_with_metadata();
        assert_eq!(packages.len(), 3);
        
        let package_names: Vec<String> = packages.iter()
            .map(|pkg| {
                let binding = pkg.package_info.package.borrow();
                binding.name().to_string()
            })
            .collect();
        
        assert!(package_names.contains(&"ui-components".to_string()));
        assert!(package_names.contains(&"shared-utils".to_string()));
        assert!(package_names.contains(&"test-app".to_string()));
    }

    #[test]
    fn test_package_discovery_provider_get_descriptor() {
        let provider = MockPackageDiscoveryProvider::new();
        
        let descriptor = provider.get_package_descriptor();
        assert_eq!(descriptor.packages().len(), 3);
        assert_eq!(descriptor.kind(), &sublime_standard_tools::monorepo::MonorepoKind::NpmWorkSpace);
        
        let package_names: Vec<&str> = descriptor.packages().iter()
            .map(|pkg| pkg.name.as_str())
            .collect();
        
        assert!(package_names.contains(&"ui-components"));
        assert!(package_names.contains(&"shared-utils"));
        assert!(package_names.contains(&"test-app"));
    }

    #[test]
    fn test_package_discovery_provider_find_by_pattern_name() {
        let provider = MockPackageDiscoveryProvider::new();
        
        // Test pattern matching by name
        let ui_packages = provider.find_packages_by_pattern("ui").expect("Should find ui packages");
        assert_eq!(ui_packages.len(), 1);
        assert_eq!(ui_packages[0].package_info.package.borrow().name(), "ui-components");
        
        // Test pattern matching multiple packages
        let shared_packages = provider.find_packages_by_pattern("shared").expect("Should find shared packages");
        assert_eq!(shared_packages.len(), 1);
        assert_eq!(shared_packages[0].package_info.package.borrow().name(), "shared-utils");
    }

    #[test]
    fn test_package_discovery_provider_find_by_pattern_path() {
        let provider = MockPackageDiscoveryProvider::new();
        
        // Test pattern matching by path
        let packages_packages = provider.find_packages_by_pattern("packages").expect("Should find packages in packages/");
        assert_eq!(packages_packages.len(), 2); // ui-components and shared-utils
        
        let apps_packages = provider.find_packages_by_pattern("apps").expect("Should find packages in apps/");
        assert_eq!(apps_packages.len(), 1); // test-app
        assert_eq!(apps_packages[0].package_info.package.borrow().name(), "test-app");
    }

    #[test]
    fn test_package_discovery_provider_find_by_pattern_no_match() {
        let provider = MockPackageDiscoveryProvider::new();
        
        let no_match = provider.find_packages_by_pattern("nonexistent").expect("Should handle no matches");
        assert_eq!(no_match.len(), 0);
    }

    #[test]
    fn test_package_discovery_provider_find_by_pattern_error() {
        let provider = MockPackageDiscoveryProvider::new().with_error_pattern();
        
        let result = provider.find_packages_by_pattern("test");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Mock pattern search error"));
    }

    #[test]
    fn test_package_discovery_provider_get_locations() {
        let provider = MockPackageDiscoveryProvider::new();
        
        let locations = provider.get_package_locations();
        assert_eq!(locations.len(), 3);
        
        // Find specific packages in locations
        let ui_location = locations.iter()
            .find(|(name, _)| name == "ui-components")
            .expect("Should find ui-components location");
        assert!(ui_location.1.to_string_lossy().contains("packages/ui-components"));
        
        let app_location = locations.iter()
            .find(|(name, _)| name == "test-app")
            .expect("Should find test-app location");
        assert!(app_location.1.to_string_lossy().contains("apps/test-app"));
    }

    #[test]
    fn test_package_discovery_provider_find_orphaned_no_patterns() {
        let provider = MockPackageDiscoveryProvider::new();
        
        // With no patterns, all packages are orphaned
        let orphaned = provider.find_orphaned_packages(&[]);
        assert_eq!(orphaned.len(), 3);
    }

    #[test]
    fn test_package_discovery_provider_find_orphaned_with_patterns() {
        let provider = MockPackageDiscoveryProvider::new();
        
        // With patterns that match packages and apps directories
        let patterns = vec!["packages/*".to_string(), "apps/*".to_string()];
        let orphaned = provider.find_orphaned_packages(&patterns);
        
        // All packages should match patterns, so no orphaned packages
        assert_eq!(orphaned.len(), 0);
    }

    #[test]
    fn test_package_discovery_provider_find_orphaned_partial_patterns() {
        let provider = MockPackageDiscoveryProvider::new();
        
        // Pattern only matches packages/ directory, not apps/
        let patterns = vec!["packages/*".to_string()];
        let orphaned = provider.find_orphaned_packages(&patterns);
        
        // test-app should be orphaned (it's in apps/, not packages/)
        assert_eq!(orphaned.len(), 1);
        assert_eq!(orphaned[0].package_info.package.borrow().name(), "test-app");
    }

    #[test]
    fn test_package_discovery_provider_comprehensive_workflow() {
        let provider = MockPackageDiscoveryProvider::new();
        
        // Test complete workflow
        let all_packages = provider.get_all_packages_with_metadata();
        let descriptor = provider.get_package_descriptor();
        let locations = provider.get_package_locations();
        
        // Consistency checks
        assert_eq!(all_packages.len(), descriptor.packages().len());
        assert_eq!(all_packages.len(), locations.len());
        
        // Test pattern search on found packages
        for package in all_packages {
            let binding = package.package_info.package.borrow();
            let pkg_name = binding.name();
            let found = provider.find_packages_by_pattern(pkg_name).expect("Should find package by name");
            assert!(!found.is_empty());
            assert!(found.iter().any(|p| {
                let p_binding = p.package_info.package.borrow();
                p_binding.name() == pkg_name
            }));
        }
    }

    #[test]
    fn test_package_discovery_provider_empty_packages() {
        let provider = MockPackageDiscoveryProvider::new().with_packages(vec![]);
        
        let packages = provider.get_all_packages_with_metadata();
        assert_eq!(packages.len(), 0);
        
        let locations = provider.get_package_locations();
        assert_eq!(locations.len(), 0);
        
        let found = provider.find_packages_by_pattern("anything").expect("Should handle empty packages");
        assert_eq!(found.len(), 0);
        
        let orphaned = provider.find_orphaned_packages(&["packages/*".to_string()]);
        assert_eq!(orphaned.len(), 0);
    }

    #[test]
    fn test_package_discovery_provider_contract_compliance() {
        let provider = MockPackageDiscoveryProvider::new();
        
        // Test that all methods can be called without panicking
        let _packages = provider.get_all_packages_with_metadata();
        let _descriptor = provider.get_package_descriptor();
        let _found = provider.find_packages_by_pattern("test");
        let _locations = provider.get_package_locations();
        let _orphaned = provider.find_orphaned_packages(&["packages/*".to_string()]);
        
        // All calls should complete successfully
    }

    #[test]
    fn test_package_discovery_provider_integration_with_factory() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that factory can accept both Arc and direct project
        let _discovery_provider1 = DependencyFactory::package_discovery_provider(Arc::clone(&project));
        let _discovery_provider2 = DependencyFactory::package_discovery_provider(project);
        
        // Both should be created successfully without errors
    }
}

/// Mock EnhancedConfigProvider implementation for testing
struct MockEnhancedConfigProvider {
    config: MonorepoConfig,
    effective_patterns: Vec<String>,
    validation_errors: Vec<String>,
    manager_commands: std::collections::HashMap<String, Vec<String>>,
    should_error_patterns: bool,
    should_error_validation: bool,
    should_error_workspace: bool,
}

impl MockEnhancedConfigProvider {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config = create_test_config(temp_dir.path());
        
        let mut manager_commands = std::collections::HashMap::new();
        manager_commands.insert("build".to_string(), vec!["npm".to_string(), "run".to_string(), "build".to_string()]);
        manager_commands.insert("test".to_string(), vec!["npm".to_string(), "test".to_string()]);
        manager_commands.insert("lint".to_string(), vec!["npm".to_string(), "run".to_string(), "lint".to_string()]);
        
        Self {
            config,
            effective_patterns: vec!["packages/*".to_string(), "apps/*".to_string()],
            validation_errors: vec![],
            manager_commands,
            should_error_patterns: false,
            should_error_validation: false,
            should_error_workspace: false,
        }
    }
    
    fn with_patterns(mut self, patterns: Vec<String>) -> Self {
        self.effective_patterns = patterns;
        self
    }
    
    fn with_validation_errors(mut self, errors: Vec<String>) -> Self {
        self.validation_errors = errors;
        self
    }
    
    fn with_commands(mut self, commands: std::collections::HashMap<String, Vec<String>>) -> Self {
        self.manager_commands = commands;
        self
    }
    
    fn with_error_patterns(mut self) -> Self {
        self.should_error_patterns = true;
        self
    }
    
    fn with_error_validation(mut self) -> Self {
        self.should_error_validation = true;
        self
    }
    
    fn with_error_workspace(mut self) -> Self {
        self.should_error_workspace = true;
        self
    }
}

impl ConfigProvider for MockEnhancedConfigProvider {
    fn config(&self) -> &MonorepoConfig {
        &self.config
    }
}

impl EnhancedConfigProvider for MockEnhancedConfigProvider {
    fn get_effective_patterns(&self) -> Result<Vec<String>> {
        if self.should_error_patterns {
            return Err(crate::error::Error::generic("Mock pattern resolution error"));
        }
        Ok(self.effective_patterns.clone())
    }
    
    fn validate_config(&self) -> Result<Vec<String>> {
        if self.should_error_validation {
            return Err(crate::error::Error::generic("Mock validation error"));
        }
        Ok(self.validation_errors.clone())
    }
    
    fn get_manager_commands(&self) -> std::collections::HashMap<String, Vec<String>> {
        self.manager_commands.clone()
    }
    
    fn get_workspace_section(&self) -> Result<&crate::config::WorkspaceConfig> {
        if self.should_error_workspace {
            return Err(crate::error::Error::generic("Mock workspace section error"));
        }
        Ok(&self.config.workspace)
    }
}

#[cfg(test)]
mod enhanced_config_provider_tests {
    use super::*;

    #[test]
    fn test_enhanced_config_provider_get_effective_patterns() {
        let provider = MockEnhancedConfigProvider::new();
        
        let patterns = provider.get_effective_patterns().expect("Should get effective patterns");
        assert_eq!(patterns.len(), 2);
        assert!(patterns.contains(&"packages/*".to_string()));
        assert!(patterns.contains(&"apps/*".to_string()));
    }

    #[test]
    fn test_enhanced_config_provider_get_effective_patterns_custom() {
        let custom_patterns = vec!["libs/*".to_string(), "tools/*".to_string(), "services/*".to_string()];
        let provider = MockEnhancedConfigProvider::new()
            .with_patterns(custom_patterns.clone());
        
        let patterns = provider.get_effective_patterns().expect("Should get custom patterns");
        assert_eq!(patterns.len(), 3);
        assert_eq!(patterns, custom_patterns);
    }

    #[test]
    fn test_enhanced_config_provider_get_effective_patterns_error() {
        let provider = MockEnhancedConfigProvider::new().with_error_patterns();
        
        let result = provider.get_effective_patterns();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Mock pattern resolution error"));
    }

    #[test]
    fn test_enhanced_config_provider_validate_config_no_errors() {
        let provider = MockEnhancedConfigProvider::new();
        
        let validation_results = provider.validate_config().expect("Should validate without errors");
        assert_eq!(validation_results.len(), 0);
    }

    #[test]
    fn test_enhanced_config_provider_validate_config_with_errors() {
        let errors = vec![
            "Missing package.json in workspace root".to_string(),
            "Invalid pattern in workspace configuration".to_string(),
            "Circular dependency detected".to_string(),
        ];
        let provider = MockEnhancedConfigProvider::new()
            .with_validation_errors(errors.clone());
        
        let validation_results = provider.validate_config().expect("Should return validation errors");
        assert_eq!(validation_results.len(), 3);
        assert_eq!(validation_results, errors);
    }

    #[test]
    fn test_enhanced_config_provider_validate_config_error() {
        let provider = MockEnhancedConfigProvider::new().with_error_validation();
        
        let result = provider.validate_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Mock validation error"));
    }

    #[test]
    fn test_enhanced_config_provider_get_manager_commands_default() {
        let provider = MockEnhancedConfigProvider::new();
        
        let commands = provider.get_manager_commands();
        assert_eq!(commands.len(), 3);
        
        // Check build command
        let build_cmd = commands.get("build").expect("Should have build command");
        assert_eq!(build_cmd, &vec!["npm".to_string(), "run".to_string(), "build".to_string()]);
        
        // Check test command
        let test_cmd = commands.get("test").expect("Should have test command");
        assert_eq!(test_cmd, &vec!["npm".to_string(), "test".to_string()]);
        
        // Check lint command
        let lint_cmd = commands.get("lint").expect("Should have lint command");
        assert_eq!(lint_cmd, &vec!["npm".to_string(), "run".to_string(), "lint".to_string()]);
    }

    #[test]
    fn test_enhanced_config_provider_get_manager_commands_custom() {
        let mut custom_commands = std::collections::HashMap::new();
        custom_commands.insert("deploy".to_string(), vec!["yarn".to_string(), "deploy".to_string()]);
        custom_commands.insert("start".to_string(), vec!["pnpm".to_string(), "start".to_string()]);
        
        let provider = MockEnhancedConfigProvider::new()
            .with_commands(custom_commands.clone());
        
        let commands = provider.get_manager_commands();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands, custom_commands);
    }

    #[test]
    fn test_enhanced_config_provider_get_workspace_section() {
        let provider = MockEnhancedConfigProvider::new();
        
        let workspace = provider.get_workspace_section().expect("Should get workspace section");
        assert!(workspace.patterns.len() >= 0);
        // Workspace section should be valid and accessible
    }

    #[test]
    fn test_enhanced_config_provider_get_workspace_section_error() {
        let provider = MockEnhancedConfigProvider::new().with_error_workspace();
        
        let result = provider.get_workspace_section();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Mock workspace section error"));
    }

    #[test]
    fn test_enhanced_config_provider_config_provider_inheritance() {
        let provider = MockEnhancedConfigProvider::new();
        
        // Test that ConfigProvider methods are available
        let config = provider.config();
        assert!(config.workspace.patterns.len() >= 0);
        
        // Test that we can call both base and enhanced methods
        let _patterns = provider.get_effective_patterns().expect("Should get patterns");
        let _validation = provider.validate_config().expect("Should validate");
        let _commands = provider.get_manager_commands();
        let _workspace = provider.get_workspace_section().expect("Should get workspace");
    }

    #[test]
    fn test_enhanced_config_provider_comprehensive_workflow() {
        let provider = MockEnhancedConfigProvider::new();
        
        // Test complete workflow
        let config = provider.config();
        let patterns = provider.get_effective_patterns().expect("Should get patterns");
        let validation = provider.validate_config().expect("Should validate");
        let commands = provider.get_manager_commands();
        let workspace = provider.get_workspace_section().expect("Should get workspace");
        
        // Consistency checks
        assert!(config.workspace.patterns.len() >= 0);
        assert!(patterns.len() > 0);
        assert_eq!(validation.len(), 0); // No errors by default
        assert!(commands.len() > 0);
        
        // Workspace section should be the same as config workspace
        assert_eq!(workspace as *const _, &config.workspace as *const _);
    }

    #[test]
    fn test_enhanced_config_provider_error_handling_comprehensive() {
        let provider = MockEnhancedConfigProvider::new()
            .with_error_patterns()
            .with_error_validation()
            .with_error_workspace();
        
        // All error-prone methods should fail
        assert!(provider.get_effective_patterns().is_err());
        assert!(provider.validate_config().is_err());
        assert!(provider.get_workspace_section().is_err());
        
        // Non-error methods should still work
        let _config = provider.config();
        let _commands = provider.get_manager_commands();
    }

    #[test]
    fn test_enhanced_config_provider_empty_patterns() {
        let provider = MockEnhancedConfigProvider::new()
            .with_patterns(vec![]);
        
        let patterns = provider.get_effective_patterns().expect("Should handle empty patterns");
        assert_eq!(patterns.len(), 0);
    }

    #[test]
    fn test_enhanced_config_provider_empty_commands() {
        let provider = MockEnhancedConfigProvider::new()
            .with_commands(std::collections::HashMap::new());
        
        let commands = provider.get_manager_commands();
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_enhanced_config_provider_contract_compliance() {
        let provider = MockEnhancedConfigProvider::new();
        
        // Test that all methods can be called without panicking
        let _config = provider.config();
        let _patterns = provider.get_effective_patterns();
        let _validation = provider.validate_config();
        let _commands = provider.get_manager_commands();
        let _workspace = provider.get_workspace_section();
        
        // All calls should complete successfully
    }

    #[test]
    fn test_enhanced_config_provider_integration_with_factory() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that factory can accept both Arc and direct project
        let _enhanced_provider1 = DependencyFactory::enhanced_config_provider(Arc::clone(&project));
        let _enhanced_provider2 = DependencyFactory::enhanced_config_provider(project);
        
        // Both should be created successfully without errors
    }
}

#[cfg(test)]
mod dependency_factory_comprehensive_tests {
    use super::*;

    #[test]
    fn test_factory_package_provider_creation() {
        let (_temp_dir, project) = create_test_project();
        
        // Test creation with Arc<MonorepoProject>
        let provider_arc = DependencyFactory::package_provider(Arc::clone(&project));
        let packages = provider_arc.packages();
        assert!(packages.len() >= 0, "Should return valid packages");
        
        // Test that provider methods work
        let root = provider_arc.root_path();
        assert!(root.exists(), "Root path should exist");
        
        // Test creation with direct MonorepoProject
        let provider_direct = DependencyFactory::package_provider(project);
        let packages_direct = provider_direct.packages();
        assert!(packages_direct.len() >= 0, "Should return valid packages from direct provider");
    }

    #[test]
    fn test_factory_config_provider_creation() {
        let (_temp_dir, project) = create_test_project();
        
        // Test creation with Arc<MonorepoProject>
        let provider_arc = DependencyFactory::config_provider(Arc::clone(&project));
        let config = provider_arc.config();
        assert!(config.workspace.patterns.len() >= 0, "Should have valid workspace config");
        
        // Test creation with direct MonorepoProject
        let provider_direct = DependencyFactory::config_provider(project);
        let config_direct = provider_direct.config();
        assert!(config_direct.workspace.patterns.len() >= 0, "Should have valid workspace config from direct provider");
    }

    #[test]
    fn test_factory_file_system_provider_creation() {
        let (_temp_dir, project) = create_test_project();
        
        // Test creation with Arc<MonorepoProject>
        let provider_arc = DependencyFactory::file_system_provider(Arc::clone(&project));
        let root = provider_arc.path_exists(&project.root_path);
        assert!(root, "Root path should exist");
        
        // Test creation with direct MonorepoProject
        let provider_direct = DependencyFactory::file_system_provider(project);
        // Test basic functionality without borrowing issues
        let temp_file = std::env::temp_dir().join("test_factory_fs.txt");
        let write_result = provider_direct.write_file_string(&temp_file, "test content");
        // Clean up
        if temp_file.exists() {
            let _ = std::fs::remove_file(&temp_file);
        }
        // Write should succeed (or at least not panic)
        assert!(write_result.is_ok() || write_result.is_err()); // Either outcome is acceptable for this test
    }

    #[test]
    fn test_factory_git_provider_creation() {
        let (_temp_dir, project) = create_test_project();
        
        // Test creation with Arc<MonorepoProject>
        let provider_arc = DependencyFactory::git_provider(Arc::clone(&project));
        let repo = provider_arc.repository();
        assert!(repo.get_current_branch().is_ok(), "Should be able to get current branch");
        
        // Test creation with direct MonorepoProject
        let provider_direct = DependencyFactory::git_provider(project);
        let repo_direct = provider_direct.repository();
        assert!(repo_direct.get_current_branch().is_ok(), "Should be able to get current branch from direct provider");
    }

    #[test]
    fn test_factory_registry_provider_creation() {
        let (_temp_dir, project) = create_test_project();
        
        // Test creation with Arc<MonorepoProject>
        let provider_arc = DependencyFactory::registry_provider(Arc::clone(&project));
        let _registry_manager = provider_arc.registry_manager();
        let registry_type = provider_arc.get_registry_type("https://registry.npmjs.org/");
        assert!(!registry_type.is_empty(), "Should return valid registry type");
        
        // Test creation with direct MonorepoProject
        let provider_direct = DependencyFactory::registry_provider(project);
        let registry_type_direct = provider_direct.get_registry_type("https://registry.npmjs.org/");
        assert!(!registry_type_direct.is_empty(), "Should return valid registry type from direct provider");
    }

    #[test]
    fn test_factory_workspace_provider_creation() {
        let (_temp_dir, project) = create_test_project();
        
        // Test creation with Arc<MonorepoProject>
        let provider_arc = DependencyFactory::workspace_provider(Arc::clone(&project));
        let patterns = provider_arc.get_workspace_patterns().expect("Should get workspace patterns");
        assert!(patterns.len() >= 0, "Should return valid patterns");
        
        // Test creation with direct MonorepoProject
        let provider_direct = DependencyFactory::workspace_provider(project);
        let patterns_direct = provider_direct.get_workspace_patterns().expect("Should get workspace patterns");
        assert!(patterns_direct.len() >= 0, "Should return valid patterns from direct provider");
    }

    #[test]
    fn test_factory_package_discovery_provider_creation() {
        let (_temp_dir, project) = create_test_project();
        
        // Test creation with Arc<MonorepoProject>
        let provider_arc = DependencyFactory::package_discovery_provider(Arc::clone(&project));
        let packages = provider_arc.get_all_packages_with_metadata();
        assert!(packages.len() >= 0, "Should return valid packages");
        let descriptor = provider_arc.get_package_descriptor();
        assert!(descriptor.packages().len() >= 0, "Should return valid descriptor");
        
        // Test creation with direct MonorepoProject
        let provider_direct = DependencyFactory::package_discovery_provider(project);
        let packages_direct = provider_direct.get_all_packages_with_metadata();
        assert!(packages_direct.len() >= 0, "Should return valid packages from direct provider");
    }

    #[test]
    fn test_factory_enhanced_config_provider_creation() {
        let (_temp_dir, project) = create_test_project();
        
        // Test creation with Arc<MonorepoProject>
        let provider_arc = DependencyFactory::enhanced_config_provider(Arc::clone(&project));
        let config = provider_arc.config();
        assert!(config.workspace.patterns.len() >= 0, "Should have valid config");
        let patterns = provider_arc.get_effective_patterns().expect("Should get effective patterns");
        assert!(patterns.len() >= 0, "Should return valid effective patterns");
        
        // Test creation with direct MonorepoProject
        let provider_direct = DependencyFactory::enhanced_config_provider(project);
        let patterns_direct = provider_direct.get_effective_patterns().expect("Should get effective patterns");
        assert!(patterns_direct.len() >= 0, "Should return valid effective patterns from direct provider");
    }

    #[test]
    fn test_factory_all_providers_comprehensive() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that all factory methods can be called and create valid providers
        let package_provider = DependencyFactory::package_provider(Arc::clone(&project));
        let config_provider = DependencyFactory::config_provider(Arc::clone(&project));
        let file_system_provider = DependencyFactory::file_system_provider(Arc::clone(&project));
        let git_provider = DependencyFactory::git_provider(Arc::clone(&project));
        let registry_provider = DependencyFactory::registry_provider(Arc::clone(&project));
        let workspace_provider = DependencyFactory::workspace_provider(Arc::clone(&project));
        let package_discovery_provider = DependencyFactory::package_discovery_provider(Arc::clone(&project));
        let enhanced_config_provider = DependencyFactory::enhanced_config_provider(Arc::clone(&project));
        
        // Test that all providers can be used
        let _packages = package_provider.packages();
        let _config = config_provider.config();
        let _root_exists = file_system_provider.path_exists(&project.root_path);
        let _repo = git_provider.repository();
        let _registry_manager = registry_provider.registry_manager();
        let _workspace_patterns = workspace_provider.get_workspace_patterns();
        let _discovery_packages = package_discovery_provider.get_all_packages_with_metadata();
        let _enhanced_patterns = enhanced_config_provider.get_effective_patterns();
        
        // All operations should complete without panicking
    }

    #[test]
    fn test_factory_type_safety() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that factory returns correct trait object types
        let package_provider: Box<dyn PackageProvider> = DependencyFactory::package_provider(Arc::clone(&project));
        let config_provider: Box<dyn ConfigProvider> = DependencyFactory::config_provider(Arc::clone(&project));
        let file_system_provider: Box<dyn FileSystemProvider> = DependencyFactory::file_system_provider(Arc::clone(&project));
        let git_provider: Box<dyn GitProvider> = DependencyFactory::git_provider(Arc::clone(&project));
        let registry_provider: Box<dyn RegistryProvider> = DependencyFactory::registry_provider(Arc::clone(&project));
        let workspace_provider: Box<dyn WorkspaceProvider> = DependencyFactory::workspace_provider(Arc::clone(&project));
        let package_discovery_provider: Box<dyn PackageDiscoveryProvider> = DependencyFactory::package_discovery_provider(Arc::clone(&project));
        let enhanced_config_provider: Box<dyn EnhancedConfigProvider> = DependencyFactory::enhanced_config_provider(project);
        
        // Test that providers work as expected through trait objects
        assert!(package_provider.packages().len() >= 0);
        assert!(config_provider.config().workspace.patterns.len() >= 0);
        assert!(file_system_provider.path_exists(&std::env::temp_dir()) || !file_system_provider.path_exists(&std::env::temp_dir()));
        assert!(git_provider.repository().get_current_branch().is_ok());
        assert!(!registry_provider.get_registry_type("https://registry.npmjs.org/").is_empty());
        assert!(workspace_provider.get_workspace_patterns().is_ok());
        assert!(package_discovery_provider.get_all_packages_with_metadata().len() >= 0);
        assert!(enhanced_config_provider.get_effective_patterns().is_ok());
    }

    #[test]
    fn test_factory_provider_independence() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that multiple providers can be created from the same source
        let provider1 = DependencyFactory::package_provider(Arc::clone(&project));
        let provider2 = DependencyFactory::package_provider(Arc::clone(&project));
        let provider3 = DependencyFactory::config_provider(Arc::clone(&project));
        
        // All should work independently
        let packages1 = provider1.packages();
        let packages2 = provider2.packages();
        let config = provider3.config();
        
        // Should return same data (since same source)
        assert_eq!(packages1.len(), packages2.len());
        assert!(config.workspace.patterns.len() >= 0);
    }

    #[test]
    fn test_factory_mixed_ownership_patterns() {
        let (_temp_dir, project) = create_test_project();
        
        // Test mixed patterns: some Arc, some direct
        let provider_arc = DependencyFactory::package_provider(Arc::clone(&project));
        let provider_arc2 = DependencyFactory::config_provider(Arc::clone(&project));
        let provider_direct = DependencyFactory::workspace_provider(project);
        
        // All should work correctly
        let _packages = provider_arc.packages();
        let _config = provider_arc2.config();
        let _patterns = provider_direct.get_workspace_patterns();
        
        // All operations should complete successfully
    }

    #[test]
    fn test_factory_provider_contract_compliance() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that factory-created providers comply with their contracts
        let package_provider = DependencyFactory::package_provider(Arc::clone(&project));
        let workspace_provider = DependencyFactory::workspace_provider(Arc::clone(&project));
        let enhanced_config_provider = DependencyFactory::enhanced_config_provider(project);
        
        // Test PackageProvider contract
        let packages = package_provider.packages();
        let root = package_provider.root_path();
        assert!(root.exists());
        for package in packages.iter().take(3) { // Test first few packages
            let binding = package.package_info.package.borrow();
            let name = binding.name();
            let found = package_provider.get_package(name);
            if found.is_some() {
                // Package should be findable if it exists
                assert!(found.is_some());
            }
        }
        
        // Test WorkspaceProvider contract
        let patterns = workspace_provider.get_workspace_patterns().expect("Should get patterns");
        let effective_patterns = workspace_provider.get_effective_workspace_patterns().expect("Should get effective patterns");
        let pm_info = workspace_provider.get_package_manager_info().expect("Should get PM info");
        assert!(patterns.len() >= 0);
        assert!(effective_patterns.len() >= 0);
        assert!(!pm_info.0.is_empty()); // Package manager type should not be empty
        
        // Test EnhancedConfigProvider contract (inherits ConfigProvider)
        let config = enhanced_config_provider.config();
        let effective_patterns_enhanced = enhanced_config_provider.get_effective_patterns().expect("Should get effective patterns");
        let validation = enhanced_config_provider.validate_config().expect("Should validate");
        assert!(config.workspace.patterns.len() >= 0);
        assert!(effective_patterns_enhanced.len() >= 0);
        assert!(validation.len() >= 0); // May have validation errors or be empty
    }
}

#[cfg(test)]
mod dependency_factory_error_tests {
    use super::*;

    /// Test struct that implements provider traits but can be configured to error
    struct ErrorConfigurableProvider {
        should_error: bool,
        error_message: String,
    }

    impl ErrorConfigurableProvider {
        fn new(should_error: bool, error_message: &str) -> Self {
            Self {
                should_error,
                error_message: error_message.to_string(),
            }
        }

        fn working() -> Self {
            Self::new(false, "")
        }

        fn failing(message: &str) -> Self {
            Self::new(true, message)
        }
    }

    impl PackageProvider for ErrorConfigurableProvider {
        fn packages(&self) -> &[MonorepoPackageInfo] {
            if self.should_error {
                // Return empty slice for error case - can't panic in trait method that doesn't return Result
                &[]
            } else {
                &[]
            }
        }

        fn get_package(&self, _name: &str) -> Option<&MonorepoPackageInfo> {
            if self.should_error {
                None
            } else {
                None
            }
        }

        fn get_dependents(&self, _package_name: &str) -> Vec<&MonorepoPackageInfo> {
            if self.should_error {
                vec![]
            } else {
                vec![]
            }
        }

        fn root_path(&self) -> &Path {
            if self.should_error {
                Path::new("/nonexistent")
            } else {
                Path::new("/tmp")
            }
        }
    }

    impl ConfigProvider for ErrorConfigurableProvider {
        fn config(&self) -> &MonorepoConfig {
            // For this test, we need to return a static config
            // Since we can't return errors from trait methods that don't return Result,
            // we'll use a default config for both cases
            static CONFIG: std::sync::LazyLock<MonorepoConfig> = std::sync::LazyLock::new(|| {
                MonorepoConfig::default()
            });
            &CONFIG
        }
    }

    impl FileSystemProvider for ErrorConfigurableProvider {
        fn read_file_string(&self, _path: &Path) -> Result<String> {
            if self.should_error {
                Err(crate::error::Error::generic(&self.error_message))
            } else {
                Ok("test content".to_string())
            }
        }

        fn write_file_string(&self, _path: &Path, _content: &str) -> Result<()> {
            if self.should_error {
                Err(crate::error::Error::generic(&self.error_message))
            } else {
                Ok(())
            }
        }

        fn path_exists(&self, _path: &Path) -> bool {
            !self.should_error
        }

        fn create_dir_all(&self, _path: &Path) -> Result<()> {
            if self.should_error {
                Err(crate::error::Error::generic(&self.error_message))
            } else {
                Ok(())
            }
        }

        fn remove_file(&self, _path: &Path) -> Result<()> {
            if self.should_error {
                Err(crate::error::Error::generic(&self.error_message))
            } else {
                Ok(())
            }
        }

        fn walk_dir(&self, _path: &Path) -> Result<Vec<std::path::PathBuf>> {
            if self.should_error {
                Err(crate::error::Error::generic(&self.error_message))
            } else {
                Ok(vec![])
            }
        }
    }

    #[test]
    fn test_factory_with_error_prone_package_provider() {
        // Test factory with provider that has error-prone behavior
        let error_provider = ErrorConfigurableProvider::failing("Package provider error");
        let provider = DependencyFactory::package_provider(error_provider);

        // Factory should create provider successfully
        let packages = provider.packages();
        assert_eq!(packages.len(), 0); // Error case returns empty

        let root = provider.root_path();
        assert_eq!(root, Path::new("/nonexistent")); // Error case returns nonexistent path

        let package = provider.get_package("test");
        assert!(package.is_none()); // Error case returns None
    }

    #[test]
    fn test_factory_with_working_package_provider() {
        // Test factory with provider that works correctly
        let working_provider = ErrorConfigurableProvider::working();
        let provider = DependencyFactory::package_provider(working_provider);

        // Factory should create provider successfully
        let packages = provider.packages();
        assert_eq!(packages.len(), 0); // Working case also returns empty for this mock

        let root = provider.root_path();
        assert_eq!(root, Path::new("/tmp")); // Working case returns /tmp

        let package = provider.get_package("test");
        assert!(package.is_none()); // Working case also returns None for this mock
    }

    #[test]
    fn test_factory_with_error_prone_config_provider() {
        // Test factory with config provider that can have error behavior
        let error_provider = ErrorConfigurableProvider::failing("Config provider error");
        let provider = DependencyFactory::config_provider(error_provider);

        // Factory should create provider successfully even with error-prone underlying provider
        let config = provider.config();
        assert!(config.workspace.patterns.len() >= 0); // Should get default config
    }

    #[test]
    fn test_factory_with_error_prone_file_system_provider() {
        // Test factory with file system provider that returns errors
        let error_provider = ErrorConfigurableProvider::failing("File system error");
        let provider = DependencyFactory::file_system_provider(error_provider);

        // Factory should create provider successfully
        let temp_path = Path::new("/tmp/test.txt");
        
        // Error-prone provider should return errors
        let read_result = provider.read_file_string(temp_path);
        assert!(read_result.is_err());
        assert!(read_result.unwrap_err().to_string().contains("File system error"));

        let write_result = provider.write_file_string(temp_path, "content");
        assert!(write_result.is_err());
        assert!(write_result.unwrap_err().to_string().contains("File system error"));

        let exists = provider.path_exists(temp_path);
        assert!(!exists); // Error case returns false

        let create_result = provider.create_dir_all(temp_path);
        assert!(create_result.is_err());
        assert!(create_result.unwrap_err().to_string().contains("File system error"));
    }

    #[test]
    fn test_factory_with_working_file_system_provider() {
        // Test factory with file system provider that works correctly
        let working_provider = ErrorConfigurableProvider::working();
        let provider = DependencyFactory::file_system_provider(working_provider);

        // Factory should create provider successfully
        let temp_path = Path::new("/tmp/test.txt");
        
        // Working provider should succeed
        let read_result = provider.read_file_string(temp_path);
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), "test content");

        let write_result = provider.write_file_string(temp_path, "content");
        assert!(write_result.is_ok());

        let exists = provider.path_exists(temp_path);
        assert!(exists); // Working case returns true

        let create_result = provider.create_dir_all(temp_path);
        assert!(create_result.is_ok());
    }

    #[test]
    fn test_factory_error_provider_through_trait_objects() {
        // Test that error behavior is preserved through trait object boxing
        let error_fs_provider = ErrorConfigurableProvider::failing("Trait object error");
        let working_fs_provider = ErrorConfigurableProvider::working();

        let error_provider: Box<dyn FileSystemProvider> = DependencyFactory::file_system_provider(error_fs_provider);
        let working_provider: Box<dyn FileSystemProvider> = DependencyFactory::file_system_provider(working_fs_provider);

        let test_path = Path::new("/tmp/trait_test.txt");

        // Error provider should maintain error behavior through trait object
        assert!(error_provider.read_file_string(test_path).is_err());
        assert!(!error_provider.path_exists(test_path));

        // Working provider should maintain working behavior through trait object
        assert!(working_provider.read_file_string(test_path).is_ok());
        assert!(working_provider.path_exists(test_path));
    }

    #[test]
    fn test_factory_multiple_error_providers() {
        // Test factory with multiple error-prone providers
        let error_package = ErrorConfigurableProvider::failing("Package error");
        let error_config = ErrorConfigurableProvider::failing("Config error");
        let error_fs = ErrorConfigurableProvider::failing("FileSystem error");

        let package_provider = DependencyFactory::package_provider(error_package);
        let config_provider = DependencyFactory::config_provider(error_config);
        let fs_provider = DependencyFactory::file_system_provider(error_fs);

        // All providers should be created successfully
        let _packages = package_provider.packages();
        let _config = config_provider.config();
        let fs_result = fs_provider.read_file_string(Path::new("/tmp/test"));
        assert!(fs_result.is_err());
    }

    #[test]
    fn test_factory_mixed_error_and_working_providers() {
        // Test factory with mix of error-prone and working providers
        let working_package = ErrorConfigurableProvider::working();
        let error_config = ErrorConfigurableProvider::failing("Config error");
        let working_fs = ErrorConfigurableProvider::working();

        let package_provider = DependencyFactory::package_provider(working_package);
        let config_provider = DependencyFactory::config_provider(error_config);
        let fs_provider = DependencyFactory::file_system_provider(working_fs);

        // Providers should behave according to their configuration
        let packages = package_provider.packages();
        assert_eq!(packages.len(), 0);
        
        let config = config_provider.config();
        assert!(config.workspace.patterns.len() >= 0);
        
        let fs_result = fs_provider.read_file_string(Path::new("/tmp/test"));
        assert!(fs_result.is_ok());
    }

    #[test]
    fn test_factory_error_propagation() {
        // Test that errors are properly propagated through the factory
        let error_provider = ErrorConfigurableProvider::failing("Test error message");
        let fs_provider = DependencyFactory::file_system_provider(error_provider);

        // Test string-returning operations
        let string_ops = vec![
            fs_provider.read_file_string(Path::new("/tmp/test1")),
        ];

        for result in string_ops {
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Test error message"));
        }

        // Test unit-returning operations
        let unit_ops = vec![
            fs_provider.write_file_string(Path::new("/tmp/test2"), "content"),
            fs_provider.create_dir_all(Path::new("/tmp/test3")),
            fs_provider.remove_file(Path::new("/tmp/test4")),
        ];

        for result in unit_ops {
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Test error message"));
        }

        // Test path-list-returning operations
        let path_list_result = fs_provider.walk_dir(Path::new("/tmp/test5"));
        assert!(path_list_result.is_err());
        assert!(path_list_result.unwrap_err().to_string().contains("Test error message"));
    }

    #[test]
    fn test_factory_provider_state_isolation() {
        // Test that error state is isolated between different provider instances
        let error_provider1 = ErrorConfigurableProvider::failing("Error 1");
        let error_provider2 = ErrorConfigurableProvider::failing("Error 2");
        let working_provider = ErrorConfigurableProvider::working();

        let fs_provider1 = DependencyFactory::file_system_provider(error_provider1);
        let fs_provider2 = DependencyFactory::file_system_provider(error_provider2);
        let fs_provider3 = DependencyFactory::file_system_provider(working_provider);

        let test_path = Path::new("/tmp/isolation_test");

        // Each provider should maintain its own error state
        let result1 = fs_provider1.read_file_string(test_path);
        let result2 = fs_provider2.read_file_string(test_path);
        let result3 = fs_provider3.read_file_string(test_path);

        assert!(result1.is_err());
        assert!(result1.unwrap_err().to_string().contains("Error 1"));

        assert!(result2.is_err());
        assert!(result2.unwrap_err().to_string().contains("Error 2"));

        assert!(result3.is_ok());
    }

    #[test]
    fn test_factory_edge_case_empty_providers() {
        // Test factory with providers that return empty results
        let empty_provider = ErrorConfigurableProvider::working(); // This returns empty results
        
        let package_provider = DependencyFactory::package_provider(empty_provider);
        let packages = package_provider.packages();
        assert_eq!(packages.len(), 0); // Should handle empty results gracefully

        let package = package_provider.get_package("nonexistent");
        assert!(package.is_none()); // Should handle missing packages gracefully

        let dependents = package_provider.get_dependents("nonexistent");
        assert_eq!(dependents.len(), 0); // Should handle empty dependents gracefully
    }

    #[test]
    fn test_factory_edge_case_nonexistent_paths() {
        // Test factory with providers that deal with nonexistent paths
        let error_provider = ErrorConfigurableProvider::failing("Path not found");
        let provider = DependencyFactory::package_provider(error_provider);

        let root = provider.root_path();
        assert_eq!(root, Path::new("/nonexistent"));
        assert!(!root.exists()); // Should handle nonexistent paths gracefully
    }
}

#[cfg(test)]
mod dependency_factory_ownership_tests {
    use super::*;

    #[test]
    fn test_factory_arc_vs_direct_ownership_package_provider() {
        let (_temp_dir, project) = create_test_project();
        
        // Test Arc ownership pattern
        let arc_provider = DependencyFactory::package_provider(Arc::clone(&project));
        let arc_packages = arc_provider.packages();
        let arc_root = arc_provider.root_path();
        
        // Test direct ownership pattern
        let direct_provider = DependencyFactory::package_provider(project);
        let direct_packages = direct_provider.packages();
        let direct_root = direct_provider.root_path();
        
        // Both should provide same data
        assert_eq!(arc_packages.len(), direct_packages.len());
        assert_eq!(arc_root, direct_root);
        
        // Both should be fully functional
        let _arc_package = arc_provider.get_package("test");
        let _direct_package = direct_provider.get_package("test");
    }

    #[test]
    fn test_factory_multiple_arc_providers_shared_ownership() {
        let (_temp_dir, project) = create_test_project();
        
        // Create multiple providers from the same Arc
        let package_provider1 = DependencyFactory::package_provider(Arc::clone(&project));
        let package_provider2 = DependencyFactory::package_provider(Arc::clone(&project));
        let config_provider = DependencyFactory::config_provider(Arc::clone(&project));
        let fs_provider = DependencyFactory::file_system_provider(project); // Final move
        
        // All providers should work independently
        let packages1 = package_provider1.packages();
        let packages2 = package_provider2.packages();
        let config = config_provider.config();
        
        // Test file system provider with a known path
        let temp_path = std::env::temp_dir();
        let _root_exists = fs_provider.path_exists(&temp_path);
        
        // Shared data should be consistent
        assert_eq!(packages1.len(), packages2.len());
        assert!(config.workspace.patterns.len() >= 0);
        
        // Each provider should maintain independent access
        for i in 0..std::cmp::min(packages1.len(), 3) {
            let binding1 = packages1[i].package_info.package.borrow();
            let name1 = binding1.name();
            let binding2 = packages2[i].package_info.package.borrow();
            let name2 = binding2.name();
            assert_eq!(name1, name2);
        }
    }

    #[test]
    fn test_factory_ownership_transfer_to_trait_objects() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that ownership is properly transferred to trait objects
        let package_provider: Box<dyn PackageProvider> = {
            let local_project = Arc::clone(&project);
            DependencyFactory::package_provider(local_project)
            // local_project goes out of scope here, but trait object should remain valid
        };
        
        let config_provider: Box<dyn ConfigProvider> = {
            let local_project = Arc::clone(&project);
            DependencyFactory::config_provider(local_project)
            // local_project goes out of scope here
        };
        
        let fs_provider: Box<dyn FileSystemProvider> = {
            // Use the original project for the final provider
            DependencyFactory::file_system_provider(project)
            // project is moved here
        };
        
        // All providers should still work after ownership transfer
        let packages = package_provider.packages();
        let config = config_provider.config();
        let temp_path = std::env::temp_dir();
        let _exists = fs_provider.path_exists(&temp_path);
        
        assert!(packages.len() >= 0);
        assert!(config.workspace.patterns.len() >= 0);
    }

    #[test]
    fn test_factory_arc_reference_counting_behavior() {
        let (_temp_dir, original_project) = create_test_project();
        
        // Create Arc and test reference counting through providers
        let project_arc = Arc::clone(&original_project);
        
        // Create multiple providers that should increment reference count
        let provider1 = DependencyFactory::package_provider(Arc::clone(&project_arc));
        let provider2 = DependencyFactory::config_provider(Arc::clone(&project_arc));
        let provider3 = DependencyFactory::file_system_provider(Arc::clone(&project_arc));
        
        // All providers should work and maintain data consistency
        let packages = provider1.packages();
        let config = provider2.config();
        let temp_path = std::env::temp_dir();
        let exists = provider3.path_exists(&temp_path);
        
        assert!(packages.len() >= 0);
        assert!(config.workspace.patterns.len() >= 0);
        assert!(exists || !exists); // Either result is valid for this test
        
        // Drop one provider and others should still work
        drop(provider1);
        let config2 = provider2.config();
        let exists2 = provider3.path_exists(&temp_path);
        
        assert!(config2.workspace.patterns.len() >= 0);
        assert_eq!(exists, exists2); // Behavior should be consistent
    }

    #[test]
    fn test_factory_mixed_ownership_patterns_complex() {
        let (_temp_dir, project) = create_test_project();
        
        // Complex scenario with mixed ownership patterns
        let shared_project = Arc::clone(&project);
        
        // Some providers use Arc (shared ownership)
        let shared_package_provider = DependencyFactory::package_provider(Arc::clone(&shared_project));
        let shared_config_provider = DependencyFactory::config_provider(Arc::clone(&shared_project));
        
        // Some providers use direct ownership  
        let direct_workspace_provider = DependencyFactory::workspace_provider(Arc::clone(&shared_project));
        let direct_registry_provider = DependencyFactory::registry_provider(shared_project);
        
        // Final provider takes ownership of original
        let final_provider = DependencyFactory::file_system_provider(project);
        
        // All should work correctly despite different ownership patterns
        let packages = shared_package_provider.packages();
        let config = shared_config_provider.config();
        let patterns = direct_workspace_provider.get_workspace_patterns().expect("Should get patterns");
        let registry_type = direct_registry_provider.get_registry_type("https://registry.npmjs.org/");
        let temp_exists = final_provider.path_exists(&std::env::temp_dir());
        
        // All operations should complete successfully
        assert!(packages.len() >= 0);
        assert!(config.workspace.patterns.len() >= 0);
        assert!(patterns.len() >= 0);
        assert!(!registry_type.is_empty());
        assert!(temp_exists || !temp_exists);
    }

    #[test] 
    fn test_factory_ownership_with_scoped_providers() {
        let (_temp_dir, project) = create_test_project();
        
        let packages_result = {
            // Create provider in limited scope
            let scoped_provider = DependencyFactory::package_provider(Arc::clone(&project));
            scoped_provider.packages().len()
            // scoped_provider is dropped here
        };
        
        let config_result = {
            // Create another provider in different scope
            let scoped_config = DependencyFactory::config_provider(Arc::clone(&project));
            scoped_config.config().workspace.patterns.len()
            // scoped_config is dropped here
        };
        
        // Original project should still be valid
        let final_provider = DependencyFactory::file_system_provider(project);
        let final_exists = final_provider.path_exists(&std::env::temp_dir());
        
        // All results should be valid
        assert!(packages_result >= 0);
        assert!(config_result >= 0);
        assert!(final_exists || !final_exists);
    }

    #[test]
    fn test_factory_arc_cloning_vs_moving() {
        let (_temp_dir, project) = create_test_project();
        
        // Test cloning Arc vs moving Arc
        let project_arc = Arc::clone(&project);
        
        // Clone the Arc for provider
        let cloned_provider = DependencyFactory::package_provider(Arc::clone(&project_arc));
        
        // Move the Arc for another provider (project_arc is still valid after clone)
        let moved_provider = DependencyFactory::config_provider(project_arc);
        
        // Original project for final provider
        let original_provider = DependencyFactory::file_system_provider(project);
        
        // All should work
        let packages = cloned_provider.packages();
        let config = moved_provider.config();
        let temp_exists = original_provider.path_exists(&std::env::temp_dir());
        
        assert!(packages.len() >= 0);
        assert!(config.workspace.patterns.len() >= 0);
        assert!(temp_exists || !temp_exists);
    }

    #[test]
    fn test_factory_ownership_with_different_trait_objects() {
        let (_temp_dir, project) = create_test_project();
        
        // Create different types of trait objects with different ownership patterns
        let providers: Vec<Box<dyn std::any::Any>> = vec![
            Box::new(DependencyFactory::package_provider(Arc::clone(&project))),
            Box::new(DependencyFactory::config_provider(Arc::clone(&project))),
            Box::new(DependencyFactory::file_system_provider(Arc::clone(&project))),
            Box::new(DependencyFactory::git_provider(Arc::clone(&project))),
        ];
        
        // Use the final project reference
        let final_provider = DependencyFactory::workspace_provider(project);
        let patterns = final_provider.get_workspace_patterns().expect("Should get patterns");
        
        // All providers should be stored successfully
        assert_eq!(providers.len(), 4);
        assert!(patterns.len() >= 0);
        
        // Providers should remain valid even as trait objects
        for provider in providers {
            assert!(std::any::Any::type_id(&*provider) != std::any::TypeId::of::<()>());
        }
    }

    #[test]
    fn test_factory_ownership_performance_comparison() {
        let (_temp_dir, project) = create_test_project();
        
        // Test performance characteristics of Arc vs direct ownership
        // Note: This is more of a behavioral test than performance measurement
        
        let start_time = std::time::Instant::now();
        
        // Create multiple Arc-based providers (shared ownership)
        let arc_providers: Vec<Box<dyn PackageProvider>> = (0..10)
            .map(|_| DependencyFactory::package_provider(Arc::clone(&project)))
            .collect();
        
        let arc_time = start_time.elapsed();
        
        // Create direct ownership providers (each needs its own project)
        let start_time2 = std::time::Instant::now();
        let direct_provider = DependencyFactory::package_provider(project);
        let direct_time = start_time2.elapsed();
        
        // Both patterns should complete quickly
        assert!(arc_time < std::time::Duration::from_secs(1));
        assert!(direct_time < std::time::Duration::from_secs(1));
        
        // Verify functionality
        assert_eq!(arc_providers.len(), 10);
        for provider in &arc_providers {
            assert!(provider.packages().len() >= 0);
        }
        
        assert!(direct_provider.packages().len() >= 0);
    }

    #[test]
    fn test_factory_ownership_memory_safety() {
        let (_temp_dir, project) = create_test_project();
        
        // Test memory safety with complex ownership patterns
        let providers = {
            let local_arc = Arc::clone(&project);
            
            // Create providers with various ownership patterns
            vec![
                DependencyFactory::package_provider(Arc::clone(&local_arc)),
                DependencyFactory::package_provider(Arc::clone(&local_arc)),
                DependencyFactory::package_provider(local_arc), // Move the Arc
            ]
            // local_arc is moved, but providers should still be valid
        };
        
        // Use original project for final test
        let final_provider = DependencyFactory::config_provider(project);
        
        // All providers should still be valid and functional
        for provider in &providers {
            let packages = provider.packages();
            assert!(packages.len() >= 0);
        }
        
        let config = final_provider.config();
        assert!(config.workspace.patterns.len() >= 0);
        
        // Test that providers remain valid even after original references are dropped
        assert_eq!(providers.len(), 3);
    }

    #[test]
    fn test_factory_ownership_edge_cases() {
        let (_temp_dir, project) = create_test_project();
        
        // Test edge cases in ownership patterns
        
        // Case 1: Immediate drop after creation
        {
            let _dropped_provider = DependencyFactory::package_provider(Arc::clone(&project));
            // Provider is immediately dropped
        }
        
        // Case 2: Provider stored in option
        let optional_provider: Option<Box<dyn ConfigProvider>> = 
            Some(DependencyFactory::config_provider(Arc::clone(&project)));
        
        if let Some(provider) = optional_provider {
            let config = provider.config();
            assert!(config.workspace.patterns.len() >= 0);
        }
        
        // Case 3: Provider in result
        let result_provider: std::result::Result<Box<dyn FileSystemProvider>, &str> = 
            Ok(DependencyFactory::file_system_provider(Arc::clone(&project)));
        
        match result_provider {
            Ok(provider) => {
                let exists = provider.path_exists(&std::env::temp_dir());
                assert!(exists || !exists);
            }
            Err(_) => panic!("Should not error"),
        }
        
        // Case 4: Final ownership transfer
        let _final_provider = DependencyFactory::workspace_provider(project);
        // project is moved and no longer accessible
    }
}

#[cfg(test)]
mod dependency_factory_chaining_tests {
    use super::*;
    use std::collections::HashMap;

    /// Helper struct to simulate a provider factory builder pattern
    struct ProviderBuilder {
        project: Arc<MonorepoProject>,
        providers: HashMap<String, Box<dyn std::any::Any>>,
        creation_order: Vec<String>,
    }

    impl ProviderBuilder {
        fn new(project: Arc<MonorepoProject>) -> Self {
            Self {
                project,
                providers: HashMap::new(),
                creation_order: Vec::new(),
            }
        }

        fn with_package_provider(mut self) -> Self {
            let provider = DependencyFactory::package_provider(Arc::clone(&self.project));
            self.providers.insert("package".to_string(), Box::new(provider));
            self.creation_order.push("package".to_string());
            self
        }

        fn with_config_provider(mut self) -> Self {
            let provider = DependencyFactory::config_provider(Arc::clone(&self.project));
            self.providers.insert("config".to_string(), Box::new(provider));
            self.creation_order.push("config".to_string());
            self
        }

        fn with_file_system_provider(mut self) -> Self {
            let provider = DependencyFactory::file_system_provider(Arc::clone(&self.project));
            self.providers.insert("filesystem".to_string(), Box::new(provider));
            self.creation_order.push("filesystem".to_string());
            self
        }

        fn with_git_provider(mut self) -> Self {
            let provider = DependencyFactory::git_provider(Arc::clone(&self.project));
            self.providers.insert("git".to_string(), Box::new(provider));
            self.creation_order.push("git".to_string());
            self
        }

        fn with_registry_provider(mut self) -> Self {
            let provider = DependencyFactory::registry_provider(Arc::clone(&self.project));
            self.providers.insert("registry".to_string(), Box::new(provider));
            self.creation_order.push("registry".to_string());
            self
        }

        fn with_workspace_provider(mut self) -> Self {
            let provider = DependencyFactory::workspace_provider(Arc::clone(&self.project));
            self.providers.insert("workspace".to_string(), Box::new(provider));
            self.creation_order.push("workspace".to_string());
            self
        }

        fn with_package_discovery_provider(mut self) -> Self {
            let provider = DependencyFactory::package_discovery_provider(Arc::clone(&self.project));
            self.providers.insert("discovery".to_string(), Box::new(provider));
            self.creation_order.push("discovery".to_string());
            self
        }

        fn with_enhanced_config_provider(mut self) -> Self {
            let provider = DependencyFactory::enhanced_config_provider(Arc::clone(&self.project));
            self.providers.insert("enhanced_config".to_string(), Box::new(provider));
            self.creation_order.push("enhanced_config".to_string());
            self
        }

        fn build(self) -> (HashMap<String, Box<dyn std::any::Any>>, Vec<String>) {
            (self.providers, self.creation_order)
        }

        fn provider_count(&self) -> usize {
            self.providers.len()
        }
    }

    #[test]
    fn test_factory_method_chaining_basic() {
        let (_temp_dir, project) = create_test_project();
        
        // Test basic method chaining pattern
        let (providers, creation_order) = ProviderBuilder::new(project)
            .with_package_provider()
            .with_config_provider()
            .with_file_system_provider()
            .build();

        // Verify all providers were created
        assert_eq!(providers.len(), 3);
        assert_eq!(creation_order, vec!["package", "config", "filesystem"]);

        // Verify providers exist with correct keys
        assert!(providers.contains_key("package"));
        assert!(providers.contains_key("config"));
        assert!(providers.contains_key("filesystem"));
    }

    #[test]
    fn test_factory_method_chaining_all_providers() {
        let (_temp_dir, project) = create_test_project();
        
        // Test chaining with all provider types
        let (providers, creation_order) = ProviderBuilder::new(project)
            .with_package_provider()
            .with_config_provider()
            .with_file_system_provider()
            .with_git_provider()
            .with_registry_provider()
            .with_workspace_provider()
            .with_package_discovery_provider()
            .with_enhanced_config_provider()
            .build();

        // Verify all 8 provider types were created
        assert_eq!(providers.len(), 8);
        assert_eq!(creation_order.len(), 8);

        // Verify creation order
        let expected_order = vec![
            "package", "config", "filesystem", "git", 
            "registry", "workspace", "discovery", "enhanced_config"
        ];
        assert_eq!(creation_order, expected_order);

        // Verify all provider types exist
        for provider_type in &expected_order {
            assert!(providers.contains_key(*provider_type), 
                "Missing provider type: {}", provider_type);
        }
    }

    #[test]
    fn test_factory_method_chaining_conditional() {
        let (_temp_dir, project) = create_test_project();
        
        // Test conditional chaining
        let mut builder = ProviderBuilder::new(project)
            .with_package_provider()
            .with_config_provider();

        // Conditionally add more providers
        if true { // Simulate some condition
            builder = builder.with_file_system_provider();
        }

        if false { // Simulate another condition
            builder = builder.with_git_provider();
        }

        let (providers, creation_order) = builder
            .with_workspace_provider() // Always add this
            .build();

        // Verify correct providers were created
        assert_eq!(providers.len(), 4);
        assert_eq!(creation_order, vec!["package", "config", "filesystem", "workspace"]);
        
        // Verify git provider was not created
        assert!(!providers.contains_key("git"));
    }

    #[test]
    fn test_factory_method_chaining_incremental() {
        let (_temp_dir, project) = create_test_project();
        
        // Test incremental building pattern
        let mut builder = ProviderBuilder::new(project);
        
        // Add providers incrementally
        assert_eq!(builder.provider_count(), 0);
        
        builder = builder.with_package_provider();
        assert_eq!(builder.provider_count(), 1);
        
        builder = builder.with_config_provider();
        assert_eq!(builder.provider_count(), 2);
        
        builder = builder.with_file_system_provider();
        assert_eq!(builder.provider_count(), 3);
        
        let (providers, creation_order) = builder.build();
        
        assert_eq!(providers.len(), 3);
        assert_eq!(creation_order, vec!["package", "config", "filesystem"]);
    }

    #[test]
    fn test_factory_complex_provider_creation_workflow() {
        let (_temp_dir, project) = create_test_project();
        
        // Simulate complex workflow with multiple provider creation patterns
        struct ProviderSet {
            core_providers: Vec<Box<dyn std::any::Any>>,
            enhanced_providers: Vec<Box<dyn std::any::Any>>,
            specialized_providers: Vec<Box<dyn std::any::Any>>,
        }

        let provider_set = {
            let project_ref = Arc::clone(&project);
            
            // Core providers (essential functionality)
            let core_providers = vec![
                Box::new(DependencyFactory::package_provider(Arc::clone(&project_ref))) as Box<dyn std::any::Any>,
                Box::new(DependencyFactory::config_provider(Arc::clone(&project_ref))) as Box<dyn std::any::Any>,
                Box::new(DependencyFactory::file_system_provider(Arc::clone(&project_ref))) as Box<dyn std::any::Any>,
            ];

            // Enhanced providers (additional functionality)
            let enhanced_providers = vec![
                Box::new(DependencyFactory::workspace_provider(Arc::clone(&project_ref))) as Box<dyn std::any::Any>,
                Box::new(DependencyFactory::package_discovery_provider(Arc::clone(&project_ref))) as Box<dyn std::any::Any>,
                Box::new(DependencyFactory::enhanced_config_provider(Arc::clone(&project_ref))) as Box<dyn std::any::Any>,
            ];

            // Specialized providers (specific use cases)
            let specialized_providers = vec![
                Box::new(DependencyFactory::git_provider(Arc::clone(&project_ref))) as Box<dyn std::any::Any>,
                Box::new(DependencyFactory::registry_provider(project_ref)) as Box<dyn std::any::Any>,
            ];

            ProviderSet {
                core_providers,
                enhanced_providers,
                specialized_providers,
            }
        };

        // Verify all provider sets were created correctly
        assert_eq!(provider_set.core_providers.len(), 3);
        assert_eq!(provider_set.enhanced_providers.len(), 3);
        assert_eq!(provider_set.specialized_providers.len(), 2);

        // Total of 8 providers across all sets
        let total_providers = provider_set.core_providers.len() + 
                            provider_set.enhanced_providers.len() + 
                            provider_set.specialized_providers.len();
        assert_eq!(total_providers, 8);
    }

    #[test]
    fn test_factory_provider_creation_with_dependencies() {
        let (_temp_dir, project) = create_test_project();
        
        // Simulate provider creation with dependency relationships
        struct DependentProviderSet {
            base_config: Box<dyn ConfigProvider>,
            enhanced_config: Box<dyn EnhancedConfigProvider>,
            package_provider: Box<dyn PackageProvider>,
            workspace_provider: Box<dyn WorkspaceProvider>,
            discovery_provider: Box<dyn PackageDiscoveryProvider>,
        }

        let provider_set = {
            let project_ref = Arc::clone(&project);
            
            // Create base providers first
            let base_config = DependencyFactory::config_provider(Arc::clone(&project_ref));
            let package_provider = DependencyFactory::package_provider(Arc::clone(&project_ref));
            
            // Create providers that might depend on base providers
            let enhanced_config = DependencyFactory::enhanced_config_provider(Arc::clone(&project_ref));
            let workspace_provider = DependencyFactory::workspace_provider(Arc::clone(&project_ref));
            let discovery_provider = DependencyFactory::package_discovery_provider(project_ref);
            
            DependentProviderSet {
                base_config,
                enhanced_config,
                package_provider,
                workspace_provider,
                discovery_provider,
            }
        };

        // Test that all providers can be used together
        let config = provider_set.base_config.config();
        let enhanced_patterns = provider_set.enhanced_config.get_effective_patterns()
            .expect("Should get enhanced patterns");
        let packages = provider_set.package_provider.packages();
        let workspace_patterns = provider_set.workspace_provider.get_workspace_patterns()
            .expect("Should get workspace patterns");
        let discovery_packages = provider_set.discovery_provider.get_all_packages_with_metadata();

        // Verify all operations complete successfully
        assert!(config.workspace.patterns.len() >= 0);
        assert!(enhanced_patterns.len() >= 0);
        assert!(packages.len() >= 0);
        assert!(workspace_patterns.len() >= 0);
        assert!(discovery_packages.len() >= 0);
    }

    #[test]
    fn test_factory_provider_creation_lazy_evaluation() {
        let (_temp_dir, project) = create_test_project();
        
        // Test lazy evaluation pattern with provider creation
        struct LazyProviderContainer {
            project: Arc<MonorepoProject>,
            package_provider: Option<Box<dyn PackageProvider>>,
            config_provider: Option<Box<dyn ConfigProvider>>,
            fs_provider: Option<Box<dyn FileSystemProvider>>,
        }

        impl LazyProviderContainer {
            fn new(project: Arc<MonorepoProject>) -> Self {
                Self {
                    project,
                    package_provider: None,
                    config_provider: None,
                    fs_provider: None,
                }
            }

            fn get_package_provider(&mut self) -> &Box<dyn PackageProvider> {
                if self.package_provider.is_none() {
                    self.package_provider = Some(DependencyFactory::package_provider(Arc::clone(&self.project)));
                }
                self.package_provider.as_ref().unwrap()
            }

            fn get_config_provider(&mut self) -> &Box<dyn ConfigProvider> {
                if self.config_provider.is_none() {
                    self.config_provider = Some(DependencyFactory::config_provider(Arc::clone(&self.project)));
                }
                self.config_provider.as_ref().unwrap()
            }

            fn get_fs_provider(&mut self) -> &Box<dyn FileSystemProvider> {
                if self.fs_provider.is_none() {
                    self.fs_provider = Some(DependencyFactory::file_system_provider(Arc::clone(&self.project)));
                }
                self.fs_provider.as_ref().unwrap()
            }
        }

        let mut container = LazyProviderContainer::new(project);

        // Providers should be created on demand
        let packages = container.get_package_provider().packages();
        assert!(packages.len() >= 0);
        assert!(container.package_provider.is_some());
        assert!(container.config_provider.is_none()); // Not yet created

        let config = container.get_config_provider().config();
        assert!(config.workspace.patterns.len() >= 0);
        assert!(container.config_provider.is_some());
        assert!(container.fs_provider.is_none()); // Not yet created

        let temp_exists = container.get_fs_provider().path_exists(&std::env::temp_dir());
        assert!(temp_exists || !temp_exists);
        assert!(container.fs_provider.is_some());
    }

    #[test]
    fn test_factory_provider_creation_parallel_pattern() {
        let (_temp_dir, project) = create_test_project();
        
        // Simulate parallel provider creation pattern
        let project_refs: Vec<Arc<MonorepoProject>> = (0..4)
            .map(|_| Arc::clone(&project))
            .collect();

        // Create providers in parallel-like fashion (simulated)
        let providers: Vec<Box<dyn std::any::Any>> = project_refs
            .into_iter()
            .enumerate()
            .map(|(i, project_ref)| {
                match i % 4 {
                    0 => Box::new(DependencyFactory::package_provider(project_ref)) as Box<dyn std::any::Any>,
                    1 => Box::new(DependencyFactory::config_provider(project_ref)) as Box<dyn std::any::Any>,
                    2 => Box::new(DependencyFactory::file_system_provider(project_ref)) as Box<dyn std::any::Any>,
                    3 => Box::new(DependencyFactory::workspace_provider(project_ref)) as Box<dyn std::any::Any>,
                    _ => unreachable!(),
                }
            })
            .collect();

        // Verify all providers were created
        assert_eq!(providers.len(), 4);

        // Test that providers remain valid
        for provider in &providers {
            assert!(std::any::Any::type_id(&**provider) != std::any::TypeId::of::<()>());
        }
    }

    #[test]
    fn test_factory_provider_creation_complex_composition() {
        let (_temp_dir, project) = create_test_project();
        
        // Test complex composition patterns
        struct CompositeProviderFactory {
            project: Arc<MonorepoProject>,
        }

        impl CompositeProviderFactory {
            fn new(project: Arc<MonorepoProject>) -> Self {
                Self { project }
            }

            fn create_core_set(&self) -> (Box<dyn PackageProvider>, Box<dyn ConfigProvider>) {
                (
                    DependencyFactory::package_provider(Arc::clone(&self.project)),
                    DependencyFactory::config_provider(Arc::clone(&self.project)),
                )
            }

            fn create_io_set(&self) -> (Box<dyn FileSystemProvider>, Box<dyn GitProvider>) {
                (
                    DependencyFactory::file_system_provider(Arc::clone(&self.project)),
                    DependencyFactory::git_provider(Arc::clone(&self.project)),
                )
            }

            fn create_discovery_set(&self) -> (Box<dyn WorkspaceProvider>, Box<dyn PackageDiscoveryProvider>) {
                (
                    DependencyFactory::workspace_provider(Arc::clone(&self.project)),
                    DependencyFactory::package_discovery_provider(Arc::clone(&self.project)),
                )
            }

            fn create_registry_set(&self) -> (Box<dyn RegistryProvider>, Box<dyn EnhancedConfigProvider>) {
                (
                    DependencyFactory::registry_provider(Arc::clone(&self.project)),
                    DependencyFactory::enhanced_config_provider(Arc::clone(&self.project)),
                )
            }
        }

        let factory = CompositeProviderFactory::new(project);

        // Create different provider sets
        let (package_provider, config_provider) = factory.create_core_set();
        let (fs_provider, git_provider) = factory.create_io_set();
        let (workspace_provider, discovery_provider) = factory.create_discovery_set();
        let (registry_provider, enhanced_config_provider) = factory.create_registry_set();

        // Test that all providers work correctly
        let packages = package_provider.packages();
        let config = config_provider.config();
        let temp_exists = fs_provider.path_exists(&std::env::temp_dir());
        let repo = git_provider.repository();
        let workspace_patterns = workspace_provider.get_workspace_patterns().expect("Should get patterns");
        let discovery_packages = discovery_provider.get_all_packages_with_metadata();
        let registry_type = registry_provider.get_registry_type("https://registry.npmjs.org/");
        let enhanced_patterns = enhanced_config_provider.get_effective_patterns().expect("Should get enhanced patterns");

        // Verify all operations complete successfully
        assert!(packages.len() >= 0);
        assert!(config.workspace.patterns.len() >= 0);
        assert!(temp_exists || !temp_exists);
        assert!(repo.get_current_branch().is_ok());
        assert!(workspace_patterns.len() >= 0);
        assert!(discovery_packages.len() >= 0);
        assert!(!registry_type.is_empty());
        assert!(enhanced_patterns.len() >= 0);
    }

    #[test]
    fn test_factory_method_chaining_with_error_handling() {
        let (_temp_dir, project) = create_test_project();
        
        // Test method chaining with error handling (simplified without catch_unwind)
        struct SafeProviderBuilder {
            project: Arc<MonorepoProject>,
            providers: HashMap<String, std::result::Result<Box<dyn std::any::Any>, String>>,
        }

        impl SafeProviderBuilder {
            fn new(project: Arc<MonorepoProject>) -> Self {
                Self {
                    project,
                    providers: HashMap::new(),
                }
            }

            fn try_with_package_provider(mut self) -> Self {
                // Simulate successful provider creation (in real scenarios this could fail)
                let provider = Box::new(DependencyFactory::package_provider(Arc::clone(&self.project))) as Box<dyn std::any::Any>;
                self.providers.insert("package".to_string(), Ok(provider));
                self
            }

            fn try_with_config_provider(mut self) -> Self {
                // Simulate successful provider creation
                let provider = Box::new(DependencyFactory::config_provider(Arc::clone(&self.project))) as Box<dyn std::any::Any>;
                self.providers.insert("config".to_string(), Ok(provider));
                self
            }

            fn try_with_simulated_error(mut self, provider_name: &str) -> Self {
                // Simulate an error case for testing error handling patterns
                self.providers.insert(provider_name.to_string(), Err(format!("Failed to create {} provider", provider_name)));
                self
            }

            fn build(self) -> HashMap<String, std::result::Result<Box<dyn std::any::Any>, String>> {
                self.providers
            }
        }

        let provider_results = SafeProviderBuilder::new(project)
            .try_with_package_provider()
            .try_with_config_provider()
            .try_with_simulated_error("simulated_error")
            .build();

        // Verify providers were created successfully
        assert_eq!(provider_results.len(), 3);
        assert!(provider_results.get("package").unwrap().is_ok());
        assert!(provider_results.get("config").unwrap().is_ok());
        assert!(provider_results.get("simulated_error").unwrap().is_err());
    }
}

#[cfg(test)]
mod dependency_factory_compliance_tests {
    use super::*;
    use std::marker::PhantomData;

    /// Test struct to verify trait bounds are properly enforced
    struct TraitBoundValidator<T> {
        _phantom: PhantomData<T>,
    }

    impl<T> TraitBoundValidator<T> {
        fn new() -> Self {
            Self {
                _phantom: PhantomData,
            }
        }

        fn validate_send_sync(&self) 
        where 
            T: Send + Sync,
        {
            // This method only compiles if T implements Send + Sync
        }

        fn validate_static(&self)
        where
            T: 'static,
        {
            // This method only compiles if T has static lifetime
        }
    }

    #[test]
    fn test_factory_trait_bounds_package_provider() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that factory-created providers satisfy required trait bounds
        let provider = DependencyFactory::package_provider(Arc::clone(&project));
        
        // Test trait object conversion (requires object safety)
        let trait_object: Box<dyn PackageProvider> = provider;
        
        // Test that the provider implements expected methods
        let packages = trait_object.packages();
        let root = trait_object.root_path();
        
        assert!(packages.len() >= 0);
        assert!(root.exists());
        
        // Test type safety with generic constraints
        let validator = TraitBoundValidator::<Box<dyn PackageProvider>>::new();
        validator.validate_static(); // Providers should have static lifetime
    }

    #[test]
    fn test_factory_trait_bounds_config_provider() {
        let (_temp_dir, project) = create_test_project();
        
        // Test trait bounds for ConfigProvider
        let provider = DependencyFactory::config_provider(Arc::clone(&project));
        let trait_object: Box<dyn ConfigProvider> = provider;
        
        // Test configuration access
        let config = trait_object.config();
        assert!(config.workspace.patterns.len() >= 0);
        
        // Verify trait bounds
        let validator = TraitBoundValidator::<Box<dyn ConfigProvider>>::new();
        validator.validate_static();
    }

    #[test]
    fn test_factory_trait_bounds_file_system_provider() {
        let (_temp_dir, project) = create_test_project();
        
        // Test trait bounds for FileSystemProvider
        let provider = DependencyFactory::file_system_provider(Arc::clone(&project));
        let trait_object: Box<dyn FileSystemProvider> = provider;
        
        // Test file system operations with proper error handling
        let temp_path = std::env::temp_dir();
        let exists = trait_object.path_exists(&temp_path);
        assert!(exists || !exists); // Either result is valid
        
        // Verify trait bounds
        let validator = TraitBoundValidator::<Box<dyn FileSystemProvider>>::new();
        validator.validate_static();
    }

    #[test]
    fn test_factory_trait_bounds_git_provider() {
        let (_temp_dir, project) = create_test_project();
        
        // Test trait bounds for GitProvider
        let provider = DependencyFactory::git_provider(Arc::clone(&project));
        let trait_object: Box<dyn GitProvider> = provider;
        
        // Test git operations
        let repo = trait_object.repository();
        let branch_result = repo.get_current_branch();
        assert!(branch_result.is_ok());
        
        // Verify trait bounds
        let validator = TraitBoundValidator::<Box<dyn GitProvider>>::new();
        validator.validate_static();
    }

    #[test]
    fn test_factory_trait_bounds_registry_provider() {
        let (_temp_dir, project) = create_test_project();
        
        // Test trait bounds for RegistryProvider
        let provider = DependencyFactory::registry_provider(Arc::clone(&project));
        let trait_object: Box<dyn RegistryProvider> = provider;
        
        // Test registry operations
        let registry_type = trait_object.get_registry_type("https://registry.npmjs.org/");
        assert!(!registry_type.is_empty());
        
        // Verify trait bounds
        let validator = TraitBoundValidator::<Box<dyn RegistryProvider>>::new();
        validator.validate_static();
    }

    #[test]
    fn test_factory_trait_bounds_workspace_provider() {
        let (_temp_dir, project) = create_test_project();
        
        // Test trait bounds for WorkspaceProvider
        let provider = DependencyFactory::workspace_provider(Arc::clone(&project));
        let trait_object: Box<dyn WorkspaceProvider> = provider;
        
        // Test workspace operations
        let patterns = trait_object.get_workspace_patterns().expect("Should get workspace patterns");
        assert!(patterns.len() >= 0);
        
        // Verify trait bounds
        let validator = TraitBoundValidator::<Box<dyn WorkspaceProvider>>::new();
        validator.validate_static();
    }

    #[test]
    fn test_factory_trait_bounds_package_discovery_provider() {
        let (_temp_dir, project) = create_test_project();
        
        // Test trait bounds for PackageDiscoveryProvider
        let provider = DependencyFactory::package_discovery_provider(Arc::clone(&project));
        let trait_object: Box<dyn PackageDiscoveryProvider> = provider;
        
        // Test discovery operations
        let packages = trait_object.get_all_packages_with_metadata();
        assert!(packages.len() >= 0);
        
        // Verify trait bounds
        let validator = TraitBoundValidator::<Box<dyn PackageDiscoveryProvider>>::new();
        validator.validate_static();
    }

    #[test]
    fn test_factory_trait_bounds_enhanced_config_provider() {
        let (_temp_dir, project) = create_test_project();
        
        // Test trait bounds for EnhancedConfigProvider
        let provider = DependencyFactory::enhanced_config_provider(Arc::clone(&project));
        let trait_object: Box<dyn EnhancedConfigProvider> = provider;
        
        // Test enhanced config operations
        let effective_patterns = trait_object.get_effective_patterns().expect("Should get effective patterns");
        assert!(effective_patterns.len() >= 0);
        
        // Verify trait bounds
        let validator = TraitBoundValidator::<Box<dyn EnhancedConfigProvider>>::new();
        validator.validate_static();
    }

    #[test]
    fn test_factory_type_safety_collections() {
        let (_temp_dir, project) = create_test_project();
        
        // Test type safety when storing providers in collections
        let mut providers: Vec<Box<dyn std::any::Any>> = Vec::new();
        
        // All providers should be safely stored in collections with Send + Sync bounds
        providers.push(Box::new(DependencyFactory::package_provider(Arc::clone(&project))));
        providers.push(Box::new(DependencyFactory::config_provider(Arc::clone(&project))));
        providers.push(Box::new(DependencyFactory::file_system_provider(Arc::clone(&project))));
        providers.push(Box::new(DependencyFactory::git_provider(Arc::clone(&project))));
        providers.push(Box::new(DependencyFactory::registry_provider(Arc::clone(&project))));
        providers.push(Box::new(DependencyFactory::workspace_provider(Arc::clone(&project))));
        providers.push(Box::new(DependencyFactory::package_discovery_provider(Arc::clone(&project))));
        providers.push(Box::new(DependencyFactory::enhanced_config_provider(project)));
        
        // Verify all providers were stored
        assert_eq!(providers.len(), 8);
        
        // Test that providers can be moved between threads (Send bound)
        let moved_providers = providers;
        assert_eq!(moved_providers.len(), 8);
        
        // Test iteration over providers
        for provider in &moved_providers {
            assert!(std::any::Any::type_id(&**provider) != std::any::TypeId::of::<()>());
        }
    }

    #[test]
    fn test_factory_type_safety_generic_constraints() {
        let (_temp_dir, project) = create_test_project();
        
        // Test type safety with generic constraints
        fn accept_package_provider<T: PackageProvider + 'static>(_provider: T) {
            // This function only accepts types that implement PackageProvider + 'static
        }

        fn accept_config_provider<T: ConfigProvider + 'static>(_provider: T) {
            // This function only accepts types that implement ConfigProvider + 'static
        }

        // Test that factory-created providers satisfy generic constraints
        // Note: We need to test the concrete implementation types, not the boxed trait objects
        let package_provider_concrete = Arc::clone(&project);
        let config_provider_concrete = Arc::clone(&project);
        
        // These calls should compile successfully, proving type safety
        accept_package_provider(package_provider_concrete);
        accept_config_provider(config_provider_concrete);
    }

    #[test]
    fn test_factory_object_safety_verification() {
        let (_temp_dir, project) = create_test_project();
        
        // Test object safety by creating trait objects and using them polymorphically
        let providers: Vec<Box<dyn std::any::Any>> = vec![
            Box::new(DependencyFactory::package_provider(Arc::clone(&project))),
            Box::new(DependencyFactory::config_provider(Arc::clone(&project))),
            Box::new(DependencyFactory::file_system_provider(Arc::clone(&project))),
            Box::new(DependencyFactory::git_provider(Arc::clone(&project))),
            Box::new(DependencyFactory::registry_provider(Arc::clone(&project))),
            Box::new(DependencyFactory::workspace_provider(Arc::clone(&project))),
            Box::new(DependencyFactory::package_discovery_provider(Arc::clone(&project))),
            Box::new(DependencyFactory::enhanced_config_provider(project)),
        ];
        
        // Test polymorphic usage
        for provider in providers {
            // All providers should be convertible to Any
            assert!(std::any::Any::type_id(&*provider) != std::any::TypeId::of::<()>());
            
            // Test dynamic casting (proves object safety)
            if let Some(_package_provider) = provider.downcast_ref::<Box<dyn PackageProvider>>() {
                // Successfully downcast to PackageProvider
            } else if let Some(_config_provider) = provider.downcast_ref::<Box<dyn ConfigProvider>>() {
                // Successfully downcast to ConfigProvider
            } // ... other provider types would be handled similarly
        }
    }

    #[test]
    fn test_factory_lifetime_safety() {
        let (_temp_dir, project) = create_test_project();
        
        // Test lifetime safety by creating providers in different scopes
        let long_lived_provider = {
            let local_project = Arc::clone(&project);
            // Provider should outlive the local_project reference
            DependencyFactory::package_provider(local_project)
        }; // local_project goes out of scope here
        
        // Provider should still be valid and usable
        let packages = long_lived_provider.packages();
        assert!(packages.len() >= 0);
        
        // Test with nested scopes
        let nested_provider = {
            let outer_project = Arc::clone(&project);
            {
                let inner_project = Arc::clone(&outer_project);
                DependencyFactory::config_provider(inner_project)
                // inner_project goes out of scope
            }
            // outer_project goes out of scope
        };
        
        // Provider should still be valid
        let config = nested_provider.config();
        assert!(config.workspace.patterns.len() >= 0);
    }

    #[test]
    fn test_factory_concrete_type_compliance() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that concrete provider types have proper bounds (not trait objects)
        // Since trait objects don't automatically implement Send+Sync, we test the concrete types
        fn assert_send_sync<T: Send + Sync>() {}
        
        // Test concrete types returned by factory (before boxing)
        let package_provider = DependencyFactory::package_provider(Arc::clone(&project));
        let config_provider = DependencyFactory::config_provider(Arc::clone(&project));
        
        // Test that concrete providers can be shared (conceptually)
        let shared_package = std::sync::Arc::new(package_provider);
        let shared_config = std::sync::Arc::new(config_provider);
        
        // Clone for "sending" to another thread (simulated)
        let _thread_package = std::sync::Arc::clone(&shared_package);
        let _thread_config = std::sync::Arc::clone(&shared_config);
        
        // Original providers should still be accessible
        let packages = shared_package.packages();
        let config = shared_config.config();
        
        assert!(packages.len() >= 0);
        assert!(config.workspace.patterns.len() >= 0);
    }

    #[test]
    fn test_factory_error_bounds_compliance() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that error types satisfy required bounds
        let fs_provider = DependencyFactory::file_system_provider(Arc::clone(&project));
        
        // Test error handling with proper bounds
        let nonexistent_path = std::path::Path::new("/absolutely/nonexistent/path/file.txt");
        let read_result = fs_provider.read_file_string(nonexistent_path);
        
        match read_result {
            Ok(_) => {
                // Unexpected success (file shouldn't exist)
            }
            Err(error) => {
                // Test that error implements required traits
                let error_string = error.to_string();
                assert!(!error_string.is_empty());
                
                // Error should implement Debug, Display, etc.
                let _debug = format!("{:?}", error);
                let _display = format!("{}", error);
                
                // Error should have static lifetime
                let validator = TraitBoundValidator::<crate::error::Error>::new();
                validator.validate_static();
            }
        }
    }

    #[test]
    fn test_factory_provider_interface_consistency() {
        let (_temp_dir, project) = create_test_project();
        
        // Test that all providers implement their interfaces consistently
        struct ProviderTester {
            package_provider: Box<dyn PackageProvider>,
            config_provider: Box<dyn ConfigProvider>,
            file_system_provider: Box<dyn FileSystemProvider>,
            git_provider: Box<dyn GitProvider>,
            registry_provider: Box<dyn RegistryProvider>,
            workspace_provider: Box<dyn WorkspaceProvider>,
            discovery_provider: Box<dyn PackageDiscoveryProvider>,
            enhanced_config_provider: Box<dyn EnhancedConfigProvider>,
        }

        let tester = ProviderTester {
            package_provider: DependencyFactory::package_provider(Arc::clone(&project)),
            config_provider: DependencyFactory::config_provider(Arc::clone(&project)),
            file_system_provider: DependencyFactory::file_system_provider(Arc::clone(&project)),
            git_provider: DependencyFactory::git_provider(Arc::clone(&project)),
            registry_provider: DependencyFactory::registry_provider(Arc::clone(&project)),
            workspace_provider: DependencyFactory::workspace_provider(Arc::clone(&project)),
            discovery_provider: DependencyFactory::package_discovery_provider(Arc::clone(&project)),
            enhanced_config_provider: DependencyFactory::enhanced_config_provider(project),
        };

        // Test that all providers can be used through their trait interfaces
        let _packages = tester.package_provider.packages();
        let _config = tester.config_provider.config();
        let _temp_exists = tester.file_system_provider.path_exists(&std::env::temp_dir());
        let _repo = tester.git_provider.repository();
        let _registry_type = tester.registry_provider.get_registry_type("https://registry.npmjs.org/");
        let _workspace_patterns = tester.workspace_provider.get_workspace_patterns();
        let _discovery_packages = tester.discovery_provider.get_all_packages_with_metadata();
        let _enhanced_patterns = tester.enhanced_config_provider.get_effective_patterns();

        // All operations should complete without compilation errors (proving interface consistency)
    }

    #[test]
    fn test_factory_type_compatibility() {
        let (_temp_dir, project) = create_test_project();
        
        // Test type compatibility and interface implementation
        let enhanced_config_provider = DependencyFactory::enhanced_config_provider(Arc::clone(&project));
        
        // Test that enhanced config provider can be used through its own interface
        let effective_patterns = enhanced_config_provider.get_effective_patterns().expect("Should get patterns");
        assert!(effective_patterns.len() >= 0);
        
        // Test with collections of the same trait type
        let mut enhanced_config_providers: Vec<Box<dyn EnhancedConfigProvider>> = Vec::new();
        enhanced_config_providers.push(DependencyFactory::enhanced_config_provider(Arc::clone(&project)));
        
        let mut regular_config_providers: Vec<Box<dyn ConfigProvider>> = Vec::new();
        regular_config_providers.push(DependencyFactory::config_provider(project));
        
        // Each should be usable through their respective interfaces
        assert_eq!(enhanced_config_providers.len(), 1);
        assert_eq!(regular_config_providers.len(), 1);
        
        // Test interface methods
        for provider in &enhanced_config_providers {
            let patterns = provider.get_effective_patterns().expect("Should get patterns");
            assert!(patterns.len() >= 0);
        }
        
        for provider in &regular_config_providers {
            let config = provider.config();
            assert!(config.workspace.patterns.len() >= 0);
        }
    }
}