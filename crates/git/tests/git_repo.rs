#[cfg(test)]
mod repo_tests {
    #[cfg(not(windows))]
    use std::fs::set_permissions;
    #[cfg(not(windows))]
    use std::os::unix::fs::PermissionsExt;
    use std::{
        env::temp_dir,
        fs::{canonicalize, create_dir, remove_dir_all, File},
        io::Write,
        path::PathBuf,
    };
    use ws_git::{error::RepositoryError, repo::Repository};
    use ws_std::command::execute;

    fn create_workspace() -> Result<PathBuf, std::io::Error> {
        let temp_dir = temp_dir();
        let monorepo_root_dir = temp_dir.join("monorepo-workspace");

        if monorepo_root_dir.exists() {
            remove_dir_all(&monorepo_root_dir)?;
        }

        create_dir(&monorepo_root_dir)?;

        let mut readme_file = File::create(monorepo_root_dir.join("README.md").as_path())?;
        readme_file.write_all(b"HELLO WORLD")?;

        #[cfg(not(windows))]
        set_permissions(&monorepo_root_dir, std::fs::Permissions::from_mode(0o777))?;

        Ok(monorepo_root_dir)
    }

    fn create_monorepo() -> Result<PathBuf, RepositoryError> {
        let monorepo_root_dir = create_workspace()?;
        let repo = Repository::new(monorepo_root_dir.as_path());
        repo.init("main", "Sublime Machine", "machine@websublime.dev")?;

        execute(
            "git",
            monorepo_root_dir.as_path(),
            ["remote", "add", "origin", monorepo_root_dir.to_str().unwrap()],
            |_, stdout| Ok(stdout.status.success()),
        )?;

        execute("git", monorepo_root_dir.as_path(), ["add", "."], |_, stdout| {
            Ok(stdout.status.success())
        })?;

        execute(
            "git",
            monorepo_root_dir.as_path(),
            ["commit", "-m", "chore: init project"],
            |_, stdout| Ok(stdout.status.success()),
        )?;

        Ok(monorepo_root_dir)
    }

    #[test]
    fn test_repo_path() -> Result<(), std::io::Error> {
        let monorepo_root_dir = create_workspace()?;
        let root_dir = canonicalize(monorepo_root_dir.as_os_str())?;

        let repo = Repository::new(monorepo_root_dir.as_path());

        assert_eq!(repo.get_repo_path(), root_dir.as_path());

        remove_dir_all(&root_dir)?;

        Ok(())
    }

    #[test]
    fn test_init_repo() -> Result<(), RepositoryError> {
        let monorepo_root_dir = create_workspace()?;

        let repo = Repository::new(monorepo_root_dir.as_path());
        let inited = repo.init("main", "Sublime Machine", "machine@websublime.dev")?;

        assert!(inited);

        remove_dir_all(&monorepo_root_dir)?;

        Ok(())
    }

    #[test]
    fn test_config_repo() -> Result<(), RepositoryError> {
        let monorepo_root_dir = create_workspace()?;

        let repo = Repository::new(monorepo_root_dir.as_path());
        let inited = repo.init("main", "Sublime Machine", "machine@websublime.dev")?;

        execute(
            "git",
            monorepo_root_dir.as_path(),
            ["remote", "add", "origin", monorepo_root_dir.to_str().unwrap()],
            |_, stdout| Ok(stdout.status.success()),
        )?;

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

    #[test]
    fn test_vcs_repo() -> Result<(), RepositoryError> {
        let monorepo_root_dir = create_workspace()?;

        let repo = Repository::new(monorepo_root_dir.as_path());
        repo.init("main", "Sublime Machine", "machine@websublime.dev")?;
        let is_vcs = repo.is_vcs()?;

        assert!(is_vcs);

        remove_dir_all(&monorepo_root_dir)?;

        Ok(())
    }

    #[test]
    fn test_create_branch_repo() -> Result<(), RepositoryError> {
        let monorepo_root_dir = create_workspace()?;

        let repo = Repository::new(monorepo_root_dir.as_path());
        repo.init("main", "Sublime Machine", "machine@websublime.dev")?;
        let branch_created = repo.create_branch("feature/awesome")?;

        assert!(branch_created);

        remove_dir_all(&monorepo_root_dir)?;

        Ok(())
    }

    #[test]
    fn test_checkout_branch_repo() -> Result<(), RepositoryError> {
        let monorepo_root_dir = create_monorepo()?;

        let repo = Repository::new(monorepo_root_dir.as_path());
        repo.create_branch("feature/awesome")?;
        let checkouted = repo.checkout("main")?;

        let branches = repo.list_branches()?;

        assert!(checkouted);
        assert!(branches.contains("main"));
        assert!(branches.contains("feature/awesome"));

        remove_dir_all(&monorepo_root_dir)?;

        Ok(())
    }

    #[test]
    fn test_list_branch_repo() -> Result<(), RepositoryError> {
        let monorepo_root_dir = create_monorepo()?;

        let repo = Repository::new(monorepo_root_dir.as_path());
        repo.create_branch("feature/awesome")?;

        let branches = repo.list_branches()?;

        assert!(branches.contains("main"));
        assert!(branches.contains("feature/awesome"));

        remove_dir_all(&monorepo_root_dir)?;

        Ok(())
    }

    #[test]
    fn test_log_repo() -> Result<(), RepositoryError> {
        let monorepo_root_dir = create_monorepo()?;

        let repo = Repository::new(monorepo_root_dir.as_path());
        repo.create_branch("feature/awesome")?;

        let mut main_file = File::create(monorepo_root_dir.join("main.mjs").as_path())?;
        main_file.write_all(b"const msg = 'Hello';")?;

        execute("git", monorepo_root_dir.as_path(), ["add", "."], |_, stdout| {
            Ok(stdout.status.success())
        })?;

        execute(
            "git",
            monorepo_root_dir.as_path(),
            ["commit", "-m", "chore: add main.js"],
            |_, stdout| Ok(stdout.status.success()),
        )?;

        let logs = repo.log()?;

        assert!(logs.contains("chore: add main.js"));

        remove_dir_all(&monorepo_root_dir)?;

        Ok(())
    }

    #[test]
    fn test_diff_repo() -> Result<(), RepositoryError> {
        let monorepo_root_dir = create_monorepo()?;

        let repo = Repository::new(monorepo_root_dir.as_path());
        repo.create_branch("feature/awesome")?;

        let main_file_path = monorepo_root_dir.join("main.mjs");
        let mut main_file = File::create(main_file_path.as_path())?;
        main_file.write_all(b"const msg = 'Hello';")?;

        execute("git", monorepo_root_dir.as_path(), ["add", "."], |_, stdout| {
            Ok(stdout.status.success())
        })?;

        execute(
            "git",
            monorepo_root_dir.as_path(),
            ["commit", "-m", "chore: add main.js"],
            |_, stdout| Ok(stdout.status.success()),
        )?;

        let diff_branch = repo.diff(Some(main_file_path.to_str().unwrap().to_string()))?;

        #[cfg(not(windows))]
        assert!(diff_branch.is_empty());

        #[cfg(windows)]
        assert!(!diff_branch.is_empty());

        remove_dir_all(&monorepo_root_dir)?;

        Ok(())
    }
}
