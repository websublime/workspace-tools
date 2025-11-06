//! # Error type definitions
//!
//! ## What
//! This file contains the core type definitions for errors used throughout
//! the `sublime_standard_tools` crate. It defines error enums and result type
//! aliases for various domains.
//!
//! ## How
//! Errors are defined using thiserror for automatic trait implementations.
//! Each error variant includes descriptive fields and error messages.
//!
//! ## Why
//! Centralizing error type definitions provides a clear overview of all
//! possible error conditions and ensures consistency in error handling.

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{ChildStderr, ChildStdout},
    sync::mpsc,
    time::timeout,
};

use super::types::{CommandStream, StreamConfig, StreamOutput};
use crate::error::{CommandError, Error, Result};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

impl Default for StreamConfig {
    fn default() -> Self {
        Self { buffer_size: 1024, read_timeout: Duration::from_secs(1) }
    }
}

impl StreamConfig {
    /// Creates a new `StreamConfig` with custom settings
    ///
    /// # Arguments
    ///
    /// * `buffer_size` - Size of the output buffer
    /// * `read_timeout` - Timeout for reading each line
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::command::StreamConfig;
    /// use std::time::Duration;
    ///
    /// let config = StreamConfig::new(2048, Duration::from_secs(2));
    /// ```
    #[must_use]
    pub fn new(buffer_size: usize, read_timeout: Duration) -> Self {
        Self { buffer_size, read_timeout }
    }
}

impl CommandStream {
    /// Creates a new command stream from stdout and stderr handles
    ///
    /// # Arguments
    ///
    /// * `stdout` - Standard output handle
    /// * `stderr` - Standard error handle
    /// * `config` - Stream configuration
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use sublime_standard_tools::command::{CommandStream, StreamConfig};
    /// use tokio::process::Command;
    /// use std::process::Stdio;
    ///
    /// # async fn example() {
    /// let mut child = Command::new("ls")
    ///     .stdout(Stdio::piped())
    ///     .stderr(Stdio::piped())
    ///     .spawn()
    ///     .unwrap();
    ///
    /// let stdout = child.stdout.take().unwrap();
    /// let stderr = child.stderr.take().unwrap();
    ///
    /// let stream = CommandStream::new(stdout, stderr, StreamConfig::default());
    /// # }
    /// ```
    #[must_use]
    pub fn new(stdout: ChildStdout, stderr: ChildStderr, config: &StreamConfig) -> Self {
        let (tx, rx) = mpsc::channel(config.buffer_size);
        let cancel = Arc::new(AtomicBool::new(false));

        let stdout_cancel = Arc::clone(&cancel);
        let stderr_cancel = Arc::clone(&cancel);
        let stdout_tx = tx.clone();
        let stderr_tx = tx;

        // Spawn stdout reader
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if stdout_cancel.load(Ordering::Relaxed) {
                    break;
                }

                if stdout_tx.send(StreamOutput::Stdout(line)).await.is_err() {
                    break;
                }
            }
        });

        // Spawn stderr reader
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if stderr_cancel.load(Ordering::Relaxed) {
                    break;
                }

                if stderr_tx.send(StreamOutput::Stderr(line)).await.is_err() {
                    break;
                }
            }
        });

        Self { rx, cancel }
    }

    /// Receives the next output line with timeout
    ///
    /// # Arguments
    ///
    /// * `timeout_duration` - Maximum time to wait for next line
    ///
    /// # Returns
    ///
    /// Result containing the next output line or error if timeout occurs
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use sublime_standard_tools::command::{CommandStream, StreamConfig};
    /// use std::time::Duration;
    ///
    /// # async fn example(mut stream: CommandStream) {
    /// while let Ok(Some(output)) = stream.next_timeout(Duration::from_secs(1)).await {
    ///     println!("Received: {:?}", output);
    /// }
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns an error if the timeout is reached while waiting for output.
    pub async fn next_timeout(
        &mut self,
        timeout_duration: Duration,
    ) -> Result<Option<StreamOutput>> {
        match timeout(timeout_duration, self.rx.recv()).await {
            Ok(Some(output)) => Ok(Some(output)),
            Ok(None) => Ok(None),
            Err(_) => Err(Error::Command(CommandError::Timeout { duration: timeout_duration })),
        }
    }

    /// Cancels the stream
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use sublime_standard_tools::command::{CommandStream, StreamConfig};
    ///
    /// # async fn example(mut stream: CommandStream) {
    /// // In another task
    /// stream.cancel();
    /// # }
    /// ```
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
