//! Tests to verify ownership boundaries are properly maintained
//!
//! These tests ensure that components follow the established ownership patterns
//! and don't violate Rust's ownership principles.

#[cfg(test)]
mod tests {
    use crate::core::{
        MonorepoPackageInfo, 
        components::{
            PackageVersionManager, PackageChangesetManager,
            PackageDependencyManager,
            PackageInfoReader
        }
    };
    use crate::config::{
        ConfigManager
    };

    /// Test that version manager follows ownership transfer pattern
    #[test]
    fn test_version_manager_ownership_transfer() {
        // Create a mock package
        let package = create_test_package();
        let original_name = package.name().to_string();
        
        // Manager takes ownership
        let mut manager = PackageVersionManager::new(package);
        // package is now moved and cannot be used
        
        // Operations work on owned data
        manager.update_version("2.0.0").unwrap();
        
        // Get ownership back
        let updated_package = manager.into_package();
        
        // Verify we can use the package again
        assert_eq!(updated_package.name(), original_name);
        assert_eq!(updated_package.version(), "2.0.0");
    }

    /// Test that changeset manager follows ownership transfer pattern
    #[test]
    fn test_changeset_manager_ownership_transfer() {
        let package = create_test_package();
        
        // Manager takes ownership
        let mut manager = PackageChangesetManager::new(package);
        
        // Add changeset
        let changeset = create_test_changeset();
        manager.add_changeset(changeset);
        
        // Get ownership back
        let updated_package = manager.into_package();
        
        // Verify changeset was added
        assert_eq!(updated_package.changesets.len(), 1);
    }

    /// Test that readers don't take ownership
    #[test]
    fn test_reader_borrows_only() {
        let package = create_test_package();
        
        // Reader only borrows
        let reader = PackageInfoReader::new(&package);
        let name = reader.name();
        
        // We can still use package
        assert_eq!(package.name(), name);
        
        // We can create multiple readers
        let reader2 = PackageInfoReader::new(&package);
        assert_eq!(reader2.version(), package.version());
    }

    /// Test ownership chain for multiple operations
    #[test]
    fn test_operation_ownership_chain() {
        let package = create_test_package();
        
        // Chain operations by transferring ownership
        let mut version_manager = PackageVersionManager::new(package);
        version_manager.update_version("2.0.0").unwrap();
        let package = version_manager.into_package();
            
        let mut changeset_manager = PackageChangesetManager::new(package);
        changeset_manager.add_changeset(create_test_changeset());
        let package = changeset_manager.into_package();
            
        let mut dependency_manager = PackageDependencyManager::new(package);
        dependency_manager.add_dependent("other-package".to_string());
        let package = dependency_manager.into_package();
        
        // Verify all operations applied
        assert_eq!(package.version(), "2.0.0");
        assert_eq!(package.changesets.len(), 1);
        assert_eq!(package.dependents.len(), 1);
    }

    /// Test config manager ownership patterns
    #[test]
    fn test_config_manager_ownership() {
        let config = create_test_config();
        let mut manager = ConfigManager::with_config(config);
        
        // Traditional API (mutable update)
        manager.update(|cfg| {
            cfg.versioning.auto_tag = true;
        }).unwrap();
        
        // Functional API (ownership transfer)
        let manager = manager.with_update(|cfg| {
            cfg.versioning.auto_tag = false;
        }).unwrap();
        
        // Verify we own the manager
        assert!(!manager.config.versioning.auto_tag);
    }

    /// Test that components can't be used after moving
    #[test]
    fn test_moved_component_unusable() {
        let package = create_test_package();
        let manager = PackageVersionManager::new(package);
        
        // This would fail to compile:
        // let name = package.name(); // Error: package moved
        
        // Must get ownership back first
        let package = manager.into_package();
        let _name = package.name(); // OK
    }

    /// Test no Arc or Rc in component APIs
    #[test]
    fn test_no_shared_ownership_in_apis() {
        // This test verifies at compile time that we don't expose
        // Arc or Rc in our public APIs by trying to use them
        
        let package = create_test_package();
        
        // These would fail to compile if APIs required Arc/Rc:
        let _manager = PackageVersionManager::new(package); // Takes owned value
        
        let package = create_test_package();
        let _reader = PackageInfoReader::new(&package); // Takes reference
        
        // No need for Arc::new() or Rc::new() anywhere
    }

    // Helper functions
    fn create_test_package() -> MonorepoPackageInfo {
        use sublime_package_tools::{Package, PackageInfo};
        use sublime_standard_tools::monorepo::WorkspacePackage;
        use std::path::PathBuf;
        
        let pkg = Package::new("test-package", "1.0.0", None).unwrap();
        let pkg_info = PackageInfo::new(
            pkg,
            "/test/package.json".to_string(),
            "/test".to_string(),
            "test".to_string(),
            serde_json::json!({
                "name": "test-package",
                "version": "1.0.0"
            })
        );
        
        let workspace_pkg = WorkspacePackage {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            absolute_path: PathBuf::from("/test"),
            location: PathBuf::from("test"),
            workspace_dependencies: vec![],
            workspace_dev_dependencies: vec![],
        };
        
        MonorepoPackageInfo::new(pkg_info, workspace_pkg, true)
    }
    
    fn create_test_changeset() -> crate::core::Changeset {
        use crate::core::{Changeset, ChangesetStatus};
        use crate::config::VersionBumpType;
        
        Changeset {
            id: "test-changeset".to_string(),
            package: "test-package".to_string(),
            version_bump: VersionBumpType::Minor,
            description: "Test changeset".to_string(),
            branch: "main".to_string(),
            development_environments: vec![],
            production_deployment: false,
            created_at: chrono::Utc::now(),
            author: "test-author".to_string(),
            status: ChangesetStatus::Pending,
        }
    }
    
    fn create_test_config() -> crate::config::MonorepoConfig {
        crate::config::MonorepoConfig::default()
    }
}