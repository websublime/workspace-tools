#[derive(Debug, Clone)]
pub struct PackageScopeMetadata {
    pub full: String,
    pub name: String,
    pub version: String,
    pub path: Option<String>,
}

/// Parse package scope, name, and version from a string
pub fn package_scope_name_version(pkg_name: &str) -> Option<PackageScopeMetadata> {
    // Must start with @ to be a scoped package
    if !pkg_name.starts_with('@') {
        return None;
    }

    let full = pkg_name.to_string();
    let mut name = String::new();
    let mut version = "latest".to_string();
    let mut path = None;

    // First check for colon format: @scope/name:version
    if pkg_name.contains(':') {
        let parts: Vec<&str> = pkg_name.split(':').collect();
        name = parts[0].to_string();
        if parts.len() > 1 {
            version = parts[1].to_string();
        }
    }
    // Handle @ format: @scope/name@version or @scope/name@version@path
    else {
        let parts: Vec<&str> = pkg_name.split('@').collect();

        // First part is empty because it starts with @
        if parts.len() >= 2 {
            // Format: @scope/name
            name = format!("@{}", parts[1]);

            // Check if there's a version
            if parts.len() >= 3 {
                // Format: @scope/name@version
                version = parts[2].to_string();

                // Check if there's a path
                if parts.len() >= 4 {
                    // Format: @scope/name@version@path
                    path = Some(parts[3].to_string());
                }
            }
        }
    }

    Some(PackageScopeMetadata { full, name, version, path })
}
