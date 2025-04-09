//! Functionality for generating changelogs from tracked changes.
//!
//! This module extends the VersionManager with functionality to generate Markdown
//! changelogs based on changes tracked in the change tracker. It supports
//! customizable formatting, updating existing changelogs, and organizing
//! changes by version and type.

use crate::{Change, ChangelogOptions, VersionManager, VersioningResult};
use chrono::Utc;
use log::warn;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

/// Implementation of changelog generation functionality for VersionManager
impl<'a> VersionManager<'a> {
    /// Generate changelogs for packages.
    ///
    /// Creates or updates changelog files for packages with changes.
    /// Changes are organized by version and type, with customizable formatting.
    ///
    /// # Arguments
    ///
    /// * `options` - Configuration options for changelog generation
    /// * `dry_run` - If true, generate changelogs without writing to disk
    ///
    /// # Returns
    ///
    /// A map of package names to changelog content.
    ///
    /// # Errors
    ///
    /// Returns an error if no change tracker was provided or changelog generation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{ChangelogOptions, VersionManager};
    ///
    /// # fn example(manager: &VersionManager) -> Result<(), Box<dyn std::error::Error>> {
    /// // Create custom options
    /// let options = ChangelogOptions {
    ///     update_existing: true,
    ///     filename: "CHANGELOG.md".to_string(),
    ///     include_version_details: true,
    ///     include_release_date: true,
    ///     header_template: "# Changelog\n\n".to_string(),
    ///     change_template: "- {type}: {description} {breaking}\n".to_string(),
    /// };
    ///
    /// // Generate changelogs
    /// let changelogs = manager.generate_changelogs(&options, false)?;
    /// println!("Generated {} changelogs", changelogs.len());
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::manual_let_else)]
    pub fn generate_changelogs(
        &self,
        options: &ChangelogOptions,
        dry_run: bool,
    ) -> VersioningResult<HashMap<String, String>> {
        // Ensure we have a change tracker
        let change_tracker = self.get_change_tracker()?;

        let mut changelogs = HashMap::new();

        // Get all changes by package
        let changes_by_package = change_tracker.store().get_all_changes_by_package()?;

        for (package_name, changes) in changes_by_package {
            // Skip packages with no changes
            if changes.is_empty() {
                continue;
            }

            // Get the package info
            let package_info = match self.get_workspace().get_package(&package_name) {
                Some(info) => info,
                None => {
                    warn!("Package {} not found in workspace", package_name);
                    continue;
                }
            };

            // Group changes by version
            let changes_by_version =
                change_tracker.store().get_changes_by_version(&package_name)?;

            // Generate changelog content
            let changelog_content = generate_changelog_content(&changes_by_version, options);

            // Store generated content
            changelogs.insert(package_name.clone(), changelog_content.clone());

            // Write to disk if not dry run
            if !dry_run {
                // Determine the changelog path
                let package_info_borrow = package_info.borrow();
                let package_path = Path::new(&package_info_borrow.package_path);
                let changelog_path = package_path.join(&options.filename);

                // Write the changelog file
                write_changelog_file(&changelog_path, &changelog_content, options.update_existing)?;
            }
        }

        Ok(changelogs)
    }
}

/// Generate changelog content from changes.
///
/// Creates formatted changelog content from a collection of changes,
/// organized by version and change type.
///
/// # Arguments
///
/// * `changes_by_version` - Map of version strings to changes
/// * `options` - Changelog formatting options
///
/// # Returns
///
/// Formatted changelog content as a string.
fn generate_changelog_content(
    changes_by_version: &HashMap<String, Vec<Change>>,
    options: &ChangelogOptions,
) -> String {
    let mut content = String::new();

    // Add header
    content.push_str(&options.header_template);

    // Process each version
    let mut versions: Vec<(&String, &Vec<Change>)> = changes_by_version.iter().collect();

    // Put "unreleased" first, then sort the rest by semver (newest first)
    versions.sort_by(|(v1, _), (v2, _)| {
        if *v1 == "unreleased" {
            return std::cmp::Ordering::Less;
        }
        if *v2 == "unreleased" {
            return std::cmp::Ordering::Greater;
        }

        // Parse versions and compare in reverse order (newest first)
        if let (Ok(ver1), Ok(ver2)) = (semver::Version::parse(v1), semver::Version::parse(v2)) {
            return ver2.cmp(&ver1);
        }

        // Fallback to string comparison
        v2.cmp(v1)
    });

    // Generate content for each version
    for (version, changes) in versions {
        // Skip empty change lists
        if changes.is_empty() {
            continue;
        }

        let formatted_version = if version == "unreleased" {
            "## Unreleased".to_string()
        } else if options.include_version_details {
            format!("## Version {version}")
        } else {
            format!("## {version}")
        };

        content.push_str(&formatted_version);
        content.push('\n');

        // Add date if configured
        if options.include_release_date && version != "unreleased" {
            let date = Utc::now().format("%Y-%m-%d").to_string();
            content.push_str(&format!("*Released: {date}*\n"));
        }

        content.push('\n');

        // Group changes by type
        let mut changes_by_type: HashMap<String, Vec<&Change>> = HashMap::new();

        for change in changes {
            let type_key = format!("{}", change.change_type);
            changes_by_type.entry(type_key).or_default().push(change);
        }

        // Add changes by type
        for (change_type, type_changes) in changes_by_type {
            content.push_str(&format!("### {}\n\n", capitalize(&change_type)));

            for change in type_changes {
                let breaking_indicator = if change.breaking { "⚠️ " } else { "" };

                // Format the change using the template
                let mut line = options.change_template.clone();
                line = line.replace("{type}", &change_type);
                line = line.replace("{description}", &change.description);
                line = line.replace("{breaking}", breaking_indicator);

                content.push_str(&line);
            }

            content.push('\n');
        }
    }

    content
}

/// Capitalize the first letter of a string.
///
/// # Arguments
///
/// * `s` - String to capitalize
///
/// # Returns
///
/// String with first letter capitalized.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Write changelog content to a file, updating existing if needed.
///
/// # Arguments
///
/// * `path` - Path to write to
/// * `content` - Content to write
/// * `update_existing` - Whether to update existing file
///
/// # Returns
///
/// `Ok(())` if successful, or an error if writing fails.
///
/// # Errors
///
/// Returns an error if reading or writing the file fails.
fn write_changelog_file(path: &Path, content: &str, update_existing: bool) -> VersioningResult<()> {
    if update_existing && path.exists() {
        // Read existing content
        let mut file = File::open(path)?;
        let mut existing_content = String::new();
        file.read_to_string(&mut existing_content)?;

        // Merge with new content, preserving header
        let merged_content = merge_changelog_content(&existing_content, content);

        // Write back to file
        let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
        file.write_all(merged_content.as_bytes())?;
    } else {
        // Create new file
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
    }

    Ok(())
}

/// Merge existing and new changelog content.
///
/// Preserves the existing header while adding new content.
///
/// # Arguments
///
/// * `existing` - Existing changelog content
/// * `new` - New changelog content
///
/// # Returns
///
/// Merged changelog content.
fn merge_changelog_content(existing: &str, new: &str) -> String {
    // Simple approach: Find the first version header in the existing content
    if let Some(pos) = existing.find("## ") {
        let header = &existing[0..pos];

        // If new content has its own header, skip it
        if let Some(new_header_end) = new.find("## ") {
            format!("{}{}", header, &new[new_header_end..])
        } else {
            format!("{header}{new}")
        }
    } else {
        // No proper structure in existing file, just use new content
        new.to_string()
    }
}
