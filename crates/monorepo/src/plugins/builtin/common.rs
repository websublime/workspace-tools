//! Common utilities and helper functions shared across builtin plugins
//!
//! This module provides reusable functionality that is used by multiple
//! builtin plugins, promoting code reuse and consistency.

use super::super::types::PluginResult;
use std::time::Instant;

/// Creates a standardized error result for uninitialized plugins
pub(crate) fn plugin_not_initialized_error() -> PluginResult {
    PluginResult::error("Plugin not initialized. Call initialize() first.".to_string())
}

/// Creates a standardized error result for unknown commands
pub(crate) fn unknown_command_error(command: &str) -> PluginResult {
    PluginResult::error(format!("Unknown command: {command}"))
}

/// Creates a success result with execution timing
pub(crate) fn success_with_timing<T>(data: T, start_time: Instant) -> PluginResult 
where
    T: serde::Serialize,
{
    let execution_time = start_time.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
    PluginResult::success_with_time(data, execution_time)
}

