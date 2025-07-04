//! Changeset manager implementation
//!
//! This module provides the main `ChangesetManager` for creating, managing, and deploying
//! changesets in the monorepo. It integrates with the storage system, Git repository,
//! and task execution to provide a complete changeset workflow.

use chrono::Utc;
use uuid::Uuid;

use super::types::{
    Changeset, ChangesetApplication, ChangesetFilter, ChangesetManager, ChangesetSpec,
    ChangesetStatus, ChangesetStorage, ValidationResult,
};
#[allow(unused_imports)] // Used in documentation examples and validation logic
use crate::config::types::Environment;
use crate::error::Error;
use crate::VersionBumpType;
use sublime_standard_tools::filesystem::FileSystem;

impl<'a> ChangesetManager<'a> {
    /// Creates a new changeset manager with direct borrowing from project
    ///
    /// Uses borrowing instead of trait objects to eliminate Arc proliferation
    /// and work with Rust ownership principles. Focused on CRUD operations for CLI consumption.
    ///
    /// # Arguments
    ///
    /// * `storage` - Changeset storage for persistence
    /// * `config` - Direct reference to configuration
    /// * `file_system` - Direct reference to file system manager
    /// * `packages` - Direct reference to packages
    /// * `repository` - Direct reference to git repository
    /// * `root_path` - Direct reference to root path
    ///
    /// # Returns
    ///
    /// A new changeset manager instance
    pub fn new(
        storage: ChangesetStorage<'a>,
        config: &'a crate::config::MonorepoConfig,
        file_system: &'a sublime_standard_tools::filesystem::FileSystemManager,
        packages: &'a [crate::core::MonorepoPackageInfo],
        repository: &'a sublime_git_tools::Repo,
        root_path: &'a std::path::Path,
    ) -> Self {
        Self { storage, config, file_system, packages, repository, root_path }
    }

    /// Creates a new changeset manager from project (convenience method)
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to the monorepo project
    ///
    /// # Returns
    ///
    /// A new changeset manager instance.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the required components cannot be initialized.
    pub fn from_project(
        project: &'a crate::core::MonorepoProject,
    ) -> Result<Self, crate::error::Error> {
        let storage = crate::changesets::ChangesetStorage::new(
            project.config.changesets.clone(),
            &project.file_system,
            &project.root_path,
        );

        Ok(Self::new(
            storage,
            &project.config,
            &project.file_system,
            &project.packages,
            &project.repository,
            &project.root_path,
        ))
    }

    /// Creates a new changeset with the specified parameters
    ///
    /// Generates a unique ID, sets creation timestamp, and saves the changeset
    /// to storage. The author is determined from the changeset spec or Git config.
    ///
    /// # Arguments
    ///
    /// * `spec` - Specification for the new changeset
    ///
    /// # Returns
    ///
    /// The created changeset.
    ///
    /// # Errors
    ///
    /// Returns an error if the changeset cannot be created or saved.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::{ChangesetSpec, VersionBumpType, Environment};
    ///
    /// # async fn example(manager: &ChangesetManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let spec = ChangesetSpec {
    ///     package: "@test/core".to_string(),
    ///     version_bump: VersionBumpType::Minor,
    ///     description: "Add new API endpoint".to_string(),
    ///     development_environments: vec![Environment::Development, Environment::Staging],
    ///     production_deployment: false,
    ///     author: None, // Will be inferred from Git config
    /// };
    ///
    /// let changeset = manager.create_changeset(spec)?;
    /// println!("Created changeset: {}", changeset.id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_changeset(&self, spec: ChangesetSpec) -> Result<Changeset, Error> {
        // Generate unique ID
        let id = Uuid::new_v4().to_string();

        // Get current branch
        let branch = self
            .repository
            .get_current_branch()
            .map_err(|e| Error::changeset(format!("Failed to get current branch: {e}")))?;

        // Determine author
        let author = match spec.author {
            Some(author) => author,
            None => self.get_author_from_git_config(),
        };

        // Create changeset
        let changeset = Changeset {
            id,
            package: spec.package,
            version_bump: spec.version_bump,
            description: spec.description,
            branch,
            development_environments: spec.development_environments,
            production_deployment: spec.production_deployment,
            created_at: Utc::now(),
            author,
            status: ChangesetStatus::Pending,
        };

        // Validate changeset
        let validation = self.validate_changeset(&changeset)?;
        if !validation.is_valid {
            return Err(Error::changeset(format!(
                "Changeset validation failed: {}",
                validation.errors.join(", ")
            )));
        }

        // Save to storage
        self.storage.save(&changeset)?;

        Ok(changeset)
    }

    /// Creates a changeset interactively with user prompts
    ///
    /// This method would normally prompt the user for changeset details.
    /// For now, it creates a basic changeset structure that can be extended.
    ///
    /// # Arguments
    ///
    /// * `package` - Optional package name (if None, will prompt or auto-detect)
    ///
    /// # Returns
    ///
    /// The created changeset.
    ///
    /// # Errors
    ///
    /// Returns an error if the interactive process fails or the changeset cannot be created.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # async fn example(manager: &ChangesetManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let changeset = manager.create_changeset_interactive(Some("@test/core".to_string()))?;
    /// println!("Created interactive changeset: {}", changeset.id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_changeset_interactive(
        &self,
        package: Option<String>,
    ) -> Result<Changeset, Error> {
        // For now, create a basic changeset
        // In a real implementation, this would use prompts

        let package_name = match package {
            Some(name) => name,
            None => {
                // Auto-detect from current changes
                self.detect_affected_package()?
            }
        };

        let spec = ChangesetSpec {
            package: package_name,
            version_bump: VersionBumpType::Patch, // Default to patch
            description: "Interactive changeset".to_string(),
            development_environments: self.config.changesets.default_environments.clone(),
            production_deployment: false,
            author: None,
        };

        self.create_changeset(spec)
    }

    /// Applies changesets on merge for the specified branch
    ///
    /// Finds all changesets for the branch and applies them, updating
    /// package versions and triggering any necessary tasks.
    ///
    /// # Arguments
    ///
    /// * `branch` - The branch being merged
    ///
    /// # Returns
    ///
    /// Vector of changeset applications with results.
    ///
    /// # Errors
    ///
    /// Returns an error if changesets cannot be applied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # async fn example(manager: &ChangesetManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let applications = manager.apply_changesets_on_merge("feature/new-api")?;
    /// for app in applications {
    ///     println!("Applied changeset {} to {}: {} -> {}",
    ///         app.changeset_id, app.package, app.old_version, app.new_version);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn apply_changesets_on_merge(
        &self,
        branch: &str,
    ) -> Result<Vec<ChangesetApplication>, Error> {
        // Find changesets for this branch
        let filter = ChangesetFilter {
            branch: Some(branch.to_string()),
            status: Some(ChangesetStatus::Pending),
            ..Default::default()
        };

        let changesets = self.storage.list(&filter)?;
        let mut applications = Vec::new();

        for mut changeset in changesets {
            // Apply the changeset
            let application = self.apply_changeset(&mut changeset)?;
            applications.push(application);
        }

        Ok(applications)
    }

    /// Lists changesets matching the given filter
    ///
    /// # Arguments
    ///
    /// * `filter` - Filter criteria for changesets
    ///
    /// # Returns
    ///
    /// Vector of changesets matching the filter.
    ///
    /// # Errors
    ///
    /// Returns an error if changesets cannot be retrieved.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::ChangesetFilter;
    ///
    /// # async fn example(manager: &ChangesetManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let filter = ChangesetFilter {
    ///     package: Some("@test/core".to_string()),
    ///     ..Default::default()
    /// };
    /// let changesets = manager.list_changesets(filter)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_changesets(&self, filter: &ChangesetFilter) -> Result<Vec<Changeset>, Error> {
        self.storage.list(filter)
    }

    /// Validates a changeset before applying
    ///
    /// Performs various validation checks including package existence,
    /// version bump validity, and environment configuration.
    ///
    /// # Arguments
    ///
    /// * `changeset` - The changeset to validate
    ///
    /// # Returns
    ///
    /// Validation result with any errors or warnings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn example(manager: &ChangesetManager, changeset: &Changeset) -> Result<(), Box<dyn std::error::Error>> {
    /// let validation = manager.validate_changeset(changeset)?;
    /// if !validation.is_valid {
    ///     for error in &validation.errors {
    ///         eprintln!("Validation error: {}", error);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate_changeset(&self, changeset: &Changeset) -> Result<ValidationResult, Error> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut metadata = std::collections::HashMap::new();

        // Validate package exists
        if changeset.package.is_empty() {
            errors.push("Package name cannot be empty".to_string());
        } else {
            // Check if package actually exists in the project
            if self.packages.iter().any(|p| p.name() == changeset.package) {
                // Validate version bump is appropriate
                match self.validate_version_bump(&changeset.package, changeset.version_bump) {
                    Ok(current_version) => {
                        metadata.insert("current_version".to_string(), current_version);
                    }
                    Err(e) => {
                        errors.push(format!("Version bump validation failed: {e}"));
                    }
                }
            } else {
                errors.push(format!(
                    "Package '{package}' not found in project",
                    package = changeset.package
                ));
            }
        }

        // Validate description
        if changeset.description.is_empty() {
            errors.push("Description cannot be empty".to_string());
        } else if changeset.description.len() < 10 {
            warnings.push("Description is very short - consider providing more detail".to_string());
        }

        // Validate environments
        if changeset.development_environments.is_empty() && !changeset.production_deployment {
            warnings.push("No deployment environments specified".to_string());
        }

        for env in &changeset.development_environments {
            if !self.config.environments.contains(env) {
                warnings
                    .push(format!("Environment {env} is not configured in project environments"));
            }
        }

        // Validate branch
        if changeset.branch.is_empty() {
            errors.push("Branch name cannot be empty".to_string());
        } else {
            // Check if branch follows naming conventions using configured prefixes
            let branch_config = &self.config.git.branches;
            let valid_prefixes = branch_config.get_all_valid_prefixes();
            let has_valid_prefix =
                valid_prefixes.iter().any(|prefix| changeset.branch.starts_with(prefix));

            if !has_valid_prefix && !branch_config.is_protected_branch(&changeset.branch) {
                warnings.push(format!(
                    "Branch '{}' doesn't follow conventional naming ({})",
                    changeset.branch,
                    valid_prefixes.join(", ")
                ));
            }
        }

        // Validate author
        if changeset.author.is_empty() {
            errors.push("Author cannot be empty".to_string());
        } else if !changeset.author.contains('@') {
            warnings.push("Author should be an email address".to_string());
        }

        // Check for conflicting changesets
        let filter = ChangesetFilter {
            package: Some(changeset.package.clone()),
            status: Some(ChangesetStatus::Pending),
            ..Default::default()
        };

        match self.storage.list(&filter) {
            Ok(existing_changesets) => {
                let conflicts: Vec<_> = existing_changesets
                    .iter()
                    .filter(|cs| cs.id != changeset.id && cs.branch != changeset.branch)
                    .collect();

                if !conflicts.is_empty() {
                    warnings.push(format!(
                        "Found {} pending changeset(s) for the same package from other branches",
                        conflicts.len()
                    ));

                    // Add conflict information to metadata
                    let conflict_branches: Vec<String> =
                        conflicts.iter().map(|cs| cs.branch.clone()).collect();
                    metadata
                        .insert("conflicting_branches".to_string(), conflict_branches.join(", "));
                }
            }
            Err(e) => {
                warnings.push(format!("Could not check for conflicting changesets: {e}"));
            }
        }

        // Validate dependent packages if this is a breaking change
        if changeset.version_bump == VersionBumpType::Major {
            match self.check_dependent_packages(&changeset.package) {
                Ok(dependents) => {
                    if !dependents.is_empty() {
                        warnings.push(format!(
                            "Major version bump will affect {} dependent package(s): {}",
                            dependents.len(),
                            dependents.join(", ")
                        ));
                        metadata.insert("affected_dependents".to_string(), dependents.join(", "));
                    }
                }
                Err(e) => {
                    warnings.push(format!("Could not check dependent packages: {e}"));
                }
            }
        }

        Ok(ValidationResult { is_valid: errors.is_empty(), errors, warnings, metadata })
    }


    /// Validates that a version bump is appropriate for a package
    ///
    /// Checks the current version of the package and ensures the bump type is valid.
    fn validate_version_bump(
        &self,
        package: &str,
        version_bump: VersionBumpType,
    ) -> Result<String, Error> {
        // Read current version from package.json
        let current_version = self.read_package_version(package)?;

        // Parse the current version
        let version_parts: Vec<&str> = current_version.split('.').collect();
        if version_parts.len() != 3 {
            return Err(Error::changeset(format!(
                "Invalid version format in package {package}: {current_version}"
            )));
        }

        // For now, just validate format - in a real implementation would check semver constraints
        let major: u32 = version_parts[0].parse().map_err(|_| {
            Error::changeset(format!(
                "Invalid major version in {package}: {version}",
                version = version_parts[0]
            ))
        })?;
        let minor: u32 = version_parts[1].parse().map_err(|_| {
            Error::changeset(format!(
                "Invalid minor version in {package}: {version}",
                version = version_parts[1]
            ))
        })?;
        let patch: u32 = version_parts[2].parse().map_err(|_| {
            Error::changeset(format!(
                "Invalid patch version in {package}: {version}",
                version = version_parts[2]
            ))
        })?;

        // Validate bump type makes sense
        match version_bump {
            VersionBumpType::Major | VersionBumpType::Patch => {
                // Major and patch bumps are always valid
            }
            VersionBumpType::Minor => {
                // Minor bumps are valid unless we're at major version 0
                if major == 0 {
                    return Err(Error::changeset(
                        "Minor version bumps should be Major bumps for 0.x.x versions".to_string(),
                    ));
                }
            }
            VersionBumpType::Snapshot => {
                return Err(Error::changeset(
                    "Snapshot versions not supported in changesets".to_string(),
                ));
            }
        }

        Ok(format!("{major}.{minor}.{patch}"))
    }

    /// Reads the current version from a package's package.json
    fn read_package_version(&self, package: &str) -> Result<String, Error> {
        // Get package information
        let package_info = self
            .packages
            .iter()
            .find(|p| p.name() == package)
            .ok_or_else(|| Error::changeset(format!("Package not found: {package}")))?;

        // Read package.json
        let package_json_path = package_info.path().join("package.json");
        let content = self.file_system.read_file_string(&package_json_path).map_err(|e| {
            Error::changeset(format!("Failed to read package.json for {package}: {e}"))
        })?;

        // Parse JSON to extract version
        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| Error::changeset(format!("Invalid package.json in {package}: {e}")))?;

        let version = json["version"].as_str().ok_or_else(|| {
            Error::changeset(format!("No version field in package.json for {package}"))
        })?;

        Ok(version.to_string())
    }

    /// Checks which packages depend on the given package
    #[allow(clippy::unnecessary_wraps)]
    fn check_dependent_packages(&self, package: &str) -> Result<Vec<String>, Error> {
        let mut dependents = Vec::new();

        // Check all packages in the project
        for pkg in self.packages {
            if pkg.name() == package {
                continue; // Skip self
            }

            // Read package.json to check dependencies
            let package_json_path = pkg.path().join("package.json");
            if let Ok(content) = self.file_system.read_file_string(&package_json_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    // Check dependencies, devDependencies, and peerDependencies
                    let dep_sections = ["dependencies", "devDependencies", "peerDependencies"];

                    for section in &dep_sections {
                        if let Some(deps) = json[section].as_object() {
                            if deps.contains_key(package) {
                                dependents.push(pkg.name().to_string());
                                break; // No need to check other sections for this package
                            }
                        }
                    }
                }
            }
        }

        Ok(dependents)
    }

    /// Gets the Git author from configuration
    fn get_author_from_git_config(&self) -> String {
        // Get git configuration from repository
        match self.repository.list_config() {
            Ok(config) => {
                let email = config
                    .get("user.email")
                    .cloned()
                    .unwrap_or_else(|| "user@example.com".to_string());

                let name = config.get("user.name").cloned().unwrap_or_else(|| "User".to_string());

                if email.contains('@') && !name.is_empty() {
                    format!("{name} <{email}>")
                } else {
                    email
                }
            }
            Err(_) => {
                // Fallback to default if config access fails
                "user@example.com".to_string()
            }
        }
    }

    /// Detects the affected package from current Git changes
    fn detect_affected_package(&self) -> Result<String, Error> {
        // Get staged files to determine affected packages
        let staged_files = self
            .repository
            .get_staged_files()
            .map_err(|e| Error::changeset(format!("Failed to get staged files: {e}")))?;

        if staged_files.is_empty() {
            return Err(Error::changeset(
                "No staged files found - stage some changes first".to_string(),
            ));
        }

        // Find which package each file belongs to
        let mut package_file_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for file_path in &staged_files {
            let full_path = self.root_path.join(file_path);

            // Find package that contains this file path
            for package in self.packages {
                let package_path = package.workspace_package.absolute_path.as_path();
                if full_path.starts_with(package_path) {
                    let package_name = package.name().to_string();
                    *package_file_counts.entry(package_name).or_insert(0) += 1;
                    break; // Found the package, no need to continue
                }
            }
        }

        if package_file_counts.is_empty() {
            return Err(Error::changeset("No packages affected by staged changes".to_string()));
        }

        // Return the package with the most changes
        let most_affected_package = package_file_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(package, _)| package)
            .ok_or_else(|| {
                Error::changeset("Failed to determine most affected package".to_string())
            })?;

        Ok(most_affected_package)
    }

    /// Applies a single changeset
    fn apply_changeset(&self, changeset: &mut Changeset) -> Result<ChangesetApplication, Error> {
        // Read current version
        let old_version = self.read_package_version(&changeset.package)?;

        // Calculate new version based on version bump
        let new_version = self.calculate_new_version(&old_version, &changeset.version_bump)?;

        // Update package.json version
        self.update_package_version(&changeset.package, &new_version)?;

        // Update dependencies in dependent packages if this is a breaking change
        if changeset.version_bump == VersionBumpType::Major {
            let dependents = self.check_dependent_packages(&changeset.package)?;
            for dependent in &dependents {
                self.update_dependency_version(dependent, &changeset.package, &new_version)?;
            }
        }

        // Update changeset status
        changeset.status =
            ChangesetStatus::Merged { merged_at: Utc::now(), final_version: new_version.clone() };

        // Save updated changeset
        self.storage.save(changeset)?;

        Ok(ChangesetApplication {
            changeset_id: changeset.id.clone(),
            package: changeset.package.clone(),
            old_version,
            new_version,
            environments_deployed: changeset.development_environments.clone(),
            success: true,
        })
    }

    /// Calculates the new version based on current version and bump type
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[allow(clippy::unused_self)]
    fn calculate_new_version(
        &self,
        current_version: &str,
        version_bump: &VersionBumpType,
    ) -> Result<String, Error> {
        let version_parts: Vec<&str> = current_version.split('.').collect();
        if version_parts.len() != 3 {
            return Err(Error::changeset(format!("Invalid version format: {current_version}")));
        }

        let major: u32 = version_parts[0].parse().map_err(|_| {
            Error::changeset(format!(
                "Invalid major version: {version}",
                version = version_parts[0]
            ))
        })?;
        let minor: u32 = version_parts[1].parse().map_err(|_| {
            Error::changeset(format!(
                "Invalid minor version: {version}",
                version = version_parts[1]
            ))
        })?;
        let patch: u32 = version_parts[2].parse().map_err(|_| {
            Error::changeset(format!(
                "Invalid patch version: {version}",
                version = version_parts[2]
            ))
        })?;

        let (new_major, new_minor, new_patch) = match version_bump {
            VersionBumpType::Major => (major + 1, 0, 0),
            VersionBumpType::Minor => (major, minor + 1, 0),
            VersionBumpType::Patch => (major, minor, patch + 1),
            VersionBumpType::Snapshot => {
                return Err(Error::changeset(
                    "Snapshot versions not supported in changesets".to_string(),
                ));
            }
        };

        Ok(format!("{new_major}.{new_minor}.{new_patch}"))
    }

    /// Updates the version in a package's package.json
    fn update_package_version(&self, package: &str, new_version: &str) -> Result<(), Error> {
        // Get package information
        let package_info = self
            .packages
            .iter()
            .find(|p| p.name() == package)
            .ok_or_else(|| Error::changeset(format!("Package not found: {package}")))?;

        // Read package.json
        let package_json_path = package_info.path().join("package.json");
        let content = self.file_system.read_file_string(&package_json_path).map_err(|e| {
            Error::changeset(format!("Failed to read package.json for {package}: {e}"))
        })?;

        // Parse JSON
        let mut json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| Error::changeset(format!("Invalid package.json in {package}: {e}")))?;

        // Update version
        json["version"] = serde_json::Value::String(new_version.to_string());

        // Write back to file with proper formatting
        let updated_content = serde_json::to_string_pretty(&json).map_err(|e| {
            Error::changeset(format!("Failed to serialize updated package.json: {e}"))
        })?;

        self.file_system.write_file_string(&package_json_path, &updated_content).map_err(|e| {
            Error::changeset(format!("Failed to write updated package.json for {package}: {e}"))
        })?;

        Ok(())
    }

    /// Updates dependency version in dependent packages
    fn update_dependency_version(
        &self,
        dependent_package: &str,
        dependency: &str,
        new_version: &str,
    ) -> Result<(), Error> {
        // Get package information
        let package_info =
            self.packages.iter().find(|p| p.name() == dependent_package).ok_or_else(|| {
                Error::changeset(format!("Dependent package not found: {dependent_package}"))
            })?;

        // Read package.json
        let package_json_path = package_info.path().join("package.json");
        let content = self.file_system.read_file_string(&package_json_path).map_err(|e| {
            Error::changeset(format!("Failed to read package.json for {dependent_package}: {e}"))
        })?;

        // Parse JSON
        let mut json: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            Error::changeset(format!("Invalid package.json in {dependent_package}: {e}"))
        })?;

        let mut updated = false;

        // Update dependency in all sections
        let dep_sections = ["dependencies", "devDependencies", "peerDependencies"];
        for section in &dep_sections {
            if let Some(deps) = json[section].as_object_mut() {
                if deps.contains_key(dependency) {
                    deps[dependency] = serde_json::Value::String(format!("^{new_version}"));
                    updated = true;
                }
            }
        }

        if updated {
            // Write back to file
            let updated_content = serde_json::to_string_pretty(&json).map_err(|e| {
                Error::changeset(format!("Failed to serialize updated package.json: {e}"))
            })?;

            self.file_system.write_file_string(&package_json_path, &updated_content).map_err(
                |e| {
                    Error::changeset(format!(
                        "Failed to write updated package.json for {dependent_package}: {e}"
                    ))
                },
            )?;
        }

        Ok(())
    }

}
