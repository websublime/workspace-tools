//! JavaScript bindings for package manager detection utilities

use napi_derive::napi;
use ws_std::manager::{
    detect_package_manager as ws_detect_pm, CorePackageManager as WsCorePackageManager,
};

/// JavaScript binding for ws_std::manager::CorePackageManager
#[napi]
pub enum CorePackageManager {
    /// npm package manager
    Npm,
    /// Yarn package manager
    Yarn,
    /// pnpm package manager
    Pnpm,
    /// Bun package manager
    Bun,
}

impl From<WsCorePackageManager> for CorePackageManager {
    fn from(pm: WsCorePackageManager) -> Self {
        match pm {
            WsCorePackageManager::Npm => Self::Npm,
            WsCorePackageManager::Yarn => Self::Yarn,
            WsCorePackageManager::Pnpm => Self::Pnpm,
            WsCorePackageManager::Bun => Self::Bun,
        }
    }
}

/// Detect which package manager is available in the workspace
///
/// @param {string} path - Directory path to check for package manager files
/// @returns {CorePackageManager | null} The detected package manager or null if none found
#[napi]
pub fn detect_package_manager(path: String) -> Option<CorePackageManager> {
    ws_detect_pm(&std::path::PathBuf::from(path)).map(|pm| pm.into())
}

#[cfg(test)]
mod manager_binding_tests {
    use super::*;
    use std::{
        env::temp_dir,
        fs::{create_dir, remove_dir_all, File},
        io::Write,
        path::PathBuf,
    };

    #[cfg(not(windows))]
    use std::os::unix::fs::PermissionsExt;

    #[cfg(not(windows))]
    use std::fs::set_permissions;

    fn create_workspace(manager_file: &str) -> Result<PathBuf, std::io::Error> {
        let temp_dir = temp_dir();
        let monorepo_root_dir = temp_dir.join("monorepo-workspace");

        if monorepo_root_dir.exists() {
            remove_dir_all(&monorepo_root_dir)?;
        }

        create_dir(&monorepo_root_dir)?;

        let mut file = File::create(monorepo_root_dir.join(manager_file).as_path())?;
        file.write_all(b"{}")?;

        #[cfg(not(windows))]
        set_permissions(&monorepo_root_dir, std::fs::Permissions::from_mode(0o777))?;

        Ok(monorepo_root_dir)
    }

    #[test]
    fn test_npm_manager_detection() -> Result<(), std::io::Error> {
        let root = &create_workspace("package-lock.json")?;

        let manager = detect_package_manager(root.to_string_lossy().to_string());
        assert!(manager.is_some());
        assert!(matches!(manager.unwrap(), CorePackageManager::Npm));

        remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn test_yarn_manager_detection() -> Result<(), std::io::Error> {
        let root = &create_workspace("yarn.lock")?;

        let manager = detect_package_manager(root.to_string_lossy().to_string());
        assert!(manager.is_some());
        assert!(matches!(manager.unwrap(), CorePackageManager::Yarn));

        remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn test_pnpm_manager_detection() -> Result<(), std::io::Error> {
        let root = &create_workspace("pnpm-lock.yaml")?;

        let manager = detect_package_manager(root.to_string_lossy().to_string());
        assert!(manager.is_some());
        assert!(matches!(manager.unwrap(), CorePackageManager::Pnpm));

        remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn test_bun_manager_detection() -> Result<(), std::io::Error> {
        let root = &create_workspace("bun.lockb")?;

        let manager = detect_package_manager(root.to_string_lossy().to_string());
        assert!(manager.is_some());
        assert!(matches!(manager.unwrap(), CorePackageManager::Bun));

        remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn test_no_manager_detection() -> Result<(), std::io::Error> {
        let root = &create_workspace("no-lock-file.txt")?;

        let manager = detect_package_manager(root.to_string_lossy().to_string());
        assert!(manager.is_none());

        remove_dir_all(root)?;
        Ok(())
    }
}
