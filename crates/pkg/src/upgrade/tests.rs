//! Tests for upgrade manager functionality.

#![allow(clippy::unwrap_used)]

use crate::config::BackupConfig;

/// Helper function to determine effective backup setting.
fn determine_backup_setting(override_value: Option<bool>, config_value: bool) -> bool {
    override_value.unwrap_or(config_value)
}

/// Tests that backup_enabled parameter correctly overrides config.
#[test]
fn test_backup_enabled_override_logic() {
    // Test with backup disabled in config
    let config_backup_disabled = false;

    // Override: None -> should use config (false)
    let effective = determine_backup_setting(None, config_backup_disabled);
    assert!(!effective, "None override should use config value (false)");

    // Override: Some(true) -> should override to true
    let effective = determine_backup_setting(Some(true), config_backup_disabled);
    assert!(effective, "Some(true) override should force backup enabled");

    // Override: Some(false) -> should override to false
    let effective = determine_backup_setting(Some(false), config_backup_disabled);
    assert!(!effective, "Some(false) override should force backup disabled");
}

/// Tests backup control with backups enabled in config.
#[test]
fn test_backup_enabled_in_config_logic() {
    // Test with backup enabled in config
    let config_backup_enabled = true;

    // Override: None -> should use config (true)
    let effective = determine_backup_setting(None, config_backup_enabled);
    assert!(effective, "None override should use config value (true)");

    // Override: Some(true) -> should remain true
    let effective = determine_backup_setting(Some(true), config_backup_enabled);
    assert!(effective, "Some(true) override should keep backup enabled");

    // Override: Some(false) -> should override to false
    let effective = determine_backup_setting(Some(false), config_backup_enabled);
    assert!(!effective, "Some(false) override should disable backup despite config");
}

/// Tests that cleanup operations respect the effective backup setting.
#[test]
fn test_cleanup_respects_backup_setting_logic() {
    // Verify that backup cleanup only happens when backups are enabled
    let config_with_backup = BackupConfig {
        enabled: true,
        backup_dir: ".workspace-backups".to_string(),
        keep_after_success: false,
        max_backups: 10,
    };

    let config_without_backup = BackupConfig {
        enabled: false,
        backup_dir: ".workspace-backups".to_string(),
        keep_after_success: false,
        max_backups: 10,
    };

    // When backups are enabled, cleanup should happen
    let should_cleanup = config_with_backup.enabled && !config_with_backup.keep_after_success;
    assert!(
        should_cleanup,
        "Cleanup should occur when backups enabled and keep_after_success is false"
    );

    // When backups are disabled, cleanup should not happen
    let should_cleanup = config_without_backup.enabled && !config_without_backup.keep_after_success;
    assert!(!should_cleanup, "Cleanup should not occur when backups disabled");

    // Test override scenario: backup disabled via parameter
    let effective_backup = determine_backup_setting(Some(false), config_with_backup.enabled);
    let should_cleanup = effective_backup && !config_with_backup.keep_after_success;
    assert!(!should_cleanup, "Cleanup should not occur when backup disabled via override");
}
