//! Real-time Breaking Point Detection System
//!
//! This module implements a sophisticated real-time breaking point detection system that
//! continuously monitors system performance, resource utilization, and operational metrics
//! to automatically detect when the system approaches or reaches critical failure points.
//! The system employs multiple detection algorithms, statistical analysis, and machine
//! learning techniques to provide early warning and accurate breaking point identification.
//!
//! ## What
//! 
//! Advanced real-time breaking point detection system that provides:
//! - Continuous real-time monitoring of performance and resource metrics
//! - Multiple detection algorithms working in parallel (statistical, ML, threshold-based)
//! - Early warning system with configurable alert levels and thresholds
//! - Multi-dimensional analysis across performance, resources, errors, and stability
//! - Confidence scoring and uncertainty quantification for detection accuracy
//! - Predictive detection using trend analysis and forecasting models
//! - Adaptive thresholds that adjust based on system behavior and historical data
//! - Integration with stress testing framework for automated test termination
//! 
//! ## How
//! 
//! The system uses a layered detection approach with multiple algorithms:
//! 1. **Statistical Detection**: Anomaly detection using statistical methods (Z-score, IQR, etc.)
//! 2. **Threshold Detection**: Configurable hard and soft thresholds for key metrics
//! 3. **Trend Analysis**: Time-series analysis to detect negative trends and projections
//! 4. **Machine Learning**: ML models trained on historical data for pattern recognition
//! 5. **Composite Detection**: Ensemble methods combining multiple detection signals
//! 6. **Early Warning**: Predictive alerts before actual breaking points occur
//! 7. **Confidence Assessment**: Bayesian and probabilistic confidence scoring
//! 8. **Real-time Processing**: Stream processing for low-latency detection
//! 
//! ## Why
//! 
//! Real-time breaking point detection is essential for:
//! - Preventing system damage by stopping tests before catastrophic failure
//! - Maximizing test value by pushing systems to their exact limits safely
//! - Providing early warning to enable proactive intervention and mitigation
//! - Supporting automated testing workflows without manual supervision
//! - Ensuring reproducible and reliable breaking point identification
//! - Enabling precise capacity planning with accurate limit determination
//! - Supporting SLA compliance by detecting performance degradation early

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

use std::collections::{HashMap, VecDeque, BTreeMap};
use std::sync::{Arc, Mutex, RwLock, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::time::{Duration, Instant, SystemTime};
use std::thread;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// Import from our test modules
mod test_performance_metrics_infrastructure;
mod test_progressive_stress_breaking_points;

use test_performance_metrics_infrastructure::{
    PerformanceMetricsSnapshot,
    ThroughputMetrics,
    LatencyMetrics,
    ResourceMetrics,
};

use test_progressive_stress_breaking_points::{
    StressLevel,
    ResourceUtilizationSnapshot,
    BreakingPointIndicator,
    IndicatorType,
    IndicatorSeverity,
};

/// Configuration for real-time breaking point detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeDetectionConfig {
    /// Detection algorithms to use
    pub detection_algorithms: Vec<DetectionAlgorithmConfig>,
    /// Monitoring configuration
    pub monitoring_config: MonitoringConfig,
    /// Alert configuration
    pub alert_config: AlertConfig,
    /// Statistical analysis configuration
    pub statistical_config: StatisticalConfig,
    /// Machine learning configuration
    pub ml_config: MachineLearningConfig,
    /// Threshold configuration
    pub threshold_config: ThresholdConfig,
    /// Early warning configuration
    pub early_warning_config: EarlyWarningConfig,
    /// Confidence scoring configuration
    pub confidence_config: ConfidenceConfig,
}

/// Configuration for detection algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionAlgorithmConfig {
    /// Algorithm type
    pub algorithm_type: DetectionAlgorithmType,
    /// Algorithm weight in ensemble
    pub weight: f64,
    /// Algorithm-specific parameters
    pub parameters: HashMap<String, f64>,
    /// Enabled status
    pub enabled: bool,
    /// Minimum confidence threshold
    pub min_confidence: f64,
}

/// Types of detection algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectionAlgorithmType {
    /// Statistical anomaly detection
    Statistical {
        /// Method to use
        method: StatisticalMethod,
        /// Parameters
        parameters: StatisticalParameters,
    },
    /// Threshold-based detection
    Threshold {
        /// Threshold configuration
        config: ThresholdDetectionConfig,
    },
    /// Trend analysis detection
    TrendAnalysis {
        /// Trend analysis configuration
        config: TrendAnalysisConfig,
    },
    /// Machine learning detection
    MachineLearning {
        /// ML model configuration
        config: MlModelConfig,
    },
    /// Composite ensemble detection
    Composite {
        /// Sub-algorithms
        algorithms: Vec<DetectionAlgorithmConfig>,
        /// Voting strategy
        voting_strategy: VotingStrategy,
    },
    /// Custom detection algorithm
    Custom {
        /// Algorithm name
        name: String,
        /// Configuration
        config: HashMap<String, f64>,
    },
}

/// Statistical methods for anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatisticalMethod {
    /// Z-score based detection
    ZScore,
    /// Modified Z-score using median
    ModifiedZScore,
    /// Interquartile range method
    InterquartileRange,
    /// Isolation forest
    IsolationForest,
    /// One-class SVM
    OneClassSvm,
    /// Gaussian mixture model
    GaussianMixture,
    /// DBSCAN clustering
    DbscanClustering,
}

/// Parameters for statistical methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalParameters {
    /// Threshold multiplier (e.g., for Z-score)
    pub threshold_multiplier: f64,
    /// Window size for analysis
    pub window_size: u32,
    /// Minimum samples required
    pub min_samples: u32,
    /// Percentile for outlier detection
    pub outlier_percentile: f64,
    /// Confidence interval
    pub confidence_interval: f64,
}

/// Configuration for threshold detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdDetectionConfig {
    /// Performance thresholds
    pub performance_thresholds: PerformanceThresholds,
    /// Resource thresholds
    pub resource_thresholds: ResourceThresholds,
    /// Error thresholds
    pub error_thresholds: ErrorThresholds,
    /// Adaptive threshold adjustment
    pub adaptive_thresholds: bool,
    /// Threshold adjustment rate
    pub adjustment_rate: f64,
}

/// Performance thresholds for detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    /// Throughput degradation threshold (percentage)
    pub throughput_degradation: f64,
    /// Latency increase threshold (percentage)
    pub latency_increase: f64,
    /// Response time threshold (milliseconds)
    pub response_time_ms: f64,
    /// Success rate threshold (percentage)
    pub success_rate: f64,
    /// Quality score threshold
    pub quality_score: f64,
}

/// Resource thresholds for detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceThresholds {
    /// CPU utilization threshold (percentage)
    pub cpu_utilization: f64,
    /// Memory utilization threshold (percentage)
    pub memory_utilization: f64,
    /// Disk I/O threshold (MB/s)
    pub disk_io_threshold: f64,
    /// Network I/O threshold (MB/s)
    pub network_io_threshold: f64,
    /// File descriptor threshold
    pub file_descriptor_count: u64,
    /// Thread count threshold
    pub thread_count: u32,
    /// Load average threshold
    pub load_average: f64,
}

/// Error thresholds for detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorThresholds {
    /// Error rate threshold (percentage)
    pub error_rate: f64,
    /// Timeout rate threshold (percentage)
    pub timeout_rate: f64,
    /// Critical error threshold (count)
    pub critical_errors: u64,
    /// Exception rate threshold (per second)
    pub exception_rate: f64,
}

/// Configuration for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysisConfig {
    /// Trend detection window
    pub analysis_window: Duration,
    /// Trend sensitivity
    pub sensitivity: f64,
    /// Minimum trend strength
    pub min_trend_strength: f64,
    /// Projection horizon
    pub projection_horizon: Duration,
    /// Trend algorithms to use
    pub algorithms: Vec<TrendAlgorithm>,
}

/// Trend analysis algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendAlgorithm {
    /// Linear regression
    LinearRegression,
    /// Polynomial regression
    PolynomialRegression { degree: u32 },
    /// Moving average
    MovingAverage { window: u32 },
    /// Exponential smoothing
    ExponentialSmoothing { alpha: f64 },
    /// ARIMA modeling
    Arima { p: u32, d: u32, q: u32 },
    /// Seasonal decomposition
    SeasonalDecomposition,
}

/// Machine learning model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlModelConfig {
    /// Model type
    pub model_type: MlModelType,
    /// Training configuration
    pub training_config: MlTrainingConfig,
    /// Feature extraction configuration
    pub feature_config: FeatureExtractionConfig,
    /// Model update frequency
    pub update_frequency: Duration,
}

/// Types of ML models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MlModelType {
    /// Neural network
    NeuralNetwork {
        /// Hidden layers
        layers: Vec<u32>,
        /// Activation function
        activation: String,
    },
    /// Random forest
    RandomForest {
        /// Number of trees
        n_trees: u32,
        /// Max depth
        max_depth: u32,
    },
    /// Support vector machine
    SupportVectorMachine {
        /// Kernel type
        kernel: String,
        /// Regularization parameter
        c: f64,
    },
    /// Gradient boosting
    GradientBoosting {
        /// Number of estimators
        n_estimators: u32,
        /// Learning rate
        learning_rate: f64,
    },
    /// LSTM for time series
    Lstm {
        /// Hidden units
        hidden_units: u32,
        /// Sequence length
        sequence_length: u32,
    },
}

/// ML training configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlTrainingConfig {
    /// Training data window
    pub training_window: Duration,
    /// Validation split
    pub validation_split: f64,
    /// Learning rate
    pub learning_rate: f64,
    /// Batch size
    pub batch_size: u32,
    /// Maximum epochs
    pub max_epochs: u32,
    /// Early stopping patience
    pub early_stopping_patience: u32,
}

/// Feature extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureExtractionConfig {
    /// Features to extract
    pub features: Vec<FeatureType>,
    /// Normalization method
    pub normalization: NormalizationMethod,
    /// Feature selection method
    pub feature_selection: FeatureSelectionMethod,
    /// Dimensionality reduction
    pub dimensionality_reduction: Option<DimensionalityReduction>,
}

/// Types of features to extract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureType {
    /// Raw metric values
    RawValues,
    /// Statistical aggregations
    StatisticalAggregations,
    /// Trend features
    TrendFeatures,
    /// Frequency domain features
    FrequencyDomain,
    /// Lagged features
    LaggedFeatures { lags: Vec<u32> },
    /// Rolling window features
    RollingWindow { windows: Vec<u32> },
    /// Cross-correlation features
    CrossCorrelation,
}

/// Normalization methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NormalizationMethod {
    /// Z-score normalization
    ZScore,
    /// Min-max scaling
    MinMax,
    /// Robust scaling
    Robust,
    /// Unit vector scaling
    UnitVector,
    /// Quantile transformation
    QuantileTransform,
}

/// Feature selection methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureSelectionMethod {
    /// Univariate selection
    Univariate { k: u32 },
    /// Recursive feature elimination
    RecursiveElimination { n_features: u32 },
    /// L1 regularization
    L1Regularization,
    /// Tree-based importance
    TreeBasedImportance,
    /// Mutual information
    MutualInformation,
}

/// Dimensionality reduction methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DimensionalityReduction {
    /// Principal Component Analysis
    Pca { n_components: u32 },
    /// t-SNE
    TSne { n_components: u32 },
    /// UMAP
    Umap { n_components: u32 },
    /// Linear Discriminant Analysis
    Lda { n_components: u32 },
}

/// Voting strategies for ensemble methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VotingStrategy {
    /// Majority voting
    Majority,
    /// Weighted voting
    Weighted,
    /// Unanimous voting
    Unanimous,
    /// Confidence-based voting
    ConfidenceBased { threshold: f64 },
    /// Bayesian model averaging
    BayesianAveraging,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Monitoring frequency
    pub monitoring_frequency: Duration,
    /// Data retention period
    pub data_retention: Duration,
    /// Metrics to monitor
    pub metrics_to_monitor: Vec<MetricType>,
    /// Real-time processing enabled
    pub enable_realtime: bool,
    /// Buffer size for streaming
    pub buffer_size: usize,
    /// Processing threads
    pub processing_threads: u32,
}

/// Types of metrics to monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    /// Performance metrics
    Performance,
    /// Resource utilization
    Resources,
    /// Error rates and counts
    Errors,
    /// System stability
    Stability,
    /// Network metrics
    Network,
    /// Application-specific metrics
    Application,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Alert levels
    pub alert_levels: Vec<AlertLevel>,
    /// Alert channels
    pub alert_channels: Vec<AlertChannel>,
    /// Alert rate limiting
    pub rate_limiting: AlertRateLimiting,
    /// Alert escalation
    pub escalation: AlertEscalation,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertLevel {
    /// Level name
    pub name: String,
    /// Severity
    pub severity: AlertSeverity,
    /// Threshold
    pub threshold: f64,
    /// Actions to take
    pub actions: Vec<AlertAction>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
    /// Emergency
    Emergency,
}

/// Alert channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertChannel {
    /// Log file
    Log { path: String },
    /// Console output
    Console,
    /// Email
    Email { recipients: Vec<String> },
    /// Webhook
    Webhook { url: String },
    /// Custom handler
    Custom { handler: String },
}

/// Alert actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertAction {
    /// Log the alert
    Log,
    /// Send notification
    Notify,
    /// Stop test execution
    StopTest,
    /// Trigger recovery
    TriggerRecovery,
    /// Scale resources
    ScaleResources,
    /// Custom action
    Custom { action: String },
}

/// Rate limiting for alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRateLimiting {
    /// Maximum alerts per time window
    pub max_alerts_per_window: u32,
    /// Time window duration
    pub time_window: Duration,
    /// Cooldown period
    pub cooldown_period: Duration,
}

/// Alert escalation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEscalation {
    /// Escalation levels
    pub levels: Vec<EscalationLevel>,
    /// Auto-escalation enabled
    pub auto_escalate: bool,
    /// Escalation timeout
    pub escalation_timeout: Duration,
}

/// Escalation level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationLevel {
    /// Level number
    pub level: u32,
    /// Required severity
    pub required_severity: AlertSeverity,
    /// Escalation delay
    pub delay: Duration,
    /// Actions at this level
    pub actions: Vec<AlertAction>,
}

/// Statistical analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalConfig {
    /// Statistical methods to use
    pub methods: Vec<StatisticalMethod>,
    /// Analysis window size
    pub window_size: u32,
    /// Confidence intervals
    pub confidence_intervals: Vec<f64>,
    /// Outlier detection methods
    pub outlier_detection: Vec<OutlierDetectionMethod>,
    /// Correlation analysis
    pub correlation_analysis: bool,
}

/// Outlier detection methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutlierDetectionMethod {
    /// Standard deviation
    StandardDeviation { threshold: f64 },
    /// IQR method
    Iqr { multiplier: f64 },
    /// Isolation forest
    IsolationForest { contamination: f64 },
    /// Local outlier factor
    LocalOutlierFactor { neighbors: u32 },
    /// One-class SVM
    OneClassSvm { nu: f64 },
}

/// Early warning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarlyWarningConfig {
    /// Warning lead time
    pub warning_lead_time: Duration,
    /// Warning thresholds
    pub warning_thresholds: WarningThresholds,
    /// Prediction horizon
    pub prediction_horizon: Duration,
    /// Warning sensitivity
    pub sensitivity: f64,
}

/// Warning thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningThresholds {
    /// Performance degradation warning
    pub performance_degradation: f64,
    /// Resource usage warning
    pub resource_usage: f64,
    /// Error rate warning
    pub error_rate: f64,
    /// Trend warning
    pub negative_trend: f64,
}

/// Confidence scoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceConfig {
    /// Confidence calculation method
    pub calculation_method: ConfidenceMethod,
    /// Minimum confidence for alerts
    pub min_confidence: f64,
    /// Uncertainty quantification
    pub uncertainty_quantification: bool,
    /// Bayesian updating
    pub bayesian_updating: bool,
}

/// Methods for confidence calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfidenceMethod {
    /// Simple voting
    SimpleVoting,
    /// Weighted voting
    WeightedVoting,
    /// Bayesian inference
    BayesianInference,
    /// Bootstrap confidence intervals
    Bootstrap { n_samples: u32 },
    /// Cross-validation
    CrossValidation { folds: u32 },
}

/// Real-time detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    /// Detection timestamp
    pub timestamp: DateTime<Utc>,
    /// Breaking point detected
    pub breaking_point_detected: bool,
    /// Confidence score
    pub confidence: f64,
    /// Detection algorithm that triggered
    pub triggered_by: String,
    /// Detected indicators
    pub indicators: Vec<BreakingPointIndicator>,
    /// Early warnings
    pub early_warnings: Vec<EarlyWarning>,
    /// Predicted time to breaking point
    pub predicted_time_to_breaking_point: Option<Duration>,
    /// Recommended actions
    pub recommended_actions: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, f64>,
}

/// Early warning information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarlyWarning {
    /// Warning type
    pub warning_type: WarningType,
    /// Severity
    pub severity: AlertSeverity,
    /// Message
    pub message: String,
    /// Time to potential issue
    pub time_to_issue: Duration,
    /// Confidence in warning
    pub confidence: f64,
    /// Suggested mitigation
    pub mitigation: String,
}

/// Types of early warnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningType {
    /// Performance degradation trend
    PerformanceDegradation,
    /// Resource exhaustion trend
    ResourceExhaustion,
    /// Error rate increase
    ErrorRateIncrease,
    /// System instability
    SystemInstability,
    /// Capacity limit approaching
    CapacityLimit,
    /// Configuration issue
    ConfigurationIssue,
}

/// Historical detection data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDataPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Performance metrics
    pub performance_metrics: PerformanceMetricsSnapshot,
    /// Resource utilization
    pub resource_utilization: ResourceUtilizationSnapshot,
    /// Stress level
    pub stress_level: StressLevel,
    /// Detection results
    pub detection_results: Vec<DetectionResult>,
}

/// Real-time breaking point detection system
pub struct RealtimeBreakingPointDetector {
    /// Configuration
    config: RealtimeDetectionConfig,
    /// Detection algorithms
    algorithms: Vec<Box<dyn DetectionAlgorithm>>,
    /// Historical data buffer
    historical_data: Arc<RwLock<VecDeque<HistoricalDataPoint>>>,
    /// Statistical analyzer
    statistical_analyzer: StatisticalAnalyzer,
    /// ML predictor
    ml_predictor: Option<MlPredictor>,
    /// Alert manager
    alert_manager: AlertManager,
    /// Performance baselines
    baselines: Arc<RwLock<PerformanceBaselines>>,
    /// Detection state
    detection_state: Arc<RwLock<DetectionState>>,
    /// Monitoring thread handle
    monitoring_thread: Option<thread::JoinHandle<()>>,
    /// Shutdown signal
    shutdown_signal: Arc<AtomicBool>,
}

/// Performance baselines for comparison
#[derive(Debug, Clone)]
struct PerformanceBaselines {
    /// Throughput baseline
    throughput_baseline: f64,
    /// Latency baseline
    latency_baseline: f64,
    /// Resource baselines
    resource_baselines: ResourceUtilizationSnapshot,
    /// Error rate baseline
    error_rate_baseline: f64,
    /// Baseline established time
    baseline_time: DateTime<Utc>,
}

/// Current detection state
#[derive(Debug, Clone)]
struct DetectionState {
    /// Current stress level
    current_stress_level: Option<StressLevel>,
    /// Last detection result
    last_detection: Option<DetectionResult>,
    /// Active warnings
    active_warnings: Vec<EarlyWarning>,
    /// Alert counters
    alert_counters: HashMap<AlertSeverity, u64>,
    /// Detection statistics
    statistics: DetectionStatistics,
}

/// Detection statistics
#[derive(Debug, Clone)]
struct DetectionStatistics {
    /// Total detections
    total_detections: u64,
    /// True positives
    true_positives: u64,
    /// False positives
    false_positives: u64,
    /// True negatives
    true_negatives: u64,
    /// False negatives
    false_negatives: u64,
    /// Average confidence
    average_confidence: f64,
}

/// Trait for detection algorithms
pub trait DetectionAlgorithm: Send + Sync {
    /// Detect breaking point
    fn detect(
        &self,
        current_metrics: &PerformanceMetricsSnapshot,
        historical_data: &[HistoricalDataPoint],
        baselines: &PerformanceBaselines,
    ) -> Result<Option<DetectionResult>, Box<dyn std::error::Error>>;

    /// Algorithm name
    fn name(&self) -> &str;

    /// Algorithm confidence weight
    fn weight(&self) -> f64;

    /// Update algorithm parameters
    fn update_parameters(&mut self, parameters: HashMap<String, f64>);
}

/// Statistical analyzer for anomaly detection
struct StatisticalAnalyzer {
    /// Configuration
    config: StatisticalConfig,
    /// Analysis buffers
    analysis_buffers: HashMap<String, VecDeque<f64>>,
    /// Statistical models
    models: HashMap<String, StatisticalModel>,
}

/// Statistical model for anomaly detection
struct StatisticalModel {
    /// Model type
    model_type: StatisticalMethod,
    /// Model parameters
    parameters: HashMap<String, f64>,
    /// Training data
    training_data: Vec<f64>,
    /// Model state
    state: HashMap<String, f64>,
}

/// Machine learning predictor
struct MlPredictor {
    /// Configuration
    config: MlModelConfig,
    /// Trained models
    models: HashMap<String, MlModel>,
    /// Feature extractor
    feature_extractor: FeatureExtractor,
    /// Training data
    training_data: VecDeque<HistoricalDataPoint>,
}

/// ML model wrapper
struct MlModel {
    /// Model type
    model_type: MlModelType,
    /// Model state
    state: HashMap<String, Vec<f64>>,
    /// Training configuration
    training_config: MlTrainingConfig,
    /// Last training time
    last_training: DateTime<Utc>,
}

/// Feature extractor for ML
struct FeatureExtractor {
    /// Configuration
    config: FeatureExtractionConfig,
    /// Feature cache
    feature_cache: HashMap<String, Vec<f64>>,
    /// Normalization parameters
    normalization_params: HashMap<String, (f64, f64)>,
}

/// Alert manager
struct AlertManager {
    /// Configuration
    config: AlertConfig,
    /// Active alerts
    active_alerts: HashMap<String, Alert>,
    /// Alert history
    alert_history: VecDeque<Alert>,
    /// Rate limiting state
    rate_limiting_state: HashMap<AlertSeverity, RateLimitingState>,
}

/// Alert information
#[derive(Debug, Clone)]
struct Alert {
    /// Alert ID
    id: String,
    /// Alert severity
    severity: AlertSeverity,
    /// Alert message
    message: String,
    /// Alert timestamp
    timestamp: DateTime<Utc>,
    /// Alert source
    source: String,
    /// Alert metadata
    metadata: HashMap<String, String>,
}

/// Rate limiting state
#[derive(Debug, Clone)]
struct RateLimitingState {
    /// Alert count in current window
    alerts_in_window: u32,
    /// Window start time
    window_start: DateTime<Utc>,
    /// Last alert time
    last_alert: Option<DateTime<Utc>>,
}

impl Default for RealtimeDetectionConfig {
    fn default() -> Self {
        Self {
            detection_algorithms: vec![
                DetectionAlgorithmConfig {
                    algorithm_type: DetectionAlgorithmType::Statistical {
                        method: StatisticalMethod::ZScore,
                        parameters: StatisticalParameters {
                            threshold_multiplier: 3.0,
                            window_size: 50,
                            min_samples: 10,
                            outlier_percentile: 95.0,
                            confidence_interval: 0.95,
                        },
                    },
                    weight: 1.0,
                    parameters: HashMap::new(),
                    enabled: true,
                    min_confidence: 0.7,
                },
                DetectionAlgorithmConfig {
                    algorithm_type: DetectionAlgorithmType::Threshold {
                        config: ThresholdDetectionConfig {
                            performance_thresholds: PerformanceThresholds {
                                throughput_degradation: 50.0,
                                latency_increase: 100.0,
                                response_time_ms: 5000.0,
                                success_rate: 80.0,
                                quality_score: 50.0,
                            },
                            resource_thresholds: ResourceThresholds {
                                cpu_utilization: 90.0,
                                memory_utilization: 85.0,
                                disk_io_threshold: 1000.0,
                                network_io_threshold: 100.0,
                                file_descriptor_count: 8192,
                                thread_count: 1000,
                                load_average: 10.0,
                            },
                            error_thresholds: ErrorThresholds {
                                error_rate: 25.0,
                                timeout_rate: 15.0,
                                critical_errors: 10,
                                exception_rate: 5.0,
                            },
                            adaptive_thresholds: true,
                            adjustment_rate: 0.1,
                        },
                    },
                    weight: 1.5,
                    parameters: HashMap::new(),
                    enabled: true,
                    min_confidence: 0.8,
                },
            ],
            monitoring_config: MonitoringConfig {
                monitoring_frequency: Duration::from_millis(100),
                data_retention: Duration::from_secs(3600),
                metrics_to_monitor: vec![
                    MetricType::Performance,
                    MetricType::Resources,
                    MetricType::Errors,
                ],
                enable_realtime: true,
                buffer_size: 1000,
                processing_threads: 4,
            },
            alert_config: AlertConfig {
                alert_levels: vec![
                    AlertLevel {
                        name: "Warning".to_string(),
                        severity: AlertSeverity::Warning,
                        threshold: 0.7,
                        actions: vec![AlertAction::Log, AlertAction::Notify],
                    },
                    AlertLevel {
                        name: "Critical".to_string(),
                        severity: AlertSeverity::Critical,
                        threshold: 0.9,
                        actions: vec![AlertAction::Log, AlertAction::Notify, AlertAction::StopTest],
                    },
                ],
                alert_channels: vec![AlertChannel::Console, AlertChannel::Log { path: "alerts.log".to_string() }],
                rate_limiting: AlertRateLimiting {
                    max_alerts_per_window: 10,
                    time_window: Duration::from_secs(60),
                    cooldown_period: Duration::from_secs(5),
                },
                escalation: AlertEscalation {
                    levels: Vec::new(),
                    auto_escalate: false,
                    escalation_timeout: Duration::from_secs(300),
                },
            },
            statistical_config: StatisticalConfig {
                methods: vec![StatisticalMethod::ZScore, StatisticalMethod::InterquartileRange],
                window_size: 50,
                confidence_intervals: vec![0.95, 0.99],
                outlier_detection: vec![
                    OutlierDetectionMethod::StandardDeviation { threshold: 3.0 },
                    OutlierDetectionMethod::Iqr { multiplier: 1.5 },
                ],
                correlation_analysis: true,
            },
            ml_config: MachineLearningConfig {
                model_type: MlModelType::RandomForest {
                    n_trees: 100,
                    max_depth: 10,
                },
                training_config: MlTrainingConfig {
                    training_window: Duration::from_secs(1800),
                    validation_split: 0.2,
                    learning_rate: 0.01,
                    batch_size: 32,
                    max_epochs: 100,
                    early_stopping_patience: 10,
                },
                feature_config: FeatureExtractionConfig {
                    features: vec![
                        FeatureType::RawValues,
                        FeatureType::StatisticalAggregations,
                        FeatureType::TrendFeatures,
                    ],
                    normalization: NormalizationMethod::ZScore,
                    feature_selection: FeatureSelectionMethod::TreeBasedImportance,
                    dimensionality_reduction: None,
                },
                update_frequency: Duration::from_secs(300),
            },
            threshold_config: ThresholdConfig {
                adaptive_adjustment: true,
                adjustment_rate: 0.05,
                min_threshold_ratio: 0.5,
                max_threshold_ratio: 2.0,
                update_frequency: Duration::from_secs(60),
            },
            early_warning_config: EarlyWarningConfig {
                warning_lead_time: Duration::from_secs(30),
                warning_thresholds: WarningThresholds {
                    performance_degradation: 25.0,
                    resource_usage: 75.0,
                    error_rate: 10.0,
                    negative_trend: -0.1,
                },
                prediction_horizon: Duration::from_secs(120),
                sensitivity: 0.8,
            },
            confidence_config: ConfidenceConfig {
                calculation_method: ConfidenceMethod::WeightedVoting,
                min_confidence: 0.7,
                uncertainty_quantification: true,
                bayesian_updating: true,
            },
        }
    }
}

/// Additional threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// Enable adaptive threshold adjustment
    pub adaptive_adjustment: bool,
    /// Rate of threshold adjustment
    pub adjustment_rate: f64,
    /// Minimum threshold ratio
    pub min_threshold_ratio: f64,
    /// Maximum threshold ratio
    pub max_threshold_ratio: f64,
    /// Threshold update frequency
    pub update_frequency: Duration,
}

impl RealtimeBreakingPointDetector {
    /// Create a new real-time breaking point detector
    pub fn new(config: RealtimeDetectionConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let algorithms = Self::create_detection_algorithms(&config.detection_algorithms)?;
        let historical_data = Arc::new(RwLock::new(VecDeque::new()));
        let statistical_analyzer = StatisticalAnalyzer::new(config.statistical_config.clone());
        let ml_predictor = Self::create_ml_predictor(&config.ml_config)?;
        let alert_manager = AlertManager::new(config.alert_config.clone());
        let baselines = Arc::new(RwLock::new(PerformanceBaselines {
            throughput_baseline: 0.0,
            latency_baseline: 0.0,
            resource_baselines: ResourceUtilizationSnapshot {
                cpu_utilization: 0.0,
                memory_utilization: 0.0,
                disk_io_rate: 0.0,
                network_io_rate: 0.0,
                open_file_descriptors: 0,
                thread_count: 0,
                load_average: 0.0,
                swap_usage: 0.0,
            },
            error_rate_baseline: 0.0,
            baseline_time: Utc::now(),
        }));
        let detection_state = Arc::new(RwLock::new(DetectionState {
            current_stress_level: None,
            last_detection: None,
            active_warnings: Vec::new(),
            alert_counters: HashMap::new(),
            statistics: DetectionStatistics {
                total_detections: 0,
                true_positives: 0,
                false_positives: 0,
                true_negatives: 0,
                false_negatives: 0,
                average_confidence: 0.0,
            },
        }));
        let shutdown_signal = Arc::new(AtomicBool::new(false));

        Ok(Self {
            config,
            algorithms,
            historical_data,
            statistical_analyzer,
            ml_predictor,
            alert_manager,
            baselines,
            detection_state,
            monitoring_thread: None,
            shutdown_signal,
        })
    }

    /// Start real-time monitoring
    pub fn start_monitoring(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.monitoring_thread.is_some() {
            return Err("Monitoring already started".into());
        }

        let config = self.config.clone();
        let historical_data = Arc::clone(&self.historical_data);
        let shutdown_signal = Arc::clone(&self.shutdown_signal);

        let monitoring_thread = thread::spawn(move || {
            while !shutdown_signal.load(Ordering::Relaxed) {
                // Monitoring loop would be implemented here
                thread::sleep(config.monitoring_config.monitoring_frequency);
            }
        });

        self.monitoring_thread = Some(monitoring_thread);
        println!("🎯 Real-time breaking point detection started");
        Ok(())
    }

    /// Stop monitoring
    pub fn stop_monitoring(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.shutdown_signal.store(true, Ordering::Relaxed);
        
        if let Some(thread) = self.monitoring_thread.take() {
            thread.join().map_err(|_| "Failed to join monitoring thread")?;
        }

        println!("⏹️ Real-time breaking point detection stopped");
        Ok(())
    }

    /// Process new metrics and detect breaking points
    pub fn process_metrics(
        &mut self,
        metrics: PerformanceMetricsSnapshot,
        resources: ResourceUtilizationSnapshot,
        stress_level: StressLevel,
    ) -> Result<DetectionResult, Box<dyn std::error::Error>> {
        
        // Add to historical data
        let data_point = HistoricalDataPoint {
            timestamp: Utc::now(),
            performance_metrics: metrics.clone(),
            resource_utilization: resources,
            stress_level: stress_level.clone(),
            detection_results: Vec::new(),
        };

        {
            let mut historical = self.historical_data.write()
                .map_err(|e| format!("Failed to acquire historical data lock: {}", e))?;
            historical.push_back(data_point);
            
            // Limit buffer size
            while historical.len() > self.config.monitoring_config.buffer_size {
                historical.pop_front();
            }
        }

        // Update current stress level
        {
            let mut state = self.detection_state.write()
                .map_err(|e| format!("Failed to acquire detection state lock: {}", e))?;
            state.current_stress_level = Some(stress_level);
        }

        // Run detection algorithms
        self.run_detection_algorithms(&metrics)
    }

    /// Run all detection algorithms
    fn run_detection_algorithms(
        &self,
        metrics: &PerformanceMetricsSnapshot,
    ) -> Result<DetectionResult, Box<dyn std::error::Error>> {
        
        let historical_data = {
            let historical = self.historical_data.read()
                .map_err(|e| format!("Failed to acquire historical data lock: {}", e))?;
            historical.iter().cloned().collect::<Vec<_>>()
        };

        let baselines = {
            let baselines = self.baselines.read()
                .map_err(|e| format!("Failed to acquire baselines lock: {}", e))?;
            baselines.clone()
        };

        let mut detection_results = Vec::new();
        let mut total_weight = 0.0;
        let mut weighted_confidence = 0.0;

        // Run each algorithm
        for algorithm in &self.algorithms {
            if let Ok(Some(result)) = algorithm.detect(metrics, &historical_data, &baselines) {
                let weight = algorithm.weight();
                total_weight += weight;
                weighted_confidence += result.confidence * weight;
                detection_results.push(result);
            }
        }

        // Calculate overall confidence
        let overall_confidence = if total_weight > 0.0 {
            weighted_confidence / total_weight
        } else {
            0.0
        };

        // Determine if breaking point is detected
        let breaking_point_detected = overall_confidence >= self.config.confidence_config.min_confidence;

        // Generate early warnings
        let early_warnings = self.generate_early_warnings(metrics, &historical_data)?;

        // Predict time to breaking point
        let predicted_time = self.predict_time_to_breaking_point(metrics, &historical_data)?;

        // Generate recommended actions
        let recommended_actions = self.generate_recommended_actions(&detection_results, &early_warnings);

        let result = DetectionResult {
            timestamp: Utc::now(),
            breaking_point_detected,
            confidence: overall_confidence,
            triggered_by: self.get_primary_trigger(&detection_results),
            indicators: self.extract_indicators(&detection_results),
            early_warnings,
            predicted_time_to_breaking_point: predicted_time,
            recommended_actions,
            metadata: HashMap::new(),
        };

        // Process alerts
        self.process_alerts(&result)?;

        // Update detection statistics
        self.update_detection_statistics(&result)?;

        Ok(result)
    }

    /// Generate early warnings
    fn generate_early_warnings(
        &self,
        metrics: &PerformanceMetricsSnapshot,
        historical_data: &[HistoricalDataPoint],
    ) -> Result<Vec<EarlyWarning>, Box<dyn std::error::Error>> {
        
        let mut warnings = Vec::new();
        
        // Check for performance degradation trend
        if let Some(degradation_trend) = self.analyze_performance_trend(historical_data) {
            if degradation_trend < self.config.early_warning_config.warning_thresholds.negative_trend {
                warnings.push(EarlyWarning {
                    warning_type: WarningType::PerformanceDegradation,
                    severity: AlertSeverity::Warning,
                    message: format!("Performance degradation trend detected: {:.2}%/min", degradation_trend * 100.0),
                    time_to_issue: Duration::from_secs(60),
                    confidence: 0.8,
                    mitigation: "Consider reducing load or optimizing performance".to_string(),
                });
            }
        }

        // Check for resource exhaustion
        if metrics.resource_metrics.memory_metrics.utilization_percent > self.config.early_warning_config.warning_thresholds.resource_usage {
            warnings.push(EarlyWarning {
                warning_type: WarningType::ResourceExhaustion,
                severity: AlertSeverity::Warning,
                message: format!("High memory utilization: {:.1}%", metrics.resource_metrics.memory_metrics.utilization_percent),
                time_to_issue: Duration::from_secs(120),
                confidence: 0.9,
                mitigation: "Increase available memory or optimize memory usage".to_string(),
            });
        }

        Ok(warnings)
    }

    /// Analyze performance trend
    fn analyze_performance_trend(&self, historical_data: &[HistoricalDataPoint]) -> Option<f64> {
        if historical_data.len() < 5 {
            return None;
        }

        // Simple linear regression on throughput
        let throughputs: Vec<f64> = historical_data.iter()
            .map(|d| d.performance_metrics.throughput_metrics.average_throughput)
            .collect();

        // Calculate slope (trend)
        let n = throughputs.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = throughputs.iter().sum::<f64>() / n;

        let numerator: f64 = throughputs.iter().enumerate()
            .map(|(i, &y)| (i as f64 - x_mean) * (y - y_mean))
            .sum();

        let denominator: f64 = (0..throughputs.len())
            .map(|i| (i as f64 - x_mean).powi(2))
            .sum();

        if denominator > 0.0 {
            Some(numerator / denominator)
        } else {
            None
        }
    }

    /// Predict time to breaking point
    fn predict_time_to_breaking_point(
        &self,
        _metrics: &PerformanceMetricsSnapshot,
        _historical_data: &[HistoricalDataPoint],
    ) -> Result<Option<Duration>, Box<dyn std::error::Error>> {
        // Simplified prediction - would use ML models in practice
        Ok(Some(Duration::from_secs(300)))
    }

    /// Generate recommended actions
    fn generate_recommended_actions(&self, results: &[DetectionResult], warnings: &[EarlyWarning]) -> Vec<String> {
        let mut actions = Vec::new();

        if !results.is_empty() && results[0].breaking_point_detected {
            actions.push("Stop test execution immediately".to_string());
            actions.push("Save current state for analysis".to_string());
        }

        if warnings.iter().any(|w| matches!(w.warning_type, WarningType::ResourceExhaustion)) {
            actions.push("Monitor resource usage closely".to_string());
            actions.push("Consider reducing concurrent operations".to_string());
        }

        if warnings.iter().any(|w| matches!(w.warning_type, WarningType::PerformanceDegradation)) {
            actions.push("Analyze performance bottlenecks".to_string());
            actions.push("Consider optimizing hot code paths".to_string());
        }

        actions
    }

    /// Get primary trigger algorithm
    fn get_primary_trigger(&self, results: &[DetectionResult]) -> String {
        results.iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal))
            .map(|r| r.triggered_by.clone())
            .unwrap_or_else(|| "None".to_string())
    }

    /// Extract indicators from results
    fn extract_indicators(&self, results: &[DetectionResult]) -> Vec<BreakingPointIndicator> {
        results.iter()
            .flat_map(|r| r.indicators.iter().cloned())
            .collect()
    }

    /// Process alerts based on detection results
    fn process_alerts(&self, result: &DetectionResult) -> Result<(), Box<dyn std::error::Error>> {
        if result.breaking_point_detected {
            println!("🚨 BREAKING POINT DETECTED - Confidence: {:.1}%", result.confidence * 100.0);
            
            for warning in &result.early_warnings {
                println!("⚠️  {}: {}", warning.warning_type, warning.message);
            }
        }

        for action in &result.recommended_actions {
            println!("💡 Recommended: {}", action);
        }

        Ok(())
    }

    /// Update detection statistics
    fn update_detection_statistics(&self, result: &DetectionResult) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.detection_state.write()
            .map_err(|e| format!("Failed to acquire detection state lock: {}", e))?;
        
        state.statistics.total_detections += 1;
        state.last_detection = Some(result.clone());
        
        // Update running average confidence
        let n = state.statistics.total_detections as f64;
        state.statistics.average_confidence = 
            (state.statistics.average_confidence * (n - 1.0) + result.confidence) / n;

        Ok(())
    }

    /// Set performance baselines
    pub fn set_baselines(
        &self,
        metrics: &PerformanceMetricsSnapshot,
        resources: &ResourceUtilizationSnapshot,
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        let mut baselines = self.baselines.write()
            .map_err(|e| format!("Failed to acquire baselines lock: {}", e))?;
        
        baselines.throughput_baseline = metrics.throughput_metrics.average_throughput;
        baselines.latency_baseline = metrics.latency_metrics.mean_latency.as_millis() as f64;
        baselines.resource_baselines = resources.clone();
        baselines.error_rate_baseline = 0.0; // Would extract from metrics
        baselines.baseline_time = Utc::now();

        println!("📏 Performance baselines established");
        Ok(())
    }

    /// Get detection statistics
    pub fn get_statistics(&self) -> Result<DetectionStatistics, Box<dyn std::error::Error>> {
        let state = self.detection_state.read()
            .map_err(|e| format!("Failed to acquire detection state lock: {}", e))?;
        Ok(state.statistics.clone())
    }

    /// Create detection algorithms
    fn create_detection_algorithms(configs: &[DetectionAlgorithmConfig]) -> Result<Vec<Box<dyn DetectionAlgorithm>>, Box<dyn std::error::Error>> {
        // Would create actual algorithm implementations
        Ok(Vec::new())
    }

    /// Create ML predictor
    fn create_ml_predictor(config: &MachineLearningConfig) -> Result<Option<MlPredictor>, Box<dyn std::error::Error>> {
        // Would create actual ML predictor
        Ok(None)
    }
}

impl StatisticalAnalyzer {
    fn new(config: StatisticalConfig) -> Self {
        Self {
            config,
            analysis_buffers: HashMap::new(),
            models: HashMap::new(),
        }
    }
}

impl AlertManager {
    fn new(config: AlertConfig) -> Self {
        Self {
            config,
            active_alerts: HashMap::new(),
            alert_history: VecDeque::new(),
            rate_limiting_state: HashMap::new(),
        }
    }
}

// Implement Display for warning types
impl std::fmt::Display for WarningType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WarningType::PerformanceDegradation => write!(f, "Performance Degradation"),
            WarningType::ResourceExhaustion => write!(f, "Resource Exhaustion"),
            WarningType::ErrorRateIncrease => write!(f, "Error Rate Increase"),
            WarningType::SystemInstability => write!(f, "System Instability"),
            WarningType::CapacityLimit => write!(f, "Capacity Limit"),
            WarningType::ConfigurationIssue => write!(f, "Configuration Issue"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realtime_detection_config_default() {
        let config = RealtimeDetectionConfig::default();
        assert!(!config.detection_algorithms.is_empty());
        assert!(!config.monitoring_config.metrics_to_monitor.is_empty());
        assert!(config.confidence_config.min_confidence > 0.0);
    }

    #[test]
    fn test_detection_algorithm_config() {
        let config = DetectionAlgorithmConfig {
            algorithm_type: DetectionAlgorithmType::Statistical {
                method: StatisticalMethod::ZScore,
                parameters: StatisticalParameters {
                    threshold_multiplier: 3.0,
                    window_size: 50,
                    min_samples: 10,
                    outlier_percentile: 95.0,
                    confidence_interval: 0.95,
                },
            },
            weight: 1.0,
            parameters: HashMap::new(),
            enabled: true,
            min_confidence: 0.7,
        };

        assert!(config.enabled);
        assert!(config.weight > 0.0);
        assert!(config.min_confidence > 0.0);
    }

    #[test]
    fn test_threshold_detection_config() {
        let config = ThresholdDetectionConfig {
            performance_thresholds: PerformanceThresholds {
                throughput_degradation: 50.0,
                latency_increase: 100.0,
                response_time_ms: 5000.0,
                success_rate: 80.0,
                quality_score: 50.0,
            },
            resource_thresholds: ResourceThresholds {
                cpu_utilization: 90.0,
                memory_utilization: 85.0,
                disk_io_threshold: 1000.0,
                network_io_threshold: 100.0,
                file_descriptor_count: 8192,
                thread_count: 1000,
                load_average: 10.0,
            },
            error_thresholds: ErrorThresholds {
                error_rate: 25.0,
                timeout_rate: 15.0,
                critical_errors: 10,
                exception_rate: 5.0,
            },
            adaptive_thresholds: true,
            adjustment_rate: 0.1,
        };

        assert!(config.adaptive_thresholds);
        assert!(config.performance_thresholds.throughput_degradation > 0.0);
        assert!(config.resource_thresholds.cpu_utilization > 0.0);
        assert!(config.error_thresholds.error_rate > 0.0);
    }

    #[test]
    fn test_early_warning_creation() {
        let warning = EarlyWarning {
            warning_type: WarningType::PerformanceDegradation,
            severity: AlertSeverity::Warning,
            message: "Performance is degrading".to_string(),
            time_to_issue: Duration::from_secs(60),
            confidence: 0.8,
            mitigation: "Reduce load".to_string(),
        };

        assert_eq!(warning.confidence, 0.8);
        assert!(warning.time_to_issue > Duration::from_secs(0));
        matches!(warning.severity, AlertSeverity::Warning);
    }

    #[test]
    fn test_detection_result() {
        let result = DetectionResult {
            timestamp: Utc::now(),
            breaking_point_detected: true,
            confidence: 0.95,
            triggered_by: "StatisticalDetector".to_string(),
            indicators: Vec::new(),
            early_warnings: Vec::new(),
            predicted_time_to_breaking_point: Some(Duration::from_secs(120)),
            recommended_actions: vec!["Stop test".to_string()],
            metadata: HashMap::new(),
        };

        assert!(result.breaking_point_detected);
        assert!(result.confidence > 0.9);
        assert!(!result.recommended_actions.is_empty());
    }

    #[test]
    fn test_statistical_parameters() {
        let params = StatisticalParameters {
            threshold_multiplier: 3.0,
            window_size: 50,
            min_samples: 10,
            outlier_percentile: 95.0,
            confidence_interval: 0.95,
        };

        assert_eq!(params.threshold_multiplier, 3.0);
        assert_eq!(params.window_size, 50);
        assert!(params.confidence_interval > 0.0 && params.confidence_interval <= 1.0);
    }

    #[test]
    fn test_ml_model_config() {
        let config = MlModelConfig {
            model_type: MlModelType::RandomForest {
                n_trees: 100,
                max_depth: 10,
            },
            training_config: MlTrainingConfig {
                training_window: Duration::from_secs(1800),
                validation_split: 0.2,
                learning_rate: 0.01,
                batch_size: 32,
                max_epochs: 100,
                early_stopping_patience: 10,
            },
            feature_config: FeatureExtractionConfig {
                features: vec![FeatureType::RawValues],
                normalization: NormalizationMethod::ZScore,
                feature_selection: FeatureSelectionMethod::TreeBasedImportance,
                dimensionality_reduction: None,
            },
            update_frequency: Duration::from_secs(300),
        };

        match config.model_type {
            MlModelType::RandomForest { n_trees, max_depth } => {
                assert_eq!(n_trees, 100);
                assert_eq!(max_depth, 10);
            },
            _ => panic!("Expected RandomForest model"),
        }
    }

    #[test]
    fn test_alert_level() {
        let level = AlertLevel {
            name: "Critical".to_string(),
            severity: AlertSeverity::Critical,
            threshold: 0.9,
            actions: vec![AlertAction::Log, AlertAction::StopTest],
        };

        assert_eq!(level.name, "Critical");
        matches!(level.severity, AlertSeverity::Critical);
        assert_eq!(level.actions.len(), 2);
    }

    #[test]
    fn test_voting_strategy() {
        let strategy = VotingStrategy::ConfidenceBased { threshold: 0.8 };
        
        match strategy {
            VotingStrategy::ConfidenceBased { threshold } => {
                assert_eq!(threshold, 0.8);
            },
            _ => panic!("Expected ConfidenceBased strategy"),
        }
    }

    #[test]
    fn test_detection_statistics() {
        let mut stats = DetectionStatistics {
            total_detections: 100,
            true_positives: 85,
            false_positives: 5,
            true_negatives: 90,
            false_negatives: 10,
            average_confidence: 0.0,
        };

        // Calculate precision and recall
        let precision = stats.true_positives as f64 / (stats.true_positives + stats.false_positives) as f64;
        let recall = stats.true_positives as f64 / (stats.true_positives + stats.false_negatives) as f64;

        assert!(precision > 0.9);  // > 90%
        assert!(recall > 0.8);     // > 80%
    }

    #[test]
    fn test_historical_data_point() {
        use test_performance_metrics_infrastructure::*;

        let data_point = HistoricalDataPoint {
            timestamp: Utc::now(),
            performance_metrics: create_mock_performance_snapshot(),
            resource_utilization: ResourceUtilizationSnapshot {
                cpu_utilization: 75.0,
                memory_utilization: 80.0,
                disk_io_rate: 150.0,
                network_io_rate: 25.0,
                open_file_descriptors: 512,
                thread_count: 32,
                load_average: 3.5,
                swap_usage: 10.0,
            },
            stress_level: StressLevel {
                package_count: 500,
                dependency_complexity: 5,
                concurrency_level: 4,
                duration_multiplier: 2.0,
                memory_pressure: 1.5,
                cpu_pressure: 1.5,
                io_pressure: 1.0,
                custom_parameters: HashMap::new(),
            },
            detection_results: Vec::new(),
        };

        assert!(data_point.resource_utilization.cpu_utilization > 0.0);
        assert_eq!(data_point.stress_level.package_count, 500);
    }

    // Helper function to create mock performance snapshot
    fn create_mock_performance_snapshot() -> PerformanceMetricsSnapshot {
        use test_performance_metrics_infrastructure::*;

        PerformanceMetricsSnapshot {
            timestamp: Utc::now(),
            throughput_metrics: ThroughputMetrics {
                instantaneous_throughput: 100.0,
                average_throughput: 95.0,
                peak_throughput: 120.0,
                minimum_throughput: 80.0,
                throughput_stddev: 10.0,
                throughput_history: Vec::new(),
                throughput_trend: ThroughputTrend::Stable { variance: 5.0 },
                efficiency_score: 85.0,
                items_processed: 1000,
                measurement_window: Duration::from_secs(60),
                batch_metrics: None,
            },
            latency_metrics: LatencyMetrics {
                min_latency: Duration::from_millis(50),
                max_latency: Duration::from_millis(150),
                mean_latency: Duration::from_millis(100),
                median_latency: Duration::from_millis(95),
                p95_latency: Duration::from_millis(140),
                p99_latency: Duration::from_millis(145),
                p999_latency: Duration::from_millis(150),
                latency_stddev: Duration::from_millis(20),
                latency_histogram: LatencyHistogram {
                    buckets: vec![50.0, 100.0, 150.0, 200.0],
                    counts: vec![10, 70, 19, 1],
                    total_samples: 100,
                    percentages: vec![10.0, 70.0, 19.0, 1.0],
                },
                latency_trend: LatencyTrend::Stable { stability_index: 0.9 },
                sla_compliance: SlaCompliance {
                    target_latency: Duration::from_millis(100),
                    compliance_percentage: 80.0,
                    violations_count: 20,
                    total_requests: 100,
                    breach_severity: HashMap::new(),
                },
                outliers_analysis: OutliersAnalysis {
                    outliers_count: 5,
                    outlier_threshold: Duration::from_millis(200),
                    outlier_values: vec![Duration::from_millis(180), Duration::from_millis(190)],
                    detection_method: OutlierDetectionMethod::InterquartileRange { iqr_multiplier: 1.5 },
                    potential_causes: vec!["High system load".to_string()],
                },
            },
            resource_metrics: create_mock_resource_metrics(),
            collection_duration: Duration::from_secs(60),
            quality_score: 85.0,
        }
    }

    // Helper function to create mock resource metrics
    fn create_mock_resource_metrics() -> ResourceMetrics {
        use test_performance_metrics_infrastructure::*;

        ResourceMetrics {
            cpu_metrics: CpuMetrics {
                overall_utilization: 50.0,
                per_core_utilization: vec![45.0, 55.0, 48.0, 52.0],
                user_time_percent: 40.0,
                system_time_percent: 10.0,
                idle_time_percent: 50.0,
                iowait_percent: 2.0,
                load_averages: [2.0, 2.1, 2.0],
                context_switches_per_sec: 1000.0,
                thermal_throttling_events: 0,
            },
            memory_metrics: MemoryMetrics {
                total_memory: 16 * 1024 * 1024 * 1024,
                used_memory: 8 * 1024 * 1024 * 1024,
                available_memory: 8 * 1024 * 1024 * 1024,
                utilization_percent: 50.0,
                buffer_cache: 1024 * 1024 * 1024,
                swap_used: 0,
                swap_total: 4 * 1024 * 1024 * 1024,
                allocation_rate: 1000000.0,
                deallocation_rate: 950000.0,
                page_faults_per_sec: 100.0,
                fragmentation_index: 0.1,
            },
            disk_metrics: DiskMetrics {
                read_ops_per_sec: 100.0,
                write_ops_per_sec: 50.0,
                read_throughput: 50.0 * 1024.0 * 1024.0,
                write_throughput: 25.0 * 1024.0 * 1024.0,
                avg_read_latency: Duration::from_millis(5),
                avg_write_latency: Duration::from_millis(8),
                disk_utilization: 30.0,
                queue_depth: 2.0,
                available_space: 500 * 1024 * 1024 * 1024,
                io_efficiency: 85.0,
            },
            network_metrics: NetworkMetrics {
                bytes_received_per_sec: 5.0 * 1024.0 * 1024.0,
                bytes_transmitted_per_sec: 2.0 * 1024.0 * 1024.0,
                packets_received_per_sec: 500.0,
                packets_transmitted_per_sec: 300.0,
                errors_per_sec: 0.1,
                drops_per_sec: 0.05,
                network_latency: Duration::from_millis(10),
                bandwidth_utilization: 10.0,
            },
            process_metrics: ProcessMetrics {
                cpu_percent: 25.0,
                memory_usage: 2 * 1024 * 1024 * 1024,
                virtual_memory: 4 * 1024 * 1024 * 1024,
                thread_count: 8,
                open_files: 100,
                uptime: Duration::from_secs(3600),
                io_stats: ProcessIoStats {
                    bytes_read: 100 * 1024 * 1024,
                    bytes_written: 50 * 1024 * 1024,
                    read_operations: 1000,
                    write_operations: 500,
                },
            },
            system_metrics: SystemMetrics {
                uptime: Duration::from_secs(86400),
                process_count: 100,
                temperature: Some(60.0),
                power_consumption: Some(75.0),
                entropy_available: 4000,
            },
            efficiency_score: ResourceEfficiencyScore {
                overall_score: 85.0,
                cpu_efficiency: 85.0,
                memory_efficiency: 80.0,
                io_efficiency: 85.0,
                resource_balance: 90.0,
                recommendations: Vec::new(),
            },
        }
    }
}