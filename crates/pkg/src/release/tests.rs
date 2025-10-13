#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod release_tests {
    use crate::{
        release::{PackageRelease, ReleaseManager, ReleasePlan, ReleaseStrategy},
        version::Version,
    };
    use std::{path::PathBuf, str::FromStr};

    #[test]
    fn test_release_manager_creation() {
        let manager = ReleaseManager::new(ReleaseStrategy::Independent, false, 5, 300);
        assert_eq!(manager.strategy(), ReleaseStrategy::Independent);
        assert!(!manager.is_dry_run());
    }

    #[test]
    fn test_release_plan_creation() {
        let plan = ReleasePlan::new(
            "test-changeset".to_string(),
            "prod".to_string(),
            ReleaseStrategy::Independent,
        );

        assert_eq!(plan.changeset_id, "test-changeset");
        assert_eq!(plan.environment, "prod");
        assert_eq!(plan.package_count(), 0);
    }

    #[test]
    fn test_package_release_creation() {
        let current = Version::from_str("1.0.0").unwrap();
        let next = Version::from_str("1.1.0").unwrap();

        let package = PackageRelease::new(
            "test-package".to_string(),
            current.into(),
            next.into(),
            PathBuf::from("./packages/test"),
        );

        assert_eq!(package.name, "test-package");
        assert!(package.publish);
        assert_eq!(package.access, "public");
        assert_eq!(package.tag, "latest");
    }

    #[test]
    fn test_package_release_builder() {
        let current = Version::from_str("1.0.0").unwrap();
        let next = Version::from_str("1.1.0").unwrap();

        let package = PackageRelease::new(
            "test-package".to_string(),
            current.into(),
            next.into(),
            PathBuf::from("./packages/test"),
        )
        .with_publish(false)
        .with_reason("Test release".to_string())
        .with_access("restricted".to_string())
        .with_tag("beta".to_string());

        assert!(!package.publish);
        assert_eq!(package.reason, "Test release");
        assert_eq!(package.access, "restricted");
        assert_eq!(package.tag, "beta");
    }

    #[test]
    fn test_release_plan_package_management() {
        let mut plan = ReleasePlan::new(
            "test-changeset".to_string(),
            "prod".to_string(),
            ReleaseStrategy::Independent,
        );

        let current = Version::from_str("1.0.0").unwrap();
        let next = Version::from_str("1.1.0").unwrap();

        let package = PackageRelease::new(
            "test-package".to_string(),
            current.into(),
            next.into(),
            PathBuf::from("./packages/test"),
        );

        plan.add_package(package);
        assert_eq!(plan.package_count(), 1);
    }

    #[test]
    fn test_duration_estimation() {
        let mut plan = ReleasePlan::new(
            "test-changeset".to_string(),
            "prod".to_string(),
            ReleaseStrategy::Independent,
        );

        // Empty plan should have base overhead
        assert_eq!(plan.estimate_duration(), 60);

        // Add a package
        let current = Version::from_str("1.0.0").unwrap();
        let next = Version::from_str("1.1.0").unwrap();
        let package = PackageRelease::new(
            "test-package".to_string(),
            current.into(),
            next.into(),
            PathBuf::from("./packages/test"),
        );

        plan.add_package(package);
        assert_eq!(plan.estimate_duration(), 90); // 30 + 60
    }

    #[test]
    fn test_release_strategy_default() {
        assert_eq!(ReleaseStrategy::default(), ReleaseStrategy::Independent);
    }

    #[test]
    fn test_unified_strategy() {
        let strategy = ReleaseStrategy::Unified { version: "2.0.0".to_string() };
        match strategy {
            ReleaseStrategy::Unified { version } => assert_eq!(version, "2.0.0"),
            _ => panic!("Expected Unified strategy"),
        }
    }

    #[test]
    fn test_package_release_serialization() {
        let current = Version::from_str("1.5.2").unwrap();
        let next = Version::from_str("1.6.0").unwrap();

        let package = PackageRelease::new(
            "serialize-test".to_string(),
            current.into(),
            next.into(),
            PathBuf::from("./test/path"),
        )
        .with_reason("Testing serialization".to_string())
        .with_access("restricted".to_string());

        // Test JSON serialization
        let json_result = serde_json::to_string(&package);
        assert!(json_result.is_ok());

        // Test JSON deserialization
        let json_str = json_result.unwrap();
        let deserialized: Result<PackageRelease, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let deserialized_package = deserialized.unwrap();
        assert_eq!(deserialized_package.name, "serialize-test");
        assert_eq!(deserialized_package.reason, "Testing serialization");
        assert_eq!(deserialized_package.access, "restricted");
    }

    #[test]
    fn test_release_plan_serialization() {
        let mut plan = ReleasePlan::new(
            "changeset-123".to_string(),
            "staging".to_string(),
            ReleaseStrategy::Independent,
        );

        let current = Version::from_str("2.1.0").unwrap();
        let next = Version::from_str("2.2.0").unwrap();
        let package = PackageRelease::new(
            "plan-package".to_string(),
            current.into(),
            next.into(),
            PathBuf::from("./packages/plan"),
        );

        plan.add_package(package);

        // Test JSON serialization
        let json_result = serde_json::to_string(&plan);
        assert!(json_result.is_ok());

        // Test JSON deserialization
        let json_str = json_result.unwrap();
        let deserialized: Result<ReleasePlan, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let deserialized_plan = deserialized.unwrap();
        assert_eq!(deserialized_plan.changeset_id, "changeset-123");
        assert_eq!(deserialized_plan.environment, "staging");
        assert_eq!(deserialized_plan.package_count(), 1);
    }

    #[test]
    fn test_package_release_with_all_options() {
        let current = Version::from_str("0.5.0").unwrap();
        let next = Version::from_str("1.0.0").unwrap();

        let package = PackageRelease::new(
            "full-options-package".to_string(),
            current.into(),
            next.into(),
            PathBuf::from("./packages/full"),
        )
        .with_publish(false)
        .with_reason("Major version release".to_string())
        .with_access("restricted".to_string())
        .with_tag("next".to_string());

        assert_eq!(package.name, "full-options-package");
        assert!(!package.publish);
        assert_eq!(package.reason, "Major version release");
        assert_eq!(package.access, "restricted");
        assert_eq!(package.tag, "next");
        assert_eq!(package.path, PathBuf::from("./packages/full"));
    }

    #[test]
    fn test_release_plan_multiple_packages() {
        let mut plan = ReleasePlan::new(
            "multi-package-changeset".to_string(),
            "production".to_string(),
            ReleaseStrategy::Independent,
        );

        // Add multiple packages
        for i in 1..=5 {
            let current = Version::from_str(&format!("{}.0.0", i)).unwrap();
            let next = Version::from_str(&format!("{}.1.0", i)).unwrap();
            let package = PackageRelease::new(
                format!("package-{}", i),
                current.into(),
                next.into(),
                PathBuf::from(format!("./packages/package-{}", i)),
            );
            plan.add_package(package);
        }

        assert_eq!(plan.package_count(), 5);
        // Duration should be 60 (base) + 5 * 30 (per package) = 210
        assert_eq!(plan.estimate_duration(), 210);
    }

    #[test]
    fn test_release_strategy_variants() {
        let independent = ReleaseStrategy::Independent;
        let unified = ReleaseStrategy::Unified { version: "4.0.0".to_string() };

        assert_eq!(format!("{:?}", independent), "Independent");

        match unified {
            ReleaseStrategy::Unified { version } => {
                assert_eq!(version, "4.0.0");
            }
            _ => panic!("Expected Unified variant"),
        }
    }
}
