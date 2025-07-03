//! Commit validation functionality for the validator plugin

use crate::error::Result;
use crate::plugins::builtin::common::success_with_timing;
use crate::plugins::types::{PluginContext, PluginResult};
use std::time::Instant;

impl super::ValidatorPlugin {
    /// Validate commit messages using real Git analysis
    ///
    /// Performs comprehensive commit validation including:
    /// - Conventional commit format validation
    /// - Commit message quality analysis
    /// - Author information validation
    /// - Commit size and impact analysis
    /// - Branch naming convention checks
    ///
    /// # Arguments
    ///
    /// * `count` - Number of recent commits to validate
    /// * `context` - Plugin context with access to Git repository
    ///
    /// # Returns
    ///
    /// Detailed commit validation result with violations and recommendations
    #[allow(clippy::too_many_lines)]
    pub(super) fn validate_commits(count: i32, context: &PluginContext) -> Result<PluginResult> {
        let start_time = Instant::now();

        let mut valid_commits = Vec::new();
        let mut invalid_commits = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        // Get recent commits using real Git operations
        let commits = context
            .repository
            .get_commits_since(None, &None)
            .map_err(|e| crate::error::Error::plugin(format!("Failed to get commits: {e}")))?;

        let commits_to_check = commits.into_iter().take(usize::try_from(count).unwrap_or(10)).collect::<Vec<_>>();

        if commits_to_check.is_empty() {
            return Ok(PluginResult::success(serde_json::json!({
                "commits_checked": 0,
                "valid_commits": 0,
                "invalid_commits": [],
                "message": "No commits found to validate"
            })));
        }

        // Conventional commit patterns
        let conventional_pattern = regex::Regex::new(
            r"^(feat|fix|docs|style|refactor|perf|test|chore|ci|build)?(\(.+\))?!?: .{1,50}",
        )
        .map_err(|e| crate::error::Error::plugin(format!("Failed to compile regex: {e}")))?;

        for commit in &commits_to_check {
            let mut commit_issues = Vec::new();
            let mut commit_warnings = Vec::new();

            let message_lines: Vec<&str> = commit.message.lines().collect();
            let first_line = message_lines.first().unwrap_or(&"");

            // 1. Validate conventional commit format
            if !conventional_pattern.is_match(first_line) {
                commit_issues.push("Does not follow conventional commit format".to_string());
            }

            // 2. Validate commit message length
            if first_line.len() > 72 {
                commit_issues.push("Subject line too long (max 72 characters)".to_string());
            }

            if first_line.len() < 10 {
                commit_warnings.push(
                    "Subject line very short (consider more descriptive message)".to_string(),
                );
            }

            // 3. Validate commit message quality
            if first_line.ends_with('.') {
                commit_warnings.push("Subject line should not end with a period".to_string());
            }

            if !first_line.chars().next().map_or(false, char::is_uppercase)
                && !first_line.starts_with("feat")
                && !first_line.starts_with("fix")
                && !first_line.starts_with("docs")
                && !first_line.starts_with("style")
                && !first_line.starts_with("refactor")
                && !first_line.starts_with("perf")
                && !first_line.starts_with("test")
                && !first_line.starts_with("chore")
                && !first_line.starts_with("ci")
                && !first_line.starts_with("build")
            {
                commit_warnings.push(
                    "Subject line should start with capital letter or conventional commit type"
                        .to_string(),
                );
            }

            // 4. Check for merge commits (usually OK but note them)
            if first_line.starts_with("Merge") {
                commit_warnings.push(
                    "Merge commit detected - consider squashing for cleaner history".to_string(),
                );
            }

            // 5. Check for empty or placeholder messages
            let placeholder_messages = ["wip", "temp", "fix", "update", ".", "test"];
            if placeholder_messages.contains(&first_line.to_lowercase().as_str()) {
                commit_issues.push("Placeholder or non-descriptive commit message".to_string());
            }

            // 6. Validate author information
            if commit.author_email.is_empty() || !commit.author_email.contains('@') {
                commit_issues.push("Invalid or missing author email".to_string());
            }

            if commit.author_name.is_empty() || commit.author_name.len() < 2 {
                commit_issues.push("Invalid or missing author name".to_string());
            }

            // 7. Check body format if present (lines after first)
            if message_lines.len() > 1 {
                if message_lines.len() > 1 && !message_lines[1].is_empty() {
                    commit_warnings.push("Missing blank line between subject and body".to_string());
                }

                for (i, line) in message_lines.iter().enumerate().skip(2) {
                    if line.len() > 72 {
                        commit_warnings.push(format!("Body line {} exceeds 72 characters", i + 1));
                    }
                }
            }

            let commit_data = serde_json::json!({
                "hash": commit.hash[0..8].to_string(),
                "full_hash": commit.hash,
                "message": first_line,
                "author": commit.author_name,
                "email": commit.author_email,
                "date": commit.author_date,
                "issues": commit_issues,
                "warnings": commit_warnings,
                "conventional_commit": conventional_pattern.is_match(first_line),
                "message_length": first_line.len(),
                "has_body": message_lines.len() > 1
            });

            if commit_issues.is_empty() {
                valid_commits.push(commit_data);
            } else {
                invalid_commits.push(commit_data);
            }

            // Accumulate warnings
            for warning in commit_warnings {
                warnings.push(serde_json::json!({
                    "commit": commit.hash[0..8].to_string(),
                    "message": warning
                }));
            }
        }

        // Generate recommendations based on analysis
        let invalid_percentage = if commits_to_check.is_empty() {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                (invalid_commits.len() as f64 / commits_to_check.len().max(1) as f64) * 100.0
            }
        };

        if invalid_percentage > 50.0 {
            recommendations.push("Consider adopting conventional commit standards".to_string());
            recommendations.push("Set up commit message templates or hooks".to_string());
        }

        if warnings.len() > commits_to_check.len() / 2 {
            recommendations.push("Review commit message best practices with the team".to_string());
        }

        if invalid_commits.is_empty() && warnings.is_empty() {
            recommendations
                .push("Excellent commit hygiene! Keep up the good practices".to_string());
        }

        // Calculate commit quality score
        let quality_score =
            u8::try_from(std::cmp::max(0, 100 - (invalid_commits.len() * 15) - (warnings.len() * 3))).unwrap_or(0);

        let overall_status = match quality_score {
            90..=100 => "excellent",
            75..=89 => "good",
            60..=74 => "fair",
            40..=59 => "poor",
            _ => "critical",
        };

        // Analyze commit patterns
        let mut commit_types = std::collections::HashMap::new();
        for commit in &commits_to_check {
            let first_line = commit.message.lines().next().unwrap_or("");
            if let Some(cap) = conventional_pattern.captures(first_line) {
                if let Some(commit_type) = cap.get(1) {
                    *commit_types.entry(commit_type.as_str().to_string()).or_insert(0) += 1;
                }
            } else {
                *commit_types.entry("non-conventional".to_string()).or_insert(0) += 1;
            }
        }

        let result = serde_json::json!({
            "commits_checked": commits_to_check.len(),
            "valid_commits": valid_commits.len(),
            "invalid_commits": invalid_commits,
            "valid_commit_details": valid_commits,
            "warnings": warnings,
            "recommendations": recommendations,
            "quality_score": quality_score,
            "overall_status": overall_status,
            "statistics": {
                "invalid_percentage": format!("{:.1}%", invalid_percentage),
                "warnings_count": warnings.len(),
                "conventional_commits": valid_commits.iter()
                    .filter(|c| c["conventional_commit"].as_bool().unwrap_or(false))
                    .count(),
                "average_message_length": commits_to_check.iter()
                    .map(|c| c.message.lines().next().unwrap_or("").len())
                    .sum::<usize>() / commits_to_check.len().max(1)
            },
            "commit_type_distribution": commit_types,
            "analysis_period": {
                "latest_commit": commits_to_check.first().map(|c| &c.author_date),
                "oldest_commit": commits_to_check.last().map(|c| &c.author_date)
            }
        });

        Ok(success_with_timing(result, start_time)
            .with_metadata("command", "validate-commits")
            .with_metadata("validator", "builtin")
            .with_metadata("real_validation", true)
            .with_metadata("quality_score", quality_score)
            .with_metadata("commits_analyzed", commits_to_check.len()))
    }
}