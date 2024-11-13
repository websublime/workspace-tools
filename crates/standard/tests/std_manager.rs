#[cfg(test)]
mod manager_tests {
    use std::{
        env::temp_dir,
        fs::{create_dir, remove_dir_all, set_permissions, File},
        io::Write,
        path::PathBuf,
    };

    #[cfg(not(windows))]
    use std::os::unix::fs::PermissionsExt;

    use ws_std::manager::{detect_package_manager, CorePackageManager};

    fn create_workspace(manager_file: &str) -> Result<PathBuf, std::io::Error> {
        let temp_dir = temp_dir();
        let monorepo_root_dir = temp_dir.join("monorepo-workspace");

        if monorepo_root_dir.exists() {
            remove_dir_all(&monorepo_root_dir)?;
        }

        create_dir(&monorepo_root_dir)?;

        let mut readme_file = File::create(monorepo_root_dir.join(manager_file).as_path())?;
        readme_file.write_all(b"{}")?;

        #[cfg(not(windows))]
        set_permissions(&monorepo_root_dir, std::fs::Permissions::from_mode(0o777))?;

        Ok(monorepo_root_dir)
    }

    #[test]
    fn test_npm_manager() -> Result<(), std::io::Error> {
        let root = &create_workspace("package-lock.json")?;

        let core_manager = detect_package_manager(root.as_path());
        let manager = core_manager.unwrap();

        assert_eq!(manager, CorePackageManager::Npm);

        remove_dir_all(root)?;

        Ok(())
    }

    #[test]
    fn test_pnpm_manager() -> Result<(), std::io::Error> {
        let root = &create_workspace("pnpm-lock.yaml")?;

        let core_manager = detect_package_manager(root.as_path());
        let manager = core_manager.unwrap();

        assert_eq!(manager, CorePackageManager::Pnpm);

        remove_dir_all(root)?;

        Ok(())
    }

    #[test]
    fn test_yarn_manager() -> Result<(), std::io::Error> {
        let root = &create_workspace("yarn.lock")?;

        let core_manager = detect_package_manager(root.as_path());
        let manager = core_manager.unwrap();

        assert_eq!(manager, CorePackageManager::Yarn);

        remove_dir_all(root)?;

        Ok(())
    }

    #[test]
    fn test_bun_manager() -> Result<(), std::io::Error> {
        let root = &create_workspace("bun.lockb")?;

        let core_manager = detect_package_manager(root.as_path());
        let manager = core_manager.unwrap();

        assert_eq!(manager, CorePackageManager::Bun);

        remove_dir_all(root)?;

        Ok(())
    }

    #[test]
    fn test_from_manager() {
        let npm_manager = CorePackageManager::from("npm".to_string());
        let yarn_manager = CorePackageManager::from("yarn".to_string());
        let pnpm_manager = CorePackageManager::from("pnpm".to_string());
        let bun_manager = CorePackageManager::from("bun".to_string());

        assert_eq!(npm_manager, CorePackageManager::Npm);
        assert_eq!(yarn_manager, CorePackageManager::Yarn);
        assert_eq!(pnpm_manager, CorePackageManager::Pnpm);
        assert_eq!(bun_manager, CorePackageManager::Bun);
    }

    #[test]
    #[should_panic(expected = "Unable to identify package manager: unknown")]
    fn test_unknown_manager() {
        let _ = CorePackageManager::from("unknown".to_string());
    }
}
