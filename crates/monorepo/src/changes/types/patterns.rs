//! File pattern and condition types for change detection

use serde::{Deserialize, Serialize};

/// File pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePattern {
    /// Pattern type
    pub pattern_type: PatternType,

    /// The pattern itself
    pub pattern: String,

    /// Whether the pattern should match or exclude
    pub exclude: bool,
}

/// Types of file patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// Glob pattern (e.g., "src/**/*.ts")
    Glob,

    /// Regular expression
    Regex,

    /// Simple path contains
    Contains,

    /// File extension
    Extension,

    /// Exact path match
    Exact,
}

/// Additional rule conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConditions {
    /// Minimum number of files that must match
    pub min_files: Option<usize>,

    /// Maximum number of files that must match
    pub max_files: Option<usize>,

    /// File size thresholds
    pub file_size: Option<FileSizeCondition>,

    /// Custom script to run for validation
    pub custom_script: Option<String>,
}

/// File size conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSizeCondition {
    /// Minimum total size of changed files
    pub min_total_size: Option<u64>,

    /// Maximum total size of changed files
    pub max_total_size: Option<u64>,

    /// Minimum size of largest changed file
    pub min_largest_file: Option<u64>,
}
