use napi::{Error, Status};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;
use sublime_standard_tools::monorepo::{ConfigManager, Project, ProjectManager};

/// MonorepoProject class.
/// Represents a monorepo project. If `root` is not provided, it defaults to the current working directory.
/// If the `project.json` file is not found, it will be created. If the monorepo cannot be detected, an error will be thrown.
///
/// @class MonorepoProject - The MonorepoProject class.
/// @param {string|undefined} root - The root of the monorepo.
/// @throws {Error} If the monorepo cannot be detected.
///
/// @example
///
/// ```typescript
/// const monorepoProject = new MonorepoProject(".");
/// ```
#[allow(dead_code)]
#[napi(js_name = "MonorepoProject")]
pub struct MonorepoProject {
    pub(crate) project_manager_instance: Rc<RefCell<ProjectManager>>,
    pub(crate) config_manager_instance: Rc<RefCell<ConfigManager>>,
    pub(crate) project_instance: Rc<RefCell<Project>>,
}

#[napi(object, js_name = "MonorepoProjectDescription")]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct MonorepoProjectDescription {
    pub root: String,
    pub package_json: Value,
    pub package_manager: String,
}

pub enum MonorepoProjectError {
    PackageNotFound,
    ProjectNotFound,
    ProjectSymbolicLinkFail,
    ConfigNotFound,
    NapiError(Error<Status>),
}
