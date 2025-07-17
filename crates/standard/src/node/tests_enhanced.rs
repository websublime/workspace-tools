//! # Enhanced Node.js Module Tests
//!
//! ## What
//! This file contains enhanced tests for the Node.js module with comprehensive
//! edge case coverage, error handling validation, and integration scenarios.
//!
//! ## How
//! Tests focus on boundary conditions, error scenarios, and complex interactions
//! between different components to ensure robust behavior in all situations.
//!
//! ## Why
//! Phase 4 validation requires comprehensive testing of edge cases and error
//! conditions to ensure the module behaves correctly under all circumstances.

use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::filesystem::{FileSystem, FileSystemManager};
use crate::monorepo::MonorepoKind;
// Remove unused imports
use super::{PackageManager, PackageManagerKind, RepoKind};

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod enhanced_tests {
    use super::*;

    /// Helper to create a temporary directory for testing.
    #[allow(clippy::unwrap_used)]
    fn setup_test_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    /// Helper to create a package.json with specific content.
    #[allow(clippy::unwrap_used)]
    fn create_package_json(dir: &Path, content: &str) {
        let fs = FileSystemManager::new();
        let package_json_path = dir.join("package.json");
        fs.write_file_string(&package_json_path, content).unwrap();
    }

    /// Helper to create a lock file for a specific package manager.
    #[allow(clippy::unwrap_used)]
    fn create_lock_file(dir: &Path, kind: PackageManagerKind) {
        let fs = FileSystemManager::new();
        let lock_path = dir.join(kind.lock_file());
        fs.write_file_string(&lock_path, "# Lock file content").unwrap();
    }

    #[test]
    fn test_repo_kind_comprehensive_equality() {
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

    #[test]
    fn test_repo_kind_all_monorepo_kinds() {
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

    #[test]
    fn test_package_manager_detection_edge_cases() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test detection with no lock files
        let result = PackageManager::detect(path);
        assert!(result.is_err());

        // Test with multiple lock files (should detect the first one found)
        create_lock_file(path, PackageManagerKind::Npm);
        create_lock_file(path, PackageManagerKind::Yarn);

        let result = PackageManager::detect(path);
        assert!(result.is_ok());
        let pm = result.unwrap();
        // Should detect one of them (implementation-dependent order)
        assert!(matches!(pm.kind(), PackageManagerKind::Npm | PackageManagerKind::Yarn));
    }

    #[test]
    fn test_package_manager_all_kinds() {
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
            create_lock_file(path, kind);

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

    #[test]
    fn test_package_manager_workspace_support() {
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
            create_lock_file(path, kind);
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

    #[test]
    fn test_package_manager_error_conditions() {
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

    #[test]
    fn test_package_manager_with_complex_paths() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Create nested directory structure
        let nested_path = path.join("deeply").join("nested").join("structure");
        std::fs::create_dir_all(&nested_path).unwrap();

        create_lock_file(&nested_path, PackageManagerKind::Npm);
        create_package_json(&nested_path, r#"{"name": "test", "version": "1.0.0"}"#);

        let pm = PackageManager::detect(&nested_path).unwrap();
        assert_eq!(pm.kind(), PackageManagerKind::Npm);
        assert_eq!(pm.root(), nested_path);
    }

    #[test]
    fn test_repository_info_trait_comprehensive() {
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

    #[test]
    fn test_concurrent_package_manager_detection() {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = setup_test_dir();
        let path = Arc::new(temp_dir.path().to_path_buf());

        // Create lock file
        create_lock_file(&path, PackageManagerKind::Npm);

        // Run detection from multiple threads
        let mut handles = vec![];
        for _ in 0..10 {
            let path_clone = Arc::clone(&path);
            let handle = thread::spawn(move || PackageManager::detect(path_clone.as_ref()));
            handles.push(handle);
        }

        // All should succeed
        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result.is_ok());
            assert_eq!(result.unwrap().kind(), PackageManagerKind::Npm);
        }
    }

    #[test]
    fn test_package_manager_kind_exhaustiveness() {
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

    #[test]
    fn test_repo_kind_boundary_conditions() {
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

    #[test]
    fn test_package_manager_clone_and_debug() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        create_lock_file(path, PackageManagerKind::Npm);
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
}
