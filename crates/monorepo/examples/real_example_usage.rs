//! Real-world monorepo workflow example
//!
//! This example demonstrates a complete development workflow using sublime_monorepo_tools.
//! It implements a realistic scenario with feature development, change detection,
//! dependency management, and version propagation across multiple packages.
//!
//! ## Scenario Overview
//!
//! We have a monorepo with 4 packages:
//! - `@acme/shared` - Shared utilities and types
//! - `@acme/ui-lib` - UI component library
//! - `@acme/core-lib` - Core business logic library
//! - `@acme/web-app` - Main web application
//!
//! ## Workflow Steps
//!
//! 1. **UI Feature**: Add new Button component to ui-lib
//! 2. **App Integration**: Use new Button in web-app
//! 3. **Shared Enhancement**: Add new utility to shared package
//! 4. **Library Update**: Use new shared utility in core-lib
//! 5. **Dependency Audit**: Find and upgrade outdated external dependencies
//! 6. **Version Propagation**: Apply semantic versioning across packages
//! 7. **Changelog Generation**: Generate changelogs for all affected packages

#![allow(clippy::print_stdout)] // This is an example that demonstrates workflow through output
#![allow(clippy::needless_raw_string_hashes)] // Raw strings are used for code templates
#![allow(clippy::too_many_lines)] // Example needs to be comprehensive
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::wildcard_imports)]

use serde_json::Value;
use std::path::{Path, PathBuf};
use sublime_git_tools::Repo;
use sublime_monorepo_tools::{
    changes::ChangeDetectionEngine,
    config::{types::*, ConfigManager, MonorepoConfig, VersionBumpType},
    MonorepoAnalyzer, MonorepoProject, Result,
};
use tempfile::TempDir;

/// Main example execution
fn main() -> Result<()> {
    println!("🚀 Starting Real-World Monorepo Workflow Example");
    println!("================================================\n");

    // Create temporary workspace for example
    let temp_dir = TempDir::new().map_err(|e| {
        sublime_monorepo_tools::Error::generic(format!("Failed to create temp dir: {}", e))
    })?;

    let workspace = WorkflowExample::new(temp_dir)?;

    // Execute complete workflow
    workspace.run_complete_workflow()?;

    println!("✅ Workflow completed successfully!");
    println!("This example demonstrated:");
    println!("  - Change detection and analysis");
    println!("  - Dependency graph management");
    println!("  - Version bump propagation");
    println!("  - External dependency auditing");
    println!("  - Multi-package coordination");

    Ok(())
}

/// Represents our example monorepo workspace
#[allow(dead_code)]
struct WorkflowExample {
    temp_dir: TempDir,
    root_path: PathBuf,
    project: MonorepoProject,
    repo: Repo,
}

impl WorkflowExample {
    /// Initialize the example workspace with realistic monorepo structure
    fn new(temp_dir: TempDir) -> Result<Self> {
        let root_path = temp_dir.path().to_path_buf();

        println!("📁 Setting up monorepo structure...");
        Self::create_monorepo_structure(&root_path)?;

        println!("⚙️  Initializing configuration...");
        Self::setup_configuration(&root_path)?;

        println!("📦 Setting up packages...");
        Self::setup_packages(&root_path)?;

        println!("🔧 Initializing git repository...");
        let repo = Self::setup_git_repository(&root_path)?;

        println!("🏗️  Creating MonorepoProject...");
        let project = MonorepoProject::new(&root_path)?;

        println!("✅ Workspace setup complete!\n");

        Ok(Self { temp_dir, root_path, project, repo })
    }

    /// Execute the complete development workflow
    fn run_complete_workflow(&self) -> Result<()> {
        println!("🎯 Starting Development Workflow");
        println!("================================\n");

        // Step 1: Initial analysis
        self.step_1_initial_analysis()?;

        // Step 2: UI feature development
        self.step_2_ui_feature_development()?;

        // Step 3: App integration
        self.step_3_app_integration()?;

        // Step 4: Shared utility enhancement
        self.step_4_shared_enhancement()?;

        // Step 5: Library update
        self.step_5_library_update()?;

        // Step 6: Dependency audit and upgrades
        self.step_6_dependency_audit()?;

        // Step 7: Version propagation and changelog
        self.step_7_version_propagation()?;

        Ok(())
    }

    /// Step 1: Analyze current monorepo state
    fn step_1_initial_analysis(&self) -> Result<()> {
        println!("📊 Step 1: Initial Monorepo Analysis");
        println!("====================================");

        let analyzer = MonorepoAnalyzer::new(&self.project);

        // Analyze monorepo structure
        let monorepo_info = analyzer.detect_monorepo_info(&self.root_path)?;
        println!("📦 Found {} packages:", monorepo_info.packages.internal_packages.len());
        for pkg in &monorepo_info.packages.internal_packages {
            println!("  - {} v{}", pkg.name, pkg.version);
        }

        // Analyze package manager
        let pm_analysis = analyzer.analyze_package_manager()?;
        println!("📋 Package Manager: {:?}", pm_analysis.kind);
        println!("🔒 Lock file: {}", pm_analysis.lock_file.display());

        // Build dependency graph
        let dep_graph = analyzer.build_dependency_graph()?;
        println!(
            "🕸️  Dependency Graph: {} nodes, {} edges",
            dep_graph.node_count, dep_graph.edge_count
        );

        // Analyze workspace configuration
        let workspace_analysis = analyzer.analyze_workspace_config()?;
        println!("🎯 Workspace patterns: {:?}", workspace_analysis.patterns);
        println!("✅ Matched packages: {}", workspace_analysis.matched_packages);

        // Check for external dependencies
        let external_deps = self.project.external_dependencies();
        println!("📚 External dependencies: {}", external_deps.len());
        for dep in external_deps.iter().take(3) {
            println!("  - {}", dep);
        }

        println!("✅ Initial analysis complete\n");
        Ok(())
    }

    /// Step 2: Develop new UI component feature
    fn step_2_ui_feature_development(&self) -> Result<()> {
        println!("🎨 Step 2: UI Feature Development");
        println!("=================================");

        // Create feature branch
        println!("🌿 Creating feature branch: feature/ui-button-component");
        self.create_git_branch("feature/ui-button-component")?;

        // Add Button component
        let button_component = "
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
  disabled = false
}) => {
  return (
    <button
      className={`btn btn-${variant} btn-${size}`}
      onClick={onClick}
      disabled={disabled}
    >
      {children}
    </button>
  );
};
";

        self.create_file("packages/ui-lib/src/components/Button.tsx", button_component)?;

        // Update ui-lib index
        let ui_index = "
export { Button } from './components/Button';
export type { ButtonProps } from './components/Button';
";
        self.update_file("packages/ui-lib/src/index.ts", ui_index)?;

        // Add tests
        let button_tests = "
import { render, screen, fireEvent } from '@testing-library/react';
import { Button } from './Button';

describe('Button', () => {
  it('renders with children', () => {
    render(<Button>Click me</Button>);
    expect(screen.getByText('Click me')).toBeInTheDocument();
  });

  it('calls onClick when clicked', () => {
    const handleClick = jest.fn();
    render(<Button onClick={handleClick}>Click me</Button>);

    fireEvent.click(screen.getByText('Click me'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('respects disabled prop', () => {
    const handleClick = jest.fn();
    render(<Button onClick={handleClick} disabled>Click me</Button>);

    fireEvent.click(screen.getByText('Click me'));
    expect(handleClick).not.toHaveBeenCalled();
  });
});
";
        self.create_file("packages/ui-lib/src/components/Button.test.tsx", button_tests)?;

        // Commit changes
        self.git_add_and_commit("feat(ui-lib): add Button component with variants and tests")?;

        // Analyze changes
        self.analyze_changes("UI Button component feature")?;

        // Simulate build and test
        self.run_build_and_test("packages/ui-lib")?;

        println!("✅ UI feature development complete\n");
        Ok(())
    }

    /// Step 3: Integrate UI component in app
    fn step_3_app_integration(&self) -> Result<()> {
        println!("🔗 Step 3: App Integration");
        println!("==========================");

        // Merge UI feature to main
        self.checkout_branch("main")?;
        self.merge_branch("feature/ui-button-component")?;

        // Create new feature branch for app
        println!("🌿 Creating feature branch: feature/app-use-button");
        self.create_git_branch("feature/app-use-button")?;

        // Update app to use new Button
        let app_component = r#"
import React from 'react';
import { Button } from '@acme/ui-lib';
import { useAppLogic } from '@acme/core-lib';

export const App: React.FC = () => {
  const { user, handleLogin, handleLogout } = useAppLogic();

  return (
    <div className="app">
      <header>
        <h1>Acme Web Application</h1>
      </header>

      <main>
        {user ? (
          <div>
            <p>Welcome back, {user.name}!</p>
            <Button
              variant="secondary"
              onClick={handleLogout}
            >
              Logout
            </Button>
          </div>
        ) : (
          <div>
            <p>Please log in to continue</p>
            <Button
              variant="primary"
              size="large"
              onClick={handleLogin}
            >
              Login
            </Button>
          </div>
        )}
      </main>
    </div>
  );
};
"#;

        self.update_file("packages/web-app/src/App.tsx", app_component)?;

        // Update package dependencies
        self.update_package_dependency("packages/web-app", "@acme/ui-lib", "^1.1.0")?;

        // Commit changes
        self.git_add_and_commit("feat(web-app): integrate new Button component from ui-lib")?;

        // Analyze impact
        self.analyze_changes("App integration with new Button")?;

        // Test integration
        self.run_integration_tests()?;

        println!("✅ App integration complete\n");
        Ok(())
    }

    /// Step 4: Add shared utility enhancement
    fn step_4_shared_enhancement(&self) -> Result<()> {
        println!("🛠️  Step 4: Shared Utility Enhancement");
        println!("======================================");

        // Merge app feature to main
        self.checkout_branch("main")?;
        self.merge_branch("feature/app-use-button")?;

        // Create feature branch for shared enhancement
        println!("🌿 Creating feature branch: feature/shared-validation-utils");
        self.create_git_branch("feature/shared-validation-utils")?;

        // Add validation utilities
        let validation_utils = r"
/**
 * Validation utilities for form inputs and user data
 */

export interface ValidationResult {
  isValid: boolean;
  errors: string[];
}

export const validateEmail = (email: string): ValidationResult => {
  const errors: string[] = [];

  if (!email) {
    errors.push('Email is required');
  } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
    errors.push('Invalid email format');
  }

  return {
    isValid: errors.length === 0,
    errors
  };
};

export const validatePassword = (password: string): ValidationResult => {
  const errors: string[] = [];

  if (!password) {
    errors.push('Password is required');
  } else {
    if (password.length < 8) {
      errors.push('Password must be at least 8 characters');
    }
    if (!/(?=.*[a-z])/.test(password)) {
      errors.push('Password must contain lowercase letter');
    }
    if (!/(?=.*[A-Z])/.test(password)) {
      errors.push('Password must contain uppercase letter');
    }
    if (!/(?=.*\d)/.test(password)) {
      errors.push('Password must contain a number');
    }
  }

  return {
    isValid: errors.length === 0,
    errors
  };
};

export const validateForm = <T extends Record<string, any>>(
  data: T,
  validators: Record<keyof T, (value: any) => ValidationResult>
): ValidationResult => {
  const allErrors: string[] = [];

  for (const [field, validator] of Object.entries(validators)) {
    const result = validator(data[field]);
    if (!result.isValid) {
      allErrors.push(...result.errors.map(err => `${field}: ${err}`));
    }
  }

  return {
    isValid: allErrors.length === 0,
    errors: allErrors
  };
};
";

        self.create_file("packages/shared/src/validation.ts", validation_utils)?;

        // Update shared index
        let shared_index = "
export * from './types';
export * from './utils';
export * from './validation';
";
        self.update_file("packages/shared/src/index.ts", shared_index)?;

        // Add comprehensive tests
        let validation_tests = "
import { validateEmail, validatePassword, validateForm } from './validation';

describe('Validation Utils', () => {
  describe('validateEmail', () => {
    it('validates correct email', () => {
      const result = validateEmail('user@example.com');
      expect(result.isValid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('rejects invalid email', () => {
      const result = validateEmail('invalid-email');
      expect(result.isValid).toBe(false);
      expect(result.errors).toContain('Invalid email format');
    });

    it('rejects empty email', () => {
      const result = validateEmail('');
      expect(result.isValid).toBe(false);
      expect(result.errors).toContain('Email is required');
    });
  });

  describe('validatePassword', () => {
    it('validates strong password', () => {
      const result = validatePassword('SecurePass123');
      expect(result.isValid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('rejects weak password', () => {
      const result = validatePassword('weak');
      expect(result.isValid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });
  });

  describe('validateForm', () => {
    it('validates complete form', () => {
      const formData = {
        email: 'user@example.com',
        password: 'SecurePass123'
      };

      const validators = {
        email: validateEmail,
        password: validatePassword
      };

      const result = validateForm(formData, validators);
      expect(result.isValid).toBe(true);
    });
  });
});
";
        self.create_file("packages/shared/src/validation.test.ts", validation_tests)?;

        // Commit changes
        self.git_add_and_commit("feat(shared): add comprehensive validation utilities")?;

        // Analyze changes
        self.analyze_changes("Shared validation utilities")?;

        // Test shared package
        self.run_build_and_test("packages/shared")?;

        println!("✅ Shared enhancement complete\n");
        Ok(())
    }

    /// Step 5: Update core library to use shared utilities
    fn step_5_library_update(&self) -> Result<()> {
        println!("📚 Step 5: Core Library Update");
        println!("==============================");

        // Merge shared feature to main
        self.checkout_branch("main")?;
        self.merge_branch("feature/shared-validation-utils")?;

        // Create feature branch for core lib update
        println!("🌿 Creating feature branch: feature/core-use-validation");
        self.create_git_branch("feature/core-use-validation")?;

        // Update core lib to use validation
        let auth_service = "
import { validateEmail, validatePassword, validateForm } from '@acme/shared';

export interface User {
  id: string;
  name: string;
  email: string;
}

export interface LoginCredentials {
  email: string;
  password: string;
}

export class AuthService {
  private users: User[] = [];
  private currentUser: User | null = null;

  async login(credentials: LoginCredentials): Promise<User> {
    // Validate credentials using shared utilities
    const validationResult = validateForm(credentials, {
      email: validateEmail,
      password: validatePassword
    });

    if (!validationResult.isValid) {
      throw new Error(`Validation failed: ${validationResult.errors.join(', ')}`);
    }

    // Simulate authentication
    const user = this.users.find(u => u.email === credentials.email);
    if (!user) {
      throw new Error('User not found');
    }

    this.currentUser = user;
    return user;
  }

  async logout(): Promise<void> {
    this.currentUser = null;
  }

  getCurrentUser(): User | null {
    return this.currentUser;
  }

  async registerUser(userData: Omit<User, 'id'> & { password: string }): Promise<User> {
    // Validate user data
    const emailValidation = validateEmail(userData.email);
    const passwordValidation = validatePassword(userData.password);

    if (!emailValidation.isValid) {
      throw new Error(`Email validation failed: ${emailValidation.errors.join(', ')}`);
    }

    if (!passwordValidation.isValid) {
      throw new Error(`Password validation failed: ${passwordValidation.errors.join(', ')}`);
    }

    const newUser: User = {
      id: `user_${Date.now()}`,
      name: userData.name,
      email: userData.email
    };

    this.users.push(newUser);
    return newUser;
  }
}
";

        self.create_file("packages/core-lib/src/auth.ts", auth_service)?;

        // Update core lib index
        let core_index = "
export * from './types';
export * from './auth';
export * from './hooks';
";
        self.update_file("packages/core-lib/src/index.ts", core_index)?;

        // Update package dependencies
        self.update_package_dependency("packages/core-lib", "@acme/shared", "^1.2.0")?;

        // Add tests
        let auth_tests = "
import { AuthService } from './auth';

describe('AuthService', () => {
  let authService: AuthService;

  beforeEach(() => {
    authService = new AuthService();
  });

  it('registers user with valid data', async () => {
    const userData = {
      name: 'John Doe',
      email: 'john@example.com',
      password: 'SecurePass123'
    };

    const user = await authService.registerUser(userData);
    expect(user.name).toBe('John Doe');
    expect(user.email).toBe('john@example.com');
    expect(user.id).toBeDefined();
  });

  it('rejects user with invalid email', async () => {
    const userData = {
      name: 'John Doe',
      email: 'invalid-email',
      password: 'SecurePass123'
    };

    await expect(authService.registerUser(userData)).rejects.toThrow('Email validation failed');
  });

  it('rejects user with weak password', async () => {
    const userData = {
      name: 'John Doe',
      email: 'john@example.com',
      password: 'weak'
    };

    await expect(authService.registerUser(userData)).rejects.toThrow('Password validation failed');
  });
});
";
        self.create_file("packages/core-lib/src/auth.test.ts", auth_tests)?;

        // Commit changes
        self.git_add_and_commit(
            "feat(core-lib): integrate shared validation utilities in auth service",
        )?;

        // Analyze changes and impact
        self.analyze_changes("Core library validation integration")?;
        self.analyze_dependency_impact()?;

        // Test core library
        self.run_build_and_test("packages/core-lib")?;

        println!("✅ Core library update complete\n");
        Ok(())
    }

    /// Step 6: Audit and upgrade external dependencies
    fn step_6_dependency_audit(&self) -> Result<()> {
        println!("🔍 Step 6: Dependency Audit & Upgrades");
        println!("=======================================");

        // Merge core lib feature to main
        self.checkout_branch("main")?;
        self.merge_branch("feature/core-use-validation")?;

        let analyzer = MonorepoAnalyzer::new(&self.project);

        // Analyze current dependencies
        println!("📊 Analyzing external dependencies...");
        let upgrade_analysis = analyzer.analyze_available_upgrades()?;
        println!("📦 Total packages analyzed: {}", upgrade_analysis.total_packages);
        println!("🔄 Packages with available upgrades: {}", upgrade_analysis.upgradable_count);

        // Simulate finding outdated dependencies
        let outdated_deps = vec![
            ("react", "17.0.2", "18.2.0"),
            ("typescript", "4.8.4", "5.1.6"),
            ("@types/node", "18.15.0", "20.4.2"),
        ];

        for (dep, current, latest) in &outdated_deps {
            println!("📅 {} {} → {} (upgrade available)", dep, current, latest);

            // Create individual upgrade branch
            let branch_name = format!("deps/upgrade-{}", dep.replace('/', "-").replace('@', ""));
            println!("🌿 Creating upgrade branch: {}", branch_name);
            self.create_git_branch(&branch_name)?;

            // Update dependency in all affected packages
            self.upgrade_dependency_across_packages(dep, latest)?;

            // Commit upgrade
            let commit_msg = format!("deps: upgrade {} from {} to {}", dep, current, latest);
            self.git_add_and_commit(&commit_msg)?;

            // Test upgrade
            self.test_dependency_upgrade(dep)?;

            // Merge back to main
            self.checkout_branch("main")?;
            self.merge_branch(&branch_name)?;

            println!("✅ {} upgrade completed", dep);
        }

        // Analyze security and registry status
        self.analyze_registry_status()?;

        println!("✅ Dependency audit complete\n");
        Ok(())
    }

    /// Step 7: Version propagation and changelog generation
    fn step_7_version_propagation(&self) -> Result<()> {
        println!("📈 Step 7: Version Propagation & Changelog");
        println!("==========================================");

        let _analyzer = MonorepoAnalyzer::new(&self.project);

        // Analyze all changes since last release
        println!("📊 Analyzing changes for version bumps...");

        let _change_engine = ChangeDetectionEngine::new();

        // Simulate version bump analysis for each package
        let version_bumps = vec![
            ("@acme/shared", VersionBumpType::Minor, "Added validation utilities"),
            ("@acme/ui-lib", VersionBumpType::Minor, "Added Button component"),
            ("@acme/core-lib", VersionBumpType::Minor, "Integrated validation in auth service"),
            (
                "@acme/web-app",
                VersionBumpType::Patch,
                "Updated to use new components and validation",
            ),
        ];

        for (package, bump_type, reason) in &version_bumps {
            println!("📦 {}: {:?} bump ({})", package, bump_type, reason);
        }

        // Simulate changelog generation
        self.generate_changelogs(&version_bumps)?;

        // Show dependency impact analysis
        self.show_version_impact_analysis()?;

        // Create release summary
        self.create_release_summary()?;

        println!("✅ Version propagation complete\n");
        Ok(())
    }

    // === Helper Methods ===

    /// Create the basic monorepo directory structure
    fn create_monorepo_structure(root_path: &Path) -> Result<()> {
        let dirs = [
            "packages/shared/src",
            "packages/ui-lib/src/components",
            "packages/core-lib/src",
            "packages/web-app/src",
            ".github/workflows",
            "docs",
        ];

        for dir in dirs {
            std::fs::create_dir_all(root_path.join(dir))
                .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;
        }

        Ok(())
    }

    /// Setup monorepo configuration
    fn setup_configuration(root_path: &Path) -> Result<()> {
        // Create comprehensive custom configuration demonstrating all features
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

        // Configure file patterns for better change detection
        config.workspace.file_patterns.source_patterns = vec![
            "src/**/*.{ts,tsx,js,jsx}".to_string(),
            "lib/**/*.{ts,tsx,js,jsx}".to_string(),
            "components/**/*.{ts,tsx,js,jsx}".to_string(),
        ];

        config.workspace.file_patterns.test_patterns = vec![
            "**/*.{test,spec}.{ts,tsx,js,jsx}".to_string(),
            "**/__tests__/**/*.{ts,tsx,js,jsx}".to_string(),
            "**/tests/**/*.{ts,tsx,js,jsx}".to_string(),
        ];

        // Configure validation rules
        config.validation.change_detection_rules.significance_priorities.public_api_changes = 100;
        config.validation.change_detection_rules.significance_priorities.internal_changes = 80;
        config.validation.version_bump_rules.breaking_changes_priority = 100;
        config.validation.version_bump_rules.feature_changes_priority = 80;

        // Configure versioning with snapshot format including git hash
        config.versioning.snapshot_format = "{version}-snapshot.{sha}".to_string();
        config.versioning.auto_tag = true;
        config.versioning.tag_prefix = "v".to_string();
        config.versioning.propagate_changes = true;

        // Configure task groups
        config.tasks.default_tasks =
            vec!["lint".to_string(), "typecheck".to_string(), "test".to_string()];
        config.tasks.parallel = true;
        config.tasks.max_concurrent = 4;
        config.tasks.timeout = 300;

        // Configure hook automation
        config.hooks.enabled = true;
        config.hooks.pre_commit.enabled = true;
        config.hooks.pre_commit.validate_changeset = true;
        config.hooks.pre_commit.run_tasks = vec!["lint".to_string(), "typecheck".to_string()];
        config.hooks.pre_push.enabled = true;
        config.hooks.pre_push.run_tasks = vec!["test".to_string(), "build".to_string()];

        // Configure changesets
        config.changesets.required = true;
        config.changesets.changeset_dir = ".changesets".to_string().into();
        config.changesets.auto_deploy = true;

        // Configure quality gates
        config.validation.quality_gates.min_test_coverage = 80.0;
        config.validation.quality_gates.max_cyclomatic_complexity = 10;
        config.validation.quality_gates.max_dependencies_per_package = 50;

        let config_manager = ConfigManager::with_config(config);
        config_manager.save_to_file(root_path.join("monorepo.toml"))?;

        println!("📝 Custom configuration created with:");
        println!("  - Workspace patterns for packages/*");
        println!("  - Enhanced file pattern detection");
        println!("  - Custom validation priorities");
        println!("  - Snapshot versioning with git hash");
        println!("  - Git hooks automation (pre-commit, pre-push)");
        println!("  - Changesets management");
        println!("  - Quality gates with coverage thresholds");

        // Create root package.json
        let root_package_json = "{
  \"name\": \"acme-monorepo\",
  \"version\": \"1.0.0\",
  \"private\": true,
  \"workspaces\": [\"packages/*\"],
  \"scripts\": {
    \"build\": \"npm run build:packages\",
    \"build:packages\": \"lerna run build\",
    \"test\": \"lerna run test\",
    \"lint\": \"lerna run lint\",
    \"clean\": \"lerna run clean\",
    \"publish\": \"lerna publish\"
  },
  \"devDependencies\": {
    \"@lerna/cli\": \"^7.1.4\",
    \"typescript\": \"^5.1.6\",
    \"@types/node\": \"^20.4.2\",
    \"jest\": \"^29.6.1\",
    \"@testing-library/react\": \"^13.4.0\",
    \"eslint\": \"^8.45.0\",
    \"prettier\": \"^3.0.0\"
  }
}";

        std::fs::write(root_path.join("package.json"), root_package_json)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        // Create package-lock.json (required for MonorepoDetector)
        let package_lock = "{
  \"name\": \"acme-monorepo\",
  \"version\": \"1.0.0\",
  \"lockfileVersion\": 3,
  \"requires\": true,
  \"packages\": {
    \"\": {
      \"name\": \"acme-monorepo\",
      \"version\": \"1.0.0\",
      \"workspaces\": [\"packages/*\"]
    }
  }
}";

        std::fs::write(root_path.join("package-lock.json"), package_lock)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        Ok(())
    }

    /// Setup individual packages
    fn setup_packages(root_path: &Path) -> Result<()> {
        // Shared package
        Self::setup_shared_package(root_path)?;

        // UI lib package
        Self::setup_ui_lib_package(root_path)?;

        // Core lib package
        Self::setup_core_lib_package(root_path)?;

        // Web app package
        Self::setup_web_app_package(root_path)?;

        Ok(())
    }

    fn setup_shared_package(root_path: &Path) -> Result<()> {
        let package_json = "{
  \"name\": \"@acme/shared\",
  \"version\": \"1.0.0\",
  \"description\": \"Shared utilities and types\",
  \"main\": \"dist/index.js\",
  \"types\": \"dist/index.d.ts\",
  \"scripts\": {
    \"build\": \"tsc\",
    \"test\": \"jest\",
    \"lint\": \"eslint src/**/*.ts\"
  },
  \"dependencies\": {},
  \"devDependencies\": {
    \"typescript\": \"^5.1.6\",
    \"jest\": \"^29.6.1\",
    \"@types/jest\": \"^29.5.3\"
  }
}";

        std::fs::write(root_path.join("packages/shared/package.json"), package_json)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        let types_file = "
export interface User {
  id: string;
  name: string;
  email: string;
}

export interface AppConfig {
  apiUrl: string;
  environment: 'development' | 'staging' | 'production';
  features: Record<string, boolean>;
}
";

        std::fs::write(root_path.join("packages/shared/src/types.ts"), types_file)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        let utils_file = "
export const formatDate = (date: Date): string => {
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  });
};

export const debounce = <T extends (...args: any[]) => void>(
  func: T,
  delay: number
): ((...args: Parameters<T>) => void) => {
  let timeoutId: NodeJS.Timeout;

  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => func(...args), delay);
  };
};
";

        std::fs::write(root_path.join("packages/shared/src/utils.ts"), utils_file)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        let index_file = "
export * from './types';
export * from './utils';
";

        std::fs::write(root_path.join("packages/shared/src/index.ts"), index_file)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        Ok(())
    }

    fn setup_ui_lib_package(root_path: &Path) -> Result<()> {
        let package_json = "{
  \"name\": \"@acme/ui-lib\",
  \"version\": \"1.0.0\",
  \"description\": \"React UI component library\",
  \"main\": \"dist/index.js\",
  \"types\": \"dist/index.d.ts\",
  \"scripts\": {
    \"build\": \"tsc && rollup -c\",
    \"test\": \"jest\",
    \"lint\": \"eslint src/**/*.{ts,tsx}\"
  },
  \"dependencies\": {
    \"react\": \"^18.2.0\",
    \"@acme/shared\": \"^1.0.0\"
  },
  \"devDependencies\": {
    \"typescript\": \"^5.1.6\",
    \"jest\": \"^29.6.1\",
    \"@testing-library/react\": \"^13.4.0\",
    \"@types/react\": \"^18.2.15\"
  },
  \"peerDependencies\": {
    \"react\": \">=17.0.0\"
  }
}";

        std::fs::write(root_path.join("packages/ui-lib/package.json"), package_json)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        let index_file = "
// UI Library entry point
// Components will be added via feature development
";

        std::fs::write(root_path.join("packages/ui-lib/src/index.ts"), index_file)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        Ok(())
    }

    fn setup_core_lib_package(root_path: &Path) -> Result<()> {
        let package_json = "{
  \"name\": \"@acme/core-lib\",
  \"version\": \"1.0.0\",
  \"description\": \"Core business logic library\",
  \"main\": \"dist/index.js\",
  \"types\": \"dist/index.d.ts\",
  \"scripts\": {
    \"build\": \"tsc\",
    \"test\": \"jest\",
    \"lint\": \"eslint src/**/*.ts\"
  },
  \"dependencies\": {
    \"@acme/shared\": \"^1.0.0\"
  },
  \"devDependencies\": {
    \"typescript\": \"^5.1.6\",
    \"jest\": \"^29.6.1\",
    \"@types/jest\": \"^29.5.3\"
  }
}";

        std::fs::write(root_path.join("packages/core-lib/package.json"), package_json)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        let hooks_file = "
import { User } from '@acme/shared';
import { useState, useCallback } from 'react';

export const useAppLogic = () => {
  const [user, setUser] = useState<User | null>(null);

  const handleLogin = useCallback(() => {
    // Simulate login logic
    const mockUser: User = {
      id: 'user_123',
      name: 'John Doe',
      email: 'john@example.com'
    };
    setUser(mockUser);
  }, []);

  const handleLogout = useCallback(() => {
    setUser(null);
  }, []);

  return {
    user,
    handleLogin,
    handleLogout
  };
};
";

        std::fs::write(root_path.join("packages/core-lib/src/hooks.ts"), hooks_file)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        let index_file = "
export * from './types';
export * from './hooks';
";

        std::fs::write(root_path.join("packages/core-lib/src/index.ts"), index_file)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        Ok(())
    }

    fn setup_web_app_package(root_path: &Path) -> Result<()> {
        let package_json = "{
  \"name\": \"@acme/web-app\",
  \"version\": \"1.0.0\",
  \"description\": \"Main web application\",
  \"private\": true,
  \"scripts\": {
    \"dev\": \"vite\",
    \"build\": \"tsc && vite build\",
    \"test\": \"jest\",
    \"test:e2e\": \"playwright test\",
    \"lint\": \"eslint src/**/*.{ts,tsx}\"
  },
  \"dependencies\": {
    \"react\": \"^18.2.0\",
    \"react-dom\": \"^18.2.0\",
    \"@acme/shared\": \"^1.0.0\",
    \"@acme/core-lib\": \"^1.0.0\"
  },
  \"devDependencies\": {
    \"typescript\": \"^5.1.6\",
    \"vite\": \"^4.4.0\",
    \"@vitejs/plugin-react\": \"^4.0.0\",
    \"jest\": \"^29.6.1\",
    \"@playwright/test\": \"^1.36.0\"
  }
}";

        std::fs::write(root_path.join("packages/web-app/package.json"), package_json)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        let app_file = "
import React from 'react';
import { useAppLogic } from '@acme/core-lib';

export const App: React.FC = () => {
  const { user, handleLogin, handleLogout } = useAppLogic();

  return (
    <div className=\"app\">
      <header>
        <h1>Acme Web Application</h1>
      </header>

      <main>
        {user ? (
          <div>
            <p>Welcome back, {user.name}!</p>
            <button onClick={handleLogout}>Logout</button>
          </div>
        ) : (
          <div>
            <p>Please log in to continue</p>
            <button onClick={handleLogin}>Login</button>
          </div>
        )}
      </main>
    </div>
  );
};
";

        std::fs::write(root_path.join("packages/web-app/src/App.tsx"), app_file)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        Ok(())
    }

    /// Setup git repository with initial commit
    fn setup_git_repository(root_path: &Path) -> Result<Repo> {
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

        // Initial commit
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(root_path)
            .output()
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        std::process::Command::new("git")
            .args(["commit", "-m", "feat: initial monorepo setup with 4 packages"])
            .current_dir(root_path)
            .output()
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        // Open repo
        let repo_path = root_path
            .to_str()
            .ok_or_else(|| sublime_monorepo_tools::Error::generic("Invalid UTF-8 in path"))?;

        Repo::open(repo_path)
            .map_err(|e| sublime_monorepo_tools::Error::git(format!("Failed to open repo: {}", e)))
    }

    // === Git Helper Methods ===

    fn create_git_branch(&self, branch_name: &str) -> Result<()> {
        std::process::Command::new("git")
            .args(["checkout", "-b", branch_name])
            .current_dir(&self.root_path)
            .output()
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;
        Ok(())
    }

    fn checkout_branch(&self, branch_name: &str) -> Result<()> {
        std::process::Command::new("git")
            .args(["checkout", branch_name])
            .current_dir(&self.root_path)
            .output()
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;
        Ok(())
    }

    fn merge_branch(&self, branch_name: &str) -> Result<()> {
        std::process::Command::new("git")
            .args(["merge", branch_name, "--no-ff"])
            .current_dir(&self.root_path)
            .output()
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;
        Ok(())
    }

    fn git_add_and_commit(&self, message: &str) -> Result<()> {
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&self.root_path)
            .output()
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        std::process::Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&self.root_path)
            .output()
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        Ok(())
    }

    /// Get short git hash for snapshot versions
    fn get_short_git_hash(&self) -> Result<String> {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--short=7", "HEAD"])
            .current_dir(&self.root_path)
            .output()
            .map_err(|e| {
                sublime_monorepo_tools::Error::generic(format!("Git command failed: {}", e))
            })?;

        if output.status.success() {
            let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(hash)
        } else {
            Err(sublime_monorepo_tools::Error::generic("Failed to get git hash".to_string()))
        }
    }

    // === File Helper Methods ===

    fn create_file(&self, relative_path: &str, content: &str) -> Result<()> {
        let file_path = self.root_path.join(relative_path);

        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;
        }

        std::fs::write(file_path, content)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;
        Ok(())
    }

    fn update_file(&self, relative_path: &str, content: &str) -> Result<()> {
        self.create_file(relative_path, content)
    }

    // === Analysis Helper Methods ===

    fn analyze_changes(&self, context: &str) -> Result<()> {
        println!("🔍 Analyzing changes: {}", context);

        let analyzer = MonorepoAnalyzer::new(&self.project);

        // Detect changes since last commit
        let changes = analyzer.detect_changes_since("HEAD~1", None)?;

        println!("📝 Changes detected:");
        println!("  - Files changed: {}", changes.changed_files.len());
        println!("  - Packages affected: {}", changes.directly_affected.len());

        for package in &changes.directly_affected {
            println!("    + {}", package);
        }

        if !changes.dependents_affected.is_empty() {
            println!("  - Dependent packages: {}", changes.dependents_affected.len());
            for dependent in &changes.dependents_affected {
                println!("    ~ {}", dependent);
            }
        }

        Ok(())
    }

    fn analyze_dependency_impact(&self) -> Result<()> {
        println!("🕸️  Analyzing dependency impact...");

        let analyzer = MonorepoAnalyzer::new(&self.project);
        let dep_graph = analyzer.build_dependency_graph()?;

        println!("📊 Dependency graph stats:");
        println!("  - Total nodes: {}", dep_graph.node_count);
        println!("  - Total edges: {}", dep_graph.edge_count);
        println!("  - Has cycles: {}", dep_graph.has_cycles);

        if dep_graph.has_cycles {
            println!("  - Cycles found: {}", dep_graph.cycles.len());
        }

        Ok(())
    }

    fn analyze_registry_status(&self) -> Result<()> {
        println!("🏛️  Analyzing registry status...");

        let analyzer = MonorepoAnalyzer::new(&self.project);
        let registry_analysis = analyzer.analyze_registries()?;

        println!("📋 Registry analysis:");
        println!("  - Default registry: {}", registry_analysis.default_registry);
        println!("  - Configured registries: {}", registry_analysis.registries.len());

        for registry in &registry_analysis.registries {
            println!("    + {} ({})", registry.url, registry.registry_type);
        }

        // Check auth status
        let has_auth = analyzer.check_registry_auth(&registry_analysis.default_registry);
        println!(
            "  - Registry auth: {}",
            if has_auth { "✅ Authenticated" } else { "❌ Not authenticated" }
        );

        Ok(())
    }

    // === Build & Test Helper Methods ===

    fn run_build_and_test(&self, package_path: &str) -> Result<()> {
        println!("🏗️  Building and testing {}...", package_path);

        // Simulate build process
        println!("  ✅ TypeScript compilation successful");
        println!("  ✅ Bundle generation complete");
        println!("  ✅ All tests passed (12/12)");
        println!("  ✅ Linting passed with no issues");

        Ok(())
    }

    fn run_integration_tests(&self) -> Result<()> {
        println!("🧪 Running integration tests...");

        // Simulate integration testing
        println!("  ✅ Component integration tests passed");
        println!("  ✅ API integration tests passed");
        println!("  ✅ E2E tests passed");

        Ok(())
    }

    fn test_dependency_upgrade(&self, dependency: &str) -> Result<()> {
        println!("🔧 Testing {} upgrade...", dependency);

        // Simulate testing the upgrade
        println!("  ✅ Compatibility check passed");
        println!("  ✅ Breaking changes analysis: none found");
        println!("  ✅ All tests continue to pass");

        Ok(())
    }

    // === Package Management Helper Methods ===

    fn update_package_dependency(
        &self,
        package_path: &str,
        dep_name: &str,
        version: &str,
    ) -> Result<()> {
        println!("📦 Updating {} dependency in {}: {}", dep_name, package_path, version);

        let package_json_path = self.root_path.join(package_path).join("package.json");

        // Read existing package.json
        let content = std::fs::read_to_string(&package_json_path)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        let mut package_json: Value = serde_json::from_str(&content).map_err(|e| {
            sublime_monorepo_tools::Error::generic(format!("JSON parse error: {}", e))
        })?;

        // Update dependency version
        if let Some(dependencies) =
            package_json.get_mut("dependencies").and_then(|v| v.as_object_mut())
        {
            if dependencies.contains_key(dep_name) {
                dependencies.insert(dep_name.to_string(), Value::String(version.to_string()));
                println!("  ✅ Updated {} in dependencies", dep_name);
            }
        }

        if let Some(dev_dependencies) =
            package_json.get_mut("devDependencies").and_then(|v| v.as_object_mut())
        {
            if dev_dependencies.contains_key(dep_name) {
                dev_dependencies.insert(dep_name.to_string(), Value::String(version.to_string()));
                println!("  ✅ Updated {} in devDependencies", dep_name);
            }
        }

        // Write updated package.json
        let updated_content = serde_json::to_string_pretty(&package_json).map_err(|e| {
            sublime_monorepo_tools::Error::generic(format!("JSON serialize error: {}", e))
        })?;

        std::fs::write(&package_json_path, updated_content)
            .map_err(|e| sublime_monorepo_tools::Error::generic(format!("IO error: {}", e)))?;

        println!("  📝 Package.json updated successfully");
        Ok(())
    }

    fn upgrade_dependency_across_packages(&self, dep_name: &str, version: &str) -> Result<()> {
        println!("🔄 Upgrading {} to {} across all packages...", dep_name, version);

        let packages =
            ["packages/shared", "packages/ui-lib", "packages/core-lib", "packages/web-app"];

        for package in packages {
            println!("  📦 Updating {} in {}", dep_name, package);
        }

        Ok(())
    }

    // === Changelog & Version Helper Methods ===

    fn generate_changelogs(&self, version_bumps: &[(&str, VersionBumpType, &str)]) -> Result<()> {
        println!("📄 Generating changelogs...");

        for (package, bump_type, reason) in version_bumps {
            println!("📝 Changelog for {}:", package);

            let new_version = match bump_type {
                VersionBumpType::Major => "2.0.0".to_string(),
                VersionBumpType::Minor => "1.1.0".to_string(),
                VersionBumpType::Patch => "1.0.1".to_string(),
                VersionBumpType::Snapshot => {
                    // Get short git hash for snapshot version
                    let git_hash =
                        self.get_short_git_hash().unwrap_or_else(|_| "unknown".to_string());
                    format!("1.0.0-snapshot.{}", git_hash)
                }
            };

            println!("  ## v{} ({})", new_version, chrono::Utc::now().format("%Y-%m-%d"));
            println!(
                "  ### {} Changes",
                match bump_type {
                    VersionBumpType::Major => "Breaking",
                    VersionBumpType::Minor => "Features",
                    VersionBumpType::Patch => "Bug Fixes",
                    VersionBumpType::Snapshot => "Snapshot",
                }
            );
            println!("  - {}", reason);
            println!();
        }

        Ok(())
    }

    fn show_version_impact_analysis(&self) -> Result<()> {
        println!("📈 Version Impact Analysis:");
        println!("==========================");

        // Simulate impact analysis
        let impacts = [
            ("@acme/shared", vec!["@acme/core-lib", "@acme/ui-lib", "@acme/web-app"]),
            ("@acme/ui-lib", vec!["@acme/web-app"]),
            ("@acme/core-lib", vec!["@acme/web-app"]),
        ];

        for (package, dependents) in impacts {
            if !dependents.is_empty() {
                println!("📦 {} affects {} packages:", package, dependents.len());
                for dependent in dependents {
                    println!("  → {}", dependent);
                }
                println!();
            }
        }

        Ok(())
    }

    fn create_release_summary(&self) -> Result<()> {
        println!("📋 Release Summary");
        println!("==================");
        println!("🎯 Release: v1.1.0 (Feature Release)");
        println!("📅 Date: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M UTC"));
        println!();
        println!("📦 Packages Updated:");
        println!("  - @acme/shared: 1.0.0 → 1.1.0 (Minor)");
        println!("  - @acme/ui-lib: 1.0.0 → 1.1.0 (Minor)");
        println!("  - @acme/core-lib: 1.0.0 → 1.1.0 (Minor)");
        println!("  - @acme/web-app: 1.0.0 → 1.0.1 (Patch)");
        println!();
        println!("🎉 Key Features:");
        println!("  - New Button component with variants");
        println!("  - Comprehensive validation utilities");
        println!("  - Enhanced authentication service");
        println!("  - Updated external dependencies");
        println!();
        println!("📊 Stats:");
        println!("  - 4 packages updated");
        println!("  - 6 feature branches merged");
        println!("  - 3 external dependencies upgraded");
        println!("  - 100% test coverage maintained");
        println!();

        Ok(())
    }
}

// Additional traits and implementations would go here for more complex scenarios
// This example demonstrates the core workflow and API usage
