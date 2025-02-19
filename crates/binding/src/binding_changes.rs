#![allow(clippy::bind_instead_of_map)]
#![allow(clippy::needless_pass_by_value)]
use napi::{bindgen_prelude::Object, Error, Result};
use napi::{Env, Status};
use std::path::PathBuf;

use ws_monorepo::{
    changes::{Change, Changes},
    config::get_workspace_config,
};

pub enum ChangesError {
    InvalidPackageProperty,
    InvalidReleaseAsProperty,
    InvalidChange,
    FailCreateObject,
    FailSetObjectProperty,
    FailParsing,
    NapiError(Error<Status>),
}

impl AsRef<str> for ChangesError {
    fn as_ref(&self) -> &str {
        match self {
            Self::InvalidPackageProperty => "Invalid package property",
            Self::InvalidReleaseAsProperty => "Invalid releaseAs property",
            Self::InvalidChange => "Invalid change",
            Self::FailCreateObject => "Failed to create object",
            Self::FailSetObjectProperty => "Failed to set object property",
            Self::FailParsing => "Failed to parse struct",
            Self::NapiError(e) => e.status.as_ref(),
        }
    }
}

#[napi(js_name = "initChanges", ts_return_type = "Result<Changes>")]
pub fn js_init_changes(env: Env, cwd: Option<String>) -> Result<Object, ChangesError> {
    let mut changes_object = env.create_object().or_else(|_| {
        Err(Error::new(ChangesError::FailCreateObject, "Failed to create changes object"))
    })?;

    let root = cwd.map(PathBuf::from);

    let config = &get_workspace_config(root);
    let changes = Changes::from(config);

    let data = changes.init();

    data.changes.iter().for_each(|(key, change)| {
        let value = serde_json::to_value(change)
            .or_else(|_| {
                Err(Error::new(ChangesError::FailParsing, "Failed to parse changes struct"))
            })
            .unwrap();
        changes_object
            .set(key.as_str(), value)
            .or_else(|_| {
                Err(Error::new(
                    ChangesError::FailSetObjectProperty,
                    "Failed to set branch object property",
                ))
            })
            .unwrap();
    });

    Ok(changes_object)
}

#[napi(
    js_name = "addChange",
    ts_args_type = "change: Change, deploy_envs?: string[], cwd?: string",
    ts_return_type = "Result<boolean>"
)]
pub fn js_add_change(
    change: Object,
    deploy_envs: Option<Vec<String>>,
    cwd: Option<String>,
) -> Result<bool, ChangesError> {
    let package_name = change.get_named_property::<String>("package").or_else(|_| {
        Err(Error::new(ChangesError::InvalidPackageProperty, "Failed to get package property"))
    })?;

    let release_as = change.get_named_property::<String>("releaseAs").or_else(|_| {
        Err(Error::new(ChangesError::InvalidReleaseAsProperty, "Failed to get releaseAs property"))
    })?;

    let envs = deploy_envs.unwrap_or_default();
    let change = &Change { package: package_name, release_as };
    let root = cwd.map(PathBuf::from);
    let config = &get_workspace_config(root);
    let changes = Changes::from(config);

    Ok(changes.add(change, Some(envs)))
}

#[napi(js_name = "removeChange", ts_args_type = "branch: string, cwd?: string")]
pub fn js_remove_change(branch: String, cwd: Option<String>) -> bool {
    let root = cwd.map(PathBuf::from);
    let config = &get_workspace_config(root);
    let changes = Changes::from(config);

    changes.remove(branch.as_str())
}

#[napi(js_name = "getChanges", ts_args_type = "cwd?: string", ts_return_type = "Result<Changes>")]
pub fn js_get_changes(env: Env, cwd: Option<String>) -> Result<Object, ChangesError> {
    let mut changes_object = env.create_object().or_else(|_| {
        Err(Error::new(ChangesError::FailCreateObject, "Failed to create changes object"))
    })?;

    let root = cwd.map(PathBuf::from);
    let config = &get_workspace_config(root);
    let changes = Changes::from(config);

    let data = changes.changes();

    data.iter().for_each(|(key, change)| {
        let value = serde_json::to_value(change)
            .or_else(|_| {
                Err(Error::new(ChangesError::FailParsing, "Failed to parse changes struct"))
            })
            .unwrap();
        changes_object
            .set(key.as_str(), value)
            .or_else(|_| {
                Err(Error::new(
                    ChangesError::FailSetObjectProperty,
                    "Failed to set branch object property",
                ))
            })
            .unwrap();
    });

    Ok(changes_object)
}

#[napi(
    js_name = "getChangesByBranch",
    ts_args_type = "branch: string, cwd?: string",
    ts_return_type = "Result<{deploy: string[]; pkgs: Changes[]}|null>"
)]
pub fn js_get_change_by_branch(
    env: Env,
    branch: String,
    cwd: Option<String>,
) -> Result<Option<Object>, ChangesError> {
    let root = cwd.map(PathBuf::from);
    let config = &get_workspace_config(root);
    let changes = Changes::from(config);

    let change_meta = changes.changes_by_branch(branch.as_str());

    if change_meta.is_none() {
        return Ok(None);
    }

    let change = change_meta.ok_or_else(|| {
        Error::new(ChangesError::InvalidChange, format!("Invalid change for branch {branch}"))
    })?;

    let mut change_object = env.create_object().or_else(|_| {
        Err(Error::new(ChangesError::FailCreateObject, "Failed to create changes object"))
    })?;
    let deploy_value = serde_json::to_value(change.deploy)
        .or_else(|_| Err(Error::new(ChangesError::FailParsing, "Failed to parse deploy value")))?;
    let pkgs_value = serde_json::to_value(change.pkgs)
        .or_else(|_| Err(Error::new(ChangesError::FailParsing, "Failed to parse pkgs value")))?;

    change_object.set("deploy", deploy_value).or_else(|_| {
        Err(Error::new(ChangesError::FailSetObjectProperty, "Failed to set deploy object property"))
    })?;

    change_object.set("pkgs", pkgs_value).or_else(|_| {
        Err(Error::new(ChangesError::FailSetObjectProperty, "Failed to set pkgs object property"))
    })?;

    Ok(Some(change_object))
}

#[napi(
    js_name = "getChangesByPackage",
    ts_args_type = "package: string, branch: string, cwd?: string",
    ts_return_type = "Result<Change|null>"
)]
pub fn js_get_changes_by_package(
    env: Env,
    package: String,
    branch: String,
    cwd: Option<String>,
) -> Result<Option<Object>, ChangesError> {
    let root = cwd.map(PathBuf::from);
    let config = &get_workspace_config(root);
    let changes = Changes::from(config);

    let change_meta = changes.changes_by_package(package.as_str(), branch.as_str());

    if change_meta.is_none() {
        return Ok(None);
    }

    let change = change_meta.ok_or_else(|| {
        Error::new(ChangesError::InvalidChange, format!("Invalid change for package {package}"))
    })?;

    let mut change_object = env.create_object().or_else(|_| {
        Err(Error::new(ChangesError::FailCreateObject, "Failed to create changes object"))
    })?;

    let package_value = serde_json::to_value(change.package)
        .or_else(|_| Err(Error::new(ChangesError::FailParsing, "Failed to parse package value")))?;
    let release_value = serde_json::to_value(change.release_as)
        .or_else(|_| Err(Error::new(ChangesError::FailParsing, "Failed to parse release value")))?;

    change_object.set("package", package_value).or_else(|_| {
        Err(Error::new(
            ChangesError::FailSetObjectProperty,
            "Failed to set package object property",
        ))
    })?;

    change_object.set("releaseAs", release_value).or_else(|_| {
        Err(Error::new(
            ChangesError::FailSetObjectProperty,
            "Failed to set releaseAs object property",
        ))
    })?;

    Ok(Some(change_object))
}

#[napi(js_name = "changeExists", ts_args_type = "branch: string, package: string, cwd?: string")]
pub fn js_change_exists(branch: String, package: String, cwd: Option<String>) -> bool {
    let root = cwd.map(PathBuf::from);
    let config = &get_workspace_config(root);
    let changes = Changes::from(config);

    changes.exist(branch.as_str(), package.as_str())
}
