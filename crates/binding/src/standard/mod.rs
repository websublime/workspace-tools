pub mod command;
pub mod manager;
pub mod paths;
pub mod utils;

// Re-export main types for convenience
pub use command::{execute, execute_with_status};
pub use manager::{detect_package_manager, CorePackageManager};
pub use paths::get_project_root_path;
pub use utils::strip_trailing_newline;
