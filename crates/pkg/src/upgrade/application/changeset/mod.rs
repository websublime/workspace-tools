//! Automatic changeset creation for dependency upgrades.
//!
//! **What**: Provides functionality to automatically create or update changesets after
//! applying dependency upgrades, tracking which packages were affected and ensuring
//! proper version bump configuration. Also provides high-level integration functions
//! that combine upgrade application with automatic changeset creation.
//!
//! **How**: This module integrates with the changeset manager to create changesets
//! with affected packages, configurable version bump types, and proper metadata.
//! It supports both creating new changesets and updating existing ones based on
//! the current git branch. The applier module wraps the core upgrade application
//! logic and adds automatic changeset creation based on configuration.
//!
//! **Why**: To enable automated tracking of dependency upgrades through the changeset
//! workflow, ensuring upgrades are properly versioned and documented without manual
//! intervention, providing a seamless integration between dependency upgrades and
//! the changeset workflow.

mod applier;
mod creator;

#[cfg(test)]
mod tests;

// Re-export public API
pub use applier::apply_with_changeset;
