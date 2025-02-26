use ws_pkg::version::Version;

/// Bumps the version of the package to major.
///
/// @param {string} version - The version of the package.
/// @returns {string} The new version of the package.
#[napi(js_name = "bumpMajor")]
pub fn js_bump_major(version: String) -> String {
    let version = Version::bump_major(version.as_str());
    version.to_string()
}

/// Bumps the version of the package to minor.
///
/// @param {string} version - The version of the package.
/// @returns {string} The new version of the package.
#[napi(js_name = "bumpMinor")]
pub fn js_bump_minor(version: String) -> String {
    let version = Version::bump_minor(version.as_str());
    version.to_string()
}

/// Bumps the version of the package to patch.
///
/// @param {string} version - The version of the package.
/// @returns {string} The new version of the package.
#[napi(js_name = "bumpPatch")]
pub fn js_bump_patch(version: String) -> String {
    let version = Version::bump_patch(version.as_str());
    version.to_string()
}

/// Bumps the version of the package to snapshot.
///
/// @param {string} version - The version of the package.
/// @param {string} snapshot - The snapshot.
/// @returns {string} The new version of the package.
#[napi(js_name = "bumpSnapshot")]
pub fn js_bump_snapshot(version: String, snapshot: String) -> String {
    let version = Version::bump_snapshot(version.as_str(), snapshot.as_str());
    version.to_string()
}
