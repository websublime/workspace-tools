mod dependency;
mod errors;
mod graph;
mod package;
mod registry;
mod upgrader;
mod version;

pub use package::{
    cache::CacheEntry,
    change::ChangeType,
    diff::PackageDiff,
    info::PackageInfo,
    package::Package,
    registry::{NpmRegistry, PackageRegistry},
    scope::{package_scope_name_version, PackageScopeMetadata},
};

pub use dependency::{
    change::DependencyChange, dependency::Dependency, filter::DependencyFilter,
    graph::DependencyGraph, registry::DependencyRegistry, resolution::ResolutionResult,
    update::DependencyUpdate, upgrader::DependencyUpgrader,
};

pub use errors::{
    dependency::DependencyResolutionError,
    package::{PackageError, PackageRegistryError},
    registry::RegistryError,
    version::VersionError,
};

pub use registry::{local::LocalRegistry, manager::RegistryManager};

pub use version::version::{Version, VersionRelationship, VersionStability, VersionUpdateStrategy};

pub use graph::{
    builder::{build_dependency_graph_from_package_infos, build_dependency_graph_from_packages},
    node::{Node, Step},
    validation::{ValidationIssue, ValidationReport},
    visualization::{generate_ascii, generate_dot, save_dot_to_file},
};

pub use upgrader::{
    config::{ExecutionMode, UpgradeConfig},
    status::{AvailableUpgrade, UpgradeStatus},
};
