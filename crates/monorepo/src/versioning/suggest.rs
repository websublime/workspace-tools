//! Logic for suggesting version bumps based on changes.
//!
//! This module implements the algorithms for determining appropriate version
//! bumps based on changes, dependencies, and cycle relationships between packages.
//! It handles the complex logic of propagating version bumps through the dependency graph.

use crate::{
    BumpReason, BumpType, Change, ChangeTracker, ChangeType, VersionBumpStrategy, VersioningError,
    VersioningResult, Workspace,
};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Maximum number of update propagation waves to prevent infinite loops
const MAX_WAVES: usize = 10;

/// Version bump suggestion for a package.
///
/// Contains information about a suggested version bump, including
/// the current and suggested versions, bump type, and reasons for the bump.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::{BumpReason, BumpType, VersionSuggestion};
///
/// let suggestion = VersionSuggestion::new(
///     "ui".to_string(),
///     "1.0.0".to_string(),
///     "1.1.0".to_string(),
///     BumpType::Minor
/// )
/// .with_reason(BumpReason::Feature("Add button component".to_string()));
///
/// assert_eq!(suggestion.current_version, "1.0.0");
/// assert_eq!(suggestion.suggested_version, "1.1.0");
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionSuggestion {
    /// Package name
    pub package_name: String,
    /// Current version
    pub current_version: String,
    /// Suggested next version
    pub suggested_version: String,
    /// Type of bump
    pub bump_type: BumpType,
    /// Reasons for bump
    pub reasons: Vec<BumpReason>,
    /// Cycle group this package belongs to (if any)
    pub cycle_group: Option<Vec<String>>,
}

impl VersionSuggestion {
    /// Create a new version suggestion.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package
    /// * `current_version` - Current version string
    /// * `suggested_version` - Suggested new version string
    /// * `bump_type` - Type of version bump
    ///
    /// # Returns
    ///
    /// A new version suggestion.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::{BumpType, VersionSuggestion};
    ///
    /// let suggestion = VersionSuggestion::new(
    ///     "api".to_string(),
    ///     "1.0.0".to_string(),
    ///     "2.0.0".to_string(),
    ///     BumpType::Major
    /// );
    /// ```
    pub fn new(
        package_name: String,
        current_version: String,
        suggested_version: String,
        bump_type: BumpType,
    ) -> Self {
        Self {
            package_name,
            current_version,
            suggested_version,
            bump_type,
            reasons: Vec::new(),
            cycle_group: None,
        }
    }

    /// Add a reason for this version bump.
    ///
    /// # Arguments
    ///
    /// * `reason` - Reason for the bump
    ///
    /// # Returns
    ///
    /// The modified suggestion.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::{BumpReason, BumpType, VersionSuggestion};
    ///
    /// let suggestion = VersionSuggestion::new(
    ///     "api".to_string(),
    ///     "1.0.0".to_string(),
    ///     "2.0.0".to_string(),
    ///     BumpType::Major
    /// )
    /// .with_reason(BumpReason::Breaking("Changed authentication API".to_string()));
    /// ```
    #[must_use]
    pub fn with_reason(mut self, reason: BumpReason) -> Self {
        self.reasons.push(reason);
        self
    }

    /// Add multiple reasons for this version bump.
    ///
    /// # Arguments
    ///
    /// * `reasons` - Reasons for the bump
    ///
    /// # Returns
    ///
    /// The modified suggestion.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::{BumpReason, BumpType, VersionSuggestion};
    ///
    /// let suggestion = VersionSuggestion::new(
    ///     "api".to_string(),
    ///     "1.0.0".to_string(),
    ///     "2.0.0".to_string(),
    ///     BumpType::Major
    /// )
    /// .with_reasons(vec![
    ///     BumpReason::Breaking("Changed authentication API".to_string()),
    ///     BumpReason::Feature("Added user profiles".to_string())
    /// ]);
    /// ```
    #[must_use]
    pub fn with_reasons(mut self, reasons: Vec<BumpReason>) -> Self {
        self.reasons.extend(reasons);
        self
    }

    /// Set the cycle group information
    ///
    /// # Arguments
    ///
    /// * `group` - List of package names in the cycle group
    ///
    /// # Returns
    ///
    /// The modified suggestion.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::{BumpType, VersionSuggestion};
    ///
    /// let suggestion = VersionSuggestion::new(
    ///     "ui".to_string(),
    ///     "1.0.0".to_string(),
    ///     "1.1.0".to_string(),
    ///     BumpType::Minor
    /// )
    /// .with_cycle_group(vec!["ui".to_string(), "core".to_string()]);
    /// ```
    #[must_use]
    pub fn with_cycle_group(mut self, group: Vec<String>) -> Self {
        self.cycle_group = Some(group);
        self
    }
}

/// Preview of version bumps to be applied.
///
/// Contains information about version changes suggested by the version bump algorithm,
/// including cycle information for better understanding of dependency relationships.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::{BumpType, VersionBumpPreview, VersionSuggestion};
///
/// let preview = VersionBumpPreview {
///     changes: vec![
///         VersionSuggestion::new(
///             "ui".to_string(),
///             "1.0.0".to_string(),
///             "1.1.0".to_string(),
///             BumpType::Minor
///         )
///     ],
///     cycle_detected: false,
///     cycle_groups: Vec::new(),
/// };
/// ```
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct VersionBumpPreview {
    /// Version changes to be applied
    pub changes: Vec<VersionSuggestion>,
    /// Cycle detected in dependencies preventing some strategies
    pub cycle_detected: bool,
    /// Groups of packages forming cycles
    #[serde(default)]
    pub cycle_groups: Vec<Vec<String>>,
}

/// Determine the bump type based on a change type.
///
/// Maps change types to version bump types according to the specified strategy.
///
/// # Arguments
///
/// * `change` - The change to analyze
/// * `strategy` - The version bump strategy to use
///
/// # Returns
///
/// The appropriate bump type for the change.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::{
///     BumpType, Change, ChangeType, VersionBumpStrategy, determine_bump_type_from_change
/// };
///
/// let strategy = VersionBumpStrategy::Independent {
///     major_if_breaking: true,
///     minor_if_feature: true,
///     patch_otherwise: true,
/// };
///
/// // Breaking change -> Major bump
/// let breaking_change = Change::new("api", ChangeType::Fix, "Fix auth", true);
/// assert_eq!(determine_bump_type_from_change(&breaking_change, &strategy), BumpType::Major);
///
/// // Feature change -> Minor bump
/// let feature_change = Change::new("api", ChangeType::Feature, "Add endpoint", false);
/// assert_eq!(determine_bump_type_from_change(&feature_change, &strategy), BumpType::Minor);
///
/// // Fix change -> Patch bump
/// let fix_change = Change::new("api", ChangeType::Fix, "Fix bug", false);
/// assert_eq!(determine_bump_type_from_change(&fix_change, &strategy), BumpType::Patch);
/// ```
#[allow(clippy::wildcard_in_or_patterns)]
pub fn determine_bump_type_from_change(
    change: &Change,
    strategy: &VersionBumpStrategy,
) -> BumpType {
    match strategy {
        VersionBumpStrategy::Independent {
            major_if_breaking,
            minor_if_feature,
            patch_otherwise,
        } => {
            if change.breaking && *major_if_breaking {
                BumpType::Major
            } else if matches!(change.change_type, ChangeType::Feature) && *minor_if_feature {
                BumpType::Minor
            } else if *patch_otherwise {
                BumpType::Patch
            } else {
                BumpType::None
            }
        }
        VersionBumpStrategy::Synchronized { .. } => {
            // For synchronized strategy, bump type is determined at workspace level
            BumpType::None
        }
        VersionBumpStrategy::ConventionalCommits { .. } => {
            // For conventional commits, determine based on the change type
            if change.breaking {
                BumpType::Major
            } else {
                match change.change_type {
                    ChangeType::Feature => BumpType::Minor,
                    ChangeType::Breaking => BumpType::Major,
                    ChangeType::Fix | _ => BumpType::Patch,
                }
            }
        }
        VersionBumpStrategy::Manual(_) => {
            // For manual strategy, bump type is specified directly
            BumpType::None
        }
    }
}

/// Get the highest bump type from a collection of changes.
///
/// Analyzes a set of changes and returns the highest priority bump type needed.
///
/// # Arguments
///
/// * `changes` - Collection of changes to analyze
/// * `strategy` - The version bump strategy to use
///
/// # Returns
///
/// The highest priority bump type needed.
///
/// # Examples
///
/// ```
/// # use sublime_monorepo_tools::{Change, ChangeType, VersionBumpStrategy, get_highest_bump_type};
/// # // This function is private, so we can't actually test it directly in examples
/// # fn example() {
/// #    let strategy = VersionBumpStrategy::default();
/// #    let changes = vec![
/// #        &Change::new("api", ChangeType::Feature, "Add endpoint", false),
/// #        &Change::new("api", ChangeType::Fix, "Fix bug", false),
/// #    ];
/// # }
/// ```
fn get_highest_bump_type(changes: &[&Change], strategy: &VersionBumpStrategy) -> BumpType {
    let mut highest = BumpType::None;

    for change in changes {
        let bump_type = determine_bump_type_from_change(change, strategy);
        highest = match (highest, bump_type) {
            (_, BumpType::Major) | (BumpType::Major, _) => BumpType::Major,
            (BumpType::Minor, _) | (_, BumpType::Minor) => BumpType::Minor,
            (BumpType::Patch, _) | (_, BumpType::Patch) => BumpType::Patch,
            (BumpType::Snapshot, _) | (_, BumpType::Snapshot) => BumpType::Snapshot,
            (BumpType::None, BumpType::None) => BumpType::None,
        };
    }

    highest
}

/// Generate version suggestions for packages based on their changes.
///
/// Analyzes changes and the dependency graph to suggest appropriate version
/// bumps for packages in the workspace, with cycle harmonization enabled by default.
///
/// # Arguments
///
/// * `workspace` - The workspace to analyze
/// * `change_tracker` - The change tracker containing change history
/// * `strategy` - The version bump strategy to use
///
/// # Returns
///
/// A map of package names to version suggestions.
///
/// # Errors
///
/// Returns an error if generating suggestions fails.
///
/// # Examples
///
/// ```no_run
/// use sublime_monorepo_tools::{ChangeTracker, VersionBumpStrategy, Workspace, suggest_version_bumps};
///
/// # fn example(workspace: &Workspace, tracker: &ChangeTracker) -> Result<(), Box<dyn std::error::Error>> {
/// let strategy = VersionBumpStrategy::default();
///
/// // Get version suggestions
/// let suggestions = suggest_version_bumps(workspace, tracker, &strategy)?;
///
/// // Process suggestions
/// for (package, suggestion) in &suggestions {
///     println!("{}: {} -> {}", package, suggestion.current_version, suggestion.suggested_version);
/// }
/// # Ok(())
/// # }
/// ```
pub fn suggest_version_bumps(
    workspace: &Workspace,
    change_tracker: &ChangeTracker,
    strategy: &VersionBumpStrategy,
) -> VersioningResult<HashMap<String, VersionSuggestion>> {
    suggest_version_bumps_with_options(workspace, change_tracker, strategy, true)
}

/// Generate version suggestions for packages based on their changes, with options for cycle handling.
///
/// Similar to `suggest_version_bumps`, but with additional control over cycle harmonization.
///
/// # Arguments
///
/// * `workspace` - The workspace to analyze
/// * `change_tracker` - The change tracker containing change history
/// * `strategy` - The version bump strategy to use
/// * `harmonize_cycles` - Whether to ensure packages in the same cycle get consistent version bumps
///
/// # Returns
///
/// A map of package names to version suggestions.
///
/// # Errors
///
/// Returns an error if generating suggestions fails.
///
/// # Examples
///
/// ```no_run
/// use sublime_monorepo_tools::{
///     ChangeTracker, VersionBumpStrategy, Workspace, suggest_version_bumps_with_options
/// };
///
/// # fn example(workspace: &Workspace, tracker: &ChangeTracker) -> Result<(), Box<dyn std::error::Error>> {
/// let strategy = VersionBumpStrategy::default();
///
/// // Get version suggestions without cycle harmonization
/// let suggestions = suggest_version_bumps_with_options(workspace, tracker, &strategy, false)?;
/// # Ok(())
/// # }
/// ```
#[allow(clippy::too_many_lines)]
pub fn suggest_version_bumps_with_options(
    workspace: &Workspace,
    change_tracker: &ChangeTracker,
    strategy: &VersionBumpStrategy,
    harmonize_cycles: bool,
) -> VersioningResult<HashMap<String, VersionSuggestion>> {
    let mut suggestions = HashMap::new();
    let mut sha = String::new();

    // Get Git SHA for snapshot versions if available
    if let Some(git_repo) = workspace.git_repo() {
        if let Ok(current_sha) = git_repo.get_current_sha() {
            // Use the first 7 characters of the SHA
            sha = if current_sha.len() > 7 { current_sha[0..7].to_string() } else { current_sha };
        }
    }

    // Get all unreleased changes grouped by package
    let unreleased_changes = change_tracker.unreleased_changes()?;

    match strategy {
        VersionBumpStrategy::Synchronized { version } => {
            // All packages get the same version - no special handling needed for cycles
            for package_info in workspace.sorted_packages() {
                let pkg_info = package_info.borrow();
                let pkg = pkg_info.package.borrow();
                let package_name = pkg.name().to_string();
                let current_version = pkg.version_str();

                // Create suggestion with the synchronized version
                let suggestion = VersionSuggestion::new(
                    package_name.clone(),
                    current_version.clone(),
                    version.clone(),
                    BumpType::None, // Actual bump type doesn't matter for synchronized
                )
                .with_reason(BumpReason::Manual);

                suggestions.insert(package_name, suggestion);
            }
        }

        VersionBumpStrategy::Independent { .. }
        | VersionBumpStrategy::ConventionalCommits { .. } => {
            // First pass: Determine initial bump types for individual packages
            let mut initial_bumps: HashMap<String, BumpType> = HashMap::new();

            for package_info in workspace.sorted_packages() {
                let pkg_info = package_info.borrow();
                let pkg = pkg_info.package.borrow();
                let package_name = pkg.name().to_string();
                let current_version = pkg.version_str();

                // Get changes for this package
                let package_changes = match unreleased_changes.get(&package_name) {
                    Some(changes) => changes.iter().collect::<Vec<_>>(),
                    None => Vec::new(),
                };

                // Determine bump type based on changes
                let bump_type = if package_changes.is_empty() {
                    BumpType::None
                } else {
                    get_highest_bump_type(&package_changes, strategy)
                };

                if bump_type != BumpType::None {
                    initial_bumps.insert(package_name.clone(), bump_type);

                    // Create reasons from changes
                    let reasons = package_changes
                        .iter()
                        .map(|change| {
                            if change.breaking {
                                BumpReason::Breaking(change.description.clone())
                            } else {
                                match change.change_type {
                                    ChangeType::Feature => {
                                        BumpReason::Feature(change.description.clone())
                                    }
                                    ChangeType::Fix => BumpReason::Fix(change.description.clone()),
                                    _ => BumpReason::Other(change.description.clone()),
                                }
                            }
                        })
                        .collect::<Vec<_>>();

                    // Calculate the new version
                    let new_version = match bump_type {
                        BumpType::Major => {
                            sublime_package_tools::Version::bump_major(&current_version)?
                        }
                        BumpType::Minor => {
                            sublime_package_tools::Version::bump_minor(&current_version)?
                        }
                        BumpType::Patch => {
                            sublime_package_tools::Version::bump_patch(&current_version)?
                        }
                        BumpType::Snapshot => {
                            if sha.is_empty() {
                                return Err(VersioningError::NoVersionSuggestion(
                                    package_name,
                                    "Cannot create snapshot version: Git SHA not available"
                                        .to_string(),
                                ));
                            }
                            sublime_package_tools::Version::bump_snapshot(&current_version, &sha)?
                        }
                        BumpType::None => continue,
                    };

                    // Create suggestion
                    let suggestion = VersionSuggestion::new(
                        package_name.clone(),
                        current_version,
                        new_version.to_string(),
                        bump_type,
                    )
                    .with_reasons(reasons);

                    suggestions.insert(package_name, suggestion);
                }
            }

            // Second pass: Handle cycles and propagate bumps (only if harmonize_cycles is true)
            if harmonize_cycles {
                // Get cycle information from the workspace
                let sorted_with_cycles = workspace.get_sorted_packages_with_circulars();
                let has_cycles = !sorted_with_cycles.circular.is_empty();

                if has_cycles {
                    // Process each cycle group
                    for cycle_group in &sorted_with_cycles.circular {
                        let mut highest_bump = BumpType::None;
                        let mut cycle_names = Vec::new();

                        // Find the highest required bump in this cycle
                        for pkg_rc in cycle_group {
                            let pkg_name = pkg_rc.borrow().package.borrow().name().to_string();
                            cycle_names.push(pkg_name.clone());

                            if let Some(bump) = initial_bumps.get(&pkg_name) {
                                if bump_higher_than(highest_bump, *bump) {
                                    highest_bump = *bump;
                                }
                            }
                        }

                        // If any package in the cycle needs a bump, apply it consistently to all packages
                        if highest_bump != BumpType::None {
                            for pkg_rc in cycle_group {
                                let pkg_info = pkg_rc.borrow();
                                let pkg = pkg_info.package.borrow();
                                let pkg_name = pkg.name().to_string();
                                let current_version = pkg.version_str();

                                // Only override if this package doesn't already have a higher bump
                                let existing_bump =
                                    initial_bumps.get(&pkg_name).unwrap_or(&BumpType::None);
                                if bump_higher_than(highest_bump, *existing_bump) {
                                    // Calculate the new version
                                    let new_version = match highest_bump {
                                        BumpType::Major => {
                                            sublime_package_tools::Version::bump_major(
                                                &current_version,
                                            )?
                                        }
                                        BumpType::Minor => {
                                            sublime_package_tools::Version::bump_minor(
                                                &current_version,
                                            )?
                                        }
                                        BumpType::Patch => {
                                            sublime_package_tools::Version::bump_patch(
                                                &current_version,
                                            )?
                                        }
                                        BumpType::Snapshot => {
                                            if sha.is_empty() {
                                                return Err(VersioningError::NoVersionSuggestion(
                                                    pkg_name.clone(),
                                                    "Cannot create snapshot version: Git SHA not available".to_string(),
                                                ));
                                            }
                                            sublime_package_tools::Version::bump_snapshot(
                                                &current_version,
                                                &sha,
                                            )?
                                        }
                                        BumpType::None => continue,
                                    };

                                    // Create a suggestion with cycle reasoning
                                    let reason = BumpReason::Other(format!(
                                        "Part of dependency cycle including: {}",
                                        cycle_names.join(", ")
                                    ));

                                    let suggestion =
                                        if let Some(mut existing) = suggestions.remove(&pkg_name) {
                                            // Update existing suggestion
                                            existing.bump_type = highest_bump;
                                            existing.suggested_version = new_version.to_string();
                                            existing.cycle_group = Some(cycle_names.clone());
                                            existing.with_reason(reason)
                                        } else {
                                            // Create new suggestion
                                            VersionSuggestion::new(
                                                pkg_name.clone(),
                                                current_version,
                                                new_version.to_string(),
                                                highest_bump,
                                            )
                                            .with_reason(reason)
                                            .with_cycle_group(cycle_names.clone())
                                        };

                                    suggestions.insert(pkg_name, suggestion);
                                }
                            }
                        }
                    }
                }
            }
        }

        VersionBumpStrategy::Manual(manual_versions) => {
            // For manual strategy, bump type is specified directly - no special cycle handling needed
            for (package_name, new_version) in manual_versions {
                if let Some(package_info) = workspace.get_package(package_name) {
                    let package_info_borrow = package_info.borrow();
                    let pkg = package_info_borrow.package.borrow();
                    let current_version = pkg.version_str();

                    // Create suggestion with manual version
                    let suggestion = VersionSuggestion::new(
                        package_name.clone(),
                        current_version,
                        new_version.clone(),
                        BumpType::None, // Actual bump type doesn't matter for manual
                    )
                    .with_reason(BumpReason::Manual);

                    suggestions.insert(package_name.clone(), suggestion);
                } else {
                    return Err(VersioningError::PackageNotFound(package_name.clone()));
                }
            }
        }
    }

    // Handle dependent packages that need updates due to dependency changes
    handle_dependency_updates(workspace, &mut suggestions, harmonize_cycles)?;

    Ok(suggestions)
}

/// Update packages that depend on packages with version changes.
///
/// Propagates version bumps through the dependency graph, ensuring that
/// packages depending on updated packages are also updated appropriately.
///
/// # Arguments
///
/// * `workspace` - The workspace to analyze
/// * `suggestions` - Map of existing version suggestions to update
///
/// # Returns
///
/// `Ok(())` if propagation succeeds.
///
/// # Errors
///
/// Returns an error if propagation fails.
#[allow(clippy::too_many_lines)]
#[allow(clippy::wildcard_in_or_patterns)]
fn handle_dependency_updates(
    workspace: &Workspace,
    suggestions: &mut HashMap<String, VersionSuggestion>,
    harmonize_cycles: bool, // Added parameter
) -> VersioningResult<()> {
    // Get cycle information only if harmonization is enabled
    let cycle_groups = if harmonize_cycles {
        let sorted_with_cycles = workspace.get_sorted_packages_with_circulars();
        sorted_with_cycles
            .circular
            .iter()
            .map(|group| {
                group
                    .iter()
                    .map(|p| p.borrow().package.borrow().name().to_string())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new() // Empty if we're not harmonizing cycles
    };

    // Create a cycle membership map only if harmonization is enabled
    let mut cycle_membership: HashMap<String, usize> = HashMap::new();
    if harmonize_cycles {
        for (i, group) in cycle_groups.iter().enumerate() {
            for pkg_name in group {
                cycle_membership.insert(pkg_name.clone(), i);
            }
        }
    }

    // Get all packages that have version changes
    let packages_with_changes: Vec<String> = suggestions.keys().cloned().collect();

    if packages_with_changes.is_empty() {
        return Ok(());
    }

    // Track packages we've processed and the propagation wave
    let mut processed_packages = suggestions.keys().cloned().collect::<HashSet<_>>();
    let mut new_updates = Vec::new();
    let mut wave_counter = 0;

    // Process in waves until no new changes or we hit max waves
    while wave_counter < MAX_WAVES {
        wave_counter += 1;
        debug!("Processing dependency update wave {}", wave_counter);
        new_updates.clear();
        let current_packages = processed_packages.clone();

        // Collect dependents of all packages updated in previous waves
        let mut wave_dependents: HashMap<String, Vec<String>> = HashMap::new();

        for package_name in &current_packages {
            // Find dependents (explicitly skip cycle detection)
            let dependents = workspace.dependents_of(package_name);

            for dependent_info in dependents {
                let dep_info = dependent_info.borrow();
                let dep_name = dep_info.package.borrow().name().to_string();

                // Skip if already processed
                if processed_packages.contains(&dep_name) {
                    continue;
                }

                // Add to wave_dependents for batch processing
                wave_dependents.entry(dep_name).or_default().push(package_name.clone());
            }
        }

        if wave_dependents.is_empty() {
            break; // No more updates needed
        }

        // Process each dependent in this wave
        for (dependent_name, dependencies) in wave_dependents {
            // Get the dependent package
            let Some(pkg_info) = workspace.get_package(&dependent_name) else {
                continue;
            };

            let current_version = pkg_info.borrow().package.borrow().version_str();

            // Determine the appropriate bump type based on dependency changes
            let mut bump_type = BumpType::Patch; // Default to patch for dependency updates
            let mut dependency_reasons = Vec::new();

            // Check if any dependency had a major/breaking change
            for dep_name in &dependencies {
                if let Some(suggestion) = suggestions.get(dep_name) {
                    if suggestion.bump_type == BumpType::Major {
                        bump_type = BumpType::Minor; // Propagate as minor when dependency has major change
                    }

                    // Create reason string
                    dependency_reasons.push(format!(
                        "Dependency {} was updated from {} to {}",
                        dep_name, suggestion.current_version, suggestion.suggested_version
                    ));
                }
            }

            // Check if this package is part of a cycle (only if harmonize_cycles is true)
            let mut cycle_info = None;
            if harmonize_cycles {
                if let Some(&cycle_index) = cycle_membership.get(&dependent_name) {
                    let cycle_group = &cycle_groups[cycle_index];

                    // Check if other packages in this cycle have already been bumped
                    for cycle_pkg in cycle_group {
                        if let Some(suggestion) = suggestions.get(cycle_pkg) {
                            // Use the highest bump type from the cycle
                            if bump_higher_than(suggestion.bump_type, bump_type) {
                                bump_type = suggestion.bump_type;
                            }
                        }
                    }

                    cycle_info = Some(cycle_group.clone());

                    // Add cycle-specific reason
                    dependency_reasons
                        .push(format!("Part of dependency cycle: {}", cycle_group.join(" → ")));
                }
            }

            // Calculate new version
            let new_version = match bump_type {
                BumpType::Major => sublime_package_tools::Version::bump_major(&current_version)?,
                BumpType::Minor => sublime_package_tools::Version::bump_minor(&current_version)?,
                BumpType::Patch | _ => {
                    sublime_package_tools::Version::bump_patch(&current_version)?
                }
            };

            // Create version suggestion
            let mut suggestion = VersionSuggestion::new(
                dependent_name.clone(),
                current_version,
                new_version.to_string(),
                bump_type,
            );

            // Add all dependency update reasons
            for reason_msg in dependency_reasons {
                suggestion = suggestion.with_reason(BumpReason::DependencyUpdate(reason_msg));
            }

            // Add cycle group if applicable
            if let Some(cycle_group) = cycle_info {
                suggestion = suggestion.with_cycle_group(cycle_group);
            }

            // Store for later batch addition
            new_updates.push((dependent_name.clone(), suggestion));
            processed_packages.insert(dependent_name);
        }

        // Add all updates from this wave to the suggestions map
        for (name, suggestion) in new_updates.drain(..) {
            suggestions.insert(name, suggestion);
        }
    }

    // Log warning if we hit the max waves limit
    if wave_counter >= MAX_WAVES {
        log::warn!("Hit maximum wave limit ({}) when propagating dependency updates. The dependency graph may contain complex cycles.", MAX_WAVES);
    }

    Ok(())
}

/// Prints a version bump preview to stdout.
///
/// Helper function for displaying a formatted representation of version changes.
///
/// # Arguments
///
/// * `preview` - The version bump preview to display
///
/// # Examples
///
/// ```no_run
/// use sublime_monorepo_tools::{VersionBumpPreview, print_version_bump_preview};
///
/// # fn example(preview: VersionBumpPreview) {
/// // Print the preview
/// print_version_bump_preview(&preview);
/// # }
/// ```
#[allow(clippy::print_stdout)]
pub fn print_version_bump_preview(preview: &VersionBumpPreview) {
    println!("Version Bump Preview:");
    println!("---------------------");

    if preview.cycle_detected {
        println!("\nCyclic Dependencies Detected:");
        for (i, group) in preview.cycle_groups.iter().enumerate() {
            println!("  Cycle Group {}: {}", i + 1, group.join(" → "));
        }
        println!("\nNote: Version bumps within cycles are harmonized to maintain consistency.");
    }

    println!("\nProposed Version Changes:");
    if preview.changes.is_empty() {
        println!("  No version changes needed.");
        return;
    }

    // Group changes by type for clearer presentation
    let mut major_changes = Vec::new();
    let mut minor_changes = Vec::new();
    let mut patch_changes = Vec::new();
    let mut dependency_changes = Vec::new();
    let mut cycle_changes = Vec::new();

    for change in &preview.changes {
        let is_cycle = change.cycle_group.is_some();
        let is_dependency_update =
            change.reasons.iter().any(|r| matches!(r, BumpReason::DependencyUpdate(_)));

        match (change.bump_type, is_cycle, is_dependency_update) {
            (_, true, _) => cycle_changes.push(change),
            (_, _, true) => dependency_changes.push(change),
            (BumpType::Major, _, _) => major_changes.push(change),
            (BumpType::Minor, _, _) => minor_changes.push(change),
            (BumpType::Patch, _, _) => patch_changes.push(change),
            _ => {}
        }
    }

    if !major_changes.is_empty() {
        println!("\n  Major Version Changes:");
        for change in major_changes {
            println!(
                "    {} {} → {}",
                change.package_name, change.current_version, change.suggested_version
            );
        }
    }

    if !minor_changes.is_empty() {
        println!("\n  Minor Version Changes:");
        for change in minor_changes {
            println!(
                "    {} {} → {}",
                change.package_name, change.current_version, change.suggested_version
            );
        }
    }

    if !patch_changes.is_empty() {
        println!("\n  Patch Version Changes:");
        for change in patch_changes {
            println!(
                "    {} {} → {}",
                change.package_name, change.current_version, change.suggested_version
            );
        }
    }

    if !dependency_changes.is_empty() {
        println!("\n  Dependency-Driven Changes:");
        for change in dependency_changes {
            println!(
                "    {} {} → {}",
                change.package_name, change.current_version, change.suggested_version
            );
            for reason in &change.reasons {
                if let BumpReason::DependencyUpdate(msg) = reason {
                    println!("      - {msg}");
                }
            }
        }
    }

    if !cycle_changes.is_empty() {
        println!("\n  Cycle-Harmonized Changes:");
        for change in cycle_changes {
            println!(
                "    {} {} → {}",
                change.package_name, change.current_version, change.suggested_version
            );
            if let Some(group) = &change.cycle_group {
                println!("      - Part of cycle: {}", group.join(" → "));
            }
        }
    }
}

/// Compares bump types to determine which has higher priority.
///
/// Helper function for comparing bump types according to semantic versioning principles.
///
/// # Arguments
///
/// * `a` - First bump type
/// * `b` - Second bump type
///
/// # Returns
///
/// `true` if `a` has higher priority than `b`, `false` otherwise.
///
/// # Examples
///
/// ```
/// # use sublime_monorepo_tools::BumpType;
/// # // This function is private, so we can't actually test it directly in examples
/// # fn example() {
/// #    // Major > Minor > Patch > Snapshot > None
/// #    // assert!(bump_higher_than(BumpType::Major, BumpType::Minor));
/// #    // assert!(bump_higher_than(BumpType::Minor, BumpType::Patch));
/// # }
/// ```
fn bump_higher_than(a: BumpType, b: BumpType) -> bool {
    matches!(
        (a, b),
        (BumpType::Major, _)
            | (BumpType::Minor, BumpType::Patch | BumpType::Snapshot | BumpType::None)
            | (BumpType::Patch, BumpType::Snapshot | BumpType::None)
            | (BumpType::Snapshot, BumpType::None)
    )
}
