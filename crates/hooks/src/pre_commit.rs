use sublime_monorepo_tools::{Workspace, WorkspaceConfig, DiscoveryOptions};
use sublime_git_tools::GitChangedFile;
use std::path::Path;

use crate::{hook::HookContext, HookResult};

pub struct PreCommitHook<'a> {
    ctx: &'a HookContext,
}

impl<'a> PreCommitHook<'a> {
    pub fn new(ctx: &'a HookContext) -> Self {
        Self { ctx }
    }

    /// Runs the pre-commit hook
    pub fn run(&self) -> HookResult<Vec<String>> {
        let changed_files = self.ctx.repo.get_staged_files()?;
        let mut workspace = Workspace::new(
            self.ctx.repo.get_repo_path().to_path_buf(),
            WorkspaceConfig::default(),
            Some(self.ctx.repo.clone()),
        )?;
        workspace.discover_packages_with_options(&DiscoveryOptions::default())?;
        let changes_by_package = self.group_changes_by_package(&changed_files)?;
        let package_manager = self.ctx.package_manager();
        
        let mut decisions = Vec::new();
        
        for (package, _changes) in changes_by_package {
            if package_manager.has_affected_dependents(&package)? {
                decisions.push(package);
            }
        }
        
        Ok(decisions)
    }

    /// Groups changed files by package
    fn group_changes_by_package(&self, files: &[GitChangedFile]) -> HookResult<Vec<(String, Vec<GitChangedFile>)>> {
        let mut changes_by_package: Vec<(String, Vec<GitChangedFile>)> = Vec::new();
        for file in files {
            if self.ctx.config.ignore_paths.iter().any(|p| file.path.starts_with(p)) {
                continue;
            }

            if let Some(package) = self.get_package_for_file(&file.path)? {
                if let Some(entry) = changes_by_package.iter_mut().find(|(p, _)| p == &package) {
                    entry.1.push(file.clone());
                } else {
                    changes_by_package.push((package, vec![file.clone()]));
                }
            }
        }
        Ok(changes_by_package)
    }

    fn get_package_for_file(&self, file: &str) -> HookResult<Option<String>> {
        let mut workspace = Workspace::new(
            self.ctx.repo.get_repo_path().to_path_buf(),
            WorkspaceConfig::default(),
            Some(self.ctx.repo.clone()),
        )?;
        workspace.discover_packages_with_options(&DiscoveryOptions::default())?;
        let file_path = Path::new(file);
        for package in workspace.sorted_packages() {
            let package_path = package.borrow().package_path.clone();
            if file_path.starts_with(package_path) {
                return Ok(Some(package.borrow().package.borrow().name().to_string()));
            }
        }
        Ok(None)
    }
} 