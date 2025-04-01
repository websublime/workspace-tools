use sublime_git_tools::Repo;
use sublime_monorepo_tools::ChangeStore;
use sublime_package_tools::Version;

use crate::{package::PackageManager, HookResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDecision {
    /// Package name
    pub package: String,
    /// Current version
    pub current_version: String,
    /// Decided version bump type
    pub bump_type: Option<Version>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// Paths to ignore
    pub ignore_paths: Vec<String>,
    /// Protected branch patterns
    pub protected_branches: Vec<String>,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            ignore_paths: Vec::new(),
            protected_branches: vec!["main".to_string(), "master".to_string()],
        }
    }
}

pub struct HookContext {
    /// Git repository
    pub repo: Repo,
    /// Hook configuration
    pub config: HookConfig,
    /// Change store
    pub change_store: Box<dyn ChangeStore>,
    /// Package manager
    package_manager: PackageManager,
}

impl HookContext {
    /// Creates a new hook context
    pub fn new(
        repo: Repo,
        config: HookConfig,
        change_store: Box<dyn ChangeStore>,
    ) -> HookResult<Self> {
        let repo_path = repo.get_repo_path();
        Ok(Self {
            package_manager: PackageManager::new(&repo_path)?,
            repo,
            config,
            change_store,
        })
    }

    /// Gets the package manager instance
    pub fn package_manager(&self) -> &PackageManager {
        &self.package_manager
    }

    /// Checks if current branch is protected
    pub fn is_protected_branch(&self) -> HookResult<bool> {
        let current_branch = self.repo.get_current_branch()?;
        Ok(self.config.protected_branches.iter().any(|pattern| {
            if let Ok(glob) = glob::Pattern::new(pattern) {
                glob.matches(&current_branch)
            } else {
                false
            }
        }))
    }
} 