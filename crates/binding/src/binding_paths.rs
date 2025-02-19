#![allow(clippy::manual_map)]
use std::path::PathBuf;

use ws_std::paths::get_project_root_path;

#[napi(js_name = "getProjectRootPath")]
pub fn js_get_project_root_path(cwd: Option<String>) -> Option<String> {
    let root = match cwd {
        Some(dir) => Some(PathBuf::from(dir)),
        None => None,
    };

    let project_root = get_project_root_path(root);

    project_root.map(|path| path.to_string_lossy().to_string())
}
