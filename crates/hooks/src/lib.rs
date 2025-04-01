mod error;
mod hook;
mod package;
mod pre_commit;

pub use error::{HookError, HookResult};
pub use hook::{HookConfig, HookContext, VersionDecision};
pub use pre_commit::PreCommitHook; 