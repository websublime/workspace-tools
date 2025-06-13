//! Default rule implementations and factory functions

use std::collections::HashMap;
use crate::changes::types::{
    ChangeDetectionRules, ChangeTypeRule, SignificanceRule, VersionBumpRule,
    FilePattern, PatternType, PackageChangeType, ChangeSignificance, VersionBumpType,
};

impl Default for ChangeDetectionRules {
    fn default() -> Self {
        Self {
            change_type_rules: default_change_type_rules(),
            significance_rules: default_significance_rules(),
            version_bump_rules: default_version_bump_rules(),
            project_overrides: HashMap::new(),
        }
    }
}

/// Create default change type rules
#[allow(clippy::too_many_lines)]
pub fn default_change_type_rules() -> Vec<ChangeTypeRule> {
    vec![
        ChangeTypeRule {
            name: "dependency_changes".to_string(),
            priority: 100,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Exact,
                    pattern: "package.json".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Exact,
                    pattern: "package-lock.json".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Exact,
                    pattern: "yarn.lock".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Exact,
                    pattern: "pnpm-lock.yaml".to_string(),
                    exclude: false,
                },
            ],
            change_type: PackageChangeType::Dependencies,
            conditions: None,
        },
        ChangeTypeRule {
            name: "source_code_changes".to_string(),
            priority: 80,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "src/**/*.{ts,js,tsx,jsx}".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "lib/**/*.{ts,js}".to_string(),
                    exclude: false,
                },
            ],
            change_type: PackageChangeType::SourceCode,
            conditions: None,
        },
        ChangeTypeRule {
            name: "configuration_changes".to_string(),
            priority: 70,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "*.config.{js,ts,json}".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: ".{eslintrc,prettierrc}*".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Exact,
                    pattern: "tsconfig.json".to_string(),
                    exclude: false,
                },
            ],
            change_type: PackageChangeType::Configuration,
            conditions: None,
        },
        ChangeTypeRule {
            name: "test_changes".to_string(),
            priority: 60,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "**/*.{test,spec}.{ts,js}".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "__tests__/**/*".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "tests/**/*".to_string(),
                    exclude: false,
                },
            ],
            change_type: PackageChangeType::Tests,
            conditions: None,
        },
        ChangeTypeRule {
            name: "documentation_changes".to_string(),
            priority: 50,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "**/*.md".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "docs/**/*".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Exact,
                    pattern: "README.md".to_string(),
                    exclude: false,
                },
            ],
            change_type: PackageChangeType::Documentation,
            conditions: None,
        },
    ]
}

/// Create change type rules using configurable priorities
#[allow(clippy::too_many_lines)]
pub fn create_change_type_rules_from_config(config: &crate::config::types::ValidationConfig) -> Vec<ChangeTypeRule> {
    vec![
        ChangeTypeRule {
            name: "dependency_changes".to_string(),
            priority: config.change_detection_rules.dependency_changes_priority,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Exact,
                    pattern: "package.json".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Exact,
                    pattern: "package-lock.json".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Exact,
                    pattern: "yarn.lock".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Exact,
                    pattern: "pnpm-lock.yaml".to_string(),
                    exclude: false,
                },
            ],
            change_type: PackageChangeType::Dependencies,
            conditions: None,
        },
        ChangeTypeRule {
            name: "source_code_changes".to_string(),
            priority: config.change_detection_rules.source_code_changes_priority,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "src/**/*.{ts,js,tsx,jsx}".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "lib/**/*.{ts,js}".to_string(),
                    exclude: false,
                },
            ],
            change_type: PackageChangeType::SourceCode,
            conditions: None,
        },
        ChangeTypeRule {
            name: "configuration_changes".to_string(),
            priority: config.change_detection_rules.configuration_changes_priority,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "*.config.{js,ts,json}".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "tsconfig*.json".to_string(),
                    exclude: false,
                },
            ],
            change_type: PackageChangeType::Configuration,
            conditions: None,
        },
        ChangeTypeRule {
            name: "test_changes".to_string(),
            priority: config.change_detection_rules.test_changes_priority,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "**/*.{test,spec}.{ts,js,tsx,jsx}".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "**/__tests__/**/*.{ts,js,tsx,jsx}".to_string(),
                    exclude: false,
                },
            ],
            change_type: PackageChangeType::Tests,
            conditions: None,
        },
        ChangeTypeRule {
            name: "documentation_changes".to_string(),
            priority: config.change_detection_rules.documentation_changes_priority,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "**/*.md".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "**/docs/**/*".to_string(),
                    exclude: false,
                },
            ],
            change_type: PackageChangeType::Documentation,
            conditions: None,
        },
    ]
}

/// Create default significance rules
pub fn default_significance_rules() -> Vec<SignificanceRule> {
    vec![
        SignificanceRule {
            name: "public_api_changes".to_string(),
            priority: 100,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "src/index.{ts,js}".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "src/api/**/*.{ts,js}".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "src/public/**/*.{ts,js}".to_string(),
                    exclude: false,
                },
            ],
            git_status: None,
            significance: ChangeSignificance::High,
            conditions: None,
        },
        SignificanceRule {
            name: "internal_changes".to_string(),
            priority: 80,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "src/internal/**/*".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "src/private/**/*".to_string(),
                    exclude: false,
                },
            ],
            git_status: None,
            significance: ChangeSignificance::Medium,
            conditions: None,
        },
        SignificanceRule {
            name: "test_changes".to_string(),
            priority: 60,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "**/*.{test,spec}.{ts,js}".to_string(),
                    exclude: false,
                },
            ],
            git_status: None,
            significance: ChangeSignificance::Low,
            conditions: None,
        },
        SignificanceRule {
            name: "documentation_changes".to_string(),
            priority: 40,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "**/*.md".to_string(),
                    exclude: false,
                },
            ],
            git_status: None,
            significance: ChangeSignificance::Low,
            conditions: None,
        },
    ]
}

/// Create significance rules using configurable priorities
pub fn create_significance_rules_from_config(config: &crate::config::types::ValidationConfig) -> Vec<SignificanceRule> {
    vec![
        SignificanceRule {
            name: "public_api_changes".to_string(),
            priority: config.change_detection_rules.significance_priorities.public_api_changes,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "src/**/*.{ts,js,tsx,jsx}".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "lib/**/*.{ts,js}".to_string(),
                    exclude: false,
                },
            ],
            significance: ChangeSignificance::High,
            git_status: None,
            conditions: None,
        },
        SignificanceRule {
            name: "internal_changes".to_string(),
            priority: config.change_detection_rules.significance_priorities.internal_changes,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "src/internal/**/*.{ts,js}".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "src/utils/**/*.{ts,js}".to_string(),
                    exclude: false,
                },
            ],
            significance: ChangeSignificance::Medium,
            git_status: None,
            conditions: None,
        },
        SignificanceRule {
            name: "test_changes".to_string(),
            priority: config.change_detection_rules.significance_priorities.test_changes,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "**/*.{test,spec}.{ts,js,tsx,jsx}".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "**/__tests__/**/*.{ts,js,tsx,jsx}".to_string(),
                    exclude: false,
                },
            ],
            significance: ChangeSignificance::Low,
            git_status: None,
            conditions: None,
        },
        SignificanceRule {
            name: "documentation_changes".to_string(),
            priority: config.change_detection_rules.significance_priorities.documentation_changes,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "**/*.md".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "**/docs/**/*".to_string(),
                    exclude: false,
                },
            ],
            significance: ChangeSignificance::Low,
            git_status: None,
            conditions: None,
        },
    ]
}

/// Create default version bump rules
pub fn default_version_bump_rules() -> Vec<VersionBumpRule> {
    vec![
        VersionBumpRule {
            name: "breaking_changes".to_string(),
            change_type: None,
            significance: Some(ChangeSignificance::High),
            version_bump: VersionBumpType::Major,
            priority: 100,
        },
        VersionBumpRule {
            name: "feature_changes".to_string(),
            change_type: Some(PackageChangeType::SourceCode),
            significance: Some(ChangeSignificance::Medium),
            version_bump: VersionBumpType::Minor,
            priority: 80,
        },
        VersionBumpRule {
            name: "dependency_changes".to_string(),
            change_type: Some(PackageChangeType::Dependencies),
            significance: None,
            version_bump: VersionBumpType::Minor,
            priority: 70,
        },
        VersionBumpRule {
            name: "patch_changes".to_string(),
            change_type: None,
            significance: Some(ChangeSignificance::Low),
            version_bump: VersionBumpType::Patch,
            priority: 60,
        },
        VersionBumpRule {
            name: "test_documentation_changes".to_string(),
            change_type: Some(PackageChangeType::Tests),
            significance: None,
            version_bump: VersionBumpType::Patch,
            priority: 50,
        },
        VersionBumpRule {
            name: "documentation_only_changes".to_string(),
            change_type: Some(PackageChangeType::Documentation),
            significance: None,
            version_bump: VersionBumpType::Patch,
            priority: 40,
        },
    ]
}

/// Create version bump rules using configurable priorities
pub fn create_version_bump_rules_from_config(config: &crate::config::types::ValidationConfig) -> Vec<VersionBumpRule> {
    vec![
        VersionBumpRule {
            name: "breaking_changes".to_string(),
            change_type: Some(PackageChangeType::SourceCode),
            significance: Some(ChangeSignificance::High),
            version_bump: VersionBumpType::Major,
            priority: config.version_bump_rules.breaking_changes_priority,
        },
        VersionBumpRule {
            name: "feature_changes".to_string(),
            change_type: Some(PackageChangeType::SourceCode),
            significance: Some(ChangeSignificance::Medium),
            version_bump: VersionBumpType::Minor,
            priority: config.version_bump_rules.feature_changes_priority,
        },
        VersionBumpRule {
            name: "dependency_changes".to_string(),
            change_type: Some(PackageChangeType::Dependencies),
            significance: Some(ChangeSignificance::Medium),
            version_bump: VersionBumpType::Patch,
            priority: config.version_bump_rules.dependency_changes_priority,
        },
        VersionBumpRule {
            name: "patch_changes".to_string(),
            change_type: Some(PackageChangeType::SourceCode),
            significance: Some(ChangeSignificance::Low),
            version_bump: VersionBumpType::Patch,
            priority: config.version_bump_rules.patch_changes_priority,
        },
        VersionBumpRule {
            name: "test_documentation_changes".to_string(),
            change_type: Some(PackageChangeType::Tests),
            significance: Some(ChangeSignificance::Low),
            version_bump: VersionBumpType::Patch,
            priority: config.version_bump_rules.test_documentation_priority,
        },
        VersionBumpRule {
            name: "documentation_only_changes".to_string(),
            change_type: Some(PackageChangeType::Documentation),
            significance: Some(ChangeSignificance::Low),
            version_bump: VersionBumpType::Patch,
            priority: config.version_bump_rules.documentation_only_priority,
        },
    ]
}