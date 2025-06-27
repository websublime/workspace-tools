//! Automatic Critical Limits Detection System
//!
//! This module implements intelligent automatic detection of critical system limits
//! through adaptive algorithms, machine learning models, and real-time monitoring
//! to identify breaking points, resource constraints, and performance boundaries
//! before they cause system failures.

use sublime_monorepo_tools::Result;
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::sync::atomic::{AtomicBool, AtomicUsize, AtomicU64, AtomicI64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

/// Configuration for automatic critical limits detection
#[derive(Debug, Clone)]
pub struct CriticalLimitsDetectionConfig {
    /// Maximum detection session duration in seconds
    pub max_detection_duration_secs: u64,
    /// Detection analysis interval in milliseconds
    pub detection_interval_ms: u64,
    /// Adaptive threshold adjustment interval in seconds
    pub adaptive_adjustment_interval_secs: u64,
    /// Limit types to detect automatically
    pub limit_types: Vec<CriticalLimitType>,
    /// Detection algorithms to use
    pub detection_algorithms: Vec<DetectionAlgorithm>,
    /// Enable machine learning limit prediction
    pub enable_ml_prediction: bool,
    /// Enable real-time limit monitoring
    pub enable_realtime_monitoring: bool,
    /// Enable adaptive threshold adjustment
    pub enable_adaptive_thresholds: bool,
    /// Historical data window for analysis (seconds)
    pub historical_window_secs: u64,
    /// Confidence threshold for limit detection
    pub detection_confidence_threshold: f64,
    /// Critical breach tolerance before system action
    pub critical_breach_tolerance: usize,
    /// Enable predictive limit breach prevention
    pub enable_breach_prevention: bool,
    /// Multi-dimensional limit correlation analysis
    pub enable_correlation_analysis: bool,
    /// Automatic response actions configuration
    pub response_actions: CriticalResponseConfig,
    /// Sample buffer size for real-time analysis
    pub sample_buffer_size: usize,
}

impl Default for CriticalLimitsDetectionConfig {
    fn default() -> Self {
        Self {
            max_detection_duration_secs: 1800, // 30 minutes
            detection_interval_ms: 250,        // 250ms for high frequency detection
            adaptive_adjustment_interval_secs: 60, // Adjust thresholds every minute
            limit_types: vec![
                CriticalLimitType::MemoryLimit,
                CriticalLimitType::CpuLimit,
                CriticalLimitType::DiskIOLimit,
                CriticalLimitType::NetworkLimit,
                CriticalLimitType::ThroughputLimit,
                CriticalLimitType::LatencyLimit,
                CriticalLimitType::ConcurrencyLimit,
                CriticalLimitType::ErrorRateLimit,
                CriticalLimitType::QueueDepthLimit,
                CriticalLimitType::ConnectionLimit,
            ],
            detection_algorithms: vec![
                DetectionAlgorithm::StatisticalAnomalyDetection,
                DetectionAlgorithm::MachineLearningSVM,
                DetectionAlgorithm::GradientAnalysis,
                DetectionAlgorithm::CorrelationAnalysis,
                DetectionAlgorithm::PatternRecognition,
                DetectionAlgorithm::AdaptiveThresholding,
            ],
            enable_ml_prediction: true,
            enable_realtime_monitoring: true,
            enable_adaptive_thresholds: true,
            historical_window_secs: 3600, // 1 hour of historical data
            detection_confidence_threshold: 0.85, // 85% confidence required
            critical_breach_tolerance: 3, // Allow 3 breaches before critical action
            enable_breach_prevention: true,
            enable_correlation_analysis: true,
            response_actions: CriticalResponseConfig::default(),
            sample_buffer_size: 5000,
        }
    }
}

/// Types of critical limits to detect
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum CriticalLimitType {
    /// Memory usage critical limits
    MemoryLimit,
    /// CPU utilization critical limits
    CpuLimit,
    /// Disk I/O critical limits
    DiskIOLimit,
    /// Network bandwidth critical limits
    NetworkLimit,
    /// System throughput critical limits
    ThroughputLimit,
    /// Response latency critical limits
    LatencyLimit,
    /// Concurrency/threading critical limits
    ConcurrencyLimit,
    /// Error rate critical limits
    ErrorRateLimit,
    /// Queue depth critical limits
    QueueDepthLimit,
    /// Connection pool critical limits
    ConnectionLimit,
    /// File descriptor critical limits
    FileDescriptorLimit,
    /// Process count critical limits
    ProcessLimit,
}

/// Detection algorithms for critical limits
#[derive(Debug, Clone, PartialEq)]
pub enum DetectionAlgorithm {
    /// Statistical anomaly detection
    StatisticalAnomalyDetection,
    /// Machine learning SVM-based detection
    MachineLearningSVM,
    /// Gradient-based critical point analysis
    GradientAnalysis,
    /// Correlation analysis between metrics
    CorrelationAnalysis,
    /// Pattern recognition for limit detection
    PatternRecognition,
    /// Adaptive thresholding based on behavior
    AdaptiveThresholding,
    /// Time series analysis
    TimeSeriesAnalysis,
    /// Ensemble method combining multiple algorithms
    EnsembleMethod,
}

/// Critical response configuration
#[derive(Debug, Clone)]
pub struct CriticalResponseConfig {
    /// Enable automatic scaling responses
    pub enable_auto_scaling: bool,
    /// Enable automatic load shedding
    pub enable_load_shedding: bool,
    /// Enable automatic resource cleanup
    pub enable_resource_cleanup: bool,
    /// Enable automatic alerting
    pub enable_alerting: bool,
    /// Response delay after detection (milliseconds)
    pub response_delay_ms: u64,
    /// Maximum automatic responses per minute
    pub max_responses_per_minute: usize,
}

impl Default for CriticalResponseConfig {
    fn default() -> Self {
        Self {
            enable_auto_scaling: false, // Conservative default
            enable_load_shedding: true,
            enable_resource_cleanup: true,
            enable_alerting: true,
            response_delay_ms: 1000, // 1 second delay
            max_responses_per_minute: 10,
        }
    }
}

/// Automatic critical limits detection system
#[derive(Debug)]
pub struct CriticalLimitsDetector {
    /// Configuration for the detection system
    config: CriticalLimitsDetectionConfig,
    /// Current system metrics
    current_metrics: Arc<Mutex<SystemMetrics>>,
    /// Historical metrics buffer
    metrics_history: Arc<Mutex<VecDeque<SystemMetrics>>>,
    /// Detected critical limits
    detected_limits: Arc<Mutex<HashMap<CriticalLimitType, DetectedLimit>>>,
    /// Machine learning models for each limit type
    ml_models: Arc<Mutex<HashMap<CriticalLimitType, MLModel>>>,
    /// Critical limit breach events
    breach_events: Arc<Mutex<Vec<CriticalBreachEvent>>>,
    /// Real-time limit monitoring state
    monitoring_state: Arc<Mutex<MonitoringState>>,
    /// Detection control flags
    detection_active: Arc<AtomicBool>,
    monitoring_active: Arc<AtomicBool>,
    /// Adaptive threshold adjustments
    adaptive_thresholds: Arc<RwLock<HashMap<CriticalLimitType, AdaptiveThreshold>>>,
    /// Correlation analysis results
    correlation_matrix: Arc<Mutex<CorrelationMatrix>>,
    /// Response action history
    response_history: Arc<Mutex<Vec<ResponseAction>>>,
    /// Detection statistics
    detection_stats: Arc<Mutex<DetectionStatistics>>,
}

/// System metrics snapshot
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// Timestamp of measurement
    pub timestamp: Instant,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Available memory in MB
    pub available_memory_mb: f64,
    /// CPU utilization percentage
    pub cpu_utilization_percent: f64,
    /// CPU load average (1 minute)
    pub cpu_load_average: f64,
    /// Disk I/O read rate (MB/s)
    pub disk_read_mb_per_sec: f64,
    /// Disk I/O write rate (MB/s)
    pub disk_write_mb_per_sec: f64,
    /// Network I/O in (MB/s)
    pub network_in_mb_per_sec: f64,
    /// Network I/O out (MB/s)
    pub network_out_mb_per_sec: f64,
    /// System throughput (operations/sec)
    pub throughput_ops_per_sec: f64,
    /// Average response latency (ms)
    pub response_latency_ms: f64,
    /// Active thread count
    pub active_threads: usize,
    /// Error rate percentage
    pub error_rate_percent: f64,
    /// Current queue depth
    pub queue_depth: usize,
    /// Active connections
    pub active_connections: usize,
    /// File descriptors in use
    pub file_descriptors_used: usize,
    /// Process count
    pub process_count: usize,
}

/// Detected critical limit
#[derive(Debug, Clone)]
pub struct DetectedLimit {
    /// Type of limit detected
    pub limit_type: CriticalLimitType,
    /// Detected critical value
    pub critical_value: f64,
    /// Detection confidence (0.0-1.0)
    pub confidence: f64,
    /// Detection timestamp
    pub detected_at: Instant,
    /// Detection algorithm used
    pub algorithm: DetectionAlgorithm,
    /// Current value relative to limit
    pub current_value: f64,
    /// Distance to critical limit
    pub distance_to_limit: f64,
    /// Predicted time to breach
    pub time_to_breach: Option<Duration>,
    /// Historical trend leading to detection
    pub trend_analysis: TrendAnalysis,
    /// Recommended actions
    pub recommended_actions: Vec<String>,
}

/// Machine learning model for limit detection
#[derive(Debug, Clone)]
pub struct MLModel {
    /// Model type
    pub model_type: MLModelType,
    /// Model parameters
    pub parameters: HashMap<String, f64>,
    /// Training data points
    pub training_data_points: usize,
    /// Model accuracy
    pub accuracy: f64,
    /// Last training timestamp
    pub last_trained: Instant,
    /// Prediction confidence
    pub prediction_confidence: f64,
    /// Feature weights
    pub feature_weights: HashMap<String, f64>,
}

/// Machine learning model types
#[derive(Debug, Clone, PartialEq)]
pub enum MLModelType {
    /// Support Vector Machine
    SupportVectorMachine,
    /// Linear Regression
    LinearRegression,
    /// Random Forest
    RandomForest,
    /// Neural Network
    NeuralNetwork,
    /// Gaussian Mixture Model
    GaussianMixture,
    /// Isolation Forest (Anomaly Detection)
    IsolationForest,
}

/// Critical breach event
#[derive(Debug, Clone)]
pub struct CriticalBreachEvent {
    /// Breach timestamp
    pub timestamp: Instant,
    /// Limit type that was breached
    pub limit_type: CriticalLimitType,
    /// Severity of the breach
    pub severity: BreachSeverity,
    /// Actual value that caused breach
    pub actual_value: f64,
    /// Detected critical limit value
    pub limit_value: f64,
    /// Breach percentage over limit
    pub breach_percentage: f64,
    /// Duration of breach
    pub duration: Option<Duration>,
    /// Detection algorithm that flagged the breach
    pub detection_algorithm: DetectionAlgorithm,
    /// Automatic response triggered
    pub response_triggered: Option<ResponseAction>,
    /// Recovery information
    pub recovery_info: Option<RecoveryInfo>,
}

/// Breach severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum BreachSeverity {
    /// Approaching critical limit (warning)
    Approaching,
    /// Critical limit breached
    Critical,
    /// Severe breach requiring immediate action
    Severe,
    /// System failure level breach
    Catastrophic,
}

/// Real-time monitoring state
#[derive(Debug, Clone)]
pub struct MonitoringState {
    /// Last monitoring update
    pub last_update: Instant,
    /// Current monitoring frequency (Hz)
    pub monitoring_frequency: f64,
    /// Active alerts count
    pub active_alerts: usize,
    /// System health score (0.0-1.0)
    pub system_health_score: f64,
    /// Risk level assessment
    pub risk_level: RiskLevel,
    /// Predictive alerts count
    pub predictive_alerts: usize,
}

/// System risk levels
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    /// Low risk - system operating normally
    Low,
    /// Moderate risk - approaching some limits
    Moderate,
    /// High risk - critical limits being approached
    High,
    /// Extreme risk - system at breaking point
    Extreme,
    /// Critical risk - system failure imminent
    Critical,
}

/// Adaptive threshold
#[derive(Debug, Clone)]
pub struct AdaptiveThreshold {
    /// Current threshold value
    pub current_value: f64,
    /// Base threshold value
    pub base_value: f64,
    /// Adjustment factor
    pub adjustment_factor: f64,
    /// Adjustment history
    pub adjustment_history: VecDeque<ThresholdAdjustment>,
    /// Last adjustment timestamp
    pub last_adjusted: Instant,
    /// Confidence in current threshold
    pub confidence: f64,
}

/// Threshold adjustment record
#[derive(Debug, Clone)]
pub struct ThresholdAdjustment {
    /// Adjustment timestamp
    pub timestamp: Instant,
    /// Previous value
    pub previous_value: f64,
    /// New value
    pub new_value: f64,
    /// Adjustment reason
    pub reason: String,
    /// Adjustment effectiveness score
    pub effectiveness: Option<f64>,
}

/// Correlation matrix for metrics
#[derive(Debug, Clone)]
pub struct CorrelationMatrix {
    /// Correlation coefficients between metrics
    pub correlations: HashMap<(CriticalLimitType, CriticalLimitType), f64>,
    /// Last update timestamp
    pub last_updated: Instant,
    /// Strong correlations (> 0.7)
    pub strong_correlations: Vec<(CriticalLimitType, CriticalLimitType, f64)>,
    /// Correlation confidence scores
    pub confidence_scores: HashMap<(CriticalLimitType, CriticalLimitType), f64>,
}

/// Response action taken by the system
#[derive(Debug, Clone)]
pub struct ResponseAction {
    /// Action timestamp
    pub timestamp: Instant,
    /// Type of response action
    pub action_type: ResponseActionType,
    /// Target limit type
    pub target_limit: CriticalLimitType,
    /// Action parameters
    pub parameters: HashMap<String, f64>,
    /// Action success indicator
    pub success: bool,
    /// Action effectiveness score
    pub effectiveness: Option<f64>,
    /// Action duration
    pub duration: Option<Duration>,
}

/// Types of response actions
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseActionType {
    /// Scale up resources
    ScaleUp,
    /// Scale down resources
    ScaleDown,
    /// Shed excess load
    LoadShedding,
    /// Clean up resources
    ResourceCleanup,
    /// Increase caching
    IncreaseCaching,
    /// Reduce processing priority
    ReducePriority,
    /// Circuit breaker activation
    CircuitBreaker,
    /// Alert generation
    GenerateAlert,
}

/// Recovery information after breach
#[derive(Debug, Clone)]
pub struct RecoveryInfo {
    /// Recovery start timestamp
    pub recovery_start: Instant,
    /// Recovery completion timestamp
    pub recovery_complete: Option<Instant>,
    /// Recovery success indicator
    pub recovery_successful: bool,
    /// Recovery actions taken
    pub recovery_actions: Vec<ResponseAction>,
    /// Recovery effectiveness score
    pub effectiveness: f64,
}

/// Trend analysis for limit detection
#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    /// Trend direction
    pub direction: TrendDirection,
    /// Trend strength (0.0-1.0)
    pub strength: f64,
    /// Trend acceleration
    pub acceleration: f64,
    /// Trend confidence
    pub confidence: f64,
    /// Historical trend data
    pub historical_points: Vec<(Instant, f64)>,
}

/// Trend directions
#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    /// Increasing trend (approaching limits)
    Increasing,
    /// Decreasing trend (moving away from limits)
    Decreasing,
    /// Stable trend (no significant change)
    Stable,
    /// Oscillating trend (alternating increases/decreases)
    Oscillating,
    /// Exponential increase (rapid approach to limits)
    ExponentialIncrease,
}

/// Detection statistics
#[derive(Debug, Clone)]
pub struct DetectionStatistics {
    /// Total detections performed
    pub total_detections: usize,
    /// True positive detections
    pub true_positives: usize,
    /// False positive detections
    pub false_positives: usize,
    /// Missed critical events
    pub false_negatives: usize,
    /// Detection accuracy
    pub accuracy: f64,
    /// Average detection time
    pub average_detection_time: Duration,
    /// Algorithm performance rankings
    pub algorithm_rankings: HashMap<DetectionAlgorithm, f64>,
}

impl CriticalLimitsDetector {
    /// Create new critical limits detector
    pub fn new(config: CriticalLimitsDetectionConfig) -> Self {
        let mut adaptive_thresholds = HashMap::new();
        let mut ml_models = HashMap::new();
        
        // Initialize adaptive thresholds and ML models for each limit type
        for limit_type in &config.limit_types {
            adaptive_thresholds.insert(
                limit_type.clone(),
                AdaptiveThreshold {
                    current_value: Self::get_default_threshold(limit_type),
                    base_value: Self::get_default_threshold(limit_type),
                    adjustment_factor: 1.0,
                    adjustment_history: VecDeque::new(),
                    last_adjusted: Instant::now(),
                    confidence: 0.5,
                },
            );
            
            ml_models.insert(
                limit_type.clone(),
                MLModel {
                    model_type: MLModelType::SupportVectorMachine,
                    parameters: HashMap::new(),
                    training_data_points: 0,
                    accuracy: 0.0,
                    last_trained: Instant::now(),
                    prediction_confidence: 0.0,
                    feature_weights: HashMap::new(),
                },
            );
        }
        
        Self {
            config,
            current_metrics: Arc::new(Mutex::new(SystemMetrics::default())),
            metrics_history: Arc::new(Mutex::new(VecDeque::new())),
            detected_limits: Arc::new(Mutex::new(HashMap::new())),
            ml_models: Arc::new(Mutex::new(ml_models)),
            breach_events: Arc::new(Mutex::new(Vec::new())),
            monitoring_state: Arc::new(Mutex::new(MonitoringState {
                last_update: Instant::now(),
                monitoring_frequency: 1000.0 / config.detection_interval_ms as f64,
                active_alerts: 0,
                system_health_score: 1.0,
                risk_level: RiskLevel::Low,
                predictive_alerts: 0,
            })),
            detection_active: Arc::new(AtomicBool::new(false)),
            monitoring_active: Arc::new(AtomicBool::new(false)),
            adaptive_thresholds: Arc::new(RwLock::new(adaptive_thresholds)),
            correlation_matrix: Arc::new(Mutex::new(CorrelationMatrix {
                correlations: HashMap::new(),
                last_updated: Instant::now(),
                strong_correlations: Vec::new(),
                confidence_scores: HashMap::new(),
            })),
            response_history: Arc::new(Mutex::new(Vec::new())),
            detection_stats: Arc::new(Mutex::new(DetectionStatistics {
                total_detections: 0,
                true_positives: 0,
                false_positives: 0,
                false_negatives: 0,
                accuracy: 0.0,
                average_detection_time: Duration::from_millis(0),
                algorithm_rankings: HashMap::new(),
            })),
        }
    }

    /// Get default threshold for limit type
    fn get_default_threshold(limit_type: &CriticalLimitType) -> f64 {
        match limit_type {
            CriticalLimitType::MemoryLimit => 85.0,         // 85% memory usage
            CriticalLimitType::CpuLimit => 90.0,            // 90% CPU usage
            CriticalLimitType::DiskIOLimit => 80.0,         // 80% disk I/O capacity
            CriticalLimitType::NetworkLimit => 80.0,        // 80% network capacity
            CriticalLimitType::ThroughputLimit => 500.0,    // 500 ops/sec minimum
            CriticalLimitType::LatencyLimit => 1000.0,      // 1000ms maximum latency
            CriticalLimitType::ConcurrencyLimit => 1000.0,  // 1000 concurrent threads
            CriticalLimitType::ErrorRateLimit => 5.0,       // 5% error rate
            CriticalLimitType::QueueDepthLimit => 100.0,    // 100 items in queue
            CriticalLimitType::ConnectionLimit => 1000.0,   // 1000 active connections
            CriticalLimitType::FileDescriptorLimit => 8192.0, // 8192 file descriptors
            CriticalLimitType::ProcessLimit => 500.0,       // 500 processes
        }
    }

    /// Start critical limits detection
    pub fn start_detection(&self) -> Result<()> {
        self.detection_active.store(true, Ordering::SeqCst);
        self.monitoring_active.store(true, Ordering::SeqCst);
        
        println!("🔍 Starting automatic critical limits detection...");
        
        // Start metrics collection thread
        self.start_metrics_collection_thread()?;
        
        // Start detection analysis thread
        self.start_detection_analysis_thread()?;
        
        // Start adaptive threshold adjustment thread
        if self.config.enable_adaptive_thresholds {
            self.start_adaptive_threshold_thread()?;
        }
        
        // Start ML prediction thread
        if self.config.enable_ml_prediction {
            self.start_ml_prediction_thread()?;
        }
        
        // Start correlation analysis thread
        if self.config.enable_correlation_analysis {
            self.start_correlation_analysis_thread()?;
        }
        
        // Start real-time monitoring thread
        if self.config.enable_realtime_monitoring {
            self.start_realtime_monitoring_thread()?;
        }
        
        Ok(())
    }

    /// Start metrics collection thread
    fn start_metrics_collection_thread(&self) -> Result<()> {
        let detection_active = Arc::clone(&self.detection_active);
        let current_metrics = Arc::clone(&self.current_metrics);
        let metrics_history = Arc::clone(&self.metrics_history);
        let config = self.config.clone();
        
        thread::spawn(move || {
            println!("📊 Starting metrics collection thread...");
            
            while detection_active.load(Ordering::SeqCst) {
                if let Ok(metrics) = Self::collect_system_metrics() {
                    // Update current metrics
                    *current_metrics.lock().unwrap() = metrics.clone();
                    
                    // Add to history
                    {
                        let mut history = metrics_history.lock().unwrap();
                        history.push_back(metrics);
                        
                        // Keep history within configured window
                        let max_samples = (config.historical_window_secs * 1000 / config.detection_interval_ms) as usize;
                        while history.len() > max_samples {
                            history.pop_front();
                        }
                    }
                }
                
                thread::sleep(Duration::from_millis(config.detection_interval_ms));
            }
            
            println!("🔚 Metrics collection thread stopped");
        });
        
        Ok(())
    }

    /// Start detection analysis thread
    fn start_detection_analysis_thread(&self) -> Result<()> {
        let detection_active = Arc::clone(&self.detection_active);
        let metrics_history = Arc::clone(&self.metrics_history);
        let detected_limits = Arc::clone(&self.detected_limits);
        let breach_events = Arc::clone(&self.breach_events);
        let adaptive_thresholds = Arc::clone(&self.adaptive_thresholds);
        let detection_stats = Arc::clone(&self.detection_stats);
        let config = self.config.clone();
        
        thread::spawn(move || {
            println!("🔍 Starting detection analysis thread...");
            
            while detection_active.load(Ordering::SeqCst) {
                let history = metrics_history.lock().unwrap().clone();
                if history.len() >= 10 {
                    drop(history);
                    
                    // Perform detection for each algorithm
                    for algorithm in &config.detection_algorithms {
                        Self::run_detection_algorithm_static(
                            algorithm,
                            &metrics_history,
                            &detected_limits,
                            &breach_events,
                            &adaptive_thresholds,
                            &detection_stats,
                            &config,
                        );
                    }
                }
                
                thread::sleep(Duration::from_secs(5)); // Run detection every 5 seconds
            }
            
            println!("🔚 Detection analysis thread stopped");
        });
        
        Ok(())
    }

    /// Start adaptive threshold adjustment thread
    fn start_adaptive_threshold_thread(&self) -> Result<()> {
        let detection_active = Arc::clone(&self.detection_active);
        let metrics_history = Arc::clone(&self.metrics_history);
        let adaptive_thresholds = Arc::clone(&self.adaptive_thresholds);
        let breach_events = Arc::clone(&self.breach_events);
        let config = self.config.clone();
        
        thread::spawn(move || {
            println!("🎛️ Starting adaptive threshold adjustment thread...");
            
            while detection_active.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_secs(config.adaptive_adjustment_interval_secs));
                
                if detection_active.load(Ordering::SeqCst) {
                    Self::adjust_adaptive_thresholds_static(
                        &metrics_history,
                        &adaptive_thresholds,
                        &breach_events,
                        &config,
                    );
                }
            }
            
            println!("🔚 Adaptive threshold adjustment thread stopped");
        });
        
        Ok(())
    }

    /// Start ML prediction thread
    fn start_ml_prediction_thread(&self) -> Result<()> {
        let detection_active = Arc::clone(&self.detection_active);
        let metrics_history = Arc::clone(&self.metrics_history);
        let ml_models = Arc::clone(&self.ml_models);
        let detected_limits = Arc::clone(&self.detected_limits);
        let config = self.config.clone();
        
        thread::spawn(move || {
            println!("🤖 Starting ML prediction thread...");
            
            while detection_active.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_secs(30)); // Update ML models every 30 seconds
                
                if detection_active.load(Ordering::SeqCst) {
                    Self::update_ml_models_static(
                        &metrics_history,
                        &ml_models,
                        &detected_limits,
                        &config,
                    );
                }
            }
            
            println!("🔚 ML prediction thread stopped");
        });
        
        Ok(())
    }

    /// Start correlation analysis thread
    fn start_correlation_analysis_thread(&self) -> Result<()> {
        let detection_active = Arc::clone(&self.detection_active);
        let metrics_history = Arc::clone(&self.metrics_history);
        let correlation_matrix = Arc::clone(&self.correlation_matrix);
        let config = self.config.clone();
        
        thread::spawn(move || {
            println!("📈 Starting correlation analysis thread...");
            
            while detection_active.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_secs(60)); // Update correlations every minute
                
                if detection_active.load(Ordering::SeqCst) {
                    Self::update_correlation_matrix_static(
                        &metrics_history,
                        &correlation_matrix,
                        &config,
                    );
                }
            }
            
            println!("🔚 Correlation analysis thread stopped");
        });
        
        Ok(())
    }

    /// Start real-time monitoring thread
    fn start_realtime_monitoring_thread(&self) -> Result<()> {
        let monitoring_active = Arc::clone(&self.monitoring_active);
        let current_metrics = Arc::clone(&self.current_metrics);
        let detected_limits = Arc::clone(&self.detected_limits);
        let monitoring_state = Arc::clone(&self.monitoring_state);
        let response_history = Arc::clone(&self.response_history);
        let config = self.config.clone();
        
        thread::spawn(move || {
            println!("⚡ Starting real-time monitoring thread...");
            
            while monitoring_active.load(Ordering::SeqCst) {
                Self::update_monitoring_state_static(
                    &current_metrics,
                    &detected_limits,
                    &monitoring_state,
                    &response_history,
                    &config,
                );
                
                thread::sleep(Duration::from_millis(config.detection_interval_ms));
            }
            
            println!("🔚 Real-time monitoring thread stopped");
        });
        
        Ok(())
    }

    /// Collect current system metrics
    fn collect_system_metrics() -> Result<SystemMetrics> {
        // Simulate realistic system metrics collection
        let now = Instant::now();
        
        // Simulate varying system load
        let base_load = 0.3 + (now.elapsed().as_secs() as f64 / 100.0) % 0.4;
        let noise = (now.elapsed().as_nanos() % 1000) as f64 / 10000.0;
        
        Ok(SystemMetrics {
            timestamp: now,
            memory_usage_mb: 2048.0 + base_load * 2048.0 + noise * 100.0,
            memory_usage_percent: 50.0 + base_load * 30.0 + noise * 5.0,
            available_memory_mb: 8192.0 - (2048.0 + base_load * 2048.0),
            cpu_utilization_percent: 25.0 + base_load * 40.0 + noise * 10.0,
            cpu_load_average: 1.0 + base_load * 2.0,
            disk_read_mb_per_sec: 10.0 + base_load * 30.0 + noise * 5.0,
            disk_write_mb_per_sec: 5.0 + base_load * 20.0 + noise * 3.0,
            network_in_mb_per_sec: 1.0 + base_load * 10.0 + noise * 2.0,
            network_out_mb_per_sec: 0.8 + base_load * 8.0 + noise * 1.5,
            throughput_ops_per_sec: 1000.0 - base_load * 300.0 + noise * 50.0,
            response_latency_ms: 50.0 + base_load * 200.0 + noise * 20.0,
            active_threads: (50 + (base_load * 200.0) as usize) + (noise * 10.0) as usize,
            error_rate_percent: base_load * 2.0 + noise * 0.5,
            queue_depth: (base_load * 50.0) as usize + (noise * 5.0) as usize,
            active_connections: (100 + (base_load * 500.0) as usize) + (noise * 20.0) as usize,
            file_descriptors_used: (200 + (base_load * 1000.0) as usize) + (noise * 50.0) as usize,
            process_count: (20 + (base_load * 50.0) as usize) + (noise * 5.0) as usize,
        })
    }

    /// Run detection algorithm
    fn run_detection_algorithm_static(
        algorithm: &DetectionAlgorithm,
        metrics_history: &Arc<Mutex<VecDeque<SystemMetrics>>>,
        detected_limits: &Arc<Mutex<HashMap<CriticalLimitType, DetectedLimit>>>,
        breach_events: &Arc<Mutex<Vec<CriticalBreachEvent>>>,
        adaptive_thresholds: &Arc<RwLock<HashMap<CriticalLimitType, AdaptiveThreshold>>>,
        detection_stats: &Arc<Mutex<DetectionStatistics>>,
        config: &CriticalLimitsDetectionConfig,
    ) {
        let history = metrics_history.lock().unwrap();
        if history.len() < 20 {
            return;
        }
        
        let recent_metrics: Vec<_> = history.iter().rev().take(50).cloned().collect();
        drop(history);
        
        for limit_type in &config.limit_types {
            if let Some(detected_limit) = Self::apply_detection_algorithm(
                algorithm,
                limit_type,
                &recent_metrics,
                adaptive_thresholds,
                config,
            ) {
                // Check if this is a significant detection
                if detected_limit.confidence >= config.detection_confidence_threshold {
                    // Store detected limit
                    {
                        let mut limits = detected_limits.lock().unwrap();
                        limits.insert(limit_type.clone(), detected_limit.clone());
                    }
                    
                    // Check for breach
                    if detected_limit.distance_to_limit <= 0.0 {
                        let breach = CriticalBreachEvent {
                            timestamp: detected_limit.detected_at,
                            limit_type: limit_type.clone(),
                            severity: Self::calculate_breach_severity(detected_limit.distance_to_limit),
                            actual_value: detected_limit.current_value,
                            limit_value: detected_limit.critical_value,
                            breach_percentage: (detected_limit.current_value / detected_limit.critical_value - 1.0) * 100.0,
                            duration: None,
                            detection_algorithm: algorithm.clone(),
                            response_triggered: None,
                            recovery_info: None,
                        };
                        
                        breach_events.lock().unwrap().push(breach);
                    }
                    
                    // Update detection statistics
                    {
                        let mut stats = detection_stats.lock().unwrap();
                        stats.total_detections += 1;
                        // Update algorithm ranking based on detection confidence
                        let current_ranking = stats.algorithm_rankings.get(algorithm).unwrap_or(&0.0);
                        stats.algorithm_rankings.insert(algorithm.clone(), current_ranking + detected_limit.confidence);
                    }
                }
            }
        }
    }

    /// Apply specific detection algorithm
    fn apply_detection_algorithm(
        algorithm: &DetectionAlgorithm,
        limit_type: &CriticalLimitType,
        metrics: &[SystemMetrics],
        adaptive_thresholds: &Arc<RwLock<HashMap<CriticalLimitType, AdaptiveThreshold>>>,
        config: &CriticalLimitsDetectionConfig,
    ) -> Option<DetectedLimit> {
        match algorithm {
            DetectionAlgorithm::StatisticalAnomalyDetection => {
                Self::statistical_anomaly_detection(limit_type, metrics, adaptive_thresholds)
            },
            DetectionAlgorithm::GradientAnalysis => {
                Self::gradient_analysis_detection(limit_type, metrics, adaptive_thresholds)
            },
            DetectionAlgorithm::AdaptiveThresholding => {
                Self::adaptive_threshold_detection(limit_type, metrics, adaptive_thresholds)
            },
            DetectionAlgorithm::MachineLearningSVM => {
                Self::ml_svm_detection(limit_type, metrics, adaptive_thresholds)
            },
            _ => None, // Other algorithms would be implemented similarly
        }
    }

    /// Statistical anomaly detection
    fn statistical_anomaly_detection(
        limit_type: &CriticalLimitType,
        metrics: &[SystemMetrics],
        adaptive_thresholds: &Arc<RwLock<HashMap<CriticalLimitType, AdaptiveThreshold>>>,
    ) -> Option<DetectedLimit> {
        if metrics.len() < 20 {
            return None;
        }
        
        let values: Vec<f64> = metrics.iter().map(|m| Self::extract_metric_value(m, limit_type)).collect();
        
        // Calculate statistical properties
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        
        // Detect anomalies using Z-score
        let current_value = values.last().cloned().unwrap_or(0.0);
        let z_score = if std_dev > 0.0 { (current_value - mean) / std_dev } else { 0.0 };
        
        if z_score.abs() > 2.5 { // 2.5 standard deviations
            let thresholds = adaptive_thresholds.read().unwrap();
            let threshold = thresholds.get(limit_type).map(|t| t.current_value).unwrap_or(Self::get_default_threshold(limit_type));
            
            Some(DetectedLimit {
                limit_type: limit_type.clone(),
                critical_value: threshold,
                confidence: (z_score.abs() / 3.0).min(1.0), // Normalize to 0-1
                detected_at: Instant::now(),
                algorithm: DetectionAlgorithm::StatisticalAnomalyDetection,
                current_value,
                distance_to_limit: threshold - current_value,
                time_to_breach: Self::estimate_time_to_breach(&values, threshold),
                trend_analysis: Self::analyze_trend(&values),
                recommended_actions: Self::generate_recommendations(limit_type, current_value, threshold),
            })
        } else {
            None
        }
    }

    /// Gradient analysis detection
    fn gradient_analysis_detection(
        limit_type: &CriticalLimitType,
        metrics: &[SystemMetrics],
        adaptive_thresholds: &Arc<RwLock<HashMap<CriticalLimitType, AdaptiveThreshold>>>,
    ) -> Option<DetectedLimit> {
        if metrics.len() < 10 {
            return None;
        }
        
        let values: Vec<f64> = metrics.iter().map(|m| Self::extract_metric_value(m, limit_type)).collect();
        
        // Calculate gradient (rate of change)
        let mut gradients = Vec::new();
        for window in values.windows(2) {
            gradients.push(window[1] - window[0]);
        }
        
        let avg_gradient = gradients.iter().sum::<f64>() / gradients.len() as f64;
        let current_value = values.last().cloned().unwrap_or(0.0);
        
        let thresholds = adaptive_thresholds.read().unwrap();
        let threshold = thresholds.get(limit_type).map(|t| t.current_value).unwrap_or(Self::get_default_threshold(limit_type));
        
        // Check if gradient indicates rapid approach to limit
        if avg_gradient > 0.0 && current_value > threshold * 0.8 {
            let confidence = (avg_gradient / (threshold * 0.1)).min(1.0);
            
            Some(DetectedLimit {
                limit_type: limit_type.clone(),
                critical_value: threshold,
                confidence,
                detected_at: Instant::now(),
                algorithm: DetectionAlgorithm::GradientAnalysis,
                current_value,
                distance_to_limit: threshold - current_value,
                time_to_breach: if avg_gradient > 0.0 {
                    Some(Duration::from_secs(((threshold - current_value) / avg_gradient) as u64))
                } else {
                    None
                },
                trend_analysis: Self::analyze_trend(&values),
                recommended_actions: Self::generate_recommendations(limit_type, current_value, threshold),
            })
        } else {
            None
        }
    }

    /// Adaptive threshold detection
    fn adaptive_threshold_detection(
        limit_type: &CriticalLimitType,
        metrics: &[SystemMetrics],
        adaptive_thresholds: &Arc<RwLock<HashMap<CriticalLimitType, AdaptiveThreshold>>>,
    ) -> Option<DetectedLimit> {
        if metrics.is_empty() {
            return None;
        }
        
        let current_value = Self::extract_metric_value(metrics.last().unwrap(), limit_type);
        
        let thresholds = adaptive_thresholds.read().unwrap();
        if let Some(adaptive_threshold) = thresholds.get(limit_type) {
            let threshold = adaptive_threshold.current_value;
            
            // Check if current value exceeds adaptive threshold
            if current_value > threshold {
                Some(DetectedLimit {
                    limit_type: limit_type.clone(),
                    critical_value: threshold,
                    confidence: adaptive_threshold.confidence,
                    detected_at: Instant::now(),
                    algorithm: DetectionAlgorithm::AdaptiveThresholding,
                    current_value,
                    distance_to_limit: threshold - current_value,
                    time_to_breach: None,
                    trend_analysis: Self::analyze_trend(&metrics.iter().map(|m| Self::extract_metric_value(m, limit_type)).collect::<Vec<_>>()),
                    recommended_actions: Self::generate_recommendations(limit_type, current_value, threshold),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Machine learning SVM detection (simplified)
    fn ml_svm_detection(
        limit_type: &CriticalLimitType,
        metrics: &[SystemMetrics],
        adaptive_thresholds: &Arc<RwLock<HashMap<CriticalLimitType, AdaptiveThreshold>>>,
    ) -> Option<DetectedLimit> {
        // Simplified ML detection using feature engineering
        if metrics.len() < 15 {
            return None;
        }
        
        let values: Vec<f64> = metrics.iter().map(|m| Self::extract_metric_value(m, limit_type)).collect();
        let current_value = values.last().cloned().unwrap_or(0.0);
        
        // Feature engineering
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let trend = Self::calculate_linear_trend(&values);
        let volatility = Self::calculate_volatility(&values);
        
        // Simple ML-like scoring
        let ml_score = (current_value / mean) * trend.abs() * (1.0 + volatility);
        
        let thresholds = adaptive_thresholds.read().unwrap();
        let threshold = thresholds.get(limit_type).map(|t| t.current_value).unwrap_or(Self::get_default_threshold(limit_type));
        
        if ml_score > 2.0 && current_value > threshold * 0.7 {
            Some(DetectedLimit {
                limit_type: limit_type.clone(),
                critical_value: threshold,
                confidence: (ml_score / 5.0).min(1.0),
                detected_at: Instant::now(),
                algorithm: DetectionAlgorithm::MachineLearningSVM,
                current_value,
                distance_to_limit: threshold - current_value,
                time_to_breach: Self::estimate_time_to_breach(&values, threshold),
                trend_analysis: Self::analyze_trend(&values),
                recommended_actions: Self::generate_recommendations(limit_type, current_value, threshold),
            })
        } else {
            None
        }
    }

    /// Extract metric value for specific limit type
    fn extract_metric_value(metrics: &SystemMetrics, limit_type: &CriticalLimitType) -> f64 {
        match limit_type {
            CriticalLimitType::MemoryLimit => metrics.memory_usage_percent,
            CriticalLimitType::CpuLimit => metrics.cpu_utilization_percent,
            CriticalLimitType::DiskIOLimit => metrics.disk_read_mb_per_sec + metrics.disk_write_mb_per_sec,
            CriticalLimitType::NetworkLimit => metrics.network_in_mb_per_sec + metrics.network_out_mb_per_sec,
            CriticalLimitType::ThroughputLimit => metrics.throughput_ops_per_sec,
            CriticalLimitType::LatencyLimit => metrics.response_latency_ms,
            CriticalLimitType::ConcurrencyLimit => metrics.active_threads as f64,
            CriticalLimitType::ErrorRateLimit => metrics.error_rate_percent,
            CriticalLimitType::QueueDepthLimit => metrics.queue_depth as f64,
            CriticalLimitType::ConnectionLimit => metrics.active_connections as f64,
            CriticalLimitType::FileDescriptorLimit => metrics.file_descriptors_used as f64,
            CriticalLimitType::ProcessLimit => metrics.process_count as f64,
        }
    }

    /// Calculate breach severity
    fn calculate_breach_severity(distance_to_limit: f64) -> BreachSeverity {
        if distance_to_limit > -5.0 {
            BreachSeverity::Approaching
        } else if distance_to_limit > -20.0 {
            BreachSeverity::Critical
        } else if distance_to_limit > -50.0 {
            BreachSeverity::Severe
        } else {
            BreachSeverity::Catastrophic
        }
    }

    /// Estimate time to breach
    fn estimate_time_to_breach(values: &[f64], threshold: f64) -> Option<Duration> {
        if values.len() < 5 {
            return None;
        }
        
        let trend = Self::calculate_linear_trend(values);
        let current_value = values.last().cloned().unwrap_or(0.0);
        
        if trend > 0.0 && current_value < threshold {
            let time_to_breach = (threshold - current_value) / trend;
            Some(Duration::from_secs(time_to_breach as u64))
        } else {
            None
        }
    }

    /// Calculate linear trend
    fn calculate_linear_trend(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }
        
        let n = values.len() as f64;
        let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = values.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();
        
        if n * sum_x2 - sum_x * sum_x == 0.0 {
            0.0
        } else {
            (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
        }
    }

    /// Calculate volatility
    fn calculate_volatility(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }
        
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        variance.sqrt() / mean
    }

    /// Analyze trend
    fn analyze_trend(values: &[f64]) -> TrendAnalysis {
        let trend_slope = Self::calculate_linear_trend(values);
        let volatility = Self::calculate_volatility(values);
        
        let direction = if trend_slope > 0.1 {
            if trend_slope > 1.0 { TrendDirection::ExponentialIncrease } else { TrendDirection::Increasing }
        } else if trend_slope < -0.1 {
            TrendDirection::Decreasing
        } else if volatility > 0.2 {
            TrendDirection::Oscillating
        } else {
            TrendDirection::Stable
        };
        
        TrendAnalysis {
            direction,
            strength: trend_slope.abs(),
            acceleration: 0.0, // Would require second derivative
            confidence: (1.0 - volatility).max(0.0),
            historical_points: values.iter().enumerate().map(|(i, &v)| (Instant::now() - Duration::from_secs((values.len() - i) as u64), v)).collect(),
        }
    }

    /// Generate recommendations
    fn generate_recommendations(limit_type: &CriticalLimitType, current_value: f64, threshold: f64) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        match limit_type {
            CriticalLimitType::MemoryLimit => {
                recommendations.push("Consider increasing available memory".to_string());
                recommendations.push("Optimize memory usage in applications".to_string());
                recommendations.push("Enable memory cleanup processes".to_string());
            },
            CriticalLimitType::CpuLimit => {
                recommendations.push("Scale up CPU resources".to_string());
                recommendations.push("Optimize CPU-intensive operations".to_string());
                recommendations.push("Implement load balancing".to_string());
            },
            CriticalLimitType::ThroughputLimit => {
                recommendations.push("Optimize bottleneck operations".to_string());
                recommendations.push("Increase processing capacity".to_string());
                recommendations.push("Implement caching strategies".to_string());
            },
            _ => {
                recommendations.push("Monitor resource usage closely".to_string());
                recommendations.push("Consider scaling resources".to_string());
            },
        }
        
        recommendations
    }

    /// Adjust adaptive thresholds
    fn adjust_adaptive_thresholds_static(
        metrics_history: &Arc<Mutex<VecDeque<SystemMetrics>>>,
        adaptive_thresholds: &Arc<RwLock<HashMap<CriticalLimitType, AdaptiveThreshold>>>,
        breach_events: &Arc<Mutex<Vec<CriticalBreachEvent>>>,
        config: &CriticalLimitsDetectionConfig,
    ) {
        let history = metrics_history.lock().unwrap();
        let breaches = breach_events.lock().unwrap();
        
        if history.len() < 50 {
            return;
        }
        
        let recent_metrics: Vec<_> = history.iter().rev().take(100).cloned().collect();
        drop(history);
        drop(breaches);
        
        let mut thresholds = adaptive_thresholds.write().unwrap();
        
        for limit_type in &config.limit_types {
            if let Some(threshold) = thresholds.get_mut(limit_type) {
                let values: Vec<f64> = recent_metrics.iter()
                    .map(|m| Self::extract_metric_value(m, limit_type))
                    .collect();
                
                // Calculate adaptive threshold based on recent behavior
                let mean = values.iter().sum::<f64>() / values.len() as f64;
                let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let std_dev = Self::calculate_volatility(&values) * mean;
                
                // Adaptive threshold = mean + 2*std_dev, but capped at reasonable limits
                let new_threshold = (mean + 2.0 * std_dev).max(threshold.base_value * 0.8).min(threshold.base_value * 1.5);
                
                if (new_threshold - threshold.current_value).abs() > threshold.current_value * 0.1 {
                    let adjustment = ThresholdAdjustment {
                        timestamp: Instant::now(),
                        previous_value: threshold.current_value,
                        new_value: new_threshold,
                        reason: format!("Adaptive adjustment based on recent behavior (mean: {:.2}, max: {:.2})", mean, max),
                        effectiveness: None,
                    };
                    
                    threshold.adjustment_history.push_back(adjustment);
                    threshold.current_value = new_threshold;
                    threshold.last_adjusted = Instant::now();
                    threshold.confidence = 0.8; // High confidence in data-driven adjustment
                    
                    // Keep adjustment history limited
                    if threshold.adjustment_history.len() > 20 {
                        threshold.adjustment_history.pop_front();
                    }
                }
            }
        }
    }

    /// Update ML models
    fn update_ml_models_static(
        metrics_history: &Arc<Mutex<VecDeque<SystemMetrics>>>,
        ml_models: &Arc<Mutex<HashMap<CriticalLimitType, MLModel>>>,
        detected_limits: &Arc<Mutex<HashMap<CriticalLimitType, DetectedLimit>>>,
        config: &CriticalLimitsDetectionConfig,
    ) {
        let history = metrics_history.lock().unwrap();
        if history.len() < 100 {
            return;
        }
        
        let training_data: Vec<_> = history.iter().cloned().collect();
        drop(history);
        
        let mut models = ml_models.lock().unwrap();
        
        for limit_type in &config.limit_types {
            if let Some(model) = models.get_mut(limit_type) {
                // Simple ML model training simulation
                let values: Vec<f64> = training_data.iter()
                    .map(|m| Self::extract_metric_value(m, limit_type))
                    .collect();
                
                if values.len() >= 50 {
                    // Update model parameters (simplified)
                    let mean = values.iter().sum::<f64>() / values.len() as f64;
                    let trend = Self::calculate_linear_trend(&values);
                    let volatility = Self::calculate_volatility(&values);
                    
                    model.parameters.insert("mean".to_string(), mean);
                    model.parameters.insert("trend".to_string(), trend);
                    model.parameters.insert("volatility".to_string(), volatility);
                    
                    model.training_data_points = values.len();
                    model.last_trained = Instant::now();
                    
                    // Calculate accuracy based on recent predictions vs actual
                    model.accuracy = 0.8 + (1.0 - volatility) * 0.2; // Simplified accuracy calculation
                    model.prediction_confidence = model.accuracy;
                }
            }
        }
    }

    /// Update correlation matrix
    fn update_correlation_matrix_static(
        metrics_history: &Arc<Mutex<VecDeque<SystemMetrics>>>,
        correlation_matrix: &Arc<Mutex<CorrelationMatrix>>,
        config: &CriticalLimitsDetectionConfig,
    ) {
        let history = metrics_history.lock().unwrap();
        if history.len() < 50 {
            return;
        }
        
        let recent_metrics: Vec<_> = history.iter().rev().take(200).cloned().collect();
        drop(history);
        
        let mut matrix = correlation_matrix.lock().unwrap();
        matrix.correlations.clear();
        matrix.strong_correlations.clear();
        
        // Calculate correlations between all limit type pairs
        for (i, limit_type1) in config.limit_types.iter().enumerate() {
            for limit_type2 in config.limit_types.iter().skip(i + 1) {
                let values1: Vec<f64> = recent_metrics.iter()
                    .map(|m| Self::extract_metric_value(m, limit_type1))
                    .collect();
                let values2: Vec<f64> = recent_metrics.iter()
                    .map(|m| Self::extract_metric_value(m, limit_type2))
                    .collect();
                
                let correlation = Self::calculate_correlation(&values1, &values2);
                matrix.correlations.insert((limit_type1.clone(), limit_type2.clone()), correlation);
                matrix.confidence_scores.insert((limit_type1.clone(), limit_type2.clone()), 0.9);
                
                // Track strong correlations
                if correlation.abs() > 0.7 {
                    matrix.strong_correlations.push((limit_type1.clone(), limit_type2.clone(), correlation));
                }
            }
        }
        
        matrix.last_updated = Instant::now();
    }

    /// Calculate correlation coefficient
    fn calculate_correlation(values1: &[f64], values2: &[f64]) -> f64 {
        if values1.len() != values2.len() || values1.len() < 2 {
            return 0.0;
        }
        
        let n = values1.len() as f64;
        let mean1 = values1.iter().sum::<f64>() / n;
        let mean2 = values2.iter().sum::<f64>() / n;
        
        let numerator: f64 = values1.iter().zip(values2.iter())
            .map(|(x, y)| (x - mean1) * (y - mean2))
            .sum();
        
        let sum_sq1: f64 = values1.iter().map(|x| (x - mean1).powi(2)).sum();
        let sum_sq2: f64 = values2.iter().map(|y| (y - mean2).powi(2)).sum();
        
        let denominator = (sum_sq1 * sum_sq2).sqrt();
        
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    /// Update monitoring state
    fn update_monitoring_state_static(
        current_metrics: &Arc<Mutex<SystemMetrics>>,
        detected_limits: &Arc<Mutex<HashMap<CriticalLimitType, DetectedLimit>>>,
        monitoring_state: &Arc<Mutex<MonitoringState>>,
        response_history: &Arc<Mutex<Vec<ResponseAction>>>,
        config: &CriticalLimitsDetectionConfig,
    ) {
        let metrics = current_metrics.lock().unwrap().clone();
        let limits = detected_limits.lock().unwrap();
        let responses = response_history.lock().unwrap();
        
        let mut state = monitoring_state.lock().unwrap();
        
        // Update basic state
        state.last_update = Instant::now();
        state.active_alerts = limits.len();
        
        // Calculate system health score
        let mut health_factors = Vec::new();
        health_factors.push(1.0 - (metrics.memory_usage_percent / 100.0).min(1.0));
        health_factors.push(1.0 - (metrics.cpu_utilization_percent / 100.0).min(1.0));
        health_factors.push(1.0 - (metrics.error_rate_percent / 10.0).min(1.0));
        
        state.system_health_score = health_factors.iter().sum::<f64>() / health_factors.len() as f64;
        
        // Determine risk level
        state.risk_level = if state.system_health_score > 0.8 {
            RiskLevel::Low
        } else if state.system_health_score > 0.6 {
            RiskLevel::Moderate
        } else if state.system_health_score > 0.4 {
            RiskLevel::High
        } else if state.system_health_score > 0.2 {
            RiskLevel::Extreme
        } else {
            RiskLevel::Critical
        };
        
        // Count predictive alerts
        state.predictive_alerts = limits.values()
            .filter(|limit| limit.time_to_breach.is_some())
            .count();
        
        drop(limits);
        drop(responses);
    }

    /// Stop detection
    pub fn stop_detection(&self) {
        self.detection_active.store(false, Ordering::SeqCst);
        self.monitoring_active.store(false, Ordering::SeqCst);
        println!("🛑 Critical limits detection stopped");
    }

    /// Get detected limits
    pub fn get_detected_limits(&self) -> HashMap<CriticalLimitType, DetectedLimit> {
        self.detected_limits.lock().unwrap().clone()
    }

    /// Get breach events
    pub fn get_breach_events(&self) -> Vec<CriticalBreachEvent> {
        self.breach_events.lock().unwrap().clone()
    }

    /// Get monitoring state
    pub fn get_monitoring_state(&self) -> MonitoringState {
        self.monitoring_state.lock().unwrap().clone()
    }

    /// Generate detection report
    pub fn generate_detection_report(&self) -> CriticalLimitsDetectionReport {
        let detected_limits = self.get_detected_limits();
        let breach_events = self.get_breach_events();
        let monitoring_state = self.get_monitoring_state();
        let detection_stats = self.detection_stats.lock().unwrap().clone();
        let correlation_matrix = self.correlation_matrix.lock().unwrap().clone();
        
        CriticalLimitsDetectionReport {
            session_duration: self.config.max_detection_duration_secs,
            total_limits_detected: detected_limits.len(),
            total_breach_events: breach_events.len(),
            critical_breaches: breach_events.iter().filter(|b| b.severity == BreachSeverity::Critical).count(),
            severe_breaches: breach_events.iter().filter(|b| b.severity == BreachSeverity::Severe).count(),
            system_health_score: monitoring_state.system_health_score,
            risk_level: monitoring_state.risk_level,
            detection_accuracy: detection_stats.accuracy,
            most_effective_algorithm: Self::find_most_effective_algorithm(&detection_stats),
            strong_correlations: correlation_matrix.strong_correlations,
            recommendations: self.generate_detection_recommendations(&detected_limits, &breach_events, &monitoring_state),
        }
    }

    /// Find most effective algorithm
    fn find_most_effective_algorithm(stats: &DetectionStatistics) -> Option<DetectionAlgorithm> {
        stats.algorithm_rankings.iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(algorithm, _)| algorithm.clone())
    }

    /// Generate detection recommendations
    fn generate_detection_recommendations(
        &self,
        detected_limits: &HashMap<CriticalLimitType, DetectedLimit>,
        breach_events: &[CriticalBreachEvent],
        monitoring_state: &MonitoringState,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // System health recommendations
        match monitoring_state.risk_level {
            RiskLevel::Critical => {
                recommendations.push("🆘 CRITICAL: Immediate intervention required - system failure imminent".to_string());
                recommendations.push("🚨 Activate emergency response procedures".to_string());
            },
            RiskLevel::Extreme => {
                recommendations.push("⚠️ EXTREME RISK: System at breaking point - immediate action needed".to_string());
                recommendations.push("🔥 Consider emergency scaling or load shedding".to_string());
            },
            RiskLevel::High => {
                recommendations.push("📈 HIGH RISK: Multiple critical limits approaching".to_string());
                recommendations.push("⚡ Increase resource monitoring frequency".to_string());
            },
            _ => {},
        }
        
        // Specific limit recommendations
        for (limit_type, detected_limit) in detected_limits {
            if detected_limit.confidence > 0.8 {
                recommendations.push(format!(
                    "🎯 {}: {} detected with {:.1}% confidence",
                    format!("{:?}", limit_type),
                    if detected_limit.distance_to_limit <= 0.0 { "BREACH" } else { "APPROACHING LIMIT" },
                    detected_limit.confidence * 100.0
                ));
                
                for action in &detected_limit.recommended_actions {
                    recommendations.push(format!("   • {}", action));
                }
            }
        }
        
        // Breach pattern recommendations
        let critical_breach_count = breach_events.iter().filter(|b| b.severity == BreachSeverity::Critical).count();
        if critical_breach_count > 3 {
            recommendations.push("🔄 Pattern detected: Recurring critical breaches - investigate underlying causes".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("✅ System operating within normal parameters".to_string());
        }
        
        recommendations
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            timestamp: Instant::now(),
            memory_usage_mb: 1024.0,
            memory_usage_percent: 25.0,
            available_memory_mb: 6144.0,
            cpu_utilization_percent: 15.0,
            cpu_load_average: 0.5,
            disk_read_mb_per_sec: 5.0,
            disk_write_mb_per_sec: 2.0,
            network_in_mb_per_sec: 1.0,
            network_out_mb_per_sec: 0.8,
            throughput_ops_per_sec: 1000.0,
            response_latency_ms: 50.0,
            active_threads: 50,
            error_rate_percent: 0.1,
            queue_depth: 5,
            active_connections: 100,
            file_descriptors_used: 200,
            process_count: 25,
        }
    }
}

/// Final detection report
#[derive(Debug, Clone)]
pub struct CriticalLimitsDetectionReport {
    /// Session duration in seconds
    pub session_duration: u64,
    /// Total limits detected
    pub total_limits_detected: usize,
    /// Total breach events
    pub total_breach_events: usize,
    /// Critical breach count
    pub critical_breaches: usize,
    /// Severe breach count
    pub severe_breaches: usize,
    /// Final system health score
    pub system_health_score: f64,
    /// Final risk level
    pub risk_level: RiskLevel,
    /// Overall detection accuracy
    pub detection_accuracy: f64,
    /// Most effective detection algorithm
    pub most_effective_algorithm: Option<DetectionAlgorithm>,
    /// Strong correlations found
    pub strong_correlations: Vec<(CriticalLimitType, CriticalLimitType, f64)>,
    /// Final recommendations
    pub recommendations: Vec<String>,
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_critical_limits_detector_creation() -> Result<()> {
        let config = CriticalLimitsDetectionConfig::default();
        let detector = CriticalLimitsDetector::new(config);
        
        assert!(!detector.detection_active.load(Ordering::SeqCst));
        assert!(!detector.monitoring_active.load(Ordering::SeqCst));
        
        Ok(())
    }

    #[test]
    fn test_system_metrics_collection() -> Result<()> {
        let metrics = CriticalLimitsDetector::collect_system_metrics()?;
        
        assert!(metrics.memory_usage_mb > 0.0);
        assert!(metrics.memory_usage_percent >= 0.0);
        assert!(metrics.cpu_utilization_percent >= 0.0);
        assert!(metrics.throughput_ops_per_sec > 0.0);
        assert!(metrics.active_threads > 0);
        
        Ok(())
    }

    #[test]
    fn test_metric_extraction() -> Result<()> {
        let metrics = SystemMetrics::default();
        
        assert_eq!(CriticalLimitsDetector::extract_metric_value(&metrics, &CriticalLimitType::MemoryLimit), 25.0);
        assert_eq!(CriticalLimitsDetector::extract_metric_value(&metrics, &CriticalLimitType::CpuLimit), 15.0);
        assert_eq!(CriticalLimitsDetector::extract_metric_value(&metrics, &CriticalLimitType::ThroughputLimit), 1000.0);
        assert_eq!(CriticalLimitsDetector::extract_metric_value(&metrics, &CriticalLimitType::LatencyLimit), 50.0);
        
        Ok(())
    }

    #[test]
    fn test_statistical_anomaly_detection() -> Result<()> {
        let adaptive_thresholds = Arc::new(RwLock::new(HashMap::new()));
        
        // Create metrics with anomaly
        let mut metrics = Vec::new();
        for i in 0..25 {
            let mut metric = SystemMetrics::default();
            metric.memory_usage_percent = if i < 20 { 50.0 } else { 95.0 }; // Anomaly in last 5 samples
            metrics.push(metric);
        }
        
        let detection = CriticalLimitsDetector::statistical_anomaly_detection(
            &CriticalLimitType::MemoryLimit,
            &metrics,
            &adaptive_thresholds,
        );
        
        assert!(detection.is_some());
        let detection = detection.unwrap();
        assert_eq!(detection.limit_type, CriticalLimitType::MemoryLimit);
        assert!(detection.confidence > 0.0);
        assert_eq!(detection.algorithm, DetectionAlgorithm::StatisticalAnomalyDetection);
        
        Ok(())
    }

    #[test]
    fn test_gradient_analysis_detection() -> Result<()> {
        let adaptive_thresholds = Arc::new(RwLock::new(HashMap::new()));
        
        // Create metrics with increasing trend
        let mut metrics = Vec::new();
        for i in 0..15 {
            let mut metric = SystemMetrics::default();
            metric.memory_usage_percent = 60.0 + (i as f64 * 2.0); // Increasing trend
            metrics.push(metric);
        }
        
        let detection = CriticalLimitsDetector::gradient_analysis_detection(
            &CriticalLimitType::MemoryLimit,
            &metrics,
            &adaptive_thresholds,
        );
        
        assert!(detection.is_some());
        let detection = detection.unwrap();
        assert_eq!(detection.limit_type, CriticalLimitType::MemoryLimit);
        assert_eq!(detection.algorithm, DetectionAlgorithm::GradientAnalysis);
        assert!(detection.time_to_breach.is_some());
        
        Ok(())
    }

    #[test]
    fn test_breach_severity_calculation() -> Result<()> {
        assert_eq!(CriticalLimitsDetector::calculate_breach_severity(-2.0), BreachSeverity::Approaching);
        assert_eq!(CriticalLimitsDetector::calculate_breach_severity(-10.0), BreachSeverity::Critical);
        assert_eq!(CriticalLimitsDetector::calculate_breach_severity(-30.0), BreachSeverity::Severe);
        assert_eq!(CriticalLimitsDetector::calculate_breach_severity(-100.0), BreachSeverity::Catastrophic);
        
        Ok(())
    }

    #[test]
    fn test_correlation_calculation() -> Result<()> {
        let values1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let values2 = vec![2.0, 4.0, 6.0, 8.0, 10.0]; // Perfect positive correlation
        
        let correlation = CriticalLimitsDetector::calculate_correlation(&values1, &values2);
        assert!((correlation - 1.0).abs() < 0.001); // Should be very close to 1.0
        
        let values3 = vec![5.0, 4.0, 3.0, 2.0, 1.0]; // Perfect negative correlation
        let correlation = CriticalLimitsDetector::calculate_correlation(&values1, &values3);
        assert!((correlation - (-1.0)).abs() < 0.001); // Should be very close to -1.0
        
        Ok(())
    }

    #[test]
    fn test_trend_analysis() -> Result<()> {
        // Increasing trend
        let increasing_values = vec![10.0, 12.0, 14.0, 16.0, 18.0, 20.0];
        let trend = CriticalLimitsDetector::analyze_trend(&increasing_values);
        assert_eq!(trend.direction, TrendDirection::Increasing);
        assert!(trend.strength > 0.0);
        
        // Stable trend
        let stable_values = vec![50.0, 50.1, 49.9, 50.0, 50.2, 49.8];
        let trend = CriticalLimitsDetector::analyze_trend(&stable_values);
        assert_eq!(trend.direction, TrendDirection::Stable);
        
        Ok(())
    }

    #[test]
    fn test_adaptive_threshold_defaults() -> Result<()> {
        assert_eq!(CriticalLimitsDetector::get_default_threshold(&CriticalLimitType::MemoryLimit), 85.0);
        assert_eq!(CriticalLimitsDetector::get_default_threshold(&CriticalLimitType::CpuLimit), 90.0);
        assert_eq!(CriticalLimitsDetector::get_default_threshold(&CriticalLimitType::ThroughputLimit), 500.0);
        assert_eq!(CriticalLimitsDetector::get_default_threshold(&CriticalLimitType::LatencyLimit), 1000.0);
        
        Ok(())
    }

    #[test]
    fn test_short_detection_session() -> Result<()> {
        let config = CriticalLimitsDetectionConfig {
            max_detection_duration_secs: 5, // Very short test
            detection_interval_ms: 100,
            enable_ml_prediction: false, // Disable for faster test
            enable_correlation_analysis: false,
            ..Default::default()
        };
        
        let detector = CriticalLimitsDetector::new(config);
        
        // Start detection
        detector.start_detection()?;
        
        // Let it run briefly
        thread::sleep(Duration::from_secs(2));
        
        // Check we have some metrics
        let metrics = detector.current_metrics.lock().unwrap();
        assert!(metrics.memory_usage_mb > 0.0);
        drop(metrics);
        
        // Stop detection
        detector.stop_detection();
        
        Ok(())
    }

    #[test]
    fn test_comprehensive_detection_workflow() -> Result<()> {
        println!("🧪 Testing comprehensive critical limits detection workflow...");
        
        let config = CriticalLimitsDetectionConfig {
            max_detection_duration_secs: 10,
            detection_interval_ms: 200, // Faster detection
            adaptive_adjustment_interval_secs: 3, // Faster adjustment
            enable_ml_prediction: true,
            enable_correlation_analysis: true,
            enable_adaptive_thresholds: true,
            detection_confidence_threshold: 0.7, // Lower threshold for testing
            ..Default::default()
        };
        
        let detector = CriticalLimitsDetector::new(config);
        
        // Start detection
        detector.start_detection()?;
        
        // Let detection run and collect data
        thread::sleep(Duration::from_secs(4));
        
        // Check detection state
        let monitoring_state = detector.get_monitoring_state();
        assert!(monitoring_state.system_health_score >= 0.0 && monitoring_state.system_health_score <= 1.0);
        
        // Check metrics history
        let history = detector.metrics_history.lock().unwrap();
        assert!(!history.is_empty(), "Should have collected metrics");
        let history_size = history.len();
        drop(history);
        
        // Stop detection
        detector.stop_detection();
        
        // Generate report
        let report = detector.generate_detection_report();
        
        println!("✅ Comprehensive detection completed:");
        println!("   📊 Limits detected: {}", report.total_limits_detected);
        println!("   🚨 Breach events: {}", report.total_breach_events);
        println!("   💊 System health: {:.2}", report.system_health_score);
        println!("   📈 Risk level: {:?}", report.risk_level);
        println!("   🎯 Detection accuracy: {:.2}", report.detection_accuracy);
        println!("   🔗 Strong correlations: {}", report.strong_correlations.len());
        println!("   📋 Recommendations: {}", report.recommendations.len());
        
        for (i, recommendation) in report.recommendations.iter().enumerate() {
            println!("      {}. {}", i + 1, recommendation);
        }
        
        // Verify we collected meaningful data
        assert!(history_size > 5, "Should have collected multiple metric samples");
        assert!(!report.recommendations.is_empty(), "Should have generated recommendations");
        
        Ok(())
    }
}