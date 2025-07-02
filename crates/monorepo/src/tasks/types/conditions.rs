//! Task execution conditions
//!
//! Types that define when and how tasks should be executed based on
//! various conditions like file changes, package changes, or environment.

use crate::config::Environment;
use serde::{Deserialize, Serialize};

/// Conditions that must be met for a task to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskCondition {
    /// Specific packages have changed
    PackagesChanged {
        /// List of package names
        packages: Vec<String>,
    },

    /// Files matching patterns have changed
    FilesChanged {
        /// Glob patterns to match
        patterns: Vec<FilePattern>,
    },

    /// Dependencies have changed
    DependenciesChanged {
        /// Optional filter for specific dependencies
        filter: Option<DependencyFilter>,
    },

    /// Current branch matches pattern
    OnBranch {
        /// Branch pattern (supports wildcards)
        pattern: BranchCondition,
    },

    /// Specific environment is active
    Environment {
        /// Environment condition
        env: EnvironmentCondition,
    },

    /// Custom script returns true
    CustomScript {
        /// Script to execute
        script: String,
        /// Expected output
        expected_output: Option<String>,
    },

    /// All conditions must be true
    All {
        /// List of conditions
        conditions: Vec<TaskCondition>,
    },

    /// Any condition must be true
    Any {
        /// List of conditions
        conditions: Vec<TaskCondition>,
    },

    /// Condition must be false
    Not {
        /// Condition to negate
        condition: Box<TaskCondition>,
    },
}

/// Scope of task execution
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskScope {
    /// Run globally (once)
    Global,

    /// Run for a specific package
    Package(String),

    /// Run for all affected packages
    AffectedPackages,

    /// Run for all packages in monorepo
    AllPackages,

    /// Run for packages matching pattern
    PackagesMatching {
        /// Pattern to match package names
        pattern: String,
    },

    /// Custom scope with filter
    Custom {
        /// Filter function name
        filter: String,
    },
}

impl Default for TaskScope {
    fn default() -> Self {
        Self::AffectedPackages
    }
}

/// Task trigger conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskTrigger {
    /// Manual trigger only
    Manual,

    /// On file save
    OnSave,

    /// On commit
    OnCommit,

    /// On push
    OnPush,

    /// On pull request
    OnPullRequest,

    /// On merge
    OnMerge,

    /// On schedule
    Scheduled {
        /// Cron expression
        cron: String,
    },

    /// On webhook
    OnWebhook {
        /// Webhook event name
        event: String,
    },
}

/// File pattern for matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePattern {
    /// The pattern to match
    pub pattern: String,

    /// Whether this is an exclude pattern
    pub exclude: bool,

    /// Pattern type
    pub pattern_type: FilePatternType,
}

/// Type of file pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilePatternType {
    /// Glob pattern (default)
    Glob,
    /// Regular expression
    Regex,
    /// Exact path match
    Exact,
    /// Path prefix
    Prefix,
    /// Path suffix
    Suffix,
}

/// Filter for dependency changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyFilter {
    /// Include only these dependencies
    pub include: Vec<String>,

    /// Exclude these dependencies
    pub exclude: Vec<String>,

    /// Include dev dependencies
    pub include_dev: bool,

    /// Include peer dependencies
    pub include_peer: bool,

    /// Version change threshold
    pub version_change: VersionChangeThreshold,
}

impl Default for DependencyFilter {
    fn default() -> Self {
        Self {
            include: Vec::new(),
            exclude: Vec::new(),
            include_dev: false,
            include_peer: false,
            version_change: VersionChangeThreshold::Any,
        }
    }
}

/// Version change threshold for dependency updates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionChangeThreshold {
    /// Any version change
    Any,
    /// Major version changes only
    Major,
    /// Minor or major changes
    MinorOrMajor,
    /// Patch, minor, or major changes
    PatchOrHigher,
}

/// Branch condition for task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BranchCondition {
    /// Exact branch name
    Equals(String),

    /// Branch name pattern (with wildcards)
    Matches(String),

    /// Is main branch (main/master/develop)
    IsMain,

    /// Is feature branch
    IsFeature,

    /// Is release branch
    IsRelease,

    /// Is hotfix branch
    IsHotfix,

    /// One of multiple branches
    OneOf(Vec<String>),

    /// Not on these branches
    NoneOf(Vec<String>),
}

/// Environment condition for task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvironmentCondition {
    /// Specific environment
    Is(Environment),

    /// One of multiple environments
    OneOf(Vec<Environment>),

    /// Not these environments
    Not(Vec<Environment>),

    /// Environment variable exists
    VariableExists {
        /// Variable name
        key: String,
    },

    /// Environment variable has value
    VariableEquals {
        /// Variable name
        key: String,
        /// Expected value
        value: String,
    },

    /// Environment variable matches pattern
    VariableMatches {
        /// Variable name
        key: String,
        /// Pattern to match
        pattern: String,
    },

    /// Custom environment check
    Custom {
        /// Checker function name
        checker: String,
    },
}
