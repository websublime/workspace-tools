//! Workspace detection utilities
//!
//! Provides functions to detect workspaces in various monorepo setups
//! based on package manager configuration files.

use log::{debug, info};
use serde_json::Value;
use std::fs;
use std::path::Path;
use sublime_standard_tools::CorePackageManager;

/// Detect workspace patterns based on package manager and config files
pub fn detect_workspace_patterns(path: &Path) -> Vec<String> {
    let mut patterns = Vec::new();

    // Try to detect the package manager first
    let package_manager = sublime_standard_tools::detect_package_manager(path);
    info!("Detected package manager: {:?}", package_manager);

    // Check for workspace config based on package manager
    match package_manager {
        // For pnpm, prioritize checking pnpm-workspace.yaml
        Some(CorePackageManager::Pnpm) => {
            let pnpm_patterns = extract_patterns_from_pnpm_workspace(path);
            patterns.extend(pnpm_patterns);

            // If no patterns found in pnpm-workspace.yaml, check package.json as fallback
            if patterns.is_empty() {
                patterns.extend(extract_patterns_from_package_json(path));
            }
        }

        // For yarn/npm/bun, prioritize checking package.json workspaces
        Some(CorePackageManager::Yarn)
        | Some(CorePackageManager::Npm)
        | Some(CorePackageManager::Bun) => {
            patterns.extend(extract_patterns_from_package_json(path));
        }

        // If no package manager detected, check all possible config files
        None => {
            // Try all config files in order
            patterns.extend(extract_patterns_from_package_json(path));

            if patterns.is_empty() {
                patterns.extend(extract_patterns_from_pnpm_workspace(path));
            }

            if patterns.is_empty() {
                patterns.extend(extract_patterns_from_lerna_config(path));
            }
        }
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
pub fn extract_patterns_from_package_json(path: &Path) -> Vec<String> {
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
pub fn extract_patterns_from_pnpm_workspace(path: &Path) -> Vec<String> {
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
pub fn extract_patterns_from_lerna_config(path: &Path) -> Vec<String> {
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
