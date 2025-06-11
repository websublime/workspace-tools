//! Changeset-related types

use crate::config::Environment;
use serde::{Deserialize, Serialize};

/// Changeset information for a package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changeset {
    /// Unique identifier for the changeset
    pub id: String,
    
    /// Package this changeset applies to
    pub package: String,
    
    /// Type of version bump
    pub version_bump: crate::config::VersionBumpType,
    
    /// Description of the changes
    pub description: String,
    
    /// Branch where the changeset was created
    pub branch: String,
    
    /// Development environments where this has been deployed
    pub development_environments: Vec<Environment>,
    
    /// Whether this has been deployed to production
    pub production_deployment: bool,
    
    /// When the changeset was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Author of the changeset
    pub author: String,
    
    /// Status of the changeset
    pub status: ChangesetStatus,
}

/// Status of a changeset
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangesetStatus {
    /// Changeset is pending application
    Pending,
    /// Changeset has been partially deployed
    PartiallyDeployed { 
        /// Environments where this has been deployed
        environments: Vec<Environment> 
    },
    /// Changeset has been fully deployed
    FullyDeployed { 
        /// When it was fully deployed
        deployed_at: chrono::DateTime<chrono::Utc> 
    },
    /// Changeset has been merged
    Merged { 
        /// When the changeset was merged
        merged_at: chrono::DateTime<chrono::Utc>,
        /// Final version after merge
        final_version: String,
    },
}