use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use sublime_monorepo_tools::{
    Task, TaskConfig, TaskStatus, TaskExecution, TaskFilter
};

#[test]
fn test_task_creation() {
    // Test basic task creation
    let task = Task::new("test", "echo hello");
    assert_eq!(task.name, "test");
    assert_eq!(task.command, "echo hello");
    assert!(task.package.is_none());
    assert!(task.dependencies.is_empty());
    
    // Test task with package
    let task = Task::new("test", "echo hello").with_package("my-package");
    assert_eq!(task.name, "test");
    assert_eq!(task.command, "echo hello");
    assert_eq!(task.package, Some("my-package".to_string()));
    
    // Test task with dependency
    let task = Task::new("test", "echo hello").with_dependency("dependency");
    assert_eq!(task.name, "test");
    assert_eq!(task.command, "echo hello");
    assert_eq!(task.dependencies, vec!["dependency"]);
    
    // Test task with multiple dependencies
    let task = Task::new("test", "echo hello")
        .with_dependencies(vec!["dep1", "dep2"]);
    assert_eq!(task.name, "test");
    assert_eq!(task.command, "echo hello");
    assert_eq!(task.dependencies, vec!["dep1", "dep2"]);
    
    // Test task with all options
    let task = Task::new("test", "echo hello")
        .with_package("my-package")
        .with_dependency("dependency")
        .with_cwd(PathBuf::from("/tmp"))
        .with_env("ENV_VAR", "value")
        .with_timeout(Duration::from_secs(10))
        .ignore_error(true)
        .live_output(false);
        
    assert_eq!(task.name, "test");
    assert_eq!(task.command, "echo hello");
    assert_eq!(task.package, Some("my-package".to_string()));
    assert_eq!(task.dependencies, vec!["dependency"]);
    assert_eq!(task.config.cwd, Some(PathBuf::from("/tmp")));
    assert_eq!(task.config.env.get("ENV_VAR"), Some(&"value".to_string()));
    assert_eq!(task.config.timeout, Some(Duration::from_secs(10)));
    assert_eq!(task.config.ignore_error, true);
    assert_eq!(task.config.live_output, false);
}

#[test]
fn test_task_config_default() {
    let config = TaskConfig::default();
    assert_eq!(config.cwd, None);
    assert!(config.env.is_empty());
    assert_eq!(config.timeout, None);
    assert_eq!(config.ignore_error, false);
    assert_eq!(config.live_output, true);
}

#[test]
fn test_task_execution() {
    // Test successful execution
    let execution = TaskExecution {
        exit_code: 0,
        stdout: "output".to_string(),
        stderr: "".to_string(),
        duration: Duration::from_secs(1),
        status: TaskStatus::Success,
    };
    
    assert_eq!(execution.exit_code, 0);
    assert_eq!(execution.stdout, "output");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.duration, Duration::from_secs(1));
    assert_eq!(execution.status, TaskStatus::Success);
    
    // Test failed execution
    let execution = TaskExecution {
        exit_code: 1,
        stdout: "".to_string(),
        stderr: "error".to_string(),
        duration: Duration::from_secs(1),
        status: TaskStatus::Failed,
    };
    
    assert_eq!(execution.exit_code, 1);
    assert_eq!(execution.stdout, "");
    assert_eq!(execution.stderr, "error");
    assert_eq!(execution.duration, Duration::from_secs(1));
    assert_eq!(execution.status, TaskStatus::Failed);
}

#[test]
fn test_task_filter() {
    // Create a filter
    let filter = TaskFilter::new()
        .with_include(vec!["test*"])
        .with_exclude(vec!["*ignore"])
        .with_packages(vec!["pkg1", "pkg2"])
        .include_dependencies(true)
        .include_dependents(false);
        
    assert_eq!(filter.include, vec!["test*"]);
    assert_eq!(filter.exclude, vec!["*ignore"]);
    assert_eq!(filter.packages, vec!["pkg1", "pkg2"]);
    assert_eq!(filter.include_dependencies, true);
    assert_eq!(filter.include_dependents, false);
    
    // Test default filter
    let default_filter = TaskFilter::default();
    assert!(default_filter.include.is_empty());
    assert!(default_filter.exclude.is_empty());
    assert!(default_filter.packages.is_empty());
    assert_eq!(default_filter.include_dependencies, true);
    assert_eq!(default_filter.include_dependents, false);
}

#[test]
fn test_task_filter_apply() {
    // Create some tasks
    let task1 = Task::new("test1", "echo 1").with_package("pkg1");
    let task2 = Task::new("test2", "echo 2").with_package("pkg2");
    let task3 = Task::new("build", "echo 3").with_package("pkg1");
    let task4 = Task::new("test-ignore", "echo 4").with_package("pkg3");
    let task5 = Task::new("dep", "echo 5");
    
    let task_with_dep = Task::new("with-dep", "echo 6").with_dependency("dep");
    
    let tasks = vec![
        task1.clone(),
        task2.clone(),
        task3.clone(),
        task4.clone(),
        task5.clone(),
        task_with_dep.clone(),
    ];
    
    // Filter by name pattern
    let filter = TaskFilter::new().with_include(vec!["test*"]);
    let filtered = filter.apply(&tasks).unwrap();
    assert_eq!(filtered.len(), 3);
    assert!(filtered.contains(&task1));
    assert!(filtered.contains(&task2));
    assert!(filtered.contains(&task4));
    
    // Filter with exclusion
    let filter = TaskFilter::new()
        .with_include(vec!["test*"])
        .with_exclude(vec!["*ignore"]);
    let filtered = filter.apply(&tasks).unwrap();
    assert_eq!(filtered.len(), 2);
    assert!(filtered.contains(&task1));
    assert!(filtered.contains(&task2));
    
    // Filter by package
    let filter = TaskFilter::new().with_packages(vec!["pkg1"]);
    let filtered = filter.apply(&tasks).unwrap();
    assert_eq!(filtered.len(), 2);
    assert!(filtered.contains(&task1));
    assert!(filtered.contains(&task3));
    
    // Filter with dependencies
    let filter = TaskFilter::new()
        .with_include(vec!["with-dep"])
        .include_dependencies(true);
    let filtered = filter.apply(&tasks).unwrap();
    assert_eq!(filtered.len(), 2);
    assert!(filtered.contains(&task_with_dep));
    assert!(filtered.contains(&task5)); // dep is included
    
    // Filter without dependencies
    let filter = TaskFilter::new()
        .with_include(vec!["with-dep"])
        .include_dependencies(false);
    let filtered = filter.apply(&tasks).unwrap();
    assert_eq!(filtered.len(), 1);
    assert!(filtered.contains(&task_with_dep));
    
    // Filter with dependents
    let filter = TaskFilter::new()
        .with_include(vec!["dep"])
        .include_dependents(true);
    let filtered = filter.apply(&tasks).unwrap();
    assert_eq!(filtered.len(), 2);
    assert!(filtered.contains(&task5)); // dep is included
    assert!(filtered.contains(&task_with_dep)); // dependent is included
} 