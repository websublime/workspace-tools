mod dependency;
mod errors;
mod package;
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
    change::DependencyChange, dependency::Dependency, registry::DependencyRegistry,
    resolution::ResolutionResult, update::DependencyUpdate,
};

pub use errors::{
    dependency::DependencyResolutionError,
    package::{PackageError, PackageRegistryError},
    version::VersionError,
};

pub use version::version::{Version, VersionRelationship, VersionStability, VersionUpdateStrategy};
