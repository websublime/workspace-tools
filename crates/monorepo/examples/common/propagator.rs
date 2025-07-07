//! Intelligent dependency propagation for version bumps
//!
//! This module implements smart dependency propagation that analyzes
//! the impact of version changes and propagates bumps intelligently,
//! with fallback to patch bumps when needed.

#![allow(dead_code)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::wildcard_enum_match_arm)]
#![allow(clippy::cast_possible_truncation)]

use sublime_monorepo_tools::{Result, config::VersionBumpType};
use super::terminal::{TerminalOutput, Icons, StepStatus};
use std::collections::{HashMap, HashSet, VecDeque};

/// Propagates version bumps across dependent packages
pub struct DependencyPropagator {
    terminal: TerminalOutput,
}

impl DependencyPropagator {
    /// Create a new dependency propagator
    pub fn new() -> Self {
        Self {
            terminal: TerminalOutput::new(),
        }
    }

    /// Propagate version bumps intelligently across the dependency graph
    pub fn propagate_bumps(
        &self,
        initial_bumps: &HashMap<String, VersionBumpType>,
        dependency_graph: &DependencyGraph,
    ) -> Result<PropagationResult> {
        self.terminal.step(Icons::GRAPH, "Analyzing dependency propagation...")?;
        
        let mut final_bumps = initial_bumps.clone();
        let mut propagation_reasons = HashMap::new();
        
        // Track packages we've processed to avoid cycles
        let mut processed = HashSet::new();
        let mut queue = VecDeque::new();
        
        // Start with initial packages
        for (package, bump_type) in initial_bumps {
            queue.push_back((package.clone(), *bump_type));
            propagation_reasons.insert(package.clone(), PropagationReason::DirectChange);
        }
        
        self.terminal.sub_step("Calculating transitive impacts", StepStatus::InProgress)?;
        
        while let Some((package, source_bump)) = queue.pop_front() {
            if !processed.insert(package.clone()) {
                continue; // Already processed
            }
            
            // Get dependents of this package
            if let Some(dependents) = dependency_graph.get_dependents(&package) {
                for dependent in dependents {
                    // Determine intelligent bump for dependent
                    let dependent_bump = self.determine_dependent_bump(
                        &package,
                        &source_bump,
                        dependent,
                        &final_bumps,
                    )?;
                    
                    // Apply bump if needed
                    if let Some(bump) = dependent_bump {
                        let reason = PropagationReason::DependencyUpdate {
                            dependency: package.clone(),
                            dependency_bump: source_bump,
                        };
                        
                        // Use highest bump if already has one
                        match final_bumps.get(dependent) {
                            Some(existing) if self.compare_bumps(existing, &bump) >= 0 => {
                                // Keep existing higher bump
                            }
                            _ => {
                                final_bumps.insert(dependent.clone(), bump);
                                propagation_reasons.insert(dependent.clone(), reason);
                                queue.push_back((dependent.clone(), bump));
                            }
                        }
                    }
                }
            }
        }
        
        // Show propagation summary
        let propagated_count = final_bumps.len() - initial_bumps.len();
        self.terminal.sub_step(
            &format!("Propagated to {} additional packages", propagated_count),
            StepStatus::Success
        )?;
        
        // Show intelligent decisions
        let smart_propagations = propagation_reasons.values()
            .filter(|r| matches!(r, PropagationReason::DependencyUpdate { .. }))
            .count();
        
        self.terminal.sub_step_final(
            &format!("Made {} intelligent propagation decisions", smart_propagations),
            StepStatus::Success
        )?;
        
        let total_affected = final_bumps.len();
        
        Ok(PropagationResult {
            final_bumps,
            propagation_reasons,
            total_packages_affected: total_affected,
            smart_propagations,
        })
    }

    /// Determine the appropriate bump for a dependent package
    fn determine_dependent_bump(
        &self,
        dependency: &str,
        dependency_bump: &VersionBumpType,
        dependent: &str,
        current_bumps: &HashMap<String, VersionBumpType>,
    ) -> Result<Option<VersionBumpType>> {
        // If dependent already has a bump, we might not need to do anything
        if let Some(existing_bump) = current_bumps.get(dependent) {
            // If it's already major, no need to propagate
            if matches!(existing_bump, VersionBumpType::Major) {
                return Ok(None);
            }
        }
        
        // Smart propagation rules
        let propagated_bump = match dependency_bump {
            VersionBumpType::Major => {
                // Major bumps in dependencies usually require at least minor bump
                self.terminal.info(&format!(
                    "  🧠 {} has major change in {}, suggesting minor bump",
                    dependent, dependency
                ))?;
                VersionBumpType::Minor
            }
            VersionBumpType::Minor => {
                // Minor bumps might only need patch, unless it's a critical dependency
                if self.is_critical_dependency(dependency, dependent) {
                    self.terminal.info(&format!(
                        "  🧠 {} critically depends on {}, suggesting minor bump",
                        dependent, dependency
                    ))?;
                    VersionBumpType::Minor
                } else {
                    self.terminal.info(&format!(
                        "  🧠 {} has minor change in {}, suggesting patch bump",
                        dependent, dependency
                    ))?;
                    VersionBumpType::Patch
                }
            }
            VersionBumpType::Patch => {
                // Patch bumps usually only need patch propagation
                VersionBumpType::Patch
            }
            VersionBumpType::Snapshot => {
                // Fallback to patch for any other cases
                VersionBumpType::Patch
            }
        };
        
        Ok(Some(propagated_bump))
    }

    /// Check if a dependency is critical (would affect public API)
    fn is_critical_dependency(&self, dependency: &str, dependent: &str) -> bool {
        // Simulation: UI components are critical for apps
        if dependency.contains("ui") && dependent.contains("app") {
            return true;
        }
        
        // Core/shared libraries are always critical
        if dependency.contains("core") || dependency.contains("shared") {
            return true;
        }
        
        false
    }

    /// Compare bump types (returns -1, 0, 1 like comparison)
    fn compare_bumps(&self, a: &VersionBumpType, b: &VersionBumpType) -> i8 {
        let level = |bump: &VersionBumpType| match bump {
            VersionBumpType::Major => 3,
            VersionBumpType::Minor => 2,
            VersionBumpType::Patch => 1,
            VersionBumpType::Snapshot => 0,
        };
        
        (level(a) as i8) - (level(b) as i8)
    }
}

/// Simple dependency graph for demo
pub struct DependencyGraph {
    edges: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    /// Create a demo dependency graph
    pub fn demo() -> Self {
        let mut edges = HashMap::new();
        
        // Define dependencies (package -> dependents)
        edges.insert("@acme/shared".to_string(), vec![
            "@acme/ui-lib".to_string(),
            "@acme/core-lib".to_string(),
        ]);
        
        edges.insert("@acme/core-lib".to_string(), vec![
            "@acme/web-app".to_string(),
        ]);
        
        edges.insert("@acme/ui-lib".to_string(), vec![
            "@acme/web-app".to_string(),
        ]);
        
        Self { edges }
    }

    /// Get packages that depend on the given package
    pub fn get_dependents(&self, package: &str) -> Option<&Vec<String>> {
        self.edges.get(package)
    }
}

/// Result of dependency propagation
#[derive(Debug)]
pub struct PropagationResult {
    pub final_bumps: HashMap<String, VersionBumpType>,
    pub propagation_reasons: HashMap<String, PropagationReason>,
    pub total_packages_affected: usize,
    pub smart_propagations: usize,
}

/// Reason for propagating a version bump
#[derive(Debug, Clone)]
pub enum PropagationReason {
    DirectChange,
    DependencyUpdate {
        dependency: String,
        dependency_bump: VersionBumpType,
    },
}

impl Default for DependencyPropagator {
    fn default() -> Self {
        Self::new()
    }
}