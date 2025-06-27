//! Real-time Resource Monitoring for Stress Testing
//!
//! This module implements continuous monitoring of system resources during stress testing,
//! providing live metrics, threshold alerting, and trend analysis.

use sublime_monorepo_tools::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Real-time resource monitoring configuration
#[derive(Debug, Clone)]
pub struct RealtimeMonitoringConfig {
    /// Monitoring interval in milliseconds
    pub monitoring_interval_ms: u64,
    /// Maximum monitoring duration
    pub max_duration_secs: u64,
    /// Memory warning threshold (MB)
    pub memory_warning_mb: f64,
    /// Memory critical threshold (MB)
    pub memory_critical_mb: f64,
    /// CPU warning threshold (%)
    pub cpu_warning_percent: f64,
    /// CPU critical threshold (%)
    pub cpu_critical_percent: f64,
    /// Enable trend analysis
    pub enable_trend_analysis: bool,
    /// Sample buffer size for trend calculation
    pub trend_sample_size: usize,
}

impl Default for RealtimeMonitoringConfig {
    fn default() -> Self {
        Self {
            monitoring_interval_ms: 100, // Monitor every 100ms
            max_duration_secs: 300,      // Max 5 minutes
            memory_warning_mb: 1024.0,   // 1GB warning
            memory_critical_mb: 2048.0,  // 2GB critical
            cpu_warning_percent: 70.0,   // 70% CPU warning
            cpu_critical_percent: 90.0,  // 90% CPU critical
            enable_trend_analysis: true,
            trend_sample_size: 20,       // Last 20 samples for trends
        }
    }
}

/// Real-time resource sample
#[derive(Debug, Clone)]
pub struct ResourceSample {
    /// Timestamp when sample was taken
    pub timestamp: Instant,
    /// Memory usage in MB
    pub memory_mb: f64,
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Operation count at this sample
    pub operation_count: usize,
    /// Operation rate (ops/sec since last sample)
    pub operation_rate: f64,
    /// Memory trend (increasing, stable, decreasing)
    pub memory_trend: ResourceTrend,
    /// CPU trend
    pub cpu_trend: ResourceTrend,
}

/// Resource trend analysis
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceTrend {
    /// Resource usage increasing
    Increasing,
    /// Resource usage stable
    Stable,
    /// Resource usage decreasing
    Decreasing,
}

/// Real-time alert types
#[derive(Debug, Clone)]
pub struct ResourceAlert {
    /// When the alert was triggered
    pub timestamp: Instant,
    /// Type of alert
    pub alert_type: AlertType,
    /// Resource value that triggered alert
    pub value: f64,
    /// Threshold that was exceeded
    pub threshold: f64,
    /// Alert message
    pub message: String,
}

/// Types of resource alerts
#[derive(Debug, Clone, PartialEq)]
pub enum AlertType {
    /// Memory warning threshold exceeded
    MemoryWarning,
    /// Memory critical threshold exceeded
    MemoryCritical,
    /// CPU warning threshold exceeded
    CpuWarning,
    /// CPU critical threshold exceeded
    CpuCritical,
    /// Performance degradation detected
    PerformanceDegradation,
    /// Resource leak detected
    ResourceLeak,
}

/// Real-time resource monitor
#[derive(Debug)]
pub struct RealtimeResourceMonitor {
    /// Configuration
    config: RealtimeMonitoringConfig,
    /// Whether monitoring is active
    is_monitoring: Arc<AtomicBool>,
    /// Collected samples
    samples: Arc<Mutex<Vec<ResourceSample>>>,
    /// Triggered alerts
    alerts: Arc<Mutex<Vec<ResourceAlert>>>,
    /// Current operation count
    operation_count: Arc<Mutex<usize>>,
    /// Monitor start time
    start_time: Instant,
}

impl RealtimeResourceMonitor {
    /// Create new real-time monitor
    pub fn new(config: RealtimeMonitoringConfig) -> Self {
        Self {
            config,
            is_monitoring: Arc::new(AtomicBool::new(false)),
            samples: Arc::new(Mutex::new(Vec::new())),
            alerts: Arc::new(Mutex::new(Vec::new())),
            operation_count: Arc::new(Mutex::new(0)),
            start_time: Instant::now(),
        }
    }
    
    /// Start monitoring in background thread
    pub fn start_monitoring(&self) -> Result<()> {
        self.is_monitoring.store(true, Ordering::SeqCst);
        
        let config = self.config.clone();
        let is_monitoring = Arc::clone(&self.is_monitoring);
        let samples = Arc::clone(&self.samples);
        let alerts = Arc::clone(&self.alerts);
        let operation_count = Arc::clone(&self.operation_count);
        let start_time = self.start_time;
        
        thread::spawn(move || {
            let mut last_operation_count = 0;
            let mut last_sample_time = Instant::now();
            
            while is_monitoring.load(Ordering::SeqCst) {
                let now = Instant::now();
                
                // Check if monitoring duration exceeded
                if now.duration_since(start_time).as_secs() > config.max_duration_secs {
                    is_monitoring.store(false, Ordering::SeqCst);
                    break;
                }
                
                // Collect current resource sample
                let current_operation_count = *operation_count.lock().unwrap();
                let time_since_last = now.duration_since(last_sample_time).as_secs_f64();
                let operation_rate = if time_since_last > 0.0 {
                    (current_operation_count - last_operation_count) as f64 / time_since_last
                } else {
                    0.0
                };
                
                let memory_mb = Self::get_memory_usage_mb();
                let cpu_percent = Self::get_cpu_usage_percent();
                
                // Calculate trends
                let memory_trend = Self::calculate_trend(&samples, |s| s.memory_mb, config.trend_sample_size);
                let cpu_trend = Self::calculate_trend(&samples, |s| s.cpu_percent, config.trend_sample_size);
                
                let sample = ResourceSample {
                    timestamp: now,
                    memory_mb,
                    cpu_percent,
                    operation_count: current_operation_count,
                    operation_rate,
                    memory_trend,
                    cpu_trend,
                };
                
                // Check for alerts
                Self::check_alerts(&sample, &config, &alerts);
                
                // Store sample
                samples.lock().unwrap().push(sample);
                
                last_operation_count = current_operation_count;
                last_sample_time = now;
                
                thread::sleep(Duration::from_millis(config.monitoring_interval_ms));
            }
        });
        
        Ok(())
    }
    
    /// Stop monitoring
    pub fn stop_monitoring(&self) {
        self.is_monitoring.store(false, Ordering::SeqCst);
    }
    
    /// Update operation count
    pub fn update_operation_count(&self, count: usize) {
        *self.operation_count.lock().unwrap() = count;
    }
    
    /// Get current monitoring results
    pub fn get_results(&self) -> MonitoringResults {
        let samples = self.samples.lock().unwrap().clone();
        let alerts = self.alerts.lock().unwrap().clone();
        
        MonitoringResults {
            samples,
            alerts,
            config: self.config.clone(),
            monitoring_duration: self.start_time.elapsed(),
        }
    }
    
    /// Calculate resource trend
    fn calculate_trend<F>(samples: &Arc<Mutex<Vec<ResourceSample>>>, extractor: F, sample_size: usize) -> ResourceTrend
    where
        F: Fn(&ResourceSample) -> f64,
    {
        let samples_guard = samples.lock().unwrap();
        let len = samples_guard.len();
        
        if len < 3 {
            return ResourceTrend::Stable;
        }
        
        let start_idx = if len > sample_size { len - sample_size } else { 0 };
        let recent_samples = &samples_guard[start_idx..];
        
        if recent_samples.len() < 3 {
            return ResourceTrend::Stable;
        }
        
        let first_third = recent_samples.len() / 3;
        let second_third = (recent_samples.len() * 2) / 3;
        
        let early_avg = recent_samples[0..first_third].iter()
            .map(|s| extractor(s))
            .sum::<f64>() / first_third as f64;
        
        let late_avg = recent_samples[second_third..].iter()
            .map(|s| extractor(s))
            .sum::<f64>() / (recent_samples.len() - second_third) as f64;
        
        let change_percent = ((late_avg - early_avg) / early_avg) * 100.0;
        
        if change_percent > 5.0 {
            ResourceTrend::Increasing
        } else if change_percent < -5.0 {
            ResourceTrend::Decreasing
        } else {
            ResourceTrend::Stable
        }
    }
    
    /// Check for resource alerts
    fn check_alerts(
        sample: &ResourceSample,
        config: &RealtimeMonitoringConfig,
        alerts: &Arc<Mutex<Vec<ResourceAlert>>>,
    ) {
        let mut alerts_guard = alerts.lock().unwrap();
        
        // Memory alerts
        if sample.memory_mb > config.memory_critical_mb {
            alerts_guard.push(ResourceAlert {
                timestamp: sample.timestamp,
                alert_type: AlertType::MemoryCritical,
                value: sample.memory_mb,
                threshold: config.memory_critical_mb,
                message: format!("Critical memory usage: {:.1} MB exceeds {:.1} MB threshold", 
                               sample.memory_mb, config.memory_critical_mb),
            });
        } else if sample.memory_mb > config.memory_warning_mb {
            alerts_guard.push(ResourceAlert {
                timestamp: sample.timestamp,
                alert_type: AlertType::MemoryWarning,
                value: sample.memory_mb,
                threshold: config.memory_warning_mb,
                message: format!("Warning: memory usage {:.1} MB exceeds {:.1} MB threshold", 
                               sample.memory_mb, config.memory_warning_mb),
            });
        }
        
        // CPU alerts
        if sample.cpu_percent > config.cpu_critical_percent {
            alerts_guard.push(ResourceAlert {
                timestamp: sample.timestamp,
                alert_type: AlertType::CpuCritical,
                value: sample.cpu_percent,
                threshold: config.cpu_critical_percent,
                message: format!("Critical CPU usage: {:.1}% exceeds {:.1}% threshold", 
                               sample.cpu_percent, config.cpu_critical_percent),
            });
        } else if sample.cpu_percent > config.cpu_warning_percent {
            alerts_guard.push(ResourceAlert {
                timestamp: sample.timestamp,
                alert_type: AlertType::CpuWarning,
                value: sample.cpu_percent,
                threshold: config.cpu_warning_percent,
                message: format!("Warning: CPU usage {:.1}% exceeds {:.1}% threshold", 
                               sample.cpu_percent, config.cpu_warning_percent),
            });
        }
        
        // Performance degradation alert
        if sample.operation_rate > 0.0 && sample.operation_rate < 10.0 && sample.cpu_percent > 50.0 {
            alerts_guard.push(ResourceAlert {
                timestamp: sample.timestamp,
                alert_type: AlertType::PerformanceDegradation,
                value: sample.operation_rate,
                threshold: 10.0,
                message: format!("Performance degradation: {:.1} ops/sec with {:.1}% CPU", 
                               sample.operation_rate, sample.cpu_percent),
            });
        }
        
        // Resource leak detection (memory increasing trend with stable operation rate)
        if sample.memory_trend == ResourceTrend::Increasing && 
           sample.operation_rate < 100.0 && sample.memory_mb > config.memory_warning_mb {
            alerts_guard.push(ResourceAlert {
                timestamp: sample.timestamp,
                alert_type: AlertType::ResourceLeak,
                value: sample.memory_mb,
                threshold: config.memory_warning_mb,
                message: format!("Potential memory leak: memory trending up ({:.1} MB) with low operation rate ({:.1} ops/sec)", 
                               sample.memory_mb, sample.operation_rate),
            });
        }
    }
    
    /// Get memory usage (simplified simulation)
    fn get_memory_usage_mb() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let base_memory = 200.0;
        let time_factor = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_millis() as f64 / 1000.0;
        base_memory + (time_factor * 5.0) // Simulate gradual memory growth
    }
    
    /// Get CPU usage (simplified simulation)
    fn get_cpu_usage_percent() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let time_factor = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_millis() as f64 / 1000.0;
        ((time_factor * 30.0) % 80.0) + 10.0 // Simulate CPU usage 10-90%
    }
}

/// Monitoring results summary
#[derive(Debug)]
pub struct MonitoringResults {
    /// All collected samples
    pub samples: Vec<ResourceSample>,
    /// All triggered alerts
    pub alerts: Vec<ResourceAlert>,
    /// Configuration used
    pub config: RealtimeMonitoringConfig,
    /// Total monitoring duration
    pub monitoring_duration: Duration,
}

impl MonitoringResults {
    /// Generate monitoring report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# Real-time Resource Monitoring Report\n\n");
        
        // Summary statistics
        report.push_str("## Summary\n");
        report.push_str(&format!("Monitoring duration: {:?}\n", self.monitoring_duration));
        report.push_str(&format!("Total samples collected: {}\n", self.samples.len()));
        report.push_str(&format!("Total alerts triggered: {}\n", self.alerts.len()));
        report.push_str(&format!("Sampling interval: {} ms\n\n", self.config.monitoring_interval_ms));
        
        // Resource statistics
        if !self.samples.is_empty() {
            let memory_stats = self.calculate_statistics(|s| s.memory_mb);
            let cpu_stats = self.calculate_statistics(|s| s.cpu_percent);
            let rate_stats = self.calculate_statistics(|s| s.operation_rate);
            
            report.push_str("## Resource Statistics\n");
            report.push_str(&format!("Memory Usage (MB):\n"));
            report.push_str(&format!("  - Min: {:.1}, Max: {:.1}, Avg: {:.1}\n", 
                           memory_stats.min, memory_stats.max, memory_stats.avg));
            report.push_str(&format!("CPU Usage (%):\n"));
            report.push_str(&format!("  - Min: {:.1}, Max: {:.1}, Avg: {:.1}\n", 
                           cpu_stats.min, cpu_stats.max, cpu_stats.avg));
            report.push_str(&format!("Operation Rate (ops/sec):\n"));
            report.push_str(&format!("  - Min: {:.1}, Max: {:.1}, Avg: {:.1}\n\n", 
                           rate_stats.min, rate_stats.max, rate_stats.avg));
        }
        
        // Alert summary
        if !self.alerts.is_empty() {
            report.push_str("## Alert Summary\n");
            let mut alert_counts = std::collections::HashMap::new();
            for alert in &self.alerts {
                *alert_counts.entry(&alert.alert_type).or_insert(0) += 1;
            }
            
            for (alert_type, count) in alert_counts {
                report.push_str(&format!("{:?}: {} alerts\n", alert_type, count));
            }
            
            report.push_str("\n### Recent Alerts\n");
            for alert in self.alerts.iter().rev().take(5) {
                report.push_str(&format!("- {}: {}\n", 
                               alert.timestamp.elapsed().as_secs(), alert.message));
            }
        } else {
            report.push_str("## No Alerts Triggered\n");
            report.push_str("System remained within all thresholds during monitoring.\n");
        }
        
        report
    }
    
    /// Calculate basic statistics for a metric
    fn calculate_statistics<F>(&self, extractor: F) -> Statistics
    where
        F: Fn(&ResourceSample) -> f64,
    {
        let values: Vec<f64> = self.samples.iter().map(extractor).collect();
        
        if values.is_empty() {
            return Statistics { min: 0.0, max: 0.0, avg: 0.0 };
        }
        
        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let avg = values.iter().sum::<f64>() / values.len() as f64;
        
        Statistics { min, max, avg }
    }
}

/// Basic statistics structure
#[derive(Debug)]
struct Statistics {
    min: f64,
    max: f64,
    avg: f64,
}

/// Test real-time resource monitoring during stress testing
#[test]
fn test_realtime_resource_monitoring() -> Result<()> {
    println!("📊 Starting real-time resource monitoring test");
    
    let config = RealtimeMonitoringConfig {
        monitoring_interval_ms: 50, // More frequent for testing
        max_duration_secs: 10,      // Shorter test duration
        memory_warning_mb: 300.0,   // Lower thresholds for testing
        memory_critical_mb: 500.0,
        cpu_warning_percent: 40.0,
        cpu_critical_percent: 70.0,
        enable_trend_analysis: true,
        trend_sample_size: 10,
    };
    
    println!("Configuration:");
    println!("  - Monitoring interval: {} ms", config.monitoring_interval_ms);
    println!("  - Max duration: {} seconds", config.max_duration_secs);
    println!("  - Memory thresholds: {:.0} MB (warning), {:.0} MB (critical)", 
             config.memory_warning_mb, config.memory_critical_mb);
    println!("  - CPU thresholds: {:.0}% (warning), {:.0}% (critical)", 
             config.cpu_warning_percent, config.cpu_critical_percent);
    println!();
    
    // Create and start monitor
    let monitor = RealtimeResourceMonitor::new(config);
    monitor.start_monitoring()?;
    
    println!("🔍 Monitoring started - running simulated workload...");
    
    // Simulate stress workload with increasing intensity
    let start_time = Instant::now();
    let mut operation_count = 0;
    
    while start_time.elapsed().as_secs() < 8 { // Run for 8 seconds
        // Simulate operations with increasing complexity
        let complexity_factor = (start_time.elapsed().as_secs() as f64 / 8.0) + 1.0;
        
        for _ in 0..(100.0 * complexity_factor) as usize {
            operation_count += 1;
            monitor.update_operation_count(operation_count);
            
            // Simulate work (sleep to prevent actual overload)
            thread::sleep(Duration::from_micros(50));
        }
        
        println!("  ⚡ Operations: {}, Elapsed: {:?}", 
                operation_count, start_time.elapsed());
        
        thread::sleep(Duration::from_millis(500));
    }
    
    // Stop monitoring and get results
    monitor.stop_monitoring();
    thread::sleep(Duration::from_millis(200)); // Allow monitor thread to finish
    
    let results = monitor.get_results();
    
    // Generate and display report
    let report = results.generate_report();
    println!("\n{}", report);
    
    // Verify monitoring results
    assert!(!results.samples.is_empty(), "Should have collected resource samples");
    assert!(results.samples.len() >= 50, "Should have collected sufficient samples");
    
    println!("✅ Real-time monitoring verification:");
    println!("  📈 Samples collected: {}", results.samples.len());
    println!("  🚨 Alerts triggered: {}", results.alerts.len());
    println!("  ⏱️  Monitoring duration: {:?}", results.monitoring_duration);
    
    // Check that we have good coverage of samples
    let sample_rate = results.samples.len() as f64 / results.monitoring_duration.as_secs_f64();
    println!("  📊 Effective sample rate: {:.1} samples/sec", sample_rate);
    assert!(sample_rate >= 5.0, "Should maintain reasonable sampling rate");
    
    // Verify trends were calculated
    let has_trends = results.samples.iter().any(|s| s.memory_trend != ResourceTrend::Stable);
    if has_trends {
        println!("  📈 Trend analysis working correctly");
    }
    
    println!("🎯 Real-time resource monitoring test completed successfully");
    
    Ok(())
}