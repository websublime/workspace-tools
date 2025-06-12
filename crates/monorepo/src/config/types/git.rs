//! Git configuration types
//!
//! This module defines configuration structures for Git operations,
//! including default references and branch configurations.

use serde::{Deserialize, Serialize};

/// Configuration for Git operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// Default reference for change detection (e.g., "HEAD~1", "main")
    pub default_since_ref: String,
    
    /// Default target for comparisons (e.g., "HEAD")
    pub default_until_ref: String,
    
    /// Remote name for push operations (e.g., "origin")
    pub default_remote: String,
    
    /// Branch configuration
    pub branches: BranchConfig,
}

/// Configuration for branch operations and classifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchConfig {
    /// List of main/production branches
    pub main_branches: Vec<String>,
    
    /// List of development branches
    pub develop_branches: Vec<String>,
    
    /// List of release branch prefixes
    pub release_prefixes: Vec<String>,
    
    /// List of feature branch prefixes
    pub feature_prefixes: Vec<String>,
    
    /// List of hotfix branch prefixes
    pub hotfix_prefixes: Vec<String>,
    
    /// Default branch for new features
    pub default_base_branch: String,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            default_since_ref: "HEAD~1".to_string(),
            default_until_ref: "HEAD".to_string(),
            default_remote: "origin".to_string(),
            branches: BranchConfig::default(),
        }
    }
}

impl Default for BranchConfig {
    fn default() -> Self {
        Self {
            main_branches: vec![
                "main".to_string(),
                "master".to_string(),
                "trunk".to_string(),
            ],
            develop_branches: vec![
                "develop".to_string(),
                "dev".to_string(),
                "development".to_string(),
            ],
            release_prefixes: vec![
                "release/".to_string(),
                "releases/".to_string(),
            ],
            feature_prefixes: vec![
                "feature/".to_string(),
                "feat/".to_string(),
                "features/".to_string(),
            ],
            hotfix_prefixes: vec![
                "hotfix/".to_string(),
                "fix/".to_string(),
                "bugfix/".to_string(),
            ],
            default_base_branch: "main".to_string(),
        }
    }
}

impl GitConfig {
    /// Create a new GitConfig with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a GitConfig with custom default reference
    #[must_use]
    pub fn with_default_since_ref(mut self, since_ref: impl Into<String>) -> Self {
        self.default_since_ref = since_ref.into();
        self
    }
    
    /// Create a GitConfig with custom branch configuration
    #[must_use]
    pub fn with_branches(mut self, branches: BranchConfig) -> Self {
        self.branches = branches;
        self
    }
}

impl BranchConfig {
    /// Check if a branch is considered a main/production branch
    #[must_use]
    pub fn is_main_branch(&self, branch: &str) -> bool {
        self.main_branches.iter().any(|main_branch| branch == main_branch)
    }
    
    /// Check if a branch is considered a development branch
    #[must_use]
    pub fn is_develop_branch(&self, branch: &str) -> bool {
        self.develop_branches.iter().any(|dev_branch| branch == dev_branch)
    }
    
    /// Check if a branch is a release branch
    #[must_use]
    pub fn is_release_branch(&self, branch: &str) -> bool {
        self.release_prefixes.iter().any(|prefix| branch.starts_with(prefix))
    }
    
    /// Check if a branch is a feature branch
    #[must_use]
    pub fn is_feature_branch(&self, branch: &str) -> bool {
        self.feature_prefixes.iter().any(|prefix| branch.starts_with(prefix))
    }
    
    /// Check if a branch is a hotfix branch
    #[must_use]
    pub fn is_hotfix_branch(&self, branch: &str) -> bool {
        self.hotfix_prefixes.iter().any(|prefix| branch.starts_with(prefix))
    }
    
    /// Check if a branch is protected (main or develop)
    #[must_use]
    pub fn is_protected_branch(&self, branch: &str) -> bool {
        self.is_main_branch(branch) || self.is_develop_branch(branch)
    }
    
    /// Get the appropriate base branch for a new branch
    #[must_use]
    pub fn get_base_branch(&self, branch_type: BranchType) -> &str {
        match branch_type {
            BranchType::Feature | BranchType::Release => &self.default_base_branch,
            BranchType::Hotfix => {
                // Hotfixes typically branch from main
                self.main_branches.first().unwrap_or(&self.default_base_branch)
            }
            BranchType::Main | BranchType::Develop => branch_type.as_str(),
        }
    }
}

/// Types of branches in a Git workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchType {
    /// Main/production branch
    Main,
    /// Development branch
    Develop, 
    /// Feature branch
    Feature,
    /// Release branch
    Release,
    /// Hotfix branch
    Hotfix,
}

impl BranchType {
    /// Get the branch type as a string
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Develop => "develop",
            Self::Feature => "feature",
            Self::Release => "release", 
            Self::Hotfix => "hotfix",
        }
    }
}

impl BranchConfig {
    /// Determine the type of a branch based on its name
    #[must_use]
    pub fn get_branch_type(&self, branch: &str) -> BranchType {
        if self.is_main_branch(branch) {
            BranchType::Main
        } else if self.is_develop_branch(branch) {
            BranchType::Develop
        } else if self.is_release_branch(branch) {
            BranchType::Release
        } else if self.is_hotfix_branch(branch) {
            BranchType::Hotfix
        } else {
            BranchType::Feature // Default to feature branch
        }
    }
}