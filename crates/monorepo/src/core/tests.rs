//! Unit tests for core module functionality

#[cfg(test)]
mod tests {
    use crate::config::{Environment, VersionBumpType};
    use crate::core::*;
    use chrono::Utc;

    #[test]
    fn test_changeset_status_variants() {
        // Test that all ChangesetStatus variants exist
        let pending = ChangesetStatus::Pending;
        let partially_deployed =
            ChangesetStatus::PartiallyDeployed { environments: vec![Environment::Development] };

        // Test comparison
        assert_eq!(pending, ChangesetStatus::Pending);
        assert_ne!(pending, partially_deployed);
    }

    #[test]
    fn test_version_status_variants() {
        // Test that all VersionStatus variants exist
        let stable = VersionStatus::Stable;
        let dirty = VersionStatus::Dirty;
        let snapshot = VersionStatus::Snapshot { sha: "abc123".to_string() };
        let pre_release = VersionStatus::PreRelease { tag: "alpha.1".to_string() };

        // Test comparison
        assert_eq!(stable, VersionStatus::Stable);
        assert_ne!(stable, dirty);
        assert!(matches!(snapshot, VersionStatus::Snapshot { .. }));
        assert!(matches!(pre_release, VersionStatus::PreRelease { .. }));
    }

    #[test]
    fn test_environment_variants() {
        let dev = Environment::Development;
        let staging = Environment::Staging;
        let prod = Environment::Production;
        let integration = Environment::Integration;

        assert_eq!(dev, Environment::Development);
        assert_ne!(dev, prod);
        assert_ne!(staging, integration);
    }

    #[test]
    fn test_conflict_type_variants() {
        let pending_changesets = ConflictType::PendingChangesets;
        let dirty_wd = ConflictType::DirtyWorkingDirectory;
        let breaking_change = ConflictType::PotentialBreakingChange;
        let dep_mismatch = ConflictType::DependencyMismatch;
        let circular_dep = ConflictType::CircularDependency;

        assert_eq!(pending_changesets, ConflictType::PendingChangesets);
        assert_ne!(pending_changesets, dirty_wd);
        assert_ne!(dirty_wd, breaking_change);
        assert_ne!(breaking_change, dep_mismatch);
        assert_ne!(dep_mismatch, circular_dep);
    }

    #[test]
    fn test_versioning_strategies() {
        let default_strategy = DefaultVersioningStrategy;
        let conservative_strategy = ConservativeVersioningStrategy;
        let aggressive_strategy = AggressiveVersioningStrategy;

        // Test that strategies can be created
        assert_eq!(
            std::any::type_name_of_val(&default_strategy),
            "sublime_monorepo_tools::core::types::DefaultVersioningStrategy"
        );
        assert_eq!(
            std::any::type_name_of_val(&conservative_strategy),
            "sublime_monorepo_tools::core::types::ConservativeVersioningStrategy"
        );
        assert_eq!(
            std::any::type_name_of_val(&aggressive_strategy),
            "sublime_monorepo_tools::core::types::AggressiveVersioningStrategy"
        );
    }

    #[test]
    fn test_changeset_creation() {
        let changeset = Changeset {
            id: "test-changeset".to_string(),
            package: "test-package".to_string(),
            version_bump: VersionBumpType::Minor,
            description: "Test changeset".to_string(),
            branch: "feature/test".to_string(),
            development_environments: vec![Environment::Development],
            production_deployment: false,
            created_at: Utc::now(),
            author: "test-author".to_string(),
            status: ChangesetStatus::Pending,
        };

        // Test initial state
        assert_eq!(changeset.status, ChangesetStatus::Pending);
        assert!(!changeset.production_deployment);
        assert_eq!(changeset.package, "test-package");
        assert_eq!(changeset.version_bump, VersionBumpType::Minor);
        assert_eq!(changeset.author, "test-author");
        assert_eq!(changeset.branch, "feature/test");
    }

    #[test]
    fn test_version_impact_analysis() {
        let analysis = VersionImpactAnalysis {
            affected_packages: std::collections::HashMap::new(),
            total_packages_affected: 2,
            breaking_changes: vec![BreakingChangeAnalysis {
                package_name: "test-pkg".to_string(),
                reason: "API changed".to_string(),
                affected_dependents: vec!["pkg-a".to_string()],
            }],
            dependency_chain_impacts: vec![],
            estimated_propagation_depth: 1,
        };

        assert_eq!(analysis.total_packages_affected, 2);
        assert_eq!(analysis.breaking_changes.len(), 1);
        assert_eq!(analysis.estimated_propagation_depth, 1);
    }

    #[test]
    fn test_versioning_plan() {
        let plan = VersioningPlan {
            steps: vec![],
            total_packages: 5,
            estimated_duration: std::time::Duration::from_secs(30),
            conflicts: vec![],
            impact_analysis: VersionImpactAnalysis {
                affected_packages: std::collections::HashMap::new(),
                total_packages_affected: 5,
                breaking_changes: vec![],
                dependency_chain_impacts: vec![],
                estimated_propagation_depth: 0,
            },
        };

        assert_eq!(plan.total_packages, 5);
        assert!(plan.conflicts.is_empty());
        assert_eq!(plan.estimated_duration.as_secs(), 30);
    }
}
