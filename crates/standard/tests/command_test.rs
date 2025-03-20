#[cfg(test)]
mod commands_tests {
    use sublime_standard_tools::{execute, CommandError};

    #[test]
    fn test_git_command() -> Result<(), CommandError> {
        let result = execute("git", ".", ["--version"], |_message, output| {
            Ok(output.status.code().unwrap())
        })?;

        assert!(result == 0);
        Ok(())
    }

    #[test]
    fn test_git_version() -> Result<(), CommandError> {
        let result =
            execute("git", ".", ["--version"], |message, _output| Ok(message.to_string()))?;

        assert!(result.contains("git version"));
        Ok(())
    }
}
