//! JavaScript bindings for graph node trait and implementations.

use napi_derive::napi;

/// Represents a node in a dependency graph
///
/// This is a marker interface in JavaScript - the actual Node trait
/// is implemented by Package in Rust.
#[napi]
pub struct Node {
    // This is just a marker class for the JavaScript interface
    // The actual Node implementations are in Rust
}

#[napi]
#[allow(clippy::new_without_default)]
impl Node {
    /// Create a new Node instance
    ///
    /// @returns {Node} a new Node instance
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {}
    }
}
