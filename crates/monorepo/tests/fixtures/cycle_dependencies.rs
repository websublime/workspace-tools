use crate::fixtures::{create_monorepo_base, create_package};
use tempfile::TempDir;

/// Create a monorepo with cycle dependencies using the specified package names
pub fn create_cycle_dependencies_monorepo(package_manager: Option<&str>) -> TempDir {
    let temp_dir = create_monorepo_base();
    let repo_path = temp_dir.path();

    // Add package manager files if specified
    if let Some(manager) = package_manager {
        match manager {
            "npm" => crate::fixtures::add_npm_manager(repo_path).unwrap(),
            "yarn" => crate::fixtures::add_yarn_manager(repo_path).unwrap(),
            "pnpm" => crate::fixtures::add_pnpm_manager(repo_path).unwrap(),
            "bun" => crate::fixtures::add_bun_manager(repo_path).unwrap(),
            _ => panic!("Unsupported package manager: {manager}"),
        }
    } else {
        // Default to npm if no package manager specified
        crate::fixtures::add_npm_manager(repo_path).unwrap();
    }

    // Using exactly the packages from the spec to create a cycle:
    // foo -> bar -> baz -> foo

    // First, create modified package-foo that depends on package-bar
    let pkg_foo_json = r#"{
        "name": "@scope/package-foo",
        "version": "1.0.0",
        "dependencies": {
            "@scope/package-bar": "1.0.0"
        }
    }"#;

    // Create package-bar that depends on package-baz
    let pkg_bar_json = r#"{
        "name": "@scope/package-bar",
        "version": "1.0.0",
        "dependencies": {
            "@scope/package-baz": "1.0.0"
        }
    }"#;

    // Create package-baz that depends on package-foo (creating the cycle)
    let pkg_baz_json = r#"{
        "name": "@scope/package-baz",
        "version": "1.0.0",
        "dependencies": {
            "@scope/package-foo": "1.0.0"
        }
    }"#;

    let index_mjs = r#"export const dummy = "dummy";"#;

    create_package(
        repo_path,
        "package-foo",
        pkg_foo_json,
        index_mjs,
        "feature/package-foo-cycle",
        "feat: add package foo with cycle",
        "@scope/package-foo@1.0.0",
        "chore: release package-foo@1.0.0",
    )
    .unwrap();

    create_package(
        repo_path,
        "package-bar",
        pkg_bar_json,
        index_mjs,
        "feature/package-bar-cycle",
        "feat: add package bar with cycle",
        "@scope/package-bar@1.0.0",
        "chore: release package-bar@1.0.0",
    )
    .unwrap();

    create_package(
        repo_path,
        "package-baz",
        pkg_baz_json,
        index_mjs,
        "feature/package-baz-cycle",
        "feat: add package baz with cycle",
        "@scope/package-baz@1.0.0",
        "chore: release package-baz@1.0.0",
    )
    .unwrap();

    temp_dir
}

// Rstest fixtures
use rstest::*;

#[fixture]
pub fn npm_monorepo() -> TempDir {
    crate::fixtures::create_complete_monorepo(Some("npm"))
}

#[fixture]
pub fn yarn_monorepo() -> TempDir {
    crate::fixtures::create_complete_monorepo(Some("yarn"))
}

#[fixture]
pub fn pnpm_monorepo() -> TempDir {
    crate::fixtures::create_complete_monorepo(Some("pnpm"))
}

#[fixture]
pub fn bun_monorepo() -> TempDir {
    crate::fixtures::create_complete_monorepo(Some("bun"))
}

#[fixture]
pub fn npm_cycle_monorepo() -> TempDir {
    create_cycle_dependencies_monorepo(Some("npm"))
}

#[fixture]
pub fn yarn_cycle_monorepo() -> TempDir {
    create_cycle_dependencies_monorepo(Some("yarn"))
}

#[fixture]
pub fn pnpm_cycle_monorepo() -> TempDir {
    create_cycle_dependencies_monorepo(Some("pnpm"))
}

#[fixture]
pub fn bun_cycle_monorepo() -> TempDir {
    create_cycle_dependencies_monorepo(Some("bun"))
}
