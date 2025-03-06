/*#[cfg(test)]
mod config_tests {

    use std::path::PathBuf;

    use ws_monorepo::{config::get_workspace_config, test::MonorepoWorkspace};
    use ws_std::manager::CorePackageManager;

    #[test]
    fn test_workspace_config() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(CorePackageManager::Pnpm)?;

        let config = get_workspace_config(Some(root.clone()));

        assert_eq!(config.package_manager, CorePackageManager::Pnpm);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_default_workspace_config() {
        let current_dir = PathBuf::from(".");
        let config = get_workspace_config(Some(current_dir.clone()));

        #[cfg(not(windows))]
        let canonic_path = &std::fs::canonicalize(current_dir.as_path()).unwrap();
        #[cfg(not(windows))]
        let root = canonic_path.as_path();
        #[cfg(windows)]
        let root = current_dir.as_path();

        assert_ne!(config.workspace_root.as_path(), root);
    }
}*/
