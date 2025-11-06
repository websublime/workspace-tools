//! # Project Descriptor Types
//!
//! ## What
//! This module defines the `ProjectDescriptor` enum and related types
//! for representing different project types in a unified way.
//!
//! ## How
//! The `ProjectDescriptor` enum serves as a container for different
//! project types, providing type-safe access through a common interface.
//!
//! ## Why
//! A unified descriptor enables consistent handling of different project
//! types while maintaining type safety and clean API boundaries.

use crate::project::Project;
use crate::project::types::ProjectInfo;

/// Represents different types of Node.js projects with their specific data.
///
/// This enum serves as a container that can hold either a simple project
/// or a monorepo descriptor, providing type-safe access to project-specific
/// functionality while maintaining a common interface through the `ProjectInfo` trait.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::{ProjectDescriptor, ProjectInfo};
///
/// fn process_project(descriptor: ProjectDescriptor) {
///     match descriptor {
///         ProjectDescriptor::NodeJs(project) => {
///             println!("Processing project at {}", project.root().display());
///             if project.is_monorepo() {
///                 println!("Type: Monorepo with {} packages", project.internal_dependencies.len());
///             } else {
///                 println!("Type: Simple project");
///             }
///         }
///     }
/// }
/// ```
#[derive(Debug)]
pub enum ProjectDescriptor {
    /// A Node.js project (simple or monorepo)
    NodeJs(Project),
}

impl ProjectDescriptor {
    /// Returns a reference to the project as a trait object.
    ///
    /// This method provides unified access to project information
    /// regardless of the underlying type.
    ///
    /// # Returns
    ///
    /// A reference to the project implementing the `ProjectInfo` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::{ProjectDescriptor, ProjectInfo};
    /// # fn example(descriptor: ProjectDescriptor) {
    /// let info = descriptor.as_project_info();
    /// println!("Project type: {}", info.kind().name());
    /// # }
    /// ```
    #[must_use]
    pub fn as_project_info(&self) -> &dyn ProjectInfo {
        match self {
            Self::NodeJs(project) => project,
        }
    }
}
