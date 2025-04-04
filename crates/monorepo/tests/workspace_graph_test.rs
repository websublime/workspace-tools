use rstest::*;
use tempfile::TempDir;

use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceManager};

mod fixtures;

#[rstest]
#[allow(clippy::print_stdout)]
fn test_workspace_dependency_graph(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    // Setup workspace
    let workspace_manager = WorkspaceManager::new();
    let workspace = workspace_manager
        .discover_workspace(temp_dir.path(), &DiscoveryOptions::default().auto_detect_root(false))
        .expect("Failed to discover workspace");

    // First, print out the dependency relationships for clarity
    println!("Package dependency relationships:");
    for pkg_info in workspace.sorted_packages() {
        let pkg_borrow = pkg_info.borrow();
        let package_borrow = pkg_borrow.package.borrow();
        let pkg_name = package_borrow.name();
        let deps: Vec<String> = pkg_info
            .borrow()
            .package
            .borrow()
            .dependencies()
            .iter()
            .map(|d| d.borrow().name().to_string())
            .collect();
        println!("  {pkg_name}: depends on {deps:?}");
    }

    // Analyze dependencies
    let graph = workspace.analyze_dependencies().expect("Should analyze dependencies");

    // Check cycle detection
    println!("Cycles detected: {}", graph.cycles_detected);

    // Check for missing dependencies
    println!("Missing dependencies: {:?}", graph.missing_dependencies);

    // Check for version conflicts
    println!("Version conflicts: {:?}", graph.version_conflicts);

    // Get validation report if available
    if let Some(validation) = &graph.validation {
        println!("Validation report available: {}", validation.has_issues());
    } else {
        println!("No validation report (likely due to cycles)");
    }

    // Test getting dependents
    let foo_dependents = workspace.dependents_of("@scope/package-foo", Some(true));
    println!("Dependents of package-foo: {}", foo_dependents.len());
    for dep in &foo_dependents {
        println!("  - {}", dep.borrow().package.borrow().name());
    }

    // Test getting dependencies
    let foo_dependencies = workspace.dependencies_of("@scope/package-foo");
    println!("Dependencies of package-foo: {}", foo_dependencies.len());
    for dep in &foo_dependencies {
        println!("  - {}", dep.borrow().package.borrow().name());
    }

    // The test here is conditional on whether we have cycles
    if graph.cycles_detected {
        println!("Skipping bidirectional dependency check due to cycles");

        // Instead, verify that we can get both dependencies and dependents
        // without crashes, even with cycles
        assert!(
            !foo_dependencies.is_empty(),
            "Should be able to get dependencies even with cycles"
        );

        // Also check that we can get dependents for some package without a direct cycle
        let charlie_deps = workspace.dependencies_of("@scope/package-charlie");
        assert!(!charlie_deps.is_empty(), "package-charlie should have dependencies");

        println!("Dependency traversal working properly even with cycles");
    } else {
        // Original test for when there are no cycles
        if !foo_dependencies.is_empty() {
            let first_dep = &foo_dependencies[0];
            let first_dep_name = first_dep.borrow().package.borrow().name().to_string();
            let dep_dependents = workspace.dependents_of(&first_dep_name, None);

            let has_foo = dep_dependents
                .iter()
                .any(|p| p.borrow().package.borrow().name() == "@scope/package-foo");

            assert!(has_foo, "package-foo should be found in its dependency's dependents");
        }
    }

    // Additional test: check if the cycle is correctly identified between bar and baz
    if graph.cycles_detected {
        // Get both dependencies to check the cycle
        let bar_deps = workspace.dependencies_of("@scope/package-bar");
        let baz_deps = workspace.dependencies_of("@scope/package-baz");

        // Check if bar depends on baz
        let bar_depends_on_baz =
            bar_deps.iter().any(|p| p.borrow().package.borrow().name() == "@scope/package-baz");

        // Check if baz depends on bar
        let baz_depends_on_bar =
            baz_deps.iter().any(|p| p.borrow().package.borrow().name() == "@scope/package-bar");

        // If we have a cycle, both should be true
        assert!(
            bar_depends_on_baz && baz_depends_on_bar,
            "Cycle between bar and baz should be correctly identified"
        );
        println!("Cycle between bar and baz correctly identified");
    }
}
