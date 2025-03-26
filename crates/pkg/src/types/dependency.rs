use crate::VersionError;
use semver::{Version, VersionReq};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::rc::Rc;

/// Dependency represents a package dependency with version requirements
#[derive(Debug, Clone)]
pub struct Dependency {
    pub(crate) name: String,
    pub(crate) version: Rc<RefCell<VersionReq>>,
}

impl Display for Dependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}@{}", self.name, self.version.borrow())
    }
}

impl Dependency {
    pub fn new(name: &str, version: &str) -> Result<Self, VersionError> {
        if version.starts_with('*') | version.contains("workspace:*") {
            return Err(VersionError::InvalidVersion(format!(
                "Looks like you are trying to update a internal package: {version}"
            )));
        };

        let version_req = VersionReq::parse(version).map_err(VersionError::from)?;
        Ok(Self { name: name.to_string(), version: Rc::new(RefCell::new(version_req)) })
    }

    /// Get the dependency name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the version requirement
    pub fn version(&self) -> VersionReq {
        self.version.borrow().clone()
    }

    pub fn fixed_version(&self) -> Result<Version, VersionError> {
        let req_str = self.version.borrow().to_string();
        // Remove operators and parse
        let clean_version = req_str.trim_start_matches(|c| "^~=".contains(c)).trim();

        Version::parse(clean_version).map_err(VersionError::from)
    }

    pub fn compare_versions(&self, other: &str) -> Result<Ordering, VersionError> {
        let self_version = self.fixed_version()?;
        let other_version = Version::parse(other)?;

        Ok(self_version.cmp(&other_version))
    }

    pub fn update_version(&self, new_version: &str) -> Result<(), VersionError> {
        if new_version.starts_with('*') | new_version.contains("workspace:*") {
            return Err(VersionError::InvalidVersion(format!(
                "Looks like you are trying to update a internal package: {new_version}"
            )));
        };

        let new_req = VersionReq::parse(new_version).map_err(VersionError::from)?;
        *self.version.borrow_mut() = new_req;
        Ok(())
    }

    pub fn matches(&self, version: &str) -> Result<bool, VersionError> {
        let version = Version::parse(version).map_err(VersionError::from)?;

        Ok(self.version.borrow().matches(&version))
    }
}
