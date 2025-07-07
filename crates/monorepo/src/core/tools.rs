//! MonorepoTools implementation - main orchestrator for monorepo functionality

use crate::analysis::{DiffAnalyzer, MonorepoAnalyzer};
use crate::config::types::HookStrategy;
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

    /// Install git hooks using intelligent strategy detection
    ///
    /// Automatically detects the best hook strategy (native Git, Husky, or hybrid)
    /// and installs appropriate hooks. This method provides seamless integration
    /// with existing developer workflows while supporting modern tooling.
    ///
    /// # Hook Strategies
    ///
    /// - **Auto**: Detects project setup and chooses optimal strategy
    /// - **Native**: Uses traditional Git hooks in `.git/hooks/`
    /// - **Husky**: Creates Husky-compatible hooks in `.husky/`
    /// - **Hybrid**: Uses both systems for maximum compatibility
    ///
    /// # Returns
    ///
    /// A vector of hook names that were successfully installed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The Git repository cannot be accessed
    /// - Hook directories cannot be created
    /// - Hook files cannot be written
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
        // Get the effective hook strategy
        let strategy = self.get_effective_hook_strategy();
        
        log::info!("Installing hooks using strategy: {:?}", strategy);
        
        match strategy {
            HookStrategy::Auto => {
                // This should not happen as get_effective_hook_strategy resolves Auto
                log::warn!("Auto strategy not resolved, falling back to detection");
                self.install_git_hooks_with_strategy(self.detect_hook_strategy())
            }
            HookStrategy::Native => self.install_native_git_hooks(),
            HookStrategy::Husky => self.install_husky_hooks(),
            HookStrategy::Hybrid => {
                // Install both systems
                let mut all_hooks = Vec::new();
                
                match self.install_native_git_hooks() {
                    Ok(mut native_hooks) => {
                        all_hooks.append(&mut native_hooks);
                    }
                    Err(e) => log::warn!("Failed to install native hooks in hybrid mode: {}", e),
                }
                
                match self.install_husky_hooks() {
                    Ok(mut husky_hooks) => {
                        // Prefix Husky hooks to distinguish them
                        for hook in &mut husky_hooks {
                            *hook = format!("husky-{}", hook);
                        }
                        all_hooks.append(&mut husky_hooks);
                    }
                    Err(e) => log::warn!("Failed to install Husky hooks in hybrid mode: {}", e),
                }
                
                if all_hooks.is_empty() {
                    return Err(crate::error::Error::git(
                        "Failed to install any hooks in hybrid mode".to_string()
                    ));
                }
                
                Ok(all_hooks)
            }
        }
    }

    /// Install hooks using a specific strategy (internal helper)
    fn install_git_hooks_with_strategy(&self, strategy: HookStrategy) -> Result<Vec<String>> {
        match strategy {
            HookStrategy::Auto => unreachable!("Auto strategy should be resolved before this point"),
            HookStrategy::Native => self.install_native_git_hooks(),
            HookStrategy::Husky => self.install_husky_hooks(),
            HookStrategy::Hybrid => {
                // Recursive call for hybrid logic - this could be improved
                self.install_git_hooks()
            }
        }
    }

    /// Install native Git hooks in .git/hooks/
    fn install_native_git_hooks(&self) -> Result<Vec<String>> {
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

    /// Generate Husky-compatible pre-commit hook script
    ///
    /// Creates a Husky-style hook script that integrates with the Node.js ecosystem
    /// and uses package manager commands for better integration with monorepo tools.
    fn generate_husky_pre_commit_script(&self) -> String {
        let config = &self.project.config;
        let tasks = &config.hooks.pre_commit.run_tasks;
        let husky_config = &config.hooks.husky;
        
        let mut script = String::from("#!/usr/bin/env sh\n");
        script.push_str(". \"$(dirname -- \"$0\")/_/husky.sh\"\n\n");
        script.push_str("# Generated by sublime-monorepo-tools\n");
        script.push_str("# Husky pre-commit hook\n\n");
        
        script.push_str("set -e\n\n");
        
        script.push_str("echo \"🚀 Running pre-commit validation...\"\n\n");
        
        if config.hooks.pre_commit.validate_changeset {
            script.push_str("# Check for changesets if required\n");
            script.push_str("echo \"📋 Checking for changesets...\"\n");
            script.push_str("# TODO: Add changeset validation when CLI is available\n\n");
        }
        
        if !tasks.is_empty() {
            script.push_str("# Run configured tasks on affected packages\n");
            
            let package_manager = self.detect_package_manager();
            
            for task in tasks {
                script.push_str(&format!("echo \"🔧 Running task: {task}...\"\n"));
                
                if husky_config.use_package_scripts {
                    // Use package manager scripts for better integration
                    match package_manager.as_str() {
                        "yarn" => script.push_str(&format!("yarn {task}\n")),
                        "pnpm" => script.push_str(&format!("pnpm run {task}\n")),
                        "bun" => script.push_str(&format!("bun run {task}\n")),
                        _ => script.push_str(&format!("npm run {task}\n")),
                    }
                } else {
                    // Direct command execution
                    script.push_str(&format!("{task}\n"));
                }
            }
            script.push('\n');
        }
        
        script.push_str("echo \"✅ Pre-commit validation completed successfully!\"\n");
        
        script
    }

    /// Generate Husky-compatible pre-push hook script
    ///
    /// Creates a Husky-style pre-push hook that can handle affected packages
    /// and integrates with monorepo workflows.
    fn generate_husky_pre_push_script(&self) -> String {
        let config = &self.project.config;
        let tasks = &config.hooks.pre_push.run_tasks;
        let husky_config = &config.hooks.husky;
        
        let mut script = String::from("#!/usr/bin/env sh\n");
        script.push_str(". \"$(dirname -- \"$0\")/_/husky.sh\"\n\n");
        script.push_str("# Generated by sublime-monorepo-tools\n");
        script.push_str("# Husky pre-push hook\n\n");
        
        script.push_str("set -e\n\n");
        
        script.push_str("echo \"🚀 Running pre-push validation...\"\n\n");
        
        if !tasks.is_empty() {
            script.push_str("# Run configured tasks on affected packages\n");
            
            let package_manager = self.detect_package_manager();
            
            for task in tasks {
                script.push_str(&format!("echo \"🔧 Running task: {task}...\"\n"));
                
                if husky_config.use_package_scripts {
                    // Use package manager scripts for better integration
                    match package_manager.as_str() {
                        "yarn" => script.push_str(&format!("yarn {task}\n")),
                        "pnpm" => script.push_str(&format!("pnpm run {task}\n")),
                        "bun" => script.push_str(&format!("bun run {task}\n")),
                        _ => script.push_str(&format!("npm run {task}\n")),
                    }
                } else {
                    script.push_str(&format!("{task}\n"));
                }
            }
            script.push('\n');
        }
        
        script.push_str("echo \"✅ Pre-push validation completed successfully!\"\n");
        
        script
    }

    /// Detect the package manager used in this project
    ///
    /// Analyzes lock files and configuration to determine the preferred package manager.
    fn detect_package_manager(&self) -> String {
        let root = &self.project.root_path;
        
        // Check for explicit configuration first
        if let Some(ref pm) = self.project.config.hooks.husky.package_manager {
            return pm.clone();
        }
        
        // Auto-detect based on lock files
        if root.join("bun.lockb").exists() {
            "bun".to_string()
        } else if root.join("pnpm-lock.yaml").exists() {
            "pnpm".to_string()
        } else if root.join("yarn.lock").exists() {
            "yarn".to_string()
        } else {
            "npm".to_string()
        }
    }

    /// Install Husky hooks in the project
    ///
    /// Creates Husky-compatible hook files in the .husky directory with proper
    /// structure and integration with the Node.js ecosystem.
    ///
    /// # Returns
    ///
    /// A vector of hook names that were successfully installed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The .husky directory cannot be created
    /// - Hook files cannot be written
    /// - File permissions cannot be set
    pub fn install_husky_hooks(&self) -> Result<Vec<String>> {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let config = &self.project.config.hooks;
        let husky_dir = self.project.root_path.join(&config.husky.husky_dir);
        
        // Ensure .husky directory exists
        fs::create_dir_all(&husky_dir)
            .map_err(|e| crate::error::Error::git(format!("Failed to create .husky directory: {e}")))?;

        let mut installed_hooks = Vec::new();

        // Install pre-commit hook if enabled
        if config.enabled && config.pre_commit.enabled {
            let pre_commit_content = self.generate_husky_pre_commit_script();
            let pre_commit_path = husky_dir.join("pre-commit");
            
            fs::write(&pre_commit_path, pre_commit_content)
                .map_err(|e| crate::error::Error::git(format!("Failed to write Husky pre-commit hook: {e}")))?;
            
            // Make executable
            let mut perms = fs::metadata(&pre_commit_path)
                .map_err(|e| crate::error::Error::git(format!("Failed to get Husky hook permissions: {e}")))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&pre_commit_path, perms)
                .map_err(|e| crate::error::Error::git(format!("Failed to set Husky hook permissions: {e}")))?;
            
            installed_hooks.push("pre-commit".to_string());
            log::info!("Installed Husky pre-commit hook");
        }

        // Install pre-push hook if enabled
        if config.enabled && config.pre_push.enabled {
            let pre_push_content = self.generate_husky_pre_push_script();
            let pre_push_path = husky_dir.join("pre-push");
            
            fs::write(&pre_push_path, pre_push_content)
                .map_err(|e| crate::error::Error::git(format!("Failed to write Husky pre-push hook: {e}")))?;
            
            // Make executable
            let mut perms = fs::metadata(&pre_push_path)
                .map_err(|e| crate::error::Error::git(format!("Failed to get Husky hook permissions: {e}")))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&pre_push_path, perms)
                .map_err(|e| crate::error::Error::git(format!("Failed to set Husky hook permissions: {e}")))?;
            
            installed_hooks.push("pre-push".to_string());
            log::info!("Installed Husky pre-push hook");
        }

        Ok(installed_hooks)
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

    /// Detect the optimal hook strategy for this project
    ///
    /// Analyzes the project structure to determine whether to use native Git hooks,
    /// Husky, or a hybrid approach. This method implements intelligent detection
    /// based on project characteristics and existing tooling.
    ///
    /// # Returns
    ///
    /// The recommended `HookStrategy` for this project
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::MonorepoTools;
    ///
    /// let tools = MonorepoTools::initialize("/path/to/monorepo")?;
    /// let strategy = tools.detect_hook_strategy();
    /// println!("Recommended strategy: {:?}", strategy);
    /// ```
    #[must_use]
    pub fn detect_hook_strategy(&self) -> HookStrategy {
        let config = &self.project.config.hooks;
        
        // If auto-detection is disabled, use configured strategy
        if !config.auto_detection.enabled {
            return config.strategy;
        }

        let detection_result = self.analyze_project_for_hooks();
        
        match (detection_result.has_husky, detection_result.has_nodejs, detection_result.has_git_hooks) {
            // Husky is already installed and this is a Node.js project
            (true, true, _) => HookStrategy::Husky,
            
            // Node.js project without Husky but auto-detection prefers Husky
            (false, true, false) if config.auto_detection.prefer_husky => HookStrategy::Husky,
            
            // Has existing Git hooks, prefer to keep them
            (false, _, true) => HookStrategy::Native,
            
            // Both systems exist, use hybrid approach
            (true, _, true) => HookStrategy::Hybrid,
            
            // Default to native Git hooks for non-Node.js projects
            (false, false, _) => HookStrategy::Native,
            
            // Default case: use native hooks
            _ => HookStrategy::Native,
        }
    }

    /// Check if Husky is installed and configured in this project
    ///
    /// Analyzes the project to determine if Husky is present by checking
    /// package.json dependencies, .husky directory, and Husky configuration.
    ///
    /// # Returns
    ///
    /// `true` if Husky is detected in the project
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::MonorepoTools;
    ///
    /// let tools = MonorepoTools::initialize("/path/to/monorepo")?;
    /// if tools.has_husky() {
    ///     println!("Husky is configured in this project");
    /// }
    /// ```
    #[must_use]
    pub fn has_husky(&self) -> bool {
        self.analyze_project_for_hooks().has_husky
    }

    /// Check if this appears to be a Node.js project
    ///
    /// Analyzes the project for Node.js indicators such as package.json,
    /// lock files, and node_modules directory.
    ///
    /// # Returns
    ///
    /// `true` if Node.js indicators are found
    #[must_use]
    pub fn is_nodejs_project(&self) -> bool {
        self.analyze_project_for_hooks().has_nodejs
    }

    /// Get the effective hook strategy for this project
    ///
    /// Returns the hook strategy that will actually be used, taking into account
    /// the configured strategy and auto-detection results.
    ///
    /// # Returns
    ///
    /// The `HookStrategy` that will be used for hook installation
    #[must_use]
    pub fn get_effective_hook_strategy(&self) -> HookStrategy {
        match self.project.config.hooks.strategy {
            HookStrategy::Auto => self.detect_hook_strategy(),
            strategy => strategy,
        }
    }

    /// Internal helper to analyze project structure for hook strategy detection
    fn analyze_project_for_hooks(&self) -> HookDetectionResult {
        let root = &self.project.root_path;
        let config = &self.project.config.hooks.auto_detection;
        
        let mut result = HookDetectionResult::default();

        // Check for Node.js indicators
        if config.check_package_json {
            result.has_nodejs = config.nodejs_indicators.iter().any(|indicator| {
                root.join(indicator).exists()
            });
        }

        // Check for Husky directory
        if config.check_husky_dir {
            let husky_dir = root.join(&self.project.config.hooks.husky.husky_dir);
            result.has_husky = husky_dir.exists() && husky_dir.is_dir();
        }

        // Check package.json for Husky configuration
        if config.check_package_json && !result.has_husky {
            result.has_husky = self.check_package_json_for_husky();
        }

        // Check for existing Git hooks
        if config.check_git_hooks {
            let git_hooks_dir = root.join(".git").join("hooks");
            result.has_git_hooks = ["pre-commit", "pre-push", "post-merge"]
                .iter()
                .any(|hook| git_hooks_dir.join(hook).exists());
        }

        result
    }

    /// Check package.json for Husky configuration
    fn check_package_json_for_husky(&self) -> bool {
        let package_json_path = self.project.root_path.join("package.json");
        
        if !package_json_path.exists() {
            return false;
        }

        // Read and parse package.json
        match std::fs::read_to_string(&package_json_path) {
            Ok(content) => {
                // Simple string checks for Husky indicators
                content.contains("\"husky\"") || 
                content.contains("husky install") ||
                content.contains(".husky")
            }
            Err(_) => false,
        }
    }
}

/// Result of hook detection analysis
#[derive(Debug, Default)]
struct HookDetectionResult {
    /// Whether Husky is detected in the project
    has_husky: bool,
    /// Whether this appears to be a Node.js project
    has_nodejs: bool,
    /// Whether existing Git hooks are present
    has_git_hooks: bool,
}
