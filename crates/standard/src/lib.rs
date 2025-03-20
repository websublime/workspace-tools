mod command;
mod error;
mod manager;
mod path;
mod utils;

pub use command::{execute, ComandResult};
pub use error::CommandError;
pub use manager::{detect_package_manager, CorePackageManager, CorePackageManagerError};
pub use path::get_project_root_path;
pub use utils::strip_trailing_newline;
