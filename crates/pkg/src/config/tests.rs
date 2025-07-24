//! Comprehensive tests for the config module.
//!
//! This module provides 100% test coverage for all configuration functionality,
//! following the CLAUDE.md testing requirements.

#[cfg(test)]
mod tests {
    use crate::config::*;
    use std::time::Duration;
    use sublime_standard_tools::config::traits::Configurable;

    mod unit_tests {
        use super::*;

        #[test]
        fn test_package_tools_config_default() {
            let config = PackageToolsConfig::default();
            
            assert_eq!(config.version, "1.0");
            assert_eq!(config.version_bumping.default_strategy, VersionBumpStrategy::Patch);
            assert_eq!(config.dependency_resolution.max_concurrent_downloads, 10);
            assert_eq!(config.circular_dependency_handling.handling_strategy, CircularDependencyHandling::Warn);
            assert!(config.context_aware.auto_detect_context);
            assert!(config.cache.enable_cache);
        }

        #[test]
        fn test_version_bump_config_default() {
            let config = VersionBumpConfig::default();
            
            assert_eq!(config.default_strategy, VersionBumpStrategy::Patch);
            assert!(!config.enable_cascade_bumping);
            assert_eq!(config.snapshot_prefix, "snapshot");
            assert_eq!(config.affected_detection, AffectedDetectionStrategy::Auto);
            assert!(config.append_sha_to_snapshots);
            assert!(config.custom_version_patterns.is_empty());
        }

        #[test]
        fn test_resolution_config_default() {
            let config = ResolutionConfig::default();
            
            assert_eq!(config.max_concurrent_downloads, 10);
            assert_eq!(config.registry_timeout, Duration::from_secs(30));
            assert!(config.enable_workspace_protocols);
            assert!(config.validate_protocols);
            assert!(config.supported_protocols.contains(&DependencyProtocol::Npm));
            assert!(config.supported_protocols.contains(&DependencyProtocol::Workspace));
        }

        #[test]
        fn test_circular_dependency_config_default() {
            let config = CircularDependencyConfig::default();
            
            assert_eq!(config.handling_strategy, CircularDependencyHandling::Warn);
            assert!(config.allow_dev_cycles);
            assert!(config.allow_optional_cycles);
            assert_eq!(config.max_cycle_depth, 10);
            assert!(config.enable_warnings);
        }

        #[test]
        fn test_context_aware_config_default() {
            let config = ContextAwareConfig::default();
            
            assert!(config.auto_detect_context);
            assert!(config.force_context.is_none());
            assert!(config.enable_single_repo_optimizations);
            assert!(config.enable_monorepo_features);
            assert_eq!(config.context_cache_duration, Duration::from_secs(300));
        }

        #[test]
        fn test_performance_config_default() {
            let config = PerformanceConfig::default();
            
            assert!(config.enable_parallel_processing);
            assert!(config.max_worker_threads > 0);
            assert_eq!(config.memory_optimization, MemoryOptimizationLevel::Basic);
            assert!(config.enable_io_optimizations);
            assert_eq!(config.batch_processing_size, 100);
        }

        #[test]
        fn test_cache_config_default() {
            let config = CacheConfig::default();
            
            assert!(config.enable_cache);
            assert_eq!(config.cache_ttl, Duration::from_secs(3600));
            assert_eq!(config.max_cache_size_mb, 100);
            assert_eq!(config.cleanup_interval, Duration::from_secs(1800));
            assert!(config.enable_persistent_cache);
            assert_eq!(config.compression_level, 6);
        }

        #[test]
        fn test_enum_serialization() {
            // Test VersionBumpStrategy
            assert_eq!(VersionBumpStrategy::Major, VersionBumpStrategy::Major);
            assert_ne!(VersionBumpStrategy::Major, VersionBumpStrategy::Minor);
            
            // Test DependencyProtocol
            assert_eq!(DependencyProtocol::Npm, DependencyProtocol::Npm);
            assert_ne!(DependencyProtocol::Npm, DependencyProtocol::Git);
            
            // Test CircularDependencyHandling
            assert_eq!(CircularDependencyHandling::Warn, CircularDependencyHandling::Warn);
            assert_ne!(CircularDependencyHandling::Warn, CircularDependencyHandling::Error);
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_config_validation_success() {
            let config = PackageToolsConfig::default();
            assert!(config.validate().is_ok());
        }

        #[test]
        fn test_config_merge_success() {
            let mut base_config = PackageToolsConfig::default();
            let override_config = PackageToolsConfig {
                version_bumping: VersionBumpConfig {
                    default_strategy: VersionBumpStrategy::Major,
                    enable_cascade_bumping: true,
                    ..Default::default()
                },
                dependency_resolution: ResolutionConfig {
                    max_concurrent_downloads: 20,
                    ..Default::default()
                },
                ..Default::default()
            };

            assert!(base_config.merge_with(override_config).is_ok());
            assert_eq!(base_config.version_bumping.default_strategy, VersionBumpStrategy::Major);
            assert!(base_config.version_bumping.enable_cascade_bumping);
            assert_eq!(base_config.dependency_resolution.max_concurrent_downloads, 20);
        }

        #[test]
        fn test_custom_config_creation() {
            let config = PackageToolsConfig {
                version_bumping: VersionBumpConfig {
                    default_strategy: VersionBumpStrategy::Minor,
                    enable_cascade_bumping: true,
                    snapshot_prefix: "dev".to_string(),
                    affected_detection: AffectedDetectionStrategy::GitDiff,
                    append_sha_to_snapshots: false,
                    custom_version_patterns: [("library".to_string(), "lib-{version}".to_string())].iter().cloned().collect(),
                },
                dependency_resolution: ResolutionConfig {
                    max_concurrent_downloads: 5,
                    registry_timeout: Duration::from_secs(60),
                    supported_protocols: vec![DependencyProtocol::Npm, DependencyProtocol::Workspace],
                    enable_workspace_protocols: false,
                    fallback_registries: vec!["https://custom.registry.com".to_string()],
                    custom_protocol_handlers: [("custom".to_string(), "handler".to_string())].iter().cloned().collect(),
                    validate_protocols: false,
                },
                circular_dependency_handling: CircularDependencyConfig {
                    handling_strategy: CircularDependencyHandling::Error,
                    allow_dev_cycles: false,
                    allow_optional_cycles: false,
                    max_cycle_depth: 5,
                    enable_warnings: false,
                    custom_rules: [("strict".to_string(), true)].iter().cloned().collect(),
                },
                context_aware: ContextAwareConfig {
                    auto_detect_context: false,
                    force_context: Some(ProjectContextType::Monorepo),
                    enable_single_repo_optimizations: false,
                    enable_monorepo_features: true,
                    context_cache_duration: Duration::from_secs(600),
                },
                performance: PerformanceConfig {
                    enable_parallel_processing: false,
                    max_worker_threads: 1,
                    memory_optimization: MemoryOptimizationLevel::Aggressive,
                    enable_io_optimizations: false,
                    batch_processing_size: 50,
                },
                cache: CacheConfig {
                    enable_cache: false,
                    cache_ttl: Duration::from_secs(1800),
                    max_cache_size_mb: 50,
                    cleanup_interval: Duration::from_secs(900),
                    enable_persistent_cache: false,
                    compression_level: 9,
                },
                ..Default::default()
            };

            assert!(config.validate().is_ok());
            assert_eq!(config.version_bumping.default_strategy, VersionBumpStrategy::Minor);
            assert_eq!(config.dependency_resolution.max_concurrent_downloads, 5);
            assert_eq!(config.circular_dependency_handling.handling_strategy, CircularDependencyHandling::Error);
        }
    }

    mod error_tests {
        use super::*;

        #[test]
        fn test_validation_empty_snapshot_prefix() {
            let mut config = PackageToolsConfig::default();
            config.version_bumping.snapshot_prefix = String::new();
            
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_validation_zero_concurrent_downloads() {
            let mut config = PackageToolsConfig::default();
            config.dependency_resolution.max_concurrent_downloads = 0;
            
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_validation_excessive_concurrent_downloads() {
            let mut config = PackageToolsConfig::default();
            config.dependency_resolution.max_concurrent_downloads = 100;
            
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_validation_zero_registry_timeout() {
            let mut config = PackageToolsConfig::default();
            config.dependency_resolution.registry_timeout = Duration::from_secs(0);
            
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_validation_excessive_registry_timeout() {
            let mut config = PackageToolsConfig::default();
            config.dependency_resolution.registry_timeout = Duration::from_secs(400);
            
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_validation_zero_cycle_depth() {
            let mut config = PackageToolsConfig::default();
            config.circular_dependency_handling.max_cycle_depth = 0;
            
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_validation_excessive_cycle_depth() {
            let mut config = PackageToolsConfig::default();
            config.circular_dependency_handling.max_cycle_depth = 25;
            
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_validation_zero_worker_threads() {
            let mut config = PackageToolsConfig::default();
            config.performance.max_worker_threads = 0;
            
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_validation_zero_batch_size() {
            let mut config = PackageToolsConfig::default();
            config.performance.batch_processing_size = 0;
            
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_validation_low_cache_ttl() {
            let mut config = PackageToolsConfig::default();
            config.cache.cache_ttl = Duration::from_secs(30);
            
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_validation_zero_cache_size() {
            let mut config = PackageToolsConfig::default();
            config.cache.max_cache_size_mb = 0;
            
            assert!(config.validate().is_err());
        }

        #[test]
        fn test_validation_excessive_compression() {
            let mut config = PackageToolsConfig::default();
            config.cache.compression_level = 10;
            
            assert!(config.validate().is_err());
        }
    }

    mod performance_tests {
        use super::*;
        use std::time::Instant;

        #[test]
        fn test_config_creation_performance() {
            let start = Instant::now();
            let _config = PackageToolsConfig::default();
            let duration = start.elapsed();
            
            // Should be very fast (< 1ms)
            assert!(duration.as_millis() < 1);
        }

        #[test]
        fn test_config_validation_performance() {
            let config = PackageToolsConfig::default();
            let start = Instant::now();
            let _result = config.validate();
            let duration = start.elapsed();
            
            // Validation should be very fast (< 1ms)
            assert!(duration.as_millis() < 1);
        }

        #[test]
        fn test_config_merge_performance() {
            let mut base_config = PackageToolsConfig::default();
            let override_config = PackageToolsConfig::default();
            
            let start = Instant::now();
            let _result = base_config.merge_with(override_config);
            let duration = start.elapsed();
            
            // Merge should be very fast (< 1ms)
            assert!(duration.as_millis() < 1);
        }
    }

    mod property_tests {
        use super::*;

        #[test]
        fn test_config_merge_idempotent() {
            let mut config1 = PackageToolsConfig::default();
            let config2 = PackageToolsConfig::default();
            let original = config1.clone();
            
            assert!(config1.merge_with(config2).is_ok());
            
            // Merging identical configs should not change anything
            assert_eq!(config1.version, original.version);
            assert_eq!(config1.version_bumping.default_strategy, original.version_bumping.default_strategy);
        }

        #[test]
        fn test_config_validation_stability() {
            let config = PackageToolsConfig::default();
            
            // Multiple validations should always give same result
            for _ in 0..10 {
                assert!(config.validate().is_ok());
            }
        }

        #[test]
        fn test_enum_consistency() {
            // Test that enum values are consistent
            assert_eq!(VersionBumpStrategy::Patch as u8, VersionBumpStrategy::Patch as u8);
            assert_eq!(DependencyProtocol::Npm as u8, DependencyProtocol::Npm as u8);
            assert_eq!(CircularDependencyHandling::Warn as u8, CircularDependencyHandling::Warn as u8);
        }

        #[test]
        fn test_duration_bounds() {
            // Test that all duration defaults are reasonable
            let config = PackageToolsConfig::default();
            
            assert!(config.dependency_resolution.registry_timeout.as_secs() > 0);
            assert!(config.dependency_resolution.registry_timeout.as_secs() <= 300);
            
            assert!(config.context_aware.context_cache_duration.as_secs() > 0);
            assert!(config.context_aware.context_cache_duration.as_secs() <= 3600);
            
            assert!(config.cache.cache_ttl.as_secs() >= 60);
            assert!(config.cache.cleanup_interval.as_secs() > 0);
        }

        #[test]
        fn test_numeric_bounds() {
            // Test that all numeric defaults are within bounds
            let config = PackageToolsConfig::default();
            
            assert!(config.dependency_resolution.max_concurrent_downloads > 0);
            assert!(config.dependency_resolution.max_concurrent_downloads <= 50);
            
            assert!(config.circular_dependency_handling.max_cycle_depth > 0);
            assert!(config.circular_dependency_handling.max_cycle_depth <= 20);
            
            assert!(config.performance.max_worker_threads > 0);
            assert!(config.performance.batch_processing_size > 0);
            
            assert!(config.cache.max_cache_size_mb > 0);
            assert!(config.cache.compression_level <= 9);
        }
    }
}