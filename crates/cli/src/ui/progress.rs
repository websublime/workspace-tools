//! Progress indicators for CLI operations.
//!
//! Provides components for showing operation progress, spinners,
//! and activity indicators.

use indicatif::{MultiProgress, ProgressBar, ProgressIterator, ProgressStyle};
use std::time::Duration;

/// Create a spinner with custom message
pub fn spinner<S: Into<String>>(message: S) -> ProgressBar {
    let pb = ProgressBar::new_spinner();

    // Use Unicode spinners if supported, otherwise ASCII
    let spinner_chars = &["-", "\\", "|", "/"];

    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(spinner_chars)
            .template("{spinner:.cyan} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );

    pb.set_message(message.into());
    pb.enable_steady_tick(Duration::from_millis(100));

    pb
}

/// Create a determinate progress bar
pub fn progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);

    // Use Unicode block characters if supported, otherwise ASCII
    let progress_chars = "=>-";

    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars(progress_chars)
    );

    pb
}

/// Create a download progress bar
pub fn download_progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);

    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("=>-")
    );

    pb
}

/// Create a multi-progress display for showing multiple progress indicators
pub fn multi_progress() -> MultiProgress {
    MultiProgress::new()
}

/// Create a progress bar with a custom template
pub fn custom_progress_bar(total: u64, template: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);

    pb.set_style(
        ProgressStyle::default_bar()
            .template(template)
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("=>-"),
    );

    pb
}

/// Wrap an iterator with a progress bar
///
/// Note: If you don't know the total items, provide a reasonable estimate
/// or collect your iterator into a collection first.
pub fn progress_iterator<It>(
    iterator: It,
    total: u64,
    message: &str,
) -> impl Iterator<Item = It::Item>
where
    It: Iterator,
{
    let pb = progress_bar(total);
    pb.set_message(message.to_string());

    iterator.progress_with(pb)
}

/// Create a progress iterator from a collection whose size we can determine
pub fn progress_collection<C, T>(collection: C, message: &str) -> impl Iterator<Item = T>
where
    C: IntoIterator<Item = T>,
    C: Clone + ExactSizeIterator,
{
    let total = collection.len() as u64;
    let pb = progress_bar(total);
    pb.set_message(message.to_string());

    collection.into_iter().progress_with(pb)
}

/// Convenience function for creating a progress bar for a vector
pub fn progress_vec<T>(vec: Vec<T>, message: &str) -> impl Iterator<Item = T> {
    let total = vec.len() as u64;
    let pb = progress_bar(total);
    pb.set_message(message.to_string());

    vec.into_iter().progress_with(pb)
}

/// Helper for progress operations with stages
#[allow(dead_code)]
pub struct StagedProgress {
    multi: MultiProgress,
    overall: ProgressBar,
    current: Option<ProgressBar>,
    total_stages: u64,
    completed_stages: u64,
}

impl StagedProgress {
    /// Create a new staged progress tracker
    pub fn new(total_stages: u64, title: &str) -> Self {
        let multi = MultiProgress::new();

        // Create the overall progress bar
        let overall = multi.add(ProgressBar::new(total_stages));
        overall.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "{{spinner:.green}} {} [{{bar:40.cyan/blue}}] {{pos}}/{{len}} stages",
                    title
                ))
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("=>-"),
        );

        StagedProgress { multi, overall, current: None, total_stages, completed_stages: 0 }
    }

    /// Start a new stage with the given name and total items
    pub fn start_stage(&mut self, name: &str, total: u64) -> &mut Self {
        // Complete previous stage if it exists
        if let Some(current) = self.current.take() {
            current.finish_and_clear();
        }

        // Create a new progress bar for this stage
        let stage_pb = self.multi.add(ProgressBar::new(total));
        stage_pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "{{spinner:.blue}} Stage {{pos}}/{{len}}: {} [{{bar:30.cyan/blue}}]",
                    name
                ))
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("=>-"),
        );
        stage_pb.set_position(0);

        self.current = Some(stage_pb);
        self
    }

    /// Increment the current stage progress
    pub fn inc(&mut self, delta: u64) -> &mut Self {
        if let Some(current) = &self.current {
            current.inc(delta);
        }
        self
    }

    /// Set message for the current stage
    pub fn set_stage_message(&mut self, msg: &str) -> &mut Self {
        if let Some(current) = &self.current {
            current.set_message(msg.to_string());
        }
        self
    }

    /// Complete the current stage and increment the overall progress
    pub fn complete_stage(&mut self) -> &mut Self {
        if let Some(current) = self.current.take() {
            current.finish_and_clear();
        }

        self.completed_stages += 1;
        self.overall.inc(1);
        self
    }

    /// Complete all progress
    pub fn finish(self) {
        if let Some(current) = self.current {
            current.finish_and_clear();
        }

        self.overall.finish_with_message("All stages complete");
    }

    /// Get the overall progress bar reference
    pub fn overall(&self) -> &ProgressBar {
        &self.overall
    }

    /// Get the current stage progress bar reference
    pub fn current(&self) -> Option<&ProgressBar> {
        self.current.as_ref()
    }
}
