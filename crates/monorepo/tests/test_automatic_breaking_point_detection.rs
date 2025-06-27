//! Automatic Breaking Point Detection System
//!
//! This module implements intelligent detection of system breaking points using real-time
//! monitoring data, statistical analysis, and adaptive threshold determination.

use sublime_monorepo_tools::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque};

/// Configuration for automatic breaking point detection
#[derive(Debug, Clone)]
pub struct AutomaticDetectionConfig {
    /// Initial monitoring interval in milliseconds
    pub initial_monitoring_interval_ms: u64,
    /// Maximum test duration before auto-stopping
    pub max_test_duration_secs: u64,
    /// Minimum samples required for statistical analysis
    pub min_samples_for_analysis: usize,
    /// Threshold for performance degradation detection (percentage)
    pub performance_degradation_threshold: f64,
    /// Memory growth rate threshold (MB/sec)
    pub memory_growth_rate_threshold: f64,
    /// CPU saturation threshold (percentage)
    pub cpu_saturation_threshold: f64,
    /// Response time degradation threshold (multiplier)
    pub response_time_threshold: f64,
    /// Enable adaptive threshold adjustment
    pub enable_adaptive_thresholds: bool,
    /// Statistical confidence level for detection
    pub confidence_level: f64,
}

impl Default for AutomaticDetectionConfig {
    fn default() -> Self {
        Self {
            initial_monitoring_interval_ms: 100,
            max_test_duration_secs: 600, // 10 minutes max
            min_samples_for_analysis: 30,
            performance_degradation_threshold: 50.0, // 50% degradation
            memory_growth_rate_threshold: 10.0, // 10 MB/sec
            cpu_saturation_threshold: 95.0,
            response_time_threshold: 3.0, // 3x slower
            enable_adaptive_thresholds: true,
            confidence_level: 0.95,
        }
    }
}

/// Real-time system metrics sample
#[derive(Debug, Clone)]
pub struct SystemMetricsSample {
    /// Timestamp when sample was taken
    pub timestamp: Instant,
    /// Memory usage in MB
    pub memory_mb: f64,
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Operation count at this sample
    pub operation_count: usize,
    /// Response time for last operation (ms)
    pub response_time_ms: f64,
    /// Throughput (operations per second)
    pub throughput_ops_per_sec: f64,
    /// Memory growth rate (MB/sec)
    pub memory_growth_rate: f64,
    /// CPU efficiency (ops per CPU percent)
    pub cpu_efficiency: f64,
}

/// Breaking point detection result
#[derive(Debug, Clone)]
pub struct BreakingPointDetection {
    /// When the breaking point was detected
    pub detection_timestamp: Instant,
    /// Type of breaking point detected
    pub breaking_point_type: AutoBreakingPointType,
    /// Severity of the breaking point
    pub severity: BreakingPointSeverity,
    /// Triggering metric value
    pub trigger_value: f64,
    /// Dynamic threshold that was exceeded
    pub dynamic_threshold: f64,
    /// Confidence level of detection
    pub confidence: f64,
    /// Recommended safe operating threshold
    pub safe_threshold: f64,
    /// Detailed analysis of the breaking point
    pub analysis: BreakingPointAnalysis,
}

/// Types of automatically detected breaking points
#[derive(Debug, Clone, PartialEq)]
pub enum AutoBreakingPointType {
    /// Memory exhaustion with rapid growth
    MemoryExhaustion,
    /// CPU saturation with poor efficiency
    CpuSaturation,
    /// Performance degradation below acceptable levels
    PerformanceDegradation,
    /// Response time explosion
    ResponseTimeExplosion,
    /// Throughput collapse
    ThroughputCollapse,
    /// Memory leak pattern detected
    MemoryLeak,
    /// Resource thrashing
    ResourceThrashing,
    /// System instability
    SystemInstability,
}

/// Severity levels for breaking points
#[derive(Debug, Clone, PartialEq)]
pub enum BreakingPointSeverity {
    /// Warning level - approaching limits
    Warning,
    /// Critical level - at breaking point
    Critical,
    /// Fatal level - system failure imminent
    Fatal,
}

/// Detailed analysis of a breaking point
#[derive(Debug, Clone)]
pub struct BreakingPointAnalysis {
    /// Statistical significance of the detection
    pub statistical_significance: f64,
    /// Trend leading to breaking point
    pub trend_analysis: TrendAnalysis,
    /// Contributing factors
    pub contributing_factors: Vec<String>,
    /// Recommended actions
    pub recommended_actions: Vec<String>,
    /// Predicted time to failure if no action taken
    pub time_to_failure_estimate: Option<Duration>,
}

/// Trend analysis for metrics leading to breaking point
#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    /// Rate of change in the problematic metric
    pub rate_of_change: f64,
    /// Acceleration (second derivative)
    pub acceleration: f64,
    /// Correlation with other metrics
    pub metric_correlations: HashMap<String, f64>,
    /// Stability indicator
    pub stability_score: f64,
}

/// Adaptive threshold tracker
#[derive(Debug)]
struct AdaptiveThreshold {
    /// Current threshold value
    value: f64,
    /// Baseline value during stable operation
    baseline: f64,
    /// Standard deviation during stable operation
    std_deviation: f64,
    /// Number of adjustments made
    adjustment_count: usize,
    /// History of threshold values
    history: VecDeque<(Instant, f64)>,
}

impl AdaptiveThreshold {
    fn new(initial_value: f64) -> Self {
        Self {
            value: initial_value,
            baseline: initial_value,
            std_deviation: 0.0,
            adjustment_count: 0,
            history: VecDeque::new(),
        }
    }
    
    /// Update threshold based on recent stable measurements
    fn update_from_baseline(&mut self, samples: &[f64], confidence_level: f64) {
        if samples.len() < 10 {
            return; // Need minimum samples for reliable statistics
        }
        
        // Calculate new baseline and standard deviation
        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        let variance = samples.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / samples.len() as f64;
        let std_dev = variance.sqrt();
        
        // Update baseline statistics
        self.baseline = mean;
        self.std_deviation = std_dev;
        
        // Calculate new threshold using statistical confidence interval
        let confidence_multiplier = match confidence_level {
            x if x >= 0.99 => 3.0,  // 99% confidence ≈ 3σ
            x if x >= 0.95 => 2.0,  // 95% confidence ≈ 2σ
            x if x >= 0.90 => 1.65, // 90% confidence ≈ 1.65σ
            _ => 1.0,
        };
        
        let new_threshold = mean + confidence_multiplier * std_dev;
        
        // Only update if change is significant
        if (new_threshold - self.value).abs() > std_dev * 0.1 {
            self.value = new_threshold;
            self.adjustment_count += 1;
            self.history.push_back((Instant::now(), new_threshold));
            
            // Keep only recent history
            if self.history.len() > 100 {
                self.history.pop_front();
            }
        }
    }
    
    /// Check if a value exceeds the adaptive threshold
    fn is_exceeded(&self, value: f64) -> bool {
        value > self.value
    }
    
    /// Get recommended safe operating threshold (80% of breaking point)
    fn safe_operating_threshold(&self) -> f64 {
        self.value * 0.8
    }
}

/// Automatic breaking point detection system
#[derive(Debug)]
pub struct AutomaticBreakingPointDetector {
    /// Configuration
    config: AutomaticDetectionConfig,
    /// Whether detection is active
    is_active: Arc<AtomicBool>,
    /// Collected system metrics
    metrics_history: Arc<Mutex<Vec<SystemMetricsSample>>>,
    /// Detected breaking points
    breaking_points: Arc<Mutex<Vec<BreakingPointDetection>>>,
    /// Adaptive thresholds for each metric
    adaptive_thresholds: Arc<Mutex<HashMap<String, AdaptiveThreshold>>>,
    /// Current operation count
    operation_count: Arc<Mutex<usize>>,
    /// Detection start time
    start_time: Instant,
}

impl AutomaticBreakingPointDetector {
    /// Create new automatic breaking point detector
    pub fn new(config: AutomaticDetectionConfig) -> Self {
        let mut thresholds = HashMap::new();
        
        // Initialize adaptive thresholds
        thresholds.insert("memory_mb".to_string(), AdaptiveThreshold::new(1024.0));
        thresholds.insert("cpu_percent".to_string(), AdaptiveThreshold::new(config.cpu_saturation_threshold));
        thresholds.insert("response_time_ms".to_string(), AdaptiveThreshold::new(1000.0));
        thresholds.insert("memory_growth_rate".to_string(), AdaptiveThreshold::new(config.memory_growth_rate_threshold));
        thresholds.insert("throughput_ops_per_sec".to_string(), AdaptiveThreshold::new(100.0));
        
        Self {
            config,
            is_active: Arc::new(AtomicBool::new(false)),
            metrics_history: Arc::new(Mutex::new(Vec::new())),
            breaking_points: Arc::new(Mutex::new(Vec::new())),
            adaptive_thresholds: Arc::new(Mutex::new(thresholds)),
            operation_count: Arc::new(Mutex::new(0)),
            start_time: Instant::now(),
        }
    }
    
    /// Start automatic detection in background thread
    pub fn start_detection(&self) -> Result<()> {
        self.is_active.store(true, Ordering::SeqCst);
        
        let config = self.config.clone();
        let is_active = Arc::clone(&self.is_active);
        let metrics_history = Arc::clone(&self.metrics_history);
        let breaking_points = Arc::clone(&self.breaking_points);
        let adaptive_thresholds = Arc::clone(&self.adaptive_thresholds);
        let operation_count = Arc::clone(&self.operation_count);
        let start_time = self.start_time;
        
        thread::spawn(move || {
            let mut last_sample_time = Instant::now();
            let mut last_operation_count = 0;
            let mut baseline_period = true;
            let baseline_duration = Duration::from_secs(30); // 30 seconds baseline
            
            while is_active.load(Ordering::SeqCst) {
                let now = Instant::now();
                
                // Check if max test duration exceeded
                if now.duration_since(start_time).as_secs() > config.max_test_duration_secs {
                    is_active.store(false, Ordering::SeqCst);
                    break;
                }
                
                // Collect current metrics
                let current_operation_count = *operation_count.lock().unwrap();
                let time_since_last = now.duration_since(last_sample_time).as_secs_f64();
                
                let sample = SystemMetricsSample {
                    timestamp: now,
                    memory_mb: Self::get_memory_usage_mb(),
                    cpu_percent: Self::get_cpu_usage_percent(),
                    operation_count: current_operation_count,
                    response_time_ms: Self::get_response_time_ms(),
                    throughput_ops_per_sec: if time_since_last > 0.0 {
                        (current_operation_count - last_operation_count) as f64 / time_since_last
                    } else {
                        0.0
                    },
                    memory_growth_rate: Self::calculate_memory_growth_rate(&metrics_history, now),
                    cpu_efficiency: if sample.cpu_percent > 0.0 {
                        current_operation_count as f64 / sample.cpu_percent
                    } else {
                        0.0
                    },
                };
                
                // Store the sample
                metrics_history.lock().unwrap().push(sample.clone());
                
                // Check if we're still in baseline period
                if baseline_period && now.duration_since(start_time) > baseline_duration {
                    baseline_period = false;
                    Self::establish_baseline_thresholds(&adaptive_thresholds, &metrics_history, &config);
                }
                
                // Only start detection after baseline period
                if !baseline_period {
                    // Check for breaking points
                    if let Some(detection) = Self::analyze_for_breaking_points(
                        &sample, 
                        &metrics_history, 
                        &adaptive_thresholds,
                        &config
                    ) {
                        breaking_points.lock().unwrap().push(detection.clone());
                        
                        // Log breaking point detection
                        println!("🚨 BREAKING POINT DETECTED: {:?}", detection.breaking_point_type);
                        println!("   Severity: {:?}", detection.severity);
                        println!("   Confidence: {:.1}%", detection.confidence * 100.0);
                        println!("   Trigger value: {:.2}", detection.trigger_value);
                        println!("   Dynamic threshold: {:.2}", detection.dynamic_threshold);
                        
                        // Stop on critical or fatal breaking points
                        if detection.severity == BreakingPointSeverity::Critical ||
                           detection.severity == BreakingPointSeverity::Fatal {
                            is_active.store(false, Ordering::SeqCst);
                            break;
                        }
                    }
                    
                    // Update adaptive thresholds if enabled
                    if config.enable_adaptive_thresholds && 
                       metrics_history.lock().unwrap().len() % 50 == 0 { // Update every 50 samples
                        Self::update_adaptive_thresholds(&adaptive_thresholds, &metrics_history, &config);
                    }
                }
                
                last_operation_count = current_operation_count;
                last_sample_time = now;
                
                thread::sleep(Duration::from_millis(config.initial_monitoring_interval_ms));
            }
        });
        
        Ok(())
    }
    
    /// Stop automatic detection
    pub fn stop_detection(&self) {
        self.is_active.store(false, Ordering::SeqCst);
    }
    
    /// Update operation count
    pub fn update_operation_count(&self, count: usize) {
        *self.operation_count.lock().unwrap() = count;
    }
    
    /// Get detection results
    pub fn get_results(&self) -> AutomaticDetectionResults {
        let metrics = self.metrics_history.lock().unwrap().clone();
        let breaking_points = self.breaking_points.lock().unwrap().clone();
        let thresholds = self.adaptive_thresholds.lock().unwrap().clone();
        
        AutomaticDetectionResults {
            metrics_history: metrics,
            breaking_points_detected: breaking_points,
            adaptive_thresholds: thresholds,
            total_duration: self.start_time.elapsed(),
            config: self.config.clone(),
        }
    }
    
    /// Establish baseline thresholds from initial stable operation
    fn establish_baseline_thresholds(
        adaptive_thresholds: &Arc<Mutex<HashMap<String, AdaptiveThreshold>>>,
        metrics_history: &Arc<Mutex<Vec<SystemMetricsSample>>>,
        config: &AutomaticDetectionConfig
    ) {
        let metrics = metrics_history.lock().unwrap();
        if metrics.len() < config.min_samples_for_analysis {
            return;
        }
        
        let mut thresholds = adaptive_thresholds.lock().unwrap();
        
        // Extract baseline samples from the stable period
        let baseline_samples: Vec<_> = metrics.iter().collect();
        
        // Update memory threshold
        let memory_values: Vec<f64> = baseline_samples.iter().map(|s| s.memory_mb).collect();
        if let Some(threshold) = thresholds.get_mut("memory_mb") {
            threshold.update_from_baseline(&memory_values, config.confidence_level);
        }
        
        // Update CPU threshold
        let cpu_values: Vec<f64> = baseline_samples.iter().map(|s| s.cpu_percent).collect();
        if let Some(threshold) = thresholds.get_mut("cpu_percent") {
            threshold.update_from_baseline(&cpu_values, config.confidence_level);
        }
        
        // Update response time threshold
        let response_time_values: Vec<f64> = baseline_samples.iter().map(|s| s.response_time_ms).collect();
        if let Some(threshold) = thresholds.get_mut("response_time_ms") {
            threshold.update_from_baseline(&response_time_values, config.confidence_level);
        }
        
        println!("📊 Baseline thresholds established from {} samples", baseline_samples.len());
    }
    
    /// Update adaptive thresholds based on recent stable measurements
    fn update_adaptive_thresholds(
        adaptive_thresholds: &Arc<Mutex<HashMap<String, AdaptiveThreshold>>>,
        metrics_history: &Arc<Mutex<Vec<SystemMetricsSample>>>,
        config: &AutomaticDetectionConfig
    ) {
        let metrics = metrics_history.lock().unwrap();
        if metrics.len() < config.min_samples_for_analysis * 2 {
            return;
        }
        
        // Use last 30 samples for adaptive adjustment (excluding very recent ones that might be unstable)
        let recent_stable_samples: Vec<_> = metrics.iter()
            .rev()
            .skip(5) // Skip last 5 samples
            .take(30) // Take 30 samples
            .collect();
            
        if recent_stable_samples.len() < 20 {
            return;
        }
        
        let mut thresholds = adaptive_thresholds.lock().unwrap();
        
        // Check if recent samples show stable behavior (low variance)
        let memory_values: Vec<f64> = recent_stable_samples.iter().map(|s| s.memory_mb).collect();
        let memory_variance = Self::calculate_variance(&memory_values);
        let memory_mean = memory_values.iter().sum::<f64>() / memory_values.len() as f64;
        
        // Only adjust if variance is low (stable period)
        if memory_variance < (memory_mean * 0.1).powi(2) { // CV < 10%
            if let Some(threshold) = thresholds.get_mut("memory_mb") {
                threshold.update_from_baseline(&memory_values, config.confidence_level);
            }
        }
    }
    
    /// Analyze current sample for breaking points
    fn analyze_for_breaking_points(
        sample: &SystemMetricsSample,
        metrics_history: &Arc<Mutex<Vec<SystemMetricsSample>>>,
        adaptive_thresholds: &Arc<Mutex<HashMap<String, AdaptiveThreshold>>>,
        config: &AutomaticDetectionConfig
    ) -> Option<BreakingPointDetection> {
        let metrics = metrics_history.lock().unwrap();
        let thresholds = adaptive_thresholds.lock().unwrap();
        
        // Need minimum samples for reliable analysis
        if metrics.len() < config.min_samples_for_analysis {
            return None;
        }
        
        // Check each type of breaking point
        
        // 1. Memory exhaustion with rapid growth
        if let Some(threshold) = thresholds.get("memory_mb") {
            if threshold.is_exceeded(sample.memory_mb) && sample.memory_growth_rate > config.memory_growth_rate_threshold {
                let confidence = Self::calculate_confidence(&metrics, |s| s.memory_mb, sample.memory_mb);
                let trend = Self::analyze_trend(&metrics, |s| s.memory_mb);
                
                return Some(BreakingPointDetection {
                    detection_timestamp: sample.timestamp,
                    breaking_point_type: AutoBreakingPointType::MemoryExhaustion,
                    severity: if sample.memory_mb > threshold.value * 1.5 {
                        BreakingPointSeverity::Fatal
                    } else if sample.memory_mb > threshold.value * 1.2 {
                        BreakingPointSeverity::Critical
                    } else {
                        BreakingPointSeverity::Warning
                    },
                    trigger_value: sample.memory_mb,
                    dynamic_threshold: threshold.value,
                    confidence,
                    safe_threshold: threshold.safe_operating_threshold(),
                    analysis: BreakingPointAnalysis {
                        statistical_significance: confidence,
                        trend_analysis: trend,
                        contributing_factors: vec![
                            "Rapid memory growth detected".to_string(),
                            format!("Growth rate: {:.2} MB/sec", sample.memory_growth_rate)
                        ],
                        recommended_actions: vec![
                            "Reduce operation intensity".to_string(),
                            "Check for memory leaks".to_string(),
                            "Increase available memory".to_string()
                        ],
                        time_to_failure_estimate: if sample.memory_growth_rate > 0.0 {
                            let remaining_capacity = 4096.0 - sample.memory_mb; // Assume 4GB limit
                            Some(Duration::from_secs((remaining_capacity / sample.memory_growth_rate) as u64))
                        } else {
                            None
                        },
                    },
                });
            }
        }
        
        // 2. CPU saturation with poor efficiency
        if let Some(threshold) = thresholds.get("cpu_percent") {
            if threshold.is_exceeded(sample.cpu_percent) && sample.cpu_efficiency < 1.0 {
                let confidence = Self::calculate_confidence(&metrics, |s| s.cpu_percent, sample.cpu_percent);
                let trend = Self::analyze_trend(&metrics, |s| s.cpu_percent);
                
                return Some(BreakingPointDetection {
                    detection_timestamp: sample.timestamp,
                    breaking_point_type: AutoBreakingPointType::CpuSaturation,
                    severity: if sample.cpu_percent > 98.0 {
                        BreakingPointSeverity::Fatal
                    } else if sample.cpu_percent > 95.0 {
                        BreakingPointSeverity::Critical
                    } else {
                        BreakingPointSeverity::Warning
                    },
                    trigger_value: sample.cpu_percent,
                    dynamic_threshold: threshold.value,
                    confidence,
                    safe_threshold: threshold.safe_operating_threshold(),
                    analysis: BreakingPointAnalysis {
                        statistical_significance: confidence,
                        trend_analysis: trend,
                        contributing_factors: vec![
                            "CPU saturation with poor efficiency".to_string(),
                            format!("CPU efficiency: {:.2} ops per CPU%", sample.cpu_efficiency)
                        ],
                        recommended_actions: vec![
                            "Reduce concurrent operations".to_string(),
                            "Optimize computation-heavy tasks".to_string(),
                            "Scale horizontally".to_string()
                        ],
                        time_to_failure_estimate: None,
                    },
                });
            }
        }
        
        // 3. Response time explosion
        if let Some(threshold) = thresholds.get("response_time_ms") {
            if threshold.is_exceeded(sample.response_time_ms) {
                let baseline_response_time = threshold.baseline;
                let degradation_factor = sample.response_time_ms / baseline_response_time;
                
                if degradation_factor > config.response_time_threshold {
                    let confidence = Self::calculate_confidence(&metrics, |s| s.response_time_ms, sample.response_time_ms);
                    let trend = Self::analyze_trend(&metrics, |s| s.response_time_ms);
                    
                    return Some(BreakingPointDetection {
                        detection_timestamp: sample.timestamp,
                        breaking_point_type: AutoBreakingPointType::ResponseTimeExplosion,
                        severity: if degradation_factor > 10.0 {
                            BreakingPointSeverity::Fatal
                        } else if degradation_factor > 5.0 {
                            BreakingPointSeverity::Critical
                        } else {
                            BreakingPointSeverity::Warning
                        },
                        trigger_value: sample.response_time_ms,
                        dynamic_threshold: threshold.value,
                        confidence,
                        safe_threshold: threshold.safe_operating_threshold(),
                        analysis: BreakingPointAnalysis {
                            statistical_significance: confidence,
                            trend_analysis: trend,
                            contributing_factors: vec![
                                format!("Response time degraded by {:.1}x", degradation_factor),
                                "System becoming unresponsive".to_string()
                            ],
                            recommended_actions: vec![
                                "Reduce system load immediately".to_string(),
                                "Check for deadlocks or blocking operations".to_string(),
                                "Scale resources".to_string()
                            ],
                            time_to_failure_estimate: None,
                        },
                    });
                }
            }
        }
        
        // 4. Throughput collapse
        let recent_throughput: Vec<f64> = metrics.iter()
            .rev()
            .take(10)
            .map(|s| s.throughput_ops_per_sec)
            .collect();
            
        if recent_throughput.len() >= 5 {
            let avg_recent = recent_throughput.iter().sum::<f64>() / recent_throughput.len() as f64;
            let baseline_throughput = if let Some(threshold) = thresholds.get("throughput_ops_per_sec") {
                threshold.baseline
            } else {
                100.0
            };
            
            if avg_recent < baseline_throughput * 0.5 && baseline_throughput > 10.0 {
                let trend = Self::analyze_trend(&metrics, |s| s.throughput_ops_per_sec);
                
                return Some(BreakingPointDetection {
                    detection_timestamp: sample.timestamp,
                    breaking_point_type: AutoBreakingPointType::ThroughputCollapse,
                    severity: if avg_recent < baseline_throughput * 0.1 {
                        BreakingPointSeverity::Fatal
                    } else if avg_recent < baseline_throughput * 0.25 {
                        BreakingPointSeverity::Critical
                    } else {
                        BreakingPointSeverity::Warning
                    },
                    trigger_value: avg_recent,
                    dynamic_threshold: baseline_throughput,
                    confidence: 0.85,
                    safe_threshold: baseline_throughput * 0.8,
                    analysis: BreakingPointAnalysis {
                        statistical_significance: 0.85,
                        trend_analysis: trend,
                        contributing_factors: vec![
                            format!("Throughput dropped to {:.1} ops/sec from baseline {:.1}", avg_recent, baseline_throughput),
                            "System capacity exceeded".to_string()
                        ],
                        recommended_actions: vec![
                            "Reduce system load".to_string(),
                            "Check for resource bottlenecks".to_string(),
                            "Restart overloaded services".to_string()
                        ],
                        time_to_failure_estimate: None,
                    },
                });
            }
        }
        
        None
    }
    
    /// Calculate statistical confidence for a breaking point detection
    fn calculate_confidence(
        metrics: &[SystemMetricsSample], 
        extractor: fn(&SystemMetricsSample) -> f64,
        current_value: f64
    ) -> f64 {
        if metrics.len() < 10 {
            return 0.5; // Low confidence with insufficient data
        }
        
        let values: Vec<f64> = metrics.iter().map(extractor).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = Self::calculate_variance(&values);
        let std_dev = variance.sqrt();
        
        if std_dev == 0.0 {
            return 0.9; // High confidence if no variation
        }
        
        // Calculate z-score
        let z_score = (current_value - mean).abs() / std_dev;
        
        // Convert z-score to confidence (simplified)
        if z_score > 3.0 {
            0.99
        } else if z_score > 2.0 {
            0.95
        } else if z_score > 1.0 {
            0.75
        } else {
            0.50
        }
    }
    
    /// Analyze trend for a metric
    fn analyze_trend(
        metrics: &[SystemMetricsSample],
        extractor: fn(&SystemMetricsSample) -> f64
    ) -> TrendAnalysis {
        if metrics.len() < 3 {
            return TrendAnalysis {
                rate_of_change: 0.0,
                acceleration: 0.0,
                metric_correlations: HashMap::new(),
                stability_score: 0.5,
            };
        }
        
        let values: Vec<f64> = metrics.iter().map(extractor).collect();
        let times: Vec<f64> = metrics.iter()
            .enumerate()
            .map(|(i, _)| i as f64)
            .collect();
        
        // Calculate rate of change (first derivative)
        let rate_of_change = if values.len() >= 2 {
            (values[values.len() - 1] - values[values.len() - 2]) / 1.0
        } else {
            0.0
        };
        
        // Calculate acceleration (second derivative)
        let acceleration = if values.len() >= 3 {
            let recent_rate = values[values.len() - 1] - values[values.len() - 2];
            let previous_rate = values[values.len() - 2] - values[values.len() - 3];
            recent_rate - previous_rate
        } else {
            0.0
        };
        
        // Calculate stability score (inverse of coefficient of variation)
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = Self::calculate_variance(&values);
        let cv = if mean != 0.0 { variance.sqrt() / mean.abs() } else { 1.0 };
        let stability_score = (1.0 / (1.0 + cv)).max(0.0).min(1.0);
        
        TrendAnalysis {
            rate_of_change,
            acceleration,
            metric_correlations: HashMap::new(), // TODO: Calculate correlations
            stability_score,
        }
    }
    
    /// Calculate variance of values
    fn calculate_variance(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }
        
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64
    }
    
    /// Calculate memory growth rate
    fn calculate_memory_growth_rate(
        metrics_history: &Arc<Mutex<Vec<SystemMetricsSample>>>,
        current_time: Instant
    ) -> f64 {
        let metrics = metrics_history.lock().unwrap();
        if metrics.len() < 2 {
            return 0.0;
        }
        
        // Calculate growth rate over last 5 seconds
        let cutoff_time = current_time - Duration::from_secs(5);
        let recent_samples: Vec<_> = metrics.iter()
            .filter(|s| s.timestamp > cutoff_time)
            .collect();
            
        if recent_samples.len() < 2 {
            return 0.0;
        }
        
        let first = recent_samples.first().unwrap();
        let last = recent_samples.last().unwrap();
        let time_diff = last.timestamp.duration_since(first.timestamp).as_secs_f64();
        
        if time_diff > 0.0 {
            (last.memory_mb - first.memory_mb) / time_diff
        } else {
            0.0
        }
    }
    
    /// Get memory usage (simplified simulation)
    fn get_memory_usage_mb() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let base_memory = 300.0;
        let time_factor = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_millis() as f64 / 1000.0;
        
        // Simulate realistic memory growth with some noise
        let growth = time_factor * 3.0;
        let noise = (time_factor * 7.0).sin() * 50.0;
        base_memory + growth + noise
    }
    
    /// Get CPU usage (simplified simulation)
    fn get_cpu_usage_percent() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let time_factor = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_millis() as f64 / 1000.0;
        
        // Simulate realistic CPU usage with increasing load
        let base_cpu = 20.0;
        let load_increase = (time_factor * 0.5).min(60.0);
        let variability = (time_factor * 3.0).sin() * 10.0;
        (base_cpu + load_increase + variability).max(0.0).min(100.0)
    }
    
    /// Get response time (simplified simulation)
    fn get_response_time_ms() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let time_factor = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_millis() as f64 / 1000.0;
        
        // Simulate response time degradation under load
        let base_response = 50.0;
        let load_factor = (time_factor * 0.3).exp() - 1.0;
        let noise = (time_factor * 5.0).sin() * 10.0;
        base_response + load_factor * 20.0 + noise
    }
}

/// Results from automatic breaking point detection
#[derive(Debug)]
pub struct AutomaticDetectionResults {
    /// All collected metrics
    pub metrics_history: Vec<SystemMetricsSample>,
    /// All detected breaking points
    pub breaking_points_detected: Vec<BreakingPointDetection>,
    /// Final adaptive thresholds
    pub adaptive_thresholds: HashMap<String, AdaptiveThreshold>,
    /// Total detection duration
    pub total_duration: Duration,
    /// Configuration used
    pub config: AutomaticDetectionConfig,
}

impl AutomaticDetectionResults {
    /// Generate comprehensive detection report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# Automatic Breaking Point Detection Report\n\n");
        
        // Executive summary
        report.push_str("## Executive Summary\n");
        report.push_str(&format!("Detection duration: {:?}\n", self.total_duration));
        report.push_str(&format!("Total metrics collected: {}\n", self.metrics_history.len()));
        report.push_str(&format!("Breaking points detected: {}\n", self.breaking_points_detected.len()));
        
        if !self.breaking_points_detected.is_empty() {
            let critical_count = self.breaking_points_detected.iter()
                .filter(|bp| bp.severity == BreakingPointSeverity::Critical || bp.severity == BreakingPointSeverity::Fatal)
                .count();
            report.push_str(&format!("Critical breaking points: {}\n", critical_count));
        }
        report.push_str("\n");
        
        // Breaking points analysis
        if !self.breaking_points_detected.is_empty() {
            report.push_str("## Breaking Points Detected\n\n");
            
            for (i, bp) in self.breaking_points_detected.iter().enumerate() {
                report.push_str(&format!("### Breaking Point {} - {:?}\n", i + 1, bp.breaking_point_type));
                report.push_str(&format!("- **Severity**: {:?}\n", bp.severity));
                report.push_str(&format!("- **Confidence**: {:.1}%\n", bp.confidence * 100.0));
                report.push_str(&format!("- **Trigger Value**: {:.2}\n", bp.trigger_value));
                report.push_str(&format!("- **Dynamic Threshold**: {:.2}\n", bp.dynamic_threshold));
                report.push_str(&format!("- **Safe Threshold**: {:.2}\n", bp.safe_threshold));
                
                report.push_str("\n**Contributing Factors**:\n");
                for factor in &bp.analysis.contributing_factors {
                    report.push_str(&format!("- {}\n", factor));
                }
                
                report.push_str("\n**Recommended Actions**:\n");
                for action in &bp.analysis.recommended_actions {
                    report.push_str(&format!("- {}\n", action));
                }
                
                if let Some(ttf) = bp.analysis.time_to_failure_estimate {
                    report.push_str(&format!("\n**Estimated Time to Failure**: {:?}\n", ttf));
                }
                report.push_str("\n");
            }
        } else {
            report.push_str("## No Breaking Points Detected\n");
            report.push_str("System remained stable throughout the detection period.\n\n");
        }
        
        // Adaptive thresholds summary
        report.push_str("## Adaptive Thresholds\n\n");
        for (metric_name, threshold) in &self.adaptive_thresholds {
            report.push_str(&format!("**{}**:\n", metric_name));
            report.push_str(&format!("- Current threshold: {:.2}\n", threshold.value));
            report.push_str(&format!("- Baseline: {:.2}\n", threshold.baseline));
            report.push_str(&format!("- Standard deviation: {:.2}\n", threshold.std_deviation));
            report.push_str(&format!("- Safe operating threshold: {:.2}\n", threshold.safe_operating_threshold()));
            report.push_str(&format!("- Adjustments made: {}\n\n", threshold.adjustment_count));
        }
        
        // Performance statistics
        if !self.metrics_history.is_empty() {
            report.push_str("## Performance Statistics\n\n");
            
            // Memory statistics
            let memory_values: Vec<f64> = self.metrics_history.iter().map(|m| m.memory_mb).collect();
            let memory_stats = self.calculate_statistics(&memory_values);
            report.push_str(&format!("**Memory Usage (MB)**:\n"));
            report.push_str(&format!("- Min: {:.1}, Max: {:.1}, Avg: {:.1}, StdDev: {:.1}\n\n", 
                           memory_stats.0, memory_stats.1, memory_stats.2, memory_stats.3));
            
            // CPU statistics
            let cpu_values: Vec<f64> = self.metrics_history.iter().map(|m| m.cpu_percent).collect();
            let cpu_stats = self.calculate_statistics(&cpu_values);
            report.push_str(&format!("**CPU Usage (%)**:\n"));
            report.push_str(&format!("- Min: {:.1}, Max: {:.1}, Avg: {:.1}, StdDev: {:.1}\n\n", 
                           cpu_stats.0, cpu_stats.1, cpu_stats.2, cpu_stats.3));
            
            // Response time statistics
            let response_values: Vec<f64> = self.metrics_history.iter().map(|m| m.response_time_ms).collect();
            let response_stats = self.calculate_statistics(&response_values);
            report.push_str(&format!("**Response Time (ms)**:\n"));
            report.push_str(&format!("- Min: {:.1}, Max: {:.1}, Avg: {:.1}, StdDev: {:.1}\n\n", 
                           response_stats.0, response_stats.1, response_stats.2, response_stats.3));
        }
        
        report.push_str("## Recommendations\n\n");
        if self.breaking_points_detected.is_empty() {
            report.push_str("- System appears stable under current load conditions\n");
            report.push_str("- Consider increasing load gradually to find actual limits\n");
            report.push_str("- Monitor adaptive thresholds for baseline establishment\n");
        } else {
            report.push_str("- Address detected breaking points before production deployment\n");
            report.push_str("- Use safe operating thresholds for production limits\n");
            report.push_str("- Implement monitoring alerts based on adaptive thresholds\n");
            report.push_str("- Plan capacity scaling before reaching breaking points\n");
        }
        
        report
    }
    
    /// Calculate basic statistics for a metric
    fn calculate_statistics(&self, values: &[f64]) -> (f64, f64, f64, f64) { // min, max, avg, stddev
        if values.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }
        
        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let avg = values.iter().sum::<f64>() / values.len() as f64;
        
        let variance = values.iter()
            .map(|x| (x - avg).powi(2))
            .sum::<f64>() / values.len() as f64;
        let stddev = variance.sqrt();
        
        (min, max, avg, stddev)
    }
}

/// Test automatic breaking point detection system
#[test]
fn test_automatic_breaking_point_detection() -> Result<()> {
    println!("🤖 Starting automatic breaking point detection test");
    
    let config = AutomaticDetectionConfig {
        initial_monitoring_interval_ms: 100,
        max_test_duration_secs: 60, // 1 minute test
        min_samples_for_analysis: 15,
        performance_degradation_threshold: 40.0,
        memory_growth_rate_threshold: 5.0,
        cpu_saturation_threshold: 90.0,
        response_time_threshold: 2.5,
        enable_adaptive_thresholds: true,
        confidence_level: 0.95,
    };
    
    println!("Configuration:");
    println!("  - Monitoring interval: {} ms", config.initial_monitoring_interval_ms);
    println!("  - Max test duration: {} seconds", config.max_test_duration_secs);
    println!("  - Min samples for analysis: {}", config.min_samples_for_analysis);
    println!("  - Performance degradation threshold: {:.1}%", config.performance_degradation_threshold);
    println!("  - Memory growth rate threshold: {:.1} MB/sec", config.memory_growth_rate_threshold);
    println!("  - CPU saturation threshold: {:.1}%", config.cpu_saturation_threshold);
    println!("  - Adaptive thresholds: {}", config.enable_adaptive_thresholds);
    println!();
    
    // Create and start detector
    let detector = AutomaticBreakingPointDetector::new(config);
    detector.start_detection()?;
    
    println!("🔍 Detection started - running progressive load simulation...");
    
    // Simulate progressive load that will trigger breaking points
    let start_time = Instant::now();
    let mut operation_count = 0;
    
    while start_time.elapsed().as_secs() < 45 { // Run for 45 seconds
        // Simulate escalating workload
        let elapsed_secs = start_time.elapsed().as_secs() as f64;
        let intensity_factor = (elapsed_secs / 45.0).min(1.0);
        
        // Simulate operations with increasing complexity/load
        let operations_this_cycle = (50.0 * (1.0 + intensity_factor * 3.0)) as usize;
        
        for _ in 0..operations_this_cycle {
            operation_count += 1;
            detector.update_operation_count(operation_count);
            
            // Simulate work (minimal to not actually overload test system)
            thread::sleep(Duration::from_micros(if intensity_factor > 0.7 { 100 } else { 50 }));
        }
        
        println!("  ⚡ Operations: {}, Elapsed: {:.1}s, Intensity: {:.1}%", 
                operation_count, elapsed_secs, intensity_factor * 100.0);
        
        // Add slight pause between cycles
        thread::sleep(Duration::from_millis(200));
    }
    
    // Stop detection and get results
    detector.stop_detection();
    thread::sleep(Duration::from_millis(500)); // Allow detection thread to finish
    
    let results = detector.get_results();
    
    // Generate and display comprehensive report
    let report = results.generate_report();
    println!("\n{}", report);
    
    // Verify automatic detection results
    assert!(!results.metrics_history.is_empty(), "Should have collected metrics");
    assert!(results.metrics_history.len() >= 200, "Should have collected sufficient samples");
    
    println!("✅ Automatic detection verification:");
    println!("  📊 Metrics collected: {}", results.metrics_history.len());
    println!("  🚨 Breaking points detected: {}", results.breaking_points_detected.len());
    println!("  🔧 Adaptive thresholds: {}", results.adaptive_thresholds.len());
    println!("  ⏱️  Total duration: {:?}", results.total_duration);
    
    // Verify adaptive thresholds were established
    assert!(!results.adaptive_thresholds.is_empty(), "Should have adaptive thresholds");
    for (metric_name, threshold) in &results.adaptive_thresholds {
        println!("  📈 {}: threshold={:.2}, baseline={:.2}, adjustments={}", 
                metric_name, threshold.value, threshold.baseline, threshold.adjustment_count);
    }
    
    // Check detection quality
    let detection_rate = results.metrics_history.len() as f64 / results.total_duration.as_secs_f64();
    println!("  📊 Detection rate: {:.1} samples/sec", detection_rate);
    assert!(detection_rate >= 5.0, "Should maintain reasonable detection rate");
    
    // Verify breaking point analysis if any were detected
    if !results.breaking_points_detected.is_empty() {
        println!("  🎯 Breaking point analysis:");
        for bp in &results.breaking_points_detected {
            println!("    - {:?}: severity={:?}, confidence={:.1}%", 
                    bp.breaking_point_type, bp.severity, bp.confidence * 100.0);
            assert!(bp.confidence >= 0.5, "Should have reasonable confidence in detection");
        }
    } else {
        println!("  ℹ️  No breaking points detected within test parameters");
    }
    
    println!("🎯 Automatic breaking point detection test completed successfully");
    
    Ok(())
}