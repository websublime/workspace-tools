#[cfg(test)]
mod workspace_tests {

    use std::fs::File;
    use std::io::Write;

    use ws_git::repo::Repository;
    use ws_monorepo::test::MonorepoWorkspace;
    use ws_monorepo::workspace::Workspace;
    use ws_std::manager::CorePackageManager;

    #[test]
    fn test_get_npm_packages() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_workspace(CorePackageManager::Npm)?;

        let workspace = Workspace::new(root);
        let packages = workspace.get_packages();

        assert_eq!(packages.len(), 6);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_yarn_packages() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_workspace(CorePackageManager::Yarn)?;

        let workspace = Workspace::new(root);
        let packages = workspace.get_packages();

        assert_eq!(packages.len(), 6);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_pnpm_packages() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_workspace(CorePackageManager::Pnpm)?;

        let workspace = Workspace::new(root);
        let packages = workspace.get_packages();

        assert_eq!(packages.len(), 6);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_changed_packages() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        let js_path = root.join("packages/package-foo/main.mjs");

        monorepo.create_workspace(CorePackageManager::Npm)?;

        let workspace = Workspace::new(root.clone());
        let repo = Repository::new(root.as_path());

        let _ = repo.create_branch("feat/message").expect("Failed to create branch");

        let mut js_file = File::create(js_path.as_path()).expect("Failed to create main.js file");
        js_file.write_all(r#"export const message = "hello";"#.as_bytes())?;

        let _ = repo.add_all().expect("Failed to add files");
        let _ = repo.commit("feat: message to the world", None, None).expect("Failed to commit");

        let packages = workspace.get_changed_packages(Some("main".to_string()));
        let package = packages.as_slice().first().expect("No packages found");

        assert_eq!(packages.len(), 1);
        assert_eq!(package.package.name, "@scope/package-foo");

        monorepo.delete_repository();

        Ok(())
    }
}
