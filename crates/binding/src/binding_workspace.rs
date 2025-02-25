use napi::{
    bindgen_prelude::{FromNapiValue, Object},
    sys, Error, Result,
};
use napi::{Env, Status};
use serde::{Deserialize, Serialize};
use ws_monorepo::workspace::Workspace as RepoWorkspace;
use ws_pkg::bump::BumpOptions as RepoBumpOptions;
use ws_pkg::version::Version;

pub enum WorkspaceError {
    InvalidWorkspaceMetadata,
    PackageInfoNotFound,
    FailCreateObject,
    FailSetObjectProperty,
    FailParsing,
    NapiError(Error<Status>),
}

#[napi(js_name = "Workspace")]
pub struct Workspace {
    instance: RepoWorkspace,
}

#[napi(object)]
#[derive(Debug, Deserialize, Serialize)]
pub struct BumpOptions {
    pub since: Option<String>,
    pub release_as: Option<String>,
    pub fetch_all: Option<bool>,
    pub fetch_tags: Option<bool>,
    pub sync_deps: Option<bool>,
    pub push: Option<bool>,
}

impl AsRef<str> for WorkspaceError {
    fn as_ref(&self) -> &str {
        match self {
            Self::InvalidWorkspaceMetadata => "Invalid workspace name,version and scope",
            Self::FailCreateObject => "Failed to create object",
            Self::FailSetObjectProperty => "Failed to set object property",
            Self::FailParsing => "Failed to parse struct",
            Self::PackageInfoNotFound => "Package info not found",
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

            let package_relative_path_value = serde_json::to_value(&package.package_relative_path)
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

    #[napi(js_name = "getPackageInfo", ts_return_type = "Result<PackageInfo>")]
    pub fn get_package_info(
        &self,
        env: Env,
        package_name: String,
    ) -> Result<Object, WorkspaceError> {
        let package_info =
            self.instance.get_package_info(package_name.as_str()).ok_or_else(|| {
                Error::new(WorkspaceError::PackageInfoNotFound, "Failed to get package info")
            })?;

        let mut pkg_object = env.create_object().or_else(|_| {
            Err(Error::new(
                WorkspaceError::FailCreateObject,
                "Failed to create package info object",
            ))
        })?;

        let package_value = serde_json::to_value(&package_info.package).or_else(|_| {
            Err(Error::new(WorkspaceError::FailParsing, "Failed to parse package value"))
        })?;
        pkg_object.set("package", package_value).or_else(|_| {
            Err(Error::new(
                WorkspaceError::FailSetObjectProperty,
                "Failed to set package info object property",
            ))
        })?;

        let package_json_path_value = serde_json::to_value(&package_info.package_json_path)
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

        let package_path_value =
            serde_json::to_value(&package_info.package_path).or_else(|_| {
                Err(Error::new(WorkspaceError::FailParsing, "Failed to parse package path value"))
            })?;
        pkg_object.set("packagePath", package_path_value).or_else(|_| {
            Err(Error::new(
                WorkspaceError::FailSetObjectProperty,
                "Failed to set package path object property",
            ))
        })?;

        let package_relative_path_value = serde_json::to_value(&package_info.package_relative_path)
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

        pkg_object.set("packageJson", package_info.pkg_json.clone()).or_else(|_| {
            Err(Error::new(
                WorkspaceError::FailSetObjectProperty,
                "Failed to set package scope name version object property",
            ))
        })?;

        Ok(pkg_object)
    }

    #[napi(js_name = "getChangedPackages", ts_return_type = "Result<Array<PackageInfo>>")]
    pub fn get_changed_packages(
        &self,
        env: Env,
        sha: Option<String>,
    ) -> Result<Object, WorkspaceError> {
        let (pkgs, changed_files) = self.instance.get_changed_packages(sha);

        let mut package_info_list = env.create_array_with_length(pkgs.len()).or_else(|_| {
            Err(Error::new(WorkspaceError::FailCreateObject, "Failed to create package info array"))
        })?;

        for (i, package) in pkgs.iter().enumerate() {
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

            let package_relative_path_value = serde_json::to_value(&package.package_relative_path)
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

            let pkg_changed_files =
                changed_files.get(&package.package.name).cloned().unwrap_or_default();

            let changed_files_value = serde_json::to_value(pkg_changed_files).or_else(|_| {
                Err(Error::new(WorkspaceError::FailParsing, "Failed to parse changed files value"))
            })?;

            pkg_object.set("changedFiles", changed_files_value).or_else(|_| {
                Err(Error::new(
                    WorkspaceError::FailSetObjectProperty,
                    "Failed to set changed files object property",
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

    #[napi(js_name = "getPackageRecommendBump", ts_return_type = "Result<RecommendBumpPackage>")]
    pub fn get_package_recommend_bump(
        &self,
        env: Env,
        package_name: String,
        bump_options: Option<BumpOptions>,
    ) -> Result<Object, WorkspaceError> {
        let repo_bump_options = if let Some(options) = bump_options {
            RepoBumpOptions {
                since: options.since,
                release_as: options.release_as.map(|s| Version::from(s.as_str())),
                fetch_all: options.fetch_all,
                fetch_tags: options.fetch_tags,
                sync_deps: options.sync_deps,
                push: options.push,
            }
        } else {
            RepoBumpOptions {
                since: None,
                release_as: None,
                fetch_all: None,
                fetch_tags: None,
                sync_deps: None,
                push: None,
            }
        };

        let package_info =
            self.instance.get_package_info(package_name.as_str()).ok_or_else(|| {
                Error::new(WorkspaceError::PackageInfoNotFound, "Failed to get package info")
            })?;

        let recommend =
            self.instance.get_package_recommend_bump(&package_info, Some(repo_bump_options));

        let mut pkg_object = env.create_object().or_else(|_| {
            Err(Error::new(
                WorkspaceError::FailCreateObject,
                "Failed to create package recommend bump object",
            ))
        })?;

        let from_value = serde_json::to_value(&recommend.from).or_else(|_| {
            Err(Error::new(WorkspaceError::FailParsing, "Failed to parse from value"))
        })?;
        pkg_object.set("from", from_value).or_else(|_| {
            Err(Error::new(
                WorkspaceError::FailSetObjectProperty,
                "Failed to set package recommend bump object property",
            ))
        })?;

        let to_value = serde_json::to_value(&recommend.to).or_else(|_| {
            Err(Error::new(WorkspaceError::FailParsing, "Failed to parse to value"))
        })?;
        pkg_object.set("to", to_value).or_else(|_| {
            Err(Error::new(
                WorkspaceError::FailSetObjectProperty,
                "Failed to set package recommend bump object property",
            ))
        })?;

        let package_info_value = serde_json::to_value(&recommend.package_info).or_else(|_| {
            Err(Error::new(WorkspaceError::FailParsing, "Failed to parse package info value"))
        })?;
        pkg_object.set("packageInfo", package_info_value).or_else(|_| {
            Err(Error::new(
                WorkspaceError::FailSetObjectProperty,
                "Failed to set package recommend bump object property",
            ))
        })?;

        let conventional_value = serde_json::to_value(&recommend.conventional).or_else(|_| {
            Err(Error::new(WorkspaceError::FailParsing, "Failed to parse conventional value"))
        })?;
        pkg_object.set("conventional", conventional_value).or_else(|_| {
            Err(Error::new(
                WorkspaceError::FailSetObjectProperty,
                "Failed to set package recommend bump object property",
            ))
        })?;

        let deploy_value = serde_json::to_value(&recommend.deploy_to).or_else(|_| {
            Err(Error::new(WorkspaceError::FailParsing, "Failed to parse deploy value"))
        })?;
        pkg_object.set("deployTo", deploy_value).or_else(|_| {
            Err(Error::new(
                WorkspaceError::FailSetObjectProperty,
                "Failed to set package recommend bump object property",
            ))
        })?;

        let changed_files_value = serde_json::to_value(&recommend.changed_files).or_else(|_| {
            Err(Error::new(WorkspaceError::FailParsing, "Failed to parse changed files value"))
        })?;
        pkg_object.set("changedFiles", changed_files_value).or_else(|_| {
            Err(Error::new(
                WorkspaceError::FailSetObjectProperty,
                "Failed to set changed files object property",
            ))
        })?;

        Ok(pkg_object)
    }
}
