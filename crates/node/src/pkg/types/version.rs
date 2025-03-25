use sublime_package_tools::Version as SublimeVersion;

#[napi]
pub fn bump_major(version: String) -> Option<String> {
    if let Ok(version) = SublimeVersion::bump_major(version.as_str()) {
        Some(version.to_string())
    } else {
        None
    }
}

#[napi]
pub fn bump_minor(version: String) -> Option<String> {
    if let Ok(version) = SublimeVersion::bump_minor(version.as_str()) {
        Some(version.to_string())
    } else {
        None
    }
}

#[napi]
pub fn bump_patch(version: String) -> Option<String> {
    if let Ok(version) = SublimeVersion::bump_patch(version.as_str()) {
        Some(version.to_string())
    } else {
        None
    }
}

#[napi]
pub fn bump_snapshot(version: String, suffix: String) -> Option<String> {
    if let Ok(version) = SublimeVersion::bump_snapshot(version.as_str(), suffix.as_str()) {
        Some(version.to_string())
    } else {
        None
    }
}
