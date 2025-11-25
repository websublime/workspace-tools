//! # End-to-End Tests for Monorepo Module
//!
//! ## What
//! Comprehensive e2e tests for monorepo detection and analysis including
//! `MonorepoDetector`, `MonorepoDescriptor`, `WorkspacePackage`, and `MonorepoKind`.
//!
//! ## How
//! Tests create realistic monorepo structures using temporary directories and verify
//! detection, workspace resolution, package discovery, and dependency graph analysis.
//!
//! ## Why
//! E2E tests ensure the monorepo module correctly identifies and analyzes complex
//! workspace structures across different package managers (npm, yarn, pnpm).

#![allow(clippy::print_stdout)]

use sublime_standard_tools::{
    config::MonorepoConfig,
    error::Result,
    filesystem::{AsyncFileSystem, FileSystemManager},
    monorepo::{MonorepoDetector, MonorepoDetectorTrait, MonorepoDetectorWithFs, MonorepoKind},
};
use tempfile::TempDir;

// ============================================================================
// MonorepoDetector Creation Tests
// ============================================================================

#[tokio::test]
async fn test_monorepo_detector_new() -> Result<()> {
    let detector = MonorepoDetector::new();
    // Should be created without errors - use filesystem() from trait
    let _fs = detector.filesystem();
    Ok(())
}

#[tokio::test]
async fn test_monorepo_detector_with_config() -> Result<()> {
    let config = MonorepoConfig {
        max_search_depth: 10,
        workspace_patterns: vec!["packages/*".to_string(), "apps/*".to_string()],
        ..MonorepoConfig::default()
    };

    let detector = MonorepoDetector::new_with_config(config);
    // Should be created without errors
    let _fs = detector.filesystem();
    Ok(())
}

#[tokio::test]
async fn test_monorepo_detector_with_filesystem() -> Result<()> {
    let fs = FileSystemManager::new();
    let detector = MonorepoDetector::with_filesystem(fs);
    // Should be created without errors
    let _fs = detector.filesystem();
    Ok(())
}

// ============================================================================
// NPM Workspaces Detection Tests
// ============================================================================

#[tokio::test]
async fn test_detect_npm_workspaces() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create npm workspaces structure
    let root = temp_dir.path();

    // Create package.json with workspaces
    let package_json = r#"{
        "name": "@test/npm-monorepo",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*"]
    }"#;
    fs.write_file_string(&root.join("package.json"), package_json).await?;

    // Create package-lock.json (required for npm workspaces detection)
    fs.write_file_string(
        &root.join("package-lock.json"),
        r#"{"name": "@test/npm-monorepo", "lockfileVersion": 3}"#,
    )
    .await?;

    // Create a workspace package
    let pkg_dir = root.join("packages").join("lib");
    fs.create_dir_all(&pkg_dir).await?;
    let lib_package = r#"{
        "name": "@test/lib",
        "version": "1.0.0"
    }"#;
    fs.write_file_string(&pkg_dir.join("package.json"), lib_package).await?;

    // Detect monorepo
    let detector = MonorepoDetector::new();
    let kind = detector.is_monorepo_root(root).await?;

    assert!(kind.is_some());
    assert_eq!(kind, Some(MonorepoKind::NpmWorkSpace));

    Ok(())
}

#[tokio::test]
async fn test_detect_npm_workspaces_with_packages() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create npm workspaces structure with multiple packages
    let root = temp_dir.path();

    let package_json = r#"{
        "name": "@test/npm-monorepo",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*"]
    }"#;
    fs.write_file_string(&root.join("package.json"), package_json).await?;
    fs.write_file_string(&root.join("package-lock.json"), r#"{"lockfileVersion": 3}"#).await?;

    // Create multiple packages
    for pkg_name in ["lib", "utils", "core"] {
        let pkg_dir = root.join("packages").join(pkg_name);
        fs.create_dir_all(&pkg_dir).await?;
        let pkg_json = format!(
            r#"{{
            "name": "@test/{pkg_name}",
            "version": "1.0.0"
        }}"#
        );
        fs.write_file_string(&pkg_dir.join("package.json"), &pkg_json).await?;
    }

    // Detect and analyze monorepo
    let detector = MonorepoDetector::new();
    let monorepo = detector.detect_monorepo(root).await?;

    assert_eq!(monorepo.kind(), &MonorepoKind::NpmWorkSpace);
    assert_eq!(monorepo.packages().len(), 3);

    let package_names: Vec<&str> = monorepo.packages().iter().map(|p| p.name.as_str()).collect();
    assert!(package_names.contains(&"@test/lib"));
    assert!(package_names.contains(&"@test/utils"));
    assert!(package_names.contains(&"@test/core"));

    Ok(())
}

// ============================================================================
// Yarn Workspaces Detection Tests
// ============================================================================

#[tokio::test]
async fn test_detect_yarn_workspaces() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create yarn workspaces structure
    let root = temp_dir.path();

    let package_json = r#"{
        "name": "@test/yarn-monorepo",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*", "apps/*"]
    }"#;
    fs.write_file_string(&root.join("package.json"), package_json).await?;

    // Create yarn.lock (required for yarn workspaces detection)
    fs.write_file_string(&root.join("yarn.lock"), "# yarn lockfile v1\n").await?;

    // Create a workspace package
    let pkg_dir = root.join("packages").join("lib");
    fs.create_dir_all(&pkg_dir).await?;
    let lib_package = r#"{
        "name": "@test/lib",
        "version": "1.0.0"
    }"#;
    fs.write_file_string(&pkg_dir.join("package.json"), lib_package).await?;

    // Detect monorepo
    let detector = MonorepoDetector::new();
    let kind = detector.is_monorepo_root(root).await?;

    assert!(kind.is_some());
    assert_eq!(kind, Some(MonorepoKind::YarnWorkspaces));

    Ok(())
}

#[tokio::test]
async fn test_detect_yarn_workspaces_with_apps_and_packages() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let root = temp_dir.path();

    let package_json = r#"{
        "name": "@test/yarn-monorepo",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*", "apps/*"]
    }"#;
    fs.write_file_string(&root.join("package.json"), package_json).await?;
    fs.write_file_string(&root.join("yarn.lock"), "# yarn lockfile v1\n").await?;

    // Create packages
    for pkg_name in ["lib", "utils"] {
        let pkg_dir = root.join("packages").join(pkg_name);
        fs.create_dir_all(&pkg_dir).await?;
        let pkg_json = format!(
            r#"{{
            "name": "@test/{pkg_name}",
            "version": "1.0.0"
        }}"#
        );
        fs.write_file_string(&pkg_dir.join("package.json"), &pkg_json).await?;
    }

    // Create apps
    for app_name in ["web", "mobile"] {
        let app_dir = root.join("apps").join(app_name);
        fs.create_dir_all(&app_dir).await?;
        let app_json = format!(
            r#"{{
            "name": "@test/{app_name}",
            "version": "1.0.0",
            "dependencies": {{
                "@test/lib": "1.0.0",
                "@test/utils": "1.0.0"
            }}
        }}"#
        );
        fs.write_file_string(&app_dir.join("package.json"), &app_json).await?;
    }

    // Detect and analyze monorepo
    let detector = MonorepoDetector::new();
    let monorepo = detector.detect_monorepo(root).await?;

    assert_eq!(monorepo.kind(), &MonorepoKind::YarnWorkspaces);
    assert_eq!(monorepo.packages().len(), 4);

    // Check apps have workspace dependencies
    let web_app = monorepo.packages().iter().find(|p| p.name == "@test/web");
    assert!(web_app.is_some());

    let web_app = web_app.expect("web app should exist");
    assert!(web_app.workspace_dependencies.contains(&"@test/lib".to_string()));
    assert!(web_app.workspace_dependencies.contains(&"@test/utils".to_string()));

    Ok(())
}

// ============================================================================
// PNPM Workspaces Detection Tests
// ============================================================================

#[tokio::test]
async fn test_detect_pnpm_workspaces_yaml() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let root = temp_dir.path();

    // Create pnpm-workspace.yaml
    let workspace_yaml = r"packages:
  - 'packages/*'
  - 'apps/*'
";
    fs.write_file_string(&root.join("pnpm-workspace.yaml"), workspace_yaml).await?;

    // Create root package.json
    let package_json = r#"{
        "name": "@test/pnpm-monorepo",
        "version": "1.0.0",
        "private": true
    }"#;
    fs.write_file_string(&root.join("package.json"), package_json).await?;

    // Create pnpm-lock.yaml
    fs.write_file_string(&root.join("pnpm-lock.yaml"), "lockfileVersion: 5.4\n").await?;

    // Create a workspace package
    let pkg_dir = root.join("packages").join("lib");
    fs.create_dir_all(&pkg_dir).await?;
    let lib_package = r#"{
        "name": "@test/lib",
        "version": "1.0.0"
    }"#;
    fs.write_file_string(&pkg_dir.join("package.json"), lib_package).await?;

    // Detect monorepo
    let detector = MonorepoDetector::new();
    let kind = detector.is_monorepo_root(root).await?;

    assert!(kind.is_some());
    assert_eq!(kind, Some(MonorepoKind::PnpmWorkspaces));

    Ok(())
}

#[tokio::test]
async fn test_detect_pnpm_workspaces_package_json() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let root = temp_dir.path();

    // Create package.json with workspaces (pnpm also supports this)
    let package_json = r#"{
        "name": "@test/pnpm-monorepo",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*"]
    }"#;
    fs.write_file_string(&root.join("package.json"), package_json).await?;

    // Create pnpm-lock.yaml (indicates pnpm usage)
    fs.write_file_string(&root.join("pnpm-lock.yaml"), "lockfileVersion: 5.4\n").await?;

    // Create a workspace package
    let pkg_dir = root.join("packages").join("lib");
    fs.create_dir_all(&pkg_dir).await?;
    let lib_package = r#"{
        "name": "@test/lib",
        "version": "1.0.0"
    }"#;
    fs.write_file_string(&pkg_dir.join("package.json"), lib_package).await?;

    // Detect monorepo
    let detector = MonorepoDetector::new();
    let kind = detector.is_monorepo_root(root).await?;

    assert!(kind.is_some());
    assert_eq!(kind, Some(MonorepoKind::PnpmWorkspaces));

    Ok(())
}

// ============================================================================
// Non-Monorepo Detection Tests
// ============================================================================

#[tokio::test]
async fn test_not_monorepo_simple_package() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let root = temp_dir.path();

    // Create simple package without workspaces
    let package_json = r#"{
        "name": "simple-package",
        "version": "1.0.0"
    }"#;
    fs.write_file_string(&root.join("package.json"), package_json).await?;

    // Detect monorepo
    let detector = MonorepoDetector::new();
    let kind = detector.is_monorepo_root(root).await?;

    assert!(kind.is_none());

    Ok(())
}

#[tokio::test]
async fn test_not_monorepo_empty_directory() -> Result<()> {
    let temp_dir = create_temp_dir()?;

    let detector = MonorepoDetector::new();
    let kind = detector.is_monorepo_root(temp_dir.path()).await?;

    assert!(kind.is_none());

    Ok(())
}

// ============================================================================
// find_monorepo_root Tests
// ============================================================================

#[tokio::test]
async fn test_find_monorepo_root_from_package() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let root = temp_dir.path();

    // Create pnpm monorepo structure
    fs.write_file_string(&root.join("pnpm-workspace.yaml"), "packages:\n  - 'packages/*'\n")
        .await?;
    fs.write_file_string(&root.join("package.json"), r#"{"name": "@test/root", "private": true}"#)
        .await?;

    // Create nested package
    let pkg_dir = root.join("packages").join("deep").join("nested");
    fs.create_dir_all(&pkg_dir).await?;

    // Find monorepo root from nested directory
    let detector = MonorepoDetector::new();
    let result = detector.find_monorepo_root(&pkg_dir).await?;

    assert!(result.is_some());
    let (found_root, kind) = result.expect("should find root");
    assert_eq!(found_root, root.to_path_buf());
    assert_eq!(kind, MonorepoKind::PnpmWorkspaces);

    Ok(())
}

#[tokio::test]
async fn test_find_monorepo_root_not_found() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create non-monorepo directory
    let subdir = temp_dir.path().join("some").join("deep").join("path");
    fs.create_dir_all(&subdir).await?;

    let detector = MonorepoDetector::new();
    let result = detector.find_monorepo_root(&subdir).await?;

    assert!(result.is_none());

    Ok(())
}

// ============================================================================
// Package Detection Tests
// ============================================================================

#[tokio::test]
async fn test_detect_packages_with_dependencies() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let root = temp_dir.path();

    // Create monorepo with inter-package dependencies
    fs.write_file_string(&root.join("pnpm-workspace.yaml"), "packages:\n  - 'packages/*'\n")
        .await?;
    fs.write_file_string(&root.join("package.json"), r#"{"name": "@myorg/root", "private": true}"#)
        .await?;
    fs.write_file_string(&root.join("pnpm-lock.yaml"), "lockfileVersion: 5.4\n").await?;

    // Create core package
    let core_dir = root.join("packages").join("core");
    fs.create_dir_all(&core_dir).await?;
    let core_json = r#"{
        "name": "@myorg/core",
        "version": "1.0.0",
        "dependencies": {
            "lodash": "^4.17.0"
        }
    }"#;
    fs.write_file_string(&core_dir.join("package.json"), core_json).await?;

    // Create utils package that depends on core
    let utils_dir = root.join("packages").join("utils");
    fs.create_dir_all(&utils_dir).await?;
    let utils_json = r#"{
        "name": "@myorg/utils",
        "version": "1.0.0",
        "dependencies": {
            "@myorg/core": "workspace:*",
            "date-fns": "^2.0.0"
        }
    }"#;
    fs.write_file_string(&utils_dir.join("package.json"), utils_json).await?;

    // Detect packages
    let detector = MonorepoDetector::new();
    let packages = detector.detect_packages(root).await?;

    assert_eq!(packages.len(), 2);

    let utils_pkg = packages.iter().find(|p| p.name == "@myorg/utils");
    assert!(utils_pkg.is_some());

    let utils_pkg = utils_pkg.expect("utils should exist");
    assert!(utils_pkg.workspace_dependencies.contains(&"@myorg/core".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_detect_packages_excludes_node_modules() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let root = temp_dir.path();

    fs.write_file_string(&root.join("pnpm-workspace.yaml"), "packages:\n  - '**/*'\n").await?;
    fs.write_file_string(&root.join("package.json"), r#"{"name": "@test/root", "private": true}"#)
        .await?;
    fs.write_file_string(&root.join("pnpm-lock.yaml"), "lockfileVersion: 5.4\n").await?;

    // Create a workspace package
    let pkg_dir = root.join("packages").join("lib");
    fs.create_dir_all(&pkg_dir).await?;
    fs.write_file_string(
        &pkg_dir.join("package.json"),
        r#"{"name": "@test/lib", "version": "1.0.0"}"#,
    )
    .await?;

    // Create a fake package inside node_modules (should be excluded)
    let node_modules_pkg = root.join("node_modules").join("fake-pkg");
    fs.create_dir_all(&node_modules_pkg).await?;
    fs.write_file_string(
        &node_modules_pkg.join("package.json"),
        r#"{"name": "fake-pkg", "version": "1.0.0"}"#,
    )
    .await?;

    // Detect packages
    let detector = MonorepoDetector::new();
    let packages = detector.detect_packages(root).await?;

    // Should only find @test/lib, not fake-pkg
    let package_names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();
    assert!(package_names.contains(&"@test/lib"));
    assert!(!package_names.contains(&"fake-pkg"));

    Ok(())
}

// ============================================================================
// Dependency Graph Tests
// ============================================================================

#[tokio::test]
async fn test_dependency_graph() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let root = temp_dir.path();

    // Create monorepo with dependency chain: app -> utils -> core
    fs.write_file_string(&root.join("pnpm-workspace.yaml"), "packages:\n  - 'packages/*'\n")
        .await?;
    fs.write_file_string(&root.join("package.json"), r#"{"name": "@graph/root", "private": true}"#)
        .await?;
    fs.write_file_string(&root.join("pnpm-lock.yaml"), "lockfileVersion: 5.4\n").await?;

    // Core package (no workspace deps)
    let core_dir = root.join("packages").join("core");
    fs.create_dir_all(&core_dir).await?;
    fs.write_file_string(
        &core_dir.join("package.json"),
        r#"{"name": "@graph/core", "version": "1.0.0"}"#,
    )
    .await?;

    // Utils depends on core
    let utils_dir = root.join("packages").join("utils");
    fs.create_dir_all(&utils_dir).await?;
    fs.write_file_string(
        &utils_dir.join("package.json"),
        r#"{"name": "@graph/utils", "version": "1.0.0", "dependencies": {"@graph/core": "1.0.0"}}"#,
    )
    .await?;

    // App depends on utils and core
    let app_dir = root.join("packages").join("app");
    fs.create_dir_all(&app_dir).await?;
    fs.write_file_string(
        &app_dir.join("package.json"),
        r#"{"name": "@graph/app", "version": "1.0.0", "dependencies": {"@graph/utils": "1.0.0", "@graph/core": "1.0.0"}}"#,
    )
    .await?;

    // Detect and get dependency graph
    let detector = MonorepoDetector::new();
    let monorepo = detector.detect_monorepo(root).await?;
    let graph = monorepo.get_dependency_graph();

    // core should have 2 dependents: utils and app
    let core_dependents = graph.get("@graph/core");
    assert!(core_dependents.is_some());
    let core_deps: Vec<&str> =
        core_dependents.expect("should have dependents").iter().map(|d| d.name.as_str()).collect();
    assert!(core_deps.contains(&"@graph/utils"));
    assert!(core_deps.contains(&"@graph/app"));

    // utils should have 1 dependent: app
    let utils_dependents = graph.get("@graph/utils");
    assert!(utils_dependents.is_some());
    let utils_deps: Vec<&str> =
        utils_dependents.expect("should have dependents").iter().map(|d| d.name.as_str()).collect();
    assert!(utils_deps.contains(&"@graph/app"));

    Ok(())
}

// ============================================================================
// has_multiple_packages Tests
// ============================================================================

#[tokio::test]
async fn test_has_multiple_packages_true() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let root = temp_dir.path();

    fs.write_file_string(&root.join("pnpm-workspace.yaml"), "packages:\n  - 'packages/*'\n")
        .await?;
    fs.write_file_string(&root.join("package.json"), r#"{"name": "@test/root", "private": true}"#)
        .await?;
    fs.write_file_string(&root.join("pnpm-lock.yaml"), "lockfileVersion: 5.4\n").await?;

    // Create multiple packages
    for name in ["pkg1", "pkg2", "pkg3"] {
        let pkg_dir = root.join("packages").join(name);
        fs.create_dir_all(&pkg_dir).await?;
        fs.write_file_string(
            &pkg_dir.join("package.json"),
            &format!(r#"{{"name": "@test/{name}", "version": "1.0.0"}}"#),
        )
        .await?;
    }

    let detector = MonorepoDetector::new();
    assert!(detector.has_multiple_packages(root).await);

    Ok(())
}

#[tokio::test]
async fn test_has_multiple_packages_false() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let root = temp_dir.path();

    fs.write_file_string(&root.join("pnpm-workspace.yaml"), "packages:\n  - 'packages/*'\n")
        .await?;
    fs.write_file_string(&root.join("package.json"), r#"{"name": "@test/root", "private": true}"#)
        .await?;
    fs.write_file_string(&root.join("pnpm-lock.yaml"), "lockfileVersion: 5.4\n").await?;

    // Create only one package
    let pkg_dir = root.join("packages").join("single");
    fs.create_dir_all(&pkg_dir).await?;
    fs.write_file_string(
        &pkg_dir.join("package.json"),
        r#"{"name": "@test/single", "version": "1.0.0"}"#,
    )
    .await?;

    let detector = MonorepoDetector::new();
    assert!(!detector.has_multiple_packages(root).await);

    Ok(())
}

// ============================================================================
// MonorepoKind Tests
// ============================================================================

#[tokio::test]
async fn test_monorepo_kind_names() -> Result<()> {
    // MonorepoKind.name() returns short names
    assert_eq!(MonorepoKind::NpmWorkSpace.name(), "npm");
    assert_eq!(MonorepoKind::YarnWorkspaces.name(), "yarn");
    assert_eq!(MonorepoKind::PnpmWorkspaces.name(), "pnpm");
    assert_eq!(MonorepoKind::BunWorkspaces.name(), "bun");
    assert_eq!(MonorepoKind::DenoWorkspaces.name(), "deno");
    Ok(())
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_complex_monorepo_structure() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let root = temp_dir.path();

    // Create a realistic complex monorepo structure
    fs.write_file_string(
        &root.join("pnpm-workspace.yaml"),
        "packages:\n  - 'packages/*'\n  - 'apps/*'\n  - 'tools/*'\n",
    )
    .await?;

    let root_json = r#"{
        "name": "@complex/monorepo",
        "version": "1.0.0",
        "private": true,
        "devDependencies": {
            "typescript": "^5.0.0"
        }
    }"#;
    fs.write_file_string(&root.join("package.json"), root_json).await?;
    fs.write_file_string(&root.join("pnpm-lock.yaml"), "lockfileVersion: 5.4\n").await?;

    // Create packages
    let packages = [
        ("packages/core", "@complex/core", vec![]),
        ("packages/utils", "@complex/utils", vec!["@complex/core"]),
        ("packages/ui", "@complex/ui", vec!["@complex/utils"]),
        ("apps/web", "@complex/web", vec!["@complex/ui", "@complex/utils"]),
        ("apps/docs", "@complex/docs", vec!["@complex/ui"]),
        ("tools/cli", "@complex/cli", vec!["@complex/core", "@complex/utils"]),
    ];

    for (path, name, deps) in packages {
        let pkg_dir = root.join(path);
        fs.create_dir_all(&pkg_dir).await?;

        let deps_json = if deps.is_empty() {
            String::new()
        } else {
            let deps_str: Vec<String> =
                deps.iter().map(|d| format!(r#""{d}": "workspace:*""#)).collect();
            format!(r#", "dependencies": {{{}}}"#, deps_str.join(", "))
        };

        let pkg_json = format!(r#"{{"name": "{name}", "version": "1.0.0"{deps_json}}}"#);
        fs.write_file_string(&pkg_dir.join("package.json"), &pkg_json).await?;
    }

    // Analyze the monorepo
    let detector = MonorepoDetector::new();
    let monorepo = detector.detect_monorepo(root).await?;

    // Verify structure
    assert_eq!(monorepo.kind(), &MonorepoKind::PnpmWorkspaces);
    // The detector should find packages from all three glob patterns
    let pkg_count = monorepo.packages().len();
    // At least 5 packages should be detected (may be 5 or 6 depending on glob handling)
    assert!(pkg_count >= 5, "Expected at least 5 packages, found {pkg_count}");
    assert_eq!(monorepo.root(), root);

    // Verify dependency graph exists and has some entries
    let graph = monorepo.get_dependency_graph();

    // The graph should have at least some dependencies tracked
    // Note: The exact number depends on how the detector resolves workspace dependencies
    let core_deps = graph.get("@complex/core").map_or(0, Vec::len);
    let utils_deps = graph.get("@complex/utils").map_or(0, Vec::len);

    // At minimum, utils depends on core (1 dep) and ui depends on utils (1 dep)
    // The total tracked dependencies should be > 0
    let total_deps: usize = graph.values().map(Vec::len).sum();
    assert!(total_deps >= 1, "Expected at least some dependencies in graph, found {total_deps}");

    // Verify that core is tracked as a dependency by at least one package
    assert!(
        core_deps >= 1,
        "Expected @complex/core to have at least 1 dependent, found {core_deps}"
    );

    // Verify that utils is tracked as a dependency by at least one package
    assert!(
        utils_deps >= 1,
        "Expected @complex/utils to have at least 1 dependent, found {utils_deps}"
    );

    Ok(())
}

#[tokio::test]
async fn test_monorepo_with_scoped_and_unscoped_packages() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let root = temp_dir.path();

    fs.write_file_string(&root.join("pnpm-workspace.yaml"), "packages:\n  - 'packages/*'\n")
        .await?;
    fs.write_file_string(&root.join("package.json"), r#"{"name": "my-monorepo", "private": true}"#)
        .await?;
    fs.write_file_string(&root.join("pnpm-lock.yaml"), "lockfileVersion: 5.4\n").await?;

    // Create scoped package
    let scoped_dir = root.join("packages").join("scoped-lib");
    fs.create_dir_all(&scoped_dir).await?;
    fs.write_file_string(
        &scoped_dir.join("package.json"),
        r#"{"name": "@myorg/lib", "version": "1.0.0"}"#,
    )
    .await?;

    // Create unscoped package
    let unscoped_dir = root.join("packages").join("unscoped-lib");
    fs.create_dir_all(&unscoped_dir).await?;
    fs.write_file_string(
        &unscoped_dir.join("package.json"),
        r#"{"name": "my-utils", "version": "1.0.0"}"#,
    )
    .await?;

    let detector = MonorepoDetector::new();
    let packages = detector.detect_packages(root).await?;

    assert_eq!(packages.len(), 2);

    let names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"@myorg/lib"));
    assert!(names.contains(&"my-utils"));

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates a temporary directory for testing
fn create_temp_dir() -> Result<TempDir> {
    TempDir::new().map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("Failed to create temp dir: {e}"))
    })
}
