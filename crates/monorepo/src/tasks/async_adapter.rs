//! Async Boundary Adapter
//!
//! Provides a clean boundary between synchronous condition checking and
//! asynchronous operations like custom script execution. This adapter follows
//! Rust ownership principles and provides clear async/sync separation.

use crate::error::{Error, Result};
use crate::tasks::types::ExecutionContext;
use crate::tasks::ConditionChecker;
use std::future::Future;
use std::pin::Pin;

/// Async boundary adapter for condition checking
///
/// This adapter provides a clean interface that handles both synchronous
/// condition checking and asynchronous operations (like custom scripts)
/// while maintaining clear boundaries and proper Rust ownership.
///
/// # Usage
///
/// The adapter automatically detects whether conditions require async execution
/// and routes them appropriately:
///
/// ```rust
/// use sublime_monorepo_tools::tasks::{AsyncConditionAdapter, TaskCondition, ExecutionContext};
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create the adapter with required providers
/// let adapter = AsyncConditionAdapterBuilder::new()
///     .git_provider(git_provider)
///     .config_provider(config_provider)
///     .package_provider(package_provider)
///     .file_system_provider(file_system_provider)
///     .build()?;
///
/// // Mix of sync and async conditions
/// let conditions = vec![
///     TaskCondition::PackagesChanged { packages: vec!["my-package".to_string()] },
///     TaskCondition::CustomScript { 
///         script: "echo 'test'".to_string(), 
///         expected_output: Some("test".to_string()) 
///     },
/// ];
///
/// let context = ExecutionContext::default();
/// let result = adapter.evaluate_conditions_adaptive(&conditions, &context).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Async Boundary Design
///
/// The adapter follows these principles:
/// 
/// 1. **Automatic Detection**: Automatically detects async requirements
/// 2. **Sync Fast Path**: Pure sync conditions use fast synchronous execution
/// 3. **Mixed Handling**: Mixed conditions handle each appropriately
/// 4. **Clear Errors**: Provides clear error messages when sync methods encounter async conditions
///
/// # Performance
///
/// - Synchronous conditions are executed immediately without async overhead
/// - Only conditions requiring async execution (custom scripts, environment checkers) use async
/// - Mixed conditions are evaluated efficiently with minimal async overhead
pub struct AsyncConditionAdapter {
    /// The synchronous condition checker
    checker: ConditionChecker,
}

/// Represents the result of a condition evaluation
pub enum ConditionResult {
    /// Synchronous result (immediate)
    Sync(Result<bool>),
    /// Asynchronous operation required
    Async(Pin<Box<dyn Future<Output = Result<bool>>>>),
}

impl AsyncConditionAdapter {
    /// Create a new async boundary adapter
    #[must_use]
    pub fn new(checker: ConditionChecker) -> Self {
        Self { checker }
    }

    /// Get a reference to the underlying synchronous checker
    #[must_use]
    pub fn sync_checker(&self) -> &ConditionChecker {
        &self.checker
    }

    /// Execute a custom script asynchronously (the only truly async operation)
    pub async fn execute_custom_script(
        &self,
        script: &str,
        expected_output: &Option<String>,
        context: &ExecutionContext,
    ) -> Result<bool> {
        // Use the genuine async function from the checker
        self.checker.execute_custom_script(script, expected_output, context).await
    }

    /// Execute a custom environment checker asynchronously
    pub async fn execute_custom_environment_checker(
        &self,
        checker_name: &str,
        context: &ExecutionContext,
    ) -> Result<bool> {
        // Use the genuine async function from the checker
        self.checker.execute_custom_environment_checker(checker_name, context).await
    }

    /// Check if conditions require async execution
    #[must_use]
    pub fn requires_async_execution(conditions: &[crate::tasks::TaskCondition]) -> bool {
        use crate::tasks::TaskCondition;
        
        conditions.iter().any(|condition| match condition {
            TaskCondition::CustomScript { .. } => true,
            TaskCondition::Environment { env } => {
                matches!(env, crate::tasks::EnvironmentCondition::Custom { .. })
            },
            TaskCondition::All { conditions } => Self::requires_async_execution(conditions),
            TaskCondition::Any { conditions } => Self::requires_async_execution(conditions),
            TaskCondition::Not { condition } => Self::requires_async_execution(&[*condition.clone()]),
            _ => false,
        })
    }

    /// Evaluate conditions with proper async/sync boundary
    pub async fn evaluate_conditions_adaptive(
        &self,
        conditions: &[crate::tasks::TaskCondition],
        context: &ExecutionContext,
    ) -> Result<bool> {
        // Quick check if any async operations are needed
        if !Self::requires_async_execution(conditions) {
            // All conditions are synchronous - use sync path
            return self.checker.check_conditions_with_context(conditions, context);
        }

        // Mixed sync/async conditions - need to handle individually
        self.evaluate_mixed_conditions(conditions, context).await
    }

    /// Handle mixed sync/async conditions
    fn evaluate_mixed_conditions<'a>(
        &'a self,
        conditions: &'a [crate::tasks::TaskCondition],
        context: &'a ExecutionContext,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + 'a>> {
        Box::pin(async move {
        use crate::tasks::TaskCondition;

        // All conditions must be met
        for condition in conditions {
            let result = match condition {
                TaskCondition::CustomScript { script, expected_output } => {
                    // Async operation
                    self.execute_custom_script(script, expected_output, context).await?
                }
                
                TaskCondition::Environment { env } => {
                    match env {
                        crate::tasks::EnvironmentCondition::Custom { checker } => {
                            // Async operation
                            self.execute_custom_environment_checker(checker, context).await?
                        }
                        _ => {
                            // Sync operation
                            self.checker.check_environment_condition(env, context)?
                        }
                    }
                }

                TaskCondition::All { conditions } => {
                    // Recursive evaluation using Box::pin
                    self.evaluate_mixed_conditions(conditions, context).await?
                }

                TaskCondition::Any { conditions } => {
                    // At least one condition must be true
                    let mut any_true = false;
                    for cond in conditions {
                        if self.evaluate_mixed_conditions(&[cond.clone()], context).await? {
                            any_true = true;
                            break;
                        }
                    }
                    any_true
                }

                TaskCondition::Not { condition } => {
                    // Evaluate and negate using Box::pin
                    let result = self.evaluate_mixed_conditions(&[*condition.clone()], context).await?;
                    !result
                }

                // All other conditions are synchronous
                _ => {
                    self.checker.evaluate_condition(condition, context)?
                }
            };

            if !result {
                return Ok(false);
            }
        }

        Ok(true)
        })
    }
}

/// Async boundary adapter builder
pub struct AsyncConditionAdapterBuilder {
    git_provider: Option<Box<dyn crate::core::GitProvider>>,
    config_provider: Option<Box<dyn crate::core::ConfigProvider>>,
    package_provider: Option<Box<dyn crate::core::PackageProvider>>,
    file_system_provider: Option<Box<dyn crate::core::FileSystemProvider>>,
}

impl AsyncConditionAdapterBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            git_provider: None,
            config_provider: None,
            package_provider: None,
            file_system_provider: None,
        }
    }

    /// Set the git provider
    #[must_use]
    pub fn git_provider(mut self, provider: Box<dyn crate::core::GitProvider>) -> Self {
        self.git_provider = Some(provider);
        self
    }

    /// Set the config provider
    #[must_use]
    pub fn config_provider(mut self, provider: Box<dyn crate::core::ConfigProvider>) -> Self {
        self.config_provider = Some(provider);
        self
    }

    /// Set the package provider
    #[must_use]
    pub fn package_provider(mut self, provider: Box<dyn crate::core::PackageProvider>) -> Self {
        self.package_provider = Some(provider);
        self
    }

    /// Set the file system provider
    #[must_use]
    pub fn file_system_provider(mut self, provider: Box<dyn crate::core::FileSystemProvider>) -> Self {
        self.file_system_provider = Some(provider);
        self
    }

    /// Build the adapter
    ///
    /// # Errors
    ///
    /// Returns an error if any required provider is missing
    pub fn build(self) -> Result<AsyncConditionAdapter> {
        let git_provider = self.git_provider
            .ok_or_else(|| Error::task("Git provider is required".to_string()))?;
        let config_provider = self.config_provider
            .ok_or_else(|| Error::task("Config provider is required".to_string()))?;
        let package_provider = self.package_provider
            .ok_or_else(|| Error::task("Package provider is required".to_string()))?;
        let file_system_provider = self.file_system_provider
            .ok_or_else(|| Error::task("File system provider is required".to_string()))?;

        let checker = ConditionChecker::new(git_provider, config_provider, package_provider, file_system_provider);
        Ok(AsyncConditionAdapter::new(checker))
    }
}

impl Default for AsyncConditionAdapterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::TaskCondition;

    #[test]
    fn test_requires_async_execution() {
        use crate::tasks::{EnvironmentCondition, FilePattern, FilePatternType};

        // Sync conditions
        let sync_conditions = vec![
            TaskCondition::PackagesChanged { 
                packages: vec!["test".to_string()] 
            },
            TaskCondition::FilesChanged { 
                patterns: vec![FilePattern {
                    pattern: "*.rs".to_string(),
                    pattern_type: FilePatternType::Glob,
                    exclude: false,
                }] 
            },
        ];
        
        assert!(!AsyncConditionAdapter::requires_async_execution(&sync_conditions));

        // Async conditions
        let async_conditions = vec![
            TaskCondition::CustomScript { 
                script: "echo test".to_string(),
                expected_output: None 
            },
        ];
        
        assert!(AsyncConditionAdapter::requires_async_execution(&async_conditions));

        // Mixed conditions
        let mut mixed_conditions = sync_conditions;
        mixed_conditions.extend(async_conditions);
        
        assert!(AsyncConditionAdapter::requires_async_execution(&mixed_conditions));
    }

    #[test]
    fn test_requires_async_with_environment_custom() {
        let custom_env_condition = vec![
            TaskCondition::Environment { 
                env: EnvironmentCondition::Custom { 
                    checker: "test-checker".to_string() 
                } 
            },
        ];
        
        assert!(AsyncConditionAdapter::requires_async_execution(&custom_env_condition));
    }
}