//! JavaScript bindings for graph validation utilities.

use napi_derive::napi;
use ws_pkg::graph::{ValidationIssue as WsValidationIssue, ValidationReport as WsValidationReport};

/// JavaScript binding for types of validation issues
#[napi]
pub enum ValidationIssueType {
    /// Circular dependency detected
    CircularDependency,
    /// Unresolved dependency
    UnresolvedDependency,
    /// Version conflict
    VersionConflict,
}

/// JavaScript binding for validation issue
#[napi(object)]
pub struct ValidationIssueInfo {
    /// Type of the issue
    pub issue_type: ValidationIssueType,
    /// Human-readable message describing the issue
    pub message: String,
    /// Whether this is a critical issue
    pub critical: bool,

    /// Additional data for circular dependency
    pub path: Option<Vec<String>>,
    /// Additional data for unresolved dependency
    pub dependency_name: Option<String>,
    /// Additional data for unresolved dependency
    pub version_req: Option<String>,
    /// Additional data for version conflict
    pub conflicting_versions: Option<Vec<String>>,
}

/// JavaScript binding for validation report
#[napi]
pub struct ValidationReport {
    pub(crate) inner: WsValidationReport,
}

#[napi]
impl ValidationReport {
    /// Check if there are any issues in the report
    ///
    /// @returns {boolean} True if there are any issues
    #[napi(getter)]
    pub fn has_issues(&self) -> bool {
        self.inner.has_issues()
    }

    /// Check if there are any critical issues
    ///
    /// @returns {boolean} True if there are critical issues
    #[napi(getter)]
    pub fn has_critical_issues(&self) -> bool {
        self.inner.has_critical_issues()
    }

    /// Check if there are any warnings (non-critical issues)
    ///
    /// @returns {boolean} True if there are warnings
    #[napi(getter)]
    pub fn has_warnings(&self) -> bool {
        self.inner.has_warnings()
    }

    /// Get all issues in the report
    ///
    /// @returns {ValidationIssueInfo[]} Array of validation issues
    #[napi]
    pub fn get_issues(&self) -> Vec<ValidationIssueInfo> {
        self.inner
            .issues()
            .iter()
            .map(|issue| {
                let (issue_type, path, name, version_req, versions) = match issue {
                    WsValidationIssue::CircularDependency { path } => (
                        ValidationIssueType::CircularDependency,
                        Some(path.clone()),
                        None,
                        None,
                        None,
                    ),
                    WsValidationIssue::UnresolvedDependency { name, version_req } => (
                        ValidationIssueType::UnresolvedDependency,
                        None,
                        Some(name.clone()),
                        Some(version_req.clone()),
                        None,
                    ),
                    WsValidationIssue::VersionConflict { name, versions } => (
                        ValidationIssueType::VersionConflict,
                        None,
                        Some(name.clone()),
                        None,
                        Some(versions.clone()),
                    ),
                };

                ValidationIssueInfo {
                    issue_type,
                    message: issue.message(),
                    critical: issue.is_critical(),
                    path,
                    dependency_name: name,
                    version_req,
                    conflicting_versions: versions,
                }
            })
            .collect()
    }

    /// Get critical issues only
    ///
    /// @returns {ValidationIssueInfo[]} Array of critical validation issues
    #[napi]
    pub fn get_critical_issues(&self) -> Vec<ValidationIssueInfo> {
        self.get_issues().into_iter().filter(|issue| issue.critical).collect()
    }

    /// Get warnings only (non-critical issues)
    ///
    /// @returns {ValidationIssueInfo[]} Array of warning validation issues
    #[napi]
    pub fn get_warnings(&self) -> Vec<ValidationIssueInfo> {
        self.get_issues().into_iter().filter(|issue| !issue.critical).collect()
    }
}

#[cfg(test)]
mod validation_binding_tests {
    use crate::graph::builder::build_dependency_graph_from_packages;
    use crate::types::dependency::Dependency;
    use crate::types::package::Package;

    #[test]
    fn test_validation_report() {
        // Create packages with circular dependency
        let mut pkg1 = Package::new("pkg1".to_string(), "1.0.0".to_string());
        let mut pkg2 = Package::new("pkg2".to_string(), "1.0.0".to_string());

        // Create dependencies
        let dep1 = Dependency::new("pkg2".to_string(), "^1.0.0".to_string());
        let dep2 = Dependency::new("pkg1".to_string(), "^1.0.0".to_string());

        // Add dependencies to create a circular reference
        pkg1.add_dependency(&dep1);
        pkg2.add_dependency(&dep2);

        // Build graph
        let graph = build_dependency_graph_from_packages(vec![&pkg1, &pkg2]);

        // Print debug info
        println!("Graph created with {} packages", 2);

        // Detect circular dependency directly
        let cycle = graph.detect_circular_dependencies();
        println!("Circular dependency detected: {:?}", cycle);

        // Validate in a safer way
        match graph.validate_package_dependencies() {
            Ok(report) => {
                println!("Validation successful, has issues: {}", report.has_issues());
                assert!(
                    report.has_issues(),
                    "Expected validation issues due to circular dependency"
                );

                // Get issues safely
                let issues = report.get_issues();
                println!("Found {} issues", issues.len());

                assert!(!issues.is_empty(), "Expected at least one validation issue");

                // Only if we have issues, check the first one
                if !issues.is_empty() {
                    let first_issue = &issues[0];
                    println!("First issue is critical: {}", first_issue.critical);
                    assert!(first_issue.critical, "Expected critical issue");
                }
            }
            Err(e) => {
                println!("Validation failed with error: {:?}", e);
                panic!("Validation should not fail with error");
            }
        }
    }
}
