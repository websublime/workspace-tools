#[cfg(test)]
mod repo_tests {
    use std::{
        env::temp_dir,
        fs::{canonicalize, create_dir, remove_dir_all, File},
        io::Write,
        path::PathBuf,
    };
    use sublime_git_tools::{Repo, RepoError};
    use sublime_standard_tools::get_project_root_path;

    #[cfg(not(windows))]
    use std::os::unix::fs::PermissionsExt;

    #[cfg(not(windows))]
    use std::fs::set_permissions;

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

        let root = canonicalize(monorepo_root_dir.as_path()).expect("Failed to canonicalize path");

        Ok(root)
    }

    #[test]
    fn test_repo_open() {
        let current_dir = std::env::current_dir().unwrap();
        let project_root = get_project_root_path(Some(current_dir)).unwrap();

        let repo = Repo::open(project_root.display().to_string().as_str()).unwrap();

        assert_eq!(repo.get_repo_path().display().to_string(), project_root.display().to_string());
    }

    #[test]
    fn test_create_branch() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        let result = repo.create_branch("feat/my-new-feature");

        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_list_branches() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.create_branch("feat/my-new-feature")?;
        let branches = repo.list_branches()?;

        // Check if branches contain main and feat/my-new-feature
        assert!(branches.contains(&String::from("main")));
        assert!(branches.contains(&String::from("feat/my-new-feature")));

        Ok(())
    }

    #[test]
    fn test_list_config() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        let config = repo.list_config()?;

        assert!(!config.is_empty());

        Ok(())
    }

    #[test]
    fn test_checkout_branch() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.create_branch("feat/my-new-feature")?;
        repo.checkout("feat/my-new-feature")?;

        let current_branch = repo.get_current_branch()?;

        assert_eq!(current_branch, String::from("feat/my-new-feature"));

        Ok(())
    }

    #[test]
    fn test_get_current_branch() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        let current_branch = repo.get_current_branch()?;

        assert_eq!(current_branch, String::from("main"));

        Ok(())
    }

    #[test]
    fn test_get_last_tag() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.create_tag("v1.0.0", None)?;
        repo.create_tag("v1.1.0", Some("chore: tag".to_string()))?;
        let last_tag = repo.get_last_tag()?;

        assert_eq!(last_tag, String::from("v1.1.0"));

        Ok(())
    }

    #[test]
    fn test_get_current_sha() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        let current_sha = repo.get_current_sha()?;

        assert!(!current_sha.is_empty());

        Ok(())
    }

    #[test]
    fn test_commit_changes() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.create_branch("feat/my-new-feature")?;
        repo.checkout("feat/my-new-feature")?;

        // create a file and commit
        let file_path = workspace_path.join("README.md");
        std::fs::write(&file_path, "Hello, world!").expect("Failed to write Readme file");
        repo.add(file_path.display().to_string().as_str())?;
        let commit_id = repo.commit_changes("feat: add README.md")?;

        assert!(!commit_id.is_empty());

        Ok(())
    }

    #[test]
    fn test_add_all() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        repo.create_branch("feat/my-new-feature")?;
        repo.checkout("feat/my-new-feature")?;

        // create a file and commit
        let file_path = workspace_path.join("README.md");
        std::fs::write(&file_path, "Hello, world!").expect("Failed to write Readme file");
        let commit_id = repo.add_all()?.commit("feat: add README.md")?;

        assert!(!commit_id.is_empty());

        Ok(())
    }

    #[test]
    fn test_get_previous_sha() -> Result<(), RepoError> {
        let workspace_path = &create_workspace().unwrap();

        let repo = Repo::create(workspace_path.display().to_string().as_str())?;
        let parent_sha = repo.get_previous_sha()?;

        assert!(!parent_sha.is_empty());

        Ok(())
    }
}
