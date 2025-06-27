//! Breaking Point Recovery Testing System
//!
//! This module implements comprehensive testing of system recovery capabilities after
//! breaking points are detected, validating graceful degradation and restoration patterns.

use sublime_monorepo_tools::Result;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque};

/// Configuration for breaking point recovery testing
#[derive(Debug, Clone)]
pub struct RecoveryTestConfig {
    /// Maximum time to wait for recovery
    pub max_recovery_time_secs: u64,
    /// Recovery success threshold (percentage of baseline performance)
    pub recovery_success_threshold: f64,
    /// Grace period after load reduction before measuring recovery
    pub recovery_grace_period_secs: u64,
    /// Number of recovery verification cycles
    pub recovery_verification_cycles: usize,
    /// Enable automatic load shedding during recovery
    pub enable_automatic_load_shedding: bool,
    /// Enable circuit breaker pattern
    pub enable_circuit_breaker: bool,
    /// Circuit breaker failure threshold
    pub circuit_breaker_failure_threshold: usize,
    /// Circuit breaker recovery timeout
    pub circuit_breaker_timeout_secs: u64,
}

impl Default for RecoveryTestConfig {
    fn default() -> Self {
        Self {
            max_recovery_time_secs: 180, // 3 minutes
            recovery_success_threshold: 0.8, // 80% of baseline
            recovery_grace_period_secs: 10,
            recovery_verification_cycles: 5,
            enable_automatic_load_shedding: true,
            enable_circuit_breaker: true,
            circuit_breaker_failure_threshold: 3,
            circuit_breaker_timeout_secs: 30,
        }
    }
}

/// Recovery strategy types
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    /// Gradually reduce load until system stabilizes
    GradualLoadReduction,
    /// Immediately drop to safe operating level
    ImmediateLoadShedding,
    /// Circuit breaker pattern - stop all operations temporarily
    CircuitBreaker,
    /// Adaptive recovery based on system response
    AdaptiveRecovery,
    /// Resource cleanup and restart
    ResourceCleanupRestart,
}

/// Recovery phase tracking
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryPhase {
    /// Breaking point detected, starting recovery
    DetectionPhase,
    /// Applying recovery strategy
    StrategyApplication,
    /// Load reduction in progress
    LoadReduction,
    /// Monitoring for stabilization
    StabilizationMonitoring,
    /// Verifying recovery success
    RecoveryVerification,
    /// Recovery completed successfully
    RecoveryCompleted,
    /// Recovery failed
    RecoveryFailed,
}

/// Recovery attempt result
#[derive(Debug, Clone)]
pub struct RecoveryAttempt {
    /// When recovery was attempted
    pub attempt_timestamp: Instant,
    /// Strategy used for recovery
    pub strategy: RecoveryStrategy,
    /// Recovery phase reached
    pub phase_reached: RecoveryPhase,
    /// Duration of recovery attempt
    pub duration: Duration,
    /// Whether recovery was successful
    pub success: bool,
    /// Performance metrics before recovery
    pub pre_recovery_metrics: SystemPerformanceSnapshot,
    /// Performance metrics after recovery
    pub post_recovery_metrics: Option<SystemPerformanceSnapshot>,
    /// Recovery effectiveness score (0.0-1.0)
    pub effectiveness_score: f64,
    /// Detailed recovery log
    pub recovery_log: Vec<String>,
}

/// System performance snapshot for recovery comparison
#[derive(Debug, Clone)]
pub struct SystemPerformanceSnapshot {
    /// Timestamp of snapshot
    pub timestamp: Instant,
    /// Memory usage in MB
    pub memory_mb: f64,
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Response time in milliseconds
    pub response_time_ms: f64,
    /// Throughput (operations per second)
    pub throughput_ops_per_sec: f64,
    /// Error rate (percentage)
    pub error_rate_percent: f64,
    /// Number of active operations
    pub active_operations: usize,
    /// System stability score (0.0-1.0)
    pub stability_score: f64,
}

impl SystemPerformanceSnapshot {
    /// Capture current system performance
    pub fn capture(active_operations: usize) -> Self {
        Self {
            timestamp: Instant::now(),
            memory_mb: Self::get_memory_usage(),
            cpu_percent: Self::get_cpu_usage(),
            response_time_ms: Self::get_response_time(),
            throughput_ops_per_sec: Self::get_throughput(),
            error_rate_percent: Self::get_error_rate(),
            active_operations,
            stability_score: Self::calculate_stability_score(),
        }
    }
    
    /// Compare with another snapshot to calculate recovery effectiveness
    pub fn calculate_recovery_effectiveness(&self, baseline: &SystemPerformanceSnapshot) -> f64 {
        let memory_score = if baseline.memory_mb > 0.0 {
            (baseline.memory_mb / self.memory_mb.max(baseline.memory_mb)).min(1.0)
        } else {
            1.0
        };
        
        let cpu_score = if baseline.cpu_percent > 0.0 {
            (baseline.cpu_percent / self.cpu_percent.max(baseline.cpu_percent)).min(1.0)
        } else {
            1.0
        };
        
        let response_score = if self.response_time_ms > 0.0 {
            (baseline.response_time_ms / self.response_time_ms).min(1.0)
        } else {
            1.0
        };
        
        let throughput_score = if baseline.throughput_ops_per_sec > 0.0 {
            (self.throughput_ops_per_sec / baseline.throughput_ops_per_sec).min(1.0)
        } else {
            1.0
        };
        
        let error_score = if self.error_rate_percent <= baseline.error_rate_percent {
            1.0
        } else {
            (baseline.error_rate_percent / self.error_rate_percent.max(0.1)).min(1.0)
        };
        
        // Weighted average of all scores
        (memory_score * 0.2 + cpu_score * 0.2 + response_score * 0.25 + 
         throughput_score * 0.25 + error_score * 0.1).max(0.0).min(1.0)
    }
    
    /// Check if this snapshot indicates system stability
    pub fn is_stable(&self, baseline: &SystemPerformanceSnapshot, threshold: f64) -> bool {
        let effectiveness = self.calculate_recovery_effectiveness(baseline);
        effectiveness >= threshold && self.stability_score >= 0.7
    }
    
    // Simplified system metric getters
    fn get_memory_usage() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let time_factor = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_millis() as f64 / 1000.0;
        200.0 + (time_factor * 2.0) % 100.0 // Simulated memory usage
    }
    
    fn get_cpu_usage() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let time_factor = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_millis() as f64 / 1000.0;
        10.0 + (time_factor * 1.5) % 80.0 // Simulated CPU usage
    }
    
    fn get_response_time() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let time_factor = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_millis() as f64 / 1000.0;
        50.0 + (time_factor * 0.8) % 200.0 // Simulated response time
    }
    
    fn get_throughput() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let time_factor = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_millis() as f64 / 1000.0;
        100.0 + (time_factor * 0.5).sin() * 50.0 // Simulated throughput
    }
    
    fn get_error_rate() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let time_factor = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_millis() as f64 / 1000.0;
        (time_factor * 2.0).sin().abs() * 5.0 // Simulated error rate 0-5%
    }
    
    fn calculate_stability_score() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let time_factor = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_millis() as f64 / 1000.0;
        0.5 + (time_factor * 0.3).cos().abs() * 0.5 // Simulated stability 0.5-1.0
    }
}

/// Circuit breaker state for recovery pattern
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    /// Normal operation - all requests allowed
    Closed,
    /// Failure threshold exceeded - block all requests
    Open,
    /// Testing if service has recovered
    HalfOpen,
}

/// Circuit breaker implementation
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Current state
    state: Arc<Mutex<CircuitBreakerState>>,
    /// Failure count
    failure_count: Arc<AtomicUsize>,
    /// Success count in half-open state
    success_count: Arc<AtomicUsize>,
    /// Last failure time
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    /// Configuration
    config: RecoveryTestConfig,
}

impl CircuitBreaker {
    /// Create new circuit breaker
    pub fn new(config: RecoveryTestConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitBreakerState::Closed)),
            failure_count: Arc::new(AtomicUsize::new(0)),
            success_count: Arc::new(AtomicUsize::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
            config,
        }
    }
    
    /// Check if operation should be allowed
    pub fn should_allow_operation(&self) -> bool {
        let state = *self.state.lock().unwrap();
        
        match state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if timeout has elapsed
                if let Some(last_failure) = *self.last_failure_time.lock().unwrap() {
                    if last_failure.elapsed().as_secs() >= self.config.circuit_breaker_timeout_secs {
                        // Move to half-open state
                        *self.state.lock().unwrap() = CircuitBreakerState::HalfOpen;
                        self.success_count.store(0, Ordering::SeqCst);
                        return true;
                    }
                }
                false
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }
    
    /// Record operation success
    pub fn record_success(&self) {
        let mut state = self.state.lock().unwrap();
        
        match *state {
            CircuitBreakerState::HalfOpen => {
                let success_count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if success_count >= 3 { // Need 3 successes to close
                    *state = CircuitBreakerState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                }
            }
            CircuitBreakerState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::SeqCst);
            }
            _ => {}
        }
    }
    
    /// Record operation failure
    pub fn record_failure(&self) {
        let failure_count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        *self.last_failure_time.lock().unwrap() = Some(Instant::now());
        
        let mut state = self.state.lock().unwrap();
        
        match *state {
            CircuitBreakerState::Closed => {
                if failure_count >= self.config.circuit_breaker_failure_threshold {
                    *state = CircuitBreakerState::Open;
                }
            }
            CircuitBreakerState::HalfOpen => {
                *state = CircuitBreakerState::Open;
                self.success_count.store(0, Ordering::SeqCst);
            }
            _ => {}
        }
    }
    
    /// Get current state
    pub fn get_state(&self) -> CircuitBreakerState {
        *self.state.lock().unwrap()
    }
}

/// Breaking point recovery testing system
#[derive(Debug)]
pub struct BreakingPointRecoveryTester {
    /// Configuration
    config: RecoveryTestConfig,
    /// Current recovery phase
    current_phase: Arc<Mutex<RecoveryPhase>>,
    /// Recovery attempts history
    recovery_attempts: Arc<Mutex<Vec<RecoveryAttempt>>>,
    /// Baseline performance metrics
    baseline_metrics: Arc<Mutex<Option<SystemPerformanceSnapshot>>>,
    /// Current operation load
    operation_load: Arc<AtomicUsize>,
    /// Maximum safe operation load
    safe_operation_load: Arc<AtomicUsize>,
    /// Circuit breaker
    circuit_breaker: Arc<CircuitBreaker>,
    /// Recovery monitoring active
    monitoring_active: Arc<AtomicBool>,
    /// Test start time
    start_time: Instant,
}

impl BreakingPointRecoveryTester {
    /// Create new recovery tester
    pub fn new(config: RecoveryTestConfig) -> Self {
        let circuit_breaker = Arc::new(CircuitBreaker::new(config.clone()));
        
        Self {
            config,
            current_phase: Arc::new(Mutex::new(RecoveryPhase::DetectionPhase)),
            recovery_attempts: Arc::new(Mutex::new(Vec::new())),
            baseline_metrics: Arc::new(Mutex::new(None)),
            operation_load: Arc::new(AtomicUsize::new(0)),
            safe_operation_load: Arc::new(AtomicUsize::new(100)),
            circuit_breaker,
            monitoring_active: Arc::new(AtomicBool::new(false)),
            start_time: Instant::now(),
        }
    }
    
    /// Set baseline performance metrics
    pub fn set_baseline_metrics(&self, metrics: SystemPerformanceSnapshot) {
        *self.baseline_metrics.lock().unwrap() = Some(metrics);
    }
    
    /// Update current operation load
    pub fn update_operation_load(&self, load: usize) {
        self.operation_load.store(load, Ordering::SeqCst);
    }
    
    /// Trigger recovery test with specific strategy
    pub fn trigger_recovery_test(&self, strategy: RecoveryStrategy) -> Result<()> {
        println!("🚨 Triggering recovery test with strategy: {:?}", strategy);
        
        // Capture pre-recovery metrics
        let pre_recovery_metrics = SystemPerformanceSnapshot::capture(
            self.operation_load.load(Ordering::SeqCst)
        );
        
        // Start recovery attempt
        let attempt_start = Instant::now();
        let mut recovery_log = Vec::new();
        
        // Update phase
        *self.current_phase.lock().unwrap() = RecoveryPhase::StrategyApplication;
        recovery_log.push(format!("Started recovery with strategy: {:?}", strategy));
        
        // Apply recovery strategy
        let phase_reached = self.apply_recovery_strategy(&strategy, &mut recovery_log)?;
        
        // Monitor recovery progress
        let (success, post_recovery_metrics) = if phase_reached != RecoveryPhase::RecoveryFailed {
            self.monitor_recovery_progress(&mut recovery_log)?
        } else {
            (false, None)
        };
        
        // Calculate effectiveness
        let effectiveness_score = if let (Some(baseline), Some(post_metrics)) = 
            (self.baseline_metrics.lock().unwrap().as_ref(), &post_recovery_metrics) {
            post_metrics.calculate_recovery_effectiveness(baseline)
        } else {
            0.0
        };
        
        // Record recovery attempt
        let attempt = RecoveryAttempt {
            attempt_timestamp: attempt_start,
            strategy,
            phase_reached,
            duration: attempt_start.elapsed(),
            success,
            pre_recovery_metrics,
            post_recovery_metrics,
            effectiveness_score,
            recovery_log,
        };
        
        self.recovery_attempts.lock().unwrap().push(attempt);
        
        println!("✅ Recovery test completed - Success: {}, Effectiveness: {:.1}%", 
                success, effectiveness_score * 100.0);
        
        Ok(())
    }
    
    /// Apply specific recovery strategy
    fn apply_recovery_strategy(
        &self, 
        strategy: &RecoveryStrategy, 
        log: &mut Vec<String>
    ) -> Result<RecoveryPhase> {
        match strategy {
            RecoveryStrategy::GradualLoadReduction => {
                self.apply_gradual_load_reduction(log)
            }
            RecoveryStrategy::ImmediateLoadShedding => {
                self.apply_immediate_load_shedding(log)
            }
            RecoveryStrategy::CircuitBreaker => {
                self.apply_circuit_breaker_pattern(log)
            }
            RecoveryStrategy::AdaptiveRecovery => {
                self.apply_adaptive_recovery(log)
            }
            RecoveryStrategy::ResourceCleanupRestart => {
                self.apply_resource_cleanup_restart(log)
            }
        }
    }
    
    /// Apply gradual load reduction strategy
    fn apply_gradual_load_reduction(&self, log: &mut Vec<String>) -> Result<RecoveryPhase> {
        *self.current_phase.lock().unwrap() = RecoveryPhase::LoadReduction;
        log.push("Starting gradual load reduction".to_string());
        
        let initial_load = self.operation_load.load(Ordering::SeqCst);
        let target_load = initial_load / 4; // Reduce to 25% of current load
        
        // Gradually reduce load over 10 steps
        for step in 1..=10 {
            let reduction_factor = step as f64 / 10.0;
            let current_target = initial_load - ((initial_load - target_load) as f64 * reduction_factor) as usize;
            
            self.operation_load.store(current_target, Ordering::SeqCst);
            log.push(format!("Step {}: Reduced load to {}", step, current_target));
            
            thread::sleep(Duration::from_millis(500)); // 500ms between steps
            
            // Check if system is stabilizing
            let metrics = SystemPerformanceSnapshot::capture(current_target);
            if let Some(baseline) = self.baseline_metrics.lock().unwrap().as_ref() {
                if metrics.is_stable(baseline, self.config.recovery_success_threshold) {
                    log.push(format!("System stabilized at load level {}", current_target));
                    self.safe_operation_load.store(current_target, Ordering::SeqCst);
                    break;
                }
            }
        }
        
        Ok(RecoveryPhase::StabilizationMonitoring)
    }
    
    /// Apply immediate load shedding strategy
    fn apply_immediate_load_shedding(&self, log: &mut Vec<String>) -> Result<RecoveryPhase> {
        *self.current_phase.lock().unwrap() = RecoveryPhase::LoadReduction;
        log.push("Applying immediate load shedding".to_string());
        
        let current_load = self.operation_load.load(Ordering::SeqCst);
        let safe_load = self.safe_operation_load.load(Ordering::SeqCst);
        
        // Immediately drop to safe operating level
        self.operation_load.store(safe_load, Ordering::SeqCst);
        log.push(format!("Load immediately reduced from {} to {}", current_load, safe_load));
        
        // Wait for system to stabilize
        thread::sleep(Duration::from_secs(self.config.recovery_grace_period_secs));
        
        Ok(RecoveryPhase::StabilizationMonitoring)
    }
    
    /// Apply circuit breaker pattern
    fn apply_circuit_breaker_pattern(&self, log: &mut Vec<String>) -> Result<RecoveryPhase> {
        *self.current_phase.lock().unwrap() = RecoveryPhase::LoadReduction;
        log.push("Activating circuit breaker pattern".to_string());
        
        // Force circuit breaker to open state
        for _ in 0..self.config.circuit_breaker_failure_threshold {
            self.circuit_breaker.record_failure();
        }
        
        log.push(format!("Circuit breaker opened - blocking all operations for {} seconds", 
                        self.config.circuit_breaker_timeout_secs));
        
        // Set load to zero while circuit breaker is open
        self.operation_load.store(0, Ordering::SeqCst);
        
        // Wait for circuit breaker timeout
        thread::sleep(Duration::from_secs(self.config.circuit_breaker_timeout_secs));
        
        // Circuit breaker will automatically transition to half-open
        // Start with minimal load
        self.operation_load.store(10, Ordering::SeqCst);
        log.push("Circuit breaker transitioning to half-open - starting minimal load".to_string());
        
        Ok(RecoveryPhase::StabilizationMonitoring)
    }
    
    /// Apply adaptive recovery strategy
    fn apply_adaptive_recovery(&self, log: &mut Vec<String>) -> Result<RecoveryPhase> {
        *self.current_phase.lock().unwrap() = RecoveryPhase::LoadReduction;
        log.push("Starting adaptive recovery".to_string());
        
        let mut current_load = self.operation_load.load(Ordering::SeqCst);
        let baseline = self.baseline_metrics.lock().unwrap().clone();
        
        if baseline.is_none() {
            log.push("No baseline metrics available - falling back to gradual reduction".to_string());
            return self.apply_gradual_load_reduction(log);
        }
        
        let baseline = baseline.unwrap();
        
        // Adaptive reduction based on system response
        for iteration in 1..=15 {
            let metrics = SystemPerformanceSnapshot::capture(current_load);
            let effectiveness = metrics.calculate_recovery_effectiveness(&baseline);
            
            log.push(format!("Iteration {}: Load={}, Effectiveness={:.2}", 
                           iteration, current_load, effectiveness));
            
            if effectiveness >= self.config.recovery_success_threshold {
                log.push(format!("Recovery target achieved at load {}", current_load));
                self.safe_operation_load.store(current_load, Ordering::SeqCst);
                break;
            }
            
            // Adaptive reduction based on current effectiveness
            let reduction_factor = if effectiveness < 0.3 {
                0.5 // Aggressive reduction
            } else if effectiveness < 0.6 {
                0.7 // Moderate reduction
            } else {
                0.85 // Conservative reduction
            };
            
            current_load = (current_load as f64 * reduction_factor) as usize;
            self.operation_load.store(current_load, Ordering::SeqCst);
            
            thread::sleep(Duration::from_millis(800));
        }
        
        Ok(RecoveryPhase::StabilizationMonitoring)
    }
    
    /// Apply resource cleanup and restart strategy
    fn apply_resource_cleanup_restart(&self, log: &mut Vec<String>) -> Result<RecoveryPhase> {
        *self.current_phase.lock().unwrap() = RecoveryPhase::LoadReduction;
        log.push("Starting resource cleanup and restart".to_string());
        
        // Step 1: Stop all operations
        self.operation_load.store(0, Ordering::SeqCst);
        log.push("All operations stopped for cleanup".to_string());
        
        // Step 2: Simulate resource cleanup
        thread::sleep(Duration::from_secs(5));
        log.push("Resource cleanup completed".to_string());
        
        // Step 3: Restart with minimal load
        let restart_load = 20;
        self.operation_load.store(restart_load, Ordering::SeqCst);
        log.push(format!("System restarted with minimal load: {}", restart_load));
        
        // Step 4: Gradual ramp-up
        for step in 1..=5 {
            thread::sleep(Duration::from_secs(2));
            let new_load = restart_load * (step + 1);
            self.operation_load.store(new_load, Ordering::SeqCst);
            log.push(format!("Ramp-up step {}: Load increased to {}", step, new_load));
            
            // Check stability at each step
            let metrics = SystemPerformanceSnapshot::capture(new_load);
            if let Some(baseline) = self.baseline_metrics.lock().unwrap().as_ref() {
                if !metrics.is_stable(baseline, 0.6) { // Lower threshold during ramp-up
                    log.push(format!("Instability detected at load {} - stopping ramp-up", new_load));
                    self.safe_operation_load.store(new_load / 2, Ordering::SeqCst);
                    break;
                }
            }
        }
        
        Ok(RecoveryPhase::StabilizationMonitoring)
    }
    
    /// Monitor recovery progress and verify success
    fn monitor_recovery_progress(&self, log: &mut Vec<String>) -> Result<(bool, Option<SystemPerformanceSnapshot>)> {
        *self.current_phase.lock().unwrap() = RecoveryPhase::StabilizationMonitoring;
        log.push("Starting recovery progress monitoring".to_string());
        
        // Wait for grace period
        thread::sleep(Duration::from_secs(self.config.recovery_grace_period_secs));
        
        let baseline = if let Some(b) = self.baseline_metrics.lock().unwrap().clone() {
            b
        } else {
            log.push("No baseline metrics - cannot verify recovery".to_string());
            *self.current_phase.lock().unwrap() = RecoveryPhase::RecoveryFailed;
            return Ok((false, None));
        };
        
        // Monitor stability over multiple cycles
        *self.current_phase.lock().unwrap() = RecoveryPhase::RecoveryVerification;
        log.push("Starting recovery verification cycles".to_string());
        
        let mut stable_cycles = 0;
        let mut verification_metrics = Vec::new();
        
        for cycle in 1..=self.config.recovery_verification_cycles {
            thread::sleep(Duration::from_secs(3)); // Wait between measurements
            
            let current_load = self.operation_load.load(Ordering::SeqCst);
            let metrics = SystemPerformanceSnapshot::capture(current_load);
            verification_metrics.push(metrics.clone());
            
            let effectiveness = metrics.calculate_recovery_effectiveness(&baseline);
            let is_stable = metrics.is_stable(&baseline, self.config.recovery_success_threshold);
            
            log.push(format!("Verification cycle {}: Effectiveness={:.2}, Stable={}", 
                           cycle, effectiveness, is_stable));
            
            if is_stable {
                stable_cycles += 1;
            } else {
                stable_cycles = 0; // Reset counter on instability
            }
            
            // Need at least 3 consecutive stable cycles
            if stable_cycles >= 3 {
                log.push("Recovery verification successful - system stable".to_string());
                *self.current_phase.lock().unwrap() = RecoveryPhase::RecoveryCompleted;
                
                // Return average of stable measurements
                let final_metrics = self.calculate_average_metrics(&verification_metrics[verification_metrics.len()-3..]);
                return Ok((true, Some(final_metrics)));
            }
        }
        
        // Recovery verification failed
        log.push(format!("Recovery verification failed - only {} stable cycles", stable_cycles));
        *self.current_phase.lock().unwrap() = RecoveryPhase::RecoveryFailed;
        
        // Return last measurement
        Ok((false, verification_metrics.last().cloned()))
    }
    
    /// Calculate average metrics from multiple snapshots
    fn calculate_average_metrics(&self, snapshots: &[SystemPerformanceSnapshot]) -> SystemPerformanceSnapshot {
        if snapshots.is_empty() {
            return SystemPerformanceSnapshot::capture(0);
        }
        
        let count = snapshots.len() as f64;
        
        SystemPerformanceSnapshot {
            timestamp: snapshots.last().unwrap().timestamp,
            memory_mb: snapshots.iter().map(|s| s.memory_mb).sum::<f64>() / count,
            cpu_percent: snapshots.iter().map(|s| s.cpu_percent).sum::<f64>() / count,
            response_time_ms: snapshots.iter().map(|s| s.response_time_ms).sum::<f64>() / count,
            throughput_ops_per_sec: snapshots.iter().map(|s| s.throughput_ops_per_sec).sum::<f64>() / count,
            error_rate_percent: snapshots.iter().map(|s| s.error_rate_percent).sum::<f64>() / count,
            active_operations: snapshots.last().unwrap().active_operations,
            stability_score: snapshots.iter().map(|s| s.stability_score).sum::<f64>() / count,
        }
    }
    
    /// Get recovery test results
    pub fn get_results(&self) -> RecoveryTestResults {
        RecoveryTestResults {
            attempts: self.recovery_attempts.lock().unwrap().clone(),
            baseline_metrics: self.baseline_metrics.lock().unwrap().clone(),
            final_phase: *self.current_phase.lock().unwrap(),
            safe_operation_load: self.safe_operation_load.load(Ordering::SeqCst),
            total_test_duration: self.start_time.elapsed(),
            config: self.config.clone(),
        }
    }
}

/// Results from recovery testing
#[derive(Debug)]
pub struct RecoveryTestResults {
    /// All recovery attempts
    pub attempts: Vec<RecoveryAttempt>,
    /// Baseline performance metrics
    pub baseline_metrics: Option<SystemPerformanceSnapshot>,
    /// Final recovery phase reached
    pub final_phase: RecoveryPhase,
    /// Determined safe operation load
    pub safe_operation_load: usize,
    /// Total test duration
    pub total_test_duration: Duration,
    /// Configuration used
    pub config: RecoveryTestConfig,
}

impl RecoveryTestResults {
    /// Generate comprehensive recovery test report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# Breaking Point Recovery Test Report\n\n");
        
        // Executive summary
        report.push_str("## Executive Summary\n");
        report.push_str(&format!("Total test duration: {:?}\n", self.total_test_duration));
        report.push_str(&format!("Recovery attempts: {}\n", self.attempts.len()));
        report.push_str(&format!("Final phase: {:?}\n", self.final_phase));
        report.push_str(&format!("Safe operation load: {}\n", self.safe_operation_load));
        
        let successful_attempts = self.attempts.iter().filter(|a| a.success).count();
        let success_rate = if !self.attempts.is_empty() {
            successful_attempts as f64 / self.attempts.len() as f64 * 100.0
        } else {
            0.0
        };
        report.push_str(&format!("Recovery success rate: {:.1}%\n\n", success_rate));
        
        // Recovery attempts analysis
        if !self.attempts.is_empty() {
            report.push_str("## Recovery Attempts Analysis\n\n");
            
            for (i, attempt) in self.attempts.iter().enumerate() {
                report.push_str(&format!("### Attempt {} - {:?}\n", i + 1, attempt.strategy));
                report.push_str(&format!("- **Duration**: {:?}\n", attempt.duration));
                report.push_str(&format!("- **Success**: {}\n", attempt.success));
                report.push_str(&format!("- **Phase Reached**: {:?}\n", attempt.phase_reached));
                report.push_str(&format!("- **Effectiveness Score**: {:.1}%\n", attempt.effectiveness_score * 100.0));
                
                if let Some(post_metrics) = &attempt.post_recovery_metrics {
                    report.push_str("\n**Performance Comparison**:\n");
                    report.push_str(&format!("- Memory: {:.1} → {:.1} MB\n", 
                                           attempt.pre_recovery_metrics.memory_mb, post_metrics.memory_mb));
                    report.push_str(&format!("- CPU: {:.1} → {:.1}%\n", 
                                           attempt.pre_recovery_metrics.cpu_percent, post_metrics.cpu_percent));
                    report.push_str(&format!("- Response Time: {:.1} → {:.1} ms\n", 
                                           attempt.pre_recovery_metrics.response_time_ms, post_metrics.response_time_ms));
                    report.push_str(&format!("- Throughput: {:.1} → {:.1} ops/sec\n", 
                                           attempt.pre_recovery_metrics.throughput_ops_per_sec, 
                                           post_metrics.throughput_ops_per_sec));
                }
                
                report.push_str("\n**Recovery Log**:\n");
                for log_entry in &attempt.recovery_log {
                    report.push_str(&format!("- {}\n", log_entry));
                }
                report.push_str("\n");
            }
        }
        
        // Strategy effectiveness analysis
        if !self.attempts.is_empty() {
            report.push_str("## Strategy Effectiveness Analysis\n\n");
            
            let mut strategy_stats: HashMap<RecoveryStrategy, (usize, usize, f64)> = HashMap::new();
            
            for attempt in &self.attempts {
                let entry = strategy_stats.entry(attempt.strategy.clone()).or_insert((0, 0, 0.0));
                entry.0 += 1; // Total attempts
                if attempt.success {
                    entry.1 += 1; // Successful attempts
                }
                entry.2 += attempt.effectiveness_score; // Sum of effectiveness scores
            }
            
            for (strategy, (total, successful, total_effectiveness)) in strategy_stats {
                let success_rate = (successful as f64 / total as f64) * 100.0;
                let avg_effectiveness = (total_effectiveness / total as f64) * 100.0;
                
                report.push_str(&format!("**{:?}**:\n", strategy));
                report.push_str(&format!("- Attempts: {}\n", total));
                report.push_str(&format!("- Success Rate: {:.1}%\n", success_rate));
                report.push_str(&format!("- Average Effectiveness: {:.1}%\n\n", avg_effectiveness));
            }
        }
        
        // Baseline comparison
        if let Some(baseline) = &self.baseline_metrics {
            report.push_str("## Baseline Performance\n\n");
            report.push_str(&format!("- Memory: {:.1} MB\n", baseline.memory_mb));
            report.push_str(&format!("- CPU: {:.1}%\n", baseline.cpu_percent));
            report.push_str(&format!("- Response Time: {:.1} ms\n", baseline.response_time_ms));
            report.push_str(&format!("- Throughput: {:.1} ops/sec\n", baseline.throughput_ops_per_sec));
            report.push_str(&format!("- Error Rate: {:.2}%\n", baseline.error_rate_percent));
            report.push_str(&format!("- Stability Score: {:.2}\n\n", baseline.stability_score));
        }
        
        // Recommendations
        report.push_str("## Recommendations\n\n");
        
        if successful_attempts > 0 {
            report.push_str("### Successful Recovery Strategies\n");
            let best_strategy = self.attempts.iter()
                .filter(|a| a.success)
                .max_by(|a, b| a.effectiveness_score.partial_cmp(&b.effectiveness_score).unwrap());
                
            if let Some(best) = best_strategy {
                report.push_str(&format!("- **Primary recommendation**: {:?} (Effectiveness: {:.1}%)\n", 
                               best.strategy, best.effectiveness_score * 100.0));
            }
            
            report.push_str(&format!("- **Safe operation limit**: {} operations\n", self.safe_operation_load));
            report.push_str("- Implement automated recovery triggers at 90% of safe limit\n");
            report.push_str("- Set up monitoring alerts for early warning\n\n");
        } else {
            report.push_str("### Recovery Improvements Needed\n");
            report.push_str("- No recovery strategies were fully successful\n");
            report.push_str("- Consider implementing more aggressive load shedding\n");
            report.push_str("- Review system architecture for bottlenecks\n");
            report.push_str("- Implement additional monitoring and alerting\n\n");
        }
        
        if success_rate < 80.0 {
            report.push_str("### Critical Recommendations\n");
            report.push_str("- Recovery success rate is below acceptable threshold (80%)\n");
            report.push_str("- Implement redundancy and failover mechanisms\n");
            report.push_str("- Consider horizontal scaling solutions\n");
            report.push_str("- Review capacity planning and load testing procedures\n");
        }
        
        report
    }
}

/// Test breaking point recovery system
#[test]
fn test_breaking_point_recovery() -> Result<()> {
    println!("🔄 Starting breaking point recovery test");
    
    let config = RecoveryTestConfig {
        max_recovery_time_secs: 60,
        recovery_success_threshold: 0.75,
        recovery_grace_period_secs: 5,
        recovery_verification_cycles: 4,
        enable_automatic_load_shedding: true,
        enable_circuit_breaker: true,
        circuit_breaker_failure_threshold: 3,
        circuit_breaker_timeout_secs: 10,
    };
    
    println!("Configuration:");
    println!("  - Max recovery time: {} seconds", config.max_recovery_time_secs);
    println!("  - Recovery success threshold: {:.1}%", config.recovery_success_threshold * 100.0);
    println!("  - Grace period: {} seconds", config.recovery_grace_period_secs);
    println!("  - Verification cycles: {}", config.recovery_verification_cycles);
    println!("  - Circuit breaker enabled: {}", config.enable_circuit_breaker);
    println!();
    
    // Create recovery tester
    let tester = BreakingPointRecoveryTester::new(config);
    
    // Establish baseline metrics
    let baseline_metrics = SystemPerformanceSnapshot::capture(100);
    tester.set_baseline_metrics(baseline_metrics);
    
    println!("📊 Baseline metrics established");
    println!("  - Memory: {:.1} MB", baseline_metrics.memory_mb);
    println!("  - CPU: {:.1}%", baseline_metrics.cpu_percent);
    println!("  - Response time: {:.1} ms", baseline_metrics.response_time_ms);
    println!("  - Throughput: {:.1} ops/sec", baseline_metrics.throughput_ops_per_sec);
    println!();
    
    // Simulate breaking point condition
    println!("⚠️  Simulating breaking point condition...");
    tester.update_operation_load(500); // High load that would trigger breaking point
    thread::sleep(Duration::from_secs(2));
    
    // Test different recovery strategies
    let strategies = vec![
        RecoveryStrategy::ImmediateLoadShedding,
        RecoveryStrategy::GradualLoadReduction,
        RecoveryStrategy::CircuitBreaker,
        RecoveryStrategy::AdaptiveRecovery,
    ];
    
    for (i, strategy) in strategies.iter().enumerate() {
        println!("🧪 Testing recovery strategy {}: {:?}", i + 1, strategy);
        
        // Reset to breaking point condition before each test
        tester.update_operation_load(450 + i * 20); // Slightly different load each time
        thread::sleep(Duration::from_millis(500));
        
        // Execute recovery test
        tester.trigger_recovery_test(strategy.clone())?;
        
        // Brief pause between tests
        thread::sleep(Duration::from_secs(2));
    }
    
    // Get comprehensive results
    let results = tester.get_results();
    
    // Generate and display report
    let report = results.generate_report();
    println!("\n{}", report);
    
    // Verify recovery test results
    assert!(!results.attempts.is_empty(), "Should have recorded recovery attempts");
    assert_eq!(results.attempts.len(), 4, "Should have tested 4 recovery strategies");
    
    println!("✅ Recovery test verification:");
    println!("  🔄 Recovery attempts: {}", results.attempts.len());
    println!("  🎯 Final phase: {:?}", results.final_phase);
    println!("  🛡️  Safe operation load: {}", results.safe_operation_load);
    
    let successful_recoveries = results.attempts.iter().filter(|a| a.success).count();
    println!("  ✅ Successful recoveries: {}/{}", successful_recoveries, results.attempts.len());
    
    // Analyze effectiveness of each strategy
    for attempt in &results.attempts {
        println!("  📈 {:?}: Success={}, Effectiveness={:.1}%, Duration={:?}", 
                attempt.strategy, attempt.success, 
                attempt.effectiveness_score * 100.0, attempt.duration);
    }
    
    // Verify at least one recovery strategy worked
    assert!(successful_recoveries > 0, "At least one recovery strategy should succeed");
    
    // Verify safe operation load was determined
    assert!(results.safe_operation_load > 0, "Should determine safe operation load");
    assert!(results.safe_operation_load < 500, "Safe load should be less than breaking point load");
    
    println!("🎯 Breaking point recovery test completed successfully");
    
    Ok(())
}