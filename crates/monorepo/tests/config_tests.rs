//! Tests for configuration management

use sublime_monorepo_tools::config::{
    ConfigManager, MonorepoConfig, VersionBumpType, Environment,
};
use tempfile::TempDir;

#[test]
fn test_default_config() {
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
    assert_eq!(library.hooks.pre_push.run_tasks.len(), 3);
    
    let app = MonorepoConfig::application_project();
    assert!(app.versioning.snapshot_format.contains("{branch}"));
    assert_eq!(app.environments.len(), 4);
    assert!(app.changesets.auto_deploy);
}

#[test]
fn test_config_manager() {
    let manager = ConfigManager::new();
    let config = manager.get().unwrap();
    
    assert_eq!(config.versioning.default_bump, VersionBumpType::Patch);
    
    // Test update
    manager.update(|config| {
        config.versioning.default_bump = VersionBumpType::Minor;
        config.versioning.auto_tag = false;
    }).unwrap();
    
    let updated_config = manager.get().unwrap();
    assert_eq!(updated_config.versioning.default_bump, VersionBumpType::Minor);
    assert!(!updated_config.versioning.auto_tag);
}

#[test]
fn test_config_save_load_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    
    // Create and save config
    let original_config = MonorepoConfig::library_project();
    let manager = ConfigManager::with_config(original_config);
    manager.save_to_file(&config_path).unwrap();
    
    // Load config
    let loaded_manager = ConfigManager::load_from_file(&config_path).unwrap();
    let loaded_config = loaded_manager.get().unwrap();
    
    assert_eq!(loaded_config.versioning.default_bump, VersionBumpType::Minor);
    assert!(loaded_config.changelog.include_breaking_changes);
}

#[test]
fn test_config_save_load_toml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    
    // Create and save config
    let original_config = MonorepoConfig::small_project();
    let manager = ConfigManager::with_config(original_config);
    manager.save_to_file(&config_path).unwrap();
    
    // Load config
    let loaded_manager = ConfigManager::load_from_file(&config_path).unwrap();
    let loaded_config = loaded_manager.get().unwrap();
    
    assert!(!loaded_config.tasks.parallel);
    assert_eq!(loaded_config.tasks.max_concurrent, 2);
}

#[test]
fn test_config_save_load_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    // Create and save config
    let original_config = MonorepoConfig::large_project();
    let manager = ConfigManager::with_config(original_config);
    manager.save_to_file(&config_path).unwrap();
    
    // Load config
    let loaded_manager = ConfigManager::load_from_file(&config_path).unwrap();
    let loaded_config = loaded_manager.get().unwrap();
    
    assert!(loaded_config.tasks.parallel);
    assert_eq!(loaded_config.tasks.max_concurrent, 8);
}

#[test]
fn test_find_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let monorepo_dir = temp_dir.path().join(".monorepo");
    std::fs::create_dir_all(&monorepo_dir).unwrap();
    
    let config_path = monorepo_dir.join("config.json");
    let config = MonorepoConfig::default();
    let json = serde_json::to_string_pretty(&config).unwrap();
    std::fs::write(&config_path, json).unwrap();
    
    // Should find config in .monorepo directory
    let found = ConfigManager::find_config_file(temp_dir.path());
    assert!(found.is_some());
    assert_eq!(found.unwrap(), config_path);
    
    // Should find config in parent directory
    let sub_dir = temp_dir.path().join("sub").join("directory");
    std::fs::create_dir_all(&sub_dir).unwrap();
    let found_from_sub = ConfigManager::find_config_file(&sub_dir);
    assert!(found_from_sub.is_some());
    assert_eq!(found_from_sub.unwrap(), config_path);
}

#[test]
fn test_create_default_config_files() {
    let temp_dir = TempDir::new().unwrap();
    
    ConfigManager::create_default_config_files(temp_dir.path()).unwrap();
    
    let config_dir = temp_dir.path().join(".monorepo");
    assert!(config_dir.exists());
    assert!(config_dir.is_dir());
    
    let json_path = config_dir.join("config.json");
    assert!(json_path.exists());
    
    let toml_path = config_dir.join("config.toml");
    assert!(toml_path.exists());
    
    // Verify configs can be loaded
    let json_manager = ConfigManager::load_from_file(&json_path).unwrap();
    let json_config = json_manager.get().unwrap();
    assert_eq!(json_config.versioning.default_bump, VersionBumpType::Patch);
    
    let toml_manager = ConfigManager::load_from_file(&toml_path).unwrap();
    let toml_config = toml_manager.get().unwrap();
    assert_eq!(toml_config.versioning.default_bump, VersionBumpType::Patch);
}

#[test]
fn test_config_sections() {
    let manager = ConfigManager::new();
    
    let versioning = manager.get_versioning().unwrap();
    assert_eq!(versioning.default_bump, VersionBumpType::Patch);
    
    let tasks = manager.get_tasks().unwrap();
    assert_eq!(tasks.default_tasks, vec!["test", "lint"]);
    
    let changelog = manager.get_changelog().unwrap();
    assert!(changelog.include_breaking_changes);
    
    let hooks = manager.get_hooks().unwrap();
    assert!(hooks.enabled);
    
    let changesets = manager.get_changesets().unwrap();
    assert!(changesets.required);
    
    let plugins = manager.get_plugins().unwrap();
    assert!(plugins.enabled.is_empty());
    
    let environments = manager.get_environments().unwrap();
    assert_eq!(environments.len(), 3);
}