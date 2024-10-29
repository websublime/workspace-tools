#[cfg(test)]
mod repo_tests {
    #[cfg(not(windows))]
    use std::fs::set_permissions;
    #[cfg(not(windows))]
    use std::os::unix::fs::PermissionsExt;
    use std::{
        env::temp_dir,
        fs::{canonicalize, create_dir, remove_dir_all},
        path::PathBuf,
    };
    use ws_git::{error::RepositoryError, repo::Repository};
    use ws_std::command::execute;

    fn create_monorepo() -> Result<PathBuf, std::io::Error> {
        let temp_dir = temp_dir();
        let monorepo_root_dir = temp_dir.join("monorepo-workspace");

        if monorepo_root_dir.exists() {
            remove_dir_all(&monorepo_root_dir)?;
        }

        create_dir(&monorepo_root_dir)?;

        #[cfg(not(windows))]
        set_permissions(&monorepo_root_dir, std::fs::Permissions::from_mode(0o777))?;

        Ok(monorepo_root_dir)
    }

    #[test]
    fn test_repo_path() -> Result<(), std::io::Error> {
        let monorepo_root_dir = create_monorepo()?;
        let root_dir = canonicalize(monorepo_root_dir.as_os_str())?;

        let repo = Repository::new(monorepo_root_dir.as_path());

        assert_eq!(repo.get_repo_path(), root_dir.as_path());

        remove_dir_all(&root_dir)?;

        Ok(())
    }

    #[test]
    fn test_init_repo() -> Result<(), RepositoryError> {
        let monorepo_root_dir = create_monorepo()?;

        let repo = Repository::new(monorepo_root_dir.as_path());
        let inited = repo.init("main", "Sublime Machine", "machine@websublime.dev")?;

        assert!(inited);

        remove_dir_all(&monorepo_root_dir)?;

        Ok(())
    }

    #[test]
    fn test_config_repo() -> Result<(), RepositoryError> {
        let monorepo_root_dir = create_monorepo()?;

        let repo = Repository::new(monorepo_root_dir.as_path());
        let inited = repo.init("main", "Sublime Machine", "machine@websublime.dev")?;

        let message = execute("git", repo.get_repo_path(), ["config", "--list"], |message, _| {
            Ok(message.to_string())
        })?;

        let has_username = message.contains("user.name=Sublime Machine");
        let has_email = message.contains("user.email=machine@websublime.dev");

        assert!(inited);
        assert!(has_username);
        assert!(has_email);

        remove_dir_all(&monorepo_root_dir)?;

        Ok(())
    }
}
