//! Progressive Degradation Analysis and Threshold Identification
//!
//! This module implements comprehensive analysis of progressive performance degradation
//! patterns, identification of critical thresholds, and prediction of system breaking
//! points through continuous monitoring and intelligent analysis under increasing load.

use sublime_monorepo_tools::Result;
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::sync::atomic::{AtomicBool, AtomicUsize, AtomicU64, AtomicI64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

/// Configuration for progressive degradation analysis
#[derive(Debug, Clone)]
pub struct ProgressiveDegradationConfig {
    /// Maximum test duration in seconds
    pub max_test_duration_secs: u64,
    /// Analysis interval in milliseconds
    pub analysis_interval_ms: u64,
    /// Progressive load increase interval in seconds
    pub load_increase_interval_secs: u64,
    /// Load increment step (percentage increase)
    pub load_increment_step_percent: f64,
    /// Maximum load multiplier to reach
    pub max_load_multiplier: f64,
    /// Performance degradation threshold (percentage decrease)
    pub performance_degradation_threshold_percent: f64,
    /// Critical threshold breach tolerance
    pub critical_threshold_breach_tolerance: f64,
    /// Enable predictive threshold analysis
    pub enable_predictive_analysis: bool,
    /// Enable real-time degradation monitoring
    pub enable_realtime_monitoring: bool,
    /// Threshold types to monitor
    pub threshold_types: Vec<ThresholdType>,
    /// Degradation patterns to detect
    pub degradation_patterns: Vec<DegradationPattern>,
    /// Sample history size for trend analysis
    pub sample_history_size: usize,
    /// Minimum confidence for predictions
    pub min_prediction_confidence: f64,
    /// Enable automatic threshold adjustment
    pub enable_auto_threshold_adjustment: bool,
    /// Stability window for threshold validation (seconds)
    pub stability_window_secs: u64,
}

impl Default for ProgressiveDegradationConfig {
    fn default() -> Self {
        Self {
            max_test_duration_secs: 900, // 15 minutes
            analysis_interval_ms: 500,   // 500ms analysis
            load_increase_interval_secs: 30, // Increase load every 30 seconds
            load_increment_step_percent: 10.0, // 10% load increase each step
            max_load_multiplier: 5.0,    // Up to 5x normal load
            performance_degradation_threshold_percent: 20.0, // 20% performance drop
            critical_threshold_breach_tolerance: 3,  // Allow 3 breaches before critical
            enable_predictive_analysis: true,
            enable_realtime_monitoring: true,
            threshold_types: vec![
                ThresholdType::PerformanceLatency,
                ThresholdType::PerformanceThroughput,
                ThresholdType::MemoryUsage,
                ThresholdType::CpuUtilization,
                ThresholdType::ErrorRate,
                ThresholdType::ResourceExhaustion,
                ThresholdType::ResponseTime,
                ThresholdType::QueueDepth,
            ],
            degradation_patterns: vec![
                DegradationPattern::LinearDecline,
                DegradationPattern::ExponentialDecline,
                DegradationPattern::StepwiseDecline,
                DegradationPattern::CyclicalDegradation,
                DegradationPattern::SuddenDropoff,
                DegradationPattern::ProgressiveInstability,
            ],
            sample_history_size: 2000,
            min_prediction_confidence: 0.8,
            enable_auto_threshold_adjustment: true,
            stability_window_secs: 60,
        }
    }
}

/// Types of thresholds to monitor
#[derive(Debug, Clone, PartialEq)]
pub enum ThresholdType {
    /// Performance latency thresholds
    PerformanceLatency,
    /// Performance throughput thresholds
    PerformanceThroughput,
    /// Memory usage thresholds
    MemoryUsage,
    /// CPU utilization thresholds
    CpuUtilization,
    /// Error rate thresholds
    ErrorRate,
    /// Resource exhaustion thresholds
    ResourceExhaustion,
    /// Response time thresholds
    ResponseTime,
    /// Queue depth thresholds
    QueueDepth,
    /// System stability thresholds
    SystemStability,
    /// Network bandwidth thresholds
    NetworkBandwidth,
}

/// Patterns of performance degradation
#[derive(Debug, Clone, PartialEq)]
pub enum DegradationPattern {
    /// Linear performance decline
    LinearDecline,
    /// Exponential performance decline
    ExponentialDecline,
    /// Stepwise performance decline
    StepwiseDecline,
    /// Cyclical degradation pattern
    CyclicalDegradation,
    /// Sudden performance dropoff
    SuddenDropoff,
    /// Progressive system instability
    ProgressiveInstability,
    /// Oscillating performance
    OscillatingPerformance,
    /// Plateau before decline
    PlateauBeforeDecline,
}

/// Progressive degradation analysis system
#[derive(Debug)]
pub struct ProgressiveDegradationAnalyzer {
    /// Configuration for degradation analysis
    config: ProgressiveDegradationConfig,
    /// Performance measurement history
    performance_history: Arc<Mutex<VecDeque<PerformanceMeasurement>>>,
    /// Threshold monitoring data
    threshold_monitoring: Arc<Mutex<HashMap<ThresholdType, ThresholdMonitoring>>>,
    /// Degradation analysis results
    degradation_analysis: Arc<Mutex<VecDeque<DegradationAnalysisResult>>>,
    /// Current system load multiplier
    current_load_multiplier: Arc<Mutex<f64>>,
    /// Analysis control flags
    analysis_active: Arc<AtomicBool>,
    prediction_active: Arc<AtomicBool>,
    /// Performance baseline
    performance_baseline: Arc<Mutex<Option<PerformanceBaseline>>>,
    /// Critical threshold breaches
    threshold_breaches: Arc<Mutex<Vec<ThresholdBreach>>>,
    /// Predictive models for thresholds
    prediction_models: Arc<Mutex<HashMap<ThresholdType, PredictionModel>>>,
    /// System stability metrics
    stability_metrics: Arc<Mutex<SystemStabilityMetrics>>,
}

/// Performance measurement sample
#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    /// Timestamp of measurement
    pub timestamp: Instant,
    /// Current load multiplier
    pub load_multiplier: f64,
    /// Latency in milliseconds
    pub latency_ms: f64,
    /// Throughput (operations per second)
    pub throughput_ops_per_sec: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// CPU utilization percentage
    pub cpu_utilization_percent: f64,
    /// Error rate percentage
    pub error_rate_percent: f64,
    /// Response time in milliseconds
    pub response_time_ms: f64,
    /// Queue depth
    pub queue_depth: usize,
    /// System stability score (0.0-1.0)
    pub stability_score: f64,
}

/// Threshold monitoring data
#[derive(Debug, Clone)]
pub struct ThresholdMonitoring {
    /// Threshold type
    pub threshold_type: ThresholdType,
    /// Current threshold values
    pub thresholds: ThresholdValues,
    /// Breach count
    pub breach_count: usize,
    /// Last breach timestamp
    pub last_breach: Option<Instant>,
    /// Breach history
    pub breach_history: VecDeque<ThresholdBreach>,
    /// Threshold adjustment history
    pub adjustment_history: VecDeque<ThresholdAdjustment>,
    /// Prediction confidence
    pub prediction_confidence: f64,
}

/// Threshold values for different severity levels
#[derive(Debug, Clone)]
pub struct ThresholdValues {
    /// Warning threshold
    pub warning: f64,
    /// Critical threshold
    pub critical: f64,
    /// Emergency threshold
    pub emergency: f64,
    /// Baseline value
    pub baseline: f64,
}

/// Threshold breach event
#[derive(Debug, Clone)]
pub struct ThresholdBreach {
    /// Threshold type that was breached
    pub threshold_type: ThresholdType,
    /// Breach severity
    pub severity: BreachSeverity,
    /// Timestamp of breach
    pub timestamp: Instant,
    /// Load multiplier at time of breach
    pub load_multiplier: f64,
    /// Actual value that breached threshold
    pub actual_value: f64,
    /// Threshold value that was breached
    pub threshold_value: f64,
    /// Breach duration
    pub duration: Option<Duration>,
    /// Recovery time
    pub recovery_time: Option<Duration>,
}

/// Severity levels for threshold breaches
#[derive(Debug, Clone, PartialEq)]
pub enum BreachSeverity {
    /// Warning level breach
    Warning,
    /// Critical level breach
    Critical,
    /// Emergency level breach
    Emergency,
    /// System failure level
    SystemFailure,
}

/// Degradation analysis result
#[derive(Debug, Clone)]
pub struct DegradationAnalysisResult {
    /// Analysis timestamp
    pub timestamp: Instant,
    /// Analysis window (start, end)
    pub analysis_window: (Instant, Instant),
    /// Detected degradation patterns
    pub detected_patterns: Vec<DetectedPattern>,
    /// Performance trend analysis
    pub performance_trend: PerformanceTrend,
    /// Threshold predictions
    pub threshold_predictions: Vec<ThresholdPrediction>,
    /// System health score (0.0-1.0)
    pub system_health_score: f64,
    /// Degradation velocity (rate of performance decline)
    pub degradation_velocity: f64,
    /// Stability assessment
    pub stability_assessment: StabilityAssessment,
}

/// Detected degradation pattern
#[derive(Debug, Clone)]
pub struct DetectedPattern {
    /// Pattern type
    pub pattern_type: DegradationPattern,
    /// Detection confidence (0.0-1.0)
    pub confidence: f64,
    /// Pattern start time
    pub start_time: Instant,
    /// Pattern characteristics
    pub characteristics: PatternCharacteristics,
    /// Affected metrics
    pub affected_metrics: Vec<ThresholdType>,
}

/// Pattern characteristics
#[derive(Debug, Clone)]
pub struct PatternCharacteristics {
    /// Rate of change
    pub rate_of_change: f64,
    /// Severity of degradation
    pub severity: f64,
    /// Pattern duration
    pub duration: Duration,
    /// Predictability score
    pub predictability: f64,
}

/// Performance trend analysis
#[derive(Debug, Clone)]
pub struct PerformanceTrend {
    /// Overall trend direction
    pub trend_direction: TrendDirection,
    /// Trend slope (rate of change)
    pub trend_slope: f64,
    /// Trend confidence
    pub trend_confidence: f64,
    /// Projected breaking point
    pub projected_breaking_point: Option<Instant>,
    /// Performance metrics trends
    pub metric_trends: HashMap<String, MetricTrend>,
}

/// Trend direction
#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    /// Performance improving
    Improving,
    /// Performance stable
    Stable,
    /// Performance degrading slowly
    SlowDegradation,
    /// Performance degrading rapidly
    RapidDegradation,
    /// Performance critically degrading
    CriticalDegradation,
}

/// Metric-specific trend
#[derive(Debug, Clone)]
pub struct MetricTrend {
    /// Metric name
    pub metric_name: String,
    /// Current value
    pub current_value: f64,
    /// Baseline value
    pub baseline_value: f64,
    /// Percentage change from baseline
    pub percent_change: f64,
    /// Trend slope
    pub slope: f64,
    /// Prediction for next interval
    pub next_prediction: f64,
}

/// Threshold prediction
#[derive(Debug, Clone)]
pub struct ThresholdPrediction {
    /// Threshold type
    pub threshold_type: ThresholdType,
    /// Predicted breach time
    pub predicted_breach_time: Option<Instant>,
    /// Prediction confidence
    pub confidence: f64,
    /// Predicted severity
    pub predicted_severity: BreachSeverity,
    /// Recommended actions
    pub recommended_actions: Vec<String>,
}

/// Prediction model for thresholds
#[derive(Debug, Clone)]
pub struct PredictionModel {
    /// Model type
    pub model_type: PredictionModelType,
    /// Model parameters
    pub parameters: HashMap<String, f64>,
    /// Model accuracy
    pub accuracy: f64,
    /// Training data points
    pub training_data_points: usize,
    /// Last update timestamp
    pub last_update: Instant,
}

/// Types of prediction models
#[derive(Debug, Clone, PartialEq)]
pub enum PredictionModelType {
    /// Linear regression
    LinearRegression,
    /// Exponential regression
    ExponentialRegression,
    /// Moving average
    MovingAverage,
    /// Weighted moving average
    WeightedMovingAverage,
    /// ARIMA model
    Arima,
    /// Neural network
    NeuralNetwork,
}

/// Performance baseline
#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    /// Baseline establishment timestamp
    pub timestamp: Instant,
    /// Baseline measurements
    pub measurements: PerformanceMeasurement,
    /// Baseline stability period
    pub stability_period: Duration,
    /// Baseline confidence
    pub confidence: f64,
    /// Environment conditions during baseline
    pub environment_conditions: EnvironmentConditions,
}

/// Environment conditions during measurement
#[derive(Debug, Clone)]
pub struct EnvironmentConditions {
    /// System load
    pub system_load: f64,
    /// Available memory
    pub available_memory_mb: f64,
    /// CPU temperature (if available)
    pub cpu_temperature: Option<f64>,
    /// Network conditions
    pub network_conditions: NetworkConditions,
}

/// Network conditions
#[derive(Debug, Clone)]
pub struct NetworkConditions {
    /// Bandwidth utilization
    pub bandwidth_utilization: f64,
    /// Latency
    pub latency_ms: f64,
    /// Packet loss rate
    pub packet_loss_rate: f64,
}

/// System stability metrics
#[derive(Debug, Clone)]
pub struct SystemStabilityMetrics {
    /// Overall stability score (0.0-1.0)
    pub overall_stability: f64,
    /// Stability variance
    pub stability_variance: f64,
    /// Stability trend
    pub stability_trend: TrendDirection,
    /// Instability events count
    pub instability_events: usize,
    /// Mean time between instability
    pub mtbi: Option<Duration>,
    /// Recovery time statistics
    pub recovery_stats: RecoveryStatistics,
}

/// Recovery statistics
#[derive(Debug, Clone)]
pub struct RecoveryStatistics {
    /// Average recovery time
    pub average_recovery_time: Duration,
    /// Maximum recovery time
    pub max_recovery_time: Duration,
    /// Successful recovery rate
    pub success_rate: f64,
    /// Recovery attempts
    pub total_recovery_attempts: usize,
}

/// Stability assessment
#[derive(Debug, Clone)]
pub struct StabilityAssessment {
    /// Current stability level
    pub stability_level: StabilityLevel,
    /// Stability confidence
    pub confidence: f64,
    /// Key instability factors
    pub instability_factors: Vec<String>,
    /// Stability prediction
    pub stability_prediction: StabilityPrediction,
}

/// System stability levels
#[derive(Debug, Clone, PartialEq)]
pub enum StabilityLevel {
    /// Highly stable system
    HighlyStable,
    /// Stable system
    Stable,
    /// Moderately stable
    ModeratelyStable,
    /// Unstable system
    Unstable,
    /// Highly unstable
    HighlyUnstable,
    /// Critical instability
    CriticallyUnstable,
}

/// Stability prediction
#[derive(Debug, Clone)]
pub struct StabilityPrediction {
    /// Predicted stability in next interval
    pub predicted_stability: StabilityLevel,
    /// Prediction confidence
    pub confidence: f64,
    /// Time to stability change
    pub time_to_change: Option<Duration>,
}

/// Threshold adjustment event
#[derive(Debug, Clone)]
pub struct ThresholdAdjustment {
    /// Adjustment timestamp
    pub timestamp: Instant,
    /// Previous threshold values
    pub previous_values: ThresholdValues,
    /// New threshold values
    pub new_values: ThresholdValues,
    /// Adjustment reason
    pub reason: String,
    /// Adjustment confidence
    pub confidence: f64,
}

impl ProgressiveDegradationAnalyzer {
    /// Create new progressive degradation analyzer
    pub fn new(config: ProgressiveDegradationConfig) -> Self {
        let mut threshold_monitoring = HashMap::new();
        
        // Initialize threshold monitoring for each type
        for threshold_type in &config.threshold_types {
            threshold_monitoring.insert(
                threshold_type.clone(),
                ThresholdMonitoring {
                    threshold_type: threshold_type.clone(),
                    thresholds: Self::default_threshold_values(threshold_type),
                    breach_count: 0,
                    last_breach: None,
                    breach_history: VecDeque::new(),
                    adjustment_history: VecDeque::new(),
                    prediction_confidence: 0.0,
                },
            );
        }

        Self {
            config,
            performance_history: Arc::new(Mutex::new(VecDeque::new())),
            threshold_monitoring: Arc::new(Mutex::new(threshold_monitoring)),
            degradation_analysis: Arc::new(Mutex::new(VecDeque::new())),
            current_load_multiplier: Arc::new(Mutex::new(1.0)),
            analysis_active: Arc::new(AtomicBool::new(false)),
            prediction_active: Arc::new(AtomicBool::new(false)),
            performance_baseline: Arc::new(Mutex::new(None)),
            threshold_breaches: Arc::new(Mutex::new(Vec::new())),
            prediction_models: Arc::new(Mutex::new(HashMap::new())),
            stability_metrics: Arc::new(Mutex::new(SystemStabilityMetrics {
                overall_stability: 1.0,
                stability_variance: 0.0,
                stability_trend: TrendDirection::Stable,
                instability_events: 0,
                mtbi: None,
                recovery_stats: RecoveryStatistics {
                    average_recovery_time: Duration::from_secs(0),
                    max_recovery_time: Duration::from_secs(0),
                    success_rate: 1.0,
                    total_recovery_attempts: 0,
                },
            })),
        }
    }

    /// Get default threshold values for a threshold type
    fn default_threshold_values(threshold_type: &ThresholdType) -> ThresholdValues {
        match threshold_type {
            ThresholdType::PerformanceLatency => ThresholdValues {
                baseline: 100.0,
                warning: 200.0,
                critical: 500.0,
                emergency: 1000.0,
            },
            ThresholdType::PerformanceThroughput => ThresholdValues {
                baseline: 1000.0,
                warning: 800.0,
                critical: 500.0,
                emergency: 200.0,
            },
            ThresholdType::MemoryUsage => ThresholdValues {
                baseline: 50.0,
                warning: 70.0,
                critical: 85.0,
                emergency: 95.0,
            },
            ThresholdType::CpuUtilization => ThresholdValues {
                baseline: 30.0,
                warning: 70.0,
                critical: 85.0,
                emergency: 95.0,
            },
            ThresholdType::ErrorRate => ThresholdValues {
                baseline: 0.1,
                warning: 1.0,
                critical: 5.0,
                emergency: 10.0,
            },
            ThresholdType::ResponseTime => ThresholdValues {
                baseline: 50.0,
                warning: 100.0,
                critical: 300.0,
                emergency: 1000.0,
            },
            ThresholdType::QueueDepth => ThresholdValues {
                baseline: 10.0,
                warning: 50.0,
                critical: 100.0,
                emergency: 500.0,
            },
            _ => ThresholdValues {
                baseline: 50.0,
                warning: 70.0,
                critical: 85.0,
                emergency: 95.0,
            },
        }
    }

    /// Start progressive degradation analysis
    pub fn start_analysis(&self) -> Result<()> {
        self.analysis_active.store(true, Ordering::SeqCst);
        
        // Start baseline establishment
        self.establish_baseline()?;
        
        // Start monitoring thread
        self.start_monitoring_thread()?;
        
        // Start load progression thread
        self.start_load_progression_thread()?;
        
        // Start analysis thread
        self.start_analysis_thread()?;
        
        if self.config.enable_predictive_analysis {
            self.start_prediction_thread()?;
        }
        
        Ok(())
    }

    /// Establish performance baseline
    fn establish_baseline(&self) -> Result<()> {
        println!("🎯 Establishing performance baseline...");
        
        let baseline_start = Instant::now();
        let baseline_duration = Duration::from_secs(30); // 30 seconds baseline
        
        let mut measurements = Vec::new();
        
        while baseline_start.elapsed() < baseline_duration {
            let measurement = self.take_performance_measurement(1.0)?;
            measurements.push(measurement);
            thread::sleep(Duration::from_millis(1000)); // 1 second intervals
        }
        
        // Calculate baseline from measurements
        if let Some(baseline_measurement) = self.calculate_baseline_from_measurements(&measurements) {
            let baseline = PerformanceBaseline {
                timestamp: baseline_start,
                measurements: baseline_measurement,
                stability_period: baseline_duration,
                confidence: 0.95, // High confidence for controlled baseline
                environment_conditions: self.capture_environment_conditions()?,
            };
            
            *self.performance_baseline.lock().unwrap() = Some(baseline);
            println!("✅ Performance baseline established successfully");
        }
        
        Ok(())
    }

    /// Calculate baseline measurement from multiple samples
    fn calculate_baseline_from_measurements(&self, measurements: &[PerformanceMeasurement]) -> Option<PerformanceMeasurement> {
        if measurements.is_empty() {
            return None;
        }
        
        let count = measurements.len() as f64;
        
        Some(PerformanceMeasurement {
            timestamp: measurements[0].timestamp,
            load_multiplier: 1.0,
            latency_ms: measurements.iter().map(|m| m.latency_ms).sum::<f64>() / count,
            throughput_ops_per_sec: measurements.iter().map(|m| m.throughput_ops_per_sec).sum::<f64>() / count,
            memory_usage_mb: measurements.iter().map(|m| m.memory_usage_mb).sum::<f64>() / count,
            cpu_utilization_percent: measurements.iter().map(|m| m.cpu_utilization_percent).sum::<f64>() / count,
            error_rate_percent: measurements.iter().map(|m| m.error_rate_percent).sum::<f64>() / count,
            response_time_ms: measurements.iter().map(|m| m.response_time_ms).sum::<f64>() / count,
            queue_depth: (measurements.iter().map(|m| m.queue_depth).sum::<usize>() as f64 / count) as usize,
            stability_score: measurements.iter().map(|m| m.stability_score).sum::<f64>() / count,
        })
    }

    /// Capture current environment conditions
    fn capture_environment_conditions(&self) -> Result<EnvironmentConditions> {
        Ok(EnvironmentConditions {
            system_load: self.measure_system_load()?,
            available_memory_mb: self.measure_available_memory()?,
            cpu_temperature: None, // Would require system-specific implementation
            network_conditions: NetworkConditions {
                bandwidth_utilization: 0.1, // 10% baseline
                latency_ms: 5.0,
                packet_loss_rate: 0.0,
            },
        })
    }

    /// Start monitoring thread
    fn start_monitoring_thread(&self) -> Result<()> {
        let analysis_active = Arc::clone(&self.analysis_active);
        let performance_history = Arc::clone(&self.performance_history);
        let current_load_multiplier = Arc::clone(&self.current_load_multiplier);
        let threshold_monitoring = Arc::clone(&self.threshold_monitoring);
        let threshold_breaches = Arc::clone(&self.threshold_breaches);
        let config = self.config.clone();
        
        thread::spawn(move || {
            println!("📊 Starting performance monitoring thread...");
            
            while analysis_active.load(Ordering::SeqCst) {
                let load_multiplier = *current_load_multiplier.lock().unwrap();
                
                if let Ok(measurement) = Self::take_performance_measurement_static(load_multiplier) {
                    // Store measurement
                    {
                        let mut history = performance_history.lock().unwrap();
                        history.push_back(measurement.clone());
                        
                        // Keep history within limits
                        if history.len() > config.sample_history_size {
                            history.pop_front();
                        }
                    }
                    
                    // Check thresholds
                    Self::check_thresholds_static(&measurement, &threshold_monitoring, &threshold_breaches, &config);
                }
                
                thread::sleep(Duration::from_millis(config.analysis_interval_ms));
            }
            
            println!("🔚 Performance monitoring thread stopped");
        });
        
        Ok(())
    }

    /// Start load progression thread
    fn start_load_progression_thread(&self) -> Result<()> {
        let analysis_active = Arc::clone(&self.analysis_active);
        let current_load_multiplier = Arc::clone(&self.current_load_multiplier);
        let config = self.config.clone();
        
        thread::spawn(move || {
            println!("📈 Starting load progression thread...");
            
            let mut load_multiplier = 1.0;
            
            while analysis_active.load(Ordering::SeqCst) && load_multiplier <= config.max_load_multiplier {
                thread::sleep(Duration::from_secs(config.load_increase_interval_secs));
                
                if analysis_active.load(Ordering::SeqCst) {
                    load_multiplier += load_multiplier * (config.load_increment_step_percent / 100.0);
                    *current_load_multiplier.lock().unwrap() = load_multiplier;
                    
                    println!("🔄 Load increased to {:.2}x baseline", load_multiplier);
                }
            }
            
            println!("🏁 Load progression completed at {:.2}x baseline", load_multiplier);
        });
        
        Ok(())
    }

    /// Start analysis thread
    fn start_analysis_thread(&self) -> Result<()> {
        let analysis_active = Arc::clone(&self.analysis_active);
        let performance_history = Arc::clone(&self.performance_history);
        let degradation_analysis = Arc::clone(&self.degradation_analysis);
        let threshold_monitoring = Arc::clone(&self.threshold_monitoring);
        let stability_metrics = Arc::clone(&self.stability_metrics);
        let config = self.config.clone();
        
        thread::spawn(move || {
            println!("🔍 Starting degradation analysis thread...");
            
            let analysis_interval = Duration::from_secs(10); // Analyze every 10 seconds
            
            while analysis_active.load(Ordering::SeqCst) {
                thread::sleep(analysis_interval);
                
                if let Ok(analysis_result) = Self::perform_degradation_analysis_static(
                    &performance_history,
                    &threshold_monitoring,
                    &stability_metrics,
                    &config,
                ) {
                    let mut analysis = degradation_analysis.lock().unwrap();
                    analysis.push_back(analysis_result);
                    
                    // Keep analysis history within limits
                    if analysis.len() > 100 {
                        analysis.pop_front();
                    }
                }
            }
            
            println!("🔚 Degradation analysis thread stopped");
        });
        
        Ok(())
    }

    /// Start prediction thread
    fn start_prediction_thread(&self) -> Result<()> {
        let prediction_active = Arc::clone(&self.prediction_active);
        let performance_history = Arc::clone(&self.performance_history);
        let threshold_monitoring = Arc::clone(&self.threshold_monitoring);
        let prediction_models = Arc::clone(&self.prediction_models);
        let config = self.config.clone();
        
        self.prediction_active.store(true, Ordering::SeqCst);
        
        thread::spawn(move || {
            println!("🔮 Starting prediction thread...");
            
            while prediction_active.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_secs(30)); // Update predictions every 30 seconds
                
                Self::update_prediction_models_static(&performance_history, &threshold_monitoring, &prediction_models, &config);
            }
            
            println!("🔚 Prediction thread stopped");
        });
        
        Ok(())
    }

    /// Take performance measurement
    fn take_performance_measurement(&self, load_multiplier: f64) -> Result<PerformanceMeasurement> {
        Self::take_performance_measurement_static(load_multiplier)
    }

    /// Static version of take_performance_measurement for use in threads
    fn take_performance_measurement_static(load_multiplier: f64) -> Result<PerformanceMeasurement> {
        // Simulate workload based on load multiplier
        let workload_duration = Duration::from_millis((50.0 * load_multiplier) as u64);
        let start_time = Instant::now();
        
        // Simulate CPU-intensive work
        let mut sum = 0u64;
        for i in 0..(1000 * load_multiplier as u64) {
            sum = sum.wrapping_add(i * 17);
        }
        
        let latency = start_time.elapsed().as_millis() as f64;
        
        // Calculate simulated metrics based on load
        let base_throughput = 1000.0;
        let throughput = base_throughput / (1.0 + (load_multiplier - 1.0) * 0.3);
        
        let base_memory = 100.0;
        let memory_usage = base_memory * (1.0 + (load_multiplier - 1.0) * 0.2);
        
        let base_cpu = 20.0;
        let cpu_utilization = base_cpu * load_multiplier.min(4.0);
        
        let error_rate = if load_multiplier > 3.0 {
            (load_multiplier - 3.0) * 2.0
        } else {
            0.1
        };
        
        let stability_score = (2.0 - load_multiplier.min(2.0)) / 2.0;
        
        Ok(PerformanceMeasurement {
            timestamp: Instant::now(),
            load_multiplier,
            latency_ms: latency,
            throughput_ops_per_sec: throughput,
            memory_usage_mb: memory_usage,
            cpu_utilization_percent: cpu_utilization,
            error_rate_percent: error_rate,
            response_time_ms: latency * 1.2, // Response time slightly higher than latency
            queue_depth: (load_multiplier * 10.0) as usize,
            stability_score,
        })
    }

    /// Check thresholds against current measurement
    fn check_thresholds_static(
        measurement: &PerformanceMeasurement,
        threshold_monitoring: &Arc<Mutex<HashMap<ThresholdType, ThresholdMonitoring>>>,
        threshold_breaches: &Arc<Mutex<Vec<ThresholdBreach>>>,
        config: &ProgressiveDegradationConfig,
    ) {
        let mut monitoring = threshold_monitoring.lock().unwrap();
        let mut breaches = threshold_breaches.lock().unwrap();
        
        for threshold_type in &config.threshold_types {
            if let Some(monitor) = monitoring.get_mut(threshold_type) {
                let current_value = Self::get_metric_value(measurement, threshold_type);
                let breach_info = Self::check_threshold_breach(current_value, &monitor.thresholds, threshold_type);
                
                if let Some((severity, threshold_value)) = breach_info {
                    let breach = ThresholdBreach {
                        threshold_type: threshold_type.clone(),
                        severity,
                        timestamp: measurement.timestamp,
                        load_multiplier: measurement.load_multiplier,
                        actual_value: current_value,
                        threshold_value,
                        duration: None,
                        recovery_time: None,
                    };
                    
                    monitor.breach_count += 1;
                    monitor.last_breach = Some(measurement.timestamp);
                    monitor.breach_history.push_back(breach.clone());
                    breaches.push(breach);
                    
                    // Keep breach history within limits
                    if monitor.breach_history.len() > 100 {
                        monitor.breach_history.pop_front();
                    }
                }
            }
        }
    }

    /// Get metric value for threshold type
    fn get_metric_value(measurement: &PerformanceMeasurement, threshold_type: &ThresholdType) -> f64 {
        match threshold_type {
            ThresholdType::PerformanceLatency => measurement.latency_ms,
            ThresholdType::PerformanceThroughput => measurement.throughput_ops_per_sec,
            ThresholdType::MemoryUsage => measurement.memory_usage_mb,
            ThresholdType::CpuUtilization => measurement.cpu_utilization_percent,
            ThresholdType::ErrorRate => measurement.error_rate_percent,
            ThresholdType::ResponseTime => measurement.response_time_ms,
            ThresholdType::QueueDepth => measurement.queue_depth as f64,
            _ => measurement.stability_score,
        }
    }

    /// Check if a threshold is breached
    fn check_threshold_breach(
        value: f64,
        thresholds: &ThresholdValues,
        threshold_type: &ThresholdType,
    ) -> Option<(BreachSeverity, f64)> {
        // For metrics where lower is better (latency, error rate, etc.)
        let lower_is_better = matches!(threshold_type, 
            ThresholdType::PerformanceLatency | 
            ThresholdType::ErrorRate | 
            ThresholdType::ResponseTime
        );
        
        if lower_is_better {
            if value >= thresholds.emergency {
                Some((BreachSeverity::Emergency, thresholds.emergency))
            } else if value >= thresholds.critical {
                Some((BreachSeverity::Critical, thresholds.critical))
            } else if value >= thresholds.warning {
                Some((BreachSeverity::Warning, thresholds.warning))
            } else {
                None
            }
        } else {
            // For metrics where higher is better (throughput, etc.)
            if value <= thresholds.emergency {
                Some((BreachSeverity::Emergency, thresholds.emergency))
            } else if value <= thresholds.critical {
                Some((BreachSeverity::Critical, thresholds.critical))
            } else if value <= thresholds.warning {
                Some((BreachSeverity::Warning, thresholds.warning))
            } else {
                None
            }
        }
    }

    /// Perform degradation analysis
    fn perform_degradation_analysis_static(
        performance_history: &Arc<Mutex<VecDeque<PerformanceMeasurement>>>,
        threshold_monitoring: &Arc<Mutex<HashMap<ThresholdType, ThresholdMonitoring>>>,
        stability_metrics: &Arc<Mutex<SystemStabilityMetrics>>,
        config: &ProgressiveDegradationConfig,
    ) -> Result<DegradationAnalysisResult> {
        let history = performance_history.lock().unwrap();
        let monitoring = threshold_monitoring.lock().unwrap();
        
        if history.len() < 10 {
            return Err("Insufficient data for analysis".into());
        }
        
        let analysis_start = history.iter().nth(history.len() - 50).map(|m| m.timestamp)
            .unwrap_or_else(|| history.front().unwrap().timestamp);
        let analysis_end = history.back().unwrap().timestamp;
        
        // Detect patterns
        let detected_patterns = Self::detect_degradation_patterns(&history, config);
        
        // Analyze performance trend
        let performance_trend = Self::analyze_performance_trend(&history);
        
        // Generate threshold predictions
        let threshold_predictions = Self::generate_threshold_predictions(&history, &monitoring, config);
        
        // Calculate system health score
        let system_health_score = Self::calculate_system_health_score(&history, &monitoring);
        
        // Calculate degradation velocity
        let degradation_velocity = Self::calculate_degradation_velocity(&history);
        
        // Assess stability
        let stability_assessment = Self::assess_system_stability(&history, &stability_metrics);
        
        Ok(DegradationAnalysisResult {
            timestamp: Instant::now(),
            analysis_window: (analysis_start, analysis_end),
            detected_patterns,
            performance_trend,
            threshold_predictions,
            system_health_score,
            degradation_velocity,
            stability_assessment,
        })
    }

    /// Detect degradation patterns in performance data
    fn detect_degradation_patterns(
        history: &VecDeque<PerformanceMeasurement>,
        config: &ProgressiveDegradationConfig,
    ) -> Vec<DetectedPattern> {
        let mut patterns = Vec::new();
        
        if history.len() < 20 {
            return patterns;
        }
        
        // Check for linear decline
        if let Some(pattern) = Self::detect_linear_decline(history) {
            patterns.push(pattern);
        }
        
        // Check for exponential decline
        if let Some(pattern) = Self::detect_exponential_decline(history) {
        patterns.push(pattern);
        }
        
        // Check for sudden dropoff
        if let Some(pattern) = Self::detect_sudden_dropoff(history) {
            patterns.push(pattern);
        }
        
        patterns
    }

    /// Detect linear decline pattern
    fn detect_linear_decline(history: &VecDeque<PerformanceMeasurement>) -> Option<DetectedPattern> {
        let recent_samples = 20;
        let samples: Vec<_> = history.iter().rev().take(recent_samples).collect();
        
        if samples.len() < recent_samples {
            return None;
        }
        
        // Calculate linear regression for throughput
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        let n = samples.len() as f64;
        
        for (i, sample) in samples.iter().enumerate() {
            let x = i as f64;
            let y = sample.throughput_ops_per_sec;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        
        // Check if slope indicates significant decline
        if slope < -10.0 { // Significant negative slope
            let start_time = samples.last().unwrap().timestamp;
            let duration = samples[0].timestamp.duration_since(start_time);
            
            Some(DetectedPattern {
                pattern_type: DegradationPattern::LinearDecline,
                confidence: 0.8,
                start_time,
                characteristics: PatternCharacteristics {
                    rate_of_change: slope,
                    severity: (-slope / 100.0).min(1.0),
                    duration,
                    predictability: 0.9,
                },
                affected_metrics: vec![ThresholdType::PerformanceThroughput],
            })
        } else {
            None
        }
    }

    /// Detect exponential decline pattern
    fn detect_exponential_decline(history: &VecDeque<PerformanceMeasurement>) -> Option<DetectedPattern> {
        let recent_samples = 15;
        let samples: Vec<_> = history.iter().rev().take(recent_samples).collect();
        
        if samples.len() < recent_samples {
            return None;
        }
        
        // Check for exponential decay in performance metrics
        let mut decline_count = 0;
        let mut max_decline_rate = 0.0;
        
        for window in samples.windows(3) {
            let rate1 = window[1].throughput_ops_per_sec / window[2].throughput_ops_per_sec;
            let rate2 = window[0].throughput_ops_per_sec / window[1].throughput_ops_per_sec;
            
            if rate1 < 0.95 && rate2 < rate1 { // Accelerating decline
                decline_count += 1;
                max_decline_rate = max_decline_rate.max(1.0 - rate2);
            }
        }
        
        if decline_count > 5 && max_decline_rate > 0.1 {
            let start_time = samples.last().unwrap().timestamp;
            let duration = samples[0].timestamp.duration_since(start_time);
            
            Some(DetectedPattern {
                pattern_type: DegradationPattern::ExponentialDecline,
                confidence: 0.85,
                start_time,
                characteristics: PatternCharacteristics {
                    rate_of_change: max_decline_rate,
                    severity: max_decline_rate,
                    duration,
                    predictability: 0.7,
                },
                affected_metrics: vec![ThresholdType::PerformanceThroughput],
            })
        } else {
            None
        }
    }

    /// Detect sudden dropoff pattern
    fn detect_sudden_dropoff(history: &VecDeque<PerformanceMeasurement>) -> Option<DetectedPattern> {
        let recent_samples = 10;
        let samples: Vec<_> = history.iter().rev().take(recent_samples).collect();
        
        if samples.len() < recent_samples {
            return None;
        }
        
        // Look for sudden large decrease in performance
        for window in samples.windows(2) {
            let current = &window[0];
            let previous = &window[1];
            
            let throughput_drop = (previous.throughput_ops_per_sec - current.throughput_ops_per_sec) / previous.throughput_ops_per_sec;
            let latency_increase = (current.latency_ms - previous.latency_ms) / previous.latency_ms;
            
            if throughput_drop > 0.3 || latency_increase > 0.5 { // 30% throughput drop or 50% latency increase
                Some(DetectedPattern {
                    pattern_type: DegradationPattern::SuddenDropoff,
                    confidence: 0.9,
                    start_time: current.timestamp,
                    characteristics: PatternCharacteristics {
                        rate_of_change: throughput_drop.max(latency_increase),
                        severity: throughput_drop.max(latency_increase),
                        duration: Duration::from_secs(1),
                        predictability: 0.3, // Sudden events are hard to predict
                    },
                    affected_metrics: vec![
                        ThresholdType::PerformanceThroughput,
                        ThresholdType::PerformanceLatency,
                    ],
                })
            } else {
                continue;
            }
        }
        
        None
    }

    /// Analyze overall performance trend
    fn analyze_performance_trend(history: &VecDeque<PerformanceMeasurement>) -> PerformanceTrend {
        if history.len() < 10 {
            return PerformanceTrend {
                trend_direction: TrendDirection::Stable,
                trend_slope: 0.0,
                trend_confidence: 0.0,
                projected_breaking_point: None,
                metric_trends: HashMap::new(),
            };
        }
        
        let samples: Vec<_> = history.iter().collect();
        let n = samples.len();
        
        // Calculate trend for multiple metrics
        let mut metric_trends = HashMap::new();
        
        // Throughput trend
        let throughput_slope = Self::calculate_metric_slope(&samples, |m| m.throughput_ops_per_sec);
        metric_trends.insert("throughput".to_string(), MetricTrend {
            metric_name: "throughput".to_string(),
            current_value: samples[n-1].throughput_ops_per_sec,
            baseline_value: samples[0].throughput_ops_per_sec,
            percent_change: ((samples[n-1].throughput_ops_per_sec - samples[0].throughput_ops_per_sec) / samples[0].throughput_ops_per_sec) * 100.0,
            slope: throughput_slope,
            next_prediction: samples[n-1].throughput_ops_per_sec + throughput_slope,
        });
        
        // Latency trend
        let latency_slope = Self::calculate_metric_slope(&samples, |m| m.latency_ms);
        metric_trends.insert("latency".to_string(), MetricTrend {
            metric_name: "latency".to_string(),
            current_value: samples[n-1].latency_ms,
            baseline_value: samples[0].latency_ms,
            percent_change: ((samples[n-1].latency_ms - samples[0].latency_ms) / samples[0].latency_ms) * 100.0,
            slope: latency_slope,
            next_prediction: samples[n-1].latency_ms + latency_slope,
        });
        
        // Determine overall trend direction
        let trend_direction = if throughput_slope < -20.0 || latency_slope > 50.0 {
            TrendDirection::CriticalDegradation
        } else if throughput_slope < -10.0 || latency_slope > 20.0 {
            TrendDirection::RapidDegradation
        } else if throughput_slope < -5.0 || latency_slope > 10.0 {
            TrendDirection::SlowDegradation
        } else if throughput_slope.abs() < 2.0 && latency_slope.abs() < 5.0 {
            TrendDirection::Stable
        } else {
            TrendDirection::Improving
        };
        
        let overall_slope = (throughput_slope.abs() + latency_slope.abs()) / 2.0;
        
        PerformanceTrend {
            trend_direction,
            trend_slope: overall_slope,
            trend_confidence: 0.8, // Static confidence for now
            projected_breaking_point: Self::calculate_breaking_point(&samples, throughput_slope, latency_slope),
            metric_trends,
        }
    }

    /// Calculate slope for a specific metric
    fn calculate_metric_slope<F>(samples: &[&PerformanceMeasurement], extractor: F) -> f64
    where
        F: Fn(&PerformanceMeasurement) -> f64,
    {
        if samples.len() < 2 {
            return 0.0;
        }
        
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        let n = samples.len() as f64;
        
        for (i, sample) in samples.iter().enumerate() {
            let x = i as f64;
            let y = extractor(sample);
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }
        
        if n * sum_x2 - sum_x * sum_x == 0.0 {
            return 0.0;
        }
        
        (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
    }

    /// Calculate projected breaking point
    fn calculate_breaking_point(
        samples: &[&PerformanceMeasurement],
        throughput_slope: f64,
        latency_slope: f64,
    ) -> Option<Instant> {
        if samples.is_empty() {
            return None;
        }
        
        let last_sample = samples.last().unwrap();
        
        // Calculate when throughput might hit zero (if declining)
        let throughput_breaking_point = if throughput_slope < -1.0 {
            let time_to_zero = last_sample.throughput_ops_per_sec / (-throughput_slope);
            Some(last_sample.timestamp + Duration::from_secs(time_to_zero as u64))
        } else {
            None
        };
        
        // Calculate when latency might hit critical threshold (1000ms)
        let latency_breaking_point = if latency_slope > 1.0 {
            let time_to_critical = (1000.0 - last_sample.latency_ms) / latency_slope;
            if time_to_critical > 0.0 {
                Some(last_sample.timestamp + Duration::from_secs(time_to_critical as u64))
            } else {
                None
            }
        } else {
            None
        };
        
        // Return the earliest breaking point
        match (throughput_breaking_point, latency_breaking_point) {
            (Some(tp), Some(lp)) => Some(tp.min(lp)),
            (Some(tp), None) => Some(tp),
            (None, Some(lp)) => Some(lp),
            (None, None) => None,
        }
    }

    /// Generate threshold predictions
    fn generate_threshold_predictions(
        history: &VecDeque<PerformanceMeasurement>,
        monitoring: &HashMap<ThresholdType, ThresholdMonitoring>,
        config: &ProgressiveDegradationConfig,
    ) -> Vec<ThresholdPrediction> {
        let mut predictions = Vec::new();
        
        for threshold_type in &config.threshold_types {
            if let Some(monitor) = monitoring.get(threshold_type) {
                if let Some(prediction) = Self::predict_threshold_breach(history, threshold_type, monitor) {
                    predictions.push(prediction);
                }
            }
        }
        
        predictions
    }

    /// Predict threshold breach for specific threshold type
    fn predict_threshold_breach(
        history: &VecDeque<PerformanceMeasurement>,
        threshold_type: &ThresholdType,
        monitor: &ThresholdMonitoring,
    ) -> Option<ThresholdPrediction> {
        if history.len() < 10 {
            return None;
        }
        
        let recent_samples: Vec<_> = history.iter().rev().take(20).collect();
        let slope = Self::calculate_metric_slope(&recent_samples, |m| Self::get_metric_value(m, threshold_type));
        
        if slope.abs() < 0.1 {
            return None; // Not enough change to predict
        }
        
        let current_value = Self::get_metric_value(recent_samples[0], threshold_type);
        let critical_threshold = monitor.thresholds.critical;
        
        // Calculate time to breach critical threshold
        let time_to_breach = if slope != 0.0 {
            let distance_to_threshold = match threshold_type {
                ThresholdType::PerformanceThroughput => current_value - critical_threshold,
                _ => critical_threshold - current_value,
            };
            
            if distance_to_threshold > 0.0 {
                distance_to_threshold / slope.abs()
            } else {
                0.0 // Already breached
            }
        } else {
            return None;
        };
        
        if time_to_breach > 0.0 && time_to_breach < 300.0 { // Within 5 minutes
            let predicted_breach_time = recent_samples[0].timestamp + Duration::from_secs(time_to_breach as u64);
            
            Some(ThresholdPrediction {
                threshold_type: threshold_type.clone(),
                predicted_breach_time: Some(predicted_breach_time),
                confidence: (1.0 - (time_to_breach / 300.0)).max(0.6),
                predicted_severity: BreachSeverity::Critical,
                recommended_actions: vec![
                    "Consider reducing system load".to_string(),
                    "Monitor resource utilization".to_string(),
                    "Prepare scaling resources".to_string(),
                ],
            })
        } else {
            None
        }
    }

    /// Calculate system health score
    fn calculate_system_health_score(
        history: &VecDeque<PerformanceMeasurement>,
        monitoring: &HashMap<ThresholdType, ThresholdMonitoring>,
    ) -> f64 {
        if history.is_empty() {
            return 0.0;
        }
        
        let recent_samples: Vec<_> = history.iter().rev().take(10).collect();
        let mut health_scores = Vec::new();
        
        // Calculate health based on different metrics
        for sample in &recent_samples {
            let mut sample_health = 1.0;
            
            // Stability score contributes directly
            sample_health *= sample.stability_score;
            
            // Error rate impact
            sample_health *= (1.0 - (sample.error_rate_percent / 100.0).min(0.5));
            
            // Performance degradation impact
            if sample.load_multiplier > 1.0 {
                let expected_throughput = 1000.0 / sample.load_multiplier;
                let performance_ratio = sample.throughput_ops_per_sec / expected_throughput;
                sample_health *= performance_ratio.min(1.0);
            }
            
            health_scores.push(sample_health);
        }
        
        // Average health score
        health_scores.iter().sum::<f64>() / health_scores.len() as f64
    }

    /// Calculate degradation velocity
    fn calculate_degradation_velocity(history: &VecDeque<PerformanceMeasurement>) -> f64 {
        if history.len() < 5 {
            return 0.0;
        }
        
        let recent_samples: Vec<_> = history.iter().rev().take(10).collect();
        
        // Calculate rate of change in multiple metrics
        let throughput_velocity = Self::calculate_metric_slope(&recent_samples, |m| m.throughput_ops_per_sec);
        let latency_velocity = Self::calculate_metric_slope(&recent_samples, |m| m.latency_ms);
        let stability_velocity = Self::calculate_metric_slope(&recent_samples, |m| m.stability_score);
        
        // Combine velocities (higher absolute value = faster degradation)
        (throughput_velocity.abs() + latency_velocity.abs() + stability_velocity.abs()) / 3.0
    }

    /// Assess system stability
    fn assess_system_stability(
        history: &VecDeque<PerformanceMeasurement>,
        stability_metrics: &Arc<Mutex<SystemStabilityMetrics>>,
    ) -> StabilityAssessment {
        if history.is_empty() {
            return StabilityAssessment {
                stability_level: StabilityLevel::Stable,
                confidence: 0.0,
                instability_factors: vec![],
                stability_prediction: StabilityPrediction {
                    predicted_stability: StabilityLevel::Stable,
                    confidence: 0.0,
                    time_to_change: None,
                },
            };
        }
        
        let recent_samples: Vec<_> = history.iter().rev().take(20).collect();
        let mut stability_scores: Vec<f64> = recent_samples.iter().map(|s| s.stability_score).collect();
        
        // Calculate stability statistics
        let mean_stability = stability_scores.iter().sum::<f64>() / stability_scores.len() as f64;
        let stability_variance = {
            let variance_sum: f64 = stability_scores.iter()
                .map(|score| (score - mean_stability).powi(2))
                .sum();
            variance_sum / stability_scores.len() as f64
        };
        
        // Determine stability level
        let stability_level = if mean_stability > 0.9 && stability_variance < 0.01 {
            StabilityLevel::HighlyStable
        } else if mean_stability > 0.8 && stability_variance < 0.02 {
            StabilityLevel::Stable
        } else if mean_stability > 0.6 && stability_variance < 0.05 {
            StabilityLevel::ModeratelyStable
        } else if mean_stability > 0.4 {
            StabilityLevel::Unstable
        } else if mean_stability > 0.2 {
            StabilityLevel::HighlyUnstable
        } else {
            StabilityLevel::CriticallyUnstable
        };
        
        // Identify instability factors
        let mut instability_factors = Vec::new();
        if stability_variance > 0.05 {
            instability_factors.push("High stability variance".to_string());
        }
        if recent_samples.iter().any(|s| s.error_rate_percent > 5.0) {
            instability_factors.push("High error rates".to_string());
        }
        if recent_samples.iter().any(|s| s.cpu_utilization_percent > 90.0) {
            instability_factors.push("CPU resource exhaustion".to_string());
        }
        
        // Update stability metrics
        {
            let mut metrics = stability_metrics.lock().unwrap();
            metrics.overall_stability = mean_stability;
            metrics.stability_variance = stability_variance;
            
            if mean_stability < 0.5 {
                metrics.instability_events += 1;
            }
        }
        
        StabilityAssessment {
            stability_level: stability_level.clone(),
            confidence: 0.85,
            instability_factors,
            stability_prediction: StabilityPrediction {
                predicted_stability: stability_level,
                confidence: 0.75,
                time_to_change: None,
            },
        }
    }

    /// Update prediction models
    fn update_prediction_models_static(
        performance_history: &Arc<Mutex<VecDeque<PerformanceMeasurement>>>,
        threshold_monitoring: &Arc<Mutex<HashMap<ThresholdType, ThresholdMonitoring>>>,
        prediction_models: &Arc<Mutex<HashMap<ThresholdType, PredictionModel>>>,
        config: &ProgressiveDegradationConfig,
    ) {
        let history = performance_history.lock().unwrap();
        let mut models = prediction_models.lock().unwrap();
        
        if history.len() < 20 {
            return;
        }
        
        for threshold_type in &config.threshold_types {
            let model = models.entry(threshold_type.clone()).or_insert_with(|| {
                PredictionModel {
                    model_type: PredictionModelType::LinearRegression,
                    parameters: HashMap::new(),
                    accuracy: 0.0,
                    training_data_points: 0,
                    last_update: Instant::now(),
                }
            });
            
            // Update model with recent data
            Self::train_prediction_model(&history, threshold_type, model);
        }
    }

    /// Train prediction model
    fn train_prediction_model(
        history: &VecDeque<PerformanceMeasurement>,
        threshold_type: &ThresholdType,
        model: &mut PredictionModel,
    ) {
        let samples: Vec<_> = history.iter().rev().take(50).collect();
        
        if samples.len() < 10 {
            return;
        }
        
        // Simple linear regression training
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        let n = samples.len() as f64;
        
        for (i, sample) in samples.iter().enumerate() {
            let x = i as f64;
            let y = Self::get_metric_value(sample, threshold_type);
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }
        
        if n * sum_x2 - sum_x * sum_x != 0.0 {
            let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
            let intercept = (sum_y - slope * sum_x) / n;
            
            model.parameters.insert("slope".to_string(), slope);
            model.parameters.insert("intercept".to_string(), intercept);
            model.training_data_points = samples.len();
            model.last_update = Instant::now();
            
            // Calculate model accuracy (simplified R-squared)
            let mean_y = sum_y / n;
            let mut ss_tot = 0.0;
            let mut ss_res = 0.0;
            
            for (i, sample) in samples.iter().enumerate() {
                let x = i as f64;
                let y = Self::get_metric_value(sample, threshold_type);
                let y_pred = slope * x + intercept;
                
                ss_tot += (y - mean_y).powi(2);
                ss_res += (y - y_pred).powi(2);
            }
            
            model.accuracy = if ss_tot > 0.0 {
                1.0 - (ss_res / ss_tot)
            } else {
                0.0
            };
        }
    }

    /// Measure system load
    fn measure_system_load(&self) -> Result<f64> {
        // Simplified system load measurement
        Ok(0.3) // 30% baseline load
    }

    /// Measure available memory
    fn measure_available_memory(&self) -> Result<f64> {
        // Simplified memory measurement
        Ok(8192.0) // 8GB baseline
    }

    /// Stop analysis
    pub fn stop_analysis(&self) {
        self.analysis_active.store(false, Ordering::SeqCst);
        self.prediction_active.store(false, Ordering::SeqCst);
        println!("🛑 Progressive degradation analysis stopped");
    }

    /// Get analysis results
    pub fn get_analysis_results(&self) -> Vec<DegradationAnalysisResult> {
        self.degradation_analysis.lock().unwrap().iter().cloned().collect()
    }

    /// Get threshold breaches
    pub fn get_threshold_breaches(&self) -> Vec<ThresholdBreach> {
        self.threshold_breaches.lock().unwrap().clone()
    }

    /// Generate analysis report
    pub fn generate_analysis_report(&self) -> AnalysisReport {
        let analysis_results = self.get_analysis_results();
        let threshold_breaches = self.get_threshold_breaches();
        let performance_history = self.performance_history.lock().unwrap().clone();
        let stability_metrics = self.stability_metrics.lock().unwrap().clone();
        
        AnalysisReport {
            test_duration: self.config.max_test_duration_secs,
            total_measurements: performance_history.len(),
            total_threshold_breaches: threshold_breaches.len(),
            analysis_results_count: analysis_results.len(),
            final_system_health: analysis_results.last().map(|r| r.system_health_score).unwrap_or(0.0),
            critical_breaches: threshold_breaches.iter().filter(|b| b.severity == BreachSeverity::Critical).count(),
            emergency_breaches: threshold_breaches.iter().filter(|b| b.severity == BreachSeverity::Emergency).count(),
            stability_assessment: stability_metrics,
            recommendations: self.generate_recommendations(&analysis_results, &threshold_breaches),
        }
    }

    /// Generate recommendations based on analysis
    fn generate_recommendations(&self, analysis_results: &[DegradationAnalysisResult], threshold_breaches: &[ThresholdBreach]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Check for critical patterns
        for result in analysis_results {
            for pattern in &result.detected_patterns {
                match pattern.pattern_type {
                    DegradationPattern::ExponentialDecline => {
                        recommendations.push("🚨 CRITICAL: Exponential performance decline detected - immediate intervention required".to_string());
                    },
                    DegradationPattern::SuddenDropoff => {
                        recommendations.push("⚠️ Sudden performance dropoff detected - investigate resource bottlenecks".to_string());
                    },
                    DegradationPattern::LinearDecline => {
                        recommendations.push("📉 Linear performance decline - monitor trends and consider optimization".to_string());
                    },
                    _ => {},
                }
            }
        }
        
        // Check breach patterns
        let critical_breach_count = threshold_breaches.iter().filter(|b| b.severity == BreachSeverity::Critical).count();
        let emergency_breach_count = threshold_breaches.iter().filter(|b| b.severity == BreachSeverity::Emergency).count();
        
        if emergency_breach_count > 0 {
            recommendations.push("🆘 EMERGENCY: System has reached emergency thresholds - immediate action required".to_string());
        } else if critical_breach_count > 5 {
            recommendations.push("🔥 Multiple critical thresholds breached - system approaching limits".to_string());
        }
        
        // System health recommendations
        if let Some(latest_result) = analysis_results.last() {
            if latest_result.system_health_score < 0.5 {
                recommendations.push("🏥 System health critically low - consider load reduction and resource scaling".to_string());
            } else if latest_result.system_health_score < 0.7 {
                recommendations.push("⚡ System health declining - monitor resource utilization".to_string());
            }
        }
        
        if recommendations.is_empty() {
            recommendations.push("✅ System performing within acceptable parameters".to_string());
        }
        
        recommendations
    }
}

/// Final analysis report
#[derive(Debug, Clone)]
pub struct AnalysisReport {
    /// Total test duration in seconds
    pub test_duration: u64,
    /// Total performance measurements taken
    pub total_measurements: usize,
    /// Total threshold breaches
    pub total_threshold_breaches: usize,
    /// Number of analysis results generated
    pub analysis_results_count: usize,
    /// Final system health score
    pub final_system_health: f64,
    /// Critical breach count
    pub critical_breaches: usize,
    /// Emergency breach count
    pub emergency_breaches: usize,
    /// Final stability assessment
    pub stability_assessment: SystemStabilityMetrics,
    /// Recommendations based on analysis
    pub recommendations: Vec<String>,
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progressive_degradation_analyzer_creation() -> Result<()> {
        let config = ProgressiveDegradationConfig::default();
        let analyzer = ProgressiveDegradationAnalyzer::new(config);
        
        assert!(!analyzer.analysis_active.load(Ordering::SeqCst));
        assert!(!analyzer.prediction_active.load(Ordering::SeqCst));
        
        Ok(())
    }

    #[test]
    fn test_threshold_values_defaults() -> Result<()> {
        let latency_thresholds = ProgressiveDegradationAnalyzer::default_threshold_values(&ThresholdType::PerformanceLatency);
        assert_eq!(latency_thresholds.baseline, 100.0);
        assert_eq!(latency_thresholds.warning, 200.0);
        assert_eq!(latency_thresholds.critical, 500.0);
        assert_eq!(latency_thresholds.emergency, 1000.0);
        
        let memory_thresholds = ProgressiveDegradationAnalyzer::default_threshold_values(&ThresholdType::MemoryUsage);
        assert_eq!(memory_thresholds.baseline, 50.0);
        assert_eq!(memory_thresholds.warning, 70.0);
        assert_eq!(memory_thresholds.critical, 85.0);
        assert_eq!(memory_thresholds.emergency, 95.0);
        
        Ok(())
    }

    #[test]
    fn test_performance_measurement() -> Result<()> {
        let measurement = ProgressiveDegradationAnalyzer::take_performance_measurement_static(1.0)?;
        
        assert_eq!(measurement.load_multiplier, 1.0);
        assert!(measurement.latency_ms > 0.0);
        assert!(measurement.throughput_ops_per_sec > 0.0);
        assert!(measurement.memory_usage_mb > 0.0);
        assert!(measurement.cpu_utilization_percent >= 0.0);
        assert!(measurement.stability_score >= 0.0 && measurement.stability_score <= 1.0);
        
        Ok(())
    }

    #[test]
    fn test_progressive_load_effects() -> Result<()> {
        let measurement_1x = ProgressiveDegradationAnalyzer::take_performance_measurement_static(1.0)?;
        let measurement_2x = ProgressiveDegradationAnalyzer::take_performance_measurement_static(2.0)?;
        let measurement_3x = ProgressiveDegradationAnalyzer::take_performance_measurement_static(3.0)?;
        
        // Throughput should decrease with higher load
        assert!(measurement_1x.throughput_ops_per_sec >= measurement_2x.throughput_ops_per_sec);
        assert!(measurement_2x.throughput_ops_per_sec >= measurement_3x.throughput_ops_per_sec);
        
        // Memory usage should increase with higher load
        assert!(measurement_1x.memory_usage_mb <= measurement_2x.memory_usage_mb);
        assert!(measurement_2x.memory_usage_mb <= measurement_3x.memory_usage_mb);
        
        // CPU utilization should increase with higher load
        assert!(measurement_1x.cpu_utilization_percent <= measurement_2x.cpu_utilization_percent);
        assert!(measurement_2x.cpu_utilization_percent <= measurement_3x.cpu_utilization_percent);
        
        Ok(())
    }

    #[test]
    fn test_threshold_breach_detection() -> Result<()> {
        let thresholds = ThresholdValues {
            baseline: 100.0,
            warning: 200.0,
            critical: 300.0,
            emergency: 400.0,
        };
        
        // Test normal value (no breach)
        let breach = ProgressiveDegradationAnalyzer::check_threshold_breach(
            150.0, &thresholds, &ThresholdType::PerformanceLatency
        );
        assert!(breach.is_none());
        
        // Test warning breach
        let breach = ProgressiveDegradationAnalyzer::check_threshold_breach(
            250.0, &thresholds, &ThresholdType::PerformanceLatency
        );
        assert!(breach.is_some());
        assert_eq!(breach.unwrap().0, BreachSeverity::Warning);
        
        // Test critical breach
        let breach = ProgressiveDegradationAnalyzer::check_threshold_breach(
            350.0, &thresholds, &ThresholdType::PerformanceLatency
        );
        assert!(breach.is_some());
        assert_eq!(breach.unwrap().0, BreachSeverity::Critical);
        
        // Test emergency breach
        let breach = ProgressiveDegradationAnalyzer::check_threshold_breach(
            450.0, &thresholds, &ThresholdType::PerformanceLatency
        );
        assert!(breach.is_some());
        assert_eq!(breach.unwrap().0, BreachSeverity::Emergency);
        
        Ok(())
    }

    #[test]
    fn test_degradation_analysis_short_test() -> Result<()> {
        let config = ProgressiveDegradationConfig {
            max_test_duration_secs: 10, // Short test
            load_increase_interval_secs: 2,
            ..Default::default()
        };
        let analyzer = ProgressiveDegradationAnalyzer::new(config);
        
        // Start analysis
        analyzer.start_analysis()?;
        
        // Let it run for a few seconds
        thread::sleep(Duration::from_secs(3));
        
        // Stop analysis
        analyzer.stop_analysis();
        
        // Check that we have some measurements
        let history = analyzer.performance_history.lock().unwrap();
        assert!(!history.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_baseline_establishment() -> Result<()> {
        let config = ProgressiveDegradationConfig::default();
        let analyzer = ProgressiveDegradationAnalyzer::new(config);
        
        // Establish baseline
        analyzer.establish_baseline()?;
        
        // Check baseline was created
        let baseline = analyzer.performance_baseline.lock().unwrap();
        assert!(baseline.is_some());
        
        let baseline = baseline.as_ref().unwrap();
        assert!(baseline.measurements.throughput_ops_per_sec > 0.0);
        assert!(baseline.confidence > 0.0);
        
        Ok(())
    }

    #[test]
    fn test_metric_extraction() -> Result<()> {
        let measurement = PerformanceMeasurement {
            timestamp: Instant::now(),
            load_multiplier: 1.0,
            latency_ms: 100.0,
            throughput_ops_per_sec: 1000.0,
            memory_usage_mb: 512.0,
            cpu_utilization_percent: 50.0,
            error_rate_percent: 1.0,
            response_time_ms: 120.0,
            queue_depth: 10,
            stability_score: 0.95,
        };
        
        assert_eq!(ProgressiveDegradationAnalyzer::get_metric_value(&measurement, &ThresholdType::PerformanceLatency), 100.0);
        assert_eq!(ProgressiveDegradationAnalyzer::get_metric_value(&measurement, &ThresholdType::PerformanceThroughput), 1000.0);
        assert_eq!(ProgressiveDegradationAnalyzer::get_metric_value(&measurement, &ThresholdType::MemoryUsage), 512.0);
        assert_eq!(ProgressiveDegradationAnalyzer::get_metric_value(&measurement, &ThresholdType::CpuUtilization), 50.0);
        assert_eq!(ProgressiveDegradationAnalyzer::get_metric_value(&measurement, &ThresholdType::ErrorRate), 1.0);
        assert_eq!(ProgressiveDegradationAnalyzer::get_metric_value(&measurement, &ThresholdType::ResponseTime), 120.0);
        assert_eq!(ProgressiveDegradationAnalyzer::get_metric_value(&measurement, &ThresholdType::QueueDepth), 10.0);
        
        Ok(())
    }

    #[test]
    fn test_linear_decline_detection() -> Result<()> {
        let mut history = VecDeque::new();
        let start_time = Instant::now();
        
        // Create samples with declining throughput
        for i in 0..25 {
            let measurement = PerformanceMeasurement {
                timestamp: start_time + Duration::from_secs(i),
                load_multiplier: 1.0,
                latency_ms: 100.0,
                throughput_ops_per_sec: 1000.0 - (i as f64 * 20.0), // Declining throughput
                memory_usage_mb: 512.0,
                cpu_utilization_percent: 50.0,
                error_rate_percent: 1.0,
                response_time_ms: 120.0,
                queue_depth: 10,
                stability_score: 0.95,
            };
            history.push_back(measurement);
        }
        
        let pattern = ProgressiveDegradationAnalyzer::detect_linear_decline(&history);
        assert!(pattern.is_some());
        
        let pattern = pattern.unwrap();
        assert_eq!(pattern.pattern_type, DegradationPattern::LinearDecline);
        assert!(pattern.confidence > 0.0);
        assert!(pattern.characteristics.rate_of_change < 0.0); // Negative slope
        
        Ok(())
    }

    #[test]
    fn test_system_health_calculation() -> Result<()> {
        let mut history = VecDeque::new();
        let monitoring = HashMap::new();
        
        // Add some measurements
        for i in 0..10 {
            let measurement = PerformanceMeasurement {
                timestamp: Instant::now(),
                load_multiplier: 1.0,
                latency_ms: 100.0,
                throughput_ops_per_sec: 1000.0,
                memory_usage_mb: 512.0,
                cpu_utilization_percent: 50.0,
                error_rate_percent: 2.0, // 2% error rate
                response_time_ms: 120.0,
                queue_depth: 10,
                stability_score: 0.8,
            };
            history.push_back(measurement);
        }
        
        let health_score = ProgressiveDegradationAnalyzer::calculate_system_health_score(&history, &monitoring);
        assert!(health_score > 0.0 && health_score <= 1.0);
        assert!(health_score < 0.8); // Should be lower due to error rate and stability
        
        Ok(())
    }

    #[test]
    fn test_prediction_model_training() -> Result<()> {
        let mut history = VecDeque::new();
        
        // Create training data with clear trend
        for i in 0..20 {
            let measurement = PerformanceMeasurement {
                timestamp: Instant::now(),
                load_multiplier: 1.0,
                latency_ms: 100.0 + (i as f64 * 5.0), // Increasing latency
                throughput_ops_per_sec: 1000.0,
                memory_usage_mb: 512.0,
                cpu_utilization_percent: 50.0,
                error_rate_percent: 1.0,
                response_time_ms: 120.0,
                queue_depth: 10,
                stability_score: 0.95,
            };
            history.push_back(measurement);
        }
        
        let mut model = PredictionModel {
            model_type: PredictionModelType::LinearRegression,
            parameters: HashMap::new(),
            accuracy: 0.0,
            training_data_points: 0,
            last_update: Instant::now(),
        };
        
        ProgressiveDegradationAnalyzer::train_prediction_model(&history, &ThresholdType::PerformanceLatency, &mut model);
        
        assert!(model.parameters.contains_key("slope"));
        assert!(model.parameters.contains_key("intercept"));
        assert!(model.training_data_points > 0);
        assert!(model.accuracy >= 0.0);
        
        // Slope should be positive for increasing latency
        let slope = model.parameters.get("slope").unwrap();
        assert!(*slope > 0.0);
        
        Ok(())
    }

    #[test]
    fn test_comprehensive_analysis_workflow() -> Result<()> {
        println!("🧪 Testing comprehensive progressive degradation analysis workflow...");
        
        let config = ProgressiveDegradationConfig {
            max_test_duration_secs: 5, // Very short test
            analysis_interval_ms: 100,
            load_increase_interval_secs: 1,
            load_increment_step_percent: 50.0, // Aggressive load increase
            max_load_multiplier: 3.0,
            enable_predictive_analysis: true,
            enable_realtime_monitoring: true,
            ..Default::default()
        };
        
        let analyzer = ProgressiveDegradationAnalyzer::new(config);
        
        // Start analysis
        analyzer.start_analysis()?;
        
        // Let analysis run
        thread::sleep(Duration::from_secs(3));
        
        // Check we have data
        let history = analyzer.performance_history.lock().unwrap();
        assert!(!history.is_empty(), "Should have performance measurements");
        
        let first_measurement = history.front().unwrap();
        let last_measurement = history.back().unwrap();
        
        // Load should have increased
        assert!(last_measurement.load_multiplier >= first_measurement.load_multiplier);
        
        drop(history);
        
        // Stop analysis
        analyzer.stop_analysis();
        
        // Generate report
        let report = analyzer.generate_analysis_report();
        assert!(report.total_measurements > 0);
        assert!(report.analysis_results_count >= 0);
        assert!(!report.recommendations.is_empty());
        
        println!("✅ Comprehensive analysis completed:");
        println!("   📊 Total measurements: {}", report.total_measurements);
        println!("   🔍 Analysis results: {}", report.analysis_results_count);
        println!("   ⚠️ Threshold breaches: {}", report.total_threshold_breaches);
        println!("   💊 System health: {:.2}", report.final_system_health);
        println!("   📋 Recommendations: {}", report.recommendations.len());
        
        for (i, recommendation) in report.recommendations.iter().enumerate() {
            println!("      {}. {}", i + 1, recommendation);
        }
        
        Ok(())
    }
}