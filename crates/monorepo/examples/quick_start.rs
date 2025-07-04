use sublime_monorepo_tools::{MonorepoProject, MonorepoAnalyzer, Result};

fn main() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempfile::TempDir::new().unwrap();
    let root_path = temp_dir.path();
    
    // Create a basic package.json for testing
    std::fs::write(
        root_path.join("package.json"), 
        r#"{"name": "test-monorepo", "version": "1.0.0", "workspaces": ["packages/*"]}"#
    ).unwrap();
    
    // Create a packages directory with a test package
    std::fs::create_dir_all(root_path.join("packages/core")).unwrap();
    std::fs::write(
        root_path.join("packages/core/package.json"),
        r#"{"name": "@test/core", "version": "1.0.0"}"#
    ).unwrap();
    
    // Initialize git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root_path)
        .output()
        .unwrap();
        
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(root_path)
        .output()
        .unwrap();
        
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(root_path)
        .output()
        .unwrap();
        
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root_path)
        .output()
        .unwrap();
        
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(root_path)
        .output()
        .unwrap();

    // Initialize project with direct base crate integration
    let project = MonorepoProject::new(root_path)?;
    
    // Perform analysis with sub-second performance
    let analyzer = MonorepoAnalyzer::new(&project);
    let packages = analyzer.get_packages();
    
    println!("Found {} packages", packages.len());
    for package in packages {
        println!("- {}: {}", package.name(), package.version());
    }
    
    println!("Quick start example completed successfully!");
    Ok(())
}