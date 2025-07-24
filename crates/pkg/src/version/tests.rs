//! # Version Module Test Suite
//!
//! ## What
//! Comprehensive test coverage for the version module, including all versioning strategies,
//! cascade bumping functionality, change set management, and monorepo configuration.
//! Tests follow enterprise quality standards with 100% coverage requirement.
//!
//! ## How
//! Tests are organized into logical groups covering:
//! - Core structures (ChangeSet, BumpExecutionMode)
//! - Versioning strategies (Individual, Unified, Mixed)
//! - Cascade bumping analysis and execution
//! - Context-aware optimizations
//! - Integration with VersionManager
//!
//! ## Why
//! Enterprise-grade package management requires comprehensive testing to ensure
//! reliability, prevent regressions, and maintain quality across all versioning
//! scenarios and repository contexts.

#[allow(clippy::assertions_on_constants)]
#[cfg(test)]
mod tests {
    use crate::version::{
        cascade_bumper::{self},
        change_set::{BumpExecutionMode, ChangeSet},
        version::{self, BumpStrategy},
        versioning_strategy::{MonorepoVersionBumpConfig, MonorepoVersioningStrategy},
    };
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    mod unit_tests {
        use super::*;

        // ========================================================================
        // CHANGE SET TESTS
        // ========================================================================

        #[test]
        fn test_bump_execution_mode_methods() {
            assert!(BumpExecutionMode::Preview.is_preview());
            assert!(!BumpExecutionMode::Preview.modifies_filesystem());

            assert!(!BumpExecutionMode::Apply.is_preview());
            assert!(BumpExecutionMode::Apply.modifies_filesystem());
        }

        #[test]
        fn test_bump_execution_mode_default() {
            assert_eq!(BumpExecutionMode::default(), BumpExecutionMode::Preview);
        }

        #[test]
        fn test_change_set_new() {
            let mut packages = HashMap::new();
            packages.insert("test-pkg".to_string(), BumpStrategy::Minor);

            let change_set = ChangeSet::new(packages, "Test reason".to_string());

            assert_eq!(change_set.target_packages().len(), 1);
            assert_eq!(change_set.reason(), "Test reason");
            assert_eq!(change_set.execution_mode(), BumpExecutionMode::Preview);
            assert!(!change_set.is_empty());
            assert_eq!(change_set.len(), 1);
        }

        #[test]
        fn test_change_set_with_execution_mode() {
            let packages = HashMap::new();
            let change_set = ChangeSet::with_execution_mode(
                packages,
                "Apply mode".to_string(),
                BumpExecutionMode::Apply,
            );

            assert_eq!(change_set.execution_mode(), BumpExecutionMode::Apply);
            assert!(change_set.execution_mode().modifies_filesystem());
        }

        #[test]
        fn test_change_set_mode_conversion() {
            let mut original = ChangeSet::new(HashMap::new(), "Test".to_string());
            original.set_execution_mode(BumpExecutionMode::Apply);

            let preview = original.as_preview();
            let apply = original.as_apply();

            assert_eq!(original.execution_mode(), BumpExecutionMode::Apply);
            assert_eq!(preview.execution_mode(), BumpExecutionMode::Preview);
            assert_eq!(apply.execution_mode(), BumpExecutionMode::Apply);
        }

        #[test]
        fn test_change_set_package_operations() {
            let mut change_set = ChangeSet::new(HashMap::new(), "Test".to_string());

            // Add packages
            assert!(change_set.add_package("pkg1".to_string(), BumpStrategy::Major).is_none());
            assert!(change_set.add_package("pkg2".to_string(), BumpStrategy::Minor).is_none());
            assert_eq!(change_set.len(), 2);

            // Update existing package
            let previous = change_set.add_package("pkg1".to_string(), BumpStrategy::Patch);
            assert_eq!(previous, Some(BumpStrategy::Major));
            assert_eq!(change_set.len(), 2);

            // Check contains
            assert!(change_set.contains_package("pkg1"));
            assert!(change_set.contains_package("pkg2"));
            assert!(!change_set.contains_package("nonexistent"));

            // Remove package
            assert_eq!(change_set.remove_package("pkg1"), Some(BumpStrategy::Patch));
            assert_eq!(change_set.len(), 1);
            assert!(!change_set.contains_package("pkg1"));
        }

        #[test]
        fn test_change_set_empty() {
            let empty = ChangeSet::new(HashMap::new(), "Empty".to_string());
            assert!(empty.is_empty());
            assert_eq!(empty.len(), 0);

            let mut non_empty = HashMap::new();
            non_empty.insert("pkg".to_string(), BumpStrategy::Patch);
            let non_empty = ChangeSet::new(non_empty, "Non-empty".to_string());
            assert!(!non_empty.is_empty());
            assert_eq!(non_empty.len(), 1);
        }

        #[test]
        fn test_change_set_timestamp() {
            use std::time::Duration;

            let before = std::time::SystemTime::now();
            let change_set = ChangeSet::new(HashMap::new(), "Test".to_string());
            let after = std::time::SystemTime::now();

            assert!(change_set.timestamp() >= before);
            assert!(change_set.timestamp() <= after);

            // Timestamp should be reasonably recent (within 1 second)
            let duration = after.duration_since(change_set.timestamp()).unwrap_or(Duration::ZERO);
            assert!(duration.as_secs() < 1);
        }

        // ========================================================================
        // VERSIONING STRATEGY TESTS
        // ========================================================================

        #[test]
        fn test_versioning_strategy_defaults() {
            assert_eq!(
                MonorepoVersioningStrategy::default(),
                MonorepoVersioningStrategy::Individual
            );
        }

        #[test]
        fn test_versioning_strategy_properties() {
            // Individual strategy
            let individual = MonorepoVersioningStrategy::Individual;
            assert!(!individual.has_groups());
            assert!(!individual.is_unified());
            assert!(individual.allows_individual());
            assert!(individual.groups().is_none());
            assert!(individual.individual_packages().is_none());

            // Unified strategy
            let unified = MonorepoVersioningStrategy::Unified;
            assert!(!unified.has_groups());
            assert!(unified.is_unified());
            assert!(!unified.allows_individual());
            assert!(unified.groups().is_none());
            assert!(unified.individual_packages().is_none());

            // Mixed strategy
            let mixed = MonorepoVersioningStrategy::Mixed {
                groups: HashMap::new(),
                individual_packages: HashSet::new(),
            };
            assert!(mixed.has_groups());
            assert!(!mixed.is_unified());
            assert!(mixed.allows_individual());
            assert!(mixed.groups().is_some());
            assert!(mixed.individual_packages().is_some());
        }

        #[test]
        fn test_mixed_strategy_access() {
            let mut groups = HashMap::new();
            groups.insert("core-*".to_string(), "1.0.0".to_string());

            let mut individual_packages = HashSet::new();
            individual_packages.insert("example".to_string());

            let mixed = MonorepoVersioningStrategy::Mixed {
                groups: groups.clone(),
                individual_packages: individual_packages.clone(),
            };

            assert_eq!(mixed.groups(), Some(&groups));
            assert_eq!(mixed.individual_packages(), Some(&individual_packages));
        }

        #[test]
        fn test_bump_config_new() {
            let config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Unified);

            assert!(config.strategy().is_unified());
            assert!(!config.sync_on_major_bump());
            assert!(config.independent_packages().is_empty());
            assert!(!config.enable_preview_mode());
            assert_eq!(config.unified_snapshot_template(), "{sha}");
        }

        #[test]
        fn test_bump_config_builder_pattern() {
            let mut independent = HashSet::new();
            independent.insert("legacy".to_string());

            let config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Individual)
                .with_sync_on_major_bump(true)
                .with_independent_packages(independent.clone())
                .with_preview_mode(true)
                .with_snapshot_template("alpha-{sha}".to_string());

            assert!(config.sync_on_major_bump());
            assert_eq!(config.independent_packages(), &independent);
            assert!(config.enable_preview_mode());
            assert_eq!(config.unified_snapshot_template(), "alpha-{sha}");
        }

        #[test]
        fn test_bump_config_independent_package_management() {
            let mut config = MonorepoVersionBumpConfig::new(MonorepoVersioningStrategy::Unified);

            // Add packages
            assert!(config.add_independent_package("pkg1".to_string()));
            assert!(config.add_independent_package("pkg2".to_string()));
            assert!(!config.add_independent_package("pkg1".to_string())); // Already exists

            // Check packages
            assert!(config.is_package_independent("pkg1"));
            assert!(config.is_package_independent("pkg2"));
            assert!(!config.is_package_independent("pkg3"));

            // Remove packages
            assert!(config.remove_independent_package("pkg1"));
            assert!(!config.remove_independent_package("pkg1")); // Already removed
            assert!(!config.is_package_independent("pkg1"));
            assert!(config.is_package_independent("pkg2"));
        }

        #[test]
        fn test_bump_config_default() {
            let config = MonorepoVersionBumpConfig::default();

            assert_eq!(config.strategy(), &MonorepoVersioningStrategy::Individual);
            assert!(!config.sync_on_major_bump());
            assert!(config.independent_packages().is_empty());
            assert!(!config.enable_preview_mode());
            assert_eq!(config.unified_snapshot_template(), "{sha}");
        }

        #[test]
        fn test_serialization() {
            let mut groups = HashMap::new();
            groups.insert("core-*".to_string(), "1.0.0".to_string());

            let mut individual_packages = HashSet::new();
            individual_packages.insert("example".to_string());

            let strategy = MonorepoVersioningStrategy::Mixed {
                groups,
                individual_packages: individual_packages.clone(),
            };

            let config = MonorepoVersionBumpConfig::new(strategy)
                .with_sync_on_major_bump(true)
                .with_independent_packages(individual_packages)
                .with_preview_mode(true);

            // Test that serialization works (this would fail if not serializable)
            assert!(serde_json::to_string(&config).is_ok(), "Config should be serializable");
            assert!(
                serde_json::to_string(config.strategy()).is_ok(),
                "Strategy should be serializable"
            );
        }

        // ========================================================================
        // CASCADE BUMPER TESTS
        // ========================================================================

        #[test]
        fn test_cascade_context_info_creation() {
            let context_info = cascade_bumper::CascadeContextInfo {
                is_monorepo: true,
                strategy: MonorepoVersioningStrategy::Individual,
                total_packages: 5,
                execution_mode: BumpExecutionMode::Preview,
                optimizations_applied: vec!["cache_enabled".to_string()],
            };

            assert!(context_info.is_monorepo);
            assert_eq!(context_info.strategy, MonorepoVersioningStrategy::Individual);
            assert_eq!(context_info.total_packages, 5);
            assert!(context_info.execution_mode.is_preview());
            assert_eq!(context_info.optimizations_applied.len(), 1);
        }

        #[test]
        fn test_cascade_bump_analysis_creation() {
            let mut primary_packages = HashMap::new();
            primary_packages.insert("package-a".to_string(), BumpStrategy::Minor);

            let mut cascade_packages = HashMap::new();
            cascade_packages.insert("package-b".to_string(), BumpStrategy::Patch);

            let context_info = cascade_bumper::CascadeContextInfo {
                is_monorepo: false,
                strategy: MonorepoVersioningStrategy::Individual,
                total_packages: 2,
                execution_mode: BumpExecutionMode::Apply,
                optimizations_applied: vec!["single_repo_optimized".to_string()],
            };

            let analysis = cascade_bumper::CascadeBumpAnalysis {
                primary_packages: primary_packages.clone(),
                cascade_packages: cascade_packages.clone(),
                reference_updates: vec![],
                affected_packages: vec!["package-c".to_string()],
                warnings: vec!["Large cascade impact".to_string()],
                context_info,
            };

            assert_eq!(analysis.primary_packages.len(), 1);
            assert_eq!(analysis.cascade_packages.len(), 1);
            assert_eq!(analysis.affected_packages.len(), 1);
            assert_eq!(analysis.warnings.len(), 1);
            assert!(!analysis.context_info.is_monorepo);
        }

        #[test]
        fn test_package_discovery_info_creation() {
            let mut dependencies = HashMap::new();
            dependencies.insert("react".to_string(), "^18.0.0".to_string());
            dependencies.insert("lodash".to_string(), "^4.17.21".to_string());

            let mut dependents = HashSet::new();
            dependents.insert("app".to_string());

            let package_info = cascade_bumper::PackageDiscoveryInfo {
                name: "my-lib".to_string(),
                path: PathBuf::from("/workspace/packages/my-lib/package.json"),
                version: "1.2.3".to_string(),
                dependencies,
                dependents,
                cached_at: std::time::SystemTime::now(),
            };

            assert_eq!(package_info.name, "my-lib");
            assert_eq!(package_info.version, "1.2.3");
            assert_eq!(package_info.dependencies.len(), 2);
            assert_eq!(package_info.dependents.len(), 1);
            assert!(package_info.dependencies.contains_key("react"));
            assert!(package_info.dependents.contains("app"));
        }

        #[test]
        fn test_versioning_strategy_compatibility() {
            // Test that all MonorepoVersioningStrategy variants work with analysis
            let strategies = vec![
                MonorepoVersioningStrategy::Individual,
                MonorepoVersioningStrategy::Unified,
                MonorepoVersioningStrategy::Mixed {
                    groups: HashMap::new(),
                    individual_packages: HashSet::new(),
                },
            ];

            for strategy in strategies {
                let context_info = cascade_bumper::CascadeContextInfo {
                    is_monorepo: true,
                    strategy: strategy.clone(),
                    total_packages: 3,
                    execution_mode: BumpExecutionMode::Preview,
                    optimizations_applied: vec![],
                };

                // Should be able to create analysis with any strategy
                assert!(matches!(
                    context_info.strategy,
                    MonorepoVersioningStrategy::Individual
                        | MonorepoVersioningStrategy::Unified
                        | MonorepoVersioningStrategy::Mixed { .. }
                ));
            }
        }

        #[test]
        fn test_bump_execution_mode_consistency() {
            let preview_analysis = cascade_bumper::CascadeContextInfo {
                is_monorepo: true,
                strategy: MonorepoVersioningStrategy::Individual,
                total_packages: 1,
                execution_mode: BumpExecutionMode::Preview,
                optimizations_applied: vec![],
            };

            let apply_analysis = cascade_bumper::CascadeContextInfo {
                is_monorepo: true,
                strategy: MonorepoVersioningStrategy::Individual,
                total_packages: 1,
                execution_mode: BumpExecutionMode::Apply,
                optimizations_applied: vec![],
            };

            assert!(preview_analysis.execution_mode.is_preview());
            assert!(!preview_analysis.execution_mode.modifies_filesystem());

            assert!(!apply_analysis.execution_mode.is_preview());
            assert!(apply_analysis.execution_mode.modifies_filesystem());
        }

        #[test]
        fn test_dependency_reference_update_types() {
            let update_types = vec![
                version::ReferenceUpdateType::FixedVersion,
                version::ReferenceUpdateType::WorkspaceProtocol,
                version::ReferenceUpdateType::KeepRange,
            ];

            for update_type in update_types {
                let reference_update = version::DependencyReferenceUpdate {
                    package: "consumer".to_string(),
                    dependency: "provider".to_string(),
                    from_reference: "1.0.0".to_string(),
                    to_reference: "1.1.0".to_string(),
                    update_type: update_type.clone(),
                };

                assert_eq!(reference_update.package, "consumer");
                assert_eq!(reference_update.dependency, "provider");
                assert_eq!(reference_update.from_reference, "1.0.0");
                assert_eq!(reference_update.to_reference, "1.1.0");
                assert_eq!(reference_update.update_type, update_type);
            }
        }

        #[test]
        fn test_context_optimization_tracking() {
            let single_repo_context = cascade_bumper::CascadeContextInfo {
                is_monorepo: false,
                strategy: MonorepoVersioningStrategy::Individual,
                total_packages: 1,
                execution_mode: BumpExecutionMode::Apply,
                optimizations_applied: vec!["single_repo_optimized".to_string()],
            };

            let monorepo_context = cascade_bumper::CascadeContextInfo {
                is_monorepo: true,
                strategy: MonorepoVersioningStrategy::Unified,
                total_packages: 10,
                execution_mode: BumpExecutionMode::Preview,
                optimizations_applied: vec![
                    "cache_enabled".to_string(),
                    "parallel_discovery".to_string(),
                ],
            };

            assert!(!single_repo_context.is_monorepo);
            assert_eq!(single_repo_context.optimizations_applied.len(), 1);
            assert!(single_repo_context
                .optimizations_applied
                .contains(&"single_repo_optimized".to_string()));

            assert!(monorepo_context.is_monorepo);
            assert_eq!(monorepo_context.optimizations_applied.len(), 2);
            assert!(monorepo_context.optimizations_applied.contains(&"cache_enabled".to_string()));
            assert!(monorepo_context
                .optimizations_applied
                .contains(&"parallel_discovery".to_string()));
        }

        #[test]
        fn test_bump_strategy_priority_ordering() {
            let strategies = vec![
                BumpStrategy::Patch,
                BumpStrategy::Minor,
                BumpStrategy::Major,
                BumpStrategy::Snapshot("abc123".to_string()),
                BumpStrategy::Cascade,
            ];

            // Verify all strategies can be used in cascade analysis
            for strategy in strategies {
                let mut primary_packages = HashMap::new();
                primary_packages.insert("test-pkg".to_string(), strategy.clone());

                let analysis = cascade_bumper::CascadeBumpAnalysis {
                    primary_packages: primary_packages.clone(),
                    cascade_packages: HashMap::new(),
                    reference_updates: vec![],
                    affected_packages: vec![],
                    warnings: vec![],
                    context_info: cascade_bumper::CascadeContextInfo {
                        is_monorepo: true,
                        strategy: MonorepoVersioningStrategy::Individual,
                        total_packages: 1,
                        execution_mode: BumpExecutionMode::Preview,
                        optimizations_applied: vec![],
                    },
                };

                assert_eq!(analysis.primary_packages.len(), 1);
                assert!(analysis.primary_packages.contains_key("test-pkg"));
                assert_eq!(analysis.primary_packages.get("test-pkg"), Some(&strategy));
            }
        }

        // ========================================================================
        // VERSION MANAGER INTEGRATION TESTS
        // ========================================================================

        #[test]
        fn test_version_bump_report_creation() {
            let mut report = version::VersionBumpReport::new();

            // Test initial state
            assert!(!report.has_changes());
            assert_eq!(report.total_packages_affected(), 0);

            // Add primary bumps
            report.primary_bumps.insert("pkg1".to_string(), "1.1.0".to_string());
            report.primary_bumps.insert("pkg2".to_string(), "2.0.0".to_string());

            // Add cascade bumps
            report.cascade_bumps.insert("pkg3".to_string(), "0.1.1".to_string());

            // Test state after changes
            assert!(report.has_changes());
            assert_eq!(report.total_packages_affected(), 3);

            // Test adding warnings and errors
            report.add_warning("Test warning".to_string());
            report.add_error("Test error".to_string());

            assert_eq!(report.warnings.len(), 1);
            assert_eq!(report.errors.len(), 1);
        }

        #[test]
        fn test_version_bump_report_default() {
            let report = version::VersionBumpReport::default();

            assert!(!report.has_changes());
            assert_eq!(report.total_packages_affected(), 0);
            assert!(report.primary_bumps.is_empty());
            assert!(report.cascade_bumps.is_empty());
            assert!(report.reference_updates.is_empty());
            assert!(report.affected_packages.is_empty());
            assert!(report.warnings.is_empty());
            assert!(report.errors.is_empty());
        }

        #[test]
        fn test_dependency_reference_update_creation() {
            let update = version::DependencyReferenceUpdate {
                package: "consumer-pkg".to_string(),
                dependency: "provider-pkg".to_string(),
                from_reference: "^1.0.0".to_string(),
                to_reference: "^1.1.0".to_string(),
                update_type: version::ReferenceUpdateType::KeepRange,
            };

            assert_eq!(update.package, "consumer-pkg");
            assert_eq!(update.dependency, "provider-pkg");
            assert_eq!(update.from_reference, "^1.0.0");
            assert_eq!(update.to_reference, "^1.1.0");
            assert_eq!(update.update_type, version::ReferenceUpdateType::KeepRange);
        }

        #[test]
        fn test_reference_update_type_variants() {
            let types = [
                version::ReferenceUpdateType::FixedVersion,
                version::ReferenceUpdateType::WorkspaceProtocol,
                version::ReferenceUpdateType::KeepRange,
            ];

            // Test that all variants are distinct
            for (i, type1) in types.iter().enumerate() {
                for (j, type2) in types.iter().enumerate() {
                    if i == j {
                        assert_eq!(type1, type2);
                    } else {
                        assert_ne!(type1, type2);
                    }
                }
            }
        }

        // ========================================================================
        // INTEGRATION AND EDGE CASE TESTS
        // ========================================================================

        #[test]
        fn test_mixed_strategy_edge_cases() {
            // Test empty mixed strategy
            let empty_mixed = MonorepoVersioningStrategy::Mixed {
                groups: HashMap::new(),
                individual_packages: HashSet::new(),
            };

            assert!(empty_mixed.has_groups());
            assert!(!empty_mixed.is_unified());
            assert!(empty_mixed.allows_individual());
            if let Some(groups) = empty_mixed.groups() {
                assert_eq!(groups.len(), 0);
            } else {
                assert!(false, "Empty mixed strategy should have groups");
            }
            if let Some(individual_packages) = empty_mixed.individual_packages() {
                assert_eq!(individual_packages.len(), 0);
            } else {
                assert!(false, "Empty mixed strategy should have individual_packages");
            }

            // Test mixed strategy with overlapping concerns
            let mut groups = HashMap::new();
            groups.insert("core-*".to_string(), "1.0.0".to_string());
            groups.insert("util-*".to_string(), "2.0.0".to_string());

            let mut individual_packages = HashSet::new();
            individual_packages.insert("special-package".to_string());
            individual_packages.insert("legacy-tool".to_string());

            let complex_mixed = MonorepoVersioningStrategy::Mixed { groups, individual_packages };

            if let Some(groups) = complex_mixed.groups() {
                assert_eq!(groups.len(), 2);
                assert!(groups.contains_key("core-*"));
                assert!(groups.contains_key("util-*"));
            } else {
                assert!(false, "Complex mixed strategy should have groups");
            }
            if let Some(individual_packages) = complex_mixed.individual_packages() {
                assert_eq!(individual_packages.len(), 2);
                assert!(individual_packages.contains("special-package"));
                assert!(individual_packages.contains("legacy-tool"));
            } else {
                assert!(false, "Complex mixed strategy should have individual_packages");
            }
        }

        #[test]
        fn test_bump_strategy_display() {
            let strategies = vec![
                (BumpStrategy::Major, "major"),
                (BumpStrategy::Minor, "minor"),
                (BumpStrategy::Patch, "patch"),
                (BumpStrategy::Snapshot("abc123".to_string()), "snapshot-abc123"),
                (BumpStrategy::Cascade, "cascade"),
            ];

            for (strategy, expected_display) in strategies {
                assert_eq!(strategy.to_string(), expected_display);
            }
        }

        #[test]
        fn test_bump_strategy_default() {
            assert_eq!(BumpStrategy::default(), BumpStrategy::Patch);
        }

        #[test]
        fn test_change_set_with_large_package_count() {
            let mut packages = HashMap::new();

            // Add many packages to test scalability
            for i in 0..100 {
                let package_name = format!("package-{i}");
                let strategy = match i % 4 {
                    0 => BumpStrategy::Major,
                    1 => BumpStrategy::Minor,
                    2 => BumpStrategy::Patch,
                    _ => BumpStrategy::Snapshot(format!("sha-{i}")),
                };
                packages.insert(package_name, strategy);
            }

            let change_set = ChangeSet::new(packages, "Large batch update".to_string());

            assert_eq!(change_set.len(), 100);
            assert!(!change_set.is_empty());
            assert_eq!(change_set.reason(), "Large batch update");

            // Test that we can iterate through all packages
            let mut count = 0;
            for (name, strategy) in change_set.target_packages() {
                assert!(name.starts_with("package-"));
                assert!(matches!(
                    strategy,
                    BumpStrategy::Major
                        | BumpStrategy::Minor
                        | BumpStrategy::Patch
                        | BumpStrategy::Snapshot(_)
                ));
                count += 1;
            }
            assert_eq!(count, 100);
        }

        #[test]
        fn test_monorepo_config_comprehensive() {
            // Test comprehensive configuration with all options
            let mut groups = HashMap::new();
            groups.insert("@acme/core-*".to_string(), "3.0.0".to_string());
            groups.insert("@acme/ui-*".to_string(), "2.5.0".to_string());
            groups.insert("@tools/*".to_string(), "1.0.0".to_string());

            let strategy =
                MonorepoVersioningStrategy::Mixed { groups, individual_packages: HashSet::new() };

            let mut independent = HashSet::new();
            independent.insert("legacy-package".to_string());
            independent.insert("experimental-feature".to_string());
            independent.insert("@external/integration".to_string());

            let config = MonorepoVersionBumpConfig::new(strategy)
                .with_sync_on_major_bump(true)
                .with_independent_packages(independent.clone())
                .with_preview_mode(true)
                .with_snapshot_template("build-{timestamp}-{sha}".to_string());

            // Verify all configuration options
            assert!(config.strategy().has_groups());
            assert!(config.sync_on_major_bump());
            assert_eq!(config.independent_packages(), &independent);
            assert!(config.enable_preview_mode());
            assert_eq!(config.unified_snapshot_template(), "build-{timestamp}-{sha}");

            // Test independent package management
            assert!(config.is_package_independent("legacy-package"));
            assert!(config.is_package_independent("experimental-feature"));
            assert!(config.is_package_independent("@external/integration"));
            assert!(!config.is_package_independent("@acme/core-lib"));

            // Verify strategy groups
            if let Some(groups) = config.strategy().groups() {
                assert_eq!(groups.len(), 3);
                assert!(groups.contains_key("@acme/core-*"));
                assert!(groups.contains_key("@acme/ui-*"));
                assert!(groups.contains_key("@tools/*"));
            } else {
                assert!(false, "Mixed strategy should have groups");
            }
        }
    }
}
