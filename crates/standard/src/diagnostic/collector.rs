use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Severity level for diagnostic information.
///
/// Represents the importance and impact of a diagnostic entry.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::diagnostic::DiagnosticLevel;
///
/// let info = DiagnosticLevel::Info;
/// let error = DiagnosticLevel::Error;
///
/// // Compare severity levels
/// assert!(error > info);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DiagnosticLevel {
    /// Informational message, no impact
    Info,
    /// Warning message, potential issues
    Warning,
    /// Error message, operation failed
    Error,
    /// Critical message, system integrity compromised
    Critical,
}

/// A single diagnostic entry with context and metadata.
///
/// Represents a piece of diagnostic information with associated context,
/// severity level, and timing information.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::diagnostic::{DiagnosticEntry, DiagnosticLevel};
/// use std::time::Duration;
///
/// let entry = DiagnosticEntry::new(
///     DiagnosticLevel::Warning,
///     "fs_operations",
///     "Slow file read detected"
/// ).with_data("file", "large_data.json")
///   .with_duration(Duration::from_millis(1500));
/// ```
#[derive(Debug, Clone)]
pub struct DiagnosticEntry {
    /// Timestamp when the entry was created
    pub timestamp: Instant,
    /// Severity level of the diagnostic
    pub level: DiagnosticLevel,
    /// Context where the diagnostic occurred
    pub context: String,
    /// Message describing the diagnostic
    pub message: String,
    /// Additional structured data
    pub data: HashMap<String, String>,
    /// Duration of operation, if applicable
    pub duration: Option<Duration>,
}

impl DiagnosticEntry {
    /// Creates a new diagnostic entry.
    ///
    /// # Arguments
    ///
    /// * `level` - Severity level of the diagnostic
    /// * `context` - The context where the diagnostic occurred
    /// * `message` - Descriptive message about the diagnostic
    ///
    /// # Returns
    ///
    /// A new diagnostic entry with the given information
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::diagnostic::{DiagnosticEntry, DiagnosticLevel};
    ///
    /// let entry = DiagnosticEntry::new(
    ///     DiagnosticLevel::Info,
    ///     "initialization",
    ///     "System startup complete"
    /// );
    /// ```
    #[must_use]
    pub fn new(
        level: DiagnosticLevel,
        context: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: Instant::now(),
            level,
            context: context.into(),
            message: message.into(),
            data: HashMap::new(),
            duration: None,
        }
    }

    /// Adds a key-value pair to the diagnostic data.
    ///
    /// # Arguments
    ///
    /// * `key` - Key for the data point
    /// * `value` - Value for the data point
    ///
    /// # Returns
    ///
    /// Self with added data for method chaining
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::diagnostic::{DiagnosticEntry, DiagnosticLevel};
    ///
    /// let entry = DiagnosticEntry::new(
    ///     DiagnosticLevel::Warning,
    ///     "disk_space",
    ///     "Low disk space detected"
    /// ).with_data("available_mb", "250")
    ///   .with_data("path", "/var/logs");
    /// ```
    #[must_use]
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }

    /// Sets the operation duration for this diagnostic.
    ///
    /// # Arguments
    ///
    /// * `duration` - Duration of the operation
    ///
    /// # Returns
    ///
    /// Self with added duration for method chaining
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::diagnostic::{DiagnosticEntry, DiagnosticLevel};
    /// use std::time::Duration;
    ///
    /// let entry = DiagnosticEntry::new(
    ///     DiagnosticLevel::Info,
    ///     "database",
    ///     "Query completed"
    /// ).with_duration(Duration::from_millis(350));
    /// ```
    #[must_use]
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }
}

/// Collector for diagnostic information.
///
/// Thread-safe collector for storing and retrieving diagnostic entries
/// with various severity levels.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::diagnostic::{DiagnosticCollector, DiagnosticLevel};
///
/// let collector = DiagnosticCollector::new();
///
/// // Record diagnostics
/// collector.info("startup", "Application initialized");
/// collector.warning("memory", "Memory usage above 80%");
/// collector.error("network", "Failed to connect to API");
///
/// // Get all entries with Warning or higher severity
/// let warnings_and_errors = collector.entries_with_level_at_or_above(DiagnosticLevel::Warning);
/// ```
#[derive(Debug, Clone)]
pub struct DiagnosticCollector {
    /// Collection of diagnostic entries
    entries: Arc<Mutex<Vec<DiagnosticEntry>>>,
    /// Maximum number of entries to store
    max_entries: usize,
}

impl DiagnosticCollector {
    /// Creates a new diagnostic collector with default settings.
    ///
    /// Default maximum entries is 1000.
    ///
    /// # Returns
    ///
    /// A new diagnostic collector
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::diagnostic::DiagnosticCollector;
    ///
    /// let collector = DiagnosticCollector::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { entries: Arc::new(Mutex::new(Vec::new())), max_entries: 1000 }
    }

    /// Creates a new diagnostic collector with a maximum entries limit.
    ///
    /// # Arguments
    ///
    /// * `max_entries` - Maximum number of entries to keep
    ///
    /// # Returns
    ///
    /// A new diagnostic collector with specified capacity
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::diagnostic::DiagnosticCollector;
    ///
    /// // Create a collector that stores up to 500 entries
    /// let collector = DiagnosticCollector::with_max_entries(500);
    /// ```
    #[must_use]
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self { entries: Arc::new(Mutex::new(Vec::new())), max_entries }
    }

    /// Adds a diagnostic entry to the collector.
    ///
    /// When the maximum number of entries is reached, the oldest entry is removed.
    ///
    /// # Arguments
    ///
    /// * `entry` - The diagnostic entry to add
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::diagnostic::{DiagnosticCollector, DiagnosticEntry, DiagnosticLevel};
    ///
    /// let collector = DiagnosticCollector::new();
    /// let entry = DiagnosticEntry::new(
    ///     DiagnosticLevel::Info,
    ///     "system",
    ///     "Operation completed"
    /// );
    ///
    /// collector.add(entry);
    /// ```
    pub fn add(&self, entry: DiagnosticEntry) {
        let mut entries = self.entries.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

        // Implement circular buffer if max entries reached
        if entries.len() >= self.max_entries {
            entries.remove(0);
        }

        entries.push(entry);
    }

    /// Records an informational diagnostic.
    ///
    /// # Arguments
    ///
    /// * `context` - The context where the diagnostic occurred
    /// * `message` - Descriptive message about the diagnostic
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::diagnostic::DiagnosticCollector;
    ///
    /// let collector = DiagnosticCollector::new();
    /// collector.info("startup", "System initialized successfully");
    /// ```
    pub fn info(&self, context: impl Into<String>, message: impl Into<String>) {
        self.add(DiagnosticEntry::new(DiagnosticLevel::Info, context, message));
    }

    /// Records a warning diagnostic.
    ///
    /// # Arguments
    ///
    /// * `context` - The context where the diagnostic occurred
    /// * `message` - Descriptive message about the diagnostic
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::diagnostic::DiagnosticCollector;
    ///
    /// let collector = DiagnosticCollector::new();
    /// collector.warning("disk_space", "Low disk space detected");
    /// ```
    pub fn warning(&self, context: impl Into<String>, message: impl Into<String>) {
        self.add(DiagnosticEntry::new(DiagnosticLevel::Warning, context, message));
    }

    /// Records an error diagnostic.
    ///
    /// # Arguments
    ///
    /// * `context` - The context where the diagnostic occurred
    /// * `message` - Descriptive message about the diagnostic
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::diagnostic::DiagnosticCollector;
    ///
    /// let collector = DiagnosticCollector::new();
    /// collector.error("network", "Failed to connect to server");
    /// ```
    pub fn error(&self, context: impl Into<String>, message: impl Into<String>) {
        self.add(DiagnosticEntry::new(DiagnosticLevel::Error, context, message));
    }

    /// Records a critical diagnostic.
    ///
    /// # Arguments
    ///
    /// * `context` - The context where the diagnostic occurred
    /// * `message` - Descriptive message about the diagnostic
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::diagnostic::DiagnosticCollector;
    ///
    /// let collector = DiagnosticCollector::new();
    /// collector.critical("database", "Database connection lost");
    /// ```
    pub fn critical(&self, context: impl Into<String>, message: impl Into<String>) {
        self.add(DiagnosticEntry::new(DiagnosticLevel::Critical, context, message));
    }

    /// Gets all diagnostic entries.
    ///
    /// # Returns
    ///
    /// A vector containing all diagnostic entries
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::diagnostic::DiagnosticCollector;
    ///
    /// let collector = DiagnosticCollector::new();
    /// collector.info("system", "Test diagnostic");
    ///
    /// let all_entries = collector.entries();
    /// println!("Collected {} diagnostics", all_entries.len());
    /// ```
    pub fn entries(&self) -> Vec<DiagnosticEntry> {
        let entries = self.entries.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        entries.clone()
    }

    /// Gets entries with level at or above the specified level.
    ///
    /// # Arguments
    ///
    /// * `level` - The minimum severity level to include
    ///
    /// # Returns
    ///
    /// A vector containing diagnostic entries at or above the specified level
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::diagnostic::{DiagnosticCollector, DiagnosticLevel};
    ///
    /// let collector = DiagnosticCollector::new();
    /// collector.info("system", "Info message");
    /// collector.error("system", "Error message");
    ///
    /// // Get only errors and above
    /// let errors = collector.entries_with_level_at_or_above(DiagnosticLevel::Error);
    /// assert_eq!(errors.len(), 1);
    /// ```
    pub fn entries_with_level_at_or_above(&self, level: DiagnosticLevel) -> Vec<DiagnosticEntry> {
        let entries = self.entries.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        entries.iter().filter(|e| e.level >= level).cloned().collect()
    }

    /// Clears all diagnostic entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::diagnostic::DiagnosticCollector;
    ///
    /// let collector = DiagnosticCollector::new();
    /// collector.info("system", "Test diagnostic");
    /// assert_eq!(collector.entries().len(), 1);
    ///
    /// collector.clear();
    /// assert_eq!(collector.entries().len(), 0);
    /// ```
    pub fn clear(&self) {
        let mut entries = self.entries.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        entries.clear();
    }
}

impl Default for DiagnosticCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_diagnostic_entry() {
        let entry = DiagnosticEntry::new(DiagnosticLevel::Warning, "test_context", "test message")
            .with_data("key1", "value1")
            .with_data("key2", "value2")
            .with_duration(Duration::from_millis(100));

        assert_eq!(entry.level, DiagnosticLevel::Warning);
        assert_eq!(entry.context, "test_context");
        assert_eq!(entry.message, "test message");
        assert_eq!(entry.data.get("key1"), Some(&"value1".to_string()));
        assert_eq!(entry.data.get("key2"), Some(&"value2".to_string()));
        assert_eq!(entry.duration, Some(Duration::from_millis(100)));
    }

    #[test]
    fn test_collector_add_and_retrieve() {
        let collector = DiagnosticCollector::new();

        collector.info("context1", "info message");
        collector.warning("context2", "warning message");
        collector.error("context3", "error message");
        collector.critical("context4", "critical message");

        let entries = collector.entries();
        assert_eq!(entries.len(), 4);

        // Check levels
        assert_eq!(entries[0].level, DiagnosticLevel::Info);
        assert_eq!(entries[1].level, DiagnosticLevel::Warning);
        assert_eq!(entries[2].level, DiagnosticLevel::Error);
        assert_eq!(entries[3].level, DiagnosticLevel::Critical);

        // Check filtering
        let warnings_and_above = collector.entries_with_level_at_or_above(DiagnosticLevel::Warning);
        assert_eq!(warnings_and_above.len(), 3); // Warning, Error, Critical

        let errors_and_above = collector.entries_with_level_at_or_above(DiagnosticLevel::Error);
        assert_eq!(errors_and_above.len(), 2); // Error, Critical
    }

    #[test]
    fn test_collector_max_entries() {
        let collector = DiagnosticCollector::with_max_entries(3);

        // Add more entries than max
        collector.info("test1", "message1");
        collector.info("test2", "message2");
        collector.info("test3", "message3");
        collector.info("test4", "message4"); // Should replace the oldest

        let entries = collector.entries();
        assert_eq!(entries.len(), 3);

        // First entry should be removed
        assert_eq!(entries[0].message, "message2");
        assert_eq!(entries[1].message, "message3");
        assert_eq!(entries[2].message, "message4");
    }

    #[test]
    fn test_collector_clear() {
        let collector = DiagnosticCollector::new();

        collector.info("test", "message");
        assert_eq!(collector.entries().len(), 1);

        collector.clear();
        assert_eq!(collector.entries().len(), 0);
    }

    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_collector_thread_safety() {
        let collector = DiagnosticCollector::new();
        let collector_clone = collector.clone();

        let handle = thread::spawn(move || {
            for i in 0..10 {
                collector_clone.info("thread", format!("message {i}"));
            }
        });

        for i in 0..10 {
            collector.info("main", format!("message {i}"));
        }

        handle.join().unwrap();

        assert_eq!(collector.entries().len(), 20);
    }
}
