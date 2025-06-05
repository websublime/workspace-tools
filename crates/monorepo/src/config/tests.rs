//! Unit tests for configuration module

#[cfg(test)]
mod tests {
    use crate::config::*;

    #[test]
    fn test_default_config_values() {
        let config = MonorepoConfig::default();
        
        assert_eq!(config.versioning.default_bump, VersionBumpType::Patch);
        assert!(config.versioning.propagate_changes);
        assert_eq!(config.versioning.tag_prefix, "v");
        assert!(config.versioning.auto_tag);
        
        assert_eq!(config.tasks.default_tasks, vec!["test", "lint"]);
        assert!(config.tasks.parallel);
        assert_eq!(config.tasks.max_concurrent, 4);
        
        assert!(config.hooks.enabled);
        assert!(config.hooks.pre_commit.enabled);
        assert!(config.hooks.pre_push.enabled);
        
        assert!(config.changesets.required);
        assert_eq!(config.changesets.changeset_dir.to_str().unwrap(), ".changesets");
        
        assert_eq!(config.environments.len(), 3);
        assert!(config.environments.contains(&Environment::Development));
        assert!(config.environments.contains(&Environment::Staging));
        assert!(config.environments.contains(&Environment::Production));
    }

    #[test]
    fn test_config_presets() {
        let small = MonorepoConfig::small_project();
        assert!(!small.tasks.parallel);
        assert_eq!(small.tasks.max_concurrent, 2);
        assert!(!small.versioning.propagate_changes);
        
        let large = MonorepoConfig::large_project();
        assert!(large.tasks.parallel);
        assert_eq!(large.tasks.max_concurrent, 8);
        assert_eq!(large.tasks.timeout, 600);
        assert!(large.versioning.propagate_changes);
        assert!(large.changesets.required);
        
        let library = MonorepoConfig::library_project();
        assert_eq!(library.versioning.default_bump, VersionBumpType::Minor);
        assert!(library.changelog.include_breaking_changes);
    }

    #[test]
    fn test_config_manager_creation() {
        let config_manager = ConfigManager::new();
        
        // Test that manager is created with default config
        assert_eq!(config_manager.get().expect("Config should be available").versioning.default_bump, VersionBumpType::Patch);
    }

    #[test]
    fn test_config_update() {
        let config_manager = ConfigManager::new();
        let mut new_config = MonorepoConfig::default();
        new_config.versioning.default_bump = VersionBumpType::Major;
        
        config_manager.update(|config| {
            config.versioning.default_bump = new_config.versioning.default_bump;
        }).expect("Config update should succeed");
        assert_eq!(config_manager.get().expect("Config should be available").versioning.default_bump, VersionBumpType::Major);
    }

    #[test]
    fn test_version_bump_type_variants() {
        let patch = VersionBumpType::Patch;
        let minor = VersionBumpType::Minor;
        let major = VersionBumpType::Major;
        
        assert_eq!(patch, VersionBumpType::Patch);
        assert_ne!(patch, minor);
        assert_ne!(minor, major);
    }

    #[test]
    fn test_environment_variants() {
        let dev = Environment::Development;
        let staging = Environment::Staging;
        let prod = Environment::Production;
        let integration = Environment::Integration;
        
        assert_eq!(dev, Environment::Development);
        assert_ne!(dev, prod);
        assert_ne!(staging, integration);
    }
}