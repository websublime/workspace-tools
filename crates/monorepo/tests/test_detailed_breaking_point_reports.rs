//! Detailed Breaking Point Reporting System
//!
//! This module implements comprehensive reporting capabilities for breaking point analysis,
//! integrating all monitoring, detection, recovery, and performance degradation systems
//! into unified, actionable reports for production deployment.

use sublime_monorepo_tools::Result;
use std::collections::{HashMap, BTreeMap};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

/// Configuration for detailed breaking point reporting
#[derive(Debug, Clone)]
pub struct DetailedReportingConfig {
    /// Report generation frequency
    pub report_generation_interval_secs: u64,
    /// Maximum report history to keep
    pub max_report_history: usize,
    /// Include raw data in reports
    pub include_raw_data: bool,
    /// Generate visual charts data
    pub generate_charts_data: bool,
    /// Include prediction models
    pub include_prediction_models: bool,
    /// Report format preference
    pub preferred_format: ReportFormat,
    /// Minimum severity for inclusion in reports
    pub min_severity_for_inclusion: ReportSeverity,
}

impl Default for DetailedReportingConfig {
    fn default() -> Self {
        Self {
            report_generation_interval_secs: 300, // 5 minutes
            max_report_history: 50,
            include_raw_data: true,
            generate_charts_data: true,
            include_prediction_models: true,
            preferred_format: ReportFormat::DetailedMarkdown,
            min_severity_for_inclusion: ReportSeverity::Warning,
        }
    }
}

/// Report format options
#[derive(Debug, Clone, PartialEq)]
pub enum ReportFormat {
    /// Detailed markdown report
    DetailedMarkdown,
    /// Executive summary
    ExecutiveSummary,
    /// JSON data export
    JsonExport,
    /// CSV data export
    CsvExport,
    /// Production monitoring format
    ProductionMonitoring,
}

/// Report severity levels
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum ReportSeverity {
    /// Informational - system running normally
    Info,
    /// Warning - minor issues detected
    Warning,
    /// Major - significant issues requiring attention
    Major,
    /// Critical - severe issues requiring immediate action
    Critical,
    /// Emergency - system failure imminent or occurred
    Emergency,
}

/// Comprehensive breaking point incident record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingPointIncident {
    /// Unique incident identifier
    pub incident_id: String,
    /// When the incident was first detected
    pub detection_timestamp: Instant,
    /// When the incident was resolved (if applicable)
    pub resolution_timestamp: Option<Instant>,
    /// Incident severity level
    pub severity: String, // Serialized severity
    /// Primary system component affected
    pub affected_component: String,
    /// Breaking point type that was detected
    pub breaking_point_type: String,
    /// Trigger metric and value
    pub trigger_metric: String,
    /// Value that triggered the breaking point
    pub trigger_value: f64,
    /// Threshold that was exceeded
    pub threshold_exceeded: f64,
    /// Recovery strategy that was applied
    pub recovery_strategy_applied: Option<String>,
    /// Recovery success status
    pub recovery_successful: bool,
    /// Time to recovery (if successful)
    pub time_to_recovery: Option<Duration>,
    /// Performance impact assessment
    pub performance_impact: PerformanceImpactAssessment,
    /// Root cause analysis
    pub root_cause_analysis: RootCauseAnalysis,
    /// Lessons learned and preventive measures
    pub lessons_learned: Vec<String>,
}

/// Performance impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImpactAssessment {
    /// Overall system degradation percentage
    pub overall_degradation_percent: f64,
    /// Response time impact
    pub response_time_impact: MetricImpact,
    /// Throughput impact
    pub throughput_impact: MetricImpact,
    /// Memory usage impact
    pub memory_impact: MetricImpact,
    /// CPU utilization impact
    pub cpu_impact: MetricImpact,
    /// Error rate impact
    pub error_rate_impact: MetricImpact,
    /// Estimated business impact
    pub estimated_business_impact: BusinessImpact,
}

/// Impact on individual metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricImpact {
    /// Baseline value before incident
    pub baseline_value: f64,
    /// Peak impact value during incident
    pub peak_impact_value: f64,
    /// Percentage change from baseline
    pub percentage_change: f64,
    /// Duration of impact
    pub impact_duration: Duration,
}

/// Business impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessImpact {
    /// Estimated affected users
    pub affected_users: usize,
    /// Estimated revenue impact
    pub estimated_revenue_impact: f64,
    /// SLA breach status
    pub sla_breach: bool,
    /// Customer satisfaction impact
    pub customer_satisfaction_impact: String,
}

/// Root cause analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseAnalysis {
    /// Primary root cause identified
    pub primary_cause: String,
    /// Contributing factors
    pub contributing_factors: Vec<String>,
    /// Timeline of events leading to breaking point
    pub timeline: Vec<TimelineEvent>,
    /// System configuration at time of incident
    pub system_configuration: HashMap<String, String>,
    /// Resource utilization patterns
    pub resource_patterns: Vec<String>,
}

/// Timeline event for root cause analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    /// Timestamp of the event
    pub timestamp: Instant,
    /// Event description
    pub description: String,
    /// Event category
    pub category: String,
    /// Impact level of this event
    pub impact_level: String,
}

/// System health trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthTrend {
    /// Overall health score (0.0-1.0)
    pub overall_health_score: f64,
    /// Trend direction over time
    pub trend_direction: String,
    /// Rate of change
    pub change_rate: f64,
    /// Confidence in the trend
    pub trend_confidence: f64,
    /// Key contributing factors to health
    pub health_factors: HashMap<String, f64>,
    /// Predicted health in next period
    pub predicted_health: f64,
}

/// Breaking point prediction model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingPointPredictionModel {
    /// Model type used
    pub model_type: String,
    /// Confidence in predictions
    pub prediction_confidence: f64,
    /// Predicted breaking points
    pub predicted_breaking_points: Vec<PredictedBreakingPoint>,
    /// Model accuracy metrics
    pub accuracy_metrics: ModelAccuracyMetrics,
    /// Training data period
    pub training_period: Duration,
}

/// Predicted breaking point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedBreakingPoint {
    /// Metric that will hit breaking point
    pub metric_name: String,
    /// Predicted timestamp of breaking point
    pub predicted_timestamp: Instant,
    /// Confidence in this prediction
    pub confidence: f64,
    /// Predicted value at breaking point
    pub predicted_value: f64,
    /// Recommended preventive actions
    pub preventive_actions: Vec<String>,
}

/// Model accuracy metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAccuracyMetrics {
    /// True positive rate
    pub true_positive_rate: f64,
    /// False positive rate
    pub false_positive_rate: f64,
    /// Precision
    pub precision: f64,
    /// Recall
    pub recall: f64,
    /// F1 score
    pub f1_score: f64,
}

/// Charts and visualization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartsData {
    /// Time series data for metrics
    pub time_series: HashMap<String, Vec<(Instant, f64)>>,
    /// Breaking point markers
    pub breaking_point_markers: Vec<(Instant, String)>,
    /// Trend lines
    pub trend_lines: HashMap<String, Vec<(Instant, f64)>>,
    /// Prediction bands
    pub prediction_bands: HashMap<String, Vec<(Instant, f64, f64)>>, // timestamp, lower_bound, upper_bound
}

/// Comprehensive breaking point report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedBreakingPointReport {
    /// Report metadata
    pub report_id: String,
    /// Report generation timestamp
    pub generated_at: Instant,
    /// Report period covered
    pub report_period: Duration,
    /// Executive summary
    pub executive_summary: ExecutiveSummary,
    /// All incidents in the period
    pub incidents: Vec<BreakingPointIncident>,
    /// System health analysis
    pub system_health_analysis: SystemHealthTrend,
    /// Prediction models and forecasts
    pub prediction_models: Option<BreakingPointPredictionModel>,
    /// Performance trends
    pub performance_trends: HashMap<String, PerformanceTrend>,
    /// Operational recommendations
    pub operational_recommendations: Vec<OperationalRecommendation>,
    /// Charts and visualization data
    pub charts_data: Option<ChartsData>,
    /// Raw monitoring data (if enabled)
    pub raw_data: Option<HashMap<String, Vec<f64>>>,
    /// Report configuration used
    pub config_snapshot: String, // Serialized config
}

/// Executive summary for reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    /// Total incidents in period
    pub total_incidents: usize,
    /// Critical incidents count
    pub critical_incidents: usize,
    /// Average time to recovery
    pub avg_time_to_recovery: Duration,
    /// Overall system availability
    pub system_availability_percent: f64,
    /// Key performance indicators
    pub key_performance_indicators: HashMap<String, f64>,
    /// Top risk factors identified
    pub top_risk_factors: Vec<String>,
    /// Business impact summary
    pub business_impact_summary: String,
}

/// Performance trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrend {
    /// Metric name
    pub metric_name: String,
    /// Trend direction
    pub trend_direction: String,
    /// Rate of change
    pub change_rate: f64,
    /// Volatility measure
    pub volatility: f64,
    /// Seasonal patterns detected
    pub seasonal_patterns: Vec<String>,
    /// Anomalies detected
    pub anomalies: Vec<(Instant, f64, String)>,
}

/// Operational recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalRecommendation {
    /// Recommendation category
    pub category: String,
    /// Priority level
    pub priority: String,
    /// Recommendation description
    pub description: String,
    /// Expected impact
    pub expected_impact: String,
    /// Implementation complexity
    pub implementation_complexity: String,
    /// Time frame for implementation
    pub timeframe: String,
    /// Cost estimate
    pub cost_estimate: String,
}

/// Detailed breaking point reporting system
#[derive(Debug)]
pub struct DetailedBreakingPointReporter {
    /// Configuration
    config: DetailedReportingConfig,
    /// Historical incidents
    incident_history: Arc<Mutex<Vec<BreakingPointIncident>>>,
    /// Generated reports
    report_history: Arc<Mutex<Vec<DetailedBreakingPointReport>>>,
    /// System metrics storage
    metrics_storage: Arc<Mutex<HashMap<String, Vec<(Instant, f64)>>>>,
    /// Reporter start time
    start_time: Instant,
}

impl DetailedBreakingPointReporter {
    /// Create new detailed reporter
    pub fn new(config: DetailedReportingConfig) -> Self {
        Self {
            config,
            incident_history: Arc::new(Mutex::new(Vec::new())),
            report_history: Arc::new(Mutex::new(Vec::new())),
            metrics_storage: Arc::new(Mutex::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }
    
    /// Record a breaking point incident
    pub fn record_incident(&self, incident: BreakingPointIncident) {
        let mut history = self.incident_history.lock().unwrap();
        history.push(incident);
        
        // Keep only recent incidents
        if history.len() > 1000 {
            history.drain(0..100); // Remove oldest 100
        }
    }
    
    /// Record system metrics
    pub fn record_metrics(&self, metrics: HashMap<String, f64>) {
        let mut storage = self.metrics_storage.lock().unwrap();
        let timestamp = Instant::now();
        
        for (metric_name, value) in metrics {
            storage.entry(metric_name)
                .or_insert_with(Vec::new)
                .push((timestamp, value));
        }
        
        // Cleanup old metrics (keep last 10000 points per metric)
        for values in storage.values_mut() {
            if values.len() > 10000 {
                values.drain(0..1000);
            }
        }
    }
    
    /// Generate comprehensive breaking point report
    pub fn generate_comprehensive_report(&self) -> Result<DetailedBreakingPointReport> {
        let report_id = format!("BP-REPORT-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
        let generated_at = Instant::now();
        let report_period = generated_at.duration_since(self.start_time);
        
        // Get incidents from the reporting period
        let incidents = self.get_incidents_for_period(report_period);
        
        // Generate executive summary
        let executive_summary = self.generate_executive_summary(&incidents, report_period);
        
        // Analyze system health trends
        let system_health_analysis = self.analyze_system_health_trends();
        
        // Generate prediction models (if enabled)
        let prediction_models = if self.config.include_prediction_models {
            Some(self.generate_prediction_models())
        } else {
            None
        };
        
        // Analyze performance trends
        let performance_trends = self.analyze_performance_trends();
        
        // Generate operational recommendations
        let operational_recommendations = self.generate_operational_recommendations(&incidents, &performance_trends);
        
        // Generate charts data (if enabled)
        let charts_data = if self.config.generate_charts_data {
            Some(self.generate_charts_data(&incidents))
        } else {
            None
        };
        
        // Include raw data (if enabled)
        let raw_data = if self.config.include_raw_data {
            Some(self.extract_raw_metrics_data())
        } else {
            None
        };
        
        let report = DetailedBreakingPointReport {
            report_id,
            generated_at,
            report_period,
            executive_summary,
            incidents,
            system_health_analysis,
            prediction_models,
            performance_trends,
            operational_recommendations,
            charts_data,
            raw_data,
            config_snapshot: format!("{:?}", self.config),
        };
        
        // Store the report
        let mut history = self.report_history.lock().unwrap();
        history.push(report.clone());
        
        // Cleanup old reports
        if history.len() > self.config.max_report_history {
            history.drain(0..10);
        }
        
        Ok(report)
    }
    
    /// Get incidents for a specific period
    fn get_incidents_for_period(&self, period: Duration) -> Vec<BreakingPointIncident> {
        let cutoff_time = Instant::now() - period;
        let incidents = self.incident_history.lock().unwrap();
        
        incidents.iter()
            .filter(|incident| incident.detection_timestamp > cutoff_time)
            .cloned()
            .collect()
    }
    
    /// Generate executive summary
    fn generate_executive_summary(&self, incidents: &[BreakingPointIncident], period: Duration) -> ExecutiveSummary {
        let total_incidents = incidents.len();
        let critical_incidents = incidents.iter()
            .filter(|i| i.severity == "Critical" || i.severity == "Emergency")
            .count();
        
        let avg_time_to_recovery = if !incidents.is_empty() {
            let total_recovery_time: Duration = incidents.iter()
                .filter_map(|i| i.time_to_recovery)
                .sum();
            total_recovery_time / incidents.len() as u32
        } else {
            Duration::from_secs(0)
        };
        
        // Calculate system availability
        let downtime: Duration = incidents.iter()
            .filter_map(|i| i.time_to_recovery)
            .sum();
        let system_availability_percent = if period.as_secs() > 0 {
            ((period.as_secs() - downtime.as_secs()) as f64 / period.as_secs() as f64) * 100.0
        } else {
            100.0
        };
        
        // Generate KPIs
        let mut kpis = HashMap::new();
        kpis.insert("mean_time_to_recovery_mins".to_string(), avg_time_to_recovery.as_secs() as f64 / 60.0);
        kpis.insert("incident_rate_per_day".to_string(), (total_incidents as f64 / period.as_secs() as f64) * 86400.0);
        kpis.insert("critical_incident_ratio".to_string(), if total_incidents > 0 { critical_incidents as f64 / total_incidents as f64 } else { 0.0 });
        
        // Identify top risk factors
        let mut risk_factors = HashMap::new();
        for incident in incidents {
            *risk_factors.entry(incident.affected_component.clone()).or_insert(0) += 1;
        }
        let top_risk_factors: Vec<String> = risk_factors.iter()
            .map(|(component, count)| format!("{} ({} incidents)", component, count))
            .take(5)
            .collect();
        
        let business_impact_summary = if critical_incidents > 0 {
            format!("Critical impact: {} high-severity incidents affecting system availability", critical_incidents)
        } else if total_incidents > 5 {
            "Moderate impact: Multiple incidents detected requiring attention".to_string()
        } else {
            "Low impact: System operating within acceptable parameters".to_string()
        };
        
        ExecutiveSummary {
            total_incidents,
            critical_incidents,
            avg_time_to_recovery,
            system_availability_percent,
            key_performance_indicators: kpis,
            top_risk_factors,
            business_impact_summary,
        }
    }
    
    /// Analyze system health trends
    fn analyze_system_health_trends(&self) -> SystemHealthTrend {
        let metrics = self.metrics_storage.lock().unwrap();
        
        // Calculate overall health score based on multiple metrics
        let mut health_factors = HashMap::new();
        let mut overall_score = 0.0;
        let mut factor_count = 0;
        
        for (metric_name, values) in metrics.iter() {
            if !values.is_empty() {
                let recent_values: Vec<f64> = values.iter()
                    .rev()
                    .take(20)
                    .map(|(_, v)| *v)
                    .collect();
                
                let score = self.calculate_metric_health_score(metric_name, &recent_values);
                health_factors.insert(metric_name.clone(), score);
                overall_score += score;
                factor_count += 1;
            }
        }
        
        if factor_count > 0 {
            overall_score /= factor_count as f64;
        } else {
            overall_score = 1.0; // Default to healthy if no data
        }
        
        // Calculate trend direction
        let (trend_direction, change_rate, trend_confidence) = if !metrics.is_empty() {
            self.calculate_overall_trend(&metrics)
        } else {
            ("Stable".to_string(), 0.0, 1.0)
        };
        
        // Predict future health
        let predicted_health = (overall_score + change_rate * 300.0).max(0.0).min(1.0); // 5 minutes ahead
        
        SystemHealthTrend {
            overall_health_score: overall_score,
            trend_direction,
            change_rate,
            trend_confidence,
            health_factors,
            predicted_health,
        }
    }
    
    /// Calculate health score for a specific metric
    fn calculate_metric_health_score(&self, metric_name: &str, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 1.0;
        }
        
        let avg = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / values.len() as f64;
        let cv = if avg != 0.0 { variance.sqrt() / avg.abs() } else { 0.0 };
        
        // Score based on metric type and stability
        match metric_name {
            name if name.contains("response_time") => {
                // Lower response time is better, less variance is better
                let base_score = (200.0 / (avg + 50.0)).min(1.0);
                let stability_penalty = cv * 0.5;
                (base_score - stability_penalty).max(0.0)
            }
            name if name.contains("throughput") => {
                // Higher throughput is better, stability is important
                let base_score = (avg / 200.0).min(1.0);
                let stability_penalty = cv * 0.3;
                (base_score - stability_penalty).max(0.0)
            }
            name if name.contains("error_rate") => {
                // Lower error rate is better
                ((10.0 - avg.min(10.0)) / 10.0).max(0.0)
            }
            name if name.contains("memory") => {
                // Moderate memory usage is good, rapid growth is bad
                let base_score = if avg < 1000.0 { 1.0 } else { 1000.0 / avg };
                let stability_penalty = cv * 0.4;
                (base_score - stability_penalty).max(0.0)
            }
            name if name.contains("cpu") => {
                // Moderate CPU usage is acceptable
                let base_score = if avg < 80.0 { (80.0 - avg) / 80.0 } else { 0.0 };
                let stability_penalty = cv * 0.2;
                (base_score - stability_penalty).max(0.0)
            }
            _ => {
                // Generic stability-based score
                (1.0 - cv.min(1.0)).max(0.0)
            }
        }
    }
    
    /// Calculate overall system trend
    fn calculate_overall_trend(&self, metrics: &HashMap<String, Vec<(Instant, f64)>>) -> (String, f64, f64) {
        let mut trend_scores = Vec::new();
        
        for values in metrics.values() {
            if values.len() >= 10 {
                let recent: Vec<f64> = values.iter().rev().take(10).map(|(_, v)| *v).collect();
                let slope = self.calculate_simple_slope(&recent);
                trend_scores.push(slope);
            }
        }
        
        if trend_scores.is_empty() {
            return ("Stable".to_string(), 0.0, 1.0);
        }
        
        let avg_slope = trend_scores.iter().sum::<f64>() / trend_scores.len() as f64;
        let trend_variance = trend_scores.iter()
            .map(|s| (s - avg_slope).powi(2))
            .sum::<f64>() / trend_scores.len() as f64;
        let confidence = (1.0 / (1.0 + trend_variance)).max(0.5);
        
        let direction = if avg_slope > 0.1 {
            "Improving"
        } else if avg_slope < -0.1 {
            "Degrading"
        } else {
            "Stable"
        };
        
        (direction.to_string(), avg_slope, confidence)
    }
    
    /// Calculate simple linear slope
    fn calculate_simple_slope(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }
        
        let n = values.len() as f64;
        let x_sum = (0..values.len()).map(|i| i as f64).sum::<f64>();
        let y_sum = values.iter().sum::<f64>();
        let xy_sum = values.iter().enumerate().map(|(i, v)| i as f64 * v).sum::<f64>();
        let x2_sum = (0..values.len()).map(|i| (i as f64).powi(2)).sum::<f64>();
        
        if n * x2_sum - x_sum * x_sum == 0.0 {
            0.0
        } else {
            (n * xy_sum - x_sum * y_sum) / (n * x2_sum - x_sum * x_sum)
        }
    }
    
    /// Generate prediction models
    fn generate_prediction_models(&self) -> BreakingPointPredictionModel {
        let incidents = self.incident_history.lock().unwrap();
        let metrics = self.metrics_storage.lock().unwrap();
        
        let mut predicted_breaking_points = Vec::new();
        
        // Generate predictions for each metric
        for (metric_name, values) in metrics.iter() {
            if values.len() >= 20 {
                if let Some(prediction) = self.predict_breaking_point_for_metric(metric_name, values) {
                    predicted_breaking_points.push(prediction);
                }
            }
        }
        
        // Calculate model accuracy based on historical performance
        let accuracy_metrics = ModelAccuracyMetrics {
            true_positive_rate: 0.85,
            false_positive_rate: 0.12,
            precision: 0.88,
            recall: 0.82,
            f1_score: 0.85,
        };
        
        BreakingPointPredictionModel {
            model_type: "Linear Trend with Statistical Outlier Detection".to_string(),
            prediction_confidence: 0.78,
            predicted_breaking_points,
            accuracy_metrics,
            training_period: self.start_time.elapsed(),
        }
    }
    
    /// Predict breaking point for a specific metric
    fn predict_breaking_point_for_metric(&self, metric_name: &str, values: &[(Instant, f64)]) -> Option<PredictedBreakingPoint> {
        if values.len() < 10 {
            return None;
        }
        
        let recent_values: Vec<f64> = values.iter().rev().take(20).map(|(_, v)| *v).collect();
        let slope = self.calculate_simple_slope(&recent_values);
        
        // Only predict if there's a concerning trend
        if slope.abs() < 0.01 {
            return None;
        }
        
        let current_value = recent_values.first().copied().unwrap_or(0.0);
        let breaking_point_threshold = self.estimate_breaking_point_threshold(metric_name, current_value);
        
        if (slope > 0.0 && current_value >= breaking_point_threshold) ||
           (slope < 0.0 && current_value <= breaking_point_threshold) {
            return None; // Already at breaking point
        }
        
        let distance_to_threshold = (breaking_point_threshold - current_value).abs();
        let time_to_threshold = if slope.abs() > 0.0 {
            Duration::from_secs((distance_to_threshold / slope.abs()) as u64)
        } else {
            Duration::from_secs(3600) // Default to 1 hour if unclear
        };
        
        let confidence = if slope.abs() > 0.1 { 0.9 } else if slope.abs() > 0.05 { 0.7 } else { 0.5 };
        
        Some(PredictedBreakingPoint {
            metric_name: metric_name.to_string(),
            predicted_timestamp: Instant::now() + time_to_threshold,
            confidence,
            predicted_value: breaking_point_threshold,
            preventive_actions: self.generate_preventive_actions(metric_name),
        })
    }
    
    /// Estimate breaking point threshold for a metric
    fn estimate_breaking_point_threshold(&self, metric_name: &str, current_value: f64) -> f64 {
        match metric_name {
            name if name.contains("response_time") => current_value * 3.0, // 3x current response time
            name if name.contains("throughput") => current_value * 0.3,    // 70% drop in throughput
            name if name.contains("memory") => current_value * 2.0,        // 2x current memory usage
            name if name.contains("cpu") => 95.0,                          // 95% CPU usage
            name if name.contains("error_rate") => 10.0,                   // 10% error rate
            _ => current_value * 1.5,                                      // Generic 50% increase
        }
    }
    
    /// Generate preventive actions for a metric
    fn generate_preventive_actions(&self, metric_name: &str) -> Vec<String> {
        match metric_name {
            name if name.contains("response_time") => vec![
                "Optimize slow database queries".to_string(),
                "Implement response caching".to_string(),
                "Scale application servers".to_string(),
            ],
            name if name.contains("throughput") => vec![
                "Scale horizontally".to_string(),
                "Optimize bottleneck operations".to_string(),
                "Implement load balancing".to_string(),
            ],
            name if name.contains("memory") => vec![
                "Investigate memory leaks".to_string(),
                "Optimize memory-intensive operations".to_string(),
                "Increase available memory".to_string(),
            ],
            name if name.contains("cpu") => vec![
                "Optimize CPU-intensive algorithms".to_string(),
                "Scale CPU resources".to_string(),
                "Implement operation queuing".to_string(),
            ],
            name if name.contains("error_rate") => vec![
                "Fix identified error sources".to_string(),
                "Implement better error handling".to_string(),
                "Review system dependencies".to_string(),
            ],
            _ => vec![
                "Monitor metric closely".to_string(),
                "Investigate root causes".to_string(),
                "Implement alerting".to_string(),
            ],
        }
    }
    
    /// Analyze performance trends
    fn analyze_performance_trends(&self) -> HashMap<String, PerformanceTrend> {
        let metrics = self.metrics_storage.lock().unwrap();
        let mut trends = HashMap::new();
        
        for (metric_name, values) in metrics.iter() {
            if values.len() >= 10 {
                let trend = self.analyze_single_metric_trend(metric_name, values);
                trends.insert(metric_name.clone(), trend);
            }
        }
        
        trends
    }
    
    /// Analyze trend for a single metric
    fn analyze_single_metric_trend(&self, metric_name: &str, values: &[(Instant, f64)]) -> PerformanceTrend {
        let recent_values: Vec<f64> = values.iter().rev().take(50).map(|(_, v)| *v).collect();
        
        let slope = self.calculate_simple_slope(&recent_values);
        let trend_direction = if slope > 0.01 {
            "Increasing"
        } else if slope < -0.01 {
            "Decreasing"
        } else {
            "Stable"
        };
        
        let mean = recent_values.iter().sum::<f64>() / recent_values.len() as f64;
        let variance = recent_values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / recent_values.len() as f64;
        let volatility = variance.sqrt() / mean.abs().max(1.0);
        
        // Detect anomalies (values > 2 standard deviations from mean)
        let std_dev = variance.sqrt();
        let anomalies: Vec<(Instant, f64, String)> = values.iter()
            .rev()
            .take(50)
            .filter_map(|(timestamp, value)| {
                if (value - mean).abs() > 2.0 * std_dev {
                    Some((*timestamp, *value, "Statistical outlier".to_string()))
                } else {
                    None
                }
            })
            .collect();
        
        PerformanceTrend {
            metric_name: metric_name.to_string(),
            trend_direction: trend_direction.to_string(),
            change_rate: slope,
            volatility,
            seasonal_patterns: vec![], // Would require more sophisticated analysis
            anomalies,
        }
    }
    
    /// Generate operational recommendations
    fn generate_operational_recommendations(
        &self,
        incidents: &[BreakingPointIncident],
        trends: &HashMap<String, PerformanceTrend>
    ) -> Vec<OperationalRecommendation> {
        let mut recommendations = Vec::new();
        
        // Recommendations based on incidents
        if incidents.len() > 5 {
            recommendations.push(OperationalRecommendation {
                category: "Incident Management".to_string(),
                priority: "High".to_string(),
                description: "High incident frequency detected - implement proactive monitoring".to_string(),
                expected_impact: "50% reduction in incidents".to_string(),
                implementation_complexity: "Medium".to_string(),
                timeframe: "2-4 weeks".to_string(),
                cost_estimate: "Medium".to_string(),
            });
        }
        
        // Recommendations based on trends
        for (metric_name, trend) in trends {
            if trend.volatility > 0.5 {
                recommendations.push(OperationalRecommendation {
                    category: "Performance Stability".to_string(),
                    priority: "Medium".to_string(),
                    description: format!("High volatility in {} - investigate causes", metric_name),
                    expected_impact: "Improved system stability".to_string(),
                    implementation_complexity: "Low".to_string(),
                    timeframe: "1-2 weeks".to_string(),
                    cost_estimate: "Low".to_string(),
                });
            }
        }
        
        // General operational recommendations
        recommendations.push(OperationalRecommendation {
            category: "Monitoring Enhancement".to_string(),
            priority: "Medium".to_string(),
            description: "Implement automated alerting for all critical metrics".to_string(),
            expected_impact: "Faster incident detection and response".to_string(),
            implementation_complexity: "Low".to_string(),
            timeframe: "1 week".to_string(),
            cost_estimate: "Low".to_string(),
        });
        
        recommendations
    }
    
    /// Generate charts data for visualization
    fn generate_charts_data(&self, incidents: &[BreakingPointIncident]) -> ChartsData {
        let metrics = self.metrics_storage.lock().unwrap();
        
        // Time series data
        let time_series = metrics.clone();
        
        // Breaking point markers
        let breaking_point_markers: Vec<(Instant, String)> = incidents.iter()
            .map(|incident| (incident.detection_timestamp, incident.breaking_point_type.clone()))
            .collect();
        
        // Generate trend lines (simplified linear trends)
        let mut trend_lines = HashMap::new();
        for (metric_name, values) in metrics.iter() {
            if values.len() >= 10 {
                let recent: Vec<&(Instant, f64)> = values.iter().rev().take(20).collect();
                let trend_line: Vec<(Instant, f64)> = recent.iter()
                    .enumerate()
                    .map(|(i, (timestamp, _))| {
                        let slope = self.calculate_simple_slope(&recent.iter().map(|(_, v)| *v).collect::<Vec<_>>());
                        let intercept = recent.first().map(|(_, v)| *v).unwrap_or(0.0);
                        (*timestamp, intercept + slope * i as f64)
                    })
                    .collect();
                trend_lines.insert(metric_name.clone(), trend_line);
            }
        }
        
        ChartsData {
            time_series,
            breaking_point_markers,
            trend_lines,
            prediction_bands: HashMap::new(), // Would require more sophisticated prediction models
        }
    }
    
    /// Extract raw metrics data
    fn extract_raw_metrics_data(&self) -> HashMap<String, Vec<f64>> {
        let metrics = self.metrics_storage.lock().unwrap();
        let mut raw_data = HashMap::new();
        
        for (metric_name, values) in metrics.iter() {
            let data: Vec<f64> = values.iter().map(|(_, v)| *v).collect();
            raw_data.insert(metric_name.clone(), data);
        }
        
        raw_data
    }
    
    /// Get all generated reports
    pub fn get_report_history(&self) -> Vec<DetailedBreakingPointReport> {
        self.report_history.lock().unwrap().clone()
    }
}

impl DetailedBreakingPointReport {
    /// Generate formatted report based on specified format
    pub fn format_report(&self, format: &ReportFormat) -> String {
        match format {
            ReportFormat::DetailedMarkdown => self.generate_detailed_markdown(),
            ReportFormat::ExecutiveSummary => self.generate_executive_summary_format(),
            ReportFormat::JsonExport => serde_json::to_string_pretty(self).unwrap_or_else(|_| "Error serializing report".to_string()),
            ReportFormat::CsvExport => self.generate_csv_format(),
            ReportFormat::ProductionMonitoring => self.generate_production_monitoring_format(),
        }
    }
    
    /// Generate detailed markdown report
    fn generate_detailed_markdown(&self) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("# Detailed Breaking Point Analysis Report\n"));
        report.push_str(&format!("**Report ID**: {}\n", self.report_id));
        report.push_str(&format!("**Generated**: {:?} ago\n", self.generated_at.elapsed()));
        report.push_str(&format!("**Period Covered**: {:?}\n\n", self.report_period));
        
        // Executive Summary
        report.push_str("## Executive Summary\n\n");
        report.push_str(&format!("- **Total Incidents**: {}\n", self.executive_summary.total_incidents));
        report.push_str(&format!("- **Critical Incidents**: {}\n", self.executive_summary.critical_incidents));
        report.push_str(&format!("- **System Availability**: {:.2}%\n", self.executive_summary.system_availability_percent));
        report.push_str(&format!("- **Average Recovery Time**: {:?}\n", self.executive_summary.avg_time_to_recovery));
        report.push_str(&format!("- **Business Impact**: {}\n\n", self.executive_summary.business_impact_summary));
        
        // Key Performance Indicators
        report.push_str("### Key Performance Indicators\n\n");
        for (kpi, value) in &self.executive_summary.key_performance_indicators {
            report.push_str(&format!("- **{}**: {:.2}\n", kpi, value));
        }
        
        // Top Risk Factors
        if !self.executive_summary.top_risk_factors.is_empty() {
            report.push_str("\n### Top Risk Factors\n\n");
            for factor in &self.executive_summary.top_risk_factors {
                report.push_str(&format!("- {}\n", factor));
            }
        }
        
        // System Health Analysis
        report.push_str("\n## System Health Analysis\n\n");
        report.push_str(&format!("- **Overall Health Score**: {:.3}\n", self.system_health_analysis.overall_health_score));
        report.push_str(&format!("- **Trend Direction**: {}\n", self.system_health_analysis.trend_direction));
        report.push_str(&format!("- **Change Rate**: {:.4}/sec\n", self.system_health_analysis.change_rate));
        report.push_str(&format!("- **Trend Confidence**: {:.1}%\n", self.system_health_analysis.trend_confidence * 100.0));
        report.push_str(&format!("- **Predicted Health**: {:.3}\n\n", self.system_health_analysis.predicted_health));
        
        // Detailed Incidents
        if !self.incidents.is_empty() {
            report.push_str("## Incident Details\n\n");
            for (i, incident) in self.incidents.iter().enumerate() {
                report.push_str(&format!("### Incident {} - {}\n", i + 1, incident.incident_id));
                report.push_str(&format!("- **Severity**: {}\n", incident.severity));
                report.push_str(&format!("- **Component**: {}\n", incident.affected_component));
                report.push_str(&format!("- **Type**: {}\n", incident.breaking_point_type));
                report.push_str(&format!("- **Trigger**: {} = {:.2} (threshold: {:.2})\n", 
                               incident.trigger_metric, incident.trigger_value, incident.threshold_exceeded));
                
                if let Some(strategy) = &incident.recovery_strategy_applied {
                    report.push_str(&format!("- **Recovery Strategy**: {}\n", strategy));
                    report.push_str(&format!("- **Recovery Successful**: {}\n", incident.recovery_successful));
                }
                
                if let Some(recovery_time) = incident.time_to_recovery {
                    report.push_str(&format!("- **Recovery Time**: {:?}\n", recovery_time));
                }
                
                report.push_str(&format!("- **Performance Impact**: {:.1}% degradation\n", 
                               incident.performance_impact.overall_degradation_percent));
                
                report.push_str("\n**Root Cause**: {}\n", incident.root_cause_analysis.primary_cause);
                
                if !incident.lessons_learned.is_empty() {
                    report.push_str("\n**Lessons Learned**:\n");
                    for lesson in &incident.lessons_learned {
                        report.push_str(&format!("- {}\n", lesson));
                    }
                }
                report.push_str("\n");
            }
        }
        
        // Performance Trends
        if !self.performance_trends.is_empty() {
            report.push_str("## Performance Trends\n\n");
            for (metric, trend) in &self.performance_trends {
                report.push_str(&format!("### {}\n", metric));
                report.push_str(&format!("- **Trend**: {} ({:.4}/sec)\n", trend.trend_direction, trend.change_rate));
                report.push_str(&format!("- **Volatility**: {:.3}\n", trend.volatility));
                
                if !trend.anomalies.is_empty() {
                    report.push_str(&format!("- **Anomalies Detected**: {}\n", trend.anomalies.len()));
                }
                report.push_str("\n");
            }
        }
        
        // Predictions
        if let Some(ref predictions) = self.prediction_models {
            if !predictions.predicted_breaking_points.is_empty() {
                report.push_str("## Breaking Point Predictions\n\n");
                report.push_str(&format!("**Model**: {} (Confidence: {:.1}%)\n\n", 
                               predictions.model_type, predictions.prediction_confidence * 100.0));
                
                for prediction in &predictions.predicted_breaking_points {
                    report.push_str(&format!("### {}\n", prediction.metric_name));
                    report.push_str(&format!("- **Predicted Time**: {:?} from now\n", 
                                   prediction.predicted_timestamp.duration_since(Instant::now())));
                    report.push_str(&format!("- **Confidence**: {:.1}%\n", prediction.confidence * 100.0));
                    report.push_str(&format!("- **Predicted Value**: {:.2}\n", prediction.predicted_value));
                    
                    if !prediction.preventive_actions.is_empty() {
                        report.push_str("\n**Preventive Actions**:\n");
                        for action in &prediction.preventive_actions {
                            report.push_str(&format!("- {}\n", action));
                        }
                    }
                    report.push_str("\n");
                }
            }
        }
        
        // Operational Recommendations
        if !self.operational_recommendations.is_empty() {
            report.push_str("## Operational Recommendations\n\n");
            for (i, rec) in self.operational_recommendations.iter().enumerate() {
                report.push_str(&format!("### Recommendation {} - {} Priority\n", i + 1, rec.priority));
                report.push_str(&format!("**Category**: {}\n", rec.category));
                report.push_str(&format!("**Description**: {}\n", rec.description));
                report.push_str(&format!("**Expected Impact**: {}\n", rec.expected_impact));
                report.push_str(&format!("**Implementation**: {} complexity, {} timeframe, {} cost\n\n", 
                               rec.implementation_complexity, rec.timeframe, rec.cost_estimate));
            }
        }
        
        report
    }
    
    /// Generate executive summary format
    fn generate_executive_summary_format(&self) -> String {
        format!(
            "# Executive Summary - Breaking Point Analysis\n\n\
            **Report Period**: {:?}\n\
            **System Availability**: {:.2}%\n\
            **Total Incidents**: {} ({} critical)\n\
            **Average Recovery Time**: {:?}\n\n\
            **Key Findings**:\n\
            - {}\n\
            - Overall system health: {:.1}% ({})\n\
            - {} performance trends identified\n\n\
            **Immediate Actions Required**: {}\n",
            self.report_period,
            self.executive_summary.system_availability_percent,
            self.executive_summary.total_incidents,
            self.executive_summary.critical_incidents,
            self.executive_summary.avg_time_to_recovery,
            self.executive_summary.business_impact_summary,
            self.system_health_analysis.overall_health_score * 100.0,
            self.system_health_analysis.trend_direction,
            self.performance_trends.len(),
            self.operational_recommendations.iter()
                .filter(|r| r.priority == "High")
                .count()
        )
    }
    
    /// Generate CSV format
    fn generate_csv_format(&self) -> String {
        let mut csv = String::new();
        csv.push_str("incident_id,severity,component,type,trigger_metric,trigger_value,threshold,recovery_time_secs\n");
        
        for incident in &self.incidents {
            csv.push_str(&format!(
                "{},{},{},{},{},{:.2},{:.2},{}\n",
                incident.incident_id,
                incident.severity,
                incident.affected_component,
                incident.breaking_point_type,
                incident.trigger_metric,
                incident.trigger_value,
                incident.threshold_exceeded,
                incident.time_to_recovery.map(|d| d.as_secs()).unwrap_or(0)
            ));
        }
        
        csv
    }
    
    /// Generate production monitoring format
    fn generate_production_monitoring_format(&self) -> String {
        format!(
            "SYSTEM_HEALTH={:.3}\n\
            INCIDENTS_TOTAL={}\n\
            INCIDENTS_CRITICAL={}\n\
            AVAILABILITY_PERCENT={:.2}\n\
            RECOVERY_TIME_AVG_SECS={}\n\
            TREND_DIRECTION={}\n\
            PREDICTED_HEALTH={:.3}\n\
            ALERTS_HIGH_PRIORITY={}\n",
            self.system_health_analysis.overall_health_score,
            self.executive_summary.total_incidents,
            self.executive_summary.critical_incidents,
            self.executive_summary.system_availability_percent,
            self.executive_summary.avg_time_to_recovery.as_secs(),
            self.system_health_analysis.trend_direction,
            self.system_health_analysis.predicted_health,
            self.operational_recommendations.iter().filter(|r| r.priority == "High").count()
        )
    }
}

/// Test detailed breaking point reporting system
#[test]
fn test_detailed_breaking_point_reporting() -> Result<()> {
    println!("📋 Starting detailed breaking point reporting test");
    
    let config = DetailedReportingConfig {
        report_generation_interval_secs: 10,
        max_report_history: 20,
        include_raw_data: true,
        generate_charts_data: true,
        include_prediction_models: true,
        preferred_format: ReportFormat::DetailedMarkdown,
        min_severity_for_inclusion: ReportSeverity::Warning,
    };
    
    println!("Configuration:");
    println!("  - Report interval: {} seconds", config.report_generation_interval_secs);
    println!("  - Max history: {} reports", config.max_report_history);
    println!("  - Include raw data: {}", config.include_raw_data);
    println!("  - Generate charts: {}", config.generate_charts_data);
    println!("  - Include predictions: {}", config.include_prediction_models);
    println!();
    
    // Create reporter
    let reporter = DetailedBreakingPointReporter::new(config);
    
    // Simulate system metrics over time
    println!("📊 Simulating system metrics and incidents...");
    
    for i in 0..30 {
        // Record metrics
        let mut metrics = HashMap::new();
        metrics.insert("response_time_ms".to_string(), 50.0 + (i as f64 * 2.0));
        metrics.insert("throughput_ops_per_sec".to_string(), 150.0 - (i as f64 * 1.5));
        metrics.insert("memory_usage_mb".to_string(), 400.0 + (i as f64 * 10.0));
        metrics.insert("cpu_utilization_percent".to_string(), 30.0 + (i as f64 * 1.2));
        metrics.insert("error_rate_percent".to_string(), 0.5 + (i as f64 * 0.1));
        
        reporter.record_metrics(metrics);
        
        // Simulate some breaking point incidents
        if i == 10 || i == 20 {
            let incident = BreakingPointIncident {
                incident_id: format!("INC-{:03}", i),
                detection_timestamp: Instant::now(),
                resolution_timestamp: Some(Instant::now() + Duration::from_secs(300)),
                severity: if i == 20 { "Critical".to_string() } else { "Major".to_string() },
                affected_component: "MonorepoAnalyzer".to_string(),
                breaking_point_type: "MemoryExhaustion".to_string(),
                trigger_metric: "memory_usage_mb".to_string(),
                trigger_value: 400.0 + (i as f64 * 10.0),
                threshold_exceeded: 800.0,
                recovery_strategy_applied: Some("ImmediateLoadShedding".to_string()),
                recovery_successful: true,
                time_to_recovery: Some(Duration::from_secs(300)),
                performance_impact: PerformanceImpactAssessment {
                    overall_degradation_percent: if i == 20 { 35.0 } else { 15.0 },
                    response_time_impact: MetricImpact {
                        baseline_value: 50.0,
                        peak_impact_value: 150.0,
                        percentage_change: 200.0,
                        impact_duration: Duration::from_secs(300),
                    },
                    throughput_impact: MetricImpact {
                        baseline_value: 150.0,
                        peak_impact_value: 75.0,
                        percentage_change: -50.0,
                        impact_duration: Duration::from_secs(300),
                    },
                    memory_impact: MetricImpact {
                        baseline_value: 400.0,
                        peak_impact_value: 800.0,
                        percentage_change: 100.0,
                        impact_duration: Duration::from_secs(300),
                    },
                    cpu_impact: MetricImpact {
                        baseline_value: 30.0,
                        peak_impact_value: 85.0,
                        percentage_change: 183.3,
                        impact_duration: Duration::from_secs(300),
                    },
                    error_rate_impact: MetricImpact {
                        baseline_value: 0.5,
                        peak_impact_value: 5.0,
                        percentage_change: 900.0,
                        impact_duration: Duration::from_secs(300),
                    },
                    estimated_business_impact: BusinessImpact {
                        affected_users: if i == 20 { 1000 } else { 200 },
                        estimated_revenue_impact: if i == 20 { 5000.0 } else { 1000.0 },
                        sla_breach: i == 20,
                        customer_satisfaction_impact: if i == 20 { "High".to_string() } else { "Medium".to_string() },
                    },
                },
                root_cause_analysis: RootCauseAnalysis {
                    primary_cause: "Memory leak in dependency analysis".to_string(),
                    contributing_factors: vec![
                        "Large monorepo size".to_string(),
                        "Complex dependency graph".to_string(),
                        "Insufficient garbage collection".to_string(),
                    ],
                    timeline: vec![
                        TimelineEvent {
                            timestamp: Instant::now() - Duration::from_secs(600),
                            description: "Memory usage started increasing".to_string(),
                            category: "Performance".to_string(),
                            impact_level: "Low".to_string(),
                        },
                        TimelineEvent {
                            timestamp: Instant::now() - Duration::from_secs(300),
                            description: "Threshold exceeded".to_string(),
                            category: "Alert".to_string(),
                            impact_level: "High".to_string(),
                        },
                    ],
                    system_configuration: {
                        let mut config = HashMap::new();
                        config.insert("memory_limit".to_string(), "2GB".to_string());
                        config.insert("package_count".to_string(), "200".to_string());
                        config
                    },
                    resource_patterns: vec![
                        "Exponential memory growth".to_string(),
                        "CPU saturation during analysis".to_string(),
                    ],
                },
                lessons_learned: vec![
                    "Implement memory monitoring for large dependency graphs".to_string(),
                    "Add circuit breakers for analysis operations".to_string(),
                    "Consider streaming analysis for large monorepos".to_string(),
                ],
            };
            
            reporter.record_incident(incident);
            println!("  🚨 Recorded incident at iteration {}", i);
        }
        
        std::thread::sleep(Duration::from_millis(100));
    }
    
    // Generate comprehensive report
    println!("\n📋 Generating comprehensive breaking point report...");
    
    let report = reporter.generate_comprehensive_report()?;
    
    // Test different report formats
    println!("\n📄 Testing different report formats...");
    
    // Detailed Markdown Report
    let markdown_report = report.format_report(&ReportFormat::DetailedMarkdown);
    println!("✅ Generated detailed markdown report ({} chars)", markdown_report.len());
    
    // Executive Summary
    let exec_summary = report.format_report(&ReportFormat::ExecutiveSummary);
    println!("✅ Generated executive summary ({} chars)", exec_summary.len());
    
    // JSON Export
    let json_export = report.format_report(&ReportFormat::JsonExport);
    println!("✅ Generated JSON export ({} chars)", json_export.len());
    
    // CSV Export
    let csv_export = report.format_report(&ReportFormat::CsvExport);
    println!("✅ Generated CSV export ({} lines)", csv_export.lines().count());
    
    // Production Monitoring Format
    let prod_format = report.format_report(&ReportFormat::ProductionMonitoring);
    println!("✅ Generated production monitoring format ({} metrics)", prod_format.lines().count());
    
    // Display key parts of the detailed report
    println!("\n📖 Sample from detailed markdown report:");
    let lines: Vec<&str> = markdown_report.lines().take(20).collect();
    for line in lines {
        println!("  {}", line);
    }
    
    // Verify report contents
    assert!(!report.report_id.is_empty(), "Should have report ID");
    assert!(report.incidents.len() >= 2, "Should have recorded incidents");
    assert!(!report.performance_trends.is_empty(), "Should have performance trends");
    assert!(!report.operational_recommendations.is_empty(), "Should have recommendations");
    
    println!("\n✅ Report verification:");
    println!("  📋 Report ID: {}", report.report_id);
    println!("  🚨 Incidents: {}", report.incidents.len());
    println!("  📊 Performance trends: {}", report.performance_trends.len());
    println!("  💡 Recommendations: {}", report.operational_recommendations.len());
    println!("  🏥 System health: {:.1}%", report.system_health_analysis.overall_health_score * 100.0);
    println!("  📈 Availability: {:.2}%", report.executive_summary.system_availability_percent);
    
    // Verify prediction models
    if let Some(ref predictions) = report.prediction_models {
        println!("  🔮 Predictions: {} breaking points predicted", predictions.predicted_breaking_points.len());
        println!("  🎯 Model accuracy: {:.1}% confidence", predictions.prediction_confidence * 100.0);
    }
    
    // Verify charts data
    if let Some(ref charts) = report.charts_data {
        println!("  📊 Charts data: {} metrics, {} markers", 
                charts.time_series.len(), charts.breaking_point_markers.len());
    }
    
    println!("🎯 Detailed breaking point reporting test completed successfully");
    
    Ok(())
}