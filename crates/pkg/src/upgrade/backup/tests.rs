//! Tests for backup and rollback functionality.

#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use super::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use sublime_standard_tools::error::{FileSystemError, Result as StandardResult};

/// Mock filesystem for testing backup operations.
#[derive(Debug, Clone)]
struct MockFileSystem {
    files: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
}

impl MockFileSystem {
    fn new() -> Self {
        Self { files: Arc::new(Mutex::new(HashMap::new())) }
    }

    fn add_file(&self, path: PathBuf, content: &str) {
        self.files.lock().unwrap().insert(path, content.as_bytes().to_vec());
    }

    fn get_file(&self, path: &Path) -> Option<String> {
        self.files.lock().unwrap().get(path).map(|bytes| String::from_utf8_lossy(bytes).to_string())
    }

    fn file_exists(&self, path: &Path) -> bool {
        self.files.lock().unwrap().contains_key(path)
    }
}

#[async_trait::async_trait]
impl AsyncFileSystem for MockFileSystem {
    async fn read_file(&self, path: &Path) -> StandardResult<Vec<u8>> {
        self.files
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| FileSystemError::NotFound { path: path.to_path_buf() }.into())
    }

    async fn write_file(&self, path: &Path, contents: &[u8]) -> StandardResult<()> {
        self.files.lock().unwrap().insert(path.to_path_buf(), contents.to_vec());
        Ok(())
    }

    async fn read_file_string(&self, path: &Path) -> StandardResult<String> {
        let bytes = self.read_file(path).await?;
        String::from_utf8(bytes).map_err(|e| {
            FileSystemError::Utf8Decode { path: path.to_path_buf(), message: e.to_string() }.into()
        })
    }

    async fn write_file_string(&self, path: &Path, contents: &str) -> StandardResult<()> {
        self.write_file(path, contents.as_bytes()).await
    }

    async fn create_dir_all(&self, _path: &Path) -> StandardResult<()> {
        // Mock implementation - directories are implicit
        Ok(())
    }

    async fn remove(&self, path: &Path) -> StandardResult<()> {
        let path_str = path.to_string_lossy().to_string();
        let mut files = self.files.lock().unwrap();
        // Remove the path itself and all children (simulates recursive removal)
        files.retain(|p, _| {
            let p_str = p.to_string_lossy();
            // Normalize paths for cross-platform comparison
            let normalized_path = path_str.replace('\\', "/");
            let normalized_p = p_str.replace('\\', "/");
            normalized_p != normalized_path
                && !normalized_p.starts_with(&format!("{}/", normalized_path))
        });
        Ok(())
    }

    async fn exists(&self, path: &Path) -> bool {
        let files = self.files.lock().unwrap();
        // Check if path exists as a file
        if files.contains_key(path) {
            return true;
        }
        // Check if path exists as a directory (has children)
        let path_str = path.to_string_lossy();
        let normalized_path = path_str.replace('\\', "/");
        files.keys().any(|p| {
            let p_str = p.to_string_lossy();
            let normalized_p = p_str.replace('\\', "/");
            normalized_p.starts_with(&format!("{}/", normalized_path))
        })
    }

    async fn read_dir(&self, path: &Path) -> StandardResult<Vec<PathBuf>> {
        let path_str = path.to_string_lossy().to_string();
        let normalized_path = path_str.replace('\\', "/");
        let files: Vec<PathBuf> = self
            .files
            .lock()
            .unwrap()
            .keys()
            .filter(|p| {
                let p_str = p.to_string_lossy();
                let normalized_p = p_str.replace('\\', "/");
                normalized_p.starts_with(&normalized_path)
            })
            .cloned()
            .collect();
        Ok(files)
    }

    async fn walk_dir(&self, path: &Path) -> StandardResult<Vec<PathBuf>> {
        self.read_dir(path).await
    }

    async fn metadata(&self, _path: &Path) -> StandardResult<std::fs::Metadata> {
        Err(FileSystemError::Operation("Metadata not supported in mock".to_string()).into())
    }
}

fn create_test_manager(config: BackupConfig) -> BackupManager<MockFileSystem> {
    BackupManager::new(PathBuf::from("/workspace"), config, MockFileSystem::new())
}

#[tokio::test]
async fn test_create_backup_success() {
    let config = BackupConfig {
        enabled: true,
        backup_dir: ".pkg-backups".to_string(),
        keep_after_success: false,
        max_backups: 5,
    };
    let manager = create_test_manager(config);

    // Add test files
    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "test"}"#);
    manager.fs.add_file(
        PathBuf::from("/workspace/packages/core/package.json"),
        r#"{"name": "@test/core"}"#,
    );

    let files = vec![
        PathBuf::from("/workspace/package.json"),
        PathBuf::from("/workspace/packages/core/package.json"),
    ];

    let backup_id = manager.create_backup(&files, "upgrade").await.unwrap();

    assert!(backup_id.contains("upgrade"));
    assert!(backup_id.contains("-"));

    // Verify backup files exist
    let backup_path = manager.backup_path(&backup_id);
    assert!(manager.fs.file_exists(&backup_path.join("package.json")));
    assert!(manager.fs.file_exists(&backup_path.join("packages/core/package.json")));

    // Verify metadata
    let backups = manager.list_backups().await.unwrap();
    assert_eq!(backups.len(), 1);
    assert_eq!(backups[0].id, backup_id);
    assert_eq!(backups[0].operation, "upgrade");
    assert_eq!(backups[0].files.len(), 2);
    assert!(!backups[0].success);
}

#[tokio::test]
async fn test_create_backup_disabled() {
    let config = BackupConfig {
        enabled: false,
        backup_dir: ".pkg-backups".to_string(),
        keep_after_success: false,
        max_backups: 5,
    };
    let manager = create_test_manager(config);

    let files = vec![PathBuf::from("/workspace/package.json")];

    let result = manager.create_backup(&files, "upgrade").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        UpgradeError::BackupFailed { reason, .. } => {
            assert!(reason.contains("disabled"));
        }
        _ => panic!("Expected BackupFailed error"),
    }
}

#[tokio::test]
async fn test_create_backup_nonexistent_file() {
    let config = BackupConfig::default();
    let manager = create_test_manager(config);

    let files = vec![PathBuf::from("/workspace/nonexistent.json")];

    let result = manager.create_backup(&files, "upgrade").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        UpgradeError::FileSystemError { reason, .. } => {
            assert!(reason.contains("does not exist"));
        }
        _ => panic!("Expected FileSystemError"),
    }
}

#[tokio::test]
async fn test_restore_backup_success() {
    let config = BackupConfig::default();
    let manager = create_test_manager(config);

    // Add original files
    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "test"}"#);

    let files = vec![PathBuf::from("/workspace/package.json")];
    let backup_id = manager.create_backup(&files, "upgrade").await.unwrap();

    // Modify the file
    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "modified"}"#);

    // Restore backup
    manager.restore_backup(&backup_id).await.unwrap();

    // Verify file was restored
    let content = manager.fs.get_file(&PathBuf::from("/workspace/package.json")).unwrap();
    assert!(content.contains(r#""name": "test""#));
}

#[tokio::test]
async fn test_restore_backup_nonexistent() {
    let config = BackupConfig::default();
    let manager = create_test_manager(config);

    let result = manager.restore_backup("nonexistent-backup").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        UpgradeError::NoBackup { .. } => {}
        _ => panic!("Expected NoBackup error"),
    }
}

#[tokio::test]
async fn test_restore_last_backup() {
    let config = BackupConfig::default();
    let manager = create_test_manager(config);

    // Add files and create multiple backups
    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "v1"}"#);
    let _backup1 = manager
        .create_backup(&[PathBuf::from("/workspace/package.json")], "upgrade")
        .await
        .unwrap();

    // Wait a bit to ensure different timestamps
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "v2"}"#);
    let _backup2 = manager
        .create_backup(&[PathBuf::from("/workspace/package.json")], "upgrade")
        .await
        .unwrap();

    // Modify file
    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "v3"}"#);

    // Restore last backup (should be backup2)
    manager.restore_last_backup().await.unwrap();

    let content = manager.fs.get_file(&PathBuf::from("/workspace/package.json")).unwrap();
    assert!(content.contains(r#""name": "v2""#));
}

#[tokio::test]
async fn test_restore_last_backup_no_backups() {
    let config = BackupConfig::default();
    let manager = create_test_manager(config);

    let result = manager.restore_last_backup().await;

    assert!(result.is_err());
    match result.unwrap_err() {
        UpgradeError::NoBackup { .. } => {}
        _ => panic!("Expected NoBackup error"),
    }
}

#[tokio::test]
async fn test_list_backups() {
    let config = BackupConfig::default();
    let manager = create_test_manager(config);

    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "test"}"#);

    // Create multiple backups
    let backup1 = manager
        .create_backup(&[PathBuf::from("/workspace/package.json")], "upgrade")
        .await
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let backup2 = manager
        .create_backup(&[PathBuf::from("/workspace/package.json")], "rollback")
        .await
        .unwrap();

    let backups = manager.list_backups().await.unwrap();

    assert_eq!(backups.len(), 2);
    // Should be sorted newest first
    assert_eq!(backups[0].id, backup2);
    assert_eq!(backups[1].id, backup1);
}

#[tokio::test]
async fn test_list_backups_empty() {
    let config = BackupConfig::default();
    let manager = create_test_manager(config);

    let backups = manager.list_backups().await.unwrap();
    assert!(backups.is_empty());
}

#[tokio::test]
async fn test_delete_backup() {
    let config = BackupConfig::default();
    let manager = create_test_manager(config);

    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "test"}"#);

    let backup_id = manager
        .create_backup(&[PathBuf::from("/workspace/package.json")], "upgrade")
        .await
        .unwrap();

    // Verify backup exists
    let backups = manager.list_backups().await.unwrap();
    assert_eq!(backups.len(), 1);

    // Delete backup
    manager.delete_backup(&backup_id).await.unwrap();

    // Verify backup is gone
    let backups = manager.list_backups().await.unwrap();
    assert!(backups.is_empty());
}

#[tokio::test]
async fn test_delete_backup_nonexistent() {
    let config = BackupConfig::default();
    let manager = create_test_manager(config);

    let result = manager.delete_backup("nonexistent-backup").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        UpgradeError::NoBackup { .. } => {}
        _ => panic!("Expected NoBackup error"),
    }
}

#[tokio::test]
async fn test_mark_success() {
    let config = BackupConfig::default();
    let manager = create_test_manager(config);

    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "test"}"#);

    let backup_id = manager
        .create_backup(&[PathBuf::from("/workspace/package.json")], "upgrade")
        .await
        .unwrap();

    // Initially marked as not successful
    let backups = manager.list_backups().await.unwrap();
    assert!(!backups[0].success);

    // Mark as successful
    manager.mark_success(&backup_id).await.unwrap();

    // Verify success flag
    let backups = manager.list_backups().await.unwrap();
    assert!(backups[0].success);
}

#[tokio::test]
async fn test_mark_success_nonexistent() {
    let config = BackupConfig::default();
    let manager = create_test_manager(config);

    let result = manager.mark_success("nonexistent-backup").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        UpgradeError::NoBackup { .. } => {}
        _ => panic!("Expected NoBackup error"),
    }
}

#[tokio::test]
async fn test_cleanup_removes_successful_backups() {
    let config = BackupConfig {
        enabled: true,
        backup_dir: ".pkg-backups".to_string(),
        keep_after_success: false,
        max_backups: 5,
    };
    let manager = create_test_manager(config);

    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "test"}"#);

    // Create backup and mark as successful
    let backup_id = manager
        .create_backup(&[PathBuf::from("/workspace/package.json")], "upgrade")
        .await
        .unwrap();

    manager.mark_success(&backup_id).await.unwrap();

    // Verify backup exists
    let backups = manager.list_backups().await.unwrap();
    assert_eq!(backups.len(), 1);

    // Cleanup should remove successful backups
    manager.cleanup_old_backups().await.unwrap();

    let backups = manager.list_backups().await.unwrap();
    assert!(backups.is_empty());
}

#[tokio::test]
async fn test_cleanup_keeps_successful_backups() {
    let config = BackupConfig {
        enabled: true,
        backup_dir: ".pkg-backups".to_string(),
        keep_after_success: true,
        max_backups: 5,
    };
    let manager = create_test_manager(config);

    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "test"}"#);

    let backup_id = manager
        .create_backup(&[PathBuf::from("/workspace/package.json")], "upgrade")
        .await
        .unwrap();

    manager.mark_success(&backup_id).await.unwrap();

    // Cleanup should keep successful backups
    manager.cleanup_old_backups().await.unwrap();

    let backups = manager.list_backups().await.unwrap();
    assert_eq!(backups.len(), 1);
}

#[tokio::test]
async fn test_cleanup_removes_old_backups() {
    let config = BackupConfig {
        enabled: true,
        backup_dir: ".pkg-backups".to_string(),
        keep_after_success: true,
        max_backups: 3,
    };
    let manager = create_test_manager(config);

    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "test"}"#);

    // Create 5 backups
    for _ in 0..5 {
        manager
            .create_backup(&[PathBuf::from("/workspace/package.json")], "upgrade")
            .await
            .unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let backups = manager.list_backups().await.unwrap();
    assert_eq!(backups.len(), 3); // Should only keep max_backups
}

#[tokio::test]
async fn test_cleanup_priority_removes_successful_before_count() {
    let config = BackupConfig {
        enabled: true,
        backup_dir: ".pkg-backups".to_string(),
        keep_after_success: false,
        max_backups: 2,
    };
    let manager = create_test_manager(config);

    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "test"}"#);

    // Create 3 backups, mark 2 as successful
    let backup1 = manager
        .create_backup(&[PathBuf::from("/workspace/package.json")], "upgrade")
        .await
        .unwrap();
    manager.mark_success(&backup1).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let backup2 = manager
        .create_backup(&[PathBuf::from("/workspace/package.json")], "upgrade")
        .await
        .unwrap();
    manager.mark_success(&backup2).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let _backup3 = manager
        .create_backup(&[PathBuf::from("/workspace/package.json")], "upgrade")
        .await
        .unwrap();

    // Cleanup should remove successful backups first
    manager.cleanup_old_backups().await.unwrap();

    let backups = manager.list_backups().await.unwrap();
    assert_eq!(backups.len(), 1); // Only the failed one should remain
    assert!(!backups[0].success);
}

#[tokio::test]
async fn test_relative_path_handling() {
    let config = BackupConfig::default();
    let manager = create_test_manager(config);

    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "test"}"#);

    // Use relative path
    let files = vec![PathBuf::from("package.json")];
    let backup_id = manager.create_backup(&files, "upgrade").await.unwrap();

    // Verify backup was created with correct path
    let backups = manager.list_backups().await.unwrap();
    assert_eq!(backups[0].files[0], PathBuf::from("/workspace/package.json"));

    // Restore should work with the resolved path
    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "modified"}"#);
    manager.restore_backup(&backup_id).await.unwrap();

    let content = manager.fs.get_file(&PathBuf::from("/workspace/package.json")).unwrap();
    assert!(content.contains(r#""name": "test""#));
}

#[tokio::test]
async fn test_nested_directory_structure() {
    let config = BackupConfig::default();
    let manager = create_test_manager(config);

    // Add deeply nested file
    manager.fs.add_file(
        PathBuf::from("/workspace/packages/core/src/lib/package.json"),
        r#"{"name": "@test/core"}"#,
    );

    let files = vec![PathBuf::from("/workspace/packages/core/src/lib/package.json")];
    let backup_id = manager.create_backup(&files, "upgrade").await.unwrap();

    // Verify backup preserves directory structure
    let backup_path = manager.backup_path(&backup_id);
    assert!(manager.fs.file_exists(&backup_path.join("packages/core/src/lib/package.json")));
}

#[tokio::test]
async fn test_multiple_files_backup_and_restore() {
    let config = BackupConfig::default();
    let manager = create_test_manager(config);

    // Add multiple files
    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "root"}"#);
    manager.fs.add_file(PathBuf::from("/workspace/packages/a/package.json"), r#"{"name": "a"}"#);
    manager.fs.add_file(PathBuf::from("/workspace/packages/b/package.json"), r#"{"name": "b"}"#);

    let files = vec![
        PathBuf::from("/workspace/package.json"),
        PathBuf::from("/workspace/packages/a/package.json"),
        PathBuf::from("/workspace/packages/b/package.json"),
    ];

    let backup_id = manager.create_backup(&files, "upgrade").await.unwrap();

    // Modify all files
    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "root-mod"}"#);
    manager
        .fs
        .add_file(PathBuf::from("/workspace/packages/a/package.json"), r#"{"name": "a-mod"}"#);
    manager
        .fs
        .add_file(PathBuf::from("/workspace/packages/b/package.json"), r#"{"name": "b-mod"}"#);

    // Restore
    manager.restore_backup(&backup_id).await.unwrap();

    // Verify all files restored
    assert!(manager
        .fs
        .get_file(&PathBuf::from("/workspace/package.json"))
        .unwrap()
        .contains("root"));
    assert!(manager
        .fs
        .get_file(&PathBuf::from("/workspace/packages/a/package.json"))
        .unwrap()
        .contains(r#""name": "a""#));
    assert!(manager
        .fs
        .get_file(&PathBuf::from("/workspace/packages/b/package.json"))
        .unwrap()
        .contains(r#""name": "b""#));
}

#[tokio::test]
async fn test_backup_metadata_serialization() {
    let metadata = BackupMetadata {
        id: "2024-01-15T10-30-45-upgrade".to_string(),
        created_at: Utc::now(),
        operation: "upgrade".to_string(),
        files: vec![PathBuf::from("/workspace/package.json")],
        success: true,
    };

    let json = serde_json::to_string(&metadata).unwrap();
    let deserialized: BackupMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(metadata.id, deserialized.id);
    assert_eq!(metadata.operation, deserialized.operation);
    assert_eq!(metadata.files, deserialized.files);
    assert_eq!(metadata.success, deserialized.success);
}

#[tokio::test]
async fn test_backup_id_format() {
    let config = BackupConfig::default();
    let manager = create_test_manager(config);

    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "test"}"#);

    let backup_id = manager
        .create_backup(&[PathBuf::from("/workspace/package.json")], "upgrade")
        .await
        .unwrap();

    // Verify format: YYYY-MM-DDTHH-MM-SS-mmm-operation (with milliseconds)
    assert!(backup_id.ends_with("-upgrade"));
    assert!(backup_id.contains('T'));
    assert_eq!(backup_id.matches('-').count(), 6); // 5 in timestamp (including milliseconds) + 1 before operation
}

#[tokio::test]
async fn test_concurrent_backups() {
    let config = BackupConfig::default();
    let manager = Arc::new(create_test_manager(config));

    manager.fs.add_file(PathBuf::from("/workspace/package.json"), r#"{"name": "test"}"#);

    // Create multiple backups concurrently
    let mut handles = vec![];
    for i in 0..5 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(i * 10)).await;
            manager_clone
                .create_backup(&[PathBuf::from("/workspace/package.json")], "upgrade")
                .await
        });
        handles.push(handle);
    }

    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should succeed
    assert_eq!(results.len(), 5);
    for result in results {
        assert!(result.unwrap().is_ok());
    }

    // Verify all backups were created
    let backups = manager.list_backups().await.unwrap();
    assert!(backups.len() <= 5); // May be less due to cleanup
}
