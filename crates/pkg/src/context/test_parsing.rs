//! # Test file for dependency parsing
//!
//! This is a temporary test file to verify that the dependency parsing
//! logic works correctly with different protocols and contexts.

#[cfg(test)]
mod tests {
    use crate::context::{
        DependencyParser, DependencySource, ProjectContext, SingleRepositoryContext,
        MonorepoContext, WorkspaceConstraint, GitReference,
    };
    use semver::VersionReq;
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
                    panic!("Expected Registry dependency, got: {:?}", dep_source);
                }
            }
            Err(e) => {
                panic!("Parsing failed with error: {}", e);
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
                    panic!("Expected Scoped dependency, got: {:?}", dep_source);
                }
            }
            Err(e) => {
                panic!("Parsing failed with error: {}", e);
            }
        }
    }

    #[test]
    fn test_workspace_dependency_rejected_in_single_repo() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("internal-package", "workspace:*");
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("workspace"));
        assert!(error.to_string().contains("single repository"));
    }

    #[test]
    fn test_workspace_dependency_accepted_in_monorepo() {
        let context = ProjectContext::Monorepo(MonorepoContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("internal-package", "workspace:*");
        assert!(result.is_ok());

        if let Ok(DependencySource::Workspace { name, constraint }) = result {
            assert_eq!(name, "internal-package");
            assert_eq!(constraint, WorkspaceConstraint::Any);
        } else {
            panic!("Expected Workspace dependency");
        }
    }

    #[test]
    fn test_workspace_path_dependency() {
        let context = ProjectContext::Monorepo(MonorepoContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("local-package", "workspace:../packages/core");
        assert!(result.is_ok());

        if let Ok(DependencySource::WorkspacePath { name, path }) = result {
            assert_eq!(name, "local-package");
            assert_eq!(path, PathBuf::from("../packages/core"));
        } else {
            panic!("Expected WorkspacePath dependency");
        }
    }

    #[test]
    fn test_npm_protocol_parsing() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("package", "npm:@mui/styled-engine-sc@^5.3.0");
        match result {
            Ok(dep_source) => {
                if let DependencySource::Npm { name, version_req } = dep_source {
                    assert_eq!(name, "@mui/styled-engine-sc");
                    // The version requirement should be parsed correctly
                    assert_eq!(version_req.to_string(), "^5.3.0");
                } else {
                    panic!("Expected Npm dependency, got: {:?}", dep_source);
                }
            }
            Err(e) => {
                panic!("Parsing failed with error: {}", e);
            }
        }
    }

    #[test]
    fn test_jsr_protocol_parsing() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("package", "jsr:@luca/cases@^1.0.1");
        assert!(result.is_ok());

        if let Ok(DependencySource::Jsr { scope, name, version_req }) = result {
            assert_eq!(scope, "luca");
            assert_eq!(name, "cases");
            assert_eq!(version_req.to_string(), "^1.0.1");
        } else {
            panic!("Expected Jsr dependency");
        }
    }

    #[test]
    fn test_git_dependency_parsing() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("my-lib", "git+https://github.com/user/my-lib.git#main");
        assert!(result.is_ok());

        if let Ok(DependencySource::Git { name, repo, reference }) = result {
            assert_eq!(name, "my-lib");
            assert_eq!(repo, "https://github.com/user/my-lib.git");
            assert_eq!(reference, GitReference::Branch("main".to_string()));
        } else {
            panic!("Expected Git dependency");
        }
    }

    #[test]
    fn test_github_shorthand_parsing() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("repo", "user/repo");
        assert!(result.is_ok());

        if let Ok(DependencySource::GitHub { name, user, repo, reference }) = result {
            assert_eq!(name, "repo");
            assert_eq!(user, "user");
            assert_eq!(repo, "repo");
            assert_eq!(reference, None);
        } else {
            panic!("Expected GitHub dependency");
        }
    }

    #[test]
    fn test_file_dependency_parsing() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("local-package", "file:../local-package");
        assert!(result.is_ok());

        if let Ok(DependencySource::File { name, path }) = result {
            assert_eq!(name, "local-package");
            assert_eq!(path, PathBuf::from("../local-package"));
        } else {
            panic!("Expected File dependency");
        }
    }

    #[test]
    fn test_url_dependency_parsing() {
        let context = ProjectContext::Single(SingleRepositoryContext::default());
        let parser = DependencyParser::new(context);

        let result = parser.parse("package", "https://example.com/package.tgz");
        assert!(result.is_ok());

        if let Ok(DependencySource::Url { name, url }) = result {
            assert_eq!(name, "package");
            assert_eq!(url, "https://example.com/package.tgz");
        } else {
            panic!("Expected Url dependency");
        }
    }

    #[test]
    fn test_context_awareness() {
        let single_context = ProjectContext::Single(SingleRepositoryContext::default());
        let monorepo_context = ProjectContext::Monorepo(MonorepoContext::default());

        let single_parser = DependencyParser::new(single_context);
        let monorepo_parser = DependencyParser::new(monorepo_context);

        // Test that workspace protocols are supported differently
        assert!(!single_parser.supports_workspace_protocols());
        assert!(monorepo_parser.supports_workspace_protocols());

        // Test validation
        assert!(single_parser.validate("react", "^18.0.0"));
        assert!(!single_parser.validate("internal", "workspace:*"));

        assert!(monorepo_parser.validate("react", "^18.0.0"));
        assert!(monorepo_parser.validate("internal", "workspace:*"));
    }
}