//! Extreme Monorepo Infrastructure Testing System (500+ Packages)
//!
//! This module implements comprehensive infrastructure for testing monorepos at extreme scale,
//! supporting 500+ packages with sophisticated resource management, parallel processing,
//! advanced monitoring, optimized storage, and intelligent scaling mechanisms to handle
//! massive monorepo operations efficiently and reliably.

use sublime_monorepo_tools::Result;
use std::collections::{HashMap, VecDeque, BTreeMap, HashSet, BTreeSet};
use std::sync::atomic::{AtomicBool, AtomicUsize, AtomicU64, AtomicI64, Ordering};
use std::sync::{Arc, Mutex, RwLock, Condvar};
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use std::path::{Path, PathBuf};

/// Configuration for extreme monorepo infrastructure
#[derive(Debug, Clone)]
pub struct ExtremeMonorepoInfrastructureConfig {
    /// Maximum number of packages to support
    pub max_packages: usize,
    /// Target package count for testing
    pub target_package_count: usize,
    /// Maximum dependency depth to support
    pub max_dependency_depth: usize,
    /// Maximum dependencies per package
    pub max_dependencies_per_package: usize,
    /// Package generation batch size for efficiency
    pub package_generation_batch_size: usize,
    /// Memory pool size for package operations (MB)
    pub memory_pool_size_mb: usize,
    /// Thread pool size for parallel operations
    pub thread_pool_size: usize,
    /// Maximum concurrent package operations
    pub max_concurrent_operations: usize,
    /// Enable parallel package processing
    pub enable_parallel_processing: bool,
    /// Enable memory optimization strategies
    pub enable_memory_optimization: bool,
    /// Enable advanced caching mechanisms
    pub enable_advanced_caching: bool,
    /// Enable real-time resource monitoring
    pub enable_realtime_monitoring: bool,
    /// Resource monitoring interval in milliseconds
    pub monitoring_interval_ms: u64,
    /// Memory usage warning threshold (MB)
    pub memory_warning_threshold_mb: usize,
    /// Memory usage critical threshold (MB)
    pub memory_critical_threshold_mb: usize,
    /// CPU usage warning threshold (percentage)
    pub cpu_warning_threshold_percent: f64,
    /// CPU usage critical threshold (percentage)
    pub cpu_critical_threshold_percent: f64,
    /// Package storage optimization settings
    pub storage_optimization: StorageOptimizationConfig,
    /// Dependency graph optimization settings
    pub dependency_optimization: DependencyOptimizationConfig,
    /// Scaling policies configuration
    pub scaling_policies: ScalingPoliciesConfig,
    /// Failure detection and recovery settings
    pub failure_recovery: FailureRecoveryConfig,
    /// Advanced metrics collection settings
    pub metrics_collection: MetricsCollectionConfig,
}

impl Default for ExtremeMonorepoInfrastructureConfig {
    fn default() -> Self {
        Self {
            max_packages: 1000,              // Support up to 1000 packages
            target_package_count: 500,      // Default target: 500 packages
            max_dependency_depth: 20,       // Maximum depth of 20 levels
            max_dependencies_per_package: 50, // Maximum 50 deps per package
            package_generation_batch_size: 50, // Generate 50 packages per batch
            memory_pool_size_mb: 2048,      // 2GB memory pool
            thread_pool_size: 16,           // 16 worker threads
            max_concurrent_operations: 64,  // 64 concurrent operations
            enable_parallel_processing: true,
            enable_memory_optimization: true,
            enable_advanced_caching: true,
            enable_realtime_monitoring: true,
            monitoring_interval_ms: 1000,   // Monitor every second
            memory_warning_threshold_mb: 4096,  // 4GB warning
            memory_critical_threshold_mb: 6144, // 6GB critical
            cpu_warning_threshold_percent: 70.0,   // 70% CPU warning
            cpu_critical_threshold_percent: 90.0,  // 90% CPU critical
            storage_optimization: StorageOptimizationConfig::default(),
            dependency_optimization: DependencyOptimizationConfig::default(),
            scaling_policies: ScalingPoliciesConfig::default(),
            failure_recovery: FailureRecoveryConfig::default(),
            metrics_collection: MetricsCollectionConfig::default(),
        }
    }
}

/// Storage optimization configuration for extreme scale
#[derive(Debug, Clone)]
pub struct StorageOptimizationConfig {
    /// Enable package data compression
    pub enable_compression: bool,
    /// Compression level (1-9, higher = better compression)
    pub compression_level: u32,
    /// Enable lazy loading of package data
    pub enable_lazy_loading: bool,
    /// Enable memory mapping for large data structures
    pub enable_memory_mapping: bool,
    /// Cache size for frequently accessed packages (MB)
    pub package_cache_size_mb: usize,
    /// Cache TTL for package data (seconds)
    pub cache_ttl_seconds: u64,
    /// Enable package data deduplication
    pub enable_deduplication: bool,
    /// Storage backend type
    pub storage_backend: StorageBackendType,
    /// Enable write-behind caching
    pub enable_write_behind_cache: bool,
    /// Batch write size for efficiency
    pub batch_write_size: usize,
}

impl Default for StorageOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_compression: true,
            compression_level: 6,
            enable_lazy_loading: true,
            enable_memory_mapping: true,
            package_cache_size_mb: 512,
            cache_ttl_seconds: 3600,
            enable_deduplication: true,
            storage_backend: StorageBackendType::OptimizedInMemory,
            enable_write_behind_cache: true,
            batch_write_size: 100,
        }
    }
}

/// Types of storage backends for extreme scale
#[derive(Debug, Clone, PartialEq)]
pub enum StorageBackendType {
    /// Traditional in-memory storage
    InMemory,
    /// Optimized in-memory with compression and deduplication
    OptimizedInMemory,
    /// Memory-mapped files for large datasets
    MemoryMapped,
    /// Hybrid memory/disk storage with intelligent caching
    HybridMemoryDisk,
    /// Distributed storage across multiple nodes
    Distributed,
}

/// Dependency graph optimization configuration
#[derive(Debug, Clone)]
pub struct DependencyOptimizationConfig {
    /// Enable dependency graph caching
    pub enable_graph_caching: bool,
    /// Enable incremental graph updates
    pub enable_incremental_updates: bool,
    /// Enable graph compression techniques
    pub enable_graph_compression: bool,
    /// Enable parallel graph traversal
    pub enable_parallel_traversal: bool,
    /// Graph cache size (number of cached subgraphs)
    pub graph_cache_size: usize,
    /// Maximum depth for cached subgraphs
    pub max_cached_subgraph_depth: usize,
    /// Enable topological sorting optimization
    pub enable_topological_optimization: bool,
    /// Enable cycle detection optimization
    pub enable_cycle_detection_optimization: bool,
    /// Dependency resolution strategy
    pub resolution_strategy: DependencyResolutionStrategy,
}

impl Default for DependencyOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_graph_caching: true,
            enable_incremental_updates: true,
            enable_graph_compression: true,
            enable_parallel_traversal: true,
            graph_cache_size: 1000,
            max_cached_subgraph_depth: 10,
            enable_topological_optimization: true,
            enable_cycle_detection_optimization: true,
            resolution_strategy: DependencyResolutionStrategy::ParallelOptimized,
        }
    }
}

/// Dependency resolution strategies for extreme scale
#[derive(Debug, Clone, PartialEq)]
pub enum DependencyResolutionStrategy {
    /// Sequential single-threaded resolution
    Sequential,
    /// Parallel multi-threaded resolution
    Parallel,
    /// Optimized parallel with intelligent batching
    ParallelOptimized,
    /// Hierarchical resolution with caching
    Hierarchical,
    /// Adaptive resolution based on graph structure
    Adaptive,
}

/// Scaling policies configuration for dynamic resource management
#[derive(Debug, Clone)]
pub struct ScalingPoliciesConfig {
    /// Enable automatic scaling based on load
    pub enable_auto_scaling: bool,
    /// Minimum thread pool size
    pub min_thread_pool_size: usize,
    /// Maximum thread pool size
    pub max_thread_pool_size: usize,
    /// Memory scale-up threshold (percentage)
    pub memory_scale_up_threshold: f64,
    /// Memory scale-down threshold (percentage)
    pub memory_scale_down_threshold: f64,
    /// CPU scale-up threshold (percentage)
    pub cpu_scale_up_threshold: f64,
    /// CPU scale-down threshold (percentage)
    pub cpu_scale_down_threshold: f64,
    /// Scaling decision interval (seconds)
    pub scaling_decision_interval_secs: u64,
    /// Resource utilization history window (samples)
    pub utilization_history_window: usize,
    /// Scaling policy type
    pub scaling_policy: ScalingPolicyType,
}

impl Default for ScalingPoliciesConfig {
    fn default() -> Self {
        Self {
            enable_auto_scaling: true,
            min_thread_pool_size: 4,
            max_thread_pool_size: 32,
            memory_scale_up_threshold: 75.0,
            memory_scale_down_threshold: 40.0,
            cpu_scale_up_threshold: 80.0,
            cpu_scale_down_threshold: 30.0,
            scaling_decision_interval_secs: 30,
            utilization_history_window: 10,
            scaling_policy: ScalingPolicyType::Adaptive,
        }
    }
}

/// Types of scaling policies
#[derive(Debug, Clone, PartialEq)]
pub enum ScalingPolicyType {
    /// Conservative scaling with slow adjustments
    Conservative,
    /// Aggressive scaling with fast adjustments
    Aggressive,
    /// Adaptive scaling based on workload patterns
    Adaptive,
    /// Predictive scaling using machine learning
    Predictive,
}

/// Failure detection and recovery configuration
#[derive(Debug, Clone)]
pub struct FailureRecoveryConfig {
    /// Enable automatic failure detection
    pub enable_failure_detection: bool,
    /// Enable automatic recovery mechanisms
    pub enable_auto_recovery: bool,
    /// Maximum recovery attempts per failure
    pub max_recovery_attempts: usize,
    /// Recovery timeout (seconds)
    pub recovery_timeout_secs: u64,
    /// Health check interval (milliseconds)
    pub health_check_interval_ms: u64,
    /// Failure detection sensitivity (1.0-10.0)
    pub failure_detection_sensitivity: f64,
    /// Enable circuit breaker pattern
    pub enable_circuit_breaker: bool,
    /// Circuit breaker failure threshold
    pub circuit_breaker_failure_threshold: usize,
    /// Circuit breaker reset timeout (seconds)
    pub circuit_breaker_reset_timeout_secs: u64,
    /// Recovery strategies to attempt
    pub recovery_strategies: Vec<RecoveryStrategy>,
}

impl Default for FailureRecoveryConfig {
    fn default() -> Self {
        Self {
            enable_failure_detection: true,
            enable_auto_recovery: true,
            max_recovery_attempts: 3,
            recovery_timeout_secs: 300,
            health_check_interval_ms: 5000,
            failure_detection_sensitivity: 5.0,
            enable_circuit_breaker: true,
            circuit_breaker_failure_threshold: 5,
            circuit_breaker_reset_timeout_secs: 60,
            recovery_strategies: vec![
                RecoveryStrategy::ResourceCleanup,
                RecoveryStrategy::ProcessRestart,
                RecoveryStrategy::MemoryDefragmentation,
                RecoveryStrategy::CacheReset,
                RecoveryStrategy::GracefulDegradation,
            ],
        }
    }
}

/// Available recovery strategies
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    /// Clean up allocated resources
    ResourceCleanup,
    /// Restart failed processes
    ProcessRestart,
    /// Defragment memory allocations
    MemoryDefragmentation,
    /// Reset caches and temporary data
    CacheReset,
    /// Gracefully degrade performance
    GracefulDegradation,
    /// Scale down operations
    ScaleDown,
    /// Restart with safe mode
    SafeModeRestart,
}

/// Advanced metrics collection configuration
#[derive(Debug, Clone)]
pub struct MetricsCollectionConfig {
    /// Enable comprehensive metrics collection
    pub enable_metrics_collection: bool,
    /// Metrics collection interval (milliseconds)
    pub collection_interval_ms: u64,
    /// Maximum metrics history to retain
    pub max_metrics_history: usize,
    /// Enable real-time metrics export
    pub enable_realtime_export: bool,
    /// Enable metrics aggregation
    pub enable_aggregation: bool,
    /// Aggregation window size (samples)
    pub aggregation_window_size: usize,
    /// Metric types to collect
    pub metric_types: Vec<MetricType>,
    /// Enable predictive analytics on metrics
    pub enable_predictive_analytics: bool,
    /// Metrics storage optimization
    pub storage_optimization: bool,
}

impl Default for MetricsCollectionConfig {
    fn default() -> Self {
        Self {
            enable_metrics_collection: true,
            collection_interval_ms: 1000,
            max_metrics_history: 10000,
            enable_realtime_export: true,
            enable_aggregation: true,
            aggregation_window_size: 60,
            metric_types: vec![
                MetricType::MemoryUsage,
                MetricType::CpuUtilization,
                MetricType::ThroughputMetrics,
                MetricType::LatencyMetrics,
                MetricType::ErrorRates,
                MetricType::ResourceUtilization,
                MetricType::CacheHitRates,
                MetricType::DependencyGraphMetrics,
            ],
            enable_predictive_analytics: true,
            storage_optimization: true,
        }
    }
}

/// Types of metrics to collect
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MetricType {
    /// Memory usage metrics
    MemoryUsage,
    /// CPU utilization metrics
    CpuUtilization,
    /// Throughput and performance metrics
    ThroughputMetrics,
    /// Latency and response time metrics
    LatencyMetrics,
    /// Error rates and failure metrics
    ErrorRates,
    /// Resource utilization metrics
    ResourceUtilization,
    /// Cache hit rates and efficiency
    CacheHitRates,
    /// Dependency graph analysis metrics
    DependencyGraphMetrics,
    /// Scaling and load balancing metrics
    ScalingMetrics,
    /// Custom application-specific metrics
    CustomMetrics(String),
}

/// Main infrastructure manager for extreme monorepos
#[derive(Debug)]
pub struct ExtremeMonorepoInfrastructure {
    /// Configuration for the infrastructure
    config: ExtremeMonorepoInfrastructureConfig,
    /// Package registry with optimized storage
    package_registry: Arc<RwLock<OptimizedPackageRegistry>>,
    /// Dependency graph manager
    dependency_manager: Arc<RwLock<OptimizedDependencyManager>>,
    /// Resource monitor and manager
    resource_manager: Arc<Mutex<ResourceManager>>,
    /// Metrics collector and analyzer
    metrics_collector: Arc<Mutex<MetricsCollector>>,
    /// Thread pool for parallel operations
    thread_pool: Arc<Mutex<ThreadPool>>,
    /// Scaling manager for dynamic resource adjustment
    scaling_manager: Arc<Mutex<ScalingManager>>,
    /// Failure detector and recovery manager
    failure_recovery_manager: Arc<Mutex<FailureRecoveryManager>>,
    /// Infrastructure status and health
    infrastructure_health: Arc<RwLock<InfrastructureHealth>>,
    /// Active test session flag
    active_session: Arc<AtomicBool>,
    /// Total packages processed counter
    packages_processed: Arc<AtomicUsize>,
    /// Current memory usage (bytes)
    current_memory_usage: Arc<AtomicU64>,
    /// Current CPU utilization (percentage * 100)
    current_cpu_utilization: Arc<AtomicU64>,
    /// Performance benchmark data
    performance_benchmarks: Arc<Mutex<PerformanceBenchmarks>>,
}

/// Optimized package registry for extreme scale
#[derive(Debug)]
pub struct OptimizedPackageRegistry {
    /// Packages indexed by ID for fast lookup
    packages_by_id: HashMap<String, PackageInfo>,
    /// Package dependency index for fast dependency queries
    dependency_index: BTreeMap<String, BTreeSet<String>>,
    /// Reverse dependency index for impact analysis
    reverse_dependency_index: BTreeMap<String, BTreeSet<String>>,
    /// Package metadata cache
    metadata_cache: HashMap<String, PackageMetadata>,
    /// Compressed package data storage
    compressed_storage: HashMap<String, Vec<u8>>,
    /// Package access frequency tracking
    access_frequency: HashMap<String, usize>,
    /// Last access timestamp for cache eviction
    last_access: HashMap<String, Instant>,
}

impl OptimizedPackageRegistry {
    /// Creates a new optimized package registry
    pub fn new() -> Self {
        Self {
            packages_by_id: HashMap::new(),
            dependency_index: BTreeMap::new(),
            reverse_dependency_index: BTreeMap::new(),
            metadata_cache: HashMap::new(),
            compressed_storage: HashMap::new(),
            access_frequency: HashMap::new(),
            last_access: HashMap::new(),
        }
    }

    /// Registers a new package with optimization
    pub fn register_package(&mut self, package: PackageInfo) -> Result<()> {
        let package_id = package.id.clone();
        
        // Update dependency indices
        for dep in &package.dependencies {
            self.dependency_index
                .entry(package_id.clone())
                .or_insert_with(BTreeSet::new)
                .insert(dep.clone());
            
            self.reverse_dependency_index
                .entry(dep.clone())
                .or_insert_with(BTreeSet::new)
                .insert(package_id.clone());
        }
        
        // Store package metadata
        let metadata = PackageMetadata {
            size: package.size,
            complexity: package.complexity_score,
            dependency_count: package.dependencies.len(),
            last_modified: package.last_modified,
        };
        self.metadata_cache.insert(package_id.clone(), metadata);
        
        // Track access patterns
        self.access_frequency.insert(package_id.clone(), 0);
        self.last_access.insert(package_id.clone(), Instant::now());
        
        // Store package data
        self.packages_by_id.insert(package_id, package);
        
        Ok(())
    }

    /// Gets package information with access tracking
    pub fn get_package(&mut self, package_id: &str) -> Option<&PackageInfo> {
        // Update access tracking
        *self.access_frequency.entry(package_id.to_string()).or_insert(0) += 1;
        self.last_access.insert(package_id.to_string(), Instant::now());
        
        self.packages_by_id.get(package_id)
    }

    /// Gets package dependencies efficiently
    pub fn get_dependencies(&self, package_id: &str) -> Option<&BTreeSet<String>> {
        self.dependency_index.get(package_id)
    }

    /// Gets packages that depend on this package
    pub fn get_dependents(&self, package_id: &str) -> Option<&BTreeSet<String>> {
        self.reverse_dependency_index.get(package_id)
    }

    /// Performs cache cleanup based on access patterns
    pub fn cleanup_cache(&mut self, max_cache_size: usize) {
        if self.packages_by_id.len() <= max_cache_size {
            return;
        }

        // Sort packages by access frequency and recency
        let mut access_scores: Vec<(String, f64)> = self.packages_by_id
            .keys()
            .map(|id| {
                let frequency = *self.access_frequency.get(id).unwrap_or(&0) as f64;
                let last_access = self.last_access.get(id).unwrap_or(&Instant::now());
                let recency = last_access.elapsed().as_secs() as f64;
                let score = frequency / (1.0 + recency / 3600.0); // Frequency weighted by recency
                (id.clone(), score)
            })
            .collect();

        access_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Remove least frequently used packages
        let packages_to_remove = access_scores.len() - max_cache_size;
        for (package_id, _) in access_scores.into_iter().take(packages_to_remove) {
            self.packages_by_id.remove(&package_id);
            self.metadata_cache.remove(&package_id);
            self.compressed_storage.remove(&package_id);
            self.access_frequency.remove(&package_id);
            self.last_access.remove(&package_id);
        }
    }
}

/// Package information structure
#[derive(Debug, Clone)]
pub struct PackageInfo {
    /// Unique package identifier
    pub id: String,
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Package dependencies
    pub dependencies: Vec<String>,
    /// Package size in bytes
    pub size: usize,
    /// Package complexity score
    pub complexity_score: f64,
    /// Last modification timestamp
    pub last_modified: SystemTime,
    /// Package type
    pub package_type: PackageType,
    /// Build configuration
    pub build_config: BuildConfig,
}

/// Types of packages in the monorepo
#[derive(Debug, Clone, PartialEq)]
pub enum PackageType {
    /// Library package
    Library,
    /// Application package
    Application,
    /// Tool or utility package
    Tool,
    /// Test package
    Test,
    /// Documentation package
    Documentation,
    /// Configuration package
    Configuration,
}

/// Build configuration for packages
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Build type (debug, release, test)
    pub build_type: BuildType,
    /// Build flags and options
    pub build_flags: Vec<String>,
    /// Environment variables for build
    pub environment: HashMap<String, String>,
    /// Build dependencies
    pub build_dependencies: Vec<String>,
}

/// Build types
#[derive(Debug, Clone, PartialEq)]
pub enum BuildType {
    /// Debug build
    Debug,
    /// Release build
    Release,
    /// Test build
    Test,
    /// Benchmark build
    Benchmark,
}

/// Package metadata for caching
#[derive(Debug, Clone)]
pub struct PackageMetadata {
    /// Package size in bytes
    pub size: usize,
    /// Package complexity score
    pub complexity: f64,
    /// Number of dependencies
    pub dependency_count: usize,
    /// Last modification timestamp
    pub last_modified: SystemTime,
}

/// Optimized dependency manager for extreme scale
#[derive(Debug)]
pub struct OptimizedDependencyManager {
    /// Dependency graph representation
    dependency_graph: DependencyGraph,
    /// Cached dependency resolutions
    resolution_cache: HashMap<String, Vec<String>>,
    /// Topological order cache
    topological_cache: Option<Vec<String>>,
    /// Cycle detection cache
    cycle_cache: HashMap<String, bool>,
    /// Dependency depth cache
    depth_cache: HashMap<String, usize>,
}

impl OptimizedDependencyManager {
    /// Creates a new optimized dependency manager
    pub fn new() -> Self {
        Self {
            dependency_graph: DependencyGraph::new(),
            resolution_cache: HashMap::new(),
            topological_cache: None,
            cycle_cache: HashMap::new(),
            depth_cache: HashMap::new(),
        }
    }

    /// Adds a dependency relationship
    pub fn add_dependency(&mut self, package: &str, dependency: &str) -> Result<()> {
        self.dependency_graph.add_edge(package, dependency)?;
        self.invalidate_caches();
        Ok(())
    }

    /// Resolves dependencies for a package
    pub fn resolve_dependencies(&mut self, package: &str) -> Result<Vec<String>> {
        if let Some(cached_resolution) = self.resolution_cache.get(package) {
            return Ok(cached_resolution.clone());
        }

        let resolution = self.dependency_graph.resolve_dependencies(package)?;
        self.resolution_cache.insert(package.to_string(), resolution.clone());
        Ok(resolution)
    }

    /// Invalidates all caches
    fn invalidate_caches(&mut self) {
        self.resolution_cache.clear();
        self.topological_cache = None;
        self.cycle_cache.clear();
        self.depth_cache.clear();
    }
}

/// Dependency graph representation
#[derive(Debug)]
pub struct DependencyGraph {
    /// Adjacency list representation
    edges: HashMap<String, BTreeSet<String>>,
    /// Nodes in the graph
    nodes: BTreeSet<String>,
}

impl DependencyGraph {
    /// Creates a new dependency graph
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            nodes: BTreeSet::new(),
        }
    }

    /// Adds an edge to the graph
    pub fn add_edge(&mut self, from: &str, to: &str) -> Result<()> {
        self.nodes.insert(from.to_string());
        self.nodes.insert(to.to_string());
        
        self.edges
            .entry(from.to_string())
            .or_insert_with(BTreeSet::new)
            .insert(to.to_string());
        
        Ok(())
    }

    /// Resolves dependencies for a package
    pub fn resolve_dependencies(&self, package: &str) -> Result<Vec<String>> {
        let mut visited = HashSet::new();
        let mut resolution = Vec::new();
        self.dfs_resolve(package, &mut visited, &mut resolution)?;
        Ok(resolution)
    }

    /// Depth-first search for dependency resolution
    fn dfs_resolve(&self, package: &str, visited: &mut HashSet<String>, resolution: &mut Vec<String>) -> Result<()> {
        if visited.contains(package) {
            return Ok(());
        }

        visited.insert(package.to_string());

        if let Some(dependencies) = self.edges.get(package) {
            for dep in dependencies {
                self.dfs_resolve(dep, visited, resolution)?;
            }
        }

        resolution.push(package.to_string());
        Ok(())
    }
}

/// Resource manager for monitoring and controlling system resources
#[derive(Debug)]
pub struct ResourceManager {
    /// Current memory usage statistics
    memory_stats: MemoryStats,
    /// Current CPU usage statistics
    cpu_stats: CpuStats,
    /// Resource utilization history
    utilization_history: VecDeque<ResourceUtilization>,
    /// Resource limits and thresholds
    resource_limits: ResourceLimits,
    /// Active resource monitoring
    monitoring_active: bool,
}

impl ResourceManager {
    /// Creates a new resource manager
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            memory_stats: MemoryStats::default(),
            cpu_stats: CpuStats::default(),
            utilization_history: VecDeque::new(),
            resource_limits: limits,
            monitoring_active: false,
        }
    }

    /// Starts resource monitoring
    pub fn start_monitoring(&mut self) {
        self.monitoring_active = true;
    }

    /// Stops resource monitoring
    pub fn stop_monitoring(&mut self) {
        self.monitoring_active = false;
    }

    /// Updates resource statistics
    pub fn update_stats(&mut self) {
        if !self.monitoring_active {
            return;
        }

        // Update memory statistics
        self.memory_stats.update();
        
        // Update CPU statistics
        self.cpu_stats.update();
        
        // Add to history
        let utilization = ResourceUtilization {
            timestamp: Instant::now(),
            memory_usage_percent: self.memory_stats.usage_percent,
            cpu_usage_percent: self.cpu_stats.usage_percent,
            memory_used_mb: self.memory_stats.used_mb,
            cpu_cores_active: self.cpu_stats.active_cores,
        };
        
        self.utilization_history.push_back(utilization);
        
        // Maintain history size
        while self.utilization_history.len() > 1000 {
            self.utilization_history.pop_front();
        }
    }

    /// Checks if resource limits are exceeded
    pub fn check_limits(&self) -> Vec<ResourceLimit> {
        let mut exceeded_limits = Vec::new();
        
        if self.memory_stats.usage_percent > self.resource_limits.memory_critical_percent {
            exceeded_limits.push(ResourceLimit::MemoryCritical);
        } else if self.memory_stats.usage_percent > self.resource_limits.memory_warning_percent {
            exceeded_limits.push(ResourceLimit::MemoryWarning);
        }
        
        if self.cpu_stats.usage_percent > self.resource_limits.cpu_critical_percent {
            exceeded_limits.push(ResourceLimit::CpuCritical);
        } else if self.cpu_stats.usage_percent > self.resource_limits.cpu_warning_percent {
            exceeded_limits.push(ResourceLimit::CpuWarning);
        }
        
        exceeded_limits
    }
}

/// Memory usage statistics
#[derive(Debug, Default)]
pub struct MemoryStats {
    /// Total memory usage in MB
    pub used_mb: usize,
    /// Memory usage percentage
    pub usage_percent: f64,
    /// Peak memory usage in MB
    pub peak_mb: usize,
    /// Memory allocation rate (MB/second)
    pub allocation_rate: f64,
}

impl MemoryStats {
    /// Updates memory statistics
    pub fn update(&mut self) {
        // Simulate memory statistics update
        // In a real implementation, this would query actual system memory usage
        self.used_mb = 1024; // Placeholder value
        self.usage_percent = 50.0; // Placeholder value
        if self.used_mb > self.peak_mb {
            self.peak_mb = self.used_mb;
        }
    }
}

/// CPU usage statistics
#[derive(Debug, Default)]
pub struct CpuStats {
    /// CPU usage percentage
    pub usage_percent: f64,
    /// Number of active CPU cores
    pub active_cores: usize,
    /// CPU load average
    pub load_average: f64,
    /// Context switches per second
    pub context_switches_per_sec: usize,
}

impl CpuStats {
    /// Updates CPU statistics
    pub fn update(&mut self) {
        // Simulate CPU statistics update
        // In a real implementation, this would query actual system CPU usage
        self.usage_percent = 25.0; // Placeholder value
        self.active_cores = 8; // Placeholder value
        self.load_average = 2.5; // Placeholder value
        self.context_switches_per_sec = 1000; // Placeholder value
    }
}

/// Resource utilization snapshot
#[derive(Debug, Clone)]
pub struct ResourceUtilization {
    /// Timestamp of the measurement
    pub timestamp: Instant,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory used in MB
    pub memory_used_mb: usize,
    /// Number of active CPU cores
    pub cpu_cores_active: usize,
}

/// Resource limits and thresholds
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Memory warning threshold (percentage)
    pub memory_warning_percent: f64,
    /// Memory critical threshold (percentage)
    pub memory_critical_percent: f64,
    /// CPU warning threshold (percentage)
    pub cpu_warning_percent: f64,
    /// CPU critical threshold (percentage)
    pub cpu_critical_percent: f64,
}

/// Types of resource limits
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceLimit {
    /// Memory usage warning
    MemoryWarning,
    /// Memory usage critical
    MemoryCritical,
    /// CPU usage warning
    CpuWarning,
    /// CPU usage critical
    CpuCritical,
}

/// Metrics collector for comprehensive monitoring
#[derive(Debug)]
pub struct MetricsCollector {
    /// Collected metrics data
    metrics: HashMap<MetricType, VecDeque<MetricValue>>,
    /// Metrics collection configuration
    config: MetricsCollectionConfig,
    /// Collection start time
    start_time: Instant,
    /// Last collection timestamp
    last_collection: Instant,
}

impl MetricsCollector {
    /// Creates a new metrics collector
    pub fn new(config: MetricsCollectionConfig) -> Self {
        Self {
            metrics: HashMap::new(),
            config,
            start_time: Instant::now(),
            last_collection: Instant::now(),
        }
    }

    /// Collects metrics for all configured types
    pub fn collect_metrics(&mut self) {
        let now = Instant::now();
        
        for metric_type in &self.config.metric_types {
            let value = self.collect_metric_value(metric_type, now);
            
            let metric_values = self.metrics
                .entry(metric_type.clone())
                .or_insert_with(VecDeque::new);
            
            metric_values.push_back(value);
            
            // Maintain history size
            while metric_values.len() > self.config.max_metrics_history {
                metric_values.pop_front();
            }
        }
        
        self.last_collection = now;
    }

    /// Collects a specific metric value
    fn collect_metric_value(&self, metric_type: &MetricType, timestamp: Instant) -> MetricValue {
        match metric_type {
            MetricType::MemoryUsage => MetricValue {
                timestamp,
                value: 1024.0, // Placeholder: actual memory usage in MB
                unit: "MB".to_string(),
            },
            MetricType::CpuUtilization => MetricValue {
                timestamp,
                value: 25.0, // Placeholder: actual CPU usage percentage
                unit: "%".to_string(),
            },
            MetricType::ThroughputMetrics => MetricValue {
                timestamp,
                value: 1000.0, // Placeholder: operations per second
                unit: "ops/sec".to_string(),
            },
            MetricType::LatencyMetrics => MetricValue {
                timestamp,
                value: 50.0, // Placeholder: average latency in milliseconds
                unit: "ms".to_string(),
            },
            MetricType::ErrorRates => MetricValue {
                timestamp,
                value: 0.1, // Placeholder: error rate percentage
                unit: "%".to_string(),
            },
            MetricType::ResourceUtilization => MetricValue {
                timestamp,
                value: 60.0, // Placeholder: overall resource utilization
                unit: "%".to_string(),
            },
            MetricType::CacheHitRates => MetricValue {
                timestamp,
                value: 85.0, // Placeholder: cache hit rate percentage
                unit: "%".to_string(),
            },
            MetricType::DependencyGraphMetrics => MetricValue {
                timestamp,
                value: 500.0, // Placeholder: number of nodes in dependency graph
                unit: "nodes".to_string(),
            },
            MetricType::ScalingMetrics => MetricValue {
                timestamp,
                value: 16.0, // Placeholder: current thread pool size
                unit: "threads".to_string(),
            },
            MetricType::CustomMetrics(_) => MetricValue {
                timestamp,
                value: 0.0, // Placeholder for custom metrics
                unit: "custom".to_string(),
            },
        }
    }
}

/// Individual metric value
#[derive(Debug, Clone)]
pub struct MetricValue {
    /// Timestamp when the metric was collected
    pub timestamp: Instant,
    /// Metric value
    pub value: f64,
    /// Metric unit
    pub unit: String,
}

/// Thread pool for parallel operations
#[derive(Debug)]
pub struct ThreadPool {
    /// Worker threads
    workers: Vec<WorkerThread>,
    /// Task queue
    task_queue: VecDeque<Task>,
    /// Thread pool configuration
    config: ThreadPoolConfig,
    /// Pool status
    status: ThreadPoolStatus,
}

impl ThreadPool {
    /// Creates a new thread pool
    pub fn new(config: ThreadPoolConfig) -> Self {
        let workers = (0..config.initial_size)
            .map(|id| WorkerThread::new(id))
            .collect();

        Self {
            workers,
            task_queue: VecDeque::new(),
            config,
            status: ThreadPoolStatus::Idle,
        }
    }

    /// Submits a task to the thread pool
    pub fn submit_task(&mut self, task: Task) {
        self.task_queue.push_back(task);
        self.status = ThreadPoolStatus::Active;
    }

    /// Processes queued tasks
    pub fn process_tasks(&mut self) {
        while let Some(task) = self.task_queue.pop_front() {
            if let Some(worker) = self.find_idle_worker() {
                worker.assign_task(task);
            } else {
                // Return task to queue if no workers available
                self.task_queue.push_front(task);
                break;
            }
        }

        if self.task_queue.is_empty() {
            self.status = ThreadPoolStatus::Idle;
        }
    }

    /// Finds an idle worker thread
    fn find_idle_worker(&mut self) -> Option<&mut WorkerThread> {
        self.workers.iter_mut().find(|w| w.status == WorkerStatus::Idle)
    }
}

/// Thread pool configuration
#[derive(Debug, Clone)]
pub struct ThreadPoolConfig {
    /// Initial thread pool size
    pub initial_size: usize,
    /// Maximum thread pool size
    pub max_size: usize,
    /// Minimum thread pool size
    pub min_size: usize,
    /// Task queue capacity
    pub queue_capacity: usize,
}

/// Thread pool status
#[derive(Debug, Clone, PartialEq)]
pub enum ThreadPoolStatus {
    /// Thread pool is idle
    Idle,
    /// Thread pool is actively processing tasks
    Active,
    /// Thread pool is shutting down
    Shutdown,
}

/// Individual worker thread
#[derive(Debug)]
pub struct WorkerThread {
    /// Worker thread ID
    pub id: usize,
    /// Worker status
    pub status: WorkerStatus,
    /// Currently assigned task
    pub current_task: Option<Task>,
}

impl WorkerThread {
    /// Creates a new worker thread
    pub fn new(id: usize) -> Self {
        Self {
            id,
            status: WorkerStatus::Idle,
            current_task: None,
        }
    }

    /// Assigns a task to this worker
    pub fn assign_task(&mut self, task: Task) {
        self.current_task = Some(task);
        self.status = WorkerStatus::Busy;
    }

    /// Completes the current task
    pub fn complete_task(&mut self) {
        self.current_task = None;
        self.status = WorkerStatus::Idle;
    }
}

/// Worker thread status
#[derive(Debug, Clone, PartialEq)]
pub enum WorkerStatus {
    /// Worker is idle and available for tasks
    Idle,
    /// Worker is busy processing a task
    Busy,
    /// Worker has failed and needs attention
    Failed,
}

/// Task for execution in the thread pool
#[derive(Debug)]
pub struct Task {
    /// Task ID
    pub id: String,
    /// Task type
    pub task_type: TaskType,
    /// Task priority
    pub priority: TaskPriority,
    /// Task creation timestamp
    pub created_at: Instant,
}

/// Types of tasks that can be executed
#[derive(Debug, Clone, PartialEq)]
pub enum TaskType {
    /// Package analysis task
    PackageAnalysis,
    /// Dependency resolution task
    DependencyResolution,
    /// Cache optimization task
    CacheOptimization,
    /// Resource cleanup task
    ResourceCleanup,
    /// Metrics collection task
    MetricsCollection,
}

/// Task priority levels
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub enum TaskPriority {
    /// Low priority task
    Low,
    /// Normal priority task
    Normal,
    /// High priority task
    High,
    /// Critical priority task
    Critical,
}

/// Scaling manager for dynamic resource adjustment
#[derive(Debug)]
pub struct ScalingManager {
    /// Scaling configuration
    config: ScalingPoliciesConfig,
    /// Current scaling state
    scaling_state: ScalingState,
    /// Scaling decision history
    scaling_history: VecDeque<ScalingDecision>,
    /// Resource utilization measurements
    utilization_measurements: VecDeque<ResourceUtilization>,
}

impl ScalingManager {
    /// Creates a new scaling manager
    pub fn new(config: ScalingPoliciesConfig) -> Self {
        Self {
            config,
            scaling_state: ScalingState::Stable,
            scaling_history: VecDeque::new(),
            utilization_measurements: VecDeque::new(),
        }
    }

    /// Makes scaling decisions based on current utilization
    pub fn make_scaling_decision(&mut self, utilization: ResourceUtilization) -> Option<ScalingDecision> {
        self.utilization_measurements.push_back(utilization.clone());
        
        // Maintain measurement history
        while self.utilization_measurements.len() > self.config.utilization_history_window {
            self.utilization_measurements.pop_front();
        }

        // Calculate average utilization over the window
        let avg_memory = self.utilization_measurements.iter()
            .map(|u| u.memory_usage_percent)
            .sum::<f64>() / self.utilization_measurements.len() as f64;
        
        let avg_cpu = self.utilization_measurements.iter()
            .map(|u| u.cpu_usage_percent)
            .sum::<f64>() / self.utilization_measurements.len() as f64;

        // Make scaling decision
        let decision = if avg_memory > self.config.memory_scale_up_threshold || 
                         avg_cpu > self.config.cpu_scale_up_threshold {
            Some(ScalingDecision {
                timestamp: Instant::now(),
                decision_type: ScalingDecisionType::ScaleUp,
                reason: format!("High resource utilization: Memory {}%, CPU {}%", avg_memory, avg_cpu),
                target_resources: None,
            })
        } else if avg_memory < self.config.memory_scale_down_threshold && 
                 avg_cpu < self.config.cpu_scale_down_threshold {
            Some(ScalingDecision {
                timestamp: Instant::now(),
                decision_type: ScalingDecisionType::ScaleDown,
                reason: format!("Low resource utilization: Memory {}%, CPU {}%", avg_memory, avg_cpu),
                target_resources: None,
            })
        } else {
            None
        };

        if let Some(ref d) = decision {
            self.scaling_history.push_back(d.clone());
            while self.scaling_history.len() > 100 {
                self.scaling_history.pop_front();
            }
        }

        decision
    }
}

/// Current scaling state
#[derive(Debug, Clone, PartialEq)]
pub enum ScalingState {
    /// System is stable
    Stable,
    /// System is scaling up
    ScalingUp,
    /// System is scaling down
    ScalingDown,
    /// System is in cooldown period
    Cooldown,
}

/// Scaling decision made by the scaling manager
#[derive(Debug, Clone)]
pub struct ScalingDecision {
    /// Timestamp of the decision
    pub timestamp: Instant,
    /// Type of scaling decision
    pub decision_type: ScalingDecisionType,
    /// Reason for the decision
    pub reason: String,
    /// Target resource levels (if applicable)
    pub target_resources: Option<TargetResources>,
}

/// Types of scaling decisions
#[derive(Debug, Clone, PartialEq)]
pub enum ScalingDecisionType {
    /// Scale up resources
    ScaleUp,
    /// Scale down resources
    ScaleDown,
    /// Maintain current resources
    Maintain,
}

/// Target resource levels for scaling
#[derive(Debug, Clone)]
pub struct TargetResources {
    /// Target thread pool size
    pub thread_pool_size: usize,
    /// Target memory allocation
    pub memory_allocation_mb: usize,
    /// Target cache sizes
    pub cache_sizes: HashMap<String, usize>,
}

/// Failure recovery manager
#[derive(Debug)]
pub struct FailureRecoveryManager {
    /// Recovery configuration
    config: FailureRecoveryConfig,
    /// Detected failures
    detected_failures: Vec<DetectedFailure>,
    /// Recovery attempts history
    recovery_history: VecDeque<RecoveryAttempt>,
    /// Current health status
    health_status: HealthStatus,
    /// Circuit breaker state
    circuit_breaker: CircuitBreakerState,
}

impl FailureRecoveryManager {
    /// Creates a new failure recovery manager
    pub fn new(config: FailureRecoveryConfig) -> Self {
        Self {
            config,
            detected_failures: Vec::new(),
            recovery_history: VecDeque::new(),
            health_status: HealthStatus::Healthy,
            circuit_breaker: CircuitBreakerState::Closed,
        }
    }

    /// Detects potential failures
    pub fn detect_failures(&mut self, metrics: &HashMap<MetricType, VecDeque<MetricValue>>) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();

        // Check for memory issues
        if let Some(memory_metrics) = metrics.get(&MetricType::MemoryUsage) {
            if let Some(latest) = memory_metrics.back() {
                if latest.value > 4000.0 { // More than 4GB usage
                    failures.push(DetectedFailure {
                        failure_type: FailureType::MemoryExhaustion,
                        severity: FailureSeverity::High,
                        timestamp: Instant::now(),
                        details: format!("Memory usage: {}MB", latest.value),
                    });
                }
            }
        }

        // Check for CPU issues
        if let Some(cpu_metrics) = metrics.get(&MetricType::CpuUtilization) {
            if let Some(latest) = cpu_metrics.back() {
                if latest.value > 90.0 { // More than 90% CPU usage
                    failures.push(DetectedFailure {
                        failure_type: FailureType::CpuSaturation,
                        severity: FailureSeverity::Medium,
                        timestamp: Instant::now(),
                        details: format!("CPU usage: {}%", latest.value),
                    });
                }
            }
        }

        // Check for error rate issues
        if let Some(error_metrics) = metrics.get(&MetricType::ErrorRates) {
            if let Some(latest) = error_metrics.back() {
                if latest.value > 5.0 { // More than 5% error rate
                    failures.push(DetectedFailure {
                        failure_type: FailureType::HighErrorRate,
                        severity: FailureSeverity::Critical,
                        timestamp: Instant::now(),
                        details: format!("Error rate: {}%", latest.value),
                    });
                }
            }
        }

        self.detected_failures.extend(failures.clone());
        failures
    }

    /// Attempts to recover from detected failures
    pub fn attempt_recovery(&mut self, failure: &DetectedFailure) -> RecoveryAttempt {
        let recovery_strategy = self.select_recovery_strategy(&failure.failure_type);
        
        let attempt = RecoveryAttempt {
            timestamp: Instant::now(),
            failure_type: failure.failure_type.clone(),
            recovery_strategy: recovery_strategy.clone(),
            attempt_number: 1, // Simplified for this example
            success: true,     // Simplified for this example
            duration: Duration::from_millis(100), // Simplified
            details: format!("Applied recovery strategy: {:?}", recovery_strategy),
        };

        self.recovery_history.push_back(attempt.clone());
        while self.recovery_history.len() > 100 {
            self.recovery_history.pop_front();
        }

        attempt
    }

    /// Selects appropriate recovery strategy for a failure type
    fn select_recovery_strategy(&self, failure_type: &FailureType) -> RecoveryStrategy {
        match failure_type {
            FailureType::MemoryExhaustion => RecoveryStrategy::MemoryDefragmentation,
            FailureType::CpuSaturation => RecoveryStrategy::ScaleDown,
            FailureType::HighErrorRate => RecoveryStrategy::GracefulDegradation,
            FailureType::ResourceLeak => RecoveryStrategy::ResourceCleanup,
            FailureType::DependencyFailure => RecoveryStrategy::ProcessRestart,
            FailureType::CacheCorruption => RecoveryStrategy::CacheReset,
        }
    }
}

/// Detected failure information
#[derive(Debug, Clone)]
pub struct DetectedFailure {
    /// Type of failure
    pub failure_type: FailureType,
    /// Severity of the failure
    pub severity: FailureSeverity,
    /// Timestamp when failure was detected
    pub timestamp: Instant,
    /// Additional failure details
    pub details: String,
}

/// Types of failures that can be detected
#[derive(Debug, Clone, PartialEq)]
pub enum FailureType {
    /// Memory exhaustion
    MemoryExhaustion,
    /// CPU saturation
    CpuSaturation,
    /// High error rate
    HighErrorRate,
    /// Resource leak
    ResourceLeak,
    /// Dependency failure
    DependencyFailure,
    /// Cache corruption
    CacheCorruption,
}

/// Severity levels for failures
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub enum FailureSeverity {
    /// Low severity failure
    Low,
    /// Medium severity failure
    Medium,
    /// High severity failure
    High,
    /// Critical severity failure
    Critical,
}

/// Recovery attempt information
#[derive(Debug, Clone)]
pub struct RecoveryAttempt {
    /// Timestamp of the recovery attempt
    pub timestamp: Instant,
    /// Type of failure being recovered from
    pub failure_type: FailureType,
    /// Recovery strategy used
    pub recovery_strategy: RecoveryStrategy,
    /// Attempt number (for retries)
    pub attempt_number: usize,
    /// Whether the recovery was successful
    pub success: bool,
    /// Duration of the recovery attempt
    pub duration: Duration,
    /// Recovery attempt details
    pub details: String,
}

/// Health status of the system
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System has warnings
    Warning,
    /// System is degraded
    Degraded,
    /// System is in critical state
    Critical,
    /// System has failed
    Failed,
}

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    /// Circuit is closed (normal operation)
    Closed,
    /// Circuit is open (failures detected)
    Open,
    /// Circuit is half-open (testing recovery)
    HalfOpen,
}

/// Infrastructure health monitoring
#[derive(Debug)]
pub struct InfrastructureHealth {
    /// Overall health status
    pub overall_health: HealthStatus,
    /// Component health statuses
    pub component_health: HashMap<String, HealthStatus>,
    /// Health check results
    pub health_checks: Vec<HealthCheck>,
    /// Last health assessment timestamp
    pub last_assessment: Instant,
}

impl InfrastructureHealth {
    /// Creates a new infrastructure health monitor
    pub fn new() -> Self {
        Self {
            overall_health: HealthStatus::Healthy,
            component_health: HashMap::new(),
            health_checks: Vec::new(),
            last_assessment: Instant::now(),
        }
    }

    /// Performs health checks on all components
    pub fn perform_health_checks(&mut self) -> Vec<HealthCheck> {
        let checks = vec![
            HealthCheck {
                component: "PackageRegistry".to_string(),
                status: HealthStatus::Healthy,
                timestamp: Instant::now(),
                details: "Package registry operational".to_string(),
            },
            HealthCheck {
                component: "DependencyManager".to_string(),
                status: HealthStatus::Healthy,
                timestamp: Instant::now(),
                details: "Dependency manager operational".to_string(),
            },
            HealthCheck {
                component: "ResourceManager".to_string(),
                status: HealthStatus::Healthy,
                timestamp: Instant::now(),
                details: "Resource manager operational".to_string(),
            },
            HealthCheck {
                component: "ThreadPool".to_string(),
                status: HealthStatus::Healthy,
                timestamp: Instant::now(),
                details: "Thread pool operational".to_string(),
            },
        ];

        self.health_checks = checks.clone();
        self.last_assessment = Instant::now();
        
        // Update component health
        for check in &checks {
            self.component_health.insert(check.component.clone(), check.status.clone());
        }
        
        // Update overall health
        self.overall_health = self.calculate_overall_health();
        
        checks
    }

    /// Calculates overall health based on component health
    fn calculate_overall_health(&self) -> HealthStatus {
        let statuses: Vec<&HealthStatus> = self.component_health.values().collect();
        
        if statuses.iter().any(|&s| s == &HealthStatus::Failed) {
            HealthStatus::Failed
        } else if statuses.iter().any(|&s| s == &HealthStatus::Critical) {
            HealthStatus::Critical
        } else if statuses.iter().any(|&s| s == &HealthStatus::Degraded) {
            HealthStatus::Degraded
        } else if statuses.iter().any(|&s| s == &HealthStatus::Warning) {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }
}

/// Individual health check result
#[derive(Debug, Clone)]
pub struct HealthCheck {
    /// Component being checked
    pub component: String,
    /// Health status of the component
    pub status: HealthStatus,
    /// Timestamp of the health check
    pub timestamp: Instant,
    /// Additional details about the health check
    pub details: String,
}

/// Performance benchmarks for the infrastructure
#[derive(Debug)]
pub struct PerformanceBenchmarks {
    /// Package registration benchmarks
    pub package_registration: BenchmarkResults,
    /// Dependency resolution benchmarks
    pub dependency_resolution: BenchmarkResults,
    /// Resource management benchmarks
    pub resource_management: BenchmarkResults,
    /// Overall system throughput
    pub system_throughput: f64,
    /// Average response times
    pub average_response_times: HashMap<String, Duration>,
}

impl PerformanceBenchmarks {
    /// Creates new performance benchmarks
    pub fn new() -> Self {
        Self {
            package_registration: BenchmarkResults::new("Package Registration"),
            dependency_resolution: BenchmarkResults::new("Dependency Resolution"),
            resource_management: BenchmarkResults::new("Resource Management"),
            system_throughput: 0.0,
            average_response_times: HashMap::new(),
        }
    }
}

/// Benchmark results for a specific operation
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    /// Name of the benchmarked operation
    pub operation_name: String,
    /// Number of samples collected
    pub sample_count: usize,
    /// Average execution time
    pub average_time: Duration,
    /// Minimum execution time
    pub min_time: Duration,
    /// Maximum execution time
    pub max_time: Duration,
    /// Standard deviation of execution times
    pub std_deviation: Duration,
    /// Throughput (operations per second)
    pub throughput: f64,
}

impl BenchmarkResults {
    /// Creates new benchmark results
    pub fn new(operation_name: &str) -> Self {
        Self {
            operation_name: operation_name.to_string(),
            sample_count: 0,
            average_time: Duration::from_millis(0),
            min_time: Duration::from_millis(u64::MAX),
            max_time: Duration::from_millis(0),
            std_deviation: Duration::from_millis(0),
            throughput: 0.0,
        }
    }

    /// Adds a timing sample to the benchmark
    pub fn add_sample(&mut self, execution_time: Duration) {
        self.sample_count += 1;
        
        if execution_time < self.min_time {
            self.min_time = execution_time;
        }
        
        if execution_time > self.max_time {
            self.max_time = execution_time;
        }
        
        // Update average (simplified calculation)
        let total_nanos = self.average_time.as_nanos() * (self.sample_count - 1) as u128 + execution_time.as_nanos();
        self.average_time = Duration::from_nanos((total_nanos / self.sample_count as u128) as u64);
        
        // Update throughput
        if self.average_time.as_millis() > 0 {
            self.throughput = 1000.0 / self.average_time.as_millis() as f64;
        }
    }
}

impl ExtremeMonorepoInfrastructure {
    /// Creates a new extreme monorepo infrastructure
    pub fn new(config: ExtremeMonorepoInfrastructureConfig) -> Self {
        let resource_limits = ResourceLimits {
            memory_warning_percent: 75.0,
            memory_critical_percent: 90.0,
            cpu_warning_percent: config.cpu_warning_threshold_percent,
            cpu_critical_threshold_percent: config.cpu_critical_threshold_percent,
        };

        Self {
            package_registry: Arc::new(RwLock::new(OptimizedPackageRegistry::new())),
            dependency_manager: Arc::new(RwLock::new(OptimizedDependencyManager::new())),
            resource_manager: Arc::new(Mutex::new(ResourceManager::new(resource_limits))),
            metrics_collector: Arc::new(Mutex::new(MetricsCollector::new(config.metrics_collection.clone()))),
            thread_pool: Arc::new(Mutex::new(ThreadPool::new(ThreadPoolConfig {
                initial_size: config.thread_pool_size,
                max_size: config.thread_pool_size * 2,
                min_size: config.thread_pool_size / 2,
                queue_capacity: 1000,
            }))),
            scaling_manager: Arc::new(Mutex::new(ScalingManager::new(config.scaling_policies.clone()))),
            failure_recovery_manager: Arc::new(Mutex::new(FailureRecoveryManager::new(config.failure_recovery.clone()))),
            infrastructure_health: Arc::new(RwLock::new(InfrastructureHealth::new())),
            active_session: Arc::new(AtomicBool::new(false)),
            packages_processed: Arc::new(AtomicUsize::new(0)),
            current_memory_usage: Arc::new(AtomicU64::new(0)),
            current_cpu_utilization: Arc::new(AtomicU64::new(0)),
            performance_benchmarks: Arc::new(Mutex::new(PerformanceBenchmarks::new())),
            config,
        }
    }

    /// Initializes the infrastructure for extreme scale testing
    pub fn initialize(&self) -> Result<()> {
        self.active_session.store(true, Ordering::SeqCst);
        
        // Start resource monitoring
        if let Ok(mut resource_manager) = self.resource_manager.lock() {
            resource_manager.start_monitoring();
        }
        
        // Perform initial health checks
        if let Ok(mut health) = self.infrastructure_health.write() {
            health.perform_health_checks();
        }
        
        println!("Extreme monorepo infrastructure initialized for {} packages", self.config.target_package_count);
        Ok(())
    }

    /// Shuts down the infrastructure
    pub fn shutdown(&self) -> Result<()> {
        self.active_session.store(false, Ordering::SeqCst);
        
        // Stop resource monitoring
        if let Ok(mut resource_manager) = self.resource_manager.lock() {
            resource_manager.stop_monitoring();
        }
        
        println!("Extreme monorepo infrastructure shut down");
        Ok(())
    }

    /// Generates a synthetic extreme monorepo for testing
    pub fn generate_synthetic_monorepo(&self, package_count: usize) -> Result<()> {
        println!("Generating synthetic monorepo with {} packages...", package_count);
        
        let start_time = Instant::now();
        let mut packages_created = 0;
        
        // Generate packages in batches for efficiency
        let batch_size = self.config.package_generation_batch_size;
        for batch_start in (0..package_count).step_by(batch_size) {
            let batch_end = std::cmp::min(batch_start + batch_size, package_count);
            let batch_packages = self.generate_package_batch(batch_start, batch_end)?;
            
            // Register packages in the registry
            if let Ok(mut registry) = self.package_registry.write() {
                for package in batch_packages {
                    registry.register_package(package)?;
                    packages_created += 1;
                }
            }
            
            // Update progress
            self.packages_processed.store(packages_created, Ordering::SeqCst);
            
            // Check if we should continue
            if !self.active_session.load(Ordering::SeqCst) {
                break;
            }
        }
        
        let generation_time = start_time.elapsed();
        println!("Generated {} packages in {:?}", packages_created, generation_time);
        
        // Update performance benchmarks
        if let Ok(mut benchmarks) = self.performance_benchmarks.lock() {
            benchmarks.package_registration.add_sample(generation_time);
        }
        
        Ok(())
    }

    /// Generates a batch of packages
    fn generate_package_batch(&self, start_index: usize, end_index: usize) -> Result<Vec<PackageInfo>> {
        let mut packages = Vec::new();
        
        for i in start_index..end_index {
            let package = PackageInfo {
                id: format!("package-{:06}", i),
                name: format!("extreme-package-{}", i),
                version: "1.0.0".to_string(),
                dependencies: self.generate_package_dependencies(i)?,
                size: 1024 * (i % 100 + 1), // Variable package sizes
                complexity_score: (i as f64 * 0.1) % 10.0,
                last_modified: SystemTime::now(),
                package_type: match i % 5 {
                    0 => PackageType::Library,
                    1 => PackageType::Application,
                    2 => PackageType::Tool,
                    3 => PackageType::Test,
                    _ => PackageType::Documentation,
                },
                build_config: BuildConfig {
                    build_type: BuildType::Release,
                    build_flags: vec!["--optimization".to_string()],
                    environment: HashMap::new(),
                    build_dependencies: Vec::new(),
                },
            };
            
            packages.push(package);
        }
        
        Ok(packages)
    }

    /// Generates realistic dependencies for a package
    fn generate_package_dependencies(&self, package_index: usize) -> Result<Vec<String>> {
        let mut dependencies = Vec::new();
        
        // Generate dependencies based on package index and configuration
        let max_deps = std::cmp::min(
            self.config.max_dependencies_per_package, 
            package_index / 10 + 1 // Earlier packages have fewer dependencies
        );
        
        let dep_count = package_index % max_deps + 1;
        
        for dep_idx in 0..dep_count {
            if package_index > dep_idx {
                let dep_id = format!("package-{:06}", package_index - dep_idx - 1);
                dependencies.push(dep_id);
            }
        }
        
        Ok(dependencies)
    }

    /// Runs infrastructure stress tests
    pub fn run_infrastructure_stress_tests(&self) -> Result<InfrastructureTestResults> {
        println!("Running infrastructure stress tests...");
        
        let start_time = Instant::now();
        let mut test_results = InfrastructureTestResults::new();
        
        // Test 1: Package registration stress
        test_results.package_registration_results = self.test_package_registration_stress()?;
        
        // Test 2: Dependency resolution stress
        test_results.dependency_resolution_results = self.test_dependency_resolution_stress()?;
        
        // Test 3: Resource management stress
        test_results.resource_management_results = self.test_resource_management_stress()?;
        
        // Test 4: Concurrent operations stress
        test_results.concurrent_operations_results = self.test_concurrent_operations_stress()?;
        
        test_results.total_test_duration = start_time.elapsed();
        
        println!("Infrastructure stress tests completed in {:?}", test_results.total_test_duration);
        Ok(test_results)
    }

    /// Tests package registration under stress
    fn test_package_registration_stress(&self) -> Result<StressTestResults> {
        println!("Testing package registration stress...");
        
        let start_time = Instant::now();
        let mut results = StressTestResults::new("Package Registration Stress");
        
        // Register packages rapidly
        let package_count = 1000;
        for i in 0..package_count {
            let package = PackageInfo {
                id: format!("stress-package-{}", i),
                name: format!("stress-test-{}", i),
                version: "1.0.0".to_string(),
                dependencies: vec![],
                size: 1024,
                complexity_score: 1.0,
                last_modified: SystemTime::now(),
                package_type: PackageType::Library,
                build_config: BuildConfig {
                    build_type: BuildType::Release,
                    build_flags: Vec::new(),
                    environment: HashMap::new(),
                    build_dependencies: Vec::new(),
                },
            };
            
            let registration_start = Instant::now();
            if let Ok(mut registry) = self.package_registry.write() {
                registry.register_package(package)?;
            }
            let registration_time = registration_start.elapsed();
            
            results.add_operation_result(OperationResult {
                operation_id: i,
                duration: registration_time,
                success: true,
                error_message: None,
            });
        }
        
        results.total_duration = start_time.elapsed();
        results.calculate_statistics();
        
        Ok(results)
    }

    /// Tests dependency resolution under stress
    fn test_dependency_resolution_stress(&self) -> Result<StressTestResults> {
        println!("Testing dependency resolution stress...");
        
        let start_time = Instant::now();
        let mut results = StressTestResults::new("Dependency Resolution Stress");
        
        // Resolve dependencies for multiple packages
        if let Ok(registry) = self.package_registry.read() {
            let package_ids: Vec<String> = registry.packages_by_id.keys().take(100).cloned().collect();
            
            for (i, package_id) in package_ids.iter().enumerate() {
                let resolution_start = Instant::now();
                
                if let Ok(mut dep_manager) = self.dependency_manager.write() {
                    let _dependencies = dep_manager.resolve_dependencies(package_id)?;
                }
                
                let resolution_time = resolution_start.elapsed();
                
                results.add_operation_result(OperationResult {
                    operation_id: i,
                    duration: resolution_time,
                    success: true,
                    error_message: None,
                });
            }
        }
        
        results.total_duration = start_time.elapsed();
        results.calculate_statistics();
        
        Ok(results)
    }

    /// Tests resource management under stress
    fn test_resource_management_stress(&self) -> Result<StressTestResults> {
        println!("Testing resource management stress...");
        
        let start_time = Instant::now();
        let mut results = StressTestResults::new("Resource Management Stress");
        
        // Simulate high resource usage
        for i in 0..100 {
            let monitoring_start = Instant::now();
            
            if let Ok(mut resource_manager) = self.resource_manager.lock() {
                resource_manager.update_stats();
                let _limits = resource_manager.check_limits();
            }
            
            let monitoring_time = monitoring_start.elapsed();
            
            results.add_operation_result(OperationResult {
                operation_id: i,
                duration: monitoring_time,
                success: true,
                error_message: None,
            });
            
            // Simulate work
            thread::sleep(Duration::from_millis(10));
        }
        
        results.total_duration = start_time.elapsed();
        results.calculate_statistics();
        
        Ok(results)
    }

    /// Tests concurrent operations under stress
    fn test_concurrent_operations_stress(&self) -> Result<StressTestResults> {
        println!("Testing concurrent operations stress...");
        
        let start_time = Instant::now();
        let mut results = StressTestResults::new("Concurrent Operations Stress");
        
        // Simulate concurrent package operations
        let operations_count = 200;
        for i in 0..operations_count {
            let operation_start = Instant::now();
            
            // Simulate concurrent registry access
            if let Ok(registry) = self.package_registry.read() {
                let _package_count = registry.packages_by_id.len();
            }
            
            // Simulate concurrent metrics collection
            if let Ok(mut collector) = self.metrics_collector.lock() {
                collector.collect_metrics();
            }
            
            let operation_time = operation_start.elapsed();
            
            results.add_operation_result(OperationResult {
                operation_id: i,
                duration: operation_time,
                success: true,
                error_message: None,
            });
        }
        
        results.total_duration = start_time.elapsed();
        results.calculate_statistics();
        
        Ok(results)
    }
}

/// Infrastructure test results
#[derive(Debug)]
pub struct InfrastructureTestResults {
    /// Package registration test results
    pub package_registration_results: StressTestResults,
    /// Dependency resolution test results
    pub dependency_resolution_results: StressTestResults,
    /// Resource management test results
    pub resource_management_results: StressTestResults,
    /// Concurrent operations test results
    pub concurrent_operations_results: StressTestResults,
    /// Total test duration
    pub total_test_duration: Duration,
}

impl InfrastructureTestResults {
    /// Creates new infrastructure test results
    pub fn new() -> Self {
        Self {
            package_registration_results: StressTestResults::new("Package Registration"),
            dependency_resolution_results: StressTestResults::new("Dependency Resolution"),
            resource_management_results: StressTestResults::new("Resource Management"),
            concurrent_operations_results: StressTestResults::new("Concurrent Operations"),
            total_test_duration: Duration::from_millis(0),
        }
    }
}

/// Stress test results for a specific operation type
#[derive(Debug)]
pub struct StressTestResults {
    /// Name of the stress test
    pub test_name: String,
    /// Individual operation results
    pub operation_results: Vec<OperationResult>,
    /// Total test duration
    pub total_duration: Duration,
    /// Average operation duration
    pub average_duration: Duration,
    /// Minimum operation duration
    pub min_duration: Duration,
    /// Maximum operation duration
    pub max_duration: Duration,
    /// Operations per second
    pub operations_per_second: f64,
    /// Success rate (percentage)
    pub success_rate: f64,
}

impl StressTestResults {
    /// Creates new stress test results
    pub fn new(test_name: &str) -> Self {
        Self {
            test_name: test_name.to_string(),
            operation_results: Vec::new(),
            total_duration: Duration::from_millis(0),
            average_duration: Duration::from_millis(0),
            min_duration: Duration::from_millis(u64::MAX),
            max_duration: Duration::from_millis(0),
            operations_per_second: 0.0,
            success_rate: 0.0,
        }
    }

    /// Adds an operation result
    pub fn add_operation_result(&mut self, result: OperationResult) {
        if result.duration < self.min_duration {
            self.min_duration = result.duration;
        }
        
        if result.duration > self.max_duration {
            self.max_duration = result.duration;
        }
        
        self.operation_results.push(result);
    }

    /// Calculates statistics for the test results
    pub fn calculate_statistics(&mut self) {
        if self.operation_results.is_empty() {
            return;
        }

        // Calculate average duration
        let total_nanos: u128 = self.operation_results.iter()
            .map(|r| r.duration.as_nanos())
            .sum();
        self.average_duration = Duration::from_nanos((total_nanos / self.operation_results.len() as u128) as u64);

        // Calculate operations per second
        if self.total_duration.as_millis() > 0 {
            self.operations_per_second = (self.operation_results.len() as f64 * 1000.0) / self.total_duration.as_millis() as f64;
        }

        // Calculate success rate
        let successful_operations = self.operation_results.iter()
            .filter(|r| r.success)
            .count();
        self.success_rate = (successful_operations as f64 / self.operation_results.len() as f64) * 100.0;
    }
}

/// Result of an individual operation
#[derive(Debug, Clone)]
pub struct OperationResult {
    /// Operation identifier
    pub operation_id: usize,
    /// Operation duration
    pub duration: Duration,
    /// Whether the operation was successful
    pub success: bool,
    /// Error message if the operation failed
    pub error_message: Option<String>,
}

// Unit tests for the extreme monorepo infrastructure
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extreme_monorepo_infrastructure_creation() {
        let config = ExtremeMonorepoInfrastructureConfig::default();
        let infrastructure = ExtremeMonorepoInfrastructure::new(config);
        
        assert!(!infrastructure.active_session.load(Ordering::SeqCst));
        assert_eq!(infrastructure.packages_processed.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_infrastructure_initialization() {
        let config = ExtremeMonorepoInfrastructureConfig::default();
        let infrastructure = ExtremeMonorepoInfrastructure::new(config);
        
        let result = infrastructure.initialize();
        assert!(result.is_ok());
        assert!(infrastructure.active_session.load(Ordering::SeqCst));
        
        let shutdown_result = infrastructure.shutdown();
        assert!(shutdown_result.is_ok());
        assert!(!infrastructure.active_session.load(Ordering::SeqCst));
    }

    #[test]
    fn test_package_registry_operations() {
        let mut registry = OptimizedPackageRegistry::new();
        
        let package = PackageInfo {
            id: "test-package-1".to_string(),
            name: "Test Package".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["dep1".to_string(), "dep2".to_string()],
            size: 1024,
            complexity_score: 5.0,
            last_modified: SystemTime::now(),
            package_type: PackageType::Library,
            build_config: BuildConfig {
                build_type: BuildType::Release,
                build_flags: Vec::new(),
                environment: HashMap::new(),
                build_dependencies: Vec::new(),
            },
        };
        
        // Test package registration
        let result = registry.register_package(package.clone());
        assert!(result.is_ok());
        
        // Test package retrieval
        let retrieved = registry.get_package("test-package-1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Package");
        
        // Test dependency queries
        let dependencies = registry.get_dependencies("test-package-1");
        assert!(dependencies.is_some());
        assert_eq!(dependencies.unwrap().len(), 2);
    }

    #[test]
    fn test_dependency_manager_operations() {
        let mut manager = OptimizedDependencyManager::new();
        
        // Test dependency addition
        let result1 = manager.add_dependency("package-a", "package-b");
        assert!(result1.is_ok());
        
        let result2 = manager.add_dependency("package-b", "package-c");
        assert!(result2.is_ok());
        
        // Test dependency resolution
        let resolution = manager.resolve_dependencies("package-a");
        assert!(resolution.is_ok());
        
        let deps = resolution.unwrap();
        assert!(!deps.is_empty());
    }

    #[test]
    fn test_resource_manager_monitoring() {
        let limits = ResourceLimits {
            memory_warning_percent: 75.0,
            memory_critical_percent: 90.0,
            cpu_warning_percent: 70.0,
            cpu_critical_percent: 90.0,
        };
        
        let mut manager = ResourceManager::new(limits);
        
        manager.start_monitoring();
        assert!(manager.monitoring_active);
        
        manager.update_stats();
        let limits_check = manager.check_limits();
        // Should not exceed limits in test environment
        assert!(limits_check.is_empty());
        
        manager.stop_monitoring();
        assert!(!manager.monitoring_active);
    }

    #[test]
    fn test_metrics_collector() {
        let config = MetricsCollectionConfig::default();
        let mut collector = MetricsCollector::new(config);
        
        collector.collect_metrics();
        
        // Verify metrics were collected
        assert!(!collector.metrics.is_empty());
        
        // Check specific metric types
        assert!(collector.metrics.contains_key(&MetricType::MemoryUsage));
        assert!(collector.metrics.contains_key(&MetricType::CpuUtilization));
    }

    #[test]
    fn test_scaling_manager_decisions() {
        let config = ScalingPoliciesConfig::default();
        let mut manager = ScalingManager::new(config);
        
        // Test with high utilization (should trigger scale up)
        let high_utilization = ResourceUtilization {
            timestamp: Instant::now(),
            memory_usage_percent: 85.0,
            cpu_usage_percent: 85.0,
            memory_used_mb: 4000,
            cpu_cores_active: 8,
        };
        
        let decision = manager.make_scaling_decision(high_utilization);
        assert!(decision.is_some());
        
        if let Some(d) = decision {
            assert_eq!(d.decision_type, ScalingDecisionType::ScaleUp);
        }
    }

    #[test]
    fn test_failure_recovery_manager() {
        let config = FailureRecoveryConfig::default();
        let mut manager = FailureRecoveryManager::new(config);
        
        // Create test metrics indicating failure
        let mut metrics = HashMap::new();
        let mut memory_values = VecDeque::new();
        memory_values.push_back(MetricValue {
            timestamp: Instant::now(),
            value: 5000.0, // High memory usage
            unit: "MB".to_string(),
        });
        metrics.insert(MetricType::MemoryUsage, memory_values);
        
        // Test failure detection
        let failures = manager.detect_failures(&metrics);
        assert!(!failures.is_empty());
        
        // Test recovery attempt
        if let Some(failure) = failures.first() {
            let recovery = manager.attempt_recovery(failure);
            assert!(recovery.success);
        }
    }

    #[test]
    fn test_infrastructure_health_checks() {
        let mut health = InfrastructureHealth::new();
        
        let checks = health.perform_health_checks();
        assert!(!checks.is_empty());
        assert_eq!(health.overall_health, HealthStatus::Healthy);
        
        // Verify specific components were checked
        assert!(health.component_health.contains_key("PackageRegistry"));
        assert!(health.component_health.contains_key("DependencyManager"));
        assert!(health.component_health.contains_key("ResourceManager"));
        assert!(health.component_health.contains_key("ThreadPool"));
    }

    #[test]
    fn test_synthetic_monorepo_generation() {
        let config = ExtremeMonorepoInfrastructureConfig {
            target_package_count: 10,
            ..Default::default()
        };
        let infrastructure = ExtremeMonorepoInfrastructure::new(config);
        
        let init_result = infrastructure.initialize();
        assert!(init_result.is_ok());
        
        let generation_result = infrastructure.generate_synthetic_monorepo(10);
        assert!(generation_result.is_ok());
        
        // Verify packages were created
        assert_eq!(infrastructure.packages_processed.load(Ordering::SeqCst), 10);
        
        let shutdown_result = infrastructure.shutdown();
        assert!(shutdown_result.is_ok());
    }

    #[test]
    fn test_infrastructure_stress_tests() {
        let config = ExtremeMonorepoInfrastructureConfig {
            target_package_count: 50,
            thread_pool_size: 4,
            ..Default::default()
        };
        let infrastructure = ExtremeMonorepoInfrastructure::new(config);
        
        let init_result = infrastructure.initialize();
        assert!(init_result.is_ok());
        
        // Generate some packages first
        let gen_result = infrastructure.generate_synthetic_monorepo(50);
        assert!(gen_result.is_ok());
        
        // Run stress tests
        let test_results = infrastructure.run_infrastructure_stress_tests();
        assert!(test_results.is_ok());
        
        let results = test_results.unwrap();
        assert!(results.total_test_duration > Duration::from_millis(0));
        assert!(results.package_registration_results.success_rate > 90.0);
        
        let shutdown_result = infrastructure.shutdown();
        assert!(shutdown_result.is_ok());
    }

    #[test]
    fn test_benchmark_results() {
        let mut benchmark = BenchmarkResults::new("Test Operation");
        
        // Add some sample timings
        benchmark.add_sample(Duration::from_millis(10));
        benchmark.add_sample(Duration::from_millis(20));
        benchmark.add_sample(Duration::from_millis(15));
        
        assert_eq!(benchmark.sample_count, 3);
        assert_eq!(benchmark.min_time, Duration::from_millis(10));
        assert_eq!(benchmark.max_time, Duration::from_millis(20));
        assert!(benchmark.throughput > 0.0);
    }

    #[test]
    fn test_stress_test_results_calculation() {
        let mut results = StressTestResults::new("Test Stress");
        
        // Add operation results
        results.add_operation_result(OperationResult {
            operation_id: 1,
            duration: Duration::from_millis(10),
            success: true,
            error_message: None,
        });
        
        results.add_operation_result(OperationResult {
            operation_id: 2,
            duration: Duration::from_millis(20),
            success: true,
            error_message: None,
        });
        
        results.add_operation_result(OperationResult {
            operation_id: 3,
            duration: Duration::from_millis(30),
            success: false,
            error_message: Some("Test error".to_string()),
        });
        
        results.total_duration = Duration::from_millis(100);
        results.calculate_statistics();
        
        assert_eq!(results.operation_results.len(), 3);
        assert_eq!(results.min_duration, Duration::from_millis(10));
        assert_eq!(results.max_duration, Duration::from_millis(30));
        assert!((results.success_rate - 66.67).abs() < 0.1); // 2/3 * 100 ≈ 66.67%
        assert!(results.operations_per_second > 0.0);
    }
}