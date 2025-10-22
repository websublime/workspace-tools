//! Changelog parser for reading and parsing existing CHANGELOG.md files.
//!
//! **What**: Provides functionality to parse existing changelog files and extract
//! version information, dates, and content sections.
//!
//! **How**: This module reads changelog files in various formats (Keep a Changelog,
//! Conventional Commits, etc.) and extracts structured data including version numbers,
//! release dates, and section content.
//!
//! **Why**: To support updating existing changelogs while preserving their content
//! and format, and to enable querying historical changelog information.

use crate::error::{ChangelogError, ChangelogResult};
use chrono::{DateTime, NaiveDate, Utc};
use regex::Regex;
use std::collections::HashMap;

/// Represents a parsed version section from a changelog.
///
/// Contains all information about a single version entry in the changelog,
/// including the version number, release date, and content.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::parser::ParsedVersion;
///
/// let version = ParsedVersion {
///     version: "1.0.0".to_string(),
///     date: Some(chrono::Utc::now()),
///     content: "## Features\n- Add new feature".to_string(),
///     raw_header: "## [1.0.0] - 2024-01-15".to_string(),
/// };
///
/// println!("Version: {}", version.version);
/// ```
#[derive(Debug, Clone)]
pub struct ParsedVersion {
    /// The version string (e.g., "1.0.0").
    pub version: String,

    /// The release date, if present in the changelog.
    pub date: Option<DateTime<Utc>>,

    /// The content of this version section (everything between this version and the next).
    pub content: String,

    /// The raw header line as it appears in the changelog.
    pub raw_header: String,
}

/// Represents a parsed changelog file.
///
/// Contains the header content and all parsed version sections.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::parser::ParsedChangelog;
///
/// let changelog = ParsedChangelog {
///     header: "# Changelog\n\nAll notable changes...".to_string(),
///     versions: vec![],
/// };
///
/// println!("Found {} versions", changelog.versions.len());
/// ```
#[derive(Debug, Clone)]
pub struct ParsedChangelog {
    /// The header content (everything before the first version).
    pub header: String,

    /// All parsed version sections, ordered as they appear in the file.
    pub versions: Vec<ParsedVersion>,
}

impl ParsedChangelog {
    /// Gets a specific version from the parsed changelog.
    ///
    /// # Arguments
    ///
    /// * `version` - The version string to search for
    ///
    /// # Returns
    ///
    /// An optional reference to the `ParsedVersion` if found.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::parser::ParsedChangelog;
    ///
    /// let changelog = ParsedChangelog {
    ///     header: "# Changelog".to_string(),
    ///     versions: vec![],
    /// };
    ///
    /// if let Some(version) = changelog.get_version("1.0.0") {
    ///     println!("Found version: {}", version.version);
    /// }
    /// ```
    #[must_use]
    pub fn get_version(&self, version: &str) -> Option<&ParsedVersion> {
        self.versions.iter().find(|v| v.version == version)
    }

    /// Gets the most recent version from the changelog.
    ///
    /// # Returns
    ///
    /// An optional reference to the most recent `ParsedVersion`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::parser::ParsedChangelog;
    ///
    /// let changelog = ParsedChangelog {
    ///     header: "# Changelog".to_string(),
    ///     versions: vec![],
    /// };
    ///
    /// if let Some(latest) = changelog.latest_version() {
    ///     println!("Latest version: {}", latest.version);
    /// }
    /// ```
    #[must_use]
    pub fn latest_version(&self) -> Option<&ParsedVersion> {
        self.versions.first()
    }

    /// Gets all version strings in order.
    ///
    /// # Returns
    ///
    /// A vector of version strings.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::parser::ParsedChangelog;
    ///
    /// let changelog = ParsedChangelog {
    ///     header: "# Changelog".to_string(),
    ///     versions: vec![],
    /// };
    ///
    /// for version in changelog.version_list() {
    ///     println!("Version: {}", version);
    /// }
    /// ```
    #[must_use]
    pub fn version_list(&self) -> Vec<&str> {
        self.versions.iter().map(|v| v.version.as_str()).collect()
    }

    /// Checks if the changelog contains a specific version.
    ///
    /// # Arguments
    ///
    /// * `version` - The version string to check
    ///
    /// # Returns
    ///
    /// `true` if the version exists, `false` otherwise.
    #[must_use]
    pub fn has_version(&self, version: &str) -> bool {
        self.versions.iter().any(|v| v.version == version)
    }

    /// Gets the number of versions in the changelog.
    ///
    /// # Returns
    ///
    /// The count of parsed versions.
    #[must_use]
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }
}

/// Parser for changelog files.
///
/// Provides functionality to parse changelog files in various formats and
/// extract structured data.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::parser::ChangelogParser;
///
/// let parser = ChangelogParser::new();
/// let content = "# Changelog\n\n## [1.0.0] - 2024-01-15\n- Initial release";
/// let parsed = parser.parse(content)?;
///
/// println!("Found {} versions", parsed.versions.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone)]
pub struct ChangelogParser {
    // Parser uses static regexes, no fields needed
}

impl ChangelogParser {
    /// Creates a new changelog parser.
    ///
    /// # Returns
    ///
    /// A new `ChangelogParser` instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::parser::ChangelogParser;
    ///
    /// let parser = ChangelogParser::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    /// Parses a changelog file content.
    ///
    /// This method parses the entire changelog content and extracts the header
    /// and all version sections.
    ///
    /// # Arguments
    ///
    /// * `content` - The full changelog file content
    ///
    /// # Returns
    ///
    /// A `ParsedChangelog` containing the structured data.
    ///
    /// # Errors
    ///
    /// Returns an error if the changelog format is invalid or cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::parser::ChangelogParser;
    ///
    /// let parser = ChangelogParser::new();
    /// let content = "# Changelog\n\n## [1.0.0] - 2024-01-15\n- Initial release";
    /// let parsed = parser.parse(content)?;
    ///
    /// assert_eq!(parsed.versions.len(), 1);
    /// assert_eq!(parsed.versions[0].version, "1.0.0");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse(&self, content: &str) -> ChangelogResult<ParsedChangelog> {
        let lines: Vec<&str> = content.lines().collect();

        // Find all version headers
        let version_indices = self.find_version_indices(&lines)?;

        // Extract header (everything before first version)
        let header_end = version_indices.first().copied().unwrap_or(lines.len());
        let header = lines[..header_end].join("\n");

        // Parse each version section
        let mut versions = Vec::new();
        for i in 0..version_indices.len() {
            let start = version_indices[i];
            let end = version_indices.get(i + 1).copied().unwrap_or(lines.len());

            if let Some(parsed_version) = self.parse_version_section(&lines[start..end], start)? {
                versions.push(parsed_version);
            }
        }

        Ok(ParsedChangelog { header, versions })
    }

    /// Finds the indices of all version header lines.
    ///
    /// # Arguments
    ///
    /// * `lines` - All lines of the changelog
    ///
    /// # Returns
    ///
    /// A vector of line indices where version headers are found.
    fn find_version_indices(&self, lines: &[&str]) -> ChangelogResult<Vec<usize>> {
        let version_regex =
            Regex::new(r"^##\s+\[?v?(\d+\.\d+\.\d+(?:-[a-zA-Z0-9.]+)?(?:\+[a-zA-Z0-9.]+)?)\]?")
                .map_err(|e| ChangelogError::ParseError {
                    line: 0,
                    reason: format!("Failed to compile version regex: {}", e),
                })?;

        Ok(lines
            .iter()
            .enumerate()
            .filter_map(|(i, line)| if version_regex.is_match(line) { Some(i) } else { None })
            .collect())
    }

    /// Parses a single version section.
    ///
    /// # Arguments
    ///
    /// * `lines` - The lines of this version section
    /// * `start_line` - The starting line number for error reporting
    ///
    /// # Returns
    ///
    /// An optional `ParsedVersion` if parsing succeeds.
    ///
    /// # Errors
    ///
    /// Returns an error if the version section format is invalid.
    fn parse_version_section(
        &self,
        lines: &[&str],
        start_line: usize,
    ) -> ChangelogResult<Option<ParsedVersion>> {
        if lines.is_empty() {
            return Ok(None);
        }

        let header_line = lines[0];

        // Extract version number
        let version = self.extract_version(header_line, start_line)?;

        // Extract date if present
        let date = self.extract_date(header_line);

        // Collect content (everything after the header)
        let content = if lines.len() > 1 { lines[1..].join("\n") } else { String::new() };

        Ok(Some(ParsedVersion { version, date, content, raw_header: header_line.to_string() }))
    }

    /// Extracts the version string from a header line.
    ///
    /// # Arguments
    ///
    /// * `line` - The header line
    /// * `line_num` - Line number for error reporting
    ///
    /// # Returns
    ///
    /// The extracted version string.
    ///
    /// # Errors
    ///
    /// Returns an error if the version cannot be extracted.
    fn extract_version(&self, line: &str, line_num: usize) -> ChangelogResult<String> {
        let version_regex =
            Regex::new(r"^##\s+\[?v?(\d+\.\d+\.\d+(?:-[a-zA-Z0-9.]+)?(?:\+[a-zA-Z0-9.]+)?)\]?")
                .map_err(|e| ChangelogError::ParseError {
                    line: line_num,
                    reason: format!("Failed to compile version regex: {}", e),
                })?;

        version_regex
            .captures(line)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| ChangelogError::ParseError {
                line: line_num,
                reason: format!("Failed to extract version from: {}", line),
            })
    }

    /// Extracts the date from a header line.
    ///
    /// # Arguments
    ///
    /// * `line` - The header line
    ///
    /// # Returns
    ///
    /// An optional `DateTime<Utc>` if a date is found and can be parsed.
    pub(crate) fn extract_date(&self, line: &str) -> Option<DateTime<Utc>> {
        // If regex compilation fails, return None (date parsing is optional)
        let date_regex = Regex::new(r"(\d{4}[-/]\d{2}[-/]\d{2}|\d{2}[-/]\d{2}[-/]\d{4})").ok()?;

        date_regex.captures(line).and_then(|caps| {
            caps.get(1).and_then(|m| {
                let date_str = m.as_str();
                self.parse_date_string(date_str)
            })
        })
    }

    /// Parses a date string into a DateTime.
    ///
    /// Supports various date formats:
    /// - YYYY-MM-DD
    /// - YYYY/MM/DD
    /// - DD-MM-YYYY
    /// - DD/MM/YYYY
    ///
    /// # Arguments
    ///
    /// * `date_str` - The date string to parse
    ///
    /// # Returns
    ///
    /// An optional `DateTime<Utc>` if parsing succeeds.
    fn parse_date_string(&self, date_str: &str) -> Option<DateTime<Utc>> {
        // Try YYYY-MM-DD or YYYY/MM/DD
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            return Some(date.and_hms_opt(0, 0, 0)?.and_utc());
        }
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y/%m/%d") {
            return Some(date.and_hms_opt(0, 0, 0)?.and_utc());
        }

        // Try DD-MM-YYYY or DD/MM/YYYY
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%d-%m-%Y") {
            return Some(date.and_hms_opt(0, 0, 0)?.and_utc());
        }
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%d/%m/%Y") {
            return Some(date.and_hms_opt(0, 0, 0)?.and_utc());
        }

        None
    }

    /// Parses a changelog and returns a map of versions to their content.
    ///
    /// This is a convenience method for quickly accessing version content.
    ///
    /// # Arguments
    ///
    /// * `content` - The full changelog file content
    ///
    /// # Returns
    ///
    /// A `HashMap` mapping version strings to their content.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::parser::ChangelogParser;
    ///
    /// let parser = ChangelogParser::new();
    /// let content = "# Changelog\n\n## [1.0.0] - 2024-01-15\n- Initial release";
    /// let versions = parser.parse_to_map(content)?;
    ///
    /// if let Some(content) = versions.get("1.0.0") {
    ///     println!("Version 1.0.0 content: {}", content);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse_to_map(&self, content: &str) -> ChangelogResult<HashMap<String, String>> {
        let parsed = self.parse(content)?;
        Ok(parsed.versions.into_iter().map(|v| (v.version, v.content)).collect())
    }
}

impl Default for ChangelogParser {
    fn default() -> Self {
        Self::new()
    }
}
