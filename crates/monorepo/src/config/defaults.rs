//! Default configurations for common monorepo setups

use crate::{Environment, MonorepoConfig, VersionBumpType};

impl MonorepoConfig {
    /// Create a configuration optimized for small projects
    #[must_use]
    pub fn small_project() -> Self {
        let mut config = Self::default();
        config.tasks.parallel = false;
        config.tasks.max_concurrent = 2;
        config.versioning.propagate_changes = false;
        config
    }

    /// Create a configuration optimized for large monorepos
    #[must_use]
    pub fn large_project() -> Self {
        let mut config = Self::default();
        config.tasks.parallel = true;
        config.tasks.performance.large_project.max_concurrent = 8;
        config.tasks.performance.large_project.timeout = 600; // 10 minutes
        config.versioning.propagate_changes = true;
        config.changesets.required = true;
        config
    }

    /// Create a configuration for library projects
    #[must_use]
    pub fn library_project() -> Self {
        let mut config = Self::default();
        config.versioning.default_bump = VersionBumpType::Minor;
        config.changelog.include_breaking_changes = true;
        config.hooks.pre_push.run_tasks =
            vec!["test".to_string(), "build".to_string(), "docs".to_string()];
        config
    }

    /// Create a configuration for application projects
    #[must_use]
    pub fn application_project() -> Self {
        let mut config = Self::default();
        config.versioning.snapshot_format = "{version}-{branch}.{sha}".to_string();
        config.environments = vec![
            Environment::Development,
            Environment::Staging,
            Environment::Integration,
            Environment::Production,
        ];
        config.changesets.auto_deploy = true;
        config
    }

    /// Create task groups for common workflows using configurable defaults
    #[must_use]
    pub fn with_common_task_groups(mut self) -> Self {
        // Use task groups from workspace tool configuration
        let tool_config = &self.workspace.tool_configs;
        for (group_name, commands) in &tool_config.default_task_groups {
            self.tasks.groups.insert(group_name.clone(), commands.clone());
        }

        self
    }

    /// Add conventional commit types for a specific domain
    #[must_use]
    pub fn with_domain_commit_types(mut self, domain: &str) -> Self {
        match domain {
            "backend" => {
                self.changelog
                    .conventional_commit_types
                    .insert("api".to_string(), "API Changes".to_string());
                self.changelog
                    .conventional_commit_types
                    .insert("db".to_string(), "Database Changes".to_string());
            }
            "frontend" => {
                self.changelog
                    .conventional_commit_types
                    .insert("ui".to_string(), "UI Changes".to_string());
                self.changelog
                    .conventional_commit_types
                    .insert("a11y".to_string(), "Accessibility".to_string());
                self.changelog
                    .conventional_commit_types
                    .insert("i18n".to_string(), "Internationalization".to_string());
            }
            "mobile" => {
                self.changelog
                    .conventional_commit_types
                    .insert("ios".to_string(), "iOS Specific".to_string());
                self.changelog
                    .conventional_commit_types
                    .insert("android".to_string(), "Android Specific".to_string());
            }
            _ => {}
        }
        self
    }
}
