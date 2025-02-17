#[cfg(test)]
mod workspace_tests {

    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    use ws_git::repo::Repository;
    use ws_monorepo::changes::Change;
    use ws_monorepo::test::MonorepoWorkspace;
    use ws_monorepo::workspace::Workspace;
    use ws_pkg::bump::BumpOptions;
    use ws_pkg::version::Version;
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
    #[cfg_attr(target_os = "windows", ignore)]
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
    #[cfg_attr(target_os = "windows", ignore)]
    fn test_get_changed_packages_for_branch() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        let js_path = root.join("packages/package-foo/main.mjs");

        monorepo.create_workspace(CorePackageManager::Npm)?;

        let workspace = Workspace::new(root.clone());
        let repo = Repository::new(root.as_path());

        repo.create_branch("feat/message").expect("Failed to create branch");

        let mut js_file = File::create(js_path.as_path()).expect("Failed to create main.js file");
        js_file.write_all(r#"export const message = "hello";"#.as_bytes())?;

        repo.add_all().expect("Failed to add files");
        repo.commit("feat: message to the world", None, None).expect("Failed to commit");

        let (packages, changed_files) = workspace.get_changed_packages(None);
        let package = packages.as_slice().first().expect("No packages found");

        assert_eq!(packages.len(), 1);
        assert_eq!(package.package.name, "@scope/package-foo");
        assert_eq!(changed_files.len(), 1);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    #[cfg_attr(target_os = "windows", ignore)]
    fn test_get_changed_packages_for_main() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        let js_path = root.join("packages/package-foo/main.mjs");

        monorepo.create_workspace(CorePackageManager::Npm)?;

        let workspace = Workspace::new(root.clone());
        let repo = Repository::new(root.as_path());

        repo.create_branch("feat/message").expect("Failed to create branch");

        let mut js_file = File::create(js_path.as_path()).expect("Failed to create main.js file");
        js_file.write_all(r#"export const message = "hello";"#.as_bytes())?;

        repo.add_all().expect("Failed to add files");
        repo.commit("feat: message to the world", None, None).expect("Failed to commit");

        repo.checkout("main").expect("Error checking out main branch");
        repo.merge("feat/message").expect("Error merging branch");

        let (packages, changed_files) = workspace.get_changed_packages(Some("main".to_string()));
        let package = packages.as_slice().first().expect("No packages found");

        assert_eq!(packages.len(), 1);
        assert_eq!(package.package.name, "@scope/package-foo");
        assert_eq!(changed_files.len(), 1);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    #[cfg_attr(target_os = "windows", ignore)]
    fn test_get_package_recommend_bump_for_branch() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        let js_path = root.join("packages/package-foo/main.mjs");

        monorepo.create_workspace(CorePackageManager::Npm)?;

        let workspace = Workspace::new(root.clone());
        let repo = Repository::new(root.as_path());

        repo.create_branch("feat/message").expect("Failed to create branch");

        let mut js_file = File::create(js_path.as_path()).expect("Failed to create main.js file");
        js_file.write_all(r#"export const message = "hello";"#.as_bytes())?;

        let change =
            &Change { package: "@scope/package-foo".to_string(), release_as: "patch".to_string() };
        workspace.changes.add(change, Some(vec!["production".to_string()]));

        repo.add_all().expect("Failed to add files");
        repo.commit("feat: message to the world", None, None).expect("Failed to commit");
        let commit_id = repo.get_current_sha().expect("Failed to get current sha");

        let (packages, _) = workspace.get_changed_packages(None);

        let recommended_bump = workspace.get_package_recommend_bump(
            &packages[0],
            Some(BumpOptions {
                sync_deps: Some(false),
                since: None,
                release_as: Some(Version::Snapshot),
                fetch_all: Some(false),
                fetch_tags: Some(false),
                push: Some(false),
            }),
        );

        let snapshot_version = format!("1.0.0-alpha.{commit_id}");

        assert_eq!(recommended_bump.from, "1.0.0");
        assert_eq!(recommended_bump.to, snapshot_version);
        assert_eq!(recommended_bump.changed_files.len(), 1);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    #[cfg_attr(target_os = "windows", ignore)]
    fn test_get_package_recommend_bump_for_main() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        let js_path = root.join("packages/package-foo/main.mjs");

        monorepo.create_workspace(CorePackageManager::Npm)?;

        let workspace = Workspace::new(root.clone());
        let repo = Repository::new(root.as_path());

        repo.create_branch("feat/message").expect("Failed to create branch");

        let mut js_file = File::create(js_path.as_path()).expect("Failed to create main.js file");
        js_file.write_all(r#"export const message = "hello";"#.as_bytes())?;

        let change =
            &Change { package: "@scope/package-foo".to_string(), release_as: "patch".to_string() };
        workspace.changes.add(change, Some(vec!["production".to_string()]));

        repo.add_all().expect("Failed to add files");
        repo.commit("feat: message to the world", None, None).expect("Failed to commit");

        repo.checkout("main").expect("Error checking out main branch");
        repo.merge("feat/message").expect("Error merging branch");

        let (packages, _) = workspace.get_changed_packages(Some("main".to_string()));

        let recommended_bump = workspace.get_package_recommend_bump(
            &packages[0],
            Some(BumpOptions {
                sync_deps: Some(false),
                since: Some("main".to_string()),
                release_as: Some(Version::Major),
                fetch_all: Some(false),
                fetch_tags: Some(false),
                push: Some(false),
            }),
        );

        assert_eq!(recommended_bump.from, "1.0.0");
        assert_eq!(recommended_bump.to, "2.0.0");
        assert_eq!(recommended_bump.changed_files.len(), 1);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    #[cfg_attr(target_os = "windows", ignore)]
    fn test_get_bumps_for_branch() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        let js_path = root.join("packages/package-foo/main.mjs");

        monorepo.create_workspace(CorePackageManager::Npm)?;

        let workspace = Workspace::new(root.clone());
        let repo = Repository::new(root.as_path());

        let _ = repo.create_branch("feat/message").expect("Failed to create branch");

        let mut js_file = File::create(js_path.as_path()).expect("Failed to create main.js file");
        js_file.write_all(r#"export const message = "hello";"#.as_bytes())?;

        let change =
            &Change { package: "@scope/package-foo".to_string(), release_as: "patch".to_string() };
        workspace.changes.add(change, Some(vec!["production".to_string()]));

        let _ = repo.add_all().expect("Failed to add files");
        let _ = repo.commit("feat: message to the world", None, None).expect("Failed to commit");

        let bumps = workspace.get_bumps(&BumpOptions {
            sync_deps: Some(true),
            since: None,
            release_as: None, //Some(Version::Major),
            fetch_all: Some(false),
            fetch_tags: Some(false),
            push: Some(false),
        });

        assert_eq!(bumps.len(), 2);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    #[cfg_attr(target_os = "windows", ignore)]
    fn test_get_bumps_for_main() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        let js_path = root.join("packages/package-foo/main.mjs");

        monorepo.create_workspace(CorePackageManager::Npm)?;

        let workspace = Workspace::new(root.clone());
        let repo = Repository::new(root.as_path());

        repo.create_branch("feat/message").expect("Failed to create branch");

        let mut js_file = File::create(js_path.as_path()).expect("Failed to create main.js file");
        js_file.write_all(r#"export const message = "hello";"#.as_bytes())?;

        let change =
            &Change { package: "@scope/package-foo".to_string(), release_as: "patch".to_string() };
        workspace.changes.add(change, Some(vec!["production".to_string()]));

        repo.add_all().expect("Failed to add files");
        repo.commit("feat: message to the world", None, None).expect("Failed to commit");

        repo.checkout("main").expect("Error checking out main branch");
        repo.merge("feat/message").expect("Error merging branch");

        let bumps = workspace.get_bumps(&BumpOptions {
            sync_deps: Some(true),
            since: Some("main".to_string()),
            release_as: Some(Version::Major),
            fetch_all: Some(false),
            fetch_tags: Some(false),
            push: Some(false),
        });

        assert_eq!(bumps.len(), 2);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    #[cfg_attr(target_os = "windows", ignore)]
    fn test_get_bumps_touching_dependents() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        let js_path_foo = root.join("packages/package-foo/main.mjs");
        let js_path_charlie = root.join("packages/package-charlie/main.mjs");

        monorepo.create_workspace(CorePackageManager::Npm)?;

        let workspace = Workspace::new(root.clone());
        let repo = Repository::new(root.as_path());

        repo.create_branch("feat/message").expect("Failed to create branch");

        let mut js_file_foo =
            File::create(js_path_foo.as_path()).expect("Failed to create main.js file");
        js_file_foo.write_all(r#"export const message = "hello";"#.as_bytes())?;

        let mut js_file_charlie =
            File::create(js_path_charlie.as_path()).expect("Failed to create main.js file");
        js_file_charlie.write_all(r#"export const message = "hello charlie world";"#.as_bytes())?;

        let change_foo =
            &Change { package: "@scope/package-foo".to_string(), release_as: "patch".to_string() };
        let change_charlie = &Change {
            package: "@scope/package-charlie".to_string(),
            release_as: "patch".to_string(),
        };

        workspace.changes.add(change_foo, Some(vec!["production".to_string()]));
        workspace.changes.add(change_charlie, Some(vec!["production".to_string()]));

        repo.add_all().expect("Failed to add files");
        repo.commit("feat: message to the world", None, None).expect("Failed to commit");

        //repo.checkout("main").expect("Error checking out main branch");
        //repo.merge("feat/message").expect("Error merging branch");

        let bumps = workspace.get_bumps(&BumpOptions {
            sync_deps: Some(true),
            since: None,
            release_as: None,
            fetch_all: Some(false),
            fetch_tags: Some(false),
            push: Some(false),
        });

        dbg!(bumps);

        //assert_eq!(bumps.len(), 2);

        monorepo.delete_repository();

        Ok(())
    }

    /*#[test]
    #[cfg_attr(target_os = "windows", ignore)]
    fn test_apply_bumps() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        let js_path = root.join("packages/package-foo/main.mjs");

        monorepo.create_workspace(CorePackageManager::Npm)?;

        let workspace = Workspace::new(root.clone());
        let repo = Repository::new(root.as_path());

        let _ = repo.create_branch("feat/message").expect("Failed to create branch");

        let mut js_file = File::create(js_path.as_path()).expect("Failed to create main.js file");
        js_file.write_all(r#"export const message = "hello";"#.as_bytes())?;

        let change =
            &Change { package: "@scope/package-foo".to_string(), release_as: "patch".to_string() };
        workspace.changes.add(change, Some(vec!["production".to_string()]));

        repo.add_all().expect("Failed to add files");
        repo.commit("feat: message to the world", None, None).expect("Failed to commit");

        repo.checkout("main").expect("Error checking out main branch");
        repo.merge("feat/message").expect("Error merging branch");

        let bumps = workspace.apply_bumps(&BumpOptions {
            sync_deps: Some(true),
            since: Some("main".to_string()),
            release_as: None,
            fetch_all: Some(false),
            fetch_tags: Some(false),
            push: Some(false),
        });

        dbg!(&bumps);

        /*let first = bumps.first().expect("Error getting first bump");

        let pkg_file_path =
            PathBuf::from(first.package_info.package_path.clone()).join("package.json");
        let pkg_file = File::open(pkg_file_path)?;
        let pkg_info: ws_pkg::package::PackageJson = serde_json::from_reader(pkg_file)?;
        let changelog_file_path =
            PathBuf::from(first.package_info.package_path.clone()).join("CHANGELOG.md");
        //let changelog_file = File::open(changelog_file_path.clone())?;
        let changelog_content = std::fs::read_to_string(changelog_file_path.clone())?;

        dbg!(pkg_info);
        dbg!(changelog_content);*/

        assert_eq!(bumps.len(), 2);

        monorepo.delete_repository();

        Ok(())
    }*/
}
