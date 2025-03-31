mod test_utils;

use std::rc::Rc;
use sublime_monorepo_tools::{
    Change, ChangeTracker, ChangeType, DiscoveryOptions, MemoryChangeStore, WorkspaceManager,
};
use test_utils::TestWorkspace;

#[cfg(test)]
mod changes_environment_tests {
    use super::*;

    fn setup_test() -> (TestWorkspace, ChangeTracker) {
        // Create a test workspace
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo();

        // Create workspace manager and discover workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();
        let workspace = manager.discover_workspace(test_workspace.path(), &options).unwrap();

        // Create change tracker
        let tracker = ChangeTracker::new(Rc::new(workspace), Box::new(MemoryChangeStore::new()));

        (test_workspace, tracker)
    }

    #[test]
    fn test_change_with_environments() {
        // Create a change with specific environments
        let change = Change::new("pkg-a", ChangeType::Feature, "New feature", false)
            .with_environments(vec!["staging", "production"]);

        // Verify environments were set
        assert_eq!(change.environments.len(), 2);
        assert!(change.environments.contains(&"staging".to_string()));
        assert!(change.environments.contains(&"production".to_string()));

        // Test applies_to_environment method
        assert!(change.applies_to_environment("staging"));
        assert!(change.applies_to_environment("production"));
        assert!(!change.applies_to_environment("development"));

        // Test a change with no specific environments (should apply to all)
        let generic_change = Change::new("pkg-a", ChangeType::Fix, "Bug fix", false);
        assert!(generic_change.environments.is_empty());
        assert!(generic_change.applies_to_environment("development"));
        assert!(generic_change.applies_to_environment("staging"));
        assert!(generic_change.applies_to_environment("production"));
    }

    #[test]
    fn test_environment_filtering() {
        let (_, mut tracker) = setup_test();

        // Create changes with different environment targets
        let dev_change = Change::new("pkg-a", ChangeType::Feature, "Dev feature", false)
            .with_environments(vec!["development"]);

        let staging_change = Change::new("pkg-a", ChangeType::Fix, "Staging fix", false)
            .with_environments(vec!["staging"]);

        let prod_change = Change::new("pkg-a", ChangeType::Refactor, "Prod refactor", false)
            .with_environments(vec!["production"]);

        let multi_env_change = Change::new("pkg-a", ChangeType::Performance, "Performance", false)
            .with_environments(vec!["staging", "production"]);

        let all_env_change = Change::new("pkg-a", ChangeType::Chore, "Generic change", false);

        // Record the changes
        tracker.record_change(dev_change).unwrap();
        tracker.record_change(staging_change).unwrap();
        tracker.record_change(prod_change).unwrap();
        tracker.record_change(multi_env_change).unwrap();
        tracker.record_change(all_env_change).unwrap();

        // Get changes for specific environments
        let dev_changes =
            tracker.store().get_changes_for_environment("pkg-a", "development").unwrap();
        let staging_changes =
            tracker.store().get_changes_for_environment("pkg-a", "staging").unwrap();
        let prod_changes =
            tracker.store().get_changes_for_environment("pkg-a", "production").unwrap();

        // Verify dev environment only has dev_change and all_env_change
        assert_eq!(dev_changes.len(), 2);
        assert!(dev_changes.iter().any(|c| c.description == "Dev feature"));
        assert!(dev_changes.iter().any(|c| c.description == "Generic change"));

        // Verify staging environment has staging_change, multi_env_change, and all_env_change
        assert_eq!(staging_changes.len(), 3);
        assert!(staging_changes.iter().any(|c| c.description == "Staging fix"));
        assert!(staging_changes.iter().any(|c| c.description == "Performance"));
        assert!(staging_changes.iter().any(|c| c.description == "Generic change"));

        // Verify production environment has prod_change, multi_env_change, and all_env_change
        assert_eq!(prod_changes.len(), 3);
        assert!(prod_changes.iter().any(|c| c.description == "Prod refactor"));
        assert!(prod_changes.iter().any(|c| c.description == "Performance"));
        assert!(prod_changes.iter().any(|c| c.description == "Generic change"));
    }

    #[test]
    fn test_unreleased_changes_for_environment() {
        let (_, mut tracker) = setup_test();

        // Create changes with different environment targets
        let dev_change = Change::new("pkg-a", ChangeType::Feature, "Dev feature", false)
            .with_environments(vec!["development"]);

        let staging_change = Change::new("pkg-b", ChangeType::Fix, "Staging fix", false)
            .with_environments(vec!["staging"]);

        // Record the changes
        tracker.record_change(dev_change).unwrap();
        tracker.record_change(staging_change).unwrap();

        // Get unreleased changes for development environment
        let dev_unreleased = tracker.unreleased_changes_for_environment("development").unwrap();
        assert!(dev_unreleased.contains_key("pkg-a"));
        assert!(!dev_unreleased.contains_key("pkg-b"));
        assert_eq!(dev_unreleased["pkg-a"].len(), 1);

        // Get unreleased changes for staging environment
        let staging_unreleased = tracker.unreleased_changes_for_environment("staging").unwrap();
        assert!(!staging_unreleased.contains_key("pkg-a"));
        assert!(staging_unreleased.contains_key("pkg-b"));
        assert_eq!(staging_unreleased["pkg-b"].len(), 1);
    }

    #[test]
    fn test_mark_released_for_environment() {
        let (_, mut tracker) = setup_test();

        // Create changes with different environment targets
        let staging_change = Change::new("pkg-a", ChangeType::Feature, "Staging feature", false)
            .with_environments(vec!["staging"]);

        let prod_change = Change::new("pkg-a", ChangeType::Fix, "Prod fix", false)
            .with_environments(vec!["production"]);

        let multi_env_change = Change::new("pkg-a", ChangeType::Performance, "Performance", false)
            .with_environments(vec!["staging", "production"]);

        // Record the changes
        tracker.record_change(staging_change).unwrap();
        tracker.record_change(prod_change).unwrap();
        tracker.record_change(multi_env_change).unwrap();

        // Mark staging changes as released
        let marked =
            tracker.mark_released_for_environment("pkg-a", "1.0.0", "staging", false).unwrap();
        assert_eq!(marked.len(), 2); // Should mark staging_change and multi_env_change

        // Verify that staging changes are now released
        let unreleased_staging =
            tracker.store().get_unreleased_changes_for_environment("pkg-a", "staging").unwrap();
        assert_eq!(unreleased_staging.len(), 0);

        // But production changes should still be unreleased
        let unreleased_prod =
            tracker.store().get_unreleased_changes_for_environment("pkg-a", "production").unwrap();
        assert_eq!(unreleased_prod.len(), 1);
        assert_eq!(unreleased_prod[0].description, "Prod fix");
    }
}
