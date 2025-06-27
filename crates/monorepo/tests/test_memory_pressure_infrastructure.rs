//! Memory Pressure Testing Infrastructure
//!
//! This module implements comprehensive memory pressure testing capabilities for monorepo
//! operations, including memory leak detection, resource exhaustion simulation, and
//! out-of-memory condition handling.

use sublime_monorepo_tools::Result;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

/// Configuration for memory pressure testing
#[derive(Debug, Clone)]
pub struct MemoryPressureConfig {
    /// Memory monitoring interval in milliseconds
    pub monitoring_interval_ms: u64,
    /// Maximum test duration in seconds
    pub max_test_duration_secs: u64,
    /// Memory warning threshold in MB
    pub memory_warning_threshold_mb: usize,
    /// Memory critical threshold in MB
    pub memory_critical_threshold_mb: usize,
    /// Memory allocation rate limit (MB/sec)
    pub allocation_rate_limit_mb_per_sec: f64,
    /// Enable automatic memory cleanup
    pub enable_auto_cleanup: bool,
    /// Maximum memory growth rate before triggering alerts (MB/sec)
    pub max_memory_growth_rate: f64,
    /// Enable memory fragmentation analysis
    pub enable_fragmentation_analysis: bool,
    /// Sample buffer size for trend analysis
    pub sample_buffer_size: usize,
}

impl Default for MemoryPressureConfig {
    fn default() -> Self {
        Self {
            monitoring_interval_ms: 250, // 250ms intervals for high granularity
            max_test_duration_secs: 600,  // 10 minutes max
            memory_warning_threshold_mb: 1024, // 1GB warning
            memory_critical_threshold_mb: 2048, // 2GB critical
            allocation_rate_limit_mb_per_sec: 100.0, // 100MB/sec limit
            enable_auto_cleanup: true,
            max_memory_growth_rate: 50.0, // 50MB/sec growth rate alarm
            enable_fragmentation_analysis: true,
            sample_buffer_size: 1000,
        }
    }
}

/// Memory usage snapshot with detailed metrics
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    /// Timestamp when snapshot was taken
    pub timestamp: Instant,
    /// Resident Set Size in bytes
    pub rss_bytes: usize,
    /// Virtual Memory Size in bytes
    pub virtual_bytes: usize,
    /// Heap size in bytes (estimated)
    pub heap_bytes: usize,
    /// Stack size in bytes (estimated)
    pub stack_bytes: usize,
    /// Number of memory mapped files
    pub mapped_files_count: usize,
    /// Total mapped memory in bytes
    pub mapped_memory_bytes: usize,
    /// Memory allocation rate (bytes/sec)
    pub allocation_rate: f64,
    /// Memory deallocation rate (bytes/sec)
    pub deallocation_rate: f64,
    /// Estimated memory fragmentation percentage
    pub fragmentation_percent: f64,
    /// Number of active allocations
    pub active_allocations: usize,
}

impl MemorySnapshot {
    /// Capture current memory state
    pub fn capture() -> Self {
        let timestamp = Instant::now();
        
        // In a real implementation, these would use system APIs
        // For testing, we'll simulate realistic memory metrics
        let base_memory = Self::get_base_memory_usage();
        let time_factor = timestamp.elapsed().as_secs_f64();
        
        Self {
            timestamp,
            rss_bytes: base_memory + Self::simulate_rss_growth(time_factor),
            virtual_bytes: base_memory * 2 + Self::simulate_virtual_growth(time_factor),
            heap_bytes: (base_memory as f64 * 0.7) as usize + Self::simulate_heap_growth(time_factor),
            stack_bytes: 8 * 1024 * 1024, // 8MB stack (typical)
            mapped_files_count: Self::simulate_mapped_files_count(),
            mapped_memory_bytes: Self::simulate_mapped_memory(time_factor),
            allocation_rate: Self::simulate_allocation_rate(time_factor),
            deallocation_rate: Self::simulate_deallocation_rate(time_factor),
            fragmentation_percent: Self::simulate_fragmentation_percent(time_factor),
            active_allocations: Self::simulate_active_allocations(time_factor),
        }
    }
    
    /// Calculate memory usage in MB
    pub fn total_memory_mb(&self) -> f64 {
        self.rss_bytes as f64 / (1024.0 * 1024.0)
    }
    
    /// Calculate memory efficiency (allocated vs requested)
    pub fn memory_efficiency(&self) -> f64 {
        if self.virtual_bytes > 0 {
            self.rss_bytes as f64 / self.virtual_bytes as f64
        } else {
            1.0
        }
    }
    
    /// Check if memory usage indicates a potential leak
    pub fn indicates_memory_leak(&self, baseline: &MemorySnapshot) -> bool {
        let growth = self.rss_bytes as f64 - baseline.rss_bytes as f64;
        let duration = self.timestamp.duration_since(baseline.timestamp).as_secs_f64();
        
        if duration > 0.0 {
            let growth_rate = growth / duration; // bytes per second
            let mb_per_sec = growth_rate / (1024.0 * 1024.0);
            mb_per_sec > 10.0 && self.active_allocations > baseline.active_allocations * 2
        } else {
            false
        }
    }
    
    // Simulation methods for realistic memory behavior
    fn get_base_memory_usage() -> usize {
        200 * 1024 * 1024 // 200MB base usage
    }
    
    fn simulate_rss_growth(time_factor: f64) -> usize {
        let growth = (time_factor * 5.0).exp() - 1.0; // Exponential growth simulation
        let noise = (time_factor * 3.0).sin() * 1024.0 * 1024.0; // ±1MB noise
        ((growth * 10.0 * 1024.0 * 1024.0) + noise) as usize
    }
    
    fn simulate_virtual_growth(time_factor: f64) -> usize {
        let growth = time_factor * 20.0 * 1024.0 * 1024.0; // Linear virtual memory growth
        growth as usize
    }
    
    fn simulate_heap_growth(time_factor: f64) -> usize {
        let growth = (time_factor * 0.1).sinh() * 15.0 * 1024.0 * 1024.0; // Hyperbolic sine growth
        growth as usize
    }
    
    fn simulate_mapped_files_count() -> usize {
        use std::time::{SystemTime, UNIX_EPOCH};
        let time_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as f64;
        ((time_ms / 1000.0).sin().abs() * 50.0 + 10.0) as usize
    }
    
    fn simulate_mapped_memory(time_factor: f64) -> usize {
        let base = 50.0 * 1024.0 * 1024.0; // 50MB base
        let growth = time_factor * 5.0 * 1024.0 * 1024.0; // 5MB per second growth
        (base + growth) as usize
    }
    
    fn simulate_allocation_rate(time_factor: f64) -> f64 {
        let base_rate = 10.0 * 1024.0 * 1024.0; // 10MB/sec base
        let spike = (time_factor * 2.0).sin().abs() * 20.0 * 1024.0 * 1024.0; // Variable spikes
        base_rate + spike
    }
    
    fn simulate_deallocation_rate(time_factor: f64) -> f64 {
        let allocation_rate = Self::simulate_allocation_rate(time_factor);
        allocation_rate * 0.8 // 80% of allocations are freed (20% leak simulation)
    }
    
    fn simulate_fragmentation_percent(time_factor: f64) -> f64 {
        let base_fragmentation = 5.0; // 5% base fragmentation
        let growth = (time_factor * 0.05).exp() - 1.0; // Gradual fragmentation increase
        (base_fragmentation + growth * 15.0).min(50.0) // Cap at 50%
    }
    
    fn simulate_active_allocations(time_factor: f64) -> usize {
        let base = 10000.0; // 10k base allocations
        let growth = time_factor * 1000.0; // 1k new allocations per second
        let leak_factor = (time_factor * 0.1).exp(); // Exponential leak simulation
        (base + growth * leak_factor) as usize
    }
}

/// Memory pressure alert types
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryPressureAlert {
    /// Memory usage approaching warning threshold
    MemoryWarning {
        current_mb: f64,
        threshold_mb: f64,
    },
    /// Memory usage exceeded critical threshold
    MemoryCritical {
        current_mb: f64,
        threshold_mb: f64,
    },
    /// Memory leak detected
    MemoryLeak {
        growth_rate_mb_per_sec: f64,
        confidence: f64,
    },
    /// High memory fragmentation detected
    HighFragmentation {
        fragmentation_percent: f64,
    },
    /// Excessive allocation rate
    ExcessiveAllocation {
        rate_mb_per_sec: f64,
        limit_mb_per_sec: f64,
    },
    /// Out of memory condition imminent
    OutOfMemoryImminent {
        estimated_time_to_oom: Duration,
    },
    /// Memory pressure causing performance degradation
    PerformanceDegradation {
        efficiency_percent: f64,
    },
}

/// Memory pressure test state
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryPressureState {
    /// Normal operation - no pressure detected
    Normal,
    /// Warning level - elevated memory usage
    Warning,
    /// Critical level - high memory pressure
    Critical,
    /// Emergency level - OOM imminent
    Emergency,
    /// Recovery mode - cleanup in progress
    Recovery,
}

/// Memory allocation simulator for testing
#[derive(Debug)]
pub struct MemoryAllocationSimulator {
    /// Allocated memory blocks
    allocated_blocks: Arc<RwLock<Vec<Vec<u8>>>>,
    /// Current allocation size in bytes
    current_allocation: Arc<AtomicUsize>,
    /// Whether simulation is active
    is_active: Arc<AtomicBool>,
    /// Allocation pattern configuration
    allocation_pattern: AllocationPattern,
}

/// Different memory allocation patterns for testing
#[derive(Debug, Clone)]
pub enum AllocationPattern {
    /// Steady allocation rate
    Steady { size_mb: usize, interval_ms: u64 },
    /// Exponential growth pattern
    Exponential { initial_mb: usize, growth_factor: f64 },
    /// Spike pattern with periodic bursts
    Spike { base_mb: usize, spike_mb: usize, spike_interval_secs: u64 },
    /// Fragmentation pattern (many small allocations)
    Fragmentation { block_size_kb: usize, blocks_per_sec: usize },
    /// Memory leak simulation
    Leak { leak_rate_mb_per_sec: f64 },
}

impl MemoryAllocationSimulator {
    /// Create new memory allocation simulator
    pub fn new(pattern: AllocationPattern) -> Self {
        Self {
            allocated_blocks: Arc::new(RwLock::new(Vec::new())),
            current_allocation: Arc::new(AtomicUsize::new(0)),
            is_active: Arc::new(AtomicBool::new(false)),
            allocation_pattern: pattern,
        }
    }
    
    /// Start memory allocation simulation
    pub fn start_simulation(&self) -> Result<()> {
        self.is_active.store(true, Ordering::SeqCst);
        
        let blocks = Arc::clone(&self.allocated_blocks);
        let current = Arc::clone(&self.current_allocation);
        let active = Arc::clone(&self.is_active);
        let pattern = self.allocation_pattern.clone();
        
        thread::spawn(move || {
            let start_time = Instant::now();
            
            while active.load(Ordering::SeqCst) {
                match &pattern {
                    AllocationPattern::Steady { size_mb, interval_ms } => {
                        Self::allocate_steady(&blocks, &current, *size_mb, *interval_ms);
                    }
                    AllocationPattern::Exponential { initial_mb, growth_factor } => {
                        Self::allocate_exponential(&blocks, &current, start_time, *initial_mb, *growth_factor);
                    }
                    AllocationPattern::Spike { base_mb, spike_mb, spike_interval_secs } => {
                        Self::allocate_spike(&blocks, &current, start_time, *base_mb, *spike_mb, *spike_interval_secs);
                    }
                    AllocationPattern::Fragmentation { block_size_kb, blocks_per_sec } => {
                        Self::allocate_fragmentation(&blocks, &current, *block_size_kb, *blocks_per_sec);
                    }
                    AllocationPattern::Leak { leak_rate_mb_per_sec } => {
                        Self::allocate_leak(&blocks, &current, *leak_rate_mb_per_sec);
                    }
                }
                
                thread::sleep(Duration::from_millis(100)); // Check every 100ms
            }
        });
        
        Ok(())
    }
    
    /// Stop memory allocation simulation
    pub fn stop_simulation(&self) {
        self.is_active.store(false, Ordering::SeqCst);
    }
    
    /// Cleanup allocated memory
    pub fn cleanup(&self) -> usize {
        let mut blocks = self.allocated_blocks.write().unwrap();
        let freed_bytes = blocks.iter().map(|block| block.len()).sum();
        blocks.clear();
        self.current_allocation.store(0, Ordering::SeqCst);
        freed_bytes
    }
    
    /// Get current allocation in bytes
    pub fn current_allocation_bytes(&self) -> usize {
        self.current_allocation.load(Ordering::SeqCst)
    }
    
    // Allocation pattern implementations
    fn allocate_steady(
        blocks: &Arc<RwLock<Vec<Vec<u8>>>>,
        current: &Arc<AtomicUsize>,
        size_mb: usize,
        interval_ms: u64,
    ) {
        static mut LAST_ALLOC: Option<Instant> = None;
        
        unsafe {
            let now = Instant::now();
            if LAST_ALLOC.map_or(true, |last| now.duration_since(last).as_millis() >= interval_ms as u128) {
                let block_size = size_mb * 1024 * 1024;
                let block = vec![0u8; block_size];
                
                blocks.write().unwrap().push(block);
                current.fetch_add(block_size, Ordering::SeqCst);
                LAST_ALLOC = Some(now);
            }
        }
    }
    
    fn allocate_exponential(
        blocks: &Arc<RwLock<Vec<Vec<u8>>>>,
        current: &Arc<AtomicUsize>,
        start_time: Instant,
        initial_mb: usize,
        growth_factor: f64,
    ) {
        let elapsed_secs = start_time.elapsed().as_secs_f64();
        let target_mb = (initial_mb as f64 * (growth_factor * elapsed_secs).exp()) as usize;
        let current_mb = current.load(Ordering::SeqCst) / (1024 * 1024);
        
        if target_mb > current_mb {
            let alloc_mb = target_mb - current_mb;
            let block_size = alloc_mb * 1024 * 1024;
            
            if block_size > 0 && block_size < 100 * 1024 * 1024 { // Cap at 100MB per allocation
                let block = vec![0u8; block_size];
                blocks.write().unwrap().push(block);
                current.fetch_add(block_size, Ordering::SeqCst);
            }
        }
    }
    
    fn allocate_spike(
        blocks: &Arc<RwLock<Vec<Vec<u8>>>>,
        current: &Arc<AtomicUsize>,
        start_time: Instant,
        base_mb: usize,
        spike_mb: usize,
        spike_interval_secs: u64,
    ) {
        let elapsed_secs = start_time.elapsed().as_secs();
        let is_spike_time = (elapsed_secs % spike_interval_secs) < 2; // 2-second spike duration
        
        let target_mb = if is_spike_time { base_mb + spike_mb } else { base_mb };
        let current_mb = current.load(Ordering::SeqCst) / (1024 * 1024);
        
        if target_mb > current_mb {
            let alloc_mb = (target_mb - current_mb).min(50); // Max 50MB per allocation
            let block_size = alloc_mb * 1024 * 1024;
            
            if block_size > 0 {
                let block = vec![0u8; block_size];
                blocks.write().unwrap().push(block);
                current.fetch_add(block_size, Ordering::SeqCst);
            }
        } else if target_mb < current_mb && !is_spike_time {
            // Cleanup during non-spike periods
            let mut blocks_lock = blocks.write().unwrap();
            if !blocks_lock.is_empty() && blocks_lock.len() > 5 {
                let removed = blocks_lock.remove(0);
                current.fetch_sub(removed.len(), Ordering::SeqCst);
            }
        }
    }
    
    fn allocate_fragmentation(
        blocks: &Arc<RwLock<Vec<Vec<u8>>>>,
        current: &Arc<AtomicUsize>,
        block_size_kb: usize,
        blocks_per_sec: usize,
    ) {
        static mut LAST_FRAG_ALLOC: Option<Instant> = None;
        static mut FRAG_COUNTER: usize = 0;
        
        unsafe {
            let now = Instant::now();
            if LAST_FRAG_ALLOC.map_or(true, |last| now.duration_since(last).as_millis() >= 100) {
                if FRAG_COUNTER < blocks_per_sec / 10 { // Allocate 1/10th per 100ms interval
                    let block_size_bytes = block_size_kb * 1024;
                    let block = vec![0u8; block_size_bytes];
                    
                    blocks.write().unwrap().push(block);
                    current.fetch_add(block_size_bytes, Ordering::SeqCst);
                    FRAG_COUNTER += 1;
                } else {
                    FRAG_COUNTER = 0;
                }
                LAST_FRAG_ALLOC = Some(now);
            }
        }
    }
    
    fn allocate_leak(
        blocks: &Arc<RwLock<Vec<Vec<u8>>>>,
        current: &Arc<AtomicUsize>,
        leak_rate_mb_per_sec: f64,
    ) {
        static mut LAST_LEAK_ALLOC: Option<Instant> = None;
        
        unsafe {
            let now = Instant::now();
            if LAST_LEAK_ALLOC.map_or(true, |last| now.duration_since(last).as_millis() >= 100) {
                let alloc_mb = (leak_rate_mb_per_sec * 0.1) as usize; // 100ms worth of leakage
                let block_size = alloc_mb * 1024 * 1024;
                
                if block_size > 0 {
                    let block = vec![0u8; block_size];
                    blocks.write().unwrap().push(block);
                    current.fetch_add(block_size, Ordering::SeqCst);
                }
                LAST_LEAK_ALLOC = Some(now);
            }
        }
    }
}

/// Memory pressure monitoring and analysis system
#[derive(Debug)]
pub struct MemoryPressureMonitor {
    /// Configuration
    config: MemoryPressureConfig,
    /// Whether monitoring is active
    is_monitoring: Arc<AtomicBool>,
    /// Memory snapshots history
    snapshots: Arc<Mutex<VecDeque<MemorySnapshot>>>,
    /// Generated alerts
    alerts: Arc<Mutex<Vec<(Instant, MemoryPressureAlert)>>>,
    /// Current pressure state
    current_state: Arc<RwLock<MemoryPressureState>>,
    /// Baseline memory snapshot
    baseline: Arc<RwLock<Option<MemorySnapshot>>>,
    /// Monitor start time
    start_time: Instant,
}

impl MemoryPressureMonitor {
    /// Create new memory pressure monitor
    pub fn new(config: MemoryPressureConfig) -> Self {
        Self {
            config,
            is_monitoring: Arc::new(AtomicBool::new(false)),
            snapshots: Arc::new(Mutex::new(VecDeque::new())),
            alerts: Arc::new(Mutex::new(Vec::new())),
            current_state: Arc::new(RwLock::new(MemoryPressureState::Normal)),
            baseline: Arc::new(RwLock::new(None)),
            start_time: Instant::now(),
        }
    }
    
    /// Start memory pressure monitoring
    pub fn start_monitoring(&self) -> Result<()> {
        // Capture baseline
        let baseline_snapshot = MemorySnapshot::capture();
        *self.baseline.write().unwrap() = Some(baseline_snapshot);
        
        self.is_monitoring.store(true, Ordering::SeqCst);
        
        let config = self.config.clone();
        let is_monitoring = Arc::clone(&self.is_monitoring);
        let snapshots = Arc::clone(&self.snapshots);
        let alerts = Arc::clone(&self.alerts);
        let current_state = Arc::clone(&self.current_state);
        let baseline = Arc::clone(&self.baseline);
        let start_time = self.start_time;
        
        thread::spawn(move || {
            while is_monitoring.load(Ordering::SeqCst) {
                // Check max duration
                if start_time.elapsed().as_secs() > config.max_test_duration_secs {
                    is_monitoring.store(false, Ordering::SeqCst);
                    break;
                }
                
                // Capture memory snapshot
                let snapshot = MemorySnapshot::capture();
                
                // Analyze for pressure alerts
                let new_alerts = Self::analyze_memory_pressure(&snapshot, &baseline, &config);
                
                // Update state based on alerts
                let new_state = Self::determine_pressure_state(&new_alerts, &snapshot, &config);
                *current_state.write().unwrap() = new_state;
                
                // Store snapshot and alerts
                {
                    let mut snapshots_lock = snapshots.lock().unwrap();
                    snapshots_lock.push_back(snapshot);
                    
                    // Keep only recent snapshots
                    while snapshots_lock.len() > config.sample_buffer_size {
                        snapshots_lock.pop_front();
                    }
                }
                
                {
                    let mut alerts_lock = alerts.lock().unwrap();
                    for alert in new_alerts {
                        alerts_lock.push((Instant::now(), alert));
                    }
                    
                    // Keep only recent alerts (last 1000)
                    while alerts_lock.len() > 1000 {
                        alerts_lock.remove(0);
                    }
                }
                
                thread::sleep(Duration::from_millis(config.monitoring_interval_ms));
            }
        });
        
        Ok(())
    }
    
    /// Stop memory pressure monitoring
    pub fn stop_monitoring(&self) {
        self.is_monitoring.store(false, Ordering::SeqCst);
    }
    
    /// Get current memory pressure state
    pub fn get_current_state(&self) -> MemoryPressureState {
        self.current_state.read().unwrap().clone()
    }
    
    /// Get monitoring results
    pub fn get_results(&self) -> MemoryPressureResults {
        let snapshots = self.snapshots.lock().unwrap().clone().into();
        let alerts = self.alerts.lock().unwrap().clone();
        let current_state = self.current_state.read().unwrap().clone();
        let baseline = self.baseline.read().unwrap().clone();
        
        MemoryPressureResults {
            snapshots,
            alerts,
            current_state,
            baseline,
            total_duration: self.start_time.elapsed(),
            config: self.config.clone(),
        }
    }
    
    /// Analyze memory snapshot for pressure indicators
    fn analyze_memory_pressure(
        snapshot: &MemorySnapshot,
        baseline: &Arc<RwLock<Option<MemorySnapshot>>>,
        config: &MemoryPressureConfig,
    ) -> Vec<MemoryPressureAlert> {
        let mut alerts = Vec::new();
        let current_mb = snapshot.total_memory_mb();
        
        // Check memory thresholds
        if current_mb > config.memory_critical_threshold_mb as f64 {
            alerts.push(MemoryPressureAlert::MemoryCritical {
                current_mb,
                threshold_mb: config.memory_critical_threshold_mb as f64,
            });
        } else if current_mb > config.memory_warning_threshold_mb as f64 {
            alerts.push(MemoryPressureAlert::MemoryWarning {
                current_mb,
                threshold_mb: config.memory_warning_threshold_mb as f64,
            });
        }
        
        // Check for memory leak
        if let Some(ref baseline_snapshot) = *baseline.read().unwrap() {
            if snapshot.indicates_memory_leak(baseline_snapshot) {
                let duration = snapshot.timestamp.duration_since(baseline_snapshot.timestamp).as_secs_f64();
                let growth_mb = current_mb - baseline_snapshot.total_memory_mb();
                let growth_rate = growth_mb / duration;
                
                alerts.push(MemoryPressureAlert::MemoryLeak {
                    growth_rate_mb_per_sec: growth_rate,
                    confidence: 0.85, // High confidence for obvious leaks
                });
            }
        }
        
        // Check fragmentation
        if config.enable_fragmentation_analysis && snapshot.fragmentation_percent > 30.0 {
            alerts.push(MemoryPressureAlert::HighFragmentation {
                fragmentation_percent: snapshot.fragmentation_percent,
            });
        }
        
        // Check allocation rate
        let allocation_rate_mb = snapshot.allocation_rate / (1024.0 * 1024.0);
        if allocation_rate_mb > config.allocation_rate_limit_mb_per_sec {
            alerts.push(MemoryPressureAlert::ExcessiveAllocation {
                rate_mb_per_sec: allocation_rate_mb,
                limit_mb_per_sec: config.allocation_rate_limit_mb_per_sec,
            });
        }
        
        // Check memory efficiency
        let efficiency = snapshot.memory_efficiency() * 100.0;
        if efficiency < 50.0 {
            alerts.push(MemoryPressureAlert::PerformanceDegradation {
                efficiency_percent: efficiency,
            });
        }
        
        // Estimate time to OOM
        if let Some(ref baseline_snapshot) = *baseline.read().unwrap() {
            let duration = snapshot.timestamp.duration_since(baseline_snapshot.timestamp).as_secs_f64();
            if duration > 10.0 { // Need at least 10 seconds of data
                let growth_mb = current_mb - baseline_snapshot.total_memory_mb();
                let growth_rate = growth_mb / duration;
                
                if growth_rate > 0.0 {
                    let remaining_mb = config.memory_critical_threshold_mb as f64 * 1.5 - current_mb;
                    let time_to_oom_secs = remaining_mb / growth_rate;
                    
                    if time_to_oom_secs < 300.0 && time_to_oom_secs > 0.0 { // Less than 5 minutes
                        alerts.push(MemoryPressureAlert::OutOfMemoryImminent {
                            estimated_time_to_oom: Duration::from_secs(time_to_oom_secs as u64),
                        });
                    }
                }
            }
        }
        
        alerts
    }
    
    /// Determine current pressure state based on alerts and metrics
    fn determine_pressure_state(
        alerts: &[MemoryPressureAlert],
        snapshot: &MemorySnapshot,
        config: &MemoryPressureConfig,
    ) -> MemoryPressureState {
        // Check for emergency conditions
        for alert in alerts {
            match alert {
                MemoryPressureAlert::OutOfMemoryImminent { .. } => {
                    return MemoryPressureState::Emergency;
                }
                MemoryPressureAlert::MemoryCritical { .. } => {
                    return MemoryPressureState::Critical;
                }
                _ => {}
            }
        }
        
        // Check for critical conditions
        let current_mb = snapshot.total_memory_mb();
        if current_mb > config.memory_critical_threshold_mb as f64 * 0.9 {
            return MemoryPressureState::Critical;
        }
        
        // Check for warning conditions
        if current_mb > config.memory_warning_threshold_mb as f64 {
            return MemoryPressureState::Warning;
        }
        
        // Check for concerning trends
        for alert in alerts {
            match alert {
                MemoryPressureAlert::MemoryLeak { .. } |
                MemoryPressureAlert::ExcessiveAllocation { .. } |
                MemoryPressureAlert::HighFragmentation { .. } => {
                    return MemoryPressureState::Warning;
                }
                _ => {}
            }
        }
        
        MemoryPressureState::Normal
    }
}

/// Memory pressure monitoring results
#[derive(Debug)]
pub struct MemoryPressureResults {
    /// All memory snapshots
    pub snapshots: Vec<MemorySnapshot>,
    /// All generated alerts with timestamps
    pub alerts: Vec<(Instant, MemoryPressureAlert)>,
    /// Final pressure state
    pub current_state: MemoryPressureState,
    /// Baseline snapshot
    pub baseline: Option<MemorySnapshot>,
    /// Total monitoring duration
    pub total_duration: Duration,
    /// Configuration used
    pub config: MemoryPressureConfig,
}

impl MemoryPressureResults {
    /// Generate comprehensive memory pressure report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# Memory Pressure Testing Report\n\n");
        
        // Executive summary
        report.push_str("## Executive Summary\n");
        report.push_str(&format!("Monitoring duration: {:?}\n", self.total_duration));
        report.push_str(&format!("Memory snapshots: {}\n", self.snapshots.len()));
        report.push_str(&format!("Alerts generated: {}\n", self.alerts.len()));
        report.push_str(&format!("Final state: {:?}\n\n", self.current_state));
        
        // Memory usage analysis
        if let (Some(first), Some(last)) = (self.snapshots.first(), self.snapshots.last()) {
            let initial_mb = first.total_memory_mb();
            let final_mb = last.total_memory_mb();
            let growth_mb = final_mb - initial_mb;
            let growth_rate = growth_mb / self.total_duration.as_secs_f64();
            
            report.push_str("## Memory Usage Analysis\n");
            report.push_str(&format!("Initial memory: {:.1} MB\n", initial_mb));
            report.push_str(&format!("Final memory: {:.1} MB\n", final_mb));
            report.push_str(&format!("Total growth: {:.1} MB\n", growth_mb));
            report.push_str(&format!("Growth rate: {:.2} MB/sec\n", growth_rate));
            
            if let Some(baseline) = &self.baseline {
                report.push_str(&format!("Baseline memory: {:.1} MB\n", baseline.total_memory_mb()));
                
                if last.indicates_memory_leak(baseline) {
                    report.push_str("⚠️ **Memory leak detected**\n");
                }
            }
            report.push_str("\n");
        }
        
        // Alerts summary
        if !self.alerts.is_empty() {
            report.push_str("## Alerts Summary\n\n");
            
            let mut alert_counts: HashMap<String, usize> = HashMap::new();
            for (_, alert) in &self.alerts {
                let alert_type = match alert {
                    MemoryPressureAlert::MemoryWarning { .. } => "Memory Warning",
                    MemoryPressureAlert::MemoryCritical { .. } => "Memory Critical",
                    MemoryPressureAlert::MemoryLeak { .. } => "Memory Leak",
                    MemoryPressureAlert::HighFragmentation { .. } => "High Fragmentation",
                    MemoryPressureAlert::ExcessiveAllocation { .. } => "Excessive Allocation",
                    MemoryPressureAlert::OutOfMemoryImminent { .. } => "OOM Imminent",
                    MemoryPressureAlert::PerformanceDegradation { .. } => "Performance Degradation",
                };
                *alert_counts.entry(alert_type.to_string()).or_insert(0) += 1;
            }
            
            for (alert_type, count) in alert_counts {
                report.push_str(&format!("- {}: {} alerts\n", alert_type, count));
            }
            
            report.push_str("\n### Recent Critical Alerts\n");
            for (timestamp, alert) in self.alerts.iter().rev().take(5) {
                match alert {
                    MemoryPressureAlert::MemoryCritical { current_mb, threshold_mb } => {
                        report.push_str(&format!("- {:?} ago: Memory critical {:.1}MB > {:.1}MB\n", 
                                       timestamp.elapsed(), current_mb, threshold_mb));
                    }
                    MemoryPressureAlert::MemoryLeak { growth_rate_mb_per_sec, confidence } => {
                        report.push_str(&format!("- {:?} ago: Memory leak {:.2}MB/sec (confidence: {:.1}%)\n", 
                                       timestamp.elapsed(), growth_rate_mb_per_sec, confidence * 100.0));
                    }
                    MemoryPressureAlert::OutOfMemoryImminent { estimated_time_to_oom } => {
                        report.push_str(&format!("- {:?} ago: OOM imminent in {:?}\n", 
                                       timestamp.elapsed(), estimated_time_to_oom));
                    }
                    _ => {}
                }
            }
        } else {
            report.push_str("## No Alerts Generated\n");
            report.push_str("Memory usage remained within acceptable limits.\n");
        }
        
        // Performance statistics
        if !self.snapshots.is_empty() {
            let avg_fragmentation = self.snapshots.iter()
                .map(|s| s.fragmentation_percent)
                .sum::<f64>() / self.snapshots.len() as f64;
            
            let avg_efficiency = self.snapshots.iter()
                .map(|s| s.memory_efficiency())
                .sum::<f64>() / self.snapshots.len() as f64;
            
            report.push_str("\n## Performance Statistics\n");
            report.push_str(&format!("Average fragmentation: {:.1}%\n", avg_fragmentation));
            report.push_str(&format!("Average memory efficiency: {:.1}%\n", avg_efficiency * 100.0));
            
            if let Some(last) = self.snapshots.last() {
                report.push_str(&format!("Final allocation rate: {:.1} MB/sec\n", 
                               last.allocation_rate / (1024.0 * 1024.0)));
                report.push_str(&format!("Final active allocations: {}\n", last.active_allocations));
            }
        }
        
        report.push_str("\n## Recommendations\n");
        match self.current_state {
            MemoryPressureState::Normal => {
                report.push_str("- Memory usage is within normal parameters\n");
                report.push_str("- Continue monitoring during production loads\n");
            }
            MemoryPressureState::Warning => {
                report.push_str("- Investigate memory usage patterns\n");
                report.push_str("- Consider implementing memory optimization\n");
                report.push_str("- Monitor for potential memory leaks\n");
            }
            MemoryPressureState::Critical => {
                report.push_str("- Immediate action required to reduce memory usage\n");
                report.push_str("- Implement memory cleanup procedures\n");
                report.push_str("- Scale memory resources\n");
            }
            MemoryPressureState::Emergency => {
                report.push_str("- CRITICAL: Out of memory condition imminent\n");
                report.push_str("- Stop non-essential operations immediately\n");
                report.push_str("- Trigger emergency memory cleanup\n");
            }
            MemoryPressureState::Recovery => {
                report.push_str("- System is in recovery mode\n");
                report.push_str("- Monitor recovery progress\n");
                report.push_str("- Validate system stability\n");
            }
        }
        
        report
    }
    
    /// Calculate memory statistics
    pub fn calculate_statistics(&self) -> MemoryStatistics {
        if self.snapshots.is_empty() {
            return MemoryStatistics::default();
        }
        
        let memory_values: Vec<f64> = self.snapshots.iter().map(|s| s.total_memory_mb()).collect();
        let min_memory = memory_values.iter().copied().fold(f64::INFINITY, f64::min);
        let max_memory = memory_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let avg_memory = memory_values.iter().sum::<f64>() / memory_values.len() as f64;
        
        let variance = memory_values.iter()
            .map(|x| (x - avg_memory).powi(2))
            .sum::<f64>() / memory_values.len() as f64;
        let std_dev = variance.sqrt();
        
        let peak_allocation_rate = self.snapshots.iter()
            .map(|s| s.allocation_rate / (1024.0 * 1024.0))
            .fold(f64::NEG_INFINITY, f64::max);
        
        let avg_fragmentation = self.snapshots.iter()
            .map(|s| s.fragmentation_percent)
            .sum::<f64>() / self.snapshots.len() as f64;
        
        let memory_leak_detected = if let Some(baseline) = &self.baseline {
            self.snapshots.last()
                .map(|last| last.indicates_memory_leak(baseline))
                .unwrap_or(false)
        } else {
            false
        };
        
        MemoryStatistics {
            min_memory_mb: min_memory,
            max_memory_mb: max_memory,
            avg_memory_mb: avg_memory,
            std_dev_memory_mb: std_dev,
            peak_allocation_rate_mb_per_sec: peak_allocation_rate,
            avg_fragmentation_percent: avg_fragmentation,
            total_alerts: self.alerts.len(),
            memory_leak_detected,
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Default)]
pub struct MemoryStatistics {
    /// Minimum memory usage in MB
    pub min_memory_mb: f64,
    /// Maximum memory usage in MB
    pub max_memory_mb: f64,
    /// Average memory usage in MB
    pub avg_memory_mb: f64,
    /// Standard deviation of memory usage
    pub std_dev_memory_mb: f64,
    /// Peak allocation rate in MB/sec
    pub peak_allocation_rate_mb_per_sec: f64,
    /// Average fragmentation percentage
    pub avg_fragmentation_percent: f64,
    /// Total number of alerts generated
    pub total_alerts: usize,
    /// Whether memory leak was detected
    pub memory_leak_detected: bool,
}

/// Test memory pressure infrastructure
#[test]
fn test_memory_pressure_infrastructure() -> Result<()> {
    println!("🧠 Starting memory pressure infrastructure test");
    
    let config = MemoryPressureConfig {
        monitoring_interval_ms: 100, // Faster for testing
        max_test_duration_secs: 30,
        memory_warning_threshold_mb: 500,
        memory_critical_threshold_mb: 800,
        allocation_rate_limit_mb_per_sec: 50.0,
        enable_auto_cleanup: true,
        max_memory_growth_rate: 25.0,
        enable_fragmentation_analysis: true,
        sample_buffer_size: 500,
    };
    
    println!("Configuration:");
    println!("  - Monitoring interval: {} ms", config.monitoring_interval_ms);
    println!("  - Warning threshold: {} MB", config.memory_warning_threshold_mb);
    println!("  - Critical threshold: {} MB", config.memory_critical_threshold_mb);
    println!("  - Allocation rate limit: {:.1} MB/sec", config.allocation_rate_limit_mb_per_sec);
    println!("  - Fragmentation analysis: {}", config.enable_fragmentation_analysis);
    println!();
    
    // Create and start memory pressure monitor
    let monitor = MemoryPressureMonitor::new(config);
    monitor.start_monitoring()?;
    
    println!("📊 Memory pressure monitoring started");
    
    // Test different allocation patterns
    println!("🔧 Testing different memory allocation patterns...");
    
    // Test 1: Steady allocation
    println!("  📈 Test 1: Steady allocation pattern");
    let steady_simulator = MemoryAllocationSimulator::new(
        AllocationPattern::Steady { size_mb: 10, interval_ms: 500 }
    );
    steady_simulator.start_simulation()?;
    thread::sleep(Duration::from_secs(3));
    
    let allocated = steady_simulator.current_allocation_bytes();
    println!("    Allocated: {:.1} MB", allocated as f64 / (1024.0 * 1024.0));
    
    steady_simulator.stop_simulation();
    let freed = steady_simulator.cleanup();
    println!("    Cleaned up: {:.1} MB", freed as f64 / (1024.0 * 1024.0));
    
    // Test 2: Memory leak simulation
    println!("  🔍 Test 2: Memory leak simulation");
    let leak_simulator = MemoryAllocationSimulator::new(
        AllocationPattern::Leak { leak_rate_mb_per_sec: 20.0 }
    );
    leak_simulator.start_simulation()?;
    thread::sleep(Duration::from_secs(4));
    
    let leaked = leak_simulator.current_allocation_bytes();
    println!("    Leaked: {:.1} MB", leaked as f64 / (1024.0 * 1024.0));
    
    leak_simulator.stop_simulation();
    let cleaned = leak_simulator.cleanup();
    println!("    Cleaned up: {:.1} MB", cleaned as f64 / (1024.0 * 1024.0));
    
    // Test 3: Fragmentation pattern
    println!("  🔪 Test 3: Fragmentation pattern");
    let frag_simulator = MemoryAllocationSimulator::new(
        AllocationPattern::Fragmentation { block_size_kb: 4, blocks_per_sec: 1000 }
    );
    frag_simulator.start_simulation()?;
    thread::sleep(Duration::from_secs(2));
    
    let fragmented = frag_simulator.current_allocation_bytes();
    println!("    Fragmented: {:.1} MB", fragmented as f64 / (1024.0 * 1024.0));
    
    frag_simulator.stop_simulation();
    let frag_cleaned = frag_simulator.cleanup();
    println!("    Cleaned up: {:.1} MB", frag_cleaned as f64 / (1024.0 * 1024.0));
    
    // Let monitoring continue for a bit
    println!("  ⏱️  Continuing monitoring...");
    thread::sleep(Duration::from_secs(3));
    
    // Stop monitoring and get results
    monitor.stop_monitoring();
    thread::sleep(Duration::from_millis(500)); // Allow monitor thread to finish
    
    let results = monitor.get_results();
    
    // Generate and display comprehensive report
    let report = results.generate_report();
    println!("\n{}", report);
    
    // Calculate and display statistics
    let stats = results.calculate_statistics();
    println!("📊 Memory Statistics:");
    println!("  - Memory range: {:.1} - {:.1} MB (avg: {:.1} MB)", 
             stats.min_memory_mb, stats.max_memory_mb, stats.avg_memory_mb);
    println!("  - Standard deviation: {:.1} MB", stats.std_dev_memory_mb);
    println!("  - Peak allocation rate: {:.1} MB/sec", stats.peak_allocation_rate_mb_per_sec);
    println!("  - Average fragmentation: {:.1}%", stats.avg_fragmentation_percent);
    println!("  - Total alerts: {}", stats.total_alerts);
    println!("  - Memory leak detected: {}", stats.memory_leak_detected);
    
    // Verify infrastructure results
    assert!(!results.snapshots.is_empty(), "Should have collected memory snapshots");
    assert!(results.snapshots.len() >= 50, "Should have collected sufficient snapshots");
    
    println!("\n✅ Infrastructure verification:");
    println!("  📸 Snapshots collected: {}", results.snapshots.len());
    println!("  🚨 Alerts generated: {}", results.alerts.len());
    println!("  🏁 Final state: {:?}", results.current_state);
    println!("  ⏱️  Monitoring duration: {:?}", results.total_duration);
    
    // Verify baseline was established
    assert!(results.baseline.is_some(), "Should have baseline snapshot");
    
    // Verify monitoring worked properly
    let monitoring_rate = results.snapshots.len() as f64 / results.total_duration.as_secs_f64();
    println!("  📊 Monitoring rate: {:.1} samples/sec", monitoring_rate);
    assert!(monitoring_rate >= 5.0, "Should maintain reasonable monitoring rate");
    
    // Test memory snapshot functionality
    let test_snapshot = MemorySnapshot::capture();
    println!("  🔍 Sample snapshot:");
    println!("    - Total memory: {:.1} MB", test_snapshot.total_memory_mb());
    println!("    - Memory efficiency: {:.1}%", test_snapshot.memory_efficiency() * 100.0);
    println!("    - Fragmentation: {:.1}%", test_snapshot.fragmentation_percent);
    println!("    - Active allocations: {}", test_snapshot.active_allocations);
    
    assert!(test_snapshot.total_memory_mb() > 0.0, "Should report positive memory usage");
    assert!(test_snapshot.memory_efficiency() > 0.0, "Should have positive efficiency");
    
    println!("🎯 Memory pressure infrastructure test completed successfully");
    
    Ok(())
}