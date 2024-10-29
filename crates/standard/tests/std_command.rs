#[cfg(test)]
mod commands_tests {
    #[test]
    fn test_git_command() -> Result<(), Box<dyn std::error::Error>> {
        use ws_std::command::execute;

        let result = execute("git", ".", ["--version"], |_message, output| {
            Ok(output.status.code().unwrap())
        })?;

        assert!(result == 0);
        Ok(())
    }
}
