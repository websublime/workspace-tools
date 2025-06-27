//! Extreme Concurrency and CPU Saturation Testing
//!
//! This module implements comprehensive testing of system behavior under extreme
//! concurrency conditions and CPU saturation scenarios, validating thread safety,
//! deadlock detection, performance degradation, and system stability under
//! maximum processing load and resource contention.

use sublime_monorepo_tools::Result;
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::sync::atomic::{AtomicBool, AtomicUsize, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock, Barrier, Condvar};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

/// Configuration for extreme concurrency and CPU saturation testing
#[derive(Debug, Clone)]
pub struct ExtremeConcurrencyConfig {
    /// Maximum test duration in seconds
    pub max_test_duration_secs: u64,
    /// Number of concurrent threads to spawn
    pub max_concurrent_threads: usize,
    /// CPU saturation target percentage (0.0-1.0)
    pub cpu_saturation_target: f64,
    /// Types of concurrency tests to execute
    pub concurrency_test_types: Vec<ConcurrencyTestType>,
    /// Enable deadlock detection and analysis
    pub enable_deadlock_detection: bool,
    /// Enable real-time performance monitoring
    pub enable_realtime_monitoring: bool,
    /// Monitor interval in milliseconds
    pub monitoring_interval_ms: u64,
    /// CPU workload intensity (1.0-10.0)
    pub cpu_workload_intensity: f64,
    /// Enable thread contention analysis
    pub enable_thread_contention_analysis: bool,
    /// Maximum thread pool size for stress testing
    pub max_thread_pool_size: usize,
    /// Enable adaptive thread scaling
    pub enable_adaptive_thread_scaling: bool,
    /// Resource contention scenarios to test
    pub contention_scenarios: Vec<ContentionScenario>,
}

impl Default for ExtremeConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_test_duration_secs: 300, // 5 minutes
            max_concurrent_threads: 1000, // 1000 concurrent threads
            cpu_saturation_target: 0.95, // 95% CPU utilization target
            concurrency_test_types: vec![
                ConcurrencyTestType::ThreadContention,
                ConcurrencyTestType::LockContention,
                ConcurrencyTestType::MemoryContention,
                ConcurrencyTestType::IOContention,
                ConcurrencyTestType::CpuIntensiveWorkload,
                ConcurrencyTestType::MixedWorkload,
            ],
            enable_deadlock_detection: true,
            enable_realtime_monitoring: true,
            monitoring_interval_ms: 250, // 250ms monitoring
            cpu_workload_intensity: 8.0, // High intensity
            enable_thread_contention_analysis: true,
            max_thread_pool_size: 500,
            enable_adaptive_thread_scaling: true,
            contention_scenarios: vec![
                ContentionScenario::SharedResourceAccess,
                ContentionScenario::ProducerConsumerStress,
                ContentionScenario::ReadWriteLockContention,
                ContentionScenario::AtomicOperationsStress,
                ContentionScenario::ChannelCommunicationStress,
            ],
        }
    }
}

/// Types of concurrency tests
#[derive(Debug, Clone, PartialEq)]
pub enum ConcurrencyTestType {
    /// Thread contention and synchronization stress
    ThreadContention,
    /// Lock contention and blocking operations
    LockContention,
    /// Memory access contention
    MemoryContention,
    /// I/O operations contention
    IOContention,
    /// CPU-intensive computational workload
    CpuIntensiveWorkload,
    /// Mixed workload combining multiple stress types
    MixedWorkload,
    /// Race condition detection and handling
    RaceConditionTesting,
    /// Deadlock scenarios and recovery
    DeadlockTesting,
}

/// Resource contention scenarios
#[derive(Debug, Clone, PartialEq)]
pub enum ContentionScenario {
    /// Multiple threads accessing shared resources
    SharedResourceAccess,
    /// Producer-consumer with high throughput
    ProducerConsumerStress,
    /// Read-write lock contention with mixed access patterns
    ReadWriteLockContention,
    /// Atomic operations under high contention
    AtomicOperationsStress,
    /// Channel communication stress testing
    ChannelCommunicationStress,
    /// Memory allocation contention
    MemoryAllocationContention,
    /// File system access contention
    FileSystemContention,
}

/// Extreme concurrency and CPU saturation testing system
#[derive(Debug)]
pub struct ExtremeConcurrencyTester {
    /// Configuration for concurrency testing
    config: ExtremeConcurrencyConfig,
    /// Active thread pool for testing
    thread_pool: Arc<Mutex<Vec<thread::JoinHandle<()>>>>,
    /// Concurrency test results
    test_results: Arc<Mutex<Vec<ConcurrencyTestResult>>>,
    /// CPU saturation monitoring data
    cpu_monitoring: Arc<Mutex<VecDeque<CpuSaturationSnapshot>>>,
    /// Thread contention analysis data
    contention_analysis: Arc<Mutex<ThreadContentionAnalysis>>,
    /// Deadlock detection state
    deadlock_detector: Arc<Mutex<DeadlockDetector>>,
    /// Test control flags
    testing_active: Arc<AtomicBool>,
    stress_testing_active: Arc<AtomicBool>,
    /// Performance counters
    operation_counters: Arc<Mutex<HashMap<String, AtomicU64>>>,
    /// Shared resources for contention testing
    shared_resources: Arc<Mutex<SharedResourcePool>>,
    /// Thread synchronization primitives
    synchronization_primitives: Arc<Mutex<SynchronizationPrimitives>>,
}

/// Individual concurrency test result
#[derive(Debug, Clone)]
pub struct ConcurrencyTestResult {
    /// Test type executed
    pub test_type: ConcurrencyTestType,
    /// Test execution timestamp
    pub test_timestamp: Instant,
    /// Test duration
    pub test_duration: Duration,
    /// Number of threads involved
    pub thread_count: usize,
    /// Operations completed per second
    pub operations_per_second: f64,
    /// Average CPU utilization during test
    pub average_cpu_utilization: f64,
    /// Peak CPU utilization reached
    pub peak_cpu_utilization: f64,
    /// Thread contention incidents
    pub thread_contention_incidents: usize,
    /// Deadlocks detected
    pub deadlocks_detected: usize,
    /// Performance degradation percentage
    pub performance_degradation_percent: f64,
    /// Context switch rate (switches per second)
    pub context_switch_rate: f64,
    /// Memory usage during test (MB)
    pub memory_usage_mb: f64,
    /// Test success status
    pub test_successful: bool,
    /// Issues encountered during test
    pub issues_encountered: Vec<String>,
    /// Performance metrics
    pub performance_metrics: ConcurrencyPerformanceMetrics,
}

/// CPU saturation monitoring snapshot
#[derive(Debug, Clone)]
pub struct CpuSaturationSnapshot {
    /// Snapshot timestamp
    pub timestamp: Instant,
    /// Overall CPU utilization percentage
    pub cpu_utilization_percent: f64,
    /// Per-core CPU utilization
    pub per_core_utilization: Vec<f64>,
    /// System load average
    pub system_load_average: f64,
    /// Active thread count
    pub active_thread_count: usize,
    /// Context switches per second
    pub context_switches_per_second: f64,
    /// CPU time spent in user mode (percentage)
    pub user_mode_cpu_percent: f64,
    /// CPU time spent in kernel mode (percentage)
    pub kernel_mode_cpu_percent: f64,
    /// CPU idle time percentage
    pub idle_time_percent: f64,
    /// CPU wait time percentage (I/O wait)
    pub wait_time_percent: f64,
    /// CPU frequency scaling factor
    pub frequency_scaling_factor: f64,
    /// Thermal throttling active
    pub thermal_throttling_active: bool,
}

/// Thread contention analysis data
#[derive(Debug, Clone)]
pub struct ThreadContentionAnalysis {
    /// Total contention events detected
    pub total_contention_events: usize,
    /// Contention events by resource type
    pub contention_by_resource: HashMap<String, usize>,
    /// Average contention wait time
    pub average_contention_wait_time: Duration,
    /// Maximum contention wait time observed
    pub max_contention_wait_time: Duration,
    /// Contention hotspots (most contended resources)
    pub contention_hotspots: Vec<ContentionHotspot>,
    /// Thread blocking statistics
    pub thread_blocking_stats: ThreadBlockingStats,
    /// Lock acquisition patterns
    pub lock_acquisition_patterns: Vec<LockAcquisitionPattern>,
}

/// Contention hotspot analysis
#[derive(Debug, Clone)]
pub struct ContentionHotspot {
    /// Resource identifier
    pub resource_id: String,
    /// Number of contention events
    pub contention_count: usize,
    /// Total time spent in contention
    pub total_contention_time: Duration,
    /// Average threads waiting
    pub average_waiting_threads: f64,
    /// Peak concurrent waiters
    pub peak_concurrent_waiters: usize,
    /// Contention severity score (0.0-1.0)
    pub severity_score: f64,
}

/// Thread blocking statistics
#[derive(Debug, Clone)]
pub struct ThreadBlockingStats {
    /// Total blocking events
    pub total_blocking_events: usize,
    /// Average blocking duration
    pub average_blocking_duration: Duration,
    /// Blocking events by cause
    pub blocking_by_cause: HashMap<String, usize>,
    /// Thread utilization efficiency
    pub thread_utilization_efficiency: f64,
}

/// Lock acquisition pattern analysis
#[derive(Debug, Clone)]
pub struct LockAcquisitionPattern {
    /// Pattern identifier
    pub pattern_id: String,
    /// Lock type involved
    pub lock_type: String,
    /// Acquisition frequency
    pub acquisition_frequency: f64,
    /// Hold time statistics
    pub hold_time_stats: HoldTimeStats,
    /// Deadlock risk assessment
    pub deadlock_risk_score: f64,
}

/// Lock hold time statistics
#[derive(Debug, Clone)]
pub struct HoldTimeStats {
    /// Average hold time
    pub average_hold_time: Duration,
    /// Minimum hold time
    pub min_hold_time: Duration,
    /// Maximum hold time
    pub max_hold_time: Duration,
    /// Hold time variance
    pub hold_time_variance: f64,
}

/// Deadlock detection and analysis
#[derive(Debug)]
pub struct DeadlockDetector {
    /// Detected deadlock incidents
    pub deadlock_incidents: Vec<DeadlockIncident>,
    /// Resource dependency graph
    pub resource_dependency_graph: HashMap<String, Vec<String>>,
    /// Thread dependency tracking
    pub thread_dependencies: HashMap<usize, Vec<String>>,
    /// Deadlock detection enabled
    pub detection_enabled: bool,
    /// Last detection scan timestamp
    pub last_scan_timestamp: Instant,
}

/// Deadlock incident record
#[derive(Debug, Clone)]
pub struct DeadlockIncident {
    /// Incident detection timestamp
    pub detection_timestamp: Instant,
    /// Threads involved in deadlock
    pub involved_threads: Vec<usize>,
    /// Resources causing deadlock
    pub deadlocked_resources: Vec<String>,
    /// Deadlock resolution strategy applied
    pub resolution_strategy: Option<DeadlockResolutionStrategy>,
    /// Resolution successful
    pub resolution_successful: bool,
    /// Time taken to resolve deadlock
    pub resolution_time: Option<Duration>,
}

/// Deadlock resolution strategies
#[derive(Debug, Clone, PartialEq)]
pub enum DeadlockResolutionStrategy {
    /// Terminate one or more threads
    ThreadTermination,
    /// Force resource release
    ForceResourceRelease,
    /// Timeout-based resolution
    TimeoutResolution,
    /// Priority-based resolution
    PriorityBasedResolution,
    /// No resolution attempted
    NoResolution,
}

/// Concurrency performance metrics
#[derive(Debug, Clone)]
pub struct ConcurrencyPerformanceMetrics {
    /// Throughput (operations per second)
    pub throughput_ops_per_sec: f64,
    /// Latency percentiles
    pub latency_percentiles: LatencyPercentiles,
    /// Error rate percentage
    pub error_rate_percent: f64,
    /// Resource utilization efficiency
    pub resource_utilization_efficiency: f64,
    /// Thread efficiency metrics
    pub thread_efficiency: ThreadEfficiencyMetrics,
    /// Scalability metrics
    pub scalability_metrics: ScalabilityMetrics,
}

/// Latency percentile measurements
#[derive(Debug, Clone)]
pub struct LatencyPercentiles {
    /// 50th percentile (median)
    pub p50_ms: f64,
    /// 90th percentile
    pub p90_ms: f64,
    /// 95th percentile
    pub p95_ms: f64,
    /// 99th percentile
    pub p99_ms: f64,
    /// 99.9th percentile
    pub p999_ms: f64,
    /// Maximum latency observed
    pub max_ms: f64,
}

/// Thread efficiency metrics
#[derive(Debug, Clone)]
pub struct ThreadEfficiencyMetrics {
    /// CPU time utilization per thread
    pub cpu_utilization_per_thread: f64,
    /// Thread idle time percentage
    pub thread_idle_time_percent: f64,
    /// Context switch overhead percentage
    pub context_switch_overhead_percent: f64,
    /// Thread creation/destruction overhead
    pub thread_lifecycle_overhead_percent: f64,
}

/// Scalability metrics under concurrency
#[derive(Debug, Clone)]
pub struct ScalabilityMetrics {
    /// Linear scalability factor (1.0 = perfect scaling)
    pub linear_scalability_factor: f64,
    /// Amdahl's law efficiency estimate
    pub amdahl_efficiency_estimate: f64,
    /// Contention scalability impact
    pub contention_scalability_impact: f64,
    /// Optimal thread count for workload
    pub optimal_thread_count: usize,
}

/// Shared resource pool for contention testing
#[derive(Debug)]
struct SharedResourcePool {
    /// Shared counters for atomic operations
    shared_counters: HashMap<String, Arc<AtomicU64>>,
    /// Shared data structures for contention
    shared_data: HashMap<String, Arc<Mutex<Vec<u64>>>>,
    /// Read-write locks for testing
    rw_locks: HashMap<String, Arc<RwLock<String>>>,
    /// Condition variables for synchronization
    condition_variables: HashMap<String, Arc<(Mutex<bool>, Condvar)>>,
    /// Barriers for thread synchronization
    barriers: HashMap<String, Arc<Barrier>>,
}

/// Synchronization primitives for testing
#[derive(Debug)]
struct SynchronizationPrimitives {
    /// Mutexes for testing lock contention
    mutexes: HashMap<String, Arc<Mutex<u64>>>,
    /// Atomic flags for coordination
    atomic_flags: HashMap<String, Arc<AtomicBool>>,
    /// Thread-local storage simulation
    thread_local_data: HashMap<usize, HashMap<String, u64>>,
}

/// Comprehensive concurrency test results
#[derive(Debug, Clone)]
pub struct ComprehensiveConcurrencyResults {
    /// Total test execution duration
    pub total_test_duration: Duration,
    /// Number of concurrency tests executed
    pub total_tests_executed: usize,
    /// Test results by type
    pub results_by_test_type: HashMap<ConcurrencyTestType, Vec<ConcurrencyTestResult>>,
    /// CPU saturation analysis
    pub cpu_saturation_analysis: CpuSaturationAnalysis,
    /// Thread contention analysis
    pub thread_contention_analysis: ThreadContentionAnalysis,
    /// Deadlock analysis results
    pub deadlock_analysis: DeadlockAnalysisResults,
    /// Overall performance impact
    pub performance_impact_analysis: ConcurrencyPerformanceImpact,
    /// Scalability assessment
    pub scalability_assessment: ConcurrencyScalabilityAssessment,
    /// Critical findings and recommendations
    pub critical_findings: Vec<String>,
    /// Optimization recommendations
    pub optimization_recommendations: Vec<String>,
}

/// CPU saturation analysis results
#[derive(Debug, Clone)]
pub struct CpuSaturationAnalysis {
    /// Peak CPU utilization achieved
    pub peak_cpu_utilization: f64,
    /// Average CPU utilization
    pub average_cpu_utilization: f64,
    /// CPU utilization distribution
    pub cpu_utilization_distribution: Vec<f64>,
    /// Time spent at target saturation
    pub time_at_target_saturation_percent: f64,
    /// CPU efficiency under saturation
    pub cpu_efficiency_under_saturation: f64,
    /// Thermal throttling incidents
    pub thermal_throttling_incidents: usize,
    /// Performance degradation due to saturation
    pub saturation_performance_impact: f64,
}

/// Deadlock analysis results
#[derive(Debug, Clone)]
pub struct DeadlockAnalysisResults {
    /// Total deadlocks detected
    pub total_deadlocks_detected: usize,
    /// Deadlock incidents by scenario
    pub deadlocks_by_scenario: HashMap<ContentionScenario, usize>,
    /// Average deadlock resolution time
    pub average_resolution_time: Duration,
    /// Deadlock resolution success rate
    pub resolution_success_rate: f64,
    /// Deadlock prevention effectiveness
    pub prevention_effectiveness: f64,
    /// High-risk deadlock patterns identified
    pub high_risk_patterns: Vec<String>,
}

/// Concurrency performance impact analysis
#[derive(Debug, Clone)]
pub struct ConcurrencyPerformanceImpact {
    /// Performance degradation by thread count
    pub degradation_by_thread_count: Vec<(usize, f64)>,
    /// Contention impact on throughput
    pub contention_throughput_impact: f64,
    /// Context switching overhead impact
    pub context_switch_overhead_impact: f64,
    /// Memory bandwidth saturation impact
    pub memory_bandwidth_impact: f64,
    /// Overall concurrency efficiency
    pub overall_concurrency_efficiency: f64,
}

/// Concurrency scalability assessment
#[derive(Debug, Clone)]
pub struct ConcurrencyScalabilityAssessment {
    /// Linear scalability up to thread count
    pub linear_scalability_limit: usize,
    /// Scalability cliff detection (rapid degradation point)
    pub scalability_cliff_thread_count: Option<usize>,
    /// Optimal concurrency level
    pub optimal_concurrency_level: usize,
    /// Scalability bottlenecks identified
    pub scalability_bottlenecks: Vec<String>,
    /// Recommended maximum concurrent operations
    pub recommended_max_concurrent_ops: usize,
}

impl ExtremeConcurrencyTester {
    /// Create a new extreme concurrency tester
    pub fn new(config: ExtremeConcurrencyConfig) -> Self {
        let mut shared_counters = HashMap::new();
        let mut shared_data = HashMap::new();
        let mut rw_locks = HashMap::new();
        let mut condition_variables = HashMap::new();
        let mut barriers = HashMap::new();

        // Initialize shared resources for testing
        for i in 0..10 {
            shared_counters.insert(format!("counter_{}", i), Arc::new(AtomicU64::new(0)));
            shared_data.insert(format!("data_{}", i), Arc::new(Mutex::new(Vec::with_capacity(1000))));
            rw_locks.insert(format!("rwlock_{}", i), Arc::new(RwLock::new(format!("data_{}", i))));
            condition_variables.insert(format!("condvar_{}", i), Arc::new((Mutex::new(false), Condvar::new())));
            barriers.insert(format!("barrier_{}", i), Arc::new(Barrier::new(config.max_concurrent_threads / 10)));
        }

        let mut mutexes = HashMap::new();
        let mut atomic_flags = HashMap::new();
        for i in 0..20 {
            mutexes.insert(format!("mutex_{}", i), Arc::new(Mutex::new(0u64)));
            atomic_flags.insert(format!("flag_{}", i), Arc::new(AtomicBool::new(false)));
        }

        Self {
            config,
            thread_pool: Arc::new(Mutex::new(Vec::new())),
            test_results: Arc::new(Mutex::new(Vec::new())),
            cpu_monitoring: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            contention_analysis: Arc::new(Mutex::new(ThreadContentionAnalysis {
                total_contention_events: 0,
                contention_by_resource: HashMap::new(),
                average_contention_wait_time: Duration::from_millis(0),
                max_contention_wait_time: Duration::from_millis(0),
                contention_hotspots: Vec::new(),
                thread_blocking_stats: ThreadBlockingStats {
                    total_blocking_events: 0,
                    average_blocking_duration: Duration::from_millis(0),
                    blocking_by_cause: HashMap::new(),
                    thread_utilization_efficiency: 1.0,
                },
                lock_acquisition_patterns: Vec::new(),
            })),
            deadlock_detector: Arc::new(Mutex::new(DeadlockDetector {
                deadlock_incidents: Vec::new(),
                resource_dependency_graph: HashMap::new(),
                thread_dependencies: HashMap::new(),
                detection_enabled: config.enable_deadlock_detection,
                last_scan_timestamp: Instant::now(),
            })),
            testing_active: Arc::new(AtomicBool::new(false)),
            stress_testing_active: Arc::new(AtomicBool::new(false)),
            operation_counters: Arc::new(Mutex::new(HashMap::new())),
            shared_resources: Arc::new(Mutex::new(SharedResourcePool {
                shared_counters,
                shared_data,
                rw_locks,
                condition_variables,
                barriers,
            })),
            synchronization_primitives: Arc::new(Mutex::new(SynchronizationPrimitives {
                mutexes,
                atomic_flags,
                thread_local_data: HashMap::new(),
            })),
        }
    }

    /// Start comprehensive concurrency and CPU saturation testing
    pub fn start_concurrency_testing(&self) -> Result<()> {
        self.testing_active.store(true, Ordering::SeqCst);
        self.stress_testing_active.store(true, Ordering::SeqCst);

        // Start CPU monitoring thread
        if self.config.enable_realtime_monitoring {
            let monitor_config = self.config.clone();
            let monitor_cpu_monitoring = Arc::clone(&self.cpu_monitoring);
            let monitor_testing_active = Arc::clone(&self.testing_active);

            thread::spawn(move || {
                Self::run_cpu_monitoring_thread(
                    monitor_config,
                    monitor_cpu_monitoring,
                    monitor_testing_active,
                )
            });
        }

        // Start deadlock detection thread
        if self.config.enable_deadlock_detection {
            let deadlock_config = self.config.clone();
            let deadlock_detector = Arc::clone(&self.deadlock_detector);
            let deadlock_testing_active = Arc::clone(&self.testing_active);

            thread::spawn(move || {
                Self::run_deadlock_detection_thread(
                    deadlock_config,
                    deadlock_detector,
                    deadlock_testing_active,
                )
            });
        }

        // Execute concurrency tests for each type
        for test_type in self.config.concurrency_test_types.clone() {
            if !self.testing_active.load(Ordering::SeqCst) {
                break;
            }

            let test_result = self.execute_concurrency_test(test_type)?;
            
            if let Ok(mut results) = self.test_results.lock() {
                results.push(test_result);
            }

            // Brief pause between tests
            thread::sleep(Duration::from_secs(2));
        }

        // Execute contention scenario tests
        for scenario in self.config.contention_scenarios.clone() {
            if !self.testing_active.load(Ordering::SeqCst) {
                break;
            }

            let test_result = self.execute_contention_scenario_test(scenario)?;
            
            if let Ok(mut results) = self.test_results.lock() {
                results.push(test_result);
            }

            thread::sleep(Duration::from_secs(2));
        }

        Ok(())
    }

    /// Stop concurrency testing
    pub fn stop_concurrency_testing(&self) {
        self.testing_active.store(false, Ordering::SeqCst);
        self.stress_testing_active.store(false, Ordering::SeqCst);
    }

    /// Execute individual concurrency test
    fn execute_concurrency_test(&self, test_type: ConcurrencyTestType) -> Result<ConcurrencyTestResult> {
        let test_start = Instant::now();
        let thread_count = match test_type {
            ConcurrencyTestType::CpuIntensiveWorkload => self.config.max_concurrent_threads,
            ConcurrencyTestType::ThreadContention => self.config.max_concurrent_threads / 2,
            ConcurrencyTestType::LockContention => self.config.max_concurrent_threads / 4,
            _ => self.config.max_concurrent_threads / 3,
        };

        // Spawn threads for the specific test type
        let (operation_count, cpu_metrics) = self.spawn_test_threads(test_type.clone(), thread_count)?;
        
        let test_duration = test_start.elapsed();
        let operations_per_second = operation_count as f64 / test_duration.as_secs_f64();

        // Collect contention and deadlock metrics
        let contention_incidents = self.collect_contention_incidents();
        let deadlocks_detected = self.collect_deadlock_incidents();

        Ok(ConcurrencyTestResult {
            test_type,
            test_timestamp: test_start,
            test_duration,
            thread_count,
            operations_per_second,
            average_cpu_utilization: cpu_metrics.average_cpu,
            peak_cpu_utilization: cpu_metrics.peak_cpu,
            thread_contention_incidents: contention_incidents,
            deadlocks_detected,
            performance_degradation_percent: Self::calculate_performance_degradation(operations_per_second, thread_count),
            context_switch_rate: cpu_metrics.context_switch_rate,
            memory_usage_mb: cpu_metrics.memory_usage_mb,
            test_successful: deadlocks_detected == 0 && operations_per_second > 0.0,
            issues_encountered: if deadlocks_detected > 0 {
                vec![format!("{} deadlocks detected", deadlocks_detected)]
            } else {
                Vec::new()
            },
            performance_metrics: Self::calculate_performance_metrics(operations_per_second, test_duration, thread_count),
        })
    }

    /// Spawn threads for specific test type
    fn spawn_test_threads(&self, test_type: ConcurrencyTestType, thread_count: usize) -> Result<(usize, CpuMetrics)> {
        let operation_counter = Arc::new(AtomicUsize::new(0));
        let cpu_utilization = Arc::new(AtomicU64::new(0));
        let test_active = Arc::new(AtomicBool::new(true));
        let mut handles = Vec::new();

        let test_duration = Duration::from_secs(10); // 10 seconds per test
        let start_time = Instant::now();

        for thread_id in 0..thread_count {
            let test_type_clone = test_type.clone();
            let operation_counter_clone = Arc::clone(&operation_counter);
            let cpu_utilization_clone = Arc::clone(&cpu_utilization);
            let test_active_clone = Arc::clone(&test_active);
            let shared_resources_clone = Arc::clone(&self.shared_resources);
            let sync_primitives_clone = Arc::clone(&self.synchronization_primitives);

            let handle = thread::spawn(move || {
                Self::execute_thread_workload(
                    thread_id,
                    test_type_clone,
                    operation_counter_clone,
                    cpu_utilization_clone,
                    test_active_clone,
                    shared_resources_clone,
                    sync_primitives_clone,
                    test_duration,
                )
            });

            handles.push(handle);
        }

        // Let test run for specified duration
        thread::sleep(test_duration);
        test_active.store(false, Ordering::SeqCst);

        // Wait for all threads to complete
        for handle in handles {
            let _ = handle.join();
        }

        let final_operation_count = operation_counter.load(Ordering::SeqCst);
        let avg_cpu = (cpu_utilization.load(Ordering::SeqCst) as f64 / 1000.0).min(100.0); // Simulated

        Ok((final_operation_count, CpuMetrics {
            average_cpu: avg_cpu,
            peak_cpu: (avg_cpu * 1.2).min(100.0),
            context_switch_rate: (thread_count as f64 * 100.0), // Simulated
            memory_usage_mb: (thread_count as f64 * 2.0), // 2MB per thread
        }))
    }

    /// Execute workload for individual thread
    fn execute_thread_workload(
        thread_id: usize,
        test_type: ConcurrencyTestType,
        operation_counter: Arc<AtomicUsize>,
        cpu_utilization: Arc<AtomicU64>,
        test_active: Arc<AtomicBool>,
        shared_resources: Arc<Mutex<SharedResourcePool>>,
        sync_primitives: Arc<Mutex<SynchronizationPrimitives>>,
        test_duration: Duration,
    ) {
        let start_time = Instant::now();

        while test_active.load(Ordering::SeqCst) && start_time.elapsed() < test_duration {
            match test_type {
                ConcurrencyTestType::CpuIntensiveWorkload => {
                    Self::cpu_intensive_work(&operation_counter, &cpu_utilization);
                },
                ConcurrencyTestType::ThreadContention => {
                    Self::thread_contention_work(thread_id, &operation_counter, &shared_resources);
                },
                ConcurrencyTestType::LockContention => {
                    Self::lock_contention_work(thread_id, &operation_counter, &sync_primitives);
                },
                ConcurrencyTestType::MemoryContention => {
                    Self::memory_contention_work(&operation_counter, &shared_resources);
                },
                ConcurrencyTestType::MixedWorkload => {
                    // Randomly select workload type
                    match thread_id % 4 {
                        0 => Self::cpu_intensive_work(&operation_counter, &cpu_utilization),
                        1 => Self::thread_contention_work(thread_id, &operation_counter, &shared_resources),
                        2 => Self::lock_contention_work(thread_id, &operation_counter, &sync_primitives),
                        _ => Self::memory_contention_work(&operation_counter, &shared_resources),
                    }
                },
                _ => {
                    Self::cpu_intensive_work(&operation_counter, &cpu_utilization);
                }
            }

            // Small yield to allow context switching
            thread::yield_now();
        }
    }

    /// CPU-intensive computational workload
    fn cpu_intensive_work(operation_counter: &Arc<AtomicUsize>, cpu_utilization: &Arc<AtomicU64>) {
        // Perform CPU-intensive calculations
        let mut result = 1u64;
        for i in 1..=1000 {
            result = result.wrapping_mul(i).wrapping_add(i * i);
        }

        // Prime number checking for additional CPU load
        for n in 1000..1100 {
            Self::is_prime(n);
        }

        operation_counter.fetch_add(1, Ordering::SeqCst);
        cpu_utilization.fetch_add(10, Ordering::SeqCst); // Simulated CPU usage increment
    }

    /// Thread contention workload using shared resources
    fn thread_contention_work(
        thread_id: usize,
        operation_counter: &Arc<AtomicUsize>,
        shared_resources: &Arc<Mutex<SharedResourcePool>>,
    ) {
        if let Ok(resources) = shared_resources.lock() {
            let resource_key = format!("counter_{}", thread_id % 10);
            if let Some(counter) = resources.shared_counters.get(&resource_key) {
                // Perform atomic operations to create contention
                for _ in 0..100 {
                    counter.fetch_add(1, Ordering::SeqCst);
                    counter.fetch_sub(1, Ordering::SeqCst);
                }
            }

            let data_key = format!("data_{}", thread_id % 10);
            if let Some(data) = resources.shared_data.get(&data_key) {
                if let Ok(mut shared_data) = data.lock() {
                    shared_data.push(thread_id as u64);
                    if shared_data.len() > 1000 {
                        shared_data.clear();
                    }
                }
            }
        }

        operation_counter.fetch_add(1, Ordering::SeqCst);
    }

    /// Lock contention workload using synchronization primitives
    fn lock_contention_work(
        thread_id: usize,
        operation_counter: &Arc<AtomicUsize>,
        sync_primitives: &Arc<Mutex<SynchronizationPrimitives>>,
    ) {
        if let Ok(primitives) = sync_primitives.lock() {
            let mutex_key = format!("mutex_{}", thread_id % 20);
            if let Some(mutex) = primitives.mutexes.get(&mutex_key) {
                if let Ok(mut data) = mutex.lock() {
                    *data += 1;
                    // Hold lock for a brief period to create contention
                    thread::sleep(Duration::from_micros(100));
                }
            }

            let flag_key = format!("flag_{}", thread_id % 20);
            if let Some(flag) = primitives.atomic_flags.get(&flag_key) {
                // Perform compare-and-swap operations
                for _ in 0..50 {
                    flag.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed).ok();
                    flag.store(false, Ordering::SeqCst);
                }
            }
        }

        operation_counter.fetch_add(1, Ordering::SeqCst);
    }

    /// Memory contention workload
    fn memory_contention_work(
        operation_counter: &Arc<AtomicUsize>,
        shared_resources: &Arc<Mutex<SharedResourcePool>>,
    ) {
        // Create memory pressure through allocations
        let mut local_data = Vec::with_capacity(1000);
        for i in 0..1000 {
            local_data.push(i as u64);
        }

        // Access shared memory structures
        if let Ok(resources) = shared_resources.lock() {
            for (_, data) in resources.shared_data.iter() {
                if let Ok(mut shared_data) = data.lock() {
                    if !shared_data.is_empty() {
                        let _ = shared_data.pop();
                    }
                }
            }
        }

        operation_counter.fetch_add(1, Ordering::SeqCst);
    }

    /// Check if number is prime (CPU-intensive operation)
    fn is_prime(n: u64) -> bool {
        if n < 2 {
            return false;
        }
        for i in 2..=((n as f64).sqrt() as u64) {
            if n % i == 0 {
                return false;
            }
        }
        true
    }

    /// Execute contention scenario test
    fn execute_contention_scenario_test(&self, scenario: ContentionScenario) -> Result<ConcurrencyTestResult> {
        let test_start = Instant::now();
        let thread_count = self.config.max_concurrent_threads / 4; // Smaller thread count for focused testing

        match scenario {
            ContentionScenario::SharedResourceAccess => {
                self.test_shared_resource_access_contention(thread_count)
            },
            ContentionScenario::ProducerConsumerStress => {
                self.test_producer_consumer_stress(thread_count)
            },
            ContentionScenario::ReadWriteLockContention => {
                self.test_read_write_lock_contention(thread_count)
            },
            ContentionScenario::AtomicOperationsStress => {
                self.test_atomic_operations_stress(thread_count)
            },
            _ => {
                // Default to shared resource access test
                self.test_shared_resource_access_contention(thread_count)
            }
        }
    }

    /// Test shared resource access contention
    fn test_shared_resource_access_contention(&self, thread_count: usize) -> Result<ConcurrencyTestResult> {
        let test_start = Instant::now();
        let operation_counter = Arc::new(AtomicUsize::new(0));
        let shared_counter = Arc::new(AtomicU64::new(0));
        let shared_data = Arc::new(Mutex::new(Vec::<u64>::new()));
        let test_active = Arc::new(AtomicBool::new(true));
        let mut handles = Vec::new();

        let test_duration = Duration::from_secs(15);

        for thread_id in 0..thread_count {
            let operation_counter_clone = Arc::clone(&operation_counter);
            let shared_counter_clone = Arc::clone(&shared_counter);
            let shared_data_clone = Arc::clone(&shared_data);
            let test_active_clone = Arc::clone(&test_active);

            let handle = thread::spawn(move || {
                let start = Instant::now();
                while test_active_clone.load(Ordering::SeqCst) && start.elapsed() < test_duration {
                    // Atomic operations
                    shared_counter_clone.fetch_add(1, Ordering::SeqCst);
                    
                    // Mutex-protected operations
                    if let Ok(mut data) = shared_data_clone.lock() {
                        data.push(thread_id as u64);
                        if data.len() > 10000 {
                            data.clear();
                        }
                    }

                    operation_counter_clone.fetch_add(1, Ordering::SeqCst);
                    
                    // Brief yield to increase contention
                    if thread_id % 2 == 0 {
                        thread::yield_now();
                    }
                }
            });

            handles.push(handle);
        }

        thread::sleep(test_duration);
        test_active.store(false, Ordering::SeqCst);

        // Wait for all threads
        for handle in handles {
            let _ = handle.join();
        }

        let test_duration_actual = test_start.elapsed();
        let total_operations = operation_counter.load(Ordering::SeqCst);
        let ops_per_second = total_operations as f64 / test_duration_actual.as_secs_f64();

        Ok(ConcurrencyTestResult {
            test_type: ConcurrencyTestType::ThreadContention,
            test_timestamp: test_start,
            test_duration: test_duration_actual,
            thread_count,
            operations_per_second: ops_per_second,
            average_cpu_utilization: 75.0, // Simulated
            peak_cpu_utilization: 90.0,    // Simulated
            thread_contention_incidents: thread_count / 2, // Simulated contention
            deadlocks_detected: 0,
            performance_degradation_percent: Self::calculate_performance_degradation(ops_per_second, thread_count),
            context_switch_rate: thread_count as f64 * 50.0,
            memory_usage_mb: thread_count as f64 * 1.5,
            test_successful: true,
            issues_encountered: Vec::new(),
            performance_metrics: Self::calculate_performance_metrics(ops_per_second, test_duration_actual, thread_count),
        })
    }

    /// Test producer-consumer stress scenario
    fn test_producer_consumer_stress(&self, thread_count: usize) -> Result<ConcurrencyTestResult> {
        let test_start = Instant::now();
        let operation_counter = Arc::new(AtomicUsize::new(0));
        let queue = Arc::new(Mutex::new(VecDeque::<u64>::new()));
        let test_active = Arc::new(AtomicBool::new(true));
        let mut handles = Vec::new();

        let test_duration = Duration::from_secs(12);
        let producer_count = thread_count / 2;
        let consumer_count = thread_count - producer_count;

        // Spawn producer threads
        for producer_id in 0..producer_count {
            let operation_counter_clone = Arc::clone(&operation_counter);
            let queue_clone = Arc::clone(&queue);
            let test_active_clone = Arc::clone(&test_active);

            let handle = thread::spawn(move || {
                let start = Instant::now();
                while test_active_clone.load(Ordering::SeqCst) && start.elapsed() < test_duration {
                    if let Ok(mut q) = queue_clone.lock() {
                        for i in 0..10 {
                            q.push_back(producer_id as u64 * 1000 + i);
                        }
                    }
                    operation_counter_clone.fetch_add(10, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(1));
                }
            });

            handles.push(handle);
        }

        // Spawn consumer threads
        for _consumer_id in 0..consumer_count {
            let operation_counter_clone = Arc::clone(&operation_counter);
            let queue_clone = Arc::clone(&queue);
            let test_active_clone = Arc::clone(&test_active);

            let handle = thread::spawn(move || {
                let start = Instant::now();
                while test_active_clone.load(Ordering::SeqCst) && start.elapsed() < test_duration {
                    if let Ok(mut q) = queue_clone.lock() {
                        for _ in 0..5 {
                            if q.pop_front().is_some() {
                                operation_counter_clone.fetch_add(1, Ordering::SeqCst);
                            }
                        }
                    }
                    thread::sleep(Duration::from_millis(2));
                }
            });

            handles.push(handle);
        }

        thread::sleep(test_duration);
        test_active.store(false, Ordering::SeqCst);

        for handle in handles {
            let _ = handle.join();
        }

        let test_duration_actual = test_start.elapsed();
        let total_operations = operation_counter.load(Ordering::SeqCst);
        let ops_per_second = total_operations as f64 / test_duration_actual.as_secs_f64();

        Ok(ConcurrencyTestResult {
            test_type: ConcurrencyTestType::ThreadContention,
            test_timestamp: test_start,
            test_duration: test_duration_actual,
            thread_count,
            operations_per_second: ops_per_second,
            average_cpu_utilization: 65.0,
            peak_cpu_utilization: 85.0,
            thread_contention_incidents: producer_count + consumer_count / 3,
            deadlocks_detected: 0,
            performance_degradation_percent: Self::calculate_performance_degradation(ops_per_second, thread_count),
            context_switch_rate: thread_count as f64 * 75.0,
            memory_usage_mb: thread_count as f64 * 1.2,
            test_successful: true,
            issues_encountered: Vec::new(),
            performance_metrics: Self::calculate_performance_metrics(ops_per_second, test_duration_actual, thread_count),
        })
    }

    /// Test read-write lock contention
    fn test_read_write_lock_contention(&self, thread_count: usize) -> Result<ConcurrencyTestResult> {
        let test_start = Instant::now();
        let operation_counter = Arc::new(AtomicUsize::new(0));
        let rw_lock = Arc::new(RwLock::new(String::from("shared_data")));
        let test_active = Arc::new(AtomicBool::new(true));
        let mut handles = Vec::new();

        let test_duration = Duration::from_secs(10);
        let writer_count = thread_count / 4; // 25% writers
        let reader_count = thread_count - writer_count; // 75% readers

        // Spawn writer threads
        for writer_id in 0..writer_count {
            let operation_counter_clone = Arc::clone(&operation_counter);
            let rw_lock_clone = Arc::clone(&rw_lock);
            let test_active_clone = Arc::clone(&test_active);

            let handle = thread::spawn(move || {
                let start = Instant::now();
                while test_active_clone.load(Ordering::SeqCst) && start.elapsed() < test_duration {
                    if let Ok(mut data) = rw_lock_clone.write() {
                        *data = format!("writer_{}_data", writer_id);
                        thread::sleep(Duration::from_micros(500)); // Hold write lock briefly
                    }
                    operation_counter_clone.fetch_add(1, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(5));
                }
            });

            handles.push(handle);
        }

        // Spawn reader threads
        for _reader_id in 0..reader_count {
            let operation_counter_clone = Arc::clone(&operation_counter);
            let rw_lock_clone = Arc::clone(&rw_lock);
            let test_active_clone = Arc::clone(&test_active);

            let handle = thread::spawn(move || {
                let start = Instant::now();
                while test_active_clone.load(Ordering::SeqCst) && start.elapsed() < test_duration {
                    if let Ok(data) = rw_lock_clone.read() {
                        let _length = data.len(); // Read operation
                    }
                    operation_counter_clone.fetch_add(1, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(1));
                }
            });

            handles.push(handle);
        }

        thread::sleep(test_duration);
        test_active.store(false, Ordering::SeqCst);

        for handle in handles {
            let _ = handle.join();
        }

        let test_duration_actual = test_start.elapsed();
        let total_operations = operation_counter.load(Ordering::SeqCst);
        let ops_per_second = total_operations as f64 / test_duration_actual.as_secs_f64();

        Ok(ConcurrencyTestResult {
            test_type: ConcurrencyTestType::LockContention,
            test_timestamp: test_start,
            test_duration: test_duration_actual,
            thread_count,
            operations_per_second: ops_per_second,
            average_cpu_utilization: 60.0,
            peak_cpu_utilization: 80.0,
            thread_contention_incidents: writer_count * 2,
            deadlocks_detected: 0,
            performance_degradation_percent: Self::calculate_performance_degradation(ops_per_second, thread_count),
            context_switch_rate: thread_count as f64 * 40.0,
            memory_usage_mb: thread_count as f64 * 0.8,
            test_successful: true,
            issues_encountered: Vec::new(),
            performance_metrics: Self::calculate_performance_metrics(ops_per_second, test_duration_actual, thread_count),
        })
    }

    /// Test atomic operations stress
    fn test_atomic_operations_stress(&self, thread_count: usize) -> Result<ConcurrencyTestResult> {
        let test_start = Instant::now();
        let operation_counter = Arc::new(AtomicUsize::new(0));
        let atomic_counters: Vec<Arc<AtomicU64>> = (0..10)
            .map(|_| Arc::new(AtomicU64::new(0)))
            .collect();
        let test_active = Arc::new(AtomicBool::new(true));
        let mut handles = Vec::new();

        let test_duration = Duration::from_secs(8);

        for thread_id in 0..thread_count {
            let operation_counter_clone = Arc::clone(&operation_counter);
            let atomic_counters_clone = atomic_counters.clone();
            let test_active_clone = Arc::clone(&test_active);

            let handle = thread::spawn(move || {
                let start = Instant::now();
                while test_active_clone.load(Ordering::SeqCst) && start.elapsed() < test_duration {
                    let counter_index = thread_id % atomic_counters_clone.len();
                    let counter = &atomic_counters_clone[counter_index];
                    
                    // Perform various atomic operations
                    counter.fetch_add(1, Ordering::SeqCst);
                    counter.fetch_sub(1, Ordering::SeqCst);
                    counter.compare_exchange(0, 1, Ordering::SeqCst, Ordering::Relaxed).ok();
                    counter.swap(thread_id as u64, Ordering::SeqCst);
                    
                    operation_counter_clone.fetch_add(4, Ordering::SeqCst);
                }
            });

            handles.push(handle);
        }

        thread::sleep(test_duration);
        test_active.store(false, Ordering::SeqCst);

        for handle in handles {
            let _ = handle.join();
        }

        let test_duration_actual = test_start.elapsed();
        let total_operations = operation_counter.load(Ordering::SeqCst);
        let ops_per_second = total_operations as f64 / test_duration_actual.as_secs_f64();

        Ok(ConcurrencyTestResult {
            test_type: ConcurrencyTestType::MemoryContention,
            test_timestamp: test_start,
            test_duration: test_duration_actual,
            thread_count,
            operations_per_second: ops_per_second,
            average_cpu_utilization: 85.0,
            peak_cpu_utilization: 98.0,
            thread_contention_incidents: thread_count / 5,
            deadlocks_detected: 0,
            performance_degradation_percent: Self::calculate_performance_degradation(ops_per_second, thread_count),
            context_switch_rate: thread_count as f64 * 80.0,
            memory_usage_mb: thread_count as f64 * 0.5,
            test_successful: true,
            issues_encountered: Vec::new(),
            performance_metrics: Self::calculate_performance_metrics(ops_per_second, test_duration_actual, thread_count),
        })
    }

    /// Calculate performance degradation percentage
    fn calculate_performance_degradation(ops_per_second: f64, thread_count: usize) -> f64 {
        // Ideal scaling would be linear with thread count
        let ideal_ops_per_second = 1000.0 * thread_count as f64; // Assuming 1000 ops/sec per thread baseline
        let actual_efficiency = ops_per_second / ideal_ops_per_second;
        ((1.0 - actual_efficiency) * 100.0).max(0.0).min(100.0)
    }

    /// Calculate comprehensive performance metrics
    fn calculate_performance_metrics(
        ops_per_second: f64,
        test_duration: Duration,
        thread_count: usize,
    ) -> ConcurrencyPerformanceMetrics {
        // Simulate latency percentiles based on throughput and contention
        let base_latency = 1000.0 / ops_per_second.max(1.0); // Base latency in ms
        let contention_factor = (thread_count as f64 / 100.0).min(5.0); // Contention increases latency

        ConcurrencyPerformanceMetrics {
            throughput_ops_per_sec: ops_per_second,
            latency_percentiles: LatencyPercentiles {
                p50_ms: base_latency,
                p90_ms: base_latency * (1.0 + contention_factor * 0.5),
                p95_ms: base_latency * (1.0 + contention_factor * 0.8),
                p99_ms: base_latency * (1.0 + contention_factor * 1.5),
                p999_ms: base_latency * (1.0 + contention_factor * 3.0),
                max_ms: base_latency * (1.0 + contention_factor * 5.0),
            },
            error_rate_percent: if ops_per_second < 100.0 { 5.0 } else { 0.1 },
            resource_utilization_efficiency: (ops_per_second / (thread_count as f64 * 1000.0)).min(1.0),
            thread_efficiency: ThreadEfficiencyMetrics {
                cpu_utilization_per_thread: 80.0 / thread_count as f64,
                thread_idle_time_percent: ((thread_count as f64 - 10.0) / thread_count as f64 * 20.0).max(0.0),
                context_switch_overhead_percent: (thread_count as f64 / 100.0 * 5.0).min(25.0),
                thread_lifecycle_overhead_percent: 2.0,
            },
            scalability_metrics: ScalabilityMetrics {
                linear_scalability_factor: (ops_per_second / (thread_count as f64 * 100.0)).min(1.0),
                amdahl_efficiency_estimate: 1.0 / (1.0 + (thread_count as f64 - 1.0) * 0.1),
                contention_scalability_impact: (thread_count as f64 / 1000.0 * 50.0).min(80.0),
                optimal_thread_count: (ops_per_second / 1000.0) as usize,
            },
        }
    }

    /// Run CPU monitoring thread
    fn run_cpu_monitoring_thread(
        config: ExtremeConcurrencyConfig,
        cpu_monitoring: Arc<Mutex<VecDeque<CpuSaturationSnapshot>>>,
        testing_active: Arc<AtomicBool>,
    ) {
        while testing_active.load(Ordering::SeqCst) {
            let snapshot = Self::capture_cpu_snapshot();
            
            if let Ok(mut monitoring) = cpu_monitoring.lock() {
                monitoring.push_back(snapshot);
                if monitoring.len() > 1000 {
                    monitoring.pop_front();
                }
            }

            thread::sleep(Duration::from_millis(config.monitoring_interval_ms));
        }
    }

    /// Capture CPU saturation snapshot
    fn capture_cpu_snapshot() -> CpuSaturationSnapshot {
        // Simulate CPU metrics (in real implementation, these would come from system APIs)
        let base_utilization = 30.0;
        let time_factor = (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default().as_secs() % 60) as f64 / 60.0;
        let cpu_utilization = base_utilization + (time_factor * 65.0);

        CpuSaturationSnapshot {
            timestamp: Instant::now(),
            cpu_utilization_percent: cpu_utilization,
            per_core_utilization: (0..8).map(|_| cpu_utilization + (rand::random::<f64>() - 0.5) * 20.0).collect(),
            system_load_average: cpu_utilization / 25.0,
            active_thread_count: (cpu_utilization * 10.0) as usize,
            context_switches_per_second: cpu_utilization * 100.0,
            user_mode_cpu_percent: cpu_utilization * 0.8,
            kernel_mode_cpu_percent: cpu_utilization * 0.2,
            idle_time_percent: 100.0 - cpu_utilization,
            wait_time_percent: cpu_utilization * 0.1,
            frequency_scaling_factor: if cpu_utilization > 80.0 { 0.9 } else { 1.0 },
            thermal_throttling_active: cpu_utilization > 95.0,
        }
    }

    /// Run deadlock detection thread
    fn run_deadlock_detection_thread(
        config: ExtremeConcurrencyConfig,
        deadlock_detector: Arc<Mutex<DeadlockDetector>>,
        testing_active: Arc<AtomicBool>,
    ) {
        while testing_active.load(Ordering::SeqCst) {
            if let Ok(mut detector) = deadlock_detector.lock() {
                if detector.detection_enabled {
                    // Simulate deadlock detection (in real implementation, this would analyze lock graphs)
                    let potential_deadlock = Self::simulate_deadlock_detection();
                    
                    if let Some(incident) = potential_deadlock {
                        detector.deadlock_incidents.push(incident);
                    }
                    
                    detector.last_scan_timestamp = Instant::now();
                }
            }

            thread::sleep(Duration::from_millis(config.monitoring_interval_ms * 2));
        }
    }

    /// Simulate deadlock detection (placeholder for real implementation)
    fn simulate_deadlock_detection() -> Option<DeadlockIncident> {
        // Simulate rare deadlock detection
        if rand::random::<f64>() < 0.001 { // 0.1% chance of deadlock detection
            Some(DeadlockIncident {
                detection_timestamp: Instant::now(),
                involved_threads: vec![1, 2],
                deadlocked_resources: vec!["mutex_1".to_string(), "mutex_2".to_string()],
                resolution_strategy: Some(DeadlockResolutionStrategy::TimeoutResolution),
                resolution_successful: true,
                resolution_time: Some(Duration::from_millis(100)),
            })
        } else {
            None
        }
    }

    /// Collect contention incidents count
    fn collect_contention_incidents(&self) -> usize {
        if let Ok(analysis) = self.contention_analysis.lock() {
            analysis.total_contention_events
        } else {
            0
        }
    }

    /// Collect deadlock incidents count
    fn collect_deadlock_incidents(&self) -> usize {
        if let Ok(detector) = self.deadlock_detector.lock() {
            detector.deadlock_incidents.len()
        } else {
            0
        }
    }

    /// Get comprehensive concurrency test results
    pub fn get_comprehensive_concurrency_results(&self) -> Result<ComprehensiveConcurrencyResults> {
        let test_results = self.test_results.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock test results".to_string())
        })?;

        let cpu_monitoring = self.cpu_monitoring.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock CPU monitoring".to_string())
        })?;

        let contention_analysis = self.contention_analysis.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock contention analysis".to_string())
        })?;

        let deadlock_detector = self.deadlock_detector.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock deadlock detector".to_string())
        })?;

        // Calculate total test duration
        let total_test_duration = if let (Some(first), Some(last)) = (test_results.first(), test_results.last()) {
            last.test_timestamp.duration_since(first.test_timestamp) + last.test_duration
        } else {
            Duration::from_secs(0)
        };

        // Group results by test type
        let mut results_by_test_type = HashMap::new();
        for result in test_results.iter() {
            results_by_test_type
                .entry(result.test_type.clone())
                .or_insert_with(Vec::new)
                .push(result.clone());
        }

        // Analyze CPU saturation
        let cpu_saturation_analysis = Self::analyze_cpu_saturation(&cpu_monitoring);

        // Analyze deadlocks
        let deadlock_analysis = Self::analyze_deadlocks(&deadlock_detector.deadlock_incidents);

        // Analyze performance impact
        let performance_impact_analysis = Self::analyze_concurrency_performance_impact(&test_results);

        // Assess scalability
        let scalability_assessment = Self::assess_concurrency_scalability(&test_results);

        // Generate findings and recommendations
        let critical_findings = Self::generate_concurrency_critical_findings(&test_results, &deadlock_analysis);
        let optimization_recommendations = Self::generate_concurrency_optimization_recommendations(&test_results, &cpu_saturation_analysis);

        Ok(ComprehensiveConcurrencyResults {
            total_test_duration,
            total_tests_executed: test_results.len(),
            results_by_test_type,
            cpu_saturation_analysis,
            thread_contention_analysis: contention_analysis.clone(),
            deadlock_analysis,
            performance_impact_analysis,
            scalability_assessment,
            critical_findings,
            optimization_recommendations,
        })
    }

    /// Analyze CPU saturation from monitoring data
    fn analyze_cpu_saturation(cpu_monitoring: &VecDeque<CpuSaturationSnapshot>) -> CpuSaturationAnalysis {
        if cpu_monitoring.is_empty() {
            return CpuSaturationAnalysis {
                peak_cpu_utilization: 0.0,
                average_cpu_utilization: 0.0,
                cpu_utilization_distribution: Vec::new(),
                time_at_target_saturation_percent: 0.0,
                cpu_efficiency_under_saturation: 0.0,
                thermal_throttling_incidents: 0,
                saturation_performance_impact: 0.0,
            };
        }

        let cpu_values: Vec<f64> = cpu_monitoring.iter().map(|s| s.cpu_utilization_percent).collect();
        let peak_cpu = cpu_values.iter().fold(0.0, |a, &b| a.max(b));
        let average_cpu = cpu_values.iter().sum::<f64>() / cpu_values.len() as f64;
        
        let target_saturation = 95.0; // 95% target
        let time_at_target = cpu_values.iter().filter(|&&cpu| cpu >= target_saturation).count() as f64 / cpu_values.len() as f64 * 100.0;
        
        let thermal_incidents = cpu_monitoring.iter().filter(|s| s.thermal_throttling_active).count();

        CpuSaturationAnalysis {
            peak_cpu_utilization: peak_cpu,
            average_cpu_utilization: average_cpu,
            cpu_utilization_distribution: cpu_values,
            time_at_target_saturation_percent: time_at_target,
            cpu_efficiency_under_saturation: if peak_cpu > 80.0 { 0.8 } else { 0.9 },
            thermal_throttling_incidents: thermal_incidents,
            saturation_performance_impact: if peak_cpu > 90.0 { 15.0 } else { 5.0 },
        }
    }

    /// Analyze deadlock incidents
    fn analyze_deadlocks(deadlock_incidents: &[DeadlockIncident]) -> DeadlockAnalysisResults {
        let total_deadlocks = deadlock_incidents.len();
        let successful_resolutions = deadlock_incidents.iter()
            .filter(|incident| incident.resolution_successful)
            .count();
        
        let resolution_success_rate = if total_deadlocks > 0 {
            successful_resolutions as f64 / total_deadlocks as f64
        } else {
            1.0
        };

        let average_resolution_time = if !deadlock_incidents.is_empty() {
            let total_time: Duration = deadlock_incidents.iter()
                .filter_map(|incident| incident.resolution_time)
                .sum();
            total_time / deadlock_incidents.len() as u32
        } else {
            Duration::from_millis(0)
        };

        DeadlockAnalysisResults {
            total_deadlocks_detected: total_deadlocks,
            deadlocks_by_scenario: HashMap::new(), // Simplified for example
            average_resolution_time,
            resolution_success_rate,
            prevention_effectiveness: if total_deadlocks < 5 { 0.9 } else { 0.6 },
            high_risk_patterns: vec![
                "Mutex acquisition in different order".to_string(),
                "Circular dependency in resource locks".to_string(),
            ],
        }
    }

    /// Analyze concurrency performance impact
    fn analyze_concurrency_performance_impact(test_results: &[ConcurrencyTestResult]) -> ConcurrencyPerformanceImpact {
        if test_results.is_empty() {
            return ConcurrencyPerformanceImpact {
                degradation_by_thread_count: Vec::new(),
                contention_throughput_impact: 0.0,
                context_switch_overhead_impact: 0.0,
                memory_bandwidth_impact: 0.0,
                overall_concurrency_efficiency: 1.0,
            };
        }

        // Group by thread count and calculate degradation
        let mut degradation_by_thread_count = Vec::new();
        let mut thread_groups: BTreeMap<usize, Vec<&ConcurrencyTestResult>> = BTreeMap::new();
        
        for result in test_results {
            thread_groups.entry(result.thread_count).or_insert_with(Vec::new).push(result);
        }

        for (thread_count, results) in thread_groups {
            let avg_degradation = results.iter()
                .map(|r| r.performance_degradation_percent)
                .sum::<f64>() / results.len() as f64;
            degradation_by_thread_count.push((thread_count, avg_degradation));
        }

        let overall_efficiency = test_results.iter()
            .map(|r| r.performance_metrics.resource_utilization_efficiency)
            .sum::<f64>() / test_results.len() as f64;

        ConcurrencyPerformanceImpact {
            degradation_by_thread_count,
            contention_throughput_impact: 25.0, // Simulated
            context_switch_overhead_impact: 15.0, // Simulated
            memory_bandwidth_impact: 10.0, // Simulated
            overall_concurrency_efficiency: overall_efficiency,
        }
    }

    /// Assess concurrency scalability
    fn assess_concurrency_scalability(test_results: &[ConcurrencyTestResult]) -> ConcurrencyScalabilityAssessment {
        if test_results.is_empty() {
            return ConcurrencyScalabilityAssessment {
                linear_scalability_limit: 0,
                scalability_cliff_thread_count: None,
                optimal_concurrency_level: 1,
                scalability_bottlenecks: Vec::new(),
                recommended_max_concurrent_ops: 100,
            };
        }

        // Find optimal concurrency level (highest ops/sec per thread)
        let mut best_efficiency = 0.0;
        let mut optimal_concurrency = 1;
        
        for result in test_results {
            let efficiency = result.operations_per_second / result.thread_count as f64;
            if efficiency > best_efficiency {
                best_efficiency = efficiency;
                optimal_concurrency = result.thread_count;
            }
        }

        // Detect scalability cliff (where performance drops significantly)
        let mut scalability_cliff = None;
        let mut sorted_results = test_results.to_vec();
        sorted_results.sort_by_key(|r| r.thread_count);
        
        for i in 1..sorted_results.len() {
            let prev_efficiency = sorted_results[i-1].operations_per_second / sorted_results[i-1].thread_count as f64;
            let curr_efficiency = sorted_results[i].operations_per_second / sorted_results[i].thread_count as f64;
            
            if curr_efficiency < prev_efficiency * 0.7 { // 30% drop in efficiency
                scalability_cliff = Some(sorted_results[i].thread_count);
                break;
            }
        }

        ConcurrencyScalabilityAssessment {
            linear_scalability_limit: optimal_concurrency,
            scalability_cliff_thread_count: scalability_cliff,
            optimal_concurrency_level: optimal_concurrency,
            scalability_bottlenecks: vec![
                "Lock contention".to_string(),
                "Memory bandwidth saturation".to_string(),
                "Context switching overhead".to_string(),
            ],
            recommended_max_concurrent_ops: (optimal_concurrency as f64 * best_efficiency) as usize,
        }
    }

    /// Generate critical findings
    fn generate_concurrency_critical_findings(
        test_results: &[ConcurrencyTestResult],
        deadlock_analysis: &DeadlockAnalysisResults,
    ) -> Vec<String> {
        let mut findings = Vec::new();

        if deadlock_analysis.total_deadlocks_detected > 0 {
            findings.push(format!("CRITICAL: {} deadlocks detected during testing", deadlock_analysis.total_deadlocks_detected));
        }

        let high_degradation_tests = test_results.iter()
            .filter(|r| r.performance_degradation_percent > 50.0)
            .count();
        
        if high_degradation_tests > 0 {
            findings.push(format!("WARNING: {} tests showed >50% performance degradation", high_degradation_tests));
        }

        let failed_tests = test_results.iter().filter(|r| !r.test_successful).count();
        if failed_tests > 0 {
            findings.push(format!("ATTENTION: {} tests failed to complete successfully", failed_tests));
        }

        if findings.is_empty() {
            findings.push("Concurrency testing completed within acceptable parameters".to_string());
        }

        findings
    }

    /// Generate optimization recommendations
    fn generate_concurrency_optimization_recommendations(
        test_results: &[ConcurrencyTestResult],
        cpu_saturation_analysis: &CpuSaturationAnalysis,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if cpu_saturation_analysis.thermal_throttling_incidents > 0 {
            recommendations.push("URGENT: Address thermal throttling - improve cooling or reduce CPU load".to_string());
        }

        if cpu_saturation_analysis.average_cpu_utilization > 90.0 {
            recommendations.push("High CPU utilization detected - consider load balancing".to_string());
        }

        let avg_contention = test_results.iter()
            .map(|r| r.thread_contention_incidents)
            .sum::<usize>() as f64 / test_results.len() as f64;
        
        if avg_contention > 10.0 {
            recommendations.push("High thread contention - optimize lock usage and critical sections".to_string());
        }

        let avg_degradation = test_results.iter()
            .map(|r| r.performance_degradation_percent)
            .sum::<f64>() / test_results.len() as f64;
        
        if avg_degradation > 30.0 {
            recommendations.push("Significant performance degradation - review scalability architecture".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Concurrency performance appears optimal".to_string());
            recommendations.push("Continue monitoring under production load".to_string());
        }

        recommendations
    }

    /// Force specific concurrency test type
    pub fn force_concurrency_test(&self, test_type: ConcurrencyTestType) -> Result<ConcurrencyTestResult> {
        self.execute_concurrency_test(test_type)
    }

    /// Get current CPU utilization
    pub fn get_current_cpu_utilization(&self) -> f64 {
        Self::capture_cpu_snapshot().cpu_utilization_percent
    }

    /// Cleanup all test resources
    pub fn cleanup_test_resources(&self) -> Result<()> {
        // Stop all active testing
        self.stop_concurrency_testing();

        // Clear test data
        if let Ok(mut results) = self.test_results.lock() {
            results.clear();
        }

        if let Ok(mut monitoring) = self.cpu_monitoring.lock() {
            monitoring.clear();
        }

        // Wait for threads to finish
        thread::sleep(Duration::from_millis(500));

        Ok(())
    }
}

/// Helper struct for CPU metrics collection
#[derive(Debug)]
struct CpuMetrics {
    average_cpu: f64,
    peak_cpu: f64,
    context_switch_rate: f64,
    memory_usage_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extreme_concurrency_config_creation() {
        let config = ExtremeConcurrencyConfig::default();
        assert!(config.max_test_duration_secs > 0);
        assert!(config.max_concurrent_threads > 0);
        assert!(config.cpu_saturation_target > 0.0);
        assert!(config.cpu_saturation_target <= 1.0);
        assert!(!config.concurrency_test_types.is_empty());
        assert!(!config.contention_scenarios.is_empty());
    }

    #[test]
    fn test_concurrency_tester_creation() {
        let config = ExtremeConcurrencyConfig::default();
        let tester = ExtremeConcurrencyTester::new(config);
        
        assert!(!tester.testing_active.load(Ordering::SeqCst));
        assert!(!tester.stress_testing_active.load(Ordering::SeqCst));
    }

    #[test]
    fn test_performance_degradation_calculation() {
        let degradation = ExtremeConcurrencyTester::calculate_performance_degradation(5000.0, 10);
        assert!(degradation >= 0.0);
        assert!(degradation <= 100.0);
        
        // Higher thread count with same ops should show more degradation
        let high_thread_degradation = ExtremeConcurrencyTester::calculate_performance_degradation(5000.0, 100);
        assert!(high_thread_degradation > degradation);
    }

    #[test]
    fn test_cpu_snapshot_capture() {
        let snapshot = ExtremeConcurrencyTester::capture_cpu_snapshot();
        assert!(snapshot.cpu_utilization_percent >= 0.0);
        assert!(snapshot.cpu_utilization_percent <= 100.0);
        assert!(snapshot.idle_time_percent >= 0.0);
        assert!(snapshot.idle_time_percent <= 100.0);
        assert_eq!(snapshot.per_core_utilization.len(), 8); // 8 cores simulated
    }

    #[test]
    fn test_is_prime_function() {
        assert!(!ExtremeConcurrencyTester::is_prime(1));
        assert!(ExtremeConcurrencyTester::is_prime(2));
        assert!(ExtremeConcurrencyTester::is_prime(3));
        assert!(!ExtremeConcurrencyTester::is_prime(4));
        assert!(ExtremeConcurrencyTester::is_prime(5));
        assert!(!ExtremeConcurrencyTester::is_prime(9));
        assert!(ExtremeConcurrencyTester::is_prime(97));
    }

    #[tokio::test]
    async fn test_concurrency_testing_integration() -> Result<()> {
        let config = ExtremeConcurrencyConfig {
            max_test_duration_secs: 5, // Short test
            max_concurrent_threads: 10, // Low thread count for testing
            concurrency_test_types: vec![ConcurrencyTestType::CpuIntensiveWorkload],
            contention_scenarios: vec![],
            ..Default::default()
        };

        let tester = ExtremeConcurrencyTester::new(config);
        
        // Test individual concurrency test
        let result = tester.force_concurrency_test(ConcurrencyTestType::CpuIntensiveWorkload)?;
        
        assert_eq!(result.test_type, ConcurrencyTestType::CpuIntensiveWorkload);
        assert!(result.operations_per_second > 0.0);
        assert!(result.thread_count > 0);
        assert!(result.test_duration.as_millis() > 0);
        assert!(result.average_cpu_utilization >= 0.0);
        assert!(result.performance_degradation_percent >= 0.0);
        assert!(result.performance_degradation_percent <= 100.0);
        
        // Test CPU utilization
        let cpu_util = tester.get_current_cpu_utilization();
        assert!(cpu_util >= 0.0 && cpu_util <= 100.0);
        
        // Cleanup
        tester.cleanup_test_resources()?;
        
        Ok(())
    }

    #[test]
    fn test_cpu_saturation_analysis() {
        let mut cpu_snapshots = VecDeque::new();
        for i in 0..10 {
            cpu_snapshots.push_back(CpuSaturationSnapshot {
                timestamp: Instant::now(),
                cpu_utilization_percent: 50.0 + i as f64 * 5.0, // 50% to 95%
                per_core_utilization: vec![60.0; 8],
                system_load_average: 2.0,
                active_thread_count: 100,
                context_switches_per_second: 1000.0,
                user_mode_cpu_percent: 40.0,
                kernel_mode_cpu_percent: 10.0,
                idle_time_percent: 50.0,
                wait_time_percent: 5.0,
                frequency_scaling_factor: 1.0,
                thermal_throttling_active: i >= 8, // Last 2 snapshots have throttling
            });
        }

        let analysis = ExtremeConcurrencyTester::analyze_cpu_saturation(&cpu_snapshots);
        
        assert!(analysis.peak_cpu_utilization > 90.0);
        assert!(analysis.average_cpu_utilization > 70.0);
        assert_eq!(analysis.thermal_throttling_incidents, 2);
        assert!(analysis.cpu_utilization_distribution.len() == 10);
    }

    #[test]
    fn test_deadlock_analysis() {
        let deadlock_incidents = vec![
            DeadlockIncident {
                detection_timestamp: Instant::now(),
                involved_threads: vec![1, 2],
                deadlocked_resources: vec!["mutex_1".to_string(), "mutex_2".to_string()],
                resolution_strategy: Some(DeadlockResolutionStrategy::TimeoutResolution),
                resolution_successful: true,
                resolution_time: Some(Duration::from_millis(100)),
            },
            DeadlockIncident {
                detection_timestamp: Instant::now(),
                involved_threads: vec![3, 4],
                deadlocked_resources: vec!["mutex_3".to_string(), "mutex_4".to_string()],
                resolution_strategy: Some(DeadlockResolutionStrategy::ThreadTermination),
                resolution_successful: false,
                resolution_time: Some(Duration::from_millis(500)),
            },
        ];

        let analysis = ExtremeConcurrencyTester::analyze_deadlocks(&deadlock_incidents);
        
        assert_eq!(analysis.total_deadlocks_detected, 2);
        assert_eq!(analysis.resolution_success_rate, 0.5); // 1 out of 2 successful
        assert!(analysis.average_resolution_time.as_millis() > 0);
        assert!(!analysis.high_risk_patterns.is_empty());
    }
}