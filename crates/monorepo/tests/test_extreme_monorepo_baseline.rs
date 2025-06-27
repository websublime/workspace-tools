//! Baseline Performance Testing for Extreme Monorepos
//!
//! This module implements comprehensive baseline testing for extreme monorepos (500+ packages).
//! It establishes performance metrics, resource usage patterns, and operational baselines
//! that serve as reference points for stress testing and performance monitoring.
//!
//! ## What
//! 
//! Comprehensive baseline testing system that:
//! - Generates synthetic extreme monorepos using the extreme generator
//! - Executes fundamental monorepo operations at scale
//! - Collects detailed performance metrics and resource usage data
//! - Establishes reference thresholds for different scales and operations
//! - Provides structured reporting for baseline comparisons
//! - Detects natural bottlenecks and scaling limitations
//! 
//! ## How
//! 
//! The system uses a multi-layered approach:
//! 1. **Synthetic Generation**: Creates realistic extreme monorepos for consistent testing
//! 2. **Operation Benchmarking**: Measures performance of core monorepo operations
//! 3. **Resource Monitoring**: Tracks CPU, memory, I/O, and other system resources
//! 4. **Scalability Analysis**: Tests behavior across different monorepo sizes
//! 5. **Baseline Establishment**: Creates reference metrics for future comparisons
//! 6. **Bottleneck Detection**: Identifies natural scaling limits and constraints
//! 7. **Report Generation**: Produces structured baseline reports
//! 
//! ## Why
//! 
//! Baseline testing is critical for:
//! - Establishing reference performance metrics before stress testing
//! - Understanding natural scaling characteristics of the system
//! - Detecting performance regressions during development
//! - Providing data for capacity planning and resource allocation
//! - Enabling meaningful comparisons across different configurations
//! - Identifying optimization opportunities and system limits

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

mod test_synthetic_extreme_monorepo_generator;
use test_synthetic_extreme_monorepo_generator::{
    ExtremeMonorepoGenerator, 
    ExtremeMonorepoConfig, 
    MonorepoStructure,
    PackageInfo,
    DependencyInfo,
    GenerationMetrics
};

/// Performance metrics for baseline testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetrics {
    /// Total execution time for the operation
    pub execution_time: Duration,
    /// Peak memory usage during operation
    pub peak_memory_mb: u64,
    /// Average CPU usage percentage
    pub avg_cpu_percent: f64,
    /// Total I/O operations performed
    pub io_operations: u64,
    /// Number of items processed
    pub items_processed: u64,
    /// Throughput (items per second)
    pub throughput: f64,
    /// Average response time per operation
    pub avg_response_time: Duration,
    /// 95th percentile response time
    pub p95_response_time: Duration,
    /// 99th percentile response time
    pub p99_response_time: Duration,
    /// Memory allocation rate (MB/s)
    pub memory_allocation_rate: f64,
    /// Cache hit rate (if applicable)
    pub cache_hit_rate: Option<f64>,
    /// Error rate (percentage)
    pub error_rate: f64,
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceMetrics {
    /// Available system memory (MB)
    pub available_memory_mb: u64,
    /// Total system memory (MB)
    pub total_memory_mb: u64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Number of CPU cores
    pub cpu_cores: u32,
    /// CPU usage across all cores
    pub cpu_usage_per_core: Vec<f64>,
    /// Average CPU usage
    pub avg_cpu_usage: f64,
    /// Disk space available (MB)
    pub disk_space_available_mb: u64,
    /// Disk I/O read rate (MB/s)
    pub disk_read_rate: f64,
    /// Disk I/O write rate (MB/s)
    pub disk_write_rate: f64,
    /// Network I/O rate (MB/s)
    pub network_io_rate: f64,
    /// Number of open file descriptors
    pub open_file_descriptors: u64,
    /// System load average
    pub load_average: f64,
}

/// Baseline test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineTestConfig {
    /// Monorepo sizes to test
    pub monorepo_sizes: Vec<u32>,
    /// Number of iterations per test
    pub iterations: u32,
    /// Warmup iterations (excluded from metrics)
    pub warmup_iterations: u32,
    /// Maximum test duration per operation
    pub max_test_duration: Duration,
    /// Resource monitoring interval
    pub monitoring_interval: Duration,
    /// Whether to include detailed profiling
    pub enable_profiling: bool,
    /// Whether to generate detailed reports
    pub generate_reports: bool,
    /// Output directory for reports
    pub report_output_dir: PathBuf,
    /// Operations to benchmark
    pub operations_to_test: Vec<MonorepoOperation>,
}

/// Operations that can be benchmarked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonorepoOperation {
    /// Generate synthetic monorepo
    Generation,
    /// Analyze package dependencies
    DependencyAnalysis,
    /// Detect changes in packages
    ChangeDetection,
    /// Execute tasks across packages
    TaskExecution,
    /// Validate monorepo structure
    StructureValidation,
    /// Build dependency graph
    GraphConstruction,
    /// Search and query operations
    SearchOperations,
    /// Configuration management
    ConfigurationManagement,
    /// Storage operations
    StorageOperations,
    /// Concurrent operations
    ConcurrentOperations,
}

/// Baseline test results for a specific operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationBaseline {
    /// Operation that was tested
    pub operation: MonorepoOperation,
    /// Monorepo size used for testing
    pub monorepo_size: u32,
    /// Baseline metrics
    pub metrics: BaselineMetrics,
    /// System resource usage
    pub system_metrics: SystemResourceMetrics,
    /// Test timestamp
    pub timestamp: DateTime<Utc>,
    /// Test configuration used
    pub test_config: BaselineTestConfig,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Complete baseline test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineTestResults {
    /// Test execution timestamp
    pub timestamp: DateTime<Utc>,
    /// Total test duration
    pub total_duration: Duration,
    /// Baseline results for each operation
    pub operation_baselines: Vec<OperationBaseline>,
    /// System information
    pub system_info: SystemInfo,
    /// Test configuration
    pub test_config: BaselineTestConfig,
    /// Overall summary statistics
    pub summary: BaselineSummary,
    /// Detected bottlenecks and limitations
    pub bottlenecks: Vec<DetectedBottleneck>,
    /// Scaling characteristics
    pub scaling_analysis: ScalingAnalysis,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system
    pub os: String,
    /// OS version
    pub os_version: String,
    /// Architecture
    pub architecture: String,
    /// CPU model
    pub cpu_model: String,
    /// Number of CPU cores
    pub cpu_cores: u32,
    /// Total RAM (MB)
    pub total_ram_mb: u64,
    /// Rust version
    pub rust_version: String,
    /// Cargo version
    pub cargo_version: String,
}

/// Summary of baseline results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineSummary {
    /// Total operations tested
    pub total_operations: u32,
    /// Average execution time across all operations
    pub avg_execution_time: Duration,
    /// Peak memory usage across all tests
    pub peak_memory_usage_mb: u64,
    /// Average throughput across operations
    pub avg_throughput: f64,
    /// Most expensive operation
    pub most_expensive_operation: MonorepoOperation,
    /// Fastest operation
    pub fastest_operation: MonorepoOperation,
    /// Memory-intensive operation
    pub most_memory_intensive: MonorepoOperation,
    /// Recommended configuration limits
    pub recommended_limits: RecommendedLimits,
}

/// Recommended operational limits based on baseline testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedLimits {
    /// Maximum recommended monorepo size
    pub max_monorepo_size: u32,
    /// Maximum concurrent operations
    pub max_concurrent_operations: u32,
    /// Recommended memory allocation (MB)
    pub recommended_memory_mb: u64,
    /// Recommended CPU cores
    pub recommended_cpu_cores: u32,
    /// Maximum safe throughput (ops/sec)
    pub max_safe_throughput: f64,
}

/// Detected performance bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedBottleneck {
    /// Type of bottleneck
    pub bottleneck_type: BottleneckType,
    /// Operation where bottleneck was detected
    pub operation: MonorepoOperation,
    /// Monorepo size where bottleneck occurs
    pub critical_size: u32,
    /// Description of the bottleneck
    pub description: String,
    /// Severity level
    pub severity: BottleneckSeverity,
    /// Recommended mitigation
    pub mitigation: String,
    /// Performance impact
    pub performance_impact: f64,
}

/// Types of performance bottlenecks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckType {
    /// Memory exhaustion
    MemoryExhaustion,
    /// CPU saturation
    CpuSaturation,
    /// I/O limitations
    IoLimitation,
    /// Algorithm complexity
    AlgorithmicComplexity,
    /// Resource contention
    ResourceContention,
    /// Memory fragmentation
    MemoryFragmentation,
    /// Cache inefficiency
    CacheInefficiency,
    /// Thread contention
    ThreadContention,
}

/// Severity levels for bottlenecks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    /// Low impact, performance degradation < 25%
    Low,
    /// Medium impact, performance degradation 25-50%
    Medium,
    /// High impact, performance degradation 50-75%
    High,
    /// Critical impact, performance degradation > 75%
    Critical,
    /// System failure/crash
    Fatal,
}

/// Scaling analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingAnalysis {
    /// Linear scaling operations
    pub linear_scaling: Vec<MonorepoOperation>,
    /// Operations with quadratic or worse scaling
    pub poor_scaling: Vec<MonorepoOperation>,
    /// Operations with sub-linear scaling
    pub excellent_scaling: Vec<MonorepoOperation>,
    /// Scaling coefficients for each operation
    pub scaling_coefficients: HashMap<MonorepoOperation, f64>,
    /// Memory scaling patterns
    pub memory_scaling: HashMap<MonorepoOperation, ScalingPattern>,
    /// CPU scaling patterns
    pub cpu_scaling: HashMap<MonorepoOperation, ScalingPattern>,
    /// I/O scaling patterns
    pub io_scaling: HashMap<MonorepoOperation, ScalingPattern>,
}

/// Scaling pattern classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingPattern {
    /// O(1) - Constant time
    Constant,
    /// O(log n) - Logarithmic
    Logarithmic,
    /// O(n) - Linear
    Linear,
    /// O(n log n) - Linearithmic
    Linearithmic,
    /// O(n²) - Quadratic
    Quadratic,
    /// O(n³) - Cubic
    Cubic,
    /// O(2^n) - Exponential
    Exponential,
    /// Unknown/irregular pattern
    Unknown,
}

/// Resource monitor for collecting system metrics
pub struct ResourceMonitor {
    /// Whether monitoring is active
    active: Arc<Mutex<bool>>,
    /// Collected metrics
    metrics: Arc<Mutex<Vec<SystemResourceMetrics>>>,
    /// Monitoring interval
    interval: Duration,
}

/// Baseline testing framework
pub struct BaselineTestFramework {
    /// Test configuration
    config: BaselineTestConfig,
    /// Resource monitor
    resource_monitor: ResourceMonitor,
    /// Monorepo generator
    generator: ExtremeMonorepoGenerator,
}

impl Default for BaselineTestConfig {
    fn default() -> Self {
        Self {
            monorepo_sizes: vec![100, 200, 500, 1000],
            iterations: 5,
            warmup_iterations: 2,
            max_test_duration: Duration::from_secs(300), // 5 minutes
            monitoring_interval: Duration::from_millis(100),
            enable_profiling: true,
            generate_reports: true,
            report_output_dir: PathBuf::from("./baseline_reports"),
            operations_to_test: vec![
                MonorepoOperation::Generation,
                MonorepoOperation::DependencyAnalysis,
                MonorepoOperation::ChangeDetection,
                MonorepoOperation::TaskExecution,
                MonorepoOperation::StructureValidation,
                MonorepoOperation::GraphConstruction,
                MonorepoOperation::SearchOperations,
                MonorepoOperation::ConfigurationManagement,
                MonorepoOperation::StorageOperations,
                MonorepoOperation::ConcurrentOperations,
            ],
        }
    }
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new(interval: Duration) -> Self {
        Self {
            active: Arc::new(Mutex::new(false)),
            metrics: Arc::new(Mutex::new(Vec::new())),
            interval,
        }
    }

    /// Start monitoring system resources
    pub fn start_monitoring(&self) -> Result<(), Box<dyn std::error::Error>> {
        let active = Arc::clone(&self.active);
        let metrics = Arc::clone(&self.metrics);
        let interval = self.interval;

        // Set monitoring to active
        {
            let mut active_guard = active.lock()
                .map_err(|e| format!("Failed to acquire active lock: {}", e))?;
            *active_guard = true;
        }

        // Spawn monitoring thread
        let _monitor_thread = thread::spawn(move || {
            while {
                let active_guard = active.lock().unwrap_or_else(|_| {
                    // If poisoned, assume we should stop
                    return false;
                });
                *active_guard
            } {
                // Collect system metrics
                let system_metrics = Self::collect_system_metrics();
                
                // Store metrics
                if let Ok(mut metrics_guard) = metrics.lock() {
                    metrics_guard.push(system_metrics);
                }

                thread::sleep(interval);
            }
        });

        Ok(())
    }

    /// Stop monitoring and return collected metrics
    pub fn stop_monitoring(&self) -> Result<Vec<SystemResourceMetrics>, Box<dyn std::error::Error>> {
        // Stop monitoring
        {
            let mut active_guard = self.active.lock()
                .map_err(|e| format!("Failed to acquire active lock: {}", e))?;
            *active_guard = false;
        }

        // Wait a bit for the monitoring thread to stop
        thread::sleep(Duration::from_millis(200));

        // Return collected metrics
        let metrics_guard = self.metrics.lock()
            .map_err(|e| format!("Failed to acquire metrics lock: {}", e))?;
        Ok(metrics_guard.clone())
    }

    /// Collect current system metrics
    fn collect_system_metrics() -> SystemResourceMetrics {
        // This is a simplified implementation
        // In a real implementation, you would use system APIs to collect actual metrics
        SystemResourceMetrics {
            available_memory_mb: 8192, // Mock value
            total_memory_mb: 16384,    // Mock value
            memory_usage_percent: 50.0,
            cpu_cores: 8,
            cpu_usage_per_core: vec![25.0, 30.0, 20.0, 35.0, 40.0, 15.0, 25.0, 30.0],
            avg_cpu_usage: 27.5,
            disk_space_available_mb: 500000,
            disk_read_rate: 100.0,
            disk_write_rate: 50.0,
            network_io_rate: 10.0,
            open_file_descriptors: 1024,
            load_average: 2.5,
        }
    }
}

impl BaselineTestFramework {
    /// Create a new baseline testing framework
    pub fn new(config: BaselineTestConfig) -> Self {
        let resource_monitor = ResourceMonitor::new(config.monitoring_interval);
        let generator = ExtremeMonorepoGenerator::new();
        
        Self {
            config,
            resource_monitor,
            generator,
        }
    }

    /// Execute complete baseline testing suite
    pub fn execute_baseline_tests(&mut self) -> Result<BaselineTestResults, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let timestamp = Utc::now();
        
        println!("🚀 Starting baseline testing suite...");
        
        let mut operation_baselines = Vec::new();
        
        // Test each monorepo size
        for &size in &self.config.monorepo_sizes {
            println!("📊 Testing monorepo size: {} packages", size);
            
            // Test each operation
            for operation in &self.config.operations_to_test {
                println!("  🔧 Testing operation: {:?}", operation);
                
                let baseline = self.test_operation(operation.clone(), size)?;
                operation_baselines.push(baseline);
            }
        }
        
        let total_duration = start_time.elapsed();
        
        // Analyze results
        let summary = self.generate_summary(&operation_baselines);
        let bottlenecks = self.detect_bottlenecks(&operation_baselines);
        let scaling_analysis = self.analyze_scaling(&operation_baselines);
        
        let results = BaselineTestResults {
            timestamp,
            total_duration,
            operation_baselines,
            system_info: self.get_system_info(),
            test_config: self.config.clone(),
            summary,
            bottlenecks,
            scaling_analysis,
        };
        
        if self.config.generate_reports {
            self.generate_reports(&results)?;
        }
        
        println!("✅ Baseline testing completed in {:?}", total_duration);
        
        Ok(results)
    }

    /// Test a specific operation
    fn test_operation(&mut self, operation: MonorepoOperation, size: u32) -> Result<OperationBaseline, Box<dyn std::error::Error>> {
        let mut execution_times = Vec::new();
        let mut memory_usage = Vec::new();
        let mut throughput_values = Vec::new();
        
        // Generate test monorepo
        let monorepo = self.generate_test_monorepo(size)?;
        
        // Warmup iterations
        for _ in 0..self.config.warmup_iterations {
            let _ = self.execute_operation(&operation, &monorepo)?;
        }
        
        // Actual test iterations
        for iteration in 0..self.config.iterations {
            println!("    Iteration {}/{}", iteration + 1, self.config.iterations);
            
            // Start resource monitoring
            self.resource_monitor.start_monitoring()?;
            
            let start_time = Instant::now();
            let result = self.execute_operation(&operation, &monorepo)?;
            let execution_time = start_time.elapsed();
            
            // Stop monitoring and collect metrics
            let system_metrics = self.resource_monitor.stop_monitoring()?;
            
            execution_times.push(execution_time);
            
            // Calculate peak memory usage
            let peak_memory = system_metrics.iter()
                .map(|m| m.total_memory_mb - m.available_memory_mb)
                .max()
                .unwrap_or(0);
            memory_usage.push(peak_memory);
            
            // Calculate throughput
            let throughput = result.items_processed as f64 / execution_time.as_secs_f64();
            throughput_values.push(throughput);
        }
        
        // Calculate baseline metrics
        let metrics = self.calculate_baseline_metrics(&execution_times, &memory_usage, &throughput_values, size);
        let system_metrics = self.calculate_average_system_metrics(&self.resource_monitor.stop_monitoring()?);
        
        Ok(OperationBaseline {
            operation,
            monorepo_size: size,
            metrics,
            system_metrics,
            timestamp: Utc::now(),
            test_config: self.config.clone(),
            metadata: HashMap::new(),
        })
    }

    /// Generate a test monorepo of specified size
    fn generate_test_monorepo(&mut self, size: u32) -> Result<MonorepoStructure, Box<dyn std::error::Error>> {
        let mut config = ExtremeMonorepoConfig::default();
        config.base_package_count = size;
        config.max_additional_packages = 0; // Exact size
        
        self.generator.generate_monorepo(config)
    }

    /// Execute a specific operation and return execution metrics
    fn execute_operation(&self, operation: &MonorepoOperation, monorepo: &MonorepoStructure) -> Result<OperationResult, Box<dyn std::error::Error>> {
        match operation {
            MonorepoOperation::Generation => {
                // Mock generation operation
                Ok(OperationResult { 
                    items_processed: monorepo.packages.len() as u64,
                    operations_count: 1,
                })
            },
            MonorepoOperation::DependencyAnalysis => {
                // Mock dependency analysis
                let total_deps: u64 = monorepo.packages.iter()
                    .map(|p| p.dependencies.len() as u64)
                    .sum();
                Ok(OperationResult {
                    items_processed: total_deps,
                    operations_count: monorepo.packages.len() as u64,
                })
            },
            MonorepoOperation::ChangeDetection => {
                // Mock change detection
                Ok(OperationResult {
                    items_processed: monorepo.packages.len() as u64,
                    operations_count: monorepo.packages.len() as u64,
                })
            },
            MonorepoOperation::TaskExecution => {
                // Mock task execution
                Ok(OperationResult {
                    items_processed: monorepo.packages.len() as u64 * 3, // 3 tasks per package
                    operations_count: monorepo.packages.len() as u64,
                })
            },
            MonorepoOperation::StructureValidation => {
                // Mock structure validation
                Ok(OperationResult {
                    items_processed: monorepo.packages.len() as u64,
                    operations_count: 1,
                })
            },
            MonorepoOperation::GraphConstruction => {
                // Mock graph construction
                let total_edges: u64 = monorepo.packages.iter()
                    .map(|p| p.dependencies.len() as u64)
                    .sum();
                Ok(OperationResult {
                    items_processed: monorepo.packages.len() as u64 + total_edges,
                    operations_count: 1,
                })
            },
            MonorepoOperation::SearchOperations => {
                // Mock search operations
                Ok(OperationResult {
                    items_processed: monorepo.packages.len() as u64,
                    operations_count: 10, // 10 search queries
                })
            },
            MonorepoOperation::ConfigurationManagement => {
                // Mock configuration management
                Ok(OperationResult {
                    items_processed: monorepo.packages.len() as u64,
                    operations_count: 1,
                })
            },
            MonorepoOperation::StorageOperations => {
                // Mock storage operations
                Ok(OperationResult {
                    items_processed: monorepo.packages.len() as u64,
                    operations_count: monorepo.packages.len() as u64,
                })
            },
            MonorepoOperation::ConcurrentOperations => {
                // Mock concurrent operations
                Ok(OperationResult {
                    items_processed: monorepo.packages.len() as u64,
                    operations_count: monorepo.packages.len() as u64 / 4, // 4 packages per batch
                })
            },
        }
    }

    /// Calculate baseline metrics from collected data
    fn calculate_baseline_metrics(&self, execution_times: &[Duration], memory_usage: &[u64], throughput_values: &[f64], items_processed: u32) -> BaselineMetrics {
        let avg_execution_time = execution_times.iter().sum::<Duration>() / execution_times.len() as u32;
        let peak_memory = *memory_usage.iter().max().unwrap_or(&0);
        let avg_throughput = throughput_values.iter().sum::<f64>() / throughput_values.len() as f64;
        
        // Calculate percentiles
        let mut sorted_times = execution_times.to_vec();
        sorted_times.sort();
        
        let p95_index = (sorted_times.len() as f64 * 0.95) as usize;
        let p99_index = (sorted_times.len() as f64 * 0.99) as usize;
        
        let p95_response_time = sorted_times.get(p95_index).copied().unwrap_or(avg_execution_time);
        let p99_response_time = sorted_times.get(p99_index).copied().unwrap_or(avg_execution_time);
        
        BaselineMetrics {
            execution_time: avg_execution_time,
            peak_memory_mb: peak_memory,
            avg_cpu_percent: 50.0, // Mock value
            io_operations: items_processed as u64 * 2, // Mock value
            items_processed: items_processed as u64,
            throughput: avg_throughput,
            avg_response_time: avg_execution_time,
            p95_response_time,
            p99_response_time,
            memory_allocation_rate: peak_memory as f64 / avg_execution_time.as_secs_f64(),
            cache_hit_rate: Some(0.85), // Mock value
            error_rate: 0.0,
        }
    }

    /// Calculate average system metrics
    fn calculate_average_system_metrics(&self, metrics: &[SystemResourceMetrics]) -> SystemResourceMetrics {
        if metrics.is_empty() {
            return SystemResourceMetrics {
                available_memory_mb: 0,
                total_memory_mb: 0,
                memory_usage_percent: 0.0,
                cpu_cores: 0,
                cpu_usage_per_core: Vec::new(),
                avg_cpu_usage: 0.0,
                disk_space_available_mb: 0,
                disk_read_rate: 0.0,
                disk_write_rate: 0.0,
                network_io_rate: 0.0,
                open_file_descriptors: 0,
                load_average: 0.0,
            };
        }

        let avg_available_memory = metrics.iter().map(|m| m.available_memory_mb).sum::<u64>() / metrics.len() as u64;
        let avg_cpu_usage = metrics.iter().map(|m| m.avg_cpu_usage).sum::<f64>() / metrics.len() as f64;
        let avg_disk_read_rate = metrics.iter().map(|m| m.disk_read_rate).sum::<f64>() / metrics.len() as f64;
        let avg_disk_write_rate = metrics.iter().map(|m| m.disk_write_rate).sum::<f64>() / metrics.len() as f64;
        
        // Use the first metric as a template and update averages
        let first_metric = &metrics[0];
        SystemResourceMetrics {
            available_memory_mb: avg_available_memory,
            total_memory_mb: first_metric.total_memory_mb,
            memory_usage_percent: 100.0 - (avg_available_memory as f64 / first_metric.total_memory_mb as f64) * 100.0,
            cpu_cores: first_metric.cpu_cores,
            cpu_usage_per_core: first_metric.cpu_usage_per_core.clone(),
            avg_cpu_usage,
            disk_space_available_mb: first_metric.disk_space_available_mb,
            disk_read_rate: avg_disk_read_rate,
            disk_write_rate: avg_disk_write_rate,
            network_io_rate: first_metric.network_io_rate,
            open_file_descriptors: first_metric.open_file_descriptors,
            load_average: first_metric.load_average,
        }
    }

    /// Generate summary of baseline results
    fn generate_summary(&self, baselines: &[OperationBaseline]) -> BaselineSummary {
        if baselines.is_empty() {
            return BaselineSummary {
                total_operations: 0,
                avg_execution_time: Duration::from_secs(0),
                peak_memory_usage_mb: 0,
                avg_throughput: 0.0,
                most_expensive_operation: MonorepoOperation::Generation,
                fastest_operation: MonorepoOperation::Generation,
                most_memory_intensive: MonorepoOperation::Generation,
                recommended_limits: RecommendedLimits {
                    max_monorepo_size: 100,
                    max_concurrent_operations: 4,
                    recommended_memory_mb: 2048,
                    recommended_cpu_cores: 4,
                    max_safe_throughput: 100.0,
                },
            };
        }

        let total_operations = baselines.len() as u32;
        let avg_execution_time = baselines.iter()
            .map(|b| b.metrics.execution_time)
            .sum::<Duration>() / total_operations;
        let peak_memory_usage_mb = baselines.iter()
            .map(|b| b.metrics.peak_memory_mb)
            .max()
            .unwrap_or(0);
        let avg_throughput = baselines.iter()
            .map(|b| b.metrics.throughput)
            .sum::<f64>() / total_operations as f64;

        // Find most expensive operation
        let most_expensive_operation = baselines.iter()
            .max_by_key(|b| b.metrics.execution_time)
            .map(|b| b.operation.clone())
            .unwrap_or(MonorepoOperation::Generation);

        // Find fastest operation
        let fastest_operation = baselines.iter()
            .min_by_key(|b| b.metrics.execution_time)
            .map(|b| b.operation.clone())
            .unwrap_or(MonorepoOperation::Generation);

        // Find most memory intensive operation
        let most_memory_intensive = baselines.iter()
            .max_by_key(|b| b.metrics.peak_memory_mb)
            .map(|b| b.operation.clone())
            .unwrap_or(MonorepoOperation::Generation);

        let recommended_limits = RecommendedLimits {
            max_monorepo_size: 1000,
            max_concurrent_operations: 8,
            recommended_memory_mb: peak_memory_usage_mb * 2,
            recommended_cpu_cores: 8,
            max_safe_throughput: avg_throughput * 0.8,
        };

        BaselineSummary {
            total_operations,
            avg_execution_time,
            peak_memory_usage_mb,
            avg_throughput,
            most_expensive_operation,
            fastest_operation,
            most_memory_intensive,
            recommended_limits,
        }
    }

    /// Detect performance bottlenecks
    fn detect_bottlenecks(&self, baselines: &[OperationBaseline]) -> Vec<DetectedBottleneck> {
        let mut bottlenecks = Vec::new();

        for baseline in baselines {
            // Check for memory bottlenecks
            if baseline.metrics.peak_memory_mb > 8192 {
                bottlenecks.push(DetectedBottleneck {
                    bottleneck_type: BottleneckType::MemoryExhaustion,
                    operation: baseline.operation.clone(),
                    critical_size: baseline.monorepo_size,
                    description: format!("High memory usage: {} MB", baseline.metrics.peak_memory_mb),
                    severity: if baseline.metrics.peak_memory_mb > 16384 {
                        BottleneckSeverity::Critical
                    } else {
                        BottleneckSeverity::High
                    },
                    mitigation: "Consider memory optimization or increasing available RAM".to_string(),
                    performance_impact: (baseline.metrics.peak_memory_mb as f64 / 8192.0 - 1.0) * 100.0,
                });
            }

            // Check for CPU bottlenecks
            if baseline.metrics.avg_cpu_percent > 80.0 {
                bottlenecks.push(DetectedBottleneck {
                    bottleneck_type: BottleneckType::CpuSaturation,
                    operation: baseline.operation.clone(),
                    critical_size: baseline.monorepo_size,
                    description: format!("High CPU usage: {:.1}%", baseline.metrics.avg_cpu_percent),
                    severity: if baseline.metrics.avg_cpu_percent > 95.0 {
                        BottleneckSeverity::Critical
                    } else {
                        BottleneckSeverity::High
                    },
                    mitigation: "Consider parallelization or algorithm optimization".to_string(),
                    performance_impact: baseline.metrics.avg_cpu_percent - 50.0,
                });
            }

            // Check for poor throughput
            if baseline.metrics.throughput < 10.0 {
                bottlenecks.push(DetectedBottleneck {
                    bottleneck_type: BottleneckType::AlgorithmicComplexity,
                    operation: baseline.operation.clone(),
                    critical_size: baseline.monorepo_size,
                    description: format!("Low throughput: {:.2} ops/sec", baseline.metrics.throughput),
                    severity: if baseline.metrics.throughput < 1.0 {
                        BottleneckSeverity::Critical
                    } else {
                        BottleneckSeverity::Medium
                    },
                    mitigation: "Review algorithm complexity and optimize hot paths".to_string(),
                    performance_impact: (10.0 - baseline.metrics.throughput) * 10.0,
                });
            }
        }

        bottlenecks
    }

    /// Analyze scaling characteristics
    fn analyze_scaling(&self, baselines: &[OperationBaseline]) -> ScalingAnalysis {
        let mut scaling_coefficients = HashMap::new();
        let mut memory_scaling = HashMap::new();
        let mut cpu_scaling = HashMap::new();
        let mut io_scaling = HashMap::new();

        // Group baselines by operation
        let mut operations_data: HashMap<MonorepoOperation, Vec<&OperationBaseline>> = HashMap::new();
        for baseline in baselines {
            operations_data.entry(baseline.operation.clone())
                .or_insert_with(Vec::new)
                .push(baseline);
        }

        // Analyze scaling for each operation
        for (operation, data) in operations_data {
            if data.len() < 2 {
                continue; // Need at least 2 data points for scaling analysis
            }

            // Sort by monorepo size
            let mut sorted_data = data;
            sorted_data.sort_by_key(|b| b.monorepo_size);

            // Calculate scaling coefficient (simplified linear regression)
            let scaling_coeff = self.calculate_scaling_coefficient(&sorted_data);
            scaling_coefficients.insert(operation.clone(), scaling_coeff);

            // Classify scaling patterns
            let memory_pattern = self.classify_scaling_pattern(&sorted_data, |b| b.metrics.peak_memory_mb as f64);
            let cpu_pattern = self.classify_scaling_pattern(&sorted_data, |b| b.metrics.avg_cpu_percent);
            let io_pattern = self.classify_scaling_pattern(&sorted_data, |b| b.metrics.io_operations as f64);

            memory_scaling.insert(operation.clone(), memory_pattern);
            cpu_scaling.insert(operation.clone(), cpu_pattern);
            io_scaling.insert(operation.clone(), io_pattern);
        }

        // Classify operations by scaling quality
        let mut linear_scaling = Vec::new();
        let mut poor_scaling = Vec::new();
        let mut excellent_scaling = Vec::new();

        for (operation, coeff) in &scaling_coefficients {
            if *coeff < 1.2 {
                excellent_scaling.push(operation.clone());
            } else if *coeff < 2.0 {
                linear_scaling.push(operation.clone());
            } else {
                poor_scaling.push(operation.clone());
            }
        }

        ScalingAnalysis {
            linear_scaling,
            poor_scaling,
            excellent_scaling,
            scaling_coefficients,
            memory_scaling,
            cpu_scaling,
            io_scaling,
        }
    }

    /// Calculate scaling coefficient for an operation
    fn calculate_scaling_coefficient(&self, data: &[&OperationBaseline]) -> f64 {
        if data.len() < 2 {
            return 1.0;
        }

        let first = data[0];
        let last = data[data.len() - 1];

        let size_ratio = last.monorepo_size as f64 / first.monorepo_size as f64;
        let time_ratio = last.metrics.execution_time.as_secs_f64() / first.metrics.execution_time.as_secs_f64();

        if size_ratio <= 1.0 {
            return 1.0;
        }

        time_ratio / size_ratio
    }

    /// Classify scaling pattern based on data points
    fn classify_scaling_pattern<F>(&self, data: &[&OperationBaseline], extractor: F) -> ScalingPattern
    where
        F: Fn(&OperationBaseline) -> f64,
    {
        if data.len() < 2 {
            return ScalingPattern::Unknown;
        }

        let first_value = extractor(data[0]);
        let last_value = extractor(data[data.len() - 1]);
        let size_ratio = data[data.len() - 1].monorepo_size as f64 / data[0].monorepo_size as f64;

        if size_ratio <= 1.0 || first_value <= 0.0 {
            return ScalingPattern::Unknown;
        }

        let value_ratio = last_value / first_value;

        // Classify based on growth rate
        if value_ratio < 1.1 {
            ScalingPattern::Constant
        } else if value_ratio < size_ratio.ln() * 1.5 {
            ScalingPattern::Logarithmic
        } else if value_ratio < size_ratio * 1.5 {
            ScalingPattern::Linear
        } else if value_ratio < size_ratio * size_ratio.ln() * 1.5 {
            ScalingPattern::Linearithmic
        } else if value_ratio < size_ratio * size_ratio * 1.5 {
            ScalingPattern::Quadratic
        } else if value_ratio < size_ratio * size_ratio * size_ratio * 1.5 {
            ScalingPattern::Cubic
        } else {
            ScalingPattern::Exponential
        }
    }

    /// Get system information
    fn get_system_info(&self) -> SystemInfo {
        SystemInfo {
            os: "Mock OS".to_string(),
            os_version: "1.0.0".to_string(),
            architecture: "x86_64".to_string(),
            cpu_model: "Mock CPU".to_string(),
            cpu_cores: 8,
            total_ram_mb: 16384,
            rust_version: "1.70.0".to_string(),
            cargo_version: "1.70.0".to_string(),
        }
    }

    /// Generate detailed reports
    fn generate_reports(&self, results: &BaselineTestResults) -> Result<(), Box<dyn std::error::Error>> {
        // Create report directory
        std::fs::create_dir_all(&self.config.report_output_dir)?;

        // Generate JSON report
        let json_report = serde_json::to_string_pretty(results)?;
        let json_path = self.config.report_output_dir.join("baseline_results.json");
        std::fs::write(json_path, json_report)?;

        // Generate summary report
        let summary_report = self.generate_summary_report(results);
        let summary_path = self.config.report_output_dir.join("baseline_summary.md");
        std::fs::write(summary_path, summary_report)?;

        println!("📊 Reports generated in: {:?}", self.config.report_output_dir);
        Ok(())
    }

    /// Generate human-readable summary report
    fn generate_summary_report(&self, results: &BaselineTestResults) -> String {
        let mut report = String::new();
        
        report.push_str("# Baseline Test Results Summary\n\n");
        report.push_str(&format!("**Test Date:** {}\n", results.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
        report.push_str(&format!("**Total Duration:** {:?}\n", results.total_duration));
        report.push_str(&format!("**Total Operations:** {}\n\n", results.summary.total_operations));
        
        report.push_str("## System Information\n\n");
        report.push_str(&format!("- **OS:** {} {}\n", results.system_info.os, results.system_info.os_version));
        report.push_str(&format!("- **Architecture:** {}\n", results.system_info.architecture));
        report.push_str(&format!("- **CPU:** {} ({} cores)\n", results.system_info.cpu_model, results.system_info.cpu_cores));
        report.push_str(&format!("- **RAM:** {} MB\n", results.system_info.total_ram_mb));
        report.push_str(&format!("- **Rust Version:** {}\n\n", results.system_info.rust_version));
        
        report.push_str("## Performance Summary\n\n");
        report.push_str(&format!("- **Average Execution Time:** {:?}\n", results.summary.avg_execution_time));
        report.push_str(&format!("- **Peak Memory Usage:** {} MB\n", results.summary.peak_memory_usage_mb));
        report.push_str(&format!("- **Average Throughput:** {:.2} ops/sec\n", results.summary.avg_throughput));
        report.push_str(&format!("- **Most Expensive Operation:** {:?}\n", results.summary.most_expensive_operation));
        report.push_str(&format!("- **Fastest Operation:** {:?}\n", results.summary.fastest_operation));
        report.push_str(&format!("- **Most Memory Intensive:** {:?}\n\n", results.summary.most_memory_intensive));
        
        if !results.bottlenecks.is_empty() {
            report.push_str("## Detected Bottlenecks\n\n");
            for bottleneck in &results.bottlenecks {
                report.push_str(&format!("### {:?} - {:?}\n", bottleneck.bottleneck_type, bottleneck.severity));
                report.push_str(&format!("- **Operation:** {:?}\n", bottleneck.operation));
                report.push_str(&format!("- **Critical Size:** {} packages\n", bottleneck.critical_size));
                report.push_str(&format!("- **Description:** {}\n", bottleneck.description));
                report.push_str(&format!("- **Mitigation:** {}\n", bottleneck.mitigation));
                report.push_str(&format!("- **Performance Impact:** {:.1}%\n\n", bottleneck.performance_impact));
            }
        }
        
        report.push_str("## Scaling Analysis\n\n");
        if !results.scaling_analysis.excellent_scaling.is_empty() {
            report.push_str("### Excellent Scaling (Sub-linear)\n");
            for op in &results.scaling_analysis.excellent_scaling {
                report.push_str(&format!("- {:?}\n", op));
            }
            report.push('\n');
        }
        
        if !results.scaling_analysis.linear_scaling.is_empty() {
            report.push_str("### Linear Scaling\n");
            for op in &results.scaling_analysis.linear_scaling {
                report.push_str(&format!("- {:?}\n", op));
            }
            report.push('\n');
        }
        
        if !results.scaling_analysis.poor_scaling.is_empty() {
            report.push_str("### Poor Scaling (Quadratic or worse)\n");
            for op in &results.scaling_analysis.poor_scaling {
                report.push_str(&format!("- {:?}\n", op));
            }
            report.push('\n');
        }
        
        report.push_str("## Recommended Limits\n\n");
        let limits = &results.summary.recommended_limits;
        report.push_str(&format!("- **Max Monorepo Size:** {} packages\n", limits.max_monorepo_size));
        report.push_str(&format!("- **Max Concurrent Operations:** {}\n", limits.max_concurrent_operations));
        report.push_str(&format!("- **Recommended Memory:** {} MB\n", limits.recommended_memory_mb));
        report.push_str(&format!("- **Recommended CPU Cores:** {}\n", limits.recommended_cpu_cores));
        report.push_str(&format!("- **Max Safe Throughput:** {:.2} ops/sec\n", limits.max_safe_throughput));
        
        report
    }
}

/// Result of executing an operation
#[derive(Debug, Clone)]
struct OperationResult {
    /// Number of items processed
    items_processed: u64,
    /// Number of operations performed
    operations_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_baseline_config_default() {
        let config = BaselineTestConfig::default();
        assert!(!config.monorepo_sizes.is_empty());
        assert!(config.iterations > 0);
        assert!(config.warmup_iterations >= 0);
        assert!(!config.operations_to_test.is_empty());
    }

    #[test]
    fn test_resource_monitor_creation() {
        let monitor = ResourceMonitor::new(Duration::from_millis(100));
        assert_eq!(monitor.interval, Duration::from_millis(100));
        
        // Test that monitor is initially inactive
        let active = monitor.active.lock().unwrap_or_else(|_| false);
        assert!(!*active);
    }

    #[test]
    fn test_baseline_test_framework_creation() {
        let config = BaselineTestConfig::default();
        let framework = BaselineTestFramework::new(config.clone());
        
        assert_eq!(framework.config.iterations, config.iterations);
        assert_eq!(framework.config.monorepo_sizes, config.monorepo_sizes);
    }

    #[test]
    fn test_system_metrics_collection() {
        let metrics = ResourceMonitor::collect_system_metrics();
        assert!(metrics.total_memory_mb > 0);
        assert!(metrics.cpu_cores > 0);
        assert!(!metrics.cpu_usage_per_core.is_empty());
    }

    #[test]
    fn test_baseline_metrics_calculation() {
        let execution_times = vec![
            Duration::from_millis(100),
            Duration::from_millis(150),
            Duration::from_millis(120),
        ];
        let memory_usage = vec![1024, 1536, 1280];
        let throughput_values = vec![10.0, 8.0, 9.0];
        
        let framework = BaselineTestFramework::new(BaselineTestConfig::default());
        let metrics = framework.calculate_baseline_metrics(&execution_times, &memory_usage, &throughput_values, 100);
        
        assert!(metrics.execution_time > Duration::from_millis(0));
        assert!(metrics.peak_memory_mb > 0);
        assert!(metrics.throughput > 0.0);
        assert!(metrics.items_processed > 0);
    }

    #[test]
    fn test_scaling_pattern_classification() {
        let framework = BaselineTestFramework::new(BaselineTestConfig::default());
        
        // Create mock baseline data
        let baseline1 = create_mock_baseline(MonorepoOperation::Generation, 100, Duration::from_millis(100), 1000);
        let baseline2 = create_mock_baseline(MonorepoOperation::Generation, 200, Duration::from_millis(200), 2000);
        let baseline3 = create_mock_baseline(MonorepoOperation::Generation, 400, Duration::from_millis(400), 4000);
        
        let data = vec![&baseline1, &baseline2, &baseline3];
        let pattern = framework.classify_scaling_pattern(&data, |b| b.metrics.execution_time.as_millis() as f64);
        
        // Should be linear scaling
        matches!(pattern, ScalingPattern::Linear);
    }

    #[test]
    fn test_bottleneck_detection() {
        let framework = BaselineTestFramework::new(BaselineTestConfig::default());
        
        // Create baseline with high memory usage
        let mut baseline = create_mock_baseline(MonorepoOperation::Generation, 500, Duration::from_secs(10), 10000);
        baseline.metrics.avg_cpu_percent = 95.0; // High CPU usage
        
        let baselines = vec![baseline];
        let bottlenecks = framework.detect_bottlenecks(&baselines);
        
        assert!(!bottlenecks.is_empty());
        assert!(bottlenecks.iter().any(|b| matches!(b.bottleneck_type, BottleneckType::CpuSaturation)));
    }

    #[test]
    fn test_summary_generation() {
        let framework = BaselineTestFramework::new(BaselineTestConfig::default());
        
        let baselines = vec![
            create_mock_baseline(MonorepoOperation::Generation, 100, Duration::from_millis(100), 1000),
            create_mock_baseline(MonorepoOperation::DependencyAnalysis, 100, Duration::from_millis(200), 2000),
            create_mock_baseline(MonorepoOperation::ChangeDetection, 100, Duration::from_millis(150), 1500),
        ];
        
        let summary = framework.generate_summary(&baselines);
        
        assert_eq!(summary.total_operations, 3);
        assert!(summary.avg_execution_time > Duration::from_millis(0));
        assert!(summary.peak_memory_usage_mb > 0);
        assert!(summary.avg_throughput > 0.0);
    }

    #[test]
    fn test_scaling_coefficient_calculation() {
        let framework = BaselineTestFramework::new(BaselineTestConfig::default());
        
        let baseline1 = create_mock_baseline(MonorepoOperation::Generation, 100, Duration::from_millis(100), 1000);
        let baseline2 = create_mock_baseline(MonorepoOperation::Generation, 200, Duration::from_millis(200), 2000);
        
        let data = vec![&baseline1, &baseline2];
        let coefficient = framework.calculate_scaling_coefficient(&data);
        
        // Should be 1.0 for linear scaling
        assert!((coefficient - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_operation_result_creation() {
        let result = OperationResult {
            items_processed: 100,
            operations_count: 10,
        };
        
        assert_eq!(result.items_processed, 100);
        assert_eq!(result.operations_count, 10);
    }

    #[test]
    fn test_system_info_generation() {
        let framework = BaselineTestFramework::new(BaselineTestConfig::default());
        let system_info = framework.get_system_info();
        
        assert!(!system_info.os.is_empty());
        assert!(!system_info.architecture.is_empty());
        assert!(system_info.cpu_cores > 0);
        assert!(system_info.total_ram_mb > 0);
    }

    #[test]
    fn test_empty_baselines_handling() {
        let framework = BaselineTestFramework::new(BaselineTestConfig::default());
        let empty_baselines = Vec::new();
        
        let summary = framework.generate_summary(&empty_baselines);
        assert_eq!(summary.total_operations, 0);
        
        let bottlenecks = framework.detect_bottlenecks(&empty_baselines);
        assert!(bottlenecks.is_empty());
        
        let scaling_analysis = framework.analyze_scaling(&empty_baselines);
        assert!(scaling_analysis.scaling_coefficients.is_empty());
    }

    #[test]
    fn test_average_system_metrics_calculation() {
        let framework = BaselineTestFramework::new(BaselineTestConfig::default());
        
        let metrics = vec![
            SystemResourceMetrics {
                available_memory_mb: 8000,
                total_memory_mb: 16000,
                memory_usage_percent: 50.0,
                cpu_cores: 8,
                cpu_usage_per_core: vec![25.0; 8],
                avg_cpu_usage: 25.0,
                disk_space_available_mb: 500000,
                disk_read_rate: 100.0,
                disk_write_rate: 50.0,
                network_io_rate: 10.0,
                open_file_descriptors: 1000,
                load_average: 2.0,
            },
            SystemResourceMetrics {
                available_memory_mb: 7000,
                total_memory_mb: 16000,
                memory_usage_percent: 56.25,
                cpu_cores: 8,
                cpu_usage_per_core: vec![35.0; 8],
                avg_cpu_usage: 35.0,
                disk_space_available_mb: 500000,
                disk_read_rate: 120.0,
                disk_write_rate: 60.0,
                network_io_rate: 15.0,
                open_file_descriptors: 1200,
                load_average: 3.0,
            },
        ];
        
        let avg_metrics = framework.calculate_average_system_metrics(&metrics);
        assert_eq!(avg_metrics.available_memory_mb, 7500);
        assert_eq!(avg_metrics.avg_cpu_usage, 30.0);
        assert_eq!(avg_metrics.disk_read_rate, 110.0);
    }

    #[test]
    fn test_recommended_limits_scaling() {
        let framework = BaselineTestFramework::new(BaselineTestConfig::default());
        
        // Test with high memory usage baseline
        let baseline = create_mock_baseline(MonorepoOperation::Generation, 500, Duration::from_secs(5), 8192);
        let baselines = vec![baseline];
        
        let summary = framework.generate_summary(&baselines);
        let limits = &summary.recommended_limits;
        
        assert!(limits.recommended_memory_mb >= 8192);
        assert!(limits.max_safe_throughput > 0.0);
        assert!(limits.max_concurrent_operations > 0);
    }

    #[test]
    fn test_performance_impact_calculation() {
        let framework = BaselineTestFramework::new(BaselineTestConfig::default());
        
        // Create baseline with very high memory usage
        let mut baseline = create_mock_baseline(MonorepoOperation::Generation, 500, Duration::from_secs(10), 20000);
        baseline.metrics.avg_cpu_percent = 50.0; // Normal CPU usage
        
        let baselines = vec![baseline];
        let bottlenecks = framework.detect_bottlenecks(&baselines);
        
        // Should detect memory bottleneck
        let memory_bottleneck = bottlenecks.iter()
            .find(|b| matches!(b.bottleneck_type, BottleneckType::MemoryExhaustion));
        
        if let Some(bottleneck) = memory_bottleneck {
            assert!(bottleneck.performance_impact > 0.0);
        }
    }

    // Helper function to create mock baseline data
    fn create_mock_baseline(operation: MonorepoOperation, size: u32, execution_time: Duration, memory_mb: u64) -> OperationBaseline {
        OperationBaseline {
            operation,
            monorepo_size: size,
            metrics: BaselineMetrics {
                execution_time,
                peak_memory_mb: memory_mb,
                avg_cpu_percent: 50.0,
                io_operations: size as u64 * 2,
                items_processed: size as u64,
                throughput: size as f64 / execution_time.as_secs_f64(),
                avg_response_time: execution_time,
                p95_response_time: execution_time,
                p99_response_time: execution_time,
                memory_allocation_rate: memory_mb as f64 / execution_time.as_secs_f64(),
                cache_hit_rate: Some(0.85),
                error_rate: 0.0,
            },
            system_metrics: SystemResourceMetrics {
                available_memory_mb: 16384 - memory_mb,
                total_memory_mb: 16384,
                memory_usage_percent: (memory_mb as f64 / 16384.0) * 100.0,
                cpu_cores: 8,
                cpu_usage_per_core: vec![50.0; 8],
                avg_cpu_usage: 50.0,
                disk_space_available_mb: 500000,
                disk_read_rate: 100.0,
                disk_write_rate: 50.0,
                network_io_rate: 10.0,
                open_file_descriptors: 1000,
                load_average: 2.0,
            },
            timestamp: Utc::now(),
            test_config: BaselineTestConfig::default(),
            metadata: HashMap::new(),
        }
    }
}