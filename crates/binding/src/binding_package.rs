use napi::{
    bindgen_prelude::{FromNapiValue, Object},
    sys, Error, Result,
};
use napi::{Env, Status};
use ws_pkg::dependency::Node;
use ws_pkg::package::{
    package_scope_name_version, Dependency as RepoDependency, Package as RepoPackage,
};

pub enum PackageError {
    InvalidPackageMetadata,
    FailCreateObject,
    FailSetObjectProperty,
    FailParsing,
    NapiError(Error<Status>),
}

#[napi(js_name = "Package")]
pub struct Package {
    instance: RepoPackage,
}

#[napi(js_name = "Dependency")]
#[derive(Clone)]
pub struct Dependency {
    instance: RepoDependency,
}

impl AsRef<str> for PackageError {
    fn as_ref(&self) -> &str {
        match self {
            Self::InvalidPackageMetadata => "Invalid package name,version and scope",
            Self::FailCreateObject => "Failed to create object",
            Self::FailSetObjectProperty => "Failed to set object property",
            Self::FailParsing => "Failed to parse struct",
            Self::NapiError(e) => e.status.as_ref(),
        }
    }
}

#[napi]
impl Dependency {
    #[napi(constructor)]
    pub fn new(name: String, version: String) -> Self {
        Self { instance: RepoDependency { name, version: version.parse().unwrap() } }
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        self.instance.name.to_string()
    }

    #[napi(getter)]
    pub fn version(&self) -> String {
        self.instance.version.to_string()
    }
}

impl FromNapiValue for Dependency {
    unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
        // Implement conversion from napi value to DependencyClass
        let obj = Object::from_napi_value(env, napi_val)?;
        let name: String = obj.get_named_property("name")?;
        let version: String = obj.get_named_property("version")?;
        Ok(Dependency { instance: RepoDependency { name, version: version.parse().unwrap() } })
    }
}

#[napi]
impl Package {
    #[napi(constructor)]
    pub fn new(
        name: String,
        version: String,
        #[napi(ts_arg_type = "Array<Dependency>")] deps: Option<Vec<Dependency>>,
    ) -> Self {
        let deps = deps.map(|deps| deps.into_iter().map(|dep| dep.instance).collect());
        Self { instance: RepoPackage::new(name.as_str(), version.as_str(), deps) }
    }

    #[napi(js_name = "updateVersion")]
    pub fn update_version(&mut self, version: String) {
        self.instance.update_version(version.as_str());
    }

    #[napi(js_name = "updateDependencyVersion")]
    pub fn update_dependency_version(&mut self, name: String, version: String) {
        self.instance.update_dependency_version(name.as_str(), version.as_str());
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        self.instance.name.to_string()
    }

    #[napi(getter)]
    pub fn version(&self) -> String {
        self.instance.version.to_string()
    }

    #[napi(getter)]
    pub fn dependencies(&self) -> Vec<Dependency> {
        self.instance
            .dependencies()
            .iter()
            .map(|dep| Dependency { instance: dep.clone() })
            .collect()
    }
}

/// Get package scope name version and path
///
/// @param {string} pk_name_scope_name_version - The package name, version and optional file path.
/// @returns {Object} - The package scope name version and path.
#[napi(js_name = "getPackageScopeNameVersion", ts_return_type = "Result<PackageScopeMetadata>")]
pub fn js_package_scope_name_version(
    env: Env,
    pk_name_scope_name_version: String,
) -> Result<Option<Object>, PackageError> {
    let mut scope_metadata_object = env.create_object().or_else(|_| {
        Err(Error::new(PackageError::FailCreateObject, "Failed to create metadata object"))
    })?;

    let scope_metadata = package_scope_name_version(pk_name_scope_name_version.as_str());

    match scope_metadata {
        Some(metadata) => {
            let full_value = serde_json::to_value(metadata.full).or_else(|_| {
                Err(Error::new(PackageError::FailParsing, "Failed to parse full value"))
            })?;
            scope_metadata_object.set("full", full_value).or_else(|_| {
                Err(Error::new(
                    PackageError::FailSetObjectProperty,
                    "Failed to set full object property",
                ))
            })?;

            let name_value = serde_json::to_value(metadata.name).or_else(|_| {
                Err(Error::new(PackageError::FailParsing, "Failed to parse name value"))
            })?;
            scope_metadata_object.set("name", name_value).or_else(|_| {
                Err(Error::new(
                    PackageError::FailSetObjectProperty,
                    "Failed to set name object property",
                ))
            })?;

            let version_value = serde_json::to_value(metadata.version).or_else(|_| {
                Err(Error::new(PackageError::FailParsing, "Failed to parse version value"))
            })?;
            scope_metadata_object.set("version", version_value).or_else(|_| {
                Err(Error::new(
                    PackageError::FailSetObjectProperty,
                    "Failed to set version object property",
                ))
            })?;

            let version_value =
                serde_json::to_value(metadata.path.unwrap_or_default()).or_else(|_| {
                    Err(Error::new(PackageError::FailParsing, "Failed to parse path value"))
                })?;
            scope_metadata_object.set("path", version_value).or_else(|_| {
                Err(Error::new(
                    PackageError::FailSetObjectProperty,
                    "Failed to set path object property",
                ))
            })?;

            Ok(Some(scope_metadata_object))
        }
        None => {
            return Ok(None);
        }
    }
}
