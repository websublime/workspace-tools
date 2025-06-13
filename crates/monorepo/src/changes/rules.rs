//! Default rule implementations and factory functions

use crate::changes::types::{
    ChangeDetectionRules, ChangeSignificance, ChangeTypeRule, FilePattern, PackageChangeType,
    PatternType, SignificanceRule, VersionBumpRule, VersionBumpType,
};
use std::collections::HashMap;

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

/// Create default significance rules
#[allow(clippy::too_many_lines)]
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
                    pattern: "**/*.d.ts".to_string(),
                    exclude: false,
                },
            ],
            git_status: Some(vec![
                sublime_git_tools::GitFileStatus::Modified,
                sublime_git_tools::GitFileStatus::Added,
            ]),
            significance: ChangeSignificance::High,
            conditions: None,
        },
        SignificanceRule {
            name: "internal_implementation_changes".to_string(),
            priority: 80,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "src/**/*.{ts,js}".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "lib/**/*.{ts,js}".to_string(),
                    exclude: false,
                },
            ],
            git_status: Some(vec![sublime_git_tools::GitFileStatus::Modified]),
            significance: ChangeSignificance::Medium,
            conditions: None,
        },
        SignificanceRule {
            name: "breaking_change_indicators".to_string(),
            priority: 120,
            patterns: vec![FilePattern {
                pattern_type: PatternType::Regex,
                pattern: r"BREAKING[:_\s]*CHANGE".to_string(),
                exclude: false,
            }],
            git_status: Some(vec![
                sublime_git_tools::GitFileStatus::Modified,
                sublime_git_tools::GitFileStatus::Added,
            ]),
            significance: ChangeSignificance::High,
            conditions: None,
        },
        SignificanceRule {
            name: "dependency_updates".to_string(),
            priority: 70,
            patterns: vec![FilePattern {
                pattern_type: PatternType::Exact,
                pattern: "package.json".to_string(),
                exclude: false,
            }],
            git_status: Some(vec![sublime_git_tools::GitFileStatus::Modified]),
            significance: ChangeSignificance::Medium,
            conditions: None,
        },
        SignificanceRule {
            name: "configuration_updates".to_string(),
            priority: 60,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "*.config.{js,ts,json}".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Exact,
                    pattern: "tsconfig.json".to_string(),
                    exclude: false,
                },
            ],
            git_status: Some(vec![
                sublime_git_tools::GitFileStatus::Modified,
                sublime_git_tools::GitFileStatus::Added,
            ]),
            significance: ChangeSignificance::Low,
            conditions: None,
        },
        SignificanceRule {
            name: "documentation_updates".to_string(),
            priority: 40,
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
            ],
            git_status: Some(vec![
                sublime_git_tools::GitFileStatus::Modified,
                sublime_git_tools::GitFileStatus::Added,
            ]),
            significance: ChangeSignificance::Low,
            conditions: None,
        },
        SignificanceRule {
            name: "test_updates".to_string(),
            priority: 30,
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
            ],
            git_status: Some(vec![
                sublime_git_tools::GitFileStatus::Modified,
                sublime_git_tools::GitFileStatus::Added,
            ]),
            significance: ChangeSignificance::Low,
            conditions: None,
        },
    ]
}

/// Create default version bump rules
pub fn default_version_bump_rules() -> Vec<VersionBumpRule> {
    vec![
        VersionBumpRule {
            name: "breaking_changes".to_string(),
            change_type: Some(PackageChangeType::SourceCode),
            significance: Some(ChangeSignificance::High),
            version_bump: VersionBumpType::Major,
            priority: 100,
        },
        VersionBumpRule {
            name: "new_features".to_string(),
            change_type: Some(PackageChangeType::SourceCode),
            significance: Some(ChangeSignificance::Medium),
            version_bump: VersionBumpType::Minor,
            priority: 80,
        },
        VersionBumpRule {
            name: "bug_fixes".to_string(),
            change_type: Some(PackageChangeType::SourceCode),
            significance: Some(ChangeSignificance::Low),
            version_bump: VersionBumpType::Patch,
            priority: 60,
        },
        VersionBumpRule {
            name: "dependency_updates".to_string(),
            change_type: Some(PackageChangeType::Dependencies),
            significance: Some(ChangeSignificance::Medium),
            version_bump: VersionBumpType::Minor,
            priority: 70,
        },
        VersionBumpRule {
            name: "documentation_only_changes".to_string(),
            change_type: Some(PackageChangeType::Documentation),
            significance: Some(ChangeSignificance::Low),
            version_bump: VersionBumpType::Patch,
            priority: 40,
        },
    ]
}
