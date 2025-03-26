mod dependency;
mod errors;
mod package;
mod version;

pub use package::{
    info::PackageInfo,
    package::Package,
    scope::{package_scope_name_version, PackageScopeMetadata},
};

pub use dependency::{
    dependency::Dependency, registry::DependencyRegistry, resolution::ResolutionResult,
    update::DependencyUpdate,
};

pub use errors::{
    dependency::DependencyResolutionError, package::PackageError, version::VersionError,
};

pub use version::version::{Version, VersionRelationship, VersionStability, VersionUpdateStrategy};
