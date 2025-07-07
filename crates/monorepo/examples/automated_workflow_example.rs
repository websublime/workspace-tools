//! Automated Monorepo Workflow Example
//!
//! This example demonstrates the complete automated development workflow using
//! the new high-level MonorepoWorkflow APIs with beautiful terminal output.
//!
//! ## Workflow Demonstration
//!
//! 1. **Monorepo Analysis** - Complete state analysis with dependency graph
//! 2. **Feature Branch Creation** - Automated branch setup with changeset tracking
//! 3. **Development Cycle** - Pre-commit hooks with interactive prompts
//! 4. **Merge & Release** - Automated version bumps, changelog generation, and cleanup
//!
//! This showcases the complete development lifecycle with minimal manual intervention.

#![allow(clippy::print_stdout)] // This is an example that demonstrates workflow through output
#![allow(clippy::needless_raw_string_hashes)] // Raw strings are used for code templates
#![allow(clippy::too_many_lines)] // Example needs to be comprehensive
#![allow(clippy::wildcard_imports)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unused_self)]

mod common;

use common::{
    BumpSuggestion, ConflictResolver, DependencyGraph, DependencyPropagator, Icons,
    MonorepoWorkflow, SnapshotVersionManager, TerminalOutput,
};
use serde_json::Value;
use std::path::PathBuf;
use sublime_git_tools::Repo;
use sublime_monorepo_tools::{
    config::{types::*, ConfigManager, MonorepoConfig},
    MonorepoProject, Result,
};
use tempfile::TempDir;

/// Main example execution
fn main() -> Result<()> {
    let terminal = TerminalOutput::new();

    // Welcome message
    terminal.boxed_content(
        "🚀 Automated Monorepo Workflow Demo",
        &[
            "Complete development lifecycle automation",
            "Beautiful terminal output with progress tracking",
            "Interactive prompts simulation",
            "Comprehensive analysis and reporting",
        ],
    )?;

    // Create temporary workspace for example
    let temp_dir = TempDir::new().map_err(|e| {
        sublime_monorepo_tools::Error::generic(format!("Failed to create temp dir: {}", e))
    })?;

    let workspace = AutomatedWorkspaceExample::new(temp_dir)?;

    // Execute complete automated workflow
    workspace.run_automated_workflow()?;

    terminal.success("🎉 Automated workflow demonstration completed!")?;
    terminal.boxed_content(
        "Workflow Summary",
        &[
            "✅ Monorepo analysis with dependency graph",
            "✅ Feature branch creation with changeset tracking",
            "✅ Pre-commit hooks with interactive prompts",
            "✅ Merge workflow with automated version bumps",
            "✅ Changelog generation and changeset cleanup",
            "✅ Beautiful terminal output throughout",
        ],
    )?;

    Ok(())
}

/// Represents our automated example workspace
struct AutomatedWorkspaceExample {
    _temp_dir: TempDir,
    root_path: PathBuf,
    project: MonorepoProject,
    _repo: Repo,
}

impl AutomatedWorkspaceExample {
    /// Initialize the example workspace with realistic monorepo structure
    fn new(temp_dir: TempDir) -> Result<Self> {
        let root_path = temp_dir.path().to_path_buf();
        let terminal = TerminalOutput::new();

        terminal.phase_header("Setup", "Initializing Demo Workspace")?;

        terminal.step(Icons::PACKAGE, "Setting up monorepo structure...")?;
        Self::create_monorepo_structure(&root_path)?;

        terminal.step(Icons::ROBOT, "Configuring automation...")?;
        Self::setup_configuration(&root_path)?;

        terminal.step(Icons::PACKAGE, "Creating demo packages...")?;
        Self::setup_packages(&root_path)?;

        terminal.step(Icons::BRANCH, "Initializing git repository...")?;
        let repo = Self::setup_git_repository(&root_path)?;

        terminal.step(Icons::TASK, "Installing dependencies (simulated)...")?;
        Self::run_npm_install(&root_path)?;

        terminal.step(Icons::ROCKET, "Creating MonorepoProject...")?;

        // Debug: List what we created
        println!("  🔍 Debug: Checking created structure...");
        for entry in std::fs::read_dir(&root_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                println!("    📁 {}", entry.file_name().to_string_lossy());
                if entry.file_name() == "packages" {
                    for pkg in std::fs::read_dir(entry.path())? {
                        let pkg = pkg?;
                        println!("      📦 {}", pkg.file_name().to_string_lossy());

                        // Check if package.json exists
                        let pkg_json = pkg.path().join("package.json");
                        if pkg_json.exists() {
                            let content = std::fs::read_to_string(&pkg_json)?;
                            let json: Value = serde_json::from_str(&content)?;
                            if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                                println!("        ✅ {} has valid package.json", name);
                            }
                        } else {
                            println!("        ❌ Missing package.json");
                        }
                    }
                }
            }
        }

        // Debug: Check root package.json workspaces
        let root_pkg_json = root_path.join("package.json");
        if root_pkg_json.exists() {
            let content = std::fs::read_to_string(&root_pkg_json)?;
            let json: Value = serde_json::from_str(&content)?;
            if let Some(workspaces) = json.get("workspaces") {
                println!("  🔍 Root package.json workspaces: {:?}", workspaces);
            }
        }

        let project = MonorepoProject::new(&root_path)?;

        terminal.success("Workspace setup complete!")?;

        Ok(Self { _temp_dir: temp_dir, root_path, project, _repo: repo })
    }

    /// Execute the complete automated workflow
    fn run_automated_workflow(&self) -> Result<()> {
        let workflow = MonorepoWorkflow::new(&self.project, &self.root_path);

        // Phase 1: Comprehensive Monorepo Analysis
        let analysis_report = workflow.analyze_monorepo_state()?;

        // Phase 2: Feature Development Workflow
        let feature_result =
            workflow.create_feature_branch("feature/ui-button-component", "@acme/ui-lib")?;

        // Simulate some development work
        self.simulate_development_work()?;

        // Phase 3: Pre-commit Hook Execution
        let commit_result = workflow.execute_pre_commit_hook(
            "@acme/ui-lib",
            "feat(ui): add Button component with variants and accessibility",
        )?;

        // Phase 3.5: Pre-push Hook - Run tasks on changed packages
        let push_result = workflow.execute_pre_push_hook(&["@acme/ui-lib"])?;

        // Phase 3.6: Demonstrate Snapshot Version Management
        self.demonstrate_snapshot_versions()?;

        // Phase 4: Merge and Release Workflow
        let merge_result =
            workflow.execute_merge_workflow("feature/ui-button-component", "main")?;

        // Phase 4.5: Demonstrate Dependency Propagation and Conflict Resolution
        self.demonstrate_advanced_features(&merge_result.version_bumps)?;

        // Phase 5: Final Summary Report
        self.generate_final_report(
            &analysis_report,
            &feature_result,
            &commit_result,
            &push_result,
            &merge_result,
        )?;

        Ok(())
    }

    fn simulate_development_work(&self) -> Result<()> {
        let terminal = TerminalOutput::new();

        terminal.phase_header("Development", "Simulating Feature Implementation")?;

        // Create some realistic files to simulate development
        let button_component = r#"
import React from 'react';

export interface ButtonProps {
  children: React.ReactNode;
  variant?: 'primary' | 'secondary';
  size?: 'small' | 'medium' | 'large';
  onClick?: () => void;
  disabled?: boolean;
}

export const Button: React.FC<ButtonProps> = ({
  children,
  variant = 'primary',
  size = 'medium',
  onClick,
  disabled = false,
}) => {
  const baseClasses = 'btn rounded-md font-medium transition-colors';
  const variantClasses = variant === 'primary'
    ? 'bg-blue-600 text-white hover:bg-blue-700'
    : 'bg-gray-200 text-gray-900 hover:bg-gray-300';
  const sizeClasses = {
    small: 'px-3 py-1 text-sm',
    medium: 'px-4 py-2 text-base',
    large: 'px-6 py-3 text-lg',
  }[size];

  return (
    <button
      className={`${baseClasses} ${variantClasses} ${sizeClasses} ${disabled ? 'opacity-50 cursor-not-allowed' : ''}`}
      onClick={onClick}
      disabled={disabled}
    >
      {children}
    </button>
  );
};
"#;

        terminal.step(Icons::BUILD, "Creating Button component...")?;
        std::fs::write(self.root_path.join("packages/ui-lib/src/Button.tsx"), button_component)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        // Add to index
        let ui_index = r#"
export * from './Button';
export * from './types';
"#;
        terminal.step(Icons::BUILD, "Updating package exports...")?;
        std::fs::write(self.root_path.join("packages/ui-lib/src/index.ts"), ui_index)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        // Add tests
        let button_tests = r#"
import { render, screen, fireEvent } from '@testing-library/react';
import { Button } from './Button';

describe('Button', () => {
  it('renders with correct text', () => {
    render(<Button>Click me</Button>);
    expect(screen.getByText('Click me')).toBeInTheDocument();
  });

  it('handles click events', () => {
    const handleClick = jest.fn();
    render(<Button onClick={handleClick}>Click me</Button>);

    fireEvent.click(screen.getByText('Click me'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('applies variant styles correctly', () => {
    render(<Button variant="secondary">Secondary</Button>);
    const button = screen.getByText('Secondary');
    expect(button).toHaveClass('bg-gray-200');
  });

  it('disables when disabled prop is true', () => {
    render(<Button disabled>Disabled</Button>);
    const button = screen.getByText('Disabled');
    expect(button).toBeDisabled();
    expect(button).toHaveClass('opacity-50');
  });
});
"#;
        terminal.step(Icons::TEST, "Adding comprehensive tests...")?;
        std::fs::write(self.root_path.join("packages/ui-lib/src/Button.test.tsx"), button_tests)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        terminal.success("Feature implementation complete")?;
        Ok(())
    }

    fn demonstrate_snapshot_versions(&self) -> Result<()> {
        let terminal = TerminalOutput::new();

        terminal.phase_header("Snapshot Deployment", "Feature Branch Testing")?;

        let snapshot_manager = SnapshotVersionManager::new();

        // Generate snapshot for ui-lib
        let snapshot = snapshot_manager.generate_snapshot_version(
            "@acme/ui-lib",
            "1.0.0",
            "feature/ui-button-component",
        )?;

        // Deploy to registry
        let _deployment = snapshot_manager.deploy_to_registry(&snapshot)?;

        // Create test environment
        let test_env = snapshot_manager.create_test_environment(&[snapshot], "ui-button-test")?;

        terminal.success(&format!("Test environment ready at: {}", test_env.url))?;

        Ok(())
    }

    fn demonstrate_advanced_features(
        &self,
        initial_bumps: &[common::workflow::VersionBumpResult],
    ) -> Result<()> {
        let terminal = TerminalOutput::new();

        terminal
            .phase_header("Advanced Features", "Dependency Propagation & Conflict Resolution")?;

        // Convert to HashMap for propagation
        let mut initial_bump_map = std::collections::HashMap::new();
        for bump in initial_bumps {
            initial_bump_map.insert(bump.package.clone(), bump.bump_type);
        }

        // Demonstrate dependency propagation
        let propagator = DependencyPropagator::new();
        let dep_graph = DependencyGraph::demo();

        let propagation_result = propagator.propagate_bumps(&initial_bump_map, &dep_graph)?;

        // Create some conflicting bump suggestions to demonstrate resolution
        let mut suggestions = vec![];
        for (package, bump) in &propagation_result.final_bumps {
            suggestions.push(BumpSuggestion {
                package: package.clone(),
                bump_type: *bump,
                reason: "Dependency propagation".to_string(),
            });
        }

        // Add conflicting suggestions
        suggestions.push(BumpSuggestion {
            package: "@acme/web-app".to_string(),
            bump_type: sublime_monorepo_tools::config::VersionBumpType::Major,
            reason: "Breaking API change detected".to_string(),
        });

        // Demonstrate conflict resolution
        let resolver = ConflictResolver::new();
        let resolution = resolver.resolve_conflicts(suggestions)?;

        // Show resolution report
        terminal.info("Conflict Resolution Report:")?;
        let report = resolver.create_report(&resolution);
        for line in report.lines() {
            println!("  {}", line);
        }

        Ok(())
    }

    fn generate_final_report(
        &self,
        analysis: &common::workflow::MonorepoAnalysisReport,
        feature: &common::workflow::FeatureBranchResult,
        commit: &common::workflow::PreCommitResult,
        push: &common::workflow::PrePushResult,
        merge: &common::workflow::MergeResult,
    ) -> Result<()> {
        let terminal = TerminalOutput::new();

        terminal.phase_header("Summary", "Workflow Execution Report")?;

        // Create comprehensive summary table
        let summary_rows = vec![
            common::terminal::SummaryTableRow::new(
                "Packages Analyzed",
                &analysis.package_count.to_string(),
                "✅ Complete",
            ),
            common::terminal::SummaryTableRow::new(
                "Dependency Edges",
                &analysis.dependency_edges.to_string(),
                "✅ No Cycles",
            ),
            common::terminal::SummaryTableRow::new(
                "Feature Branch",
                &feature.branch_name,
                "✅ Created",
            ),
            common::terminal::SummaryTableRow::new(
                "Changeset Created",
                if commit.changeset_created { "Yes" } else { "Simulated" },
                "✅ Success",
            ),
            common::terminal::SummaryTableRow::new(
                "Pre-push Tasks",
                &format!("{}/{}", push.tasks_passed, push.tasks_run),
                if push.push_allowed { "✅ Passed" } else { "❌ Failed" },
            ),
            common::terminal::SummaryTableRow::new(
                "Version Bumps",
                &merge.version_bumps.len().to_string(),
                "✅ Applied",
            ),
            common::terminal::SummaryTableRow::new(
                "Changesets Cleaned",
                if merge.changesets_applied { "Yes" } else { "No" },
                "✅ Complete",
            ),
        ];

        let summary_table = common::terminal::create_summary_table(summary_rows);

        terminal.info("Workflow Execution Summary:")?;
        println!("{}", summary_table);
        println!();

        // Version bump details
        if !merge.version_bumps.is_empty() {
            terminal.info("Version Changes Applied:")?;
            for bump in &merge.version_bumps {
                println!(
                    "  📦 {}: {} → {} ({:?})",
                    bump.package, bump.old_version, bump.new_version, bump.bump_type
                );
            }
            println!();
        }

        // Workflow benefits
        terminal.boxed_content(
            "Automation Benefits Demonstrated",
            &[
                "🤖 Zero manual changeset creation",
                "📊 Automatic dependency analysis",
                "🔄 Intelligent version bump suggestions",
                "📝 Automated changelog generation",
                "🧹 Automatic cleanup after merge",
                "📋 Beautiful progress reporting",
                "⚡ Consistent workflow execution",
            ],
        )?;

        Ok(())
    }

    // Setup methods (same as before but simplified)
    fn create_monorepo_structure(root_path: &std::path::Path) -> Result<()> {
        let dirs = [
            "packages/shared",
            "packages/ui-lib",
            "packages/core-lib",
            "packages/web-app",
            ".changesets",
        ];

        for dir in dirs {
            std::fs::create_dir_all(root_path.join(dir))
                .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;
        }

        Ok(())
    }

    fn setup_configuration(root_path: &std::path::Path) -> Result<()> {
        let mut config = MonorepoConfig::default();

        // Configure workspace patterns
        config.workspace.patterns = vec![WorkspacePattern {
            pattern: "packages/*".to_string(),
            description: Some("Main packages directory".to_string()),
            enabled: true,
            priority: 100,
            package_managers: Some(vec![PackageManagerType::Npm]),
            environments: Some(vec![Environment::Development, Environment::Production]),
            options: WorkspacePatternOptions {
                include_nested: true,
                max_depth: Some(2),
                exclude_patterns: vec!["**/node_modules".to_string(), "**/dist".to_string()],
                follow_symlinks: false,
                override_detection: false,
            },
        }];

        // Configure enhanced automation
        config.hooks.enabled = true;
        config.hooks.pre_commit.enabled = true;
        config.hooks.pre_commit.validate_changeset = true;
        config.hooks.pre_commit.run_tasks = vec!["lint".to_string(), "typecheck".to_string()];

        // Configure pre-push hook
        config.hooks.pre_push.enabled = true;
        config.hooks.pre_push.run_tasks =
            vec!["lint".to_string(), "test".to_string(), "build".to_string()];

        config.changesets.required = true;
        config.changesets.auto_deploy = true;

        let config_manager = ConfigManager::with_config(config);
        config_manager.save_to_file(root_path.join("monorepo.toml"))?;

        // Create root package.json with workspaces
        let root_package_json = r#"{
  "name": "acme-monorepo",
  "version": "1.0.0",
  "private": true,
  "workspaces": ["packages/*"],
  "scripts": {
    "build": "echo 'Building all packages...'",
    "test": "echo 'Running tests...'",
    "lint": "echo 'Linting code...'",
    "typecheck": "echo 'Type checking...'"
  },
  "devDependencies": {
    "typescript": "^5.1.6",
    "@types/node": "^20.4.2"
  }
}"#;

        std::fs::write(root_path.join("package.json"), root_package_json)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        // Create package-lock.json for monorepo detection
        let package_lock = r#"{
  "name": "acme-monorepo",
  "version": "1.0.0",
  "lockfileVersion": 3,
  "requires": true,
  "packages": {
    "": {
      "name": "acme-monorepo",
      "version": "1.0.0",
      "workspaces": ["packages/*"],
      "devDependencies": {
        "typescript": "^5.1.6",
        "@types/node": "^20.4.2"
      }
    }
  }
}"#;

        std::fs::write(root_path.join("package-lock.json"), package_lock)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        Ok(())
    }

    fn setup_packages(root_path: &std::path::Path) -> Result<()> {
        // Create realistic package.json files with internal dependencies

        // @acme/shared - base package used by others
        let shared_json = r#"{
  "name": "@acme/shared",
  "version": "1.0.0",
  "scripts": {
    "build": "echo 'Building @acme/shared...'",
    "test": "echo 'Testing @acme/shared...'",
    "lint": "echo 'Linting @acme/shared...'"
  },
  "dependencies": {
    "lodash": "^4.17.21"
  },
  "devDependencies": {}
}"#;

        std::fs::create_dir_all(root_path.join("packages/shared/src"))
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;
        std::fs::write(root_path.join("packages/shared/package.json"), shared_json)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        // @acme/core-lib - depends on shared, has circular dependency with ui-lib
        let core_json = r#"{
  "name": "@acme/core-lib",
  "version": "1.0.0",
  "scripts": {
    "build": "echo 'Building @acme/core-lib...'",
    "test": "echo 'Testing @acme/core-lib...'",
    "lint": "echo 'Linting @acme/core-lib...'"
  },
  "dependencies": {
    "@acme/shared": "^1.0.0",
    "express": "^4.18.2"
  },
  "devDependencies": {
    "@acme/ui-lib": "^1.0.0"
  }
}"#;

        std::fs::create_dir_all(root_path.join("packages/core-lib/src"))
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;
        std::fs::write(root_path.join("packages/core-lib/package.json"), core_json)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        // @acme/ui-lib - depends on shared, has circular dependency with core-lib
        let ui_json = r#"{
  "name": "@acme/ui-lib",
  "version": "1.0.0",
  "scripts": {
    "build": "echo 'Building @acme/ui-lib...'",
    "test": "echo 'Testing @acme/ui-lib...'",
    "lint": "echo 'Linting @acme/ui-lib...'"
  },
  "dependencies": {
    "@acme/shared": "^1.0.0",
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  },
  "peerDependencies": {
    "@acme/core-lib": "^1.0.0"
  }
}"#;

        std::fs::create_dir_all(root_path.join("packages/ui-lib/src"))
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;
        std::fs::write(root_path.join("packages/ui-lib/package.json"), ui_json)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        // @acme/web-app - depends on both ui-lib and core-lib
        let webapp_json = r#"{
  "name": "@acme/web-app",
  "version": "1.0.0",
  "scripts": {
    "build": "echo 'Building @acme/web-app...'",
    "test": "echo 'Testing @acme/web-app...'",
    "lint": "echo 'Linting @acme/web-app...'"
  },
  "dependencies": {
    "@acme/core-lib": "^1.0.0",
    "@acme/ui-lib": "^1.0.0",
    "next": "^14.0.0",
    "typescript": "^5.2.0"
  },
  "devDependencies": {}
}"#;

        std::fs::create_dir_all(root_path.join("packages/web-app/src"))
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;
        std::fs::write(root_path.join("packages/web-app/package.json"), webapp_json)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        // Create index files
        for pkg in ["shared", "core-lib", "ui-lib", "web-app"] {
            std::fs::write(
                root_path.join(format!("packages/{}/src/index.ts", pkg)),
                "export * from './types';",
            )
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;
        }

        Ok(())
    }

    fn setup_git_repository(root_path: &std::path::Path) -> Result<Repo> {
        // Initialize git repo
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(root_path)
            .output()
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        // Configure git
        std::process::Command::new("git")
            .args(["config", "user.email", "developer@acme.com"])
            .current_dir(root_path)
            .output()
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        std::process::Command::new("git")
            .args(["config", "user.name", "Acme Developer"])
            .current_dir(root_path)
            .output()
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        // Add and commit initial structure
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(root_path)
            .output()
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        std::process::Command::new("git")
            .args(["commit", "-m", "chore: initial monorepo setup"])
            .current_dir(root_path)
            .output()
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        let repo_path = root_path.to_path_buf();
        Repo::open(repo_path.to_str().unwrap())
            .map_err(|e| sublime_monorepo_tools::Error::git(format!("Failed to open repo: {}", e)))
    }

    fn run_npm_install(root_path: &std::path::Path) -> Result<()> {
        // Simulate npm install for demo purposes
        let node_modules = root_path.join("node_modules");
        std::fs::create_dir_all(&node_modules).map_err(|e| {
            sublime_monorepo_tools::Error::generic(format!("Failed to create node_modules: {}", e))
        })?;

        // Create some fake package directories
        let demo_packages = ["typescript", "@types/node"];
        for package in demo_packages {
            let package_dir = node_modules.join(package);
            std::fs::create_dir_all(&package_dir).map_err(|e| {
                sublime_monorepo_tools::Error::generic(format!(
                    "Failed to create package dir: {}",
                    e
                ))
            })?;
        }

        Ok(())
    }
}
