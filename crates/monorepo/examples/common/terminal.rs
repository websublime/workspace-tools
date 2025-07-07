//! Beautiful terminal output utilities for examples
//!
//! Provides consistent styling, progress indicators, tables, and interactive prompts
//! for demonstration purposes in monorepo workflow examples.

#![allow(dead_code)]

use console::{style, Emoji, Term};
use indicatif::{ProgressBar, ProgressStyle, ProgressState};
use tabled::{Table, Tabled, settings::{Style, Alignment, object::Rows}};
use std::time::Duration;
use std::fmt::Write;

/// Emojis for consistent visual feedback
pub struct Icons;

impl Icons {
    pub const ROCKET: Emoji<'_, '_> = Emoji("🚀 ", "");
    pub const PACKAGE: Emoji<'_, '_> = Emoji("📦 ", "");
    pub const BRANCH: Emoji<'_, '_> = Emoji("🌿 ", "");
    pub const COMMIT: Emoji<'_, '_> = Emoji("📝 ", "");
    pub const HOOK: Emoji<'_, '_> = Emoji("🪝 ", "");
    pub const TASK: Emoji<'_, '_> = Emoji("⚡ ", "");
    pub const SUCCESS: Emoji<'_, '_> = Emoji("✅ ", "");
    pub const WARNING: Emoji<'_, '_> = Emoji("⚠️ ", "");
    pub const ERROR: Emoji<'_, '_> = Emoji("❌ ", "");
    pub const INFO: Emoji<'_, '_> = Emoji("ℹ️ ", "");
    pub const SEARCH: Emoji<'_, '_> = Emoji("🔍 ", "");
    pub const GRAPH: Emoji<'_, '_> = Emoji("🕸️ ", "");
    pub const UPGRADE: Emoji<'_, '_> = Emoji("⬆️ ", "");
    pub const CHANGELOG: Emoji<'_, '_> = Emoji("📄 ", "");
    pub const VERSION: Emoji<'_, '_> = Emoji("🏷️ ", "");
    pub const MERGE: Emoji<'_, '_> = Emoji("🔀 ", "");
    pub const DEPLOY: Emoji<'_, '_> = Emoji("🚀 ", "");
    pub const TEST: Emoji<'_, '_> = Emoji("🧪 ", "");
    pub const BUILD: Emoji<'_, '_> = Emoji("🏗️ ", "");
    pub const ROBOT: Emoji<'_, '_> = Emoji("🤖 ", "");
    pub const QUESTION: Emoji<'_, '_> = Emoji("❓ ", "");
    pub const CLEAN: Emoji<'_, '_> = Emoji("🧹 ", "");
    pub const REPORT: Emoji<'_, '_> = Emoji("📊 ", "");
    pub const LOCK: Emoji<'_, '_> = Emoji("🔒 ", "");
    pub const SNAPSHOT: Emoji<'_, '_> = Emoji("📸 ", "");
}

/// Terminal output manager for consistent styling
pub struct TerminalOutput {
    term: Term,
}

impl TerminalOutput {
    /// Create a new terminal output manager
    pub fn new() -> Self {
        Self {
            term: Term::stdout(),
        }
    }

    /// Print a phase header with consistent formatting
    pub fn phase_header(&self, phase: &str, description: &str) -> std::io::Result<()> {
        self.term.write_line("")?;
        self.term.write_line(&format!("{}{}", 
            style(format!("{} {}", Icons::ROCKET, phase)).bold().cyan(),
            style(format!(" {}", description)).dim()
        ))?;
        self.term.write_line(&"=".repeat(60))?;
        Ok(())
    }

    /// Print a step with icon and description
    pub fn step(&self, icon: Emoji, description: &str) -> std::io::Result<()> {
        self.term.write_line(&format!("{}{}", 
            style(format!("{}", icon)).green(),
            description
        ))?;
        Ok(())
    }

    /// Print a sub-step with indentation
    pub fn sub_step(&self, description: &str, status: StepStatus) -> std::io::Result<()> {
        let status_icon = match status {
            StepStatus::InProgress => style("🔄").yellow(),
            StepStatus::Success => style("✅").green(),
            StepStatus::Warning => style("⚠️").yellow(),
            StepStatus::Error => style("❌").red(),
        };
        
        self.term.write_line(&format!("   ├─ {} {}...", 
            status_icon,
            description
        ))?;
        Ok(())
    }

    /// Print a final sub-step
    pub fn sub_step_final(&self, description: &str, status: StepStatus) -> std::io::Result<()> {
        let status_icon = match status {
            StepStatus::InProgress => style("🔄").yellow(),
            StepStatus::Success => style("✅").green(),
            StepStatus::Warning => style("⚠️").yellow(),
            StepStatus::Error => style("❌").red(),
        };
        
        self.term.write_line(&format!("   └─ {} {}", 
            status_icon,
            description
        ))?;
        Ok(())
    }

    /// Simulate an interactive prompt
    pub fn interactive_prompt(&self, question: &str, suggestion: &str, response: &str) -> std::io::Result<()> {
        self.term.write_line(&format!("   {} {} {}", 
            Icons::QUESTION,
            style(question).bold(),
            style(format!("[{}]", suggestion)).dim()
        ))?;
        
        // Simulate thinking time
        std::thread::sleep(Duration::from_millis(500));
        
        self.term.write_line(&format!("   {} Developer input: {}", 
            Icons::ROBOT,
            style(response).green().bold()
        ))?;
        Ok(())
    }

    /// Print an info message with proper formatting
    pub fn info(&self, message: &str) -> std::io::Result<()> {
        self.term.write_line(&format!("{}{}",
            style(format!("{}", Icons::INFO)).blue(),
            message
        ))?;
        Ok(())
    }

    /// Print a success message
    pub fn success(&self, message: &str) -> std::io::Result<()> {
        self.term.write_line(&format!("{}{}",
            style(format!("{}", Icons::SUCCESS)).green(),
            message
        ))?;
        Ok(())
    }

    /// Print a warning message
    pub fn warning(&self, message: &str) -> std::io::Result<()> {
        self.term.write_line(&format!("{}{}",
            style(format!("{}", Icons::WARNING)).yellow(),
            message
        ))?;
        Ok(())
    }

    /// Print an error message
    pub fn error(&self, message: &str) -> std::io::Result<()> {
        self.term.write_line(&format!("{}{}",
            style(format!("{}", Icons::ERROR)).red(),
            message
        ))?;
        Ok(())
    }

    /// Create a progress bar for long operations
    pub fn progress_bar(&self, total: u64, message: &str) -> ProgressBar {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}"
            )
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
                write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap();
            })
            .progress_chars("##-")
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(120));
        pb
    }

    /// Print a bordered box with content
    pub fn boxed_content(&self, title: &str, lines: &[&str]) -> std::io::Result<()> {
        let max_width = lines.iter().map(|l| l.len()).max().unwrap_or(0).max(title.len());
        let border_width = max_width + 4;
        
        self.term.write_line(&format!("╭{}╮", "─".repeat(border_width - 2)))?;
        self.term.write_line(&format!("│ {} {}│", 
            style(title).bold().cyan(),
            " ".repeat(max_width - title.len() + 1)
        ))?;
        
        if !lines.is_empty() {
            self.term.write_line(&format!("├{}┤", "─".repeat(border_width - 2)))?;
            for line in lines {
                self.term.write_line(&format!("│ {} {}│", 
                    line,
                    " ".repeat(max_width - line.len() + 1)
                ))?;
            }
        }
        
        self.term.write_line(&format!("╰{}╯", "─".repeat(border_width - 2)))?;
        Ok(())
    }
}

impl Default for TerminalOutput {
    fn default() -> Self {
        Self::new()
    }
}

/// Status indicators for steps
#[derive(Debug, Clone, Copy)]
pub enum StepStatus {
    InProgress,
    Success,
    Warning,
    Error,
}

/// Helper trait for creating beautiful tables
pub trait BeautifulTable {
    fn to_beautiful_table(&self) -> Table;
}

/// Package information for table display
#[derive(Tabled)]
pub struct PackageTableRow {
    #[tabled(rename = "Package")]
    pub name: String,
    #[tabled(rename = "Version")]
    pub version: String,
    #[tabled(rename = "Dependencies")]
    pub deps: String,
    #[tabled(rename = "Health")]
    pub health: String,
    #[tabled(rename = "Outdated")]
    pub outdated: String,
}

impl PackageTableRow {
    pub fn new(name: String, version: String, deps: usize, health: bool, outdated: usize) -> Self {
        Self {
            name,
            version,
            deps: deps.to_string(),
            health: if health { "✅ Good".to_string() } else { "❌ Issues".to_string() },
            outdated: if outdated > 0 { outdated.to_string() } else { "-".to_string() },
        }
    }
}

/// Create a beautiful table from rows
pub fn create_package_table(rows: Vec<PackageTableRow>) -> Table {
    let mut table = Table::new(rows);
    table
        .with(Style::rounded())
        .with(Alignment::left())
        .modify(Rows::first(), Alignment::center());
    table
}

/// Summary statistics for display
#[derive(Tabled)]
pub struct SummaryTableRow {
    #[tabled(rename = "Metric")]
    pub metric: String,
    #[tabled(rename = "Value")]
    pub value: String,
    #[tabled(rename = "Status")]
    pub status: String,
}

impl SummaryTableRow {
    pub fn new(metric: &str, value: &str, status: &str) -> Self {
        Self {
            metric: metric.to_string(),
            value: value.to_string(),
            status: status.to_string(),
        }
    }
}

/// Create a summary table
pub fn create_summary_table(rows: Vec<SummaryTableRow>) -> Table {
    let mut table = Table::new(rows);
    table
        .with(Style::rounded())
        .with(Alignment::left())
        .modify(Rows::first(), Alignment::center());
    table
}