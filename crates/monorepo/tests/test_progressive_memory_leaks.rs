//! Progressive Memory Leak Simulation System
//!
//! This module implements comprehensive simulation of memory leaks with different patterns,
//! progression rates, and detection mechanisms to test system resilience under memory
//! pressure conditions caused by gradual resource exhaustion.

use sublime_monorepo_tools::Result;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

/// Configuration for progressive memory leak simulation
#[derive(Debug, Clone)]
pub struct ProgressiveMemoryLeakConfig {
    /// Initial memory allocation size in MB
    pub initial_allocation_mb: usize,
    /// Rate of memory leak growth (MB per second)
    pub leak_growth_rate_mb_per_sec: f64,
    /// Pattern of memory leak progression
    pub leak_pattern: MemoryLeakPattern,
    /// Maximum test duration before auto-stopping
    pub max_test_duration_secs: u64,
    /// Detection sensitivity (0.0-1.0, higher = more sensitive)
    pub detection_sensitivity: f64,
    /// Memory threshold for leak detection warning (MB)
    pub leak_warning_threshold_mb: usize,
    /// Memory threshold for critical leak detection (MB)
    pub leak_critical_threshold_mb: usize,
    /// Enable automatic leak recovery testing
    pub enable_leak_recovery_testing: bool,
    /// Frequency of leak detection analysis
    pub detection_interval_ms: u64,
    /// Minimum leak duration for confirmation (seconds)
    pub min_leak_duration_for_confirmation_secs: u64,
}

impl Default for ProgressiveMemoryLeakConfig {
    fn default() -> Self {
        Self {
            initial_allocation_mb: 50, // Start with 50MB baseline
            leak_growth_rate_mb_per_sec: 2.0, // 2MB/sec leak rate
            leak_pattern: MemoryLeakPattern::GradualLinear,
            max_test_duration_secs: 300, // 5 minutes max
            detection_sensitivity: 0.8,
            leak_warning_threshold_mb: 200, // Warning at 200MB
            leak_critical_threshold_mb: 500, // Critical at 500MB
            enable_leak_recovery_testing: true,
            detection_interval_ms: 500, // Check every 500ms
            min_leak_duration_for_confirmation_secs: 10, // 10 seconds confirmation
        }
    }
}

/// Different patterns of memory leak progression
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryLeakPattern {
    /// Gradual linear memory growth over time
    GradualLinear,
    /// Exponential memory growth (doubles at intervals)
    ExponentialGrowth,
    /// Sporadic bursts of memory leaks
    SporadicBursts,
    /// Steady constant leak rate
    ConstantLeak,
    /// Cascading leaks (leaks create more leaks)
    CascadingLeaks,
    /// Memory fragmentation leak (allocate/deallocate with gaps)
    FragmentationLeak,
    /// Object retention leak (references not released)
    ObjectRetentionLeak,
}

/// Memory leak simulation state
#[derive(Debug)]
pub struct MemoryLeakSimulator {
    /// Configuration for the simulation
    config: ProgressiveMemoryLeakConfig,
    /// Current simulated memory usage (MB)
    current_memory_mb: Arc<AtomicUsize>,
    /// Memory allocations being tracked
    allocations: Arc<Mutex<Vec<Vec<u8>>>>,
    /// Leak detection state
    leak_detection_state: Arc<Mutex<LeakDetectionState>>,
    /// Simulation control flag
    simulation_active: Arc<AtomicBool>,
    /// Memory samples for analysis
    memory_samples: Arc<Mutex<VecDeque<MemoryLeakSample>>>,
}

/// Memory leak sample for analysis
#[derive(Debug, Clone)]
pub struct MemoryLeakSample {
    /// Timestamp of sample
    pub timestamp: Instant,
    /// Memory usage at sample time (MB)
    pub memory_mb: f64,
    /// Rate of memory growth (MB/sec)
    pub growth_rate_mb_per_sec: f64,
    /// Number of active allocations
    pub allocation_count: usize,
    /// Estimated leak rate at this point
    pub estimated_leak_rate: f64,
    /// Leak confidence score (0.0-1.0)
    pub leak_confidence: f64,
}

/// State tracking for memory leak detection
#[derive(Debug)]
struct LeakDetectionState {
    /// When leak detection first started monitoring
    monitoring_start_time: Instant,
    /// Baseline memory usage (MB)
    baseline_memory_mb: f64,
    /// Number of consecutive leak detections
    consecutive_leak_detections: usize,
    /// Historical memory usage for trend analysis
    memory_history: VecDeque<f64>,
    /// Last confirmed leak detection
    last_confirmed_leak: Option<Instant>,
    /// Current leak severity level
    current_leak_severity: LeakSeverity,
}

/// Severity levels for memory leaks
#[derive(Debug, Clone, PartialEq)]
pub enum LeakSeverity {
    /// No leak detected
    None,
    /// Minor leak detected
    Minor,
    /// Moderate leak requiring attention
    Moderate,
    /// Severe leak requiring immediate action
    Severe,
    /// Critical leak - system failure imminent
    Critical,
}

/// Memory leak detection result
#[derive(Debug, Clone)]
pub struct MemoryLeakDetection {
    /// When the leak was detected
    pub detection_timestamp: Instant,
    /// Type of leak pattern detected
    pub leak_pattern_detected: MemoryLeakPattern,
    /// Severity of the detected leak
    pub severity: LeakSeverity,
    /// Estimated leak rate (MB/sec)
    pub estimated_leak_rate_mb_per_sec: f64,
    /// Confidence in detection (0.0-1.0)
    pub detection_confidence: f64,
    /// Projected time to critical threshold
    pub projected_critical_time: Option<Duration>,
    /// Recommended actions
    pub recommended_actions: Vec<String>,
    /// Memory trend analysis
    pub trend_analysis: MemoryTrendAnalysis,
}

/// Memory trend analysis for leak detection
#[derive(Debug, Clone)]
pub struct MemoryTrendAnalysis {
    /// Linear regression slope (MB/sec)
    pub slope_mb_per_sec: f64,
    /// R-squared value for trend fit
    pub trend_fit_r_squared: f64,
    /// Acceleration of memory growth (MB/sec²)
    pub acceleration: f64,
    /// Standard deviation of memory usage
    pub memory_std_deviation: f64,
    /// Coefficient of variation
    pub coefficient_of_variation: f64,
}

impl MemoryLeakSimulator {
    /// Create a new memory leak simulator
    pub fn new(config: ProgressiveMemoryLeakConfig) -> Self {
        Self {
            config: config.clone(),
            current_memory_mb: Arc::new(AtomicUsize::new(config.initial_allocation_mb)),
            allocations: Arc::new(Mutex::new(Vec::new())),
            leak_detection_state: Arc::new(Mutex::new(LeakDetectionState {
                monitoring_start_time: Instant::now(),
                baseline_memory_mb: config.initial_allocation_mb as f64,
                consecutive_leak_detections: 0,
                memory_history: VecDeque::with_capacity(1000),
                last_confirmed_leak: None,
                current_leak_severity: LeakSeverity::None,
            })),
            simulation_active: Arc::new(AtomicBool::new(false)),
            memory_samples: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
        }
    }

    /// Start progressive memory leak simulation
    pub fn start_simulation(&self) -> Result<()> {
        self.simulation_active.store(true, Ordering::SeqCst);
        
        // Start memory leak simulation thread
        let config = self.config.clone();
        let current_memory = Arc::clone(&self.current_memory_mb);
        let allocations = Arc::clone(&self.allocations);
        let simulation_active = Arc::clone(&self.simulation_active);
        
        thread::spawn(move || {
            Self::run_memory_leak_simulation(
                config,
                current_memory,
                allocations,
                simulation_active,
            )
        });

        // Start memory monitoring thread
        let monitor_config = self.config.clone();
        let monitor_memory = Arc::clone(&self.current_memory_mb);
        let leak_detection_state = Arc::clone(&self.leak_detection_state);
        let memory_samples = Arc::clone(&self.memory_samples);
        let monitor_active = Arc::clone(&self.simulation_active);
        
        thread::spawn(move || {
            Self::run_leak_detection_monitoring(
                monitor_config,
                monitor_memory,
                leak_detection_state,
                memory_samples,
                monitor_active,
            )
        });

        Ok(())
    }

    /// Stop memory leak simulation
    pub fn stop_simulation(&self) {
        self.simulation_active.store(false, Ordering::SeqCst);
    }

    /// Get current memory usage
    pub fn get_current_memory_mb(&self) -> usize {
        self.current_memory_mb.load(Ordering::SeqCst)
    }

    /// Run the memory leak simulation according to the specified pattern
    fn run_memory_leak_simulation(
        config: ProgressiveMemoryLeakConfig,
        current_memory: Arc<AtomicUsize>,
        allocations: Arc<Mutex<Vec<Vec<u8>>>>,
        simulation_active: Arc<AtomicBool>,
    ) {
        let start_time = Instant::now();
        let mut iteration = 0;

        while simulation_active.load(Ordering::SeqCst) {
            if start_time.elapsed().as_secs() >= config.max_test_duration_secs {
                break;
            }

            let leak_amount = Self::calculate_leak_amount(&config, iteration, start_time);
            
            if leak_amount > 0 {
                Self::allocate_leaked_memory(leak_amount, &allocations, &current_memory);
            }

            // Apply pattern-specific behavior
            let sleep_duration = Self::get_pattern_sleep_duration(&config.leak_pattern, iteration);
            thread::sleep(sleep_duration);
            
            iteration += 1;
        }
    }

    /// Calculate the amount of memory to leak this iteration
    fn calculate_leak_amount(
        config: &ProgressiveMemoryLeakConfig,
        iteration: usize,
        start_time: Instant,
    ) -> usize {
        let elapsed_secs = start_time.elapsed().as_secs_f64();
        
        match config.leak_pattern {
            MemoryLeakPattern::GradualLinear => {
                (config.leak_growth_rate_mb_per_sec * 0.1) as usize // 100ms intervals
            },
            MemoryLeakPattern::ExponentialGrowth => {
                let base_leak = config.leak_growth_rate_mb_per_sec * 0.1;
                let growth_factor = 1.0 + (elapsed_secs / 60.0); // Grow faster over time
                (base_leak * growth_factor) as usize
            },
            MemoryLeakPattern::SporadicBursts => {
                if iteration % 50 == 0 { // Burst every 5 seconds (at 100ms intervals)
                    (config.leak_growth_rate_mb_per_sec * 5.0) as usize // Large burst
                } else {
                    0 // No leak between bursts
                }
            },
            MemoryLeakPattern::ConstantLeak => {
                (config.leak_growth_rate_mb_per_sec * 0.1) as usize // Steady rate
            },
            MemoryLeakPattern::CascadingLeaks => {
                let cascade_factor = (iteration / 100) + 1; // More leaks over time
                (config.leak_growth_rate_mb_per_sec * 0.1 * cascade_factor as f64) as usize
            },
            MemoryLeakPattern::FragmentationLeak => {
                // Allocate/deallocate with growing gaps
                if iteration % 3 == 0 {
                    (config.leak_growth_rate_mb_per_sec * 0.15) as usize // Slightly more
                } else {
                    (config.leak_growth_rate_mb_per_sec * 0.05) as usize // Base amount
                }
            },
            MemoryLeakPattern::ObjectRetentionLeak => {
                // Gradual increase with occasional retention spikes
                let base = config.leak_growth_rate_mb_per_sec * 0.1;
                let retention_spike = if iteration % 200 == 0 { base * 3.0 } else { 0.0 };
                (base + retention_spike) as usize
            },
        }
    }

    /// Get sleep duration based on leak pattern
    fn get_pattern_sleep_duration(pattern: &MemoryLeakPattern, iteration: usize) -> Duration {
        match pattern {
            MemoryLeakPattern::GradualLinear => Duration::from_millis(100),
            MemoryLeakPattern::ExponentialGrowth => Duration::from_millis(100),
            MemoryLeakPattern::SporadicBursts => Duration::from_millis(100),
            MemoryLeakPattern::ConstantLeak => Duration::from_millis(100),
            MemoryLeakPattern::CascadingLeaks => Duration::from_millis(100),
            MemoryLeakPattern::FragmentationLeak => {
                // Variable intervals for fragmentation
                if iteration % 3 == 0 {
                    Duration::from_millis(50) // Faster allocation
                } else {
                    Duration::from_millis(150) // Slower deallocation
                }
            },
            MemoryLeakPattern::ObjectRetentionLeak => Duration::from_millis(100),
        }
    }

    /// Allocate memory to simulate leak
    fn allocate_leaked_memory(
        amount_mb: usize,
        allocations: &Arc<Mutex<Vec<Vec<u8>>>>,
        current_memory: &Arc<AtomicUsize>,
    ) {
        let bytes_to_allocate = amount_mb * 1024 * 1024; // Convert MB to bytes
        let allocation = vec![0u8; bytes_to_allocate]; // Allocate memory
        
        if let Ok(mut allocs) = allocations.lock() {
            allocs.push(allocation);
            current_memory.fetch_add(amount_mb, Ordering::SeqCst);
        }
    }

    /// Run leak detection monitoring
    fn run_leak_detection_monitoring(
        config: ProgressiveMemoryLeakConfig,
        current_memory: Arc<AtomicUsize>,
        leak_detection_state: Arc<Mutex<LeakDetectionState>>,
        memory_samples: Arc<Mutex<VecDeque<MemoryLeakSample>>>,
        monitor_active: Arc<AtomicBool>,
    ) {
        while monitor_active.load(Ordering::SeqCst) {
            let memory_mb = current_memory.load(Ordering::SeqCst) as f64;
            let timestamp = Instant::now();
            
            // Calculate growth rate
            let growth_rate = Self::calculate_growth_rate(&memory_samples, memory_mb);
            
            // Create memory sample
            let sample = MemoryLeakSample {
                timestamp,
                memory_mb,
                growth_rate_mb_per_sec: growth_rate,
                allocation_count: Self::estimate_allocation_count(memory_mb),
                estimated_leak_rate: growth_rate,
                leak_confidence: Self::calculate_leak_confidence(&config, growth_rate, memory_mb),
            };

            // Store sample
            if let Ok(mut samples) = memory_samples.lock() {
                samples.push_back(sample);
                if samples.len() > 1000 {
                    samples.pop_front();
                }
            }

            // Update detection state
            if let Ok(mut state) = leak_detection_state.lock() {
                state.memory_history.push_back(memory_mb);
                if state.memory_history.len() > 100 {
                    state.memory_history.pop_front();
                }
                
                // Analyze for leaks
                Self::analyze_for_memory_leaks(&mut state, &config, memory_mb, growth_rate);
            }

            thread::sleep(Duration::from_millis(config.detection_interval_ms));
        }
    }

    /// Calculate memory growth rate
    fn calculate_growth_rate(
        memory_samples: &Arc<Mutex<VecDeque<MemoryLeakSample>>>,
        current_memory_mb: f64,
    ) -> f64 {
        if let Ok(samples) = memory_samples.lock() {
            if samples.len() >= 2 {
                let recent_sample = &samples[samples.len() - 1];
                let previous_sample = &samples[samples.len() - 2];
                
                let time_diff = recent_sample.timestamp.duration_since(previous_sample.timestamp).as_secs_f64();
                let memory_diff = current_memory_mb - recent_sample.memory_mb;
                
                if time_diff > 0.0 {
                    return memory_diff / time_diff;
                }
            }
        }
        0.0
    }

    /// Estimate allocation count based on memory usage
    fn estimate_allocation_count(memory_mb: f64) -> usize {
        // Estimate based on average allocation size
        let avg_allocation_kb = 512.0; // 512KB average
        let memory_kb = memory_mb * 1024.0;
        (memory_kb / avg_allocation_kb) as usize
    }

    /// Calculate leak confidence based on current metrics
    fn calculate_leak_confidence(
        config: &ProgressiveMemoryLeakConfig,
        growth_rate: f64,
        memory_mb: f64,
    ) -> f64 {
        let base_confidence = if growth_rate > 0.0 {
            (growth_rate / (config.leak_growth_rate_mb_per_sec * 2.0)).min(1.0)
        } else {
            0.0
        };

        let memory_factor = if memory_mb > config.initial_allocation_mb as f64 {
            let excess = memory_mb - config.initial_allocation_mb as f64;
            (excess / config.leak_warning_threshold_mb as f64).min(1.0)
        } else {
            0.0
        };

        ((base_confidence + memory_factor) / 2.0).min(1.0)
    }

    /// Analyze current state for memory leaks
    fn analyze_for_memory_leaks(
        state: &mut LeakDetectionState,
        config: &ProgressiveMemoryLeakConfig,
        current_memory_mb: f64,
        growth_rate: f64,
    ) {
        // Check if memory usage exceeds thresholds
        let severity = if current_memory_mb >= config.leak_critical_threshold_mb as f64 {
            LeakSeverity::Critical
        } else if current_memory_mb >= config.leak_warning_threshold_mb as f64 {
            LeakSeverity::Severe
        } else if growth_rate > config.leak_growth_rate_mb_per_sec {
            LeakSeverity::Moderate
        } else if growth_rate > config.leak_growth_rate_mb_per_sec * 0.5 {
            LeakSeverity::Minor
        } else {
            LeakSeverity::None
        };

        state.current_leak_severity = severity;

        // Update consecutive detection count
        if matches!(severity, LeakSeverity::Minor | LeakSeverity::Moderate | LeakSeverity::Severe | LeakSeverity::Critical) {
            state.consecutive_leak_detections += 1;
        } else {
            state.consecutive_leak_detections = 0;
        }

        // Confirm leak if detected consistently
        let min_detections = (config.min_leak_duration_for_confirmation_secs * 1000 / config.detection_interval_ms) as usize;
        if state.consecutive_leak_detections >= min_detections {
            state.last_confirmed_leak = Some(Instant::now());
        }
    }

    /// Perform comprehensive leak detection analysis
    pub fn detect_memory_leaks(&self) -> Result<Option<MemoryLeakDetection>> {
        let samples = self.memory_samples.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock memory samples".to_string())
        })?;

        if samples.len() < 10 {
            return Ok(None); // Need sufficient samples for analysis
        }

        let trend_analysis = Self::analyze_memory_trend(&samples)?;
        
        // Determine if there's a leak based on trend analysis
        let leak_detected = trend_analysis.slope_mb_per_sec > 0.1 && // Positive growth
                          trend_analysis.trend_fit_r_squared > 0.7;  // Good trend fit

        if !leak_detected {
            return Ok(None);
        }

        let detection_state = self.leak_detection_state.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock detection state".to_string())
        })?;

        let current_memory = self.get_current_memory_mb() as f64;
        
        // Determine leak pattern
        let leak_pattern = Self::determine_leak_pattern(&samples)?;
        
        // Calculate projected critical time
        let projected_critical_time = if trend_analysis.slope_mb_per_sec > 0.0 {
            let time_to_critical = (self.config.leak_critical_threshold_mb as f64 - current_memory) 
                                 / trend_analysis.slope_mb_per_sec;
            if time_to_critical > 0.0 {
                Some(Duration::from_secs_f64(time_to_critical))
            } else {
                None
            }
        } else {
            None
        };

        // Generate recommended actions
        let recommended_actions = Self::generate_leak_recommendations(&detection_state.current_leak_severity);

        Ok(Some(MemoryLeakDetection {
            detection_timestamp: Instant::now(),
            leak_pattern_detected: leak_pattern,
            severity: detection_state.current_leak_severity.clone(),
            estimated_leak_rate_mb_per_sec: trend_analysis.slope_mb_per_sec,
            detection_confidence: Self::calculate_overall_confidence(&trend_analysis),
            projected_critical_time,
            recommended_actions,
            trend_analysis,
        }))
    }

    /// Analyze memory trend using statistical methods
    fn analyze_memory_trend(samples: &VecDeque<MemoryLeakSample>) -> Result<MemoryTrendAnalysis> {
        if samples.len() < 2 {
            return Err(sublime_monorepo_tools::Error::generic(
                "Insufficient samples for trend analysis".to_string()
            ));
        }

        let n = samples.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x_squared = 0.0;
        let mut memory_values = Vec::new();

        let start_time = samples[0].timestamp;
        
        for (i, sample) in samples.iter().enumerate() {
            let x = sample.timestamp.duration_since(start_time).as_secs_f64();
            let y = sample.memory_mb;
            
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x_squared += x * x;
            memory_values.push(y);
        }

        // Linear regression calculation
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x_squared - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        // R-squared calculation
        let mean_y = sum_y / n;
        let mut ss_tot = 0.0;
        let mut ss_res = 0.0;

        for (i, sample) in samples.iter().enumerate() {
            let x = sample.timestamp.duration_since(start_time).as_secs_f64();
            let y = sample.memory_mb;
            let y_pred = slope * x + intercept;
            
            ss_tot += (y - mean_y).powi(2);
            ss_res += (y - y_pred).powi(2);
        }

        let r_squared = if ss_tot > 0.0 { 1.0 - (ss_res / ss_tot) } else { 0.0 };

        // Calculate acceleration (second derivative)
        let acceleration = Self::calculate_acceleration(&samples)?;

        // Calculate standard deviation
        let variance = memory_values.iter()
            .map(|x| (x - mean_y).powi(2))
            .sum::<f64>() / n;
        let std_deviation = variance.sqrt();

        // Coefficient of variation
        let coefficient_of_variation = if mean_y > 0.0 { std_deviation / mean_y } else { 0.0 };

        Ok(MemoryTrendAnalysis {
            slope_mb_per_sec: slope,
            trend_fit_r_squared: r_squared,
            acceleration,
            memory_std_deviation: std_deviation,
            coefficient_of_variation,
        })
    }

    /// Calculate memory allocation acceleration
    fn calculate_acceleration(samples: &VecDeque<MemoryLeakSample>) -> Result<f64> {
        if samples.len() < 3 {
            return Ok(0.0);
        }

        let mut accelerations = Vec::new();
        
        for i in 2..samples.len() {
            let prev_growth = samples[i-1].growth_rate_mb_per_sec;
            let curr_growth = samples[i].growth_rate_mb_per_sec;
            let time_diff = samples[i].timestamp.duration_since(samples[i-1].timestamp).as_secs_f64();
            
            if time_diff > 0.0 {
                let acceleration = (curr_growth - prev_growth) / time_diff;
                accelerations.push(acceleration);
            }
        }

        if accelerations.is_empty() {
            Ok(0.0)
        } else {
            Ok(accelerations.iter().sum::<f64>() / accelerations.len() as f64)
        }
    }

    /// Determine the type of leak pattern from samples
    fn determine_leak_pattern(samples: &VecDeque<MemoryLeakSample>) -> Result<MemoryLeakPattern> {
        if samples.len() < 10 {
            return Ok(MemoryLeakPattern::GradualLinear); // Default pattern
        }

        // Analyze growth rate variance
        let growth_rates: Vec<f64> = samples.iter().map(|s| s.growth_rate_mb_per_sec).collect();
        let mean_growth = growth_rates.iter().sum::<f64>() / growth_rates.len() as f64;
        let variance = growth_rates.iter()
            .map(|g| (g - mean_growth).powi(2))
            .sum::<f64>() / growth_rates.len() as f64;
        let std_dev = variance.sqrt();
        let coefficient_of_variation = if mean_growth > 0.0 { std_dev / mean_growth } else { 0.0 };

        // Check for different patterns based on statistical properties
        if coefficient_of_variation > 2.0 {
            // High variability suggests sporadic bursts
            Ok(MemoryLeakPattern::SporadicBursts)
        } else if mean_growth > 0.0 && Self::is_exponential_growth(&samples) {
            Ok(MemoryLeakPattern::ExponentialGrowth)
        } else if Self::has_cascading_characteristics(&samples) {
            Ok(MemoryLeakPattern::CascadingLeaks)
        } else if coefficient_of_variation < 0.1 {
            // Low variability suggests constant leak
            Ok(MemoryLeakPattern::ConstantLeak)
        } else {
            // Default to gradual linear
            Ok(MemoryLeakPattern::GradualLinear)
        }
    }

    /// Check if growth pattern is exponential
    fn is_exponential_growth(samples: &VecDeque<MemoryLeakSample>) -> bool {
        if samples.len() < 5 {
            return false;
        }

        // Check if growth rate is increasing over time
        let mut increasing_count = 0;
        for i in 1..samples.len() {
            if samples[i].growth_rate_mb_per_sec > samples[i-1].growth_rate_mb_per_sec {
                increasing_count += 1;
            }
        }

        // If more than 70% of samples show increasing growth rate
        (increasing_count as f64 / (samples.len() - 1) as f64) > 0.7
    }

    /// Check if pattern has cascading characteristics
    fn has_cascading_characteristics(samples: &VecDeque<MemoryLeakSample>) -> bool {
        if samples.len() < 10 {
            return false;
        }

        // Check for increasing acceleration in later samples
        let mid_point = samples.len() / 2;
        let early_avg = samples.iter().take(mid_point)
            .map(|s| s.growth_rate_mb_per_sec)
            .sum::<f64>() / mid_point as f64;
        
        let late_avg = samples.iter().skip(mid_point)
            .map(|s| s.growth_rate_mb_per_sec)
            .sum::<f64>() / (samples.len() - mid_point) as f64;

        // Cascading leaks show significant acceleration in later phase
        late_avg > early_avg * 1.5
    }

    /// Calculate overall confidence in leak detection
    fn calculate_overall_confidence(trend_analysis: &MemoryTrendAnalysis) -> f64 {
        let trend_confidence = trend_analysis.trend_fit_r_squared;
        let growth_confidence = if trend_analysis.slope_mb_per_sec > 0.0 {
            (trend_analysis.slope_mb_per_sec / 5.0).min(1.0) // Normalize against 5 MB/sec
        } else {
            0.0
        };
        
        let stability_confidence = 1.0 - trend_analysis.coefficient_of_variation.min(1.0);
        
        (trend_confidence + growth_confidence + stability_confidence) / 3.0
    }

    /// Generate recommended actions based on leak severity
    fn generate_leak_recommendations(severity: &LeakSeverity) -> Vec<String> {
        match severity {
            LeakSeverity::None => vec![
                "Continue monitoring memory usage".to_string(),
                "No immediate action required".to_string(),
            ],
            LeakSeverity::Minor => vec![
                "Increase monitoring frequency".to_string(),
                "Review recent code changes for memory issues".to_string(),
                "Consider running memory profiler".to_string(),
            ],
            LeakSeverity::Moderate => vec![
                "Investigate potential memory leaks immediately".to_string(),
                "Enable detailed memory tracking".to_string(),
                "Consider reducing workload temporarily".to_string(),
                "Review garbage collection settings".to_string(),
            ],
            LeakSeverity::Severe => vec![
                "Urgent memory leak investigation required".to_string(),
                "Implement emergency memory cleanup procedures".to_string(),
                "Consider system restart if leak continues".to_string(),
                "Enable verbose memory debugging".to_string(),
                "Scale down operations to critical only".to_string(),
            ],
            LeakSeverity::Critical => vec![
                "CRITICAL: Immediate system intervention required".to_string(),
                "Initiate emergency shutdown procedures".to_string(),
                "Perform immediate memory dump for analysis".to_string(),
                "Restart system with reduced capacity".to_string(),
                "Alert on-call engineering team".to_string(),
                "Implement circuit breaker pattern".to_string(),
            ],
        }
    }

    /// Get current leak detection state
    pub fn get_detection_state(&self) -> Result<LeakSeverity> {
        let state = self.leak_detection_state.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock detection state".to_string())
        })?;
        
        Ok(state.current_leak_severity.clone())
    }

    /// Get memory samples for analysis
    pub fn get_memory_samples(&self) -> Result<Vec<MemoryLeakSample>> {
        let samples = self.memory_samples.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock memory samples".to_string())
        })?;
        
        Ok(samples.iter().cloned().collect())
    }

    /// Clean up leaked memory (for recovery testing)
    pub fn cleanup_leaked_memory(&self) -> Result<usize> {
        let mut allocations = self.allocations.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock allocations".to_string())
        })?;
        
        let initial_memory = self.current_memory_mb.load(Ordering::SeqCst);
        allocations.clear(); // Release all allocations
        
        // Reset memory counter to baseline
        self.current_memory_mb.store(self.config.initial_allocation_mb, Ordering::SeqCst);
        
        let cleaned_memory = initial_memory - self.config.initial_allocation_mb;
        Ok(cleaned_memory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progressive_memory_leak_configuration() {
        let config = ProgressiveMemoryLeakConfig::default();
        assert!(config.initial_allocation_mb > 0);
        assert!(config.leak_growth_rate_mb_per_sec > 0.0);
        assert!(config.detection_sensitivity > 0.0);
        assert!(config.detection_sensitivity <= 1.0);
    }

    #[test]
    fn test_memory_leak_simulator_creation() {
        let config = ProgressiveMemoryLeakConfig::default();
        let simulator = MemoryLeakSimulator::new(config.clone());
        
        assert_eq!(simulator.get_current_memory_mb(), config.initial_allocation_mb);
        assert!(!simulator.simulation_active.load(Ordering::SeqCst));
    }

    #[test]
    fn test_memory_leak_pattern_calculation() {
        let config = ProgressiveMemoryLeakConfig {
            leak_growth_rate_mb_per_sec: 1.0,
            ..Default::default()
        };
        
        let start_time = Instant::now();
        
        // Test different patterns
        let linear_leak = MemoryLeakSimulator::calculate_leak_amount(
            &ProgressiveMemoryLeakConfig {
                leak_pattern: MemoryLeakPattern::GradualLinear,
                ..config.clone()
            },
            1,
            start_time,
        );
        assert!(linear_leak > 0);
        
        let burst_leak = MemoryLeakSimulator::calculate_leak_amount(
            &ProgressiveMemoryLeakConfig {
                leak_pattern: MemoryLeakPattern::SporadicBursts,
                ..config.clone()
            },
            50, // Burst iteration
            start_time,
        );
        assert!(burst_leak > linear_leak);
    }

    #[tokio::test]
    async fn test_memory_leak_simulation_integration() -> Result<()> {
        let config = ProgressiveMemoryLeakConfig {
            max_test_duration_secs: 2, // Short test
            leak_growth_rate_mb_per_sec: 5.0, // Fast leak for testing
            detection_interval_ms: 100, // Fast detection
            ..Default::default()
        };
        
        let simulator = MemoryLeakSimulator::new(config);
        let initial_memory = simulator.get_current_memory_mb();
        
        // Start simulation
        simulator.start_simulation()?;
        
        // Wait for simulation to run
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Check that memory increased
        let current_memory = simulator.get_current_memory_mb();
        assert!(current_memory > initial_memory, 
               "Memory should have increased: {} -> {}", initial_memory, current_memory);
        
        // Test leak detection
        tokio::time::sleep(Duration::from_millis(1000)).await;
        
        let detection = simulator.detect_memory_leaks()?;
        if let Some(leak_detection) = detection {
            assert!(leak_detection.estimated_leak_rate_mb_per_sec > 0.0);
            assert!(!matches!(leak_detection.severity, LeakSeverity::None));
        }
        
        // Stop simulation
        simulator.stop_simulation();
        
        // Test cleanup
        let cleaned_mb = simulator.cleanup_leaked_memory()?;
        assert!(cleaned_mb > 0);
        
        Ok(())
    }

    #[test]
    fn test_leak_pattern_detection() -> Result<()> {
        let mut samples = VecDeque::new();
        let start_time = Instant::now();
        
        // Create samples with exponential growth pattern
        for i in 0..20 {
            let growth_rate = (i as f64).exp() / 10.0; // Exponential growth
            samples.push_back(MemoryLeakSample {
                timestamp: start_time + Duration::from_secs(i),
                memory_mb: 50.0 + growth_rate,
                growth_rate_mb_per_sec: growth_rate,
                allocation_count: 100 + i,
                estimated_leak_rate: growth_rate,
                leak_confidence: 0.8,
            });
        }
        
        let pattern = MemoryLeakSimulator::determine_leak_pattern(&samples)?;
        assert_eq!(pattern, MemoryLeakPattern::ExponentialGrowth);
        
        Ok(())
    }

    #[test]
    fn test_trend_analysis() -> Result<()> {
        let mut samples = VecDeque::new();
        let start_time = Instant::now();
        
        // Create samples with linear growth
        for i in 0..10 {
            samples.push_back(MemoryLeakSample {
                timestamp: start_time + Duration::from_secs(i),
                memory_mb: 50.0 + (i as f64 * 2.0), // Linear growth of 2 MB/sec
                growth_rate_mb_per_sec: 2.0,
                allocation_count: 100 + i,
                estimated_leak_rate: 2.0,
                leak_confidence: 0.9,
            });
        }
        
        let trend = MemoryLeakSimulator::analyze_memory_trend(&samples)?;
        assert!(trend.slope_mb_per_sec > 1.5); // Should detect ~2 MB/sec growth
        assert!(trend.trend_fit_r_squared > 0.9); // Should have excellent fit for linear data
        
        Ok(())
    }
}