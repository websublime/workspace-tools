use std::path::PathBuf;

use napi::{Either, Error, JsUndefined};
use sublime_standard_tools::get_project_root_path;

/// Determines the root directory of the current project.
///
/// This function attempts to locate a project's root directory by:
/// 1. Using the provided path if specified
/// 2. Starting from the current working directory if no path is provided
/// 3. Walking up the directory tree searching for Git repository markers
/// 4. Falling back to the current directory if no root markers are found
///
/// @param {string} [initial_path] - Optional custom starting path to use for detecting project root
/// @returns {string} The absolute path to the detected project root directory
/// @throws {Error} If the project root path cannot be determined
#[napi(js_name = "getProjectRootPath", ts_args_type = "initial_path?: string")]
pub fn js_get_project_root_path(
    initial_path: Option<Either<String, JsUndefined>>,
) -> Result<String, Error> {
    let path_option = match initial_path {
        Some(Either::A(path)) => Some(PathBuf::from(path)),
        Some(Either::B(_)) | None => None,
    };

    let result = get_project_root_path(path_option)
        .ok_or_else(|| Error::from_reason("Failed to determine project root path"))?;

    Ok(result.to_string_lossy().into_owned())
}
