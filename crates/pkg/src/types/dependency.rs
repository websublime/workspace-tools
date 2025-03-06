//! Dependency type and related functionality.

use crate::error::{PkgError, Result};
use semver::VersionReq;
use std::cell::RefCell;
use std::rc::Rc;

/// Dependency represents a package dependency with version requirements
#[derive(Debug, Clone)]
pub struct Dependency {
    pub(crate) name: String,
    pub(crate) version: Rc<RefCell<VersionReq>>,
}

impl Dependency {
    /// Create a new dependency with name and version
    pub fn new(name: &str, version: &str) -> Result<Self> {
        let parsed_version = if version.starts_with('^') || version.starts_with('~') {
            version.parse().map_err(|e| PkgError::VersionReqParseError {
                requirement: version.to_string(),
                source: e,
            })?
        } else {
            let req_str = format!("^{version}");
            req_str
                .parse()
                .map_err(|e| PkgError::VersionReqParseError { requirement: req_str, source: e })?
        };

        Ok(Self { name: name.to_string(), version: Rc::new(RefCell::new(parsed_version)) })
    }

    /// Get the dependency name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the version requirement
    pub fn version(&self) -> VersionReq {
        self.version.borrow().clone()
    }

    /// Get the version requirement as a string
    pub fn version_str(&self) -> String {
        self.version.borrow().to_string()
    }

    /// Update the version requirement
    pub fn update_version(&self, new_version: &str) -> Result<()> {
        let parsed_version = if new_version.starts_with('^') || new_version.starts_with('~') {
            new_version.parse().map_err(|e| PkgError::VersionReqParseError {
                requirement: new_version.to_string(),
                source: e,
            })?
        } else {
            let req_str = format!("^{new_version}");
            req_str
                .parse()
                .map_err(|e| PkgError::VersionReqParseError { requirement: req_str, source: e })?
        };
        *self.version.borrow_mut() = parsed_version;
        Ok(())
    }
}
