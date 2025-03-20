use std::{env::current_dir, path::PathBuf};

use napi::{bindgen_prelude::*, JsUndefined};
use sublime_standard_tools::detect_package_manager;

/**
 * Detects the package manager used in a project directory.
 *
 * This function analyzes the given directory to determine which package manager
 * is being used in the project. It looks for specific files and patterns that
 * indicate the presence of npm, yarn, pnpm, or other package managers.
 *
 * @param {string} [root_path] - Optional path to the project directory to analyze.
 *                              If not provided, the current working directory is used.
 * @returns {string} The detected package manager name (e.g., "npm", "yarn", "pnpm").
 * @throws {Error} If no package manager could be detected in the specified directory.
 *
 * @example
 * // Detect package manager in the current directory
 * const packageManager = detectPackageManager();
 * console.log(packageManager); // Outputs: "npm", "yarn", etc.
 *
 * @example
 * // Detect package manager in a specific directory
 * const packageManager = detectPackageManager("/path/to/project");
 * console.log(packageManager);
 */
#[napi(js_name = "detectPackageManager", ts_args_type = "root_path?: string")]
pub fn js_detect_package_manager(root_path: Option<Either<String, JsUndefined>>) -> Result<String> {
    // Use current directory if path is not provided
    let path_buf = match root_path {
        Some(Either::A(p)) => PathBuf::from(p),
        Some(Either::B(_)) | None => current_dir().unwrap_or_default(),
    };

    let result = detect_package_manager(&path_buf);

    if let Some(package_manager) = result {
        Ok(package_manager.to_string())
    } else {
        Err(Error::from_reason("Fail to recognize package manager"))
    }
}
