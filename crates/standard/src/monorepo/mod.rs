//! # Monorepo Management Module - Async Only
//!
//! ## What
//! This module provides tools for detecting, parsing, and working with
//! monorepo projects across different package managers (npm, yarn, pnpm, etc.).
//! It identifies workspace structures and manages dependencies between packages
//! using async operations for optimal performance.
//!
//! ## How
//! The module defines types to represent monorepo structures and provides
//! async methods to analyze workspace relationships. It supports different monorepo
//! formats including npm workspaces, Yarn workspaces, pnpm workspaces and more.
//!
//! ## Why
//! Monorepos require special handling to understand project structure,
//! dependencies between packages, and ensure commands are executed in the
//! correct context. This async-only module simplifies working with these complex
//! structures by providing a unified API with non-blocking operations.

mod descriptor;
mod detector;
mod kinds;
mod manager;
mod types;

#[cfg(test)]
mod tests;

pub use detector::{MonorepoDetector, MonorepoDetectorTrait, MonorepoDetectorWithFs};
pub use types::{MonorepoDescriptor, MonorepoKind, PnpmWorkspaceConfig, WorkspacePackage};
