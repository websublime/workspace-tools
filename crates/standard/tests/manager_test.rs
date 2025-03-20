#[cfg(test)]
mod manager_tests {
    use std::{
        convert::TryFrom,
        env::temp_dir,
        fs::{create_dir, remove_dir_all, File},
        io::Write,
        path::PathBuf,
    };

    #[cfg(not(windows))]
    use std::os::unix::fs::PermissionsExt;

    #[cfg(not(windows))]
    use std::fs::set_permissions;

    use sublime_standard_tools::{
        detect_package_manager, CorePackageManager, CorePackageManagerError,
    };

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
    fn test_try_from_manager() {
        // Test valid package managers
        let npm_manager = CorePackageManager::try_from("npm").unwrap();
        let yarn_manager = CorePackageManager::try_from("yarn").unwrap();
        let pnpm_manager = CorePackageManager::try_from("pnpm").unwrap();
        let bun_manager = CorePackageManager::try_from("bun").unwrap();

        assert_eq!(npm_manager, CorePackageManager::Npm);
        assert_eq!(yarn_manager, CorePackageManager::Yarn);
        assert_eq!(pnpm_manager, CorePackageManager::Pnpm);
        assert_eq!(bun_manager, CorePackageManager::Bun);

        // Test with String type
        let npm_string = String::from("npm");
        let npm_from_string = CorePackageManager::try_from(npm_string).unwrap();
        assert_eq!(npm_from_string, CorePackageManager::Npm);
    }

    #[test]
    fn test_unknown_manager() {
        // Test with an unknown package manager
        let result = CorePackageManager::try_from("unknown");

        // Make sure it returns an error
        assert!(result.is_err());

        // Verify the error message contains the unknown manager name
        if let Err(CorePackageManagerError::ParsePackageManagerError(msg)) = result {
            assert_eq!(msg, "unknown");
        } else {
            panic!("Expected CorePackageManagerError::Parse");
        }
    }
}
