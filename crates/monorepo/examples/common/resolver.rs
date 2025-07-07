//! Conflict resolution for version bumps
//!
//! This module handles conflicts when multiple changesets or propagation
//! rules suggest different version bumps for the same package.

#![allow(dead_code)]
#![allow(clippy::if_not_else)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::useless_format)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::wildcard_enum_match_arm)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::single_char_add_str)]
#![allow(clippy::map_clone)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unused_self)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::print_stdout)]
#![allow(clippy::print_stderr)]
#![allow(clippy::use_self)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::similar_names)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::fn_params_excessive_bools)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::string_add)]
#![allow(clippy::string_add_assign)]

use sublime_monorepo_tools::{Result, config::VersionBumpType};
use super::terminal::{TerminalOutput, Icons};
use std::collections::HashMap;

/// Resolves conflicts between version bumps
pub struct ConflictResolver {
    terminal: TerminalOutput,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new() -> Self {
        Self {
            terminal: TerminalOutput::new(),
        }
    }

    /// Resolve conflicts between multiple version bump suggestions
    pub fn resolve_conflicts(
        &self,
        bump_suggestions: Vec<BumpSuggestion>,
    ) -> Result<ConflictResolution> {
        self.terminal.step(Icons::MERGE, "Resolving version bump conflicts...")?;
        
        // Group suggestions by package
        let mut suggestions_by_package: HashMap<String, Vec<BumpSuggestion>> = HashMap::new();
        for suggestion in bump_suggestions {
            suggestions_by_package
                .entry(suggestion.package.clone())
                .or_default()
                .push(suggestion);
        }
        
        let mut resolved_bumps = HashMap::new();
        let mut conflicts = Vec::new();
        
        for (package, suggestions) in suggestions_by_package {
            if suggestions.len() == 1 {
                // No conflict
                let suggestion = &suggestions[0];
                resolved_bumps.insert(package.clone(), suggestion.bump_type);
            } else {
                // Conflict detected - apply "highest bump wins" rule
                let conflict = self.resolve_package_conflict(&package, suggestions)?;
                
                self.terminal.info(&format!(
                    "  ⚔️ Conflict in {}: {} suggestions → resolved to {:?}",
                    package,
                    conflict.suggestions.len(),
                    conflict.resolved_bump
                ))?;
                
                resolved_bumps.insert(package.clone(), conflict.resolved_bump);
                conflicts.push(conflict);
            }
        }
        
        if conflicts.is_empty() {
            self.terminal.success("No conflicts detected")?;
        } else {
            self.terminal.success(&format!(
                "Resolved {} conflicts using 'highest bump wins' strategy",
                conflicts.len()
            ))?;
        }
        
        Ok(ConflictResolution {
            resolved_bumps,
            conflicts,
        })
    }

    /// Resolve conflict for a single package
    fn resolve_package_conflict(
        &self,
        package: &str,
        suggestions: Vec<BumpSuggestion>,
    ) -> Result<PackageConflict> {
        // Find the highest bump
        let highest_bump = suggestions
            .iter()
            .map(|s| &s.bump_type)
            .max_by_key(|bump| self.bump_priority(bump))
            .copied()
            .unwrap_or(VersionBumpType::Patch);
        
        // Show conflict details
        for suggestion in &suggestions {
            self.terminal.info(&format!(
                "    • {:?} suggested by: {}",
                suggestion.bump_type,
                suggestion.reason
            ))?;
        }
        
        Ok(PackageConflict {
            package: package.to_string(),
            suggestions,
            resolved_bump: highest_bump,
            resolution_strategy: ResolutionStrategy::HighestBumpWins,
        })
    }

    /// Get priority of a bump type (higher number = higher priority)
    fn bump_priority(&self, bump: &VersionBumpType) -> u8 {
        match bump {
            VersionBumpType::Major => 3,
            VersionBumpType::Minor => 2,
            VersionBumpType::Patch => 1,
            VersionBumpType::Snapshot => 0,
        }
    }

    /// Create a conflict resolution report
    pub fn create_report(&self, resolution: &ConflictResolution) -> String {
        let mut report = String::new();
        
        report.push_str("# Version Bump Conflict Resolution Report\n\n");
        
        if resolution.conflicts.is_empty() {
            report.push_str("✅ No conflicts detected - all version bumps are consistent\n");
        } else {
            report.push_str(&format!("## Conflicts Resolved: {}\n\n", resolution.conflicts.len()));
            
            for conflict in &resolution.conflicts {
                report.push_str(&format!("### Package: {}\n", conflict.package));
                report.push_str(&format!("**Resolution**: {:?} ({})\n\n", 
                    conflict.resolved_bump,
                    conflict.resolution_strategy
                ));
                report.push_str("**Suggestions**:\n");
                
                for suggestion in &conflict.suggestions {
                    report.push_str(&format!("- {:?} - {}\n", 
                        suggestion.bump_type,
                        suggestion.reason
                    ));
                }
                report.push_str("\n");
            }
        }
        
        report.push_str("\n## Final Version Bumps\n\n");
        for (package, bump) in &resolution.resolved_bumps {
            report.push_str(&format!("- {}: {:?}\n", package, bump));
        }
        
        report
    }
}

/// A version bump suggestion
#[derive(Debug, Clone)]
pub struct BumpSuggestion {
    pub package: String,
    pub bump_type: VersionBumpType,
    pub reason: String,
}

/// Result of conflict resolution
#[derive(Debug)]
pub struct ConflictResolution {
    pub resolved_bumps: HashMap<String, VersionBumpType>,
    pub conflicts: Vec<PackageConflict>,
}

/// A package with conflicting bump suggestions
#[derive(Debug)]
pub struct PackageConflict {
    pub package: String,
    pub suggestions: Vec<BumpSuggestion>,
    pub resolved_bump: VersionBumpType,
    pub resolution_strategy: ResolutionStrategy,
}

/// Strategy used to resolve conflicts
#[derive(Debug)]
pub enum ResolutionStrategy {
    HighestBumpWins,
}

impl std::fmt::Display for ResolutionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolutionStrategy::HighestBumpWins => write!(f, "highest bump wins"),
        }
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}