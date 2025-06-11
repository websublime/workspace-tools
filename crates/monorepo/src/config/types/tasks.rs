//! Task management configuration types

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
        }
    }
}