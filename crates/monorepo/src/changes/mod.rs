//! Change detection and analysis module
//!
//! This module provides comprehensive change detection capabilities including
//! configurable rules, detection engines, and change detectors for monorepos.

pub mod types;
mod detector;
mod rules;
mod engine;

#[cfg(test)]
mod tests;

// Re-export the main types for convenience
pub use types::*;