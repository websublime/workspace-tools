//! # Command Stream Types
//!
//! ## What
//! This module defines types for handling streaming command output,
//! including stream configuration and output types.
//!
//! ## How
//! The types provide a structured way to handle real-time command output
//! with proper configuration and type safety.
//!
//! ## Why
//! Streaming types enable real-time command output processing,
//! essential for interactive command execution and monitoring.

use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};
use tokio::sync::mpsc;

/// Type of output from a command stream.
///
/// Used to differentiate between standard output, standard error, and stream end markers.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::StreamOutput;
///
/// fn process_output(output: StreamOutput) {
///     match output {
///         StreamOutput::Stdout(line) => println!("STDOUT: {}", line),
///         StreamOutput::Stderr(line) => eprintln!("STDERR: {}", line),
///         StreamOutput::End => println!("Stream ended"),
///     }
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StreamOutput {
    /// Standard output line
    Stdout(String),
    /// Standard error line
    Stderr(String),
    /// Stream has ended
    End,
}

/// Configuration for command output streaming.
///
/// Defines parameters for how command output streams are handled,
/// including buffer sizes and timeouts.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::StreamConfig;
///
/// let config = StreamConfig::default();
/// ```
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Buffer size for output channel
    pub(crate) buffer_size: usize,
    /// Read timeout for each line
    pub(crate) read_timeout: Duration,
}

/// Stream of output from a running command.
///
/// Provides an asynchronous stream of stdout and stderr from a command,
/// allowing real-time processing of command output.
///
/// # Examples
///
/// ```ignore
/// // CommandStream is created internally by executors
/// use sublime_standard_tools::command::{DefaultCommandExecutor, Executor, CommandBuilder, StreamConfig, StreamOutput};
///
/// async fn stream_command() -> sublime_standard_tools::error::Result<()> {
///     let executor = DefaultCommandExecutor::new();
///     let cmd = CommandBuilder::new("ls").arg("-la").build();
///     let (mut stream, mut child) = executor.execute_stream(cmd, StreamConfig::default()).await?;
///
///     while let Ok(Some(output)) = stream.next_timeout(std::time::Duration::from_secs(1)).await {
///         match output {
///             StreamOutput::Stdout(line) => println!("STDOUT: {}", line),
///             StreamOutput::Stderr(line) => eprintln!("STDERR: {}", line),
///             StreamOutput::End => break,
///         }
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct CommandStream {
    /// Channel receiver for output
    pub(crate) rx: mpsc::Receiver<StreamOutput>,
    /// Flag for cancellation
    pub(crate) cancel: Arc<AtomicBool>,
}
