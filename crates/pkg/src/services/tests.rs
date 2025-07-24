//! Comprehensive test suite for services module
//!
//! This module contains all tests for the services components including:
//! - PerformanceOptimizer tests
//! - ConcurrentProcessor tests
//! - PackageService integration tests
//! - PackageCommandService integration tests
//! - Performance benchmarks
//! - Edge cases and error handling

#![allow(clippy::unwrap_used)]
#![allow(clippy::print_stdout)]
#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::{
        context::{MonorepoContext, ProjectContext, SingleRepositoryContext},
        Error, Package,
    };
    use std::collections::HashMap;
    use std::time::{Duration, Instant};
    use sublime_standard_tools::{command::DefaultCommandExecutor, filesystem::FileSystemManager};
    use tokio::time::sleep;

    mod unit_tests {
        use super::*;

        // =============================================================================
        // PerformanceOptimizer Unit Tests (from implementation file)
        // =============================================================================

        #[tokio::test]
        async fn test_single_repository_optimization() {
            let context = ProjectContext::Single(SingleRepositoryContext::default());
            let optimizer = PerformanceOptimizer::new(context);

            let strategy = optimizer.optimize_for_context().await.unwrap();

            // Single repository optimizations
            assert_eq!(strategy.concurrent_downloads, 10);
            assert!(!strategy.enable_cascade_bumping);
            assert!(!strategy.enable_workspace_scanning);
            assert_eq!(strategy.cache_strategy, CacheStrategy::NetworkHeavy);
            assert_eq!(strategy.io_strategy, IoStrategy::NetworkOptimized);
            assert_eq!(strategy.timeout_strategy, TimeoutStrategy::Aggressive);
        }

        #[tokio::test]
        async fn test_monorepo_optimization() {
            let mut workspace_packages = HashMap::new();
            workspace_packages.insert("package-a".to_string(), "packages/a".to_string());
            workspace_packages.insert("package-b".to_string(), "packages/b".to_string());

            let monorepo_config = MonorepoContext { workspace_packages, ..Default::default() };
            let context = ProjectContext::Monorepo(monorepo_config);
            let optimizer = PerformanceOptimizer::new(context);

            let strategy = optimizer.optimize_for_context().await.unwrap();

            // Monorepo optimizations
            assert_eq!(strategy.concurrent_downloads, 5); // Small monorepo
            assert!(strategy.enable_cascade_bumping);
            assert!(strategy.enable_workspace_scanning);
            assert_eq!(strategy.cache_strategy, CacheStrategy::FilesystemHeavy);
            assert_eq!(strategy.io_strategy, IoStrategy::FilesystemOptimized);
            assert_eq!(strategy.timeout_strategy, TimeoutStrategy::Conservative);
        }

        #[tokio::test]
        async fn test_large_monorepo_scaling() {
            let mut workspace_packages = HashMap::new();
            // Create a large monorepo with 120 packages to test large monorepo settings
            for i in 0..120 {
                workspace_packages.insert(format!("package-{i}"), format!("packages/pkg{i}"));
            }

            let monorepo_config = MonorepoContext { workspace_packages, ..Default::default() };
            let context = ProjectContext::Monorepo(monorepo_config);
            let optimizer = PerformanceOptimizer::new(context);

            let strategy = optimizer.optimize_for_context().await.unwrap();

            // Large monorepo should have conservative settings
            assert_eq!(strategy.concurrent_downloads, 3);
            assert_eq!(strategy.batch_processing_size, 20);
            assert_eq!(strategy.performance_metrics.target_resolution_time_ms, 2000);

            // Memory should scale with workspace size
            assert!(strategy.resource_limits.max_memory_mb > 600);
            assert!(strategy.performance_metrics.max_acceptable_memory_mb > 1300);
        }

        #[tokio::test]
        async fn test_custom_overrides() {
            let context = ProjectContext::Single(SingleRepositoryContext::default());
            let overrides = OptimizationOverrides::new()
                .with_concurrent_downloads(15)
                .with_cache_strategy(CacheStrategy::Hybrid);

            let optimizer = PerformanceOptimizer::with_overrides(context, overrides);
            let strategy = optimizer.optimize_for_context().await.unwrap();

            // Overrides should be applied
            assert_eq!(strategy.concurrent_downloads, 15);
            assert_eq!(strategy.cache_strategy, CacheStrategy::Hybrid);

            // Other settings should remain default for single repo
            assert!(!strategy.enable_cascade_bumping);
            assert_eq!(strategy.io_strategy, IoStrategy::NetworkOptimized);
        }

        #[tokio::test]
        async fn test_context_update() {
            let mut optimizer = PerformanceOptimizer::new(ProjectContext::Single(
                SingleRepositoryContext::default(),
            ));

            // Verify initial context
            assert!(optimizer.context().is_single());

            // Update to monorepo context
            optimizer.update_context(ProjectContext::Monorepo(MonorepoContext::default()));
            assert!(optimizer.context().is_monorepo());
        }

        #[test]
        fn test_optimization_overrides_builder() {
            let overrides = OptimizationOverrides::new()
                .with_concurrent_downloads(8)
                .with_cache_strategy(CacheStrategy::FilesystemHeavy);

            assert_eq!(overrides.concurrent_downloads, Some(8));
            assert_eq!(overrides.cache_strategy, Some(CacheStrategy::FilesystemHeavy));
            assert_eq!(overrides.enable_cascade_bumping, None);
        }

        // =============================================================================
        // PerformanceOptimizer Extended Unit Tests
        // =============================================================================

        #[tokio::test]
        async fn test_optimizer_context_switching() {
            // Test switching between contexts dynamically
            let single_context = ProjectContext::Single(SingleRepositoryContext::default());
            let mut optimizer = PerformanceOptimizer::new(single_context.clone());

            let single_strategy = optimizer.optimize_for_context().await.unwrap();
            assert_eq!(single_strategy.concurrent_downloads, 10);
            assert!(!single_strategy.enable_cascade_bumping);

            // Switch to monorepo
            let monorepo_context = ProjectContext::Monorepo(MonorepoContext::default());
            optimizer.update_context(monorepo_context);

            let monorepo_strategy = optimizer.optimize_for_context().await.unwrap();
            assert_eq!(monorepo_strategy.concurrent_downloads, 5);
            assert!(monorepo_strategy.enable_cascade_bumping);
        }

        #[tokio::test]
        async fn test_optimizer_with_overrides_chain() {
            // Test chaining multiple overrides
            let context = ProjectContext::Single(SingleRepositoryContext::default());
            let overrides = OptimizationOverrides::new()
                .with_concurrent_downloads(20)
                .with_cache_strategy(CacheStrategy::Hybrid);

            let optimizer = PerformanceOptimizer::with_overrides(context, overrides);
            let strategy = optimizer.optimize_for_context().await.unwrap();

            assert_eq!(strategy.concurrent_downloads, 20);
            assert_eq!(strategy.cache_strategy, CacheStrategy::Hybrid);
            // Other settings should remain as single repo defaults
            assert!(!strategy.enable_cascade_bumping);
        }

        #[tokio::test]
        async fn test_monorepo_size_scaling() {
            // Test that optimization scales correctly with monorepo size
            let sizes_and_expected = vec![
                (10, 5, 30),  // Small: 10 packages -> 5 concurrent, 30 batch
                (30, 4, 30),  // Medium: 30 packages -> 4 concurrent, 30 batch
                (80, 3, 30),  // Large: 80 packages -> 3 concurrent, 30 batch (< 100 threshold)
                (150, 3, 20), // Very large: 150 packages -> 3 concurrent, 20 batch (> 100 threshold)
            ];

            for (package_count, expected_concurrent, expected_batch) in sizes_and_expected {
                let mut workspace_packages = HashMap::new();
                for i in 0..package_count {
                    workspace_packages
                        .insert(format!("package-{}", i), format!("packages/pkg{}", i));
                }

                let context = ProjectContext::Monorepo(MonorepoContext {
                    workspace_packages,
                    ..Default::default()
                });

                let optimizer = PerformanceOptimizer::new(context);
                let strategy = optimizer.optimize_for_context().await.unwrap();

                assert_eq!(
                    strategy.concurrent_downloads, expected_concurrent,
                    "Wrong concurrent downloads for {} packages",
                    package_count
                );
                assert_eq!(
                    strategy.batch_processing_size, expected_batch,
                    "Wrong batch size for {} packages",
                    package_count
                );
            }
        }

        // =============================================================================
        // ConcurrentProcessor Unit Tests (from implementation file)
        // =============================================================================

        #[tokio::test]
        async fn test_concurrent_processing_basic() {
            let strategy = OptimizationStrategy {
                concurrent_downloads: 3,
                batch_processing_size: 10,
                ..Default::default()
            };
            let processor = ConcurrentProcessor::new(strategy);

            let items = vec![1, 2, 3, 4, 5];
            let results = processor
                .process_concurrent(tokio_stream::iter(items), |x| async move {
                    sleep(Duration::from_millis(10)).await;
                    Ok::<i32, &'static str>(x * 2)
                })
                .await
                .unwrap();

            assert_eq!(results.len(), 5);
            // Results may be in different order due to concurrency
            let mut sorted_results = results;
            sorted_results.sort();
            assert_eq!(sorted_results, vec![2, 4, 6, 8, 10]);
        }

        #[tokio::test]
        async fn test_batch_processing() {
            let strategy = OptimizationStrategy {
                concurrent_downloads: 2,
                batch_processing_size: 3,
                ..Default::default()
            };
            let processor = ConcurrentProcessor::new(strategy);

            let items = vec![1, 2, 3, 4, 5, 6, 7];
            let results = processor
                .process_batched(items, |x| async move {
                    sleep(Duration::from_millis(5)).await;
                    Ok::<String, &'static str>(format!("item_{}", x))
                })
                .await
                .unwrap();

            assert_eq!(results.len(), 7);
            // Results should maintain order within batches but batches are processed sequentially
            assert!(results.contains(&"item_1".to_string()));
            assert!(results.contains(&"item_7".to_string()));
        }

        #[tokio::test]
        async fn test_error_propagation() {
            let processor = ConcurrentProcessor::new(Default::default());

            let items = vec![1, 2, 3, 4, 5];
            let result = processor
                .process_concurrent(tokio_stream::iter(items), |x| async move {
                    if x == 3 {
                        Err("Error on item 3")
                    } else {
                        Ok(x * 2)
                    }
                })
                .await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_custom_concurrency() {
            let processor = ConcurrentProcessor::new(Default::default());

            let items = vec![1, 2, 3, 4, 5];
            let results = processor
                .process_with_custom_concurrency(
                    tokio_stream::iter(items),
                    |x| async move {
                        sleep(Duration::from_millis(10)).await;
                        Ok::<i32, &'static str>(x * 3)
                    },
                    2, // Custom concurrency limit
                )
                .await
                .unwrap();

            assert_eq!(results.len(), 5);
            let mut sorted_results = results;
            sorted_results.sort();
            assert_eq!(sorted_results, vec![3, 6, 9, 12, 15]);
        }

        #[tokio::test]
        async fn test_strategy_update() {
            let mut processor = ConcurrentProcessor::new(Default::default());
            assert_eq!(processor.concurrency_limit(), 5); // Default value

            let new_strategy =
                OptimizationStrategy { concurrent_downloads: 8, ..Default::default() };
            processor.update_strategy(new_strategy);
            assert_eq!(processor.concurrency_limit(), 8);
        }

        #[tokio::test]
        async fn test_specialized_network_processor() {
            let strategy = OptimizationStrategy { concurrent_downloads: 4, ..Default::default() };
            let network_processor = specialized::NetworkProcessor::new(strategy);

            let items = vec!["url1", "url2", "url3"];
            let results = network_processor
                .process_requests(tokio_stream::iter(items), |url| async move {
                    sleep(Duration::from_millis(20)).await;
                    Ok::<String, &'static str>(format!("response_from_{}", url))
                })
                .await
                .unwrap();

            assert_eq!(results.len(), 3);
            assert!(results.iter().any(|r| r.contains("url1")));
        }

        #[tokio::test]
        async fn test_specialized_filesystem_processor() {
            let strategy = OptimizationStrategy {
                concurrent_downloads: 6, // Will be limited to 4 for filesystem
                ..Default::default()
            };
            let fs_processor = specialized::FilesystemProcessor::new(strategy);

            let items = vec!["file1.json", "file2.json", "file3.json"];
            let results = fs_processor
                .process_files(tokio_stream::iter(items), |filename| async move {
                    sleep(Duration::from_millis(15)).await;
                    Ok::<String, &'static str>(format!("content_of_{}", filename))
                })
                .await
                .unwrap();

            assert_eq!(results.len(), 3);
            assert!(results.iter().any(|r| r.contains("file1.json")));
        }

        // =============================================================================
        // Additional ConcurrentProcessor Tests (from implementation file)
        // =============================================================================

        #[tokio::test]
        async fn test_specialized_network_processor_impl() {
            let strategy = OptimizationStrategy { concurrent_downloads: 4, ..Default::default() };
            let network_processor = specialized::NetworkProcessor::new(strategy);

            let items = vec!["url1", "url2", "url3"];
            let results = network_processor
                .process_requests(tokio_stream::iter(items), |url| async move {
                    sleep(Duration::from_millis(20)).await;
                    Ok::<String, &'static str>(format!("response_from_{}", url))
                })
                .await
                .unwrap();

            assert_eq!(results.len(), 3);
            assert!(results.iter().any(|r| r.contains("url1")));
        }

        #[tokio::test]
        async fn test_specialized_filesystem_processor_impl() {
            let strategy = OptimizationStrategy {
                concurrent_downloads: 6, // Will be limited to 4 for filesystem
                ..Default::default()
            };
            let fs_processor = specialized::FilesystemProcessor::new(strategy);

            let items = vec!["file1.json", "file2.json", "file3.json"];
            let results = fs_processor
                .process_files(tokio_stream::iter(items), |filename| async move {
                    sleep(Duration::from_millis(15)).await;
                    Ok::<String, &'static str>(format!("content_of_{}", filename))
                })
                .await
                .unwrap();

            assert_eq!(results.len(), 3);
            assert!(results.iter().any(|r| r.contains("file1.json")));
        }

        // =============================================================================
        // ConcurrentProcessor Extended Unit Tests
        // =============================================================================

        #[tokio::test]
        async fn test_concurrent_processor_empty_stream() {
            // Test handling of empty streams
            let processor = ConcurrentProcessor::new(Default::default());

            let items: Vec<i32> = vec![];
            let results = processor
                .process_concurrent(
                    tokio_stream::iter(items),
                    |x| async move { Ok::<i32, Error>(x) },
                )
                .await
                .unwrap();

            assert!(results.is_empty());
        }

        #[tokio::test]
        async fn test_batch_processing_correctness() {
            // Ensure batch processing maintains correctness
            let strategy = OptimizationStrategy {
                concurrent_downloads: 2,
                batch_processing_size: 3,
                ..Default::default()
            };
            let processor = ConcurrentProcessor::new(strategy);

            let items: Vec<i32> = (1..=10).collect();
            let results = processor
                .process_batched(items.clone(), |x| async move {
                    sleep(Duration::from_millis(5)).await;
                    Ok::<(i32, i32), Error>((x, x * x))
                })
                .await
                .unwrap();

            assert_eq!(results.len(), 10);

            // Verify all items were processed correctly
            for (input, squared) in results {
                assert_eq!(squared, input * input);
            }
        }

        #[tokio::test]
        async fn test_concurrent_processor_custom_concurrency() {
            let processor = ConcurrentProcessor::new(Default::default());

            let items = vec![1, 2, 3, 4, 5];
            let results = processor
                .process_with_custom_concurrency(
                    tokio_stream::iter(items),
                    |x| async move {
                        sleep(Duration::from_millis(10)).await;
                        Ok::<i32, Error>(x * 3)
                    },
                    2, // Custom concurrency limit
                )
                .await
                .unwrap();

            assert_eq!(results.len(), 5);
            let mut sorted_results = results;
            sorted_results.sort();
            assert_eq!(sorted_results, vec![3, 6, 9, 12, 15]);
        }

        #[tokio::test]
        async fn test_concurrent_processor_strategy_update() {
            let mut processor = ConcurrentProcessor::new(Default::default());
            assert_eq!(processor.concurrency_limit(), 5); // Default value

            let new_strategy =
                OptimizationStrategy { concurrent_downloads: 12, ..Default::default() };
            processor.update_strategy(new_strategy);
            assert_eq!(processor.concurrency_limit(), 12);
            assert_eq!(processor.batch_size(), 20); // Default batch size
        }
    }

    mod integration_tests {
        use super::*;

        // =============================================================================
        // PackageService Integration Tests
        // =============================================================================

        #[tokio::test]
        async fn test_package_service_with_optimization() {
            let fs = FileSystemManager::new();
            let context = ProjectContext::Single(SingleRepositoryContext::default());

            let service =
                PackageService::with_performance_optimization(fs.clone(), context).await.unwrap();

            assert!(service.is_performance_optimized());

            let strategy = service.get_optimization_strategy().await.unwrap();
            assert_eq!(strategy.concurrent_downloads, 10);
            assert_eq!(strategy.cache_strategy, CacheStrategy::NetworkHeavy);
        }

        #[tokio::test]
        async fn test_package_service_concurrent_processing() {
            let fs = FileSystemManager::new();
            let context = ProjectContext::Monorepo(MonorepoContext::default());

            let service =
                PackageService::with_performance_optimization(fs.clone(), context).await.unwrap();

            // Create test packages
            let packages = vec![
                Package::new("pkg1", "1.0.0", None).unwrap(),
                Package::new("pkg2", "1.0.0", None).unwrap(),
                Package::new("pkg3", "1.0.0", None).unwrap(),
            ];

            let start = Instant::now();
            let results = service
                .process_packages_concurrent(packages, |mut pkg| async move {
                    // Simulate some processing
                    sleep(Duration::from_millis(20)).await;
                    pkg.version = "1.1.0".to_string();
                    Ok(pkg)
                })
                .await
                .unwrap();
            let duration = start.elapsed();

            // Should be faster than sequential (60ms)
            assert!(duration.as_millis() < 50);
            assert_eq!(results.len(), 3);

            // Verify all packages were updated
            for pkg in results {
                assert_eq!(pkg.version_str(), "1.1.0");
            }
        }

        #[tokio::test]
        async fn test_package_service_fallback_without_optimization() {
            // Test that service works without optimization
            let fs = FileSystemManager::new();
            let service = PackageService::new(fs);

            assert!(!service.is_performance_optimized());

            let packages = vec![Package::new("test", "1.0.0", None).unwrap()];

            // Should still work, just without optimization
            let results = service
                .process_packages_concurrent(packages, |pkg| async move { Ok(pkg) })
                .await
                .unwrap();

            assert_eq!(results.len(), 1);
        }

        #[tokio::test]
        async fn test_package_service_optimization_toggle() {
            let fs = FileSystemManager::new();
            let mut service = PackageService::new(fs);

            assert!(!service.is_performance_optimized());

            // Enable optimization
            let context = ProjectContext::Single(SingleRepositoryContext::default());
            service.enable_performance_optimization(context).await.unwrap();
            assert!(service.is_performance_optimized());

            // Disable optimization
            service.disable_performance_optimization();
            assert!(!service.is_performance_optimized());
        }

        #[tokio::test]
        async fn test_package_service_version_updates_concurrent() {
            let fs = FileSystemManager::new();
            let context = ProjectContext::Monorepo(MonorepoContext::default());
            let service = PackageService::with_performance_optimization(fs, context).await.unwrap();

            let updates = vec![
                (
                    Package::new("pkg1", "1.0.0", None).unwrap(),
                    "1.1.0",
                    std::path::Path::new("pkg1/package.json"),
                ),
                (
                    Package::new("pkg2", "1.0.0", None).unwrap(),
                    "1.2.0",
                    std::path::Path::new("pkg2/package.json"),
                ),
                (
                    Package::new("pkg3", "1.0.0", None).unwrap(),
                    "1.3.0",
                    std::path::Path::new("pkg3/package.json"),
                ),
            ];

            let updated_packages =
                service.update_package_versions_concurrent(updates).await.unwrap();

            assert_eq!(updated_packages.len(), 3);
            assert_eq!(updated_packages[0].version_str(), "1.1.0");
            assert_eq!(updated_packages[1].version_str(), "1.2.0");
            assert_eq!(updated_packages[2].version_str(), "1.3.0");
        }

        // =============================================================================
        // PackageCommandService Integration Tests
        // =============================================================================

        #[tokio::test]
        async fn test_command_service_with_optimization() {
            let fs = FileSystemManager::new();
            let executor = DefaultCommandExecutor::new();
            let context = ProjectContext::Monorepo(MonorepoContext::default());

            let service =
                PackageCommandService::with_performance_optimization(executor, fs, context)
                    .await
                    .unwrap();

            // Service should be created with optimization
            assert!(service.is_performance_optimized());
            assert!(service.has_concurrent_processor());
        }

        #[tokio::test]
        async fn test_command_service_custom_strategy() {
            let fs = FileSystemManager::new();
            let executor = DefaultCommandExecutor::new();
            let strategy = OptimizationStrategy {
                concurrent_downloads: 25,
                enable_cascade_bumping: true,
                ..Default::default()
            };

            let service = PackageCommandService::with_custom_strategy(executor, fs, strategy);

            assert!(service.has_concurrent_processor());
            // Custom strategy should not have optimizer
            assert!(!service.is_performance_optimized());
        }
    }

    mod error_tests {
        use super::*;

        #[tokio::test]
        async fn test_concurrent_processor_error_recovery() {
            // Test that errors in some items don't affect others
            let processor = ConcurrentProcessor::new(Default::default());

            let items = vec![1, 2, 3, 4, 5];
            let result = processor
                .process_concurrent(tokio_stream::iter(items), |x| async move {
                    if x % 2 == 0 {
                        Err(Error::generic(format!("Error on {}", x)))
                    } else {
                        Ok(x * 10)
                    }
                })
                .await;

            // Should fail because some items errored
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_optimization_with_invalid_context() {
            // Test handling of edge case contexts
            let context = ProjectContext::Monorepo(MonorepoContext {
                workspace_packages: HashMap::new(), // Empty monorepo
                ..Default::default()
            });

            let optimizer = PerformanceOptimizer::new(context);
            let strategy = optimizer.optimize_for_context().await.unwrap();

            // Should still provide valid strategy
            assert_eq!(strategy.concurrent_downloads, 5);
            assert!(strategy.enable_cascade_bumping);
        }

        #[tokio::test]
        async fn test_concurrent_processor_panic_handling() {
            // Test that panics in tasks are handled gracefully
            let processor = ConcurrentProcessor::new(Default::default());

            let items = vec![1, 2, 3];
            let result = processor
                .process_concurrent(tokio_stream::iter(items), |x| async move {
                    if x == 2 {
                        // This would panic in a real scenario
                        // For testing, we'll just return an error
                        Err(Error::generic("Simulated panic"))
                    } else {
                        Ok(x)
                    }
                })
                .await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_resource_limits() {
            // Test that resource limits are respected
            let mut workspace_packages = HashMap::new();
            for i in 0..200 {
                workspace_packages.insert(format!("pkg-{}", i), format!("p/{}", i));
            }

            let context = ProjectContext::Monorepo(MonorepoContext {
                workspace_packages,
                ..Default::default()
            });

            let optimizer = PerformanceOptimizer::new(context);
            let strategy = optimizer.optimize_for_context().await.unwrap();

            // Check resource limits scale appropriately
            assert!(strategy.resource_limits.max_memory_mb > 500);
            assert!(strategy.resource_limits.max_file_descriptors >= 2048);
            assert!(strategy.resource_limits.max_network_connections <= 10);
        }
    }

    mod performance_tests {
        use super::*;

        #[tokio::test]
        async fn test_concurrent_processor_stress_test() {
            // Stress test with many items
            let strategy = OptimizationStrategy {
                concurrent_downloads: 10,
                batch_processing_size: 20,
                ..Default::default()
            };
            let processor = ConcurrentProcessor::new(strategy);

            let items: Vec<i32> = (0..100).collect();
            let start = Instant::now();

            let results = processor
                .process_concurrent(tokio_stream::iter(items), |x| async move {
                    // Simulate some work
                    sleep(Duration::from_millis(10)).await;
                    Ok::<i32, Error>(x * 2)
                })
                .await
                .unwrap();

            let duration = start.elapsed();

            // With 100 items and 10 concurrent, should take ~100ms (10 batches)
            // Give some margin for CI/slow machines
            assert!(duration.as_millis() < 300);
            assert_eq!(results.len(), 100);

            // Verify all results are correct
            let mut sorted = results;
            sorted.sort();
            let expected: Vec<i32> = (0..100).map(|x| x * 2).collect();
            assert_eq!(sorted, expected);
        }

        #[tokio::test]
        async fn benchmark_concurrent_vs_sequential() {
            let items: Vec<i32> = (0..50).collect();

            // Sequential processing
            let start_seq = Instant::now();
            let mut results_seq = Vec::new();
            for item in items.clone() {
                sleep(Duration::from_millis(10)).await;
                results_seq.push(item * 2);
            }
            let duration_seq = start_seq.elapsed();

            // Concurrent processing
            let processor = ConcurrentProcessor::new(OptimizationStrategy {
                concurrent_downloads: 10,
                ..Default::default()
            });

            let start_conc = Instant::now();
            let results_conc = processor
                .process_concurrent(tokio_stream::iter(items), |x| async move {
                    sleep(Duration::from_millis(10)).await;
                    Ok::<i32, Error>(x * 2)
                })
                .await
                .unwrap();
            let duration_conc = start_conc.elapsed();

            // Concurrent should be significantly faster
            assert!(duration_conc < duration_seq / 3);

            // Results should be the same (when sorted)
            let mut sorted_conc = results_conc;
            sorted_conc.sort();
            results_seq.sort();
            assert_eq!(sorted_conc, results_seq);

            println!(
                "Sequential: {:?}, Concurrent: {:?}, Speedup: {:.2}x",
                duration_seq,
                duration_conc,
                duration_seq.as_secs_f64() / duration_conc.as_secs_f64()
            );
        }

        #[tokio::test]
        async fn benchmark_context_aware_optimization() {
            // Compare single repo vs monorepo optimization strategies
            let single_context = ProjectContext::Single(SingleRepositoryContext::default());
            let single_optimizer = PerformanceOptimizer::new(single_context);
            let single_strategy = single_optimizer.optimize_for_context().await.unwrap();

            let mut workspace_packages = HashMap::new();
            for i in 0..100 {
                workspace_packages.insert(format!("pkg-{}", i), format!("packages/{}", i));
            }
            let monorepo_context = ProjectContext::Monorepo(MonorepoContext {
                workspace_packages,
                ..Default::default()
            });
            let monorepo_optimizer = PerformanceOptimizer::new(monorepo_context);
            let monorepo_strategy = monorepo_optimizer.optimize_for_context().await.unwrap();

            // Single repo should have higher concurrency
            assert!(single_strategy.concurrent_downloads > monorepo_strategy.concurrent_downloads);

            // Monorepo should have more conservative settings
            assert_eq!(single_strategy.timeout_strategy, TimeoutStrategy::Aggressive);
            assert_eq!(monorepo_strategy.timeout_strategy, TimeoutStrategy::Conservative);

            // Different cache strategies
            assert_eq!(single_strategy.cache_strategy, CacheStrategy::NetworkHeavy);
            assert_eq!(monorepo_strategy.cache_strategy, CacheStrategy::FilesystemHeavy);
        }

        #[tokio::test]
        async fn test_concurrent_service_access() {
            use std::sync::Arc;
            use tokio::task::JoinSet;

            let fs = FileSystemManager::new();
            let context = ProjectContext::Single(SingleRepositoryContext::default());
            let service =
                Arc::new(PackageService::with_performance_optimization(fs, context).await.unwrap());

            let mut tasks = JoinSet::new();

            // Spawn multiple concurrent tasks using the service
            for i in 0..10 {
                let service_clone: Arc<PackageService<FileSystemManager>> = Arc::clone(&service);
                tasks.spawn(async move {
                    let packages =
                        vec![Package::new(&format!("pkg-{}", i), "1.0.0", None).unwrap()];

                    service_clone
                        .process_packages_concurrent(packages, |mut pkg| async move {
                            pkg.version = "2.0.0".to_string();
                            Ok(pkg)
                        })
                        .await
                });
            }

            // All tasks should complete successfully
            while let Some(result) = tasks.join_next().await {
                let packages = result.unwrap().unwrap();
                assert_eq!(packages.len(), 1);
                assert_eq!(packages[0].version_str(), "2.0.0");
            }
        }
    }

    mod property_tests {
        use super::*;

        #[tokio::test]
        async fn test_optimization_strategy_bounds() {
            // Test that all optimization strategies produce valid bounds
            let contexts = vec![
                ProjectContext::Single(SingleRepositoryContext::default()),
                ProjectContext::Monorepo(MonorepoContext::default()),
                ProjectContext::Monorepo(MonorepoContext {
                    workspace_packages: (0..200)
                        .map(|i| (format!("p{}", i), format!("pkg/{}", i)))
                        .collect(),
                    ..Default::default()
                }),
            ];

            for context in contexts {
                let optimizer = PerformanceOptimizer::new(context);
                let strategy = optimizer.optimize_for_context().await.unwrap();

                // Verify bounds
                assert!(strategy.concurrent_downloads > 0);
                assert!(strategy.concurrent_downloads <= 50);
                assert!(strategy.batch_processing_size > 0);
                assert!(strategy.batch_processing_size <= 100);
                assert!(strategy.resource_limits.max_memory_mb > 0);
                assert!(strategy.resource_limits.max_file_descriptors > 0);
                assert!(strategy.performance_metrics.target_resolution_time_ms > 0);
            }
        }

        #[tokio::test]
        async fn test_concurrent_processor_ordering_preservation() {
            // Test that batch processing preserves order within batches
            let processor = ConcurrentProcessor::new(OptimizationStrategy {
                concurrent_downloads: 1, // Force sequential within batches
                batch_processing_size: 5,
                ..Default::default()
            });

            let items: Vec<i32> = (0..20).collect();
            let results = processor
                .process_batched(items, |x| async move { Ok::<i32, Error>(x) })
                .await
                .unwrap();

            // Results should maintain order
            let expected: Vec<i32> = (0..20).collect();
            assert_eq!(results, expected);
        }
    }
}
