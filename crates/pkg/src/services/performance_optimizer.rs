//! # Performance Optimizer Service
//!
//! ## What
//! 
//! Context-aware performance optimization service that adapts optimization strategies 
//! based on project structure (single repository vs monorepo). This service is the core
//! component of Phase 4.1 performance optimizations providing enterprise-grade performance
//! tuning for different project contexts.
//!
//! ## How
//!
//! The optimizer analyzes the project context and applies different optimization strategies:
//! - **Single Repository**: Network I/O optimized, workspace features disabled, high concurrency
//! - **Monorepo**: Filesystem I/O optimized, workspace scanning enabled, controlled concurrency
//!
//! ## Why
//!
//! Different project structures have fundamentally different performance characteristics:
//! - Single repositories benefit from aggressive network optimization and parallel downloads
//! - Monorepos benefit from filesystem caching and coordinated workspace operations
//!
//! ## Architecture
//!
//! The service uses a strategy pattern where each project context receives optimized
//! configuration values, cache strategies, and processing approaches tailored to
//! its specific performance characteristics.
//!
//! ## Examples
//!
//! ```rust
//! use sublime_package_tools::services::PerformanceOptimizer;
//! use sublime_package_tools::context::ProjectContext;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let context = ProjectContext::Single(Default::default());
//! let optimizer = PerformanceOptimizer::new(context);
//!
//! let strategy = optimizer.optimize_for_context().await?;
//! println!("Concurrent downloads: {}", strategy.concurrent_downloads);
//! println!("Cache strategy: {:?}", strategy.cache_strategy);
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};

use crate::context::{ProjectContext, SingleRepositoryContext, MonorepoContext};
use crate::errors::Result;

/// Context-aware performance optimizer providing optimized configurations
/// for different project structures
///
/// This optimizer is the central component for Phase 4.1 performance optimizations,
/// providing enterprise-grade performance tuning strategies that adapt based on
/// whether the project is a single repository or monorepo.
///
/// ## Key Features
///
/// - **Context Detection**: Automatically adapts to project structure
/// - **Strategy Optimization**: Different approaches for single repos vs monorepos  
/// - **Resource Management**: Optimal resource allocation per context
/// - **Cache Strategies**: Context-specific caching approaches
/// - **Concurrency Control**: Appropriate parallelism levels
///
/// ## Performance Characteristics
///
/// ### Single Repository Context
/// - High network concurrency (10 parallel downloads)
/// - Network-heavy caching strategy
/// - Workspace features disabled for efficiency
/// - Registry-focused optimizations
///
/// ### Monorepo Context  
/// - Controlled concurrency (5 parallel downloads to avoid rate limiting)
/// - Filesystem-heavy caching strategy  
/// - Workspace scanning and cascade operations enabled
/// - Internal dependency optimizations
#[derive(Debug, Clone)]
pub struct PerformanceOptimizer {
    /// Project context determining optimization strategy
    context: ProjectContext,
    /// Custom optimization overrides
    custom_overrides: OptimizationOverrides,
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer for the given project context
    ///
    /// # Arguments
    ///
    /// * `context` - The project context (single repository or monorepo)
    ///
    /// # Returns
    ///
    /// A new PerformanceOptimizer instance configured for the project context
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::PerformanceOptimizer;
    /// use sublime_package_tools::context::{ProjectContext, SingleRepositoryContext};
    ///
    /// let context = ProjectContext::Single(SingleRepositoryContext::default());
    /// let optimizer = PerformanceOptimizer::new(context);
    /// ```
    #[must_use]
    pub fn new(context: ProjectContext) -> Self {
        Self {
            context,
            custom_overrides: OptimizationOverrides::default(),
        }
    }

    /// Create a new performance optimizer with custom overrides
    ///
    /// # Arguments
    ///
    /// * `context` - The project context (single repository or monorepo)
    /// * `overrides` - Custom optimization parameter overrides
    ///
    /// # Returns
    ///
    /// A new PerformanceOptimizer instance with custom configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::{PerformanceOptimizer, OptimizationOverrides};
    /// use sublime_package_tools::context::{ProjectContext, SingleRepositoryContext};
    ///
    /// let context = ProjectContext::Single(SingleRepositoryContext::default());
    /// let mut overrides = OptimizationOverrides::default();
    /// overrides.concurrent_downloads = Some(15);
    /// 
    /// let optimizer = PerformanceOptimizer::with_overrides(context, overrides);
    /// ```
    #[must_use]
    pub fn with_overrides(context: ProjectContext, overrides: OptimizationOverrides) -> Self {
        Self {
            context,
            custom_overrides: overrides,
        }
    }

    /// Generate the optimal performance strategy for the current project context
    ///
    /// This method analyzes the project context and returns an optimization strategy
    /// tailored to the specific performance characteristics of single repositories
    /// or monorepos. The strategy includes concurrency settings, cache configuration,
    /// and feature enablement optimized for the detected context.
    ///
    /// # Returns
    ///
    /// An `OptimizationStrategy` with context-specific performance settings
    ///
    /// # Errors
    ///
    /// Returns an error if the optimization strategy cannot be computed due to
    /// invalid context configuration or system resource constraints.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::PerformanceOptimizer;
    /// use sublime_package_tools::context::{ProjectContext, MonorepoContext};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let context = ProjectContext::Monorepo(MonorepoContext::default());
    /// let optimizer = PerformanceOptimizer::new(context);
    ///
    /// let strategy = optimizer.optimize_for_context().await?;
    /// assert!(strategy.enable_cascade_bumping);
    /// assert!(strategy.enable_workspace_scanning);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn optimize_for_context(&self) -> Result<OptimizationStrategy> {
        let base_strategy = match &self.context {
            ProjectContext::Single(config) => self.optimize_single_repository(config).await?,
            ProjectContext::Monorepo(config) => self.optimize_monorepo(config).await?,
        };

        // Apply custom overrides if present
        Ok(self.apply_overrides(base_strategy))
    }

    /// Generate optimization strategy for single repository context
    ///
    /// Single repositories are optimized for network operations with high concurrency
    /// and simplified dependency management. Workspace features are disabled to
    /// reduce overhead and focus resources on registry interactions.
    async fn optimize_single_repository(&self, _config: &SingleRepositoryContext) -> Result<OptimizationStrategy> {
        Ok(OptimizationStrategy {
            concurrent_downloads: 10,
            enable_cascade_bumping: false,
            enable_workspace_scanning: false,
            cache_strategy: CacheStrategy::NetworkHeavy,
            memory_optimization_level: MemoryOptimizationLevel::Basic,
            io_strategy: IoStrategy::NetworkOptimized,
            batch_processing_size: 50,
            timeout_strategy: TimeoutStrategy::Aggressive,
            retry_strategy: RetryStrategy::NetworkFocused,
            resource_limits: ResourceLimits {
                max_memory_mb: 256,
                max_file_descriptors: 1024,
                max_network_connections: 50,
            },
            performance_metrics: PerformanceMetrics {
                target_resolution_time_ms: 200,
                max_acceptable_memory_mb: 500,
                concurrent_operation_limit: 20,
            },
        })
    }

    /// Generate optimization strategy for monorepo context
    ///
    /// Monorepos are optimized for filesystem operations with controlled concurrency
    /// to avoid rate limiting. Workspace features are enabled and cascade operations
    /// are supported for coordinated package management.
    async fn optimize_monorepo(&self, config: &MonorepoContext) -> Result<OptimizationStrategy> {
        // Scale resources based on workspace size
        let workspace_size = config.workspace_packages.len();
        let concurrent_downloads = if workspace_size > 50 {
            3 // Large monorepos: very conservative to avoid rate limiting
        } else if workspace_size > 20 {
            4 // Medium monorepos: moderate concurrency
        } else {
            5 // Small monorepos: standard concurrency
        };

        let batch_size = if workspace_size > 100 {
            20 // Large monorepos: smaller batches for memory efficiency
        } else {
            30 // Smaller monorepos: larger batches for performance
        };

        Ok(OptimizationStrategy {
            concurrent_downloads,
            enable_cascade_bumping: true,
            enable_workspace_scanning: true,
            cache_strategy: CacheStrategy::FilesystemHeavy,
            memory_optimization_level: MemoryOptimizationLevel::Aggressive,
            io_strategy: IoStrategy::FilesystemOptimized,
            batch_processing_size: batch_size,
            timeout_strategy: TimeoutStrategy::Conservative,
            retry_strategy: RetryStrategy::FilesystemFocused,
            resource_limits: ResourceLimits {
                max_memory_mb: 512 + (workspace_size * 2), // Scale with workspace size
                max_file_descriptors: 2048,
                max_network_connections: concurrent_downloads * 2,
            },
            performance_metrics: PerformanceMetrics {
                target_resolution_time_ms: if workspace_size > 50 { 2000 } else { 500 },
                max_acceptable_memory_mb: 1024 + (workspace_size * 5),
                concurrent_operation_limit: concurrent_downloads * 3,
            },
        })
    }

    /// Apply custom overrides to the base optimization strategy
    ///
    /// # Arguments
    ///
    /// * `base_strategy` - The base strategy generated for the project context
    ///
    /// # Returns
    ///
    /// The optimization strategy with custom overrides applied
    fn apply_overrides(&self, mut base_strategy: OptimizationStrategy) -> OptimizationStrategy {
        if let Some(concurrent_downloads) = self.custom_overrides.concurrent_downloads {
            base_strategy.concurrent_downloads = concurrent_downloads;
        }

        if let Some(enable_cascade_bumping) = self.custom_overrides.enable_cascade_bumping {
            base_strategy.enable_cascade_bumping = enable_cascade_bumping;
        }

        if let Some(cache_strategy) = &self.custom_overrides.cache_strategy {
            base_strategy.cache_strategy = cache_strategy.clone();
        }

        if let Some(memory_level) = &self.custom_overrides.memory_optimization_level {
            base_strategy.memory_optimization_level = memory_level.clone();
        }

        if let Some(batch_size) = self.custom_overrides.batch_processing_size {
            base_strategy.batch_processing_size = batch_size;
        }

        base_strategy
    }

    /// Get the current project context
    ///
    /// # Returns
    ///
    /// A reference to the project context used by this optimizer
    #[must_use]
    pub fn context(&self) -> &ProjectContext {
        &self.context
    }

    /// Update the project context and recalculate optimization strategy
    ///
    /// # Arguments
    ///
    /// * `new_context` - The new project context to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_package_tools::services::PerformanceOptimizer;
    /// use sublime_package_tools::context::{ProjectContext, MonorepoContext};
    ///
    /// let mut optimizer = PerformanceOptimizer::new(ProjectContext::Single(Default::default()));
    /// optimizer.update_context(ProjectContext::Monorepo(MonorepoContext::default()));
    /// ```
    pub fn update_context(&mut self, new_context: ProjectContext) {
        self.context = new_context;
    }
}

/// Comprehensive optimization strategy for package operations
///
/// This strategy contains all performance-related configuration values
/// that are optimized based on the project context. It provides specific
/// settings for concurrency, caching, I/O operations, and resource limits.
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizationStrategy {
    /// Number of concurrent download operations
    pub concurrent_downloads: usize,
    /// Whether cascade bumping operations are enabled
    pub enable_cascade_bumping: bool,
    /// Whether workspace scanning is enabled
    pub enable_workspace_scanning: bool,
    /// Cache strategy optimized for the context
    pub cache_strategy: CacheStrategy,
    /// Memory optimization level
    pub memory_optimization_level: MemoryOptimizationLevel,
    /// I/O operation strategy
    pub io_strategy: IoStrategy,
    /// Batch processing size for bulk operations
    pub batch_processing_size: usize,
    /// Timeout strategy for network and I/O operations  
    pub timeout_strategy: TimeoutStrategy,
    /// Retry strategy for failed operations
    pub retry_strategy: RetryStrategy,
    /// Resource usage limits
    pub resource_limits: ResourceLimits,
    /// Performance targets and metrics
    pub performance_metrics: PerformanceMetrics,
}

/// Cache strategy optimized for different project contexts
///
/// Different project types benefit from different caching approaches:
/// - Single repositories focus on network caching for registry operations
/// - Monorepos focus on filesystem caching for workspace operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CacheStrategy {
    /// Network-heavy caching for single repositories
    /// - Registry metadata cached aggressively
    /// - Package manifests cached with long TTL
    /// - Network responses prioritized over filesystem cache
    NetworkHeavy,
    /// Filesystem-heavy caching for monorepos  
    /// - Workspace package information cached extensively
    /// - Dependency graph cache prioritized
    /// - Local file system cache optimized
    FilesystemHeavy,
    /// Hybrid caching strategy balancing both approaches
    Hybrid,
}

/// Memory optimization levels for different performance requirements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryOptimizationLevel {
    /// No memory optimization - maximum performance
    None,
    /// Basic memory optimization - balanced approach
    Basic,
    /// Aggressive memory optimization - minimum memory usage
    Aggressive,
}

/// I/O operation strategy optimized for context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IoStrategy {
    /// Optimized for network I/O operations
    NetworkOptimized,
    /// Optimized for filesystem I/O operations
    FilesystemOptimized,
    /// Balanced approach for mixed workloads
    Balanced,
}

/// Timeout strategy for different operation types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimeoutStrategy {
    /// Aggressive timeouts for fast-fail behavior
    Aggressive,
    /// Conservative timeouts for reliable operations
    Conservative,
    /// Adaptive timeouts based on operation history
    Adaptive,
}

/// Retry strategy for failed operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RetryStrategy {
    /// Network-focused retry with exponential backoff
    NetworkFocused,
    /// Filesystem-focused retry with linear backoff  
    FilesystemFocused,
    /// Hybrid retry strategy
    Hybrid,
}

/// Resource usage limits for different contexts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in megabytes
    pub max_memory_mb: usize,
    /// Maximum file descriptors
    pub max_file_descriptors: usize,
    /// Maximum network connections
    pub max_network_connections: usize,
}

/// Performance targets and metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Target resolution time in milliseconds
    pub target_resolution_time_ms: u64,
    /// Maximum acceptable memory usage in megabytes
    pub max_acceptable_memory_mb: usize,
    /// Maximum concurrent operations
    pub concurrent_operation_limit: usize,
}

/// Custom optimization parameter overrides
///
/// Allows fine-tuning of the optimization strategy with custom values
/// that override the context-based defaults.
#[derive(Debug, Clone, Default)]
pub struct OptimizationOverrides {
    /// Override concurrent downloads setting
    pub concurrent_downloads: Option<usize>,
    /// Override cascade bumping enablement
    pub enable_cascade_bumping: Option<bool>,
    /// Override cache strategy
    pub cache_strategy: Option<CacheStrategy>,
    /// Override memory optimization level
    pub memory_optimization_level: Option<MemoryOptimizationLevel>,
    /// Override batch processing size
    pub batch_processing_size: Option<usize>,
}

impl OptimizationOverrides {
    /// Create a new empty set of optimization overrides
    ///
    /// # Returns
    ///
    /// A new OptimizationOverrides instance with no overrides set
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create overrides with concurrent downloads setting
    ///
    /// # Arguments
    ///
    /// * `concurrent_downloads` - Number of concurrent downloads to allow
    ///
    /// # Returns
    ///
    /// OptimizationOverrides with concurrent downloads override set
    #[must_use]
    pub fn with_concurrent_downloads(mut self, concurrent_downloads: usize) -> Self {
        self.concurrent_downloads = Some(concurrent_downloads);
        self
    }

    /// Create overrides with cache strategy setting
    ///
    /// # Arguments
    ///
    /// * `cache_strategy` - Cache strategy to use
    ///
    /// # Returns
    ///
    /// OptimizationOverrides with cache strategy override set
    #[must_use]
    pub fn with_cache_strategy(mut self, cache_strategy: CacheStrategy) -> Self {
        self.cache_strategy = Some(cache_strategy);
        self
    }
}

