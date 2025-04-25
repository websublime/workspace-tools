//! Stream implementation for command output.
//!
//! What:
//! This module provides streaming functionality for command output,
//! allowing real-time processing of command output with proper buffering
//! and cancellation support.
//!
//! Who:
//! Used by developers who need to:
//! - Process command output in real-time
//! - Handle large output streams efficiently
//! - Implement output processing pipelines
//!
//! Why:
//! Streaming output processing is essential for:
//! - Handling large output without memory issues
//! - Providing real-time feedback
//! - Supporting cancellation of long-running commands

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{ChildStderr, ChildStdout},
    sync::mpsc,
    time::timeout,
};

use crate::error::{CommandError, CommandResult};

/// Output line from a command stream
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StreamOutput {
    /// Standard output line
    Stdout(String),
    /// Standard error line
    Stderr(String),
    /// Stream has ended
    End,
}

/// Configuration for command output streaming
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Buffer size for output channel
    buffer_size: usize,
    /// Read timeout for each line
    read_timeout: Duration,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self { buffer_size: 1024, read_timeout: Duration::from_secs(1) }
    }
}

impl StreamConfig {
    /// Creates a new StreamConfig with custom settings
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

/// Stream handler for command output
#[derive(Debug)]
pub struct CommandStream {
    /// Channel receiver for output
    rx: mpsc::Receiver<StreamOutput>,
    /// Flag for cancellation
    cancel: Arc<AtomicBool>,
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
    pub async fn next_timeout(
        &mut self,
        timeout_duration: Duration,
    ) -> CommandResult<Option<StreamOutput>> {
        match timeout(timeout_duration, self.rx.recv()).await {
            Ok(Some(output)) => Ok(Some(output)),
            Ok(None) => Ok(None),
            Err(_) => Err(CommandError::Timeout { duration: timeout_duration }),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Stdio;
    use tokio::{process::Command, time::sleep};

    #[tokio::test]
    async fn test_stream_output() -> Result<(), Box<dyn std::error::Error>> {
        let mut child = Command::new("echo")
            .arg("test")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        let mut stream = CommandStream::new(stdout, stderr, &StreamConfig::default());

        if let Ok(Some(StreamOutput::Stdout(line))) =
            stream.next_timeout(Duration::from_secs(1)).await
        {
            assert_eq!(line.trim(), "test");
            Ok(())
        } else {
            Err("Expected stdout output".into())
        }
    }

    #[tokio::test]
    async fn test_stream_timeout() -> Result<(), Box<dyn std::error::Error>> {
        let mut child =
            Command::new("sleep").arg("2").stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        let mut stream =
            CommandStream::new(stdout, stderr, &StreamConfig::new(10, Duration::from_millis(100)));

        assert!(stream.next_timeout(Duration::from_millis(100)).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_stream_cancellation() -> Result<(), Box<dyn std::error::Error>> {
        let mut child =
            Command::new("yes").stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        let stream = CommandStream::new(stdout, stderr, &StreamConfig::default());

        // Let it run for a bit
        sleep(Duration::from_millis(100)).await;

        // Cancel the stream
        stream.cancel();

        // Wait a bit to ensure cancellation takes effect
        sleep(Duration::from_millis(100)).await;

        // Kill the process
        child.kill().await?;
        Ok(())
    }
}
