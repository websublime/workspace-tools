use std::collections::HashMap;

use super::update::DependencyUpdate;

#[derive(Debug)]
pub struct ResolutionResult {
    /// Resolved versions for each package
    pub resolved_versions: HashMap<String, String>,
    /// Packages that need version updates
    pub updates_required: Vec<DependencyUpdate>,
}
