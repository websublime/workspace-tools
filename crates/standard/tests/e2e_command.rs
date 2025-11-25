//! # End-to-End Tests for Command Module
//!
//! ## What
//! Comprehensive e2e tests for the command execution framework including
//! `CommandBuilder`, `DefaultCommandExecutor`.
//!
//! ## How
//! Tests execute real commands (echo, etc.) and verify output, exit codes,
//! and error handling. Uses cross-platform commands where possible.
//!
//! ## Why
//! E2E tests ensure the command module works correctly with real processes,
//! validating integration between all components in realistic scenarios.

#![allow(clippy::print_stdout)]

use std::time::Duration;

use sublime_standard_tools::{
    command::{CommandBuilder, DefaultCommandExecutor, Executor},
    config::CommandConfig,
    error::Result,
};
use tempfile::TempDir;

// ============================================================================
// CommandBuilder Tests
// ============================================================================

#[tokio::test]
async fn test_command_builder_basic() -> Result<()> {
    let command = CommandBuilder::new("echo").arg("hello").arg("world").build();

    // Verify command was built successfully by executing it
    let executor = DefaultCommandExecutor::new();
    let output = executor.execute(command).await?;
    assert!(output.success());
    assert!(output.stdout().contains("hello"));
    Ok(())
}

#[tokio::test]
async fn test_command_builder_with_current_dir() -> Result<()> {
    let temp_dir = TempDir::new().map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("Failed to create temp dir: {e}"))
    })?;

    let executor = DefaultCommandExecutor::new();

    // Cross-platform pwd/cd command
    let command = if cfg!(target_os = "windows") {
        CommandBuilder::new("cmd").arg("/C").arg("cd").current_dir(temp_dir.path()).build()
    } else {
        CommandBuilder::new("pwd").current_dir(temp_dir.path()).build()
    };

    let output = executor.execute(command).await?;
    assert!(output.success());
    Ok(())
}

#[tokio::test]
async fn test_command_builder_with_timeout() -> Result<()> {
    let command = CommandBuilder::new("echo").arg("quick").timeout(Duration::from_secs(5)).build();

    let executor = DefaultCommandExecutor::new();
    let output = executor.execute(command).await?;
    assert!(output.success());
    Ok(())
}

#[tokio::test]
async fn test_command_builder_with_env() -> Result<()> {
    let command = CommandBuilder::new("echo").env("MY_VAR", "my_value").arg("test").build();

    let executor = DefaultCommandExecutor::new();
    let output = executor.execute(command).await?;
    assert!(output.success());
    Ok(())
}

#[tokio::test]
async fn test_command_builder_chain() -> Result<()> {
    let temp_dir = TempDir::new().map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("Failed to create temp dir: {e}"))
    })?;

    let command = CommandBuilder::new("echo")
        .arg("test")
        .arg("one")
        .arg("two")
        .current_dir(temp_dir.path())
        .timeout(Duration::from_secs(30))
        .env("KEY", "VALUE")
        .build();

    let executor = DefaultCommandExecutor::new();
    let output = executor.execute(command).await?;
    assert!(output.success());
    assert!(output.stdout().contains("test"));
    Ok(())
}

// ============================================================================
// DefaultCommandExecutor Tests
// ============================================================================

#[tokio::test]
async fn test_executor_echo_command() -> Result<()> {
    let executor = DefaultCommandExecutor::new();
    let command = CommandBuilder::new("echo").arg("hello world").build();

    let output = executor.execute(command).await?;

    assert!(output.success());
    assert!(output.stdout().contains("hello world"));
    Ok(())
}

#[tokio::test]
async fn test_executor_with_env_variable() -> Result<()> {
    let executor = DefaultCommandExecutor::new();

    // Cross-platform env variable echo
    let command = if cfg!(target_os = "windows") {
        CommandBuilder::new("cmd")
            .arg("/C")
            .arg("echo %TEST_VAR%")
            .env("TEST_VAR", "test_value")
            .build()
    } else {
        CommandBuilder::new("sh")
            .arg("-c")
            .arg("echo $TEST_VAR")
            .env("TEST_VAR", "test_value")
            .build()
    };

    let output = executor.execute(command).await?;

    assert!(output.success());
    assert!(output.stdout().contains("test_value"));
    Ok(())
}

#[tokio::test]
async fn test_executor_working_directory() -> Result<()> {
    let temp_dir = TempDir::new().map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("Failed to create temp dir: {e}"))
    })?;

    let executor = DefaultCommandExecutor::new();

    // Cross-platform pwd
    let command = if cfg!(target_os = "windows") {
        CommandBuilder::new("cmd").arg("/C").arg("cd").current_dir(temp_dir.path()).build()
    } else {
        CommandBuilder::new("pwd").current_dir(temp_dir.path()).build()
    };

    let output = executor.execute(command).await?;

    assert!(output.success());
    let stdout = output.stdout();
    assert!(!stdout.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_executor_exit_code_non_zero() -> Result<()> {
    let executor = DefaultCommandExecutor::new();

    // Cross-platform command that exits with code 1
    let command = if cfg!(target_os = "windows") {
        CommandBuilder::new("cmd").arg("/C").arg("exit 1").build()
    } else {
        CommandBuilder::new("sh").arg("-c").arg("exit 1").build()
    };

    // The executor returns an error for non-zero exit codes
    let result = executor.execute(command).await;
    assert!(result.is_err());

    // Verify it's a NonZeroExitCode error
    let err_str = format!("{:?}", result.unwrap_err());
    assert!(err_str.contains("NonZeroExitCode") || err_str.contains("exit code"));

    Ok(())
}

#[tokio::test]
async fn test_executor_stderr_capture() -> Result<()> {
    let executor = DefaultCommandExecutor::new();

    // Cross-platform stderr output
    let command = if cfg!(target_os = "windows") {
        CommandBuilder::new("cmd").arg("/C").arg("echo error message 1>&2").build()
    } else {
        CommandBuilder::new("sh").arg("-c").arg("echo 'error message' >&2").build()
    };

    let output = executor.execute(command).await?;

    // stderr should contain the error message
    assert!(output.stderr().contains("error message"));
    Ok(())
}

#[tokio::test]
async fn test_executor_command_not_found() -> Result<()> {
    let executor = DefaultCommandExecutor::new();
    let command = CommandBuilder::new("nonexistent_command_xyz123").build();

    let result = executor.execute(command).await;

    // Should return an error for non-existent command
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_executor_with_timeout_success() -> Result<()> {
    let executor = DefaultCommandExecutor::new();

    // Command that completes quickly within timeout
    let command = CommandBuilder::new("echo").arg("quick").timeout(Duration::from_secs(5)).build();

    let output = executor.execute(command).await?;

    assert!(output.success());
    assert!(output.stdout().contains("quick"));
    Ok(())
}

#[tokio::test]
async fn test_executor_with_config() -> Result<()> {
    let config = CommandConfig::default();

    let executor = DefaultCommandExecutor::new_with_config(config);
    let command = CommandBuilder::new("echo").arg("configured").build();

    let output = executor.execute(command).await?;

    assert!(output.success());
    assert!(output.stdout().contains("configured"));
    Ok(())
}

// ============================================================================
// CommandOutput Tests
// ============================================================================

#[tokio::test]
async fn test_command_output_success_methods() -> Result<()> {
    let executor = DefaultCommandExecutor::new();
    let command = CommandBuilder::new("echo").arg("test output").build();

    let output = executor.execute(command).await?;

    assert!(output.success());
    assert!(!output.stdout().is_empty());
    assert!(output.stderr().is_empty() || output.stderr().trim().is_empty());
    Ok(())
}

#[tokio::test]
async fn test_command_output_failure_methods() -> Result<()> {
    let executor = DefaultCommandExecutor::new();

    let command = if cfg!(target_os = "windows") {
        CommandBuilder::new("cmd").arg("/C").arg("exit 42").build()
    } else {
        CommandBuilder::new("sh").arg("-c").arg("exit 42").build()
    };

    // The executor returns an error for non-zero exit codes
    let result = executor.execute(command).await;
    assert!(result.is_err());

    // Verify it's a NonZeroExitCode error with the expected code
    let err_str = format!("{:?}", result.unwrap_err());
    assert!(err_str.contains("NonZeroExitCode") || err_str.contains("42"));

    Ok(())
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_executor_file_operations() -> Result<()> {
    let temp_dir = TempDir::new().map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("Failed to create temp dir: {e}"))
    })?;

    let executor = DefaultCommandExecutor::new();
    let test_file = temp_dir.path().join("test.txt");

    // Create file with content
    let create_command = if cfg!(target_os = "windows") {
        CommandBuilder::new("cmd")
            .arg("/C")
            .arg(format!("echo test content > {}", test_file.display()))
            .current_dir(temp_dir.path())
            .build()
    } else {
        CommandBuilder::new("sh")
            .arg("-c")
            .arg(format!("echo 'test content' > {}", test_file.display()))
            .current_dir(temp_dir.path())
            .build()
    };

    let output = executor.execute(create_command).await?;
    assert!(output.success());

    // Read file content
    let read_command = if cfg!(target_os = "windows") {
        CommandBuilder::new("cmd").arg("/C").arg(format!("type {}", test_file.display())).build()
    } else {
        CommandBuilder::new("cat").arg(test_file.to_str().unwrap_or_default()).build()
    };

    let output = executor.execute(read_command).await?;
    assert!(output.success());
    assert!(output.stdout().contains("test content"));

    Ok(())
}

#[tokio::test]
async fn test_command_with_large_output() -> Result<()> {
    let executor = DefaultCommandExecutor::new();

    // Generate large output
    let command = if cfg!(target_os = "windows") {
        CommandBuilder::new("cmd").arg("/C").arg("for /L %i in (1,1,100) do @echo Line %i").build()
    } else {
        CommandBuilder::new("sh")
            .arg("-c")
            .arg("for i in $(seq 1 100); do echo \"Line $i\"; done")
            .build()
    };

    let output = executor.execute(command).await?;

    assert!(output.success());
    // Should have captured all 100 lines
    let lines: Vec<&str> = output.stdout().lines().collect();
    assert!(lines.len() >= 100);

    Ok(())
}

#[tokio::test]
async fn test_command_with_stdin_simulation() -> Result<()> {
    let executor = DefaultCommandExecutor::new();

    // Use echo to pipe content (simulating stdin-like behavior)
    let command = if cfg!(target_os = "windows") {
        CommandBuilder::new("cmd").arg("/C").arg("echo input data").build()
    } else {
        CommandBuilder::new("sh").arg("-c").arg("echo 'input data' | cat").build()
    };

    let output = executor.execute(command).await?;

    assert!(output.success());
    assert!(output.stdout().contains("input data"));

    Ok(())
}

#[tokio::test]
async fn test_multiple_sequential_commands() -> Result<()> {
    let executor = DefaultCommandExecutor::new();

    // Execute multiple commands sequentially
    for i in 0..5 {
        let command = CommandBuilder::new("echo").arg(format!("command-{i}")).build();

        let output = executor.execute(command).await?;
        assert!(output.success());
        assert!(output.stdout().contains(&format!("command-{i}")));
    }

    Ok(())
}
