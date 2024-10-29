#[cfg(test)]
mod commands_tests {
    use ws_std::command::execute;

    #[test]
    fn test_git_command() -> Result<(), Box<dyn std::error::Error>> {
        let result = execute("git", ".", ["--version"], |_message, output| {
            Ok(output.status.code().unwrap())
        })?;

        assert!(result == 0);
        Ok(())
    }

    #[test]
    fn test_git_version() -> Result<(), Box<dyn std::error::Error>> {
        let result =
            execute("git", ".", ["--version"], |message, _output| Ok(message.to_string()))?;

        assert!(result.contains("git version"));
        Ok(())
    }
}
