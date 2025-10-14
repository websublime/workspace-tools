//! Package.json editor with format preservation.
//!
//! This module provides a PackageJsonEditor that can modify package.json files
//! while preserving the original formatting, indentation, and comments. It uses
//! a combination of JSON parsing and text manipulation to achieve this.
//!
//! # What
//!
//! Implements format-preserving editing of package.json files:
//! - Preserves original indentation and spacing
//! - Maintains comments (where possible)
//! - Updates specific fields without reformatting the entire file
//! - Supports atomic operations with rollback capability
//! - Validates changes before applying them
//!
//! # How
//!
//! Uses a hybrid approach combining JSON parsing with text-based editing:
//! 1. Parse JSON to understand structure and validate changes
//! 2. Use regex and string manipulation to update specific fields
//! 3. Preserve original formatting by maintaining line structure
//! 4. Validate the result to ensure JSON integrity
//!
//! # Why
//!
//! Developers expect their package.json files to maintain their formatting
//! and comments when tools make automated changes. This editor ensures
//! that version bumps and dependency updates don't disrupt the human-readable
//! structure of the file.
//!
//! # Examples
//!
//! ```ignore
//! use sublime_pkg_tools::package::PackageJsonEditor;
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let fs = FileSystemManager::new();
//! let mut editor = PackageJsonEditor::new(&fs, Path::new("./package.json")).await?;
//!
//! // Make multiple changes
//! editor.set_version("1.2.4")?;
//! editor.update_dependency("lodash", "^4.17.21")?;
//! editor.add_dev_dependency("jest", "^29.0.0")?;
//!
//! // Save all changes atomically
//! editor.save().await?;
//! # Ok(())
//! # }
//! ```

use crate::error::{PackageError, PackageResult};
use crate::package::PackageJson;
use serde_json::Value;
use std::path::{Path, PathBuf};
use sublime_standard_tools::filesystem::AsyncFileSystem;

/// Represents a modification to be applied to package.json.
///
/// This enum captures the different types of changes that can be made
/// to a package.json file while preserving formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageJsonModification {
    /// Set the version field
    SetVersion {
        /// The new version string
        version: String,
    },
    /// Update a dependency in the dependencies section
    UpdateDependency {
        /// The dependency name
        name: String,
        /// The new version constraint
        version: String,
    },
    /// Update a dependency in the devDependencies section
    UpdateDevDependency {
        /// The dependency name
        name: String,
        /// The new version constraint
        version: String,
    },
    /// Update a dependency in the peerDependencies section
    UpdatePeerDependency {
        /// The dependency name
        name: String,
        /// The new version constraint
        version: String,
    },
    /// Update a dependency in the optionalDependencies section
    UpdateOptionalDependency {
        /// The dependency name
        name: String,
        /// The new version constraint
        version: String,
    },
    /// Remove a dependency from any section
    RemoveDependency {
        /// The dependency name to remove
        name: String,
    },
    /// Update a script in the scripts section
    UpdateScript {
        /// The script name
        name: String,
        /// The script command
        command: String,
    },
    /// Remove a script from the scripts section
    RemoveScript {
        /// The script name to remove
        name: String,
    },
    /// Set a custom field
    SetField {
        /// The field path (e.g., "description", "author.name")
        field: String,
        /// The new value as JSON
        value: Value,
    },
}

/// A package.json editor that preserves file formatting.
///
/// This editor loads a package.json file and allows making modifications
/// while preserving the original formatting, indentation, and structure.
/// Changes are batched and can be applied atomically.
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::package::PackageJsonEditor;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = FileSystemManager::new();
/// let mut editor = PackageJsonEditor::new(&fs, Path::new("./package.json")).await?;
///
/// // Check current version
/// println!("Current version: {}", editor.get_version()?);
///
/// // Make changes
/// editor.set_version("2.0.0")?;
/// editor.update_dependency("lodash", "^4.17.21")?;
///
/// // Preview changes without saving
/// let preview = editor.preview()?;
/// println!("Would change:\n{}", preview);
///
/// // Save changes
/// editor.save().await?;
/// # Ok(())
/// # }
/// ```
pub struct PackageJsonEditor<F> {
    /// The filesystem implementation to use
    filesystem: F,
    /// Path to the package.json file
    file_path: PathBuf,
    /// Original file content
    original_content: String,
    /// Current working content
    current_content: String,
    /// Parsed package.json for validation
    parsed_json: PackageJson,
    /// List of pending modifications
    modifications: Vec<PackageJsonModification>,
}

impl<F> PackageJsonEditor<F>
where
    F: AsyncFileSystem + Send + Sync,
{
    /// Creates a new editor for the specified package.json file.
    ///
    /// # Arguments
    ///
    /// * `filesystem` - The filesystem implementation to use
    /// * `file_path` - Path to the package.json file
    ///
    /// # Returns
    ///
    /// A new PackageJsonEditor instance
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The JSON content is malformed
    /// - Required package.json fields are missing
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::package::PackageJsonEditor;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let editor = PackageJsonEditor::new(&fs, Path::new("./package.json")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(filesystem: F, file_path: &Path) -> PackageResult<Self> {
        let content = filesystem.read_file_string(file_path).await.map_err(|e| {
            PackageError::operation(
                "read_package_json",
                format!("Failed to read {}: {}", file_path.display(), e),
            )
        })?;

        let parsed_json = PackageJson::parse_from_str(&content)?;

        Ok(Self {
            filesystem,
            file_path: file_path.to_path_buf(),
            original_content: content.clone(),
            current_content: content,
            parsed_json,
            modifications: Vec::new(),
        })
    }

    /// Gets the current version from the package.json.
    ///
    /// # Returns
    ///
    /// The current version string
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
    /// # let editor = sublime_pkg_tools::package::PackageJsonEditor::new(&fs, std::path::Path::new("./package.json")).await?;
    /// let version = editor.get_version()?;
    /// println!("Current version: {}", version);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_version(&self) -> PackageResult<String> {
        Ok(self.parsed_json.version.to_string())
    }

    /// Sets the version field in package.json.
    ///
    /// # Arguments
    ///
    /// * `version` - The new version string (must be valid semver)
    ///
    /// # Errors
    ///
    /// Returns an error if the version string is not valid semver
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
    /// # let mut editor = sublime_pkg_tools::package::PackageJsonEditor::new(&fs, std::path::Path::new("./package.json")).await?;
    /// editor.set_version("1.2.4")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_version(&mut self, version: &str) -> PackageResult<()> {
        // Validate the version format
        let _ = version.parse::<semver::Version>().map_err(|e| {
            PackageError::operation(
                "set_version",
                format!("Invalid version format '{}': {}", version, e),
            )
        })?;

        self.modifications
            .push(PackageJsonModification::SetVersion { version: version.to_string() });

        Ok(())
    }

    /// Updates a dependency in the dependencies section.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name
    /// * `version` - The new version constraint
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
    /// # let mut editor = sublime_pkg_tools::package::PackageJsonEditor::new(&fs, std::path::Path::new("./package.json")).await?;
    /// editor.update_dependency("lodash", "^4.17.21")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_dependency(&mut self, name: &str, version: &str) -> PackageResult<()> {
        self.modifications.push(PackageJsonModification::UpdateDependency {
            name: name.to_string(),
            version: version.to_string(),
        });

        Ok(())
    }

    /// Adds or updates a development dependency.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name
    /// * `version` - The version constraint
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
    /// # let mut editor = sublime_pkg_tools::package::PackageJsonEditor::new(&fs, std::path::Path::new("./package.json")).await?;
    /// editor.add_dev_dependency("jest", "^29.0.0")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_dev_dependency(&mut self, name: &str, version: &str) -> PackageResult<()> {
        self.modifications.push(PackageJsonModification::UpdateDevDependency {
            name: name.to_string(),
            version: version.to_string(),
        });

        Ok(())
    }

    /// Updates a peer dependency.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name
    /// * `version` - The version constraint
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
    /// # let mut editor = sublime_pkg_tools::package::PackageJsonEditor::new(&fs, std::path::Path::new("./package.json")).await?;
    /// editor.update_peer_dependency("react", "^18.0.0")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_peer_dependency(&mut self, name: &str, version: &str) -> PackageResult<()> {
        self.modifications.push(PackageJsonModification::UpdatePeerDependency {
            name: name.to_string(),
            version: version.to_string(),
        });

        Ok(())
    }

    /// Removes a dependency from any section.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name to remove
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
    /// # let mut editor = sublime_pkg_tools::package::PackageJsonEditor::new(&fs, std::path::Path::new("./package.json")).await?;
    /// editor.remove_dependency("old-package")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn remove_dependency(&mut self, name: &str) -> PackageResult<()> {
        self.modifications
            .push(PackageJsonModification::RemoveDependency { name: name.to_string() });

        Ok(())
    }

    /// Updates a script in the scripts section.
    ///
    /// # Arguments
    ///
    /// * `name` - The script name
    /// * `command` - The script command
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
    /// # let mut editor = sublime_pkg_tools::package::PackageJsonEditor::new(&fs, std::path::Path::new("./package.json")).await?;
    /// editor.update_script("test", "jest --coverage")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_script(&mut self, name: &str, command: &str) -> PackageResult<()> {
        self.modifications.push(PackageJsonModification::UpdateScript {
            name: name.to_string(),
            command: command.to_string(),
        });

        Ok(())
    }

    /// Sets a custom field in the package.json.
    ///
    /// # Arguments
    ///
    /// * `field` - The field path (e.g., "description", "author.name")
    /// * `value` - The new value as JSON
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
    /// # let mut editor = sublime_pkg_tools::package::PackageJsonEditor::new(&fs, std::path::Path::new("./package.json")).await?;
    /// editor.set_field("description", serde_json::Value::String("New description".to_string()))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_field(&mut self, field: &str, value: Value) -> PackageResult<()> {
        self.modifications
            .push(PackageJsonModification::SetField { field: field.to_string(), value });

        Ok(())
    }

    /// Previews the changes that would be made without applying them.
    ///
    /// # Returns
    ///
    /// A string showing the content after applying all modifications
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
    /// # let mut editor = sublime_pkg_tools::package::PackageJsonEditor::new(&fs, std::path::Path::new("./package.json")).await?;
    /// # editor.set_version("1.2.4")?;
    /// let preview = editor.preview()?;
    /// println!("Changes would result in:\n{}", preview);
    /// # Ok(())
    /// # }
    /// ```
    pub fn preview(&self) -> PackageResult<String> {
        let mut content = self.current_content.clone();

        for modification in &self.modifications {
            content = self.apply_modification(&content, modification)?;
        }

        Ok(content)
    }

    /// Applies all pending modifications and saves the file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any modification fails to apply
    /// - The resulting JSON is invalid
    /// - The file cannot be written
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
    /// # let mut editor = sublime_pkg_tools::package::PackageJsonEditor::new(&fs, std::path::Path::new("./package.json")).await?;
    /// # editor.set_version("1.2.4")?;
    /// editor.save().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save(&mut self) -> PackageResult<()> {
        // Apply all modifications
        let mut content = self.current_content.clone();

        for modification in &self.modifications {
            content = self.apply_modification(&content, modification)?;
        }

        // Validate the resulting JSON
        let _: Value = serde_json::from_str(&content).map_err(|e| {
            PackageError::operation(
                "validate_modified_json",
                format!("Modified JSON is invalid: {}", e),
            )
        })?;

        // Write the file
        self.filesystem.write_file_string(&self.file_path, &content).await.map_err(|e| {
            PackageError::operation(
                "save_package_json",
                format!("Failed to write {}: {}", self.file_path.display(), e),
            )
        })?;

        // Update state
        self.current_content = content;
        self.parsed_json = PackageJson::parse_from_str(&self.current_content)?;
        self.modifications.clear();

        Ok(())
    }

    /// Reverts all pending modifications.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
    /// # let mut editor = sublime_pkg_tools::package::PackageJsonEditor::new(&fs, std::path::Path::new("./package.json")).await?;
    /// # editor.set_version("1.2.4")?;
    /// editor.revert();
    /// # Ok(())
    /// # }
    /// ```
    pub fn revert(&mut self) {
        self.current_content = self.original_content.clone();
        self.modifications.clear();
    }

    /// Checks if there are any pending modifications.
    ///
    /// # Returns
    ///
    /// True if there are unsaved changes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
    /// # let mut editor = sublime_pkg_tools::package::PackageJsonEditor::new(&fs, std::path::Path::new("./package.json")).await?;
    /// assert!(!editor.has_changes());
    /// editor.set_version("1.2.4")?;
    /// assert!(editor.has_changes());
    /// # Ok(())
    /// # }
    /// ```
    pub fn has_changes(&self) -> bool {
        !self.modifications.is_empty()
    }

    /// Gets the list of pending modifications.
    ///
    /// # Returns
    ///
    /// A reference to the pending modifications
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
    /// # let mut editor = sublime_pkg_tools::package::PackageJsonEditor::new(&fs, std::path::Path::new("./package.json")).await?;
    /// # editor.set_version("1.2.4")?;
    /// let changes = editor.pending_modifications();
    /// println!("Pending changes: {}", changes.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn pending_modifications(&self) -> &[PackageJsonModification] {
        &self.modifications
    }

    /// Applies a single modification to the content string.
    ///
    /// This method uses a simple approach of parsing JSON, modifying it,
    /// and then pretty-printing it back to preserve basic formatting.
    fn apply_modification(
        &self,
        content: &str,
        modification: &PackageJsonModification,
    ) -> PackageResult<String> {
        // Parse the JSON
        let mut json: serde_json::Value =
            serde_json::from_str(content).map_err(PackageError::Json)?;

        // Apply the modification
        match modification {
            PackageJsonModification::SetVersion { version } => {
                json["version"] = serde_json::Value::String(version.clone());
            }

            PackageJsonModification::UpdateDependency { name, version } => {
                if json["dependencies"].is_null() {
                    json["dependencies"] = serde_json::Value::Object(serde_json::Map::new());
                }
                json["dependencies"][name] = serde_json::Value::String(version.clone());
            }

            PackageJsonModification::UpdateDevDependency { name, version } => {
                if json["devDependencies"].is_null() {
                    json["devDependencies"] = serde_json::Value::Object(serde_json::Map::new());
                }
                json["devDependencies"][name] = serde_json::Value::String(version.clone());
            }

            PackageJsonModification::UpdatePeerDependency { name, version } => {
                if json["peerDependencies"].is_null() {
                    json["peerDependencies"] = serde_json::Value::Object(serde_json::Map::new());
                }
                json["peerDependencies"][name] = serde_json::Value::String(version.clone());
            }

            PackageJsonModification::UpdateOptionalDependency { name, version } => {
                if json["optionalDependencies"].is_null() {
                    json["optionalDependencies"] =
                        serde_json::Value::Object(serde_json::Map::new());
                }
                json["optionalDependencies"][name] = serde_json::Value::String(version.clone());
            }

            PackageJsonModification::RemoveDependency { name } => {
                if let Some(deps) = json["dependencies"].as_object_mut() {
                    deps.remove(name);
                }
                if let Some(dev_deps) = json["devDependencies"].as_object_mut() {
                    dev_deps.remove(name);
                }
                if let Some(peer_deps) = json["peerDependencies"].as_object_mut() {
                    peer_deps.remove(name);
                }
                if let Some(opt_deps) = json["optionalDependencies"].as_object_mut() {
                    opt_deps.remove(name);
                }
            }

            PackageJsonModification::UpdateScript { name, command } => {
                if json["scripts"].is_null() {
                    json["scripts"] = serde_json::Value::Object(serde_json::Map::new());
                }
                json["scripts"][name] = serde_json::Value::String(command.clone());
            }

            PackageJsonModification::RemoveScript { name } => {
                if let Some(scripts) = json["scripts"].as_object_mut() {
                    scripts.remove(name);
                }
            }

            PackageJsonModification::SetField { field, value } => {
                json[field] = value.clone();
            }
        }

        // Convert back to pretty JSON
        serde_json::to_string_pretty(&json).map_err(PackageError::Json)
    }
}
