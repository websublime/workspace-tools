//! Logic for suggesting version bumps based on changes.

use std::collections::{HashMap, HashSet};

use log::debug;

use crate::{
    BumpReason, BumpType, Change, ChangeTracker, ChangeType, VersionBumpStrategy, VersioningError,
    VersioningResult, Workspace,
};

/// Version bump suggestion for a package.
#[derive(Debug, Clone)]
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
}

impl VersionSuggestion {
    /// Create a new version suggestion.
    pub fn new(
        package_name: String,
        current_version: String,
        suggested_version: String,
        bump_type: BumpType,
    ) -> Self {
        Self { package_name, current_version, suggested_version, bump_type, reasons: Vec::new() }
    }

    /// Add a reason for this version bump.
    #[must_use]
    pub fn with_reason(mut self, reason: BumpReason) -> Self {
        self.reasons.push(reason);
        self
    }

    /// Add multiple reasons for this version bump.
    #[must_use]
    pub fn with_reasons(mut self, reasons: Vec<BumpReason>) -> Self {
        self.reasons.extend(reasons);
        self
    }
}

/// Preview of version bumps to be applied.
#[derive(Debug, Clone)]
pub struct VersionBumpPreview {
    /// Version changes to be applied
    pub changes: Vec<VersionSuggestion>,
    /// Cycle detected in dependencies preventing some strategies
    pub cycle_detected: bool,
}

/// Determine the bump type based on a change type.
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
#[allow(clippy::too_many_lines)]
pub fn suggest_version_bumps(
    workspace: &Workspace,
    change_tracker: &ChangeTracker,
    strategy: &VersionBumpStrategy,
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
            // All packages get the same version
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
            // Each package gets its own version based on its changes
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

                // Only continue if there's an actual bump
                if bump_type == BumpType::None {
                    debug!("No version bump needed for package: {}", package_name);
                    continue;
                }

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
                                "Cannot create snapshot version: Git SHA not available".to_string(),
                            ));
                        }
                        sublime_package_tools::Version::bump_snapshot(&current_version, &sha)?
                    }
                    BumpType::None => continue,
                };

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

        VersionBumpStrategy::Manual(manual_versions) => {
            // Only bump versions that are specified in the manual map
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
    handle_dependency_updates(workspace, &mut suggestions)?;

    Ok(suggestions)
}

/// Update packages that depend on packages with version changes.
fn handle_dependency_updates(
    workspace: &Workspace,
    suggestions: &mut HashMap<String, VersionSuggestion>,
) -> VersioningResult<()> {
    // Get all packages that have version changes
    let packages_with_changes: Vec<String> = suggestions.keys().cloned().collect();

    if packages_with_changes.is_empty() {
        return Ok(());
    }

    let mut dependent_updates = Vec::new();

    // Track packages we've already processed to avoid infinite loops in circular deps
    let mut processed_packages = HashSet::new();
    for pkg in suggestions.keys() {
        processed_packages.insert(pkg.clone());
    }

    // For each package with changes, find packages that depend on it
    for changed_package in &packages_with_changes {
        // Find packages that depend on the changed package
        // IMPORTANT: Pass false to skip cycle detection
        let dependents = workspace.dependents_of(changed_package, Some(false));

        for dependent_info in dependents {
            let dep_info = dependent_info.borrow();
            let dependent_name = dep_info.package.borrow().name().to_string();

            // Skip if this dependent already has a version change
            if suggestions.contains_key(&dependent_name) {
                continue;
            }

            // Skip if we've already processed this package in this recursion
            if processed_packages.contains(&dependent_name) {
                continue;
            }

            processed_packages.insert(dependent_name.clone());

            // This dependent needs to be updated because its dependency changed
            let current_version = dep_info.package.borrow().version_str();
            let new_version = sublime_package_tools::Version::bump_patch(&current_version)?;

            // Create suggestion for the dependent package
            let suggestion = VersionSuggestion::new(
                dependent_name.clone(),
                current_version,
                new_version.to_string(),
                BumpType::Patch,
            )
            .with_reason(BumpReason::DependencyUpdate(format!(
                "Dependency {changed_package} was updated"
            )));

            dependent_updates.push((dependent_name, suggestion));
        }
    }

    // Add all dependent updates to the suggestions map
    for (name, suggestion) in dependent_updates {
        suggestions.insert(name, suggestion);
    }

    Ok(())
}
