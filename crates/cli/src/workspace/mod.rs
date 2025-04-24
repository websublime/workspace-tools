mod detection;
mod repository;

pub use detection::{detect_workspace_patterns, get_standard_exclude_patterns, WorkspaceDetector};
pub use repository::{Repository, RepositoryManager};

use crate::common::errors::CliResult;
use std::path::Path;

pub trait WorkspaceManager {
    fn add_repository<P: AsRef<Path>>(&mut self, path: P, name: Option<String>) -> CliResult<()>;
    fn remove_repository(&mut self, identifier: &str) -> CliResult<()>;
    fn list_repositories(&self) -> CliResult<Vec<&Repository>>;
    fn get_repository(&self, identifier: &str) -> CliResult<Option<&Repository>>;
}

// Re-export common workspace types for convenience
pub use repository::RepositoryConfig;
