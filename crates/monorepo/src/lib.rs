mod changes;
mod versioning;
mod workspace;

pub use workspace::{
    analysis::WorkspaceAnalysis, config::WorkspaceConfig, discovery::DiscoveryOptions,
    error::WorkspaceError, graph::WorkspaceGraph, manager::WorkspaceManager,
    validation::ValidationOptions, workspace::Workspace,
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
        determine_bump_type_from_change, suggest_version_bumps, VersionBumpPreview,
        VersionSuggestion,
    },
};
