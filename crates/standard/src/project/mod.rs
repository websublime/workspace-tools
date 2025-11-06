//! # Project Management Module
//!
//! ## What
//! This module provides a unified API for detecting, managing, and working with
//! Node.js projects, regardless of whether they are simple repositories or monorepos.
//! It abstracts common functionality and provides a consistent interface for all
//! project types.
//!
//! ## How
//! The module defines traits and types that represent project structures and provides
//! implementations for different project types. It uses a detector pattern to identify
//! project types and returns appropriate implementations through a common interface.
//! Project-specific configuration is handled through the `StandardConfig` type from the config module.
//!
//! ## Why
//! Node.js projects share many common characteristics regardless of their structure.
//! This module eliminates code duplication and provides a consistent API for working
//! with any Node.js project, making it easier to build tools that work across
//! different project types. For general configuration management, see the `config`
//! module which provides a comprehensive configuration framework.

mod detector;
mod manager;
pub mod project;
mod types;
mod validator;

#[cfg(test)]
mod tests;

pub use detector::{ProjectDetector, ProjectDetectorTrait, ProjectDetectorWithFs};
pub use manager::ProjectManager;
pub use project::{Dependencies, Project};
pub use types::{ProjectDescriptor, ProjectInfo, ProjectKind, ProjectValidationStatus};
pub use validator::ProjectValidator;
