#[derive(Debug)]
pub struct DependencyUpdate {
    /// Package name
    pub package_name: String,
    /// Dependency name
    pub dependency_name: String,
    /// Current version
    pub current_version: String,
    /// New version to update to
    pub new_version: String,
}
