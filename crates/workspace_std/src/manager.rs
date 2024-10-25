use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter, Result as StdResult},
    path::Path,
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CorePackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

impl From<String> for CorePackageManager {
    fn from(manager: String) -> Self {
        match manager.as_str() {
            "npm" => Self::Npm,
            "yarn" => Self::Yarn,
            "pnpm" => Self::Pnpm,
            "bun" => Self::Bun,
            _ => panic!("Unable to identify package manager: {manager}"),
        }
    }
}

impl Display for CorePackageManager {
    fn fmt(&self, f: &mut Formatter) -> StdResult {
        match self {
            Self::Npm => write!(f, "npm"),
            Self::Yarn => write!(f, "yarn"),
            Self::Pnpm => write!(f, "pnpm"),
            Self::Bun => write!(f, "bun"),
        }
    }
}

/// Detects which package manager is available in the workspace.
pub fn detect_package_manager(path: &Path) -> Option<CorePackageManager> {
    let package_manager_files = HashMap::from([
        ("package-lock.json", CorePackageManager::Npm),
        ("npm-shrinkwrap.json", CorePackageManager::Npm),
        ("yarn.lock", CorePackageManager::Yarn),
        ("pnpm-lock.yaml", CorePackageManager::Pnpm),
        ("bun.lockb", CorePackageManager::Bun),
    ]);

    for (file, package_manager) in package_manager_files {
        let lock_file = path.join(file);

        if lock_file.exists() {
            return Some(package_manager);
        }
    }

    if let Some(parent) = path.parent() {
        return detect_package_manager(parent);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::MonorepoWorkspace;

    #[test]
    fn test_npm_manager() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Npm)?;

        let core_manager = detect_package_manager(root.as_path());
        let manager = core_manager.unwrap();

        assert_eq!(manager, CorePackageManager::Npm);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_pnpm_manager() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Pnpm)?;

        let core_manager = detect_package_manager(root.as_path());
        let manager = core_manager.unwrap();

        assert_eq!(manager, CorePackageManager::Pnpm);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_yarn_manager() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Yarn)?;

        let core_manager = detect_package_manager(root.as_path());
        let manager = core_manager.unwrap();

        assert_eq!(manager, CorePackageManager::Yarn);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_bun_manager() -> Result<(), std::io::Error> {
        let monorepo = MonorepoWorkspace::new();
        let root = monorepo.get_monorepo_root().clone();
        monorepo.create_repository(&CorePackageManager::Bun)?;

        let core_manager = detect_package_manager(root.as_path());
        let manager = core_manager.unwrap();

        assert_eq!(manager, CorePackageManager::Bun);

        monorepo.delete_repository();

        Ok(())
    }

    #[test]
    fn test_from_manager() {
        let npm_manager = CorePackageManager::from("npm".to_string());
        let yarn_manager = CorePackageManager::from("yarn".to_string());
        let pnpm_manager = CorePackageManager::from("pnpm".to_string());
        let bun_manager = CorePackageManager::from("bun".to_string());

        assert_eq!(npm_manager, CorePackageManager::Npm);
        assert_eq!(yarn_manager, CorePackageManager::Yarn);
        assert_eq!(pnpm_manager, CorePackageManager::Pnpm);
        assert_eq!(bun_manager, CorePackageManager::Bun);
    }

    #[test]
    #[should_panic(expected = "Unable to identify package manager: unknown")]
    fn test_unknown_manager() {
        let _ = CorePackageManager::from("unknown".to_string());
    }
}
