use regex::Regex;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Display;

use super::dependency::Node;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Dependency {
    pub name: String,
    pub version: VersionReq,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
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

impl Package {
    pub fn update_dependency(&mut self, version: &str) {
        let version = Version::parse(version).expect("Invalid version");

        self.version = version;
    }

    pub fn update_dependency_version(&mut self, name: &str, version: &str) {
        let version = Version::parse(version).expect("Invalid version");
        let has_dependency = self.dependencies.iter().any(|dep| dep.name == name);

        if has_dependency {
            self.dependencies.iter_mut().for_each(|dep| {
                if dep.name == name {
                    dep.version = VersionReq::parse(&version.to_string()).unwrap();
                }
            });
        }
    }

    pub fn update_dev_dependency_version(&mut self, name: &str, version: &str) {
        let version = Version::parse(version).expect("Invalid version");
        let has_dependency = self.dependencies.iter().any(|dep| dep.name == name);

        if has_dependency {
            self.dependencies.iter_mut().for_each(|dep| {
                if dep.name == name {
                    dep.version = VersionReq::parse(&version.to_string()).unwrap();
                }
            });
        }
    }
}

impl PackageInfo {
    pub fn update_dependency(&mut self, version: &str) {
        let version = Version::parse(version).expect("Invalid version");

        self.pkg_json["version"] = serde_json::Value::String(version.to_string());
    }

    pub fn update_dependency_version(&mut self, name: &str, version: &str) {
        let version = Version::parse(version).expect("Invalid version");

        let dependencies = self.pkg_json["dependencies"].as_object_mut().unwrap();
        let has_dependency = dependencies.contains_key(name);

        if has_dependency {
            dependencies.insert(name.to_string(), serde_json::Value::String(version.to_string()));
        }
    }

    pub fn update_dev_dependency_version(&mut self, name: &str, version: &str) {
        let version = Version::parse(version).expect("Invalid version");
        let package_json = self.pkg_json.as_object().unwrap();

        if package_json.contains_key("devDependencies") {
            let dependencies = self.pkg_json["devDependencies"].as_object_mut().unwrap();
            let has_dependency = dependencies.contains_key(name);

            if has_dependency {
                dependencies
                    .insert(name.to_string(), serde_json::Value::String(version.to_string()));
            }
        }
    }
}

impl Node for PackageInfo {
    type DependencyType = Dependency;

    fn dependencies(&self) -> &[Self::DependencyType] {
        &self.package.dependencies[..]
    }

    fn matches(&self, dependency: &Self::DependencyType) -> bool {
        let dependency_version =
            semver::VersionReq::parse(&dependency.version.to_string()).unwrap();
        let self_version = semver::Version::parse(&self.package.version.to_string()).unwrap();

        // Check that name is an exact match, and that the dependency
        // requirements are fulfilled by our own version
        self.package.name == dependency.name && dependency_version.matches(&self_version)
    }
}

impl Node for Package {
    type DependencyType = Dependency;

    fn dependencies(&self) -> &[Self::DependencyType] {
        &self.dependencies[..]
    }

    fn matches(&self, dependency: &Self::DependencyType) -> bool {
        let dependency_version =
            semver::VersionReq::parse(&dependency.version.to_string()).unwrap();
        let self_version = semver::Version::parse(&self.version.to_string()).unwrap();

        // Check that name is an exact match, and that the dependency
        // requirements are fulfilled by our own version
        self.name == dependency.name && dependency_version.matches(&self_version)
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}
