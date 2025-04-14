pub mod changes;
pub mod tasks;
pub mod versioning;
pub mod workspace;

pub use workspace::{
    analysis::WorkspaceAnalysis,
    config::WorkspaceConfig,
    discovery::DiscoveryOptions,
    error::WorkspaceError,
    graph::WorkspaceGraph,
    manager::WorkspaceManager,
    validation::ValidationOptions,
    workspace::{SortedPackages, Workspace},
};

pub use changes::{
    change::{Change, ChangeId, ChangeResult, ChangeType},
    changeset::Changeset,
    error::ChangeError,
    file::FileChangeStore,
    memory::MemoryChangeStore,
    store::ChangeStore,
    tracker::{ChangeScope, ChangeTracker},
};

pub use versioning::{
    bump::{VersionInconsistency, VersionManager, VersionValidation},
    error::{VersioningError, VersioningResult},
    strategy::{BumpReason, BumpType, ChangelogOptions, PackageVersionChange, VersionBumpStrategy},
    suggest::{
        determine_bump_type_from_change, print_version_bump_preview, suggest_version_bumps,
        suggest_version_bumps_with_options, VersionBumpPreview, VersionSuggestion,
    },
};

pub use tasks::{
    error::{TaskError, TaskResult},
    filter::TaskFilter,
    graph::{TaskGraph, TaskSortMode},
    info::TaskResultInfo,
    parallel::{
        default_parallel_config, fail_fast_parallel_config, parallel_config_with_concurrency,
        ParallelExecutionConfig, ParallelExecutor,
    },
    runner::TaskRunner,
    task::{Task, TaskConfig, TaskExecution, TaskStatus},
};
