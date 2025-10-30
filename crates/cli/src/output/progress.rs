//! Progress indicators for long-running operations.
//!
//! This module provides progress bars and spinners for CLI operations that take time.
//! It uses the `indicatif` crate and automatically handles:
//! - TTY detection (no progress in non-TTY environments)
//! - JSON mode suppression (no progress when outputting JSON)
//! - Quiet mode suppression (no progress in quiet mode)
//! - Proper cleanup on completion or error
//!
//! # What
//!
//! Provides:
//! - `ProgressBar` wrapper for determinate operations (known total)
//! - `Spinner` wrapper for indeterminate operations (unknown duration)
//! - `MultiProgress` for managing multiple concurrent progress indicators
//! - Automatic suppression based on output format and terminal capabilities
//!
//! # How
//!
//! Uses the `indicatif` crate for rendering progress indicators with consistent
//! styling that matches the CLI theme. Automatically detects when progress should
//! be suppressed (non-TTY, JSON mode, quiet mode) and becomes a no-op in those cases.
//!
//! # Why
//!
//! Progress indicators improve user experience for long-running operations by:
//! - Showing that work is being done (preventing "is it hung?" questions)
//! - Providing feedback on completion percentage
//! - Estimating time remaining
//! - Not interfering with structured output (JSON)
//!
//! # Examples
//!
//! Using a spinner for indeterminate operations:
//!
//! ```rust
//! use sublime_cli_tools::output::progress::Spinner;
//!
//! let spinner = Spinner::new("Loading packages...");
//! // ... do work ...
//! spinner.finish_with_message("✓ Loaded 5 packages");
//! ```
//!
//! Using a progress bar for determinate operations:
//!
//! ```rust
//! use sublime_cli_tools::output::progress::ProgressBar;
//!
//! let progress = ProgressBar::new(100);
//! progress.set_message("Processing files...");
//!
//! for i in 0..100 {
//!     // ... do work ...
//!     progress.inc(1);
//! }
//!
//! progress.finish_with_message("✓ Complete");
//! ```
//!
//! Managing multiple progress indicators:
//!
//! ```rust
//! use sublime_cli_tools::output::progress::MultiProgress;
//!
//! let multi = MultiProgress::new();
//! let pb1 = multi.add_progress_bar(100);
//! let pb2 = multi.add_progress_bar(50);
//!
//! // Both progress bars update independently
//! pb1.set_message("Task 1");
//! pb2.set_message("Task 2");
//! ```

use console::Term;
use indicatif::{
    MultiProgress as IndicatifMultiProgress, ProgressBar as IndicatifProgressBar, ProgressStyle,
};
use std::time::Duration;

use super::OutputFormat;

/// Progress bar for determinate operations with known total.
///
/// Automatically suppressed in non-TTY, JSON mode, or quiet mode.
/// When suppressed, all operations become no-ops.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::progress::ProgressBar;
///
/// let pb = ProgressBar::new(100);
/// pb.set_message("Processing...");
///
/// for i in 0..100 {
///     pb.inc(1);
/// }
///
/// pb.finish_with_message("✓ Done");
/// ```
pub struct ProgressBar {
    inner: Option<IndicatifProgressBar>,
}

impl ProgressBar {
    /// Creates a new progress bar with the given length.
    ///
    /// The progress bar is automatically suppressed if:
    /// - stdout is not a TTY
    /// - Output format is JSON or quiet
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::ProgressBar;
    ///
    /// let pb = ProgressBar::new(100);
    /// ```
    #[must_use]
    pub fn new(len: u64) -> Self {
        Self::new_with_format(len, OutputFormat::Human)
    }

    /// Creates a new progress bar with explicit format control.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::{progress::ProgressBar, OutputFormat};
    ///
    /// let pb = ProgressBar::new_with_format(100, OutputFormat::Human);
    /// ```
    #[must_use]
    pub fn new_with_format(len: u64, format: OutputFormat) -> Self {
        if should_show_progress(format) {
            let pb = IndicatifProgressBar::new(len);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .map_or_else(|_| ProgressStyle::default_bar(), |s| s.progress_chars("#>-")),
            );
            pb.enable_steady_tick(Duration::from_millis(100));
            Self { inner: Some(pb) }
        } else {
            Self { inner: None }
        }
    }

    /// Sets the progress bar message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::ProgressBar;
    ///
    /// let pb = ProgressBar::new(100);
    /// pb.set_message("Downloading files...");
    /// ```
    pub fn set_message(&self, msg: impl Into<String>) {
        if let Some(ref pb) = self.inner {
            pb.set_message(msg.into());
        }
    }

    /// Sets the current position of the progress bar.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::ProgressBar;
    ///
    /// let pb = ProgressBar::new(100);
    /// pb.set_position(50);
    /// ```
    pub fn set_position(&self, pos: u64) {
        if let Some(ref pb) = self.inner {
            pb.set_position(pos);
        }
    }

    /// Increments the progress bar by the given amount.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::ProgressBar;
    ///
    /// let pb = ProgressBar::new(100);
    /// pb.inc(1);
    /// ```
    pub fn inc(&self, delta: u64) {
        if let Some(ref pb) = self.inner {
            pb.inc(delta);
        }
    }

    /// Sets the progress bar length.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::ProgressBar;
    ///
    /// let pb = ProgressBar::new(100);
    /// pb.set_length(200);
    /// ```
    pub fn set_length(&self, len: u64) {
        if let Some(ref pb) = self.inner {
            pb.set_length(len);
        }
    }

    /// Finishes the progress bar and clears it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::ProgressBar;
    ///
    /// let pb = ProgressBar::new(100);
    /// pb.finish();
    /// ```
    pub fn finish(&self) {
        if let Some(ref pb) = self.inner {
            pb.finish_and_clear();
        }
    }

    /// Finishes the progress bar with a message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::ProgressBar;
    ///
    /// let pb = ProgressBar::new(100);
    /// pb.finish_with_message("✓ Complete");
    /// ```
    pub fn finish_with_message(&self, msg: impl Into<String>) {
        if let Some(ref pb) = self.inner {
            pb.finish_with_message(msg.into());
        }
    }

    /// Finishes the progress bar and abandons it (shows as incomplete).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::ProgressBar;
    ///
    /// let pb = ProgressBar::new(100);
    /// pb.abandon();
    /// ```
    pub fn abandon(&self) {
        if let Some(ref pb) = self.inner {
            pb.abandon();
        }
    }

    /// Finishes the progress bar and abandons it with a message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::ProgressBar;
    ///
    /// let pb = ProgressBar::new(100);
    /// pb.abandon_with_message("✗ Failed");
    /// ```
    pub fn abandon_with_message(&self, msg: impl Into<String>) {
        if let Some(ref pb) = self.inner {
            pb.abandon_with_message(msg.into());
        }
    }

    /// Returns true if the progress bar is active (not suppressed).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::ProgressBar;
    ///
    /// let pb = ProgressBar::new(100);
    /// if pb.is_active() {
    ///     println!("Progress bar is active");
    /// }
    /// ```
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.inner.is_some()
    }
}

/// Spinner for indeterminate operations with unknown duration.
///
/// Automatically suppressed in non-TTY, JSON mode, or quiet mode.
/// When suppressed, all operations become no-ops.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::progress::Spinner;
///
/// let spinner = Spinner::new("Loading...");
/// // ... do work ...
/// spinner.finish_with_message("✓ Done");
/// ```
pub struct Spinner {
    inner: Option<IndicatifProgressBar>,
}

impl Spinner {
    /// Creates a new spinner with the given message.
    ///
    /// The spinner is automatically suppressed if:
    /// - stdout is not a TTY
    /// - Output format is JSON or quiet
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::Spinner;
    ///
    /// let spinner = Spinner::new("Loading packages...");
    /// ```
    #[must_use]
    pub fn new(msg: impl Into<String>) -> Self {
        Self::new_with_format(msg, OutputFormat::Human)
    }

    /// Creates a new spinner with explicit format control.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::{progress::Spinner, OutputFormat};
    ///
    /// let spinner = Spinner::new_with_format("Loading...", OutputFormat::Human);
    /// ```
    #[must_use]
    pub fn new_with_format(msg: impl Into<String>, format: OutputFormat) -> Self {
        if should_show_progress(format) {
            let pb = IndicatifProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner().template("{spinner:.green} {msg}").map_or_else(
                    |_| ProgressStyle::default_spinner(),
                    |s| s.tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
                ),
            );
            pb.set_message(msg.into());
            pb.enable_steady_tick(Duration::from_millis(80));
            Self { inner: Some(pb) }
        } else {
            Self { inner: None }
        }
    }

    /// Sets the spinner message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::Spinner;
    ///
    /// let spinner = Spinner::new("Loading...");
    /// spinner.set_message("Still loading...");
    /// ```
    pub fn set_message(&self, msg: impl Into<String>) {
        if let Some(ref pb) = self.inner {
            pb.set_message(msg.into());
        }
    }

    /// Manually ticks the spinner (usually not needed due to steady tick).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::Spinner;
    ///
    /// let spinner = Spinner::new("Loading...");
    /// spinner.tick();
    /// ```
    pub fn tick(&self) {
        if let Some(ref pb) = self.inner {
            pb.tick();
        }
    }

    /// Finishes the spinner and clears it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::Spinner;
    ///
    /// let spinner = Spinner::new("Loading...");
    /// spinner.finish();
    /// ```
    pub fn finish(&self) {
        if let Some(ref pb) = self.inner {
            pb.finish_and_clear();
        }
    }

    /// Finishes the spinner with a message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::Spinner;
    ///
    /// let spinner = Spinner::new("Loading...");
    /// spinner.finish_with_message("✓ Loaded 5 packages");
    /// ```
    pub fn finish_with_message(&self, msg: impl Into<String>) {
        if let Some(ref pb) = self.inner {
            pb.finish_with_message(msg.into());
        }
    }

    /// Finishes the spinner and abandons it (shows as incomplete).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::Spinner;
    ///
    /// let spinner = Spinner::new("Loading...");
    /// spinner.abandon();
    /// ```
    pub fn abandon(&self) {
        if let Some(ref pb) = self.inner {
            pb.abandon();
        }
    }

    /// Finishes the spinner and abandons it with a message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::Spinner;
    ///
    /// let spinner = Spinner::new("Loading...");
    /// spinner.abandon_with_message("✗ Failed to load");
    /// ```
    pub fn abandon_with_message(&self, msg: impl Into<String>) {
        if let Some(ref pb) = self.inner {
            pb.abandon_with_message(msg.into());
        }
    }

    /// Returns true if the spinner is active (not suppressed).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::Spinner;
    ///
    /// let spinner = Spinner::new("Loading...");
    /// if spinner.is_active() {
    ///     println!("Spinner is active");
    /// }
    /// ```
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.inner.is_some()
    }
}

/// Multi-progress manager for handling multiple progress indicators concurrently.
///
/// Automatically suppressed in non-TTY, JSON mode, or quiet mode.
/// When suppressed, all operations become no-ops.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::progress::MultiProgress;
///
/// let multi = MultiProgress::new();
/// let pb1 = multi.add_progress_bar(100);
/// let pb2 = multi.add_progress_bar(50);
///
/// pb1.set_message("Task 1");
/// pb2.set_message("Task 2");
/// ```
pub struct MultiProgress {
    inner: Option<IndicatifMultiProgress>,
}

impl MultiProgress {
    /// Creates a new multi-progress manager.
    ///
    /// The manager is automatically suppressed if:
    /// - stdout is not a TTY
    /// - Output format is JSON or quiet
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::MultiProgress;
    ///
    /// let multi = MultiProgress::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::new_with_format(OutputFormat::Human)
    }

    /// Creates a new multi-progress manager with explicit format control.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::{progress::MultiProgress, OutputFormat};
    ///
    /// let multi = MultiProgress::new_with_format(OutputFormat::Human);
    /// ```
    #[must_use]
    pub fn new_with_format(format: OutputFormat) -> Self {
        if should_show_progress(format) {
            let multi = IndicatifMultiProgress::new();
            Self { inner: Some(multi) }
        } else {
            Self { inner: None }
        }
    }

    /// Adds a progress bar to the multi-progress manager.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::MultiProgress;
    ///
    /// let multi = MultiProgress::new();
    /// let pb = multi.add_progress_bar(100);
    /// pb.set_message("Processing...");
    /// ```
    #[must_use]
    pub fn add_progress_bar(&self, len: u64) -> ProgressBar {
        if let Some(ref multi) = self.inner {
            let pb = multi.add(IndicatifProgressBar::new(len));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .map_or_else(|_| ProgressStyle::default_bar(), |s| s.progress_chars("#>-")),
            );
            pb.enable_steady_tick(Duration::from_millis(100));
            ProgressBar { inner: Some(pb) }
        } else {
            ProgressBar { inner: None }
        }
    }

    /// Adds a spinner to the multi-progress manager.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::MultiProgress;
    ///
    /// let multi = MultiProgress::new();
    /// let spinner = multi.add_spinner("Loading...");
    /// ```
    #[must_use]
    pub fn add_spinner(&self, msg: impl Into<String>) -> Spinner {
        if let Some(ref multi) = self.inner {
            let pb = multi.add(IndicatifProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::default_spinner().template("{spinner:.green} {msg}").map_or_else(
                    |_| ProgressStyle::default_spinner(),
                    |s| s.tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
                ),
            );
            pb.set_message(msg.into());
            pb.enable_steady_tick(Duration::from_millis(80));
            Spinner { inner: Some(pb) }
        } else {
            Spinner { inner: None }
        }
    }

    /// Clears all progress indicators.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::MultiProgress;
    ///
    /// let multi = MultiProgress::new();
    /// let pb = multi.add_progress_bar(100);
    /// multi.clear();
    /// ```
    pub fn clear(&self) {
        if let Some(ref multi) = self.inner {
            multi.clear().ok();
        }
    }

    /// Returns true if the multi-progress manager is active (not suppressed).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::progress::MultiProgress;
    ///
    /// let multi = MultiProgress::new();
    /// if multi.is_active() {
    ///     println!("Multi-progress is active");
    /// }
    /// ```
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.inner.is_some()
    }
}

impl Default for MultiProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Determines if progress indicators should be shown.
///
/// Progress is suppressed when:
/// - Output format is JSON or JsonCompact (to avoid mixing progress with structured output)
/// - Output format is Quiet (no output at all)
/// - stdout is not a TTY (e.g., piped to file or another command)
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::{OutputFormat, progress::should_show_progress};
///
/// assert!(!should_show_progress(OutputFormat::Json));
/// assert!(!should_show_progress(OutputFormat::Quiet));
/// ```
#[must_use]
pub fn should_show_progress(format: OutputFormat) -> bool {
    // Never show progress in JSON or Quiet modes
    if format.is_json() || format.is_quiet() {
        return false;
    }

    // Only show progress if stdout is a TTY
    Term::stdout().is_term()
}
