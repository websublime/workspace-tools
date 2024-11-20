#[cfg(test)]
mod changes_tests {

    use std::{fs::File, io::BufReader};
    use ws_monorepo::{
        changes::{Change, Changes, ChangesConfig},
        test::MonorepoWorkspace,
    };
    use ws_std::manager::CorePackageManager;

    #[test]
    fn test_init_changes() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;

        let changes = Changes::new(root.as_path());
        let changes_config = changes.init();

        assert_eq!(
            changes_config.message,
            Some("chore(release): |---| release new version".to_string())
        );

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_changes_file_not_exist() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;

        let changes = Changes::new(root.as_path());

        assert!(!changes.file_exist());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_changes_file_exist() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());

        assert!(changes.file_exist());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_add_new_change() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change = &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };

        changes.add(change, Some(vec!["production".to_string()]));

        let changes_path = root.join(".changes.json");
        let changes_file = File::open(changes_path.as_path())?;
        let changes_reader = BufReader::new(changes_file);
        let changes_config: ChangesConfig = serde_json::from_reader(changes_reader)?;
        let change_meta = changes_config.changes.get("main").expect("Failed to get main change");

        assert!(change_meta.deploy.contains(&"production".to_string()));
        assert_eq!(change_meta.pkgs.len(), 1);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_update_new_change_with_same_environment() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change_one =
            &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };
        let change_two =
            &Change { package: "@scope/foo".to_string(), release_as: "patch".to_string() };

        changes.add(change_one, Some(vec!["production".to_string()]));
        changes.add(change_two, Some(vec!["production".to_string()]));

        let changes_path = root.join(".changes.json");
        let changes_file = File::open(changes_path.as_path())?;
        let changes_reader = BufReader::new(changes_file);
        let changes_config: ChangesConfig = serde_json::from_reader(changes_reader)?;
        let change_meta = changes_config.changes.get("main").expect("Failed to get main change");

        assert!(change_meta.deploy.contains(&"production".to_string()));
        assert_eq!(change_meta.pkgs.len(), 2);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_update_new_change_with_diff_environment() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change_one =
            &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };
        let change_two =
            &Change { package: "@scope/foo".to_string(), release_as: "patch".to_string() };

        changes.add(change_one, Some(vec!["production".to_string()]));
        changes.add(change_two, Some(vec!["development".to_string()]));

        let changes_path = root.join(".changes.json");
        let changes_file = File::open(changes_path.as_path())?;
        let changes_reader = BufReader::new(changes_file);
        let changes_config: ChangesConfig = serde_json::from_reader(changes_reader)?;
        let change_meta = changes_config.changes.get("main").expect("Failed to get main change");

        assert!(change_meta.deploy.contains(&"production".to_string()));
        assert!(change_meta.deploy.contains(&"development".to_string()));
        assert_eq!(change_meta.pkgs.len(), 2);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_avoid_duplicate_new_change() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change_one =
            &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };
        let change_two =
            &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };

        changes.add(change_one, Some(vec!["production".to_string()]));
        changes.add(change_two, Some(vec!["development".to_string()]));

        let changes_path = root.join(".changes.json");
        let changes_file = File::open(changes_path.as_path())?;
        let changes_reader = BufReader::new(changes_file);
        let changes_config: ChangesConfig = serde_json::from_reader(changes_reader)?;
        let change_meta = changes_config.changes.get("main").expect("Failed to get main change");

        assert!(change_meta.deploy.contains(&"production".to_string()));
        assert_eq!(change_meta.pkgs.len(), 1);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_remove_change() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change = &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };

        changes.add(change, Some(vec!["production".to_string()]));
        changes.remove("main");

        let changes_path = root.join(".changes.json");
        let changes_file = File::open(changes_path.as_path())?;
        let changes_reader = BufReader::new(changes_file);
        let changes_config: ChangesConfig = serde_json::from_reader(changes_reader)?;

        assert!(changes_config.changes.is_empty());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_empty_changes() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());

        assert!(changes.changes().is_empty());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_current_changes() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change = &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };

        changes.add(change, Some(vec!["production".to_string()]));

        assert!(!changes.changes().is_empty());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_empty_changes_by_branch() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());

        assert!(changes.changes_by_branch("main").is_none());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_changes_by_branch() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change = &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };

        changes.add(change, Some(vec!["production".to_string()]));

        assert!(changes.changes_by_branch("main").is_some());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_empty_changes_by_package() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change = &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };

        changes.add(change, Some(vec!["production".to_string()]));

        assert!(changes.changes_by_package("@scope/foo", "main").is_none());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_get_changes_by_package() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change = &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };

        changes.add(change, Some(vec!["production".to_string()]));

        let change_by_package = &changes.changes_by_package("@scope/bar", "main");
        let package_change = change_by_package.as_ref().unwrap();

        assert!(change_by_package.is_some());
        assert_eq!(package_change.package, "@scope/bar".to_string());
        assert_eq!(package_change.release_as, "patch".to_string());

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_package_change_exist() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change_bar =
            &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };
        let change_foo =
            &Change { package: "@scope/foo".to_string(), release_as: "patch".to_string() };

        changes.add(change_bar, Some(vec!["production".to_string()]));
        changes.add(change_foo, Some(vec!["production".to_string()]));

        let package_change_exist = changes.exist("main", "@scope/bar");

        assert!(package_change_exist);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_package_change_not_exist() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;
        monorepo.create_changes()?;

        let changes = Changes::new(root.as_path());
        let change_bar =
            &Change { package: "@scope/bar".to_string(), release_as: "patch".to_string() };
        let change_foo =
            &Change { package: "@scope/foo".to_string(), release_as: "patch".to_string() };

        changes.add(change_bar, Some(vec!["production".to_string()]));
        changes.add(change_foo, Some(vec!["production".to_string()]));

        let package_change_exist = changes.exist("main", "@scope/baz");

        assert!(!package_change_exist);

        monorepo.delete_repository();

        Ok(())
    }
}
