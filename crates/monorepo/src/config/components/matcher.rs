//! Pattern matching component
//!
//! Provides efficient pattern matching operations for workspace patterns,
//! package paths, and other glob-based matching operations.

use crate::error::{Error, Result};
use glob::Pattern;
use std::collections::HashMap;

/// Efficient pattern matcher for repeated matching operations
pub struct PatternMatcher {
    pattern: Pattern,
    pattern_string: String,
}

impl PatternMatcher {
    /// Create a new pattern matcher
    ///
    /// # Arguments
    /// * `pattern` - Compiled glob pattern
    ///
    /// # Returns
    /// New pattern matcher instance
    #[must_use]
    pub fn new(pattern: Pattern) -> Self {
        let pattern_string = pattern.as_str().to_string();
        Self {
            pattern,
            pattern_string,
        }
    }

    /// Create a pattern matcher from a string
    ///
    /// # Arguments
    /// * `pattern_str` - Pattern string to compile
    ///
    /// # Returns
    /// New pattern matcher instance
    ///
    /// # Errors
    /// Returns an error if the pattern is invalid
    pub fn from_str(pattern_str: &str) -> Result<Self> {
        let pattern = Pattern::new(pattern_str)
            .map_err(|e| Error::config(format!("Invalid glob pattern '{pattern_str}': {e}")))?;
        
        Ok(Self::new(pattern))
    }

    /// Check if the pattern matches a given path
    ///
    /// # Arguments
    /// * `path` - Path to test against the pattern
    ///
    /// # Returns
    /// True if the pattern matches the path
    #[must_use]
    pub fn matches(&self, path: &str) -> bool {
        // Try exact match first
        if self.pattern.matches(path) {
            return true;
        }
        
        // Try with trailing slash for directories
        let path_with_slash = format!("{path}/", path = path.trim_end_matches('/'));
        if self.pattern.matches(&path_with_slash) {
            return true;
        }
        
        // Try matching just the last component (package name)
        if let Some(last_component) = path.split('/').last() {
            if self.pattern.matches(last_component) {
                return true;
            }
        }
        
        false
    }

    /// Check if the pattern matches any of the given paths
    ///
    /// # Arguments
    /// * `paths` - Paths to test against the pattern
    ///
    /// # Returns
    /// True if the pattern matches at least one path
    #[must_use]
    pub fn matches_any(&self, paths: &[String]) -> bool {
        paths.iter().any(|path| self.matches(path))
    }

    /// Get all paths that match this pattern
    ///
    /// # Arguments
    /// * `paths` - Paths to test against the pattern
    ///
    /// # Returns
    /// Vector of paths that match the pattern
    #[must_use]
    pub fn filter_matches(&self, paths: &[String]) -> Vec<String> {
        paths.iter()
            .filter(|path| self.matches(path))
            .cloned()
            .collect()
    }

    /// Get the pattern string
    #[must_use]
    pub fn pattern_string(&self) -> &str {
        &self.pattern_string
    }

    /// Get statistics about matching
    ///
    /// # Arguments
    /// * `paths` - Paths to analyze
    ///
    /// # Returns
    /// Tuple of (total_paths, matching_paths, match_percentage)
    #[must_use]
    pub fn get_match_stats(&self, paths: &[String]) -> (usize, usize, f64) {
        let total_paths = paths.len();
        let matching_paths = paths.iter().filter(|path| self.matches(path)).count();
        let match_percentage = if total_paths > 0 {
            #[allow(clippy::cast_precision_loss)]
            {
                (matching_paths as f64 / total_paths as f64) * 100.0
            }
        } else {
            0.0
        };
        
        (total_paths, matching_paths, match_percentage)
    }
}

impl std::fmt::Debug for PatternMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PatternMatcher")
            .field("pattern", &self.pattern_string)
            .finish()
    }
}

/// Multi-pattern matcher for efficient batch operations
pub struct MultiPatternMatcher {
    matchers: Vec<PatternMatcher>,
}

impl MultiPatternMatcher {
    /// Create a new multi-pattern matcher
    #[must_use]
    pub fn new() -> Self {
        Self {
            matchers: Vec::new(),
        }
    }

    /// Create a multi-pattern matcher from pattern strings
    ///
    /// # Arguments
    /// * `patterns` - Pattern strings to compile
    ///
    /// # Returns
    /// New multi-pattern matcher instance
    ///
    /// # Errors
    /// Returns an error if any pattern is invalid
    pub fn from_patterns(patterns: &[String]) -> Result<Self> {
        let mut matchers = Vec::new();
        
        for pattern_str in patterns {
            let matcher = PatternMatcher::from_str(pattern_str)?;
            matchers.push(matcher);
        }
        
        Ok(Self { matchers })
    }

    /// Add a pattern to the matcher
    ///
    /// # Arguments
    /// * `pattern` - Pattern string to add
    ///
    /// # Errors
    /// Returns an error if the pattern is invalid
    pub fn add_pattern(&mut self, pattern: &str) -> Result<()> {
        let matcher = PatternMatcher::from_str(pattern)?;
        self.matchers.push(matcher);
        Ok(())
    }

    /// Check if any pattern matches the given path
    ///
    /// # Arguments
    /// * `path` - Path to test against all patterns
    ///
    /// # Returns
    /// True if any pattern matches the path
    #[must_use]
    pub fn matches_any(&self, path: &str) -> bool {
        self.matchers.iter().any(|matcher| matcher.matches(path))
    }

    /// Check if all patterns match the given path
    ///
    /// # Arguments
    /// * `path` - Path to test against all patterns
    ///
    /// # Returns
    /// True if all patterns match the path
    #[must_use]
    pub fn matches_all(&self, path: &str) -> bool {
        self.matchers.iter().all(|matcher| matcher.matches(path))
    }

    /// Get all patterns that match the given path
    ///
    /// # Arguments
    /// * `path` - Path to test against all patterns
    ///
    /// # Returns
    /// Vector of pattern strings that match the path
    #[must_use]
    pub fn get_matching_patterns(&self, path: &str) -> Vec<String> {
        self.matchers
            .iter()
            .filter(|matcher| matcher.matches(path))
            .map(|matcher| matcher.pattern_string().to_string())
            .collect()
    }

    /// Batch match all patterns against all paths
    ///
    /// # Arguments
    /// * `paths` - Paths to test against all patterns
    ///
    /// # Returns
    /// Map of pattern -> list of matching paths
    #[must_use]
    pub fn batch_match(&self, paths: &[String]) -> HashMap<String, Vec<String>> {
        let mut results = HashMap::new();
        
        for matcher in &self.matchers {
            let matches = matcher.filter_matches(paths);
            results.insert(matcher.pattern_string().to_string(), matches);
        }
        
        results
    }

    /// Get paths that match at least one pattern
    ///
    /// # Arguments
    /// * `paths` - Paths to test
    ///
    /// # Returns
    /// Vector of paths that match at least one pattern
    #[must_use]
    pub fn get_any_matches(&self, paths: &[String]) -> Vec<String> {
        paths.iter()
            .filter(|path| self.matches_any(path))
            .cloned()
            .collect()
    }

    /// Get paths that match all patterns
    ///
    /// # Arguments
    /// * `paths` - Paths to test
    ///
    /// # Returns
    /// Vector of paths that match all patterns
    #[must_use]
    pub fn get_all_matches(&self, paths: &[String]) -> Vec<String> {
        if self.matchers.is_empty() {
            return Vec::new();
        }
        
        paths.iter()
            .filter(|path| self.matches_all(path))
            .cloned()
            .collect()
    }

    /// Get the number of patterns
    #[must_use]
    pub fn pattern_count(&self) -> usize {
        self.matchers.len()
    }

    /// Get all pattern strings
    #[must_use]
    pub fn pattern_strings(&self) -> Vec<String> {
        self.matchers
            .iter()
            .map(|matcher| matcher.pattern_string().to_string())
            .collect()
    }

    /// Check if the matcher is empty (no patterns)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.matchers.is_empty()
    }

    /// Clear all patterns
    pub fn clear(&mut self) {
        self.matchers.clear();
    }

    /// Get comprehensive matching statistics
    ///
    /// # Arguments
    /// * `paths` - Paths to analyze
    ///
    /// # Returns
    /// Map of pattern -> (total_paths, matching_paths, match_percentage)
    #[must_use]
    pub fn get_comprehensive_stats(&self, paths: &[String]) -> HashMap<String, (usize, usize, f64)> {
        let mut stats = HashMap::new();
        
        for matcher in &self.matchers {
            let (total, matching, percentage) = matcher.get_match_stats(paths);
            stats.insert(matcher.pattern_string().to_string(), (total, matching, percentage));
        }
        
        stats
    }
}

impl Default for MultiPatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for MultiPatternMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiPatternMatcher")
            .field("patterns", &self.pattern_strings())
            .finish()
    }
}