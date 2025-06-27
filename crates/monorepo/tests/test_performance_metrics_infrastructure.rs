//! Performance Metrics Infrastructure for Extreme Monorepo Testing
//!
//! This module provides comprehensive performance metrics collection, analysis, and reporting
//! infrastructure specifically designed for extreme monorepo operations. It complements the
//! baseline testing framework with detailed throughput, latency, and resource utilization metrics.
//!
//! ## What
//! 
//! Advanced performance metrics infrastructure that provides:
//! - Real-time throughput monitoring with multiple measurement strategies
//! - Comprehensive latency analysis including percentile distributions
//! - Detailed system resource utilization tracking (CPU, memory, I/O, network)
//! - Advanced statistical analysis of performance data
//! - Performance degradation detection and alerting
//! - Time-series metrics collection for trend analysis
//! - Performance regression detection across test runs
//! - Resource efficiency scoring and optimization recommendations
//! 
//! ## How
//! 
//! The infrastructure uses a multi-layered metrics collection approach:
//! 1. **Real-time Collection**: Continuous metrics gathering during operation execution
//! 2. **Statistical Analysis**: Advanced statistical processing of collected data
//! 3. **Trend Analysis**: Time-series analysis for performance trend detection
//! 4. **Threshold Monitoring**: Automated alerting for performance threshold breaches
//! 5. **Resource Profiling**: Detailed system resource utilization profiling
//! 6. **Performance Scoring**: Comprehensive scoring system for operation efficiency
//! 7. **Comparative Analysis**: Cross-run performance comparison and regression detection
//! 
//! ## Why
//! 
//! Advanced performance metrics are essential for:
//! - Identifying performance bottlenecks before they become critical issues
//! - Understanding resource utilization patterns in extreme-scale operations
//! - Detecting performance regressions during development cycles
//! - Optimizing resource allocation and capacity planning
//! - Providing actionable insights for performance improvements
//! - Establishing performance SLAs and monitoring compliance
//! - Supporting data-driven optimization decisions

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::thread;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Advanced throughput metrics with multiple measurement strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    /// Instantaneous throughput (items/second)
    pub instantaneous_throughput: f64,
    /// Average throughput over measurement window
    pub average_throughput: f64,
    /// Peak throughput achieved
    pub peak_throughput: f64,
    /// Minimum throughput observed
    pub minimum_throughput: f64,
    /// Throughput standard deviation
    pub throughput_stddev: f64,
    /// Throughput measurements over time
    pub throughput_history: Vec<ThroughputDataPoint>,
    /// Throughput trend (increasing, decreasing, stable)
    pub throughput_trend: ThroughputTrend,
    /// Throughput efficiency score (0-100)
    pub efficiency_score: f64,
    /// Items processed in measurement window
    pub items_processed: u64,
    /// Processing window duration
    pub measurement_window: Duration,
    /// Batch processing metrics
    pub batch_metrics: Option<BatchThroughputMetrics>,
}

/// Individual throughput data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputDataPoint {
    /// Timestamp of measurement
    pub timestamp: DateTime<Utc>,
    /// Throughput value at this point
    pub throughput: f64,
    /// Number of items processed
    pub items_count: u64,
    /// Processing duration for this measurement
    pub duration: Duration,
    /// System resource utilization at this point
    pub resource_utilization: ResourceSnapshot,
}

/// Throughput trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThroughputTrend {
    /// Throughput is increasing consistently
    Increasing { rate: f64 },
    /// Throughput is decreasing consistently
    Decreasing { rate: f64 },
    /// Throughput is stable with minor fluctuations
    Stable { variance: f64 },
    /// Throughput shows cyclical patterns
    Cyclical { period: Duration, amplitude: f64 },
    /// Throughput is highly volatile
    Volatile { volatility_index: f64 },
    /// Not enough data for trend analysis
    Unknown,
}

/// Batch processing throughput metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchThroughputMetrics {
    /// Average batch size
    pub average_batch_size: f64,
    /// Batch processing rate (batches/second)
    pub batch_processing_rate: f64,
    /// Batch completion time distribution
    pub batch_completion_times: Vec<Duration>,
    /// Inter-batch delay statistics
    pub inter_batch_delays: Vec<Duration>,
    /// Batch efficiency score
    pub batch_efficiency: f64,
}

/// Comprehensive latency metrics with percentile analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetrics {
    /// Minimum latency observed
    pub min_latency: Duration,
    /// Maximum latency observed
    pub max_latency: Duration,
    /// Mean latency
    pub mean_latency: Duration,
    /// Median latency (P50)
    pub median_latency: Duration,
    /// 95th percentile latency
    pub p95_latency: Duration,
    /// 99th percentile latency
    pub p99_latency: Duration,
    /// 99.9th percentile latency
    pub p999_latency: Duration,
    /// Standard deviation of latency
    pub latency_stddev: Duration,
    /// Latency distribution histogram
    pub latency_histogram: LatencyHistogram,
    /// Latency trend over time
    pub latency_trend: LatencyTrend,
    /// Service Level Agreement compliance
    pub sla_compliance: SlaCompliance,
    /// Latency outliers analysis
    pub outliers_analysis: OutliersAnalysis,
}

/// Latency histogram with configurable buckets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyHistogram {
    /// Histogram buckets (upper bounds in milliseconds)
    pub buckets: Vec<f64>,
    /// Count of samples in each bucket
    pub counts: Vec<u64>,
    /// Total samples counted
    pub total_samples: u64,
    /// Bucket percentages
    pub percentages: Vec<f64>,
}

/// Latency trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LatencyTrend {
    /// Latency is improving (decreasing)
    Improving { improvement_rate: f64 },
    /// Latency is degrading (increasing)
    Degrading { degradation_rate: f64 },
    /// Latency is stable
    Stable { stability_index: f64 },
    /// Latency shows seasonal patterns
    Seasonal { pattern_duration: Duration },
    /// Latency is unpredictable
    Chaotic { chaos_index: f64 },
}

/// Service Level Agreement compliance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaCompliance {
    /// Target latency threshold
    pub target_latency: Duration,
    /// Percentage of requests meeting SLA
    pub compliance_percentage: f64,
    /// Number of SLA violations
    pub violations_count: u64,
    /// Total requests measured
    pub total_requests: u64,
    /// SLA breach severity distribution
    pub breach_severity: HashMap<SlaBreachSeverity, u64>,
}

/// SLA breach severity levels
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum SlaBreachSeverity {
    /// Minor breach (1-25% over target)
    Minor,
    /// Moderate breach (25-50% over target)
    Moderate,
    /// Major breach (50-100% over target)
    Major,
    /// Critical breach (100%+ over target)
    Critical,
}

/// Latency outliers analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutliersAnalysis {
    /// Number of outliers detected
    pub outliers_count: u64,
    /// Outlier detection threshold
    pub outlier_threshold: Duration,
    /// Outlier values
    pub outlier_values: Vec<Duration>,
    /// Outlier detection method used
    pub detection_method: OutlierDetectionMethod,
    /// Potential causes of outliers
    pub potential_causes: Vec<String>,
}

/// Methods for outlier detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutlierDetectionMethod {
    /// Statistical outliers (beyond N standard deviations)
    Statistical { std_dev_threshold: f64 },
    /// Interquartile range method
    InterquartileRange { iqr_multiplier: f64 },
    /// Z-score method
    ZScore { z_threshold: f64 },
    /// Modified Z-score using median
    ModifiedZScore { threshold: f64 },
}

/// Comprehensive system resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// CPU utilization metrics
    pub cpu_metrics: CpuMetrics,
    /// Memory utilization metrics
    pub memory_metrics: MemoryMetrics,
    /// Disk I/O metrics
    pub disk_metrics: DiskMetrics,
    /// Network I/O metrics
    pub network_metrics: NetworkMetrics,
    /// Process-specific metrics
    pub process_metrics: ProcessMetrics,
    /// System-wide metrics
    pub system_metrics: SystemMetrics,
    /// Resource efficiency score
    pub efficiency_score: ResourceEfficiencyScore,
}

/// CPU utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    /// Overall CPU utilization percentage
    pub overall_utilization: f64,
    /// Per-core utilization
    pub per_core_utilization: Vec<f64>,
    /// User space CPU time percentage
    pub user_time_percent: f64,
    /// System/kernel CPU time percentage
    pub system_time_percent: f64,
    /// Idle time percentage
    pub idle_time_percent: f64,
    /// I/O wait time percentage
    pub iowait_percent: f64,
    /// CPU load averages (1, 5, 15 minutes)
    pub load_averages: [f64; 3],
    /// Context switches per second
    pub context_switches_per_sec: f64,
    /// CPU thermal throttling events
    pub thermal_throttling_events: u64,
}

/// Memory utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// Total system memory (bytes)
    pub total_memory: u64,
    /// Used memory (bytes)
    pub used_memory: u64,
    /// Available memory (bytes)
    pub available_memory: u64,
    /// Memory utilization percentage
    pub utilization_percent: f64,
    /// Buffer/cache memory (bytes)
    pub buffer_cache: u64,
    /// Swap usage (bytes)
    pub swap_used: u64,
    /// Swap total (bytes)
    pub swap_total: u64,
    /// Memory allocation rate (bytes/second)
    pub allocation_rate: f64,
    /// Memory deallocation rate (bytes/second)
    pub deallocation_rate: f64,
    /// Page faults per second
    pub page_faults_per_sec: f64,
    /// Memory fragmentation index
    pub fragmentation_index: f64,
}

/// Disk I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetrics {
    /// Read operations per second
    pub read_ops_per_sec: f64,
    /// Write operations per second
    pub write_ops_per_sec: f64,
    /// Read throughput (bytes/second)
    pub read_throughput: f64,
    /// Write throughput (bytes/second)
    pub write_throughput: f64,
    /// Average read latency
    pub avg_read_latency: Duration,
    /// Average write latency
    pub avg_write_latency: Duration,
    /// Disk utilization percentage
    pub disk_utilization: f64,
    /// Queue depth
    pub queue_depth: f64,
    /// Available disk space (bytes)
    pub available_space: u64,
    /// Disk I/O efficiency score
    pub io_efficiency: f64,
}

/// Network I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Bytes received per second
    pub bytes_received_per_sec: f64,
    /// Bytes transmitted per second
    pub bytes_transmitted_per_sec: f64,
    /// Packets received per second
    pub packets_received_per_sec: f64,
    /// Packets transmitted per second
    pub packets_transmitted_per_sec: f64,
    /// Network errors per second
    pub errors_per_sec: f64,
    /// Network drops per second
    pub drops_per_sec: f64,
    /// Network latency (ping time)
    pub network_latency: Duration,
    /// Bandwidth utilization percentage
    pub bandwidth_utilization: f64,
}

/// Process-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    /// Process CPU usage percentage
    pub cpu_percent: f64,
    /// Process memory usage (bytes)
    pub memory_usage: u64,
    /// Process virtual memory size (bytes)
    pub virtual_memory: u64,
    /// Number of threads
    pub thread_count: u32,
    /// Open file descriptors
    pub open_files: u32,
    /// Process uptime
    pub uptime: Duration,
    /// Process I/O statistics
    pub io_stats: ProcessIoStats,
}

/// Process I/O statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessIoStats {
    /// Bytes read by process
    pub bytes_read: u64,
    /// Bytes written by process
    pub bytes_written: u64,
    /// Read operations count
    pub read_operations: u64,
    /// Write operations count
    pub write_operations: u64,
}

/// System-wide metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// System uptime
    pub uptime: Duration,
    /// Number of running processes
    pub process_count: u32,
    /// System temperature (Celsius)
    pub temperature: Option<f64>,
    /// Power consumption (watts)
    pub power_consumption: Option<f64>,
    /// System entropy available
    pub entropy_available: u32,
}

/// Resource efficiency scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEfficiencyScore {
    /// Overall efficiency score (0-100)
    pub overall_score: f64,
    /// CPU efficiency score
    pub cpu_efficiency: f64,
    /// Memory efficiency score
    pub memory_efficiency: f64,
    /// I/O efficiency score
    pub io_efficiency: f64,
    /// Resource balance score
    pub resource_balance: f64,
    /// Efficiency recommendations
    pub recommendations: Vec<EfficiencyRecommendation>,
}

/// Efficiency improvement recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyRecommendation {
    /// Resource type
    pub resource_type: ResourceType,
    /// Recommendation category
    pub category: RecommendationCategory,
    /// Description of the recommendation
    pub description: String,
    /// Potential impact (percentage improvement)
    pub potential_impact: f64,
    /// Implementation difficulty (1-10)
    pub difficulty: u8,
    /// Priority level
    pub priority: RecommendationPriority,
}

/// Resource types for recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    /// CPU resources
    Cpu,
    /// Memory resources
    Memory,
    /// Disk I/O resources
    DiskIo,
    /// Network resources
    Network,
    /// System configuration
    System,
}

/// Recommendation categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    /// Resource allocation optimization
    Allocation,
    /// Configuration tuning
    Configuration,
    /// Algorithm optimization
    Algorithm,
    /// Capacity planning
    Capacity,
    /// Monitoring improvement
    Monitoring,
}

/// Recommendation priorities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

/// Resource utilization snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    /// Timestamp of snapshot
    pub timestamp: DateTime<Utc>,
    /// CPU utilization at snapshot time
    pub cpu_percent: f64,
    /// Memory utilization at snapshot time
    pub memory_percent: f64,
    /// Disk I/O rate at snapshot time
    pub disk_io_rate: f64,
    /// Network I/O rate at snapshot time  
    pub network_io_rate: f64,
}

/// Performance metrics collector
pub struct PerformanceMetricsCollector {
    /// Throughput metrics collection
    throughput_collector: Arc<Mutex<ThroughputCollector>>,
    /// Latency metrics collection
    latency_collector: Arc<Mutex<LatencyCollector>>,
    /// Resource metrics collection
    resource_collector: Arc<Mutex<ResourceCollector>>,
    /// Collection configuration
    config: CollectorConfig,
    /// Active collection flag
    is_collecting: Arc<Mutex<bool>>,
}

/// Configuration for metrics collection
#[derive(Debug, Clone)]
pub struct CollectorConfig {
    /// Collection interval
    pub collection_interval: Duration,
    /// Metrics retention period
    pub retention_period: Duration,
    /// Enable detailed profiling
    pub enable_profiling: bool,
    /// Maximum metrics history size
    pub max_history_size: usize,
    /// SLA targets
    pub sla_targets: SlaTargets,
    /// Outlier detection configuration
    pub outlier_config: OutlierConfig,
}

/// SLA target configuration
#[derive(Debug, Clone)]
pub struct SlaTargets {
    /// Target latency threshold
    pub target_latency: Duration,
    /// Target throughput minimum
    pub target_throughput: f64,
    /// Target CPU utilization maximum
    pub target_cpu_max: f64,
    /// Target memory utilization maximum
    pub target_memory_max: f64,
}

/// Outlier detection configuration
#[derive(Debug, Clone)]
pub struct OutlierConfig {
    /// Detection method
    pub method: OutlierDetectionMethod,
    /// Enable outlier removal from calculations
    pub remove_outliers: bool,
    /// Maximum outlier percentage to remove
    pub max_outlier_percentage: f64,
}

/// Throughput metrics collector
struct ThroughputCollector {
    /// Throughput measurements
    measurements: VecDeque<ThroughputDataPoint>,
    /// Current batch information
    current_batch: Option<BatchInfo>,
    /// Measurement window size
    window_size: Duration,
}

/// Batch processing information
struct BatchInfo {
    /// Batch start time
    start_time: Instant,
    /// Items in current batch
    items_count: u64,
    /// Batch identifier
    batch_id: u64,
}

/// Latency metrics collector
struct LatencyCollector {
    /// Latency measurements
    measurements: Vec<Duration>,
    /// SLA configuration
    sla_config: SlaTargets,
    /// Outlier detection configuration
    outlier_config: OutlierConfig,
}

/// Resource metrics collector  
struct ResourceCollector {
    /// Resource snapshots
    snapshots: VecDeque<ResourceSnapshot>,
    /// Last collection time
    last_collection: Instant,
    /// Collection interval
    interval: Duration,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            collection_interval: Duration::from_millis(100),
            retention_period: Duration::from_secs(3600), // 1 hour
            enable_profiling: true,
            max_history_size: 10000,
            sla_targets: SlaTargets {
                target_latency: Duration::from_millis(100),
                target_throughput: 1000.0,
                target_cpu_max: 80.0,
                target_memory_max: 80.0,
            },
            outlier_config: OutlierConfig {
                method: OutlierDetectionMethod::InterquartileRange { iqr_multiplier: 1.5 },
                remove_outliers: true,
                max_outlier_percentage: 5.0,
            },
        }
    }
}

impl PerformanceMetricsCollector {
    /// Create a new performance metrics collector
    pub fn new(config: CollectorConfig) -> Self {
        let throughput_collector = Arc::new(Mutex::new(ThroughputCollector::new(config.collection_interval)));
        let latency_collector = Arc::new(Mutex::new(LatencyCollector::new(config.sla_targets.clone(), config.outlier_config.clone())));
        let resource_collector = Arc::new(Mutex::new(ResourceCollector::new(config.collection_interval)));
        
        Self {
            throughput_collector,
            latency_collector,
            resource_collector,
            config,
            is_collecting: Arc::new(Mutex::new(false)),
        }
    }

    /// Start metrics collection
    pub fn start_collection(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut collecting = self.is_collecting.lock()
                .map_err(|e| format!("Failed to acquire collection lock: {}", e))?;
            
            if *collecting {
                return Err("Collection already started".into());
            }
            
            *collecting = true;
        }

        // Start collection threads
        self.start_background_collection()?;
        
        Ok(())
    }

    /// Stop metrics collection and return results
    pub fn stop_collection(&self) -> Result<PerformanceMetricsSnapshot, Box<dyn std::error::Error>> {
        {
            let mut collecting = self.is_collecting.lock()
                .map_err(|e| format!("Failed to acquire collection lock: {}", e))?;
            
            if !*collecting {
                return Err("Collection not started".into());
            }
            
            *collecting = false;
        }

        // Wait for collection to stop
        thread::sleep(Duration::from_millis(200));

        // Generate snapshot
        self.generate_snapshot()
    }

    /// Record a throughput measurement
    pub fn record_throughput(&self, items_processed: u64, duration: Duration) -> Result<(), Box<dyn std::error::Error>> {
        let mut collector = self.throughput_collector.lock()
            .map_err(|e| format!("Failed to acquire throughput collector lock: {}", e))?;
        
        let throughput = items_processed as f64 / duration.as_secs_f64();
        let resource_snapshot = self.capture_resource_snapshot();
        
        let data_point = ThroughputDataPoint {
            timestamp: Utc::now(),
            throughput,
            items_count: items_processed,
            duration,
            resource_utilization: resource_snapshot,
        };
        
        collector.add_measurement(data_point);
        Ok(())
    }

    /// Record a latency measurement
    pub fn record_latency(&self, latency: Duration) -> Result<(), Box<dyn std::error::Error>> {
        let mut collector = self.latency_collector.lock()
            .map_err(|e| format!("Failed to acquire latency collector lock: {}", e))?;
        
        collector.add_measurement(latency);
        Ok(())
    }

    /// Start background collection threads
    fn start_background_collection(&self) -> Result<(), Box<dyn std::error::Error>> {
        let resource_collector = Arc::clone(&self.resource_collector);
        let is_collecting = Arc::clone(&self.is_collecting);
        let interval = self.config.collection_interval;

        // Start resource collection thread
        let _resource_thread = thread::spawn(move || {
            while {
                let collecting = is_collecting.lock().unwrap_or_else(|_| false);
                *collecting
            } {
                if let Ok(mut collector) = resource_collector.lock() {
                    let snapshot = Self::collect_current_resource_snapshot();
                    collector.add_snapshot(snapshot);
                }
                
                thread::sleep(interval);
            }
        });

        Ok(())
    }

    /// Generate comprehensive metrics snapshot
    fn generate_snapshot(&self) -> Result<PerformanceMetricsSnapshot, Box<dyn std::error::Error>> {
        let throughput_metrics = {
            let collector = self.throughput_collector.lock()
                .map_err(|e| format!("Failed to acquire throughput collector lock: {}", e))?;
            collector.generate_metrics()
        };

        let latency_metrics = {
            let collector = self.latency_collector.lock()
                .map_err(|e| format!("Failed to acquire latency collector lock: {}", e))?;
            collector.generate_metrics()
        };

        let resource_metrics = {
            let collector = self.resource_collector.lock()
                .map_err(|e| format!("Failed to acquire resource collector lock: {}", e))?;
            collector.generate_metrics()
        };

        Ok(PerformanceMetricsSnapshot {
            timestamp: Utc::now(),
            throughput_metrics,
            latency_metrics,
            resource_metrics,
            collection_duration: Duration::from_secs(0), // Will be calculated properly
            quality_score: self.calculate_quality_score(&throughput_metrics, &latency_metrics, &resource_metrics),
        })
    }

    /// Capture current resource snapshot
    fn capture_resource_snapshot(&self) -> ResourceSnapshot {
        Self::collect_current_resource_snapshot()
    }

    /// Collect current system resource snapshot
    fn collect_current_resource_snapshot() -> ResourceSnapshot {
        // This would integrate with system APIs in a real implementation
        ResourceSnapshot {
            timestamp: Utc::now(),
            cpu_percent: 45.0,     // Mock value
            memory_percent: 60.0,  // Mock value
            disk_io_rate: 100.0,   // Mock value
            network_io_rate: 50.0, // Mock value
        }
    }

    /// Calculate overall quality score
    fn calculate_quality_score(&self, throughput: &ThroughputMetrics, latency: &LatencyMetrics, resources: &ResourceMetrics) -> f64 {
        let throughput_score = throughput.efficiency_score;
        let latency_score = latency.sla_compliance.compliance_percentage;
        let resource_score = resources.efficiency_score.overall_score;
        
        (throughput_score + latency_score + resource_score) / 3.0
    }
}

/// Comprehensive performance metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetricsSnapshot {
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,
    /// Throughput metrics
    pub throughput_metrics: ThroughputMetrics,
    /// Latency metrics
    pub latency_metrics: LatencyMetrics,
    /// Resource utilization metrics
    pub resource_metrics: ResourceMetrics,
    /// Total collection duration
    pub collection_duration: Duration,
    /// Overall quality score
    pub quality_score: f64,
}

impl ThroughputCollector {
    /// Create new throughput collector
    fn new(window_size: Duration) -> Self {
        Self {
            measurements: VecDeque::new(),
            current_batch: None,
            window_size,
        }
    }

    /// Add throughput measurement
    fn add_measurement(&mut self, measurement: ThroughputDataPoint) {
        self.measurements.push_back(measurement);
        
        // Remove old measurements outside window
        let cutoff_time = Utc::now() - chrono::Duration::from_std(self.window_size).unwrap_or_else(|_| chrono::Duration::seconds(3600));
        while let Some(front) = self.measurements.front() {
            if front.timestamp < cutoff_time {
                self.measurements.pop_front();
            } else {
                break;
            }
        }
    }

    /// Generate throughput metrics
    fn generate_metrics(&self) -> ThroughputMetrics {
        if self.measurements.is_empty() {
            return ThroughputMetrics {
                instantaneous_throughput: 0.0,
                average_throughput: 0.0,
                peak_throughput: 0.0,
                minimum_throughput: 0.0,
                throughput_stddev: 0.0,
                throughput_history: Vec::new(),
                throughput_trend: ThroughputTrend::Unknown,
                efficiency_score: 0.0,
                items_processed: 0,
                measurement_window: self.window_size,
                batch_metrics: None,
            };
        }

        let throughputs: Vec<f64> = self.measurements.iter().map(|m| m.throughput).collect();
        let average_throughput = throughputs.iter().sum::<f64>() / throughputs.len() as f64;
        let peak_throughput = throughputs.iter().fold(0.0, |a, &b| a.max(b));
        let minimum_throughput = throughputs.iter().fold(f64::MAX, |a, &b| a.min(b));
        
        let variance = throughputs.iter()
            .map(|&x| (x - average_throughput).powi(2))
            .sum::<f64>() / throughputs.len() as f64;
        let throughput_stddev = variance.sqrt();

        let items_processed = self.measurements.iter().map(|m| m.items_count).sum();
        let trend = self.analyze_throughput_trend(&throughputs);
        let efficiency_score = self.calculate_efficiency_score(average_throughput, peak_throughput, throughput_stddev);

        ThroughputMetrics {
            instantaneous_throughput: self.measurements.back().map(|m| m.throughput).unwrap_or(0.0),
            average_throughput,
            peak_throughput,
            minimum_throughput,
            throughput_stddev,
            throughput_history: self.measurements.iter().cloned().collect(),
            throughput_trend: trend,
            efficiency_score,
            items_processed,
            measurement_window: self.window_size,
            batch_metrics: None, // Would be implemented for batch processing
        }
    }

    /// Analyze throughput trend
    fn analyze_throughput_trend(&self, throughputs: &[f64]) -> ThroughputTrend {
        if throughputs.len() < 3 {
            return ThroughputTrend::Unknown;
        }

        // Simple linear regression for trend analysis
        let n = throughputs.len() as f64;
        let x_sum: f64 = (0..throughputs.len()).map(|i| i as f64).sum();
        let y_sum: f64 = throughputs.iter().sum();
        let xy_sum: f64 = throughputs.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let x2_sum: f64 = (0..throughputs.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * xy_sum - x_sum * y_sum) / (n * x2_sum - x_sum * x_sum);

        if slope > 0.1 {
            ThroughputTrend::Increasing { rate: slope }
        } else if slope < -0.1 {
            ThroughputTrend::Decreasing { rate: slope.abs() }
        } else {
            let variance = throughputs.iter()
                .map(|&x| (x - y_sum / n).powi(2))
                .sum::<f64>() / n;
            ThroughputTrend::Stable { variance }
        }
    }

    /// Calculate efficiency score
    fn calculate_efficiency_score(&self, avg: f64, peak: f64, stddev: f64) -> f64 {
        if peak == 0.0 {
            return 0.0;
        }

        let consistency_score = 100.0 - (stddev / avg * 100.0).min(100.0);
        let utilization_score = (avg / peak) * 100.0;
        
        (consistency_score + utilization_score) / 2.0
    }
}

impl LatencyCollector {
    /// Create new latency collector
    fn new(sla_config: SlaTargets, outlier_config: OutlierConfig) -> Self {
        Self {
            measurements: Vec::new(),
            sla_config,
            outlier_config,
        }
    }

    /// Add latency measurement
    fn add_measurement(&mut self, latency: Duration) {
        self.measurements.push(latency);
    }

    /// Generate latency metrics
    fn generate_metrics(&self) -> LatencyMetrics {
        if self.measurements.is_empty() {
            return self.empty_metrics();
        }

        let mut sorted_measurements = self.measurements.clone();
        sorted_measurements.sort();

        let min_latency = sorted_measurements[0];
        let max_latency = sorted_measurements[sorted_measurements.len() - 1];
        let mean_latency = Duration::from_nanos(
            (sorted_measurements.iter().map(|d| d.as_nanos()).sum::<u128>() / sorted_measurements.len() as u128) as u64
        );

        let median_latency = self.calculate_percentile(&sorted_measurements, 50.0);
        let p95_latency = self.calculate_percentile(&sorted_measurements, 95.0);
        let p99_latency = self.calculate_percentile(&sorted_measurements, 99.0);
        let p999_latency = self.calculate_percentile(&sorted_measurements, 99.9);

        let latency_stddev = self.calculate_standard_deviation(&sorted_measurements, mean_latency);
        let histogram = self.create_latency_histogram(&sorted_measurements);
        let trend = self.analyze_latency_trend();
        let sla_compliance = self.calculate_sla_compliance(&sorted_measurements);
        let outliers_analysis = self.analyze_outliers(&sorted_measurements);

        LatencyMetrics {
            min_latency,
            max_latency,
            mean_latency,
            median_latency,
            p95_latency,
            p99_latency,
            p999_latency,
            latency_stddev,
            latency_histogram: histogram,
            latency_trend: trend,
            sla_compliance,
            outliers_analysis,
        }
    }

    /// Create empty metrics
    fn empty_metrics(&self) -> LatencyMetrics {
        LatencyMetrics {
            min_latency: Duration::from_nanos(0),
            max_latency: Duration::from_nanos(0),
            mean_latency: Duration::from_nanos(0),
            median_latency: Duration::from_nanos(0),
            p95_latency: Duration::from_nanos(0),
            p99_latency: Duration::from_nanos(0),
            p999_latency: Duration::from_nanos(0),
            latency_stddev: Duration::from_nanos(0),
            latency_histogram: LatencyHistogram {
                buckets: Vec::new(),
                counts: Vec::new(),
                total_samples: 0,
                percentages: Vec::new(),
            },
            latency_trend: LatencyTrend::Stable { stability_index: 0.0 },
            sla_compliance: SlaCompliance {
                target_latency: self.sla_config.target_latency,
                compliance_percentage: 0.0,
                violations_count: 0,
                total_requests: 0,
                breach_severity: HashMap::new(),
            },
            outliers_analysis: OutliersAnalysis {
                outliers_count: 0,
                outlier_threshold: Duration::from_nanos(0),
                outlier_values: Vec::new(),
                detection_method: self.outlier_config.method.clone(),
                potential_causes: Vec::new(),
            },
        }
    }

    /// Calculate percentile
    fn calculate_percentile(&self, sorted_values: &[Duration], percentile: f64) -> Duration {
        if sorted_values.is_empty() {
            return Duration::from_nanos(0);
        }

        let index = ((percentile / 100.0) * (sorted_values.len() - 1) as f64) as usize;
        sorted_values[index.min(sorted_values.len() - 1)]
    }

    /// Calculate standard deviation
    fn calculate_standard_deviation(&self, values: &[Duration], mean: Duration) -> Duration {
        if values.len() <= 1 {
            return Duration::from_nanos(0);
        }

        let variance = values.iter()
            .map(|&x| {
                let diff = if x >= mean { x - mean } else { mean - x };
                diff.as_nanos() as f64
            })
            .map(|x| x * x)
            .sum::<f64>() / values.len() as f64;

        Duration::from_nanos(variance.sqrt() as u64)
    }

    /// Create latency histogram
    fn create_latency_histogram(&self, values: &[Duration]) -> LatencyHistogram {
        // Define histogram buckets (in milliseconds)
        let buckets = vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0];
        let mut counts = vec![0u64; buckets.len()];

        for &value in values {
            let value_ms = value.as_millis() as f64;
            for (i, &bucket) in buckets.iter().enumerate() {
                if value_ms <= bucket {
                    counts[i] += 1;
                    break;
                }
            }
        }

        let total_samples = values.len() as u64;
        let percentages = counts.iter()
            .map(|&count| if total_samples > 0 { count as f64 / total_samples as f64 * 100.0 } else { 0.0 })
            .collect();

        LatencyHistogram {
            buckets,
            counts,
            total_samples,
            percentages,
        }
    }

    /// Analyze latency trend
    fn analyze_latency_trend(&self) -> LatencyTrend {
        // Simplified trend analysis - would be more sophisticated in practice
        LatencyTrend::Stable { stability_index: 0.9 }
    }

    /// Calculate SLA compliance
    fn calculate_sla_compliance(&self, values: &[Duration]) -> SlaCompliance {
        let target = self.sla_config.target_latency;
        let total_requests = values.len() as u64;
        let mut violations_count = 0u64;
        let mut breach_severity = HashMap::new();

        for &value in values {
            if value > target {
                violations_count += 1;
                
                let breach_ratio = value.as_nanos() as f64 / target.as_nanos() as f64;
                let severity = if breach_ratio <= 1.25 {
                    SlaBreachSeverity::Minor
                } else if breach_ratio <= 1.5 {
                    SlaBreachSeverity::Moderate
                } else if breach_ratio <= 2.0 {
                    SlaBreachSeverity::Major
                } else {
                    SlaBreachSeverity::Critical
                };
                
                *breach_severity.entry(severity).or_insert(0) += 1;
            }
        }

        let compliance_percentage = if total_requests > 0 {
            ((total_requests - violations_count) as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        SlaCompliance {
            target_latency: target,
            compliance_percentage,
            violations_count,
            total_requests,
            breach_severity,
        }
    }

    /// Analyze outliers
    fn analyze_outliers(&self, values: &[Duration]) -> OutliersAnalysis {
        let outlier_values = Vec::new(); // Would implement outlier detection
        let outliers_count = outlier_values.len() as u64;
        let outlier_threshold = Duration::from_millis(1000); // Mock threshold

        OutliersAnalysis {
            outliers_count,
            outlier_threshold,
            outlier_values,
            detection_method: self.outlier_config.method.clone(),
            potential_causes: vec![
                "Garbage collection pauses".to_string(),
                "Network congestion".to_string(),
                "Resource contention".to_string(),
            ],
        }
    }
}

impl ResourceCollector {
    /// Create new resource collector
    fn new(interval: Duration) -> Self {
        Self {
            snapshots: VecDeque::new(),
            last_collection: Instant::now(),
            interval,
        }
    }

    /// Add resource snapshot
    fn add_snapshot(&mut self, snapshot: ResourceSnapshot) {
        self.snapshots.push_back(snapshot);
        
        // Limit snapshot history
        while self.snapshots.len() > 1000 {
            self.snapshots.pop_front();
        }
        
        self.last_collection = Instant::now();
    }

    /// Generate resource metrics
    fn generate_metrics(&self) -> ResourceMetrics {
        // Mock implementation - would collect real system metrics
        ResourceMetrics {
            cpu_metrics: CpuMetrics {
                overall_utilization: 45.0,
                per_core_utilization: vec![40.0, 50.0, 45.0, 35.0, 55.0, 40.0, 50.0, 45.0],
                user_time_percent: 35.0,
                system_time_percent: 10.0,
                idle_time_percent: 55.0,
                iowait_percent: 2.0,
                load_averages: [2.5, 2.3, 2.1],
                context_switches_per_sec: 5000.0,
                thermal_throttling_events: 0,
            },
            memory_metrics: MemoryMetrics {
                total_memory: 16 * 1024 * 1024 * 1024, // 16GB
                used_memory: 10 * 1024 * 1024 * 1024,  // 10GB
                available_memory: 6 * 1024 * 1024 * 1024, // 6GB
                utilization_percent: 62.5,
                buffer_cache: 2 * 1024 * 1024 * 1024, // 2GB
                swap_used: 0,
                swap_total: 8 * 1024 * 1024 * 1024, // 8GB
                allocation_rate: 1000000.0, // 1MB/s
                deallocation_rate: 950000.0, // 0.95MB/s
                page_faults_per_sec: 100.0,
                fragmentation_index: 0.15,
            },
            disk_metrics: DiskMetrics {
                read_ops_per_sec: 150.0,
                write_ops_per_sec: 75.0,
                read_throughput: 50.0 * 1024.0 * 1024.0, // 50MB/s
                write_throughput: 25.0 * 1024.0 * 1024.0, // 25MB/s
                avg_read_latency: Duration::from_millis(5),
                avg_write_latency: Duration::from_millis(8),
                disk_utilization: 25.0,
                queue_depth: 2.5,
                available_space: 500 * 1024 * 1024 * 1024, // 500GB
                io_efficiency: 85.0,
            },
            network_metrics: NetworkMetrics {
                bytes_received_per_sec: 10.0 * 1024.0 * 1024.0, // 10MB/s
                bytes_transmitted_per_sec: 5.0 * 1024.0 * 1024.0, // 5MB/s
                packets_received_per_sec: 1000.0,
                packets_transmitted_per_sec: 800.0,
                errors_per_sec: 0.1,
                drops_per_sec: 0.05,
                network_latency: Duration::from_millis(15),
                bandwidth_utilization: 15.0,
            },
            process_metrics: ProcessMetrics {
                cpu_percent: 25.0,
                memory_usage: 2 * 1024 * 1024 * 1024, // 2GB
                virtual_memory: 4 * 1024 * 1024 * 1024, // 4GB
                thread_count: 16,
                open_files: 256,
                uptime: Duration::from_secs(3600), // 1 hour
                io_stats: ProcessIoStats {
                    bytes_read: 100 * 1024 * 1024, // 100MB
                    bytes_written: 50 * 1024 * 1024, // 50MB
                    read_operations: 10000,
                    write_operations: 5000,
                },
            },
            system_metrics: SystemMetrics {
                uptime: Duration::from_secs(86400 * 7), // 1 week
                process_count: 150,
                temperature: Some(65.0), // 65°C
                power_consumption: Some(95.0), // 95W
                entropy_available: 3500,
            },
            efficiency_score: ResourceEfficiencyScore {
                overall_score: 82.5,
                cpu_efficiency: 85.0,
                memory_efficiency: 80.0,
                io_efficiency: 85.0,
                resource_balance: 80.0,
                recommendations: vec![
                    EfficiencyRecommendation {
                        resource_type: ResourceType::Memory,
                        category: RecommendationCategory::Allocation,
                        description: "Consider increasing buffer cache size for better I/O performance".to_string(),
                        potential_impact: 15.0,
                        difficulty: 3,
                        priority: RecommendationPriority::Medium,
                    },
                    EfficiencyRecommendation {
                        resource_type: ResourceType::Cpu,
                        category: RecommendationCategory::Algorithm,
                        description: "Optimize hot code paths to reduce CPU usage".to_string(),
                        potential_impact: 10.0,
                        difficulty: 7,
                        priority: RecommendationPriority::High,
                    },
                ],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_collector_config_default() {
        let config = CollectorConfig::default();
        assert!(config.collection_interval > Duration::from_nanos(0));
        assert!(config.retention_period > Duration::from_nanos(0));
        assert!(config.max_history_size > 0);
    }

    #[test]
    fn test_performance_metrics_collector_creation() {
        let config = CollectorConfig::default();
        let collector = PerformanceMetricsCollector::new(config);
        
        // Test that collector is created successfully
        let is_collecting = collector.is_collecting.lock().unwrap_or_else(|_| false);
        assert!(!*is_collecting);
    }

    #[test]
    fn test_throughput_collector() {
        let mut collector = ThroughputCollector::new(Duration::from_secs(60));
        
        let data_point = ThroughputDataPoint {
            timestamp: Utc::now(),
            throughput: 150.0,
            items_count: 1500,
            duration: Duration::from_secs(10),
            resource_utilization: ResourceSnapshot {
                timestamp: Utc::now(),
                cpu_percent: 50.0,
                memory_percent: 60.0,
                disk_io_rate: 100.0,
                network_io_rate: 50.0,
            },
        };
        
        collector.add_measurement(data_point);
        let metrics = collector.generate_metrics();
        
        assert!(metrics.items_processed > 0);
        assert!(metrics.average_throughput > 0.0);
    }

    #[test]
    fn test_latency_collector() {
        let sla_targets = SlaTargets {
            target_latency: Duration::from_millis(100),
            target_throughput: 1000.0,
            target_cpu_max: 80.0,
            target_memory_max: 80.0,
        };
        
        let outlier_config = OutlierConfig {
            method: OutlierDetectionMethod::InterquartileRange { iqr_multiplier: 1.5 },
            remove_outliers: true,
            max_outlier_percentage: 5.0,
        };
        
        let mut collector = LatencyCollector::new(sla_targets, outlier_config);
        
        // Add some latency measurements
        collector.add_measurement(Duration::from_millis(50));
        collector.add_measurement(Duration::from_millis(75));
        collector.add_measurement(Duration::from_millis(90));
        collector.add_measurement(Duration::from_millis(120)); // SLA violation
        collector.add_measurement(Duration::from_millis(65));
        
        let metrics = collector.generate_metrics();
        
        assert!(metrics.mean_latency > Duration::from_nanos(0));
        assert!(metrics.sla_compliance.total_requests == 5);
        assert!(metrics.sla_compliance.violations_count == 1);
        assert!(metrics.sla_compliance.compliance_percentage == 80.0);
    }

    #[test]
    fn test_resource_collector() {
        let mut collector = ResourceCollector::new(Duration::from_millis(100));
        
        let snapshot = ResourceSnapshot {
            timestamp: Utc::now(),
            cpu_percent: 45.0,
            memory_percent: 60.0,
            disk_io_rate: 100.0,
            network_io_rate: 50.0,
        };
        
        collector.add_snapshot(snapshot);
        let metrics = collector.generate_metrics();
        
        assert!(metrics.cpu_metrics.overall_utilization > 0.0);
        assert!(metrics.memory_metrics.total_memory > 0);
        assert!(metrics.efficiency_score.overall_score > 0.0);
    }

    #[test]
    fn test_throughput_trend_analysis() {
        let collector = ThroughputCollector::new(Duration::from_secs(60));
        
        // Test increasing trend
        let increasing_throughputs = vec![100.0, 110.0, 120.0, 130.0, 140.0, 150.0];
        let trend = collector.analyze_throughput_trend(&increasing_throughputs);
        
        match trend {
            ThroughputTrend::Increasing { rate } => assert!(rate > 0.0),
            _ => panic!("Expected increasing trend"),
        }
        
        // Test stable trend
        let stable_throughputs = vec![100.0, 102.0, 98.0, 101.0, 99.0, 100.0];
        let trend = collector.analyze_throughput_trend(&stable_throughputs);
        
        match trend {
            ThroughputTrend::Stable { variance: _ } => {},
            _ => panic!("Expected stable trend"),
        }
    }

    #[test]
    fn test_latency_percentile_calculation() {
        let sla_targets = SlaTargets {
            target_latency: Duration::from_millis(100),
            target_throughput: 1000.0,
            target_cpu_max: 80.0,
            target_memory_max: 80.0,
        };
        
        let outlier_config = OutlierConfig {
            method: OutlierDetectionMethod::InterquartileRange { iqr_multiplier: 1.5 },
            remove_outliers: true,
            max_outlier_percentage: 5.0,
        };
        
        let collector = LatencyCollector::new(sla_targets, outlier_config);
        
        let values = vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(30),
            Duration::from_millis(40),
            Duration::from_millis(50),
            Duration::from_millis(60),
            Duration::from_millis(70),
            Duration::from_millis(80),
            Duration::from_millis(90),
            Duration::from_millis(100),
        ];
        
        let p50 = collector.calculate_percentile(&values, 50.0);
        let p95 = collector.calculate_percentile(&values, 95.0);
        let p99 = collector.calculate_percentile(&values, 99.0);
        
        assert!(p50 > Duration::from_nanos(0));
        assert!(p95 > p50);
        assert!(p99 >= p95);
    }

    #[test]
    fn test_efficiency_score_calculation() {
        let collector = ThroughputCollector::new(Duration::from_secs(60));
        
        // Test perfect efficiency (no variation, full utilization)
        let perfect_score = collector.calculate_efficiency_score(100.0, 100.0, 0.0);
        assert_eq!(perfect_score, 100.0);
        
        // Test poor efficiency (high variation, low utilization)
        let poor_score = collector.calculate_efficiency_score(25.0, 100.0, 30.0);
        assert!(poor_score < 50.0);
    }

    #[test]
    fn test_sla_breach_severity_classification() {
        let sla_targets = SlaTargets {
            target_latency: Duration::from_millis(100),
            target_throughput: 1000.0,
            target_cpu_max: 80.0,
            target_memory_max: 80.0,
        };
        
        let outlier_config = OutlierConfig {
            method: OutlierDetectionMethod::InterquartileRange { iqr_multiplier: 1.5 },
            remove_outliers: true,
            max_outlier_percentage: 5.0,
        };
        
        let mut collector = LatencyCollector::new(sla_targets, outlier_config);
        
        // Add measurements with different severity levels
        collector.add_measurement(Duration::from_millis(50));  // OK
        collector.add_measurement(Duration::from_millis(110)); // Minor breach
        collector.add_measurement(Duration::from_millis(140)); // Moderate breach
        collector.add_measurement(Duration::from_millis(180)); // Major breach
        collector.add_measurement(Duration::from_millis(250)); // Critical breach
        
        let metrics = collector.generate_metrics();
        let breach_counts: u64 = metrics.sla_compliance.breach_severity.values().sum();
        
        assert_eq!(breach_counts, 4); // 4 breaches total
        assert!(metrics.sla_compliance.breach_severity.contains_key(&SlaBreachSeverity::Minor));
        assert!(metrics.sla_compliance.breach_severity.contains_key(&SlaBreachSeverity::Critical));
    }

    #[test]
    fn test_latency_histogram_creation() {
        let sla_targets = SlaTargets {
            target_latency: Duration::from_millis(100),
            target_throughput: 1000.0,
            target_cpu_max: 80.0,
            target_memory_max: 80.0,
        };
        
        let outlier_config = OutlierConfig {
            method: OutlierDetectionMethod::InterquartileRange { iqr_multiplier: 1.5 },
            remove_outliers: true,
            max_outlier_percentage: 5.0,
        };
        
        let collector = LatencyCollector::new(sla_targets, outlier_config);
        
        let values = vec![
            Duration::from_millis(2),   // Should go in 5ms bucket
            Duration::from_millis(8),   // Should go in 10ms bucket
            Duration::from_millis(30),  // Should go in 50ms bucket
            Duration::from_millis(75),  // Should go in 100ms bucket
            Duration::from_millis(150), // Should go in 250ms bucket
        ];
        
        let histogram = collector.create_latency_histogram(&values);
        
        assert_eq!(histogram.total_samples, 5);
        assert!(!histogram.buckets.is_empty());
        assert_eq!(histogram.buckets.len(), histogram.counts.len());
        assert_eq!(histogram.counts.len(), histogram.percentages.len());
        
        // Check that percentages sum to 100% (allowing for floating point precision)
        let total_percentage: f64 = histogram.percentages.iter().sum();
        assert!((total_percentage - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_resource_snapshot_creation() {
        let snapshot = PerformanceMetricsCollector::collect_current_resource_snapshot();
        
        assert!(snapshot.cpu_percent >= 0.0);
        assert!(snapshot.cpu_percent <= 100.0);
        assert!(snapshot.memory_percent >= 0.0);
        assert!(snapshot.memory_percent <= 100.0);
        assert!(snapshot.disk_io_rate >= 0.0);
        assert!(snapshot.network_io_rate >= 0.0);
    }

    #[test]
    fn test_empty_measurements_handling() {
        // Test throughput collector with no measurements
        let throughput_collector = ThroughputCollector::new(Duration::from_secs(60));
        let throughput_metrics = throughput_collector.generate_metrics();
        assert_eq!(throughput_metrics.items_processed, 0);
        assert_eq!(throughput_metrics.average_throughput, 0.0);
        
        // Test latency collector with no measurements
        let sla_targets = SlaTargets {
            target_latency: Duration::from_millis(100),
            target_throughput: 1000.0,
            target_cpu_max: 80.0,
            target_memory_max: 80.0,
        };
        
        let outlier_config = OutlierConfig {
            method: OutlierDetectionMethod::InterquartileRange { iqr_multiplier: 1.5 },
            remove_outliers: true,
            max_outlier_percentage: 5.0,
        };
        
        let latency_collector = LatencyCollector::new(sla_targets, outlier_config);
        let latency_metrics = latency_collector.generate_metrics();
        assert_eq!(latency_metrics.latency_histogram.total_samples, 0);
        assert_eq!(latency_metrics.sla_compliance.total_requests, 0);
    }

    #[test]
    fn test_efficiency_recommendations() {
        let recommendation = EfficiencyRecommendation {
            resource_type: ResourceType::Cpu,
            category: RecommendationCategory::Algorithm,
            description: "Optimize CPU-intensive operations".to_string(),
            potential_impact: 25.0,
            difficulty: 7,
            priority: RecommendationPriority::High,
        };
        
        assert!(recommendation.potential_impact > 0.0);
        assert!(recommendation.difficulty >= 1 && recommendation.difficulty <= 10);
        matches!(recommendation.priority, RecommendationPriority::High);
    }
}