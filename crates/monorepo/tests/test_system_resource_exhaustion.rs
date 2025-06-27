//! System Resource Exhaustion Testing
//!
//! This module implements comprehensive testing of system resource exhaustion scenarios
//! including file descriptors, threads, network connections, and other OS-level resources
//! to validate system resilience and recovery mechanisms under extreme resource pressure.

use sublime_monorepo_tools::Result;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicUsize, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use std::fs::{File, OpenOptions};
use std::io::{self, Write, Read};
use std::net::{TcpListener, TcpStream};

/// Configuration for system resource exhaustion testing
#[derive(Debug, Clone)]
pub struct ResourceExhaustionConfig {
    /// Maximum test duration in seconds
    pub max_test_duration_secs: u64,
    /// Resource monitoring interval in milliseconds
    pub monitoring_interval_ms: u64,
    /// Types of resources to test
    pub resource_types_to_test: Vec<ResourceType>,
    /// Maximum file descriptors to attempt opening
    pub max_file_descriptors: usize,
    /// Maximum threads to spawn
    pub max_threads: usize,
    /// Maximum network connections to open
    pub max_network_connections: usize,
    /// File size for disk space testing (KB)
    pub test_file_size_kb: usize,
    /// Maximum memory allocations for testing
    pub max_memory_allocations: usize,
    /// Enable automatic recovery testing
    pub enable_recovery_testing: bool,
    /// Recovery delay after resource exhaustion (seconds)
    pub recovery_delay_secs: u64,
    /// Enable real-time resource monitoring
    pub enable_realtime_monitoring: bool,
    /// Resource exhaustion thresholds
    pub exhaustion_thresholds: ResourceThresholds,
}

impl Default for ResourceExhaustionConfig {
    fn default() -> Self {
        Self {
            max_test_duration_secs: 300, // 5 minutes
            monitoring_interval_ms: 1000, // 1 second
            resource_types_to_test: vec![
                ResourceType::FileDescriptors,
                ResourceType::Threads,
                ResourceType::NetworkConnections,
                ResourceType::MemoryAllocations,
                ResourceType::DiskSpace,
            ],
            max_file_descriptors: 1000,
            max_threads: 100,
            max_network_connections: 500,
            test_file_size_kb: 1024, // 1MB per file
            max_memory_allocations: 1000,
            enable_recovery_testing: true,
            recovery_delay_secs: 10,
            enable_realtime_monitoring: true,
            exhaustion_thresholds: ResourceThresholds::default(),
        }
    }
}

/// Types of system resources to test for exhaustion
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// File descriptor exhaustion
    FileDescriptors,
    /// Thread exhaustion
    Threads,
    /// Network connection exhaustion
    NetworkConnections,
    /// Memory allocation exhaustion
    MemoryAllocations,
    /// Disk space exhaustion
    DiskSpace,
    /// CPU time exhaustion
    CpuTime,
    /// Process limit exhaustion
    ProcessLimits,
    /// System call rate limits
    SystemCallLimits,
}

/// Resource exhaustion thresholds
#[derive(Debug, Clone)]
pub struct ResourceThresholds {
    /// File descriptor warning threshold (percentage of limit)
    pub fd_warning_threshold: f64,
    /// Thread warning threshold (percentage of limit)
    pub thread_warning_threshold: f64,
    /// Memory warning threshold (MB)
    pub memory_warning_threshold: usize,
    /// Disk space warning threshold (MB)
    pub disk_warning_threshold: usize,
    /// Network connection warning threshold (percentage of limit)
    pub network_warning_threshold: f64,
    /// CPU utilization warning threshold (percentage)
    pub cpu_warning_threshold: f64,
}

impl Default for ResourceThresholds {
    fn default() -> Self {
        Self {
            fd_warning_threshold: 0.8,   // 80% of FD limit
            thread_warning_threshold: 0.9, // 90% of thread limit
            memory_warning_threshold: 1024, // 1GB
            disk_warning_threshold: 500,  // 500MB
            network_warning_threshold: 0.8, // 80% of connection limit
            cpu_warning_threshold: 95.0,  // 95% CPU
        }
    }
}

/// System resource exhaustion tester
#[derive(Debug)]
pub struct ResourceExhaustionTester {
    /// Configuration for the test
    config: ResourceExhaustionConfig,
    /// Active file descriptors
    active_file_descriptors: Arc<Mutex<Vec<File>>>,
    /// Active threads
    active_threads: Arc<Mutex<Vec<thread::JoinHandle<()>>>>,
    /// Active network connections
    active_network_connections: Arc<Mutex<Vec<TcpStream>>>,
    /// Active memory allocations
    active_memory_allocations: Arc<Mutex<Vec<Vec<u8>>>>,
    /// Resource usage history
    resource_usage_history: Arc<Mutex<VecDeque<ResourceUsageSnapshot>>>,
    /// Resource exhaustion events
    exhaustion_events: Arc<Mutex<Vec<ResourceExhaustionEvent>>>,
    /// Test control flag
    test_active: Arc<AtomicBool>,
    /// Resource counters
    fd_counter: Arc<AtomicUsize>,
    thread_counter: Arc<AtomicUsize>,
    connection_counter: Arc<AtomicUsize>,
    allocation_counter: Arc<AtomicUsize>,
}

/// Snapshot of resource usage at a point in time
#[derive(Debug, Clone)]
pub struct ResourceUsageSnapshot {
    /// Timestamp of snapshot
    pub timestamp: Instant,
    /// Number of open file descriptors
    pub open_file_descriptors: usize,
    /// Number of active threads
    pub active_threads: usize,
    /// Number of network connections
    pub network_connections: usize,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// Disk usage in MB
    pub disk_usage_mb: f64,
    /// CPU utilization percentage
    pub cpu_utilization_percent: f64,
    /// System load average
    pub system_load_average: f64,
    /// Available file descriptors
    pub available_file_descriptors: usize,
    /// Resource pressure level (0.0-1.0)
    pub resource_pressure_level: f64,
}

/// Resource exhaustion event
#[derive(Debug, Clone)]
pub struct ResourceExhaustionEvent {
    /// Timestamp when exhaustion occurred
    pub timestamp: Instant,
    /// Type of resource that was exhausted
    pub resource_type: ResourceType,
    /// Severity of exhaustion
    pub severity: ExhaustionSeverity,
    /// Value that triggered exhaustion
    pub trigger_value: f64,
    /// Threshold that was exceeded
    pub threshold_exceeded: f64,
    /// Error message associated with exhaustion
    pub error_message: String,
    /// Recovery action taken
    pub recovery_action: Option<String>,
    /// Whether recovery was successful
    pub recovery_successful: Option<bool>,
    /// Time to recovery (if applicable)
    pub recovery_time: Option<Duration>,
}

/// Severity levels for resource exhaustion
#[derive(Debug, Clone, PartialEq)]
pub enum ExhaustionSeverity {
    /// Warning level - approaching limits
    Warning,
    /// Critical level - at or near limits
    Critical,
    /// Failure level - operation failed due to exhaustion
    Failure,
    /// Recovery level - system recovering from exhaustion
    Recovery,
}

/// Resource exhaustion test results
#[derive(Debug, Clone)]
pub struct ResourceExhaustionResults {
    /// Test duration
    pub test_duration: Duration,
    /// Total exhaustion events by type
    pub exhaustion_events_by_type: HashMap<ResourceType, usize>,
    /// Peak resource usage
    pub peak_resource_usage: ResourceUsageSnapshot,
    /// Average resource usage
    pub average_resource_usage: ResourceUsageSnapshot,
    /// Recovery statistics
    pub recovery_statistics: RecoveryStatistics,
    /// Performance impact analysis
    pub performance_impact: PerformanceImpactAnalysis,
    /// System stability metrics
    pub stability_metrics: SystemStabilityMetrics,
    /// Recommended system limits
    pub recommended_limits: SystemLimitRecommendations,
}

/// Recovery statistics
#[derive(Debug, Clone)]
pub struct RecoveryStatistics {
    /// Total recovery attempts
    pub total_recovery_attempts: usize,
    /// Successful recoveries
    pub successful_recoveries: usize,
    /// Failed recoveries
    pub failed_recoveries: usize,
    /// Average recovery time
    pub average_recovery_time: Duration,
    /// Fastest recovery time
    pub fastest_recovery_time: Duration,
    /// Slowest recovery time
    pub slowest_recovery_time: Duration,
    /// Recovery success rate
    pub recovery_success_rate: f64,
}

/// Performance impact analysis
#[derive(Debug, Clone)]
pub struct PerformanceImpactAnalysis {
    /// Baseline performance metrics
    pub baseline_metrics: PerformanceMetrics,
    /// Performance under resource pressure
    pub under_pressure_metrics: PerformanceMetrics,
    /// Performance degradation percentage
    pub performance_degradation_percent: f64,
    /// Most impacted operations
    pub most_impacted_operations: Vec<String>,
    /// Resource efficiency scores
    pub resource_efficiency_scores: HashMap<ResourceType, f64>,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Average response time (ms)
    pub average_response_time_ms: f64,
    /// Throughput (operations per second)
    pub throughput_ops_per_sec: f64,
    /// Error rate percentage
    pub error_rate_percent: f64,
    /// Success rate percentage
    pub success_rate_percent: f64,
}

/// System stability metrics
#[derive(Debug, Clone)]
pub struct SystemStabilityMetrics {
    /// Number of system crashes
    pub system_crashes: usize,
    /// Number of resource leaks detected
    pub resource_leaks_detected: usize,
    /// System uptime percentage during test
    pub system_uptime_percent: f64,
    /// Resource cleanup effectiveness
    pub cleanup_effectiveness_percent: f64,
    /// Stability score (0.0-1.0)
    pub overall_stability_score: f64,
}

/// System limit recommendations
#[derive(Debug, Clone)]
pub struct SystemLimitRecommendations {
    /// Recommended file descriptor limit
    pub recommended_fd_limit: usize,
    /// Recommended thread limit
    pub recommended_thread_limit: usize,
    /// Recommended memory limit (MB)
    pub recommended_memory_limit_mb: usize,
    /// Recommended connection limit
    pub recommended_connection_limit: usize,
    /// Safety margin percentage
    pub safety_margin_percent: f64,
    /// Rationale for recommendations
    pub rationale: Vec<String>,
}

impl ResourceExhaustionTester {
    /// Create a new resource exhaustion tester
    pub fn new(config: ResourceExhaustionConfig) -> Self {
        Self {
            config,
            active_file_descriptors: Arc::new(Mutex::new(Vec::new())),
            active_threads: Arc::new(Mutex::new(Vec::new())),
            active_network_connections: Arc::new(Mutex::new(Vec::new())),
            active_memory_allocations: Arc::new(Mutex::new(Vec::new())),
            resource_usage_history: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            exhaustion_events: Arc::new(Mutex::new(Vec::new())),
            test_active: Arc::new(AtomicBool::new(false)),
            fd_counter: Arc::new(AtomicUsize::new(0)),
            thread_counter: Arc::new(AtomicUsize::new(0)),
            connection_counter: Arc::new(AtomicUsize::new(0)),
            allocation_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Start comprehensive resource exhaustion testing
    pub fn start_exhaustion_test(&self) -> Result<()> {
        self.test_active.store(true, Ordering::SeqCst);

        // Start monitoring thread
        if self.config.enable_realtime_monitoring {
            let monitor_config = self.config.clone();
            let monitor_history = Arc::clone(&self.resource_usage_history);
            let monitor_events = Arc::clone(&self.exhaustion_events);
            let monitor_active = Arc::clone(&self.test_active);
            let monitor_fd_counter = Arc::clone(&self.fd_counter);
            let monitor_thread_counter = Arc::clone(&self.thread_counter);
            let monitor_connection_counter = Arc::clone(&self.connection_counter);
            let monitor_allocation_counter = Arc::clone(&self.allocation_counter);

            thread::spawn(move || {
                Self::run_resource_monitoring_thread(
                    monitor_config,
                    monitor_history,
                    monitor_events,
                    monitor_active,
                    monitor_fd_counter,
                    monitor_thread_counter,
                    monitor_connection_counter,
                    monitor_allocation_counter,
                )
            });
        }

        // Start exhaustion threads for each resource type
        for resource_type in &self.config.resource_types_to_test {
            self.start_resource_exhaustion_thread(resource_type.clone())?;
        }

        Ok(())
    }

    /// Stop resource exhaustion testing
    pub fn stop_exhaustion_test(&self) {
        self.test_active.store(false, Ordering::SeqCst);
    }

    /// Start resource exhaustion thread for specific resource type
    fn start_resource_exhaustion_thread(&self, resource_type: ResourceType) -> Result<()> {
        match resource_type {
            ResourceType::FileDescriptors => {
                let config = self.config.clone();
                let file_descriptors = Arc::clone(&self.active_file_descriptors);
                let exhaustion_events = Arc::clone(&self.exhaustion_events);
                let test_active = Arc::clone(&self.test_active);
                let fd_counter = Arc::clone(&self.fd_counter);

                thread::spawn(move || {
                    Self::exhaust_file_descriptors(
                        config,
                        file_descriptors,
                        exhaustion_events,
                        test_active,
                        fd_counter,
                    )
                });
            },
            ResourceType::Threads => {
                let config = self.config.clone();
                let threads = Arc::clone(&self.active_threads);
                let exhaustion_events = Arc::clone(&self.exhaustion_events);
                let test_active = Arc::clone(&self.test_active);
                let thread_counter = Arc::clone(&self.thread_counter);

                thread::spawn(move || {
                    Self::exhaust_threads(
                        config,
                        threads,
                        exhaustion_events,
                        test_active,
                        thread_counter,
                    )
                });
            },
            ResourceType::NetworkConnections => {
                let config = self.config.clone();
                let connections = Arc::clone(&self.active_network_connections);
                let exhaustion_events = Arc::clone(&self.exhaustion_events);
                let test_active = Arc::clone(&self.test_active);
                let connection_counter = Arc::clone(&self.connection_counter);

                thread::spawn(move || {
                    Self::exhaust_network_connections(
                        config,
                        connections,
                        exhaustion_events,
                        test_active,
                        connection_counter,
                    )
                });
            },
            ResourceType::MemoryAllocations => {
                let config = self.config.clone();
                let allocations = Arc::clone(&self.active_memory_allocations);
                let exhaustion_events = Arc::clone(&self.exhaustion_events);
                let test_active = Arc::clone(&self.test_active);
                let allocation_counter = Arc::clone(&self.allocation_counter);

                thread::spawn(move || {
                    Self::exhaust_memory_allocations(
                        config,
                        allocations,
                        exhaustion_events,
                        test_active,
                        allocation_counter,
                    )
                });
            },
            ResourceType::DiskSpace => {
                let config = self.config.clone();
                let exhaustion_events = Arc::clone(&self.exhaustion_events);
                let test_active = Arc::clone(&self.test_active);

                thread::spawn(move || {
                    Self::exhaust_disk_space(
                        config,
                        exhaustion_events,
                        test_active,
                    )
                });
            },
            _ => {
                // Other resource types can be implemented as needed
                println!("Resource type {:?} not yet implemented", resource_type);
            }
        }

        Ok(())
    }

    /// Exhaust file descriptors
    fn exhaust_file_descriptors(
        config: ResourceExhaustionConfig,
        file_descriptors: Arc<Mutex<Vec<File>>>,
        exhaustion_events: Arc<Mutex<Vec<ResourceExhaustionEvent>>>,
        test_active: Arc<AtomicBool>,
        fd_counter: Arc<AtomicUsize>,
    ) {
        let start_time = Instant::now();
        
        while test_active.load(Ordering::SeqCst) {
            if start_time.elapsed().as_secs() >= config.max_test_duration_secs {
                break;
            }

            let current_fd_count = fd_counter.load(Ordering::SeqCst);
            if current_fd_count >= config.max_file_descriptors {
                Self::record_exhaustion_event(
                    &exhaustion_events,
                    ResourceType::FileDescriptors,
                    ExhaustionSeverity::Critical,
                    current_fd_count as f64,
                    config.max_file_descriptors as f64,
                    "File descriptor limit reached".to_string(),
                );
                break;
            }

            // Try to open a temporary file
            match Self::create_temporary_file() {
                Ok(file) => {
                    if let Ok(mut fds) = file_descriptors.lock() {
                        fds.push(file);
                        fd_counter.fetch_add(1, Ordering::SeqCst);
                    }
                },
                Err(e) => {
                    Self::record_exhaustion_event(
                        &exhaustion_events,
                        ResourceType::FileDescriptors,
                        ExhaustionSeverity::Failure,
                        current_fd_count as f64,
                        config.max_file_descriptors as f64,
                        format!("Failed to open file: {}", e),
                    );
                    break;
                }
            }

            thread::sleep(Duration::from_millis(10)); // Small delay between attempts
        }
    }

    /// Create a temporary file for testing
    fn create_temporary_file() -> io::Result<File> {
        let temp_file_path = format!("/tmp/resource_test_{}", std::process::id());
        OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(temp_file_path)
    }

    /// Exhaust threads
    fn exhaust_threads(
        config: ResourceExhaustionConfig,
        threads: Arc<Mutex<Vec<thread::JoinHandle<()>>>>,
        exhaustion_events: Arc<Mutex<Vec<ResourceExhaustionEvent>>>,
        test_active: Arc<AtomicBool>,
        thread_counter: Arc<AtomicUsize>,
    ) {
        let start_time = Instant::now();
        
        while test_active.load(Ordering::SeqCst) {
            if start_time.elapsed().as_secs() >= config.max_test_duration_secs {
                break;
            }

            let current_thread_count = thread_counter.load(Ordering::SeqCst);
            if current_thread_count >= config.max_threads {
                Self::record_exhaustion_event(
                    &exhaustion_events,
                    ResourceType::Threads,
                    ExhaustionSeverity::Critical,
                    current_thread_count as f64,
                    config.max_threads as f64,
                    "Thread limit reached".to_string(),
                );
                break;
            }

            // Try to spawn a new thread
            match thread::Builder::new().spawn(|| {
                // Thread does minimal work and sleeps
                thread::sleep(Duration::from_secs(30));
            }) {
                Ok(handle) => {
                    if let Ok(mut thread_handles) = threads.lock() {
                        thread_handles.push(handle);
                        thread_counter.fetch_add(1, Ordering::SeqCst);
                    }
                },
                Err(e) => {
                    Self::record_exhaustion_event(
                        &exhaustion_events,
                        ResourceType::Threads,
                        ExhaustionSeverity::Failure,
                        current_thread_count as f64,
                        config.max_threads as f64,
                        format!("Failed to spawn thread: {}", e),
                    );
                    break;
                }
            }

            thread::sleep(Duration::from_millis(50)); // Small delay between spawns
        }
    }

    /// Exhaust network connections
    fn exhaust_network_connections(
        config: ResourceExhaustionConfig,
        connections: Arc<Mutex<Vec<TcpStream>>>,
        exhaustion_events: Arc<Mutex<Vec<ResourceExhaustionEvent>>>,
        test_active: Arc<AtomicBool>,
        connection_counter: Arc<AtomicUsize>,
    ) {
        let start_time = Instant::now();
        
        // Start a simple TCP server for testing
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => listener,
            Err(e) => {
                Self::record_exhaustion_event(
                    &exhaustion_events,
                    ResourceType::NetworkConnections,
                    ExhaustionSeverity::Failure,
                    0.0,
                    0.0,
                    format!("Failed to create test server: {}", e),
                );
                return;
            }
        };

        let server_addr = listener.local_addr().unwrap();
        
        // Spawn server thread
        let server_active = Arc::clone(&test_active);
        thread::spawn(move || {
            while server_active.load(Ordering::SeqCst) {
                if let Ok((stream, _)) = listener.accept() {
                    // Handle connection briefly
                    drop(stream);
                }
            }
        });

        while test_active.load(Ordering::SeqCst) {
            if start_time.elapsed().as_secs() >= config.max_test_duration_secs {
                break;
            }

            let current_connection_count = connection_counter.load(Ordering::SeqCst);
            if current_connection_count >= config.max_network_connections {
                Self::record_exhaustion_event(
                    &exhaustion_events,
                    ResourceType::NetworkConnections,
                    ExhaustionSeverity::Critical,
                    current_connection_count as f64,
                    config.max_network_connections as f64,
                    "Network connection limit reached".to_string(),
                );
                break;
            }

            // Try to open a new connection
            match TcpStream::connect(server_addr) {
                Ok(stream) => {
                    if let Ok(mut conns) = connections.lock() {
                        conns.push(stream);
                        connection_counter.fetch_add(1, Ordering::SeqCst);
                    }
                },
                Err(e) => {
                    Self::record_exhaustion_event(
                        &exhaustion_events,
                        ResourceType::NetworkConnections,
                        ExhaustionSeverity::Failure,
                        current_connection_count as f64,
                        config.max_network_connections as f64,
                        format!("Failed to open connection: {}", e),
                    );
                    break;
                }
            }

            thread::sleep(Duration::from_millis(5)); // Small delay between connections
        }
    }

    /// Exhaust memory allocations
    fn exhaust_memory_allocations(
        config: ResourceExhaustionConfig,
        allocations: Arc<Mutex<Vec<Vec<u8>>>>,
        exhaustion_events: Arc<Mutex<Vec<ResourceExhaustionEvent>>>,
        test_active: Arc<AtomicBool>,
        allocation_counter: Arc<AtomicUsize>,
    ) {
        let start_time = Instant::now();
        let allocation_size = 1024 * 1024; // 1MB per allocation
        
        while test_active.load(Ordering::SeqCst) {
            if start_time.elapsed().as_secs() >= config.max_test_duration_secs {
                break;
            }

            let current_allocation_count = allocation_counter.load(Ordering::SeqCst);
            if current_allocation_count >= config.max_memory_allocations {
                Self::record_exhaustion_event(
                    &exhaustion_events,
                    ResourceType::MemoryAllocations,
                    ExhaustionSeverity::Critical,
                    current_allocation_count as f64,
                    config.max_memory_allocations as f64,
                    "Memory allocation limit reached".to_string(),
                );
                break;
            }

            // Try to allocate memory
            let allocation = vec![0u8; allocation_size];
            
            if let Ok(mut allocs) = allocations.lock() {
                allocs.push(allocation);
                allocation_counter.fetch_add(1, Ordering::SeqCst);
            }

            thread::sleep(Duration::from_millis(20)); // Small delay between allocations
        }
    }

    /// Exhaust disk space (simulated)
    fn exhaust_disk_space(
        config: ResourceExhaustionConfig,
        exhaustion_events: Arc<Mutex<Vec<ResourceExhaustionEvent>>>,
        test_active: Arc<AtomicBool>,
    ) {
        let start_time = Instant::now();
        let mut files_created = 0;
        let file_size_bytes = config.test_file_size_kb * 1024;
        
        while test_active.load(Ordering::SeqCst) {
            if start_time.elapsed().as_secs() >= config.max_test_duration_secs {
                break;
            }

            let temp_file_path = format!("/tmp/disk_test_{}_{}", std::process::id(), files_created);
            
            match std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&temp_file_path)
            {
                Ok(mut file) => {
                    let data = vec![0u8; file_size_bytes];
                    match file.write_all(&data) {
                        Ok(_) => {
                            files_created += 1;
                            if files_created >= 100 { // Limit to prevent actual disk filling
                                Self::record_exhaustion_event(
                                    &exhaustion_events,
                                    ResourceType::DiskSpace,
                                    ExhaustionSeverity::Warning,
                                    (files_created * config.test_file_size_kb) as f64,
                                    (100 * config.test_file_size_kb) as f64,
                                    "Disk space test limit reached".to_string(),
                                );
                                break;
                            }
                        },
                        Err(e) => {
                            Self::record_exhaustion_event(
                                &exhaustion_events,
                                ResourceType::DiskSpace,
                                ExhaustionSeverity::Failure,
                                (files_created * config.test_file_size_kb) as f64,
                                0.0,
                                format!("Failed to write to disk: {}", e),
                            );
                            break;
                        }
                    }
                },
                Err(e) => {
                    Self::record_exhaustion_event(
                        &exhaustion_events,
                        ResourceType::DiskSpace,
                        ExhaustionSeverity::Failure,
                        (files_created * config.test_file_size_kb) as f64,
                        0.0,
                        format!("Failed to create file: {}", e),
                    );
                    break;
                }
            }

            thread::sleep(Duration::from_millis(100)); // Delay between file creations
        }

        // Cleanup test files
        for i in 0..files_created {
            let temp_file_path = format!("/tmp/disk_test_{}_{}", std::process::id(), i);
            let _ = std::fs::remove_file(temp_file_path);
        }
    }

    /// Record resource exhaustion event
    fn record_exhaustion_event(
        exhaustion_events: &Arc<Mutex<Vec<ResourceExhaustionEvent>>>,
        resource_type: ResourceType,
        severity: ExhaustionSeverity,
        trigger_value: f64,
        threshold_exceeded: f64,
        error_message: String,
    ) {
        let event = ResourceExhaustionEvent {
            timestamp: Instant::now(),
            resource_type,
            severity,
            trigger_value,
            threshold_exceeded,
            error_message,
            recovery_action: None,
            recovery_successful: None,
            recovery_time: None,
        };

        if let Ok(mut events) = exhaustion_events.lock() {
            events.push(event);
        }
    }

    /// Run resource monitoring thread
    fn run_resource_monitoring_thread(
        config: ResourceExhaustionConfig,
        resource_history: Arc<Mutex<VecDeque<ResourceUsageSnapshot>>>,
        exhaustion_events: Arc<Mutex<Vec<ResourceExhaustionEvent>>>,
        test_active: Arc<AtomicBool>,
        fd_counter: Arc<AtomicUsize>,
        thread_counter: Arc<AtomicUsize>,
        connection_counter: Arc<AtomicUsize>,
        allocation_counter: Arc<AtomicUsize>,
    ) {
        while test_active.load(Ordering::SeqCst) {
            let snapshot = Self::capture_resource_snapshot(
                &fd_counter,
                &thread_counter,
                &connection_counter,
                &allocation_counter,
                &config,
            );

            // Check for threshold violations
            Self::check_resource_thresholds(&snapshot, &config, &exhaustion_events);

            // Store snapshot
            if let Ok(mut history) = resource_history.lock() {
                history.push_back(snapshot);
                if history.len() > 1000 {
                    history.pop_front();
                }
            }

            thread::sleep(Duration::from_millis(config.monitoring_interval_ms));
        }
    }

    /// Capture current resource usage snapshot
    fn capture_resource_snapshot(
        fd_counter: &Arc<AtomicUsize>,
        thread_counter: &Arc<AtomicUsize>,
        connection_counter: &Arc<AtomicUsize>,
        allocation_counter: &Arc<AtomicUsize>,
        config: &ResourceExhaustionConfig,
    ) -> ResourceUsageSnapshot {
        let fd_count = fd_counter.load(Ordering::SeqCst);
        let thread_count = thread_counter.load(Ordering::SeqCst);
        let connection_count = connection_counter.load(Ordering::SeqCst);
        let allocation_count = allocation_counter.load(Ordering::SeqCst);

        // Calculate resource pressure level
        let fd_pressure = fd_count as f64 / config.max_file_descriptors as f64;
        let thread_pressure = thread_count as f64 / config.max_threads as f64;
        let connection_pressure = connection_count as f64 / config.max_network_connections as f64;
        let allocation_pressure = allocation_count as f64 / config.max_memory_allocations as f64;
        
        let resource_pressure = (fd_pressure + thread_pressure + connection_pressure + allocation_pressure) / 4.0;

        ResourceUsageSnapshot {
            timestamp: Instant::now(),
            open_file_descriptors: fd_count,
            active_threads: thread_count,
            network_connections: connection_count,
            memory_usage_mb: (allocation_count * 1024 * 1024) as f64 / (1024.0 * 1024.0), // Convert to MB
            disk_usage_mb: 0.0, // Simulated
            cpu_utilization_percent: Self::get_simulated_cpu_usage(),
            system_load_average: Self::get_simulated_load_average(),
            available_file_descriptors: config.max_file_descriptors.saturating_sub(fd_count),
            resource_pressure_level: resource_pressure.min(1.0),
        }
    }

    /// Get simulated CPU usage
    fn get_simulated_cpu_usage() -> f64 {
        // Simulate CPU usage based on time and resource pressure
        let base_cpu = 20.0;
        let time_factor = (Instant::now().elapsed().as_secs() % 60) as f64 / 60.0;
        base_cpu + (time_factor * 30.0)
    }

    /// Get simulated system load average
    fn get_simulated_load_average() -> f64 {
        // Simulate system load
        let base_load = 1.0;
        let time_factor = (Instant::now().elapsed().as_secs() % 120) as f64 / 120.0;
        base_load + (time_factor * 2.0)
    }

    /// Check resource thresholds and record events
    fn check_resource_thresholds(
        snapshot: &ResourceUsageSnapshot,
        config: &ResourceExhaustionConfig,
        exhaustion_events: &Arc<Mutex<Vec<ResourceExhaustionEvent>>>,
    ) {
        let thresholds = &config.exhaustion_thresholds;

        // Check file descriptor threshold
        let fd_usage_ratio = snapshot.open_file_descriptors as f64 / config.max_file_descriptors as f64;
        if fd_usage_ratio >= thresholds.fd_warning_threshold {
            Self::record_exhaustion_event(
                exhaustion_events,
                ResourceType::FileDescriptors,
                ExhaustionSeverity::Warning,
                snapshot.open_file_descriptors as f64,
                thresholds.fd_warning_threshold * config.max_file_descriptors as f64,
                format!("File descriptor usage at {:.1}%", fd_usage_ratio * 100.0),
            );
        }

        // Check thread threshold
        let thread_usage_ratio = snapshot.active_threads as f64 / config.max_threads as f64;
        if thread_usage_ratio >= thresholds.thread_warning_threshold {
            Self::record_exhaustion_event(
                exhaustion_events,
                ResourceType::Threads,
                ExhaustionSeverity::Warning,
                snapshot.active_threads as f64,
                thresholds.thread_warning_threshold * config.max_threads as f64,
                format!("Thread usage at {:.1}%", thread_usage_ratio * 100.0),
            );
        }

        // Check memory threshold
        if snapshot.memory_usage_mb >= thresholds.memory_warning_threshold as f64 {
            Self::record_exhaustion_event(
                exhaustion_events,
                ResourceType::MemoryAllocations,
                ExhaustionSeverity::Warning,
                snapshot.memory_usage_mb,
                thresholds.memory_warning_threshold as f64,
                format!("Memory usage at {:.1} MB", snapshot.memory_usage_mb),
            );
        }

        // Check CPU threshold
        if snapshot.cpu_utilization_percent >= thresholds.cpu_warning_threshold {
            Self::record_exhaustion_event(
                exhaustion_events,
                ResourceType::CpuTime,
                ExhaustionSeverity::Warning,
                snapshot.cpu_utilization_percent,
                thresholds.cpu_warning_threshold,
                format!("CPU usage at {:.1}%", snapshot.cpu_utilization_percent),
            );
        }
    }

    /// Trigger recovery for specific resource type
    pub fn trigger_recovery(&self, resource_type: ResourceType) -> Result<bool> {
        let recovery_start = Instant::now();
        let mut recovery_successful = false;

        match resource_type {
            ResourceType::FileDescriptors => {
                if let Ok(mut fds) = self.active_file_descriptors.lock() {
                    let initial_count = fds.len();
                    fds.clear(); // Close all file descriptors
                    self.fd_counter.store(0, Ordering::SeqCst);
                    recovery_successful = fds.is_empty();
                    
                    self.record_recovery_event(
                        resource_type,
                        recovery_successful,
                        recovery_start.elapsed(),
                        format!("Closed {} file descriptors", initial_count),
                    );
                }
            },
            ResourceType::Threads => {
                // Note: We can't forcibly join threads, so we just mark for cleanup
                if let Ok(mut threads) = self.active_threads.lock() {
                    let initial_count = threads.len();
                    threads.clear(); // Remove handles (threads will finish naturally)
                    self.thread_counter.store(0, Ordering::SeqCst);
                    recovery_successful = true;
                    
                    self.record_recovery_event(
                        resource_type,
                        recovery_successful,
                        recovery_start.elapsed(),
                        format!("Released {} thread handles", initial_count),
                    );
                }
            },
            ResourceType::NetworkConnections => {
                if let Ok(mut connections) = self.active_network_connections.lock() {
                    let initial_count = connections.len();
                    connections.clear(); // Close all connections
                    self.connection_counter.store(0, Ordering::SeqCst);
                    recovery_successful = connections.is_empty();
                    
                    self.record_recovery_event(
                        resource_type,
                        recovery_successful,
                        recovery_start.elapsed(),
                        format!("Closed {} network connections", initial_count),
                    );
                }
            },
            ResourceType::MemoryAllocations => {
                if let Ok(mut allocations) = self.active_memory_allocations.lock() {
                    let initial_count = allocations.len();
                    allocations.clear(); // Free all allocations
                    self.allocation_counter.store(0, Ordering::SeqCst);
                    recovery_successful = allocations.is_empty();
                    
                    self.record_recovery_event(
                        resource_type,
                        recovery_successful,
                        recovery_start.elapsed(),
                        format!("Freed {} memory allocations", initial_count),
                    );
                }
            },
            _ => {
                recovery_successful = false;
            }
        }

        Ok(recovery_successful)
    }

    /// Record recovery event
    fn record_recovery_event(
        &self,
        resource_type: ResourceType,
        successful: bool,
        recovery_time: Duration,
        action_description: String,
    ) {
        if let Ok(mut events) = self.exhaustion_events.lock() {
            // Find the most recent exhaustion event for this resource type
            if let Some(event) = events.iter_mut()
                .filter(|e| e.resource_type == resource_type)
                .last() {
                event.recovery_action = Some(action_description);
                event.recovery_successful = Some(successful);
                event.recovery_time = Some(recovery_time);
            }
        }
    }

    /// Get comprehensive test results
    pub fn get_exhaustion_results(&self) -> Result<ResourceExhaustionResults> {
        let resource_history = self.resource_usage_history.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock resource history".to_string())
        })?;

        let exhaustion_events = self.exhaustion_events.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock exhaustion events".to_string())
        })?;

        // Calculate test duration
        let test_duration = if let (Some(first), Some(last)) = (resource_history.front(), resource_history.back()) {
            last.timestamp.duration_since(first.timestamp)
        } else {
            Duration::from_secs(0)
        };

        // Count exhaustion events by type
        let mut exhaustion_events_by_type = HashMap::new();
        for event in exhaustion_events.iter() {
            *exhaustion_events_by_type.entry(event.resource_type.clone()).or_insert(0) += 1;
        }

        // Calculate peak and average resource usage
        let (peak_usage, average_usage) = Self::calculate_resource_usage_statistics(&resource_history);

        // Calculate recovery statistics
        let recovery_stats = Self::calculate_recovery_statistics(&exhaustion_events);

        // Calculate performance impact
        let performance_impact = Self::calculate_performance_impact(&resource_history);

        // Calculate stability metrics
        let stability_metrics = Self::calculate_stability_metrics(&exhaustion_events, &resource_history);

        // Generate system limit recommendations
        let recommended_limits = Self::generate_limit_recommendations(&peak_usage, &exhaustion_events);

        Ok(ResourceExhaustionResults {
            test_duration,
            exhaustion_events_by_type,
            peak_resource_usage: peak_usage,
            average_resource_usage: average_usage,
            recovery_statistics: recovery_stats,
            performance_impact,
            stability_metrics,
            recommended_limits,
        })
    }

    /// Calculate peak and average resource usage
    fn calculate_resource_usage_statistics(
        history: &VecDeque<ResourceUsageSnapshot>,
    ) -> (ResourceUsageSnapshot, ResourceUsageSnapshot) {
        if history.is_empty() {
            let empty_snapshot = ResourceUsageSnapshot {
                timestamp: Instant::now(),
                open_file_descriptors: 0,
                active_threads: 0,
                network_connections: 0,
                memory_usage_mb: 0.0,
                disk_usage_mb: 0.0,
                cpu_utilization_percent: 0.0,
                system_load_average: 0.0,
                available_file_descriptors: 0,
                resource_pressure_level: 0.0,
            };
            return (empty_snapshot.clone(), empty_snapshot);
        }

        let mut peak = history[0].clone();
        let mut totals = ResourceUsageSnapshot {
            timestamp: Instant::now(),
            open_file_descriptors: 0,
            active_threads: 0,
            network_connections: 0,
            memory_usage_mb: 0.0,
            disk_usage_mb: 0.0,
            cpu_utilization_percent: 0.0,
            system_load_average: 0.0,
            available_file_descriptors: 0,
            resource_pressure_level: 0.0,
        };

        for snapshot in history.iter() {
            // Update peak values
            if snapshot.open_file_descriptors > peak.open_file_descriptors {
                peak.open_file_descriptors = snapshot.open_file_descriptors;
            }
            if snapshot.active_threads > peak.active_threads {
                peak.active_threads = snapshot.active_threads;
            }
            if snapshot.network_connections > peak.network_connections {
                peak.network_connections = snapshot.network_connections;
            }
            if snapshot.memory_usage_mb > peak.memory_usage_mb {
                peak.memory_usage_mb = snapshot.memory_usage_mb;
            }
            if snapshot.cpu_utilization_percent > peak.cpu_utilization_percent {
                peak.cpu_utilization_percent = snapshot.cpu_utilization_percent;
            }
            if snapshot.resource_pressure_level > peak.resource_pressure_level {
                peak.resource_pressure_level = snapshot.resource_pressure_level;
            }

            // Accumulate for averages
            totals.open_file_descriptors += snapshot.open_file_descriptors;
            totals.active_threads += snapshot.active_threads;
            totals.network_connections += snapshot.network_connections;
            totals.memory_usage_mb += snapshot.memory_usage_mb;
            totals.disk_usage_mb += snapshot.disk_usage_mb;
            totals.cpu_utilization_percent += snapshot.cpu_utilization_percent;
            totals.system_load_average += snapshot.system_load_average;
            totals.available_file_descriptors += snapshot.available_file_descriptors;
            totals.resource_pressure_level += snapshot.resource_pressure_level;
        }

        // Calculate averages
        let count = history.len() as f64;
        let average = ResourceUsageSnapshot {
            timestamp: Instant::now(),
            open_file_descriptors: (totals.open_file_descriptors as f64 / count) as usize,
            active_threads: (totals.active_threads as f64 / count) as usize,
            network_connections: (totals.network_connections as f64 / count) as usize,
            memory_usage_mb: totals.memory_usage_mb / count,
            disk_usage_mb: totals.disk_usage_mb / count,
            cpu_utilization_percent: totals.cpu_utilization_percent / count,
            system_load_average: totals.system_load_average / count,
            available_file_descriptors: (totals.available_file_descriptors as f64 / count) as usize,
            resource_pressure_level: totals.resource_pressure_level / count,
        };

        (peak, average)
    }

    /// Calculate recovery statistics
    fn calculate_recovery_statistics(events: &[ResourceExhaustionEvent]) -> RecoveryStatistics {
        let recovery_events: Vec<&ResourceExhaustionEvent> = events.iter()
            .filter(|e| e.recovery_action.is_some())
            .collect();

        let total_attempts = recovery_events.len();
        let successful = recovery_events.iter()
            .filter(|e| e.recovery_successful == Some(true))
            .count();
        let failed = total_attempts - successful;

        let recovery_times: Vec<Duration> = recovery_events.iter()
            .filter_map(|e| e.recovery_time)
            .collect();

        let (avg_time, fastest_time, slowest_time) = if recovery_times.is_empty() {
            (Duration::from_secs(0), Duration::from_secs(0), Duration::from_secs(0))
        } else {
            let total_ms: u64 = recovery_times.iter().map(|d| d.as_millis() as u64).sum();
            let avg = Duration::from_millis(total_ms / recovery_times.len() as u64);
            let fastest = *recovery_times.iter().min().unwrap_or(&Duration::from_secs(0));
            let slowest = *recovery_times.iter().max().unwrap_or(&Duration::from_secs(0));
            (avg, fastest, slowest)
        };

        let success_rate = if total_attempts > 0 {
            successful as f64 / total_attempts as f64
        } else {
            0.0
        };

        RecoveryStatistics {
            total_recovery_attempts: total_attempts,
            successful_recoveries: successful,
            failed_recoveries: failed,
            average_recovery_time: avg_time,
            fastest_recovery_time: fastest_time,
            slowest_recovery_time: slowest_time,
            recovery_success_rate: success_rate,
        }
    }

    /// Calculate performance impact analysis
    fn calculate_performance_impact(history: &VecDeque<ResourceUsageSnapshot>) -> PerformanceImpactAnalysis {
        if history.is_empty() {
            return PerformanceImpactAnalysis {
                baseline_metrics: PerformanceMetrics {
                    average_response_time_ms: 0.0,
                    throughput_ops_per_sec: 0.0,
                    error_rate_percent: 0.0,
                    success_rate_percent: 100.0,
                },
                under_pressure_metrics: PerformanceMetrics {
                    average_response_time_ms: 0.0,
                    throughput_ops_per_sec: 0.0,
                    error_rate_percent: 0.0,
                    success_rate_percent: 100.0,
                },
                performance_degradation_percent: 0.0,
                most_impacted_operations: Vec::new(),
                resource_efficiency_scores: HashMap::new(),
            };
        }

        // Split history into baseline (first 25%) and under pressure (last 25%)
        let quarter_point = history.len() / 4;
        let baseline_samples = &history.iter().take(quarter_point).cloned().collect::<Vec<_>>();
        let pressure_samples = &history.iter().skip(history.len() - quarter_point).cloned().collect::<Vec<_>>();

        let baseline_metrics = Self::calculate_performance_metrics(baseline_samples);
        let pressure_metrics = Self::calculate_performance_metrics(pressure_samples);

        let degradation = if baseline_metrics.throughput_ops_per_sec > 0.0 {
            ((baseline_metrics.throughput_ops_per_sec - pressure_metrics.throughput_ops_per_sec) /
             baseline_metrics.throughput_ops_per_sec) * 100.0
        } else {
            0.0
        };

        let impacted_operations = vec![
            "File operations".to_string(),
            "Thread creation".to_string(),
            "Network connections".to_string(),
            "Memory allocation".to_string(),
        ];

        let mut efficiency_scores = HashMap::new();
        efficiency_scores.insert(ResourceType::FileDescriptors, 0.8);
        efficiency_scores.insert(ResourceType::Threads, 0.7);
        efficiency_scores.insert(ResourceType::NetworkConnections, 0.9);
        efficiency_scores.insert(ResourceType::MemoryAllocations, 0.6);

        PerformanceImpactAnalysis {
            baseline_metrics,
            under_pressure_metrics: pressure_metrics,
            performance_degradation_percent: degradation,
            most_impacted_operations: impacted_operations,
            resource_efficiency_scores: efficiency_scores,
        }
    }

    /// Calculate performance metrics for a set of snapshots
    fn calculate_performance_metrics(snapshots: &[ResourceUsageSnapshot]) -> PerformanceMetrics {
        if snapshots.is_empty() {
            return PerformanceMetrics {
                average_response_time_ms: 0.0,
                throughput_ops_per_sec: 0.0,
                error_rate_percent: 0.0,
                success_rate_percent: 100.0,
            };
        }

        let avg_pressure = snapshots.iter().map(|s| s.resource_pressure_level).sum::<f64>() / snapshots.len() as f64;
        let avg_cpu = snapshots.iter().map(|s| s.cpu_utilization_percent).sum::<f64>() / snapshots.len() as f64;

        // Simulate performance metrics based on resource pressure
        let base_response_time = 10.0; // 10ms baseline
        let response_time = base_response_time * (1.0 + avg_pressure * 2.0);

        let base_throughput = 100.0; // 100 ops/sec baseline
        let throughput = base_throughput * (1.0 - avg_pressure * 0.5);

        let error_rate = avg_pressure * 10.0; // Up to 10% error rate under pressure
        let success_rate = 100.0 - error_rate;

        PerformanceMetrics {
            average_response_time_ms: response_time,
            throughput_ops_per_sec: throughput,
            error_rate_percent: error_rate,
            success_rate_percent: success_rate,
        }
    }

    /// Calculate system stability metrics
    fn calculate_stability_metrics(
        events: &[ResourceExhaustionEvent],
        history: &VecDeque<ResourceUsageSnapshot>,
    ) -> SystemStabilityMetrics {
        let system_crashes = events.iter()
            .filter(|e| matches!(e.severity, ExhaustionSeverity::Failure))
            .count();

        let resource_leaks = events.iter()
            .filter(|e| e.error_message.contains("leak"))
            .count();

        let uptime_percent = if history.is_empty() {
            100.0
        } else {
            let total_samples = history.len();
            let failed_samples = history.iter()
                .filter(|s| s.resource_pressure_level > 0.9)
                .count();
            ((total_samples - failed_samples) as f64 / total_samples as f64) * 100.0
        };

        let cleanup_effectiveness = if events.is_empty() {
            100.0
        } else {
            let total_events = events.len();
            let successfully_recovered = events.iter()
                .filter(|e| e.recovery_successful == Some(true))
                .count();
            (successfully_recovered as f64 / total_events as f64) * 100.0
        };

        let stability_score = (uptime_percent + cleanup_effectiveness) / 200.0; // Normalize to 0-1

        SystemStabilityMetrics {
            system_crashes,
            resource_leaks_detected: resource_leaks,
            system_uptime_percent: uptime_percent,
            cleanup_effectiveness_percent: cleanup_effectiveness,
            overall_stability_score: stability_score,
        }
    }

    /// Generate system limit recommendations
    fn generate_limit_recommendations(
        peak_usage: &ResourceUsageSnapshot,
        events: &[ResourceExhaustionEvent],
    ) -> SystemLimitRecommendations {
        let safety_margin = 1.5; // 50% safety margin

        let recommended_fd_limit = ((peak_usage.open_file_descriptors as f64 * safety_margin) as usize).max(1024);
        let recommended_thread_limit = ((peak_usage.active_threads as f64 * safety_margin) as usize).max(100);
        let recommended_memory_limit = ((peak_usage.memory_usage_mb * safety_margin) as usize).max(512);
        let recommended_connection_limit = ((peak_usage.network_connections as f64 * safety_margin) as usize).max(256);

        let mut rationale = Vec::new();
        rationale.push(format!("Peak file descriptor usage: {}", peak_usage.open_file_descriptors));
        rationale.push(format!("Peak thread usage: {}", peak_usage.active_threads));
        rationale.push(format!("Peak memory usage: {:.1} MB", peak_usage.memory_usage_mb));
        rationale.push(format!("Peak connection usage: {}", peak_usage.network_connections));
        rationale.push(format!("Total exhaustion events: {}", events.len()));
        rationale.push("Recommendations include 50% safety margin".to_string());

        SystemLimitRecommendations {
            recommended_fd_limit,
            recommended_thread_limit,
            recommended_memory_limit_mb: recommended_memory_limit,
            recommended_connection_limit,
            safety_margin_percent: (safety_margin - 1.0) * 100.0,
            rationale,
        }
    }

    /// Get current resource usage
    pub fn get_current_resource_usage(&self) -> ResourceUsageSnapshot {
        Self::capture_resource_snapshot(
            &self.fd_counter,
            &self.thread_counter,
            &self.connection_counter,
            &self.allocation_counter,
            &self.config,
        )
    }

    /// Force cleanup of all resources
    pub fn force_cleanup_all_resources(&self) -> Result<()> {
        // Cleanup file descriptors
        self.trigger_recovery(ResourceType::FileDescriptors)?;
        
        // Cleanup threads
        self.trigger_recovery(ResourceType::Threads)?;
        
        // Cleanup network connections
        self.trigger_recovery(ResourceType::NetworkConnections)?;
        
        // Cleanup memory allocations
        self.trigger_recovery(ResourceType::MemoryAllocations)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_exhaustion_config_creation() {
        let config = ResourceExhaustionConfig::default();
        assert!(config.max_test_duration_secs > 0);
        assert!(!config.resource_types_to_test.is_empty());
        assert!(config.max_file_descriptors > 0);
        assert!(config.max_threads > 0);
    }

    #[test]
    fn test_resource_exhaustion_tester_creation() {
        let config = ResourceExhaustionConfig::default();
        let tester = ResourceExhaustionTester::new(config);
        
        assert!(!tester.test_active.load(Ordering::SeqCst));
        assert_eq!(tester.fd_counter.load(Ordering::SeqCst), 0);
        assert_eq!(tester.thread_counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_resource_usage_snapshot_creation() {
        let config = ResourceExhaustionConfig::default();
        let fd_counter = Arc::new(AtomicUsize::new(10));
        let thread_counter = Arc::new(AtomicUsize::new(5));
        let connection_counter = Arc::new(AtomicUsize::new(3));
        let allocation_counter = Arc::new(AtomicUsize::new(7));

        let snapshot = ResourceExhaustionTester::capture_resource_snapshot(
            &fd_counter,
            &thread_counter,
            &connection_counter,
            &allocation_counter,
            &config,
        );

        assert_eq!(snapshot.open_file_descriptors, 10);
        assert_eq!(snapshot.active_threads, 5);
        assert_eq!(snapshot.network_connections, 3);
        assert!(snapshot.resource_pressure_level >= 0.0);
        assert!(snapshot.resource_pressure_level <= 1.0);
    }

    #[tokio::test]
    async fn test_resource_exhaustion_integration() -> Result<()> {
        let config = ResourceExhaustionConfig {
            max_test_duration_secs: 2, // Short test
            monitoring_interval_ms: 100,
            max_file_descriptors: 10, // Low limit for testing
            max_threads: 5,
            max_network_connections: 5,
            max_memory_allocations: 5,
            enable_realtime_monitoring: true,
            resource_types_to_test: vec![ResourceType::MemoryAllocations], // Test only memory
            ..Default::default()
        };

        let tester = ResourceExhaustionTester::new(config);
        
        // Start test
        tester.start_exhaustion_test()?;
        
        // Wait for some operations
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Check current usage
        let usage = tester.get_current_resource_usage();
        assert!(usage.memory_usage_mb >= 0.0);
        
        // Test recovery
        let recovery_result = tester.trigger_recovery(ResourceType::MemoryAllocations)?;
        assert!(recovery_result);
        
        // Stop test
        tester.stop_exhaustion_test();
        
        // Get results
        tokio::time::sleep(Duration::from_millis(100)).await;
        let results = tester.get_exhaustion_results()?;
        
        assert!(results.test_duration.as_millis() > 0);
        assert!(!results.recommended_limits.rationale.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_recovery_statistics_calculation() {
        let events = vec![
            ResourceExhaustionEvent {
                timestamp: Instant::now(),
                resource_type: ResourceType::FileDescriptors,
                severity: ExhaustionSeverity::Failure,
                trigger_value: 100.0,
                threshold_exceeded: 100.0,
                error_message: "Test error".to_string(),
                recovery_action: Some("Test recovery".to_string()),
                recovery_successful: Some(true),
                recovery_time: Some(Duration::from_millis(100)),
            },
            ResourceExhaustionEvent {
                timestamp: Instant::now(),
                resource_type: ResourceType::Threads,
                severity: ExhaustionSeverity::Failure,
                trigger_value: 50.0,
                threshold_exceeded: 50.0,
                error_message: "Test error 2".to_string(),
                recovery_action: Some("Test recovery 2".to_string()),
                recovery_successful: Some(false),
                recovery_time: Some(Duration::from_millis(200)),
            },
        ];

        let stats = ResourceExhaustionTester::calculate_recovery_statistics(&events);
        
        assert_eq!(stats.total_recovery_attempts, 2);
        assert_eq!(stats.successful_recoveries, 1);
        assert_eq!(stats.failed_recoveries, 1);
        assert_eq!(stats.recovery_success_rate, 0.5);
        assert!(stats.average_recovery_time.as_millis() > 0);
    }

    #[test]
    fn test_system_limit_recommendations() {
        let peak_usage = ResourceUsageSnapshot {
            timestamp: Instant::now(),
            open_file_descriptors: 100,
            active_threads: 50,
            network_connections: 200,
            memory_usage_mb: 512.0,
            disk_usage_mb: 0.0,
            cpu_utilization_percent: 80.0,
            system_load_average: 2.0,
            available_file_descriptors: 924,
            resource_pressure_level: 0.8,
        };

        let events = vec![];
        
        let recommendations = ResourceExhaustionTester::generate_limit_recommendations(
            &peak_usage,
            &events,
        );

        assert!(recommendations.recommended_fd_limit >= 100);
        assert!(recommendations.recommended_thread_limit >= 50);
        assert!(recommendations.recommended_memory_limit_mb >= 512);
        assert!(recommendations.recommended_connection_limit >= 200);
        assert!(recommendations.safety_margin_percent > 0.0);
        assert!(!recommendations.rationale.is_empty());
    }
}