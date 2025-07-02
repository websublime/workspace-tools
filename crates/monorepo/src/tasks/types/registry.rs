//! Task registry type definitions

use super::{TaskDefinition, TaskScope};
use std::collections::HashMap;

/// Registry for storing and managing task definitions
#[derive(Debug, Clone)]
pub struct TaskRegistry {
    /// All registered tasks indexed by name
    pub(crate) tasks: HashMap<String, TaskDefinition>,

    /// Tasks organized by scope for quick filtering
    pub(crate) scope_index: HashMap<TaskScope, Vec<String>>,

    /// Tasks organized by priority
    pub(crate) priority_index: HashMap<u32, Vec<String>>,
}
