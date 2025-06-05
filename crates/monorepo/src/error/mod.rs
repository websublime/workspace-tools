//! Error handling module for monorepo tools

mod error;

#[cfg(test)]
mod tests;

pub use error::{Error, Result};