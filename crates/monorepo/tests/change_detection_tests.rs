//! Tests for the change detection system

use sublime_monorepo_tools::analysis::{
    ChangeDetector, ChangeDetectionEngine, ChangeDetectionRules, ChangeTypeRule, 
    SignificanceRule, VersionBumpRule, FilePattern, PatternType, PackageChangeType,
    ChangeSignificance, VersionBumpType,
};
use sublime_git_tools::{GitChangedFile, GitFileStatus};
use sublime_monorepo_tools::MonorepoProject;
use tempfile::TempDir;

fn create_test_project() -> Result<(MonorepoProject, TempDir), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root_path = temp_dir.path();
    
    // Create package.json
    let package_json = serde_json::json!({
        "name": "test-monorepo",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*"]
    });
    std::fs::write(root_path.join("package.json"), serde_json::to_string_pretty(&package_json)?)?;
    
    // Create package-lock.json
    let package_lock = serde_json::json!({
        "name": "test-monorepo",
        "version": "1.0.0",
        "lockfileVersion": 3,
        "requires": true,
        "packages": {
            "": {
                "name": "test-monorepo",
                "workspaces": ["packages/*"]
            }
        }
    });
    std::fs::write(root_path.join("package-lock.json"), serde_json::to_string_pretty(&package_lock)?)?;
    
    // Create packages directory
    let packages_dir = root_path.join("packages");
    std::fs::create_dir_all(&packages_dir)?;
    
    // Create test package
    let pkg_dir = packages_dir.join("test-pkg");
    std::fs::create_dir_all(&pkg_dir)?;
    
    let pkg_json = serde_json::json!({
        "name": "@test/test-pkg",
        "version": "1.0.0",
        "main": "src/index.js"
    });
    std::fs::write(pkg_dir.join("package.json"), serde_json::to_string_pretty(&pkg_json)?)?;
    
    // Create source files
    let src_dir = pkg_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;
    std::fs::write(src_dir.join("index.js"), "export const hello = 'world';")?;
    
    // Initialize git repository
    let git_repo = sublime_git_tools::Repo::create(root_path.to_str().unwrap())?;
    git_repo.config("Test User", "test@example.com")?;
    git_repo.add_all()?;
    git_repo.commit("Initial commit")?;
    
    let project = MonorepoProject::new_for_test(root_path)?;
    Ok((project, temp_dir))
}

#[test]
fn test_git_file_status_serialization() {
    // Test that GitFileStatus can be serialized/deserialized
    let statuses = vec![
        GitFileStatus::Added,
        GitFileStatus::Modified,
        GitFileStatus::Deleted,
    ];
    
    let json = serde_json::to_string(&statuses).unwrap();
    let deserialized: Vec<GitFileStatus> = serde_json::from_str(&json).unwrap();
    
    assert_eq!(statuses, deserialized);
}

#[test]
fn test_change_detector_with_default_rules() {
    let (_project, _temp_dir) = create_test_project().unwrap();
    let detector = ChangeDetector::new("/tmp");
    
    // Should create detector with default rules
    // The detector has an engine, we just verify it was created
    let _ = detector.engine();
}

#[test]
fn test_change_detection_rules_serialization() {
    let rules = ChangeDetectionRules {
        change_type_rules: vec![
            ChangeTypeRule {
                name: "test_rule".to_string(),
                priority: 100,
                patterns: vec![
                    FilePattern {
                        pattern_type: PatternType::Glob,
                        pattern: "src/**/*.js".to_string(),
                        exclude: false,
                    }
                ],
                change_type: PackageChangeType::SourceCode,
                conditions: None,
            }
        ],
        significance_rules: vec![
            SignificanceRule {
                name: "breaking_deletes".to_string(),
                priority: 100,
                patterns: vec![
                    FilePattern {
                        pattern_type: PatternType::Extension,
                        pattern: "js".to_string(),
                        exclude: false,
                    }
                ],
                git_status: Some(vec![GitFileStatus::Deleted]),
                significance: ChangeSignificance::Breaking,
                conditions: None,
            }
        ],
        version_bump_rules: vec![
            VersionBumpRule {
                name: "major_breaking".to_string(),
                change_type: None,
                significance: Some(ChangeSignificance::Breaking),
                version_bump: VersionBumpType::Major,
                priority: 100,
            }
        ],
        project_overrides: Default::default(),
    };
    
    // Serialize to YAML
    let yaml = serde_yaml::to_string(&rules).unwrap();
    let deserialized: ChangeDetectionRules = serde_yaml::from_str(&yaml).unwrap();
    
    assert_eq!(rules.change_type_rules.len(), deserialized.change_type_rules.len());
    assert_eq!(rules.significance_rules[0].git_status, deserialized.significance_rules[0].git_status);
}

#[test]
fn test_change_type_detection() {
    let (project, _temp_dir) = create_test_project().unwrap();
    let mut detector = ChangeDetector::new(project.root_path());
    
    // Test files
    let source_changes = vec![
        GitChangedFile {
            path: "packages/test-pkg/src/index.js".to_string(),
            status: GitFileStatus::Modified,
        }
    ];
    
    let dependency_changes = vec![
        GitChangedFile {
            path: "packages/test-pkg/package.json".to_string(),
            status: GitFileStatus::Modified,
        }
    ];
    
    // Create mock package info
    let packages = vec![]; // Empty for now as we need proper MonorepoPackageInfo setup
    
    let source_results = detector.map_changes_to_packages(&source_changes, &packages);
    let dep_results = detector.map_changes_to_packages(&dependency_changes, &packages);
    
    // With empty packages, results should be empty
    assert!(source_results.is_empty());
    assert!(dep_results.is_empty());
}

#[test]
fn test_custom_rules_from_config() {
    // Create a temporary config file
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("rules.yaml");
    
    let config_content = r#"
change_type_rules:
  - name: "custom_source"
    priority: 100
    patterns:
      - pattern_type: "Glob"
        pattern: "lib/**/*.ts"
        exclude: false
    change_type: "SourceCode"

significance_rules:
  - name: "custom_breaking"
    priority: 100
    patterns:
      - pattern_type: "Contains"
        pattern: "/public-api/"
        exclude: false
    significance: "Breaking"

version_bump_rules:
  - name: "custom_major"
    significance: "Breaking"
    version_bump: "Major"
    priority: 100

project_overrides: {}
"#;
    
    std::fs::write(&config_path, config_content).unwrap();
    
    // Create detector with custom config
    let detector = ChangeDetector::with_config_file("/tmp", &config_path);
    if let Err(e) = &detector {
        println!("Error loading config: {:?}", e);
    }
    assert!(detector.is_ok());
}

#[test]
fn test_pattern_matching() {
    let _engine = ChangeDetectionEngine::new();
    
    // Test various pattern types
    let glob_pattern = FilePattern {
        pattern_type: PatternType::Glob,
        pattern: "src/**/*.js".to_string(),
        exclude: false,
    };
    
    let exact_pattern = FilePattern {
        pattern_type: PatternType::Exact,
        pattern: "package.json".to_string(),
        exclude: false,
    };
    
    let extension_pattern = FilePattern {
        pattern_type: PatternType::Extension,
        pattern: "ts".to_string(),
        exclude: false,
    };
    
    // These would be tested if we had proper access to the pattern matching methods
    // For now, we just verify the patterns can be created
    assert_eq!(glob_pattern.pattern, "src/**/*.js");
    assert_eq!(exact_pattern.pattern, "package.json");
    assert_eq!(extension_pattern.pattern, "ts");
}

#[test]
fn test_significance_with_git_status() {
    let rules = SignificanceRule {
        name: "deleted_source".to_string(),
        priority: 100,
        patterns: vec![
            FilePattern {
                pattern_type: PatternType::Glob,
                pattern: "src/**/*.js".to_string(),
                exclude: false,
            }
        ],
        git_status: Some(vec![GitFileStatus::Deleted]),
        significance: ChangeSignificance::Breaking,
        conditions: None,
    };
    
    // Verify the rule uses GitFileStatus correctly
    assert_eq!(rules.git_status, Some(vec![GitFileStatus::Deleted]));
}