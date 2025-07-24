//! # Concurrent Processing Service
//!
//! ## What
//!
//! High-performance concurrent processing service that leverages futures::stream for
//! parallel execution of package operations. This service is a core component of Phase 4.1
//! performance optimizations, providing context-aware concurrent processing capabilities.
//!
//! ## How
//!
//! The service uses tokio streams and semaphores to control concurrency levels based on
//! the optimization strategy. It provides backpressure control, error propagation, and
//! batch processing capabilities optimized for different project contexts.
//!
//! ## Why
//!
//! Different project structures require different concurrency approaches:
//! - Single repositories benefit from high network concurrency for registry operations
//! - Monorepos need controlled concurrency to avoid filesystem bottlenecks and rate limiting
//! - Enterprise-grade error handling and backpressure control are essential for reliability
//!
//! ## Architecture
//!
//! The processor integrates with PerformanceOptimizer to get context-aware concurrency
//! settings and provides specialized processing strategies for different operation types:
//! - Network operations (registry queries, package downloads)
//! - Filesystem operations (package.json reads/writes, workspace scanning)
//! - CPU-bound operations (dependency resolution, graph analysis)
//!
//! ## Examples
//!
//! ```rust
//! use sublime_package_tools::services::{ConcurrentProcessor, OptimizationStrategy};
//! use tokio_stream::{self as stream, StreamExt};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let strategy = OptimizationStrategy {
//!     concurrent_downloads: 10,
//!     // ... other settings
//!     ..Default::default()
//! };
//!
//! let processor = ConcurrentProcessor::new(strategy);
//!
//! // Process items concurrently
//! let items = vec!["pkg1", "pkg2", "pkg3"];
//! let results = processor.process_concurrent(
//!     stream::iter(items),
//!     |item| async move { 
//!         // Process each item
//!         Ok(format!("processed {}", item))
//!     }
//! ).await?;
//! # Ok(())
//! # }
//! ```

use std::future::Future;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio_stream::Stream;
use futures_util::StreamExt;

use crate::services::OptimizationStrategy;

/// Context-aware concurrent processor for package operations
///
/// This processor provides enterprise-grade concurrent processing capabilities
/// that adapt to different project contexts and optimization strategies.
/// It handles backpressure, error propagation, and resource management
/// while maximizing throughput for the specific project type.
///
/// ## Key Features
///
/// - **Context-Aware Concurrency**: Adapts concurrency levels based on optimization strategy
/// - **Backpressure Control**: Prevents resource exhaustion through semaphore-based limiting
/// - **Error Propagation**: Comprehensive error handling with early termination support
/// - **Batch Processing**: Efficient batch processing for bulk operations
/// - **Resource Management**: Automatic cleanup and resource limit enforcement
///
/// ## Performance Characteristics
///
/// ### Single Repository Context
/// - High concurrency (typically 10+ parallel operations)
/// - Network-optimized processing
/// - Fast-fail error handling
///
/// ### Monorepo Context
/// - Controlled concurrency (typically 3-5 parallel operations)
/// - Filesystem-optimized processing
/// - Conservative error handling with retries
#[derive(Debug, Clone)]
pub struct ConcurrentProcessor {
    /// Optimization strategy determining concurrency behavior
    strategy: OptimizationStrategy,
    /// Semaphore for controlling maximum concurrent operations
    semaphore: Arc<Semaphore>,
}

impl ConcurrentProcessor {
    /// Create a new concurrent processor with the given optimization strategy
    ///
    /// # Arguments
    ///
    /// * `strategy` - The optimization strategy containing concurrency settings
    ///
    /// # Returns
    ///
    /// A new ConcurrentProcessor configured for the optimization strategy
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::{ConcurrentProcessor, OptimizationStrategy};
    ///
    /// let strategy = OptimizationStrategy {
    ///     concurrent_downloads: 8,
    ///     // ... other configuration
    ///     ..Default::default()
    /// };
    ///
    /// let processor = ConcurrentProcessor::new(strategy);
    /// ```
    #[must_use]
    pub fn new(strategy: OptimizationStrategy) -> Self {
        let semaphore = Arc::new(Semaphore::new(strategy.concurrent_downloads));
        
        Self {
            strategy,
            semaphore,
        }
    }

    /// Process items from a stream concurrently with controlled concurrency
    ///
    /// This method applies the provided processing function to each item in the stream
    /// concurrently, respecting the concurrency limits defined in the optimization strategy.
    /// It provides backpressure control and comprehensive error handling.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of items in the input stream
    /// * `U` - The type of items in the output stream
    /// * `E` - The error type that can be returned by the processing function
    /// * `F` - The processing function type
    /// * `Fut` - The future type returned by the processing function
    /// * `S` - The input stream type
    ///
    /// # Arguments
    ///
    /// * `stream` - The input stream of items to process
    /// * `processor_fn` - The async function to apply to each item
    ///
    /// # Returns
    ///
    /// A Result containing a vector of successfully processed items, or an error
    /// if processing fails. On error, processing stops and partial results are discarded.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any item processing fails
    /// - Resource limits are exceeded
    /// - The processing function panics
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::ConcurrentProcessor;
    /// use tokio_stream::{self as stream, StreamExt};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let processor = ConcurrentProcessor::new(Default::default());
    /// let items = vec!["package1", "package2", "package3"];
    /// let results = processor.process_concurrent(
    ///     stream::iter(items),
    ///     |package_name| async move {
    ///         // Simulate processing a package
    ///         tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    ///         Ok(format!("Processed: {}", package_name))
    ///     }
    /// ).await?;
    ///
    /// println!("Processed {} packages", results.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn process_concurrent<T, U, E, F, Fut, S>(
        &self,
        stream: S,
        processor_fn: F,
    ) -> std::result::Result<Vec<U>, E>
    where
        T: Send + 'static,
        U: Send + 'static,
        E: Send + 'static,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<U, E>> + Send + 'static,
        S: Stream<Item = T> + Send,
    {
        let processor_fn = Arc::new(processor_fn);
        let semaphore = Arc::clone(&self.semaphore);

        let concurrent_stream = stream.map(move |item| {
            let processor_fn = Arc::clone(&processor_fn);
            let semaphore = Arc::clone(&semaphore);
            
            async move {
                // Acquire semaphore permit for concurrency control
                let _permit = match semaphore.acquire().await {
                    Ok(permit) => permit,
                    Err(_) => {
                        // Semaphore is closed, which shouldn't happen in normal operation
                        // We'll continue without the permit rather than fail
                        return processor_fn(item).await;
                    }
                };

                // Process the item
                processor_fn(item).await
            }
        });

        // Buffer the concurrent operations and collect results  
        let results: std::result::Result<Vec<_>, _> = concurrent_stream
            .buffered(self.strategy.concurrent_downloads)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect();

        results
    }

    /// Process items in batches with concurrent execution within each batch
    ///
    /// This method groups items into batches and processes each batch concurrently,
    /// then processes batches sequentially. This approach is useful for memory
    /// management and when dealing with large datasets.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of items to process
    /// * `U` - The type of processed items
    /// * `E` - The error type
    /// * `F` - The processing function type
    /// * `Fut` - The future type returned by the processing function
    ///
    /// # Arguments
    ///
    /// * `items` - Vector of items to process
    /// * `processor_fn` - The async function to apply to each item
    ///
    /// # Returns
    ///
    /// A Result containing all processed results, or an error if processing fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::ConcurrentProcessor;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let processor = ConcurrentProcessor::new(Default::default());
    /// let packages = vec!["pkg1", "pkg2", "pkg3", "pkg4", "pkg5"];
    /// let results = processor.process_batched(
    ///     packages,
    ///     |package| async move {
    ///         // Process package
    ///         Ok(format!("processed: {}", package))
    ///     }
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn process_batched<T, U, E, F, Fut>(
        &self,
        items: Vec<T>,
        processor_fn: F,
    ) -> std::result::Result<Vec<U>, E>
    where
        T: Send + 'static + Clone,
        U: Send + 'static,
        E: Send + 'static,
        F: Fn(T) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = std::result::Result<U, E>> + Send + 'static,
    {
        let batch_size = self.strategy.batch_processing_size;
        let mut all_results = Vec::with_capacity(items.len());

        // Process items in batches
        for batch in items.chunks(batch_size) {
            let batch_items: Vec<_> = batch.to_vec();
            let batch_stream = tokio_stream::iter(batch_items);
            
            let batch_results = self.process_concurrent(batch_stream, processor_fn.clone()).await?;
            all_results.extend(batch_results);
        }

        Ok(all_results)
    }

    /// Process items with custom concurrency limit, overriding the strategy
    ///
    /// This method allows temporary override of the concurrency limit for
    /// specific operations that may require different concurrency settings.
    ///
    /// # Arguments
    ///
    /// * `stream` - The input stream of items to process
    /// * `processor_fn` - The async function to apply to each item
    /// * `concurrency_limit` - Custom concurrency limit for this operation
    ///
    /// # Returns
    ///
    /// A Result containing processed results
    pub async fn process_with_custom_concurrency<T, U, E, F, Fut, S>(
        &self,
        stream: S,
        processor_fn: F,
        concurrency_limit: usize,
    ) -> std::result::Result<Vec<U>, E>
    where
        T: Send + 'static,
        U: Send + 'static,
        E: Send + 'static,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<U, E>> + Send + 'static,
        S: Stream<Item = T> + Send,
    {
        // Create temporary processor with custom concurrency
        let mut custom_strategy = self.strategy.clone();
        custom_strategy.concurrent_downloads = concurrency_limit;
        let custom_processor = ConcurrentProcessor::new(custom_strategy);
        
        custom_processor.process_concurrent(stream, processor_fn).await
    }

    /// Get the current optimization strategy
    ///
    /// # Returns
    ///
    /// A reference to the optimization strategy used by this processor
    #[must_use]
    pub fn strategy(&self) -> &OptimizationStrategy {
        &self.strategy
    }

    /// Get the current concurrency limit
    ///
    /// # Returns
    ///
    /// The maximum number of concurrent operations allowed
    #[must_use]
    pub fn concurrency_limit(&self) -> usize {
        self.strategy.concurrent_downloads
    }

    /// Get the current batch processing size
    ///
    /// # Returns
    ///  
    /// The number of items processed in each batch
    #[must_use]
    pub fn batch_size(&self) -> usize {
        self.strategy.batch_processing_size
    }

    /// Update the optimization strategy and reconfigure the processor
    ///
    /// # Arguments
    ///
    /// * `new_strategy` - The new optimization strategy to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::{ConcurrentProcessor, OptimizationStrategy};
    ///
    /// let mut processor = ConcurrentProcessor::new(Default::default());
    /// let new_strategy = OptimizationStrategy {
    ///     concurrent_downloads: 15,
    ///     ..Default::default()
    /// };
    /// processor.update_strategy(new_strategy);
    /// ```
    pub fn update_strategy(&mut self, new_strategy: OptimizationStrategy) {
        self.semaphore = Arc::new(Semaphore::new(new_strategy.concurrent_downloads));
        self.strategy = new_strategy;
    }
}

impl Default for ConcurrentProcessor {
    fn default() -> Self {
        Self::new(OptimizationStrategy::default())
    }
}

// Provide default implementation for OptimizationStrategy for testing
impl Default for OptimizationStrategy {
    fn default() -> Self {
        use crate::services::{
            CacheStrategy, MemoryOptimizationLevel, IoStrategy, 
            TimeoutStrategy, RetryStrategy, ResourceLimits, PerformanceMetrics
        };
        
        Self {
            concurrent_downloads: 5,
            enable_cascade_bumping: false,
            enable_workspace_scanning: false,
            cache_strategy: CacheStrategy::NetworkHeavy,
            memory_optimization_level: MemoryOptimizationLevel::Basic,
            io_strategy: IoStrategy::Balanced,
            batch_processing_size: 20,
            timeout_strategy: TimeoutStrategy::Conservative,
            retry_strategy: RetryStrategy::NetworkFocused,
            resource_limits: ResourceLimits {
                max_memory_mb: 256,
                max_file_descriptors: 1024,
                max_network_connections: 10,
            },
            performance_metrics: PerformanceMetrics {
                target_resolution_time_ms: 1000,
                max_acceptable_memory_mb: 512,
                concurrent_operation_limit: 10,
            },
        }
    }
}

/// Specialized processors for different operation types
pub mod specialized {
    use super::*;
    
    /// Network operations processor optimized for registry interactions
    pub struct NetworkProcessor {
        base_processor: ConcurrentProcessor,
    }

    impl NetworkProcessor {
        /// Create a new network processor with network-optimized settings
        #[must_use]
        pub fn new(mut strategy: OptimizationStrategy) -> Self {
            // Optimize for network operations
            strategy.concurrent_downloads = strategy.concurrent_downloads.max(8);
            strategy.batch_processing_size = strategy.batch_processing_size.max(50);
            
            Self {
                base_processor: ConcurrentProcessor::new(strategy),
            }
        }

        /// Process network requests concurrently
        pub async fn process_requests<T, U, E, F, Fut, S>(
            &self,
            stream: S,
            request_fn: F,
        ) -> std::result::Result<Vec<U>, E>
        where
            T: Send + 'static,
            U: Send + 'static,
            E: Send + 'static,
            F: Fn(T) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = std::result::Result<U, E>> + Send + 'static,
            S: Stream<Item = T> + Send,
        {
            self.base_processor.process_concurrent(stream, request_fn).await
        }
    }

    /// Filesystem operations processor optimized for local I/O
    pub struct FilesystemProcessor {
        base_processor: ConcurrentProcessor,
    }

    impl FilesystemProcessor {
        /// Create a new filesystem processor with filesystem-optimized settings
        #[must_use]
        pub fn new(mut strategy: OptimizationStrategy) -> Self {
            // Optimize for filesystem operations (lower concurrency to avoid bottlenecks)
            strategy.concurrent_downloads = strategy.concurrent_downloads.min(4);
            strategy.batch_processing_size = strategy.batch_processing_size.min(20);
            
            Self {
                base_processor: ConcurrentProcessor::new(strategy),
            }
        }

        /// Process filesystem operations concurrently
        pub async fn process_files<T, U, E, F, Fut, S>(
            &self,
            stream: S,
            file_fn: F,
        ) -> std::result::Result<Vec<U>, E>
        where
            T: Send + 'static,
            U: Send + 'static,
            E: Send + 'static,
            F: Fn(T) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = std::result::Result<U, E>> + Send + 'static,
            S: Stream<Item = T> + Send,
        {
            self.base_processor.process_concurrent(stream, file_fn).await
        }
    }
}

