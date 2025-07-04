use sublime_monorepo_tools::{MonorepoProject, MonorepoAnalyzer, Result};
use std::time::Instant;

fn main() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempfile::TempDir::new().unwrap();
    let root_path = temp_dir.path();
    
    // Create a basic package.json for testing
    std::fs::write(
        root_path.join("package.json"), 
        r#"{"name": "test-monorepo", "version": "1.0.0", "workspaces": ["packages/*"]}"#
    ).unwrap();
    
    // Create multiple packages to test performance
    for i in 0..10 {
        let package_dir = root_path.join(format!("packages/package-{}", i));
        std::fs::create_dir_all(&package_dir).unwrap();
        std::fs::write(
            package_dir.join("package.json"),
            format!(r#"{{"name": "@test/package-{}", "version": "1.0.0"}}"#, i)
        ).unwrap();
    }
    
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

    println!("=== Performance Test ===");
    
    // Test startup performance (< 100ms target)
    let start = Instant::now();
    let project = MonorepoProject::new(root_path)?;
    let startup_time = start.elapsed();
    println!("✓ Startup time: {:?} (target: < 100ms)", startup_time);
    
    // Test analysis performance (< 1s target)
    let start = Instant::now();
    let analyzer = MonorepoAnalyzer::new(&project);
    let packages = analyzer.get_packages();
    let analysis_time = start.elapsed();
    println!("✓ Analysis time: {:?} (target: < 1s)", analysis_time);
    println!("✓ Found {} packages", packages.len());
    
    // Performance validation
    if startup_time.as_millis() < 100 {
        println!("✅ Startup performance target met");
    } else {
        println!("❌ Startup performance target missed");
    }
    
    if analysis_time.as_secs() < 1 {
        println!("✅ Analysis performance target met");
    } else {
        println!("❌ Analysis performance target missed");
    }
    
    println!("Performance test completed!");
    Ok(())
}