mod test_utils;
use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceManager};
use test_utils::TestWorkspace;

#[cfg(test)]
mod workspace_package_relationships_tests {
    use super::*;

    #[test]
    fn test_affected_packages() {
        // Create a test workspace with a monorepo
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo();

        // Create workspace manager and discover workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();

        let workspace = manager.discover_workspace(test_workspace.path(), &options).unwrap();

        // Get affected packages if pkg-c changes
        let affected_by_c = workspace.affected_packages(&["pkg-c"]);

        // pkg-c, pkg-a, pkg-b, and web-app should all be affected
        assert_eq!(affected_by_c.len(), 4);
        assert!(affected_by_c.iter().any(|p| p.borrow().package.borrow().name() == "pkg-c"));
        assert!(affected_by_c.iter().any(|p| p.borrow().package.borrow().name() == "pkg-a"));
        assert!(affected_by_c.iter().any(|p| p.borrow().package.borrow().name() == "pkg-b"));
        assert!(affected_by_c.iter().any(|p| p.borrow().package.borrow().name() == "web-app"));

        // Get affected packages if pkg-a changes
        let affected_by_a = workspace.affected_packages(&["pkg-a"]);

        // pkg-a, pkg-b, and web-app should be affected, but not pkg-c
        assert_eq!(affected_by_a.len(), 3);
        assert!(affected_by_a.iter().any(|p| p.borrow().package.borrow().name() == "pkg-a"));
        assert!(affected_by_a.iter().any(|p| p.borrow().package.borrow().name() == "pkg-b"));
        assert!(affected_by_a.iter().any(|p| p.borrow().package.borrow().name() == "web-app"));
        assert!(!affected_by_a.iter().any(|p| p.borrow().package.borrow().name() == "pkg-c"));
    }

    #[test]
    fn test_dependents_of() {
        // Create a test workspace with a monorepo
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo();

        // Create workspace manager and discover workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();

        let workspace = manager.discover_workspace(test_workspace.path(), &options).unwrap();

        // Get dependents of pkg-a
        let dependents_of_a = workspace.dependents_of("pkg-a");

        // pkg-b and web-app should depend on pkg-a
        assert_eq!(dependents_of_a.len(), 2);
        assert!(dependents_of_a.iter().any(|p| p.borrow().package.borrow().name() == "pkg-b"));
        assert!(dependents_of_a.iter().any(|p| p.borrow().package.borrow().name() == "web-app"));

        // Get dependents of pkg-c
        let dependents_of_c = workspace.dependents_of("pkg-c");

        // Only pkg-a should depend on pkg-c
        assert_eq!(dependents_of_c.len(), 1);
        assert!(dependents_of_c.iter().any(|p| p.borrow().package.borrow().name() == "pkg-a"));

        // Get dependents of web-app
        let dependents_of_web = workspace.dependents_of("web-app");

        // No package should depend on web-app
        assert_eq!(dependents_of_web.len(), 0);
    }

    #[test]
    fn test_dependencies_of() {
        // Create a test workspace with a monorepo
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo();

        // Create workspace manager and discover workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();

        let workspace = manager.discover_workspace(test_workspace.path(), &options).unwrap();

        // Get dependencies of web-app
        let deps_of_web = workspace.dependencies_of("web-app");

        // web-app should depend on pkg-a and pkg-b
        assert_eq!(deps_of_web.len(), 2);
        assert!(deps_of_web.iter().any(|p| p.borrow().package.borrow().name() == "pkg-a"));
        assert!(deps_of_web.iter().any(|p| p.borrow().package.borrow().name() == "pkg-b"));

        // Get dependencies of pkg-a
        let deps_of_a = workspace.dependencies_of("pkg-a");

        // pkg-a should depend on pkg-c
        assert_eq!(deps_of_a.len(), 1);
        assert!(deps_of_a.iter().any(|p| p.borrow().package.borrow().name() == "pkg-c"));

        // Get dependencies of pkg-c
        let deps_of_c = workspace.dependencies_of("pkg-c");

        // pkg-c should have no dependencies
        assert_eq!(deps_of_c.len(), 0);
    }
}
