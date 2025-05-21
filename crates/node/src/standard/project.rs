use napi::{Error, Result};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use sublime_standard_tools::monorepo::{
    ConfigManager, ConfigScope, MonorepoDescriptor, MonorepoDetector, ProjectConfig,
    ProjectManager, ProjectValidationStatus,
};

use super::{MonorepoProject, MonorepoProjectDescription, MonorepoProjectError};

impl AsRef<str> for MonorepoProjectError {
    fn as_ref(&self) -> &str {
        match self {
            Self::PackageNotFound => "PackageNotFound",
            Self::ProjectNotFound => "ProjectNotFound",
            Self::ConfigNotFound => "ConfigNotFound",
            Self::ProjectSymbolicLinkFail => "ProjectSymbolicLinkFail",
            Self::NapiError(e) => e.status.as_ref(),
        }
    }
}

#[napi]
impl MonorepoProject {
    #[napi(constructor, ts_args_type = "root?: string")]
    pub fn new(root: Option<String>) -> Result<Self, MonorepoProjectError> {
        let project_root = &(match root {
            Some(root) => root,
            None => String::from("."),
        });

        let project_root = &(Path::new(project_root.as_str()).canonicalize().map_err(|_| {
            Error::new(
                MonorepoProjectError::ProjectSymbolicLinkFail,
                "Failed to canonicalize project root",
            )
        })?);

        let manager = ProjectManager::new();
        let project_config = ProjectConfig::new()
            .with_root(project_root)
            .with_detect_monorepo(true)
            .with_detect_package_manager(true)
            .with_validate_structure(true);
        let project = manager.detect_project(project_root, &project_config).map_err(|e| {
            Error::new(
                MonorepoProjectError::ProjectNotFound,
                format!("Failed to detect project: {e}"),
            )
        })?;

        let config_path = &PathBuf::from(project_root).join("project.json");
        let mut config_manager = ConfigManager::new();
        config_manager.set_path(ConfigScope::Project, config_path);

        if config_path.exists() {
            config_manager.load_from_file(config_path.as_path()).map_err(|e| {
                Error::new(
                    MonorepoProjectError::ConfigNotFound,
                    format!("Failed to detect project: {e}"),
                )
            })?;
        }

        Ok(MonorepoProject {
            project_manager_instance: Rc::new(RefCell::new(manager)),
            config_manager_instance: Rc::new(RefCell::new(config_manager)),
            project_instance: Rc::new(RefCell::new(project)),
        })
    }

    /// Get project(monorepo) root information.
    ///
    /// @returns {MonorepoProjectDescription} - The project description.
    /// @throws {Error} If the project cannot be detected.
    #[napi]
    pub fn get_project_description(
        &self,
    ) -> Result<MonorepoProjectDescription, MonorepoProjectError> {
        let project_instance = self.project_instance.borrow();

        let project_root = project_instance.root();
        let package_json = project_instance.package_json();
        let package_manager_option = project_instance.package_manager();

        let package_manager = if let Some(pm) = package_manager_option {
            pm.kind().command().to_string()
        } else {
            String::from("None")
        };

        Ok(MonorepoProjectDescription {
            root: project_root.to_string_lossy().to_string(),
            package_json: serde_json::to_value(package_json).unwrap(),
            package_manager,
        })
    }

    /// Validate the project and return the validation status.
    ///
    /// @returns {Object} - The validation status.
    /// @throws {Error} If the project cannot be validated.
    #[napi(ts_return_type = "{status: string; messages: string[]}")]
    pub fn validate(&self) -> Result<Value, MonorepoProjectError> {
        let project_instance = self.project_instance.borrow();
        let mut validation_info = json!({"status": "none", "messages": []});

        match project_instance.validation_status() {
            ProjectValidationStatus::Valid => {
                validation_info["status"] = json!("valid");
                validation_info["messages"] = json!([]);
            }
            ProjectValidationStatus::Warning(warnings) => {
                validation_info["status"] = json!("warning");
                validation_info["messages"] = json!([]);

                for warning in warnings {
                    validation_info["messages"].as_array_mut().unwrap().push(json!(warning));
                }
            }
            ProjectValidationStatus::NotValidated => {
                validation_info["status"] = json!("not_validated");
                validation_info["messages"] = json!([]);
            }
            ProjectValidationStatus::Error(errors) => {
                validation_info["status"] = json!("error");
                validation_info["messages"] = json!([]);

                for error in errors {
                    validation_info["messages"].as_array_mut().unwrap().push(json!(error));
                }
            }
        }

        Ok(validation_info)
    }

    /// Get the workspace descriptor. Packages information.
    ///
    /// @returns {Array<{absolute_path: string; location: string; name: string; version: string; workspace_dependencies: string[]; workspace_dev_dependencies: string[];}>}
    /// @throws {Error} If the project cannot be detected.
    /// @throws {Error} If the project cannot be validated.
    /// @throws {Error} If the project cannot be loaded.
    #[napi(
        ts_return_type = "Array<{absolute_path: string; location: string; name: string; version: string; workspace_dependencies: string[]; workspace_dev_dependencies: string[];}>"
    )]
    pub fn get_workspace_descriptor(&self) -> Result<Value, MonorepoProjectError> {
        let detector = MonorepoDetector::new();
        let project_instance = self.project_instance.borrow();
        let monorepo = detector.detect_monorepo(project_instance.root()).map_err(|e| {
            Error::new(
                MonorepoProjectError::ProjectNotFound,
                format!("Failed to detect project: {e}"),
            )
        })?;

        Ok(json!(monorepo.packages()))
    }

    /// Get the package descriptor. Package information.
    ///
    /// @param {string} package - The package name.
    /// @returns {Object} - The package descriptor.
    /// @throws {Error} If the project cannot be detected.
    /// @throws {Error} If the package cannot be found.
    #[napi(
        ts_return_type = "{absolute_path: string; location: string; name: string; version: string; workspace_dependencies: string[]; workspace_dev_dependencies: string[];}"
    )]
    pub fn get_package_descriptor(&self, package: String) -> Result<Value, MonorepoProjectError> {
        let detector = MonorepoDetector::new();
        let project_instance = self.project_instance.borrow();
        let monorepo = detector.detect_monorepo(project_instance.root()).map_err(|e| {
            Error::new(
                MonorepoProjectError::ProjectNotFound,
                format!("Failed to detect project: {e}"),
            )
        })?;

        monorepo.packages().iter().find(|p| p.name == package).map(|p| json!(p)).ok_or_else(|| {
            Error::new(
                MonorepoProjectError::PackageNotFound,
                format!("Package not found: {package}"),
            )
        })
    }

    /// Get the workspace dependency graph. Packages dependency information.
    ///
    /// @returns {Object} - The workspace dependency graph.
    /// @throws {Error} If the project cannot be detected.
    #[napi(ts_return_type = "Record<string, string[]>")]
    pub fn get_workspace_dependency_graph(&self) -> Result<Value, MonorepoProjectError> {
        let detector = MonorepoDetector::new();
        let project_instance = self.project_instance.borrow();
        let monorepo = detector.detect_monorepo(project_instance.root()).map_err(|e| {
            Error::new(
                MonorepoProjectError::ProjectNotFound,
                format!("Failed to detect project: {e}"),
            )
        })?;
        let project_instance = self.project_instance.borrow();
        let project_root = project_instance.root();
        let kind = monorepo.kind().clone();

        let descriptor =
            MonorepoDescriptor::new(kind, project_root.to_path_buf(), monorepo.packages().to_vec());

        Ok(json!(descriptor.get_dependency_graph()))
    }
}
