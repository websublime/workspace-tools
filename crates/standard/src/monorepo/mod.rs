//! # Monorepo Management Module
//!
//! ## What
//! This module provides tools for detecting, parsing, and working with
//! monorepo projects across different package managers (npm, yarn, pnpm, etc.).
//! It identifies workspace structures and manages dependencies between packages.
//!
//! ## How
//! The module defines types to represent monorepo structures and provides
//! methods to analyze workspace relationships. It supports different monorepo
//! formats including npm workspaces, Yarn workspaces, pnpm workspaces and more.
//!
//! ## Why
//! Monorepos require special handling to understand project structure,
//! dependencies between packages, and ensure commands are executed in the
//! correct context. This module simplifies working with these complex
//! structures by providing a unified API regardless of the underlying
//! monorepo implementation.

mod descriptor;
mod detector;
mod kinds;
mod manager;
mod types;

#[cfg(test)]
mod tests;

pub use types::{
    MonorepoDescriptor, MonorepoDetector, MonorepoKind, PnpmWorkspaceConfig, WorkspacePackage,
};

// Re-export types from project module for backwards compatibility
pub use crate::project::{ProjectConfig, ProjectManager, ProjectValidationStatus};
