//! # Node.js Module Tests
//!
//! ## What
//! This file contains comprehensive tests for the Node.js module types and
//! functionality. It validates the behavior of repository types, package
//! manager abstractions, and the unified repository interface to ensure
//! they work correctly and maintain consistency.
//!
//! ## How
//! Tests are organized by functionality area with clear test names that
//! describe what is being tested. Each test validates specific behavior
//! and edge cases, ensuring that the types work as expected in various
//! scenarios. Mock implementations are used where needed to test traits.
//!
//! ## Why
//! Comprehensive testing ensures that the fundamental Node.js abstractions
//! work correctly and maintain their contracts. These tests serve as both
//! validation and documentation of expected behavior, helping prevent
//! regressions during refactoring and migration phases.

use std::path::{Path, PathBuf};

use crate::monorepo::MonorepoKind;

use super::{PackageManager, PackageManagerKind, RepoKind, RepositoryInfo};

#[cfg(test)]
mod repo_kind_tests {
    use super::*;

    #[test]
    fn test_simple_repo_kind() {
        let simple = RepoKind::Simple;

        assert_eq!(simple.name(), "simple");
        assert!(!simple.is_monorepo());
        assert_eq!(simple.monorepo_kind(), None);
        assert!(!simple.is_monorepo_kind(&MonorepoKind::YarnWorkspaces));
    }

    #[test]
    fn test_monorepo_kind_yarn() {
        let yarn_mono = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);

        assert_eq!(yarn_mono.name(), "yarn monorepo");
        assert!(yarn_mono.is_monorepo());
        assert_eq!(yarn_mono.monorepo_kind(), Some(&MonorepoKind::YarnWorkspaces));
        assert!(yarn_mono.is_monorepo_kind(&MonorepoKind::YarnWorkspaces));
        assert!(!yarn_mono.is_monorepo_kind(&MonorepoKind::PnpmWorkspaces));
    }

    #[test]
    fn test_monorepo_kind_pnpm() {
        let pnpm_mono = RepoKind::Monorepo(MonorepoKind::PnpmWorkspaces);

        assert_eq!(pnpm_mono.name(), "pnpm monorepo");
        assert!(pnpm_mono.is_monorepo());
        assert_eq!(pnpm_mono.monorepo_kind(), Some(&MonorepoKind::PnpmWorkspaces));
        assert!(pnpm_mono.is_monorepo_kind(&MonorepoKind::PnpmWorkspaces));
        assert!(!pnpm_mono.is_monorepo_kind(&MonorepoKind::YarnWorkspaces));
    }

    #[test]
    fn test_monorepo_kind_npm() {
        let npm_mono = RepoKind::Monorepo(MonorepoKind::NpmWorkSpace);

        assert_eq!(npm_mono.name(), "npm monorepo");
        assert!(npm_mono.is_monorepo());
        assert_eq!(npm_mono.monorepo_kind(), Some(&MonorepoKind::NpmWorkSpace));
        assert!(npm_mono.is_monorepo_kind(&MonorepoKind::NpmWorkSpace));
    }

    #[test]
    fn test_repo_kind_equality() {
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

    #[test]
    fn test_repo_kind_clone() {
        let original = RepoKind::Monorepo(MonorepoKind::BunWorkspaces);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_repo_kind_debug() {
        let simple = RepoKind::Simple;
        let debug_str = format!("{simple:?}");
        assert!(debug_str.contains("Simple"));

        let yarn_mono = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);
        let debug_str = format!("{yarn_mono:?}");
        assert!(debug_str.contains("Monorepo"));
        assert!(debug_str.contains("YarnWorkspaces"));
    }
}

#[cfg(test)]
mod package_manager_kind_tests {
    use super::*;

    #[test]
    fn test_npm_package_manager() {
        let npm = PackageManagerKind::Npm;

        assert_eq!(npm.command(), "npm");
        assert_eq!(npm.lock_file(), "package-lock.json");
        assert_eq!(npm.name(), "npm");
        assert!(npm.supports_workspaces());
        assert_eq!(npm.workspace_config_file(), None);
    }

    #[test]
    fn test_yarn_package_manager() {
        let yarn = PackageManagerKind::Yarn;

        assert_eq!(yarn.command(), "yarn");
        assert_eq!(yarn.lock_file(), "yarn.lock");
        assert_eq!(yarn.name(), "yarn");
        assert!(yarn.supports_workspaces());
        assert_eq!(yarn.workspace_config_file(), None);
    }

    #[test]
    fn test_pnpm_package_manager() {
        let pnpm = PackageManagerKind::Pnpm;

        assert_eq!(pnpm.command(), "pnpm");
        assert_eq!(pnpm.lock_file(), "pnpm-lock.yaml");
        assert_eq!(pnpm.name(), "pnpm");
        assert!(pnpm.supports_workspaces());
        assert_eq!(pnpm.workspace_config_file(), Some("pnpm-workspace.yaml"));
    }

    #[test]
    fn test_bun_package_manager() {
        let bun = PackageManagerKind::Bun;

        assert_eq!(bun.command(), "bun");
        assert_eq!(bun.lock_file(), "bun.lockb");
        assert_eq!(bun.name(), "bun");
        assert!(bun.supports_workspaces());
        assert_eq!(bun.workspace_config_file(), None);
    }

    #[test]
    fn test_jsr_package_manager() {
        let jsr = PackageManagerKind::Jsr;

        assert_eq!(jsr.command(), "jsr");
        assert_eq!(jsr.lock_file(), "jsr.lock");
        assert_eq!(jsr.name(), "jsr");
        assert!(!jsr.supports_workspaces());
        assert_eq!(jsr.workspace_config_file(), None);
    }

    #[test]
    fn test_package_manager_kind_equality() {
        assert_eq!(PackageManagerKind::Npm, PackageManagerKind::Npm);
        assert_ne!(PackageManagerKind::Npm, PackageManagerKind::Yarn);
    }

    #[test]
    fn test_package_manager_kind_clone_copy() {
        let npm1 = PackageManagerKind::Npm;
        let npm2 = npm1; // Should copy, not move
        assert_eq!(npm1, npm2);

        let yarn = PackageManagerKind::Yarn;
        let yarn_cloned = yarn.clone();
        assert_eq!(yarn, yarn_cloned);
    }

    #[test]
    fn test_package_manager_kind_debug() {
        let npm = PackageManagerKind::Npm;
        let debug_str = format!("{npm:?}");
        assert!(debug_str.contains("Npm"));
    }
}

#[cfg(test)]
mod package_manager_tests {
    use super::*;

    #[test]
    fn test_package_manager_creation() {
        let manager = PackageManager::new(PackageManagerKind::Npm, "/project/root");

        assert_eq!(manager.kind(), PackageManagerKind::Npm);
        assert_eq!(manager.root(), Path::new("/project/root"));
        assert_eq!(manager.command(), "npm");
        assert_eq!(manager.lock_file(), "package-lock.json");
    }

    #[test]
    fn test_package_manager_with_pathbuf() {
        let root = PathBuf::from("/project/path");
        let manager = PackageManager::new(PackageManagerKind::Yarn, root);

        assert_eq!(manager.kind(), PackageManagerKind::Yarn);
        assert_eq!(manager.root(), Path::new("/project/path"));
    }

    #[test]
    fn test_package_manager_lock_file_path() {
        let manager = PackageManager::new(PackageManagerKind::Pnpm, "/test/project");
        let lock_path = manager.lock_file_path();

        assert_eq!(lock_path, PathBuf::from("/test/project/pnpm-lock.yaml"));
    }

    #[test]
    fn test_package_manager_workspace_support() {
        let npm_manager = PackageManager::new(PackageManagerKind::Npm, "/project");
        assert!(npm_manager.supports_workspaces());

        let jsr_manager = PackageManager::new(PackageManagerKind::Jsr, "/project");
        assert!(!jsr_manager.supports_workspaces());
    }

    #[test]
    fn test_package_manager_workspace_config_path() {
        let npm_manager = PackageManager::new(PackageManagerKind::Npm, "/project");
        assert_eq!(npm_manager.workspace_config_path(), None);

        let pnpm_manager = PackageManager::new(PackageManagerKind::Pnpm, "/project");
        let expected = Some(PathBuf::from("/project/pnpm-workspace.yaml"));
        assert_eq!(pnpm_manager.workspace_config_path(), expected);
    }

    #[test]
    fn test_package_manager_debug() {
        let manager = PackageManager::new(PackageManagerKind::Yarn, "/project");
        let debug_str = format!("{manager:?}");
        assert!(debug_str.contains("PackageManager"));
        assert!(debug_str.contains("Yarn"));
    }

    #[test]
    fn test_package_manager_clone() {
        let original = PackageManager::new(PackageManagerKind::Bun, "/original/path");
        let cloned = original.clone();

        assert_eq!(original.kind(), cloned.kind());
        assert_eq!(original.root(), cloned.root());
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod repository_info_tests {
    use super::*;

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

    #[test]
    fn test_repository_info_simple() {
        let repo = MockRepository::new("/simple/project", RepoKind::Simple);

        assert_eq!(repo.root(), Path::new("/simple/project"));
        assert_eq!(repo.kind(), RepoKind::Simple);
        assert!(!repo.is_monorepo());
        assert!(!repo.has_package_manager());
        assert_eq!(repo.display_name(), "simple");
    }

    #[test]
    fn test_repository_info_monorepo() {
        let repo =
            MockRepository::new("/yarn/monorepo", RepoKind::Monorepo(MonorepoKind::YarnWorkspaces));

        assert_eq!(repo.root(), Path::new("/yarn/monorepo"));
        assert_eq!(repo.kind(), RepoKind::Monorepo(MonorepoKind::YarnWorkspaces));
        assert!(repo.is_monorepo());
        assert_eq!(repo.display_name(), "yarn monorepo");
    }

    #[test]
    fn test_repository_info_with_package_manager() {
        let manager = PackageManager::new(PackageManagerKind::Npm, "/project");
        let repo = MockRepository::new("/project", RepoKind::Simple).with_package_manager(manager);

        assert!(repo.has_package_manager());
        assert_eq!(repo.package_manager().unwrap().kind(), PackageManagerKind::Npm);
    }

    #[test]
    fn test_repository_info_trait_object() {
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

    #[test]
    fn test_repository_info_send_sync() {
        // This test ensures that RepositoryInfo is Send + Sync
        fn require_send_sync<T: Send + Sync>() {}
        require_send_sync::<MockRepository>();
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_repo_kind_with_package_manager_integration() {
        let repo_kind = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);
        let package_manager = PackageManager::new(PackageManagerKind::Yarn, "/project");

        // These should be compatible
        assert!(repo_kind.is_monorepo());
        assert!(package_manager.supports_workspaces());
        assert_eq!(package_manager.command(), "yarn");
    }

    #[test]
    fn test_pnpm_specific_integration() {
        let repo_kind = RepoKind::Monorepo(MonorepoKind::PnpmWorkspaces);
        let package_manager = PackageManager::new(PackageManagerKind::Pnpm, "/pnpm-project");

        assert!(repo_kind.is_monorepo_kind(&MonorepoKind::PnpmWorkspaces));
        assert_eq!(package_manager.lock_file(), "pnpm-lock.yaml");
        assert_eq!(
            package_manager.workspace_config_path(),
            Some(PathBuf::from("/pnpm-project/pnpm-workspace.yaml"))
        );
    }

    #[test]
    fn test_simple_repo_with_npm() {
        let repo_kind = RepoKind::Simple;
        let package_manager = PackageManager::new(PackageManagerKind::Npm, "/simple-project");

        assert!(!repo_kind.is_monorepo());
        assert_eq!(
            package_manager.lock_file_path(),
            PathBuf::from("/simple-project/package-lock.json")
        );
    }

    #[test]
    fn test_all_monorepo_kinds_have_workspace_support() {
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

    #[test]
    fn test_package_manager_kinds_coverage() {
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
}
