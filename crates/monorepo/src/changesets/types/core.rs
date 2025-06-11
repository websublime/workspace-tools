//! Core changeset types
//!
//! This module contains the fundamental type definitions for the changeset system.
//! Changesets track planned changes to packages with environment deployment support.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::types::Environment;
use crate::VersionBumpType;

/// A changeset representing planned changes to a package
///
/// Changesets are used to track planned changes to packages with information
/// about version bumps, deployment environments, and deployment status.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::{Changeset, ChangesetStatus, Environment, VersionBumpType};
/// use chrono::Utc;
///
/// let changeset = Changeset {
///     id: "abc123".to_string(),
///     package: "@test/core".to_string(),
///     version_bump: VersionBumpType::Minor,
///     description: "Add new API endpoint".to_string(),
///     branch: "feature/new-api".to_string(),
///     development_environments: vec![Environment::Development, Environment::Staging],
///     production_deployment: false,
///     created_at: Utc::now(),
///     author: "developer@example.com".to_string(),
///     status: ChangesetStatus::Pending,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Changeset {
    /// Unique identifier for this changeset
    pub id: String,
    
    /// Package name that this changeset affects
    pub package: String,
    
    /// Type of version bump for this change
    pub version_bump: VersionBumpType,
    
    /// Human-readable description of the changes
    pub description: String,
    
    /// Git branch where this changeset was created
    pub branch: String,
    
    /// Environments where this change should be deployed during development
    pub development_environments: Vec<Environment>,
    
    /// Whether this changeset should be deployed to production
    pub production_deployment: bool,
    
    /// When this changeset was created
    pub created_at: DateTime<Utc>,
    
    /// Author of this changeset
    pub author: String,
    
    /// Current status of this changeset
    pub status: ChangesetStatus,
}

/// Status of a changeset
///
/// Tracks the deployment and merge status of a changeset through its lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangesetStatus {
    /// Changeset is created but not yet deployed anywhere
    Pending,
    
    /// Changeset has been deployed to some but not all environments
    PartiallyDeployed {
        /// Environments where this changeset has been deployed
        environments: Vec<Environment>,
    },
    
    /// Changeset has been deployed to all specified environments
    FullyDeployed {
        /// When the final deployment completed
        deployed_at: DateTime<Utc>,
    },
    
    /// Changeset has been merged and finalized
    Merged {
        /// When the changeset was merged
        merged_at: DateTime<Utc>,
        
        /// Final version that was applied
        final_version: String,
    },
}

/// Specification for creating a new changeset
///
/// Contains all the information needed to create a new changeset.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::{ChangesetSpec, Environment, VersionBumpType};
///
/// let spec = ChangesetSpec {
///     package: "@test/core".to_string(),
///     version_bump: VersionBumpType::Patch,
///     description: "Fix critical bug in authentication".to_string(),
///     development_environments: vec![Environment::Development],
///     production_deployment: true,
///     author: Some("developer@example.com".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangesetSpec {
    /// Package name that this changeset will affect
    pub package: String,
    
    /// Type of version bump for this change
    pub version_bump: VersionBumpType,
    
    /// Human-readable description of the changes
    pub description: String,
    
    /// Environments where this change should be deployed during development
    pub development_environments: Vec<Environment>,
    
    /// Whether this changeset should be deployed to production
    pub production_deployment: bool,
    
    /// Optional author (if not provided, will be inferred from Git config)
    pub author: Option<String>,
}

/// Result of applying a changeset
///
/// Contains information about what happened when a changeset was applied,
/// including version changes and deployment status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangesetApplication {
    /// ID of the changeset that was applied
    pub changeset_id: String,
    
    /// Package that was affected
    pub package: String,
    
    /// Version before the changeset was applied
    pub old_version: String,
    
    /// Version after the changeset was applied
    pub new_version: String,
    
    /// Environments where this changeset was deployed
    pub environments_deployed: Vec<Environment>,
    
    /// Whether the application was successful
    pub success: bool,
}

/// Filter for querying changesets
///
/// Allows filtering changesets by various criteria.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::{ChangesetFilter, ChangesetStatus, Environment};
///
/// // Filter for pending changesets for a specific package
/// let filter = ChangesetFilter {
///     package: Some("@test/core".to_string()),
///     status: Some(ChangesetStatus::Pending),
///     environment: None,
///     branch: None,
///     author: None,
/// };
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangesetFilter {
    /// Filter by package name
    pub package: Option<String>,
    
    /// Filter by changeset status
    pub status: Option<ChangesetStatus>,
    
    /// Filter by environment (changesets that deploy to this environment)
    pub environment: Option<Environment>,
    
    /// Filter by Git branch
    pub branch: Option<String>,
    
    /// Filter by author
    pub author: Option<String>,
}

/// Result of changeset validation
///
/// Contains validation results and any errors or warnings found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the changeset is valid
    pub is_valid: bool,
    
    /// Validation errors that prevent the changeset from being used
    pub errors: Vec<String>,
    
    /// Validation warnings that should be addressed but don't prevent usage
    pub warnings: Vec<String>,
    
    /// Additional metadata from validation
    pub metadata: HashMap<String, String>,
}

/// Result of deploying a changeset to environments
///
/// Contains information about deployment success and any failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentResult {
    /// Changeset that was deployed
    pub changeset_id: String,
    
    /// Overall success of the deployment
    pub success: bool,
    
    /// Results for each environment
    pub environment_results: HashMap<Environment, EnvironmentDeploymentResult>,
    
    /// Overall deployment duration
    pub duration: std::time::Duration,
}

/// Result of deploying to a specific environment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentDeploymentResult {
    /// Whether deployment to this environment was successful
    pub success: bool,
    
    /// Error message if deployment failed
    pub error: Option<String>,
    
    /// When deployment started
    pub started_at: DateTime<Utc>,
    
    /// When deployment completed (successfully or with failure)
    pub completed_at: Option<DateTime<Utc>>,
    
    /// Any metadata from the deployment process
    pub metadata: HashMap<String, String>,
}