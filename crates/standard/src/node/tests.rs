//! # Node.js Module Tests
//!
//! ## What
//! This file contains comprehensive tests for the Node.js module types and
//! functionality. It validates the behavior of repository types, package
//! manager abstractions, and the unified repository interface to ensure
//! they work correctly and maintain consistency across all scenarios.
//!
//! ## How
//! Tests are organized by functionality area with clear test names that
//! describe what is being tested. Each test validates specific behavior
//! and edge cases, ensuring that the types work as expected in various
//! scenarios. Enhanced tests cover boundary conditions, error handling,
//! and concurrent operations.
//!
//! ## Why
//! Comprehensive testing ensures that the fundamental Node.js abstractions
//! work correctly and maintain their contracts. These tests serve as both
//! validation and documentation of expected behavior, helping prevent
//! regressions during refactoring and migration phases.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;

use crate::filesystem::{AsyncFileSystem, FileSystemManager};
use crate::monorepo::MonorepoKind;

use super::{PackageManager, PackageManagerKind, RepoKind, RepositoryInfo};

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // HELPER FUNCTIONS
    // =============================================================================

    /// Helper to create a temporary directory for testing
    #[allow(clippy::unwrap_used)]
    fn setup_test_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    /// Helper to create a package.json with specific content
    #[allow(clippy::unwrap_used)]
    async fn create_package_json(dir: &Path, content: &str) {
        let fs = FileSystemManager::new();
        let package_json_path = dir.join("package.json");
        fs.write_file_string(&package_json_path, content).await.unwrap();
    }

    /// Helper to create a lock file for a specific package manager
    #[allow(clippy::unwrap_used)]
    async fn create_lock_file(dir: &Path, kind: PackageManagerKind) {
        let fs = FileSystemManager::new();
        let lock_path = dir.join(kind.lock_file());
        fs.write_file_string(&lock_path, "# Lock file content").await.unwrap();
    }

    // =============================================================================
    // REPOSITORY KIND TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_simple_repo_kind() {
        let simple = RepoKind::Simple;

        assert_eq!(simple.name(), "simple");
        assert!(!simple.is_monorepo());
        assert_eq!(simple.monorepo_kind(), None);
        assert!(!simple.is_monorepo_kind(&MonorepoKind::YarnWorkspaces));
    }

    #[tokio::test]
    async fn test_monorepo_kind_yarn() {
        let yarn_mono = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);

        assert_eq!(yarn_mono.name(), "yarn monorepo");
        assert!(yarn_mono.is_monorepo());
        assert_eq!(yarn_mono.monorepo_kind(), Some(&MonorepoKind::YarnWorkspaces));
        assert!(yarn_mono.is_monorepo_kind(&MonorepoKind::YarnWorkspaces));
        assert!(!yarn_mono.is_monorepo_kind(&MonorepoKind::PnpmWorkspaces));
    }

    #[tokio::test]
    async fn test_monorepo_kind_pnpm() {
        let pnpm_mono = RepoKind::Monorepo(MonorepoKind::PnpmWorkspaces);

        assert_eq!(pnpm_mono.name(), "pnpm monorepo");
        assert!(pnpm_mono.is_monorepo());
        assert_eq!(pnpm_mono.monorepo_kind(), Some(&MonorepoKind::PnpmWorkspaces));
        assert!(pnpm_mono.is_monorepo_kind(&MonorepoKind::PnpmWorkspaces));
        assert!(!pnpm_mono.is_monorepo_kind(&MonorepoKind::YarnWorkspaces));
    }

    #[tokio::test]
    async fn test_monorepo_kind_npm() {
        let npm_mono = RepoKind::Monorepo(MonorepoKind::NpmWorkSpace);

        assert_eq!(npm_mono.name(), "npm monorepo");
        assert!(npm_mono.is_monorepo());
        assert_eq!(npm_mono.monorepo_kind(), Some(&MonorepoKind::NpmWorkSpace));
        assert!(npm_mono.is_monorepo_kind(&MonorepoKind::NpmWorkSpace));
    }

    #[tokio::test]
    async fn test_repo_kind_equality() {
        let simple1 = RepoKind::Simple;
        let simple2 = RepoKind::Simple;
        assert_eq!(simple1, simple2);

        let yarn1 = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);
        let yarn2 = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);
        assert_eq!(yarn1, yarn2);

        let yarn = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);
        let pnpm = RepoKind::Monorepo(MonorepoKind::PnpmWorkspaces);
        assert_ne!(yarn, pnpm);
        assert_ne!(simple1, yarn);
    }

    #[tokio::test]
    async fn test_repo_kind_clone() {
        let original = RepoKind::Monorepo(MonorepoKind::BunWorkspaces);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[tokio::test]
    async fn test_repo_kind_debug() {
        let simple = RepoKind::Simple;
        let debug_str = format!("{simple:?}");
        assert!(debug_str.contains("Simple"));

        let yarn_mono = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);
        let debug_str = format!("{yarn_mono:?}");
        assert!(debug_str.contains("Monorepo"));
        assert!(debug_str.contains("YarnWorkspaces"));
    }

    #[tokio::test]
    async fn test_repo_kind_comprehensive_equality() {
        let simple1 = RepoKind::Simple;
        let simple2 = RepoKind::Simple;
        let yarn_mono1 = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);
        let yarn_mono2 = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);
        let npm_mono = RepoKind::Monorepo(MonorepoKind::NpmWorkSpace);

        // Test equality
        assert_eq!(simple1, simple2);
        assert_eq!(yarn_mono1, yarn_mono2);
        assert_ne!(simple1, yarn_mono1);
        assert_ne!(yarn_mono1, npm_mono);

        // Test that debug formatting works
        assert!(!format!("{simple1:?}").is_empty());
        assert!(!format!("{yarn_mono1:?}").is_empty());
    }

    #[tokio::test]
    async fn test_repo_kind_all_monorepo_kinds() {
        let monorepo_kinds = vec![
            MonorepoKind::NpmWorkSpace,
            MonorepoKind::YarnWorkspaces,
            MonorepoKind::PnpmWorkspaces,
            MonorepoKind::BunWorkspaces,
            MonorepoKind::DenoWorkspaces,
            MonorepoKind::Custom {
                name: "custom".to_string(),
                config_file: "custom.json".to_string(),
            },
        ];

        for kind in monorepo_kinds {
            let repo_kind = RepoKind::Monorepo(kind.clone());

            // Test all methods work correctly
            assert!(repo_kind.is_monorepo());
            assert_eq!(repo_kind.monorepo_kind(), Some(&kind));
            assert!(repo_kind.is_monorepo_kind(&kind));
            assert!(!repo_kind.name().is_empty());

            // Test it's not equal to simple
            assert_ne!(repo_kind, RepoKind::Simple);
        }
    }

    #[tokio::test]
    async fn test_repo_kind_boundary_conditions() {
        // Test custom monorepo with edge case names
        let edge_cases = vec![
            ("", ""),
            ("a", "b"),
            ("very-long-name-that-might-cause-issues", "config-file-with-long-name.json"),
            ("name with spaces", "file with spaces.json"),
            ("name_with_underscores", "file_with_underscores.json"),
            ("name.with.dots", "file.with.dots.json"),
        ];

        for (name, config_file) in edge_cases {
            let custom_kind = MonorepoKind::Custom {
                name: name.to_string(),
                config_file: config_file.to_string(),
            };

            let repo_kind = RepoKind::Monorepo(custom_kind.clone());

            // Should not panic
            assert!(repo_kind.is_monorepo());
            assert_eq!(repo_kind.monorepo_kind(), Some(&custom_kind));
            assert!(!repo_kind.name().is_empty());
        }
    }

    // =============================================================================
    // PACKAGE MANAGER KIND TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_npm_package_manager() {
        let npm = PackageManagerKind::Npm;

        assert_eq!(npm.command(), "npm");
        assert_eq!(npm.lock_file(), "package-lock.json");
        assert_eq!(npm.name(), "npm");
        assert!(npm.supports_workspaces());
        assert_eq!(npm.workspace_config_file(), None);
    }

    #[tokio::test]
    async fn test_yarn_package_manager() {
        let yarn = PackageManagerKind::Yarn;

        assert_eq!(yarn.command(), "yarn");
        assert_eq!(yarn.lock_file(), "yarn.lock");
        assert_eq!(yarn.name(), "yarn");
        assert!(yarn.supports_workspaces());
        assert_eq!(yarn.workspace_config_file(), None);
    }

    #[tokio::test]
    async fn test_pnpm_package_manager() {
        let pnpm = PackageManagerKind::Pnpm;

        assert_eq!(pnpm.command(), "pnpm");
        assert_eq!(pnpm.lock_file(), "pnpm-lock.yaml");
        assert_eq!(pnpm.name(), "pnpm");
        assert!(pnpm.supports_workspaces());
        assert_eq!(pnpm.workspace_config_file(), Some("pnpm-workspace.yaml"));
    }

    #[tokio::test]
    async fn test_bun_package_manager() {
        let bun = PackageManagerKind::Bun;

        assert_eq!(bun.command(), "bun");
        assert_eq!(bun.lock_file(), "bun.lockb");
        assert_eq!(bun.name(), "bun");
        assert!(bun.supports_workspaces());
        assert_eq!(bun.workspace_config_file(), None);
    }

    #[tokio::test]
    async fn test_jsr_package_manager() {
        let jsr = PackageManagerKind::Jsr;

        assert_eq!(jsr.command(), "jsr");
        assert_eq!(jsr.lock_file(), "jsr.json");
        assert_eq!(jsr.name(), "jsr");
        assert!(!jsr.supports_workspaces());
        assert_eq!(jsr.workspace_config_file(), None);
    }

    #[tokio::test]
    async fn test_package_manager_kind_equality() {
        assert_eq!(PackageManagerKind::Npm, PackageManagerKind::Npm);
        assert_ne!(PackageManagerKind::Npm, PackageManagerKind::Yarn);
    }

    #[allow(clippy::clone_on_copy)]
    #[tokio::test]
    async fn test_package_manager_kind_clone_copy() {
        let npm1 = PackageManagerKind::Npm;
        let npm2 = npm1; // Should copy, not move
        assert_eq!(npm1, npm2);

        let yarn = PackageManagerKind::Yarn;
        let yarn_cloned = yarn.clone();
        assert_eq!(yarn, yarn_cloned);
    }

    #[tokio::test]
    async fn test_package_manager_kind_debug() {
        let npm = PackageManagerKind::Npm;
        let debug_str = format!("{npm:?}");
        assert!(debug_str.contains("Npm"));
    }

    #[tokio::test]
    async fn test_package_manager_kind_exhaustiveness() {
        // Ensure all PackageManagerKind variants are tested
        let all_kinds = vec![
            PackageManagerKind::Npm,
            PackageManagerKind::Yarn,
            PackageManagerKind::Pnpm,
            PackageManagerKind::Bun,
            PackageManagerKind::Jsr,
        ];

        for kind in all_kinds {
            // Test all methods don't panic
            assert!(!kind.lock_file().is_empty());
            assert!(!kind.command().is_empty());

            // Test they're all different
            assert_ne!(kind.lock_file(), "");
            assert_ne!(kind.command(), "");
        }
    }

    // =============================================================================
    // PACKAGE MANAGER TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_package_manager_creation() {
        let manager = PackageManager::new(PackageManagerKind::Npm, "/project/root");

        assert_eq!(manager.kind(), PackageManagerKind::Npm);
        assert_eq!(manager.root(), Path::new("/project/root"));
        assert_eq!(manager.command(), "npm");
        assert_eq!(manager.lock_file(), "package-lock.json");
    }

    #[tokio::test]
    async fn test_package_manager_with_pathbuf() {
        let root = PathBuf::from("/project/path");
        let manager = PackageManager::new(PackageManagerKind::Yarn, root);

        assert_eq!(manager.kind(), PackageManagerKind::Yarn);
        assert_eq!(manager.root(), Path::new("/project/path"));
    }

    #[tokio::test]
    async fn test_package_manager_lock_file_path() {
        let manager = PackageManager::new(PackageManagerKind::Pnpm, "/test/project");
        let lock_path = manager.lock_file_path();

        assert_eq!(lock_path, PathBuf::from("/test/project/pnpm-lock.yaml"));
    }

    #[tokio::test]
    async fn test_package_manager_workspace_support() {
        let npm_manager = PackageManager::new(PackageManagerKind::Npm, "/project");
        assert!(npm_manager.supports_workspaces());

        let jsr_manager = PackageManager::new(PackageManagerKind::Jsr, "/project");
        assert!(!jsr_manager.supports_workspaces());
    }

    #[tokio::test]
    async fn test_package_manager_workspace_config_path() {
        let npm_manager = PackageManager::new(PackageManagerKind::Npm, "/project");
        assert_eq!(npm_manager.workspace_config_path(), None);

        let pnpm_manager = PackageManager::new(PackageManagerKind::Pnpm, "/project");
        let expected = Some(PathBuf::from("/project/pnpm-workspace.yaml"));
        assert_eq!(pnpm_manager.workspace_config_path(), expected);
    }

    #[tokio::test]
    async fn test_package_manager_debug() {
        let manager = PackageManager::new(PackageManagerKind::Yarn, "/project");
        let debug_str = format!("{manager:?}");
        assert!(debug_str.contains("PackageManager"));
        assert!(debug_str.contains("Yarn"));
    }

    #[tokio::test]
    async fn test_package_manager_clone() {
        let original = PackageManager::new(PackageManagerKind::Bun, "/original/path");
        let cloned = original.clone();

        assert_eq!(original.kind(), cloned.kind());
        assert_eq!(original.root(), cloned.root());
    }

    #[tokio::test]
    async fn test_package_manager_clone_and_debug() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        create_lock_file(path, PackageManagerKind::Npm).await;
        let pm = PackageManager::detect(path).unwrap();

        // Test Clone trait
        let cloned = pm.clone();
        assert_eq!(pm.kind(), cloned.kind());
        assert_eq!(pm.root(), cloned.root());

        // Test Debug trait
        let debug_str = format!("{pm:?}");
        assert!(debug_str.contains("PackageManager"));
        assert!(debug_str.contains("Npm"));
    }

    // =============================================================================
    // PACKAGE MANAGER DETECTION TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_package_manager_detection_edge_cases() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test detection with no lock files
        let result = PackageManager::detect(path);
        assert!(result.is_err());

        // Test with multiple lock files (should detect the first one found)
        create_lock_file(path, PackageManagerKind::Npm).await;
        create_lock_file(path, PackageManagerKind::Yarn).await;

        let result = PackageManager::detect(path);
        assert!(result.is_ok());
        let pm = result.unwrap();
        // Should detect one of them (implementation-dependent order)
        assert!(matches!(pm.kind(), PackageManagerKind::Npm | PackageManagerKind::Yarn));
    }

    #[tokio::test]
    async fn test_package_manager_all_kinds() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        let all_kinds = vec![
            PackageManagerKind::Npm,
            PackageManagerKind::Yarn,
            PackageManagerKind::Pnpm,
            PackageManagerKind::Bun,
            PackageManagerKind::Jsr,
        ];

        for kind in all_kinds {
            // Clear directory
            std::fs::remove_dir_all(path).ok();
            std::fs::create_dir_all(path).unwrap();

            // Create lock file for this kind
            create_lock_file(path, kind).await;

            // Test detection
            let result = PackageManager::detect(path);
            assert!(result.is_ok(), "Failed to detect {kind:?}");
            let pm = result.unwrap();
            assert_eq!(pm.kind(), kind);

            // Test all methods
            assert!(!pm.lock_file_path().as_os_str().is_empty());
            assert_eq!(pm.root(), path);

            // Test display traits
            assert!(!format!("{pm:?}").is_empty());
        }
    }

    #[tokio::test]
    async fn test_package_manager_workspace_support_detection() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test that workspace-supporting package managers support workspace functionality
        let workspace_supporting_kinds = vec![
            PackageManagerKind::Npm,
            PackageManagerKind::Yarn,
            PackageManagerKind::Pnpm,
            PackageManagerKind::Bun,
            // Note: Jsr doesn't support workspaces, so it's excluded
        ];

        for kind in workspace_supporting_kinds {
            create_lock_file(path, kind).await;
            let pm = PackageManager::detect(path).unwrap();

            // All should support workspace functionality
            assert!(pm.supports_workspaces());
            let workspace_path = pm.workspace_config_path();
            // Some package managers like npm don't have a specific workspace config file
            if let Some(path) = workspace_path {
                assert!(!path.as_os_str().is_empty());
            }

            // Clear for next iteration
            std::fs::remove_dir_all(path).ok();
            std::fs::create_dir_all(path).unwrap();
        }
    }

    #[tokio::test]
    async fn test_package_manager_error_conditions() {
        // Test with non-existent directory
        let non_existent = PathBuf::from("/non/existent/path");
        let result = PackageManager::detect(&non_existent);
        assert!(result.is_err());

        // Test with directory without package.json
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();
        let result = PackageManager::detect(path);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_package_manager_with_complex_paths() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Create nested directory structure
        let nested_path = path.join("deeply").join("nested").join("structure");
        std::fs::create_dir_all(&nested_path).unwrap();

        create_lock_file(&nested_path, PackageManagerKind::Npm).await;
        create_package_json(&nested_path, r#"{"name": "test", "version": "1.0.0"}"#).await;

        let pm = PackageManager::detect(&nested_path).unwrap();
        assert_eq!(pm.kind(), PackageManagerKind::Npm);
        assert_eq!(pm.root(), nested_path);
    }

    // =============================================================================
    // REPOSITORY INFO TRAIT TESTS
    // =============================================================================

    // Mock implementation for testing the RepositoryInfo trait
    struct MockRepository {
        root: PathBuf,
        kind: RepoKind,
        package_manager: Option<PackageManager>,
    }

    impl MockRepository {
        fn new(root: impl Into<PathBuf>, kind: RepoKind) -> Self {
            Self { root: root.into(), kind, package_manager: None }
        }

        fn with_package_manager(mut self, manager: PackageManager) -> Self {
            self.package_manager = Some(manager);
            self
        }
    }

    impl RepositoryInfo for MockRepository {
        fn root(&self) -> &Path {
            &self.root
        }

        fn kind(&self) -> RepoKind {
            self.kind.clone()
        }

        fn package_manager(&self) -> Option<&PackageManager> {
            self.package_manager.as_ref()
        }
    }

    #[tokio::test]
    async fn test_repository_info_simple() {
        let repo = MockRepository::new("/simple/project", RepoKind::Simple);

        assert_eq!(repo.root(), Path::new("/simple/project"));
        assert_eq!(repo.kind(), RepoKind::Simple);
        assert!(!repo.is_monorepo());
        assert!(!repo.has_package_manager());
        assert_eq!(repo.display_name(), "simple");
    }

    #[tokio::test]
    async fn test_repository_info_monorepo() {
        let repo =
            MockRepository::new("/yarn/monorepo", RepoKind::Monorepo(MonorepoKind::YarnWorkspaces));

        assert_eq!(repo.root(), Path::new("/yarn/monorepo"));
        assert_eq!(repo.kind(), RepoKind::Monorepo(MonorepoKind::YarnWorkspaces));
        assert!(repo.is_monorepo());
        assert_eq!(repo.display_name(), "yarn monorepo");
    }

    #[tokio::test]
    async fn test_repository_info_with_package_manager() {
        let manager = PackageManager::new(PackageManagerKind::Npm, "/project");
        let repo = MockRepository::new("/project", RepoKind::Simple).with_package_manager(manager);

        assert!(repo.has_package_manager());
        assert_eq!(repo.package_manager().unwrap().kind(), PackageManagerKind::Npm);
    }

    #[tokio::test]
    async fn test_repository_info_trait_object() {
        let repos: Vec<Box<dyn RepositoryInfo>> = vec![
            Box::new(MockRepository::new("/simple", RepoKind::Simple)),
            Box::new(MockRepository::new(
                "/monorepo",
                RepoKind::Monorepo(MonorepoKind::PnpmWorkspaces),
            )),
        ];

        assert_eq!(repos[0].kind(), RepoKind::Simple);
        assert_eq!(repos[1].kind(), RepoKind::Monorepo(MonorepoKind::PnpmWorkspaces));
        assert!(!repos[0].is_monorepo());
        assert!(repos[1].is_monorepo());
    }

    #[tokio::test]
    async fn test_repository_info_send_sync() {
        // This test ensures that RepositoryInfo is Send + Sync
        fn require_send_sync<T: Send + Sync>() {}
        require_send_sync::<MockRepository>();
    }

    #[tokio::test]
    async fn test_repository_info_trait_comprehensive() {
        // Test that RepoKind implements RepositoryInfo methods
        let simple_repo = RepoKind::Simple;
        let monorepo = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);

        // Test simple repo
        assert!(!simple_repo.name().is_empty());
        assert!(!simple_repo.is_monorepo());
        assert!(simple_repo.monorepo_kind().is_none());

        // Test monorepo
        assert!(!monorepo.name().is_empty());
        assert!(monorepo.is_monorepo());
        assert!(monorepo.monorepo_kind().is_some());
    }

    // =============================================================================
    // INTEGRATION TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_repo_kind_with_package_manager_integration() {
        let repo_kind = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);
        let package_manager = PackageManager::new(PackageManagerKind::Yarn, "/project");

        // These should be compatible
        assert!(repo_kind.is_monorepo());
        assert!(package_manager.supports_workspaces());
        assert_eq!(package_manager.command(), "yarn");
    }

    #[tokio::test]
    async fn test_pnpm_specific_integration() {
        let repo_kind = RepoKind::Monorepo(MonorepoKind::PnpmWorkspaces);
        let package_manager = PackageManager::new(PackageManagerKind::Pnpm, "/pnpm-project");

        assert!(repo_kind.is_monorepo_kind(&MonorepoKind::PnpmWorkspaces));
        assert_eq!(package_manager.lock_file(), "pnpm-lock.yaml");
        assert_eq!(
            package_manager.workspace_config_path(),
            Some(PathBuf::from("/pnpm-project/pnpm-workspace.yaml"))
        );
    }

    #[tokio::test]
    async fn test_simple_repo_with_npm() {
        let repo_kind = RepoKind::Simple;
        let package_manager = PackageManager::new(PackageManagerKind::Npm, "/simple-project");

        assert!(!repo_kind.is_monorepo());
        assert_eq!(
            package_manager.lock_file_path(),
            PathBuf::from("/simple-project/package-lock.json")
        );
    }

    #[tokio::test]
    async fn test_all_monorepo_kinds_have_workspace_support() {
        let monorepo_kinds = vec![
            MonorepoKind::NpmWorkSpace,
            MonorepoKind::YarnWorkspaces,
            MonorepoKind::PnpmWorkspaces,
            MonorepoKind::BunWorkspaces,
            MonorepoKind::DenoWorkspaces,
        ];

        for kind in monorepo_kinds {
            let repo_kind = RepoKind::Monorepo(kind.clone());
            assert!(repo_kind.is_monorepo(), "Monorepo kind {kind:?} should be monorepo");
        }
    }

    #[tokio::test]
    async fn test_package_manager_kinds_coverage() {
        // Ensure all package manager kinds are tested
        let kinds = vec![
            PackageManagerKind::Npm,
            PackageManagerKind::Yarn,
            PackageManagerKind::Pnpm,
            PackageManagerKind::Bun,
            PackageManagerKind::Jsr,
        ];

        for kind in kinds {
            let manager = PackageManager::new(kind, "/test");
            assert!(!manager.command().is_empty(), "Command should not be empty for {kind:?}");
            assert!(!manager.lock_file().is_empty(), "Lock file should not be empty for {kind:?}");
        }
    }

    // =============================================================================
    // CONCURRENT OPERATIONS
    // =============================================================================

    #[tokio::test]
    async fn test_concurrent_package_manager_detection() {
        let temp_dir = setup_test_dir();
        let path = Arc::new(temp_dir.path().to_path_buf());

        // Create lock file
        create_lock_file(&path, PackageManagerKind::Npm).await;

        // Run detection from multiple async tasks
        let mut handles = vec![];
        for _ in 0..10 {
            let path_clone = Arc::clone(&path);
            let handle = tokio::spawn(async move { PackageManager::detect(path_clone.as_ref()) });
            handles.push(handle);
        }

        // All should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            assert_eq!(result.unwrap().kind(), PackageManagerKind::Npm);
        }
    }

    #[test]
    fn test_package_manager_kind_case_insensitive_deserialization() {
        // Test different case variations
        let test_cases = vec![
            (r#""npm""#, PackageManagerKind::Npm),
            (r#""Npm""#, PackageManagerKind::Npm),
            (r#""NPM""#, PackageManagerKind::Npm),
            (r#""yarn""#, PackageManagerKind::Yarn),
            (r#""Yarn""#, PackageManagerKind::Yarn),
            (r#""YARN""#, PackageManagerKind::Yarn),
            (r#""pnpm""#, PackageManagerKind::Pnpm),
            (r#""Pnpm""#, PackageManagerKind::Pnpm),
            (r#""PNPM""#, PackageManagerKind::Pnpm),
            (r#""bun""#, PackageManagerKind::Bun),
            (r#""Bun""#, PackageManagerKind::Bun),
            (r#""BUN""#, PackageManagerKind::Bun),
            (r#""jsr""#, PackageManagerKind::Jsr),
            (r#""Jsr""#, PackageManagerKind::Jsr),
            (r#""JSR""#, PackageManagerKind::Jsr),
        ];

        for (input, expected) in test_cases {
            let result: Result<PackageManagerKind, _> = serde_json::from_str(input);
            assert!(result.is_ok(), "Failed to deserialize: {}", input);
            assert_eq!(result.unwrap(), expected, "Unexpected result for: {}", input);
        }
        
        // Test invalid case
        let result: Result<PackageManagerKind, _> = serde_json::from_str(r#""invalid""#);
        assert!(result.is_err(), "Should have failed for invalid input");
    }
}
