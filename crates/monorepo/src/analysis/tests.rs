//! Unit tests for analysis module

#[cfg(test)]
mod tests {
    use crate::analysis::*;
    use std::path::PathBuf;
    use std::collections::HashMap;

    #[test]
    fn test_monorepo_analysis_result_creation() {
        use sublime_standard_tools::monorepo::{MonorepoKind, PackageManagerKind};
        
        let result = MonorepoAnalysisResult {
            kind: MonorepoKind::NpmWorkSpace,
            root_path: PathBuf::from("/test/monorepo"),
            package_manager: PackageManagerAnalysis {
                kind: PackageManagerKind::Npm,
                version: "8.0.0".to_string(),
                lock_file: PathBuf::from("package-lock.json"),
                config_files: vec![PathBuf::from("package.json")],
                workspaces_config: serde_json::json!({"workspaces": ["packages/*"]}),
            },
            packages: PackageClassificationResult {
                internal_packages: vec![],
                external_dependencies: vec!["lodash".to_string()],
                dev_dependencies: vec!["jest".to_string()],
                peer_dependencies: vec![],
            },
            dependency_graph: DependencyGraphAnalysis {
                node_count: 0,
                edge_count: 0,
                has_cycles: false,
                cycles: vec![],
                version_conflicts: HashMap::new(),
                upgradable: HashMap::new(),
                max_depth: 0,
                most_dependencies: vec![],
                most_dependents: vec![],
            },
            registries: RegistryAnalysisResult {
                default_registry: "https://registry.npmjs.org/".to_string(),
                registries: vec![],
                scoped_registries: HashMap::new(),
                auth_status: HashMap::new(),
            },
            workspace_config: WorkspaceConfigAnalysis {
                patterns: vec!["packages/*".to_string()],
                matched_packages: 0,
                orphaned_packages: vec![],
                has_nohoist: false,
                nohoist_patterns: vec![],
            },
        };
        
        assert_eq!(result.kind, MonorepoKind::NpmWorkSpace);
        assert_eq!(result.root_path, PathBuf::from("/test/monorepo"));
        assert!(!result.registries.default_registry.is_empty());
    }

    #[test]
    fn test_package_information_creation() {
        let pkg_info = PackageInformation {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            path: PathBuf::from("/path/to/package"),
            relative_path: PathBuf::from("packages/test-package"),
            package_json: serde_json::json!({"name": "test-package"}),
            is_internal: true,
            dependencies: vec!["lodash".to_string()],
            dev_dependencies: vec!["jest".to_string()],
            workspace_dependencies: vec!["@shared/utils".to_string()],
            dependents: vec!["@app/frontend".to_string()],
        };
        
        assert_eq!(pkg_info.name, "test-package");
        assert_eq!(pkg_info.version, "1.0.0");
        assert!(pkg_info.is_internal);
        assert_eq!(pkg_info.dependencies.len(), 1);
        assert_eq!(pkg_info.workspace_dependencies.len(), 1);
    }

    #[test]
    fn test_dependency_graph_analysis() {
        let analysis = DependencyGraphAnalysis {
            node_count: 10,
            edge_count: 15,
            has_cycles: false,
            cycles: vec![],
            version_conflicts: HashMap::new(),
            upgradable: HashMap::new(),
            max_depth: 3,
            most_dependencies: vec![("pkg-a".to_string(), 5)],
            most_dependents: vec![("shared-lib".to_string(), 8)],
        };
        
        assert_eq!(analysis.node_count, 10);
        assert_eq!(analysis.edge_count, 15);
        assert!(!analysis.has_cycles);
        assert_eq!(analysis.max_depth, 3);
    }

    #[test]
    fn test_workspace_config_analysis() {
        let analysis = WorkspaceConfigAnalysis {
            patterns: vec!["packages/*".to_string(), "apps/*".to_string()],
            matched_packages: 5,
            orphaned_packages: vec![],
            has_nohoist: false,
            nohoist_patterns: vec![],
        };
        
        assert_eq!(analysis.patterns.len(), 2);
        assert_eq!(analysis.matched_packages, 5);
        assert!(analysis.orphaned_packages.is_empty());
        assert!(!analysis.has_nohoist);
    }

    #[test]
    fn test_registry_analysis_result() {
        let mut auth_status = HashMap::new();
        auth_status.insert("https://registry.npmjs.org/".to_string(), true);
        
        let result = RegistryAnalysisResult {
            default_registry: "https://registry.npmjs.org/".to_string(),
            registries: vec![RegistryInfo {
                url: "https://registry.npmjs.org/".to_string(),
                registry_type: "npm".to_string(),
                has_auth: true,
                scopes: vec!["@myorg".to_string()],
            }],
            scoped_registries: HashMap::new(),
            auth_status,
        };
        
        assert_eq!(result.default_registry, "https://registry.npmjs.org/");
        assert_eq!(result.registries.len(), 1);
        assert!(result.auth_status["https://registry.npmjs.org/"]);
    }

    #[test]
    fn test_upgrade_analysis_result() {
        let result = UpgradeAnalysisResult {
            total_packages: 10,
            upgradable_count: 3,
            major_upgrades: vec![UpgradeInfo {
                package_name: "app".to_string(),
                dependency_name: "react".to_string(),
                current_version: "17.0.0".to_string(),
                available_version: "18.0.0".to_string(),
                upgrade_type: "major".to_string(),
            }],
            minor_upgrades: vec![],
            patch_upgrades: vec![],
            up_to_date: vec!["utils".to_string(), "shared".to_string()],
        };
        
        assert_eq!(result.total_packages, 10);
        assert_eq!(result.upgradable_count, 3);
        assert_eq!(result.major_upgrades.len(), 1);
        assert_eq!(result.up_to_date.len(), 2);
    }

    #[test]
    fn test_workspace_pattern_analysis() {
        let analysis = WorkspacePatternAnalysis {
            config_patterns: vec!["packages/*".to_string()],
            auto_detected_patterns: vec!["apps/*".to_string()],
            effective_patterns: vec!["packages/*".to_string()],
            validation_errors: vec![],
            pattern_statistics: vec![PatternStatistics {
                pattern: "packages/*".to_string(),
                matches: 5,
                is_effective: true,
                specificity: 10,
            }],
            orphaned_packages: vec![],
        };
        
        assert_eq!(analysis.config_patterns.len(), 1);
        assert_eq!(analysis.auto_detected_patterns.len(), 1);
        assert!(analysis.validation_errors.is_empty());
        assert_eq!(analysis.pattern_statistics.len(), 1);
        assert!(analysis.pattern_statistics[0].is_effective);
    }

    #[test]
    fn test_monorepo_analyzer_type_exists() {
        // Test that the MonorepoAnalyzer type can be referenced
        // This validates the type exists and is accessible
        use crate::analysis::MonorepoAnalyzer;
        
        // Check that we can get the type name
        let type_name = std::any::type_name::<MonorepoAnalyzer>();
        assert!(type_name.contains("MonorepoAnalyzer"));
    }

    #[test]
    fn test_diff_analyzer_type_exists() {
        // Test that the DiffAnalyzer type can be referenced
        // This validates the type exists and is accessible
        use crate::analysis::DiffAnalyzer;
        
        // Check that we can get the type name
        let type_name = std::any::type_name::<DiffAnalyzer>();
        assert!(type_name.contains("DiffAnalyzer"));
    }
}