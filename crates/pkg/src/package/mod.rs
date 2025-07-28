pub mod cache;
pub mod change;
pub mod diff;
pub mod info;
pub mod manager;
pub mod package;
pub mod registry;
pub mod scope;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod benchmarks;

#[cfg(test)]
mod examples;
