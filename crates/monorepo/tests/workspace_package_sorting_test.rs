mod test_utils;
use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceManager};
use test_utils::TestWorkspace;

#[cfg(test)]
mod workspace_package_sorting_tests {
    use super::*;

    #[test]
    fn test_sorted_packages() {
        // Create a test workspace with a monorepo
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo();

        // Create workspace manager and discover workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();

        let workspace = manager.discover_workspace(test_workspace.path(), &options).unwrap();

        // Get packages in topological order
        let sorted_packages = workspace.sorted_packages();

        // Convert to package names for easier assertions
        let package_names: Vec<String> = sorted_packages
            .iter()
            .map(|p| p.borrow().package.borrow().name().to_string())
            .collect();

        // In a topological sort:
        // - pkg-c should come before pkg-a (since pkg-a depends on pkg-c)
        // - pkg-a should come before pkg-b (since pkg-b depends on pkg-a)
        // - pkg-a and pkg-b should come before web-app (since web-app depends on both)

        let idx_c = package_names.iter().position(|n| n == "pkg-c").unwrap();
        let idx_a = package_names.iter().position(|n| n == "pkg-a").unwrap();
        let idx_b = package_names.iter().position(|n| n == "pkg-b").unwrap();
        let idx_web = package_names.iter().position(|n| n == "web-app").unwrap();

        assert!(idx_c < idx_a, "pkg-c should come before pkg-a in topological sort");
        assert!(idx_a < idx_b, "pkg-a should come before pkg-b in topological sort");
        assert!(idx_a < idx_web, "pkg-a should come before web-app in topological sort");
        assert!(idx_b < idx_web, "pkg-b should come before web-app in topological sort");
    }

    #[test]
    fn test_sorted_packages_with_cycle() {
        // Create a workspace with circular dependencies
        let test_workspace = TestWorkspace::new();
        // This creates 4 package.json files (1 root + 3 packages)
        test_workspace.create_circular_deps_workspace();

        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();
        let workspace = manager.discover_workspace(test_workspace.path(), &options).unwrap();

        // Even with a cycle, we should still get all packages
        let sorted_packages = workspace.sorted_packages();
        assert_eq!(sorted_packages.len(), 3, "Expected all 3 packages despite cycle");

        // Just verify we have all packages (order is not guaranteed with cycles)
        let has_pkg_a =
            sorted_packages.iter().any(|p| p.borrow().package.borrow().name() == "pkg-a");
        let has_pkg_b =
            sorted_packages.iter().any(|p| p.borrow().package.borrow().name() == "pkg-b");
        let has_pkg_c =
            sorted_packages.iter().any(|p| p.borrow().package.borrow().name() == "pkg-c");

        assert!(has_pkg_a, "Missing pkg-a in result");
        assert!(has_pkg_b, "Missing pkg-b in result");
        assert!(has_pkg_c, "Missing pkg-c in result");
    }
}
