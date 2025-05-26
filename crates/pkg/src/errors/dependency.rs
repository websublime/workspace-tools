use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DependencyResolutionError {
    #[error("Failed to parse version: {0}")]
    VersionParseError(String),
    #[error(
        "Incompatible version: {name}. Versions: {versions:?}. Requirements: {requirements:?}"
    )]
    IncompatibleVersions { name: String, versions: Vec<String>, requirements: Vec<String> },
    #[error("No valid version found for {name} with requirements {requirements:?}")]
    NoValidVersion { name: String, requirements: Vec<String> },
    #[error("Dependency {name} not found in package {package}")]
    DependencyNotFound { name: String, package: String },
    // Note: We keep this error for backward compatibility, but we won't generate it anymore
    #[error("Circular dependencies found: {path:?}")]
    CircularDependency { path: Vec<String> },
}

impl AsRef<str> for DependencyResolutionError {
    fn as_ref(&self) -> &str {
        match self {
            DependencyResolutionError::VersionParseError(_) => "VersionParseError",
            DependencyResolutionError::IncompatibleVersions {
                name: _,
                versions: _,
                requirements: _,
            } => "IncompatibleVersions",
            DependencyResolutionError::NoValidVersion { name: _, requirements: _ } => {
                "NoValidVersion"
            }
            DependencyResolutionError::DependencyNotFound { name: _, package: _ } => {
                "DependencyNotFound"
            }
            DependencyResolutionError::CircularDependency { path: _ } => "CircularDependency",
        }
    }
}
