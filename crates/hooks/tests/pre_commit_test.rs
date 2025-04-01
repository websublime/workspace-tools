#[cfg(test)]
mod pre_commit_tests {
    use std::{
        env::temp_dir,
        fs::{create_dir, remove_dir_all, File},
        io::Write,
        path::PathBuf,
    };

    use sublime_hooks_tools::{
        hook::{HookConfig, HookContext},
        pre_commit::PreCommitHook,
    };
    use sublime_monorepo_tools::changes::memory::MemoryChangeStore;

    fn create_test_workspace() -> Result<PathBuf, std::io::Error> {
        let temp_dir = temp_dir();
        let workspace_dir = temp_dir.join("test-workspace");

        if workspace_dir.exists() {
            remove_dir_all(&workspace_dir)?;
        }

        create_dir(&workspace_dir)?;

        // Create a package.json
        let mut package_file = File::create(workspace_dir.join("package.json"))?;
        package_file.write_all(
            br#"{
                "name": "test-package",
                "version": "1.0.0"
            }"#,
        )?;

        // Create a test file
        let mut test_file = File::create(workspace_dir.join("test.js"))?;
        test_file.write_all(b"console.log('test');")?;

        Ok(workspace_dir)
    }

    #[test]
    fn test_pre_commit_hook_creation() -> Result<(), Box<dyn std::error::Error>> {
        let workspace_dir = create_test_workspace()?;
        let repo = sublime_git_tools::Repo::new(&workspace_dir)?;
        let config = HookConfig::default();
        let change_store = Box::new(MemoryChangeStore::new());
        
        let ctx = HookContext::new(repo, config, change_store)?;
        let hook = PreCommitHook::new(&ctx);
        
        assert!(hook.run().is_ok());
        remove_dir_all(&workspace_dir)?;
        Ok(())
    }

    #[test]
    fn test_pre_commit_hook_with_changes() -> Result<(), Box<dyn std::error::Error>> {
        let workspace_dir = create_test_workspace()?;
        let repo = sublime_git_tools::Repo::new(&workspace_dir)?;
        
        // Initialize git and make a change
        repo.init()?;
        repo.add_all()?;
        repo.commit("Initial commit")?;
        
        // Make a change to test.js
        let mut test_file = File::create(workspace_dir.join("test.js"))?;
        test_file.write_all(b"console.log('modified');")?;
        
        let config = HookConfig::default();
        let change_store = Box::new(MemoryChangeStore::new());
        
        let ctx = HookContext::new(repo, config, change_store)?;
        let hook = PreCommitHook::new(&ctx);
        
        let decisions = hook.run()?;
        assert!(!decisions.is_empty());
        
        remove_dir_all(&workspace_dir)?;
        Ok(())
    }

    #[test]
    fn test_pre_commit_hook_with_ignored_paths() -> Result<(), Box<dyn std::error::Error>> {
        let workspace_dir = create_test_workspace()?;
        let repo = sublime_git_tools::Repo::new(&workspace_dir)?;
        
        // Initialize git and make a change
        repo.init()?;
        repo.add_all()?;
        repo.commit("Initial commit")?;
        
        // Make a change to test.js
        let mut test_file = File::create(workspace_dir.join("test.js"))?;
        test_file.write_all(b"console.log('modified');")?;
        
        let mut config = HookConfig::default();
        config.ignore_paths = vec!["test.js".to_string()];
        
        let change_store = Box::new(MemoryChangeStore::new());
        let ctx = HookContext::new(repo, config, change_store)?;
        let hook = PreCommitHook::new(&ctx);
        
        let decisions = hook.run()?;
        assert!(decisions.is_empty());
        
        remove_dir_all(&workspace_dir)?;
        Ok(())
    }
} 