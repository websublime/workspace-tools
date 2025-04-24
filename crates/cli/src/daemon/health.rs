
use serde::{Serialize, Deserialize};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthInfo {
    pub status: HealthStatus,
    pub uptime_seconds: u64,
    pub memory_usage_kb: u64,
    pub repository_count: usize,
    pub last_check: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl HealthInfo {
    pub fn new(uptime: Duration, repo_count: usize) -> Self {
        let memory_usage = crate::common::utils::get_process_memory_usage();
        
        Self {
            status: HealthStatus::Healthy, // We could add more sophisticated health checking
            uptime_seconds: uptime.as_secs(),
            memory_usage_kb: memory_usage,
            repository_count: repo_count,
            last_check: SystemTime::now(),
        }
    }

    pub fn format_memory_usage(&self) -> String {
        if self.memory_usage_kb > 1024 {
            format!("{:.2} MB", self.memory_usage_kb as f64 / 1024.0)
        } else {
            format!("{} KB", self.memory_usage_kb)
        }
    }
}

