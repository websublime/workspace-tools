use regex::Regex;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::dependency::DependencyGraph;

use super::dependency::Node;

// Inner data structure for Dependency
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DependencyData {
    pub name: String,
    pub version: VersionReq,
}

// Public Dependency wrapper
#[derive(Debug, Clone)]
pub struct Dependency {
    inner: Rc<RefCell<DependencyData>>,
}

// This is a helper struct to match the DependencyType requirement
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencyInfo {
    pub name: String,
    pub version: VersionReq,
}

// Inner data structure for Package
#[derive(Debug, Clone)]
struct PackageData {
    pub name: String,
    pub version: Version,
    dependencies: Vec<Dependency>,
}

// Public Package wrapper
#[derive(Debug, Clone)]
pub struct Package {
    inner: Rc<RefCell<PackageData>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageInfo {
    pub package: Package,
    pub package_json_path: String,
    pub package_path: String,
    pub package_relative_path: String,
    pub pkg_json: Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<Vec<String>>,
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_dependencies: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_dependencies: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_dependencies: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engines: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripts: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Package scope metadata extracted from a package name.
pub struct PackageScopeMetadata {
    pub full: String,
    pub name: String,
    pub version: String,
    pub path: Option<String>,
}

/// Extracts the package scope name and version from a package name.
pub fn package_scope_name_version(pkg_name: &str) -> Option<PackageScopeMetadata> {
    let regex = Regex::new("^((?:@[^/@]+/)?[^/@]+)(?:@([^/]+))?(/.*)?$").expect("Invalid regex");

    let matches = regex.captures(pkg_name).expect("Invalid package name");

    Some(PackageScopeMetadata {
        full: matches.get(0).map_or("", |m| m.as_str()).to_string(),
        name: matches.get(1).map_or("", |m| m.as_str()).to_string(),
        version: matches.get(2).map_or("", |m| m.as_str()).to_string(),
        path: matches.get(3).map(|m| m.as_str().to_string()),
    })
}

pub fn build_dependency_graph_from_packages(packages: &[Package]) -> DependencyGraph<'_, Package> {
    DependencyGraph::from(packages)
}

pub fn build_dependency_graph_from_package_infos<'p>(
    package_infos: &[PackageInfo],
    packages: &'p mut Vec<Package>,
) -> DependencyGraph<'p, Package> {
    // Clear and refill the packages vector
    packages.clear();
    packages.extend(package_infos.iter().map(|pkg_info| pkg_info.package.clone()));

    // Create the dependency graph from the packages slice
    DependencyGraph::from(packages.as_slice())
}

impl Dependency {
    pub fn new(name: &str, version: &str) -> Self {
        let version_req = VersionReq::parse(version).expect("Invalid version requirement");
        Self {
            inner: Rc::new(RefCell::new(DependencyData {
                name: name.to_string(),
                version: version_req,
            })),
        }
    }

    pub fn name(&self) -> String {
        self.inner.borrow().name.clone()
    }

    pub fn version(&self) -> VersionReq {
        self.inner.borrow().version.clone()
    }

    pub fn version_str(&self) -> String {
        self.inner.borrow().version.to_string()
    }

    pub fn update_version(&self, version: &str) {
        let version_req = VersionReq::parse(version).expect("Invalid version requirement");
        self.inner.borrow_mut().version = version_req;
    }
}

impl Display for Dependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.name(), self.version_str())
    }
}

// Implement serialization for Dependency
impl Serialize for Dependency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let data = self.inner.borrow();
        let mut state = serializer.serialize_struct("Dependency", 2)?;
        state.serialize_field("name", &data.name)?;
        state.serialize_field("version", &data.version.to_string())?;
        state.end()
    }
}

// Implement deserialization for Dependency
impl<'de> Deserialize<'de> for Dependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct DependencyHelper {
            name: String,
            version: String,
        }

        let helper = DependencyHelper::deserialize(deserializer)?;
        Ok(Dependency::new(&helper.name, &helper.version))
    }
}

impl Package {
    pub fn new(name: &str, version: &str, deps: Option<Vec<Dependency>>) -> Self {
        let version = Version::parse(version).expect("Invalid version");
        Self {
            inner: Rc::new(RefCell::new(PackageData {
                name: name.to_string(),
                version,
                dependencies: deps.unwrap_or_default(),
            })),
        }
    }

    pub fn name(&self) -> String {
        self.inner.borrow().name.clone()
    }

    pub fn version(&self) -> Version {
        self.inner.borrow().version.clone()
    }

    pub fn version_str(&self) -> String {
        self.inner.borrow().version.to_string()
    }

    pub fn dependencies(&self) -> Vec<Dependency> {
        self.inner.borrow().dependencies.clone()
    }

    pub fn update_version(&self, version: &str) {
        let version = Version::parse(version).expect("Invalid version");
        self.inner.borrow_mut().version = version;
    }

    pub fn update_dependency_version(&self, name: &str, version: &str) {
        let inner = self.inner.borrow();
        for dep in &inner.dependencies {
            if dep.name() == name {
                dep.update_version(version);
            }
        }
    }
}

// Implement serialization for Package
impl Serialize for Package {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let data = self.inner.borrow();
        let mut state = serializer.serialize_struct("Package", 3)?;
        state.serialize_field("name", &data.name)?;
        state.serialize_field("version", &data.version.to_string())?;
        state.serialize_field("dependencies", &data.dependencies)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Package {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PackageHelper {
            name: String,
            version: String,
            dependencies: Option<Vec<Dependency>>,
        }

        let helper = PackageHelper::deserialize(deserializer)?;
        Ok(Package::new(&helper.name, &helper.version, helper.dependencies))
    }
}

impl PackageInfo {
    pub fn new(
        package: Package,
        package_json_path: String,
        package_path: String,
        package_relative_path: String,
        pkg_json: Value,
    ) -> Self {
        Self { package, package_json_path, package_path, package_relative_path, pkg_json }
    }

    pub fn get_dependency_graph(&self) -> DependencyGraph<'_, Package> {
        let packages = std::slice::from_ref(&self.package);
        DependencyGraph::from(packages)
    }

    pub fn update_version(&mut self, version: &str) {
        let version = Version::parse(version).expect("Invalid version");

        self.pkg_json["version"] = serde_json::Value::String(version.to_string());
        self.package.update_version(version.to_string().as_str());
    }

    pub fn update_dependency_version(&mut self, name: &str, version: &str) {
        let version = Version::parse(version).expect("Invalid version");

        if let Some(deps) = self.pkg_json["dependencies"].as_object_mut() {
            if deps.contains_key(name) {
                deps.insert(name.to_string(), serde_json::Value::String(version.to_string()));
                self.package.update_dependency_version(name, version.to_string().as_str());
            }
        }

        self.update_dev_dependency_version(name, version.to_string().as_str());
    }

    pub(crate) fn update_dev_dependency_version(&mut self, name: &str, version: &str) {
        let version = Version::parse(version).expect("Invalid version");

        if let Some(dev_deps) =
            self.pkg_json.get_mut("devDependencies").and_then(|v| v.as_object_mut())
        {
            if dev_deps.contains_key(name) {
                dev_deps.insert(name.to_string(), serde_json::Value::String(version.to_string()));
                self.package.update_dependency_version(name, version.to_string().as_str());
            }
        }
    }

    pub fn write_package_json(&self) -> std::io::Result<()> {
        let package_json_file = std::fs::File::create(&self.package_json_path)?;
        let package_json_writer = std::io::BufWriter::new(package_json_file);

        serde_json::to_writer_pretty(package_json_writer, &self.pkg_json)?;
        Ok(())
    }
}

impl Node for Package {
    type DependencyType = DependencyInfo;
    type Identifier = String;

    fn dependencies(&self) -> &[Self::DependencyType] {
        // This can't be implemented properly with Rc<RefCell<>>
        &[]
    }

    fn dependencies_vec(&self) -> Vec<Self::DependencyType> {
        self.dependencies()
            .iter()
            .map(|dep| DependencyInfo { name: dep.name(), version: dep.version() })
            .collect()
    }

    fn matches(&self, dependency: &Self::DependencyType) -> bool {
        let version = self.version();
        self.name() == dependency.name && dependency.version.matches(&version)
    }

    fn identifier(&self) -> Self::Identifier {
        self.name()
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.name(), self.version_str())
    }
}
