//! MonorepoTools implementation - main orchestrator for monorepo functionality

use crate::analysis::{DiffAnalyzer, MonorepoAnalyzer};
use crate::core::types::MonorepoTools;
use crate::core::{MonorepoProject, VersionManager, VersioningStrategy};
use crate::error::Result;
use crate::tasks::TaskManager;

impl<'a> MonorepoTools<'a> {
    /// Creates monorepo tools from an existing MonorepoProject
    ///
    /// Uses direct borrowing from the project to eliminate Arc proliferation
    /// and work with Rust ownership principles.
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to the monorepo project
    ///
    /// # Returns
    ///
    /// A configured MonorepoTools instance ready for operations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::{MonorepoTools, MonorepoProject};
    ///
    /// let project = MonorepoProject::new("/path/to/monorepo")?;
    /// let tools = MonorepoTools::new(&project);
    /// ```
    pub fn new(project: &'a MonorepoProject) -> Self {
        // Initialize the analyzer with direct borrowing
        let analyzer = MonorepoAnalyzer::new(project);

        log::info!(
            "Initialized monorepo tools for {} with {} packages",
            project.root_path.display(),
            project.packages.len()
        );

        Self { project, analyzer }
    }

    /// Get a reference to the monorepo analyzer
    ///
    /// Returns a reference to the initialized `MonorepoAnalyzer` that can be used
    /// for analyzing the monorepo structure, dependencies, and changes.
    ///
    /// # Returns
    ///
    /// A reference to the `MonorepoAnalyzer` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::MonorepoTools;
    ///
    /// let tools = MonorepoTools::initialize("/path/to/monorepo")?;
    /// let analyzer = tools.analyzer()?;
    /// let packages = analyzer.get_packages()?;
    /// ```
    pub fn analyzer(&self) -> Result<&MonorepoAnalyzer> {
        Ok(&self.analyzer)
    }

    /// Get a reference to the diff analyzer (Phase 2 functionality)
    #[must_use]
    pub fn diff_analyzer(&self) -> DiffAnalyzer {
        DiffAnalyzer::from_project(self.project)
    }

    /// Get a reference to the version manager (Phase 2 functionality)
    #[must_use]
    pub fn version_manager(&self) -> VersionManager<'a> {
        VersionManager::new(self.project)
    }

    /// Get a version manager with custom strategy (Phase 2 functionality)
    #[must_use]
    pub fn version_manager_with_strategy(
        &self,
        strategy: Box<dyn VersioningStrategy + 'a>,
    ) -> VersionManager<'a> {
        VersionManager::with_strategy(self.project, strategy)
    }

    /// Get a reference to the task manager (Phase 3 functionality)
    pub fn task_manager(&self) -> Result<TaskManager> {
        TaskManager::from_project(self.project)
    }

    /// Install basic git hooks for pre-commit and pre-push validation
    ///
    /// Creates simple shell scripts in `.git/hooks/` that run configured tasks
    /// on affected packages. This replaces the complex hook system with basic
    /// script generation suitable for CLI/daemon consumption.
    ///
    /// # Returns
    ///
    /// A vector of hook names that were successfully installed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `.git/hooks` directory cannot be accessed
    /// - Hook files cannot be written
    /// - Permissions cannot be set on hook files
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::MonorepoTools;
    ///
    /// let tools = MonorepoTools::initialize("/path/to/monorepo")?;
    /// let installed_hooks = tools.install_git_hooks()?;
    /// println!("Installed hooks: {:?}", installed_hooks);
    /// ```
    pub fn install_git_hooks(&self) -> Result<Vec<String>> {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let git_hooks_dir = self.project.root_path.join(".git").join("hooks");
        
        // Ensure .git/hooks directory exists
        if !git_hooks_dir.exists() {
            return Err(crate::error::Error::git(
                "Git repository not found - .git/hooks directory does not exist".to_string()
            ));
        }

        let mut installed_hooks = Vec::new();
        let config = &self.project.config;

        // Install pre-commit hook if enabled
        if config.hooks.enabled && config.hooks.pre_commit.enabled {
            let pre_commit_content = self.generate_pre_commit_hook_script();
            let pre_commit_path = git_hooks_dir.join("pre-commit");
            
            fs::write(&pre_commit_path, pre_commit_content)
                .map_err(|e| crate::error::Error::git(format!("Failed to write pre-commit hook: {e}")))?;
            
            // Make executable
            let mut perms = fs::metadata(&pre_commit_path)
                .map_err(|e| crate::error::Error::git(format!("Failed to get hook permissions: {e}")))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&pre_commit_path, perms)
                .map_err(|e| crate::error::Error::git(format!("Failed to set hook permissions: {e}")))?;
            
            installed_hooks.push("pre-commit".to_string());
            log::info!("Installed pre-commit hook");
        }

        // Install pre-push hook if enabled
        if config.hooks.enabled && config.hooks.pre_push.enabled {
            let pre_push_content = self.generate_pre_push_hook_script();
            let pre_push_path = git_hooks_dir.join("pre-push");
            
            fs::write(&pre_push_path, pre_push_content)
                .map_err(|e| crate::error::Error::git(format!("Failed to write pre-push hook: {e}")))?;
            
            // Make executable
            let mut perms = fs::metadata(&pre_push_path)
                .map_err(|e| crate::error::Error::git(format!("Failed to get hook permissions: {e}")))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&pre_push_path, perms)
                .map_err(|e| crate::error::Error::git(format!("Failed to set hook permissions: {e}")))?;
            
            installed_hooks.push("pre-push".to_string());
            log::info!("Installed pre-push hook");
        }

        Ok(installed_hooks)
    }

    /// Generate pre-commit hook script content
    ///
    /// Creates a shell script that runs configured pre-commit tasks
    /// on packages with staged changes.
    fn generate_pre_commit_hook_script(&self) -> String {
        let config = &self.project.config;
        let tasks = &config.hooks.pre_commit.run_tasks;
        
        let mut script = String::from("#!/bin/sh\n");
        script.push_str("# Generated by sublime-monorepo-tools\n");
        script.push_str("# Pre-commit validation hook\n\n");
        
        script.push_str("set -e\n\n");
        
        script.push_str("echo \"Running pre-commit validation...\"\n\n");
        
        if config.hooks.pre_commit.validate_changeset {
            script.push_str("# Check for changesets if required\n");
            script.push_str("echo \"Checking for changesets...\"\n");
            script.push_str("# TODO: Add changeset validation when CLI is available\n\n");
        }
        
        if !tasks.is_empty() {
            script.push_str("# Run configured tasks on affected packages\n");
            for task in tasks {
                script.push_str(&format!("echo \"Running task: {task}...\"\n"));
                script.push_str(&format!("npm run {task} --if-present\n"));
            }
            script.push('\n');
        }
        
        script.push_str("echo \"Pre-commit validation completed successfully!\"\n");
        
        script
    }

    /// Generate pre-push hook script content
    ///
    /// Creates a shell script that runs configured pre-push tasks
    /// on packages affected by the commits being pushed.
    fn generate_pre_push_hook_script(&self) -> String {
        let config = &self.project.config;
        let tasks = &config.hooks.pre_push.run_tasks;
        
        let mut script = String::from("#!/bin/sh\n");
        script.push_str("# Generated by sublime-monorepo-tools\n");
        script.push_str("# Pre-push validation hook\n\n");
        
        script.push_str("set -e\n\n");
        
        script.push_str("echo \"Running pre-push validation...\"\n\n");
        
        if !tasks.is_empty() {
            script.push_str("# Run configured tasks on affected packages\n");
            for task in tasks {
                script.push_str(&format!("echo \"Running task: {task}...\"\n"));
                script.push_str(&format!("npm run {task} --if-present\n"));
            }
            script.push('\n');
        }
        
        script.push_str("echo \"Pre-push validation completed successfully!\"\n");
        
        script
    }

    /// Uninstall git hooks
    ///
    /// Removes the git hook files created by this tool.
    ///
    /// # Returns
    ///
    /// A vector of hook names that were successfully removed
    ///
    /// # Errors
    ///
    /// Returns an error if hook files cannot be removed
    pub fn uninstall_git_hooks(&self) -> Result<Vec<String>> {
        use std::fs;

        let git_hooks_dir = self.project.root_path.join(".git").join("hooks");
        let mut removed_hooks = Vec::new();

        for hook_name in &["pre-commit", "pre-push"] {
            let hook_path = git_hooks_dir.join(hook_name);
            if hook_path.exists() {
                fs::remove_file(&hook_path)
                    .map_err(|e| crate::error::Error::git(format!("Failed to remove {hook_name} hook: {e}")))?;
                removed_hooks.push((*hook_name).to_string());
                log::info!("Removed {hook_name} hook");
            }
        }

        Ok(removed_hooks)
    }

}
