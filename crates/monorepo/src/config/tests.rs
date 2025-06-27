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
        assert_eq!(config_manager.get_clone().versioning.default_bump, VersionBumpType::Patch);
    }

    #[test]
    fn test_config_update() {
        let mut config_manager = ConfigManager::new();
        let mut new_config = MonorepoConfig::default();
        new_config.versioning.default_bump = VersionBumpType::Major;
        
        config_manager.update(|config| {
            config.versioning.default_bump = new_config.versioning.default_bump;
        }).expect("Config update should succeed");
        assert_eq!(config_manager.get_clone().versioning.default_bump, VersionBumpType::Major);
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

    // ConfigManager tests moved from manager.rs
    
    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_config_manager_default_config() {
        let manager = ConfigManager::new();
        let config = manager.get_clone();

        assert_eq!(config.versioning.default_bump, VersionBumpType::Patch);
        assert!(config.versioning.propagate_changes);
        assert!(config.hooks.enabled);
        assert_eq!(config.environments.len(), 3);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_config_manager_update_config() {
        let mut manager = ConfigManager::new();

        manager
            .update(|config| {
                config.versioning.default_bump = VersionBumpType::Minor;
                config.versioning.auto_tag = false;
            })
            .unwrap();

        let config = manager.get_clone();
        assert_eq!(config.versioning.default_bump, VersionBumpType::Minor);
        assert!(!config.versioning.auto_tag);
    }

    // Pattern matching tests moved from manager.rs

    #[test]
    fn test_pattern_matches_package_exact() {
        let manager = ConfigManager::new();
        
        // Exact matches
        assert!(manager.pattern_matches_package("packages/core", "packages/core"));
        assert!(!manager.pattern_matches_package("packages/core", "packages/ui"));
        assert!(!manager.pattern_matches_package("packages", "packages/core"));
    }

    #[test]
    fn test_pattern_matches_package_wildcard() {
        let manager = ConfigManager::new();
        
        // Single wildcard
        assert!(manager.pattern_matches_package("packages/*", "packages/core"));
        assert!(manager.pattern_matches_package("packages/*", "packages/ui"));
        assert!(!manager.pattern_matches_package("packages/*", "packages/apps/web"));
        assert!(!manager.pattern_matches_package("packages/*", "apps/core"));
        
        // Wildcard at beginning
        assert!(manager.pattern_matches_package("*/core", "packages/core"));
        assert!(manager.pattern_matches_package("*/core", "apps/core"));
        assert!(!manager.pattern_matches_package("*/core", "packages/ui"));
        
        // Multiple segments with wildcard
        assert!(manager.pattern_matches_package("packages/*/src", "packages/core/src"));
        assert!(manager.pattern_matches_package("packages/*/src", "packages/ui/src"));
        assert!(!manager.pattern_matches_package("packages/*/src", "packages/core/dist"));
    }

    #[test]
    fn test_pattern_matches_package_double_wildcard() {
        let manager = ConfigManager::new();
        
        // Double wildcard for recursive matching
        assert!(manager.pattern_matches_package("packages/**", "packages/core"));
        assert!(manager.pattern_matches_package("packages/**", "packages/apps/web"));
        assert!(manager.pattern_matches_package("packages/**", "packages/libs/shared/utils"));
        assert!(!manager.pattern_matches_package("packages/**", "apps/core"));
        
        // Double wildcard in the middle
        assert!(manager.pattern_matches_package("packages/**/lib", "packages/core/lib"));
        assert!(manager.pattern_matches_package("packages/**/lib", "packages/apps/web/lib"));
        assert!(!manager.pattern_matches_package("packages/**/lib", "packages/core/dist"));
    }

    #[test]
    fn test_pattern_matches_package_scoped() {
        let manager = ConfigManager::new();
        
        // Scoped packages
        assert!(manager.pattern_matches_package("@company/*", "@company/core"));
        assert!(manager.pattern_matches_package("@company/*", "@company/ui"));
        assert!(!manager.pattern_matches_package("@company/*", "@other/core"));
        assert!(!manager.pattern_matches_package("@company/*", "company/core"));
        
        // Multiple scopes
        assert!(manager.pattern_matches_package("@*/core", "@company/core"));
        assert!(manager.pattern_matches_package("@*/core", "@org/core"));
        assert!(!manager.pattern_matches_package("@*/core", "@company/ui"));
    }

    #[test]
    fn test_pattern_matches_package_character_classes() {
        let manager = ConfigManager::new();
        
        // Character classes
        assert!(manager.pattern_matches_package("packages/[abc]ore", "packages/core"));
        assert!(manager.pattern_matches_package("packages/[abc]ore", "packages/bore"));
        assert!(!manager.pattern_matches_package("packages/[abc]ore", "packages/dore"));
        
        // Negated character classes
        assert!(manager.pattern_matches_package("packages/[!_]*", "packages/core"));
        assert!(manager.pattern_matches_package("packages/[!_]*", "packages/ui"));
        assert!(!manager.pattern_matches_package("packages/[!_]*", "packages/_internal"));
        
        // Range in character class
        assert!(manager.pattern_matches_package("packages/v[0-9]", "packages/v1"));
        assert!(manager.pattern_matches_package("packages/v[0-9]", "packages/v5"));
        assert!(!manager.pattern_matches_package("packages/v[0-9]", "packages/v10"));
    }

    #[test]
    fn test_pattern_matches_package_question_mark() {
        let manager = ConfigManager::new();
        
        // Question mark for single character
        assert!(manager.pattern_matches_package("packages/cor?", "packages/core"));
        assert!(manager.pattern_matches_package("packages/cor?", "packages/cord"));
        assert!(!manager.pattern_matches_package("packages/cor?", "packages/cores"));
        assert!(!manager.pattern_matches_package("packages/cor?", "packages/cor"));
    }

    #[test]
    fn test_pattern_matches_package_edge_cases() {
        let manager = ConfigManager::new();
        
        // Empty strings
        assert!(manager.pattern_matches_package("", ""));
        assert!(!manager.pattern_matches_package("", "packages"));
        assert!(!manager.pattern_matches_package("packages", ""));
        
        // Windows path separators (should be normalized)
        assert!(manager.pattern_matches_package("packages\\*", "packages/core"));
        assert!(manager.pattern_matches_package("packages/*", "packages\\core"));
        
        // Special characters in package names
        assert!(manager.pattern_matches_package("packages/*-utils", "packages/string-utils"));
        assert!(manager.pattern_matches_package("packages/*_utils", "packages/string_utils"));
        assert!(manager.pattern_matches_package("packages/*.js", "packages/index.js"));
        
        // Invalid patterns should fall back to exact match
        assert!(!manager.pattern_matches_package("[invalid", "packages/core"));
        assert!(manager.pattern_matches_package("[invalid", "[invalid"));
    }

    #[test]
    fn test_pattern_matches_package_complex_patterns() {
        let manager = ConfigManager::new();
        
        // Complex real-world patterns
        assert!(manager.pattern_matches_package("packages/*/src/**/*.ts", "packages/core/src/index.ts"));
        assert!(manager.pattern_matches_package("packages/*/src/**/*.ts", "packages/ui/src/components/Button.ts"));
        assert!(!manager.pattern_matches_package("packages/*/src/**/*.ts", "packages/core/dist/index.js"));
        
        // Patterns with multiple wildcards
        assert!(manager.pattern_matches_package("**/node_modules/**", "packages/core/node_modules/react/index.js"));
        assert!(manager.pattern_matches_package("**/node_modules/**", "node_modules/react/index.js"));
        assert!(!manager.pattern_matches_package("**/node_modules/**", "packages/core/src/index.js"));
    }
}