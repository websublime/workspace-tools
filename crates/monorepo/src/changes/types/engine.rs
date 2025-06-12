//! Change detection engine type definitions

use super::{
    ChangeDetectionRules, ChangeTypeRule, FilePattern, PatternType, RuleConditions,
    SignificanceRule,
};
use glob::Pattern;
use regex::Regex;
use std::collections::HashMap;

/// Result type for compiled patterns
pub(crate) enum CompiledPattern<T> {
    /// Successfully compiled pattern
    Valid(T),
    /// Failed to compile, stores the error message
    Invalid(()),
}

/// Configurable change detection engine
pub struct ChangeDetectionEngine {
    /// Rules configuration
    pub(crate) rules: ChangeDetectionRules,

    /// Compiled regex patterns cache
    pub(crate) regex_cache: HashMap<String, CompiledPattern<Regex>>,

    /// Compiled glob patterns cache
    pub(crate) glob_cache: HashMap<String, CompiledPattern<Pattern>>,
}