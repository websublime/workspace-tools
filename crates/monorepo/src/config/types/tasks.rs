//! Task management configuration types
//!
//! This module defines configuration structures for task execution, including
//! timeouts, concurrency limits, and performance tuning parameters.
//! 
//! ## What
//! Provides comprehensive task management configuration including:
//! - Basic task execution settings (timeouts, concurrency)
//! - Performance optimizations for different project sizes
//! - Hook execution configuration
//! - Impact level thresholds for workflow decisions
//! 
//! ## How
//! Uses structured configuration with defaults for different scenarios:
//! - Small projects: Lower concurrency, standard timeouts
//! - Large projects: Higher concurrency, extended timeouts
//! - Configurable thresholds for impact assessment
//! 
//! ## Why
//! Centralizes all timeout and concurrency values that were previously
//! hardcoded throughout the codebase, enabling users to tune performance
//! based on their specific project needs and CI/CD constraints.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::Environment;

/// Task management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksConfig {
    /// Default tasks to run on changes
    pub default_tasks: Vec<String>,

    /// Task groups
    pub groups: HashMap<String, Vec<String>>,

    /// Whether to run tasks in parallel
    pub parallel: bool,

    /// Maximum concurrent tasks
    pub max_concurrent: usize,

    /// Task timeout in seconds
    pub timeout: u64,

    /// Deployment tasks for each environment
    pub deployment_tasks: HashMap<Environment, Vec<String>>,

    /// Performance and timing configuration
    pub performance: TaskPerformanceConfig,
}

/// Performance and timing configuration for tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPerformanceConfig {
    /// Hook execution timeout in seconds
    pub hook_timeout: u64,

    /// Version planning estimation per package in seconds
    pub version_planning_per_package: u64,

    /// Cache duration for task results in seconds
    pub cache_duration: u64,

    /// Large project configuration overrides
    pub large_project: LargeProjectConfig,

    /// Workflow impact thresholds
    pub impact_thresholds: ImpactThresholdConfig,
}

/// Configuration for large project optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeProjectConfig {
    /// Maximum concurrent tasks for large projects
    pub max_concurrent: usize,

    /// Extended timeout for large project tasks in seconds
    pub timeout: u64,
}

/// Configuration for workflow impact level thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactThresholdConfig {
    /// File count threshold for medium impact level
    pub medium_impact_files: usize,

    /// File count threshold for high impact level
    pub high_impact_files: usize,
}

impl Default for TasksConfig {
    fn default() -> Self {
        Self {
            default_tasks: vec!["test".to_string(), "lint".to_string()],
            groups: HashMap::new(),
            parallel: true,
            max_concurrent: 4,
            timeout: 300, // 5 minutes
            deployment_tasks: HashMap::new(),
            performance: TaskPerformanceConfig::default(),
        }
    }
}

impl Default for TaskPerformanceConfig {
    fn default() -> Self {
        Self {
            hook_timeout: 300, // 5 minutes
            version_planning_per_package: 5, // 5 seconds per package
            cache_duration: 300, // 5 minutes
            large_project: LargeProjectConfig::default(),
            impact_thresholds: ImpactThresholdConfig::default(),
        }
    }
}

impl Default for LargeProjectConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 8,
            timeout: 600, // 10 minutes
        }
    }
}

impl Default for ImpactThresholdConfig {
    fn default() -> Self {
        Self {
            medium_impact_files: 5,
            high_impact_files: 15,
        }
    }
}

impl TasksConfig {
    /// Get the appropriate max concurrent value based on project size
    ///
    /// # Arguments
    /// 
    /// * `is_large_project` - Whether this is considered a large project
    /// 
    /// # Returns
    /// 
    /// The maximum number of concurrent tasks for this project size
    #[must_use]
    pub fn get_max_concurrent(&self, is_large_project: bool) -> usize {
        if is_large_project {
            self.performance.large_project.max_concurrent
        } else {
            self.max_concurrent
        }
    }
    
    /// Get the appropriate timeout value based on project size
    ///
    /// # Arguments
    /// 
    /// * `is_large_project` - Whether this is considered a large project
    /// 
    /// # Returns
    /// 
    /// The timeout in seconds for this project size
    #[must_use]
    pub fn get_timeout(&self, is_large_project: bool) -> u64 {
        if is_large_project {
            self.performance.large_project.timeout
        } else {
            self.timeout
        }
    }

    /// Get hook timeout duration
    #[must_use]
    pub fn get_hook_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.performance.hook_timeout)
    }

    /// Get version planning estimation per package
    #[must_use]
    pub fn get_version_planning_per_package(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.performance.version_planning_per_package)
    }

    /// Get cache duration
    #[must_use]
    pub fn get_cache_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.performance.cache_duration)
    }
}