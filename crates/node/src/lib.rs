#[macro_use]
extern crate napi_derive;

pub mod standard;

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

pub use standard::command::js_execute;
pub use standard::manager::js_detect_package_manager;
pub use standard::path::js_get_project_root_path;
