//! Tests for workspace patterns configuration and management

use sublime_monorepo_tools::{
    config::{
        ConfigManager, WorkspacePattern, WorkspacePatternOptions, PackageManagerType,
        NpmWorkspaceConfig,
    },
    analysis::MonorepoAnalyzer,
    MonorepoProject, Environment,
};
use tempfile::TempDir;

fn create_test_monorepo() -> Result<(MonorepoProject, TempDir), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root_path = temp_dir.path();
    
    // Create package.json
    let package_json = serde_json::json!({
        "name": "test-monorepo",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*", "apps/*"]
    });
    std::fs::write(root_path.join("package.json"), serde_json::to_string_pretty(&package_json)?)?;
    
    // Create package-lock.json for proper detection
    let package_lock = serde_json::json!({
        "name": "test-monorepo",
        "version": "1.0.0",
        "lockfileVersion": 3,
        "requires": true,
        "packages": {
            "": {
                "name": "test-monorepo",
                "version": "1.0.0",
                "workspaces": ["packages/*", "apps/*"]
            }
        }
    });
    std::fs::write(root_path.join("package-lock.json"), serde_json::to_string_pretty(&package_lock)?)?;
    
    // Create packages directory with multiple packages
    let packages_dir = root_path.join("packages");
    std::fs::create_dir_all(&packages_dir)?;
    
    // Create test packages
    for pkg_name in ["pkg-a", "pkg-b", "shared"] {
        let pkg_dir = packages_dir.join(pkg_name);
        std::fs::create_dir_all(&pkg_dir)?;
        
        let pkg_json = serde_json::json!({
            "name": format!("@test/{}", pkg_name),
            "version": "1.0.0",
            "main": "index.js"
        });
        std::fs::write(pkg_dir.join("package.json"), serde_json::to_string_pretty(&pkg_json)?)?;
        std::fs::write(pkg_dir.join("index.js"), "// Empty file")?;
    }
    
    // Create apps directory
    let apps_dir = root_path.join("apps");
    std::fs::create_dir_all(&apps_dir)?;
    
    let app_dir = apps_dir.join("web");
    std::fs::create_dir_all(&app_dir)?;
    let app_json = serde_json::json!({
        "name": "@test/web",
        "version": "1.0.0",
        "private": true
    });
    std::fs::write(app_dir.join("package.json"), serde_json::to_string_pretty(&app_json)?)?;
    
    // Initialize git repository
    let git_repo = sublime_git_tools::Repo::create(root_path.to_str().unwrap())?;
    git_repo.config("Test User", "test@example.com")?;
    git_repo.add_all()?;
    git_repo.commit("Initial commit")?;
    
    let project = MonorepoProject::new_for_test(root_path)?;
    Ok((project, temp_dir))
}

#[test]
fn test_config_manager_workspace_patterns() {
    let manager = ConfigManager::new();
    
    // Test default workspace config
    let workspace = manager.get_workspace().unwrap();
    assert!(workspace.patterns.is_empty());
    assert!(workspace.merge_with_detected);
    assert!(workspace.discovery.auto_detect);
    assert!(workspace.discovery.scan_common_patterns);
    
    // Test adding workspace pattern
    let pattern = WorkspacePattern {
        pattern: "custom/*".to_string(),
        description: Some("Custom packages".to_string()),
        enabled: true,
        priority: 200,
        package_managers: Some(vec![PackageManagerType::Npm]),
        environments: Some(vec![Environment::Development]),
        options: WorkspacePatternOptions::default(),
    };
    
    manager.add_workspace_pattern(pattern.clone()).unwrap();
    
    // Verify pattern was added
    let updated_workspace = manager.get_workspace().unwrap();
    assert_eq!(updated_workspace.patterns.len(), 1);
    assert_eq!(updated_workspace.patterns[0].pattern, "custom/*");
    assert_eq!(updated_workspace.patterns[0].priority, 200);
}

#[test]
fn test_workspace_pattern_filtering() {
    let manager = ConfigManager::new();
    
    // Add patterns for different package managers and environments
    let npm_pattern = WorkspacePattern {
        pattern: "npm-packages/*".to_string(),
        enabled: true,
        priority: 100,
        package_managers: Some(vec![PackageManagerType::Npm]),
        environments: None,
        ..Default::default()
    };
    
    let yarn_pattern = WorkspacePattern {
        pattern: "yarn-packages/*".to_string(),
        enabled: true,
        priority: 90,
        package_managers: Some(vec![PackageManagerType::Yarn]),
        environments: None,
        ..Default::default()
    };
    
    let dev_pattern = WorkspacePattern {
        pattern: "dev-packages/*".to_string(),
        enabled: true,
        priority: 80,
        package_managers: None,
        environments: Some(vec![Environment::Development]),
        ..Default::default()
    };
    
    manager.add_workspace_pattern(npm_pattern).unwrap();
    manager.add_workspace_pattern(yarn_pattern).unwrap();
    manager.add_workspace_pattern(dev_pattern).unwrap();
    
    // Test filtering by package manager
    let npm_patterns = manager.get_workspace_patterns(
        Some(PackageManagerType::Npm), 
        None
    ).unwrap();
    assert_eq!(npm_patterns.len(), 2); // npm-specific + general dev pattern
    assert!(npm_patterns.iter().any(|p| p.pattern == "npm-packages/*"));
    assert!(npm_patterns.iter().any(|p| p.pattern == "dev-packages/*"));
    
    let yarn_patterns = manager.get_workspace_patterns(
        Some(PackageManagerType::Yarn), 
        None
    ).unwrap();
    assert_eq!(yarn_patterns.len(), 2); // yarn-specific + general dev pattern
    assert!(yarn_patterns.iter().any(|p| p.pattern == "yarn-packages/*"));
    
    // Test filtering by environment
    let dev_patterns = manager.get_workspace_patterns(
        None, 
        Some(&Environment::Development)
    ).unwrap();
    assert!(dev_patterns.iter().any(|p| p.pattern == "dev-packages/*"));
}

#[test]
fn test_effective_workspace_patterns() {
    let manager = ConfigManager::new();
    
    // Add a config pattern
    let config_pattern = WorkspacePattern {
        pattern: "configured/*".to_string(),
        enabled: true,
        priority: 150,
        ..Default::default()
    };
    manager.add_workspace_pattern(config_pattern).unwrap();
    
    // Test with auto-detected patterns
    let auto_detected = vec![
        "packages/*".to_string(),
        "apps/*".to_string(),
    ];
    
    let effective = manager.get_effective_workspace_patterns(
        auto_detected,
        Some(PackageManagerType::Npm),
        None
    ).unwrap();
    
    // Should include both config and auto-detected patterns
    assert!(effective.contains(&"configured/*".to_string()));
    assert!(effective.contains(&"packages/*".to_string()));
    assert!(effective.contains(&"apps/*".to_string()));
    
    // Config pattern should be first due to higher priority
    assert_eq!(effective[0], "configured/*");
}

#[test]
fn test_package_manager_specific_patterns() {
    let manager = ConfigManager::new();
    
    // Test npm patterns
    let npm_patterns = manager.get_package_manager_patterns(PackageManagerType::Npm).unwrap();
    assert!(npm_patterns.is_empty()); // No specific config yet
    
    // Update config with npm-specific override
    manager.update(|config| {
        config.workspace.package_manager_configs.npm = Some(
            NpmWorkspaceConfig {
                workspaces_override: Some(vec![
                    "npm-libs/*".to_string(),
                    "npm-apps/*".to_string(),
                ]),
                use_workspaces: true,
                options: std::collections::HashMap::new(),
            }
        );
    }).unwrap();
    
    let npm_patterns = manager.get_package_manager_patterns(PackageManagerType::Npm).unwrap();
    assert_eq!(npm_patterns.len(), 2);
    assert!(npm_patterns.contains(&"npm-libs/*".to_string()));
    assert!(npm_patterns.contains(&"npm-apps/*".to_string()));
}

#[test]
fn test_workspace_validation() {
    let manager = ConfigManager::new();
    
    // Add patterns with validation requirements
    manager.update(|config| {
        config.workspace.validation.require_pattern_matches = true;
        config.workspace.validation.validate_naming = true;
        config.workspace.validation.naming_patterns = vec![
            "@test/*".to_string(),
        ];
    }).unwrap();
    
    // Add a pattern that should match
    let pattern = WorkspacePattern {
        pattern: "packages/*".to_string(),
        enabled: true,
        ..Default::default()
    };
    manager.add_workspace_pattern(pattern).unwrap();
    
    // Add a pattern that won't match
    let no_match_pattern = WorkspacePattern {
        pattern: "nonexistent/*".to_string(),
        enabled: true,
        ..Default::default()
    };
    manager.add_workspace_pattern(no_match_pattern).unwrap();
    
    let existing_packages = vec![
        "packages/pkg-a".to_string(),
        "packages/pkg-b".to_string(),
        "@test/web".to_string(),
    ];
    
    let validation_errors = manager.validate_workspace_config(&existing_packages).unwrap();
    
    // Should have error for pattern that doesn't match
    assert!(!validation_errors.is_empty());
    assert!(validation_errors.iter().any(|e| e.contains("nonexistent/*")));
    
    // Should have error for naming pattern violations
    assert!(validation_errors.iter().any(|e| e.contains("packages/pkg-a")));
}

#[test]
fn test_pattern_matching() {
    let manager = ConfigManager::new();
    
    // Test various pattern types
    assert!(manager.pattern_matches_package("packages/*", "packages/test"));
    assert!(manager.pattern_matches_package("*-app", "web-app"));
    assert!(manager.pattern_matches_package("src/*/index.js", "src/components/index.js"));
    assert!(manager.pattern_matches_package("exact-match", "exact-match"));
    
    // Test non-matches
    assert!(!manager.pattern_matches_package("packages/*", "apps/test"));
    assert!(!manager.pattern_matches_package("*-app", "web-service"));
    assert!(!manager.pattern_matches_package("exact-match", "close-match"));
}

#[test]
fn test_analyzer_config_workspace_patterns() {
    let (project, _temp_dir) = create_test_monorepo().unwrap();
    let analyzer = MonorepoAnalyzer::new(std::sync::Arc::new(project));
    
    // Test getting config workspace patterns (should be empty initially)
    let patterns = analyzer.get_config_workspace_patterns().unwrap();
    
    // Should fall back to common patterns or auto-detected
    assert!(!patterns.is_empty());
    // Common patterns should include packages/* and apps/*
    assert!(patterns.iter().any(|p| p.contains("packages") || p.contains("apps")));
}

#[test]
fn test_analyzer_auto_detection() {
    let (project, _temp_dir) = create_test_monorepo().unwrap();
    let analyzer = MonorepoAnalyzer::new(std::sync::Arc::new(project));
    
    // Test auto-detection of patterns
    let auto_patterns = analyzer.get_auto_detected_patterns().unwrap();
    
    // Should detect packages/* and apps/* based on directory structure
    assert!(auto_patterns.iter().any(|p| p.contains("packages")));
    assert!(auto_patterns.iter().any(|p| p.contains("apps")));
}

#[test]
fn test_validated_workspace_patterns() {
    let (project, _temp_dir) = create_test_monorepo().unwrap();
    let analyzer = MonorepoAnalyzer::new(std::sync::Arc::new(project));
    
    let analysis = analyzer.get_validated_workspace_patterns().unwrap();
    
    // Should have auto-detected patterns
    assert!(!analysis.auto_detected_patterns.is_empty());
    
    // Should have effective patterns that actually match packages
    assert!(!analysis.effective_patterns.is_empty());
    
    // Should have pattern statistics
    assert!(!analysis.pattern_statistics.is_empty());
    
    // Check that pattern statistics are meaningful
    for stats in &analysis.pattern_statistics {
        if stats.is_effective {
            assert!(stats.matches > 0);
        }
    }
    
    // Should not have orphaned packages with proper patterns
    // (depending on the auto-detection quality)
    println!("Orphaned packages: {:?}", analysis.orphaned_packages);
    println!("Effective patterns: {:?}", analysis.effective_patterns);
}

#[test]
fn test_pattern_specificity_calculation() {
    let (project, _temp_dir) = create_test_monorepo().unwrap();
    let analyzer = MonorepoAnalyzer::new(std::sync::Arc::new(project));
    
    // Test pattern specificity scoring
    let exact_specificity = analyzer.calculate_pattern_specificity("packages/shared");
    let wildcard_specificity = analyzer.calculate_pattern_specificity("packages/*");
    let multi_wildcard_specificity = analyzer.calculate_pattern_specificity("*/*/*");
    
    println!("Exact: {}, Wildcard: {}, Multi: {}", exact_specificity, wildcard_specificity, multi_wildcard_specificity);
    
    // More specific patterns should have higher scores
    assert!(exact_specificity > wildcard_specificity);
    assert!(wildcard_specificity > multi_wildcard_specificity);
    
    // More path components should increase specificity
    let deep_pattern = analyzer.calculate_pattern_specificity("apps/web/src/*");
    let shallow_pattern = analyzer.calculate_pattern_specificity("apps/*");
    assert!(deep_pattern > shallow_pattern);
}

#[test]
fn test_workspace_config_integration() {
    let (project, _temp_dir) = create_test_monorepo().unwrap();
    
    // Update workspace config with custom patterns
    project.config_manager.update(|config| {
        config.workspace.patterns.push(WorkspacePattern {
            pattern: "custom-packages/*".to_string(),
            description: Some("Custom package location".to_string()),
            enabled: true,
            priority: 300,
            ..Default::default()
        });
        
        config.workspace.discovery.common_patterns.push("libs/*".to_string());
    }).unwrap();
    
    let analyzer = MonorepoAnalyzer::new(std::sync::Arc::new(project));
    let patterns = analyzer.get_config_workspace_patterns().unwrap();
    
    // Should include the custom pattern from config
    assert!(patterns.contains(&"custom-packages/*".to_string()));
}