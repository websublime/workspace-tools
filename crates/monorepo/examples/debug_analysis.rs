use sublime_monorepo_tools::{MonorepoProject, MonorepoAnalyzer, Result};

fn main() -> Result<()> {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let root_path = temp_dir.path();

    // Create the same structure as analysis tests
    std::fs::write(
        root_path.join("package.json"), 
        r#"{"name": "test-monorepo", "version": "1.0.0", "workspaces": ["packages/*", "apps/*"]}"#
    ).unwrap();

    // Create packages
    std::fs::create_dir_all(root_path.join("packages/core")).unwrap();
    std::fs::create_dir_all(root_path.join("packages/utils")).unwrap();
    std::fs::create_dir_all(root_path.join("apps/web")).unwrap();

    // Core package with external dependency
    let core_package_json = r#"{
        "name": "@test/core",
        "version": "1.0.0",
        "dependencies": {
            "lodash": "^4.17.21"
        }
    }"#;
    std::fs::write(root_path.join("packages/core/package.json"), core_package_json).unwrap();

    // Utils package with internal dependency
    let utils_package_json = r#"{
        "name": "@test/utils",
        "version": "1.0.0",
        "dependencies": {
            "@test/core": "^1.0.0"
        }
    }"#;
    std::fs::write(root_path.join("packages/utils/package.json"), utils_package_json).unwrap();

    // Web package with mixed dependencies
    let web_package_json = r#"{
        "name": "@test/web",
        "version": "1.0.0",
        "dependencies": {
            "@test/core": "^1.0.0",
            "@test/utils": "^1.0.0"
        }
    }"#;
    std::fs::write(root_path.join("apps/web/package.json"), web_package_json).unwrap();

    // Initialize git
    std::process::Command::new("git").args(["init"]).current_dir(root_path).output().unwrap();
    std::process::Command::new("git").args(["config", "user.email", "test@example.com"]).current_dir(root_path).output().unwrap();
    std::process::Command::new("git").args(["config", "user.name", "Test User"]).current_dir(root_path).output().unwrap();
    std::process::Command::new("git").args(["add", "."]).current_dir(root_path).output().unwrap();
    std::process::Command::new("git").args(["commit", "-m", "Initial commit"]).current_dir(root_path).output().unwrap();

    // Test discovery
    let project = MonorepoProject::new(root_path)?;
    let analyzer = MonorepoAnalyzer::new(&project);
    let packages = analyzer.get_packages();
    
    println!("Discovered {} packages", packages.len());
    
    for package in packages.iter() {
        println!("- {}: {}", package.name(), package.version());
        println!("  External deps: {:?}", package.dependencies_external);
    }

    // Test analyzer
    let analyzer = MonorepoAnalyzer::new(&project);
    let classification = analyzer.classify_packages()?;
    
    println!("\nClassification:");
    println!("Internal packages: {}", classification.internal_packages.len());
    println!("External dependencies: {}", classification.external_dependencies.len());
    println!("External deps: {:?}", classification.external_dependencies);

    Ok(())
}