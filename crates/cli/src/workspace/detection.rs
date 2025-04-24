use crate::common::errors::CliResult;
use log::{debug, info};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Detect workspace patterns based on package manager and config files
pub fn detect_workspace_patterns(path: &Path) -> Vec<String> {
    let mut patterns = Vec::new();

    // Try to detect patterns from package.json workspaces
    patterns.extend(extract_patterns_from_package_json(path));

    // Try pnpm-workspace.yaml if no patterns found yet
    if patterns.is_empty() {
        patterns.extend(extract_patterns_from_pnpm_workspace(path));
    }

    // Try lerna.json if still no patterns
    if patterns.is_empty() {
        patterns.extend(extract_patterns_from_lerna_config(path));
    }

    // Add fallback patterns if none were found
    if patterns.is_empty() {
        info!("No workspace patterns found, using default patterns");
        patterns.push("packages/*/package.json".to_string());
        patterns.push("apps/*/package.json".to_string());
        patterns.push("modules/*/package.json".to_string());
        patterns.push("libs/*/package.json".to_string());
    }

    info!("Detected workspace patterns: {:?}", patterns);
    patterns
}

/// Extract workspace patterns from package.json
fn extract_patterns_from_package_json(path: &Path) -> Vec<String> {
    let mut patterns = Vec::new();
    let package_json_path = path.join("package.json");

    if !package_json_path.exists() {
        debug!("No package.json found at {}", package_json_path.display());
        return patterns;
    }

    match fs::read_to_string(&package_json_path) {
        Ok(content) => {
            match serde_json::from_str::<Value>(&content) {
                Ok(json) => {
                    if let Some(workspaces) = json.get("workspaces") {
                        // Handle array format: "workspaces": ["packages/*", "apps/*"]
                        if let Some(workspaces_array) = workspaces.as_array() {
                            for workspace in workspaces_array {
                                if let Some(pattern) = workspace.as_str() {
                                    // Convert glob pattern to package.json pattern
                                    patterns.push(format!("{}/package.json", pattern));
                                }
                            }
                        }
                        // Handle object format: "workspaces": { "packages": ["packages/*"] }
                        else if let Some(workspaces_obj) = workspaces.get("packages") {
                            if let Some(packages_array) = workspaces_obj.as_array() {
                                for package in packages_array {
                                    if let Some(pattern) = package.as_str() {
                                        patterns.push(format!("{}/package.json", pattern));
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => debug!("Failed to parse package.json: {}", e),
            }
        }
        Err(e) => debug!("Failed to read package.json: {}", e),
    }

    debug!("Found {} patterns in package.json", patterns.len());
    patterns
}

/// Extract workspace patterns from pnpm-workspace.yaml
fn extract_patterns_from_pnpm_workspace(path: &Path) -> Vec<String> {
    let mut patterns = Vec::new();
    let pnpm_workspace_path = path.join("pnpm-workspace.yaml");

    if !pnpm_workspace_path.exists() {
        debug!("No pnpm-workspace.yaml found at {}", pnpm_workspace_path.display());
        return patterns;
    }

    match fs::read_to_string(&pnpm_workspace_path) {
        Ok(content) => match serde_yaml::from_str::<serde_yaml::Value>(&content) {
            Ok(yaml) => {
                if let Some(packages) = yaml.get("packages") {
                    if let Some(packages_array) = packages.as_sequence() {
                        for package in packages_array {
                            if let Some(pattern) = package.as_str() {
                                patterns.push(format!("{}/package.json", pattern));
                            }
                        }
                    }
                }
            }
            Err(e) => debug!("Failed to parse pnpm-workspace.yaml: {}", e),
        },
        Err(e) => debug!("Failed to read pnpm-workspace.yaml: {}", e),
    }

    debug!("Found {} patterns in pnpm-workspace.yaml", patterns.len());
    patterns
}

/// Extract workspace patterns from lerna.json
fn extract_patterns_from_lerna_config(path: &Path) -> Vec<String> {
    let mut patterns = Vec::new();
    let lerna_path = path.join("lerna.json");

    if !lerna_path.exists() {
        debug!("No lerna.json found at {}", lerna_path.display());
        return patterns;
    }

    match fs::read_to_string(&lerna_path) {
        Ok(content) => match serde_json::from_str::<Value>(&content) {
            Ok(json) => {
                if let Some(packages) = json.get("packages") {
                    if let Some(packages_array) = packages.as_array() {
                        for package in packages_array {
                            if let Some(pattern) = package.as_str() {
                                patterns.push(format!("{}/package.json", pattern));
                            }
                        }
                    }
                }
            }
            Err(e) => debug!("Failed to parse lerna.json: {}", e),
        },
        Err(e) => debug!("Failed to read lerna.json: {}", e),
    }

    debug!("Found {} patterns in lerna.json", patterns.len());
    patterns
}

/// Get standard exclude patterns for workspace discovery
pub fn get_standard_exclude_patterns() -> Vec<String> {
    vec![
        "**/node_modules/**".to_string(),
        "**/dist/**".to_string(),
        "**/build/**".to_string(),
        "**/.git/**".to_string(),
        "**/.cache/**".to_string(),
        "**/.vitepress/**".to_string(),
        "**/.github/**".to_string(),
    ]
}

/// Workspace detector for finding workspaces in a repository
pub struct WorkspaceDetector {
    root_path: PathBuf,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
}

impl WorkspaceDetector {
    pub fn new<P: AsRef<Path>>(root_path: P) -> Self {
        Self {
            root_path: root_path.as_ref().to_path_buf(),
            include_patterns: Vec::new(),
            exclude_patterns: get_standard_exclude_patterns(),
        }
    }

    pub fn with_patterns<P: AsRef<Path>>(
        root_path: P,
        include_patterns: Vec<String>,
        exclude_patterns: Vec<String>,
    ) -> Self {
        Self { root_path: root_path.as_ref().to_path_buf(), include_patterns, exclude_patterns }
    }

    pub fn add_include_pattern(&mut self, pattern: String) {
        self.include_patterns.push(pattern);
    }

    pub fn add_exclude_pattern(&mut self, pattern: String) {
        self.exclude_patterns.push(pattern);
    }

    pub fn set_include_patterns(&mut self, patterns: Vec<String>) {
        self.include_patterns = patterns;
    }

    pub fn set_exclude_patterns(&mut self, patterns: Vec<String>) {
        self.exclude_patterns = patterns;
    }

    pub fn detect(&self) -> CliResult<Vec<PathBuf>> {
        let mut workspaces = HashSet::new();

        // Use the existing patterns or detect them
        let patterns = if self.include_patterns.is_empty() {
            detect_workspace_patterns(&self.root_path)
        } else {
            self.include_patterns.clone()
        };

        // Find workspaces based on patterns
        for pattern in patterns {
            let glob_pattern = self.root_path.join(&pattern);
            if let Some(glob_pattern_str) = glob_pattern.to_str() {
                match glob::glob(glob_pattern_str) {
                    Ok(paths) => {
                        for entry in paths.filter_map(Result::ok) {
                            if !self.is_excluded(&entry) {
                                if entry.is_dir() {
                                    workspaces.insert(entry);
                                } else if entry.is_file() && entry.ends_with("package.json") {
                                    if let Some(parent) = entry.parent() {
                                        workspaces.insert(parent.to_path_buf());
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to process glob pattern {}: {}", pattern, e);
                    }
                }
            }
        }

        Ok(workspaces.into_iter().collect())
    }

    fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.exclude_patterns.iter().any(|pattern| {
            if let Ok(pattern) = glob::Pattern::new(pattern) {
                pattern.matches(&path_str)
            } else {
                false
            }
        })
    }

    pub fn scan_for_repositories(&self) -> CliResult<Vec<(PathBuf, Option<String>)>> {
        let workspaces = self.detect()?;
        let mut repositories = Vec::new();

        for workspace in workspaces {
            // Try to read package.json for workspace name
            let package_json = workspace.join("package.json");
            let name = if package_json.exists() {
                if let Ok(content) = fs::read_to_string(&package_json) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        json.get("name").and_then(|n| n.as_str()).map(String::from)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            repositories.push((workspace, name));
        }

        Ok(repositories)
    }
}
