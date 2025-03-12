#[macro_use]
extern crate napi_derive;

pub mod errors;
pub mod registry;
pub mod types;

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

// Re-export all the types for convenience
pub use registry::dependency::{
    DependencyRegistry, DependencyUpdateInfo, ResolutionErrorType, ResolutionResult,
};
pub use types::dependency::Dependency;
pub use types::package::Package;
pub use types::version::{Version, VersionComparisonResult, VersionUtils};
