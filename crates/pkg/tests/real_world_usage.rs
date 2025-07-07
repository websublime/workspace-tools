//! Comprehensive real-world test demonstrating all API functionality
//!
//! This test simulates a complex monorepo scenario with:
//! - Multiple packages with interdependencies
//! - External dependencies
//! - Version conflicts
//! - Circular dependencies
//! - Registry operations
//! - Dependency upgrades
//! - Graph validation and visualization
//! - Change tracking and diff generation

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::print_stdout)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unnecessary_wraps)]

use serde_json::{json, Value};
use sublime_package_tools::{
    build_dependency_graph_from_packages, generate_ascii, generate_dot, AvailableUpgrade,
    ChangeType, Dependency, DependencyGraph, DependencyRegistry, DotOptions, ExecutionMode,
    LocalRegistry, Package, PackageDiff, PackageInfo, RegistryAuth, RegistryManager, RegistryType,
    UpgradeConfig, UpgradeStatus, Upgrader, ValidationOptions, Version, VersionStability,
    VersionUpdateStrategy,
};

/// Comprehensive real-world monorepo simulation test
///
/// This test demonstrates all the major functionality of the package management API
/// through a realistic monorepo scenario with React applications, shared libraries,
/// and complex dependency relationships.
#[test]
fn test_comprehensive_monorepo_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting comprehensive monorepo workflow test");

    // === Phase 1: Setup Registry Manager ===
    println!("\n📦 Phase 1: Setting up registry manager");
    let mut registry_manager = setup_registry_manager()?;

    // === Phase 2: Create Package Dependencies ===
    println!("\n🔧 Phase 2: Creating package dependencies");
    let mut dependency_registry = DependencyRegistry::new();
    let packages = create_monorepo_packages(&mut dependency_registry)?;

    // === Phase 3: Build and Analyze Dependency Graph ===
    println!("\n🕸️  Phase 3: Building and analyzing dependency graph");
    let graph = build_dependency_graph_from_packages(&packages);
    analyze_dependency_graph(&graph)?;

    // === Phase 4: Validate Dependencies ===
    println!("\n✅ Phase 4: Validating dependencies");
    validate_dependencies(&graph)?;

    // === Phase 5: Detect and Resolve Conflicts ===
    println!("\n⚠️  Phase 5: Detecting and resolving conflicts");
    resolve_version_conflicts(&mut dependency_registry)?;

    // === Phase 6: Create Package Infos and Track Changes ===
    println!("\n📋 Phase 6: Creating package infos and tracking changes");
    create_package_infos(&packages)?;
    track_package_changes(&packages)?;

    // === Phase 7: Check for Upgrades ===
    println!("\n⬆️  Phase 7: Checking for available upgrades");
    check_available_upgrades(&mut registry_manager, &packages)?;

    // === Phase 8: Version Management ===
    println!("\n🏷️  Phase 8: Demonstrating version management");
    demonstrate_version_management()?;

    // === Phase 9: Visualize Dependencies ===
    println!("\n🎨 Phase 9: Visualizing dependency graph");
    visualize_dependency_graph(&graph)?;

    // === Phase 10: Advanced Scenarios ===
    println!("\n🚀 Phase 10: Testing advanced scenarios");
    test_advanced_scenarios(&mut dependency_registry)?;

    println!("\n🎉 Comprehensive test completed successfully!");
    Ok(())
}

/// Setup a comprehensive registry manager with multiple registries
fn setup_registry_manager() -> Result<RegistryManager, Box<dyn std::error::Error>> {
    let mut manager = RegistryManager::new();

    // Add GitHub packages registry
    manager.add_registry("https://npm.pkg.github.com", RegistryType::GitHub);

    // Add custom internal registry
    manager.add_registry(
        "https://npm.mycompany.com",
        RegistryType::Custom("MyCompany/1.0".to_string()),
    );

    // Associate scopes with registries
    manager.associate_scope("@mycompany", "https://npm.mycompany.com")?;
    manager.associate_scope("@github", "https://npm.pkg.github.com")?;

    // Add authentication
    let auth = RegistryAuth {
        token: "gho_mock_token_123".to_string(),
        token_type: "Bearer".to_string(),
        always: false,
    };
    manager.set_auth("https://npm.pkg.github.com", auth)?;

    println!("  ✓ Registry manager configured with {} registries", manager.registry_urls().len());
    println!("  ✓ Scopes configured: @mycompany, @github");

    Ok(manager)
}

/// Create a realistic monorepo with multiple packages and complex dependencies
fn create_monorepo_packages(
    registry: &mut DependencyRegistry,
) -> Result<Vec<Package>, Box<dyn std::error::Error>> {
    let mut packages = Vec::new();

    // 1. Shared UI Components Library
    let ui_lib = Package::new_with_registry(
        "@mycompany/ui-components",
        "2.1.0",
        Some(vec![
            ("react", "^17.0.2"),
            ("styled-components", "^5.3.0"),
            ("prop-types", "^15.8.1"),
        ]),
        registry,
    )?;
    packages.push(ui_lib);

    // 2. Shared Utilities Library
    let utils_lib = Package::new_with_registry(
        "@mycompany/utils",
        "1.4.2",
        Some(vec![("lodash", "^4.17.21"), ("date-fns", "^2.28.0"), ("uuid", "^8.3.2")]),
        registry,
    )?;
    packages.push(utils_lib);

    // 3. Core API Client Library (depends on utils)
    let api_client = Package::new_with_registry(
        "@mycompany/api-client",
        "3.0.1",
        Some(vec![
            ("@mycompany/utils", "^1.4.0"),
            ("axios", "^0.27.2"),
            ("jsonwebtoken", "^8.5.1"),
        ]),
        registry,
    )?;
    packages.push(api_client);

    // 4. Main Web Application (depends on all internal libs)
    let web_app = Package::new_with_registry(
        "web-application",
        "4.2.3",
        Some(vec![
            ("@mycompany/ui-components", "^2.0.0"),
            ("@mycompany/api-client", "^3.0.0"),
            ("@mycompany/utils", "^1.4.0"),
            ("react", "^17.0.2"),
            ("react-dom", "^17.0.2"),
            ("react-router-dom", "^6.3.0"),
            ("webpack", "^5.72.0"),
        ]),
        registry,
    )?;
    packages.push(web_app);

    // 5. Mobile Application (different React version - conflict!)
    let mobile_app = Package::new_with_registry(
        "mobile-application",
        "2.1.0",
        Some(vec![
            ("@mycompany/ui-components", "^2.1.0"),
            ("@mycompany/api-client", "^3.0.0"),
            ("react", "^18.0.0"), // Version conflict!
            ("react-native", "^0.69.0"),
        ]),
        registry,
    )?;
    packages.push(mobile_app);

    // 6. Development Tools Package
    let dev_tools = Package::new_with_registry(
        "@mycompany/dev-tools",
        "1.0.5",
        Some(vec![
            ("eslint", "^8.15.0"),
            ("prettier", "^2.6.2"),
            ("typescript", "^4.7.2"),
            ("jest", "^28.1.0"),
        ]),
        registry,
    )?;
    packages.push(dev_tools);

    // 7. Documentation Site (circular dependency with web-app)
    let docs_site = Package::new_with_registry(
        "documentation-site",
        "1.2.0",
        Some(vec![
            ("web-application", "^4.0.0"), // Circular dependency!
            ("@mycompany/ui-components", "^2.1.0"),
            ("gatsby", "^4.15.0"),
            ("react", "^17.0.2"),
        ]),
        registry,
    )?;
    packages.push(docs_site);

    // Create a circular dependency by making web-app depend on docs-site
    let web_app_with_circular = Package::new_with_registry(
        "web-application",
        "4.2.3",
        Some(vec![
            ("@mycompany/ui-components", "^2.0.0"),
            ("@mycompany/api-client", "^3.0.0"),
            ("@mycompany/utils", "^1.4.0"),
            ("documentation-site", "^1.0.0"), // Creates circular dependency
            ("react", "^17.0.2"),
            ("react-dom", "^17.0.2"),
            ("react-router-dom", "^6.3.0"),
            ("webpack", "^5.72.0"),
        ]),
        registry,
    )?;

    // Replace the original web-app with the circular one
    packages[3] = web_app_with_circular;

    println!("  ✓ Created {} packages in monorepo", packages.len());
    println!(
        "  ✓ Packages: UI Components, Utils, API Client, Web App, Mobile App, Dev Tools, Docs Site"
    );

    Ok(packages)
}

/// Analyze the dependency graph for various properties
fn analyze_dependency_graph(
    graph: &DependencyGraph<Package>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  📊 Analyzing dependency graph:");

    // Check if all dependencies are internally resolvable
    let internally_resolvable = graph.is_internally_resolvable();
    println!("    • Internally resolvable: {internally_resolvable}");

    // Count resolved vs unresolved dependencies
    let resolved_count = graph.resolved_dependencies().count();
    let unresolved_count = graph.unresolved_dependencies().count();
    println!("    • Resolved dependencies: {resolved_count}");
    println!("    • External dependencies: {unresolved_count}");

    // List external dependencies
    if unresolved_count > 0 {
        println!("    • External dependencies:");
        for dep in graph.unresolved_dependencies() {
            println!("      - {} {}", dep.name(), dep.version());
        }
    }

    // Check for cycles
    if graph.has_cycles() {
        println!("    • ⚠️  Circular dependencies detected:");
        for cycle in graph.get_cycle_strings() {
            println!("      - {}", cycle.join(" → "));
        }
    } else {
        println!("    • ✓ No circular dependencies");
    }

    // Check for version conflicts
    if let Some(conflicts) = graph.find_version_conflicts() {
        println!("    • ⚠️  Version conflicts detected:");
        for (package, versions) in conflicts {
            println!("      - {}: {}", package, versions.join(", "));
        }
    } else {
        println!("    • ✓ No version conflicts");
    }

    Ok(())
}

/// Validate dependencies with custom options
fn validate_dependencies(
    graph: &DependencyGraph<Package>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate with default options
    let basic_report = graph.validate_package_dependencies()?;
    println!("  📋 Basic validation report:");
    print_validation_report(&basic_report);

    // Validate with custom options (treat unresolved as external)
    let options = ValidationOptions::new()
        .treat_unresolved_as_external(true)
        .with_internal_packages(vec!["@mycompany/ui-components", "@mycompany/utils"]);

    let custom_report = graph.validate_with_options(&options)?;
    println!("  📋 Custom validation report (external deps allowed):");
    print_validation_report(&custom_report);

    Ok(())
}

/// Print a validation report
fn print_validation_report(report: &sublime_package_tools::ValidationReport) {
    if !report.has_issues() {
        println!("    ✓ No issues found");
        return;
    }

    if report.has_critical_issues() {
        println!("    ❌ Critical issues:");
        for issue in report.critical_issues() {
            println!("      - {}", issue.message());
        }
    }

    if report.has_warnings() {
        println!("    ⚠️  Warnings:");
        for warning in report.warnings() {
            println!("      - {}", warning.message());
        }
    }
}

/// Resolve version conflicts in the dependency registry
fn resolve_version_conflicts(
    registry: &mut DependencyRegistry,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔧 Resolving version conflicts:");

    // Get resolution result
    let resolution = registry.resolve_version_conflicts()?;

    if resolution.resolved_versions.is_empty() {
        println!("    ✓ No conflicts to resolve");
        return Ok(());
    }

    println!("    📦 Resolved versions:");
    for (package, version) in &resolution.resolved_versions {
        println!("      - {package}: {version}");
    }

    if !resolution.updates_required.is_empty() {
        println!("    🔄 Updates required:");
        for update in &resolution.updates_required {
            println!(
                "      - {}: {} → {}",
                update.dependency_name, update.current_version, update.new_version
            );
        }
    }

    // Apply the resolution
    registry.apply_resolution_result(&resolution)?;
    println!("    ✓ Resolution applied");

    Ok(())
}

/// Create PackageInfo objects and demonstrate their functionality
fn create_package_infos(
    packages: &[Package],
) -> Result<Vec<PackageInfo>, Box<dyn std::error::Error>> {
    println!("  📄 Creating PackageInfo objects:");

    let mut package_infos = Vec::new();

    for (i, package) in packages.iter().enumerate() {
        // Create mock package.json content
        let mut pkg_json = json!({
            "name": package.name(),
            "version": package.version_str(),
            "dependencies": {},
            "devDependencies": {}
        });

        // Add dependencies to JSON
        if let Some(deps) = pkg_json.get_mut("dependencies") {
            if let Some(deps_obj) = deps.as_object_mut() {
                for dep in package.dependencies() {
                    deps_obj
                        .insert(dep.name().to_string(), Value::String(dep.version().to_string()));
                }
            }
        }

        let package_info = PackageInfo::new(
            package.clone(),
            format!("/workspace/packages/package-{i}/package.json"),
            format!("/workspace/packages/package-{i}"),
            format!("packages/package-{i}"),
            pkg_json,
        );

        package_infos.push(package_info);
    }

    println!("    ✓ Created {} PackageInfo objects", package_infos.len());

    // Demonstrate updating a dependency version
    if !package_infos.is_empty() {
        let first_info = &package_infos[0];
        println!("    🔄 Testing dependency update:");
        println!("      - Before: {}", first_info.package.borrow().version_str());

        first_info.update_version("2.1.1")?;
        println!("      - After: {}", first_info.package.borrow().version_str());

        // Test dependency version update
        if !first_info.package.borrow().dependencies().is_empty() {
            let (dep_name, old_version) = {
                let package = first_info.package.borrow();
                let first_dep = &package.dependencies()[0];
                (first_dep.name().to_string(), first_dep.version().to_string())
            };

            first_info.update_dependency_version(&dep_name, "^99.0.0")?;

            let new_version = {
                let package = first_info.package.borrow();
                let first_dep = &package.dependencies()[0];
                first_dep.version().to_string()
            };

            println!("      - Dependency {dep_name} updated: {old_version} → {new_version}");
        }
    }

    Ok(package_infos)
}

/// Track changes between package versions
fn track_package_changes(packages: &[Package]) -> Result<(), Box<dyn std::error::Error>> {
    println!("  📊 Tracking package changes:");

    if packages.len() < 2 {
        println!("    ⚠️  Need at least 2 packages to demonstrate diff");
        return Ok(());
    }

    // Create a modified version of the first package
    let original = &packages[0];
    // Create modified version using registry approach
    let mut temp_registry = DependencyRegistry::new();
    let modified = Package::new_with_registry(
        original.name(),
        "2.2.0", // Bumped version
        Some(vec![
            // Add new dependency
            ("new-dependency", "^1.0.0"),
            // Keep one existing dependency but update version
            ("react", "^18.0.0"),
        ]),
        &mut temp_registry,
    )?;

    // Generate diff
    let diff = PackageDiff::between(original, &modified)?;
    println!("    📋 Package diff:");
    println!("      {diff}");

    // Demonstrate individual dependency changes
    println!("    🔍 Individual dependency changes:");
    for change in &diff.dependency_changes {
        match change.change_type {
            ChangeType::Added => println!(
                "      + Added: {} ({})",
                change.name,
                change.current_version.as_ref().unwrap_or(&"unknown".to_string())
            ),
            ChangeType::Removed => println!(
                "      - Removed: {} (was {})",
                change.name,
                change.previous_version.as_ref().unwrap_or(&"unknown".to_string())
            ),
            ChangeType::Updated => println!(
                "      ↑ Updated: {} {} → {} {}",
                change.name,
                change.previous_version.as_ref().unwrap_or(&"unknown".to_string()),
                change.current_version.as_ref().unwrap_or(&"unknown".to_string()),
                if change.breaking { "(BREAKING)" } else { "" }
            ),
            ChangeType::Unchanged => {
                // Skip unchanged for brevity
            }
        }
    }

    // Count changes by type
    let counts = diff.count_changes_by_type();
    println!("    📈 Change summary:");
    for (change_type, count) in counts {
        println!("      - {change_type}: {count}");
    }

    println!("    🚨 Breaking changes: {}", diff.count_breaking_changes());

    Ok(())
}

/// Check for available upgrades using the upgrader
fn check_available_upgrades(
    registry_manager: &mut RegistryManager,
    packages: &[Package],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔍 Checking for available upgrades:");

    // Create upgrader with conservative strategy
    let conservative_config = UpgradeConfig {
        update_strategy: VersionUpdateStrategy::MinorAndPatch,
        version_stability: VersionStability::StableOnly,
        execution_mode: ExecutionMode::DryRun,
        ..UpgradeConfig::default()
    };

    Upgrader::create(conservative_config, registry_manager.clone());

    // Mock some available versions in a local registry for testing
    let local_registry = setup_mock_local_registry()?;
    registry_manager
        .add_registry_instance("https://mock-registry.test", std::sync::Arc::new(local_registry));

    // Since we can't actually connect to npm registry in tests, we'll simulate upgrade checks
    simulate_upgrade_checks(packages)?;

    // Create aggressive upgrader
    let aggressive_config = UpgradeConfig {
        update_strategy: VersionUpdateStrategy::AllUpdates,
        version_stability: VersionStability::IncludePrerelease,
        execution_mode: ExecutionMode::DryRun,
        ..UpgradeConfig::default()
    };

    Upgrader::create(aggressive_config, registry_manager.clone());
    println!("    ✓ Created upgraders with different strategies");

    // Demonstrate upgrade report generation
    let mock_upgrades = create_mock_upgrades();
    let report = Upgrader::generate_upgrade_report(&mock_upgrades);
    println!("    📄 Sample upgrade report:\n{report}");

    Ok(())
}

/// Setup a mock local registry for testing
fn setup_mock_local_registry() -> Result<LocalRegistry, Box<dyn std::error::Error>> {
    let registry = LocalRegistry::default();
    // In a real scenario, this would be populated with package data
    println!("    ✓ Mock local registry created");
    Ok(registry)
}

/// Simulate upgrade checks (since we can't hit real registries in tests)
fn simulate_upgrade_checks(packages: &[Package]) -> Result<(), Box<dyn std::error::Error>> {
    println!("    🔍 Simulating upgrade checks:");

    for package in packages {
        println!("      📦 {}", package.name());

        for dep in package.dependencies() {
            let current_version = dep.version().to_string();

            // Simulate different upgrade scenarios
            let status = match dep.name() {
                "react" => UpgradeStatus::MajorAvailable("^18.0.0".to_string()),
                "lodash" => UpgradeStatus::PatchAvailable("^4.17.22".to_string()),
                "webpack" => UpgradeStatus::MinorAvailable("^5.73.0".to_string()),
                _ => UpgradeStatus::UpToDate,
            };

            if !matches!(status, UpgradeStatus::UpToDate) {
                println!("        - {}: {} ({})", dep.name(), current_version, status);
            }
        }
    }

    Ok(())
}

/// Create mock upgrades for demonstration
fn create_mock_upgrades() -> Vec<AvailableUpgrade> {
    vec![
        AvailableUpgrade {
            package_name: "web-application".to_string(),
            dependency_name: "react".to_string(),
            current_version: "^17.0.2".to_string(),
            compatible_version: Some("^17.0.2".to_string()),
            latest_version: Some("^18.2.0".to_string()),
            status: UpgradeStatus::Constrained("^18.2.0".to_string()),
        },
        AvailableUpgrade {
            package_name: "@mycompany/utils".to_string(),
            dependency_name: "lodash".to_string(),
            current_version: "^4.17.21".to_string(),
            compatible_version: Some("^4.17.22".to_string()),
            latest_version: Some("^4.17.22".to_string()),
            status: UpgradeStatus::PatchAvailable("^4.17.22".to_string()),
        },
    ]
}

/// Demonstrate version management functionality
fn demonstrate_version_management() -> Result<(), Box<dyn std::error::Error>> {
    println!("  🏷️  Demonstrating version management:");

    let current_version = "1.2.3";
    println!("    📦 Current version: {current_version}");

    // Test version bumping
    let major_bump = Version::bump_major(current_version)?;
    let minor_bump = Version::bump_minor(current_version)?;
    let patch_bump = Version::bump_patch(current_version)?;
    let snapshot = Version::bump_snapshot(current_version, "abc123")?;

    println!("    ⬆️  Version bumps:");
    println!("      - Major: {major_bump}");
    println!("      - Minor: {minor_bump}");
    println!("      - Patch: {patch_bump}");
    println!("      - Snapshot: {snapshot}");

    // Test version comparison
    println!("    🔍 Version comparisons:");
    let relationships = vec![
        ("1.0.0", "2.0.0"),
        ("1.0.0", "1.1.0"),
        ("1.0.0", "1.0.1"),
        ("1.0.0", "1.0.0"),
        ("2.0.0", "1.0.0"),
    ];

    for (v1, v2) in relationships {
        let relationship = Version::compare_versions(v1, v2);
        let is_breaking = Version::is_breaking_change(v1, v2);
        println!(
            "      - {} → {}: {} {}",
            v1,
            v2,
            relationship,
            if is_breaking { "(BREAKING)" } else { "" }
        );
    }

    Ok(())
}

#[allow(clippy::comparison_chain)]
/// Visualize the dependency graph in multiple formats
fn visualize_dependency_graph(
    graph: &DependencyGraph<Package>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🎨 Visualizing dependency graph:");

    // Generate ASCII representation
    let ascii = generate_ascii(graph)?;
    println!("    📊 ASCII representation:");
    // Print only first few lines to avoid cluttering test output
    for (i, line) in ascii.lines().enumerate() {
        if i < 10 {
            println!("      {line}");
        } else if i == 10 {
            println!("      ... (truncated)");
            break;
        }
    }

    // Generate DOT format
    let dot_options = DotOptions {
        title: "Monorepo Dependency Graph".to_string(),
        show_external: true,
        highlight_cycles: true,
    };

    let dot_output = generate_dot(graph, &dot_options)?;
    println!("    📄 DOT format generated ({} characters)", dot_output.len());

    // In a real scenario, you might save this to a file
    // save_dot_to_file(&dot_output, "dependency_graph.dot")?;

    // Show a sample of the DOT output
    let dot_lines: Vec<&str> = dot_output.lines().collect();
    if dot_lines.len() > 5 {
        println!("    📝 DOT sample:");
        for line in &dot_lines[0..5] {
            println!("      {line}");
        }
        println!("      ... (truncated)");
    }

    Ok(())
}

/// Test advanced scenarios and edge cases
fn test_advanced_scenarios(
    registry: &mut DependencyRegistry,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🚀 Testing advanced scenarios:");

    // Test workspace dependencies (should be rejected)
    println!("    🔒 Testing workspace dependency handling:");
    match Dependency::new("internal-package", "workspace:*") {
        Ok(_) => println!("      ❌ Workspace dependency was allowed (should be rejected)"),
        Err(_) => println!("      ✓ Workspace dependency correctly rejected"),
    }

    // Test invalid version handling
    println!("    ❓ Testing invalid version handling:");
    match Dependency::new("some-package", "not-a-version") {
        Ok(_) => println!("      ❌ Invalid version was allowed"),
        Err(_) => println!("      ✓ Invalid version correctly rejected"),
    }

    // Test dependency registry conflict resolution
    println!("    🔄 Testing registry conflict resolution:");
    let dep1 = registry.get_or_create("test-package", "^1.0.0")?;
    let dep2 = registry.get_or_create("test-package", "^1.2.0")?;

    // Both should reference the same dependency (with higher version)
    let same_dependency = dep1.name() == dep2.name();
    let version_updated = dep1.version().to_string() == "^1.2.0";

    println!("      - Same dependency: {same_dependency}");
    println!("      - Version updated to higher: {version_updated}");

    // Test complex version requirements
    println!("    📏 Testing complex version requirements:");
    let complex_deps = vec![
        ("range-dep", ">=1.0.0 <2.0.0"),
        ("exact-dep", "1.2.3"),
        ("tilde-dep", "~1.2.3"),
        ("caret-dep", "^1.2.3"),
    ];

    for (name, version) in complex_deps {
        match Dependency::new(name, version) {
            Ok(dep) => {
                let matches_higher = dep.matches("1.5.0").unwrap_or(false);
                let matches_lower = dep.matches("0.9.0").unwrap_or(false);
                println!(
                    "      - {name} ({version}): matches 1.5.0={matches_higher}, matches 0.9.0={matches_lower}",
                );
            }
            Err(e) => println!("      - {name} failed: {e}"),
        }
    }

    // Test resolution with no conflicts
    println!("    ✅ Testing clean resolution:");
    let clean_registry = DependencyRegistry::new();
    let clean_resolution = clean_registry.resolve_version_conflicts()?;
    println!(
        "      - Clean resolution: {} versions, {} updates",
        clean_resolution.resolved_versions.len(),
        clean_resolution.updates_required.len()
    );

    Ok(())
}
