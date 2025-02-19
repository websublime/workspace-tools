#![allow(clippy::bind_instead_of_map)]
#![allow(clippy::needless_pass_by_value)]
use std::path::Path;

use ws_std::manager::detect_package_manager;

#[napi(js_name = "detectPackageManager", ts_return_type = "Result<PackageManager>")]
pub fn js_detect_manager(cwd: String) -> Option<String> {
    let root = Path::new(&cwd);
    let package_manager = detect_package_manager(root).expect("Unable to detect package manager");

    Some(package_manager.to_string().to_lowercase())
}
