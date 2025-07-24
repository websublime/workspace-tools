//! Comprehensive tests for the context module
//!
//! This module contains all tests for context-aware functionality including
//! dependency parsing, project detection, and classification.

#[cfg(test)]
#[allow(clippy::panic)] // Tests may use panic for test failures per CLAUDE.md rules
#[allow(clippy::assertions_on_constants)] // Test assertions with false are acceptable in tests
mod tests {
    use crate::context::{
        DependencyParser, DependencySource, ProjectContext, SingleRepositoryContext,
        MonorepoContext, WorkspaceConstraint, GitReference,
    };
    use crate::context::project::InternalClassification;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_registry_dependency_parsing() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("react", "^18.0.0");
        match result {
            Ok(dep_source) => {
                if let DependencySource::Registry { name, version_req } = dep_source {
                    assert_eq!(name, "react");
                    assert_eq!(version_req.to_string(), "^18.0.0");
                } else {
                    panic!( "Expected Registry dependency, got: {:?}", dep_source);
                }
            }
            Err(e) => {
                panic!( "Failed to parse registry dependency: {:?}", e);
            }
        }
    }

    #[test]
    fn test_scoped_dependency_parsing() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("@types/node", "^20.0.0");
        match result {
            Ok(dep_source) => {
                if let DependencySource::Scoped { scope, name, version_req } = dep_source {
                    assert_eq!(scope, "types");
                    assert_eq!(name, "node");
                    assert_eq!(version_req.to_string(), "^20.0.0");
                } else {
                    panic!( "Expected Scoped dependency, got: {:?}", dep_source);
                }
            }
            Err(e) => {
                panic!( "Failed to parse scoped dependency: {:?}", e);
            }
        }
    }

    #[test]
    fn test_npm_dependency_parsing() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("react", "npm:react@^18.0.0");
        match result {
            Ok(dep_source) => {
                if let DependencySource::Npm { name, version_req } = dep_source {
                    assert_eq!(name, "react");
                    assert_eq!(version_req.to_string(), "^18.0.0");
                } else {
                    panic!( "Expected NPM dependency, got: {:?}", dep_source);
                }
            }
            Err(e) => {
                panic!( "Failed to parse npm dependency: {:?}", e);
            }
        }
    }

    #[test]
    fn test_jsr_dependency_parsing() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("cases", "jsr:@luca/cases@^1.0.1");
        match result {
            Ok(dep_source) => {
                if let DependencySource::Jsr { scope, name, version_req } = dep_source {
                    assert_eq!(scope, "luca");
                    assert_eq!(name, "cases");
                    assert_eq!(version_req.to_string(), "^1.0.1");
                } else {
                    panic!( "Expected JSR dependency, got: {:?}", dep_source);
                }
            }
            Err(e) => {
                panic!( "Failed to parse jsr dependency: {:?}", e);
            }
        }
    }

    #[test]
    fn test_file_dependency_parsing() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("local-package", "file:../local-package");
        match result {
            Ok(dep_source) => {
                if let DependencySource::File { name, path } = dep_source {
                    assert_eq!(name, "local-package");
                    assert_eq!(path, PathBuf::from("../local-package"));
                } else {
                    panic!( "Expected File dependency, got: {:?}", dep_source);
                }
            }
            Err(e) => {
                panic!( "Failed to parse file dependency: {:?}", e);
            }
        }
    }

    #[test]
    fn test_git_dependency_parsing() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("repo", "git+https://github.com/user/repo.git#main");
        match result {
            Ok(dep_source) => {
                if let DependencySource::Git { name, repo, reference } = dep_source {
                    assert_eq!(name, "repo");
                    assert_eq!(repo, "https://github.com/user/repo.git");
                    assert_eq!(reference, GitReference::Branch("main".to_string()));
                } else {
                    panic!( "Expected Git dependency, got: {:?}", dep_source);
                }
            }
            Err(e) => {
                panic!( "Failed to parse git dependency: {:?}", e);
            }
        }
    }

    #[test]
    fn test_github_dependency_parsing() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("repo", "user/repo");
        match result {
            Ok(dep_source) => {
                if let DependencySource::GitHub { name, user, repo, reference } = dep_source {
                    assert_eq!(name, "repo");
                    assert_eq!(user, "user");
                    assert_eq!(repo, "repo");
                    assert_eq!(reference, None);
                } else {
                    panic!( "Expected GitHub dependency, got: {:?}", dep_source);
                }
            }
            Err(e) => {
                panic!( "Failed to parse github dependency: {:?}", e);
            }
        }
    }

    #[test]
    fn test_url_dependency_parsing() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("package", "https://example.com/package.tgz");
        match result {
            Ok(dep_source) => {
                if let DependencySource::Url { name, url } = dep_source {
                    assert_eq!(name, "package");
                    assert_eq!(url, "https://example.com/package.tgz");
                } else {
                    panic!( "Expected URL dependency, got: {:?}", dep_source);
                }
            }
            Err(e) => {
                panic!( "Failed to parse url dependency: {:?}", e);
            }
        }
    }

    #[test]
    fn test_workspace_dependency_rejected_in_single_repo() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("internal", "workspace:*");
        assert!(result.is_err(), "Workspace dependency should be rejected in single repository context");
    }

    #[test] 
    fn test_workspace_dependency_accepted_in_monorepo() {
        let mut workspace_packages = HashMap::new();
        workspace_packages.insert("internal".to_string(), "packages/internal".to_string());
        
        let context = ProjectContext::Monorepo(MonorepoContext {
            workspace_packages,
            supported_protocols: crate::context::DependencyProtocol::all(),
            features_enabled: crate::context::MonorepoFeatures::all(),
            internal_classification: InternalClassification::NameBased,
        });
        let parser = DependencyParser::new(context);

        let result = parser.parse("internal", "workspace:*");
        match result {
            Ok(dep_source) => {
                if let DependencySource::Workspace { name, constraint } = dep_source {
                    assert_eq!(name, "internal");
                    assert_eq!(constraint, WorkspaceConstraint::Any);
                } else {
                    panic!( "Expected Workspace dependency, got: {:?}", dep_source);
                }
            }
            Err(e) => {
                panic!( "Failed to parse workspace dependency in monorepo: {:?}", e);
            }
        }
    }

    #[test]
    fn test_workspace_path_dependency_parsing() {
        let mut workspace_packages = HashMap::new();
        workspace_packages.insert("local-pkg".to_string(), "packages/local-pkg".to_string());
        
        let context = ProjectContext::Monorepo(MonorepoContext {
            workspace_packages,
            supported_protocols: crate::context::DependencyProtocol::all(),
            features_enabled: crate::context::MonorepoFeatures::all(),
            internal_classification: InternalClassification::NameBased,
        });
        let parser = DependencyParser::new(context);

        let result = parser.parse("local-pkg", "workspace:../packages/local-pkg");
        match result {
            Ok(dep_source) => {
                if let DependencySource::WorkspacePath { name, path } = dep_source {
                    assert_eq!(name, "local-pkg");
                    assert_eq!(path, PathBuf::from("../packages/local-pkg"));
                } else {
                    panic!( "Expected WorkspacePath dependency, got: {:?}", dep_source);
                }
            }
            Err(e) => {
                panic!( "Failed to parse workspace path dependency: {:?}", e);
            }
        }
    }

    #[test]
    fn test_parser_validation() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        // Valid specs should return true
        assert!(parser.validate("react", "^18.0.0"));
        assert!(parser.validate("lodash", "~4.17.21"));
        assert!(parser.validate("local", "file:../local"));
        
        // Workspace specs should return false in single repo
        assert!(!parser.validate("internal", "workspace:*"));
    }

    // =============================================================================
    // REAL-WORLD INTEGRATION TESTS
    // =============================================================================

    /// Test parsing dependencies that are commonly found in single repository projects
    #[test]
    fn test_single_repository_real_dependencies() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        // Common frontend dependencies
        let react_result = parser.parse("react", "^18.2.0");
        assert!(react_result.is_ok());

        let typescript_result = parser.parse("typescript", "~5.1.0");
        assert!(typescript_result.is_ok());

        // Scoped packages 
        let types_node = parser.parse("@types/node", "^20.0.0");
        assert!(types_node.is_ok());

        // Local development dependencies
        let local_tools = parser.parse("build-tools", "file:./tools/build-tools");
        assert!(local_tools.is_ok());

        // Git dependencies for development
        let git_dep = parser.parse("custom-lib", "git+https://github.com/company/custom-lib.git#v2.1.0");
        assert!(git_dep.is_ok());

        // NPM protocol explicit usage
        let npm_dep = parser.parse("lodash", "npm:lodash@^4.17.21");
        assert!(npm_dep.is_ok());

        // JSR dependencies for modern JS
        let jsr_dep = parser.parse("std", "jsr:@std/assert@^0.220.0");
        assert!(jsr_dep.is_ok());

        // Workspace dependencies should be rejected
        let workspace_dep = parser.parse("internal-lib", "workspace:*");
        assert!(workspace_dep.is_err(), "Single repo should reject workspace dependencies");
    }

    /// Test parsing dependencies that are commonly found in monorepo projects
    #[test]
    fn test_monorepo_real_dependencies() {
        // Simulate a real monorepo with multiple packages
        let mut workspace_packages = HashMap::new();
        workspace_packages.insert("ui-components".to_string(), "packages/ui-components".to_string());
        workspace_packages.insert("shared-utils".to_string(), "packages/shared-utils".to_string());
        workspace_packages.insert("api-client".to_string(), "packages/api-client".to_string());
        workspace_packages.insert("mobile-app".to_string(), "apps/mobile".to_string());
        workspace_packages.insert("web-app".to_string(), "apps/web".to_string());

        let context = ProjectContext::Monorepo(MonorepoContext {
            workspace_packages,
            supported_protocols: crate::context::DependencyProtocol::all(),
            features_enabled: crate::context::MonorepoFeatures::all(),
            internal_classification: InternalClassification::NameBased,
        });
        let parser = DependencyParser::new(context);

        // External dependencies (same as single repo)
        let react_result = parser.parse("react", "^18.2.0");
        assert!(react_result.is_ok());

        let lodash_result = parser.parse("lodash", "^4.17.21");
        assert!(lodash_result.is_ok());

        // Internal workspace dependencies with different patterns
        let ui_components = parser.parse("ui-components", "workspace:*");
        match ui_components {
            Ok(DependencySource::Workspace { name, constraint }) => {
                assert_eq!(name, "ui-components");
                assert_eq!(constraint, WorkspaceConstraint::Any);
            }
            other => panic!( "Expected workspace dependency, got: {:?}", other),
        }

        let shared_utils = parser.parse("shared-utils", "workspace:^");
        match shared_utils {
            Ok(DependencySource::Workspace { name, constraint }) => {
                assert_eq!(name, "shared-utils");
                assert_eq!(constraint, WorkspaceConstraint::Compatible);
            }
            other => panic!( "Expected workspace dependency, got: {:?}", other),
        }

        // Workspace path references
        let api_client = parser.parse("api-client", "workspace:../packages/api-client");
        match api_client {
            Ok(DependencySource::WorkspacePath { name, path }) => {
                assert_eq!(name, "api-client");
                assert_eq!(path, PathBuf::from("../packages/api-client"));
            }
            other => panic!( "Expected workspace path dependency, got: {:?}", other),
        }

        // Mixed external and internal dependencies in the same project
        let external_git = parser.parse("design-tokens", "git+https://github.com/company/design-tokens.git#main");
        assert!(external_git.is_ok());

        let external_npm = parser.parse("react-router", "npm:react-router-dom@^6.8.0");
        assert!(external_npm.is_ok());
    }

    /// Test complex real-world dependency scenarios
    #[test]
    fn test_complex_dependency_scenarios() {
        let mut workspace_packages = HashMap::new();
        workspace_packages.insert("core".to_string(), "packages/core".to_string());
        workspace_packages.insert("plugins".to_string(), "packages/plugins".to_string());
        workspace_packages.insert("cli".to_string(), "packages/cli".to_string());

        let context = ProjectContext::Monorepo(MonorepoContext {
            workspace_packages,
            supported_protocols: crate::context::DependencyProtocol::all(),
            features_enabled: crate::context::MonorepoFeatures::all(),
            internal_classification: InternalClassification::NameBased,
        });
        let parser = DependencyParser::new(context);

        // Scenario 1: Different git reference types
        let git_branch = parser.parse("experimental", "git+https://github.com/org/experimental.git#feature/new-api");
        assert!(git_branch.is_ok());

        let git_tag = parser.parse("stable-lib", "git+https://github.com/org/stable-lib.git#v1.2.3");
        assert!(git_tag.is_ok());

        let git_commit = parser.parse("pinned-lib", "git+https://github.com/org/pinned-lib.git#abc123def456");
        assert!(git_commit.is_ok());

        // Scenario 2: GitHub shorthand variations
        let github_simple = parser.parse("utility", "company/utility");
        assert!(github_simple.is_ok());

        let github_explicit = parser.parse("helper", "github:company/helper#v2.0.0");
        assert!(github_explicit.is_ok());

        // Scenario 3: File dependencies with different path types
        let relative_file = parser.parse("local-config", "file:../config");
        assert!(relative_file.is_ok());

        let nested_file = parser.parse("deep-dependency", "file:../../shared/deep-dependency");
        assert!(nested_file.is_ok());

        // Scenario 4: URL-based dependencies
        let tarball_url = parser.parse("custom-package", "https://registry.company.com/packages/custom-package-1.0.0.tgz");
        assert!(tarball_url.is_ok());

        // Scenario 5: Cross-registry dependencies
        let npm_scoped = parser.parse("company-lib", "npm:@company/lib@^2.1.0");
        assert!(npm_scoped.is_ok());

        let jsr_scoped = parser.parse("deno-std", "jsr:@std/path@^0.220.0");
        assert!(jsr_scoped.is_ok());
    }

    /// Test validation scenarios for different contexts
    #[test]
    fn test_validation_scenarios() {
        // Single repository validation
        let single_context = ProjectContext::Single(SingleRepositoryContext::default());
        let single_parser = DependencyParser::new(single_context);

        // Valid single repo dependencies
        assert!(single_parser.validate("react", "^18.0.0"));
        assert!(single_parser.validate("@types/node", "^20.0.0"));
        assert!(single_parser.validate("tools", "file:../tools"));
        assert!(single_parser.validate("lib", "git+https://github.com/user/lib.git"));
        assert!(single_parser.validate("package", "npm:package@^1.0.0"));
        assert!(single_parser.validate("module", "jsr:@scope/module@^1.0.0"));
        assert!(single_parser.validate("archive", "https://example.com/package.tgz"));

        // Invalid single repo dependencies
        assert!(!single_parser.validate("internal", "workspace:*"));
        assert!(!single_parser.validate("local", "workspace:../packages/local"));

        // Monorepo validation
        let mut workspace_packages = HashMap::new();
        workspace_packages.insert("internal".to_string(), "packages/internal".to_string());

        let monorepo_context = ProjectContext::Monorepo(MonorepoContext {
            workspace_packages,
            supported_protocols: crate::context::DependencyProtocol::all(),
            features_enabled: crate::context::MonorepoFeatures::all(),
            internal_classification: InternalClassification::NameBased,
        });
        let monorepo_parser = DependencyParser::new(monorepo_context);

        // All single repo dependencies should be valid in monorepo
        assert!(monorepo_parser.validate("react", "^18.0.0"));
        assert!(monorepo_parser.validate("@types/node", "^20.0.0"));
        assert!(monorepo_parser.validate("tools", "file:../tools"));

        // Workspace dependencies should be valid in monorepo
        assert!(monorepo_parser.validate("internal", "workspace:*"));
        assert!(monorepo_parser.validate("pkg", "workspace:../packages/pkg"));
    }

    /// Test edge cases and error scenarios
    #[test]
    fn test_edge_cases() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        // Edge case: empty version string (should fail)
        let empty_version = parser.parse("package", "");
        assert!(empty_version.is_err());

        // Edge case: workspace dependency in single repo (should fail)
        let workspace_in_single = parser.parse("package", "workspace:*");
        assert!(workspace_in_single.is_err());

        // Edge case: normal dependency parsing (should succeed)
        let normal_dep = parser.parse("lodash", "^4.17.21");
        assert!(normal_dep.is_ok());

        // Edge case: scoped dependency (should succeed)
        let scoped_dep = parser.parse("@types/node", "^20.0.0");
        assert!(scoped_dep.is_ok());
    }
}