use rstest::*;
use tempfile::TempDir;

use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceManager};

mod fixtures;

#[rstest]
#[allow(clippy::print_stdout)]
fn test_workspace_analysis(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    // Setup workspace
    let workspace_manager = WorkspaceManager::new();
    let workspace = workspace_manager
        .discover_workspace(temp_dir.path(), &DiscoveryOptions::default().auto_detect_root(false))
        .expect("Failed to discover workspace");

    // Test comprehensive workspace analysis
    let analysis =
        workspace_manager.analyze_workspace(&workspace).expect("Should analyze workspace");

    println!("Workspace analysis:");
    println!("  Cycles detected: {}", analysis.cycle_detected);
    println!("  Missing dependencies: {}", analysis.missing_dependencies.len());
    println!("  Version conflicts: {}", analysis.version_conflicts.len());
    println!("  Validation issues: {}", analysis.validation_issues);

    // Test affected packages calculation
    let affected = workspace.affected_packages(&["@scope/package-foo"], Some(false));
    println!("Packages affected by changes to package-foo: {}", affected.len());
    for pkg in &affected {
        println!("  - {}", pkg.borrow().package.borrow().name());
    }

    // Test if package-charlie depends on package-foo and is therefore affected
    // First verify the dependency relation exists
    let charlie_deps = workspace.dependencies_of("@scope/package-charlie");
    let charlie_depends_on_foo =
        charlie_deps.iter().any(|p| p.borrow().package.borrow().name() == "@scope/package-foo");

    if charlie_depends_on_foo {
        // Now verify affected packages calculation works
        let has_charlie =
            affected.iter().any(|p| p.borrow().package.borrow().name() == "@scope/package-charlie");
        assert!(has_charlie, "@scope/package-charlie should be affected by changes to package-foo");
    }

    // Test packages affected by multiple changed packages
    let multi_affected =
        workspace.affected_packages(&["@scope/package-foo", "@scope/package-bar"], None);
    println!("Packages affected by changes to foo and bar: {}", multi_affected.len());
}

#[rstest]
#[allow(clippy::print_stdout)]
fn test_cycle_dependencies(#[from(fixtures::cycle_monorepo)] temp_dir: TempDir) {
    // Setup workspace with known cycles
    let workspace_manager = WorkspaceManager::new();
    let workspace = workspace_manager
        .discover_workspace(temp_dir.path(), &DiscoveryOptions::default().auto_detect_root(false))
        .expect("Failed to discover workspace");

    // Analyze dependencies
    let analysis =
        workspace_manager.analyze_workspace(&workspace).expect("Should analyze workspace");

    // This fixture should definitely have cycles
    assert!(analysis.cycle_detected, "Cycle fixture should detect cycles");

    println!("Circular dependencies detected as expected");

    // Test that affected packages still works with cycles
    let affected = workspace.affected_packages(&["@scope/package-foo"], Some(true));
    println!("Even with cycles, affected packages calculation returns {} packages", affected.len());

    for pkg in &affected {
        println!("  - {}", pkg.borrow().package.borrow().name());
    }
}
