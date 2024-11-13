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
