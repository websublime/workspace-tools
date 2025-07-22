//! # MonorepoKind Tests
//!
//! ## What
//! This module tests the MonorepoKind enum functionality, including name
//! retrieval, config file identification, and custom type creation.
//!
//! ## How
//! Tests verify that each MonorepoKind variant returns the correct name,
//! config file, and can be customized properly.
//!
//! ## Why
//! Proper testing of MonorepoKind ensures reliable monorepo type detection
//! and configuration across different package managers and tools.

use crate::monorepo::MonorepoKind;

#[tokio::test]
async fn test_monorepo_kind_names() {
    assert_eq!(MonorepoKind::NpmWorkSpace.name(), "npm");
    assert_eq!(MonorepoKind::YarnWorkspaces.name(), "yarn");
    assert_eq!(MonorepoKind::PnpmWorkspaces.name(), "pnpm");
    assert_eq!(MonorepoKind::BunWorkspaces.name(), "bun");
    assert_eq!(MonorepoKind::DenoWorkspaces.name(), "deno");

    let custom =
        MonorepoKind::Custom { name: "turbo".to_string(), config_file: "turbo.json".to_string() };
    assert_eq!(custom.name(), "turbo");
}

#[tokio::test]
async fn test_monorepo_kind_config_files() {
    assert_eq!(MonorepoKind::NpmWorkSpace.config_file(), "package.json");
    assert_eq!(MonorepoKind::YarnWorkspaces.config_file(), "package.json");
    assert_eq!(MonorepoKind::PnpmWorkspaces.config_file(), "pnpm-workspace.yaml");
    assert_eq!(MonorepoKind::BunWorkspaces.config_file(), "bunfig.toml");
    assert_eq!(MonorepoKind::DenoWorkspaces.config_file(), "deno.json");

    let custom =
        MonorepoKind::Custom { name: "nx".to_string(), config_file: "nx.json".to_string() };
    assert_eq!(custom.config_file(), "nx.json");
}

#[tokio::test]
async fn test_set_custom() {
    let npm = MonorepoKind::NpmWorkSpace;
    let custom = npm.set_custom("rush".to_string(), "rush.json".to_string());

    assert_eq!(custom.name(), "rush");
    assert_eq!(custom.config_file(), "rush.json");

    // Original should be unchanged
    assert_eq!(npm.name(), "npm");
}
