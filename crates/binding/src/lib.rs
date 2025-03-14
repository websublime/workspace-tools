#[macro_use]
extern crate napi_derive;

pub mod bump;
pub mod errors;
pub mod graph;
pub mod registry;
pub mod types;
pub mod upgrader;

#[cfg(all(not(target_os = "linux"), not(target_family = "wasm")))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[ctor::ctor]
fn init() {
    // Initialize any global state here if needed
}

#[napi]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// Export the error handling utility for use in other modules
pub use errors::{handle_pkg_result, pkg_error_to_napi_error, ErrorCode};
pub use graph::{
    build_dependency_graph_from_package_infos, build_dependency_graph_from_packages,
    generate_ascii, generate_dot, save_dot_to_file, DependencyFilter, DependencyGraph, DotOptions,
    ValidationIssueInfo, ValidationIssueType, ValidationReport,
};
pub use registry::{
    DependencyRegistry, DependencyUpdateInfo, PackageRegistry, RegistryAuthConfig, RegistryManager,
    RegistryType, ResolutionErrorType, ResolutionResult,
};
pub use types::dependency::Dependency;
pub use types::diff::{ChangeType, DependencyChange, PackageDiff};
pub use types::package::{Package, PackageInfo};
pub use types::version::{Version, VersionComparisonResult, VersionUtils};
// Add the new exports
pub use bump::{bump_snapshot_version, bump_version, BumpOptions};
pub use upgrader::{
    create_default_upgrade_config, create_upgrade_config_from_strategy,
    create_upgrade_config_with_registries, AvailableUpgrade, DependencyUpgrader, ExecutionMode,
    UpgradeConfig, UpgradeStatus,
};
