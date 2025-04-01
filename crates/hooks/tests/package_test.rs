#[cfg(test)]
mod package_tests {
    use std::{
        env::temp_dir,
        fs::{create_dir, remove_dir_all, File},
        io::Write,
        path::PathBuf,
    };

    use sublime_hooks_tools::package::PackageManager;
    use sublime_pkg_tools::version::Version;

    fn create_test_workspace() -> Result<PathBuf, std::io::Error> {
        let temp_dir = temp_dir();
        let workspace_dir = temp_dir.join("test-workspace");

        if workspace_dir.exists() {
            remove_dir_all(&workspace_dir)?;
        }

        create_dir(&workspace_dir)?;

        // Create a package.json with version
        let mut package_file = File::create(workspace_dir.join("package.json"))?;
        package_file.write_all(
            br#"{
                "name": "test-package",
                "version": "1.0.0",
                "dependencies": {
                    "dep-package": "^1.0.0"
                }
            }"#,
        )?;

        Ok(workspace_dir)
    }

    #[test]
    fn test_package_manager_creation() -> Result<(), Box<dyn std::error::Error>> {
        let workspace_dir = create_test_workspace()?;
        let manager = PackageManager::new(&workspace_dir)?;
        assert!(manager.get_package("test-package").is_ok());
        remove_dir_all(&workspace_dir)?;
        Ok(())
    }

    #[test]
    fn test_get_package_version() -> Result<(), Box<dyn std::error::Error>> {
        let workspace_dir = create_test_workspace()?;
        let manager = PackageManager::new(&workspace_dir)?;
        let version = manager.get_package_version("test-package")?;
        assert_eq!(version, "1.0.0");
        remove_dir_all(&workspace_dir)?;
        Ok(())
    }

    #[test]
    fn test_record_version_decision() -> Result<(), Box<dyn std::error::Error>> {
        let workspace_dir = create_test_workspace()?;
        let manager = PackageManager::new(&workspace_dir)?;
        
        // Record a version decision
        manager.record_version_decision("test-package", Version::Minor)?;
        
        // Verify the decision was recorded
        let decision = manager.get_version_decision("test-package")?;
        assert_eq!(decision, Some(Version::Minor));
        
        remove_dir_all(&workspace_dir)?;
        Ok(())
    }

    #[test]
    fn test_has_affected_dependents() -> Result<(), Box<dyn std::error::Error>> {
        let workspace_dir = create_test_workspace()?;
        let manager = PackageManager::new(&workspace_dir)?;
        
        // Since we have a dependency in package.json, this should return true
        let has_dependents = manager.has_affected_dependents("dep-package")?;
        assert!(has_dependents);
        
        // A non-existent package should have no dependents
        let has_dependents = manager.has_affected_dependents("non-existent")?;
        assert!(!has_dependents);
        
        remove_dir_all(&workspace_dir)?;
        Ok(())
    }
} 