use rstest::*;
use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceManager};
use tempfile::TempDir;

mod fixtures;

/// Helper function to create a workspace from a temp directory
fn create_workspace(temp_dir: &TempDir) -> sublime_monorepo_tools::Workspace {
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    let options = DiscoveryOptions::default()
        .auto_detect_root(false) // We know the root path
        .include_patterns(vec!["**/package.json"]);

    workspace_manager.discover_workspace(root_path, &options).expect("Failed to discover workspace")
}

#[rstest]
fn test_get_package(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let workspace = create_workspace(&temp_dir);

    // Test getting existing package
    let package = workspace.get_package("@scope/package-foo");
    assert!(package.is_some());

    let foo_pkg = package.unwrap();
    assert_eq!(foo_pkg.borrow().package.borrow().name(), "@scope/package-foo");

    // Test getting non-existent package
    let non_existent = workspace.get_package("non-existent-package");
    assert!(non_existent.is_none());
}

#[rstest]
#[allow(clippy::print_stdout)]
fn test_sorted_packages(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    let options = DiscoveryOptions::default().auto_detect_root(false);

    let workspace = workspace_manager
        .discover_workspace(root_path, &options)
        .expect("Failed to discover workspace");

    // Debug the dependency relationships
    println!("All packages and their dependencies:");
    for pkg_info in workspace.sorted_packages() {
        let pkg = pkg_info.borrow();
        let package_borrow = pkg.package.borrow();
        let pkg_name = package_borrow.name();

        let deps: Vec<String> = pkg
            .package
            .borrow()
            .dependencies()
            .iter()
            .map(|d| d.borrow().name().to_string())
            .collect();

        println!("Package: {pkg_name}, Depends on: {deps:?}");
    }

    // Get packages with circular dependency information
    let sorted_result = workspace.get_sorted_packages_with_circulars();

    // Print information about the sorted packages and circular groups
    println!("Regular sorted packages: {}", sorted_result.sorted.len());
    for (i, pkg) in sorted_result.sorted.iter().enumerate() {
        println!("  {}: {}", i, pkg.borrow().package.borrow().name());
    }

    println!("Circular dependency groups: {}", sorted_result.circular.len());
    for (i, group) in sorted_result.circular.iter().enumerate() {
        let names: Vec<String> =
            group.iter().map(|pkg| pkg.borrow().package.borrow().name().to_string()).collect();
        println!("  Group {i}: {names:?}");
    }

    // Check if we have circular dependencies
    if sorted_result.circular.is_empty() {
        // If no circular dependencies, check the order in sorted packages
        let bar_index = sorted_result
            .sorted
            .iter()
            .position(|p| p.borrow().package.borrow().name() == "@scope/package-bar")
            .expect("package-bar not found");

        let baz_index = sorted_result
            .sorted
            .iter()
            .position(|p| p.borrow().package.borrow().name() == "@scope/package-baz")
            .expect("package-baz not found");

        // Check that baz comes before bar
        assert!(
            baz_index < bar_index,
            "package-baz should come before package-bar since bar depends on baz"
        );
    } else {
        // Find the circular group containing bar and baz
        for group in &sorted_result.circular {
            let contains_bar = group
                .iter()
                .any(|pkg| pkg.borrow().package.borrow().name() == "@scope/package-bar");

            let contains_baz = group
                .iter()
                .any(|pkg| pkg.borrow().package.borrow().name() == "@scope/package-baz");

            // If both are in the same circular group, the test passes
            if contains_bar && contains_baz {
                println!(
                    "Package-bar and package-baz are correctly identified as circular dependencies"
                );
                return; // Test passes
            }
        }

        panic!("package-bar and package-baz should be in the same circular dependency group");
    }
}

#[rstest]
fn test_dependencies_of(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let workspace = create_workspace(&temp_dir);

    // Test dependencies of package-foo (depends on package-bar)
    let foo_deps = workspace.dependencies_of("@scope/package-foo");
    assert_eq!(foo_deps.len(), 1);
    assert_eq!(foo_deps[0].borrow().package.borrow().name(), "@scope/package-bar");

    // Test dependencies of package-bar (depends on package-baz)
    let bar_deps = workspace.dependencies_of("@scope/package-bar");
    assert_eq!(bar_deps.len(), 1);
    assert_eq!(bar_deps[0].borrow().package.borrow().name(), "@scope/package-baz");

    // Test dependencies of a leaf package (package-baz has no internal dependencies)
    let baz_deps = workspace.dependencies_of("@scope/package-baz");
    assert_eq!(baz_deps.len(), 1); // External dependency on @scope/package-bar
}

#[rstest]
#[allow(clippy::print_stdout)]
fn test_dependents_of(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let workspace = create_workspace(&temp_dir);

    // Check if the dependency graph has cycles
    let analysis = workspace.analyze_dependencies().expect("Failed to analyze dependencies");

    if analysis.cycles_detected {
        // If cycles are detected, test with check_circular=false
        println!("Cycles detected, using check_circular=false");
    }

    // Test dependents of package-foo (charlie depends on foo)
    // Using the new parameter to bypass cycle detection
    let foo_deps = workspace.dependents_of("@scope/package-foo", Some(false));
    assert_eq!(foo_deps.len(), 1, "package-foo should have 1 dependent (package-charlie)");

    if !foo_deps.is_empty() {
        assert_eq!(foo_deps[0].borrow().package.borrow().name(), "@scope/package-charlie");
    }

    // Test dependents of package-bar (foo depends on bar)
    let bar_deps = workspace.dependents_of("@scope/package-bar", Some(false));

    assert_eq!(
        bar_deps.len(),
        3,
        "package-bar should have 3 dependents (package-foo, package-tom and package-baz)"
    );

    let dependent_names: Vec<String> =
        bar_deps.iter().map(|pkg| pkg.borrow().package.borrow().name().to_string()).collect();

    assert!(dependent_names.contains(&"@scope/package-foo".to_string()));
    assert!(dependent_names.contains(&"@scope/package-tom".to_string()));
    assert!(dependent_names.contains(&"@scope/package-baz".to_string()));
}

#[rstest]
#[allow(clippy::print_stdout)]
fn test_affected_packages(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let workspace = create_workspace(&temp_dir);

    // Check if the dependency graph has cycles
    let analysis = workspace.analyze_dependencies().expect("Failed to analyze dependencies");

    if analysis.cycles_detected {
        println!("Cycles detected, using check_circular=false");
    }

    // If package-bar changes, it should affect package-foo and package-charlie (transitive)
    // Using the new parameter to bypass cycle detection
    let affected = workspace.affected_packages(&["@scope/package-bar"], Some(false));

    let affected_names: Vec<String> =
        affected.iter().map(|pkg| pkg.borrow().package.borrow().name().to_string()).collect();

    assert!(affected_names.contains(&"@scope/package-foo".to_string()));
    assert!(affected_names.contains(&"@scope/package-charlie".to_string()));

    // Multiple changed packages
    let affected_multi =
        workspace.affected_packages(&["@scope/package-baz", "@scope/package-tom"], Some(false));
    let affected_multi_names: Vec<String> =
        affected_multi.iter().map(|pkg| pkg.borrow().package.borrow().name().to_string()).collect();

    assert!(affected_multi_names.contains(&"@scope/package-bar".to_string()));
    assert!(affected_multi_names.contains(&"@scope/package-foo".to_string()));
    assert!(affected_multi_names.contains(&"@scope/package-charlie".to_string()));
}

#[rstest]
fn test_is_empty(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let workspace = create_workspace(&temp_dir);
    assert!(!workspace.is_empty());

    // Create an empty workspace manually for testing
    let workspace_config =
        sublime_monorepo_tools::WorkspaceConfig::new(temp_dir.path().to_path_buf());

    // Try opening a git repo in the temp dir
    let git_repo = sublime_git_tools::Repo::open(temp_dir.path().to_str().unwrap()).ok();

    let empty_workspace = sublime_monorepo_tools::Workspace::new(
        temp_dir.path().to_path_buf(),
        workspace_config,
        git_repo,
    )
    .expect("Failed to create empty workspace");

    assert!(empty_workspace.is_empty());
}

// Add specific tests for cyclic dependencies
#[rstest]
fn test_dependents_with_cycles(#[from(fixtures::cycle_monorepo)] temp_dir: TempDir) {
    let workspace = create_workspace(&temp_dir);

    // Verify cycles are detected
    let analysis = workspace.analyze_dependencies().expect("Failed to analyze dependencies");
    assert!(analysis.cycles_detected, "cycle_monorepo fixture should have cycles");

    // Test with default behavior (should return empty due to cycles)
    let foo_deps_with_check = workspace.dependents_of("@scope/package-foo", None);
    assert_eq!(
        foo_deps_with_check.len(),
        0,
        "With cycle checking enabled, dependents_of should return empty vector"
    );

    // Test with cycle checking disabled
    let foo_deps_no_check = workspace.dependents_of("@scope/package-foo", Some(false));
    assert!(
        !foo_deps_no_check.is_empty(),
        "With cycle checking disabled, dependents_of should return results"
    );

    // Verify we get the expected dependents when ignoring cycles
    let bar_deps = workspace.dependents_of("@scope/package-bar", Some(false));
    assert!(
        bar_deps.iter().any(|pkg| pkg.borrow().package.borrow().name() == "@scope/package-foo"),
        "package-foo should be a dependent of package-bar"
    );

    let baz_deps = workspace.dependents_of("@scope/package-baz", Some(false));
    assert!(
        baz_deps.iter().any(|pkg| pkg.borrow().package.borrow().name() == "@scope/package-bar"),
        "package-bar should be a dependent of package-baz"
    );
}

#[rstest]
fn test_affected_packages_with_cycles(#[from(fixtures::cycle_monorepo)] temp_dir: TempDir) {
    let workspace = create_workspace(&temp_dir);

    // Verify cycles are detected
    let analysis = workspace.analyze_dependencies().expect("Failed to analyze dependencies");
    assert!(analysis.cycles_detected, "cycle_monorepo fixture should have cycles");

    // Test with default behavior (should return empty due to cycles)
    let affected_with_check = workspace.affected_packages(&["@scope/package-foo"], None);
    assert_eq!(
        affected_with_check.len(),
        0,
        "With cycle checking enabled, affected_packages should return empty vector"
    );

    // Test with cycle checking disabled
    let affected_no_check = workspace.affected_packages(&["@scope/package-foo"], Some(false));
    assert!(
        !affected_no_check.is_empty(),
        "With cycle checking disabled, affected_packages should return results"
    );

    // Verify that in a cyclic dependency (foo → bar → baz → foo), changing any package affects all others
    let affected_foo = workspace.affected_packages(&["@scope/package-foo"], Some(false));
    let affected_foo_names: Vec<String> =
        affected_foo.iter().map(|pkg| pkg.borrow().package.borrow().name().to_string()).collect();

    // In a cycle, changing foo should affect bar and baz
    assert!(affected_foo_names.contains(&"@scope/package-bar".to_string()));
    assert!(affected_foo_names.contains(&"@scope/package-baz".to_string()));
}
