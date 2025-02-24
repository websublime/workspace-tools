use napi::{
    bindgen_prelude::{FromNapiValue, Object},
    sys, Error, Result,
};
use napi::{Env, Status};
use ws_monorepo::workspace::Workspace as RepoWorkspace;

pub enum WorkspaceError {
    InvalidWorkspaceMetadata,
    FailCreateObject,
    FailSetObjectProperty,
    FailParsing,
    NapiError(Error<Status>),
}

#[napi(js_name = "Workspace")]
pub struct Workspace {
    instance: RepoWorkspace,
}

impl AsRef<str> for WorkspaceError {
    fn as_ref(&self) -> &str {
        match self {
            Self::InvalidWorkspaceMetadata => "Invalid workspace name,version and scope",
            Self::FailCreateObject => "Failed to create object",
            Self::FailSetObjectProperty => "Failed to set object property",
            Self::FailParsing => "Failed to parse struct",
            Self::NapiError(e) => e.status.as_ref(),
        }
    }
}

impl FromNapiValue for Workspace {
    unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
        // Implement conversion from napi value to WorkspaceClass
        let obj = Object::from_napi_value(env, napi_val)?;
        let root: String = obj.get_named_property("root")?;
        Ok(Workspace { instance: RepoWorkspace::from(root.as_str()) })
    }
}

#[napi]
impl Workspace {
    #[napi(constructor)]
    pub fn new(root: String) -> Self {
        Self { instance: RepoWorkspace::from(root.as_str()) }
    }

    #[napi(js_name = "getPackages", ts_return_type = "Result<Array<PackageInfo>>")]
    pub fn get_packages(&self, env: Env) -> Result<Object, WorkspaceError> {
        let packages = self.instance.get_packages();
        let mut package_info_list = env.create_array_with_length(packages.len()).or_else(|_| {
            Err(Error::new(WorkspaceError::FailCreateObject, "Failed to create package info array"))
        })?;

        for (i, package) in packages.iter().enumerate() {
            let mut pkg_object = env.create_object().or_else(|_| {
                Err(Error::new(
                    WorkspaceError::FailCreateObject,
                    "Failed to create package info object",
                ))
            })?;

            let package_value = serde_json::to_value(&package.package).or_else(|_| {
                Err(Error::new(WorkspaceError::FailParsing, "Failed to parse package value"))
            })?;
            pkg_object.set("package", package_value).or_else(|_| {
                Err(Error::new(
                    WorkspaceError::FailSetObjectProperty,
                    "Failed to set package info object property",
                ))
            })?;

            let package_json_path_value = serde_json::to_value(&package.package_json_path)
                .or_else(|_| {
                    Err(Error::new(
                        WorkspaceError::FailParsing,
                        "Failed to parse package json path value",
                    ))
                })?;
            pkg_object.set("packageJsonPath", package_json_path_value).or_else(|_| {
                Err(Error::new(
                    WorkspaceError::FailSetObjectProperty,
                    "Failed to set package json path object property",
                ))
            })?;

            let package_path_value = serde_json::to_value(&package.package_path).or_else(|_| {
                Err(Error::new(WorkspaceError::FailParsing, "Failed to parse package path value"))
            })?;
            pkg_object.set("packagePath", package_path_value).or_else(|_| {
                Err(Error::new(
                    WorkspaceError::FailSetObjectProperty,
                    "Failed to set package path object property",
                ))
            })?;

            let package_relative_path_value = serde_json::to_string(&package.package_relative_path)
                .or_else(|_| {
                    Err(Error::new(
                        WorkspaceError::FailParsing,
                        "Failed to parse package relative path value",
                    ))
                })?;
            pkg_object.set("packageRelativePath", package_relative_path_value).or_else(|_| {
                Err(Error::new(
                    WorkspaceError::FailSetObjectProperty,
                    "Failed to set package relative path object property",
                ))
            })?;

            pkg_object.set("packageJson", package.pkg_json.clone()).or_else(|_| {
                Err(Error::new(
                    WorkspaceError::FailSetObjectProperty,
                    "Failed to set package scope name version object property",
                ))
            })?;

            package_info_list.set_element(i as u32, pkg_object).or_else(|_| {
                Err(Error::new(
                    WorkspaceError::FailSetObjectProperty,
                    "Failed to set package info object property",
                ))
            })?;
        }

        Ok(package_info_list)
    }
}
