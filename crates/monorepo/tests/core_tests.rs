//! Tests for core monorepo types

use sublime_monorepo_tools::{
    MonorepoPackageInfo,
    VersionStatus,
    Changeset,
    ChangesetStatus,
    Environment,
    config::VersionBumpType,
};
use sublime_package_tools::{Package, PackageInfo};
use sublime_standard_tools::monorepo::WorkspacePackage;
use std::path::PathBuf;
use chrono::Utc;

fn create_test_package_info(name: &str, version: &str) -> MonorepoPackageInfo {
    let package = Package::new(name, version, None).unwrap();
    let package_json_path = format!("packages/{}/package.json", name);
    let package_path = format!("packages/{}", name);
    
    let package_info = PackageInfo::new(
        package,
        package_json_path.clone(),
        package_path.clone(),
        package_path.clone(),
        serde_json::json!({
            "name": name,
            "version": version,
        }),
    );
    
    let workspace_package = WorkspacePackage {
        name: name.to_string(),
        version: version.to_string(),
        location: PathBuf::from(&package_path),
        absolute_path: PathBuf::from(format!("/monorepo/{}", package_path)),
        workspace_dependencies: vec![],
        workspace_dev_dependencies: vec![],
    };
    
    MonorepoPackageInfo::new(package_info, workspace_package, true)
}

#[test]
fn test_monorepo_package_info_basic() {
    let pkg = create_test_package_info("test-package", "1.0.0");
    
    assert_eq!(pkg.name(), "test-package");
    assert_eq!(pkg.version(), "1.0.0");
    assert!(pkg.is_internal);
    assert!(!pkg.is_dirty());
    assert!(!pkg.has_pending_changesets());
}

#[test]
fn test_version_update() {
    let mut pkg = create_test_package_info("test-package", "1.0.0");
    
    pkg.update_version("1.1.0").unwrap();
    assert_eq!(pkg.version(), "1.1.0");
    assert!(matches!(pkg.version_status, VersionStatus::Stable));
}

#[test]
fn test_snapshot_version() {
    let mut pkg = create_test_package_info("test-package", "1.0.0");
    
    pkg.set_snapshot_version("1.0.1", "abc1234567890").unwrap();
    assert_eq!(pkg.version(), "1.0.1-snapshot.abc1234");
    assert!(matches!(pkg.version_status, VersionStatus::Snapshot { sha } if sha == "abc1234567890"));
}

#[test]
fn test_mark_dirty() {
    let mut pkg = create_test_package_info("test-package", "1.0.0");
    
    pkg.mark_dirty();
    assert!(pkg.is_dirty());
    assert!(matches!(pkg.version_status, VersionStatus::Dirty));
}

#[test]
fn test_changeset_management() {
    let mut pkg = create_test_package_info("test-package", "1.0.0");
    
    let changeset = Changeset {
        id: "cs-001".to_string(),
        package: "test-package".to_string(),
        version_bump: VersionBumpType::Minor,
        description: "Add new feature".to_string(),
        branch: "feature/new-feature".to_string(),
        development_environments: vec![Environment::Development],
        production_deployment: false,
        created_at: Utc::now(),
        author: "Test Author".to_string(),
        status: ChangesetStatus::Pending,
    };
    
    pkg.add_changeset(changeset.clone());
    
    assert!(pkg.has_pending_changesets());
    assert_eq!(pkg.pending_changesets().len(), 1);
    assert_eq!(pkg.pending_changesets()[0].id, "cs-001");
}

#[test]
fn test_apply_changeset() {
    let mut pkg = create_test_package_info("test-package", "1.0.0");
    
    let changeset = Changeset {
        id: "cs-001".to_string(),
        package: "test-package".to_string(),
        version_bump: VersionBumpType::Minor,
        description: "Add new feature".to_string(),
        branch: "feature/new-feature".to_string(),
        development_environments: vec![],
        production_deployment: false,
        created_at: Utc::now(),
        author: "Test Author".to_string(),
        status: ChangesetStatus::Pending,
    };
    
    pkg.add_changeset(changeset);
    
    // Apply changeset should bump minor version
    pkg.apply_changeset("cs-001", None).unwrap();
    assert_eq!(pkg.version(), "1.1.0");
    
    // Changeset should now be merged
    assert!(!pkg.has_pending_changesets());
    assert!(matches!(
        pkg.changesets[0].status,
        ChangesetStatus::Merged { .. }
    ));
}

#[test]
fn test_apply_changeset_with_final_version() {
    let mut pkg = create_test_package_info("test-package", "1.0.0");
    
    let changeset = Changeset {
        id: "cs-001".to_string(),
        package: "test-package".to_string(),
        version_bump: VersionBumpType::Patch,
        description: "Bug fix".to_string(),
        branch: "fix/bug".to_string(),
        development_environments: vec![],
        production_deployment: false,
        created_at: Utc::now(),
        author: "Test Author".to_string(),
        status: ChangesetStatus::Pending,
    };
    
    pkg.add_changeset(changeset);
    
    // Apply changeset with specific version
    pkg.apply_changeset("cs-001", Some("2.0.0")).unwrap();
    assert_eq!(pkg.version(), "2.0.0");
}

#[test]
fn test_deploy_changeset() {
    let mut pkg = create_test_package_info("test-package", "1.0.0");
    
    let changeset = Changeset {
        id: "cs-001".to_string(),
        package: "test-package".to_string(),
        version_bump: VersionBumpType::Minor,
        description: "Add new feature".to_string(),
        branch: "feature/new-feature".to_string(),
        development_environments: vec![],
        production_deployment: false,
        created_at: Utc::now(),
        author: "Test Author".to_string(),
        status: ChangesetStatus::Pending,
    };
    
    pkg.add_changeset(changeset);
    
    // Deploy to development
    pkg.deploy_changeset("cs-001", &[Environment::Development]).unwrap();
    assert!(matches!(
        pkg.changesets[0].status,
        ChangesetStatus::PartiallyDeployed { .. }
    ));
    assert!(pkg.changesets[0].development_environments.contains(&Environment::Development));
    
    // Deploy to staging
    pkg.deploy_changeset("cs-001", &[Environment::Staging]).unwrap();
    assert_eq!(pkg.changesets[0].development_environments.len(), 2);
    
    // Deploy to production
    pkg.deploy_changeset("cs-001", &[Environment::Production]).unwrap();
    assert!(pkg.changesets[0].production_deployment);
    assert!(matches!(
        pkg.changesets[0].status,
        ChangesetStatus::FullyDeployed { .. }
    ));
}

#[test]
fn test_suggested_version_bump() {
    let mut pkg = create_test_package_info("test-package", "1.0.0");
    
    // No changesets, no suggestion
    assert!(pkg.suggested_version_bump().is_none());
    
    // Add patch changeset
    pkg.add_changeset(Changeset {
        id: "cs-001".to_string(),
        package: "test-package".to_string(),
        version_bump: VersionBumpType::Patch,
        description: "Bug fix".to_string(),
        branch: "fix/bug".to_string(),
        development_environments: vec![],
        production_deployment: false,
        created_at: Utc::now(),
        author: "Test Author".to_string(),
        status: ChangesetStatus::Pending,
    });
    
    assert_eq!(pkg.suggested_version_bump(), Some(VersionBumpType::Patch));
    
    // Add minor changeset
    pkg.add_changeset(Changeset {
        id: "cs-002".to_string(),
        package: "test-package".to_string(),
        version_bump: VersionBumpType::Minor,
        description: "New feature".to_string(),
        branch: "feature/new".to_string(),
        development_environments: vec![],
        production_deployment: false,
        created_at: Utc::now(),
        author: "Test Author".to_string(),
        status: ChangesetStatus::Pending,
    });
    
    // Should suggest minor (higher priority)
    assert_eq!(pkg.suggested_version_bump(), Some(VersionBumpType::Minor));
    
    // Add major changeset
    pkg.add_changeset(Changeset {
        id: "cs-003".to_string(),
        package: "test-package".to_string(),
        version_bump: VersionBumpType::Major,
        description: "Breaking change".to_string(),
        branch: "feature/breaking".to_string(),
        development_environments: vec![],
        production_deployment: false,
        created_at: Utc::now(),
        author: "Test Author".to_string(),
        status: ChangesetStatus::Pending,
    });
    
    // Should suggest major (highest priority)
    assert_eq!(pkg.suggested_version_bump(), Some(VersionBumpType::Major));
}

#[test]
fn test_deployment_status() {
    let mut pkg = create_test_package_info("test-package", "1.0.0");
    
    let changeset = Changeset {
        id: "cs-001".to_string(),
        package: "test-package".to_string(),
        version_bump: VersionBumpType::Minor,
        description: "Add new feature".to_string(),
        branch: "feature/new-feature".to_string(),
        development_environments: vec![Environment::Development, Environment::Staging],
        production_deployment: true,
        created_at: Utc::now(),
        author: "Test Author".to_string(),
        status: ChangesetStatus::FullyDeployed {
            deployed_at: Utc::now(),
        },
    };
    
    pkg.add_changeset(changeset);
    
    let status = pkg.deployment_status();
    assert_eq!(status.get(&Environment::Development), Some(&true));
    assert_eq!(status.get(&Environment::Staging), Some(&true));
    assert_eq!(status.get(&Environment::Production), Some(&true));
    
    assert!(pkg.is_deployed_to(&Environment::Development));
    assert!(pkg.is_deployed_to(&Environment::Staging));
    assert!(pkg.is_deployed_to(&Environment::Production));
    assert!(!pkg.is_deployed_to(&Environment::Integration));
}