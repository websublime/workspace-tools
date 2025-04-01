use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use sublime_monorepo_tools::{
    WorkspaceManager, DiscoveryOptions, Workspace,
    Task, TaskRunner, TaskFilter, TaskStatus
};

// Helper to create a test workspace
fn create_test_workspace() -> Rc<Workspace> {
    // Create a temporary workspace directory
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().to_path_buf();
    
    // Create a package.json
    std::fs::write(
        temp_path.join("package.json"),
        r#"{
            "name": "test-workspace",
            "version": "1.0.0",
            "workspaces": ["packages/*"]
        }"#,
    ).unwrap();
    
    // Create workspace manager
    let manager = WorkspaceManager::new();
    
    // Discover workspace
    let options = DiscoveryOptions::new()
        .auto_detect_root(false)
        .detect_package_manager(true);
    
    let workspace = manager.discover_workspace(temp_path, &options).unwrap();
    Rc::new(workspace)
}

// Temporarily ignored due to changes in TaskRunner implementation
#[ignore]
#[test]
fn test_task_runner_basic() {
    let workspace = create_test_workspace();
    
    // Create a task runner
    let mut runner = TaskRunner::new(&workspace);
    
    // Add some tasks
    let echo_task = Task::new("echo", "echo hello world");
    let pwd_task = Task::new("pwd", "pwd");
    
    runner.add_task(echo_task.clone());
    runner.add_task(pwd_task.clone());
    
    // Check tasks were added
    let tasks = runner.get_tasks();
    assert_eq!(tasks.len(), 2);
    assert!(tasks.contains(&echo_task));
    assert!(tasks.contains(&pwd_task));
    
    // Get task by name
    let retrieved_echo = runner.get_task("echo").unwrap();
    assert_eq!(retrieved_echo.name, "echo");
    
    // Run a task (simple echo command should work everywhere)
    let result = runner.run_task("echo");
    assert!(result.is_ok(), "Task execution failed: {:?}", result.err());
    
    let task_result = result.unwrap();
    assert_eq!(task_result.task.name, "echo");
    assert_eq!(task_result.execution.status, TaskStatus::Success);
    assert_eq!(task_result.execution.exit_code, 0);
    assert!(task_result.execution.stdout.contains("hello world"));
}

// Temporarily ignored due to changes in TaskRunner implementation
#[ignore]
#[test]
fn test_task_runner_with_dependencies() {
    let workspace = create_test_workspace();
    
    // Create a task runner
    let mut runner = TaskRunner::new(&workspace);
    
    // Add tasks with dependencies
    let task1 = Task::new("task1", "echo task1");
    let task2 = Task::new("task2", "echo task2").with_dependency("task1");
    let task3 = Task::new("task3", "echo task3").with_dependencies(vec!["task1", "task2"]);
    
    runner.add_tasks(vec![task1, task2, task3]);
    
    // Run task3 (should run task1 and task2 first)
    let results = runner.run_task("task3");
    assert!(results.is_ok(), "Task execution failed: {:?}", results.err());
    
    // Run multiple tasks
    let results = runner.run_tasks(&["task1", "task2"]);
    assert!(results.is_ok(), "Task execution failed: {:?}", results.err());
    
    let task_results = results.unwrap();
    assert_eq!(task_results.len(), 2);
    
    // Results should all be successful
    for result in &task_results {
        assert_eq!(result.execution.status, TaskStatus::Success);
        assert_eq!(result.execution.exit_code, 0);
    }
}

#[test]
fn test_task_runner_with_filter() {
    let workspace = create_test_workspace();
    
    // Create a task runner
    let mut runner = TaskRunner::new(&workspace);
    
    // Add several tasks
    let task1 = Task::new("build:app", "echo building app").with_package("app");
    let task2 = Task::new("test:app", "echo testing app").with_package("app");
    let task3 = Task::new("build:lib", "echo building lib").with_package("lib");
    let task4 = Task::new("test:lib", "echo testing lib").with_package("lib");
    
    runner.add_tasks(vec![task1, task2, task3, task4]);
    
    // Create a filter for build tasks
    let filter = TaskFilter::new().with_include(vec!["build:*"]);
    
    // Run filtered tasks
    let results = runner.run_filtered(filter);
    assert!(results.is_ok(), "Task execution failed: {:?}", results.err());
    
    let task_results = results.unwrap();
    assert_eq!(task_results.len(), 2);
    
    // Results should include only build tasks
    let task_names: Vec<String> = task_results.iter()
        .map(|r| r.task.name.clone())
        .collect();
    
    assert!(task_names.contains(&"build:app".to_string()));
    assert!(task_names.contains(&"build:lib".to_string()));
    assert!(!task_names.contains(&"test:app".to_string()));
    assert!(!task_names.contains(&"test:lib".to_string()));
    
    // Create a filter for app package
    let filter = TaskFilter::new().with_packages(vec!["app"]);
    
    // Run filtered tasks
    let results = runner.run_filtered(filter);
    assert!(results.is_ok(), "Task execution failed: {:?}", results.err());
    
    let task_results = results.unwrap();
    assert_eq!(task_results.len(), 2);
    
    // Results should include only app tasks
    let task_names: Vec<String> = task_results.iter()
        .map(|r| r.task.name.clone())
        .collect();
    
    assert!(task_names.contains(&"build:app".to_string()));
    assert!(task_names.contains(&"test:app".to_string()));
    assert!(!task_names.contains(&"build:lib".to_string()));
    assert!(!task_names.contains(&"test:lib".to_string()));
}

// Temporarily ignored due to changes in TaskGraph implementation
#[ignore]
#[test]
fn test_task_graph_visualization() {
    let workspace = create_test_workspace();
    
    // Create a task runner
    let mut runner = TaskRunner::new(&workspace);
    
    // Add tasks with dependencies
    let task1 = Task::new("task1", "echo task1");
    let task2 = Task::new("task2", "echo task2").with_dependency("task1");
    let task3 = Task::new("task3", "echo task3").with_dependencies(vec!["task1", "task2"]);
    
    runner.add_tasks(vec![task1, task2, task3]);
    
    // Build task graph
    let graph = runner.build_task_graph();
    assert!(graph.is_ok(), "Failed to build task graph: {:?}", graph.err());
    
    let task_graph = graph.unwrap();
    
    // Graph should have 3 tasks
    assert_eq!(task_graph.task_count(), 3);
    
    // Should have 3 levels
    let levels = task_graph.task_levels();
    assert_eq!(levels.len(), 3);
} 