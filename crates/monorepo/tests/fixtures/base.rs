use std::fs;
use std::path::Path;
use sublime_git_tools::Repo;
use tempfile::TempDir;

use crate::fixtures::{EMAIL, USERNAME};

/// Create the basic monorepo structure with git initialization
pub fn create_monorepo_base() -> TempDir {
    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create root package.json
    let package_json = r#"
{
"name": "root",
"version": "0.0.0",
"workspaces": [
  "packages/package-foo",
  "packages/package-bar",
  "packages/package-baz",
  "packages/package-charlie",
  "packages/package-major",
  "packages/package-tom"
  ]
}
"#;

    // Create .config.toml
    let config_toml = r#"
[tools]
git_user_name="bot"
"#;

    // Create .gitattributes
    let gitattributes = "* text=auto\n";

    // Create packages directory
    let packages_dir = temp_path.join("packages");
    fs::create_dir_all(&packages_dir).unwrap();

    // Write root files
    fs::write(temp_path.join("package.json"), package_json.trim()).unwrap();
    fs::write(temp_path.join(".config.toml"), config_toml.trim()).unwrap();
    fs::write(temp_path.join(".gitattributes"), gitattributes).unwrap();

    // Initialize git repository and configure
    let repo = Repo::create(temp_path.to_str().unwrap()).expect("Failed to create git repo");
    repo.config(USERNAME, EMAIL).expect("Failed to configure git");

    // Add files and commit
    repo.add_all().expect("Failed to add files");
    repo.commit("chore: init monorepo workspace").expect("Failed to commit");

    temp_dir
}

/// Helper function to ensure directory exists
pub fn ensure_dir(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Helper to write file with parent directory creation
pub fn write_file(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    fs::write(path, content)
}

/// Creates a complete monorepo with all packages
pub fn create_complete_monorepo(package_manager: Option<&str>) -> TempDir {
    let temp_dir = create_monorepo_base();
    let repo_path = temp_dir.path();

    // Add package manager files if specified
    if let Some(manager) = package_manager {
        match manager {
            "npm" => crate::fixtures::add_npm_manager(repo_path).unwrap(),
            "yarn" => crate::fixtures::add_yarn_manager(repo_path).unwrap(),
            "pnpm" => crate::fixtures::add_pnpm_manager(repo_path).unwrap(),
            "bun" => crate::fixtures::add_bun_manager(repo_path).unwrap(),
            _ => panic!("Unsupported package manager: {manager}"),
        }
    }

    // Create all packages
    crate::fixtures::create_package_foo(repo_path).unwrap();
    crate::fixtures::create_package_bar(repo_path).unwrap();
    crate::fixtures::create_package_baz(repo_path).unwrap();
    crate::fixtures::create_package_charlie(repo_path).unwrap();
    crate::fixtures::create_package_major(repo_path).unwrap();
    crate::fixtures::create_package_tom(repo_path).unwrap();

    temp_dir
}

#[allow(dead_code)]
#[allow(clippy::print_stdout)]
pub fn verify_fixtures(temp_dir: &TempDir) {
    let path = temp_dir.path();
    println!("Fixture verification:");

    // Verify all expected packages exist
    let packages = [
        "package-foo",
        "package-bar",
        "package-baz",
        "package-charlie",
        "package-major",
        "package-tom",
    ];

    for pkg in packages {
        let pkg_dir = path.join("packages").join(pkg);
        let pkg_json = pkg_dir.join("package.json");

        println!(
            "Package {}: dir exists={}, package.json exists={}",
            pkg,
            pkg_dir.exists(),
            pkg_json.exists()
        );

        // If package.json exists, print its content
        if pkg_json.exists() {
            match fs::read_to_string(&pkg_json) {
                Ok(content) => {
                    println!("  package.json content: {content}");
                }
                Err(e) => {
                    println!("  Error reading package.json: {e}");
                }
            }
        }
    }
}
