//! # Monorepo Module Tests
//!
//! ## What
//! This file contains unit tests for the monorepo module functionality.
//!
//! ## How
//! Tests verify the behavior of MonorepoKind and MonorepoDescriptor,
//! covering normal operations and edge cases.
//!
//! ## Why
//! Comprehensive testing ensures that the monorepo detection and analysis
//! functions work correctly in all scenarios.

#[allow(clippy::unwrap_used)]
#[allow(clippy::get_unwrap)]
#[cfg(test)]
mod tests {
    use crate::monorepo::{MonorepoDescriptor, MonorepoKind, WorkspacePackage};
    use std::path::{Path, PathBuf};

    #[test]
    fn test_monorepo_kind_names() {
        assert_eq!(MonorepoKind::NpmWorkSpace.name(), "npm");
        assert_eq!(MonorepoKind::YarnWorkspaces.name(), "yarn");
        assert_eq!(MonorepoKind::PnpmWorkspaces.name(), "pnpm");
        assert_eq!(MonorepoKind::BunWorkspaces.name(), "bun");
        assert_eq!(MonorepoKind::DenoWorkspaces.name(), "deno");

        let custom = MonorepoKind::Custom {
            name: "turbo".to_string(),
            config_file: "turbo.json".to_string(),
        };
        assert_eq!(custom.name(), "turbo");
    }

    #[test]
    fn test_monorepo_kind_config_files() {
        assert_eq!(MonorepoKind::NpmWorkSpace.config_file(), "package.json");
        assert_eq!(MonorepoKind::YarnWorkspaces.config_file(), "package.json");
        assert_eq!(MonorepoKind::PnpmWorkspaces.config_file(), "pnpm-workspace.yaml");
        assert_eq!(MonorepoKind::BunWorkspaces.config_file(), "bunfig.toml");
        assert_eq!(MonorepoKind::DenoWorkspaces.config_file(), "deno.json");

        let custom =
            MonorepoKind::Custom { name: "nx".to_string(), config_file: "nx.json".to_string() };
        assert_eq!(custom.config_file(), "nx.json");
    }

    #[test]
    fn test_set_custom() {
        let npm = MonorepoKind::NpmWorkSpace;
        let custom = npm.set_custom("rush".to_string(), "rush.json".to_string());

        assert_eq!(custom.name(), "rush");
        assert_eq!(custom.config_file(), "rush.json");

        // Original should be unchanged
        assert_eq!(npm.name(), "npm");
    }

    #[test]
    fn test_monorepo_descriptor_creation() {
        let root = PathBuf::from("/fake/monorepo");
        let packages = vec![
            create_test_package("pkg-a", "1.0.0", "packages/a", &root, vec![], vec![]),
            create_test_package("pkg-b", "1.0.0", "packages/b", &root, vec!["pkg-a"], vec![]),
        ];

        let descriptor =
            MonorepoDescriptor::new(MonorepoKind::YarnWorkspaces, root.clone(), packages);

        assert_eq!(descriptor.kind().name(), "yarn");
        assert_eq!(descriptor.root(), root.as_path());
        assert_eq!(descriptor.packages().len(), 2);
    }

    #[test]
    fn test_get_package() {
        let root = PathBuf::from("/fake/monorepo");
        let packages = vec![
            create_test_package("pkg-a", "1.0.0", "packages/a", &root, vec![], vec![]),
            create_test_package("pkg-b", "1.0.0", "packages/b", &root, vec!["pkg-a"], vec![]),
        ];

        let descriptor = MonorepoDescriptor::new(MonorepoKind::YarnWorkspaces, root, packages);

        // Test existing package
        let pkg_a = descriptor.get_package("pkg-a");
        assert!(pkg_a.is_some());
        assert_eq!(pkg_a.unwrap().name, "pkg-a");

        // Test non-existent package
        let pkg_c = descriptor.get_package("pkg-c");
        assert!(pkg_c.is_none());
    }

    #[test]
    fn test_dependency_graph() {
        let root = PathBuf::from("/fake/monorepo");
        let packages = vec![
            create_test_package("pkg-a", "1.0.0", "packages/a", &root, vec![], vec![]),
            create_test_package("pkg-b", "1.0.0", "packages/b", &root, vec!["pkg-a"], vec![]),
            create_test_package(
                "pkg-c",
                "1.0.0",
                "packages/c",
                &root,
                vec!["pkg-a", "pkg-b"],
                vec![],
            ),
        ];

        let descriptor = MonorepoDescriptor::new(MonorepoKind::YarnWorkspaces, root, packages);

        let graph = descriptor.get_dependency_graph();

        // Check package A's dependents (B and C)
        let pkg_a_dependents = graph.get("pkg-a").unwrap();
        assert_eq!(pkg_a_dependents.len(), 2);
        assert!(pkg_a_dependents.iter().any(|pkg| pkg.name == "pkg-b"));
        assert!(pkg_a_dependents.iter().any(|pkg| pkg.name == "pkg-c"));

        // Check package B's dependents (C only)
        let pkg_b_dependents = graph.get("pkg-b").unwrap();
        assert_eq!(pkg_b_dependents.len(), 1);
        assert_eq!(pkg_b_dependents[0].name, "pkg-c");

        // Check package C's dependents (none)
        let pkg_c_dependents = graph.get("pkg-c").unwrap();
        assert_eq!(pkg_c_dependents.len(), 0);
    }

    #[test]
    fn test_find_dependencies_by_name() {
        let root = PathBuf::from("/fake/monorepo");
        let packages = vec![
            create_test_package("pkg-a", "1.0.0", "packages/a", &root, vec![], vec![]),
            create_test_package("pkg-b", "1.0.0", "packages/b", &root, vec!["pkg-a"], vec![]),
            create_test_package(
                "pkg-c",
                "1.0.0",
                "packages/c",
                &root,
                vec!["pkg-a"],
                vec!["pkg-b"],
            ),
        ];

        let descriptor = MonorepoDescriptor::new(MonorepoKind::YarnWorkspaces, root, packages);

        // Test dependencies of pkg-c (should include both pkg-a and pkg-b)
        let deps_c = descriptor.find_dependencies_by_name("pkg-c");
        assert_eq!(deps_c.len(), 2);
        assert!(deps_c.iter().any(|pkg| pkg.name == "pkg-a"));
        assert!(deps_c.iter().any(|pkg| pkg.name == "pkg-b"));

        // Test dependencies of pkg-b (should include only pkg-a)
        let deps_b = descriptor.find_dependencies_by_name("pkg-b");
        assert_eq!(deps_b.len(), 1);
        assert_eq!(deps_b[0].name, "pkg-a");

        // Test dependencies of pkg-a (should be empty)
        let deps_a = descriptor.find_dependencies_by_name("pkg-a");
        assert_eq!(deps_a.len(), 0);

        // Test non-existent package (should be empty)
        let deps_none = descriptor.find_dependencies_by_name("non-existent");
        assert_eq!(deps_none.len(), 0);
    }

    #[test]
    fn test_find_package_for_path() {
        let root = PathBuf::from("/fake/monorepo");
        let packages = vec![
            create_test_package("pkg-a", "1.0.0", "packages/a", &root, vec![], vec![]),
            create_test_package("pkg-b", "1.0.0", "packages/b", &root, vec![], vec![]),
        ];

        let descriptor =
            MonorepoDescriptor::new(MonorepoKind::YarnWorkspaces, root.clone(), packages);

        // Test absolute path in pkg-a
        let pkg_a_file = Path::new("/fake/monorepo/packages/a/src/index.js");
        let found_pkg = descriptor.find_package_for_path(pkg_a_file);
        assert!(found_pkg.is_some());
        assert_eq!(found_pkg.unwrap().name, "pkg-a");

        // Test relative path in pkg-b
        let pkg_b_file = Path::new("packages/b/src/component.js");
        let found_pkg = descriptor.find_package_for_path(pkg_b_file);
        assert!(found_pkg.is_some());
        assert_eq!(found_pkg.unwrap().name, "pkg-b");

        // Test path not in any package
        let outside_file = Path::new("/fake/monorepo/outside/file.js");
        let found_pkg = descriptor.find_package_for_path(outside_file);
        assert!(found_pkg.is_none());
    }

    // Helper function to create test packages
    fn create_test_package(
        name: &str,
        version: &str,
        location: &str,
        root: &Path,
        deps: Vec<&str>,
        dev_deps: Vec<&str>,
    ) -> WorkspacePackage {
        WorkspacePackage {
            name: name.to_string(),
            version: version.to_string(),
            location: PathBuf::from(location),
            absolute_path: root.join(location),
            workspace_dependencies: deps.into_iter().map(String::from).collect(),
            workspace_dev_dependencies: dev_deps.into_iter().map(String::from).collect(),
        }
    }
}
