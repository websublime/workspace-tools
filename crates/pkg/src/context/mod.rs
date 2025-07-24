//! # Context-Aware Architecture Module
//!
//! This module provides context-aware architecture support for package tools,
//! enabling the system to adapt its behavior based on whether it's operating
//! in a single repository or monorepo environment.
//!
//! ## Overview
//!
//! The context-aware architecture is a key differentiator that allows the package
//! tools to optimize their behavior for different project structures:
//!
//! - **Single Repository**: Optimized for network I/O, simplified dependency classification
//! - **Monorepo**: Optimized for filesystem I/O, advanced workspace features enabled
//!
//! ## Key Components
//!
//! - **ProjectContext**: Enum defining the project structure context
//! - **ContextDetector**: Auto-detects the project context
//! - **DependencyClassifier**: Context-aware dependency classification
//! - **ProtocolSupport**: Different protocols supported per context
//!
//! ## Examples
//!
//! ```rust
//! use sublime_package_tools::context::{ProjectContext, ContextDetector};
//! use sublime_standard_tools::filesystem::AsyncFileSystem;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let fs = AsyncFileSystem::new();
//! let detector = ContextDetector::new(fs);
//!
//! let context = detector.detect_context().await?;
//! match context {
//!     ProjectContext::Single(config) => {
//!         println!("Single repository detected");
//!         // Network-optimized operations
//!     }
//!     ProjectContext::Monorepo(config) => {
//!         println!("Monorepo detected with {} packages", config.workspace_packages.len());
//!         // Workspace-aware operations
//!     }
//! }
//! # Ok(())
//! # }\n//! ```

pub mod project;
pub mod detection;
pub mod classification;
pub mod protocols;
pub mod dependency_source;
pub mod dependency_parser;

#[cfg(test)]
mod test_parsing;

pub use project::{
    ProjectContext, 
    SingleRepositoryContext, 
    MonorepoContext,
    SingleRepoFeatures,
    MonorepoFeatures,
};
pub use detection::ContextDetector;
pub use classification::{DependencyClassifier, DependencyClass, InternalClassification};
pub use protocols::{DependencyProtocol, ProtocolSupport};
pub use dependency_source::{DependencySource, WorkspaceConstraint, GitReference};
pub use dependency_parser::DependencyParser;