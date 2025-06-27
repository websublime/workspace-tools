//! Automatic Out-of-Memory (OOM) Detection System
//!
//! This module implements comprehensive automatic detection of OOM conditions using
//! real-time monitoring, predictive analysis, and intelligent early warning systems
//! to prevent system failures and enable proactive memory management.

use sublime_monorepo_tools::Result;
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::sync::atomic::{AtomicBool, AtomicUsize, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

/// Configuration for automatic OOM detection
#[derive(Debug, Clone)]
pub struct OomDetectionConfig {
    /// Maximum test duration in seconds
    pub max_test_duration_secs: u64,
    /// Memory monitoring interval in milliseconds
    pub monitoring_interval_ms: u64,
    /// Memory warning threshold (percentage of available memory)
    pub memory_warning_threshold_percent: f64,
    /// Memory critical threshold (percentage of available memory)
    pub memory_critical_threshold_percent: f64,
    /// Memory imminent OOM threshold (percentage of available memory)
    pub memory_imminent_oom_threshold_percent: f64,
    /// Prediction window for OOM forecasting (seconds)
    pub prediction_window_secs: u64,
    /// Minimum confidence level for OOM predictions
    pub min_prediction_confidence: f64,
    /// Enable predictive analysis
    pub enable_predictive_analysis: bool,
    /// Enable automatic prevention actions
    pub enable_automatic_prevention: bool,
    /// Memory growth rate alert threshold (MB/sec)
    pub memory_growth_rate_alert_threshold: f64,
    /// Maximum memory allocation size for testing (MB)
    pub max_test_allocation_mb: usize,
    /// Enable real-time monitoring
    pub enable_realtime_monitoring: bool,
    /// Sample history size for trend analysis
    pub sample_history_size: usize,
}

impl Default for OomDetectionConfig {
    fn default() -> Self {
        Self {
            max_test_duration_secs: 300, // 5 minutes
            monitoring_interval_ms: 250, // 250ms for high granularity
            memory_warning_threshold_percent: 70.0,   // 70% memory usage
            memory_critical_threshold_percent: 85.0,  // 85% memory usage
            memory_imminent_oom_threshold_percent: 95.0, // 95% memory usage
            prediction_window_secs: 60, // 1 minute prediction window
            min_prediction_confidence: 0.75,
            enable_predictive_analysis: true,
            enable_automatic_prevention: true,
            memory_growth_rate_alert_threshold: 10.0, // 10 MB/sec
            max_test_allocation_mb: 1024, // 1GB max allocation for testing
            enable_realtime_monitoring: true,
            sample_history_size: 1000,
        }
    }
}

/// OOM detection and prevention system
#[derive(Debug)]
pub struct OomDetectionSystem {
    /// Configuration for the detection system
    config: OomDetectionConfig,
    /// Memory usage history for trend analysis
    memory_history: Arc<Mutex<VecDeque<MemoryUsageSample>>>,
    /// OOM alerts and events
    oom_events: Arc<Mutex<Vec<OomEvent>>>,
    /// Memory allocations for testing
    test_allocations: Arc<Mutex<Vec<Vec<u8>>>>,
    /// System control flags
    detection_active: Arc<AtomicBool>,
    prevention_active: Arc<AtomicBool>,
    /// Memory statistics
    total_allocated_mb: Arc<AtomicU64>,
    peak_memory_usage_mb: Arc<AtomicU64>,
    /// Prediction state
    prediction_state: Arc<Mutex<OomPredictionState>>,
    /// Prevention actions taken
    prevention_actions: Arc<Mutex<Vec<PreventionAction>>>,
}

/// Memory usage sample for analysis
#[derive(Debug, Clone)]
pub struct MemoryUsageSample {
    /// Timestamp of sample
    pub timestamp: Instant,
    /// Available system memory in MB
    pub available_memory_mb: f64,
    /// Used memory in MB
    pub used_memory_mb: f64,
    /// Total system memory in MB
    pub total_memory_mb: f64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Memory growth rate (MB/sec)
    pub memory_growth_rate: f64,
    /// Active allocations count
    pub active_allocations: usize,
    /// Memory fragmentation percentage
    pub fragmentation_percent: f64,
    /// System memory pressure level (0.0-1.0)
    pub memory_pressure_level: f64,
    /// Memory allocation velocity (allocations/sec)
    pub allocation_velocity: f64,
}

/// OOM event types and severity levels
#[derive(Debug, Clone)]
pub struct OomEvent {
    /// Timestamp when event occurred
    pub timestamp: Instant,
    /// Type of OOM event
    pub event_type: OomEventType,
    /// Severity level of the event
    pub severity: OomSeverity,
    /// Memory state when event was triggered
    pub memory_state: MemoryUsageSample,
    /// OOM prediction if available
    pub prediction: Option<OomPrediction>,
    /// Actions taken in response
    pub response_actions: Vec<String>,
    /// Whether the event was successfully mitigated
    pub mitigation_successful: bool,
    /// Additional context and analysis
    pub analysis_details: String,
}

/// Types of OOM events
#[derive(Debug, Clone, PartialEq)]
pub enum OomEventType {
    /// Memory usage approaching warning threshold
    MemoryWarning,
    /// Memory usage reached critical level
    MemoryCritical,
    /// OOM condition imminent
    OomImminent,
    /// Actual OOM condition detected
    OomOccurred,
    /// Rapid memory growth detected
    RapidMemoryGrowth,
    /// Memory leak pattern detected
    MemoryLeakDetected,
    /// Memory fragmentation critical
    FragmentationCritical,
    /// Allocation failure due to memory pressure
    AllocationFailure,
    /// Successful prevention action taken
    PreventionActivated,
    /// System recovered from OOM threat
    OomThreatMitigated,
}

/// OOM severity levels
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum OomSeverity {
    /// Informational - normal operation
    Info,
    /// Warning - memory usage elevated
    Warning,
    /// Critical - immediate attention required
    Critical,
    /// Emergency - OOM imminent or occurred
    Emergency,
}

/// OOM prediction result
#[derive(Debug, Clone)]
pub struct OomPrediction {
    /// Predicted time to OOM condition
    pub time_to_oom: Duration,
    /// Confidence level of prediction (0.0-1.0)
    pub confidence: f64,
    /// Predicted memory usage at OOM
    pub predicted_memory_at_oom_mb: f64,
    /// Growth pattern detected
    pub growth_pattern: MemoryGrowthPattern,
    /// Contributing factors to OOM risk
    pub contributing_factors: Vec<String>,
    /// Recommended preventive actions
    pub recommended_actions: Vec<String>,
    /// Prediction model used
    pub prediction_model: PredictionModel,
}

/// Memory growth patterns
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryGrowthPattern {
    /// Linear growth pattern
    Linear,
    /// Exponential growth pattern
    Exponential,
    /// Periodic/cyclic growth pattern
    Periodic,
    /// Step-wise growth pattern
    StepWise,
    /// Irregular/chaotic growth pattern
    Irregular,
    /// Memory leak pattern
    MemoryLeak,
    /// Burst allocation pattern
    BurstAllocations,
}

/// Prediction models available
#[derive(Debug, Clone, PartialEq)]
pub enum PredictionModel {
    /// Linear regression model
    LinearRegression,
    /// Exponential trend model
    ExponentialTrend,
    /// Moving average model
    MovingAverage,
    /// Pattern matching model
    PatternMatching,
    /// Hybrid ensemble model
    HybridEnsemble,
}

/// OOM prediction state tracking
#[derive(Debug)]
struct OomPredictionState {
    /// Last prediction made
    last_prediction: Option<OomPrediction>,
    /// Prediction accuracy history
    prediction_accuracy_history: VecDeque<f64>,
    /// Model performance metrics
    model_performance: HashMap<PredictionModel, ModelPerformanceMetrics>,
    /// Currently active prediction models
    active_models: Vec<PredictionModel>,
    /// Baseline memory usage for comparison
    baseline_memory_mb: f64,
}

/// Performance metrics for prediction models
#[derive(Debug, Clone)]
struct ModelPerformanceMetrics {
    /// Total predictions made
    total_predictions: usize,
    /// Accurate predictions (within 10% of actual)
    accurate_predictions: usize,
    /// Average prediction error (percentage)
    average_error_percent: f64,
    /// Model confidence scores
    confidence_scores: VecDeque<f64>,
    /// Response time for predictions (ms)
    average_response_time_ms: f64,
}

/// Prevention action taken by the system
#[derive(Debug, Clone)]
pub struct PreventionAction {
    /// Timestamp when action was taken
    pub timestamp: Instant,
    /// Type of prevention action
    pub action_type: PreventionActionType,
    /// Memory state that triggered the action
    pub trigger_state: MemoryUsageSample,
    /// Amount of memory freed (if applicable)
    pub memory_freed_mb: f64,
    /// Success of the action
    pub action_successful: bool,
    /// Time taken to execute action
    pub execution_time: Duration,
    /// Impact on system performance
    pub performance_impact: PerformanceImpact,
}

/// Types of prevention actions
#[derive(Debug, Clone, PartialEq)]
pub enum PreventionActionType {
    /// Emergency memory cleanup
    EmergencyCleanup,
    /// Reduce allocation rate
    ReduceAllocationRate,
    /// Force garbage collection
    ForceGarbageCollection,
    /// Reject new allocations
    RejectNewAllocations,
    /// Shrink existing allocations
    ShrinkAllocations,
    /// Emergency memory compaction
    EmergencyCompaction,
    /// Alert external systems
    AlertExternalSystems,
    /// Graceful service degradation
    GracefulDegradation,
}

/// Performance impact of prevention actions
#[derive(Debug, Clone)]
pub struct PerformanceImpact {
    /// Response time impact (percentage)
    pub response_time_impact_percent: f64,
    /// Throughput impact (percentage)
    pub throughput_impact_percent: f64,
    /// User experience impact level
    pub user_experience_impact: UserExperienceImpact,
    /// Recovery time estimate
    pub estimated_recovery_time: Duration,
}

/// User experience impact levels
#[derive(Debug, Clone, PartialEq)]
pub enum UserExperienceImpact {
    /// No noticeable impact
    None,
    /// Minor impact - slight delays
    Minor,
    /// Moderate impact - noticeable delays
    Moderate,
    /// Severe impact - significant delays
    Severe,
    /// Critical impact - service degradation
    Critical,
}

/// Comprehensive OOM analysis results
#[derive(Debug, Clone)]
pub struct OomAnalysisResults {
    /// Test duration
    pub test_duration: Duration,
    /// Total OOM events by type
    pub events_by_type: HashMap<OomEventType, usize>,
    /// Peak memory usage reached
    pub peak_memory_usage_mb: f64,
    /// OOM predictions made
    pub total_predictions_made: usize,
    /// Prediction accuracy percentage
    pub prediction_accuracy_percent: f64,
    /// Prevention actions taken
    pub prevention_actions_taken: usize,
    /// Prevention success rate
    pub prevention_success_rate: f64,
    /// Memory growth analysis
    pub growth_analysis: MemoryGrowthAnalysis,
    /// System resilience metrics
    pub resilience_metrics: SystemResilienceMetrics,
    /// Recommendations for improvement
    pub improvement_recommendations: Vec<String>,
}

/// Memory growth analysis
#[derive(Debug, Clone)]
pub struct MemoryGrowthAnalysis {
    /// Average memory growth rate (MB/sec)
    pub average_growth_rate: f64,
    /// Peak memory growth rate (MB/sec)
    pub peak_growth_rate: f64,
    /// Growth pattern identified
    pub primary_growth_pattern: MemoryGrowthPattern,
    /// Growth acceleration (MB/sec²)
    pub growth_acceleration: f64,
    /// Memory efficiency score (0.0-1.0)
    pub memory_efficiency_score: f64,
    /// Leak probability score (0.0-1.0)
    pub leak_probability_score: f64,
}

/// System resilience metrics
#[derive(Debug, Clone)]
pub struct SystemResilienceMetrics {
    /// Number of OOM threats successfully mitigated
    pub oom_threats_mitigated: usize,
    /// Average recovery time from memory pressure
    pub average_recovery_time: Duration,
    /// System stability score (0.0-1.0)
    pub stability_score: f64,
    /// Proactive intervention effectiveness
    pub proactive_effectiveness_percent: f64,
    /// Memory resource utilization efficiency
    pub resource_utilization_efficiency: f64,
}

impl OomDetectionSystem {
    /// Create a new OOM detection system
    pub fn new(config: OomDetectionConfig) -> Self {
        Self {
            config: config.clone(),
            memory_history: Arc::new(Mutex::new(VecDeque::with_capacity(config.sample_history_size))),
            oom_events: Arc::new(Mutex::new(Vec::new())),
            test_allocations: Arc::new(Mutex::new(Vec::new())),
            detection_active: Arc::new(AtomicBool::new(false)),
            prevention_active: Arc::new(AtomicBool::new(false)),
            total_allocated_mb: Arc::new(AtomicU64::new(0)),
            peak_memory_usage_mb: Arc::new(AtomicU64::new(0)),
            prediction_state: Arc::new(Mutex::new(OomPredictionState {
                last_prediction: None,
                prediction_accuracy_history: VecDeque::with_capacity(100),
                model_performance: HashMap::new(),
                active_models: vec![
                    PredictionModel::LinearRegression,
                    PredictionModel::ExponentialTrend,
                    PredictionModel::MovingAverage,
                ],
                baseline_memory_mb: 0.0,
            })),
            prevention_actions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Start automatic OOM detection
    pub fn start_oom_detection(&self) -> Result<()> {
        self.detection_active.store(true, Ordering::SeqCst);
        self.prevention_active.store(true, Ordering::SeqCst);

        // Start memory monitoring thread
        if self.config.enable_realtime_monitoring {
            let monitor_config = self.config.clone();
            let monitor_memory_history = Arc::clone(&self.memory_history);
            let monitor_oom_events = Arc::clone(&self.oom_events);
            let monitor_detection_active = Arc::clone(&self.detection_active);
            let monitor_total_allocated = Arc::clone(&self.total_allocated_mb);
            let monitor_peak_memory = Arc::clone(&self.peak_memory_usage_mb);

            thread::spawn(move || {
                Self::run_memory_monitoring_thread(
                    monitor_config,
                    monitor_memory_history,
                    monitor_oom_events,
                    monitor_detection_active,
                    monitor_total_allocated,
                    monitor_peak_memory,
                )
            });
        }

        // Start OOM prediction thread
        if self.config.enable_predictive_analysis {
            let predict_config = self.config.clone();
            let predict_memory_history = Arc::clone(&self.memory_history);
            let predict_oom_events = Arc::clone(&self.oom_events);
            let predict_prediction_state = Arc::clone(&self.prediction_state);
            let predict_detection_active = Arc::clone(&self.detection_active);

            thread::spawn(move || {
                Self::run_oom_prediction_thread(
                    predict_config,
                    predict_memory_history,
                    predict_oom_events,
                    predict_prediction_state,
                    predict_detection_active,
                )
            });
        }

        // Start prevention action thread
        if self.config.enable_automatic_prevention {
            let prevent_config = self.config.clone();
            let prevent_oom_events = Arc::clone(&self.oom_events);
            let prevent_test_allocations = Arc::clone(&self.test_allocations);
            let prevent_prevention_actions = Arc::clone(&self.prevention_actions);
            let prevent_prevention_active = Arc::clone(&self.prevention_active);
            let prevent_total_allocated = Arc::clone(&self.total_allocated_mb);

            thread::spawn(move || {
                Self::run_prevention_action_thread(
                    prevent_config,
                    prevent_oom_events,
                    prevent_test_allocations,
                    prevent_prevention_actions,
                    prevent_prevention_active,
                    prevent_total_allocated,
                )
            });
        }

        // Start memory allocation stress test
        let stress_config = self.config.clone();
        let stress_test_allocations = Arc::clone(&self.test_allocations);
        let stress_total_allocated = Arc::clone(&self.total_allocated_mb);
        let stress_detection_active = Arc::clone(&self.detection_active);

        thread::spawn(move || {
            Self::run_memory_stress_thread(
                stress_config,
                stress_test_allocations,
                stress_total_allocated,
                stress_detection_active,
            )
        });

        Ok(())
    }

    /// Stop OOM detection system
    pub fn stop_oom_detection(&self) {
        self.detection_active.store(false, Ordering::SeqCst);
        self.prevention_active.store(false, Ordering::SeqCst);
    }

    /// Run memory monitoring thread
    fn run_memory_monitoring_thread(
        config: OomDetectionConfig,
        memory_history: Arc<Mutex<VecDeque<MemoryUsageSample>>>,
        oom_events: Arc<Mutex<Vec<OomEvent>>>,
        detection_active: Arc<AtomicBool>,
        total_allocated: Arc<AtomicU64>,
        peak_memory: Arc<AtomicU64>,
    ) {
        let start_time = Instant::now();
        
        while detection_active.load(Ordering::SeqCst) {
            if start_time.elapsed().as_secs() >= config.max_test_duration_secs {
                break;
            }

            let sample = Self::capture_memory_sample(&total_allocated, &memory_history);
            
            // Update peak memory usage
            let current_memory_mb = sample.used_memory_mb as u64;
            let mut peak = peak_memory.load(Ordering::SeqCst);
            while peak < current_memory_mb {
                match peak_memory.compare_exchange_weak(peak, current_memory_mb, Ordering::SeqCst, Ordering::Relaxed) {
                    Ok(_) => break,
                    Err(new_peak) => peak = new_peak,
                }
            }

            // Check for OOM conditions
            Self::analyze_oom_conditions(&sample, &config, &oom_events);

            // Store sample in history
            if let Ok(mut history) = memory_history.lock() {
                history.push_back(sample);
                if history.len() > config.sample_history_size {
                    history.pop_front();
                }
            }

            thread::sleep(Duration::from_millis(config.monitoring_interval_ms));
        }
    }

    /// Capture current memory usage sample
    fn capture_memory_sample(
        total_allocated: &Arc<AtomicU64>,
        memory_history: &Arc<Mutex<VecDeque<MemoryUsageSample>>>,
    ) -> MemoryUsageSample {
        let timestamp = Instant::now();
        let allocated_mb = total_allocated.load(Ordering::SeqCst) as f64;
        
        // Simulate system memory metrics (in real implementation, these would come from OS)
        let total_memory_mb = 8192.0; // 8GB simulated system memory
        let base_used_memory = 2048.0; // 2GB base system usage
        let used_memory_mb = base_used_memory + allocated_mb;
        let available_memory_mb = total_memory_mb - used_memory_mb;
        let memory_usage_percent = (used_memory_mb / total_memory_mb) * 100.0;

        // Calculate growth rate
        let growth_rate = Self::calculate_memory_growth_rate(memory_history, used_memory_mb);
        
        // Simulate fragmentation and pressure
        let fragmentation_percent = Self::simulate_memory_fragmentation(allocated_mb);
        let memory_pressure_level = (memory_usage_percent / 100.0).min(1.0);
        
        // Calculate allocation velocity
        let allocation_velocity = Self::calculate_allocation_velocity(memory_history);

        MemoryUsageSample {
            timestamp,
            available_memory_mb,
            used_memory_mb,
            total_memory_mb,
            memory_usage_percent,
            memory_growth_rate: growth_rate,
            active_allocations: Self::simulate_active_allocations(allocated_mb),
            fragmentation_percent,
            memory_pressure_level,
            allocation_velocity,
        }
    }

    /// Calculate memory growth rate
    fn calculate_memory_growth_rate(
        memory_history: &Arc<Mutex<VecDeque<MemoryUsageSample>>>,
        current_memory_mb: f64,
    ) -> f64 {
        if let Ok(history) = memory_history.lock() {
            if let Some(last_sample) = history.back() {
                let time_diff = Instant::now().duration_since(last_sample.timestamp).as_secs_f64();
                if time_diff > 0.0 {
                    let memory_diff = current_memory_mb - last_sample.used_memory_mb;
                    return memory_diff / time_diff;
                }
            }
        }
        0.0
    }

    /// Simulate memory fragmentation
    fn simulate_memory_fragmentation(allocated_mb: f64) -> f64 {
        // Simulate increasing fragmentation with more allocations
        let base_fragmentation = 5.0; // 5% base fragmentation
        let fragmentation_growth = (allocated_mb / 1000.0) * 2.0; // 2% per GB
        (base_fragmentation + fragmentation_growth).min(50.0) // Cap at 50%
    }

    /// Simulate active allocations count
    fn simulate_active_allocations(allocated_mb: f64) -> usize {
        // Estimate allocations based on memory usage (assuming average 1MB per allocation)
        (allocated_mb as usize).max(1)
    }

    /// Calculate allocation velocity (allocations per second)
    fn calculate_allocation_velocity(
        memory_history: &Arc<Mutex<VecDeque<MemoryUsageSample>>>,
    ) -> f64 {
        if let Ok(history) = memory_history.lock() {
            if history.len() >= 2 {
                let current = &history[history.len() - 1];
                let previous = &history[history.len() - 2];
                let time_diff = current.timestamp.duration_since(previous.timestamp).as_secs_f64();
                if time_diff > 0.0 {
                    let allocation_diff = current.active_allocations as f64 - previous.active_allocations as f64;
                    return allocation_diff / time_diff;
                }
            }
        }
        0.0
    }

    /// Analyze memory sample for OOM conditions
    fn analyze_oom_conditions(
        sample: &MemoryUsageSample,
        config: &OomDetectionConfig,
        oom_events: &Arc<Mutex<Vec<OomEvent>>>,
    ) {
        // Check memory usage thresholds
        if sample.memory_usage_percent >= config.memory_imminent_oom_threshold_percent {
            Self::record_oom_event(
                oom_events,
                OomEventType::OomImminent,
                OomSeverity::Emergency,
                sample.clone(),
                "Memory usage reached imminent OOM threshold".to_string(),
            );
        } else if sample.memory_usage_percent >= config.memory_critical_threshold_percent {
            Self::record_oom_event(
                oom_events,
                OomEventType::MemoryCritical,
                OomSeverity::Critical,
                sample.clone(),
                "Memory usage reached critical threshold".to_string(),
            );
        } else if sample.memory_usage_percent >= config.memory_warning_threshold_percent {
            Self::record_oom_event(
                oom_events,
                OomEventType::MemoryWarning,
                OomSeverity::Warning,
                sample.clone(),
                "Memory usage reached warning threshold".to_string(),
            );
        }

        // Check memory growth rate
        if sample.memory_growth_rate >= config.memory_growth_rate_alert_threshold {
            Self::record_oom_event(
                oom_events,
                OomEventType::RapidMemoryGrowth,
                OomSeverity::Critical,
                sample.clone(),
                format!("Rapid memory growth detected: {:.2} MB/sec", sample.memory_growth_rate),
            );
        }

        // Check for fragmentation issues
        if sample.fragmentation_percent > 40.0 {
            Self::record_oom_event(
                oom_events,
                OomEventType::FragmentationCritical,
                OomSeverity::Critical,
                sample.clone(),
                format!("Critical memory fragmentation: {:.1}%", sample.fragmentation_percent),
            );
        }
    }

    /// Record OOM event
    fn record_oom_event(
        oom_events: &Arc<Mutex<Vec<OomEvent>>>,
        event_type: OomEventType,
        severity: OomSeverity,
        memory_state: MemoryUsageSample,
        analysis_details: String,
    ) {
        let event = OomEvent {
            timestamp: Instant::now(),
            event_type,
            severity,
            memory_state,
            prediction: None,
            response_actions: Vec::new(),
            mitigation_successful: false,
            analysis_details,
        };

        if let Ok(mut events) = oom_events.lock() {
            events.push(event);
        }
    }

    /// Run OOM prediction thread
    fn run_oom_prediction_thread(
        config: OomDetectionConfig,
        memory_history: Arc<Mutex<VecDeque<MemoryUsageSample>>>,
        oom_events: Arc<Mutex<Vec<OomEvent>>>,
        prediction_state: Arc<Mutex<OomPredictionState>>,
        detection_active: Arc<AtomicBool>,
    ) {
        while detection_active.load(Ordering::SeqCst) {
            if let Ok(history) = memory_history.lock() {
                if history.len() >= 10 { // Need sufficient samples for prediction
                    let predictions = Self::generate_oom_predictions(&history, &config);
                    
                    for prediction in predictions {
                        if prediction.confidence >= config.min_prediction_confidence {
                            // Update prediction state
                            if let Ok(mut state) = prediction_state.lock() {
                                state.last_prediction = Some(prediction.clone());
                            }

                            // Generate prediction event if OOM is predicted soon
                            if prediction.time_to_oom.as_secs() < config.prediction_window_secs {
                                let current_sample = history.back().unwrap().clone();
                                Self::record_oom_prediction_event(
                                    &oom_events,
                                    current_sample,
                                    prediction,
                                );
                            }
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(config.monitoring_interval_ms * 4)); // Predict less frequently
        }
    }

    /// Generate OOM predictions using multiple models
    fn generate_oom_predictions(
        history: &VecDeque<MemoryUsageSample>,
        config: &OomDetectionConfig,
    ) -> Vec<OomPrediction> {
        let mut predictions = Vec::new();

        // Linear regression prediction
        if let Some(linear_prediction) = Self::predict_oom_linear_regression(history, config) {
            predictions.push(linear_prediction);
        }

        // Exponential trend prediction
        if let Some(exponential_prediction) = Self::predict_oom_exponential_trend(history, config) {
            predictions.push(exponential_prediction);
        }

        // Moving average prediction
        if let Some(moving_avg_prediction) = Self::predict_oom_moving_average(history, config) {
            predictions.push(moving_avg_prediction);
        }

        predictions
    }

    /// Predict OOM using linear regression
    fn predict_oom_linear_regression(
        history: &VecDeque<MemoryUsageSample>,
        config: &OomDetectionConfig,
    ) -> Option<OomPrediction> {
        if history.len() < 5 {
            return None;
        }

        let samples: Vec<&MemoryUsageSample> = history.iter().rev().take(20).collect();
        let start_time = samples.last()?.timestamp;

        // Calculate linear regression for memory usage
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x_squared = 0.0;
        let n = samples.len() as f64;

        for (i, sample) in samples.iter().enumerate() {
            let x = sample.timestamp.duration_since(start_time).as_secs_f64();
            let y = sample.memory_usage_percent;
            
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x_squared += x * x;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x_squared - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        if slope <= 0.0 {
            return None; // No growth, no OOM predicted
        }

        // Calculate when we'll reach 100% memory usage
        let current_time = samples[0].timestamp.duration_since(start_time).as_secs_f64();
        let current_usage = samples[0].memory_usage_percent;
        let time_to_100_percent = (100.0 - current_usage) / slope;
        
        if time_to_100_percent <= 0.0 {
            return None;
        }

        // Calculate confidence based on R-squared
        let mean_y = sum_y / n;
        let mut ss_tot = 0.0;
        let mut ss_res = 0.0;

        for sample in &samples {
            let x = sample.timestamp.duration_since(start_time).as_secs_f64();
            let y = sample.memory_usage_percent;
            let y_pred = slope * x + intercept;
            
            ss_tot += (y - mean_y).powi(2);
            ss_res += (y - y_pred).powi(2);
        }

        let r_squared = if ss_tot > 0.0 { 1.0 - (ss_res / ss_tot) } else { 0.0 };
        let confidence = r_squared.max(0.0).min(1.0);

        let predicted_memory_at_oom = samples[0].used_memory_mb + 
            (time_to_100_percent * slope * samples[0].total_memory_mb / 100.0);

        Some(OomPrediction {
            time_to_oom: Duration::from_secs_f64(time_to_100_percent),
            confidence,
            predicted_memory_at_oom_mb: predicted_memory_at_oom,
            growth_pattern: MemoryGrowthPattern::Linear,
            contributing_factors: vec![
                "Linear memory growth detected".to_string(),
                format!("Growth rate: {:.2}%/sec", slope),
            ],
            recommended_actions: vec![
                "Monitor memory usage closely".to_string(),
                "Consider memory cleanup".to_string(),
                "Reduce allocation rate".to_string(),
            ],
            prediction_model: PredictionModel::LinearRegression,
        })
    }

    /// Predict OOM using exponential trend analysis
    fn predict_oom_exponential_trend(
        history: &VecDeque<MemoryUsageSample>,
        config: &OomDetectionConfig,
    ) -> Option<OomPrediction> {
        if history.len() < 8 {
            return None;
        }

        let samples: Vec<&MemoryUsageSample> = history.iter().rev().take(15).collect();
        
        // Check if growth is accelerating (exponential pattern)
        let mut growth_rates = Vec::new();
        for i in 1..samples.len() {
            let time_diff = samples[i-1].timestamp.duration_since(samples[i].timestamp).as_secs_f64();
            if time_diff > 0.0 {
                let usage_diff = samples[i-1].memory_usage_percent - samples[i].memory_usage_percent;
                growth_rates.push(usage_diff / time_diff);
            }
        }

        if growth_rates.len() < 3 {
            return None;
        }

        // Check if growth rate is increasing (exponential pattern)
        let recent_avg = growth_rates.iter().take(3).sum::<f64>() / 3.0;
        let older_avg = growth_rates.iter().skip(growth_rates.len() - 3).sum::<f64>() / 3.0;
        
        if recent_avg <= older_avg * 1.2 { // Require 20% acceleration
            return None;
        }

        let current_usage = samples[0].memory_usage_percent;
        let acceleration = (recent_avg - older_avg) / older_avg;
        
        // Estimate time to OOM with exponential growth
        let time_to_oom_secs = if acceleration > 0.0 {
            let exponential_factor = 1.0 + acceleration;
            ((100.0 / current_usage).ln() / exponential_factor.ln()).max(1.0)
        } else {
            f64::INFINITY
        };

        if !time_to_oom_secs.is_finite() || time_to_oom_secs > 3600.0 { // Max 1 hour prediction
            return None;
        }

        let confidence = (acceleration * 2.0).min(0.9); // Higher acceleration = higher confidence

        Some(OomPrediction {
            time_to_oom: Duration::from_secs_f64(time_to_oom_secs),
            confidence,
            predicted_memory_at_oom_mb: samples[0].total_memory_mb,
            growth_pattern: MemoryGrowthPattern::Exponential,
            contributing_factors: vec![
                "Exponential memory growth detected".to_string(),
                format!("Growth acceleration: {:.1}%", acceleration * 100.0),
                "Memory usage increasing at accelerating rate".to_string(),
            ],
            recommended_actions: vec![
                "URGENT: Immediate memory cleanup required".to_string(),
                "Stop non-critical allocations".to_string(),
                "Force garbage collection".to_string(),
                "Consider system restart".to_string(),
            ],
            prediction_model: PredictionModel::ExponentialTrend,
        })
    }

    /// Predict OOM using moving average
    fn predict_oom_moving_average(
        history: &VecDeque<MemoryUsageSample>,
        config: &OomDetectionConfig,
    ) -> Option<OomPrediction> {
        if history.len() < 10 {
            return None;
        }

        let window_size = 5;
        let samples: Vec<&MemoryUsageSample> = history.iter().rev().take(20).collect();
        
        // Calculate moving averages
        let mut moving_averages = Vec::new();
        for i in window_size..=samples.len() {
            let window = &samples[i-window_size..i];
            let avg_usage = window.iter().map(|s| s.memory_usage_percent).sum::<f64>() / window_size as f64;
            moving_averages.push(avg_usage);
        }

        if moving_averages.len() < 3 {
            return None;
        }

        // Calculate trend from moving averages
        let recent_avg = moving_averages[0];
        let older_avg = moving_averages[moving_averages.len() - 1];
        let avg_time_span = (window_size * samples.len() / moving_averages.len()) as f64 * 
                           config.monitoring_interval_ms as f64 / 1000.0;
        
        let growth_rate = (recent_avg - older_avg) / avg_time_span;
        
        if growth_rate <= 0.0 {
            return None;
        }

        let time_to_oom = (100.0 - recent_avg) / growth_rate;
        
        if time_to_oom <= 0.0 || time_to_oom > 1800.0 { // Max 30 minutes
            return None;
        }

        // Calculate confidence based on trend consistency
        let variance = moving_averages.iter()
            .map(|&avg| (avg - recent_avg).powi(2))
            .sum::<f64>() / moving_averages.len() as f64;
        let confidence = (1.0 / (1.0 + variance)).max(0.3).min(0.8);

        Some(OomPrediction {
            time_to_oom: Duration::from_secs_f64(time_to_oom),
            confidence,
            predicted_memory_at_oom_mb: samples[0].total_memory_mb,
            growth_pattern: MemoryGrowthPattern::Linear,
            contributing_factors: vec![
                "Consistent memory growth trend".to_string(),
                format!("Average growth rate: {:.2}%/sec", growth_rate),
            ],
            recommended_actions: vec![
                "Monitor trend continuation".to_string(),
                "Prepare for potential memory cleanup".to_string(),
                "Review recent allocations".to_string(),
            ],
            prediction_model: PredictionModel::MovingAverage,
        })
    }

    /// Record OOM prediction event
    fn record_oom_prediction_event(
        oom_events: &Arc<Mutex<Vec<OomEvent>>>,
        memory_state: MemoryUsageSample,
        prediction: OomPrediction,
    ) {
        let event = OomEvent {
            timestamp: Instant::now(),
            event_type: OomEventType::OomImminent,
            severity: OomSeverity::Critical,
            memory_state,
            prediction: Some(prediction),
            response_actions: Vec::new(),
            mitigation_successful: false,
            analysis_details: "OOM condition predicted by predictive analysis".to_string(),
        };

        if let Ok(mut events) = oom_events.lock() {
            events.push(event);
        }
    }

    /// Run prevention action thread
    fn run_prevention_action_thread(
        config: OomDetectionConfig,
        oom_events: Arc<Mutex<Vec<OomEvent>>>,
        test_allocations: Arc<Mutex<Vec<Vec<u8>>>>,
        prevention_actions: Arc<Mutex<Vec<PreventionAction>>>,
        prevention_active: Arc<AtomicBool>,
        total_allocated: Arc<AtomicU64>,
    ) {
        while prevention_active.load(Ordering::SeqCst) {
            // Check for events requiring intervention
            let needs_intervention = if let Ok(events) = oom_events.lock() {
                events.iter().rev().take(5).any(|event| {
                    matches!(event.event_type, 
                        OomEventType::MemoryCritical | 
                        OomEventType::OomImminent | 
                        OomEventType::RapidMemoryGrowth
                    ) && !event.mitigation_successful
                })
            } else {
                false
            };

            if needs_intervention {
                let action_taken = Self::execute_prevention_action(
                    &test_allocations,
                    &total_allocated,
                    &prevention_actions,
                );

                if action_taken {
                    // Mark recent events as mitigated
                    if let Ok(mut events) = oom_events.lock() {
                        for event in events.iter_mut().rev().take(3) {
                            if matches!(event.event_type, 
                                OomEventType::MemoryCritical | 
                                OomEventType::OomImminent | 
                                OomEventType::RapidMemoryGrowth
                            ) {
                                event.mitigation_successful = true;
                                event.response_actions.push("Emergency memory cleanup executed".to_string());
                            }
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(config.monitoring_interval_ms));
        }
    }

    /// Execute prevention action
    fn execute_prevention_action(
        test_allocations: &Arc<Mutex<Vec<Vec<u8>>>>,
        total_allocated: &Arc<AtomicU64>,
        prevention_actions: &Arc<Mutex<Vec<PreventionAction>>>,
    ) -> bool {
        let action_start = Instant::now();
        let initial_memory = total_allocated.load(Ordering::SeqCst);

        // Perform emergency cleanup
        let memory_freed = if let Ok(mut allocations) = test_allocations.lock() {
            let freed_count = allocations.len() / 2; // Free half of allocations
            let freed_mb = freed_count as f64; // Assuming 1MB per allocation
            
            allocations.truncate(allocations.len() - freed_count);
            total_allocated.fetch_sub((freed_mb as u64), Ordering::SeqCst);
            
            freed_mb
        } else {
            0.0
        };

        let execution_time = action_start.elapsed();
        let action_successful = memory_freed > 0.0;

        // Record prevention action
        let prevention_action = PreventionAction {
            timestamp: Instant::now(),
            action_type: PreventionActionType::EmergencyCleanup,
            trigger_state: MemoryUsageSample {
                timestamp: action_start,
                available_memory_mb: 1000.0, // Simulated
                used_memory_mb: initial_memory as f64,
                total_memory_mb: 8192.0,
                memory_usage_percent: (initial_memory as f64 / 8192.0) * 100.0,
                memory_growth_rate: 0.0,
                active_allocations: initial_memory as usize,
                fragmentation_percent: 20.0,
                memory_pressure_level: 0.8,
                allocation_velocity: 0.0,
            },
            memory_freed_mb: memory_freed,
            action_successful,
            execution_time,
            performance_impact: PerformanceImpact {
                response_time_impact_percent: 15.0,
                throughput_impact_percent: 10.0,
                user_experience_impact: UserExperienceImpact::Minor,
                estimated_recovery_time: Duration::from_secs(5),
            },
        };

        if let Ok(mut actions) = prevention_actions.lock() {
            actions.push(prevention_action);
        }

        action_successful
    }

    /// Run memory stress test thread
    fn run_memory_stress_thread(
        config: OomDetectionConfig,
        test_allocations: Arc<Mutex<Vec<Vec<u8>>>>,
        total_allocated: Arc<AtomicU64>,
        detection_active: Arc<AtomicBool>,
    ) {
        let start_time = Instant::now();
        let allocation_size_mb = 1; // 1MB per allocation
        let allocation_size_bytes = allocation_size_mb * 1024 * 1024;
        let max_allocations = config.max_test_allocation_mb;

        while detection_active.load(Ordering::SeqCst) {
            if start_time.elapsed().as_secs() >= config.max_test_duration_secs {
                break;
            }

            let current_allocated = total_allocated.load(Ordering::SeqCst) as usize;
            
            if current_allocated < max_allocations {
                // Allocate memory to simulate stress
                let allocation = vec![0u8; allocation_size_bytes];
                
                if let Ok(mut allocations) = test_allocations.lock() {
                    allocations.push(allocation);
                    total_allocated.fetch_add(allocation_size_mb as u64, Ordering::SeqCst);
                }

                // Variable allocation rate to create interesting patterns
                let allocation_delay = match current_allocated {
                    0..=100 => 100,      // Slow start
                    101..=300 => 50,     // Medium speed
                    301..=500 => 20,     // Fast allocation
                    _ => 200,            // Slow down near limit
                };

                thread::sleep(Duration::from_millis(allocation_delay));
            } else {
                // Near memory limit, slow down allocations
                thread::sleep(Duration::from_millis(1000));
            }
        }
    }

    /// Get comprehensive OOM analysis results
    pub fn get_oom_analysis_results(&self) -> Result<OomAnalysisResults> {
        let memory_history = self.memory_history.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock memory history".to_string())
        })?;

        let oom_events = self.oom_events.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock OOM events".to_string())
        })?;

        let prevention_actions = self.prevention_actions.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock prevention actions".to_string())
        })?;

        let prediction_state = self.prediction_state.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock prediction state".to_string())
        })?;

        // Calculate test duration
        let test_duration = if let (Some(first), Some(last)) = (memory_history.front(), memory_history.back()) {
            last.timestamp.duration_since(first.timestamp)
        } else {
            Duration::from_secs(0)
        };

        // Count events by type
        let mut events_by_type = HashMap::new();
        for event in oom_events.iter() {
            *events_by_type.entry(event.event_type.clone()).or_insert(0) += 1;
        }

        // Calculate peak memory usage
        let peak_memory_usage_mb = self.peak_memory_usage_mb.load(Ordering::SeqCst) as f64;

        // Calculate prediction metrics
        let total_predictions = prediction_state.prediction_accuracy_history.len();
        let prediction_accuracy = if total_predictions > 0 {
            prediction_state.prediction_accuracy_history.iter().sum::<f64>() / total_predictions as f64
        } else {
            0.0
        };

        // Calculate prevention metrics
        let prevention_actions_taken = prevention_actions.len();
        let successful_preventions = prevention_actions.iter()
            .filter(|action| action.action_successful)
            .count();
        let prevention_success_rate = if prevention_actions_taken > 0 {
            successful_preventions as f64 / prevention_actions_taken as f64
        } else {
            0.0
        };

        // Analyze memory growth
        let growth_analysis = Self::analyze_memory_growth(&memory_history);

        // Calculate resilience metrics
        let resilience_metrics = Self::calculate_resilience_metrics(&oom_events, &prevention_actions);

        // Generate improvement recommendations
        let improvement_recommendations = Self::generate_improvement_recommendations(
            &events_by_type,
            prediction_accuracy,
            prevention_success_rate,
            &growth_analysis,
        );

        Ok(OomAnalysisResults {
            test_duration,
            events_by_type,
            peak_memory_usage_mb,
            total_predictions_made: total_predictions,
            prediction_accuracy_percent: prediction_accuracy * 100.0,
            prevention_actions_taken,
            prevention_success_rate,
            growth_analysis,
            resilience_metrics,
            improvement_recommendations,
        })
    }

    /// Analyze memory growth patterns
    fn analyze_memory_growth(history: &VecDeque<MemoryUsageSample>) -> MemoryGrowthAnalysis {
        if history.is_empty() {
            return MemoryGrowthAnalysis {
                average_growth_rate: 0.0,
                peak_growth_rate: 0.0,
                primary_growth_pattern: MemoryGrowthPattern::Linear,
                growth_acceleration: 0.0,
                memory_efficiency_score: 1.0,
                leak_probability_score: 0.0,
            };
        }

        let growth_rates: Vec<f64> = history.iter().map(|s| s.memory_growth_rate).collect();
        let average_growth_rate = growth_rates.iter().sum::<f64>() / growth_rates.len() as f64;
        let peak_growth_rate = growth_rates.iter().fold(0.0, |a, &b| a.max(b));

        // Determine primary growth pattern
        let primary_pattern = Self::determine_primary_growth_pattern(history);

        // Calculate growth acceleration
        let growth_acceleration = Self::calculate_growth_acceleration(&growth_rates);

        // Calculate memory efficiency (how much useful work per MB)
        let memory_efficiency = Self::calculate_memory_efficiency(history);

        // Calculate leak probability based on sustained growth
        let leak_probability = Self::calculate_leak_probability(history);

        MemoryGrowthAnalysis {
            average_growth_rate,
            peak_growth_rate,
            primary_growth_pattern: primary_pattern,
            growth_acceleration,
            memory_efficiency_score: memory_efficiency,
            leak_probability_score: leak_probability,
        }
    }

    /// Determine primary memory growth pattern
    fn determine_primary_growth_pattern(history: &VecDeque<MemoryUsageSample>) -> MemoryGrowthPattern {
        if history.len() < 10 {
            return MemoryGrowthPattern::Linear;
        }

        let growth_rates: Vec<f64> = history.iter().map(|s| s.memory_growth_rate).collect();
        let usage_values: Vec<f64> = history.iter().map(|s| s.memory_usage_percent).collect();

        // Check for exponential pattern (increasing growth rates)
        let first_half_avg = growth_rates.iter().take(growth_rates.len() / 2).sum::<f64>() / (growth_rates.len() / 2) as f64;
        let second_half_avg = growth_rates.iter().skip(growth_rates.len() / 2).sum::<f64>() / (growth_rates.len() / 2) as f64;
        
        if second_half_avg > first_half_avg * 1.5 {
            return MemoryGrowthPattern::Exponential;
        }

        // Check for memory leak pattern (sustained positive growth)
        let positive_growth_ratio = growth_rates.iter().filter(|&&rate| rate > 0.1).count() as f64 / growth_rates.len() as f64;
        if positive_growth_ratio > 0.8 {
            return MemoryGrowthPattern::MemoryLeak;
        }

        // Check for burst pattern (high variance in growth rates)
        let mean_growth = growth_rates.iter().sum::<f64>() / growth_rates.len() as f64;
        let variance = growth_rates.iter().map(|&rate| (rate - mean_growth).powi(2)).sum::<f64>() / growth_rates.len() as f64;
        if variance > mean_growth * 2.0 {
            return MemoryGrowthPattern::BurstAllocations;
        }

        MemoryGrowthPattern::Linear
    }

    /// Calculate growth acceleration
    fn calculate_growth_acceleration(growth_rates: &[f64]) -> f64 {
        if growth_rates.len() < 3 {
            return 0.0;
        }

        let mut accelerations = Vec::new();
        for i in 2..growth_rates.len() {
            let acceleration = growth_rates[i] - growth_rates[i-1];
            accelerations.push(acceleration);
        }

        accelerations.iter().sum::<f64>() / accelerations.len() as f64
    }

    /// Calculate memory efficiency score
    fn calculate_memory_efficiency(history: &VecDeque<MemoryUsageSample>) -> f64 {
        if history.is_empty() {
            return 1.0;
        }

        // Efficiency based on how much memory is actually being used vs allocated
        let avg_usage = history.iter().map(|s| s.memory_usage_percent).sum::<f64>() / history.len() as f64;
        let avg_fragmentation = history.iter().map(|s| s.fragmentation_percent).sum::<f64>() / history.len() as f64;
        
        // Higher usage with lower fragmentation = higher efficiency
        let efficiency = (avg_usage / 100.0) * (1.0 - avg_fragmentation / 100.0);
        efficiency.max(0.0).min(1.0)
    }

    /// Calculate memory leak probability
    fn calculate_leak_probability(history: &VecDeque<MemoryUsageSample>) -> f64 {
        if history.len() < 20 {
            return 0.0;
        }

        // Look for sustained memory growth without corresponding decreases
        let usage_values: Vec<f64> = history.iter().map(|s| s.memory_usage_percent).collect();
        
        let mut increasing_streaks = Vec::new();
        let mut current_streak = 0;
        
        for i in 1..usage_values.len() {
            if usage_values[i] > usage_values[i-1] {
                current_streak += 1;
            } else {
                if current_streak > 0 {
                    increasing_streaks.push(current_streak);
                    current_streak = 0;
                }
            }
        }
        
        if current_streak > 0 {
            increasing_streaks.push(current_streak);
        }

        // Long increasing streaks suggest memory leaks
        let max_streak = increasing_streaks.iter().max().unwrap_or(&0);
        let leak_probability = (*max_streak as f64 / history.len() as f64).min(1.0);
        
        leak_probability
    }

    /// Calculate system resilience metrics
    fn calculate_resilience_metrics(
        oom_events: &[OomEvent],
        prevention_actions: &[PreventionAction],
    ) -> SystemResilienceMetrics {
        let oom_threats_mitigated = oom_events.iter()
            .filter(|event| event.mitigation_successful)
            .count();

        let recovery_times: Vec<Duration> = prevention_actions.iter()
            .map(|action| action.execution_time)
            .collect();

        let average_recovery_time = if recovery_times.is_empty() {
            Duration::from_secs(0)
        } else {
            let total_ms: u64 = recovery_times.iter().map(|d| d.as_millis() as u64).sum();
            Duration::from_millis(total_ms / recovery_times.len() as u64)
        };

        let successful_interventions = prevention_actions.iter()
            .filter(|action| action.action_successful)
            .count();

        let proactive_effectiveness = if prevention_actions.is_empty() {
            0.0
        } else {
            successful_interventions as f64 / prevention_actions.len() as f64 * 100.0
        };

        let stability_score = if oom_events.is_empty() {
            1.0
        } else {
            let critical_events = oom_events.iter()
                .filter(|event| matches!(event.severity, OomSeverity::Critical | OomSeverity::Emergency))
                .count();
            ((oom_events.len() - critical_events) as f64 / oom_events.len() as f64).max(0.0)
        };

        let resource_utilization = if prevention_actions.is_empty() {
            1.0
        } else {
            let total_freed = prevention_actions.iter()
                .map(|action| action.memory_freed_mb)
                .sum::<f64>();
            (total_freed / 1000.0).min(1.0) // Normalize against 1GB
        };

        SystemResilienceMetrics {
            oom_threats_mitigated,
            average_recovery_time,
            stability_score,
            proactive_effectiveness_percent: proactive_effectiveness,
            resource_utilization_efficiency: resource_utilization,
        }
    }

    /// Generate improvement recommendations
    fn generate_improvement_recommendations(
        events_by_type: &HashMap<OomEventType, usize>,
        prediction_accuracy: f64,
        prevention_success_rate: f64,
        growth_analysis: &MemoryGrowthAnalysis,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Prediction accuracy recommendations
        if prediction_accuracy < 0.7 {
            recommendations.push("Improve OOM prediction accuracy by tuning models".to_string());
            recommendations.push("Collect more memory usage samples for better trend analysis".to_string());
        }

        // Prevention effectiveness recommendations
        if prevention_success_rate < 0.8 {
            recommendations.push("Enhance prevention actions effectiveness".to_string());
            recommendations.push("Implement more aggressive memory cleanup strategies".to_string());
        }

        // Memory growth pattern recommendations
        match growth_analysis.primary_growth_pattern {
            MemoryGrowthPattern::Exponential => {
                recommendations.push("CRITICAL: Address exponential memory growth immediately".to_string());
                recommendations.push("Implement circuit breaker pattern for allocations".to_string());
            },
            MemoryGrowthPattern::MemoryLeak => {
                recommendations.push("Investigate and fix memory leaks".to_string());
                recommendations.push("Implement automatic leak detection in production".to_string());
            },
            MemoryGrowthPattern::BurstAllocations => {
                recommendations.push("Optimize burst allocation patterns".to_string());
                recommendations.push("Implement allocation rate limiting".to_string());
            },
            _ => {}
        }

        // Memory efficiency recommendations
        if growth_analysis.memory_efficiency_score < 0.6 {
            recommendations.push("Improve memory utilization efficiency".to_string());
            recommendations.push("Reduce memory fragmentation".to_string());
        }

        // Event-specific recommendations
        if events_by_type.get(&OomEventType::FragmentationCritical).unwrap_or(&0) > &5 {
            recommendations.push("Implement memory compaction strategies".to_string());
        }

        if events_by_type.get(&OomEventType::RapidMemoryGrowth).unwrap_or(&0) > &3 {
            recommendations.push("Add memory growth rate monitoring to production".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("OOM detection system performing well".to_string());
            recommendations.push("Continue monitoring for optimization opportunities".to_string());
        }

        recommendations
    }

    /// Force trigger specific OOM condition for testing
    pub fn force_oom_condition(&self, condition_type: OomEventType) -> Result<()> {
        let current_sample = if let Ok(history) = self.memory_history.lock() {
            history.back().cloned().unwrap_or_else(|| MemoryUsageSample {
                timestamp: Instant::now(),
                available_memory_mb: 1000.0,
                used_memory_mb: 6000.0,
                total_memory_mb: 8192.0,
                memory_usage_percent: 90.0,
                memory_growth_rate: 5.0,
                active_allocations: 100,
                fragmentation_percent: 25.0,
                memory_pressure_level: 0.9,
                allocation_velocity: 10.0,
            })
        } else {
            return Err(sublime_monorepo_tools::Error::generic(
                "Failed to access memory history".to_string()
            ));
        };

        let severity = match condition_type {
            OomEventType::MemoryWarning => OomSeverity::Warning,
            OomEventType::MemoryCritical => OomSeverity::Critical,
            OomEventType::OomImminent | OomEventType::OomOccurred => OomSeverity::Emergency,
            _ => OomSeverity::Critical,
        };

        Self::record_oom_event(
            &self.oom_events,
            condition_type,
            severity,
            current_sample,
            "Manually triggered OOM condition for testing".to_string(),
        );

        Ok(())
    }

    /// Get current memory usage state
    pub fn get_current_memory_state(&self) -> MemoryUsageSample {
        Self::capture_memory_sample(&self.total_allocated_mb, &self.memory_history)
    }

    /// Get latest OOM prediction
    pub fn get_latest_prediction(&self) -> Result<Option<OomPrediction>> {
        let prediction_state = self.prediction_state.lock().map_err(|_| {
            sublime_monorepo_tools::Error::generic("Failed to lock prediction state".to_string())
        })?;

        Ok(prediction_state.last_prediction.clone())
    }

    /// Force emergency memory cleanup
    pub fn force_emergency_cleanup(&self) -> Result<f64> {
        Self::execute_prevention_action(
            &self.test_allocations,
            &self.total_allocated_mb,
            &self.prevention_actions,
        );

        // Return amount of memory freed
        Ok(self.prevention_actions.lock()
            .map(|actions| actions.last().map(|action| action.memory_freed_mb).unwrap_or(0.0))
            .unwrap_or(0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oom_detection_config_creation() {
        let config = OomDetectionConfig::default();
        assert!(config.max_test_duration_secs > 0);
        assert!(config.memory_warning_threshold_percent > 0.0);
        assert!(config.memory_critical_threshold_percent > config.memory_warning_threshold_percent);
        assert!(config.memory_imminent_oom_threshold_percent > config.memory_critical_threshold_percent);
        assert!(config.min_prediction_confidence > 0.0);
        assert!(config.min_prediction_confidence <= 1.0);
    }

    #[test]
    fn test_oom_detection_system_creation() {
        let config = OomDetectionConfig::default();
        let system = OomDetectionSystem::new(config);
        
        assert!(!system.detection_active.load(Ordering::SeqCst));
        assert!(!system.prevention_active.load(Ordering::SeqCst));
        assert_eq!(system.total_allocated_mb.load(Ordering::SeqCst), 0);
        assert_eq!(system.peak_memory_usage_mb.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_memory_usage_sample_creation() {
        let total_allocated = Arc::new(AtomicU64::new(1024)); // 1GB
        let memory_history = Arc::new(Mutex::new(VecDeque::new()));
        
        let sample = OomDetectionSystem::capture_memory_sample(&total_allocated, &memory_history);
        
        assert!(sample.used_memory_mb > 0.0);
        assert!(sample.total_memory_mb > 0.0);
        assert!(sample.memory_usage_percent >= 0.0);
        assert!(sample.memory_usage_percent <= 100.0);
        assert!(sample.memory_pressure_level >= 0.0);
        assert!(sample.memory_pressure_level <= 1.0);
    }

    #[test]
    fn test_oom_prediction_linear_regression() {
        let mut history = VecDeque::new();
        let start_time = Instant::now();
        
        // Create samples with linear growth
        for i in 0..10 {
            let sample = MemoryUsageSample {
                timestamp: start_time + Duration::from_secs(i * 10),
                available_memory_mb: 8192.0 - (2048.0 + i as f64 * 100.0),
                used_memory_mb: 2048.0 + i as f64 * 100.0,
                total_memory_mb: 8192.0,
                memory_usage_percent: 25.0 + i as f64 * 1.2, // Linear growth
                memory_growth_rate: 1.2,
                active_allocations: 100 + i,
                fragmentation_percent: 15.0,
                memory_pressure_level: 0.3 + i as f64 * 0.05,
                allocation_velocity: 1.0,
            };
            history.push_back(sample);
        }

        let config = OomDetectionConfig::default();
        let prediction = OomDetectionSystem::predict_oom_linear_regression(&history, &config);
        
        assert!(prediction.is_some());
        let pred = prediction.unwrap();
        assert!(pred.confidence > 0.0);
        assert!(pred.time_to_oom.as_secs() > 0);
        assert_eq!(pred.prediction_model, PredictionModel::LinearRegression);
        assert_eq!(pred.growth_pattern, MemoryGrowthPattern::Linear);
    }

    #[test]
    fn test_memory_growth_pattern_detection() {
        let mut history = VecDeque::new();
        let start_time = Instant::now();
        
        // Create samples with exponential growth pattern
        for i in 0..15 {
            let growth_factor = (1.2_f64).powi(i); // Exponential growth
            let sample = MemoryUsageSample {
                timestamp: start_time + Duration::from_secs(i as u64 * 5),
                available_memory_mb: 8192.0,
                used_memory_mb: 100.0 * growth_factor,
                total_memory_mb: 8192.0,
                memory_usage_percent: (100.0 * growth_factor / 8192.0) * 100.0,
                memory_growth_rate: growth_factor * 0.2, // Increasing growth rate
                active_allocations: (100.0 * growth_factor) as usize,
                fragmentation_percent: 10.0,
                memory_pressure_level: 0.1,
                allocation_velocity: 1.0,
            };
            history.push_back(sample);
        }

        let pattern = OomDetectionSystem::determine_primary_growth_pattern(&history);
        assert_eq!(pattern, MemoryGrowthPattern::Exponential);
    }

    #[tokio::test]
    async fn test_oom_detection_integration() -> Result<()> {
        let config = OomDetectionConfig {
            max_test_duration_secs: 3, // Short test
            monitoring_interval_ms: 100,
            max_test_allocation_mb: 100, // Small limit for testing
            enable_realtime_monitoring: true,
            enable_predictive_analysis: true,
            enable_automatic_prevention: true,
            ..Default::default()
        };

        let system = OomDetectionSystem::new(config);
        
        // Start detection
        system.start_oom_detection()?;
        
        // Wait for some operations
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Check current memory state
        let memory_state = system.get_current_memory_state();
        assert!(memory_state.used_memory_mb > 0.0);
        
        // Force an OOM condition
        system.force_oom_condition(OomEventType::MemoryCritical)?;
        
        // Wait for detection and response
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Test emergency cleanup
        let memory_freed = system.force_emergency_cleanup()?;
        assert!(memory_freed >= 0.0);
        
        // Stop detection
        system.stop_oom_detection();
        
        // Get analysis results
        tokio::time::sleep(Duration::from_millis(100)).await;
        let results = system.get_oom_analysis_results()?;
        
        assert!(results.test_duration.as_millis() > 0);
        assert!(!results.events_by_type.is_empty());
        assert!(!results.improvement_recommendations.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_memory_efficiency_calculation() {
        let mut history = VecDeque::new();
        
        // High efficiency scenario - high usage, low fragmentation
        for i in 0..10 {
            let sample = MemoryUsageSample {
                timestamp: Instant::now(),
                available_memory_mb: 1000.0,
                used_memory_mb: 7000.0,
                total_memory_mb: 8192.0,
                memory_usage_percent: 85.0, // High usage
                memory_growth_rate: 1.0,
                active_allocations: 100,
                fragmentation_percent: 5.0, // Low fragmentation
                memory_pressure_level: 0.8,
                allocation_velocity: 1.0,
            };
            history.push_back(sample);
        }

        let efficiency = OomDetectionSystem::calculate_memory_efficiency(&history);
        assert!(efficiency > 0.7); // Should be high efficiency
        
        // Low efficiency scenario - low usage, high fragmentation
        history.clear();
        for i in 0..10 {
            let sample = MemoryUsageSample {
                timestamp: Instant::now(),
                available_memory_mb: 6000.0,
                used_memory_mb: 2000.0,
                total_memory_mb: 8192.0,
                memory_usage_percent: 25.0, // Low usage
                memory_growth_rate: 1.0,
                active_allocations: 100,
                fragmentation_percent: 40.0, // High fragmentation
                memory_pressure_level: 0.2,
                allocation_velocity: 1.0,
            };
            history.push_back(sample);
        }

        let efficiency = OomDetectionSystem::calculate_memory_efficiency(&history);
        assert!(efficiency < 0.3); // Should be low efficiency
    }

    #[test]
    fn test_leak_probability_calculation() {
        let mut history = VecDeque::new();
        let start_time = Instant::now();
        
        // Memory leak scenario - sustained growth
        for i in 0..30 {
            let sample = MemoryUsageSample {
                timestamp: start_time + Duration::from_secs(i as u64),
                available_memory_mb: 8000.0 - i as f64 * 100.0,
                used_memory_mb: 2000.0 + i as f64 * 100.0,
                total_memory_mb: 8192.0,
                memory_usage_percent: 25.0 + i as f64 * 1.2, // Sustained growth
                memory_growth_rate: 1.2,
                active_allocations: 100 + i,
                fragmentation_percent: 15.0,
                memory_pressure_level: 0.3 + i as f64 * 0.02,
                allocation_velocity: 1.0,
            };
            history.push_back(sample);
        }

        let leak_probability = OomDetectionSystem::calculate_leak_probability(&history);
        assert!(leak_probability > 0.5); // Should detect high leak probability
    }
}