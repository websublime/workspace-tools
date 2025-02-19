#![allow(clippy::bind_instead_of_map)]
#![allow(clippy::needless_pass_by_value)]
use napi::{Error, Result, Status};
use std::path::Path;
use ws_std::manager::detect_package_manager;

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

/// Detect the package manager.
///
/// @param {string} cwd - The current working directory.
/// @returns {string} The package manager.
#[napi(js_name = "detectPackageManager", ts_return_type = "Result<PackageManager>")]
#[allow(clippy::manual_let_else)]
pub fn js_detect_manager(cwd: String) -> Result<Option<String>, PackageManagerError> {
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

    Ok(Some(package_manager.to_string().to_lowercase()))
}
