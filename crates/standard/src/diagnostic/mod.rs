//! Diagnostic collection and reporting system.
//!
//! What:
//! This module provides tools for collecting, managing, and reporting diagnostic
//! information throughout the system. It helps track operations, performance metrics,
//! and potential issues for debugging and analysis purposes.
//!
//! Who:
//! Used by developers who need to:
//! - Collect diagnostic information during operations
//! - Track performance metrics across different components
//! - Identify potential issues or optimizations
//! - Generate diagnostic reports for troubleshooting
//!
//! Why:
//! Diagnostic information is essential for:
//! - Troubleshooting complex operations
//! - Performance analysis and optimization
//! - Error recovery strategies
//! - Operational visibility

mod collector;

pub use collector::{DiagnosticCollector, DiagnosticEntry, DiagnosticLevel};
