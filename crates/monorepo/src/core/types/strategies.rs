//! Versioning strategy implementations

/// Default versioning strategy implementation
#[derive(Debug, Clone)]
pub struct DefaultVersioningStrategy;

/// Conservative versioning strategy - minimal propagation
#[derive(Debug, Clone)]
pub struct ConservativeVersioningStrategy;

/// Aggressive versioning strategy - propagates all changes
#[derive(Debug, Clone)]
pub struct AggressiveVersioningStrategy;