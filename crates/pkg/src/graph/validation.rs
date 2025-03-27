#[derive(Debug)]
pub enum ValidationIssue {
    /// Circular dependency detected
    CircularDependency { path: Vec<String> },

    /// Unresolved dependency
    UnresolvedDependency { name: String, version_req: String },

    /// Version conflict
    VersionConflict { name: String, versions: Vec<String> },
}

impl ValidationIssue {
    /// Returns true if this is a critical issue that should be fixed
    pub fn is_critical(&self) -> bool {
        match self {
            Self::UnresolvedDependency { .. } | Self::CircularDependency { .. } => true,
            Self::VersionConflict { .. } => false, // Consider version conflicts as warnings
        }
    }

    /// Returns a descriptive message for this issue
    pub fn message(&self) -> String {
        match self {
            Self::CircularDependency { path } => {
                format!("Circular dependency detected: {}", path.join(" -> "))
            }
            Self::UnresolvedDependency { name, version_req } => {
                format!("Unresolved dependency: {name} {version_req}")
            }
            Self::VersionConflict { name, versions } => {
                format!("Version conflict for {}: {}", name, versions.join(", "))
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ValidationReport {
    issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    pub fn has_critical_issues(&self) -> bool {
        self.issues.iter().any(ValidationIssue::is_critical)
    }

    pub fn has_warnings(&self) -> bool {
        self.issues.iter().any(|issue| !issue.is_critical())
    }

    pub fn critical_issues(&self) -> Vec<&ValidationIssue> {
        self.issues.iter().filter(|issue| issue.is_critical()).collect()
    }

    pub fn warnings(&self) -> Vec<&ValidationIssue> {
        self.issues.iter().filter(|issue| !issue.is_critical()).collect()
    }
}
