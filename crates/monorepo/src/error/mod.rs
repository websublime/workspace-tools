//! Error handling module for monorepo tools

mod types;

#[cfg(test)]
mod tests;

pub use types::{Error, Result};