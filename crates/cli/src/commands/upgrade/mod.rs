//! Upgrade command implementations.
//!
//! This module provides functionality for managing dependency upgrades in workspace packages.
//!
//! # What
//!
//! Provides implementations for:
//! - `upgrade check` - Detect available dependency upgrades
//! - `upgrade apply` - Apply detected upgrades (Story 6.2)
//! - `upgrade backups` - Manage upgrade backups (Story 6.3)
//!   - `backups list` - List available backups
//!   - `backups restore` - Restore from backup
//!   - `backups clean` - Clean old backups
//!
//! # How
//!
//! The upgrade commands:
//! 1. Use `sublime-package-tools` upgrade module for detection and application
//! 2. Query npm registry for available versions
//! 3. Categorize upgrades by type (major, minor, patch)
//! 4. Filter by dependency type (prod, dev, peer)
//! 5. Format output as tables or JSON
//! 6. Support dry-run and confirmation workflows
//! 7. Manage backups for safe rollback
//!
//! # Why
//!
//! Keeping dependencies up-to-date is critical for:
//! - Security patches
//! - Bug fixes
//! - New features
//! - Ecosystem compatibility
//!
//! This module provides a safe, controlled workflow for managing upgrades
//! with visibility into what will change before applying updates and the
//! ability to rollback if needed.

pub mod apply;
pub mod check;
pub mod rollback;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export command implementations
pub use apply::execute_upgrade_apply;
pub use check::execute_upgrade_check;
pub use rollback::{execute_backup_clean, execute_backup_list, execute_backup_restore};
