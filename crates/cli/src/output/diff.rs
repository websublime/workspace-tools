//! Diff visualization for version changes and file modifications.
//!
//! This module provides diff visualization capabilities for displaying
//! changes in versions, files, and configurations with color coding
//! and clear formatting.
//!
//! # What
//!
//! Provides:
//! - Version diff visualization (before/after comparison)
//! - File change diff display
//! - Dependency update diffs
//! - Configuration change visualization
//! - Color-coded output for additions, deletions, and modifications
//!
//! # How
//!
//! Uses:
//! - Unified diff format for file changes
//! - Side-by-side comparison for version changes
//! - Color coding (green for additions, red for deletions, yellow for modifications)
//! - Box-drawing characters for structure
//! - Console crate for styled output
//!
//! # Why
//!
//! Diffs help users:
//! - Understand what will change before applying updates
//! - Review version bumps and their impact
//! - Identify file modifications
//! - Make informed decisions about changes
//!
//! # Examples
//!
//! Display a version diff:
//!
//! ```rust
//! use sublime_cli_tools::output::diff::{VersionDiff, DiffRenderer};
//!
//! let diff = VersionDiff::new("my-package", "1.0.0", "1.1.0");
//! let renderer = DiffRenderer::new(false);
//! let output = renderer.render_version_diff(&diff);
//! println!("{}", output);
//! ```
//!
//! Display file changes:
//!
//! ```rust
//! use sublime_cli_tools::output::diff::{FileDiff, DiffType, DiffRenderer};
//!
//! let diff = FileDiff::new("package.json", DiffType::Modified)
//!     .add_line_removed(r#"  "version": "1.0.0","#)
//!     .add_line_added(r#"  "version": "1.1.0","#);
//!
//! let renderer = DiffRenderer::new(false);
//! let output = renderer.render_file_diff(&diff);
//! println!("{}", output);
//! ```

use super::style::Style;
use console::Color;
use std::fmt;

/// Type of diff operation.
///
/// Represents the kind of change being shown in a diff.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::diff::DiffType;
///
/// let added = DiffType::Added;
/// let modified = DiffType::Modified;
/// let deleted = DiffType::Deleted;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffType {
    /// Content or file was added
    Added,

    /// Content or file was modified
    Modified,

    /// Content or file was deleted
    Deleted,

    /// No change (for context)
    Unchanged,
}

impl DiffType {
    /// Returns the color associated with this diff type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::DiffType;
    /// use console::Color;
    ///
    /// assert_eq!(DiffType::Added.color(), Color::Green);
    /// assert_eq!(DiffType::Deleted.color(), Color::Red);
    /// assert_eq!(DiffType::Modified.color(), Color::Yellow);
    /// ```
    #[must_use]
    pub const fn color(&self) -> Color {
        match self {
            Self::Added => Color::Green,
            Self::Modified => Color::Yellow,
            Self::Deleted => Color::Red,
            Self::Unchanged => Color::White,
        }
    }

    /// Returns the symbol prefix for this diff type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::DiffType;
    ///
    /// assert_eq!(DiffType::Added.symbol(), "+");
    /// assert_eq!(DiffType::Deleted.symbol(), "-");
    /// assert_eq!(DiffType::Modified.symbol(), "~");
    /// assert_eq!(DiffType::Unchanged.symbol(), " ");
    /// ```
    #[must_use]
    pub const fn symbol(&self) -> &'static str {
        match self {
            Self::Added => "+",
            Self::Deleted => "-",
            Self::Modified => "~",
            Self::Unchanged => " ",
        }
    }

    /// Returns a human-readable label for this diff type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::DiffType;
    ///
    /// assert_eq!(DiffType::Added.label(), "added");
    /// assert_eq!(DiffType::Modified.label(), "modified");
    /// ```
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Modified => "modified",
            Self::Deleted => "deleted",
            Self::Unchanged => "unchanged",
        }
    }
}

impl fmt::Display for DiffType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Represents a single line in a diff.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::diff::{DiffLine, DiffType};
///
/// let added_line = DiffLine::new(DiffType::Added, "  \"version\": \"1.1.0\",");
/// let deleted_line = DiffLine::new(DiffType::Deleted, "  \"version\": \"1.0.0\",");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    /// The type of change for this line
    pub diff_type: DiffType,

    /// The content of the line
    pub content: String,

    /// Optional line number in the original file
    pub line_number: Option<usize>,
}

impl DiffLine {
    /// Creates a new diff line.
    ///
    /// # Arguments
    ///
    /// * `diff_type` - The type of change
    /// * `content` - The line content
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{DiffLine, DiffType};
    ///
    /// let line = DiffLine::new(DiffType::Added, "new content");
    /// ```
    #[must_use]
    pub fn new(diff_type: DiffType, content: impl Into<String>) -> Self {
        Self { diff_type, content: content.into(), line_number: None }
    }

    /// Creates a new diff line with a line number.
    ///
    /// # Arguments
    ///
    /// * `diff_type` - The type of change
    /// * `content` - The line content
    /// * `line_number` - The line number in the original file
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{DiffLine, DiffType};
    ///
    /// let line = DiffLine::with_line_number(DiffType::Modified, "changed", 42);
    /// ```
    #[must_use]
    pub fn with_line_number(
        diff_type: DiffType,
        content: impl Into<String>,
        line_number: usize,
    ) -> Self {
        Self { diff_type, content: content.into(), line_number: Some(line_number) }
    }

    /// Renders this line as a formatted string with optional colors.
    ///
    /// # Arguments
    ///
    /// * `no_color` - Whether to disable color output
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{DiffLine, DiffType};
    ///
    /// let line = DiffLine::new(DiffType::Added, "new line");
    /// let rendered = line.render(false);
    /// assert!(rendered.contains("+"));
    /// ```
    #[must_use]
    pub fn render(&self, no_color: bool) -> String {
        let symbol = self.diff_type.symbol();
        let content = &self.content;

        if no_color {
            if let Some(line_num) = self.line_number {
                format!("{line_num:4} {symbol} {content}")
            } else {
                format!("{symbol} {content}")
            }
        } else {
            let color = self.diff_type.color();
            let styled_symbol = Style::color(color, symbol);
            let styled_content = if self.diff_type == DiffType::Unchanged {
                Style::dim(content)
            } else {
                Style::color(color, content)
            };

            if let Some(line_num) = self.line_number {
                let line_num_str = Style::dim(&line_num.to_string());
                format!("{line_num_str:4} {styled_symbol} {styled_content}")
            } else {
                format!("{styled_symbol} {styled_content}")
            }
        }
    }
}

/// Represents a version change diff.
///
/// Shows the before and after versions for a package.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::diff::VersionDiff;
///
/// let diff = VersionDiff::new("@org/package", "1.0.0", "2.0.0")
///     .with_reason("Major version bump due to breaking changes");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionDiff {
    /// Package name
    pub package: String,

    /// Current/old version
    pub from_version: String,

    /// Next/new version
    pub to_version: String,

    /// Optional reason for the change
    pub reason: Option<String>,

    /// Whether this change will be applied
    pub will_change: bool,
}

impl VersionDiff {
    /// Creates a new version diff.
    ///
    /// # Arguments
    ///
    /// * `package` - The package name
    /// * `from_version` - The current version
    /// * `to_version` - The new version
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::VersionDiff;
    ///
    /// let diff = VersionDiff::new("my-package", "1.0.0", "1.1.0");
    /// ```
    #[must_use]
    pub fn new(
        package: impl Into<String>,
        from_version: impl Into<String>,
        to_version: impl Into<String>,
    ) -> Self {
        Self {
            package: package.into(),
            from_version: from_version.into(),
            to_version: to_version.into(),
            reason: None,
            will_change: true,
        }
    }

    /// Sets the reason for the version change.
    ///
    /// # Arguments
    ///
    /// * `reason` - The reason for the change
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::VersionDiff;
    ///
    /// let diff = VersionDiff::new("pkg", "1.0.0", "2.0.0")
    ///     .with_reason("Breaking changes");
    /// ```
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Sets whether this change will be applied.
    ///
    /// # Arguments
    ///
    /// * `will_change` - Whether the change will be applied
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::VersionDiff;
    ///
    /// let diff = VersionDiff::new("pkg", "1.0.0", "1.0.0")
    ///     .with_will_change(false);
    /// ```
    #[must_use]
    pub fn with_will_change(mut self, will_change: bool) -> Self {
        self.will_change = will_change;
        self
    }

    /// Returns whether this is actually a change (versions are different).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::VersionDiff;
    ///
    /// let diff = VersionDiff::new("pkg", "1.0.0", "1.1.0");
    /// assert!(diff.has_change());
    ///
    /// let no_change = VersionDiff::new("pkg", "1.0.0", "1.0.0");
    /// assert!(!no_change.has_change());
    /// ```
    #[must_use]
    pub fn has_change(&self) -> bool {
        self.from_version != self.to_version
    }
}

/// Represents a file diff.
///
/// Shows changes in a specific file with added, removed, and unchanged lines.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::diff::{FileDiff, DiffType};
///
/// let diff = FileDiff::new("package.json", DiffType::Modified)
///     .add_line_removed(r#"  "version": "1.0.0","#)
///     .add_line_added(r#"  "version": "1.1.0","#);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiff {
    /// Path to the file
    pub path: String,

    /// Type of change to the file
    pub file_type: DiffType,

    /// Lines in the diff
    pub lines: Vec<DiffLine>,

    /// Optional context about the change
    pub context: Option<String>,
}

impl FileDiff {
    /// Creates a new file diff.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path
    /// * `file_type` - The type of change to the file
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{FileDiff, DiffType};
    ///
    /// let diff = FileDiff::new("README.md", DiffType::Modified);
    /// ```
    #[must_use]
    pub fn new(path: impl Into<String>, file_type: DiffType) -> Self {
        Self { path: path.into(), file_type, lines: Vec::new(), context: None }
    }

    /// Adds a line to the diff.
    ///
    /// # Arguments
    ///
    /// * `line` - The diff line to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{FileDiff, DiffLine, DiffType};
    ///
    /// let mut diff = FileDiff::new("test.txt", DiffType::Modified);
    /// diff.add_line(DiffLine::new(DiffType::Added, "new content"));
    /// ```
    pub fn add_line(&mut self, line: DiffLine) {
        self.lines.push(line);
    }

    /// Adds an added line to the diff (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `content` - The line content
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{FileDiff, DiffType};
    ///
    /// let diff = FileDiff::new("file.txt", DiffType::Modified)
    ///     .add_line_added("new line");
    /// ```
    #[must_use]
    pub fn add_line_added(mut self, content: impl Into<String>) -> Self {
        self.lines.push(DiffLine::new(DiffType::Added, content));
        self
    }

    /// Adds a removed line to the diff (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `content` - The line content
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{FileDiff, DiffType};
    ///
    /// let diff = FileDiff::new("file.txt", DiffType::Modified)
    ///     .add_line_removed("old line");
    /// ```
    #[must_use]
    pub fn add_line_removed(mut self, content: impl Into<String>) -> Self {
        self.lines.push(DiffLine::new(DiffType::Deleted, content));
        self
    }

    /// Adds a modified line to the diff (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `content` - The line content
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{FileDiff, DiffType};
    ///
    /// let diff = FileDiff::new("file.txt", DiffType::Modified)
    ///     .add_line_modified("changed line");
    /// ```
    #[must_use]
    pub fn add_line_modified(mut self, content: impl Into<String>) -> Self {
        self.lines.push(DiffLine::new(DiffType::Modified, content));
        self
    }

    /// Adds an unchanged line for context (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `content` - The line content
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{FileDiff, DiffType};
    ///
    /// let diff = FileDiff::new("file.txt", DiffType::Modified)
    ///     .add_line_context("unchanged line");
    /// ```
    #[must_use]
    pub fn add_line_context(mut self, content: impl Into<String>) -> Self {
        self.lines.push(DiffLine::new(DiffType::Unchanged, content));
        self
    }

    /// Sets the context description for this diff.
    ///
    /// # Arguments
    ///
    /// * `context` - The context description
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{FileDiff, DiffType};
    ///
    /// let diff = FileDiff::new("package.json", DiffType::Modified)
    ///     .with_context("Version bump");
    /// ```
    #[must_use]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Returns the number of added lines.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{FileDiff, DiffType};
    ///
    /// let diff = FileDiff::new("file.txt", DiffType::Modified)
    ///     .add_line_added("line 1")
    ///     .add_line_added("line 2");
    ///
    /// assert_eq!(diff.added_count(), 2);
    /// ```
    #[must_use]
    pub fn added_count(&self) -> usize {
        self.lines.iter().filter(|l| l.diff_type == DiffType::Added).count()
    }

    /// Returns the number of removed lines.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{FileDiff, DiffType};
    ///
    /// let diff = FileDiff::new("file.txt", DiffType::Modified)
    ///     .add_line_removed("line 1");
    ///
    /// assert_eq!(diff.removed_count(), 1);
    /// ```
    #[must_use]
    pub fn removed_count(&self) -> usize {
        self.lines.iter().filter(|l| l.diff_type == DiffType::Deleted).count()
    }

    /// Returns the number of modified lines.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{FileDiff, DiffType};
    ///
    /// let diff = FileDiff::new("file.txt", DiffType::Modified)
    ///     .add_line_modified("changed");
    ///
    /// assert_eq!(diff.modified_count(), 1);
    /// ```
    #[must_use]
    pub fn modified_count(&self) -> usize {
        self.lines.iter().filter(|l| l.diff_type == DiffType::Modified).count()
    }
}

/// Dependency diff for showing dependency version changes.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::diff::DependencyDiff;
///
/// let diff = DependencyDiff::new("lodash", "^4.17.20", "^4.17.21")
///     .with_package_context("my-package");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyDiff {
    /// Dependency name
    pub name: String,

    /// Current version spec
    pub from_version: String,

    /// New version spec
    pub to_version: String,

    /// Type of dependency (dependencies, devDependencies, etc.)
    pub dep_type: Option<String>,

    /// Package that contains this dependency
    pub package_context: Option<String>,
}

impl DependencyDiff {
    /// Creates a new dependency diff.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name
    /// * `from_version` - The current version
    /// * `to_version` - The new version
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::DependencyDiff;
    ///
    /// let diff = DependencyDiff::new("react", "^17.0.0", "^18.0.0");
    /// ```
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        from_version: impl Into<String>,
        to_version: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            from_version: from_version.into(),
            to_version: to_version.into(),
            dep_type: None,
            package_context: None,
        }
    }

    /// Sets the dependency type.
    ///
    /// # Arguments
    ///
    /// * `dep_type` - The dependency type (e.g., "dependencies", "devDependencies")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::DependencyDiff;
    ///
    /// let diff = DependencyDiff::new("jest", "^27.0.0", "^28.0.0")
    ///     .with_dep_type("devDependencies");
    /// ```
    #[must_use]
    pub fn with_dep_type(mut self, dep_type: impl Into<String>) -> Self {
        self.dep_type = Some(dep_type.into());
        self
    }

    /// Sets the package context.
    ///
    /// # Arguments
    ///
    /// * `package` - The package that contains this dependency
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::DependencyDiff;
    ///
    /// let diff = DependencyDiff::new("typescript", "^4.0.0", "^5.0.0")
    ///     .with_package_context("@org/frontend");
    /// ```
    #[must_use]
    pub fn with_package_context(mut self, package: impl Into<String>) -> Self {
        self.package_context = Some(package.into());
        self
    }

    /// Returns whether this is actually a change (versions are different).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::DependencyDiff;
    ///
    /// let diff = DependencyDiff::new("pkg", "1.0.0", "2.0.0");
    /// assert!(diff.has_change());
    ///
    /// let no_change = DependencyDiff::new("pkg", "1.0.0", "1.0.0");
    /// assert!(!no_change.has_change());
    /// ```
    #[must_use]
    pub fn has_change(&self) -> bool {
        self.from_version != self.to_version
    }
}

/// Renders diffs with appropriate formatting and colors.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::diff::{DiffRenderer, VersionDiff};
///
/// let renderer = DiffRenderer::new(false);
/// let diff = VersionDiff::new("my-package", "1.0.0", "2.0.0");
/// let output = renderer.render_version_diff(&diff);
/// println!("{}", output);
/// ```
#[derive(Debug, Clone)]
pub struct DiffRenderer {
    /// Whether to disable color output
    no_color: bool,

    /// Whether to show line numbers
    show_line_numbers: bool,

    /// Number of context lines to show
    context_lines: usize,
}

impl DiffRenderer {
    /// Creates a new diff renderer.
    ///
    /// # Arguments
    ///
    /// * `no_color` - Whether to disable color output
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::DiffRenderer;
    ///
    /// let renderer = DiffRenderer::new(false);
    /// ```
    #[must_use]
    pub fn new(no_color: bool) -> Self {
        Self { no_color, show_line_numbers: false, context_lines: 3 }
    }

    /// Sets whether to show line numbers.
    ///
    /// # Arguments
    ///
    /// * `show` - Whether to show line numbers
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::DiffRenderer;
    ///
    /// let renderer = DiffRenderer::new(false)
    ///     .with_line_numbers(true);
    /// ```
    #[must_use]
    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Sets the number of context lines to show.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of context lines
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::DiffRenderer;
    ///
    /// let renderer = DiffRenderer::new(false)
    ///     .with_context_lines(5);
    /// ```
    #[must_use]
    pub fn with_context_lines(mut self, count: usize) -> Self {
        self.context_lines = count;
        self
    }

    /// Renders a version diff.
    ///
    /// # Arguments
    ///
    /// * `diff` - The version diff to render
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{DiffRenderer, VersionDiff};
    ///
    /// let renderer = DiffRenderer::new(false);
    /// let diff = VersionDiff::new("pkg", "1.0.0", "2.0.0");
    /// let output = renderer.render_version_diff(&diff);
    /// ```
    #[must_use]
    pub fn render_version_diff(&self, diff: &VersionDiff) -> String {
        use std::fmt::Write;
        let mut output = String::new();

        // Package name header
        if self.no_color {
            let _ = writeln!(output, "{}", diff.package);
        } else {
            let _ = writeln!(output, "{}", Style::bold(&diff.package));
        }

        // Version change
        if diff.has_change() {
            let from_line = format!("- {}", diff.from_version);
            let to_line = format!("+ {}", diff.to_version);

            if self.no_color {
                let _ = writeln!(output, "  {from_line}");
                let _ = writeln!(output, "  {to_line}");
            } else {
                let _ = writeln!(output, "  {}", Style::color(Color::Red, &from_line));
                let _ = writeln!(output, "  {}", Style::color(Color::Green, &to_line));
            }
        } else if !diff.will_change {
            let unchanged = format!("  {} (unchanged)", diff.from_version);
            if self.no_color {
                let _ = writeln!(output, "{unchanged}");
            } else {
                let _ = writeln!(output, "{}", Style::dim(&unchanged));
            }
        }

        // Reason if provided
        if let Some(reason) = &diff.reason {
            if self.no_color {
                let _ = writeln!(output, "  Reason: {reason}");
            } else {
                let _ = writeln!(output, "  {}: {}", Style::dim("Reason"), Style::italic(reason));
            }
        }

        output
    }

    /// Renders a file diff.
    ///
    /// # Arguments
    ///
    /// * `diff` - The file diff to render
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{DiffRenderer, FileDiff, DiffType};
    ///
    /// let renderer = DiffRenderer::new(false);
    /// let diff = FileDiff::new("file.txt", DiffType::Modified)
    ///     .add_line_removed("old")
    ///     .add_line_added("new");
    /// let output = renderer.render_file_diff(&diff);
    /// ```
    #[must_use]
    pub fn render_file_diff(&self, diff: &FileDiff) -> String {
        use std::fmt::Write;
        let mut output = String::new();

        // File header
        if self.no_color {
            let _ = writeln!(output, "--- {}", diff.path);
        } else {
            let _ = writeln!(
                output,
                "{} {}",
                Style::color(Color::Cyan, "---"),
                Style::bold(&diff.path)
            );
        }

        // Context if provided
        if let Some(context) = &diff.context {
            if self.no_color {
                let _ = writeln!(output, "@@ {context} @@");
            } else {
                let ctx_str = format!("@@ {context} @@");
                let _ = writeln!(output, "{}", Style::color(Color::Cyan, &ctx_str));
            }
        }

        // Render lines
        for line in &diff.lines {
            output.push_str(&line.render(self.no_color));
            output.push('\n');
        }

        // Stats
        let stats = self.render_diff_stats(diff);
        output.push_str(&stats);

        output
    }

    /// Renders diff statistics.
    fn render_diff_stats(&self, diff: &FileDiff) -> String {
        let added = diff.added_count();
        let removed = diff.removed_count();
        let modified = diff.modified_count();

        if added == 0 && removed == 0 && modified == 0 {
            return String::new();
        }

        let mut parts = Vec::new();
        if added > 0 {
            parts.push(if self.no_color {
                format!("+{added}")
            } else {
                Style::color(Color::Green, &format!("+{added}"))
            });
        }
        if removed > 0 {
            parts.push(if self.no_color {
                format!("-{removed}")
            } else {
                Style::color(Color::Red, &format!("-{removed}"))
            });
        }
        if modified > 0 {
            parts.push(if self.no_color {
                format!("~{modified}")
            } else {
                Style::color(Color::Yellow, &format!("~{modified}"))
            });
        }

        let joined = parts.join(" ");
        format!("\n{joined}\n")
    }

    /// Renders a dependency diff.
    ///
    /// # Arguments
    ///
    /// * `diff` - The dependency diff to render
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{DiffRenderer, DependencyDiff};
    ///
    /// let renderer = DiffRenderer::new(false);
    /// let diff = DependencyDiff::new("lodash", "^4.0.0", "^5.0.0");
    /// let output = renderer.render_dependency_diff(&diff);
    /// ```
    #[must_use]
    pub fn render_dependency_diff(&self, diff: &DependencyDiff) -> String {
        use std::fmt::Write;
        let mut output = String::new();

        // Dependency name
        let name_str = if let Some(pkg) = &diff.package_context {
            format!("{} (in {pkg})", diff.name)
        } else {
            diff.name.clone()
        };

        if self.no_color {
            let _ = writeln!(output, "{name_str}");
        } else {
            let _ = writeln!(output, "{}", Style::bold(&name_str));
        }

        // Dependency type
        if let Some(dep_type) = &diff.dep_type {
            if self.no_color {
                let _ = writeln!(output, "  Type: {dep_type}");
            } else {
                let type_str = format!("Type: {dep_type}");
                let _ = writeln!(output, "  {}", Style::dim(&type_str));
            }
        }

        // Version change
        if diff.has_change() {
            let from_line = format!("- {}", diff.from_version);
            let to_line = format!("+ {}", diff.to_version);

            if self.no_color {
                let _ = writeln!(output, "  {from_line}");
                let _ = writeln!(output, "  {to_line}");
            } else {
                let _ = writeln!(output, "  {}", Style::color(Color::Red, &from_line));
                let _ = writeln!(output, "  {}", Style::color(Color::Green, &to_line));
            }
        }

        output
    }

    /// Renders multiple version diffs as a summary.
    ///
    /// # Arguments
    ///
    /// * `diffs` - The version diffs to render
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::diff::{DiffRenderer, VersionDiff};
    ///
    /// let renderer = DiffRenderer::new(false);
    /// let diffs = vec![
    ///     VersionDiff::new("pkg1", "1.0.0", "2.0.0"),
    ///     VersionDiff::new("pkg2", "3.0.0", "3.1.0"),
    /// ];
    /// let output = renderer.render_version_summary(&diffs);
    /// ```
    #[must_use]
    pub fn render_version_summary(&self, diffs: &[VersionDiff]) -> String {
        use std::fmt::Write;
        let mut output = String::new();

        // Header
        if self.no_color {
            output.push_str("Version Changes:\n\n");
        } else {
            let _ = writeln!(output, "{}\n", Style::bold("Version Changes:"));
        }

        // Count changes
        let changes = diffs.iter().filter(|d| d.has_change()).count();
        let unchanged = diffs.len() - changes;

        // Render each diff
        for (i, diff) in diffs.iter().enumerate() {
            output.push_str(&self.render_version_diff(diff));
            if i < diffs.len() - 1 {
                output.push('\n');
            }
        }

        // Summary
        let total = diffs.len();
        if self.no_color {
            let _ = writeln!(
                output,
                "\nTotal: {total} packages, {changes} changed, {unchanged} unchanged"
            );
        } else {
            let _ = writeln!(
                output,
                "\n{}: {total} packages, {} changed, {} unchanged",
                Style::bold("Total"),
                Style::color(Color::Green, &changes.to_string()),
                Style::dim(&unchanged.to_string())
            );
        }

        output
    }
}

impl Default for DiffRenderer {
    fn default() -> Self {
        Self::new(false)
    }
}

/// Creates a version diff from package information.
///
/// # Arguments
///
/// * `package` - Package name
/// * `from_version` - Current version
/// * `to_version` - New version
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::diff::version_diff;
///
/// let diff = version_diff("my-package", "1.0.0", "2.0.0");
/// ```
#[must_use]
pub fn version_diff(
    package: impl Into<String>,
    from_version: impl Into<String>,
    to_version: impl Into<String>,
) -> VersionDiff {
    VersionDiff::new(package, from_version, to_version)
}

/// Creates a file diff for a modified file.
///
/// # Arguments
///
/// * `path` - File path
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::diff::file_diff_modified;
///
/// let diff = file_diff_modified("package.json");
/// ```
#[must_use]
pub fn file_diff_modified(path: impl Into<String>) -> FileDiff {
    FileDiff::new(path, DiffType::Modified)
}

/// Creates a file diff for an added file.
///
/// # Arguments
///
/// * `path` - File path
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::diff::file_diff_added;
///
/// let diff = file_diff_added("new-file.js");
/// ```
#[must_use]
pub fn file_diff_added(path: impl Into<String>) -> FileDiff {
    FileDiff::new(path, DiffType::Added)
}

/// Creates a file diff for a deleted file.
///
/// # Arguments
///
/// * `path` - File path
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::diff::file_diff_deleted;
///
/// let diff = file_diff_deleted("old-file.js");
/// ```
#[must_use]
pub fn file_diff_deleted(path: impl Into<String>) -> FileDiff {
    FileDiff::new(path, DiffType::Deleted)
}

/// Creates a dependency diff.
///
/// # Arguments
///
/// * `name` - Dependency name
/// * `from_version` - Current version
/// * `to_version` - New version
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::diff::dependency_diff;
///
/// let diff = dependency_diff("lodash", "^4.0.0", "^5.0.0");
/// ```
#[must_use]
pub fn dependency_diff(
    name: impl Into<String>,
    from_version: impl Into<String>,
    to_version: impl Into<String>,
) -> DependencyDiff {
    DependencyDiff::new(name, from_version, to_version)
}
