#[cfg(test)]
mod paths_tests {
    use std::{
        env::temp_dir,
        fs::{create_dir, remove_dir_all, File},
        io::Write,
        ops::Deref,
        path::PathBuf,
    };

    #[cfg(not(windows))]
    use std::os::unix::fs::PermissionsExt;

    #[cfg(not(windows))]
    use std::fs::set_permissions;

    use ws_std::paths::get_project_root_path;

    fn create_workspace() -> Result<PathBuf, std::io::Error> {
        let temp_dir = temp_dir();
        let monorepo_root_dir = temp_dir.join("monorepo-workspace");

        if monorepo_root_dir.exists() {
            remove_dir_all(&monorepo_root_dir)?;
        }

        create_dir(&monorepo_root_dir)?;

        let mut readme_file = File::create(monorepo_root_dir.join("package-lock.json").as_path())?;
        readme_file.write_all(b"{}")?;

        #[cfg(not(windows))]
        set_permissions(&monorepo_root_dir, std::fs::Permissions::from_mode(0o777))?;

        Ok(monorepo_root_dir)
    }

    #[test]
    fn test_project_root_path() -> Result<(), std::io::Error> {
        let root = &create_workspace()?;

        let result = get_project_root_path(Some(root.deref().to_path_buf()));

        remove_dir_all(root)?;

        let root = root.display().to_string();
        let project_root = result.unwrap().display().to_string();

        assert_eq!(root, project_root);

        Ok(())
    }
}
