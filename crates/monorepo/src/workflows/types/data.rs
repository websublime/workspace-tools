//! Workflow data types
//!
//! Simple data structures used within workflow implementations.

/// Simple facts about package changes - no decisions, just data
///
/// This struct contains only factual information about changes,
/// without any analysis or decision-making logic.
///
/// # Examples
/// 
/// ```rust
/// use sublime_monorepo_tools::workflows::types::PackageChangeFacts;
/// 
/// let facts = PackageChangeFacts {
///     total_files: 5,
///     files_changed: vec!["src/main.rs".to_string(), "Cargo.toml".to_string()],
/// };
/// ```
#[derive(Debug, Clone)]
pub struct PackageChangeFacts {
    /// Total number of files changed
    pub total_files: usize,
    /// List of all files that changed
    pub files_changed: Vec<String>,
}