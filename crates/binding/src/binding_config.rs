#![allow(clippy::bind_instead_of_map)]
#![allow(clippy::needless_pass_by_value)]
use napi::{bindgen_prelude::Object, Error, Result};
use napi::{Env, Status};
use std::path::PathBuf;

use ws_monorepo::config::get_workspace_config;

pub enum ConfigError {
    FailCreateObject,
    FailSetObjectProperty,
    FailParsing,
    NapiError(Error<Status>),
}

impl AsRef<str> for ConfigError {
    fn as_ref(&self) -> &str {
        match self {
            Self::FailCreateObject => "Failed to create object",
            Self::FailSetObjectProperty => "Failed to set object property",
            Self::FailParsing => "Failed to parse struct",
            Self::NapiError(e) => e.status.as_ref(),
        }
    }
}

#[napi(
    js_name = "getConfig",
    ts_args_type = "cwd?: string",
    ts_return_type = "Result<WorkspaceConfig>"
)]
pub fn js_get_config(env: Env, cwd: Option<String>) -> Result<Object, ConfigError> {
    let root = cwd.map(PathBuf::from);

    let config = &get_workspace_config(root);

    let mut config_object = env.create_object().or_else(|_| {
        Err(Error::new(ConfigError::FailCreateObject, "Failed to create config object"))
    })?;

    let changes_config = serde_json::to_value(config.changes_config.clone()).or_else(|_| {
        Err(Error::new(ConfigError::FailParsing, "Failed to parse changes config struct"))
    })?;
    let cliff_config = serde_json::to_value(config.cliff_config.clone()).or_else(|_| {
        Err(Error::new(ConfigError::FailParsing, "Failed to parse cliff config struct"))
    })?;
    let manager_config = serde_json::to_value(config.package_manager).or_else(|_| {
        Err(Error::new(ConfigError::FailParsing, "Failed to parse manager config struct"))
    })?;
    let tools_config = serde_json::to_value(config.tools_config.clone()).or_else(|_| {
        Err(Error::new(ConfigError::FailParsing, "Failed to parse tools config struct"))
    })?;
    let root_config = serde_json::to_value(config.workspace_root.clone()).or_else(|_| {
        Err(Error::new(ConfigError::FailParsing, "Failed to parse root config struct"))
    })?;

    config_object.set("changesConfig", changes_config).or_else(|_| {
        Err(Error::new(
            ConfigError::FailSetObjectProperty,
            "Failed to set changesConfig object property",
        ))
    })?;
    config_object.set("cliffConfig", cliff_config).or_else(|_| {
        Err(Error::new(
            ConfigError::FailSetObjectProperty,
            "Failed to set cliffConfig object property",
        ))
    })?;
    config_object.set("packageManager", manager_config).or_else(|_| {
        Err(Error::new(
            ConfigError::FailSetObjectProperty,
            "Failed to set packageManager object property",
        ))
    })?;
    config_object.set("toolsConfig", tools_config).or_else(|_| {
        Err(Error::new(
            ConfigError::FailSetObjectProperty,
            "Failed to set toolsConfig object property",
        ))
    })?;
    config_object.set("workspaceRoot", root_config).or_else(|_| {
        Err(Error::new(
            ConfigError::FailSetObjectProperty,
            "Failed to set workspaceRoot object property",
        ))
    })?;

    Ok(config_object)
}
