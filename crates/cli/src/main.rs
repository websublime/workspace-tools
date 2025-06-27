//! Sublime Monorepo CLI
//!
//! A command-line interface for managing monorepo operations including
//! versioning, task execution, dependency analysis, and workflow automation.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

use anyhow::Result;
use clap::Parser;
use std::process;

mod app;
mod commands;
mod config;
mod error;
mod output;
mod utils;

use app::MonorepoCliApp;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    // Parse CLI arguments
    let app = MonorepoCliApp::parse();

    // Run the application
    match app.run().await {
        Ok(()) => process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}