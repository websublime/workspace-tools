#![allow(clippy::bind_instead_of_map)]
#![allow(clippy::needless_pass_by_value)]
use napi::{Error, Result, Status};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter, Result as StdResult},
    path::Path,
};
use ws_std::manager::{detect_package_manager, CorePackageManager};

pub enum PackageManagerError {
    InvalidPackageManager,
    NapiError(Error<Status>),
}

impl AsRef<str> for PackageManagerError {
    fn as_ref(&self) -> &str {
        match self {
            Self::InvalidPackageManager => "Invalid package manager",
            Self::NapiError(e) => e.status.as_ref(),
        }
    }
}

#[napi(string_enum)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

impl From<CorePackageManager> for PackageManager {
    fn from(manager: CorePackageManager) -> Self {
        match manager {
            CorePackageManager::Npm => Self::Npm,
            CorePackageManager::Yarn => Self::Yarn,
            CorePackageManager::Pnpm => Self::Pnpm,
            CorePackageManager::Bun => Self::Bun,
        }
    }
}

impl Display for PackageManager {
    fn fmt(&self, f: &mut Formatter) -> StdResult {
        match self {
            Self::Npm => write!(f, "npm"),
            Self::Yarn => write!(f, "yarn"),
            Self::Pnpm => write!(f, "pnpm"),
            Self::Bun => write!(f, "bun"),
        }
    }
}

/// Detect the package manager.
///
/// @param {string} cwd - The current working directory.
/// @returns {string} The package manager.
#[napi(js_name = "detectPackageManager", ts_return_type = "Result<PackageManager>")]
#[allow(clippy::manual_let_else)]
pub fn js_detect_manager(cwd: String) -> Result<PackageManager, PackageManagerError> {
    let root = Path::new(&cwd);
    let package_manager = match detect_package_manager(root) {
        Some(pm) => pm,
        None => {
            return Err(Error::new(
                PackageManagerError::InvalidPackageManager,
                "Failed to identify package manager",
            ))
        }
    };

    Ok(PackageManager::from(package_manager))
}
