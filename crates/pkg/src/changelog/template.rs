use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::changelog::entry::ChangelogEntry;

/// Grouped changelog section by type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogSection {
    /// Section title (Features, Bug Fixes, etc.)
    pub title: String,
    /// Entries in this section
    pub entries: Vec<ChangelogEntry>,
}

/// Template context for changelog generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContext {
    /// Package name
    pub package_name: String,
    /// Release version
    pub version: String,
    /// Release date
    pub date: String,
    /// Changelog sections
    pub sections: Vec<ChangelogSection>,
    /// Total number of commits
    pub total_commits: u32,
    /// Custom variables
    pub variables: HashMap<String, String>,
}

impl ChangelogSection {
    /// Creates a new changelog section.
    ///
    /// # Arguments
    ///
    /// * `title` - Section title
    #[must_use]
    pub fn new(title: String) -> Self {
        Self { title, entries: Vec::new() }
    }

    /// Adds an entry to the section.
    ///
    /// # Arguments
    ///
    /// * `entry` - The changelog entry to add
    pub fn add_entry(&mut self, entry: ChangelogEntry) {
        self.entries.push(entry);
    }

    /// Gets the number of entries in this section.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Checks if the section is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Formats the section as Markdown.
    #[must_use]
    pub fn format_markdown(&self) -> String {
        if self.is_empty() {
            return String::new();
        }

        let mut result = format!("### {}\n\n", self.title);

        for entry in &self.entries {
            result.push_str(&format!("- {}\n", entry.format_markdown()));
        }

        result.push('\n');
        result
    }
}
