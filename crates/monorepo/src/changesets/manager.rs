//! Changeset manager implementation
//!
//! This module provides the main `ChangesetManager` for creating, managing, and deploying
//! changesets in the monorepo. It integrates with the storage system, Git repository,
//! and task execution to provide a complete changeset workflow.

use std::sync::Arc;

use chrono::Utc;
use sublime_standard_tools::filesystem::FileSystem;
use uuid::Uuid;

use super::types::{
    Changeset, ChangesetApplication, ChangesetFilter, ChangesetSpec, ChangesetStatus,
    DeploymentResult, EnvironmentDeploymentResult, ValidationResult,
    ChangesetManager, ChangesetStorage,
};
use crate::config::types::Environment;
use crate::core::MonorepoProject;
use crate::error::Error;
use crate::tasks::TaskManager;
use crate::VersionBumpType;


impl ChangesetManager {
    /// Creates a new changeset manager
    ///
    /// This is a synchronous operation as it only initializes local structures
    /// and does not perform any I/O operations.
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to the monorepo project
    ///
    /// # Returns
    ///
    /// A new `ChangesetManager` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the manager cannot be initialized.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use sublime_monorepo_tools::{ChangesetManager, MonorepoProject};
    ///
    /// let project = Arc::new(MonorepoProject::new("/path/to/monorepo")?);
    /// let manager = ChangesetManager::new(project)?;
    /// ```
    pub fn new(project: Arc<MonorepoProject>) -> Result<Self, Error> {
        let storage = ChangesetStorage::new(Arc::clone(&project));

        let task_manager = TaskManager::new(Arc::clone(&project))?;

        Ok(Self { project, storage, task_manager })
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
            .project
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
            development_environments: self.project.config.changesets.default_environments.clone(),
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
            if self.project.get_package(&changeset.package).is_none() {
                errors.push(format!("Package '{}' not found in project", changeset.package));
            } else {
                // Validate version bump is appropriate
                match self.validate_version_bump(&changeset.package, changeset.version_bump) {
                    Ok(current_version) => {
                        metadata.insert("current_version".to_string(), current_version);
                    }
                    Err(e) => {
                        errors.push(format!("Version bump validation failed: {e}"));
                    }
                }
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
            if !self.project.config.environments.contains(env) {
                warnings
                    .push(format!("Environment {env} is not configured in project environments"));
            }
        }

        // Validate branch
        if changeset.branch.is_empty() {
            errors.push("Branch name cannot be empty".to_string());
        } else {
            // Check if branch follows naming conventions
            let valid_prefixes =
                ["feature/", "fix/", "feat/", "bugfix/", "hotfix/", "release/", "chore/"];
            let has_valid_prefix =
                valid_prefixes.iter().any(|prefix| changeset.branch.starts_with(prefix));

            let branch_config = &self.project.config.git.branches;
            if !has_valid_prefix && !branch_config.is_protected_branch(&changeset.branch) {
                warnings.push(format!(
                    "Branch '{}' doesn't follow conventional naming (feature/, fix/, etc.)",
                    changeset.branch
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

    /// Deploys a changeset to specific environments during development
    ///
    /// Executes deployment tasks for the specified environments and updates
    /// the changeset status accordingly.
    ///
    /// # Arguments
    ///
    /// * `changeset_id` - ID of the changeset to deploy
    /// * `environments` - Target environments for deployment
    ///
    /// # Returns
    ///
    /// Deployment result with success status and environment-specific results.
    ///
    /// # Errors
    ///
    /// Returns an error if the deployment cannot be initiated.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::Environment;
    ///
    /// # async fn example(manager: &ChangesetManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let environments = vec![Environment::Development, Environment::Staging];
    /// let result = manager.deploy_to_environments("changeset-123", &environments).await?;
    /// println!("Deployment success: {}", result.success);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn deploy_to_environments(
        &self,
        changeset_id: &str,
        environments: &[Environment],
    ) -> Result<DeploymentResult, Error> {
        let start_time = std::time::Instant::now();

        // Load changeset
        let changeset = self
            .storage
            .load(changeset_id)?
            .ok_or_else(|| Error::changeset(format!("Changeset {changeset_id} not found")))?;

        let mut environment_results = std::collections::HashMap::new();
        let mut overall_success = true;

        for environment in environments {
            let env_start = Utc::now();

            // Execute deployment for this environment
            let env_result = self.deploy_to_environment(&changeset, environment).await;

            let env_deployment_result = match env_result {
                Ok(()) => EnvironmentDeploymentResult {
                    success: true,
                    error: None,
                    started_at: env_start,
                    completed_at: Some(Utc::now()),
                    metadata: std::collections::HashMap::new(),
                },
                Err(e) => {
                    overall_success = false;
                    EnvironmentDeploymentResult {
                        success: false,
                        error: Some(e.to_string()),
                        started_at: env_start,
                        completed_at: Some(Utc::now()),
                        metadata: std::collections::HashMap::new(),
                    }
                }
            };

            environment_results.insert(environment.clone(), env_deployment_result);
        }

        Ok(DeploymentResult {
            changeset_id: changeset_id.to_string(),
            success: overall_success,
            environment_results,
            duration: start_time.elapsed(),
        })
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
            Error::changeset(format!("Invalid major version in {}: {}", package, version_parts[0]))
        })?;
        let minor: u32 = version_parts[1].parse().map_err(|_| {
            Error::changeset(format!("Invalid minor version in {}: {}", package, version_parts[1]))
        })?;
        let patch: u32 = version_parts[2].parse().map_err(|_| {
            Error::changeset(format!("Invalid patch version in {}: {}", package, version_parts[2]))
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
            .project
            .get_package(package)
            .ok_or_else(|| Error::changeset(format!("Package not found: {package}")))?;

        // Read package.json
        let package_json_path = package_info.path().join("package.json");
        let content =
            self.project.file_system.read_file_string(&package_json_path).map_err(|e| {
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
        for pkg in &self.project.packages {
            if pkg.name() == package {
                continue; // Skip self
            }

            // Read package.json to check dependencies
            let package_json_path = pkg.path().join("package.json");
            if let Ok(content) = self.project.file_system.read_file_string(&package_json_path) {
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
        match self.project.repository.list_config() {
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
            .project
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
            let full_path = self.project.root_path().join(file_path);

            if let Some(package) = self.project.descriptor.find_package_for_path(&full_path) {
                *package_file_counts.entry(package.name.clone()).or_insert(0) += 1;
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
            Error::changeset(format!("Invalid major version: {}", version_parts[0]))
        })?;
        let minor: u32 = version_parts[1].parse().map_err(|_| {
            Error::changeset(format!("Invalid minor version: {}", version_parts[1]))
        })?;
        let patch: u32 = version_parts[2].parse().map_err(|_| {
            Error::changeset(format!("Invalid patch version: {}", version_parts[2]))
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
            .project
            .get_package(package)
            .ok_or_else(|| Error::changeset(format!("Package not found: {package}")))?;

        // Read package.json
        let package_json_path = package_info.path().join("package.json");
        let content =
            self.project.file_system.read_file_string(&package_json_path).map_err(|e| {
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

        self.project
            .file_system
            .write_file(&package_json_path, updated_content.as_bytes())
            .map_err(|e| {
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
        let package_info = self.project.get_package(dependent_package).ok_or_else(|| {
            Error::changeset(format!("Dependent package not found: {dependent_package}"))
        })?;

        // Read package.json
        let package_json_path = package_info.path().join("package.json");
        let content =
            self.project.file_system.read_file_string(&package_json_path).map_err(|e| {
                Error::changeset(format!(
                    "Failed to read package.json for {dependent_package}: {e}"
                ))
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

            self.project
                .file_system
                .write_file(&package_json_path, updated_content.as_bytes())
                .map_err(|e| {
                    Error::changeset(format!(
                        "Failed to write updated package.json for {dependent_package}: {e}"
                    ))
                })?;
        }

        Ok(())
    }

    /// Deploys a changeset to a specific environment
    async fn deploy_to_environment(
        &self,
        changeset: &Changeset,
        environment: &Environment,
    ) -> Result<(), Error> {
        // Get deployment tasks for this environment
        let tasks = self.get_deployment_tasks_for_environment(environment)?;

        if tasks.is_empty() {
            // No specific deployment tasks for this environment
            log::info!("No deployment tasks configured for environment: {}", environment);
            return Ok(());
        }

        // Execute deployment tasks using TaskManager
        let task_results = self.task_manager.execute_tasks_batch(&tasks).await?;

        // Check if all tasks succeeded
        let failed_tasks: Vec<_> = task_results
            .iter()
            .filter(|result| {
                !matches!(result.status, crate::tasks::types::results::TaskStatus::Success)
            })
            .collect();

        if !failed_tasks.is_empty() {
            let error_msg = failed_tasks
                .iter()
                .map(|task| {
                    format!(
                        "{}: {}",
                        task.task_name,
                        task.errors.first().map_or("Unknown error", |e| &e.message)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");

            return Err(Error::changeset(format!(
                "Deployment to {} failed for changeset '{}': {}",
                environment, changeset.id, error_msg
            )));
        }

        log::info!(
            "✅ Successfully deployed changeset '{}' to environment '{}'",
            changeset.id,
            environment
        );
        Ok(())
    }

    /// Gets deployment tasks for a specific environment
    #[allow(clippy::unnecessary_wraps)]
    fn get_deployment_tasks_for_environment(
        &self,
        environment: &Environment,
    ) -> Result<Vec<String>, Error> {
        // Check if deployment tasks are configured for this environment in the project config
        let env_tasks = self.project.config.tasks.deployment_tasks.get(environment);

        if let Some(configured_tasks) = env_tasks {
            // Use configured tasks for this environment
            let available_tasks = self.task_manager.list_tasks();
            let available_task_names: std::collections::HashSet<String> =
                available_tasks.iter().map(|task| task.name.clone()).collect();

            // Filter to only include tasks that are actually registered
            let filtered_tasks: Vec<String> = configured_tasks
                .iter()
                .filter(|task_name| available_task_names.contains(*task_name))
                .cloned()
                .collect();

            if filtered_tasks.is_empty() {
                log::warn!(
                    "No registered tasks found for environment '{}' deployment",
                    environment
                );
            }

            Ok(filtered_tasks)
        } else {
            // Environment not configured, try to infer default tasks based on environment type
            let default_tasks = self.get_default_tasks_for_environment_type(environment);
            let available_tasks = self.task_manager.list_tasks();
            let available_task_names: std::collections::HashSet<String> =
                available_tasks.iter().map(|task| task.name.clone()).collect();

            // Filter default tasks to only include those that are registered
            let filtered_tasks: Vec<String> = default_tasks
                .into_iter()
                .filter(|task_name| available_task_names.contains(task_name))
                .collect();

            if filtered_tasks.is_empty() {
                log::info!(
                    "No deployment tasks available for environment '{}' - using basic validation",
                    environment
                );
                // Return basic tasks that should always exist
                Ok(vec!["build".to_string()])
            } else {
                Ok(filtered_tasks)
            }
        }
    }

    /// Gets default tasks for a given environment type when not explicitly configured
    #[allow(clippy::unused_self)]
    fn get_default_tasks_for_environment_type(&self, environment: &Environment) -> Vec<String> {
        match environment {
            Environment::Development => {
                vec!["build".to_string(), "test:unit".to_string(), "lint".to_string()]
            }
            Environment::Staging => {
                vec![
                    "build".to_string(),
                    "test:unit".to_string(),
                    "test:integration".to_string(),
                    "lint".to_string(),
                    "audit".to_string(),
                ]
            }
            Environment::Integration => {
                vec!["build".to_string(), "test:integration".to_string(), "test:e2e".to_string()]
            }
            Environment::Production => {
                vec![
                    "build".to_string(),
                    "test".to_string(),
                    "lint".to_string(),
                    "audit".to_string(),
                    "security-scan".to_string(),
                ]
            }
            Environment::Custom(_) => {
                vec!["build".to_string(), "test".to_string()]
            }
        }
    }
}
